//! MIDI mapping: one open input port, a `key -> command` map that lives in the `.spell`, and the
//! event delivered to the graph through the SAME queue as the `input` command.
//!
//! Event key: `"<status>/<data1>"` (`144/60` = note on channel 1 note 60, `176/1` = CC 1), the
//! same one as the graph `in.midi` node. Each event received:
//!
//!   1. becomes `input {key: "midi:<key>", value}` on the hooks of the live player (the `in.midi`);
//!   2. if the key is in the map, it calls the command through the registry — the same path as WS.
//!
//! `value` is `data2 / 127` (0..1). In the map `args`, `"$"` becomes that value and `"$<n>"`
//! becomes `round(value * n)`: `"$127"` is the raw MIDI byte and `"$255"` is the DMX level.
//!
//! What drains the queue is `pump()`, called at the point of the frame where `input` is already
//! consumed (`player::Rt::drain`) — the frame order does not change. `midi_last` and `midi_learn`
//! also drain, so the GUI LEARN works with no show playing.

use crate::registry::{lock, NoArgs, Registry, OPEN};
use crate::show::Show;
use protocols::midi::MidiIn;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// The port open in this process.
// ponytail: one port per process ; make it a table like the laser `FEEDS` once someone plugs a
// keyboard and a control surface at the same time (the key does not tell the ports apart).
static IN: Mutex<Option<MidiIn>> = Mutex::new(None);

/// Last event received: (key, value 0..1, raw byte). It is what the GUI LEARN reads.
static LAST: Mutex<Option<(String, f64, u8)>> = Mutex::new(None);

/// Event counter: `midi_learn` waits for this number to change instead of fighting the frame for the queue.
static SEQ: AtomicU64 = AtomicU64::new(0);

/// How long `midi_learn` waits for the next key.
const LEARN_MS: u64 = 5000;

// ---------------------------------------------------------------- registry of the map

/// How to build the registry the map calls. The CLI installs its own (with `play_show`, `net` and
/// the `laser_*`); with nothing installed, `registry::base()`.
static BUILD: Mutex<Option<fn() -> Registry>> = Mutex::new(None);

pub fn builder(f: fn() -> Registry) {
    *lock(&BUILD) = Some(f);
}

fn reg() -> &'static Registry {
    static R: OnceLock<Registry> = OnceLock::new();
    R.get_or_init(|| match *lock(&BUILD) {
        Some(f) => f(),
        None => crate::registry::base(),
    })
}

// ------------------------------------------------------------------------- map

