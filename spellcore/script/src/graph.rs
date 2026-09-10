//! Graph runtime of section 10 of the PRD: show behavior (what each button, key, OSC, MIDI,
//! timer or marker does), living in the `.spell` and running in the engine with or without GUI.
//!
//! CLOSED catalog. A new request becomes a track, not a node.
//!
//! | type | config | input pins | output pins |
//! |---|---|---|---|
//! | `in.widget`      | `widget`                        | -            | `press` |
//! | `in.key`         | `key`                           | -            | `down`  |
//! | `in.osc`         | `address`                       | -            | `out`   |
//! | `in.midi`        | `midi` ("144/60")               | -            | `out`   |
//! | `in.marker`      | `marker`                        | -            | `out`   |
//! | `in.timer`       | `every` (s)                     | -            | `out`   |
//! | `in.state`       | `what`: "t" \| "dmx" + `universe`/`address` | - | `out` |
//! | `logic.and/or`   | -                               | `a`, `b`     | `out`   |
//! | `logic.not`      | -                               | `in`         | `out`   |
//! | `logic.latch`    | -                               | `set`,`reset`| `out`   |
//! | `logic.toggle`   | -                               | `in`         | `out`   |
//! | `logic.debounce` | `ms`                            | `in`         | `out`   |
//! | `logic.counter`  | `step`                          | `in`,`reset` | `out`   |
//! | `logic.select`   | -                               | `a`,`b`,`sel`| `out`   |
//! | `math.map`       | `in_min`,`in_max`,`out_min`,`out_max`,`clamp` | `in` | `out` |
//! | `math.curve`     | `curve`, `c`                    | `in`         | `out`   |
//! | `math.expr`      | `expr` (one line of Rhai, `a` and `b` in scope) | `a`,`b` | `out` |
//! | `time.delay`     | `ms`                            | `in`         | `out`   |
//! | `time.hold`      | `ms`                            | `in`         | `out`   |
//! | `cmd`            | `cmd`, `args`                   | `trigger`    | `done`  |
//! | `out.widget`     | `widget`,`prop`,`hold_ms`       | `in`         | -       |
//! | `out.osc`        | `address`                       | `in`         | -       |
//! | `out.param`      | `target`                        | `in`         | -       |
//! | `out.notify`     | `text`                          | `in`         | -       |
//! | `state`          | `group` ("main"), `initial`     | `enter`,`exit` | `active` |
//! | `module`         | `module` (name of `modules/<name>.json`) | one per `parameter`, then one per `command` | one per `value` |
//!
//! The signal is `f64`; "on" is `>= 0.5`; a "rising edge" is going from `< 0.5` to `>= 0.5`.
//! An EVENT input node gives a ONE frame pulse with the value received in
//! `FrameHook::input`; `in.timer` and `in.state` work by level.
//!
//! # State and mute (proposal of this round; see `design/DECISOES.md`)
//!
//! Any node accepts two extra keys: `"mute": true` (the node does not emit) and
//! `"state": "<id of a state node>"` (the node only emits while that state is active).
//! "Does not emit" is the same thing in both cases: the node outputs stay at 0, no `Ev` goes
//! out and the pending `time.delay` is cancelled. On return the node comes back as it was (the
//! `q` of the toggle and of the latch, the `n` of the counter), with the edges zeroed and the
//! next value of a LEVEL output (`out.osc`, `out.param`, `out.widget`, `module`) re-emitted:
//! it is Chataigne's "re-emit on activation". A trigger already high on the frame the state
//! opens fires once.
//!
//! `state` is ONE active per `group`: a pulse on `enter` turns this one on and the others of
//! the same group off; a pulse on `exit` turns it off; `reset()` goes back to `initial`. A
//! silenced `state` node does not emit either: its `active` comes out 0, even though the
//! machine keeps the state internally.
// ponytail: the state applies from the point where the `state` node is evaluated, so a gated
// node that comes BEFORE it in the topological order sees the previous frame value ; make it a
// pre-pass of the `state` nodes only if some real show depends on the exact frame.
//!
//! `module` is the app declared in `modules/<name>.json`, read by `engine::module::load` (the
//! same folder and the same typed `Module` of the `module_*` commands; the graph has no reader
//! of its own). Each NUMERIC `parameter` is an INPUT: the value changed ->
//! `Ev::Param{target:"<module>/<path>"}`, with `norm` mapping the 0..1 signal to `[n0,n1]`
//! before the clamp into `min`..`max`. Each `value` is a level OUTPUT fed by
//! `FrameHook::input("module:<module>/<path>", v)`. Each `command` is a trigger input ->
//! `Ev::Cmd{name:"<module>/<cmd>", args:{}}`. An `enum` or `string` parameter does NOT become
//! a pin: `Ev::Param` carries an `f64`.
//!
//! JSON: `{"nodes":[{"id","type",...}], "edges":[["node.pin","node.pin"], ...]}`.
//! Compiles to a list of nodes in topological order with the pins indexed by INTEGER
//! (no text lookup on the hot path). A cycle is a compilation error.

use engine::hook::{Ev, EventSink, FrameHook};
use engine::module::Module;
use engine::{Curve, Universes};
use rhai::{Dynamic, Engine as Rhai, Scope, AST};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::path::Path;

/// Input and output event queues, pre-allocated.
// ponytail: 64 events per frame in each direction ; one GO and one OSC per frame are orders of
// magnitude below that. Raise it when some real graph touches the ceiling (the `perdidos`
// counter warns).
const FILA: usize = 64;

/// Pending items of `time.delay`.
// ponytail: 8 changes in flight per delay node ; make it a dynamic queue if a real graph
// overflows it.
const DELAY_N: usize = 8;

enum Kind {
    /// in.widget / in.key / in.osc / in.midi / in.marker: the value arrives through the queue.
    Evento,
    Timer {
        every: f64,
        prox: f64,
    },
    EstadoT,
    EstadoDmx {
        uni: u16,
        addr: u16,
    },
    And,
    Or,
    Not,
    Latch {
        q: f64,
    },
    Toggle {
        q: f64,
        p: f64,
    },
    Debounce {
        win: f64,
        ult: f64,
        p: f64,
    },
    Counter {
        passo: f64,
        n: f64,
        p: f64,
        pr: f64,
    },
    Select,
    Map {
        i0: f64,
        i1: f64,
        o0: f64,
        o1: f64,
        clamp: bool,
    },
    Curva {
        c: Curve,
        cc: (f64, f64),
    },
    Expr {
        ast: AST,
        avisou: bool,
    },
    Delay {
        d: f64,
        fila: [(f64, f64); DELAY_N],
        n: usize,
        ult: f64,
        p: f64,
    },
    Hold {
        d: f64,
        ate: f64,
        p: f64,
    },
    Cmd {
        nome: String,
        args: Value,
        p: f64,
    },
    SaiWidget {
        id: String,
        prop: String,
        hold: f64,
        ate: f64,
        aceso: bool,
        p: f64,
        ult: f64,
    },
    SaiOsc {
        addr: String,
        ult: f64,
    },
    SaiParam {
        alvo: String,
        ult: f64,
    },
    SaiNotify {
        txt: String,
        p: f64,
    },
    /// State machine: `i` is the index of this state in `Graph::estados`.
    Estado {
        i: usize,
        pe: f64,
        px: f64,
    },
    /// Declared app: one input per parameter, then one per command; one output per value.
    Modulo {
        params: Vec<Par>,
        cmds: Vec<(String, f64)>,
    },
}

