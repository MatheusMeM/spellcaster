//! Graph runtime da secao 10 do PRD: comportamento do show (o que cada botao, tecla, OSC,
//! MIDI, timer ou marker faz), vivendo no `.spell` e rodando no engine com ou sem GUI.
//!
//! Catalogo FECHADO. Pedido novo vira track, nao vira no.
//!
//! | tipo | config | pinos de entrada | pinos de saida |
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
//! | `math.expr`      | `expr` (uma linha de Rhai, `a` e `b` no escopo) | `a`,`b` | `out` |
//! | `time.delay`     | `ms`                            | `in`         | `out`   |
//! | `time.hold`      | `ms`                            | `in`         | `out`   |
//! | `cmd`            | `cmd`, `args`                   | `trigger`    | `done`  |
//! | `out.widget`     | `widget`,`prop`,`hold_ms`       | `in`         | -       |
//! | `out.osc`        | `address`                       | `in`         | -       |
//! | `out.param`      | `target`                        | `in`         | -       |
//! | `out.notify`     | `text`                          | `in`         | -       |
//! | `state`          | `group` ("main"), `initial`     | `enter`,`exit` | `active` |
//! | `module`         | `module` (nome do `modules/<nome>.json`) | um por `parameter`, depois um por `command` | um por `value` |
//!
//! Sinal e' `f64`; "ligado" e' `>= 0.5`; "borda de subida" e' passar de `< 0.5` para `>= 0.5`.
//! Um no de entrada de EVENTO da' um pulso de UM frame com o valor recebido em
//! `FrameHook::input`; `in.timer` e `in.state` valem por nivel.
//!
//! # Estado e mute (proposta desta rodada; ver `design/DECISOES.md`)
//!
//! Qualquer no aceita duas chaves a mais: `"mute": true` (o no nao emite) e
//! `"state": "<id de um no state>"` (o no so' emite enquanto aquele estado esta' ativo).
//! "Nao emite" e' a mesma coisa nos dois casos: as saidas do no ficam em 0, nenhum `Ev` sai e o
//! `time.delay` pendente e' cancelado. Ao voltar, o no volta como estava (o `q` do toggle e do
//! latch, o `n` do counter), com as bordas zeradas e o proximo valor de uma saida de NIVEL
//! (`out.osc`, `out.param`, `out.widget`, `module`) reemitido: e' o "reemitir ao ativar" do
//! Chataigne. Trigger que ja' esteja alto no frame em que o estado abre dispara uma vez.
//!
//! `state` e' UM ativo por `group`: pulso em `enter` liga este e desliga os outros do mesmo
//! grupo; pulso em `exit` desliga; `reset()` volta ao `initial`. No `state` calado tambem nao
//! emite: o `active` dele sai 0, ainda que a maquina guarde o estado por dentro.
// ponytail: o estado vale a partir do ponto em que o no `state` e' avaliado, entao um no gated
// que venha ANTES dele na ordem topologica ve o valor do frame anterior ; virar pre-passe so' dos
// nos `state` se algum show real depender do frame exato.
//!
//! `module` e' o app declarado em `modules/<nome>.json` (formato do `module.json` do Chataigne):
//! `parameters{path:{type,default,norm,min,max}}`, `values{path:{type}}`, `commands{nome:{context}}`.
//! Cada `parameter` e' uma ENTRADA: valor mudou -> `Ev::Param{target:"<modulo>/<path>"}`, com
//! `norm` mapeando o sinal 0..1 para `[n0,n1]` antes do clamp em `min`..`max`. Cada `value` e' uma
//! SAIDA de nivel alimentada por `FrameHook::input("module:<modulo>/<path>", v)`. Cada `command`
//! e' uma entrada de trigger -> `Ev::Cmd{name:"<modulo>/<cmd>", args:{}}`.
//!
//! JSON: `{"nodes":[{"id","type",...}], "edges":[["no.pino","no.pino"], ...]}`.
//! Compila para uma lista de nos em ordem topologica com os pinos indexados por INTEIRO
//! (nenhum lookup por texto no caminho quente). Ciclo e' erro na compilacao.

use engine::hook::{Ev, EventSink, FrameHook};
use engine::{Curve, Universes};
use rhai::{Dynamic, Engine as Rhai, Scope, AST};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::path::Path;

