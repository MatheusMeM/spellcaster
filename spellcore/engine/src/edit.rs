//! Edicao do show aberto neste processo (o `OPEN` do registry): show, tracks, keyframes, cues e
//! patch. Porte de `spellcaster/gui/api.py` mais a checagem de footprint de `fixtures/patch.py`.
//! GUI e MCP editam por aqui; nenhuma logica de edicao vive fora do registry.

use crate::cues;
use crate::registry::{lock, NoArgs, Registry, OPEN};
use crate::show::{self, OutputCfg, Show};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const CURVES: [&str; 6] = ["linear", "hold", "in", "out", "inout", "bezier"];

/// Show vazio (o `NEW` do `gui/api.py`): sACN no universo 1, 60 s.
pub fn novo() -> Show {
    Show {
        name: "novo show".into(),
        duration: Some(60.0),
        outputs: vec![OutputCfg::Sacn {
            universes: vec![1],
            priority: 100,
            source_name: "Spellcaster".into(),
            interfaces: None,
        }],
        extra: [
            ("patch", json!([])),
            ("cues", json!([])),
            ("markers", json!([])),
            ("in", json!(0.0)),
            ("out", json!(60.0)),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect(),
        ..Show::default()
    }
}

/// Roda `f` no show aberto (caminho, show). Sem show aberto, abre um novo — o Python nasce com
/// `SHOW = NEW`, e a IA pode chamar `track_add` antes de qualquer `show_get`.
fn com<T>(f: impl FnOnce(&mut String, &mut Show) -> Result<T, String>) -> Result<T, String> {
    let mut g = lock(&OPEN);
    let (p, sh) = g.get_or_insert_with(|| (String::new(), novo()));
    f(p, sh)
}

fn json(sh: &Show) -> Result<Value, String> {
    serde_json::to_value(sh).map_err(|e| e.to_string())
}

/// Texto que e' JSON vira JSON (`"255"`, `"[255,0,0]"`); o resto fica texto (`"amarelo"`).
/// E' o `_value` do Python, para clientes que so' mandam string (CLI, campo de formulario).
fn valor(v: Value) -> Value {
    match v {
        Value::String(s) => serde_json::from_str(&s).unwrap_or(Value::String(s)),
        v => v,
    }
}

fn track(sh: &mut Show, i: usize) -> Result<&mut Value, String> {
    let n = sh.tracks.len();
    sh.tracks
        .get_mut(i)
        .ok_or_else(|| format!("track {}: o show tem {}", i, n))
}

fn keys(tr: &mut Value) -> Result<&mut Vec<Value>, String> {
    let o = tr
        .as_object_mut()
        .ok_or_else(|| "track nao e' objeto JSON".to_string())?;
    let k = o.entry("keys").or_insert_with(|| json!([]));
    if !k.is_array() {
        *k = json!([]);
    }
    Ok(k.as_array_mut().expect("lista"))
}

fn tempo(k: &Value) -> f64 {
    k.get(0).and_then(|v| v.as_f64()).unwrap_or(f64::NAN)
}

/// Lista em `extra` ("cues", "patch"), criada vazia quando falta.
fn lista<'a>(sh: &'a mut Show, k: &str) -> &'a mut Vec<Value> {
    let v = sh.extra.entry(k.to_string()).or_insert_with(|| json!([]));
    if !v.is_array() {
        *v = json!([]);
    }
    v.as_array_mut().expect("lista")
}

// ------------------------------------------------------------------ patch

struct Perfil {
    name: String,
    size: u16,
}

/// Perfil por nome em `dir` (sem .json) ou por caminho. So' nome e footprint.
// ponytail: nomes de canal, faixas e roda ficam no Python ; entram aqui quando o track
// `fixture` for resolvido no Rust.
fn perfil(dir: &Path, p: &str) -> Result<Perfil, String> {
    let f = if Path::new(p).extension().is_some() {
        PathBuf::from(p)
    } else {
        dir.join(format!("{}.json", p))
    };
    let txt = std::fs::read_to_string(&f)
        .map_err(|_| format!("perfil {:?} nao encontrado ({})", p, f.display()))?;
    let v: Value = serde_json::from_str(&txt).map_err(|e| format!("{}: {}", f.display(), e))?;
    let name = v["name"].as_str().unwrap_or("?").to_string();
    let mut max: Option<u64> = None;
    for c in v["channels"].as_array().into_iter().flatten() {
        for k in ["offset", "fine"] {
            if let Some(o) = c[k].as_u64() {
                max = Some(max.map_or(o, |m| m.max(o)));
            }
        }
    }
    let m = max.ok_or_else(|| format!("{}: sem canais", name))?;
    if m > 511 {
        return Err(format!("{}: offset {} fora de 0..511", name, m));
    }
    Ok(Perfil {
        name,
        size: m as u16 + 1,
    })
}

