//! ILDA player: the verbs of the `spell ilda play` function as registry commands.
//!
//! It lives in the CLI, and not in the engine, for the same reason as `play_show` and `net`: the
//! engine does not know the `laser` crate. Like every product command, it is a `Registry::add`,
//! so the CLI (`spellcore commands`), MCP, OSC and GUI see it without one extra line.
//!
//! One feed = one open DAC. The `FEEDS` table is to the laser what `player::current()` is to the
//! transport: one process, N feeds, each with the DAC thread (of the `Feed` itself) and, while it
//! plays, a thread that reads the `.ild` and pushes frames at the rate asked for.
//!
//! `laser_param path -> field` (the same paths as `modules/laser.json`, the module declaration):
//!
//! | path | field | range |
//! |---|---|---|
//! | `geo/x`, `geo/y` | `Transform.x`, `Transform.y` | ILDA units, +-32767 |
//! | `geo/scale` | `Transform.scale` | 0..4 |
//! | `geo/rot` | `Transform.rot` | degrees, +-180 |
//! | `limit/r`, `limit/g`, `limit/b` | `Transform.color.0/.1/.2` | 0..1 |
//! | `safe/min_size` | `Safety.min_size` | ILDA units, 0..32767 |
//! | `safe/max_intensity` | `Safety.max_intensity` | 0..255 |
//! | `shutter` | zeroes `Safety.max_intensity` and gives the stored value back on open | 0 or 1 |
//!
// ponytail: only the fields `Transform` and `Safety` already have ; `curve/r|g|b`, `Blanking/*`
// and `Cor/Time Shift` of the ilda-player table come in when the `Feed` has a color LUT and the
// `optimize` is parameterizable at runtime.

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use engine::registry::Registry;
use engine::schemars::JsonSchema;
use laser::dac::idn;
use laser::{ild, Dac, EtherDream, Feed, Idn, Safety, Transform};
use protocols::netscan;
use serde::Deserialize;
use serde_json::{json, Value};

/// Ether Dream buffer in points. The beacon carries the real value (`buffer_capacity`), but
/// `laser_open` accepts a typed host, with no beacon.
// ponytail: capacity fixed at 1800 (the hardware default) ; read it from the beacon when
// `laser_open` accepts the id returned by `laser_dacs` instead of the host.
const CAPACITY: u16 = 1800;

// -------------------------------------------------------------------- feed table

struct Play {
    file: String,
    /// The thread clears it on exit, including when the file ends with no loop; `colher` reaps.
    run: Arc<AtomicBool>,
    th: Option<JoinHandle<()>>,
}

struct Live {
    id: u64,
    name: String,
    feed: Arc<Feed>,
    tf: Transform,
    safety: Safety,
    /// `max_intensity` stored while the shutter is closed; `None` = an open shutter.
    shut: Option<u8>,
    play: Option<Play>,
}

static FEEDS: Mutex<Vec<Live>> = Mutex::new(Vec::new());
static NEXT: AtomicU64 = AtomicU64::new(1);

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn achar(v: &mut [Live], id: u64) -> Result<&mut Live, String> {
    v.iter_mut()
        .find(|f| f.id == id)
        .ok_or_else(|| format!("feed {} does not exist", id))
}

/// Stops the playback thread and waits for it to leave. Idempotent.
fn parar(l: &mut Live) {
    if let Some(mut p) = l.play.take() {
        p.run.store(false, Ordering::Relaxed);
        if let Some(h) = p.th.take() {
            let _ = h.join();
        }
    }
}

/// Reaps the thread that finished on its own (end of the file with no loop): `play` only exists
/// while it plays, so `playing` never gets stuck at `true`.
fn colher(l: &mut Live) {
    if l.play
        .as_ref()
        .is_some_and(|p| !p.run.load(Ordering::Relaxed))
    {
        parar(l);
    }
}

// ------------------------------------------------------------------ laser_dacs

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "engine::schemars")]
struct DacsArgs {
    /// Seconds of listening per protocol.
    #[serde(default = "def_timeout")]
    timeout: f64,
}