/// Fila de eventos de entrada e de saida, pre-alocadas.
// ponytail: 64 eventos por frame em cada sentido ; um GO e um OSC por frame estao a ordens de
// grandeza disso. Sobe quando algum graph real encostar no teto (o contador `perdidos` avisa).
const FILA: usize = 64;

/// Pendencias de `time.delay`.
// ponytail: 8 mudancas em voo por no de delay ; virar fila dinamica se um graph real estourar.
const DELAY_N: usize = 8;

enum Kind {
    /// in.widget / in.key / in.osc / in.midi / in.marker: o valor chega pela fila.
    Evento,
    Timer { every: f64, prox: f64 },
    EstadoT,
    EstadoDmx { uni: u16, addr: u16 },
    And,
    Or,
    Not,
    Latch { q: f64 },
    Toggle { q: f64, p: f64 },
    Debounce { win: f64, ult: f64, p: f64 },
    Counter { passo: f64, n: f64, p: f64, pr: f64 },
    Select,
    Map { i0: f64, i1: f64, o0: f64, o1: f64, clamp: bool },
    Curva { c: Curve, cc: (f64, f64) },
    Expr { ast: AST, avisou: bool },
    Delay { d: f64, fila: [(f64, f64); DELAY_N], n: usize, ult: f64, p: f64 },
    Hold { d: f64, ate: f64, p: f64 },
    Cmd { nome: String, args: Value, p: f64 },
    SaiWidget { id: String, prop: String, hold: f64, ate: f64, aceso: bool, p: f64, ult: f64 },
    SaiOsc { addr: String, ult: f64 },
    SaiParam { alvo: String, ult: f64 },
    SaiNotify { txt: String, p: f64 },
    /// Maquina de estados: `i` e' o indice deste estado em `Graph::estados`.
    Estado { i: usize, pe: f64, px: f64 },
    /// App declarado: uma entrada por parameter, depois uma por command; uma saida por value.
    Modulo { params: Vec<Par>, cmds: Vec<(String, f64)> },
}

/// Um `parameter` de um no `module`, ja' resolvido para o caminho quente.
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
    /// `"mute": true` no JSON: o no nao emite.
    mute: bool,
    /// `"state": "<id>"` no JSON: indice em `Graph::estados` do estado que libera este no.
    estado: Option<usize>,
    /// arestas que CHEGAM neste no: (slot de origem, slot de destino)
    ins: Vec<(usize, usize)>,
}

/// Pinos de entrada e de saida de cada tipo do catalogo.
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
        // "module": pinos vem do modules/<nome>.json, resolvidos em `pinos_no`
        "module" => (&[], &[]),
        _ => return None,
    })
}

/// `modules/<nome>.json` relativo ao diretorio do show.
// ponytail: le e valida o minimo do module.json aqui ; trocar por `engine::module::load` quando
// a frente F4 entrar em main (o formato e' o mesmo).
fn carrega_modulo(base: &Path, nome: &str) -> Result<Value, String> {
    let p = base.join("modules").join(format!("{nome}.json"));
    let s = std::fs::read_to_string(&p)
        .map_err(|e| format!("module {nome:?}: {}: {e}", p.display()))?;
    serde_json::from_str(&s).map_err(|e| format!("module {nome:?}: {e}"))
}

/// Chaves de um objeto do module.json, em ordem estavel (o layout dos pinos depende dela).
/// `serde_json::Map` sem `preserve_order` e' um `BTreeMap`: `keys()` ja' sai ordenado.
fn chaves(m: &Value, k: &str) -> Vec<String> {
    match m.get(k).and_then(|x| x.as_object()) {
        Some(o) => o.keys().cloned().collect(),
        None => Vec::new(),
    }
}

