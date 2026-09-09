//! Arquivo de show `.spell` (JSON) v1 — mesmo arquivo do `spellcaster/show.py`.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::Path;

pub const VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OutputCfg {
    Sacn {
        #[serde(default)]
        universes: Vec<u16>,
        #[serde(default = "cem")]
        priority: u8,
        #[serde(default = "spellcaster")]
        source_name: String,
        #[serde(default)]
        interfaces: Option<Vec<String>>,
    },
    ArtNet {
        #[serde(default)]
        targets: Option<Vec<String>>,
        #[serde(default = "sim")]
        broadcast: bool,
    },
    /// Saida OSC do Player: tracks `osc` e `media` de reprodutor nao-Capture.
    Osc {
        #[serde(default = "local")]
        host: String,
        /// Sem "port" a saida fica em 0 e o Player a ignora (o Python levanta KeyError).
        #[serde(default)]
        port: u16,
    },
    // ponytail: tipo desconhecido guarda so' o nome (o resto da config se perde ao regravar)
    // ; virar Unknown com Map quando laser/media entrarem no `outputs` do Rust.
    // O `untagged` e' o ultimo braco: pega o "type" que nao e' nenhum dos de cima e tambem o
    // output sem "type" nenhum (vira Unknown { tipo: "" } em vez de erro de parse).
    #[serde(untagged)]
    Unknown {
        #[serde(default, rename = "type")]
        tipo: String,
    },
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
        assert_eq!(sh.outputs[2], OutputCfg::Unknown { tipo: "laser".into() });
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
