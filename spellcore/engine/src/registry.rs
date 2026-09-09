//! Registro de comandos: todo verbo do produto passa por aqui. CLI, OSC-API, GUI e MCP sao
//! clientes. Erro e' `String` — sem `anyhow` na R0.

use crate::player;
use crate::show;
use crate::timeline::Timeline;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;

pub struct Command {
    pub name: String,
    pub doc: String,
    pub schema: Value,
    pub f: Box<dyn Fn(Value) -> Result<Value, String> + Send + Sync>,
}

/// Vec na ordem de insercao (a CLI monta os subcomandos nessa ordem).
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
            None => Err(format!("comando desconhecido: {}", name)),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.cmds.iter()
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct LoadArgs {
    /// Caminho do arquivo .spell.
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct LocateArgs {
    /// Posicao em segundos.
    pub t: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct CueGoArgs {
    /// Indice da cue; ausente = a proxima.
    #[serde(default)]
    pub index: Option<usize>,
}

/// Comando sem parametro.
#[derive(Deserialize, JsonSchema)]
pub struct NoArgs {}

/// O player vivo neste processo, ou o erro que todo comando de transporte devolve sem ele.
fn vivo() -> Result<player::Handle, String> {
    player::current().ok_or_else(|| "sem player em execucao".to_string())
}

fn estado(h: &player::Handle) -> Result<Value, String> {
    serde_json::to_value(h.state()).map_err(|e| e.to_string())
}

/// Comandos do engine. `play_show` e `net` ficam na CLI (dependem de `script` e de varredura de
/// rede) e sao acrescentados de fora com `Registry::add`.
pub fn base() -> Registry {
    let mut r = Registry::new();
    r.add::<LoadArgs>("load", "Carrega um .spell e devolve nome, fps, duracao e tracks.", |a| {
        let sh = show::load(Path::new(&a.path))?;
        let tl = Timeline::new(&sh)?;
        Ok(json!({"name": sh.name, "fps": tl.fps, "duration": tl.duration,
                  "tracks": tl.tracks.len(), "ignored": tl.ignored()}))
    });
    r.add::<NoArgs>("pause", "Pausa o player em execucao neste processo.", |_| {
        let h = vivo()?;
        h.pause();
        estado(&h)
    });
    r.add::<NoArgs>("stop", "Para o player em execucao neste processo.", |_| {
        let h = vivo()?;
        h.stop();
        estado(&h)
    });
    r.add::<LocateArgs>("locate", "Salta o player para o instante t (segundos).", |a| {
        let h = vivo()?;
        h.locate(a.t);
        estado(&h)
    });
    r.add::<CueGoArgs>("cue_go", "Dispara a proxima cue (ou a de indice dado).", |a| {
        let h = vivo()?;
        h.cue_go(a.index);
        estado(&h)
    });
    r.add::<NoArgs>("transport_state", "Estado do transporte do player em execucao.", |_| {
        estado(&vivo()?)
    });
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize, JsonSchema)]
    struct Soma {
        a: i64,
        b: i64,
    }

    #[test]
    fn add_call_schema() {
        let mut r = Registry::new();
        r.add::<Soma>("soma", "Soma a + b.", |s| Ok(json!(s.a + s.b)));
        assert_eq!(r.call("soma", json!({"a": 2, "b": 3})).unwrap(), json!(5));
        assert!(r.call("soma", json!({"a": 2})).is_err(), "faltando b");
        assert!(r.call("nao_existe", json!({})).is_err());

        let sc = r.schema();
        let c = &sc.as_array().unwrap()[0];
        assert_eq!(c["name"], "soma");
        assert_eq!(c["doc"], "Soma a + b.");
        assert!(c["params"]["properties"]["a"].is_object(), "schema: {}", c["params"]);
        assert_eq!(r.get("soma").unwrap().name, "soma");
        assert_eq!(r.iter().count(), 1);
    }

    /// Sem player vivo, todo comando de transporte devolve o mesmo erro; `load` continua livre.
    #[test]
    fn base_tem_transporte_e_load() {
        let r = base();
        for c in ["load", "pause", "stop", "locate", "cue_go", "transport_state"] {
            assert!(r.get(c).is_some(), "comando {} ausente", c);
        }
        // ponytail: o teste so' vale quando nao ha player neste processo — os testes do player
        // sobem o seu em outro binario (tests/player.rs), entao aqui nunca ha CURRENT.
        for (c, a) in [
            ("pause", json!({})),
            ("stop", json!({})),
            ("locate", json!({"t": 3.5})),
            ("cue_go", json!({})),
            ("transport_state", json!({})),
        ] {
            assert_eq!(
                r.call(c, a).unwrap_err(),
                "sem player em execucao",
                "comando {}",
                c
            );
        }
        assert!(r.call("load", json!({"path": "nao_existe.spell"})).is_err());
    }
}
