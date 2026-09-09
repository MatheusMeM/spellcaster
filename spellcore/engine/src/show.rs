//! Arquivo de show `.spell` (JSON) v1 — mesmo arquivo do `spellcaster/show.py`.

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Map, Value};
use std::path::Path;

pub const VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq)]
pub enum OutputCfg {
    Sacn {
        universes: Vec<u16>,
        priority: u8,
        source_name: String,
        interfaces: Option<Vec<String>>,
    },
    ArtNet {
        targets: Option<Vec<String>>,
        broadcast: bool,
    },
    // ponytail: tipo desconhecido guarda so' o nome (o resto da config se perde ao regravar)
    // ; virar Unknown(String, Map) quando laser/media entrarem no `outputs` do Rust.
    Unknown(String),
}

impl<'de> Deserialize<'de> for OutputCfg {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<OutputCfg, D::Error> {
        let v = Value::deserialize(d)?;
        let ty = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
        let list = |k: &str| -> Option<Vec<String>> {
            v.get(k).and_then(|x| x.as_array()).map(|a| {
                a.iter()
                    .map(|s| s.as_str().unwrap_or_default().to_string())
                    .collect()
            })
        };
        Ok(match ty {
            "sacn" => OutputCfg::Sacn {
                universes: v
                    .get("universes")
                    .and_then(|x| x.as_array())
                    .map(|a| a.iter().filter_map(|n| n.as_u64()).map(|n| n as u16).collect())
                    .unwrap_or_default(),
                priority: v.get("priority").and_then(|x| x.as_u64()).unwrap_or(100) as u8,
                source_name: v
                    .get("source_name")
                    .and_then(|x| x.as_str())
                    .unwrap_or("Spellcaster")
                    .to_string(),
                interfaces: list("interfaces"),
            },
            "artnet" => OutputCfg::ArtNet {
                targets: list("targets"),
                broadcast: v.get("broadcast").and_then(|x| x.as_bool()).unwrap_or(true),
            },
            "" => return Err(D::Error::custom("output sem \"type\"")),
            other => OutputCfg::Unknown(other.to_string()),
        })
    }
}

impl Serialize for OutputCfg {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            OutputCfg::Sacn {
                universes,
                priority,
                source_name,
                interfaces,
            } => {
                let mut m = json!({"type": "sacn", "universes": universes,
                                   "priority": priority, "source_name": source_name});
                if let Some(i) = interfaces {
                    m["interfaces"] = json!(i);
                }
                m.serialize(s)
            }
            OutputCfg::ArtNet { targets, broadcast } => {
                json!({"type": "artnet", "targets": targets, "broadcast": broadcast}).serialize(s)
            }
            OutputCfg::Unknown(t) => json!({ "type": t }).serialize(s),
        }
    }
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
    /// Resto do arquivo preservado tal e qual (cues, transport, patch, ...).
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

/// v0 (rascunho sem `version`) sobe para v1; version > 1 e' erro.
pub fn migrate(mut sh: Show) -> Result<Show, String> {
    if sh.version > VERSION {
        return Err(format!(
            ".spell versao {}: mais novo que este player (v{})",
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
    out.extra.retain(|k, _| !k.starts_with('_')); // "_dir" e afins nao vao para o arquivo
    // ponytail: indent 2 do serde em vez do indent 1 do json.dump ; o Python le igual e nenhum
    // teste compara o texto ; casar byte a byte so' se o .spell entrar em diff de git.
    let txt = serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?;
    std::fs::write(path, format!("{}\n", txt)).map_err(|e| format!("{}: {}", path.display(), e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_maior_e_erro() {
        let sh: Show = serde_json::from_str(r#"{"version":2}"#).unwrap();
        assert!(migrate(sh).is_err());
        let v0: Show = serde_json::from_str(r#"{"name":"x"}"#).unwrap();
        let v0 = migrate(v0).unwrap();
        assert_eq!(v0.version, VERSION);
        assert_eq!(v0.fps, 30);
    }

    #[test]
    fn outputs_conhecidos_e_desconhecido() {
        let sh: Show = serde_json::from_str(
            r#"{"version":1,"outputs":[{"type":"sacn","universes":[1,2]},
                                       {"type":"artnet","broadcast":false},
                                       {"type":"laser","pps":25000}]}"#,
        )
        .unwrap();
        assert_eq!(sh.outputs.len(), 3);
        match &sh.outputs[0] {
            OutputCfg::Sacn { universes, priority, source_name, .. } => {
                assert_eq!(universes, &vec![1u16, 2]);
                assert_eq!(*priority, 100);
                assert_eq!(source_name, "Spellcaster");
            }
            o => panic!("esperava sacn, veio {:?}", o),
        }
        assert_eq!(sh.outputs[1], OutputCfg::ArtNet { targets: None, broadcast: false });
        assert_eq!(sh.outputs[2], OutputCfg::Unknown("laser".into()));
    }

    #[test]
    fn ida_e_volta_preserva_extras() {
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
        assert!(!b.extra.contains_key("_dir"), "chave com _ nao e' gravada");
    }
}
