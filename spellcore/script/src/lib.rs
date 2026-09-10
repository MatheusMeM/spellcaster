//! Crate `script` of R1: the `fx` track in Rhai and the Graph runtime (section 10 of the PRD).
//! Nothing here knows network or GUI; the engine does not know Rhai (the contact is
//! `engine::hook`).
//!
//! # API the host exposes to the `.rhai` file (contract)
//!
//! ```text
//! set(universe, addr, values)  writes into the given universe; `values` is an array of
//!                              numbers or a bare number. Same semantics as
//!                              engine::Universe::set: addr 1-based, truncation toward zero,
//!                              clamp 0..255, and whatever goes past channel 512 (or an addr
//!                              outside 1..512) is ignored.
//! set(addr, values)            same, in the track universe.
//! st(i)          -> float      reads slot i of the state that persists BETWEEN frames
//!                              (0.0 at the start).
//! st(i, v)                     writes slot i. The vector grows on its own; reset() zeroes
//!                              everything.
//! ```
//!
//! Plus the Rhai standard library: `sin`, `cos`, `tan`, `exp`, `ln`, `sqrt`, `abs`,
//! `atan(y, x)` (= atan2), `hypot`, `to_degrees`, `to_radians`, `int` (truncates, returns a
//! float), `to_int` (truncates, returns an integer), `floor`, `ceiling`, `round`, `PI()`,
//! `E()`, arrays. No file, no network, no process, no clock: the Engine starts with
//! `default-features = false` and Rhai brings none of those packages.
//!
//! The script MUST define `fn look(t)`. The top of the file runs ONCE, at load time.
//! The script's `print`/`debug` go to stderr (stdout belongs to the CLI).

use engine::hook::{EventSink, FrameHook};
use engine::{Show, Universes};
use rhai::{Array, CallFnOptions, Dynamic, Engine, Scope, AST};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub mod graph;

pub use graph::Graph;

/// Writes of the current frame + persistent state of the script.
/// Rhai's `set()` cannot take a `&mut Universes` (the registered closure is 'static), so it
/// notes them here and `frame()` replays them into the Universes at the end. `log`/`buf` are
/// cleared with `clear()` (keeping the capacity): zero allocation after the first frame.
#[derive(Default)]
struct Pend {
    /// (universe, 1-based addr, offset in `buf`, count)
    log: Vec<(u16, u16, u32, u32)>,
    buf: Vec<u8>,
    st: Vec<f64>,
}

pub(crate) fn num(v: &Dynamic) -> f64 {
    if let Some(f) = v.clone().try_cast::<f64>() {
        f
    } else if let Some(i) = v.clone().try_cast::<i64>() {
        i as f64
    } else {
        0.0
    }
}

impl Pend {
    fn ok(uni: i64, addr: i64) -> bool {
        (1..=65535).contains(&uni) && (1..=512).contains(&addr)
    }

    fn anota(&mut self, uni: i64, addr: i64, off: u32) {
        let n = self.buf.len() as u32 - off;
        self.log.push((uni as u16, addr as u16, off, n));
    }

    fn write(&mut self, uni: i64, addr: i64, vals: &[f64]) {
        if !Pend::ok(uni, addr) {
            return;
        }
        let off = self.buf.len() as u32;
        // `v as u8` in Rust truncates toward zero and saturates: same as Python's
        // max(0, min(255, int(v))).
        self.buf.extend(vals.iter().map(|v| *v as u8));
        self.anota(uni, addr, off);
    }

    fn write_arr(&mut self, uni: i64, addr: i64, vals: &Array) {
        if !Pend::ok(uni, addr) {
            return;
        }
        let off = self.buf.len() as u32;
        self.buf.extend(vals.iter().map(|v| num(v) as u8));
        self.anota(uni, addr, off);
    }

    fn st(&self, i: i64) -> f64 {
        if i < 0 {
            return 0.0;
        }
        self.st.get(i as usize).copied().unwrap_or(0.0)
    }

    fn st_set(&mut self, i: i64, v: f64) {
        // ponytail: ceiling of 4096 state slots ; raise it if some show needs more.
        if !(0..4096).contains(&i) {
            return;
        }
        if self.st.len() <= i as usize {
            self.st.resize(i as usize + 1, 0.0);
        }
        self.st[i as usize] = v;
    }
}

