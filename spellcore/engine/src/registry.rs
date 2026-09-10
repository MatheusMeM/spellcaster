//! Command registry: every verb of the product goes through here. CLI, OSC-API, GUI and MCP are
//! clients. An error is a `String` — no `anyhow` in R0.

use crate::player;
use crate::show;
use crate::timeline::Timeline;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Mutex;

pub struct Command {
    pub name: String,
    pub doc: String,
    pub schema: Value,
    pub f: Box<dyn Fn(Value) -> Result<Value, String> + Send + Sync>,
}

/// Vec in insertion order (the CLI builds the subcommands in that order).
pub struct Registry {
    cmds: Vec<Command>,
}

impl Default for Registry {
    fn default() -> Self {
        Registry::new()
    }
}

impl Registry {
    pub fn new() -> Registry {
        Registry { cmds: Vec::new() }
    }

    pub fn add<A: JsonSchema + DeserializeOwned>(
        &mut self,
        name: &str,
        doc: &str,
        f: impl Fn(A) -> Result<Value, String> + Send + Sync + 'static,
    ) {
        let schema = serde_json::to_value(schemars::schema_for!(A)).unwrap_or(Value::Null);
        self.cmds.push(Command {
            name: name.to_string(),
            doc: doc.to_string(),
            schema,
            f: Box::new(move |v: Value| {
                let a: A = serde_json::from_value(v).map_err(|e| e.to_string())?;
                f(a)
            }),
        });
    }

    pub fn get(&self, name: &str) -> Option<&Command> {
        self.cmds.iter().find(|c| c.name == name)
    }

    pub fn schema(&self) -> Value {
        Value::Array(
            self.cmds
                .iter()
                .map(|c| json!({"name": c.name, "doc": c.doc, "params": c.schema}))
                .collect(),
        )
    }