fn def_timeout() -> f64 {
    2.0
}

fn dacs(a: DacsArgs) -> Result<Value, String> {
    let t = Duration::from_secs_f64(a.timeout.clamp(0.1, 30.0));
    let mut out: Vec<Value> = netscan::scan_etherdream(t, &netscan::interfaces())
        .iter()
        .map(|d| {
            json!({"type": "etherdream", "id": d.mac, "host": d.ip,
                   "buffer": d.buffer_capacity, "max_pps": d.max_point_rate, "via": d.via})
        })
        .collect();
    // ponytail: no Helios in the list ; the USB DAC comes in when the driver (hidapi/rusb) does —
    // `laser::dac::helios` today only has the frame encoder, documented for the port.
    out.extend(idn::scan(std::net::Ipv4Addr::BROADCAST, t).iter().map(|u| {
        json!({"type": "idn", "id": u.unit_id.iter().map(|b| format!("{b:02x}")).collect::<String>(),
               "host": u.ip, "name": u.name})
    }));
    Ok(Value::Array(out))
}

// ------------------------------------------------------------------ laser_open

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "engine::schemars")]
struct OpenArgs {
    /// etherdream or idn.
    dac: String,
    /// "ip" or "ip:port" (the `host` of `laser_dacs`).
    #[serde(default)]
    host: String,
    /// Thousands of points per second delivered to the DAC.
    #[serde(default = "def_kpps")]
    kpps: f64,
    /// Safety of the feed: {"min_size": 2000, "max_intensity": 255, "zone": [x0,y0,x1,y1]} in
    /// ILDA units. Absent = the default. Never switchable off.
    #[serde(default)]
    safety: Option<Value>,
}

fn def_kpps() -> f64 {
    30.0
}

fn abrir(a: OpenArgs) -> Result<Value, String> {
    let pps = (a.kpps * 1000.0).clamp(1000.0, 200_000.0) as u32;
    let safety: Safety = match a.safety {
        Some(v) => serde_json::from_value(v).map_err(|e| format!("safety: {}", e))?,
        None => Safety::default(),
    };
    if a.host.is_empty() {
        return Err(format!(
            "laser_open {} requires a host (see laser_dacs)",
            a.dac
        ));
    }
    let d: Box<dyn Dac> = match a.dac.as_str() {
        "etherdream" => Box::new(
            EtherDream::connect(&a.host, CAPACITY)
                .map_err(|e| format!("etherdream {}: {}", a.host, e))?,
        ),
        "idn" => Box::new(Idn::connect(&a.host, 0).map_err(|e| format!("idn {}: {}", a.host, e))?),
        o => return Err(format!("unknown dac: {} (etherdream, idn)", o)),
    };
    let feed = Feed::start(d, pps, 2, safety).map_err(|e| e.to_string())?;
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let name = feed.name().to_string();
    lock(&FEEDS).push(Live {
        id,
        name: name.clone(),
        feed: Arc::new(feed),
        tf: Transform::default(),
        safety,
        shut: None,
        play: None,
    });
    Ok(json!({"feed": id, "dac": name, "pps": pps}))
}

// ------------------------------------------------------------------ laser_play

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "engine::schemars")]
struct LaserPlayArgs {
    /// Id returned by `laser_open`.
    feed: u64,
    /// Path of the .ild.
    file: String,
    /// Frames per second pushed to the DAC (the .ild carries no rate).
    #[serde(default = "def_fps")]
    fps: f64,
    /// Restarts from the first frame when it reaches the end.
    #[serde(default, rename = "loop")]
    #[schemars(rename = "loop")]
    looping: bool,
}

fn def_fps() -> f64 {
    30.0
}

