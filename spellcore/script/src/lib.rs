//! Crate `script` da R1: o track `fx` em Rhai e o Graph runtime (secao 10 do PRD).
//! Nada aqui conhece rede nem GUI; o engine nao conhece Rhai (o contato e' `engine::hook`).
//!
//! # API que o host expoe ao arquivo `.rhai` (contrato)
//!
//! ```text
//! set(universe, addr, values)  escreve no universo dado; `values` e' um array de numeros
//!                              ou um numero solto. Mesma semantica de engine::Universe::set:
//!                              addr 1-based, truncagem para zero, clamp 0..255, e o que
//!                              passar do canal 512 (ou addr fora de 1..512) e' ignorado.
//! set(addr, values)            idem, no universo do track.
//! st(i)          -> float      le o slot i do estado que persiste ENTRE frames (0.0 no inicio).
//! st(i, v)                     escreve o slot i. O vetor cresce sozinho; reset() zera tudo.
//! ```
//!
//! Mais a biblioteca padrao do Rhai: `sin`, `cos`, `tan`, `exp`, `ln`, `sqrt`, `abs`,
//! `atan(y, x)` (= atan2), `hypot`, `to_degrees`, `to_radians`, `int` (trunca, devolve float),
//! `to_int` (trunca, devolve inteiro), `floor`, `ceiling`, `round`, `PI()`, `E()`, arrays.
//! Sem arquivo, sem rede, sem processo, sem relogio: o Engine sobe com `default-features = false`
//! e o Rhai nao traz nenhum desses pacotes.
//!
//! O script PRECISA definir `fn look(t)`. O topo do arquivo roda UMA vez, na carga.
//! `print`/`debug` do script saem em stderr (stdout e' da CLI).

use engine::hook::{EventSink, FrameHook};
use engine::{Show, Universes};
use rhai::{Array, CallFnOptions, Dynamic, Engine, Scope, AST};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub mod graph;

pub use graph::Graph;

/// Escritas do frame corrente + estado persistente do script.
/// O `set()` do Rhai nao pode receber `&mut Universes` (a closure registrada e' 'static),
/// entao ele anota aqui e o `frame()` reproduz nos Universes no fim. `log`/`buf` sao limpos
/// com `clear()` (mantem a capacidade): zero alocacao depois do primeiro frame.
#[derive(Default)]
struct Pend {
    /// (universo, addr 1-based, offset em `buf`, quantidade)
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
        // `v as u8` no Rust trunca para zero e satura: igual ao max(0, min(255, int(v))) do Python.
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
        // ponytail: teto de 4096 slots de estado ; subir se algum show precisar de mais.
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
    e.set_max_expr_depths(256, 256); // ponytail: default 64/32 recusa o medgrupo.rhai ; show e confiavel, subir se algum script bater
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
    e.register_fn("set", move |a: i64, v: Array| p.lock().unwrap().write_arr(u, a, &v));
    let p = pend.clone();
    e.register_fn("set", move |a: i64, v: f64| p.lock().unwrap().write(u, a, &[v]));
    let p = pend.clone();
    e.register_fn("set", move |a: i64, v: i64| p.lock().unwrap().write(u, a, &[v as f64]));
    let p = pend.clone();
    e.register_fn("st", move |i: i64| p.lock().unwrap().st(i));
    let p = pend.clone();
    e.register_fn("st", move |i: i64, v: f64| p.lock().unwrap().st_set(i, v));
    let p = pend.clone();
    e.register_fn("st", move |i: i64, v: i64| p.lock().unwrap().st_set(i, v as f64));
}

/// Track `{"type":"fx","script":"...rhai","universe":N}`: compila o `.rhai` uma vez e roda
/// `look(t)` por frame, com o estado do script preservado entre frames.
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
            return Err(format!("{}: precisa definir fn look(t)", path.display()));
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
        // eval_ast(false): o topo do arquivo ja' rodou na carga; por frame so' a funcao.
        let opts = CallFnOptions::new().eval_ast(false);
        if let Err(e) =
            self.eng
                .call_fn_with_options::<Dynamic>(opts, &mut self.scope, &self.ast, "look", (t,))
        {
            // ponytail: erro em runtime desliga o hook e imprime UMA vez ; sem isso um erro de
            // script vira 30 linhas por segundo no console do operador. reset() religa.
            eprintln!("{}: {} -- fx desligado ate o proximo reset", self.nome, e);
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

/// Hooks de um show, na ordem de execucao: um `Fx` por track `"fx"` (na ordem do arquivo,
/// caminho relativo a `base`) e, depois, o `Graph` de `show["graph"]` se existir.
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
            .ok_or_else(|| "track fx sem \"script\"".to_string())?;
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
    fn set_tem_a_semantica_do_universe() {
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
        assert!(u.get(2).is_none(), "addr 513 tem que ser ignorado");
    }

    #[test]
    fn estado_persiste_entre_frames_e_reset_zera() {
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
    fn script_sem_look_e_erro_na_carga() {
        let p = escreve("spellcore_fx_semlook.rhai", "fn outra(t) { }");
        let e = match Fx::new(&p, 1) {
            Err(e) => e,
            Ok(_) => panic!("script sem look(t) tinha que falhar"),
        };
        std::fs::remove_file(&p).ok();
        assert!(e.contains("look"), "{e}");
    }

    #[test]
    fn hooks_le_os_tracks_fx_na_ordem() {
        let dir = std::env::temp_dir();
        std::fs::write(dir.join("spellcore_h1.rhai"), "fn look(t) { set(1, 1, 11); }").unwrap();
        std::fs::write(dir.join("spellcore_h2.rhai"), "fn look(t) { set(1, 2, 22); }").unwrap();
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
