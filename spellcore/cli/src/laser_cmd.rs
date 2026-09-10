//! ILDA player: os verbos da funcao `spell ilda play` como comandos do registry.
//!
//! Mora na CLI, e nao no engine, pela mesma razao de `play_show` e `net`: o engine nao conhece
//! o crate `laser`. Como todo comando do produto, e' um `Registry::add`, entao CLI (`spellcore
//! commands`), MCP, OSC e GUI o veem sem uma linha a mais.
//!
//! Um feed = um DAC aberto. A tabela `FEEDS` e' para o laser o que `player::current()` e' para o
//! transporte: um processo, N feeds, cada um com a thread do DAC (do proprio `Feed`) e, enquanto
//! toca, uma thread que le o `.ild` e empurra frames no ritmo pedido.
//!
//! `laser_param path -> campo` (os mesmos paths de `modules/laser.json`, a declaracao do modulo):
//!
//! | path | campo | faixa |
//! |---|---|---|
//! | `geo/x`, `geo/y` | `Transform.x`, `Transform.y` | unidades ILDA, +-32767 |
//! | `geo/scale` | `Transform.scale` | 0..4 |
//! | `geo/rot` | `Transform.rot` | graus, +-180 |
//! | `limit/r`, `limit/g`, `limit/b` | `Transform.color.0/.1/.2` | 0..1 |
//! | `safe/min_size` | `Safety.min_size` | unidades ILDA, 0..32767 |
//! | `safe/max_intensity` | `Safety.max_intensity` | 0..255 |
//! | `shutter` | zera `Safety.max_intensity` e devolve o valor guardado ao abrir | 0 ou 1 |
//!
// ponytail: so' os campos que `Transform` e `Safety` ja' tem ; `curve/r|g|b`, `Blanking/*` e
// `Cor/Time Shift` da tabela do ilda-player entram quando o `Feed` tiver LUT de cor e o
// `optimize` for parametrizavel em runtime.

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

/// Buffer do Ether Dream em pontos. O beacon carrega o valor real (`buffer_capacity`), mas
/// `laser_open` aceita host digitado, sem beacon.
// ponytail: capacidade fixa em 1800 (o padrao do hardware) ; ler do beacon quando `laser_open`
// aceitar o id devolvido por `laser_dacs` em vez do host.
const CAPACITY: u16 = 1800;

// -------------------------------------------------------------- tabela de feeds

struct Play {
    file: String,
    /// A thread zera ao sair, inclusive quando o arquivo acaba sem loop; `colher` recolhe.
    run: Arc<AtomicBool>,
    th: Option<JoinHandle<()>>,
}

struct Live {
    id: u64,
    name: String,
    feed: Arc<Feed>,
    tf: Transform,
    safety: Safety,
    /// `max_intensity` guardado enquanto o obturador esta' fechado; `None` = obturador aberto.
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
        .ok_or_else(|| format!("feed {} nao existe", id))
}

/// Para a thread de playback e espera ela sair. Idempotente.
fn parar(l: &mut Live) {
    if let Some(mut p) = l.play.take() {
        p.run.store(false, Ordering::Relaxed);
        if let Some(h) = p.th.take() {
            let _ = h.join();
        }
    }
}

/// Recolhe a thread que terminou sozinha (fim do arquivo sem loop): `play` so' existe
/// enquanto toca, entao `playing` nunca fica preso em `true`.
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
    /// Segundos de escuta por protocolo.
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
    // ponytail: sem Helios na lista ; o DAC USB entra quando o driver (hidapi/rusb) entrar —
    // `laser::dac::helios` hoje so' tem o encoder do frame, documentado para o porte.
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
    /// etherdream ou idn.
    dac: String,
    /// "ip" ou "ip:porta" (o `host` de `laser_dacs`).
    #[serde(default)]
    host: String,
    /// Milhares de pontos por segundo entregues ao DAC.
    #[serde(default = "def_kpps")]
    kpps: f64,
    /// Safety do feed: {"min_size": 2000, "max_intensity": 255, "zone": [x0,y0,x1,y1]} em
    /// unidades ILDA. Ausente = o padrao. Nunca desligavel.
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
        return Err(format!("laser_open {} exige host (veja laser_dacs)", a.dac));
    }
    let d: Box<dyn Dac> = match a.dac.as_str() {
        "etherdream" => Box::new(
            EtherDream::connect(&a.host, CAPACITY)
                .map_err(|e| format!("etherdream {}: {}", a.host, e))?,
        ),
        "idn" => Box::new(Idn::connect(&a.host, 0).map_err(|e| format!("idn {}: {}", a.host, e))?),
        o => return Err(format!("dac desconhecido: {} (etherdream, idn)", o)),
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
    /// Id devolvido por `laser_open`.
    feed: u64,
    /// Caminho do .ild.
    file: String,
    /// Frames por segundo empurrados ao DAC (o .ild nao carrega taxa).
    #[serde(default = "def_fps")]
    fps: f64,
    /// Recomeca do primeiro frame ao chegar no fim.
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
        return Err(format!("{}: nenhum frame", a.file));
    }
    let n = frames.len();
    let dt = Duration::from_secs_f64(1.0 / a.fps.clamp(1.0, 240.0));
    let mut v = lock(&FEEDS);
    let l = achar(&mut v, a.feed)?;
    parar(l);
    let run = Arc::new(AtomicBool::new(true));
    let (feed, r, looping) = (l.feed.clone(), run.clone(), a.looping);
    // ponytail: os frames vao inteiros para a memoria antes de tocar ; o maior .ild do repo tem
    // 1,2 MB. Vira leitura por frame quando alguem trouxer um .ild de show inteiro.
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
        r.store(false, Ordering::Relaxed); // fim do arquivo: `colher` desarma o `play`
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
    /// Id devolvido por `laser_open`.
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
        .ok_or_else(|| format!("feed {} nao existe", a.feed))?;
    let mut l = v.remove(i);
    parar(&mut l);
    drop(v); // o `Feed` para a thread do DAC no Drop; nao segure a tabela enquanto ele junta
    Ok(json!({"feed": a.feed, "dac": l.name, "closed": true}))
}

