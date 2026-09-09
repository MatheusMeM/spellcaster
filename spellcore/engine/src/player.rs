//! Player standalone: transporte (play/pause/stop/locate/loop) em thread propria sobre o Clock,
//! timeline -> sACN / Art-Net / OSC, cues e ganchos de frame. Porte de
//! `spellcaster/player/player.py` sem a parte de laser (o crate `laser` a resolve).
//!
//! Ordem de avaliacao de um frame (contrato do README, secao "R1"):
//!   1. `Timeline::apply` (tracks `dmx` e `artnet`) e os tracks `media` do Capture;
//!   2. cada `FrameHook` na ordem em que foi registrado;
//!   3. tracks de efeito colateral: `osc`, `media` nao-Capture e `cue`;
//!   4. `CueList::update` escreve o snapshot corrente;
//!   5. o programmer (`Prog`): o override manual do operador, HTP por canal, POR CIMA da cue
//!      viva (o operador sobrepoe o que a cue esta segurando);
//!   6. I/O: cada universo escrito vai para todas as saidas.
//!
//! O transporte remoto por OSC (`/spellcaster/play|pause|stop|locate f`) nunca toca nos
//! Universes: ele so' age no `Handle`, e a thread de transporte faz o resto no proximo frame.

use crate::clock::{Clock, State};
use crate::cues::CueList;
use crate::hook::FrameHook;
use crate::show::{OutputCfg, Show};
use crate::timeline::{Side, Timeline, Value};
use crate::universe::Universes;
use protocols::osc::{Arg, OscIn, OscOut};
use protocols::{artnet::ArtNetOut, sacn::SacnOut, Output};
use serde::Serialize;
use std::collections::BTreeMap;
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

/// Player vivo neste processo (o `CURRENT` do Python): quem os comandos do registry operam.
static CURRENT: Mutex<Option<Handle>> = Mutex::new(None);

/// Ganchos instalados em TODO player que subir neste processo, DEPOIS dos ganchos do show: e'
/// assim que o monitor do `serve` ve o frame que realmente sai, sem o engine conhecer GUI.
type Global = Box<dyn Fn() -> Box<dyn FrameHook> + Send + Sync>;
static GLOBAL: Mutex<Vec<Global>> = Mutex::new(Vec::new());