/// Pasta `<nome>/` do show: ao lado do .spell, um nivel acima (`shows/` e `profiles/` irmaos,
/// como no repo e no pendrive), no cwd ou ao lado do executavel — a primeira que existir.
/// `profiles/` e `modules/` (o `module::modules_dir`) resolvem pela mesma regra.
pub fn dir_do_show(spell: &str, nome: &str) -> PathBuf {
    let mut c = Vec::new();
    if !spell.is_empty() {
        let d = Path::new(spell).parent().unwrap_or(Path::new("."));
        c.push(d.join(nome));
        c.push(d.join("..").join(nome));
    }
    c.push(PathBuf::from(nome));
    if let Some(d) = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf))
    {
        c.push(d.join(nome));
    }
    c.into_iter()
        .find(|p| p.is_dir())
        .unwrap_or_else(|| PathBuf::from(nome))
}

pub fn profiles_dir(spell: &str) -> PathBuf {
    dir_do_show(spell, "profiles")
}

/// Uma linha da grade por fixture; para no primeiro erro (perfil ausente, fora de 512,
/// sobreposicao — o bug dos 17 ch em espacamento de 16 acusa aqui).
fn checar(sh: &Show, dir: &Path) -> (Vec<Value>, Option<String>) {
    let mut rows: Vec<Value> = Vec::new();
    let mut busy: HashMap<(u16, u16), usize> = HashMap::new(); // (universo, canal) -> linha
    let vazio = Vec::new();
    let patch = sh
        .extra
        .get("patch")
        .and_then(|v| v.as_array())
        .unwrap_or(&vazio);
    for f in patch {
        match linha(f, dir, &rows, &mut busy) {
            Ok(r) => rows.push(r),
            Err(e) => return (rows, Some(e)),
        }
    }
    (rows, None)
}

fn linha(
    f: &Value,
    dir: &Path,
    rows: &[Value],
    busy: &mut HashMap<(u16, u16), usize>,
) -> Result<Value, String> {
    let name = f["name"]
        .as_str()
        .ok_or_else(|| "fixture sem name".to_string())?;
    if rows.iter().any(|r| r["name"] == name) {
        return Err(format!("fixture {:?} ja esta no patch", name));
    }
    let prof = f["profile"]
        .as_str()
        .ok_or_else(|| format!("{}: sem profile", name))?;
    let p = perfil(dir, prof)?;
    let u = f["universe"].as_u64().unwrap_or(1) as u16;
    let a = f["address"]
        .as_u64()
        .ok_or_else(|| format!("{}: sem address", name))? as u16;
    if a < 1 || a as u32 + p.size as u32 - 1 > 512 {
        return Err(format!(
            "{} [{}]: endereco {} + {} ch passa de 512 (universo {})",
            name, p.name, a, p.size, u
        ));
    }
    for ch in a..a + p.size {
        if let Some(&i) = busy.get(&(u, ch)) {
            let o = &rows[i];
            return Err(format!(
                "sobreposicao no universo {} canal {}: {:?} [{}, {} ch em {}] e {:?} [{}, {} ch em {}]",
                u,
                ch,
                o["name"].as_str().unwrap_or(""),
                o["profile"].as_str().unwrap_or(""),
                o["channels"],
                o["address"],
                name,
                p.name,
                p.size,
                a
            ));
        }
    }
    let i = rows.len();
    busy.extend((a..a + p.size).map(|ch| ((u, ch), i)));
    Ok(json!({"name": name, "profile": p.name, "universe": u, "address": a, "channels": p.size}))
}

// ------------------------------------------------------------------ args

fn um() -> u16 {
    1
}
fn dmx() -> String {
    "dmx".into()
}
fn linear() -> String {
    "linear".into()
}