fn tocar(a: LaserPlayArgs) -> Result<Value, String> {
    let frames = ild::read(Path::new(&a.file))?;
    if frames.is_empty() {
        return Err(format!("{}: no frame", a.file));
    }
    let n = frames.len();
    let dt = Duration::from_secs_f64(1.0 / a.fps.clamp(1.0, 240.0));
    let mut v = lock(&FEEDS);
    let l = achar(&mut v, a.feed)?;
    parar(l);
    let run = Arc::new(AtomicBool::new(true));
    let (feed, r, looping) = (l.feed.clone(), run.clone(), a.looping);
    // ponytail: the frames go whole into memory before playing ; the biggest .ild in the repo is
    // 1.2 MB. It becomes a per-frame read when someone brings in a whole-show .ild.
    let th = std::thread::spawn(move || {
        let mut i = 0usize;
        let mut next = Instant::now();
        while r.load(Ordering::Relaxed) {
            feed.push(&frames[i].points);
            i += 1;
            if i >= frames.len() {
                if !looping {
                    break;
                }
                i = 0;
            }
            next += dt;
            let now = Instant::now();
            if next > now {
                std::thread::sleep(next - now);
            } else {
                next = now;
            }
        }
        r.store(false, Ordering::Relaxed); // end of the file: `colher` disarms the `play`
    });
    l.play = Some(Play {
        file: a.file.clone(),
        run,
        th: Some(th),
    });
    Ok(
        json!({"feed": a.feed, "file": a.file, "frames": n, "fps": 1.0 / dt.as_secs_f64(),
              "loop": a.looping}),
    )
}

// ---------------------------------------------------- laser_stop / laser_close

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "engine::schemars")]
struct FeedArgs {
    /// Id returned by `laser_open`.
    feed: u64,
}

fn parar_cmd(a: FeedArgs) -> Result<Value, String> {
    let mut v = lock(&FEEDS);
    let l = achar(&mut v, a.feed)?;
    parar(l);
    Ok(json!({"feed": a.feed, "playing": false}))
}

fn fechar(a: FeedArgs) -> Result<Value, String> {
    let mut v = lock(&FEEDS);
    let i = v
        .iter()
        .position(|f| f.id == a.feed)
        .ok_or_else(|| format!("feed {} does not exist", a.feed))?;
    let mut l = v.remove(i);
    parar(&mut l);
    drop(v); // the `Feed` stops the DAC thread on Drop; do not hold the table while it joins
    Ok(json!({"feed": a.feed, "dac": l.name, "closed": true}))
}

// ----------------------------------------------------------------- laser_param

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "engine::schemars")]
struct ParamArgs {
    /// Id returned by `laser_open`.
    feed: u64,
    /// geo/x, geo/y, geo/scale, geo/rot, limit/r, limit/g, limit/b, safe/min_size,
    /// safe/max_intensity, shutter.
    path: String,
    /// Value in the range of the `path` (the range table is in spellcore/README.md).
    value: f64,
}

const PATHS: &str =
    "geo/x geo/y geo/scale geo/rot limit/r limit/g limit/b safe/min_size safe/max_intensity shutter";

fn aplicar(l: &mut Live, path: &str, v: f64) -> Result<(), String> {
    match path {
        "geo/x" => l.tf.x = v.clamp(-32767.0, 32767.0),
        "geo/y" => l.tf.y = v.clamp(-32767.0, 32767.0),
        "geo/scale" => l.tf.scale = v.clamp(0.0, 4.0),
        "geo/rot" => l.tf.rot = v.clamp(-180.0, 180.0),
        "limit/r" => l.tf.color.0 = v.clamp(0.0, 1.0),
        "limit/g" => l.tf.color.1 = v.clamp(0.0, 1.0),
        "limit/b" => l.tf.color.2 = v.clamp(0.0, 1.0),
        "safe/min_size" => l.safety.min_size = v.clamp(0.0, 32767.0) as i32,
        // with the shutter closed the value asked for is stored and applies when it opens
        "safe/max_intensity" => {
            let m = v.clamp(0.0, 255.0) as u8;
            match l.shut {
                Some(_) => l.shut = Some(m),
                None => l.safety.max_intensity = m,
            }
        }
        "shutter" => {
            if v != 0.0 {
                if l.shut.is_none() {
                    l.shut = Some(l.safety.max_intensity);
                    l.safety.max_intensity = 0;
                }
            } else if let Some(m) = l.shut.take() {
                l.safety.max_intensity = m;
            }
        }
        p => return Err(format!("unknown path: {} ({})", p, PATHS)),
    }
    l.feed.set_transform(l.tf);
    l.feed.set_safety(l.safety);
    Ok(())
}

