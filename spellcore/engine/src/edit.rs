//! Editing of the show open in this process (the registry `OPEN`): show, tracks, keyframes,
//! cues and patch. Port of `spellcaster/gui/api.py` plus the footprint check of
//! `fixtures/patch.py`. GUI and MCP edit through here; no edit logic lives outside the registry.

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

/// Empty show (the `NEW` of `gui/api.py`): sACN on universe 1, 60 s.
pub fn novo() -> Show {
    Show {
        name: "new show".into(),
        duration: Some(60.0),
        outputs: vec![OutputCfg::Sacn(show::Sacn {
            universes: vec![1],
            priority: 100,
            source_name: "Spellcaster".into(),
            interfaces: None,
        })],
        extra: json!({"patch": [], "cues": [], "markers": [], "in": 0.0, "out": 60.0})
            .as_object()
            .cloned()
            .unwrap_or_default(),
        ..Show::default()
    }
}

/// Runs `f` on the open show (path, show). With no open show, it opens a new one — Python is
/// born with `SHOW = NEW`, and the AI may call `track_add` before any `show_get`.
fn com_ro<T>(f: impl FnOnce(&mut String, &mut Show) -> Result<T, String>) -> Result<T, String> {
    let mut g = lock(&OPEN);
    let (p, sh) = g.get_or_insert_with(|| (String::new(), novo()));
    f(p, sh)
}

/// `com_ro` plus the counter: every successful edit bumps `rev`.
pub(crate) fn com<T>(
    f: impl FnOnce(&mut String, &mut Show) -> Result<T, String>,
) -> Result<T, String> {
    let v = com_ro(f)?;
    REV.fetch_add(1, Ordering::Relaxed);
    Ok(v)
}

/// Revision of the open show: it bumps on every edit. The bus broadcasts it; whoever sends
/// `show_patch` with a stale revision gets an error instead of overwriting someone else's edit.
// ponytail: a process counter, not a file one ; make it a hash of the show if two processes come
// to edit the same .spell.
static REV: AtomicU64 = AtomicU64::new(0);

pub fn rev() -> u64 {
    REV.load(Ordering::Relaxed)
}

/// Swaps the whole open show (`load`, `show_get {file}`, `show_new`): writes `OPEN` and bumps
/// `rev`. This is what CHANGES show; the `get_or_insert_with` of `com_ro` also writes `OPEN`, but
/// only to fill the empty slot with `novo()`. Without it the `rev` the client was holding would
/// still be valid on ANOTHER show, and its `show_patch` would land error-free on the wrong file.
pub(crate) fn abre(path: String, sh: Show) {
    *lock(&OPEN) = Some((path, sh));
    REV.fetch_add(1, Ordering::Relaxed);
}

fn json(sh: &Show) -> Result<Value, String> {
    serde_json::to_value(sh).map_err(|e| e.to_string())
}

/// Text that is JSON becomes JSON (`"255"`, `"[255,0,0]"`); the rest stays text (`"yellow"`).
/// It is Python's `_value`, for clients that only send strings (CLI, form field).
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
        .ok_or_else(|| format!("track {}: the show has {}", i, n))
}

fn keys(tr: &mut Value) -> Result<&mut Vec<Value>, String> {
    let o = tr
        .as_object_mut()
        .ok_or_else(|| "track is not a JSON object".to_string())?;
    let k = o.entry("keys").or_insert_with(|| json!([]));
    if !k.is_array() {
        *k = json!([]);
    }
    Ok(k.as_array_mut().expect("array"))
}

fn tempo(k: &Value) -> f64 {
    k.get(0).and_then(|v| v.as_f64()).unwrap_or(f64::NAN)
}

/// List in `extra` ("cues", "patch"), created empty when missing.
fn lista<'a>(sh: &'a mut Show, k: &str) -> &'a mut Vec<Value> {
    let v = sh.extra.entry(k.to_string()).or_insert_with(|| json!([]));
    if !v.is_array() {
        *v = json!([]);
    }
    v.as_array_mut().expect("array")
}