fn base_engine() -> Engine {
    let mut e = Engine::new();
    // ponytail: the 64/32 default rejects medgrupo.rhai ; the show is trusted, raise it if some
    // script hits the limit
    e.set_max_expr_depths(256, 256);
    e.on_print(|s| eprintln!("fx: {s}"));
    e.on_debug(|s, _, pos| eprintln!("fx {pos}: {s}"));
    e
}

fn register(e: &mut Engine, pend: &Arc<Mutex<Pend>>, uni: u16) {
    let u = uni as i64;
    let p = pend.clone();
    e.register_fn("set", move |un: i64, a: i64, v: Array| {
        p.lock().unwrap().write_arr(un, a, &v)
    });
    let p = pend.clone();
    e.register_fn("set", move |un: i64, a: i64, v: f64| {
        p.lock().unwrap().write(un, a, &[v])
    });
    let p = pend.clone();
    e.register_fn("set", move |un: i64, a: i64, v: i64| {
        p.lock().unwrap().write(un, a, &[v as f64])
    });
    let p = pend.clone();
    e.register_fn("set", move |a: i64, v: Array| {
        p.lock().unwrap().write_arr(u, a, &v)
    });
    let p = pend.clone();
    e.register_fn("set", move |a: i64, v: f64| {
        p.lock().unwrap().write(u, a, &[v])
    });
    let p = pend.clone();
    e.register_fn("set", move |a: i64, v: i64| {
        p.lock().unwrap().write(u, a, &[v as f64])
    });
    let p = pend.clone();
    e.register_fn("st", move |i: i64| p.lock().unwrap().st(i));
    let p = pend.clone();
    e.register_fn("st", move |i: i64, v: f64| p.lock().unwrap().st_set(i, v));
    let p = pend.clone();
    e.register_fn("st", move |i: i64, v: i64| {
        p.lock().unwrap().st_set(i, v as f64)
    });
}

/// Track `{"type":"fx","script":"...rhai","universe":N}`: compiles the `.rhai` once and runs
/// `look(t)` per frame, with the script state preserved between frames.
pub struct Fx {
    eng: Engine,
    ast: AST,
    scope: Scope<'static>,
    pend: Arc<Mutex<Pend>>,
    nome: String,
    off: bool,
}

impl Fx {
    pub fn new(path: &Path, universe: u16) -> Result<Fx, String> {
        let src =
            std::fs::read_to_string(path).map_err(|e| format!("{}: {}", path.display(), e))?;
        let pend = Arc::new(Mutex::new(Pend::default()));
        let mut eng = base_engine();
        register(&mut eng, &pend, universe);
        let ast = eng
            .compile(&src)
            .map_err(|e| format!("{}: {}", path.display(), e))?;
        if !ast
            .iter_functions()
            .any(|f| f.name == "look" && f.params.len() == 1)
        {
            return Err(format!("{}: must define fn look(t)", path.display()));
        }
        let mut scope = Scope::new();
        eng.run_ast_with_scope(&mut scope, &ast)
            .map_err(|e| format!("{}: {}", path.display(), e))?;
        Ok(Fx {
            eng,
            ast,
            scope,
            pend,
            nome: path.display().to_string(),
            off: false,
        })
    }
}

impl FrameHook for Fx {
    fn frame(&mut self, t: f64, uni: &mut Universes) {
        if self.off {
            return;
        }
        {
            let mut p = self.pend.lock().unwrap();
            p.log.clear();
            p.buf.clear();
        }
        // eval_ast(false): the top of the file already ran at load time; per frame only the
        // function.
        let opts = CallFnOptions::new().eval_ast(false);
        if let Err(e) =
            self.eng
                .call_fn_with_options::<Dynamic>(opts, &mut self.scope, &self.ast, "look", (t,))
        {
            // ponytail: a runtime error switches the hook off and prints ONCE ; without this a
            // script error becomes 30 lines per second in the operator console. reset() turns
            // it back on.
            eprintln!("{}: {} -- fx off until the next reset", self.nome, e);
            self.off = true;
            return;
        }
        let p = self.pend.lock().unwrap();
        for &(u, a, o, n) in &p.log {
            uni.get_or_create(u)
                .set_bytes(a, &p.buf[o as usize..(o + n) as usize]);
        }
    }