// ----------------------------------------------------------------- laser_param

#[derive(Deserialize, JsonSchema)]
#[schemars(crate = "engine::schemars")]
struct ParamArgs {
    /// Id devolvido por `laser_open`.
    feed: u64,
    /// geo/x, geo/y, geo/scale, geo/rot, limit/r, limit/g, limit/b, safe/min_size,
    /// safe/max_intensity, shutter.
    path: String,
    /// Valor na faixa do `path` (a tabela de faixas esta no spellcore/README.md).
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
        // com o obturador fechado o valor pedido fica guardado e vale quando ele abrir
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
        p => return Err(format!("path desconhecido: {} ({})", p, PATHS)),
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

/// Os `stat/*` sao os `values` de `modules/laser.json`; so' saem os que `FeedStats` tem
/// (`stat/fps`, `stat/points` e `stat/clipped` entram quando o `Feed` os contar).
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
    /// Diretorio a listar; vazio = `shows/`.
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
    /// Caminho do .ild, ou o nome que o track usa (resolve na pasta do show aberto).
    clip: String,
    /// Tempo em segundos; com `fps` escolhe o quadro. Ignorado quando vem `index`.
    #[serde(default)]
    t: f64,
    /// Quadro pedido direto; sem ele, o quadro de `t`.
    #[serde(default)]
    index: Option<usize>,
    /// Quadros por segundo do track (o .ild nao carrega taxa).
    #[serde(default = "def_fps")]
    fps: f64,
}

/// Nome que o track usa contra a pasta do .spell aberto, igual ao `_load_clip` do Python. Caminho
/// que ja' existe passa direto.
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

/// Ultimo .ild lido. O previz pede um quadro por vez e o mesmo arquivo toca o show inteiro.
// ponytail: cache de um arquivo so', sem olhar mtime ; virar mapa com mtime quando dois clips
// tocarem juntos ou quando o previz precisar ver o .ild trocado em disco sem reabrir o show.
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
        return Err(format!("{}: nenhum frame", p.display()));
    }
    // Mesma conta do player (`spellcaster/player/player.py::laser_frame`): o clipe repete.
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

// -------------------------------------------------------------------- registro

pub fn register(r: &mut Registry) {
    r.add::<DacsArgs>(
        "laser_dacs",
        "Procura DACs de laser: Ether Dream por beacon e IDN por scan.",
        dacs,
    );
    r.add::<OpenArgs>(
        "laser_open",
        "Abre um DAC de laser e devolve o id do feed. A safety e' obrigatoria e nunca desliga.",
        abrir,
    );
    r.add::<LaserPlayArgs>(
        "laser_play",
        "Toca um .ild no feed: le os frames e empurra a fps (o .ild nao carrega taxa).",
        tocar,
    );
    r.add::<FeedArgs>(
        "laser_stop",
        "Para o playback do feed; o DAC continua aberto.",
        parar_cmd,
    );
    r.add::<FeedArgs>("laser_close", "Para e fecha o feed (apaga o DAC).", fechar);
    r.add::<ParamArgs>(
        "laser_param",
        "Ajusta um parametro do feed: geo/x geo/y geo/scale geo/rot limit/r limit/g limit/b safe/min_size safe/max_intensity shutter.",
        param,
    );
    r.add::<FeedArgs>(
        "laser_stats",
        "Estado do feed: playing, arquivo, stat/sent, stat/dropped, stat/errors, jitter e cpu.",
        stats,
    );
    r.add::<FilesArgs>(
        "laser_files",
        "Lista os .ild de um diretorio (vazio = shows/).",
        files,
    );
    r.add::<ClipFrameArgs>(
        "clip_frame",
        "Um quadro do .ild para desenhar: escolhe por `index` ou por `t` a `fps`, e devolve os pontos [x, y, r, g, b, blank] com x e y normalizados em -1..1.",
        clip_frame,
    );
}