/// Creates or replaces the keyframe of the track at `t` (`|dt| < 1 us`), keeping the list
/// sorted. The one funnel of keyframe writing: the `key_set` command and the recording (`rec.rs`)
/// both go through here, and both bump `rev` — without it the recording would edit the show from
/// the outside and no client would know.
pub fn key_put(indice: usize, t: f64, value: Value, curve: &str) -> Result<Value, String> {
    if !CURVES.contains(&curve) {
        return Err(format!("curve {:?}: use {:?}", curve, CURVES));
    }
    com(|_, sh| {
        let ks = keys(track(sh, indice)?)?;
        let v = match value {
            Value::Null => json!(0),
            v => valor(v),
        };
        let k = json!([t, v, curve]);
        match ks.iter().position(|x| (tempo(x) - t).abs() < 1e-6) {
            Some(i) => ks[i] = k,
            None => ks.push(k),
        }
        ks.sort_by(|x, y| tempo(x).total_cmp(&tempo(y)));
        Ok(json!(ks))
    })
}

/// Universe, address and width (how many channels the keyframe covers) of a `dmx`/`artnet`
/// track of the open show. The width comes from the first keyframe held in a list; with no list,
/// a single channel.
// ponytail: width taken from the existing keyframe, not from a track field ; make it a `channels`
// field if recording into an empty 4-channel track becomes the common case.
pub fn track_dmx(indice: usize) -> Result<(u16, u16, usize), String> {
    com_ro(|_, sh| {
        let tr = track(sh, indice)?;
        let tipo = tr.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if tipo != "dmx" && tipo != "artnet" {
            return Err(format!("track {}: type {:?} is not dmx", indice, tipo));
        }
        let u = tr.get("universe").and_then(|v| v.as_u64()).unwrap_or(1) as u16;
        let a = tr.get("address").and_then(|v| v.as_u64()).unwrap_or(1) as u16;
        if a == 0 || a > 512 {
            return Err(format!("track {}: address {} outside 1..512", indice, a));
        }
        let w = tr
            .get("keys")
            .and_then(|v| v.as_array())
            .and_then(|ks| ks.iter().find_map(|k| k.get(1).and_then(|v| v.as_array())))
            .map(|l| l.len().max(1))
            .unwrap_or(1);
        Ok((u, a, w.min(512 - a as usize + 1)))
    })
}

// ------------------------------------------------------------------ patch

struct Perfil {
    name: String,
    size: u16,
    json: Value,
}

/// Profile by name in `dir` (without .json) or by path.
// ponytail: ranges (`ranges`) and wheel (`wheel`) travel only in the raw `json`, for the client
// to draw ; they become a type here once per-channel-type fade exists.
fn perfil(dir: &Path, p: &str) -> Result<Perfil, String> {
    let f = if Path::new(p).extension().is_some() {
        PathBuf::from(p)
    } else {
        dir.join(format!("{}.json", p))
    };
    let txt = std::fs::read_to_string(&f)
        .map_err(|_| format!("profile {:?} not found ({})", p, f.display()))?;
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
    let m = max.ok_or_else(|| format!("{}: no channels", name))?;
    if m > 511 {
        return Err(format!("{}: offset {} outside 0..511", name, m));
    }
    Ok(Perfil {
        name,
        size: m as u16 + 1,
        json: v,
    })
}

/// Resource folder of the show (`profiles/`, `faces/`, `modules/`): next to the .spell, one
/// level up (`shows/` and `profiles/` as siblings, as in the repo and on the USB stick), in the
/// cwd or next to the executable — the first one that exists.
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