    fn reset(&mut self, _t: f64) {
        {
            let mut p = self.pend.lock().unwrap();
            p.st.iter_mut().for_each(|v| *v = 0.0);
            p.log.clear();
            p.buf.clear();
        }
        self.off = false;
    }
}

/// Hooks of a show, in execution order: one `Fx` per `"fx"` track (in file order, path
/// relative to `base`) and then the `Graph` of `show["graph"]` if it exists.
pub fn hooks(
    show: &Show,
    base: &Path,
    sink: Box<dyn EventSink>,
) -> Result<Vec<Box<dyn FrameHook>>, String> {
    let mut v: Vec<Box<dyn FrameHook>> = Vec::new();
    for tr in &show.tracks {
        if tr.get("type").and_then(|x| x.as_str()) != Some("fx") {
            continue;
        }
        let f = tr
            .get("script")
            .and_then(|x| x.as_str())
            .ok_or_else(|| "fx track without \"script\"".to_string())?;
        let u = tr.get("universe").and_then(|x| x.as_u64()).unwrap_or(1) as u16;
        v.push(Box::new(Fx::new(&base.join(f), u)?));
    }
    if let Some(g) = show.extra.get("graph") {
        v.push(Box::new(Graph::new_in(g, sink, base)?));
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn escreve(nome: &str, src: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(nome);
        std::fs::write(&p, src).unwrap();
        p
    }

    #[test]
    fn set_has_the_universe_semantics() {
        let p = escreve(
            "spellcore_fx_set.rhai",
            "fn look(t) { set(1, [10.0, -5.0, 300.0, 255.9]); set(2, 513, 1); set(9, 0.0); }",
        );
        let mut fx = Fx::new(&p, 7).unwrap();
        let mut u = Universes::new();
        fx.frame(0.0, &mut u);
        std::fs::remove_file(&p).ok();
        let d = &u.get(7).unwrap().data;
        assert_eq!(&d[0..4], &[10, 0, 255, 255]);
        assert_eq!(d[8], 0);
        assert!(u.get(2).is_none(), "addr 513 must be ignored");
    }

    #[test]
    fn state_persists_between_frames_and_reset_zeroes_it() {
        let p = escreve(
            "spellcore_fx_st.rhai",
            "fn look(t) { st(0, st(0) + 1.0); set(1, 1, st(0)); }",
        );
        let mut fx = Fx::new(&p, 1).unwrap();
        let mut u = Universes::new();
        for i in 0..5 {
            fx.frame(i as f64, &mut u);
        }
        assert_eq!(u.get(1).unwrap().data[0], 5);
        fx.reset(0.0);
        fx.frame(0.0, &mut u);
        std::fs::remove_file(&p).ok();
        assert_eq!(u.get(1).unwrap().data[0], 1);
    }

    #[test]
    fn a_script_without_look_is_a_load_error() {
        let p = escreve("spellcore_fx_semlook.rhai", "fn outra(t) { }");
        let e = match Fx::new(&p, 1) {
            Err(e) => e,
            Ok(_) => panic!("a script without look(t) had to fail"),
        };
        std::fs::remove_file(&p).ok();
        assert!(e.contains("look"), "{e}");
    }

    #[test]
    fn hooks_reads_the_fx_tracks_in_order() {
        let dir = std::env::temp_dir();
        std::fs::write(
            dir.join("spellcore_h1.rhai"),
            "fn look(t) { set(1, 1, 11); }",
        )
        .unwrap();
        std::fs::write(
            dir.join("spellcore_h2.rhai"),
            "fn look(t) { set(1, 2, 22); }",
        )
        .unwrap();
        let sh: Show = serde_json::from_str(
            r#"{"version":1,"tracks":[{"type":"fx","script":"spellcore_h1.rhai","universe":1},
                                      {"type":"dmx","universe":1,"address":1,"keys":[]},
                                      {"type":"fx","script":"spellcore_h2.rhai","universe":1}]}"#,
        )
        .unwrap();
        let mut hs = hooks(&sh, &dir, Box::new(engine::NullSink)).unwrap();
        assert_eq!(hs.len(), 2);
        let mut u = Universes::new();
        for h in hs.iter_mut() {
            h.frame(0.0, &mut u);
        }
        assert_eq!(&u.get(1).unwrap().data[0..2], &[11, 22]);
    }
}
