//! Standalone player: transport (play/pause/stop/locate/loop) on its own thread over the Clock,
//! timeline -> sACN / Art-Net / OSC, cues and frame hooks. Port of
//! `spellcaster/player/player.py` without the laser part (the `laser` crate handles that).
//!
//! Evaluation order of one frame (README contract, section "R1"):
//!   1. `Timeline::apply` (`dmx` and `artnet` tracks) and the Capture `media` tracks;
//!   2. each `FrameHook` in the order it was registered;
//!   3. side-effect tracks: `osc`, non-Capture `media` and `cue`;
//!   4. `CueList::update` writes the current snapshot;
//!   5. the programmer (`Prog`): the operator manual override, HTP per channel, ON TOP of the
//!      live cue (the operator overrides whatever the cue is holding);
//!   6. the GLOBAL hooks (`hook_global`, the `serve` monitor): the frame already complete, with
//!      the programmer inside, before it goes out on the network;
//!   7. I/O: every written universe goes to every output.
//!
//! Remote transport over OSC (`/spellcaster/play|pause|stop|locate f`) never touches the
//! Universes: it only acts on the `Handle`, and the transport thread does the rest on the next
//! frame.

use crate::clock::{Clock, State};
use crate::cues::CueList;
use crate::hook::FrameHook;
use crate::input::Inputs;
use crate::show::{OutputCfg, Show};
use crate::timeline::{Side, Timeline, Value};
use crate::universe::Universes;
use protocols::osc::{Arg, OscIn, OscOut};
use protocols::{artnet::ArtNetOut, sacn::SacnOut, Output};
use serde::Serialize;
use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

/// Live player in this process (Python's `CURRENT`): what the registry commands drive.
static CURRENT: Mutex<Option<Handle>> = Mutex::new(None);

/// Hooks installed on EVERY player started in this process, at position 6 of the frame (after
/// the programmer, before the I/O): this is how the `serve` monitor sees the frame that really
/// goes out — with the operator override inside — without the engine knowing any GUI.
type Global = Box<dyn Fn() -> Box<dyn FrameHook> + Send + Sync>;
static GLOBAL: Mutex<Vec<Global>> = Mutex::new(Vec::new());

