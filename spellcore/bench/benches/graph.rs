//! R1 acceptance: a 500-node Graph evaluated in LESS than 0.1 ms per frame.
//!
//! The graph is built with the whole catalog of section 10 of the PRD, chained (each node
//! consumes outputs of earlier nodes), so that the topological order has real depth and is not
//! a row of independent nodes. Two measurements: with `math.expr` (5% of the nodes, each one a
//! Rhai eval) and with no `math.expr` at all, which is the typical operating graph.
//!
//! `cargo bench -p bench --bench graph`

use criterion::{criterion_group, criterion_main, Criterion};
use engine::hook::FrameHook;
use engine::{NullSink, Universes};
use script::Graph;
use serde_json::{json, Value};

/// Types used in the middle of the chain, with the name of the output pin and the input pins.
const MIOLO: &[(&str, &str, &[&str])] = &[
    ("logic.or", "out", &["a", "b"]),
    ("math.map", "out", &["in"]),
    ("logic.and", "out", &["a", "b"]),
    ("math.curve", "out", &["in"]),
    ("logic.toggle", "out", &["in"]),
    ("time.hold", "out", &["in"]),
    ("logic.select", "out", &["a", "b", "sel"]),
    ("logic.counter", "out", &["in", "reset"]),
    ("logic.not", "out", &["in"]),
    ("time.delay", "out", &["in"]),
    ("logic.debounce", "out", &["in"]),
    ("logic.latch", "out", &["set", "reset"]),
    ("cmd", "done", &["trigger"]),
    ("math.expr", "out", &["a", "b"]),
    ("out.osc", "", &["in"]),
    ("out.widget", "", &["in"]),
    ("out.param", "", &["in"]),
    ("out.notify", "", &["in"]),
];

/// The 7 inputs of the catalog, with the name of the output pin of each one.
const ENTRADAS: &[(&str, &str)] = &[
    ("in.widget", "press"),
    ("in.key", "down"),
    ("in.osc", "out"),
    ("in.midi", "out"),
    ("in.marker", "out"),
    ("in.timer", "out"),
    ("in.state", "out"),
];

/// Builds a graph of `n` nodes. `com_expr = false` swaps every `math.expr` for `math.map`.
fn monta(n: usize, com_expr: bool) -> Value {
    let mut nodes = Vec::with_capacity(n);
    let mut edges: Vec<Value> = Vec::new();
    // available output of each node already created: (id, pin) - out.* nodes deliver nothing
    let mut saidas: Vec<(String, &str)> = Vec::new();
    for (i, (tipo, pino)) in ENTRADAS.iter().enumerate() {
        let id = format!("e{i}");
        let mut no = json!({"id": id, "type": tipo});
        no["widget"] = json!("go");
        no["key"] = json!("Space");
        no["address"] = json!("/spell/go");
        no["midi"] = json!("144/60");
        no["marker"] = json!("pico");
        no["every"] = json!(0.25);
        no["what"] = json!("t");
        nodes.push(no);
        saidas.push((id, pino));
    }
    for i in nodes.len()..n {
        let (mut tipo, pino, ins) = MIOLO[i % MIOLO.len()];
        if tipo == "math.expr" && !com_expr {
            tipo = "math.map";
        }
        let id = format!("n{i}");
        let mut no = json!({"id": id, "type": tipo});
        no["expr"] = json!("a * 0.5 + b");
        no["in_max"] = json!(255.0);
        no["out_max"] = json!(1.0);
        no["curve"] = json!("inout");
        no["ms"] = json!(120.0);
        no["cmd"] = json!("cue_go");
        no["address"] = json!("/spell/n");
        no["widget"] = json!("w");
        no["prop"] = json!("glow");
        no["target"] = json!("par1.dim");
        no["text"] = json!("ok");
        nodes.push(no);
        // wires each input to an already existing output: a DAG by construction
        let ins = if tipo == "math.map" { &["in"][..] } else { ins };
        for (k, pin) in ins.iter().enumerate() {
            let (src, sp) = &saidas[(i * 7 + k * 3) % saidas.len()];
            edges.push(json!([format!("{src}.{sp}"), format!("{id}.{pin}")]));
        }
        if !pino.is_empty() {
            saidas.push((id, pino));
        }
    }
    json!({"nodes": nodes, "edges": edges})
}

fn frame(c: &mut Criterion) {
    for (nome, com_expr) in [
        ("graph 500 nodes", true),
        ("graph 500 nodes without expr", false),
    ] {
        let mut g = Graph::new(&monta(500, com_expr), Box::new(NullSink)).unwrap();
        assert_eq!(g.nodes(), 500);
        let mut uni = Universes::new();
        uni.get_or_create(1);
        let mut t = 0.0f64;
        c.bench_function(nome, |b| {
            b.iter(|| {
                t += 1.0 / 60.0;
                g.input("widget:go", 1.0);
                g.frame(t, &mut uni);
            })
        });
    }
}

criterion_group!(benches, frame);
criterion_main!(benches);