fn param(a: ParamArgs) -> Result<Value, String> {
    let mut v = lock(&FEEDS);
    let l = achar(&mut v, a.feed)?;
    aplicar(l, &a.path, a.value)?;
    Ok(json!({"feed": a.feed, "path": a.path, "value": a.value, "shutter": l.shut.is_some()}))
}

// ----------------------------------------------------------------- laser_stats

/// The `stat/*` are the `values` of `modules/laser.json`; only the ones `FeedStats` has go out
/// (`stat/fps`, `stat/points` and `stat/clipped` come in when the `Feed` counts them).
fn stats(a: FeedArgs) -> Result<Value, String> {
    let mut v = lock(&FEEDS);
    let l = achar(&mut v, a.feed)?;
    colher(l);
    let s = l.feed.stats();
    let file = match &l.play {
        Some(p) => Value::String(p.file.clone()),
        None => Value::Null,
    };
    Ok(
        json!({"feed": l.id, "dac": l.name, "playing": l.play.is_some(), "file": file,
              "stat/sent": s.sent, "stat/dropped": s.dropped, "stat/errors": s.errors,
              "jitter_p50_ms": s.p50 * 1e3, "jitter_p99_ms": s.p99 * 1e3, "cpu": s.cpu,
              "shutter": l.shut.is_some(),
              "safe/min_size": l.safety.min_size,
              "safe/max_intensity": l.safety.max_intensity}),
    )
}

// ----------------------------------------------------------------- laser_files

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "engine::schemars")]
struct FilesArgs {
    /// Directory to list; empty = `shows/`.
    #[serde(default)]
    dir: String,
}

fn files(a: FilesArgs) -> Result<Value, String> {
    let dir = if a.dir.is_empty() {
        "shows"
    } else {
        a.dir.as_str()
    };
    let mut out: Vec<Value> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {}", dir, e))?
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("ild"))
        })
        .map(|e| {
            json!({"name": e.file_name().to_string_lossy(),
                   "path": e.path().to_string_lossy(),
                   "bytes": e.metadata().map(|m| m.len()).unwrap_or(0)})
        })
        .collect();
    out.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Ok(json!({"dir": dir, "files": out}))
}

// ----------------------------------------------------------------- clip_frame

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "engine::schemars")]
struct ClipFrameArgs {
    /// Path of the .ild, or the name the track uses (it resolves in the open show folder).
    clip: String,
    /// Time in seconds; with `fps` it picks the frame. Ignored when `index` comes in.
    #[serde(default)]
    t: f64,
    /// The frame asked for directly; without it, the frame of `t`.
    #[serde(default)]
    index: Option<usize>,
    /// Frames per second of the track (the .ild carries no rate).
    #[serde(default = "def_fps")]
    fps: f64,
}

/// The name the track uses against the folder of the open .spell, same as the Python
/// `_load_clip`. A path that already exists goes straight through.
fn caminho(clip: &str) -> std::path::PathBuf {
    let p = Path::new(clip);
    if p.exists() {
        return p.to_path_buf();
    }
    let base = engine::registry::open_path();
    match Path::new(&base).parent() {
        Some(d) if !base.is_empty() => d.join(clip),
        _ => p.to_path_buf(),
    }
}

/// The last .ild read. The previz asks for one frame at a time and the same file plays the whole
/// show.
// ponytail: a one-file cache, with no look at mtime ; make it a map with mtime when two clips play
// together or when the previz needs to see the .ild swapped on disk without reopening the show.
static CLIP: Mutex<Option<(std::path::PathBuf, Arc<Vec<laser::Frame>>)>> = Mutex::new(None);

fn quadros(p: &Path) -> Result<Arc<Vec<laser::Frame>>, String> {
    let mut g = lock(&CLIP);
    if let Some((q, fs)) = g.as_ref() {
        if q == p {
            return Ok(fs.clone());
        }
    }
    let fs = Arc::new(ild::read(p)?);
    *g = Some((p.to_path_buf(), fs.clone()));
    Ok(fs)
}