/// One grid row per fixture; it stops at the first error (missing profile, past 512, overlap —
/// the 17 ch on a 16 spacing bug shows up here).
fn checar(sh: &Show, dir: &Path) -> (Vec<Value>, Option<String>) {
    let mut rows: Vec<Value> = Vec::new();
    let mut busy: HashMap<(u16, u16), usize> = HashMap::new(); // (universe, channel) -> row
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
        .ok_or_else(|| "fixture without name".to_string())?;
    if rows.iter().any(|r| r["name"] == name) {
        return Err(format!("fixture {:?} is already in the patch", name));
    }
    let prof = f["profile"]
        .as_str()
        .ok_or_else(|| format!("{}: no profile", name))?;
    let p = perfil(dir, prof)?;
    let u = f["universe"].as_u64().unwrap_or(1) as u16;
    let a = f["address"]
        .as_u64()
        .ok_or_else(|| format!("{}: no address", name))? as u16;
    if a < 1 || a as u32 + p.size as u32 - 1 > 512 {
        return Err(format!(
            "{} [{}]: address {} + {} ch runs past 512 (universe {})",
            name, p.name, a, p.size, u
        ));
    }
    for ch in a..a + p.size {
        if let Some(&i) = busy.get(&(u, ch)) {
            let o = &rows[i];
            return Err(format!(
                "overlap on universe {} channel {}: {:?} [{}, {} ch at {}] and {:?} [{}, {} ch at {}]",
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

/// Splits "/a/b/c" into ("/a/b", "c"), with the last token free of the RFC 6901 escapes.
fn dividir(path: &str) -> Result<(&str, String), String> {
    let i = path
        .rfind('/')
        .ok_or_else(|| format!("path {:?}: a JSON Pointer starts with /", path))?;
    Ok((
        &path[..i],
        path[i + 1..].replace("~1", "/").replace("~0", "~"),
    ))
}

fn indice(n: usize, tok: &str, path: &str, inserindo: bool) -> Result<usize, String> {
    let i = if inserindo && tok == "-" {
        n
    } else {
        tok.parse::<usize>()
            .map_err(|_| format!("path {:?}: {:?} is not a list index", path, tok))?
    };
    if i > n || (!inserindo && i == n) {
        return Err(format!(
            "path {:?}: index {} outside the list of {}",
            path, i, n
        ));
    }
    Ok(i)
}

fn pai<'a>(doc: &'a mut Value, path: &str, p: &str) -> Result<&'a mut Value, String> {
    doc.pointer_mut(p)
        .ok_or_else(|| format!("path {:?}: {:?} does not exist", path, p))
}

/// `add`: inserts into the list ("-" = end) or writes the object key. Returns the path with the
/// index already resolved (the inverse cannot say "-") and the value that was there, if any.
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
        _ => Err(format!(
            "path {:?}: {:?} is neither object nor list",
            path, p
        )),
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
            .ok_or_else(|| format!("path {:?} does not exist", path)),
        _ => Err(format!(
            "path {:?}: {:?} is neither object nor list",
            path, p
        )),
    }
}

/// One operation; returns the operation that undoes it (`test` undoes nothing).
fn operar(doc: &mut Value, o: &PatchOp) -> Result<Option<Value>, String> {
    let valor = || {
        o.value
            .clone()
            .ok_or_else(|| format!("op {:?} at {:?}: value is missing", o.op, o.path))
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
                .ok_or_else(|| format!("path {:?} does not exist", o.path))?;
            let v = std::mem::replace(alvo, valor()?);
            json!({"op": "replace", "path": o.path, "value": v})
        }
        "test" => {
            let v = doc
                .pointer(&o.path)
                .ok_or_else(|| format!("test: path {:?} does not exist", o.path))?;
            let esperado = valor()?;
            if *v != esperado {
                return Err(format!("test: {} is {} and not {}", o.path, v, esperado));
            }
            return Ok(None);
        }
        x => return Err(format!("op {:?}: use add, remove, replace or test", x)),
    }))
}

/// Applies the whole list to a COPY of the show; it only commits if all of them pass and if the
/// result is still a valid Show (the same path as `show_set`: deserialize and check the version).
fn patch(a: &ShowPatchArgs) -> Result<Value, String> {
    com_ro(|_, sh| {
        // inside the `OPEN` lock: between the check and the write nobody swaps the show.
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
        undo.reverse(); // already in application order: the client sends it back as it came
        Ok(json!({"rev": REV.fetch_add(1, Ordering::Relaxed) + 1, "undo": undo}))
    })
}

