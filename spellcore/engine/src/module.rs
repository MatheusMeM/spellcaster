//! Module = an app declared in text. Each app (the laser, the media player, a Pi on the network)
//! publishes a `module.json` with its `parameters`, `values` and `commands`; the core only keeps
//! the table of live modules, and the PATCHBAY builds the node without knowing the app. Format of
//! `design/FUNCOES/orquestrador.md` section 5 (the Chataigne `module.json`), trimmed down: the
//! text address is the identity (rule 2) and the parameter type generates the widget (rule 1).
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

/// Accepted parameter types: this is what decides the widget (rule 1) and whether it fires,
/// toggles or holds a value (rule 3): `trigger` fires, `bool` toggles, the rest holds a value.
const TIPOS: [&str; 7] = ["float", "int", "bool", "trigger", "color", "string", "enum"];

/// Command context (the Chataigne `CommandContext`): trigger only, value only, or both.
const CONTEXTOS: [&str; 3] = ["action", "mapping", "both"];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Param {
    #[serde(rename = "type")]
    pub r#type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    /// Useful slider range, separate from the physical `min`/`max` clamp (rule 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub norm: Option<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Values of `type: enum`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<Value>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cmd {
    pub context: String,
}

// ponytail: foreign fields of the Chataigne manifest (`hasInput`, `dependency`, `label`, `unit`,
// `args`) are ignored on read and do not come back on write ; they land when some client uses them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    #[serde(default, rename = "type")]
    pub r#type: String,
    #[serde(default)]
    pub version: String,
    /// Address `a/b` -> configuration parameter (input).
    #[serde(default)]
    pub parameters: BTreeMap<String, Param>,
    /// Address `a/b` -> read-only value (output).
    #[serde(default)]
    pub values: BTreeMap<String, Param>,
    #[serde(default)]
    pub commands: BTreeMap<String, Cmd>,
}

/// Live modules in this process, in the order they arrived.
// ponytail: a per-process table, like the registry `OPEN` ; it becomes a per-session table once
// the GUI opens two shows at the same time.
static MODULES: Mutex<Vec<Module>> = Mutex::new(Vec::new());

pub fn load(path: &Path) -> Result<Module, String> {
    let txt = std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    serde_json::from_str(&txt).map_err(|e| format!("{}: {}", path.display(), e))
}

/// The `modules/` folder, by the same rule as the other show resources.
pub fn modules_dir(spell: &str) -> PathBuf {
    crate::edit::recurso_dir(spell, "modules")
}

/// Text address: `a/b`, no whitespace and no empty segment (rule 2).
fn endereco_ok(s: &str) -> bool {
    !s.is_empty() && !s.contains(char::is_whitespace) && !s.split('/').any(str::is_empty)
}

/// Manifest errors, all at once (an error is data, rule 5). Empty = it passed.
pub fn check(m: &Module) -> Vec<String> {
    let mut e = Vec::new();
    if m.name.trim().is_empty() {
        e.push("empty name".to_string());
    }
    for (grupo, tab) in [("parameters", &m.parameters), ("values", &m.values)] {
        for (path, p) in tab {
            param(grupo, path, p, &mut e);
        }
    }
    for (name, c) in &m.commands {
        if !endereco_ok(name) {
            e.push(format!(
                "commands/{}: the address must be a/b, with no whitespace",
                name
            ));
        }
        if !CONTEXTOS.contains(&c.context.as_str()) {
            e.push(format!(
                "commands/{}: context {:?} is not one of {:?}",
                name, c.context, CONTEXTOS
            ));
        }
    }
    e
}

fn param(grupo: &str, path: &str, p: &Param, e: &mut Vec<String>) {
    let em = |t: String| format!("{}/{}: {}", grupo, path, t);
    if !endereco_ok(path) {
        e.push(em("the address must be a/b, with no whitespace".into()));
    }
    if let (Some(a), Some(b)) = (p.min, p.max) {
        if a >= b {
            e.push(em(format!("min {} is not lower than max {}", a, b)));
        }
    }
    if let Some(n) = p.norm {
        if n[0] >= n[1] {
            e.push(em(format!("norm [{}, {}] is inverted", n[0], n[1])));
        }
    }
    if let Some(d) = p.default.as_ref().and_then(Value::as_f64) {
        if p.min.is_some_and(|a| d < a) || p.max.is_some_and(|b| d > b) {
            e.push(em(format!("default {} outside min/max", d)));
        }
    }
    if !TIPOS.contains(&p.r#type.as_str()) {
        e.push(em(format!("type {:?} is not one of {:?}", p.r#type, TIPOS)));
    }
    match (p.r#type == "enum", &p.options) {
        (true, None) => e.push(em("type enum without options".into())),
        (true, Some(o)) if o.is_empty() => e.push(em("empty options".into())),
        (true, Some(o)) => {
            if let Some(d) = &p.default {
                if !o.contains(d) {
                    e.push(em(format!("default {} is not in options", d)));
                }
            }
        }
        (false, Some(_)) => e.push(em("options only counts with type enum".into())),
        _ => {}
    }
}

// ------------------------------------------------------------------ commands

#[derive(Deserialize, JsonSchema)]
pub struct ModuleArgs {
    /// Name in modules/ (without .json) or path of a .json.
    #[serde(default)]
    pub file: String,
    /// The whole manifest, when it does not come from a file.
    #[serde(default)]
    pub data: Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct NameArgs {
    /// Module name (the `name` field of the manifest).
    pub name: String,
}

/// Path of the manifest: with an extension it is a path; without, a name in `modules/` (like the profile).
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
        Err("module: pass file= (name or path) or data= (the manifest)".to_string())
    }
}

pub fn register(r: &mut Registry) {
    r.add::<ModuleArgs>(
        "module_add",
        "Loads a module.json (file=) or the manifest (data=), validates it and puts it in the table of live modules; the same name replaces. Returns name and version.",
        |a| {
            let m = manifesto(&a)?;
            let e = check(&m);
            if !e.is_empty() {
                return Err(format!("module {:?}: {}", m.name, e.join("; ")));
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
        "Removes the module from the table by name. Returns the removed manifest.",
        |a| {
            let mut g = lock(&MODULES);
            match g.iter().position(|x| x.name == a.name) {
                Some(i) => serde_json::to_value(g.remove(i)).map_err(|e| e.to_string()),
                None => Err(format!("module {:?} is not loaded", a.name)),
            }
        },
    );
    r.add::<NoArgs>(
        "module_list",
        "Live modules: name, type and version.",
        |_| {
            Ok(Value::Array(
                lock(&MODULES)
                    .iter()
                    .map(|m| json!({"name": m.name, "type": m.r#type, "version": m.version}))
                    .collect(),
            ))
        },
    );
    r.add::<NameArgs>(
        "module_get",
        "The whole manifest of the loaded module (parameters, values, commands).",
        |a| match lock(&MODULES).iter().find(|x| x.name == a.name) {
            Some(m) => serde_json::to_value(m).map_err(|e| e.to_string()),
            None => Err(format!("module {:?} is not loaded", a.name)),
        },
    );
}
