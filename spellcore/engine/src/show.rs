//! Show file `.spell` (JSON) v1 — the same file as `spellcaster/show.py`.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use std::path::Path;

pub const VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sacn {
    #[serde(default)]
    pub universes: Vec<u16>,
    #[serde(default = "cem")]
    pub priority: u8,
    #[serde(default = "spellcaster")]
    pub source_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interfaces: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtNet {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<Vec<String>>,
    #[serde(default = "sim")]
    pub broadcast: bool,
}

/// OSC output of the Player: `osc` tracks and `media` tracks on a non-Capture player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Osc {
    #[serde(default = "local")]
    pub host: String,
    /// With no "port" the output stays at 0 and the Player ignores it (Python raises KeyError).
    #[serde(default)]
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OutputCfg {
    Sacn(Sacn),
    ArtNet(ArtNet),
    Osc(Osc),
    // ponytail: an unknown type keeps only the name (the rest of the config is lost on save)
    // ; make it Unknown with a Map once laser/media land in the Rust `outputs`.
    #[serde(untagged)]
    Unknown {
        #[serde(rename = "type")]
        tipo: String,
    },
}

/// A known type with invalid content is an error (the derived `untagged` would turn it into
/// `Unknown` and the output would vanish from the show with no warning). With no "type" it
/// becomes `Unknown { tipo: "" }`, as in Python.
impl<'de> Deserialize<'de> for OutputCfg {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<OutputCfg, D::Error> {
        let v = Value::deserialize(d)?;
        let tipo = v
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        match tipo.as_str() {
            "sacn" => serde_json::from_value(v).map(OutputCfg::Sacn),
            "artnet" => serde_json::from_value(v).map(OutputCfg::ArtNet),
            "osc" => serde_json::from_value(v).map(OutputCfg::Osc),
            _ => return Ok(OutputCfg::Unknown { tipo }),
        }
        .map_err(|e| D::Error::custom(format!("output \"{}\": {}", tipo, e)))
    }
}

fn cem() -> u8 {
    100
}

fn spellcaster() -> String {
    "Spellcaster".to_string()
}

fn sim() -> bool {
    true
}

fn local() -> String {
    "127.0.0.1".to_string()
}

fn def_fps() -> u32 {
    30
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Show {
    #[serde(default)]
    pub name: String,
    #[serde(default = "def_fps")]
    pub fps: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(default)]
    pub outputs: Vec<OutputCfg>,
    #[serde(default)]
    pub tracks: Vec<Value>,
    #[serde(default)]
    pub version: u32,
    /// The rest of the file preserved as is (cues, transport, patch, ...).
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Show {
    fn default() -> Show {
        Show {
            name: String::new(),
            fps: def_fps(),
            duration: None,
            outputs: Vec::new(),
            tracks: Vec::new(),
            version: VERSION,
            extra: Map::new(),
        }
    }
}

/// v0 (a draft with no `version`) is raised to v1; version > 1 is an error.
pub fn migrate(mut sh: Show) -> Result<Show, String> {
    if sh.version > VERSION {
        return Err(format!(
            ".spell version {}: newer than this player (v{})",
            sh.version, VERSION
        ));
    }
    sh.version = VERSION;
    Ok(sh)
}

pub fn load(path: &Path) -> Result<Show, String> {
    let txt = std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let sh: Show = serde_json::from_str(&txt).map_err(|e| format!("{}: {}", path.display(), e))?;
    migrate(sh)
}

pub fn save(path: &Path, show: &Show) -> Result<(), String> {
    let mut out = show.clone();
    out.version = VERSION;
    out.extra.retain(|k, _| !k.starts_with('_')); // "_dir" and friends do not go to the file
                                                  // ponytail: serde indent 2 instead of the indent 1 of json.dump ; Python reads it the same and no
                                                  // test compares the text ; match it byte for byte only if the .spell lands in a git diff.
    let txt = serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?;
    std::fs::write(path, format!("{}\n", txt)).map_err(|e| format!("{}: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn higher_version_is_an_error() {
        let sh: Show = serde_json::from_str(r#"{"version":2}"#).unwrap();
        assert!(migrate(sh).is_err());
        let v0: Show = serde_json::from_str(r#"{"name":"x"}"#).unwrap();
        let v0 = migrate(v0).unwrap();
        assert_eq!(v0.version, VERSION);
        assert_eq!(v0.fps, 30);
    }

    #[test]
    fn known_and_unknown_outputs() {
        let sh: Show = serde_json::from_str(
            r#"{"version":1,"outputs":[{"type":"sacn","universes":[1,2]},
                                       {"type":"artnet","broadcast":false},
                                       {"type":"laser","pps":25000}]}"#,
        )
        .unwrap();
        assert_eq!(sh.outputs.len(), 3);
        match &sh.outputs[0] {
            OutputCfg::Sacn(c) => {
                assert_eq!(c.universes, vec![1u16, 2]);
                assert_eq!(c.priority, 100);
                assert_eq!(c.source_name, "Spellcaster");
            }
            o => panic!("expected sacn, got {:?}", o),
        }
        assert_eq!(
            sh.outputs[1],
            OutputCfg::ArtNet(ArtNet {
                targets: None,
                broadcast: false
            })
        );
        assert_eq!(
            sh.outputs[2],
            OutputCfg::Unknown {
                tipo: "laser".into()
            }
        );
    }

    /// The derived `untagged` used to swallow this: the show loaded with the output turned into
    /// `Unknown` and no universe went out at all.
    #[test]
    fn known_type_with_invalid_content_is_an_error() {
        for src in [
            r#"{"outputs":[{"type":"sacn","universes":"1"}]}"#,
            r#"{"outputs":[{"type":"sacn","universes":[1],"priority":"alto"}]}"#,
        ] {
            let e = serde_json::from_str::<Show>(src).unwrap_err().to_string();
            assert!(e.contains("sacn"), "error without the type: {}", e);
        }
        let sh: Show =
            serde_json::from_str(r#"{"outputs":[{"type":"laser","x":1},{"pps":25000}]}"#).unwrap();
        assert_eq!(
            sh.outputs,
            vec![
                OutputCfg::Unknown {
                    tipo: "laser".into()
                },
                OutputCfg::Unknown { tipo: "".into() },
            ]
        );
    }

    #[test]
    fn save_writes_no_null() {
        let sh: Show = serde_json::from_str(
            r#"{"outputs":[{"type":"sacn","universes":[1]},{"type":"artnet"}]}"#,
        )
        .unwrap();
        let txt = serde_json::to_string(&sh).unwrap();
        assert!(!txt.contains("null"), "{}", txt);
    }

    #[test]
    fn round_trip_preserves_extras() {
        let src = r#"{"name":"t","fps":44,"duration":2.5,
                      "outputs":[{"type":"sacn","universes":[7],"priority":50,"source_name":"S"}],
                      "tracks":[{"type":"dmx","universe":1,"address":1,"keys":[[0,0,"hold"]]}],
                      "cues":[],"transport":{"osc_port":9100},"_dir":"x","version":1}"#;
        let a: Show = migrate(serde_json::from_str(src).unwrap()).unwrap();
        let dir = std::env::temp_dir().join("spellcore_show_roundtrip.spell");
        save(&dir, &a).unwrap();
        let b = load(&dir).unwrap();
        std::fs::remove_file(&dir).ok();
        assert_eq!(b.name, "t");
        assert_eq!(b.fps, 44);
        assert_eq!(b.duration, Some(2.5));
        assert_eq!(b.outputs, a.outputs);
        assert_eq!(b.tracks, a.tracks);
        assert_eq!(b.extra.get("transport"), a.extra.get("transport"));
        assert!(b.extra.contains_key("cues"));
        assert!(!b.extra.contains_key("_dir"), "a key with _ is not written");
    }
}