#[derive(Deserialize, JsonSchema)]
pub struct ShowSetArgs {
    /// O show inteiro: objeto JSON (ou texto JSON).
    pub data: Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct ShowSaveArgs {
    /// Caminho do .spell; vazio = o do ultimo load/show_get/show_save.
    #[serde(default)]
    pub file: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TrackAddArgs {
    /// dmx | artnet | osc | media | cue | fx.
    #[serde(default = "dmx", rename = "type")]
    #[schemars(rename = "type")]
    pub kind: String,
    #[serde(default = "um")]
    pub universe: u16,
    /// Endereco DMX 1..512 (tracks osc usam texto: passe pelo show_set).
    #[serde(default = "um")]
    pub address: u16,
    /// Nome do track (campo `name` do .spell).
    #[serde(default)]
    pub label: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TrackDelArgs {
    /// Indice do track em `tracks`.
    pub index: usize,
}

#[derive(Deserialize, JsonSchema)]
pub struct KeySetArgs {
    /// Indice do track.
    pub track: usize,
    /// Instante em segundos.
    pub t: f64,
    /// 255, [255, 0, 0] ou texto ("play"); ausente = 0.
    #[serde(default)]
    pub value: Value,
    /// linear | hold | in | out | inout | bezier (curva do segmento que CHEGA neste keyframe).
    #[serde(default = "linear")]
    pub curve: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct KeyDelArgs {
    pub track: usize,
    /// Instante do keyframe (tolerancia 1 ms).
    pub t: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct CueSetArgs {
    /// Indice da cue a substituir; ausente = acrescenta no fim.
    #[serde(default)]
    pub index: Option<usize>,
    #[serde(default)]
    pub name: String,
    /// Segundos de fade linear ate os valores.
    #[serde(default)]
    pub fade: f64,
    /// Segundos entre o GO e o inicio do fade.
    #[serde(default)]
    pub wait: f64,
    /// Ao terminar, dispara a proxima.
    #[serde(default)]
    pub follow: bool,
    /// {"universo/endereco": valor ou [valores]}; "100" = universo 1.
    #[serde(default)]
    pub values: Map<String, Value>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CueDelArgs {
    pub index: usize,
}

#[derive(Deserialize, JsonSchema)]
pub struct PatchAddArgs {
    /// Nome unico da fixture no show.
    pub name: String,
    /// Perfil em profiles/ (sem .json) ou caminho de um .json.
    pub profile: String,
    #[serde(default = "um")]
    pub universe: u16,
    /// Primeiro canal DMX (1..512).
    #[serde(default = "um")]
    pub address: u16,
}

#[derive(Deserialize, JsonSchema)]
pub struct PatchDelArgs {
    pub name: String,
}

// ------------------------------------------------------------------ comandos

pub fn register(r: &mut Registry) {
    r.add::<NoArgs>(
        "show_new",
        "Zera o show aberto: novo show, sACN no universo 1, 60 s. Devolve o show inteiro.",
        |_| {
            let sh = novo();
            let v = json(&sh)?;
            *lock(&OPEN) = Some((String::new(), sh));
            Ok(v)
        },
    );
    r.add::<ShowSetArgs>(
        "show_set",
        "Substitui o show aberto pelo JSON dado. Devolve o show inteiro.",
        |a| {
            let v = valor(a.data);
            if !v.is_object() {
                return Err("show_set: esperava um objeto JSON".into());
            }
            let sh: Show = serde_json::from_value(v).map_err(|e| format!("show_set: {}", e))?;
            let mut sh = show::migrate(sh)?;
            sh.extra.retain(|k, _| !k.starts_with('_'));
            com(|_, cur| {
                *cur = sh;
                json(cur)
            })
        },
    );
    r.add::<ShowSaveArgs>(
        "show_save",
        "Grava o show aberto (sem file, no caminho do ultimo aberto). Devolve o caminho.",
        |a| {
            com(|p, sh| {
                let f = if a.file.is_empty() {
                    p.clone()
                } else {
                    a.file.clone()
                };
                if f.is_empty() {
                    return Err("show_save: sem caminho (passe file=)".into());
                }
                show::save(Path::new(&f), sh)?;
                *p = f.clone();
                Ok(json!(f))
            })
        },
    );
    r.add::<TrackAddArgs>(
        "track_add",
        "Acrescenta um track vazio ao show aberto. Devolve o indice do track.",
        |a| {
            com(|_, sh| {
                let mut tr = json!({"type": a.kind, "universe": a.universe,
                                    "address": a.address, "keys": []});
                if !a.label.is_empty() {
                    tr["name"] = json!(a.label);
                }
                sh.tracks.push(tr);
                Ok(json!(sh.tracks.len() - 1))
            })
        },
    );
    r.add::<TrackDelArgs>(
        "track_del",
        "Remove o track de indice dado. Devolve o track removido.",
        |a| {
            com(|_, sh| {
                track(sh, a.index)?;
                Ok(sh.tracks.remove(a.index))
            })
        },
    );
    r.add::<KeySetArgs>(
        "key_set",
        "Cria ou substitui o keyframe do track em t. Devolve os keyframes do track.",
        |a| {
            if !CURVES.contains(&a.curve.as_str()) {
                return Err(format!("curva {:?}: use {:?}", a.curve, CURVES));
            }
            com(|_, sh| {
                let ks = keys(track(sh, a.track)?)?;
                let v = match a.value {
                    Value::Null => json!(0),
                    v => valor(v),
                };
                let k = json!([a.t, v, a.curve]);
                match ks.iter().position(|x| (tempo(x) - a.t).abs() < 1e-6) {
                    Some(i) => ks[i] = k,
                    None => ks.push(k),
                }
                ks.sort_by(|x, y| tempo(x).total_cmp(&tempo(y)));
                Ok(json!(ks))
            })
        },
    );
    r.add::<KeyDelArgs>(
        "key_del",
        "Apaga o keyframe do track em t (tolerancia 1 ms). Devolve quantos sairam.",
        |a| {
            com(|_, sh| {
                let ks = keys(track(sh, a.track)?)?;
                let n = ks.len();
                ks.retain(|x| (tempo(x) - a.t).abs() > 1e-3);
                Ok(json!(n - ks.len()))
            })
        },
    );
    r.add::<CueSetArgs>(
        "cue_set",
        "Cria (sem index) ou substitui uma cue: nome, fade, wait, follow e valores DMX. Devolve o indice.",
        |a| {
            if let Some(k) = a.values.keys().find(|k| cues::key(k).is_none()) {
                return Err(format!("cue: chave {:?} nao e' \"universo/endereco\"", k));
            }
            let c = json!({"name": a.name, "fade": a.fade, "wait": a.wait,
                           "follow": a.follow, "values": a.values});
            com(|_, sh| {
                let cs = lista(sh, "cues");
                let i = match a.index {
                    Some(i) if i < cs.len() => {
                        cs[i] = c;
                        i
                    }
                    Some(i) => return Err(format!("cue {}: o show tem {}", i, cs.len())),
                    None => {
                        cs.push(c);
                        cs.len() - 1
                    }
                };
                Ok(json!(i))
            })
        },
    );
    r.add::<CueDelArgs>(
        "cue_del",
        "Remove a cue de indice dado. Devolve a cue removida.",
        |a| {
            com(|_, sh| {
                let cs = lista(sh, "cues");
                if a.index >= cs.len() {
                    return Err(format!("cue {}: o show tem {}", a.index, cs.len()));
                }
                Ok(cs.remove(a.index))
            })
        },
    );
    r.add::<PatchAddArgs>(
        "patch_add",
        "Patcheia uma fixture (perfil, universo, endereco); recusa sobreposicao e estouro de 512. Devolve a grade do patch.",
        |a| {
            com(|p, sh| {
                let dir = profiles_dir(p);
                lista(sh, "patch").push(json!({"name": a.name, "profile": a.profile,
                                               "universe": a.universe, "address": a.address}));
                match checar(sh, &dir) {
                    (rows, None) => Ok(json!(rows)),
                    (_, Some(e)) => {
                        lista(sh, "patch").pop();
                        Err(e)
                    }
                }
            })
        },
    );
    r.add::<PatchDelArgs>(
        "patch_del",
        "Tira a fixture do patch pelo nome. Devolve a entrada removida.",
        |a| {
            com(|_, sh| {
                let ps = lista(sh, "patch");
                match ps.iter().position(|f| f["name"] == a.name) {
                    Some(i) => Ok(ps.remove(i)),
                    None => Err(format!("fixture {:?} nao esta no patch", a.name)),
                }
            })
        },
    );
    r.add::<NoArgs>(
        "patch_check",
        "Grade do patch do show aberto (nome, perfil, universo, endereco, canais) e o erro de sobreposicao, se houver.",
        |_| {
            com(|p, sh| {
                let (rows, error) = checar(sh, &profiles_dir(p));
                Ok(json!({"rows": rows, "error": error}))
            })
        },
    );
    r.add::<NoArgs>("profiles", "Nomes dos perfis disponiveis em profiles/.", |_| {
        let dir = com(|p, _| Ok(profiles_dir(p)))?;
        let mut v: Vec<String> = std::fs::read_dir(&dir)
            .map_err(|e| format!("{}: {}", dir.display(), e))?
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                if p.extension()? != "json" {
                    return None;
                }
                p.file_stem()?.to_str().map(str::to_string)
            })
            .collect();
        v.sort();
        Ok(json!(v))
    });
}