/// Cadastra uma fabrica de gancho global; vale para os proximos `start()`, nao para o player que
/// ja esta rodando.
// ponytail: so' cadastra, nao remove ; o unico cliente e' o `serve`, que vive o processo inteiro.
pub fn hook_global(f: impl Fn() -> Box<dyn FrameHook> + Send + Sync + 'static) {
    lock(&GLOBAL).push(Box::new(f));
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// ------------------------------------------------------------------- saidas

/// Abre as saidas DMX declaradas em `outputs`. Tipo desconhecido vira aviso, nao erro.
/// (Veio da CLI da R0 sem mudanca de comportamento: o Player e' quem abre as saidas agora.)
pub fn open_outputs(sh: &Show) -> Result<Vec<Box<dyn Output>>, String> {
    let mut outs: Vec<Box<dyn Output>> = Vec::new();
    for c in &sh.outputs {
        match c {
            OutputCfg::Sacn {
                universes,
                priority,
                source_name,
                interfaces,
            } => {
                let ifaces = interfaces.as_ref().map(|v| {
                    v.iter()
                        .filter_map(|s| s.parse::<Ipv4Addr>().ok())
                        .collect::<Vec<_>>()
                });
                let o = SacnOut::with(universes, *priority, source_name, ifaces)
                    .map_err(|e| format!("sacn: {}", e))?;
                outs.push(Box::new(o));
            }
            OutputCfg::ArtNet { targets, broadcast } => {
                let o = ArtNetOut::new(targets.clone(), *broadcast)
                    .map_err(|e| format!("artnet: {}", e))?;
                outs.push(Box::new(o));
            }
            OutputCfg::Osc { .. } => {} // saida de mensagem, nao de universo: vai no `osc_out`
            OutputCfg::Unknown(t) => eprintln!("aviso: saida \"{}\" ignorada", t),
        }
    }
    Ok(outs)
}

/// Primeira saida `osc` do show (o Python tambem guarda so' uma).
fn open_osc(sh: &Show) -> Result<Option<OscOut>, String> {
    for c in &sh.outputs {
        if let OutputCfg::Osc { host, port } = c {
            if *port == 0 {
                eprintln!("aviso: saida osc sem \"port\" ignorada");
                continue;
            }
            return OscOut::new(host, *port)
                .map(Some)
                .map_err(|e| format!("osc: {}", e));
        }
    }
    Ok(None)
}

// ------------------------------------------------------------------ estado

#[derive(Serialize, Clone, Debug)]
pub struct TransportState {
    pub t: f64,
    pub state: &'static str,
    pub cue: i32,
    pub frames: u64,
    pub fps: u32,
    pub duration: Option<f64>,
    pub universes: Vec<u16>,
}

/// Pedido de outra thread para a thread de transporte, consumido no inicio do frame.
enum Ctl {
    Go(Option<usize>),
    /// locate/stop: zera cues e ganchos e reancora o `prev` do disparo por borda.
    Reset(f64),
    /// Evento de entrada para os ganchos ("widget:go", "key:Space", "module:laser/stat/fps").
    Input(String, f64),
}

/// Programmer: a camada manual do operador, por cima da timeline. Um `Option<u8>` por canal =
/// valor e mascara de "tocado" na mesma estrutura.
// ponytail: um bloco de 1 KB por universo ; um show tem poucos universos, e um
// `BTreeMap<(u16,u16),u8>` alocaria por canal dentro do frame.
#[derive(Default)]
pub struct Prog {
    v: BTreeMap<u16, [Option<u8>; 512]>,
    /// Canais soltos desde o ultimo frame: zerados ANTES da timeline, senao o ultimo valor do
    /// override fica preso no buffer (ninguem mais escreve aquele canal).
    freed: Vec<(u16, u16)>,
}

impl Prog {
    /// Escreve `values` a partir de `addr` (1-based); clamp 0..255, ignora o que passa de 512.
    pub fn set(&mut self, u: u16, addr: u16, values: &[f64]) {
        if addr == 0 || addr > 512 {
            return;
        }
        let i = (addr - 1) as usize;
        let s = self.v.entry(u).or_insert([None; 512]);
        for (d, v) in s[i..].iter_mut().zip(values) {
            *d = Some(*v as u8);
        }
    }

    /// Solta o override de um universo (ou de todos); devolve quantos canais sairam.
    pub fn clear(&mut self, u: Option<u16>) -> usize {
        let mut n = 0;
        for (num, s) in self.v.iter_mut() {
            if u.is_some_and(|x| x != *num) {
                continue;
            }
            for (i, c) in s.iter_mut().enumerate() {
                if c.take().is_some() {
                    self.freed.push((*num, i as u16 + 1));
                    n += 1;
                }
            }
        }
        n
    }

    /// Canais tocados, em ordem: (universo, endereco 1-based, valor).
    pub fn levels(&self, u: Option<u16>) -> Vec<(u16, u16, u8)> {
        let mut out = Vec::new();
        for (n, s) in self.v.iter() {
            if u.is_some_and(|x| x != *n) {
                continue;
            }
            out.extend(
                s.iter()
                    .enumerate()
                    .filter_map(|(i, c)| c.map(|v| (*n, i as u16 + 1, v))),
            );
        }
        out
    }

    /// Zera no buffer os canais soltos desde o ultimo frame. Roda ANTES de `Timeline::apply`,
    /// para o que a timeline possui voltar a valer no mesmo frame.
    fn release(&mut self, uni: &mut Universes) {
        for (n, a) in self.freed.drain(..) {
            uni.get_or_create(n).set_bytes(a, &[0]);
        }
    }

    /// HTP por canal sobre o que ja esta no buffer.
    fn apply(&self, uni: &mut Universes) {
        for (n, s) in self.v.iter() {
            let d = &mut uni.get_or_create(*n).data;
            for (i, c) in s.iter().enumerate() {
                if let Some(v) = c {
                    d[i] = d[i].max(*v);
                }
            }
        }
    }
}

struct Shared {
    clock: Clock,
    ctl: Mutex<Vec<Ctl>>,
    prog: Mutex<Prog>,
    cue: AtomicI32,
    frames: AtomicU64,
    nuni: AtomicUsize,
    universes: Mutex<Vec<u16>>,
    duration: Option<f64>,
    run: AtomicBool,
    done: Mutex<bool>,
    cv: Condvar,
}

impl Shared {
    fn push(&self, c: Ctl) {
        lock(&self.ctl).push(c);
    }

    /// Acorda quem esta em `wait()`.
    fn finish(&self) {
        *lock(&self.done) = true;
        self.cv.notify_all();
    }
}

/// Transporte de fora da thread do player. Clonavel; o registry guarda um.
#[derive(Clone)]
pub struct Handle {
    s: Arc<Shared>,
}

impl Handle {
    pub fn play(&self) {
        *lock(&self.s.done) = false;
        self.s.clock.play();
    }

    pub fn pause(&self) {
        self.s.clock.pause();
    }

    /// Para e volta para o zero (o `stop()` do Python: clock.stop + cues.reset + done).
    pub fn stop(&self) {
        self.s.clock.stop();
        self.s.push(Ctl::Reset(0.0));
        self.s.finish();
    }

    pub fn locate(&self, t: f64) {
        self.s.clock.locate(t);
        self.s.push(Ctl::Reset(t));
    }

    pub fn cue_go(&self, index: Option<usize>) {
        self.s.push(Ctl::Go(index));
    }

    /// Entrega `FrameHook::input` a todos os ganchos no proximo frame: e' o comando `input` do
    /// registry, por onde a GUI e o barramento alimentam o Graph.
    pub fn input(&self, key: &str, value: f64) {
        self.s.push(Ctl::Input(key.to_string(), value));
    }

    /// Programmer: escreve no override manual (vale no proximo frame).
    pub fn level_set(&self, u: u16, addr: u16, values: &[f64]) {
        lock(&self.s.prog).set(u, addr, values);
    }

    /// Solta o override de um universo, ou de todos; devolve quantos canais sairam.
    pub fn level_clear(&self, u: Option<u16>) -> usize {
        lock(&self.s.prog).clear(u)
    }

    /// (universo, endereco, valor) de cada canal tocado pelo operador.
    pub fn levels(&self, u: Option<u16>) -> Vec<(u16, u16, u8)> {
        lock(&self.s.prog).levels(u)
    }

    pub fn state(&self) -> TransportState {
        TransportState {
            t: self.s.clock.time(),
            state: match self.s.clock.state() {
                State::Play => "play",
                State::Pause => "pause",
                State::Stop => "stop",
            },
            cue: self.s.cue.load(Ordering::Relaxed),
            frames: self.s.frames.load(Ordering::Relaxed),
            fps: self.s.clock.fps(),
            duration: self.s.duration,
            universes: lock(&self.s.universes).clone(),
        }
    }
}

// --------------------------------------------------------- runtime do frame

/// A parte do player que so' a thread de transporte toca.
struct Rt {
    tl: Timeline,
    cues: CueList,
    hooks: Vec<Box<dyn FrameHook>>,
    uni: Universes,
    outs: Vec<Box<dyn Output>>,
    osc_out: Option<OscOut>,
    looping: bool,
    prev: f64,
    pend: Vec<Ctl>, // fila drenada por swap: zero alocacao por frame
}

impl Rt {
    fn drain(&mut self, s: &Shared, t: f64) {
        {
            let mut q = lock(&s.ctl);
            if q.is_empty() {
                return;
            }
            std::mem::swap(&mut *q, &mut self.pend);
        }
        let mut pend = std::mem::take(&mut self.pend);
        for c in pend.drain(..) {
            match c {
                Ctl::Go(i) => {
                    self.cues.go(t, i);
                }
                Ctl::Reset(x) => {
                    self.cues.reset();
                    for h in self.hooks.iter_mut() {
                        h.reset(x);
                    }
                    self.prev = x;
                }
                Ctl::Input(k, v) => {
                    for h in self.hooks.iter_mut() {
                        h.input(&k, v);
                    }
                }
            }
        }
        self.pend = pend; // devolve o Vec vazio com a capacidade ja alocada
        s.cue.store(self.cues.index(), Ordering::Relaxed);
    }

    fn tick(&mut self, s: &Shared, t: f64) {
        self.drain(s, t);
        // 1. timeline (dmx/artnet) e media do Capture
        lock(&s.prog).release(&mut self.uni);
        self.tl.apply(&mut self.uni, t);
        let Rt {
            tl,
            cues,
            hooks,
            uni,
            outs,
            osc_out,
            prev,
            ..
        } = self;
        let Timeline {
            tracks,
            osc,
            media,
            cue,
            ..
        } = tl;
        for &i in media.iter() {
            let tr = &tracks[i];
            if tr.capture {
                if let Some(v) = tr.media_frame(t) {
                    uni.get_or_create(tr.universe).set(tr.address, &v);
                }
            }
        }
        // 2. ganchos de frame (tracks fx e Graph), na ordem de registro
        for h in hooks.iter_mut() {
            h.frame(t, uni);
        }
        // 3. efeito colateral: OSC, media nao-Capture e cue
        if let Some(o) = osc_out.as_ref() {
            for &i in osc.iter() {
                send_osc(&mut tracks[i], t, o, false);
            }
            for &i in media.iter() {
                if !tracks[i].capture {
                    send_osc(&mut tracks[i], t, o, true);
                }
            }
        }
        for &i in cue.iter() {
            for k in tracks[i].keys.crossed(*prev, t) {
                let idx = match &k.value {
                    Value::Text(v) if v.eq_ignore_ascii_case("go") => None,
                    Value::Text(v) => match v.trim().parse::<usize>() {
                        Ok(n) => Some(n),
                        Err(_) => continue,
                    },
                    Value::Num(n) if *n >= 0.0 => Some(*n as usize),
                    _ => continue,
                };
                cues.go(t, idx);
            }
        }
        // 4. snapshot das cues
        cues.update(t, uni);
        s.cue.store(cues.index(), Ordering::Relaxed);
        // 5. programmer: o override manual do operador, HTP sobre timeline E cue viva
        lock(&s.prog).apply(uni);
        // 6. I/O
        // ponytail: todo universo escrito vai para TODAS as saidas do show (igual a R0)
        // ; separar por saida quando um show misturar "dmx" e "artnet" no mesmo universo.
        for u in uni.iter() {
            for o in outs.iter_mut() {
                o.send(u.number, &u.data);
            }
        }
        *prev = t;
        s.frames.fetch_add(1, Ordering::Relaxed);
        if uni.len() != s.nuni.load(Ordering::Relaxed) {
            s.nuni.store(uni.len(), Ordering::Relaxed);
            *lock(&s.universes) = uni.numbers();
        }
    }

    fn close(&mut self) {
        for o in self.outs.iter_mut() {
            o.close();
        }
        if let Some(o) = self.osc_out.as_mut() {
            o.close();
        }
    }
}

/// Envia o valor corrente de um track de efeito colateral quando ele muda.
// ponytail: monta `Vec<Arg>` e `String` so' na MUDANCA de valor, nao por frame ; virar buffer
// reaproveitado se algum show passar a ter track OSC que muda todo frame com muitos argumentos.
fn send_osc(tr: &mut crate::timeline::Track, t: f64, out: &OscOut, media: bool) {
    match tr.changed(t) {
        Side::Same => {}
        // ponytail: numero vai como float OSC ('f'); o Python manda 'i' quando o keyframe e'
        // inteiro ; casar o tipo se algum aparelho recusar float.
        Side::Nums => {
            if media {
                let v = tr.nums().first().copied().unwrap_or(0.0);
                out.send(&media_address(tr.text_address.as_str(), &fmt(v)), &[]);
            } else {
                let args: Vec<Arg> = tr.nums().iter().map(|v| Arg::Float(*v as f32)).collect();
                out.send(&tr.text_address, &args);
            }
        }
        Side::Text => {
            if media {
                out.send(&media_address(tr.text_address.as_str(), tr.text()), &[]);
            } else {
                out.send(&tr.text_address, &[Arg::Str(tr.text().to_string())]);
            }
        }
    }
}

/// media nao-Capture: "endereco/valor" (o `rstrip("/") + "/" + str(v)` do Python).
fn media_address(address: &str, v: &str) -> String {
    format!("{}/{}", address.trim_end_matches('/'), v)
}

fn fmt(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{}", v)
    }
}