    pub fn call(&self, name: &str, args: Value) -> Result<Value, String> {
        match self.get(name) {
            Some(c) => (c.f)(args),
            None => Err(format!("unknown command: {}", name)),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.cmds.iter()
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct LoadArgs {
    /// Path of the .spell file (the same `file` as show_get, show_save and play_show).
    // ponytail: `path` was the name of this argument and stays accepted for one round, for
    // scripts and MCP sessions already written (no caller left in the repo) ; drop the alias in
    // round 3.
    #[serde(alias = "path")]
    pub file: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct LocateArgs {
    /// Position in seconds.
    pub t: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct LoopArgs {
    /// true turns the loop on, false turns it off.
    pub on: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct CueGoArgs {
    /// Index of the cue; absent = the next one.
    #[serde(default)]
    pub index: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ShowGetArgs {
    /// Path of the .spell to open; empty = the last one opened in this process.
    #[serde(default)]
    pub file: String,
    /// true = the whole .spell (what the GUI draws) instead of the summary.
    #[serde(default)]
    pub full: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct InputArgs {
    /// Event key: "widget:go", "key:Space", "osc:/spell/go", "module:laser/stat/fps".
    pub key: String,
    /// Event value; a trigger sends 1.
    #[serde(default)]
    pub value: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct InputGetArgs {
    /// Universe declared in `show.inputs`.
    #[serde(default = "um")]
    pub universe: u16,
}

fn um() -> u16 {
    1
}

/// Command with no parameter.
#[derive(Deserialize, JsonSchema)]
pub struct NoArgs {}

/// Last .spell opened in this process (the `OPEN` of `spellcaster/mcp/tools.py`): (path, show).
// ponytail: one open show per process, written by `load`, `show_get` and the `edit` commands
// ; make it a session id once the GUI opens two shows at the same time. `play_show` does NOT
// write here (it lives in the CLI, which does not see this state): after a play, `show_get`
// still asks for `file`.
pub(crate) static OPEN: Mutex<Option<(String, show::Show)>> = Mutex::new(None);

pub(crate) fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Path of the .spell open in this process; empty when the show exists only in memory. It is the
/// base of the files the show names — the `clip` of a laser track resolves against its folder,
/// like the `_load_clip` of `spellcaster/player/player.py`.
pub fn open_path() -> String {
    lock(&OPEN)
        .as_ref()
        .map(|(p, _)| p.clone())
        .unwrap_or_default()
}

/// Summary of the show for the AI and for the `spell://show` resource (the same fields as the
/// `summary()` of `spellcaster/mcp/tools.py`), plus the live transport when a player is running.
fn resumo(file: &str, sh: &show::Show) -> Value {
    let outputs: Vec<String> = sh
        .outputs
        .iter()
        .map(|o| {
            serde_json::to_value(o)
                .ok()
                .and_then(|v| v["type"].as_str().map(str::to_string))
                .unwrap_or_default()
        })
        .collect();
    let campo = |t: &Value, ks: [&str; 3]| {
        ks.iter()
            .find_map(|k| t.get(*k).filter(|v| !v.is_null()).cloned())
            .unwrap_or(Value::Null)
    };
    let tracks: Vec<Value> = sh
        .tracks
        .iter()
        .map(|t| {
            json!({"type": t.get("type").cloned().unwrap_or(Value::Null),
                   "name": campo(t, ["name", "fixture", "script"]),
                   "universe": t.get("universe").cloned().unwrap_or(json!(1))})
        })
        .collect();
    let cues: Vec<Value> = sh
        .extra
        .get("cues")
        .and_then(|c| c.as_array())
        .map(|a| {
            a.iter()
                .map(|c| {
                    json!({"name": c.get("name").cloned().unwrap_or(Value::Null),
                           "fade": c.get("fade").cloned().unwrap_or(json!(0.0)),
                           "wait": c.get("wait").cloned().unwrap_or(json!(0.0)),
                           "follow": c.get("follow").and_then(|v| v.as_bool()).unwrap_or(false)})
                })
                .collect()
        })
        .unwrap_or_default();
    let patch: Vec<Value> = sh
        .extra
        .get("patch")
        .and_then(|p| p.as_array())
        .map(|a| a.iter().map(|f| f["name"].clone()).collect())
        .unwrap_or_default();
    let transport = player::current()
        .and_then(|h| serde_json::to_value(h.state()).ok())
        .unwrap_or(Value::Null);
    json!({"aberto": true, "name": sh.name, "file": file, "fps": sh.fps, "duration": sh.duration,
           "outputs": outputs, "patch": patch, "tracks": tracks, "cues": cues,
           "transport": transport})
}

/// The live player in this process, or the error every transport command returns without it.
pub(crate) fn vivo() -> Result<player::Handle, String> {
    player::current().ok_or_else(|| "no player running".to_string())
}

fn estado(h: &player::Handle) -> Result<Value, String> {
    serde_json::to_value(h.state()).map_err(|e| e.to_string())
}

/// Engine commands. `play_show` and `net` live in the CLI (they depend on `script` and on the
/// network scan) and are added from outside with `Registry::add`.
pub fn base() -> Registry {
    let mut r = Registry::new();
    r.add::<LoadArgs>(
        "load",
        "Opens a .spell AND VALIDATES the timeline (which show_get{file} does not): returns name, fps, duration, how many tracks and which ones were ignored. The `path` argument is the old name of `file` (deprecated, gone next round).",
        |a| {
            let sh = show::load(Path::new(&a.file))?;
            let tl = Timeline::new(&sh)?;
            let out = json!({"name": sh.name, "fps": tl.fps, "duration": tl.duration,
                         "tracks": tl.tracks.len(), "ignored": tl.ignored()});
            crate::edit::abre(a.file.clone(), sh);
            Ok(out)
        },
    );
    r.add::<ShowGetArgs>(
        "show_get",
        "Summary of the open .spell (or of the given file): name, fps, duration, outputs, patch, tracks, cues. full=true returns the whole .spell.",
        |a| {
            if !a.file.is_empty() {
                let sh = show::load(Path::new(&a.file))?;
                crate::edit::abre(a.file.clone(), sh);
            }
            match &*lock(&OPEN) {
                Some((_, sh)) if a.full => serde_json::to_value(sh).map_err(|e| e.to_string()),
                Some((f, sh)) => Ok(resumo(f, sh)),
                None => Ok(json!({"aberto": false,
                                  "dica": "call show_get with file=<path.spell>"})),
            }
        },
    );
    // The pair of `pause`: without it, whoever paused through the registry (GUI, MCP, OSC) could
    // only play again by starting another player with `play_show`, and `serve --show`, which
    // leaves the player stopped at t=0, would have no way to release it. It is called `resume`
    // and not `play` because `play` is the CLI subcommand that STARTS a player (the registry
    // `play_show`); nothing is started here.
    r.add::<NoArgs>(
        "resume",
        "Resumes the player paused in this process (the pair of pause).",
        |_| {
            let h = vivo()?;
            h.play();
            estado(&h)
        },
    );
    r.add::<NoArgs>(
        "pause",
        "Pauses the player running in this process.",
        |_| {
            let h = vivo()?;
            h.pause();
            estado(&h)
        },
    );
    r.add::<NoArgs>("stop", "Stops the player running in this process.", |_| {
        let h = vivo()?;
        h.stop();
        estado(&h)
    });
    r.add::<LocateArgs>("locate", "Jumps the player to instant t (seconds).", |a| {
        let h = vivo()?;
        h.locate(a.t);
        estado(&h)
    });
    // The loop belongs to the PLAYER, over the In-Out range of the OPEN show (`edit::intervalo`):
    // whoever drags the In in the GUI and calls `loop_set` again already repeats over the new
    // range. With no In/Out, 0..duration.
    r.add::<LoopArgs>(
        "loop_set",
        "Turns the player loop on or off over the In-Out range of the open show.",
        |a| {
            let h = vivo()?;
            let (i, o) = crate::edit::intervalo();
            h.set_loop(a.on, i, o);
            estado(&h)
        },
    );
    r.add::<CueGoArgs>(
        "cue_go",
        "Fires the next cue (or the one at the given index).",
        |a| {
            let h = vivo()?;
            h.cue_go(a.index);
            estado(&h)
        },
    );
    r.add::<NoArgs>(
        "transport_state",
        "Transport state of the running player.",
        |_| estado(&vivo()?),
    );
    r.add::<InputArgs>(
        "input",
        "Delivers an input event to the hooks of the live player (the Graph): key + value.",
        |a| {
            vivo()?.input(&a.key, a.value);
            Ok(json!({"key": a.key, "value": a.value}))
        },
    );
    r.add::<InputGetArgs>(
        "input_get",
        "Last DMX frame received on the INPUT universe (show.inputs): 512 values.",
        |a| {
            let d = vivo()?
                .input_get(a.universe)
                .ok_or_else(|| format!("universe {}: no declared input or no frame", a.universe))?;
            Ok(json!({"universe": a.universe, "data": d.to_vec()}))
        },
    );
    crate::edit::register(&mut r);
    crate::module::register(&mut r);
    crate::rec::register(&mut r);
    crate::midi::register(&mut r);
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize, JsonSchema)]
    struct Sum {
        a: i64,
        b: i64,
    }

    #[test]
    fn add_call_schema() {
        let mut r = Registry::new();
        r.add::<Sum>("sum", "Adds a + b.", |s| Ok(json!(s.a + s.b)));
        assert_eq!(r.call("sum", json!({"a": 2, "b": 3})).unwrap(), json!(5));
        assert!(r.call("sum", json!({"a": 2})).is_err(), "b is missing");
        assert!(r.call("does_not_exist", json!({})).is_err());

        let sc = r.schema();
        let c = &sc.as_array().unwrap()[0];
        assert_eq!(c["name"], "sum");
        assert_eq!(c["doc"], "Adds a + b.");
        assert!(
            c["params"]["properties"]["a"].is_object(),
            "schema: {}",
            c["params"]
        );
        assert_eq!(r.get("sum").unwrap().name, "sum");
        assert_eq!(r.iter().count(), 1);
    }

    /// With no live player, every transport command returns the same error; `load` stays free.
    #[test]
    fn base_has_transport_and_load() {
        let r = base();
        let esperados = [
            "load",
            "show_get",
            "resume",
            "pause",
            "stop",
            "locate",
            "loop_set",
            "cue_go",
            "transport_state",
            "input",
            "input_get",
            "rec_arm",
            "rec_state",
        ];
        for c in esperados {
            assert!(r.get(c).is_some(), "command {} missing", c);
        }
        // ponytail: the test only holds when there is no player in this process — the player
        // tests start theirs in another binary (tests/player.rs), so there is never a CURRENT here.
        for (c, a) in [
            ("resume", json!({})),
            ("pause", json!({})),
            ("stop", json!({})),
            ("locate", json!({"t": 3.5})),
            ("loop_set", json!({"on": true})),
            ("cue_go", json!({})),
            ("transport_state", json!({})),
            ("input", json!({"key": "widget:go", "value": 1.0})),
            ("input_get", json!({"universe": 1})),
        ] {
            assert_eq!(
                r.call(c, a).unwrap_err(),
                "no player running",
                "command {}",
                c
            );
        }
        assert!(r
            .call("load", json!({"file": "does_not_exist.spell"}))
            .is_err());
        // the deprecated alias still gets in: a file error, not a missing-argument one
        assert!(r
            .call("load", json!({"path": "does_not_exist.spell"}))
            .unwrap_err()
            .contains("does_not_exist.spell"));
        assert!(
            r.call("load", json!({})).is_err(),
            "no file and no path: deserialization error"
        );
    }

    /// `show_get` with no open show warns; with `file` it opens, summarizes and stays open for
    /// the next call (it is what feeds the MCP `spell://show` resource).
    #[test]
    fn show_get_opens_and_remembers() {
        let r = base();
        assert_eq!(
            r.call("show_get", json!({})).unwrap()["aberto"],
            json!(false)
        );
        let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../../shows/medgrupo.spell");
        let d = r.call("show_get", json!({ "file": p })).unwrap();
        assert_eq!(d["aberto"], json!(true));
        assert_eq!(d["fps"], json!(30));
        assert!(d["file"].as_str().unwrap().ends_with("medgrupo.spell"));
        assert!(!d["tracks"].as_array().unwrap().is_empty());
        assert!(d["outputs"].as_array().unwrap().iter().any(|o| o == "sacn"));
        assert_eq!(d["transport"], Value::Null, "no player in this test binary");
        // with no `file`, it returns the same show
        assert_eq!(r.call("show_get", json!({})).unwrap(), d);
        assert!(r
            .call("show_get", json!({"file": "does_not_exist.spell"}))
            .is_err());
    }
}
