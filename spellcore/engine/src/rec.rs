//! Recording: an armed `dmx` track reads the INPUT universe (`input.rs`) every frame and writes
//! a keyframe only when the value changes. The write goes through the same funnel as a manual
//! edit (`edit::key_put`), so `rev` bumps and the bus tells the clients.
//!
//! Commands: `rec_arm {track, on}` and `rec_state`. STOPPING the transport disarms everything:
//! the transport thread calls `disarm()` on the edge into Stop, not while stopped — arming with
//! the transport stopped and only then hitting play is the operator normal path.

use crate::edit;
use crate::input::Inputs;
use crate::registry::{lock, NoArgs, Registry};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// One armed track: where to read the input and what has been recorded so far.
struct Arm {
    track: usize,
    universe: u16,
    address: u16,
    width: usize,
    /// Last value recorded; empty = nothing recorded yet (the first frame always records).
    last: Vec<u8>,
}

static ARMED: Mutex<Vec<Arm>> = Mutex::new(Vec::new());
/// Mirror of the size of `ARMED`: the transport thread reads this per frame and only takes the
/// mutex when something is armed.
static N: AtomicUsize = AtomicUsize::new(0);

/// Arms or disarms a track. The type, the universe and the address come from the OPEN show
/// (`edit`), not from the player timeline: the keyframe is going to be written in the open show.
pub fn arm(indice: usize, on: bool) -> Result<Value, String> {
    // `track_dmx` takes the OPEN lock and releases it before ARMED comes in: the order of the two
    // locks is always ARMED -> OPEN (that is what `tick` does), never the other way round.
    let info = if on {
        Some(edit::track_dmx(indice)?)
    } else {
        None
    };
    let mut g = lock(&ARMED);
    g.retain(|a| a.track != indice);
    if let Some((universe, address, width)) = info {
        g.push(Arm {
            track: indice,
            universe,
            address,
            width,
            last: Vec::new(),
        });
    }
    N.store(g.len(), Ordering::Relaxed);
    Ok(json!({"track": indice, "on": on, "tracks": lista(&g)}))
}

fn lista(g: &[Arm]) -> Vec<usize> {
    g.iter().map(|a| a.track).collect()
}

/// Armed tracks, in arming order.
pub fn state() -> Value {
    let g = lock(&ARMED);
    json!({"recording": !g.is_empty(), "tracks": lista(&g)})
}

/// Disarms everything (the transport stopped). Cheap when nothing is armed.
pub fn disarm() {
    if N.load(Ordering::Relaxed) == 0 {
        return;
    }
    lock(&ARMED).clear();
    N.store(0, Ordering::Relaxed);
}

/// One recording frame: for each armed track, it reads the input universe and writes a keyframe
/// at `t` when the channels of the track change. It runs on step 3 of the frame (side effect),
/// before the cues, and does not touch the output Universes.
// ponytail: one keyframe per CHANGE, no thinning ; a fader moving at 60 fps leaves 60 keys per
// second. Curve reduction (Douglas-Peucker) lands when the recorded file starts to hurt.
pub fn tick(t: f64, inputs: &Inputs) {
    if N.load(Ordering::Relaxed) == 0 {
        return;
    }
    let mut g = lock(&ARMED);
    let mut caiu: Vec<usize> = Vec::new();
    for (n, a) in g.iter_mut().enumerate() {
        let Some(frame) = inputs.get(a.universe) else {
            continue;
        };
        let i = (a.address - 1) as usize;
        let novo = &frame[i..(i + a.width).min(512)];
        if a.last == novo {
            continue;
        }
        a.last.clear();
        a.last.extend_from_slice(novo);
        let v = if novo.len() == 1 {
            json!(novo[0])
        } else {
            json!(novo)
        };
        if let Err(e) = edit::key_put(a.track, t, v, "linear") {
            // Failing here means the track is gone (`track_del`) or the show was swapped: the
            // index kept by the arm does not exist any more. Disarm ONCE, instead of repeating
            // the error every frame with a stale index.
            eprintln!("recording of track {}: {} - disarmed", a.track, e);
            caiu.push(n);
        }
    }
    if !caiu.is_empty() {
        for n in caiu.into_iter().rev() {
            g.remove(n);
        }
        N.store(g.len(), Ordering::Relaxed);
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct RecArmArgs {
    /// Index of the track in `tracks` (it has to be `dmx` or `artnet`).
    pub track: usize,
    /// true arms, false disarms.
    #[serde(default = "sim")]
    pub on: bool,
}

fn sim() -> bool {
    true
}

pub fn register(r: &mut Registry) {
    r.add::<RecArmArgs>(
        "rec_arm",
        "Arms (or disarms) the recording of a dmx track: with the transport playing, the input universe becomes keyframes.",
        |a| arm(a.track, a.on),
    );
    r.add::<NoArgs>(
        "rec_state",
        "Tracks armed for recording in this process.",
        |_| Ok(state()),
    );
}

// Test in `engine/tests/rec.rs`, its own binary: arming queries the OPEN show, and `OPEN` is one
// per process — a test here would open a show and take down the other tests of the same binary.