/// In-Out range of the open show (`in`/`out`); without them, 0..duration. It is the loop range:
/// the player keeps only the pair of numbers, and what knows where it is is the open show.
pub fn intervalo() -> (f64, f64) {
    com_ro(|_, sh| {
        let n = |k: &str| sh.extra.get(k).and_then(Value::as_f64);
        Ok((
            n("in").unwrap_or(0.0).max(0.0),
            n("out").or(sh.duration).unwrap_or(0.0),
        ))
    })
    .unwrap_or((0.0, 0.0))
}

// --------------------------------------------------------------- graph and face

/// The graph of the open show (`extra.graph`), empty when missing. The CLI reads it from here for
/// `graph_check`: the engine does not know the `script` crate and so compiles no graph at all.
pub fn graph() -> Value {
    com_ro(|_, sh| Ok(sh.extra.get("graph").cloned()))
        .ok()
        .flatten()
        .unwrap_or_else(|| json!({"nodes": [], "edges": []}))
}

/// The face of the show: the inline object of `extra.face`, or `faces/<name>.face.json` when it
/// is text (name without extension, or path). `null` when the show has no face.
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

// --------------------------------------------- cues and programmer (console)

/// The .spell cue object, with the keys validated. One single path for `cue_set` and `cue_capture`.
fn cue(
    name: &str,
    fade: f64,
    wait: f64,
    follow: bool,
    values: &Map<String, Value>,
) -> Result<Value, String> {
    if let Some(k) = values.keys().find(|k| cues::key(k).is_none()) {
        return Err(format!("cue: key {:?} is not \"universe/address\"", k));
    }
    Ok(json!({"name": name, "fade": fade, "wait": wait, "follow": follow, "values": values}))
}

/// Puts the cue into the open show: `index` replaces, without `index` it appends. Returns the index.
fn cue_put(index: Option<usize>, c: Value) -> Result<Value, String> {
    com(|_, sh| {
        let cs = lista(sh, "cues");
        let i = match index {
            Some(i) if i < cs.len() => {
                cs[i] = c;
                i
            }
            Some(i) => return Err(format!("cue {}: the show has {}", i, cs.len())),
            None => {
                cs.push(c);
                cs.len() - 1
            }
        };
        Ok(json!(i))
    })
}

