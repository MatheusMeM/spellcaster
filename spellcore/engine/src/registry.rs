//! Registro de comandos: todo verbo do produto passa por aqui. CLI, OSC-API, GUI e MCP sao
//! clientes. Erro e' `String` — sem `anyhow` na R0.

use crate::clock::Clock;
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
    pub mcp: bool,
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
        // ponytail: mcp = true para todo comando ; passar a flag quando existir verbo que a GUI
        // usa mas o MCP nao pode chamar (ex.: shutdown).
        let schema = serde_json::to_value(schemars::schema_for!(A)).unwrap_or(Value::Null);
        self.cmds.push(Command {
            name: name.to_string(),
            doc: doc.to_string(),
            schema,
            mcp: true,
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
                .map(|c| json!({"name": c.name, "doc": c.doc, "params": c.schema, "mcp": c.mcp}))
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

    pub fn len(&self) -> usize {
        self.cmds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cmds.is_empty()
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

/// Comandos do engine que nao precisam de rede. `play` e `net` ficam na CLI (dependem de
/// `protocols`) e sao acrescentados de fora com `Registry::add`.
pub fn base(clock: Clock) -> Registry {
    let mut r = Registry::new();
    r.add::<LoadArgs>("load", "Carrega um .spell e devolve nome, fps, duracao e tracks.", |a| {
        let sh = show::load(Path::new(&a.path))?;
        let tl = Timeline::new(&sh)?;
        Ok(json!({"name": sh.name, "fps": tl.fps, "duration": tl.duration,
                  "tracks": tl.tracks.len(), "ignored": tl.ignored()}))
    });
    r.add::<LocateArgs>("locate", "Move o relogio para t segundos.", move |a| {
        clock.locate(a.t);
        Ok(json!({ "t": clock.time() }))
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
        assert_eq!(c["mcp"], true);
        assert!(c["params"]["properties"]["a"].is_object(), "schema: {}", c["params"]);
        assert_eq!(r.get("soma").unwrap().name, "soma");
        assert_eq!(r.iter().count(), 1);
    }

    #[test]
    fn base_tem_load_e_locate() {
        let clk = Clock::new(30);
        let r = base(clk.clone());
        assert!(r.get("load").is_some() && r.get("locate").is_some());
        r.call("locate", json!({"t": 3.5})).unwrap();
        assert_eq!(clk.time(), 3.5);
        assert!(r.call("load", json!({"path": "nao_existe.spell"})).is_err());
    }
}