/// The map of the open show, or empty.
fn mapa() -> Map<String, Value> {
    lock(&OPEN)
        .as_ref()
        .and_then(|(_, sh)| sh.extra.get("midi"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

/// What the key fires: (command, raw args). It reads and RELEASES the show before any call —
/// the mapped command may be one that edits the show. Public so `tests/midi.rs` can prove that
/// what `midi_map` writes is what the event reads.
pub fn liga(key: &str) -> Option<(String, Value)> {
    let e = lock(&OPEN)
        .as_ref()
        .and_then(|(_, sh)| sh.extra.get("midi"))
        .and_then(|m| m.get(key))
        .cloned()?;
    let cmd = e.get("cmd").and_then(Value::as_str)?.to_string();
    let args = e.get("args").cloned().unwrap_or_else(|| json!({}));
    Some((cmd, args))
}

/// `"$"` becomes the value 0..1; `"$<n>"` becomes `round(value * n)` (`"$127"` = the raw MIDI
/// byte, `"$255"` = DMX level). Recursive: it holds inside a list and inside an object.
///
/// `"$<n>"` comes out as an INTEGER, not a float: command `address` and `index` are `u16`/`usize`,
/// and serde refuses `100.0` where it wants an integer (a float goes into `f64` without a
/// complaint, the other way round it does not).
pub fn expande(v: &Value, valor: f64) -> Value {
    match v {
        Value::String(s) => match s.strip_prefix('$') {
            Some("") => json!(valor),
            Some(n) => match n.parse::<f64>() {
                Ok(n) => json!((valor * n).round() as i64),
                Err(_) => v.clone(),
            },
            None => v.clone(),
        },
        Value::Array(a) => Value::Array(a.iter().map(|x| expande(x, valor)).collect()),
        Value::Object(o) => Value::Object(
            o.iter()
                .map(|(k, x)| (k.clone(), expande(x, valor)))
                .collect(),
        ),
        _ => v.clone(),
    }
}

/// Valid key: `<status 128..239>/<data1 0..127>`. The ceiling is 239 (0xEF) because only a
/// CHANNEL message becomes an event (`protocols::midi::canal` refuses 0xF0..0xFF, which is system
/// common and realtime): accepting `240/0` would accept a key that never fires.
fn chave_ok(k: &str) -> Result<(), String> {
    let erro = || format!("key \"{}\": expected <status>/<data1>, e.g. 144/60", k);
    let (s, d) = k.split_once('/').ok_or_else(erro)?;
    let s: u16 = s.parse().map_err(|_| erro())?;
    let d: u16 = d.parse().map_err(|_| erro())?;
    if !(128..240).contains(&s) || d > 127 {
        return Err(erro());
    }
    Ok(())
}

// ------------------------------------------------------------------------ pump

/// Drains the driver queue. Each event goes to the hooks of the live player and, if it is in the
/// map, to the registry.
// ponytail: the mapped command runs on the thread that called the pump (the frame, in the normal
// case) ; make it a thread if someone maps a slow command (`net` scans the network, `play_show`
// plays the whole show).
pub fn pump() {
    loop {
        // take ONE event and release the port: `midi_close` wants this lock too.
        let e = {
            let g = lock(&IN);
            match g.as_ref() {
                Some(m) => m.try_recv(),
                None => return,
            }
        };
        match e {
            Some((status, d1, d2)) => entrega(status, d1, d2),
            None => return,
        }
    }
}

/// One event: it becomes `input {key: "midi:<key>"}` on the hooks of the live player and, if the
/// key is in the map, a command call through the registry. Public because it is the whole MIDI
/// path and the test needs it with no hardware (on Windows there is no virtual port to inject a note).
pub fn entrega(status: u8, d1: u8, d2: u8) {
    let key = format!("{}/{}", status, d1);
    let valor = d2 as f64 / 127.0;
    *lock(&LAST) = Some((key.clone(), valor, d2));
    SEQ.fetch_add(1, Ordering::Relaxed);
    if let Some(h) = crate::player::current() {
        h.input(&format!("midi:{}", key), valor);
    }
    if let Some((cmd, args)) = liga(&key) {
        if let Err(e) = reg().call(&cmd, expande(&args, valor)) {
            eprintln!("midi {} -> {}: {}", key, cmd, e);
        }
    }
}

/// Opens the port (name, part of the name or index as text) and returns the real name.
fn abrir(port: &str) -> Result<String, String> {
    let m = MidiIn::open(port)?;
    let n = m.name().to_string();
    *lock(&IN) = Some(m);
    Ok(n)
}

/// `"midi_port": "<name>"` in the `.spell`: it reconnects the control surface when the show
/// starts. A failure becomes a warning — a show does not stop because the controller stayed in
/// its case.
pub fn auto(sh: &Show) {
    let Some(p) = sh
        .extra
        .get("midi_port")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    else {
        return;
    };
    if lock(&IN).as_ref().is_some_and(|m| m.name() == p) {
        return; // already open on this port
    }
    match abrir(p) {
        Ok(n) => eprintln!("midi: {}", n),
        Err(e) => eprintln!("warning: midi_port \"{}\": {}", p, e),
    }
}

fn aberta() -> Value {
    match lock(&IN).as_ref() {
        Some(m) => json!(m.name()),
        None => Value::Null,
    }
}

fn ultima() -> Value {
    match lock(&LAST).as_ref() {
        Some((k, v, r)) => {
            json!({"key": k, "value": v, "raw": r, "seq": SEQ.load(Ordering::Relaxed)})
        }
        None => json!({"key": Value::Null, "value": 0.0, "raw": 0, "seq": 0}),
    }
}

// ---------------------------------------------------------------------- commands

#[derive(Deserialize, JsonSchema)]
pub struct PortArgs {
    /// Port name (or part of it), index as text ("0"), or empty = the first one.
    #[serde(default)]
    pub port: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct MapArgs {
    /// Event key: "<status>/<data1>" (144/60 = note on note 60, 176/1 = CC 1).
    pub key: String,
    /// Name of the registry command.
    pub cmd: String,
    /// Arguments of the command. "$" becomes the value 0..1 and "$<n>" becomes round(value * n).
    #[serde(default)]
    pub args: Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct KeyArgs {
    /// Event key to remove.
    pub key: String,
}

pub fn register(r: &mut Registry) {
    r.add::<NoArgs>(
        "midi_ports",
        "Lists the MIDI input ports of the machine and which one is open.",
        |_| Ok(json!({"ports": protocols::midi::ports(), "open": aberta()})),
    );
    r.add::<PortArgs>(
        "midi_open",
        "Opens a MIDI input port (name, part of the name or index) and writes midi_port into the open show.",
        |a| {
            let n = abrir(&a.port)?;
            crate::edit::com(|_, sh| {
                sh.extra.insert("midi_port".into(), json!(n));
                Ok(json!({"open": n}))
            })
        },
    );
    r.add::<NoArgs>(
        "midi_close",
        "Closes the MIDI port and removes midi_port from the open show.",
        |_| {
            *lock(&IN) = None;
            crate::edit::com(|_, sh| {
                sh.extra.remove("midi_port");
                Ok(json!({"open": Value::Null}))
            })
        },
    );
    r.add::<MapArgs>(
        "midi_map",
        "Binds a key/CC to a command in the open show. Returns the map.",
        |a| {
            chave_ok(&a.key)?;
            if reg().get(&a.cmd).is_none() {
                return Err(format!("unknown command: {}", a.cmd));
            }
            let args = match a.args {
                Value::Null => json!({}),
                // text that is JSON becomes JSON (form field, CLI), like `valor` in edit
                Value::String(s) => serde_json::from_str(&s).map_err(|e| format!("args: {}", e))?,
                v => v,
            };
            if !args.is_object() {
                return Err("args: expected a JSON object".into());
            }
            crate::edit::com(|_, sh| {
                let m = sh
                    .extra
                    .entry("midi")
                    .or_insert_with(|| json!({}))
                    .as_object_mut()
                    .ok_or("midi: the show key \"midi\" is not an object")?;
                m.insert(a.key.clone(), json!({"cmd": a.cmd, "args": args}));
                Ok(Value::Object(m.clone()))
            })
        },
    );
    r.add::<KeyArgs>("midi_unmap", "Unbinds a key from the open show.", |a| {
        crate::edit::com(|_, sh| {
            let m = sh
                .extra
                .get_mut("midi")
                .and_then(Value::as_object_mut)
                .ok_or("midi: the show has no map")?;
            m.remove(&a.key)
                .ok_or_else(|| format!("key {} is not in the map", a.key))?;
            Ok(Value::Object(m.clone()))
        })
    });
    r.add::<NoArgs>(
        "midi_maps",
        "The key -> command map of the open show.",
        |_| Ok(Value::Object(mapa())),
    );
    r.add::<NoArgs>(
        "midi_last",
        "Last MIDI key received: {key, value, raw, seq}. It is what the GUI shows live.",
        |_| {
            pump();
            Ok(ultima())
        },
    );
    r.add::<NoArgs>(
        "midi_learn",
        "Waits up to 5 s for the next MIDI key and returns its key.",
        |_| {
            let base = SEQ.load(Ordering::Relaxed);
            let fim = Instant::now() + Duration::from_millis(LEARN_MS);
            while Instant::now() < fim {
                pump(); // with no player running, nobody else drains the queue
                if SEQ.load(Ordering::Relaxed) != base {
                    return Ok(ultima());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(format!("no MIDI event in {} s", LEARN_MS / 1000))
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_dollar() {
        let v = json!({"universe": 1, "address": 3, "values": ["$255"], "u": "$", "n": "$127"});
        let e = expande(&v, 100.0 / 127.0);
        assert_eq!(e["universe"], json!(1), "a number passes through unchanged");
        assert_eq!(e["values"], json!([201]), "$255 = DMX level");
        assert_eq!(e["n"], json!(100), "$127 = the raw MIDI byte");
        assert!(
            e["n"].is_i64(),
            "$<n> comes out integer, or `u16` refuses it"
        );
        assert!((e["u"].as_f64().unwrap() - 100.0 / 127.0).abs() < 1e-12);
        // zero and full at both ends
        assert_eq!(expande(&json!("$255"), 0.0), json!(0));
        assert_eq!(expande(&json!("$255"), 1.0), json!(255));
        assert_eq!(expande(&json!("$127"), 1.0), json!(127));
        // text that is not a dollar stays as it is
        assert_eq!(expande(&json!("$x"), 0.5), json!("$x"));
        assert_eq!(expande(&json!("go"), 0.5), json!("go"));
        assert_eq!(expande(&json!(true), 0.5), json!(true));
    }

    #[test]
    fn valid_key() {
        for k in ["144/60", "128/0", "176/1", "239/127"] {
            assert!(chave_ok(k).is_ok(), "{}", k);
        }
        // 240 (0xF0) and up is system common/realtime: `protocols::midi::canal` does not emit it
        for k in [
            "", "144", "60/144", "144/128", "x/1", "144/", "-1/1", "240/0", "255/127", "256/1",
        ] {
            assert!(chave_ok(k).is_err(), "{}", k);
        }
    }
}
