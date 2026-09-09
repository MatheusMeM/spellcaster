//! Edicao do show aberto neste processo (o `OPEN` do registry): show, tracks, keyframes, cues e
//! patch. Porte de `spellcaster/gui/api.py` mais a checagem de footprint de `fixtures/patch.py`.
//! GUI e MCP editam por aqui; nenhuma logica de edicao vive fora do registry.

use crate::cues;
use crate::registry::{lock, vivo, NoArgs, Registry, OPEN};
use crate::show::{self, OutputCfg, Show};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

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
fn com_ro<T>(f: impl FnOnce(&mut String, &mut Show) -> Result<T, String>) -> Result<T, String> {
    let mut g = lock(&OPEN);
    let (p, sh) = g.get_or_insert_with(|| (String::new(), novo()));
    f(p, sh)
}

/// `com_ro` mais o contador: toda edicao bem-sucedida sobe `rev`.
fn com<T>(f: impl FnOnce(&mut String, &mut Show) -> Result<T, String>) -> Result<T, String> {
    let v = com_ro(f)?;
    REV.fetch_add(1, Ordering::Relaxed);
    Ok(v)
}

/// Revisao do show aberto: sobe a cada edicao. O barramento faz broadcast dela; quem manda
/// `show_patch` com uma revisao velha leva erro em vez de sobrescrever a edicao do outro.
// ponytail: contador do processo, nao do arquivo ; virar hash do show se dois processos
// passarem a editar o mesmo .spell.
static REV: AtomicU64 = AtomicU64::new(0);

pub fn rev() -> u64 {
    REV.load(Ordering::Relaxed)
}

/// Troca o show aberto por inteiro (`load`, `show_get {file}`, `show_new`): grava `OPEN` e sobe
/// `rev`. Sem isso a `rev` que o cliente segurava continuaria valendo em OUTRO show, e o
/// `show_patch` dele entraria sem erro no arquivo errado.
pub(crate) fn abre(path: String, sh: Show) {
    *lock(&OPEN) = Some((path, sh));
    REV.fetch_add(1, Ordering::Relaxed);
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
    json: Value,
}

/// Perfil por nome em `dir` (sem .json) ou por caminho.
// ponytail: faixas (`ranges`) e roda (`wheel`) so' viajam no `json` cru, para o cliente
// desenhar ; viram tipo aqui quando o fade por tipo de canal existir.
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
        json: v,
    })
}

/// Pasta de recurso do show (`profiles/`, `faces/`, `modules/`): ao lado do .spell, um nivel
/// acima (`shows/` e `profiles/` irmaos, como no repo e no pendrive), no cwd ou ao lado do
/// executavel — a primeira que existir.
pub fn recurso_dir(spell: &str, nome: &str) -> PathBuf {
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

// ------------------------------------------------------------- json patch (RFC 6902)

/// Divide "/a/b/c" em ("/a/b", "c"), com o token final sem os escapes do RFC 6901.
fn dividir(path: &str) -> Result<(&str, String), String> {
    let i = path
        .rfind('/')
        .ok_or_else(|| format!("path {:?}: um JSON Pointer comeca com /", path))?;
    Ok((&path[..i], path[i + 1..].replace("~1", "/").replace("~0", "~")))
}

fn indice(n: usize, tok: &str, path: &str, inserindo: bool) -> Result<usize, String> {
    let i = if inserindo && tok == "-" {
        n
    } else {
        tok.parse::<usize>()
            .map_err(|_| format!("path {:?}: {:?} nao e' indice de lista", path, tok))?
    };
    if i > n || (!inserindo && i == n) {
        return Err(format!("path {:?}: indice {} fora da lista de {}", path, i, n));
    }
    Ok(i)
}

fn pai<'a>(doc: &'a mut Value, path: &str, p: &str) -> Result<&'a mut Value, String> {
    doc.pointer_mut(p)
        .ok_or_else(|| format!("path {:?}: {:?} nao existe", path, p))
}

/// `add`: insere na lista ("-" = fim) ou grava a chave do objeto. Devolve o path com o indice
/// ja' resolvido (o inverso nao pode dizer "-") e o valor que estava la', se havia.
fn add(doc: &mut Value, path: &str, v: Value) -> Result<(String, Option<Value>), String> {
    let (p, tok) = dividir(path)?;
    match pai(doc, path, p)? {
        Value::Array(a) => {
            let i = indice(a.len(), &tok, path, true)?;
            a.insert(i, v);
            Ok((format!("{}/{}", p, i), None))
        }
        Value::Object(o) => {
            let velho = o.insert(tok, v);
            Ok((path.to_string(), velho))
        }
        _ => Err(format!("path {:?}: {:?} nao e' objeto nem lista", path, p)),
    }
}