/// Pinos deste no. Iguais aos do tipo, menos `module`, que os tira do arquivo.
fn pinos_no(tipo: &str, m: Option<&Value>) -> (Vec<String>, Vec<String>) {
    match m {
        Some(m) => {
            let mut ins = chaves(m, "parameters");
            ins.extend(chaves(m, "commands"));
            (ins, chaves(m, "values"))
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

/// Chave de evento que este no escuta ("widget:go", "key:Space", ...), se for no de evento.
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
    m: Option<&Value>,
    // se este no E' um `state`, o indice dele em `Graph::estados`
    meu: Option<usize>,
) -> Result<Kind, String> {
    Ok(match tipo {
        "in.widget" | "in.key" | "in.osc" | "in.midi" | "in.marker" => Kind::Evento,
        "in.timer" => Kind::Timer { every: f(n, "every", 1.0).max(1e-6), prox: 0.0 },
        "in.state" => match txt(n, "what", "t").as_str() {
            "t" => Kind::EstadoT,
            "dmx" => Kind::EstadoDmx {
                uni: f(n, "universe", 1.0) as u16,
                addr: (f(n, "address", 1.0) as u16).clamp(1, 512),
            },
            // ponytail: in.state so' le "t" e um canal DMX dos Universes do frame ; cue,
            // fixture e o resto entram quando o FrameHook receber o Handle do player.
            o => return Err(format!("in.state: \"what\" desconhecido: {o} (use \"t\" ou \"dmx\")")),
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
        "logic.counter" => Kind::Counter { passo: f(n, "step", 1.0), n: 0.0, p: 0.0, pr: 0.0 },
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
            Kind::Curva { c: Curve::from_str(&txt(n, "curve", "linear")), cc: c }
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
        "time.hold" => Kind::Hold { d: f(n, "ms", 300.0) / 1000.0, ate: f64::NEG_INFINITY, p: 0.0 },
        "cmd" => Kind::Cmd {
            nome: txt(n, "cmd", ""),
            args: n.get("args").cloned().unwrap_or(Value::Object(Default::default())),
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
        "out.osc" => Kind::SaiOsc { addr: txt(n, "address", ""), ult: f64::NAN },
        "out.param" => Kind::SaiParam { alvo: txt(n, "target", ""), ult: f64::NAN },
        "out.notify" => Kind::SaiNotify { txt: txt(n, "text", ""), p: 0.0 },
        "state" => Kind::Estado { i: meu.unwrap(), pe: 0.0, px: 0.0 },
        "module" => {
            let m = m.unwrap();
            let nome = txt(n, "module", "");
            let params = chaves(m, "parameters")
                .iter()
                .map(|k| {
                    let p = &m["parameters"][k];
                    Par {
                        alvo: format!("{nome}/{k}"),
                        norm: p
                            .get("norm")
                            .and_then(|v| v.as_array())
                            .filter(|a| a.len() == 2)
                            .map(|a| (a[0].as_f64().unwrap_or(0.0), a[1].as_f64().unwrap_or(1.0))),
                        // ponytail: sem min/max declarados o clamp e' identidade ; parametro sem
                        // faixa e' o caso do app que ainda nao mediu o proprio limite.
                        min: f(p, "min", f64::NEG_INFINITY),
                        max: f(p, "max", f64::INFINITY),
                        ult: f64::NAN,
                    }
                })
                .collect();
            let cmds = chaves(m, "commands").iter().map(|k| (format!("{nome}/{k}"), 0.0)).collect();
            Kind::Modulo { params, cmds }
        }
        _ => return Err(format!("tipo fora do catalogo: {tipo}")),
    })
}

pub struct Graph {
    nos: Vec<No>,
    vals: Vec<f64>,
    /// "widget:go" / "module:laser/geo/scale" -> slots que essa chave alimenta
    por_chave: HashMap<String, Vec<usize>>,
    /// slots de saida dos nos de evento: zerados no inicio de cada frame (pulso de 1 frame)
    ev_slots: Vec<usize>,
    inq: Vec<(usize, f64)>,
    outq: Vec<Ev>,
    sink: Box<dyn EventSink>,
    perdidos: u64,
    rhai: Option<(Rhai, Scope<'static>)>,
    /// um por no `state`: ativo agora
    estados: Vec<bool>,
    /// grupo (internado) de cada estado, e o `initial` a que `reset()` volta
    grupos: Vec<usize>,
    iniciais: Vec<bool>,
    /// por estado: slot do pino `active` e indice do no `state` em `nos`
    est_slots: Vec<(usize, usize)>,
}

impl Graph {
    /// Sem diretorio de show: nenhum no `module` pode ser resolvido.
    pub fn new(spec: &Value, sink: Box<dyn EventSink>) -> Result<Graph, String> {
        Graph::new_in(spec, sink, Path::new("."))
    }

    /// `base` e' o diretorio do show: `module` le `base/modules/<nome>.json`.
    pub fn new_in(
        spec: &Value,
        sink: Box<dyn EventSink>,
        base_dir: &Path,
    ) -> Result<Graph, String> {
        let brutos = spec
            .get("nodes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "graph sem \"nodes\"".to_string())?;
        let n = brutos.len();
        let mut idx: HashMap<&str, usize> = HashMap::with_capacity(n);
        let mut tipos: Vec<&str> = Vec::with_capacity(n);
        // um por no `state`, em ordem de arquivo
        let mut est_de_id: HashMap<&str, usize> = HashMap::new();
        let mut grupos: Vec<usize> = Vec::new();
        let mut declarados: Vec<bool> = Vec::new();
        let mut nomes_grupo: HashMap<String, usize> = HashMap::new();
        // module.json de cada no `module`, lido uma vez
        let mut mods: HashMap<usize, Value> = HashMap::new();
        for (i, no) in brutos.iter().enumerate() {
            let id = no
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("no {i} sem \"id\""))?;
            let tipo = no
                .get("type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("no {id:?} sem \"type\""))?;
            if pinos(tipo).is_none() {
                return Err(format!("no {id:?}: tipo fora do catalogo: {tipo}"));
            }
            if idx.insert(id, i).is_some() {
                return Err(format!("id repetido no graph: {id:?}"));
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
        // um ativo por grupo tambem na carga: o ultimo `initial` do grupo ganha
        let mut venc: Vec<Option<usize>> = vec![None; nomes_grupo.len()];
        for (i, &d) in declarados.iter().enumerate() {
            if d {
                venc[grupos[i]] = Some(i);
            }
        }
        let mut estados = vec![false; grupos.len()];
        venc.iter().flatten().for_each(|&i| estados[i] = true);
        let iniciais = estados.clone(); // ja' com a exclusao aplicada: e' onde `reset()` volta
                                        // `"state": "<id>"` de cada no, resolvido para o indice do estado
        let mut gated: Vec<Option<usize>> = Vec::with_capacity(n);
        for no in brutos.iter() {
            gated.push(match no.get("state").and_then(|v| v.as_str()) {
                None => None,
                Some(s) => Some(*est_de_id.get(s).ok_or_else(|| {
                    format!(
                        "no {:?}: \"state\": {s:?} nao e' um no do tipo state",
                        no["id"].as_str().unwrap_or("?")
                    )
                })?),
            });
        }

        // arestas: "no.pino" -> "no.pino", resolvidas para (no, pino) inteiros
        let vazio = Vec::new();
        let brutas = match spec.get("edges") {
            None => &vazio,
            Some(v) => v.as_array().ok_or_else(|| "\"edges\" nao e' lista".to_string())?,
        };
        let mut arestas: Vec<(usize, usize, usize, usize)> = Vec::with_capacity(brutas.len());
        for a in brutas {
            let par = a
                .as_array()
                .filter(|p| p.len() == 2)
                .ok_or_else(|| format!("aresta {a} nao e' [\"no.pino\", \"no.pino\"]"))?;
            let lado = |s: &Value, saida: bool| -> Result<(usize, usize), String> {
                let s = s.as_str().ok_or_else(|| format!("aresta {a}: pino nao e' texto"))?;
                let (no, pino) = s
                    .rsplit_once('.')
                    .ok_or_else(|| format!("pino {s:?} sem ponto (use \"no.pino\")"))?;
                let i = *idx.get(no).ok_or_else(|| format!("aresta {a}: no {no:?} nao existe"))?;
                let (ins, outs) = &pinos_de[i];
                let lista = if saida { outs } else { ins };
                let p = lista.iter().position(|x| *x == pino).ok_or_else(|| {
                    format!(
                        "no {no:?} ({}) nao tem pino de {} {pino:?}; tem {lista:?}",
                        tipos[i],
                        if saida { "saida" } else { "entrada" }
                    )
                })?;
                Ok((i, p))
            };
            let (si, sp) = lado(&par[0], true)?;
            let (di, dp) = lado(&par[1], false)?;
            arestas.push((si, sp, di, dp));
        }

        // ordem topologica (Kahn, fila em ordem de arquivo: saida deterministica)
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
            return Err(format!("ciclo no graph, entre os nos: {}", presos.join(", ")));
        }
        let mut topo = vec![0usize; n]; // indice de arquivo -> indice topologico
        for (k, &i) in ordem.iter().enumerate() {
            topo[i] = k;
        }

        // slots: cada no ocupa nin + nout posicoes consecutivas em `vals`
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
                // uma SAIDA de nivel por value, alimentada por `input("module:<mod>/<path>")`
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

    /// Eventos de entrada descartados por fila cheia desde a carga.
    pub fn perdidos(&self) -> u64 {
        self.perdidos
    }

    /// Reescreve o pino `active` de cada estado: a exclusao de grupo pode acontecer depois do no
    /// `state` ter sido avaliado. No `state` calado (`mute`, ou ele proprio dentro de um estado
    /// inativo) nao emite: o `active` dele fica no 0 que o gate escreveu, ainda que a maquina
    /// continue guardando o estado por dentro.
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
        // 1. pulso de um frame: zera as saidas dos nos de evento e aplica a fila
        for &s in &self.ev_slots {
            self.vals[s] = 0.0;
        }
        for &(s, v) in &self.inq {
            self.vals[s] = v;
        }
        self.inq.clear();

        // 2. nos em ordem topologica; nenhuma alocacao aqui (so' Ev, e evento e' raro)
        for k in 0..self.nos.len() {
            let No { id, kind, base, nin, nout, mute, estado, ins } = &mut self.nos[k];
            for &(s, d) in ins.iter() {
                self.vals[d] = self.vals[s];
            }
            let b = *base;
            let o = b + *nin;
            // mute ou estado inativo: saidas em 0, nenhum Ev, delay pendente cancelado
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
                        *q = 0.0; // reset ganha do set
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
                Kind::Map { i0, i1, o0, o1, clamp } => {
                    let d = *i1 - *i0;
                    let mut u = if d == 0.0 { 0.0 } else { (self.vals[b] - *i0) / d };
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
                        // escopo de 2 entradas: get_mut e' comparacao de nome, sem alocar
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
                            *ult = v; // fila cheia: aplica na hora em vez de perder o sinal
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
                        let e = Ev::Cmd { name: nome.clone(), args: args.clone() };
                        emite(&mut self.outq, &mut self.perdidos, e);
                    }
                }
                Kind::SaiWidget { id, prop, hold, ate, aceso, p, ult } => {
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
                        let e = Ev::Osc { address: addr.clone(), args: vec![v] };
                        emite(&mut self.outq, &mut self.perdidos, e);
                    }
                }
                Kind::SaiParam { alvo, ult } => {
                    let v = self.vals[b];
                    if v != *ult {
                        *ult = v;
                        let e = Ev::Param { target: alvo.clone(), value: v };
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
                                self.estados[j] = false; // um ativo por grupo
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

        // 3. o pino `active` guarda o estado do FIM do frame
        self.ativos();

        // 4. drena as saidas para o mundo
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
        // reset = silenciar (bordas, delay pendente, saidas de nivel) mais o VALOR guardado,
        // que o silencio preserva de proposito
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

/// Silencia um no (mute ou estado inativo). Cancela o delay pendente, zera as bordas e faz a
/// proxima saida de NIVEL ser reemitida ao voltar. O valor guardado (`q` do toggle e do latch, `n`
/// do counter) fica: o no volta como estava, so' nao emitiu enquanto esteve fora.
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

/// Enfileira um evento de saida. Fila cheia nao cresce: conta em `perdidos`.
fn emite(outq: &mut Vec<Ev>, perdidos: &mut u64, e: Ev) {
    if outq.len() < FILA {
        outq.push(e);
    } else {
        *perdidos += 1;
    }
}

fn b2f(b: bool) -> f64 {
    if b {
        1.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sink de teste: guarda os eventos num Vec compartilhado.
    #[derive(Clone, Default)]
    struct Sink(std::sync::Arc<std::sync::Mutex<Vec<Ev>>>);

    impl EventSink for Sink {
        fn emit(&mut self, e: &Ev) {
            self.0.lock().unwrap().push(e.clone());
        }
    }

    fn monta_graph(j: &str) -> (Graph, Sink) {
        let s = Sink::default();
        let g = Graph::new(&serde_json::from_str::<Value>(j).unwrap(), Box::new(s.clone()))
            .expect("compila");
        (g, s)
    }

    #[test]
    fn logica_or_latch_toggle_e_not() {
        let (mut g, _) = monta_graph(
            r#"{"nodes":[{"id":"go","type":"in.widget","widget":"go"},
                         {"id":"k","type":"in.key","key":"Space"},
                         {"id":"any","type":"logic.or"},
                         {"id":"tg","type":"logic.toggle"},
                         {"id":"nao","type":"logic.not"},
                         {"id":"lt","type":"logic.latch"}],
                "edges":[["go.press","any.a"],["k.down","any.b"],["any.out","tg.in"],
                         ["tg.out","nao.in"],["any.out","lt.set"],["k.down","lt.reset"]]}"#,
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
        assert_eq!(saida(&g, "nao"), 0.0);
        assert_eq!(saida(&g, "lt"), 1.0);
        g.frame(0.2, &mut u); // o pulso dura um frame; o toggle segura
        assert_eq!(saida(&g, "any"), 0.0);
        assert_eq!(saida(&g, "tg"), 1.0);
        g.input("widget:go", 1.0);
        g.frame(0.3, &mut u);
        assert_eq!(saida(&g, "tg"), 0.0, "segunda borda desliga o toggle");
        g.input("key:Space", 1.0);
        g.frame(0.4, &mut u);
        assert_eq!(saida(&g, "lt"), 0.0, "reset ganha do set");
    }

    #[test]
    fn math_map_curve_e_expr() {
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
        assert!((v(&g, "c") - 0.5).abs() < 1e-12); // inout em 0,5 = 0,5
        assert!((v(&g, "e") - 2.0).abs() < 1e-12);
        g.frame(20.0, &mut u);
        assert_eq!(v(&g, "m"), 255.0, "map com clamp");
    }

    #[test]
    fn time_delay_e_hold() {
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
        assert_eq!(v(&g, "d"), 0.0, "o delay ainda nao venceu");
        assert_eq!(v(&g, "h"), 1.0);
        g.frame(0.2, &mut u);
        assert_eq!(v(&g, "d"), 0.0);
        assert_eq!(v(&g, "h"), 1.0);
        g.frame(0.31, &mut u);
        assert_eq!(v(&g, "d"), 1.0, "300 ms depois o delay solta");
        assert_eq!(v(&g, "h"), 0.0, "e o hold ja' caiu");
    }

    #[test]
    fn cmd_e_saidas_emitem_no_sink() {
        let (mut g, s) = monta_graph(
            r#"{"nodes":[{"id":"go","type":"in.widget","widget":"go"},
                         {"id":"next","type":"cmd","cmd":"cue_go","args":{"index":3}},
                         {"id":"flash","type":"out.widget","widget":"go","prop":"glow","hold_ms":300},
                         {"id":"eco","type":"out.osc","address":"/spell/go"},
                         {"id":"aviso","type":"out.notify","text":"GO"},
                         {"id":"par","type":"out.param","target":"par1.dim"}],
                "edges":[["go.press","next.trigger"],["next.done","flash.in"],
                         ["next.done","eco.in"],["next.done","aviso.in"],["next.done","par.in"]]}"#,
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
        assert!(evs.contains(&Ev::Widget { id: "go".into(), prop: "glow".into(), value: 1.0 }));
        assert!(evs.contains(&Ev::Osc { address: "/spell/go".into(), args: vec![1.0] }));
        assert!(evs.contains(&Ev::Notify { text: "GO".into() }));
        assert!(evs.contains(&Ev::Param { target: "par1.dim".into(), value: 1.0 }));
        // o glow do out.widget apaga sozinho depois do hold_ms
        s.0.lock().unwrap().clear();
        g.frame(0.5, &mut u);
        assert!(s
            .0
            .lock()
            .unwrap()
            .contains(&Ev::Widget { id: "go".into(), prop: "glow".into(), value: 0.0 }));
    }

    #[test]
    fn ordem_topologica_avalia_a_cadeia_em_um_frame() {
        // no arquivo os nos vem de tras para a frente: so' a ordem topologica resolve em 1 frame
        let (mut g, _) = monta_graph(
            r#"{"nodes":[{"id":"c","type":"math.expr","expr":"a + 1.0"},
                         {"id":"b","type":"math.expr","expr":"a + 1.0"},
                         {"id":"a","type":"in.state","what":"t"}],
                "edges":[["a.out","b.a"],["b.out","c.a"]]}"#,
        );
        assert_eq!(g.nos[0].id, "a", "o no sem entrada vem primeiro");
        let mut u = Universes::new();
        g.frame(1.0, &mut u);
        let k = g.nos.iter().position(|n| n.id == "c").unwrap();
        assert_eq!(g.vals[g.nos[k].base + g.nos[k].nin], 3.0);
    }

    #[test]
    fn ciclo_e_erro_com_os_nos_presos() {
        let e = Graph::new(
            &serde_json::from_str::<Value>(
                r#"{"nodes":[{"id":"a","type":"logic.not"},{"id":"b","type":"logic.not"},
                             {"id":"solto","type":"in.key","key":"X"}],
                    "edges":[["a.out","b.in"],["b.out","a.in"]]}"#,
            )
            .unwrap(),
            Box::new(engine::NullSink),
        );
        let e = match e {
            Err(e) => e,
            Ok(_) => panic!("ciclo tinha que ser erro"),
        };
        assert!(e.contains("ciclo") && e.contains('a') && e.contains('b'), "{e}");
        assert!(!e.contains("solto"), "{e}");
    }

    #[test]
    fn pino_e_tipo_desconhecidos_sao_erro_util() {
        let bad = |j: &str| -> String {
            match Graph::new(&serde_json::from_str::<Value>(j).unwrap(), Box::new(engine::NullSink))
            {
                Err(e) => e,
                Ok(_) => panic!("tinha que falhar: {j}"),
            }
        };
        assert!(bad(r#"{"nodes":[{"id":"x","type":"in.video"}]}"#).contains("catalogo"));
        let e = bad(
            r#"{"nodes":[{"id":"a","type":"in.key","key":"X"},{"id":"b","type":"logic.not"}],
                "edges":[["a.down","b.entrada"]]}"#,
        );
        assert!(e.contains("entrada") && e.contains("\"in\""), "{e}");
    }

    /// Valor do primeiro pino de saida de um no, pelo id.
    fn saida(g: &Graph, id: &str) -> f64 {
        let k = g.nos.iter().position(|n| n.id == id).unwrap();
        g.vals[g.nos[k].base + g.nos[k].nin]
    }

    #[test]
    fn dois_estados_do_mesmo_grupo_se_excluem() {
        // ordem de arquivo escolhida para que os `state` sejam avaliados ANTES do no gated
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
        assert_eq!(saida(&g, "sa"), 1.0, "initial liga o estado");
        assert_eq!(saida(&g, "sb"), 0.0);
        s.0.lock().unwrap().clear();
        g.input("widget:go", 1.0);
        g.frame(0.1, &mut u);
        assert_eq!(s.0.lock().unwrap().len(), 1, "estado ativo: o cmd sai");

        // entrar em sb desliga sa no mesmo frame; o cmd gated ja' nao emite
        s.0.lock().unwrap().clear();
        g.input("widget:b", 1.0);
        g.input("widget:go", 1.0);
        g.frame(0.2, &mut u);
        assert_eq!(saida(&g, "sa"), 0.0, "um ativo por grupo");
        assert_eq!(saida(&g, "sb"), 1.0);
        assert!(s.0.lock().unwrap().is_empty(), "estado inativo: nada sai");

        // voltar para sa religa o no
        s.0.lock().unwrap().clear();
        g.input("widget:a", 1.0);
        g.input("widget:go", 1.0);
        g.frame(0.3, &mut u);
        assert_eq!(saida(&g, "sb"), 0.0);
        assert_eq!(s.0.lock().unwrap().len(), 1, "voltou a emitir ao entrar");
    }

    #[test]
    fn estado_inativo_cancela_o_delay_pendente() {
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
        assert_eq!(saida(&g, "d"), 0.0, "delay pendente foi cancelado");
        g.input("widget:e", 1.0);
        g.frame(0.5, &mut u);
        assert_eq!(saida(&g, "d"), 0.0, "e nao solta atrasado ao voltar");
        g.input("widget:go", 1.0);
        g.frame(0.51, &mut u);
        g.frame(0.9, &mut u);
        assert_eq!(
            saida(&g, "d"),
            1.0,
            "o delay volta a funcionar no estado ativo"
        );
    }

    #[test]
    fn mute_silencia_o_no() {
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
        assert!(s.0.lock().unwrap().is_empty(), "no mutado nao emite");
        assert_eq!(saida(&g, "c"), 0.0, "e a saida dele fica em 0");
    }

    #[test]
    fn state_calado_nao_reescreve_o_active() {
        // "sm" tem mute; "sg" esta' dentro de "s0", que nunca liga. Nenhum dos dois pode sair 1,
        // nem pela fase que reescreve o `active` no fim do frame.
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
        assert_eq!(saida(&g, "sm"), 0.0, "state mutado sai 0 mesmo com initial");
        assert_eq!(saida(&g, "sg"), 0.0, "state dentro de estado inativo sai 0");
        g.input("widget:e", 1.0);
        g.frame(0.1, &mut u);
        assert_eq!(saida(&g, "sm"), 0.0, "o enter tambem nao passa pelo mute");
        // o mute cala o pino, nao a maquina: "sm" segue ativo por dentro e libera "c"
        s.0.lock().unwrap().clear();
        g.input("widget:go", 1.0);
        g.frame(0.2, &mut u);
        assert_eq!(s.0.lock().unwrap().len(), 1, "o gate por sm continua valendo");
        g.reset(0.0);
        assert_eq!(saida(&g, "sm"), 0.0, "e o reset nao acende o pino calado");
    }

    #[test]
    fn chaves_desconhecidas_do_editor_sao_ignoradas() {
        let (mut g, _) = monta_graph(
            r#"{"nodes":[{"id":"go","type":"in.widget","widget":"go"},
                         {"id":"n","type":"logic.not","x":120,"y":-40,"group":"g1",
                          "label":"inverte","cor":"ambar"}],
                "edges":[["go.press","n.in"]]}"#,
        );
        let mut u = Universes::new();
        g.frame(0.0, &mut u);
        assert_eq!(saida(&g, "n"), 1.0);
        g.input("widget:go", 1.0);
        g.frame(0.1, &mut u);
        assert_eq!(saida(&g, "n"), 0.0, "x/y/group/label nao mudam o runtime");
    }

    #[test]
    fn module_emite_param_na_mudanca_le_value_e_dispara_command() {
        let dir = std::env::temp_dir().join("spellcore_graph_modulo");
        std::fs::create_dir_all(dir.join("modules")).unwrap();
        std::fs::write(
            dir.join("modules").join("laser.json"),
            r#"{"name":"laser","type":"laser","version":"1.0.0",
                "parameters":{"geo/scale":{"type":"float","default":1,"norm":[0,2],
                                           "min":0,"max":1.5}},
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
        let mut g = Graph::new_in(&spec, Box::new(s.clone()), &dir).expect("compila");
        let mut u = Universes::new();

        g.frame(0.0, &mut u);
        assert_eq!(
            *s.0.lock().unwrap(),
            vec![Ev::Param {
                target: "laser/geo/scale".into(),
                value: 0.0
            }],
            "primeiro frame sincroniza o parametro"
        );
        s.0.lock().unwrap().clear();
        g.frame(0.1, &mut u);
        assert!(s.0.lock().unwrap().is_empty(), "sem mudanca, sem Param");

        // toggle sobe: 1.0 mapeado por norm [0,2] = 2.0, clampado em max 1.5
        g.input("widget:k", 1.0);
        g.frame(0.2, &mut u);
        assert_eq!(
            *s.0.lock().unwrap(),
            vec![Ev::Param {
                target: "laser/geo/scale".into(),
                value: 1.5
            }]
        );

        // um value chega pelo input e vira a saida do no
        s.0.lock().unwrap().clear();
        g.input("module:laser/stats/pps", 31000.0);
        g.frame(0.3, &mut u);
        assert_eq!(saida(&g, "m"), 31000.0);
        g.frame(0.4, &mut u);
        assert_eq!(saida(&g, "m"), 31000.0, "value e' nivel, nao pulso");

        // command vira Ev::Cmd na borda de subida
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
    fn in_state_dmx_le_o_universo_do_frame() {
        let (mut g, _) = monta_graph(
            r#"{"nodes":[{"id":"d","type":"in.state","what":"dmx","universe":1,"address":5}]}"#,
        );
        let mut u = Universes::new();
        u.get_or_create(1).set(5, &[200.0]);
        g.frame(0.0, &mut u);
        assert_eq!(g.vals[g.nos[0].base], 200.0);
    }
}