// ------------------------------------------------------------------- Player

pub struct Player {
    rt: Option<Rt>, // vai para a thread no start()
    s: Arc<Shared>,
    th: Option<JoinHandle<()>>,
    osc_in: Option<OscIn>,
    osc_port: Option<u16>,
    base: PathBuf,
}

impl Player {
    /// Carrega timeline + cues e abre as saidas de `show.outputs` (sacn, artnet, osc).
    /// `base` = diretorio do .spell (caminhos de `fx`/clipes sao relativos a ele).
    pub fn new(show: Show, base: PathBuf, looping: bool) -> Result<Player, String> {
        let tl = Timeline::new(&show)?;
        let ign = tl.ignored().join(", ");
        if !ign.is_empty() {
            eprintln!("aviso: tracks ignorados: {}", ign);
        }
        let cues = CueList::new(
            show.extra
                .get("cues")
                .and_then(|v| v.as_array())
                .map(|a| a.as_slice())
                .unwrap_or(&[]),
        );
        let outs = open_outputs(&show)?;
        let osc_out = open_osc(&show)?;
        let osc_port = show
            .extra
            .get("transport")
            .and_then(|v| v.get("osc_port"))
            .and_then(|v| v.as_u64())
            .map(|p| p as u16);
        Ok(Player {
            s: Arc::new(Shared {
                clock: Clock::new(show.fps),
                ctl: Mutex::new(Vec::new()),
                prog: Mutex::new(Prog::default()),
                cue: AtomicI32::new(-1),
                frames: AtomicU64::new(0),
                nuni: AtomicUsize::new(0),
                universes: Mutex::new(Vec::new()),
                duration: show.duration,
                run: AtomicBool::new(false),
                done: Mutex::new(false),
                cv: Condvar::new(),
            }),
            rt: Some(Rt {
                tl,
                cues,
                hooks: Vec::new(),
                uni: Universes::new(),
                outs,
                osc_out,
                looping,
                prev: 0.0,
                pend: Vec::new(),
            }),
            th: None,
            osc_in: None,
            osc_port,
            base,
        })
    }