fn remove(doc: &mut Value, path: &str) -> Result<Value, String> {
    let (p, tok) = dividir(path)?;
    match pai(doc, path, p)? {
        Value::Array(a) => {
            let i = indice(a.len(), &tok, path, false)?;
            Ok(a.remove(i))
        }
        Value::Object(o) => o
            .remove(&tok)
            .ok_or_else(|| format!("path {:?} nao existe", path)),
        _ => Err(format!("path {:?}: {:?} nao e' objeto nem lista", path, p)),
    }
}

/// Uma operacao; devolve a operacao que a desfaz (`test` nao desfaz nada).
fn operar(doc: &mut Value, o: &PatchOp) -> Result<Option<Value>, String> {
    let valor = || {
        o.value
            .clone()
            .ok_or_else(|| format!("op {:?} em {:?}: falta value", o.op, o.path))
    };
    Ok(Some(match o.op.as_str() {
        "add" => match add(doc, &o.path, valor()?)? {
            (p, Some(v)) => json!({"op": "replace", "path": p, "value": v}),
            (p, None) => json!({"op": "remove", "path": p}),
        },
        "remove" => {
            let v = remove(doc, &o.path)?;
            json!({"op": "add", "path": o.path, "value": v})
        }
        "replace" => {
            let alvo = doc
                .pointer_mut(&o.path)
                .ok_or_else(|| format!("path {:?} nao existe", o.path))?;
            let v = std::mem::replace(alvo, valor()?);
            json!({"op": "replace", "path": o.path, "value": v})
        }
        "test" => {
            let v = doc
                .pointer(&o.path)
                .ok_or_else(|| format!("test: path {:?} nao existe", o.path))?;
            let esperado = valor()?;
            if *v != esperado {
                return Err(format!("test: {} e' {} e nao {}", o.path, v, esperado));
            }
            return Ok(None);
        }
        x => return Err(format!("op {:?}: use add, remove, replace ou test", x)),
    }))
}

/// Aplica a lista inteira a uma COPIA do show; so' comita se todas passarem e se o resultado
/// ainda for um Show valido (mesma via do `show_set`: deserializa e checa a versao).
fn patch(a: &ShowPatchArgs) -> Result<Value, String> {
    com_ro(|_, sh| {
        // dentro do lock de `OPEN`: entre a checagem e a gravacao ninguem troca o show.
        if let Some(r) = a.rev {
            if r != rev() {
                return Err(format!("rev {} != {}", r, rev()));
            }
        }
        let mut doc = json(sh)?;
        let mut undo: Vec<Value> = Vec::with_capacity(a.ops.len());
        for o in &a.ops {
            if let Some(u) = operar(&mut doc, o)? {
                undo.push(u);
            }
        }
        let novo: Show = serde_json::from_value(doc).map_err(|e| format!("show_patch: {}", e))?;
        let mut novo = show::migrate(novo)?;
        novo.extra.retain(|k, _| !k.starts_with('_'));
        *sh = novo;
        undo.reverse(); // ja' na ordem de aplicacao: o cliente manda de volta como veio
        Ok(json!({"rev": REV.fetch_add(1, Ordering::Relaxed) + 1, "undo": undo}))
    })
}

// ------------------------------------------------------------------ graph e face

/// O graph do show aberto (`extra.graph`), vazio quando falta. A CLI le daqui para o
/// `graph_check`: o engine nao conhece o crate `script` e por isso nao compila graph nenhum.
pub fn graph() -> Value {
    com_ro(|_, sh| Ok(sh.extra.get("graph").cloned()))
        .ok()
        .flatten()
        .unwrap_or_else(|| json!({"nodes": [], "edges": []}))
}

/// A face do show: o objeto inline de `extra.face`, ou `faces/<nome>.face.json` quando e' texto
/// (nome sem extensao, ou caminho). `null` quando o show nao tem face.
fn face() -> Result<Value, String> {
    let (spell, f) = com_ro(|p, sh| Ok((p.clone(), sh.extra.get("face").cloned())))?;
    match f {
        None | Some(Value::Null) => Ok(Value::Null),
        Some(Value::String(n)) => {
            let p = recurso_dir(&spell, "faces").join(format!("{}.face.json", n));
            let t = std::fs::read_to_string(&p)
                .map_err(|e| format!("face {:?}: {} ({})", n, e, p.display()))?;
            serde_json::from_str(&t).map_err(|e| format!("{}: {}", p.display(), e))
        }
        Some(v) => Ok(v),
    }
}