/// Registers a global hook factory; it applies to the next `start()` calls, not to the player
/// already running.
// ponytail: it only registers, never removes ; the one client is `serve`, which lives for the
// whole process.
pub fn hook_global(f: impl Fn() -> Box<dyn FrameHook> + Send + Sync + 'static) {
    lock(&GLOBAL).push(Box::new(f));
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// ------------------------------------------------------------------ outputs

/// Opens the DMX outputs declared in `outputs`. An unknown type becomes a warning, not an error.
/// (Moved here from the R0 CLI with no behavior change: the Player is what opens the outputs now.)
pub fn open_outputs(sh: &Show) -> Result<Vec<Box<dyn Output>>, String> {
    let mut outs: Vec<Box<dyn Output>> = Vec::new();
    for c in &sh.outputs {
        match c {
            OutputCfg::Sacn(c) => {
                let ifaces = c.interfaces.as_ref().map(|v| {
                    v.iter()
                        .filter_map(|s| s.parse::<Ipv4Addr>().ok())
                        .collect::<Vec<_>>()
                });
                let o = SacnOut::with(&c.universes, c.priority, &c.source_name, ifaces)
                    .map_err(|e| format!("sacn: {}", e))?;
                outs.push(Box::new(o));
            }
            OutputCfg::ArtNet(c) => {
                let o = ArtNetOut::new(c.targets.clone(), c.broadcast)
                    .map_err(|e| format!("artnet: {}", e))?;
                outs.push(Box::new(o));
            }
            OutputCfg::Osc(_) => {} // a message output, not a universe one: it goes in `osc_out`
            OutputCfg::Unknown { tipo } => eprintln!("warning: output \"{}\" ignored", tipo),
        }
    }
    Ok(outs)
}

/// Bare number from the .spell (`in`, `out`): anything that is not a number counts as absent.
fn num(sh: &Show, k: &str) -> Option<f64> {
    sh.extra.get(k).and_then(|v| v.as_f64())
}

/// First `osc` output of the show (Python also keeps only one).
fn open_osc(sh: &Show) -> Result<Option<OscOut>, String> {
    for c in &sh.outputs {
        if let OutputCfg::Osc(o) = c {
            if o.port == 0 {
                eprintln!("warning: osc output without \"port\" ignored");
                continue;
            }
            return OscOut::new(&o.host, o.port)
                .map(Some)
                .map_err(|e| format!("osc: {}", e));
        }
    }
    Ok(None)
}

// ------------------------------------------------------------------- state

#[derive(Serialize, Clone, Debug)]
pub struct TransportState {
    pub t: f64,
    pub state: &'static str,
    pub cue: i32,
    pub frames: u64,
    pub fps: u32,
    pub duration: Option<f64>,
    pub universes: Vec<u16>,
    /// Loop on, and the range it repeats over. It is ENGINE state: the GUI only mirrors it.
    #[serde(rename = "loop")]
    pub looping: bool,
    pub loop_in: f64,
    pub loop_out: f64,
}

/// Transport loop: on/off and the In-Out range it repeats over. It lives here, not in the
/// client, because what walks in time is the transport thread — a GUI faking the loop on its own
/// jumped back to the In while the player kept playing to the end of the show.
#[derive(Clone, Copy, Debug)]
struct Loop {
    on: bool,
    a: f64,
    b: f64,
}

impl Loop {
    /// Instant to jump back to when `t` ran past the end of the range; None when there is no jump.
    fn wrap(&self, t: f64) -> Option<f64> {
        if self.on && self.b > self.a && t >= self.b {
            Some(self.a)
        } else {
            None
        }
    }
}

/// A request from another thread to the transport thread, consumed at the start of the frame.
enum Ctl {
    Go(Option<usize>),
    /// locate/stop: clears cues and hooks and re-anchors the `prev` of the edge trigger.
    Reset(f64),
    /// Input event for the hooks ("widget:go", "key:Space", "module:laser/stat/fps").
    Input(String, f64),
}

/// Programmer: the operator manual layer, on top of the timeline. One `Option<u8>` per channel =
/// value and "touched" mask in the same structure.
// ponytail: a 1 KB block per universe ; a show has few universes, and a `BTreeMap<(u16,u16),u8>`
// would allocate per channel inside the frame.
#[derive(Default)]
pub struct Prog {
    v: BTreeMap<u16, [Option<u8>; 512]>,
    /// Channels released since the last frame: zeroed BEFORE the timeline, otherwise the last
    /// override value stays stuck in the buffer (nobody else writes that channel).
    freed: Vec<(u16, u16)>,
}

impl Prog {
    /// Writes `values` from `addr` on (1-based); clamps to 0..255, ignores whatever runs past 512.
    pub fn set(&mut self, u: u16, addr: u16, values: &[f64]) {
        if addr == 0 || addr > 512 {
            return;
        }
        let i = (addr - 1) as usize;
        let s = self.v.entry(u).or_insert([None; 512]);
        for (d, v) in s[i..].iter_mut().zip(values) {
            *d = Some(*v as u8);
        }
    }

    /// Releases the override of one universe (or of all); returns how many channels were freed.
    pub fn clear(&mut self, u: Option<u16>) -> usize {
        let mut n = 0;
        for (num, s) in self.v.iter_mut() {
            if u.is_some_and(|x| x != *num) {
                continue;
            }
            for (i, c) in s.iter_mut().enumerate() {
                if c.take().is_some() {
                    self.freed.push((*num, i as u16 + 1));
                    n += 1;
                }
            }
        }
        n
    }

    /// Touched channels, in order: (universe, 1-based address, value).
    pub fn levels(&self, u: Option<u16>) -> Vec<(u16, u16, u8)> {
        let mut out = Vec::new();
        for (n, s) in self.v.iter() {
            if u.is_some_and(|x| x != *n) {
                continue;
            }
            out.extend(
                s.iter()
                    .enumerate()
                    .filter_map(|(i, c)| c.map(|v| (*n, i as u16 + 1, v))),
            );
        }
        out
    }

    /// Zeroes in the buffer the channels released since the last frame. It runs BEFORE
    /// `Timeline::apply`, so whatever the timeline owns takes over again on the same frame.
    fn release(&mut self, uni: &mut Universes) {
        for (n, a) in self.freed.drain(..) {
            uni.get_or_create(n).set_bytes(a, &[0]);
        }
    }

    /// HTP per channel over whatever is already in the buffer.
    fn apply(&self, uni: &mut Universes) {
        for (n, s) in self.v.iter() {
            let d = &mut uni.get_or_create(*n).data;
            for (i, c) in s.iter().enumerate() {
                if let Some(v) = c {
                    d[i] = d[i].max(*v);
                }
            }
        }
    }
}

struct Shared {
    clock: Clock,
    /// DMX inputs of the show (`show.inputs`): the last frame received on each universe.
    inputs: Arc<Inputs>,
    ctl: Mutex<Vec<Ctl>>,
    lp: Mutex<Loop>,
    prog: Mutex<Prog>,
    cue: AtomicI32,
    frames: AtomicU64,
    universes: Mutex<Vec<u16>>,
    duration: Option<f64>,
    run: AtomicBool,
    done: Mutex<bool>,
    cv: Condvar,
}

impl Shared {
    fn push(&self, c: Ctl) {
        lock(&self.ctl).push(c);
    }

    /// Wakes whoever is in `wait()`.
    fn finish(&self) {
        *lock(&self.done) = true;
        self.cv.notify_all();
    }
}

/// Transport from outside the player thread. Cloneable; the registry keeps one.
#[derive(Clone)]
pub struct Handle {
    s: Arc<Shared>,
}

impl Handle {
    pub fn play(&self) {
        *lock(&self.s.done) = false;
        self.s.clock.play();
    }

    pub fn pause(&self) {
        self.s.clock.pause();
    }

    /// Stops and returns to zero (Python's `stop()`: clock.stop + cues.reset + done).
    pub fn stop(&self) {
        self.s.clock.stop();
        self.s.push(Ctl::Reset(0.0));
        self.s.finish();
    }

    pub fn locate(&self, t: f64) {
        self.s.clock.locate(t);
        self.s.push(Ctl::Reset(t));
    }

    pub fn cue_go(&self, index: Option<usize>) {
        self.s.push(Ctl::Go(index));
    }

    /// Delivers `FrameHook::input` to every hook on the next frame: it is the registry `input`
    /// command, through which the GUI and the bus feed the Graph.
    pub fn input(&self, key: &str, value: f64) {
        self.s.push(Ctl::Input(key.to_string(), value));
    }

    /// Programmer: writes into the manual override (it takes effect on the next frame). An
    /// address outside 1..512 is an error, not silence: this is the one funnel of the `level_set`
    /// and `fixture_set` commands.
    pub fn level_set(&self, u: u16, addr: u16, values: &[f64]) -> Result<(), String> {
        if addr == 0 || addr > 512 {
            return Err(format!("address {} outside 1..512", addr));
        }
        lock(&self.s.prog).set(u, addr, values);
        Ok(())
    }

    /// Releases the override of one universe, or of all; returns how many channels were freed.
    pub fn level_clear(&self, u: Option<u16>) -> usize {
        lock(&self.s.prog).clear(u)
    }

    /// (universe, address, value) of each channel touched by the operator.
    pub fn levels(&self, u: Option<u16>) -> Vec<(u16, u16, u8)> {
        lock(&self.s.prog).levels(u)
    }

    /// Turns the loop on/off and stores the range (the show In-Out). It takes effect next frame.
    pub fn set_loop(&self, on: bool, a: f64, b: f64) {
        *lock(&self.s.lp) = Loop {
            on,
            a: a.max(0.0),
            b,
        };
    }

    /// Last frame received on the INPUT universe, or `None` (universe not declared in
    /// `show.inputs`, or nothing has arrived yet). There is no merge with the output.
    pub fn input_get(&self, universe: u16) -> Option<[u8; 512]> {
        self.s.inputs.get(universe)
    }

    /// (universe, frame) of each input that has already received something — the `serve` monitor.
    pub fn input_frames(&self) -> Vec<(u16, [u8; 512])> {
        self.s.inputs.frames()
    }

    pub fn state(&self) -> TransportState {
        let lp = *lock(&self.s.lp);
        TransportState {
            t: self.s.clock.time(),
            state: match self.s.clock.state() {
                State::Play => "play",
                State::Pause => "pause",
                State::Stop => "stop",
            },
            cue: self.s.cue.load(Ordering::Relaxed),
            frames: self.s.frames.load(Ordering::Relaxed),
            fps: self.s.clock.fps(),
            duration: self.s.duration,
            universes: lock(&self.s.universes).clone(),
            looping: lp.on,
            loop_in: lp.a,
            loop_out: lp.b,
        }
    }
}

// ------------------------------------------------------------ frame runtime

/// The part of the player that only the transport thread touches.
struct Rt {
    tl: Timeline,
    cues: CueList,
    hooks: Vec<Box<dyn FrameHook>>,
    /// Global hooks (`hook_global`): they run AFTER the programmer, at position 6, so they see
    /// the frame that actually goes out — the operator manual override included.
    globais: Vec<Box<dyn FrameHook>>,
    uni: Universes,
    outs: Vec<Box<dyn Output>>,
    osc_out: Option<OscOut>,
    inputs: Arc<Inputs>,
    prev: f64,
    nuni: usize, // how many universes went out on the last frame (only the transport thread reads it)
    pend: Vec<Ctl>, // queue drained by swap: zero allocation per frame
}

impl Rt {
    fn drain(&mut self, s: &Shared, t: f64) {
        // MIDI: each event becomes `input {key: "midi:<key>"}` in the queue below (plus the map
        // command, if there is one). Consumed HERE, where `input` is already consumed: the frame
        // order documented in the header does not change.
        crate::midi::pump();
        {
            let mut q = lock(&s.ctl);
            if q.is_empty() {
                return;
            }
            std::mem::swap(&mut *q, &mut self.pend);
        }
        let mut pend = std::mem::take(&mut self.pend);
        for c in pend.drain(..) {
            match c {
                Ctl::Go(i) => {
                    self.cues.go(t, i);
                }
                Ctl::Reset(x) => self.reset(x),
                Ctl::Input(k, v) => {
                    for h in self.hooks.iter_mut() {
                        h.input(&k, v);
                    }
                }
            }
        }
        self.pend = pend; // hands back the empty Vec with its capacity already allocated
        s.cue.store(self.cues.index(), Ordering::Relaxed);
    }

    /// Rewinds cues, hooks and the frame `prev` to instant `t`: locate, loop and stop.
    fn reset(&mut self, t: f64) {
        self.cues.reset();
        for h in self.hooks.iter_mut() {
            h.reset(t);
        }
        self.prev = t;
    }

    fn tick(&mut self, s: &Shared, t: f64) {
        self.drain(s, t);
        // Loop over the In-Out range: read every frame, so turning it on/off during play takes
        // effect at once. The clock jumps back to the In and the wrap frame goes out on the next
        // tick, already inside the range — no writing a frame from outside it onto the network.
        if let Some(a) = lock(&s.lp).wrap(t) {
            s.clock.locate(a);
            self.reset(a);
            return;
        }
        // 1. timeline (dmx/artnet) and Capture media
        lock(&s.prog).release(&mut self.uni);
        self.tl.apply(&mut self.uni, t);
        let Rt {
            tl,
            cues,
            hooks,
            globais,
            uni,
            outs,
            osc_out,
            inputs,
            prev,
            nuni,
            ..
        } = self;
        let Timeline {
            tracks,
            osc,
            media,
            cue,
            ..
        } = tl;
        for &i in media.iter() {
            let tr = &tracks[i];
            if tr.capture {
                if let Some(v) = tr.media_frame(t) {
                    uni.get_or_create(tr.universe).set(tr.address, &v);
                }
            }
        }
        // 2. frame hooks (fx tracks and Graph), in registration order
        for h in hooks.iter_mut() {
            h.frame(t, uni);
        }
        // 3. side effects: recording (it reads the INPUT and writes a keyframe into the open
        // show), OSC, non-Capture media and cue. It only records while PLAYING: when paused
        // `Clock::run` keeps calling the frame with `t` frozen, and recording there would rewrite
        // the same keyframe on every console change (`rec_arm` promises "with transport playing").
        if s.clock.state() == State::Play {
            crate::rec::tick(t, inputs);
        }
        if let Some(o) = osc_out.as_ref() {
            for &i in osc.iter() {
                send_osc(&mut tracks[i], t, o, false);
            }
            for &i in media.iter() {
                if !tracks[i].capture {
                    send_osc(&mut tracks[i], t, o, true);
                }
            }
        }
        for &i in cue.iter() {
            for k in tracks[i].keys.crossed(*prev, t) {
                let idx = match &k.value {
                    Value::Text(v) if v.eq_ignore_ascii_case("go") => None,
                    Value::Text(v) => match v.trim().parse::<usize>() {
                        Ok(n) => Some(n),
                        Err(_) => continue,
                    },
                    Value::Num(n) if *n >= 0.0 => Some(*n as usize),
                    _ => continue,
                };
                cues.go(t, idx);
            }
        }
        // 4. cue snapshot
        cues.update(t, uni);
        s.cue.store(cues.index(), Ordering::Relaxed);
        // 5. programmer: the operator manual override, HTP over the timeline AND the live cue
        lock(&s.prog).apply(uni);
        // 6. global hooks (the serve monitor): the frame already complete, before it hits the network
        for h in globais.iter_mut() {
            h.frame(t, uni);
        }
        // 7. I/O
        // ponytail: every written universe goes to EVERY output of the show (same as R0)
        // ; split per output once a show mixes "dmx" and "artnet" on the same universe.
        for u in uni.iter() {
            for o in outs.iter_mut() {
                o.send(u.number, &u.data);
            }
        }
        *prev = t;
        s.frames.fetch_add(1, Ordering::Relaxed);
        if uni.len() != *nuni {
            *nuni = uni.len();
            *lock(&s.universes) = uni.numbers();
        }
    }

    fn close(&mut self) {
        for o in self.outs.iter_mut() {
            o.close();
        }
        if let Some(o) = self.osc_out.as_mut() {
            o.close();
        }
    }
}

/// Sends the current value of a side-effect track when it changes.
// ponytail: it builds `Vec<Arg>` and `String` only on a value CHANGE, not per frame ; make it a
// reused buffer if some show comes to have an OSC track that changes every frame with many args.
fn send_osc(tr: &mut crate::timeline::Track, t: f64, out: &OscOut, media: bool) {
    match tr.changed(t) {
        Side::Same => {}
        // ponytail: a number goes out as OSC float ('f'); Python sends 'i' when the keyframe is
        // an integer ; match the type if some device refuses float.
        Side::Nums => {
            if media {
                let v = tr.nums().first().copied().unwrap_or(0.0);
                out.send(&media_address(tr.text_address.as_str(), &fmt(v)), &[]);
            } else {
                let args: Vec<Arg> = tr.nums().iter().map(|v| Arg::Float(*v as f32)).collect();
                out.send(&tr.text_address, &args);
            }
        }
        Side::Text => {
            if media {
                out.send(&media_address(tr.text_address.as_str(), tr.text()), &[]);
            } else {
                out.send(&tr.text_address, &[Arg::Str(tr.text().to_string())]);
            }
        }
    }
}

/// non-Capture media: "address/value" (Python's `rstrip("/") + "/" + str(v)`).
fn media_address(address: &str, v: &str) -> String {
    format!("{}/{}", address.trim_end_matches('/'), v)
}

fn fmt(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

// ------------------------------------------------------------------- Player

pub struct Player {
    rt: Option<Rt>, // goes to the thread on start()
    s: Arc<Shared>,
    th: Option<JoinHandle<()>>,
    osc_in: Option<OscIn>,
    osc_port: Option<u16>,
}

impl Player {
    /// Loads timeline + cues and opens the outputs of `show.outputs` (sacn, artnet, osc).
    pub fn new(show: Show, looping: bool) -> Result<Player, String> {
        let tl = Timeline::new(&show)?;
        let ign = tl.ignored().join(", ");
        if !ign.is_empty() {
            eprintln!("warning: tracks ignored: {}", ign);
        }
        let cues = CueList::new(
            show.extra
                .get("cues")
                .and_then(|v| v.as_array())
                .map(|a| a.as_slice())
                .unwrap_or(&[]),
        );
        crate::midi::auto(&show); // .spell "midi_port": reconnects the control surface when the show starts
        let outs = open_outputs(&show)?;
        let osc_out = open_osc(&show)?;
        let inputs = Arc::new(Inputs::open(&show)?);
        let osc_port = show
            .extra
            .get("transport")
            .and_then(|v| v.get("osc_port"))
            .and_then(|v| v.as_u64())
            .map(|p| p as u16);
        // Loop range = the show In-Out; without them, 0..duration (the CLI `--loop`, which
        // repeated the whole show, stays the same). `loop_set` rewrites this with the player live.
        let a = num(&show, "in").unwrap_or(0.0).max(0.0);
        let b = num(&show, "out").or(show.duration).unwrap_or(0.0);
        Ok(Player {
            s: Arc::new(Shared {
                clock: Clock::new(show.fps),
                inputs: inputs.clone(),
                ctl: Mutex::new(Vec::new()),
                lp: Mutex::new(Loop { on: looping, a, b }),
                prog: Mutex::new(Prog::default()),
                cue: AtomicI32::new(-1),
                frames: AtomicU64::new(0),
                universes: Mutex::new(Vec::new()),
                duration: show.duration,
                run: AtomicBool::new(false),
                done: Mutex::new(false),
                cv: Condvar::new(),
            }),
            rt: Some(Rt {
                tl,
                cues,
                hooks: Vec::new(),
                globais: Vec::new(),
                uni: Universes::new(),
                outs,
                osc_out,
                inputs,
                prev: 0.0,
                nuni: 0,
                pend: Vec::new(),
            }),
            th: None,
            osc_in: None,
            osc_port,
        })
    }

    /// Script/graph hooks, before `start()`, in the order they must run.
    pub fn hook(&mut self, h: Box<dyn FrameHook>) {
        if let Some(rt) = self.rt.as_mut() {
            rt.hooks.push(h);
        }
    }

    /// An extra output beyond the ones declared in the show, before `start()`.
    pub fn output(&mut self, o: Box<dyn Output>) {
        if let Some(rt) = self.rt.as_mut() {
            rt.outs.push(o);
        }
    }

    pub fn handle(&self) -> Handle {
        Handle { s: self.s.clone() }
    }

    pub fn clock(&self) -> Clock {
        self.s.clock.clone()
    }

    /// Starts the transport thread and, if `osc_port` (argument or `transport.osc_port`), the
    /// OscIn of the remote transport. Registers this player as the `current()`.
    pub fn start(&mut self, osc_port: Option<u16>) -> Result<(), String> {
        if self.th.is_some() {
            return Ok(());
        }
        let mut rt = self.rt.take().ok_or("player already closed")?;
        for f in lock(&GLOBAL).iter() {
            rt.globais.push(f());
        }
        let s = self.s.clone();
        s.run.store(true, Ordering::Relaxed);
        let th = std::thread::Builder::new()
            .name("spell-transport".into())
            .spawn(move || transport(rt, s))
            .map_err(|e| format!("transport thread: {}", e))?;
        self.th = Some(th);
        if let Some(p) = osc_port.or(self.osc_port).filter(|p| *p > 0) {
            let mut i = OscIn::new(p).map_err(|e| format!("osc {}: {}", p, e))?;
            let (a, b, c, d) = (self.handle(), self.handle(), self.handle(), self.handle());
            i.on("/spellcaster/play", move |_, _| a.play());
            i.on("/spellcaster/pause", move |_, _| b.pause());
            i.on("/spellcaster/stop", move |_, _| c.stop());
            i.on("/spellcaster/locate", move |_, g| {
                d.locate(g.first().map(argf).unwrap_or(0.0))
            });
            self.osc_in = Some(i);
        }
        *lock(&CURRENT) = Some(self.handle());
        Ok(())
    }

    /// Blocks until stop() or the end of the show. `false` = the timeout ran out.
    pub fn wait(&self, timeout: Option<Duration>) -> bool {
        let d = lock(&self.s.done);
        match timeout {
            None => {
                drop(
                    self.s
                        .cv
                        .wait_while(d, |x| !*x)
                        .unwrap_or_else(|e| e.into_inner()),
                );
                true
            }
            Some(to) => {
                let (_g, r) = self
                    .s
                    .cv
                    .wait_timeout_while(d, to, |x| !*x)
                    .unwrap_or_else(|e| e.into_inner());
                !r.timed_out()
            }
        }
    }

    /// Idempotent: stops the thread, closes the outputs and clears the `current()`.
    pub fn close(&mut self) {
        // With no player there is no recording. It is here and not only in the thread Stop branch
        // because `close()` clears `run` before the next frame: `play_show` closes the player on
        // the operator `stop`, and the thread exits without going through that branch.
        crate::rec::disarm();
        self.s.run.store(false, Ordering::Relaxed);
        self.s.clock.stop();
        self.s.finish();
        if let Some(th) = self.th.take() {
            let _ = th.join();
        }
        if let Some(mut i) = self.osc_in.take() {
            i.close();
        }
        if let Some(mut rt) = self.rt.take() {
            rt.close(); // start() was never called: the outputs are still here
        }
        let mut c = lock(&CURRENT);
        if c.as_ref().is_some_and(|h| Arc::ptr_eq(&h.s, &self.s)) {
            *c = None;
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.close();
    }
}

fn argf(a: &Arg) -> f64 {
    match a {
        Arg::Int(v) => *v as f64,
        Arg::Long(v) => *v as f64,
        Arg::Float(v) => *v as f64,
        Arg::Double(v) => *v,
        Arg::Bool(v) => *v as i32 as f64,
        Arg::Str(s) => s.trim().parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Transport thread: Python's `_loop`.
fn transport(mut rt: Rt, s: Arc<Shared>) {
    // The clock is born in Stop: it starts `parado` so arming BEFORE the first play survives.
    let mut parado = true;
    while s.run.load(Ordering::Relaxed) {
        if s.clock.state() == State::Stop {
            // Stopping disarms the recording, on the EDGE into stop and not while stopped:
            // arming with the transport stopped and only then hitting play is the operator path.
            // (The other disarm point is `Player::close`.)
            if !parado {
                crate::rec::disarm();
                parado = true;
            }
            let t = s.clock.time();
            rt.drain(&s, t); // locate/stop with the clock stopped also clears cues and hooks
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }
        parado = false;
        {
            let sc = s.clone();
            s.clock.run(|t| rt.tick(&sc, t), s.duration);
        }
        if !s.run.load(Ordering::Relaxed) {
            break;
        }
        if s.clock.state() != State::Stop {
            // it reached the duration: back to the loop In, or stop
            // ponytail: `loop` with no `duration` only repeats when there is an Out — with neither
            // end nor Out, the run only returns on stop.
            let lp = *lock(&s.lp);
            if lp.on && s.duration.is_some() {
                s.clock.locate(lp.a);
                rt.reset(lp.a);
            } else {
                s.clock.stop();
                rt.reset(0.0);
                s.cue.store(-1, Ordering::Relaxed);
                s.finish();
            }
        }
    }
    rt.close();
}

/// Live player in this process (Python's `CURRENT`). Used by the registry commands.
pub fn current() -> Option<Handle> {
    lock(&CURRENT).clone()
}