    /// Ganchos de script/graph, antes de `start()`, na ordem em que devem rodar.
    pub fn hook(&mut self, h: Box<dyn FrameHook>) {
        if let Some(rt) = self.rt.as_mut() {
            rt.hooks.push(h);
        }
    }

    /// Saida extra alem das declaradas no show, antes de `start()`.
    pub fn output(&mut self, o: Box<dyn Output>) {
        if let Some(rt) = self.rt.as_mut() {
            rt.outs.push(o);
        }
    }

    pub fn handle(&self) -> Handle {
        Handle { s: self.s.clone() }
    }

    pub fn clock(&self) -> Clock {
        self.s.clock.clone()
    }

    /// Diretorio do .spell (a CLI resolve `fx` e clipes a partir dele).
    pub fn base(&self) -> &Path {
        &self.base
    }

    /// Sobe a thread de transporte e, se `osc_port` (argumento ou `transport.osc_port`), o
    /// OscIn do transporte remoto. Registra este player como o `current()`.
    pub fn start(&mut self, osc_port: Option<u16>) -> Result<(), String> {
        if self.th.is_some() {
            return Ok(());
        }
        let mut rt = self.rt.take().ok_or("player ja encerrado")?;
        for f in lock(&GLOBAL).iter() {
            rt.hooks.push(f());
        }
        let s = self.s.clone();
        s.run.store(true, Ordering::Relaxed);
        let th = std::thread::Builder::new()
            .name("spell-transport".into())
            .spawn(move || transport(rt, s))
            .map_err(|e| format!("thread de transporte: {}", e))?;
        self.th = Some(th);
        if let Some(p) = osc_port.or(self.osc_port).filter(|p| *p > 0) {
            let mut i = OscIn::new(p).map_err(|e| format!("osc {}: {}", p, e))?;
            let (a, b, c, d) = (
                self.handle(),
                self.handle(),
                self.handle(),
                self.handle(),
            );
            i.on("/spellcaster/play", move |_, _| a.play());
            i.on("/spellcaster/pause", move |_, _| b.pause());
            i.on("/spellcaster/stop", move |_, _| c.stop());
            i.on("/spellcaster/locate", move |_, g| {
                d.locate(g.first().map(argf).unwrap_or(0.0))
            });
            self.osc_in = Some(i);
        }
        *lock(&CURRENT) = Some(self.handle());
        Ok(())
    }