// ------------------------------------------------- cues e programmer (mesa)

/// Objeto cue do .spell, com as chaves validadas. Uma so' via para `cue_set` e `cue_capture`.
fn cue(
    name: &str,
    fade: f64,
    wait: f64,
    follow: bool,
    values: &Map<String, Value>,
) -> Result<Value, String> {
    if let Some(k) = values.keys().find(|k| cues::key(k).is_none()) {
        return Err(format!("cue: chave {:?} nao e' \"universo/endereco\"", k));
    }
    Ok(json!({"name": name, "fade": fade, "wait": wait, "follow": follow, "values": values}))
}

/// Poe a cue no show aberto: `index` substitui, sem `index` acrescenta. Devolve o indice.
fn cue_put(index: Option<usize>, c: Value) -> Result<Value, String> {
    com(|_, sh| {
        let cs = lista(sh, "cues");
        let i = match index {
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
}

/// Fixture do patch pelo nome, com o perfil ja carregado.
fn fixture(nome: &str) -> Result<(u16, u16, Perfil), String> {
    com(|p, sh| {
        let dir = recurso_dir(p, "profiles");
        let f = sh
            .extra
            .get("patch")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .find(|f| f["name"] == nome)
            .ok_or_else(|| format!("fixture {:?} nao esta no patch", nome))?;
        let pr = perfil(&dir, f["profile"].as_str().unwrap_or_default())?;
        let u = f["universe"].as_u64().unwrap_or(1) as u16;
        let a = f["address"]
            .as_u64()
            .ok_or_else(|| format!("{}: sem address", nome))? as u16;
        Ok((u, a, pr))
    })
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

#[derive(Deserialize, JsonSchema)]
pub struct PatchOp {
    /// add | remove | replace | test.
    pub op: String,
    /// JSON Pointer (RFC 6901) dentro do show: "/fps", "/tracks/-", "/tracks/0/keys/2",
    /// "/graph/nodes". O documento inteiro ("") nao e' alvo: para isso ha' show_set.
    pub path: String,
    /// Valor de add, replace e test; `remove` nao usa. Ausente nos tres primeiros = erro.
    #[serde(default)]
    pub value: Option<Value>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ShowPatchArgs {
    /// As operacoes, em ordem; a primeira que falhar cancela todas.
    pub ops: Vec<PatchOp>,
    /// A revisao que o cliente tinha; diferente da atual = recusa ("rev 3 != 5").
    #[serde(default)]
    pub rev: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ProfileGetArgs {
    /// Perfil em profiles/ (sem .json) ou caminho de um .json.
    pub name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct LevelSetArgs {
    #[serde(default = "um")]
    pub universe: u16,
    /// Primeiro canal DMX (1..512).
    pub address: u16,
    /// Valores 0..255 a partir de `address`; lista vazia escreve zero no canal.
    #[serde(default)]
    pub values: Vec<f64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct LevelArgs {
    /// Universo; ausente = todos.
    #[serde(default)]
    pub universe: Option<u16>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CueCaptureArgs {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub fade: f64,
    #[serde(default)]
    pub wait: f64,
    #[serde(default)]
    pub follow: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct FixtureSetArgs {
    /// Nome da fixture no patch.
    pub name: String,
    /// Nome do canal no perfil ("dim", "r"); numero nao resolve.
    pub channel: String,
    /// 0..255.
    pub value: f64,
}

// ------------------------------------------------------------------ comandos

pub fn register(r: &mut Registry) {
    r.add::<NoArgs>(
        "show_new",
        "Zera o show aberto: novo show, sACN no universo 1, 60 s. Devolve o show inteiro.",
        |_| {
            let sh = novo();
            let v = json(&sh)?;
            abre(String::new(), sh);
            Ok(v)
        },
    );
    r.add::<ShowSetArgs>(
        "show_set",
        "Substitui o show aberto pelo JSON dado (para IMPORTAR um show inteiro; para editar, prefira show_patch). Devolve o show inteiro.",
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
            com_ro(|p, sh| {
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
        |a| cue_put(a.index, cue(&a.name, a.fade, a.wait, a.follow, &a.values)?),
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
                let dir = recurso_dir(p, "profiles");
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
            com_ro(|p, sh| {
                let (rows, error) = checar(sh, &recurso_dir(p, "profiles"));
                Ok(json!({"rows": rows, "error": error}))
            })
        },
    );
    r.add::<NoArgs>("profiles", "Nomes dos perfis disponiveis em profiles/.", |_| {
        let dir = com_ro(|p, _| Ok(recurso_dir(p, "profiles")))?;
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
    r.add::<ShowPatchArgs>(
        "show_patch",
        "Edita o show aberto por JSON Patch (RFC 6902: add, remove, replace, test). Uma op que falha cancela todas. Devolve {rev, undo}: `undo` e' a lista de ops que volta ao estado anterior, ja' na ordem de aplicacao.",
        |a| patch(&a),
    );
    r.add::<NoArgs>(
        "graph_get",
        "O graph do show aberto (secao 10 do PRD: nodes e edges); vazio quando o show nao tem graph.",
        |_| Ok(graph()),
    );
    r.add::<NoArgs>(
        "face_get",
        "A face do show aberto: o objeto inline de `face`, ou faces/<nome>.face.json quando `face` e' texto. null quando o show nao tem face.",
        |_| face(),
    );

    r.add::<ProfileGetArgs>(
        "profile_get",
        "O perfil inteiro (nome, canais com offset, ranges e wheel) para o cliente montar os widgets.",
        |a| {
            // leitura: `com_ro` para nao subir `rev` (o barramento so' avisa a GUI em edicao)
            let dir = com_ro(|p, _| Ok(recurso_dir(p, "profiles")))?;
            Ok(perfil(&dir, &a.name)?.json)
        },
    );
    r.add::<LevelSetArgs>(
        "level_set",
        "Programmer: escreve valores no override manual, por cima da timeline (HTP). Exige player em execucao.",
        |a| {
            let v = if a.values.is_empty() {
                vec![0.0]
            } else {
                a.values
            };
            let h = vivo()?;
            h.level_set(a.universe, a.address, &v);
            Ok(json!(v.len()))
        },
    );
    r.add::<LevelArgs>(
        "level_clear",
        "Solta o override do programmer (um universo, ou todos sem universe). Devolve quantos canais sairam.",
        |a| Ok(json!(vivo()?.level_clear(a.universe))),
    );
    r.add::<LevelArgs>(
        "level_get",
        "Override do programmer como {\"universo/endereco\": valor}.",
        |a| Ok(Value::Object(niveis(&vivo()?, a.universe))),
    );
    r.add::<CueCaptureArgs>(
        "cue_capture",
        "Grava o override do programmer como uma cue nova no fim da lista e solta o override. Devolve o indice.",
        |a| {
            let h = vivo()?;
            let vals = niveis(&h, None);
            if vals.is_empty() {
                return Err("cue_capture: o programmer esta vazio".into());
            }
            let i = cue_put(None, cue(&a.name, a.fade, a.wait, a.follow, &vals)?)?;
            h.level_clear(None);
            Ok(i)
        },
    );
    r.add::<FixtureSetArgs>(
        "fixture_set",
        "Escreve num canal de uma fixture do patch pelo nome do canal no perfil (via level_set).",
        |a| {
            let (u, base, pr) = fixture(&a.name)?;
            let chans = pr.json["channels"].as_array().into_iter().flatten();
            let off = chans
                .clone()
                .find(|c| c["name"].as_str() == Some(a.channel.as_str()))
                .and_then(|c| c["offset"].as_u64())
                .ok_or_else(|| {
                    format!(
                        "{}: o perfil {:?} nao tem canal {:?}; tem {:?}",
                        a.name,
                        pr.name,
                        a.channel,
                        chans
                            .filter_map(|c| c["name"].as_str())
                            .collect::<Vec<_>>()
                    )
                })? as u16;
            let h = vivo()?;
            h.level_set(u, base + off, &[a.value]);
            Ok(json!({"universe": u, "address": base + off, "value": a.value}))
        },
    );
}

/// Override do programmer no formato de `values` de cue: {"universo/endereco": valor}.
fn niveis(h: &crate::player::Handle, u: Option<u16>) -> Map<String, Value> {
    h.levels(u)
        .into_iter()
        .map(|(u, a, v)| (format!("{}/{}", u, a), json!(v)))
        .collect()
}
