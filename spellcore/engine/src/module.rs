//! Modulo = app declarado em texto. Cada app (o laser, o player de midia, um Pi na rede) publica
//! um `module.json` com os seus `parameters`, `values` e `commands`; o core so' guarda a tabela
//! dos modulos vivos, e o PATCHBAY monta o no' sem conhecer o app. Formato de
//! `design/FUNCOES/orquestrador.md` secao 5 (o `module.json` do Chataigne), enxuto:
//! endereco textual e' a identidade (regra 2) e o tipo do parametro gera o widget (regra 1).
//!
//! ```json
//! { "name": "laser", "type": "laser", "version": "0.1.0",
//!   "parameters": { "geo/scale": { "type": "float", "default": 1, "min": 0, "max": 4 } },
//!   "values":     { "stat/fps":  { "type": "float" } },
//!   "commands":   { "play": { "context": "action" } } }
//! ```

use crate::registry::{lock, NoArgs, Registry, OPEN};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Tipos de parametro aceitos: e' o que decide o widget (regra 1) e se ele dispara, liga ou
/// vale (regra 3): `trigger` dispara, `bool` liga, o resto vale.
const TIPOS: [&str; 7] = ["float", "int", "bool", "trigger", "color", "string", "enum"];

/// Contexto do comando (o `CommandContext` do Chataigne): so' disparo, so' valor, ou os dois.
const CONTEXTOS: [&str; 3] = ["action", "mapping", "both"];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Param {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    /// Faixa util do slider, separada do clamp fisico `min`/`max` (regra 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub norm: Option<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Valores do `type: enum`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<Value>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cmd {
    pub context: String,
}

// ponytail: campos alheios do manifesto do Chataigne (`hasInput`, `dependency`, `label`, `unit`, `args`)
// sao ignorados na leitura e nao voltam na gravacao ; entram quando algum cliente usar.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    #[serde(default, rename = "type")]
    pub r#type: String,
    #[serde(default)]
    pub version: String,
    /// Endereco `a/b` -> parametro de configuracao (entrada).
    #[serde(default)]
    pub parameters: BTreeMap<String, Param>,
    /// Endereco `a/b` -> valor so' de leitura (saida).
    #[serde(default)]
    pub values: BTreeMap<String, Param>,
    #[serde(default)]
    pub commands: BTreeMap<String, Cmd>,
}

/// Modulos vivos neste processo, na ordem em que entraram.
// ponytail: tabela por processo, como o `OPEN` do registry ; vira tabela por sessao quando a GUI
// abrir dois shows ao mesmo tempo.
static MODULES: Mutex<Vec<Module>> = Mutex::new(Vec::new());

pub fn load(path: &Path) -> Result<Module, String> {
    let txt = std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    serde_json::from_str(&txt).map_err(|e| format!("{}: {}", path.display(), e))
}

/// Pasta `modules/`, pela mesma regra de `edit::profiles_dir`.
pub fn modules_dir(spell: &str) -> PathBuf {
    crate::edit::dir_do_show(spell, "modules")
}

/// Endereco textual: `a/b`, sem espaco e sem segmento vazio (regra 2).
fn endereco_ok(s: &str) -> bool {
    !s.is_empty() && !s.contains(char::is_whitespace) && !s.split('/').any(str::is_empty)
}

/// Erros do manifesto, todos de uma vez (erro e' dado, regra 5). Vazio = passou.
pub fn check(m: &Module) -> Vec<String> {
    let mut e = Vec::new();
    if m.name.trim().is_empty() {
        e.push("name vazio".to_string());
    }
    for (grupo, tab) in [("parameters", &m.parameters), ("values", &m.values)] {
        for (path, p) in tab {
            param(grupo, path, p, &mut e);
        }
    }
    for (name, c) in &m.commands {
        if !endereco_ok(name) {
            e.push(format!(
                "commands/{}: endereco precisa ser a/b, sem espaco",
                name
            ));
        }
        if !CONTEXTOS.contains(&c.context.as_str()) {
            e.push(format!(
                "commands/{}: context {:?} nao e' um de {:?}",
                name, c.context, CONTEXTOS
            ));
        }
    }
    e
}