/// A `parameter` of a `module` node, already resolved for the hot path.
struct Par {
    alvo: String,
    norm: Option<(f64, f64)>,
    min: f64,
    max: f64,
    ult: f64,
}

struct No {
    id: String,
    kind: Kind,
    base: usize,
    nin: usize,
    nout: usize,
    /// `"mute": true` in the JSON: the node does not emit.
    mute: bool,
    /// `"state": "<id>"` in the JSON: index in `Graph::estados` of the state that enables this
    /// node.
    estado: Option<usize>,
    /// edges that ARRIVE at this node: (source slot, destination slot)
    ins: Vec<(usize, usize)>,
}

/// Input and output pins of each type in the catalog.
fn pinos(t: &str) -> Option<(&'static [&'static str], &'static [&'static str])> {
    Some(match t {
        "in.widget" => (&[], &["press"]),
        "in.key" => (&[], &["down"]),
        "in.osc" | "in.midi" | "in.marker" | "in.timer" | "in.state" => (&[], &["out"]),
        "logic.and" | "logic.or" => (&["a", "b"], &["out"]),
        "logic.not" | "logic.toggle" | "logic.debounce" | "math.map" | "math.curve"
        | "time.delay" | "time.hold" => (&["in"], &["out"]),
        "logic.latch" => (&["set", "reset"], &["out"]),
        "logic.counter" => (&["in", "reset"], &["out"]),
        "logic.select" => (&["a", "b", "sel"], &["out"]),
        "math.expr" => (&["a", "b"], &["out"]),
        "cmd" => (&["trigger"], &["done"]),
        "out.widget" | "out.osc" | "out.param" | "out.notify" => (&["in"], &[]),
        "state" => (&["enter", "exit"], &["active"]),
        // "module": pins come from modules/<name>.json, resolved in `pinos_no`
        "module" => (&[], &[]),
        _ => return None,
    })
}

/// The show's `modules/<name>.json`, by the same rule as `profiles/` and `faces/`: the `engine`
/// resolves the folder and reads the typed manifest; the graph has no module.json reader.
// ponytail: `base_dir` is the show directory, and `recurso_dir` wants the .spell path ; the
// synthetic name only exists to give it the right parent - it goes away when the Graph gets
// the .spell.
fn carrega_modulo(base: &Path, nome: &str) -> Result<Module, String> {
    let spell = base.join("show.spell");
    let dir = engine::module::modules_dir(&spell.to_string_lossy());
    engine::module::load(&dir.join(format!("{nome}.json")))
        .map_err(|e| format!("module {nome:?}: {e}"))
}