/// Fixture of the patch by name, with the profile already loaded.
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
            .ok_or_else(|| format!("fixture {:?} is not in the patch", nome))?;
        let pr = perfil(&dir, f["profile"].as_str().unwrap_or_default())?;
        let u = f["universe"].as_u64().unwrap_or(1) as u16;
        let a = f["address"]
            .as_u64()
            .ok_or_else(|| format!("{}: no address", nome))? as u16;
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
    /// The whole show: JSON object (or JSON text).
    pub data: Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct ShowSaveArgs {
    /// Path of the .spell; empty = the one from the last load/show_get/show_save.
    #[serde(default)]
    pub file: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TrackAddArgs {
    /// dmx | artnet | osc | media | cue | fx | laser.
    #[serde(default = "dmx", rename = "type")]
    #[schemars(rename = "type")]
    pub kind: String,
    /// Output universe, from 1 on.
    #[serde(default = "um")]
    pub universe: u16,
    /// DMX address 1..512 (osc tracks use text: go through show_set).
    #[serde(default = "um")]
    pub address: u16,
    /// Track name (the .spell `name` field). The `label` argument is the old name of this one
    /// (deprecated, gone next round).
    // ponytail: `label` alias for one round, for scripts and MCP sessions already written (no
    // caller left in the repo) ; drop it in round 3.
    #[serde(default, alias = "label")]
    pub name: String,
    /// `.ild` clip of the `laser` track (the `clip` field).
    #[serde(default)]
    pub clip: String,
    /// `.rhai` script of the `fx` track (the `script` field).
    #[serde(default)]
    pub script: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct TrackDelArgs {
    /// Index of the track in `tracks`.
    pub index: usize,
}

#[derive(Deserialize, JsonSchema)]
pub struct KeySetArgs {
    /// Index of the track.
    pub track: usize,
    /// Instant in seconds.
    pub t: f64,
    /// 255, [255, 0, 0] or text ("play"); absent = 0.
    #[serde(default)]
    pub value: Value,
    /// linear | hold | in | out | inout | bezier (curve of the segment that ARRIVES at this keyframe).
    #[serde(default = "linear")]
    pub curve: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct KeyDelArgs {
    /// Index of the track in `tracks`.
    pub track: usize,
    /// Instant of the keyframe (1 ms tolerance).
    pub t: f64,
}

#[derive(Deserialize, JsonSchema)]
pub struct CueSetArgs {
    /// Index of the cue to replace; absent = appended at the end.
    #[serde(default)]
    pub index: Option<usize>,
    /// Cue name, as it shows in the list.
    #[serde(default)]
    pub name: String,
    /// Seconds of linear fade to the values.
    #[serde(default)]
    pub fade: f64,
    /// Seconds between the GO and the start of the fade.
    #[serde(default)]
    pub wait: f64,
    /// When it ends, it fires the next one.
    #[serde(default)]
    pub follow: bool,
    /// {"universe/address": value or [values]}; "100" = universe 1.
    #[serde(default)]
    pub values: Map<String, Value>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CueDelArgs {
    /// Index of the cue in `cues`.
    pub index: usize,
}

#[derive(Deserialize, JsonSchema)]
pub struct PatchAddArgs {
    /// Unique fixture name in the show.
    pub name: String,
    /// Profile in profiles/ (without .json) or path of a .json.
    pub profile: String,
    /// Output universe, from 1 on.
    #[serde(default = "um")]
    pub universe: u16,
    /// First DMX channel (1..512).
    #[serde(default = "um")]
    pub address: u16,
}

#[derive(Deserialize, JsonSchema)]
pub struct PatchDelArgs {
    /// Fixture name in the patch.
    pub name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct PatchOp {
    /// add | remove | replace | test.
    pub op: String,
    /// JSON Pointer (RFC 6901) inside the show: "/fps", "/tracks/-", "/tracks/0/keys/2",
    /// "/graph/nodes". The whole document ("") is not a target: show_set is there for that.
    pub path: String,
    /// Value of add, replace and test; `remove` does not use it. Absent on the first three = error.
    #[serde(default)]
    pub value: Option<Value>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ShowPatchArgs {
    /// The operations, in order; the first one that fails cancels them all.
    pub ops: Vec<PatchOp>,
    /// The revision the client held; different from the current one = refused ("rev 3 != 5").
    #[serde(default)]
    pub rev: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ProfileGetArgs {
    /// Profile in profiles/ (without .json) or path of a .json.
    pub name: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct LevelSetArgs {
    /// Output universe, from 1 on.
    #[serde(default = "um")]
    pub universe: u16,
    /// First DMX channel (1..512).
    pub address: u16,
    /// Values 0..255 from `address` on; an empty list writes zero on the channel.
    #[serde(default)]
    pub values: Vec<f64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct LevelArgs {
    /// Universe; absent = all of them.
    #[serde(default)]
    pub universe: Option<u16>,
}

#[derive(Deserialize, JsonSchema)]
pub struct CueCaptureArgs {
    /// Name of the new cue.
    #[serde(default)]
    pub name: String,
    /// Seconds of linear fade to the values.
    #[serde(default)]
    pub fade: f64,
    /// Seconds between the GO and the start of the fade.
    #[serde(default)]
    pub wait: f64,
    /// When it ends, it fires the next one.
    #[serde(default)]
    pub follow: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct FixtureSetArgs {
    /// Fixture name in the patch.
    pub name: String,
    /// Channel name in the profile ("dim", "r"); a number does not resolve.
    pub channel: String,
    /// 0..255.
    pub value: f64,
}

// ------------------------------------------------------------------ commands

pub fn register(r: &mut Registry) {
    r.add::<NoArgs>(
        "show_new",
        "Clears the open show: new show, sACN on universe 1, 60 s. Returns the whole show.",
        |_| {
            let sh = novo();
            let v = json(&sh)?;
            abre(String::new(), sh);
            Ok(v)
        },
    );
    r.add::<ShowSetArgs>(
        "show_set",
        "Replaces the open show with the given JSON (to IMPORT a whole show; to edit, prefer show_patch). Returns the whole show.",
        |a| {
            let v = valor(a.data);
            if !v.is_object() {
                return Err("show_set: expected a JSON object".into());
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
        "Saves the open show (without file, at the path of the last one opened). Returns the path.",
        |a| {
            com_ro(|p, sh| {
                let f = if a.file.is_empty() {
                    p.clone()
                } else {
                    a.file.clone()
                };
                if f.is_empty() {
                    return Err("show_save: no path (pass file=)".into());
                }
                show::save(Path::new(&f), sh)?;
                *p = f.clone();
                Ok(json!(f))
            })
        },
    );
    r.add::<TrackAddArgs>(
        "track_add",
        "Appends an empty track to the open show (type dmx|artnet|osc|media|cue|fx|laser; clip for laser, script for fx). Returns the index of the track. The `label` argument is the old name of `name` (deprecated, gone next round).",
        |a| {
            com(|_, sh| {
                let mut tr = json!({"type": a.kind, "universe": a.universe,
                                    "address": a.address, "keys": []});
                if !a.name.is_empty() {
                    tr["name"] = json!(a.name);
                }
                if !a.clip.is_empty() {
                    tr["clip"] = json!(a.clip);
                }
                if !a.script.is_empty() {
                    tr["script"] = json!(a.script);
                }
                sh.tracks.push(tr);
                Ok(json!(sh.tracks.len() - 1))
            })
        },
    );
    r.add::<TrackDelArgs>(
        "track_del",
        "Removes the track at the given index. Returns the removed track.",
        |a| {
            com(|_, sh| {
                track(sh, a.index)?;
                Ok(sh.tracks.remove(a.index))
            })
        },
    );
    r.add::<KeySetArgs>(
        "key_set",
        "Creates or replaces the keyframe of the track at t. Returns the keyframes of the track.",
        |a| key_put(a.track, a.t, a.value, &a.curve),
    );
    r.add::<KeyDelArgs>(
        "key_del",
        "Deletes the keyframe of the track at t (1 ms tolerance). Returns how many were removed.",
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
        "Creates (without index) or replaces a cue: name, fade, wait, follow and DMX values. Returns the index.",
        |a| cue_put(a.index, cue(&a.name, a.fade, a.wait, a.follow, &a.values)?),
    );
    r.add::<CueDelArgs>(
        "cue_del",
        "Removes the cue at the given index. Returns the removed cue.",
        |a| {
            com(|_, sh| {
                let cs = lista(sh, "cues");
                if a.index >= cs.len() {
                    return Err(format!("cue {}: the show has {}", a.index, cs.len()));
                }
                Ok(cs.remove(a.index))
            })
        },
    );
    r.add::<PatchAddArgs>(
        "patch_add",
        "Patches a fixture (profile, universe, address); refuses overlap and running past 512. Returns the patch grid.",
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
        "Removes the fixture from the patch by name. Returns the removed entry.",
        |a| {
            com(|_, sh| {
                let ps = lista(sh, "patch");
                match ps.iter().position(|f| f["name"] == a.name) {
                    Some(i) => Ok(ps.remove(i)),
                    None => Err(format!("fixture {:?} is not in the patch", a.name)),
                }
            })
        },
    );
    r.add::<NoArgs>(
        "patch_check",
        "Patch grid of the open show (name, profile, universe, address, channels) and the overlap error, if there is one.",
        |_| {
            com_ro(|p, sh| {
                let (rows, error) = checar(sh, &recurso_dir(p, "profiles"));
                Ok(json!({"rows": rows, "error": error}))
            })
        },
    );
    r.add::<NoArgs>(
        "profiles",
        "Names of the profiles available in profiles/.",
        |_| {
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
        },
    );
    r.add::<ShowPatchArgs>(
        "show_patch",
        "Edits the open show by JSON Patch (RFC 6902: add, remove, replace, test). One failing op cancels them all. Returns {rev, undo}: `undo` is the list of ops that goes back to the previous state, already in application order.",
        |a| patch(&a),
    );
    r.add::<NoArgs>(
        "graph_get",
        "The graph of the open show (PRD section 10: nodes and edges); empty when the show has no graph.",
        |_| Ok(graph()),
    );
    r.add::<NoArgs>(
        "face_get",
        "The face of the open show: the inline object of `face`, or faces/<name>.face.json when `face` is text. null when the show has no face.",
        |_| face(),
    );

    r.add::<ProfileGetArgs>(
        "profile_get",
        "The whole profile (name, channels with offset, ranges and wheel) for the client to build the widgets.",
        |a| {
            // read: `com_ro` so it does not bump `rev` (the bus only tells the GUI on an edit)
            let dir = com_ro(|p, _| Ok(recurso_dir(p, "profiles")))?;
            Ok(perfil(&dir, &a.name)?.json)
        },
    );
    r.add::<LevelSetArgs>(
        "level_set",
        "Programmer: writes values into the manual override, on top of the timeline (HTP). Requires a running player.",
        |a| {
            let v = if a.values.is_empty() {
                vec![0.0]
            } else {
                a.values
            };
            let h = vivo()?;
            h.level_set(a.universe, a.address, &v)?;
            Ok(json!(v.len()))
        },
    );
    r.add::<LevelArgs>(
        "level_clear",
        "Releases the programmer override (one universe, or all of them without universe). Returns how many channels were freed.",
        |a| Ok(json!(vivo()?.level_clear(a.universe))),
    );
    r.add::<LevelArgs>(
        "level_get",
        "Programmer override as {\"universe/address\": value}.",
        |a| Ok(Value::Object(niveis(&vivo()?, a.universe))),
    );
    r.add::<CueCaptureArgs>(
        "cue_capture",
        "Stores the programmer override as a new cue at the end of the list and releases the override. Returns the index.",
        |a| {
            let h = vivo()?;
            let vals = niveis(&h, None);
            if vals.is_empty() {
                return Err("cue_capture: the programmer is empty".into());
            }
            let i = cue_put(None, cue(&a.name, a.fade, a.wait, a.follow, &vals)?)?;
            h.level_clear(None);
            Ok(i)
        },
    );
    r.add::<FixtureSetArgs>(
        "fixture_set",
        "Writes into a channel of a patched fixture by the channel name in the profile (via level_set).",
        |a| {
            let (u, base, pr) = fixture(&a.name)?;
            let chans = pr.json["channels"].as_array().into_iter().flatten();
            let off = chans
                .clone()
                .find(|c| c["name"].as_str() == Some(a.channel.as_str()))
                .and_then(|c| c["offset"].as_u64())
                .ok_or_else(|| {
                    format!(
                        "{}: profile {:?} has no channel {:?}; it has {:?}",
                        a.name,
                        pr.name,
                        a.channel,
                        chans.filter_map(|c| c["name"].as_str()).collect::<Vec<_>>()
                    )
                })? as u16;
            let h = vivo()?;
            h.level_set(u, base + off, &[a.value])?;
            Ok(json!({"universe": u, "address": base + off, "value": a.value}))
        },
    );
}

/// Programmer override in the cue `values` format: {"universe/address": value}.
fn niveis(h: &crate::player::Handle, u: Option<u16>) -> Map<String, Value> {
    h.levels(u)
        .into_iter()
        .map(|(u, a, v)| (format!("{}/{}", u, a), json!(v)))
        .collect()
}