fn clip_frame(a: ClipFrameArgs) -> Result<Value, String> {
    let p = caminho(&a.clip);
    let fs = quadros(&p)?;
    if fs.is_empty() {
        return Err(format!("{}: no frame", p.display()));
    }
    // The same math as the player (`spellcaster/player/player.py::laser_frame`): the clip repeats.
    let i = match a.index {
        Some(i) => i % fs.len(),
        None => ((a.t * a.fps.clamp(1.0, 240.0)).floor().max(0.0) as usize) % fs.len(),
    };
    let f = &fs[i];
    let n = |v: i16| v as f64 / laser::frame::LIM as f64;
    let pts: Vec<Value> = f
        .points
        .iter()
        .map(|q| json!([n(q.x), n(q.y), q.r, q.g, q.b, u8::from(q.blank)]))
        .collect();
    Ok(
        json!({"clip": p.to_string_lossy(), "index": i, "frames": fs.len(),
              "name": f.name, "points": pts}),
    )
}

// -------------------------------------------------------------------- registry

pub fn register(r: &mut Registry) {
    r.add::<DacsArgs>(
        "laser_dacs",
        "Looks for laser DACs: Ether Dream by beacon and IDN by scan.",
        dacs,
    );
    r.add::<OpenArgs>(
        "laser_open",
        "Opens a laser DAC and returns the feed id. The safety is mandatory and never switches off.",
        abrir,
    );
    r.add::<LaserPlayArgs>(
        "laser_play",
        "Plays a .ild on the feed: it reads the frames and pushes them at fps (the .ild carries no rate).",
        tocar,
    );
    r.add::<FeedArgs>(
        "laser_stop",
        "Stops the playback of the feed; the DAC stays open.",
        parar_cmd,
    );
    r.add::<FeedArgs>(
        "laser_close",
        "Stops and closes the feed (it shuts the DAC down).",
        fechar,
    );
    r.add::<ParamArgs>(
        "laser_param",
        "Sets a parameter of the feed: geo/x geo/y geo/scale geo/rot limit/r limit/g limit/b safe/min_size safe/max_intensity shutter.",
        param,
    );
    r.add::<FeedArgs>(
        "laser_stats",
        "State of the feed: playing, file, stat/sent, stat/dropped, stat/errors, jitter and cpu.",
        stats,
    );
    r.add::<FilesArgs>(
        "laser_files",
        "Lists the .ild of a directory (empty = shows/).",
        files,
    );
    r.add::<ClipFrameArgs>(
        "clip_frame",
        "One frame of the .ild to draw: it picks by `index` or by `t` at `fps`, and returns the points [x, y, r, g, b, blank] with x and y normalized to -1..1.",
        clip_frame,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use laser::Emulator;

    /// With no beacon (the Sitter holds UDP 7654, or the broadcast does not get through), what
    /// finds the DAC is the status request over TCP: the `Emulator` answers `a?` + `dac_status`
    /// when it accepts the connection, which is what the hardware does. It lives in the CLI
    /// because `protocols` does not depend on `laser`.
    #[test]
    fn tcp_fallback_finds_the_dac_with_no_beacon() {
        let emu = Emulator::start(1800).expect("emulator");
        let vizinhos = [("127.0.0.1".to_string(), "8a:9e:36:98:8c:ce".to_string())];
        let wait = Duration::from_millis(500);

        let achados = netscan::probe_etherdream(&vizinhos, emu.port, wait);
        assert_eq!(achados.len(), 1, "{achados:?}");
        assert_eq!(achados[0].ip, "127.0.0.1");
        assert_eq!(achados[0].mac, "8a:9e:36:98:8c:ce");
        assert_eq!(achados[0].via, "tcp");
        assert_eq!(achados[0].status.protocol, 1);

        // a port with nobody listening: an empty list, with no hang
        let livre = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let porta = livre.local_addr().unwrap().port();
        drop(livre);
        assert!(netscan::probe_etherdream(&vizinhos, porta, wait).is_empty());
    }
}