    /// Bloqueia ate stop() ou o fim do show. `false` = estourou o timeout.
    pub fn wait(&self, timeout: Option<Duration>) -> bool {
        let d = lock(&self.s.done);
        match timeout {
            None => {
                let mut d = d;
                while !*d {
                    d = self.s.cv.wait(d).unwrap_or_else(|e| e.into_inner());
                }
                true
            }
            Some(to) => {
                let (_g, r) = self
                    .s
                    .cv
                    .wait_timeout_while(d, to, |x| !*x)
                    .unwrap_or_else(|e| e.into_inner());
                !r.timed_out()
            }
        }
    }

    /// Idempotente: para a thread, fecha as saidas e limpa o `current()`.
    pub fn close(&mut self) {
        self.s.run.store(false, Ordering::Relaxed);
        self.s.clock.stop();
        self.s.finish();
        if let Some(th) = self.th.take() {
            let _ = th.join();
        }
        if let Some(mut i) = self.osc_in.take() {
            i.close();
        }
        if let Some(mut rt) = self.rt.take() {
            rt.close(); // start() nunca foi chamado: as saidas ainda estao aqui
        }
        let mut c = lock(&CURRENT);
        if c.as_ref().is_some_and(|h| Arc::ptr_eq(&h.s, &self.s)) {
            *c = None;
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.close();
    }
}

fn argf(a: &Arg) -> f64 {
    match a {
        Arg::Int(v) => *v as f64,
        Arg::Long(v) => *v as f64,
        Arg::Float(v) => *v as f64,
        Arg::Double(v) => *v,
        Arg::Bool(v) => *v as i32 as f64,
        Arg::Str(s) => s.trim().parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

/// Thread de transporte: o `_loop` do Python.
fn transport(mut rt: Rt, s: Arc<Shared>) {
    while s.run.load(Ordering::Relaxed) {
        if s.clock.state() == State::Stop {
            let t = s.clock.time();
            rt.drain(&s, t); // locate/stop com o relogio parado tambem zera cues e ganchos
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }
        {
            let sc = s.clone();
            s.clock.run(|t| rt.tick(&sc, t), s.duration);
        }
        if !s.run.load(Ordering::Relaxed) {
            break;
        }
        if s.clock.state() != State::Stop {
            // chegou na duracao: repete do inicio ou para
            // ponytail: `looping` sem `duration` nao repete — sem fim, o run so' volta no stop.
            if rt.looping && s.duration.is_some() {
                s.clock.locate(0.0);
                rt.cues.reset();
                for h in rt.hooks.iter_mut() {
                    h.reset(0.0);
                }
                rt.prev = 0.0;
            } else {
                s.clock.stop();
                rt.cues.reset();
                for h in rt.hooks.iter_mut() {
                    h.reset(0.0);
                }
                rt.prev = 0.0;
                s.cue.store(-1, Ordering::Relaxed);
                s.finish();
            }
        }
    }
    rt.close();
}

/// Player vivo neste processo (o `CURRENT` do Python). Usado pelos comandos do registry.
pub fn current() -> Option<Handle> {
    lock(&CURRENT).clone()
}