fn param(grupo: &str, path: &str, p: &Param, e: &mut Vec<String>) {
    let em = |t: String| format!("{}/{}: {}", grupo, path, t);
    if !endereco_ok(path) {
        e.push(em("endereco precisa ser a/b, sem espaco".into()));
    }
    if let (Some(a), Some(b)) = (p.min, p.max) {
        if a >= b {
            e.push(em(format!("min {} nao e' menor que max {}", a, b)));
        }
    }
    if let Some(n) = p.norm {
        if n[0] >= n[1] {
            e.push(em(format!("norm [{}, {}] invertida", n[0], n[1])));
        }
    }
    if let Some(d) = p.default.as_ref().and_then(Value::as_f64) {
        if p.min.is_some_and(|a| d < a) || p.max.is_some_and(|b| d > b) {
            e.push(em(format!("default {} fora de min/max", d)));
        }
    }
    if !TIPOS.contains(&p.r#type.as_str()) {
        e.push(em(format!("type {:?} nao e' um de {:?}", p.r#type, TIPOS)));
    }
    match (p.r#type == "enum", &p.options) {
        (true, None) => e.push(em("type enum sem options".into())),
        (true, Some(o)) if o.is_empty() => e.push(em("options vazio".into())),
        (true, Some(o)) => {
            if let Some(d) = &p.default {
                if !o.contains(d) {
                    e.push(em(format!("default {} nao esta em options", d)));
                }
            }
        }
        (false, Some(_)) => e.push(em("options so' vale com type enum".into())),
        _ => {}
    }
}

// ------------------------------------------------------------------ comandos

#[derive(Deserialize, JsonSchema)]
pub struct ModuleArgs {
    /// Nome em modules/ (sem .json) ou caminho de um .json.
    #[serde(default)]
    pub file: String,
    /// O manifesto inteiro, quando nao vem de arquivo.
    #[serde(default)]
    pub data: Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct NameArgs {
    /// Nome do modulo (o campo `name` do manifesto).
    pub name: String,
}

/// Caminho do manifesto: com extensao e' caminho; sem, e' nome em `modules/` (como o perfil).
fn arquivo(f: &str) -> PathBuf {
    if Path::new(f).extension().is_some() {
        return PathBuf::from(f);
    }
    let spell = lock(&OPEN)
        .as_ref()
        .map(|(p, _)| p.clone())
        .unwrap_or_default();
    modules_dir(&spell).join(format!("{}.json", f))
}

fn manifesto(a: &ModuleArgs) -> Result<Module, String> {
    if !a.file.is_empty() {
        load(&arquivo(&a.file))
    } else if a.data.is_object() {
        serde_json::from_value(a.data.clone()).map_err(|e| e.to_string())
    } else {
        Err("module: passe file= (nome ou caminho) ou data= (o manifesto)".to_string())
    }
}

pub fn register(r: &mut Registry) {
    r.add::<ModuleArgs>(
        "module_add",
        "Carrega um module.json (file=) ou o manifesto (data=), valida e poe na tabela de modulos vivos; mesmo nome substitui. Devolve nome e versao.",
        |a| {
            let m = manifesto(&a)?;
            let e = check(&m);
            if !e.is_empty() {
                return Err(format!("modulo {:?}: {}", m.name, e.join("; ")));
            }
            let out = json!({"name": m.name, "version": m.version});
            let mut g = lock(&MODULES);
            match g.iter().position(|x| x.name == m.name) {
                Some(i) => g[i] = m,
                None => g.push(m),
            }
            Ok(out)
        },
    );
    r.add::<NameArgs>(
        "module_del",
        "Tira o modulo da tabela pelo nome. Devolve o manifesto removido.",
        |a| {
            let mut g = lock(&MODULES);
            match g.iter().position(|x| x.name == a.name) {
                Some(i) => serde_json::to_value(g.remove(i)).map_err(|e| e.to_string()),
                None => Err(format!("modulo {:?} nao esta carregado", a.name)),
            }
        },
    );
    r.add::<NoArgs>("module_list", "Modulos vivos: nome, tipo e versao.", |_| {
        Ok(Value::Array(
            lock(&MODULES)
                .iter()
                .map(|m| json!({"name": m.name, "type": m.r#type, "version": m.version}))
                .collect(),
        ))
    });
    r.add::<NameArgs>(
        "module_get",
        "Manifesto inteiro do modulo carregado (parameters, values, commands).",
        |a| match lock(&MODULES).iter().find(|x| x.name == a.name) {
            Some(m) => serde_json::to_value(m).map_err(|e| e.to_string()),
            None => Err(format!("modulo {:?} nao esta carregado", a.name)),
        },
    );
}