/// Parameters that become pins: only the numeric ones.
// ponytail: numeric pins only ; enum/string come in when `Ev::Param` carries a `Value`.
fn params_num(m: &Module) -> impl Iterator<Item = (&String, &engine::module::Param)> {
    m.parameters
        .iter()
        .filter(|(_, p)| p.r#type != "enum" && p.r#type != "string")
}

/// Pins of this node. The same as the type's, except `module`, which takes them from the
/// manifest (`BTreeMap`: the key order is stable, and the slot layout depends on it).
fn pinos_no(tipo: &str, m: Option<&Module>) -> (Vec<String>, Vec<String>) {
    match m {
        Some(m) => {
            let mut ins: Vec<String> = params_num(m).map(|(k, _)| k.clone()).collect();
            ins.extend(m.commands.keys().cloned());
            (ins, m.values.keys().cloned().collect())
        }
        None => {
            let (i, o) = pinos(tipo).unwrap();
            (
                i.iter().map(|s| s.to_string()).collect(),
                o.iter().map(|s| s.to_string()).collect(),
            )
        }
    }
}

fn txt(n: &Value, k: &str, pad: &str) -> String {
    n.get(k).and_then(|v| v.as_str()).unwrap_or(pad).to_string()
}

fn f(n: &Value, k: &str, pad: f64) -> f64 {
    n.get(k).and_then(|v| v.as_f64()).unwrap_or(pad)
}

fn b(n: &Value, k: &str, pad: bool) -> bool {
    n.get(k).and_then(|v| v.as_bool()).unwrap_or(pad)
}

/// Event key this node listens to ("widget:go", "key:Space", ...), if it is an event node.
fn chave(tipo: &str, n: &Value) -> Option<String> {
    Some(match tipo {
        "in.widget" => format!("widget:{}", txt(n, "widget", "")),
        "in.key" => format!("key:{}", txt(n, "key", "")),
        "in.osc" => format!("osc:{}", txt(n, "address", "")),
        "in.midi" => format!("midi:{}", txt(n, "midi", "")),
        "in.marker" => format!("marker:{}", txt(n, "marker", "")),
        _ => return None,
    })
}

fn monta(
    tipo: &str,
    n: &Value,
    rhai: &mut Option<(Rhai, Scope<'static>)>,
    m: Option<&Module>,
    // if this node IS a `state`, its index in `Graph::estados`
    meu: Option<usize>,
) -> Result<Kind, String> {
    Ok(match tipo {
        "in.widget" | "in.key" | "in.osc" | "in.midi" | "in.marker" => Kind::Evento,
        "in.timer" => Kind::Timer {
            every: f(n, "every", 1.0).max(1e-6),
            prox: 0.0,
        },
        "in.state" => match txt(n, "what", "t").as_str() {
            "t" => Kind::EstadoT,
            "dmx" => Kind::EstadoDmx {
                uni: f(n, "universe", 1.0) as u16,
                addr: (f(n, "address", 1.0) as u16).clamp(1, 512),
            },
            // ponytail: in.state only reads "t" and a DMX channel from the frame Universes ;
            // cue, fixture and the rest come in when the FrameHook receives the player Handle.
            o => {
                return Err(format!(
                    "in.state: unknown \"what\": {o} (use \"t\" or \"dmx\")"
                ))
            }
        },
        "logic.and" => Kind::And,
        "logic.or" => Kind::Or,
        "logic.not" => Kind::Not,
        "logic.latch" => Kind::Latch { q: 0.0 },
        "logic.toggle" => Kind::Toggle { q: 0.0, p: 0.0 },
        "logic.debounce" => Kind::Debounce {
            win: f(n, "ms", 200.0) / 1000.0,
            ult: f64::NEG_INFINITY,
            p: 0.0,
        },
        "logic.counter" => Kind::Counter {
            passo: f(n, "step", 1.0),
            n: 0.0,
            p: 0.0,
            pr: 0.0,
        },
        "logic.select" => Kind::Select,
        "math.map" => Kind::Map {
            i0: f(n, "in_min", 0.0),
            i1: f(n, "in_max", 1.0),
            o0: f(n, "out_min", 0.0),
            o1: f(n, "out_max", 1.0),
            clamp: b(n, "clamp", true),
        },
        "math.curve" => {
            let c = n
                .get("c")
                .and_then(|v| v.as_array())
                .filter(|a| a.len() == 2)
                .map(|a| (a[0].as_f64().unwrap_or(0.42), a[1].as_f64().unwrap_or(0.58)))
                .unwrap_or(engine::BEZ);
            Kind::Curva {
                c: Curve::from_str(&txt(n, "curve", "linear")),
                cc: c,
            }
        }
        "math.expr" => {
            let src = txt(n, "expr", "a");
            if rhai.is_none() {
                let mut sc = Scope::new();
                sc.push("a", 0.0f64);
                sc.push("b", 0.0f64);
                *rhai = Some((Rhai::new(), sc));
            }
            let ast = rhai
                .as_ref()
                .unwrap()
                .0
                .compile_expression(&src)
                .map_err(|e| format!("math.expr {src:?}: {e}"))?;
            Kind::Expr { ast, avisou: false }
        }
        "time.delay" => Kind::Delay {
            d: f(n, "ms", 0.0) / 1000.0,
            fila: [(0.0, 0.0); DELAY_N],
            n: 0,
            ult: 0.0,
            p: 0.0,
        },
        "time.hold" => Kind::Hold {
            d: f(n, "ms", 300.0) / 1000.0,
            ate: f64::NEG_INFINITY,
            p: 0.0,
        },
        "cmd" => Kind::Cmd {
            nome: txt(n, "cmd", ""),
            args: n
                .get("args")
                .cloned()
                .unwrap_or(Value::Object(Default::default())),
            p: 0.0,
        },
        "out.widget" => Kind::SaiWidget {
            id: txt(n, "widget", ""),
            prop: txt(n, "prop", "value"),
            hold: f(n, "hold_ms", 0.0) / 1000.0,
            ate: 0.0,
            aceso: false,
            p: 0.0,
            ult: f64::NAN,
        },
        "out.osc" => Kind::SaiOsc {
            addr: txt(n, "address", ""),
            ult: f64::NAN,
        },
        "out.param" => Kind::SaiParam {
            alvo: txt(n, "target", ""),
            ult: f64::NAN,
        },
        "out.notify" => Kind::SaiNotify {
            txt: txt(n, "text", ""),
            p: 0.0,
        },
        "state" => Kind::Estado {
            i: meu.unwrap(),
            pe: 0.0,
            px: 0.0,
        },
        "module" => {
            let m = m.unwrap();
            let nome = txt(n, "module", "");
            let params = params_num(m)
                .map(|(k, p)| Par {
                    alvo: format!("{nome}/{k}"),
                    norm: p.norm.map(|a| (a[0], a[1])),
                    // ponytail: with no min/max declared the clamp is the identity ; a
                    // parameter without a range is the app that has not measured its own
                    // limit yet.
                    min: p.min.unwrap_or(f64::NEG_INFINITY),
                    max: p.max.unwrap_or(f64::INFINITY),
                    ult: f64::NAN,
                })
                .collect();
            let cmds = m
                .commands
                .keys()
                .map(|k| (format!("{nome}/{k}"), 0.0))
                .collect();
            Kind::Modulo { params, cmds }
        }
        _ => return Err(format!("type outside the catalog: {tipo}")),
    })
}

pub struct Graph {
    nos: Vec<No>,
    vals: Vec<f64>,
    /// "widget:go" / "module:laser/geo/scale" -> slots this key feeds
    por_chave: HashMap<String, Vec<usize>>,
    /// output slots of the event nodes: zeroed at the start of each frame (1-frame pulse)
    ev_slots: Vec<usize>,
    inq: Vec<(usize, f64)>,
    outq: Vec<Ev>,
    sink: Box<dyn EventSink>,
    perdidos: u64,
    rhai: Option<(Rhai, Scope<'static>)>,
    /// one per `state` node: active now
    estados: Vec<bool>,
    /// (interned) group of each state, and the `initial` that `reset()` goes back to
    grupos: Vec<usize>,
    iniciais: Vec<bool>,
    /// per state: slot of the `active` pin and index of the `state` node in `nos`
    est_slots: Vec<(usize, usize)>,
}

impl Graph {
    /// With no show directory: no `module` node can be resolved.
    pub fn new(spec: &Value, sink: Box<dyn EventSink>) -> Result<Graph, String> {
        Graph::new_in(spec, sink, Path::new("."))
    }

    /// `base` is the show directory: `module` reads `base/modules/<name>.json`.
    pub fn new_in(
        spec: &Value,
        sink: Box<dyn EventSink>,
        base_dir: &Path,
    ) -> Result<Graph, String> {
        let brutos = spec
            .get("nodes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "graph without \"nodes\"".to_string())?;
        let n = brutos.len();
        let mut idx: HashMap<&str, usize> = HashMap::with_capacity(n);
        let mut tipos: Vec<&str> = Vec::with_capacity(n);
        // one per `state` node, in file order
        let mut est_de_id: HashMap<&str, usize> = HashMap::new();
        let mut grupos: Vec<usize> = Vec::new();
        let mut declarados: Vec<bool> = Vec::new();
        let mut nomes_grupo: HashMap<String, usize> = HashMap::new();
        // module.json of each `module` node, read once
        let mut mods: HashMap<usize, Module> = HashMap::new();
        for (i, no) in brutos.iter().enumerate() {
            let id = no
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("node {i} without \"id\""))?;
            let tipo = no
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("node {id:?} without \"type\""))?;
            if pinos(tipo).is_none() {
                return Err(format!("node {id:?}: type outside the catalog: {tipo}"));
            }
            if idx.insert(id, i).is_some() {
                return Err(format!("duplicate id in the graph: {id:?}"));
            }
            if tipo == "state" {
                let g = txt(no, "group", "main");
                let ng = nomes_grupo.len();
                let g = *nomes_grupo.entry(g).or_insert(ng);
                est_de_id.insert(id, grupos.len());
                grupos.push(g);
                declarados.push(b(no, "initial", false));
            }
            if tipo == "module" {
                let nome = txt(no, "module", "");
                let m = carrega_modulo(base_dir, &nome).map_err(|e| format!("no {id:?}: {e}"))?;
                mods.insert(i, m);
            }
            tipos.push(tipo);
        }
        let pinos_de: Vec<(Vec<String>, Vec<String>)> =
            (0..n).map(|i| pinos_no(tipos[i], mods.get(&i))).collect();
        // one active per group at load time too: the last `initial` of the group wins
        let mut venc: Vec<Option<usize>> = vec![None; nomes_grupo.len()];
        for (i, &d) in declarados.iter().enumerate() {
            if d {
                venc[grupos[i]] = Some(i);
            }
        }
        let mut estados = vec![false; grupos.len()];
        venc.iter().flatten().for_each(|&i| estados[i] = true);
        // already with the exclusion applied: it is where `reset()` goes back to
        let iniciais = estados.clone();
        // `"state": "<id>"` of each node, resolved to the state index
        let mut gated: Vec<Option<usize>> = Vec::with_capacity(n);
        for no in brutos.iter() {
            gated.push(match no.get("state").and_then(|v| v.as_str()) {
                None => None,
                Some(s) => Some(*est_de_id.get(s).ok_or_else(|| {
                    format!(
                        "node {:?}: \"state\": {s:?} is not a node of type state",
                        no["id"].as_str().unwrap_or("?")
                    )
                })?),
            });
        }

        // edges: "node.pin" -> "node.pin", resolved to integer (node, pin)
        let vazio = Vec::new();
        let brutas = match spec.get("edges") {
            None => &vazio,
            Some(v) => v
                .as_array()
                .ok_or_else(|| "\"edges\" is not a list".to_string())?,
        };
        let mut arestas: Vec<(usize, usize, usize, usize)> = Vec::with_capacity(brutas.len());
        for a in brutas {
            let par = a
                .as_array()
                .filter(|p| p.len() == 2)
                .ok_or_else(|| format!("edge {a} is not [\"node.pin\", \"node.pin\"]"))?;
            let lado = |s: &Value, saida: bool| -> Result<(usize, usize), String> {
                let s = s
                    .as_str()
                    .ok_or_else(|| format!("edge {a}: pin is not text"))?;
                let (no, pino) = s
                    .rsplit_once('.')
                    .ok_or_else(|| format!("pin {s:?} without a dot (use \"node.pin\")"))?;
                let i = *idx
                    .get(no)
                    .ok_or_else(|| format!("edge {a}: node {no:?} does not exist"))?;
                let (ins, outs) = &pinos_de[i];
                let lista = if saida { outs } else { ins };
                let p = lista.iter().position(|x| *x == pino).ok_or_else(|| {
                    format!(
                        "node {no:?} ({}) has no {} pin {pino:?}; it has {lista:?}",
                        tipos[i],
                        if saida { "output" } else { "input" }
                    )
                })?;
                Ok((i, p))
            };
            let (si, sp) = lado(&par[0], true)?;
            let (di, dp) = lado(&par[1], false)?;
            arestas.push((si, sp, di, dp));
        }

        // topological order (Kahn, queue in file order: deterministic output)
        let mut grau = vec![0usize; n];
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        for &(s, _, d, _) in &arestas {
            adj[s].push(d);
            grau[d] += 1;
        }
        let mut fila: VecDeque<usize> = (0..n).filter(|i| grau[*i] == 0).collect();
        let mut ordem: Vec<usize> = Vec::with_capacity(n);
        while let Some(i) = fila.pop_front() {
            ordem.push(i);
            for &j in &adj[i] {
                grau[j] -= 1;
                if grau[j] == 0 {
                    fila.push_back(j);
                }
            }
        }
        if ordem.len() < n {
            let presos: Vec<&str> = (0..n)
                .filter(|i| grau[*i] > 0)
                .map(|i| brutos[i]["id"].as_str().unwrap_or("?"))
                .collect();
            return Err(format!(
                "cycle in the graph, among the nodes: {}",
                presos.join(", ")
            ));
        }
        let mut topo = vec![0usize; n]; // file index -> topological index
        for (k, &i) in ordem.iter().enumerate() {
            topo[i] = k;
        }

        // slots: each node takes nin + nout consecutive positions in `vals`
        let mut rhai = None;
        let mut nos: Vec<No> = Vec::with_capacity(n);
        let mut base = 0usize;
        let mut bases = vec![0usize; n];
        let mut ev_slots = Vec::new();
        let mut est_slots = vec![(0usize, 0usize); grupos.len()];
        let mut por_chave: HashMap<String, Vec<usize>> = HashMap::new();
        for &i in &ordem {
            let (ins, outs) = &pinos_de[i];
            bases[i] = base;
            let id = brutos[i]["id"].as_str().unwrap_or("?");
            let kind = monta(
                tipos[i],
                &brutos[i],
                &mut rhai,
                mods.get(&i),
                est_de_id.get(id).copied(),
            )
            .map_err(|e| format!("no {id:?}: {e}"))?;
            if let Some(k) = chave(tipos[i], &brutos[i]) {
                por_chave.entry(k).or_default().push(base + ins.len());
                ev_slots.push(base + ins.len());
            }
            if let Some(&e) = est_de_id.get(id) {
                est_slots[e] = (base + ins.len(), nos.len());
            }
            if tipos[i] == "module" {
                // one level OUTPUT per value, fed by `input("module:<mod>/<path>")`
                let nome = txt(&brutos[i], "module", "");
                for (j, val) in outs.iter().enumerate() {
                    let k = format!("module:{nome}/{val}");
                    por_chave.entry(k).or_default().push(base + ins.len() + j);
                }
            }
            nos.push(No {
                id: id.to_string(),
                kind,
                base,
                nin: ins.len(),
                nout: outs.len(),
                mute: b(&brutos[i], "mute", false),
                estado: gated[i],
                ins: Vec::new(),
            });
            base += ins.len() + outs.len();
        }
        for (si, sp, di, dp) in arestas {
            let src = bases[si] + pinos_de[si].0.len() + sp;
            let dst = bases[di] + dp;
            nos[topo[di]].ins.push((src, dst));
        }

        Ok(Graph {
            nos,
            vals: vec![0.0; base],
            por_chave,
            ev_slots,
            inq: Vec::with_capacity(FILA),
            outq: Vec::with_capacity(FILA),
            sink,
            perdidos: 0,
            rhai,
            estados,
            grupos,
            iniciais,
            est_slots,
        })
    }

    pub fn nodes(&self) -> usize {
        self.nos.len()
    }

    /// Rewrites the `active` pin of each state: the group exclusion can happen after the `state`
    /// node has been evaluated. A silenced `state` node (`mute`, or itself inside an inactive
    /// state) does not emit: its `active` stays at the 0 the gate wrote, even though the machine
    /// keeps holding the state internally.
    fn ativos(&mut self) {
        for i in 0..self.est_slots.len() {
            let (s, k) = self.est_slots[i];
            let no = &self.nos[k];
            if no.mute || no.estado.is_some_and(|j| !self.estados[j]) {
                continue;
            }
            self.vals[s] = b2f(self.estados[i]);
        }
    }
}

fn lig(v: f64) -> bool {
    v >= 0.5
}

fn subiu(v: f64, p: &mut f64) -> bool {
    let s = lig(v) && !lig(*p);
    *p = v;
    s
}

impl FrameHook for Graph {
    fn frame(&mut self, t: f64, uni: &mut Universes) {
        // 1. one-frame pulse: zeroes the outputs of the event nodes and applies the queue
        for &s in &self.ev_slots {
            self.vals[s] = 0.0;
        }
        for &(s, v) in &self.inq {
            self.vals[s] = v;
        }
        self.inq.clear();

        // 2. nodes in topological order; no allocation here (only Ev, and an event is rare)
        for k in 0..self.nos.len() {
            let No {
                id,
                kind,
                base,
                nin,
                nout,
                mute,
                estado,
                ins,
            } = &mut self.nos[k];
            for &(s, d) in ins.iter() {
                self.vals[d] = self.vals[s];
            }
            let b = *base;
            let o = b + *nin;
            // mute or inactive state: outputs at 0, no Ev, pending delay cancelled
            if *mute || estado.is_some_and(|i| !self.estados[i]) {
                for s in o..o + *nout {
                    self.vals[s] = 0.0;
                }
                silencia(kind);
                continue;
            }
            match kind {
                Kind::Evento => {}
                Kind::Timer { every, prox } => {
                    self.vals[o] = if t >= *prox {
                        *prox = t + *every;
                        1.0
                    } else {
                        0.0
                    };
                }
                Kind::EstadoT => self.vals[o] = t,
                Kind::EstadoDmx { uni: u, addr } => {
                    self.vals[o] = uni
                        .get(*u)
                        .map(|x| x.data[(*addr - 1) as usize] as f64)
                        .unwrap_or(0.0);
                }
                Kind::And => self.vals[o] = b2f(lig(self.vals[b]) && lig(self.vals[b + 1])),
                Kind::Or => self.vals[o] = b2f(lig(self.vals[b]) || lig(self.vals[b + 1])),
                Kind::Not => self.vals[o] = b2f(!lig(self.vals[b])),
                Kind::Latch { q } => {
                    if lig(self.vals[b]) {
                        *q = 1.0;
                    }
                    if lig(self.vals[b + 1]) {
                        *q = 0.0; // reset wins over set
                    }
                    self.vals[o] = *q;
                }
                Kind::Toggle { q, p } => {
                    if subiu(self.vals[b], p) {
                        *q = 1.0 - *q;
                    }
                    self.vals[o] = *q;
                }
                Kind::Debounce { win, ult, p } => {
                    let s = subiu(self.vals[b], p) && t - *ult >= *win;
                    if s {
                        *ult = t;
                    }
                    self.vals[o] = b2f(s);
                }
                Kind::Counter { passo, n, p, pr } => {
                    if subiu(self.vals[b], p) {
                        *n += *passo;
                    }
                    if subiu(self.vals[b + 1], pr) {
                        *n = 0.0;
                    }
                    self.vals[o] = *n;
                }
                Kind::Select => {
                    self.vals[o] = if lig(self.vals[b + 2]) {
                        self.vals[b + 1]
                    } else {
                        self.vals[b]
                    };
                }
                Kind::Map {
                    i0,
                    i1,
                    o0,
                    o1,
                    clamp,
                } => {
                    let d = *i1 - *i0;
                    let mut u = if d == 0.0 {
                        0.0
                    } else {
                        (self.vals[b] - *i0) / d
                    };
                    if *clamp {
                        u = u.clamp(0.0, 1.0);
                    }
                    self.vals[o] = *o0 + (*o1 - *o0) * u;
                }
                Kind::Curva { c, cc } => {
                    self.vals[o] = c.ease(self.vals[b].clamp(0.0, 1.0), *cc);
                }
                Kind::Expr { ast, avisou } => {
                    let (a, bb) = (self.vals[b], self.vals[b + 1]);
                    if let Some((eng, sc)) = self.rhai.as_mut() {
                        // scope of 2 inputs: get_mut is a name comparison, no allocation
                        if let Some(d) = sc.get_mut("a") {
                            *d = Dynamic::from(a);
                        }
                        if let Some(d) = sc.get_mut("b") {
                            *d = Dynamic::from(bb);
                        }
                        let r = eng.eval_ast_with_scope::<Dynamic>(sc, ast);
                        sc.rewind(2);
                        match r {
                            Ok(v) => self.vals[o] = crate::num(&v),
                            Err(e) => {
                                self.vals[o] = 0.0;
                                if !*avisou {
                                    *avisou = true;
                                    eprintln!("graph, no {id:?}: math.expr: {e}");
                                }
                            }
                        }
                    }
                }
                Kind::Delay { d, fila, n, ult, p } => {
                    let v = self.vals[b];
                    if v != *p {
                        *p = v;
                        if *n < DELAY_N {
                            fila[*n] = (t + *d, v);
                            *n += 1;
                        } else {
                            *ult = v; // queue full: apply now instead of losing the signal
                        }
                    }
                    let mut i = 0;
                    while i < *n {
                        if t >= fila[i].0 {
                            *ult = fila[i].1;
                            fila.copy_within(i + 1..*n, i);
                            *n -= 1;
                        } else {
                            i += 1;
                        }
                    }
                    self.vals[o] = *ult;
                }
                Kind::Hold { d, ate, p } => {
                    if subiu(self.vals[b], p) {
                        *ate = t + *d;
                    }
                    self.vals[o] = b2f(t < *ate);
                }
                Kind::Cmd { nome, args, p } => {
                    let s = subiu(self.vals[b], p);
                    self.vals[o] = b2f(s);
                    if s {
                        let e = Ev::Cmd {
                            name: nome.clone(),
                            args: args.clone(),
                        };
                        emite(&mut self.outq, &mut self.perdidos, e);
                    }
                }
                Kind::SaiWidget {
                    id,
                    prop,
                    hold,
                    ate,
                    aceso,
                    p,
                    ult,
                } => {
                    let v = self.vals[b];
                    let mut manda = None;
                    if *hold > 0.0 {
                        if subiu(v, p) {
                            *ate = t + *hold;
                            *aceso = true;
                            manda = Some(1.0);
                        } else if *aceso && t >= *ate {
                            *aceso = false;
                            manda = Some(0.0);
                        }
                    } else if v != *ult {
                        *ult = v;
                        manda = Some(v);
                    }
                    if let Some(value) = manda {
                        let e = Ev::Widget {
                            id: id.clone(),
                            prop: prop.clone(),
                            value,
                        };
                        emite(&mut self.outq, &mut self.perdidos, e);
                    }
                }
                Kind::SaiOsc { addr, ult } => {
                    let v = self.vals[b];
                    if v != *ult {
                        *ult = v;
                        let e = Ev::Osc {
                            address: addr.clone(),
                            args: vec![v],
                        };
                        emite(&mut self.outq, &mut self.perdidos, e);
                    }
                }
                Kind::SaiParam { alvo, ult } => {
                    let v = self.vals[b];
                    if v != *ult {
                        *ult = v;
                        let e = Ev::Param {
                            target: alvo.clone(),
                            value: v,
                        };
                        emite(&mut self.outq, &mut self.perdidos, e);
                    }
                }
                Kind::SaiNotify { txt, p } => {
                    if subiu(self.vals[b], p) {
                        let e = Ev::Notify { text: txt.clone() };
                        emite(&mut self.outq, &mut self.perdidos, e);
                    }
                }
                Kind::Estado { i, pe, px } => {
                    let i = *i;
                    if subiu(self.vals[b], pe) {
                        for j in 0..self.estados.len() {
                            if self.grupos[j] == self.grupos[i] {
                                self.estados[j] = false; // one active per group
                            }
                        }
                        self.estados[i] = true;
                    }
                    if subiu(self.vals[b + 1], px) {
                        self.estados[i] = false;
                    }
                    self.vals[o] = b2f(self.estados[i]);
                }
                Kind::Modulo { params, cmds } => {
                    for (j, par) in params.iter_mut().enumerate() {
                        let mut v = self.vals[b + j];
                        if let Some((n0, n1)) = par.norm {
                            v = n0 + v * (n1 - n0);
                        }
                        let v = v.clamp(par.min, par.max);
                        if v != par.ult {
                            par.ult = v;
                            let e = Ev::Param {
                                target: par.alvo.clone(),
                                value: v,
                            };
                            emite(&mut self.outq, &mut self.perdidos, e);
                        }
                    }
                    let np = params.len();
                    for (j, (nome, p)) in cmds.iter_mut().enumerate() {
                        if subiu(self.vals[b + np + j], p) {
                            let e = Ev::Cmd {
                                name: nome.clone(),
                                args: Value::Object(Default::default()),
                            };
                            emite(&mut self.outq, &mut self.perdidos, e);
                        }
                    }
                }
            }
        }

        // 3. the `active` pin holds the state at the END of the frame
        self.ativos();

        // 4. drains the outputs to the world
        for e in &self.outq {
            self.sink.emit(e);
        }
        self.outq.clear();
    }

    fn input(&mut self, key: &str, value: f64) {
        if let Some(v) = self.por_chave.get(key) {
            for &s in v {
                if self.inq.len() < FILA {
                    self.inq.push((s, value));
                } else {
                    self.perdidos += 1;
                }
            }
        }
    }

    fn reset(&mut self, t: f64) {
        self.vals.iter_mut().for_each(|v| *v = 0.0);
        self.inq.clear();
        self.outq.clear();
        self.estados.copy_from_slice(&self.iniciais);
        self.ativos();
        // reset = silence (edges, pending delay, level outputs) plus the stored VALUE, which
        // the silencing preserves on purpose
        for no in self.nos.iter_mut() {
            silencia(&mut no.kind);
            match &mut no.kind {
                Kind::Timer { prox, .. } => *prox = t,
                Kind::Latch { q } | Kind::Toggle { q, .. } => *q = 0.0,
                Kind::Debounce { ult, .. } => *ult = f64::NEG_INFINITY,
                Kind::Counter { n, .. } => *n = 0.0,
                _ => {}
            }
        }
    }
}

/// Silences a node (mute or inactive state). Cancels the pending delay, zeroes the edges and
/// makes the next LEVEL output be re-emitted on return. The stored value (`q` of the toggle and
/// of the latch, `n` of the counter) stays: the node comes back as it was, it just did not emit
/// while it was out.
fn silencia(k: &mut Kind) {
    match k {
        Kind::Toggle { p, .. } | Kind::Debounce { p, .. } | Kind::Cmd { p, .. } => *p = 0.0,
        Kind::SaiNotify { p, .. } => *p = 0.0,
        Kind::Counter { p, pr, .. } => {
            *p = 0.0;
            *pr = 0.0;
        }
        Kind::Hold { ate, p, .. } => {
            *ate = f64::NEG_INFINITY;
            *p = 0.0;
        }
        Kind::Delay { n, ult, p, .. } => {
            *n = 0;
            *ult = 0.0;
            *p = 0.0;
        }
        Kind::SaiWidget {
            ate, aceso, p, ult, ..
        } => {
            *ate = 0.0;
            *aceso = false;
            *p = 0.0;
            *ult = f64::NAN;
        }
        Kind::SaiOsc { ult, .. } | Kind::SaiParam { ult, .. } => *ult = f64::NAN,
        Kind::Estado { pe, px, .. } => {
            *pe = 0.0;
            *px = 0.0;
        }
        Kind::Modulo { params, cmds } => {
            params.iter_mut().for_each(|p| p.ult = f64::NAN);
            cmds.iter_mut().for_each(|c| c.1 = 0.0);
        }
        _ => {}
    }
}

/// Enqueues an output event. A full queue does not grow: it counts into `perdidos`.
fn emite(outq: &mut Vec<Ev>, perdidos: &mut u64, e: Ev) {
    if outq.len() < FILA {
        outq.push(e);
    } else {
        *perdidos += 1;
    }
}

fn b2f(b: bool) -> f64 {
    b as u8 as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test sink: keeps the events in a shared Vec.
    #[derive(Clone, Default)]
    struct Sink(std::sync::Arc<std::sync::Mutex<Vec<Ev>>>);

    impl EventSink for Sink {
        fn emit(&mut self, e: &Ev) {
            self.0.lock().unwrap().push(e.clone());
        }
    }

    fn monta_graph(j: &str) -> (Graph, Sink) {
        let s = Sink::default();
        let g = Graph::new(
            &serde_json::from_str::<Value>(j).unwrap(),
            Box::new(s.clone()),
        )
        .expect("compiles");
        (g, s)
    }

    #[test]
    fn logic_or_latch_toggle_and_not() {
        let (mut g, _) = monta_graph(
            r#"{"nodes":[{"id":"go","type":"in.widget","widget":"go"},
                         {"id":"k","type":"in.key","key":"Space"},
                         {"id":"any","type":"logic.or"},
                         {"id":"tg","type":"logic.toggle"},
                         {"id":"inv","type":"logic.not"},
                         {"id":"lt","type":"logic.latch"}],
                "edges":[["go.press","any.a"],["k.down","any.b"],["any.out","tg.in"],
                         ["tg.out","inv.in"],["any.out","lt.set"],["k.down","lt.reset"]]}"#,
        );
        let mut u = Universes::new();
        let saida = |g: &Graph, id: &str| -> f64 {
            let k = g.nos.iter().position(|n| n.id == id).unwrap();
            g.vals[g.nos[k].base + g.nos[k].nin]
        };
        g.frame(0.0, &mut u);
        assert_eq!(saida(&g, "any"), 0.0);
        g.input("widget:go", 1.0);
        g.frame(0.1, &mut u);
        assert_eq!(saida(&g, "any"), 1.0);
        assert_eq!(saida(&g, "tg"), 1.0);
        assert_eq!(saida(&g, "inv"), 0.0);
        assert_eq!(saida(&g, "lt"), 1.0);
        g.frame(0.2, &mut u); // the pulse lasts one frame; the toggle holds
        assert_eq!(saida(&g, "any"), 0.0);
        assert_eq!(saida(&g, "tg"), 1.0);
        g.input("widget:go", 1.0);
        g.frame(0.3, &mut u);
        assert_eq!(saida(&g, "tg"), 0.0, "the second edge turns the toggle off");
        g.input("key:Space", 1.0);
        g.frame(0.4, &mut u);
        assert_eq!(saida(&g, "lt"), 0.0, "reset wins over set");
    }

    #[test]
    fn math_map_curve_and_expr() {
        let (mut g, _) = monta_graph(
            r#"{"nodes":[{"id":"t","type":"in.state","what":"t"},
                         {"id":"m","type":"math.map","in_min":0,"in_max":10,"out_min":0,"out_max":255},
                         {"id":"c","type":"math.curve","curve":"inout"},
                         {"id":"e","type":"math.expr","expr":"a * 2.0 + 1.0"}],
                "edges":[["t.out","m.in"],["t.out","c.in"],["t.out","e.a"]]}"#,
        );
        let mut u = Universes::new();
        g.frame(0.5, &mut u);
        let v = |g: &Graph, id: &str| -> f64 {
            let k = g.nos.iter().position(|n| n.id == id).unwrap();
            g.vals[g.nos[k].base + g.nos[k].nin]
        };
        assert!((v(&g, "m") - 12.75).abs() < 1e-12);
        assert!((v(&g, "c") - 0.5).abs() < 1e-12); // inout at 0.5 = 0.5
        assert!((v(&g, "e") - 2.0).abs() < 1e-12);
        g.frame(20.0, &mut u);
        assert_eq!(v(&g, "m"), 255.0, "map with clamp");
    }

    #[test]
    fn time_delay_and_hold() {
        let (mut g, _) = monta_graph(
            r#"{"nodes":[{"id":"go","type":"in.widget","widget":"go"},
                         {"id":"d","type":"time.delay","ms":300},
                         {"id":"h","type":"time.hold","ms":300}],
                "edges":[["go.press","d.in"],["go.press","h.in"]]}"#,
        );
        let mut u = Universes::new();
        let v = |g: &Graph, id: &str| -> f64 {
            let k = g.nos.iter().position(|n| n.id == id).unwrap();
            g.vals[g.nos[k].base + g.nos[k].nin]
        };
        g.input("widget:go", 1.0);
        g.frame(0.0, &mut u);
        assert_eq!(v(&g, "d"), 0.0, "the delay has not fired yet");
        assert_eq!(v(&g, "h"), 1.0);
        g.frame(0.2, &mut u);
        assert_eq!(v(&g, "d"), 0.0);
        assert_eq!(v(&g, "h"), 1.0);
        g.frame(0.31, &mut u);
        assert_eq!(v(&g, "d"), 1.0, "300 ms later the delay releases");
        assert_eq!(v(&g, "h"), 0.0, "and the hold has already dropped");
    }

    #[test]
    fn cmd_and_outputs_emit_into_the_sink() {
        let (mut g, s) = monta_graph(
            r#"{"nodes":[{"id":"go","type":"in.widget","widget":"go"},
                         {"id":"next","type":"cmd","cmd":"cue_go","args":{"index":3}},
                         {"id":"flash","type":"out.widget","widget":"go","prop":"glow","hold_ms":300},
                         {"id":"echo","type":"out.osc","address":"/spell/go"},
                         {"id":"notice","type":"out.notify","text":"GO"},
                         {"id":"par","type":"out.param","target":"par1.dim"}],
                "edges":[["go.press","next.trigger"],["next.done","flash.in"],
                         ["next.done","echo.in"],["next.done","notice.in"],["next.done","par.in"]]}"#,
        );
        let mut u = Universes::new();
        g.frame(0.0, &mut u);
        s.0.lock().unwrap().clear();
        g.input("widget:go", 1.0);
        g.frame(0.1, &mut u);
        let evs = s.0.lock().unwrap().clone();
        assert!(evs.contains(&Ev::Cmd {
            name: "cue_go".into(),
            args: serde_json::json!({"index": 3})
        }));
        assert!(evs.contains(&Ev::Widget {
            id: "go".into(),
            prop: "glow".into(),
            value: 1.0
        }));
        assert!(evs.contains(&Ev::Osc {
            address: "/spell/go".into(),
            args: vec![1.0]
        }));
        assert!(evs.contains(&Ev::Notify { text: "GO".into() }));
        assert!(evs.contains(&Ev::Param {
            target: "par1.dim".into(),
            value: 1.0
        }));
        // the out.widget glow turns itself off after hold_ms
        s.0.lock().unwrap().clear();
        g.frame(0.5, &mut u);
        assert!(s.0.lock().unwrap().contains(&Ev::Widget {
            id: "go".into(),
            prop: "glow".into(),
            value: 0.0
        }));
    }

    #[test]
    fn topological_order_evaluates_the_chain_in_one_frame() {
        // in the file the nodes come back to front: only the topological order solves it in one
        // frame
        let (mut g, _) = monta_graph(
            r#"{"nodes":[{"id":"c","type":"math.expr","expr":"a + 1.0"},
                         {"id":"b","type":"math.expr","expr":"a + 1.0"},
                         {"id":"a","type":"in.state","what":"t"}],
                "edges":[["a.out","b.a"],["b.out","c.a"]]}"#,
        );
        assert_eq!(g.nos[0].id, "a", "the node with no input comes first");
        let mut u = Universes::new();
        g.frame(1.0, &mut u);
        let k = g.nos.iter().position(|n| n.id == "c").unwrap();
        assert_eq!(g.vals[g.nos[k].base + g.nos[k].nin], 3.0);
    }

    #[test]
    fn a_cycle_is_an_error_with_the_stuck_nodes() {
        let e = Graph::new(
            &serde_json::from_str::<Value>(
                r#"{"nodes":[{"id":"a","type":"logic.not"},{"id":"b","type":"logic.not"},
                             {"id":"loose","type":"in.key","key":"X"}],
                    "edges":[["a.out","b.in"],["b.out","a.in"]]}"#,
            )
            .unwrap(),
            Box::new(engine::NullSink),
        );
        let e = match e {
            Err(e) => e,
            Ok(_) => panic!("a cycle had to be an error"),
        };
        assert!(
            e.contains("cycle") && e.contains('a') && e.contains('b'),
            "{e}"
        );
        assert!(!e.contains("loose"), "{e}");
    }

    #[test]
    fn unknown_pin_and_type_are_a_useful_error() {
        let bad = |j: &str| -> String {
            match Graph::new(
                &serde_json::from_str::<Value>(j).unwrap(),
                Box::new(engine::NullSink),
            ) {
                Err(e) => e,
                Ok(_) => panic!("had to fail: {j}"),
            }
        };
        assert!(bad(r#"{"nodes":[{"id":"x","type":"in.video"}]}"#).contains("catalog"));
        let e = bad(
            r#"{"nodes":[{"id":"a","type":"in.key","key":"X"},{"id":"b","type":"logic.not"}],
                "edges":[["a.down","b.nope"]]}"#,
        );
        assert!(e.contains("nope") && e.contains("\"in\""), "{e}");
    }

    /// Value of the first output pin of a node, by id.
    fn saida(g: &Graph, id: &str) -> f64 {
        let k = g.nos.iter().position(|n| n.id == id).unwrap();
        g.vals[g.nos[k].base + g.nos[k].nin]
    }

    #[test]
    fn two_states_of_the_same_group_exclude_each_other() {
        // file order chosen so that the `state` nodes are evaluated BEFORE the gated node
        let (mut g, s) = monta_graph(
            r#"{"nodes":[{"id":"ea","type":"in.widget","widget":"a"},
                         {"id":"eb","type":"in.widget","widget":"b"},
                         {"id":"sa","type":"state","group":"ato","initial":true},
                         {"id":"sb","type":"state","group":"ato"},
                         {"id":"go","type":"in.widget","widget":"go"},
                         {"id":"c1","type":"cmd","cmd":"cue_go","state":"sa"}],
                "edges":[["ea.press","sa.enter"],["eb.press","sb.enter"],
                         ["go.press","c1.trigger"]]}"#,
        );
        let mut u = Universes::new();
        g.frame(0.0, &mut u);
        assert_eq!(saida(&g, "sa"), 1.0, "initial turns the state on");
        assert_eq!(saida(&g, "sb"), 0.0);
        s.0.lock().unwrap().clear();
        g.input("widget:go", 1.0);
        g.frame(0.1, &mut u);
        assert_eq!(
            s.0.lock().unwrap().len(),
            1,
            "active state: the cmd goes out"
        );

        // entering sb turns sa off in the same frame; the gated cmd no longer emits
        s.0.lock().unwrap().clear();
        g.input("widget:b", 1.0);
        g.input("widget:go", 1.0);
        g.frame(0.2, &mut u);
        assert_eq!(saida(&g, "sa"), 0.0, "one active per group");
        assert_eq!(saida(&g, "sb"), 1.0);
        assert!(
            s.0.lock().unwrap().is_empty(),
            "inactive state: nothing goes out"
        );

        // going back to sa switches the node on again
        s.0.lock().unwrap().clear();
        g.input("widget:a", 1.0);
        g.input("widget:go", 1.0);
        g.frame(0.3, &mut u);
        assert_eq!(saida(&g, "sb"), 0.0);
        assert_eq!(s.0.lock().unwrap().len(), 1, "emits again on entering");
    }

    #[test]
    fn an_inactive_state_cancels_the_pending_delay() {
        let (mut g, _) = monta_graph(
            r#"{"nodes":[{"id":"e","type":"in.widget","widget":"e"},
                         {"id":"x","type":"in.widget","widget":"x"},
                         {"id":"go","type":"in.widget","widget":"go"},
                         {"id":"s","type":"state","initial":true},
                         {"id":"d","type":"time.delay","ms":300,"state":"s"}],
                "edges":[["e.press","s.enter"],["x.press","s.exit"],["go.press","d.in"]]}"#,
        );
        let mut u = Universes::new();
        g.input("widget:go", 1.0);
        g.frame(0.0, &mut u);
        g.input("widget:x", 1.0);
        g.frame(0.05, &mut u);
        assert_eq!(saida(&g, "s"), 0.0);
        g.frame(0.4, &mut u);
        assert_eq!(saida(&g, "d"), 0.0, "the pending delay was cancelled");
        g.input("widget:e", 1.0);
        g.frame(0.5, &mut u);
        assert_eq!(saida(&g, "d"), 0.0, "and does not fire late on return");
        g.input("widget:go", 1.0);
        g.frame(0.51, &mut u);
        g.frame(0.9, &mut u);
        assert_eq!(
            saida(&g, "d"),
            1.0,
            "the delay works again in the active state"
        );
    }

    #[test]
    fn mute_silences_the_node() {
        let (mut g, s) = monta_graph(
            r#"{"nodes":[{"id":"go","type":"in.widget","widget":"go"},
                         {"id":"c","type":"cmd","cmd":"cue_go","mute":true},
                         {"id":"p","type":"out.param","target":"par1.dim","mute":true}],
                "edges":[["go.press","c.trigger"],["go.press","p.in"]]}"#,
        );
        let mut u = Universes::new();
        g.frame(0.0, &mut u);
        s.0.lock().unwrap().clear();
        g.input("widget:go", 1.0);
        g.frame(0.1, &mut u);
        assert!(s.0.lock().unwrap().is_empty(), "a muted node does not emit");
        assert_eq!(saida(&g, "c"), 0.0, "and its output stays at 0");
    }

    #[test]
    fn a_silenced_state_does_not_rewrite_the_active() {
        // "sm" has mute; "sg" is inside "s0", which never turns on. Neither of them may come
        // out 1, not even through the phase that rewrites the `active` at the end of the frame.
        let (mut g, s) = monta_graph(
            r#"{"nodes":[{"id":"e","type":"in.widget","widget":"e"},
                         {"id":"go","type":"in.widget","widget":"go"},
                         {"id":"s0","type":"state","group":"g0"},
                         {"id":"sm","type":"state","group":"gm","initial":true,"mute":true},
                         {"id":"sg","type":"state","group":"gg","initial":true,"state":"s0"},
                         {"id":"c","type":"cmd","cmd":"cue_go","state":"sm"}],
                "edges":[["e.press","sm.enter"],["go.press","c.trigger"]]}"#,
        );
        let mut u = Universes::new();
        g.frame(0.0, &mut u);
        assert_eq!(
            saida(&g, "sm"),
            0.0,
            "a muted state comes out 0 even with initial"
        );
        assert_eq!(
            saida(&g, "sg"),
            0.0,
            "a state inside an inactive state comes out 0"
        );
        g.input("widget:e", 1.0);
        g.frame(0.1, &mut u);
        assert_eq!(
            saida(&g, "sm"),
            0.0,
            "the enter does not get past the mute either"
        );
        // mute silences the pin, not the machine: "sm" stays active inside and enables "c"
        s.0.lock().unwrap().clear();
        g.input("widget:go", 1.0);
        g.frame(0.2, &mut u);
        assert_eq!(s.0.lock().unwrap().len(), 1, "the gate by sm still applies");
        g.reset(0.0);
        assert_eq!(
            saida(&g, "sm"),
            0.0,
            "and reset does not light the silenced pin"
        );
    }

    #[test]
    fn unknown_editor_keys_are_ignored() {
        let (mut g, _) = monta_graph(
            r#"{"nodes":[{"id":"go","type":"in.widget","widget":"go"},
                         {"id":"n","type":"logic.not","x":120,"y":-40,"group":"g1",
                          "label":"invert","cor":"amber"}],
                "edges":[["go.press","n.in"]]}"#,
        );
        let mut u = Universes::new();
        g.frame(0.0, &mut u);
        assert_eq!(saida(&g, "n"), 1.0);
        g.input("widget:go", 1.0);
        g.frame(0.1, &mut u);
        assert_eq!(
            saida(&g, "n"),
            0.0,
            "x/y/group/label do not change the runtime"
        );
    }

    #[test]
    fn module_emits_param_on_change_reads_value_and_fires_command() {
        let dir = std::env::temp_dir().join("spellcore_graph_modulo");
        std::fs::create_dir_all(dir.join("modules")).unwrap();
        std::fs::write(
            dir.join("modules").join("laser.json"),
            r#"{"name":"laser","type":"laser","version":"1.0.0",
                "parameters":{"geo/scale":{"type":"float","default":1,"norm":[0,2],
                                           "min":0,"max":1.5},
                              "dev/type":{"type":"enum","options":["idn","etherdream"]},
                              "dev/host":{"type":"string"}},
                "values":{"stats/pps":{"type":"float"}},
                "commands":{"blank":{"context":"action"}}}"#,
        )
        .unwrap();
        let s = Sink::default();
        let spec: Value = serde_json::from_str(
            r#"{"nodes":[{"id":"k","type":"in.widget","widget":"k"},
                         {"id":"t","type":"in.widget","widget":"t"},
                         {"id":"tg","type":"logic.toggle"},
                         {"id":"m","type":"module","module":"laser"}],
                "edges":[["k.press","tg.in"],["tg.out","m.geo/scale"],["t.press","m.blank"]]}"#,
        )
        .unwrap();
        let mut g = Graph::new_in(&spec, Box::new(s.clone()), &dir).expect("compiles");
        let mut u = Universes::new();

        // an enum/string parameter does not become a pin: an edge to it does not compile
        let ruim: Value = serde_json::from_str(
            r#"{"nodes":[{"id":"k","type":"in.widget","widget":"k"},
                         {"id":"m","type":"module","module":"laser"}],
                "edges":[["k.press","m.dev/host"]]}"#,
        )
        .unwrap();
        let Err(e) = Graph::new_in(&ruim, Box::new(Sink::default()), &dir) else {
            panic!("enum/string has no pin: the edge had to fail");
        };
        assert!(e.contains("dev/host"), "{e}");

        g.frame(0.0, &mut u);
        assert_eq!(
            *s.0.lock().unwrap(),
            vec![Ev::Param {
                target: "laser/geo/scale".into(),
                value: 0.0
            }],
            "the first frame syncs the parameter"
        );
        s.0.lock().unwrap().clear();
        g.frame(0.1, &mut u);
        assert!(s.0.lock().unwrap().is_empty(), "no change, no Param");

        // toggle goes up: 1.0 mapped by norm [0,2] = 2.0, clamped at max 1.5
        g.input("widget:k", 1.0);
        g.frame(0.2, &mut u);
        assert_eq!(
            *s.0.lock().unwrap(),
            vec![Ev::Param {
                target: "laser/geo/scale".into(),
                value: 1.5
            }]
        );

        // a value arrives through input and becomes the node output
        s.0.lock().unwrap().clear();
        g.input("module:laser/stats/pps", 31000.0);
        g.frame(0.3, &mut u);
        assert_eq!(saida(&g, "m"), 31000.0);
        g.frame(0.4, &mut u);
        assert_eq!(saida(&g, "m"), 31000.0, "a value is a level, not a pulse");

        // a command becomes Ev::Cmd on the rising edge
        s.0.lock().unwrap().clear();
        g.input("widget:t", 1.0);
        g.frame(0.5, &mut u);
        assert_eq!(
            *s.0.lock().unwrap(),
            vec![Ev::Cmd {
                name: "laser/blank".into(),
                args: serde_json::json!({})
            }]
        );
    }

    #[test]
    fn in_state_dmx_reads_the_frame_universe() {
        let (mut g, _) = monta_graph(
            r#"{"nodes":[{"id":"d","type":"in.state","what":"dmx","universe":1,"address":5}]}"#,
        );
        let mut u = Universes::new();
        u.get_or_create(1).set(5, &[200.0]);
        g.frame(0.0, &mut u);
        assert_eq!(g.vals[g.nos[0].base], 200.0);
    }
}
