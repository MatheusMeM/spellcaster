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
use std::sync::Mutex;

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
    /// Caminho do arquivo .spell (o mesmo `file` de show_get, show_save e play_show).
    // ponytail: `path` era o nome deste argumento e continua aceito por uma rodada, por script e
    // sessao MCP ja' escritos (no repo nao sobrou chamador) ; tirar o alias na rodada 3.
    #[serde(alias = "path")]
    pub file: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct LocateArgs {
    /// Posicao em segundos.
    pub t: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct LoopArgs {
    /// true liga o loop, false desliga.
    pub on: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct CueGoArgs {
    /// Indice da cue; ausente = a proxima.
    #[serde(default)]
    pub index: Option<usize>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ShowGetArgs {
    /// Caminho do .spell a abrir; vazio = o ultimo aberto neste processo.
    #[serde(default)]
    pub file: String,
    /// true = o .spell inteiro (o que a GUI desenha) em vez do resumo.
    #[serde(default)]
    pub full: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct InputArgs {
    /// Chave do evento: "widget:go", "key:Space", "osc:/spell/go", "module:laser/stat/fps".
    pub key: String,
    /// Valor do evento; trigger manda 1.
    #[serde(default)]
    pub value: f64,
}

/// Comando sem parametro.
#[derive(Deserialize, JsonSchema)]
pub struct NoArgs {}

/// Ultimo .spell aberto neste processo (o `OPEN` do `spellcaster/mcp/tools.py`): (caminho, show).
// ponytail: um show aberto por processo, gravado por `load`, `show_get` e os comandos de `edit`
// ; virar id de sessao quando a GUI abrir dois shows ao mesmo tempo. `play_show` NAO grava aqui
// (mora na CLI, que nao ve este estado): depois de um play, `show_get` continua pedindo `file`.
pub(crate) static OPEN: Mutex<Option<(String, show::Show)>> = Mutex::new(None);

pub(crate) fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Caminho do .spell aberto neste processo; vazio quando o show so' existe em memoria. E' a base
/// dos arquivos que o show cita por nome — o `clip` de um track laser resolve contra a pasta
/// dele, igual ao `_load_clip` de `spellcaster/player/player.py`.
pub fn open_path() -> String {
    lock(&OPEN)
        .as_ref()
        .map(|(p, _)| p.clone())
        .unwrap_or_default()
}

/// Resumo do show para a IA e para o resource `spell://show` (mesmos campos do `summary()` do
/// `spellcaster/mcp/tools.py`), mais o transporte vivo quando ha player rodando.
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

/// O player vivo neste processo, ou o erro que todo comando de transporte devolve sem ele.
pub(crate) fn vivo() -> Result<player::Handle, String> {
    player::current().ok_or_else(|| "sem player em execucao".to_string())
}

fn estado(h: &player::Handle) -> Result<Value, String> {
    serde_json::to_value(h.state()).map_err(|e| e.to_string())
}

/// Comandos do engine. `play_show` e `net` ficam na CLI (dependem de `script` e de varredura de
/// rede) e sao acrescentados de fora com `Registry::add`.
pub fn base() -> Registry {
    let mut r = Registry::new();
    r.add::<LoadArgs>(
        "load",
        "Abre um .spell E VALIDA a timeline (o que show_get{file} nao faz): devolve nome, fps, duracao, quantos tracks e quais foram ignorados. O argumento `path` e' o nome velho de `file` (deprecated, sai na proxima rodada).",
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
        "Resumo do .spell aberto (ou do arquivo dado): nome, fps, duracao, saidas, patch, tracks, cues. full=true devolve o .spell inteiro.",
        |a| {
            if !a.file.is_empty() {
                let sh = show::load(Path::new(&a.file))?;
                crate::edit::abre(a.file.clone(), sh);
            }
            match &*lock(&OPEN) {
                Some((_, sh)) if a.full => serde_json::to_value(sh).map_err(|e| e.to_string()),
                Some((f, sh)) => Ok(resumo(f, sh)),
                None => Ok(json!({"aberto": false,
                                  "dica": "chame show_get com file=<caminho.spell>"})),
            }
        },
    );
    // O par do `pause`: sem ele, quem pausou pelo registry (GUI, MCP, OSC) so' voltava a tocar
    // subindo outro player com `play_show`, e o `serve --show`, que deixa o player parado em
    // t=0, nao teria como solta-lo. Chama-se `resume` e nao `play` porque `play` e' o subcomando
    // da CLI que SOBE um player (o `play_show` do registry); aqui nao se sobe nada.
    r.add::<NoArgs>(
        "resume",
        "Retoma o player pausado neste processo (o par do pause).",
        |_| {
            let h = vivo()?;
            h.play();
            estado(&h)
        },
    );
    r.add::<NoArgs>(
        "pause",
        "Pausa o player em execucao neste processo.",
        |_| {
            let h = vivo()?;
            h.pause();
            estado(&h)
        },
    );
    r.add::<NoArgs>("stop", "Para o player em execucao neste processo.", |_| {
        let h = vivo()?;
        h.stop();
        estado(&h)
    });
    r.add::<LocateArgs>(
        "locate",
        "Salta o player para o instante t (segundos).",
        |a| {
            let h = vivo()?;
            h.locate(a.t);
            estado(&h)
        },
    );
    // O loop e' do PLAYER, no intervalo In-Out do show ABERTO (`edit::intervalo`): quem arrasta o
    // In na GUI e chama `loop_set` de novo ja' repete no intervalo novo. Sem In/Out, 0..duration.
    r.add::<LoopArgs>(
        "loop_set",
        "Liga ou desliga o loop do player no intervalo In-Out do show aberto.",
        |a| {
            let h = vivo()?;
            let (i, o) = crate::edit::intervalo();
            h.set_loop(a.on, i, o);
            estado(&h)
        },
    );
    r.add::<CueGoArgs>(
        "cue_go",
        "Dispara a proxima cue (ou a de indice dado).",
        |a| {
            let h = vivo()?;
            h.cue_go(a.index);
            estado(&h)
        },
    );
    r.add::<NoArgs>(
        "transport_state",
        "Estado do transporte do player em execucao.",
        |_| estado(&vivo()?),
    );
    r.add::<InputArgs>(
        "input",
        "Entrega um evento de entrada aos ganchos do player vivo (o Graph): key + value.",
        |a| {
            vivo()?.input(&a.key, a.value);
            Ok(json!({"key": a.key, "value": a.value}))
        },
    );
    crate::edit::register(&mut r);
    crate::module::register(&mut r);
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
        assert!(
            c["params"]["properties"]["a"].is_object(),
            "schema: {}",
            c["params"]
        );
        assert_eq!(r.get("soma").unwrap().name, "soma");
        assert_eq!(r.iter().count(), 1);
    }

    /// Sem player vivo, todo comando de transporte devolve o mesmo erro; `load` continua livre.
    #[test]
    fn base_tem_transporte_e_load() {
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
        ];
        for c in esperados {
            assert!(r.get(c).is_some(), "comando {} ausente", c);
        }
        // ponytail: o teste so' vale quando nao ha player neste processo — os testes do player
        // sobem o seu em outro binario (tests/player.rs), entao aqui nunca ha CURRENT.
        for (c, a) in [
            ("resume", json!({})),
            ("pause", json!({})),
            ("stop", json!({})),
            ("locate", json!({"t": 3.5})),
            ("loop_set", json!({"on": true})),
            ("cue_go", json!({})),
            ("transport_state", json!({})),
            ("input", json!({"key": "widget:go", "value": 1.0})),
        ] {
            assert_eq!(
                r.call(c, a).unwrap_err(),
                "sem player em execucao",
                "comando {}",
                c
            );
        }
        assert!(r.call("load", json!({"file": "nao_existe.spell"})).is_err());
        // o alias deprecated ainda entra: erro de arquivo, nao de argumento faltando
        assert!(r
            .call("load", json!({"path": "nao_existe.spell"}))
            .unwrap_err()
            .contains("nao_existe.spell"));
        assert!(
            r.call("load", json!({})).is_err(),
            "sem file nem path: erro de deserializacao"
        );
    }

    /// `show_get` sem show aberto avisa; com `file` abre, resume e fica aberto para a proxima
    /// chamada (e' o que alimenta o resource `spell://show` do MCP).
    #[test]
    fn show_get_abre_e_lembra() {
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
        assert_eq!(
            d["transport"],
            Value::Null,
            "sem player neste binario de teste"
        );
        // sem `file`, devolve o mesmo show
        assert_eq!(r.call("show_get", json!({})).unwrap(), d);
        assert!(r
            .call("show_get", json!({"file": "nao_existe.spell"}))
            .is_err());
    }
}
