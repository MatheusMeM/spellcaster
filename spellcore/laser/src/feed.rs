//! Um `Feed` = um DAC com thread propria, ring de frames de tamanho fixo e safety
//! obrigatoria antes de todo envio.
//!
//! Regra do PRD (secao 3): cada saida em sua thread com fila de tamanho fixo; frame velho
//! e descartado, nunca enfileirado. `push` do engine nunca bloqueia.
//! Caminho quente sem alocacao: os slots do ring, o buffer de trabalho da thread e o
//! buffer de comando do DAC sao reusados; depois do primeiro frame nada mais cresce.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::dac::Dac;
use crate::frame::{clamp_c, Point, Safety};

/// Transformacao por frame do track de laser (`keys` do `.spell`).
/// Ordem: rotacao em torno da origem, escala, translacao. Cor multiplica por canal.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Transform {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    /// Graus.
    pub rot: f64,
    pub color: (f64, f64, f64),
}

impl Default for Transform {
    fn default() -> Transform {
        Transform {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
            rot: 0.0,
            color: (1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn is_identity(&self) -> bool {
        self.x == 0.0
            && self.y == 0.0
            && self.scale == 1.0
            && self.rot == 0.0
            && self.color == (1.0, 1.0, 1.0)
    }

    /// Reusa `out`; nao aloca depois do primeiro frame do mesmo tamanho.
    pub fn apply(&self, src: &[Point], out: &mut Vec<Point>) {
        out.clear();
        out.reserve(src.len());
        if self.is_identity() {
            out.extend_from_slice(src);
            return;
        }
        let (s, c) = self.rot.to_radians().sin_cos();
        let k = self.scale;
        for p in src {
            let (px, py) = (p.x as f64, p.y as f64);
            out.push(Point::new(
                (px * c - py * s) * k + self.x,
                (px * s + py * c) * k + self.y,
                clamp_c(p.r as f64 * self.color.0),
                clamp_c(p.g as f64 * self.color.1),
                clamp_c(p.b as f64 * self.color.2),
                p.blank,
            ));
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FeedStats {
    /// Frames entregues ao DAC.
    pub sent: u64,
    /// Frames descartados por ring cheio (o engine correu mais que o DAC).
    pub dropped: u64,
    pub errors: u64,
    /// Jitter de entrega: desvio do intervalo entre envios em relacao a media, em segundos.
    pub p50: f64,
    pub p99: f64,
    pub max: f64,
    /// Tempo de CPU da thread do feed, em segundos (Windows: GetThreadTimes).
    pub cpu: f64,
}

const JITTER_N: usize = 4096;

struct Ring {
    /// N+1 slots: a thread fica com um enquanto o produtor escreve nos outros.
    slots: Vec<Vec<Point>>,
    head: usize,
    len: usize,
    closed: bool,
}

struct Shared {
    ring: Mutex<Ring>,
    cv: Condvar,
    params: Mutex<(Transform, Safety)>,
    stats: Mutex<(u64, u64, u64, f64, Vec<f64>)>, // sent, dropped, errors, cpu, intervalos
    run: AtomicBool,
}

pub struct Feed {
    shared: Arc<Shared>,
    handle: Option<JoinHandle<()>>,
    name: String,
}

impl Feed {
    /// Sobe a thread do DAC. `ring` = frames em voo; 2 ou 3 basta (frame velho nao serve
    /// para nada num laser). `safety` e obrigatoria e pode ser trocada em runtime, nunca
    /// desligada.
    pub fn start(
        mut dac: Box<dyn Dac>,
        pps: u32,
        ring: usize,
        safety: Safety,
    ) -> std::io::Result<Feed> {
        let name = dac.name();
        dac.begin(pps)?;
        let n = ring.max(1) + 1;
        let shared = Arc::new(Shared {
            ring: Mutex::new(Ring {
                slots: (0..n).map(|_| Vec::new()).collect(),
                head: 0,
                len: 0,
                closed: false,
            }),
            cv: Condvar::new(),
            params: Mutex::new((Transform::default(), safety)),
            stats: Mutex::new((0, 0, 0, 0.0, Vec::with_capacity(JITTER_N))),
            run: AtomicBool::new(true),
        });
        let s = shared.clone();
        let handle = std::thread::spawn(move || run(dac, s));
        Ok(Feed {
            shared,
            handle: Some(handle),
            name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Entrega um frame. Nunca bloqueia: com o ring cheio, o frame MAIS VELHO e descartado.
    pub fn push(&self, points: &[Point]) {
        let mut r = self.shared.ring.lock().unwrap_or_else(|e| e.into_inner());
        if r.closed {
            return;
        }
        let n = r.slots.len();
        let dropped = if r.len == n - 1 {
            r.head = (r.head + 1) % n;
            r.len -= 1;
            true
        } else {
            false
        };
        let i = (r.head + r.len) % n;
        r.slots[i].clear();
        r.slots[i].extend_from_slice(points);
        r.len += 1;
        drop(r);
        self.shared.cv.notify_one();
        if dropped {
            self.shared
                .stats
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .1 += 1;
        }
    }

    pub fn set_transform(&self, t: Transform) {
        self.shared
            .params
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .0 = t;
    }

    pub fn set_safety(&self, s: Safety) {
        self.shared
            .params
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .1 = s;
    }

    pub fn stats(&self) -> FeedStats {
        let g = self.shared.stats.lock().unwrap_or_else(|e| e.into_inner());
        let (sent, dropped, errors, cpu, iv) = (g.0, g.1, g.2, g.3, &g.4);
        let mut st = FeedStats {
            sent,
            dropped,
            errors,
            cpu,
            ..FeedStats::default()
        };
        if iv.len() > 1 {
            let mean = iv.iter().sum::<f64>() / iv.len() as f64;
            let mut d: Vec<f64> = iv.iter().map(|x| (x - mean).abs()).collect();
            d.sort_by(|a, b| a.partial_cmp(b).unwrap());
            st.p50 = d[d.len() / 2];
            st.p99 = d[(d.len() as f64 * 0.99) as usize % d.len()];
            st.max = *d.last().unwrap();
        }
        st
    }

    /// Para a thread e apaga o DAC. Idempotente; o `Drop` chama.
    pub fn stop(&mut self) {
        self.shared.run.store(false, Ordering::Relaxed);
        {
            let mut r = self.shared.ring.lock().unwrap_or_else(|e| e.into_inner());
            r.closed = true;
        }
        self.shared.cv.notify_all();
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for Feed {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run(mut dac: Box<dyn Dac>, sh: Arc<Shared>) {
    let cpu0 = thread_cpu();
    let mut work: Vec<Point> = Vec::new();
    let mut scratch: Vec<Point> = Vec::new();
    let mut last: Option<Instant> = None;
    let (mut sent, mut errors) = (0u64, 0u64);
    let mut iv: Vec<f64> = Vec::with_capacity(JITTER_N);
    while sh.run.load(Ordering::Relaxed) {
        // ---- pega o frame mais velho do ring, trocando o buffer (sem copiar, sem alocar)
        {
            let mut r = sh.ring.lock().unwrap_or_else(|e| e.into_inner());
            while r.len == 0 && !r.closed {
                let (g, _) = sh
                    .cv
                    .wait_timeout(r, Duration::from_millis(100))
                    .unwrap_or_else(|e| e.into_inner());
                r = g;
            }
            if r.len == 0 {
                if r.closed {
                    break;
                }
                continue;
            }
            let i = r.head;
            let n = r.slots.len();
            r.head = (r.head + 1) % n;
            r.len -= 1;
            std::mem::swap(&mut scratch, &mut r.slots[i]);
        }
        let (tf, safety) = *sh.params.lock().unwrap_or_else(|e| e.into_inner());
        tf.apply(&scratch, &mut work);
        // SAFETY DO ENGINE: sempre, depois da transformacao (que pode encolher a figura).
        safety.apply(&mut work);
        match dac.send(&work) {
            Ok(()) => {
                sent += 1;
                let now = Instant::now();
                if let Some(t) = last {
                    if iv.len() < JITTER_N {
                        iv.push(now.duration_since(t).as_secs_f64());
                    }
                }
                last = Some(now);
            }
            Err(_) => {
                errors += 1;
                last = None;
            }
        }
        let mut g = sh.stats.lock().unwrap_or_else(|e| e.into_inner());
        g.0 = sent;
        g.2 = errors;
    }
    dac.stop();
    let cpu = match (cpu0, thread_cpu()) {
        (Some(a), Some(b)) => b - a,
        _ => 0.0,
    };
    let mut g = sh.stats.lock().unwrap_or_else(|e| e.into_inner());
    g.0 = sent;
    g.2 = errors;
    g.3 = cpu;
    g.4 = iv;
}

// -------- CPU da propria thread. Mesmo padrao do bench/src/bin/throughput.rs.
// ponytail: extern "system" direto, sem windows-sys por dois simbolos ; e o que o resto
// do spellcore ja faz.

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentThread() -> isize;
    fn GetThreadTimes(h: isize, c: *mut u64, e: *mut u64, k: *mut u64, u: *mut u64) -> i32;
}

#[cfg(windows)]
fn thread_cpu() -> Option<f64> {
    let (mut c, mut e, mut k, mut u) = (0u64, 0u64, 0u64, 0u64);
    unsafe {
        if GetThreadTimes(GetCurrentThread(), &mut c, &mut e, &mut k, &mut u) == 0 {
            return None;
        }
    }
    Some((k + u) as f64 * 1e-7) // FILETIME conta 100 ns
}

#[cfg(not(windows))]
fn thread_cpu() -> Option<f64> {
    // ponytail: campos 14/15 de /proc/self/task/<tid>/stat, tick fixo em 100 Hz ; igual ao bench
    let s = std::fs::read_to_string("/proc/thread-self/stat").ok()?;
    let f: Vec<&str> = s.rsplit(')').next()?.split_whitespace().collect();
    let ut: f64 = f.get(11)?.parse().ok()?;
    let st: f64 = f.get(12)?.parse().ok()?;
    Some((ut + st) / 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dac::etherdream::{Emulator, EtherDream};

    #[test]
    fn transform_gira_escala_e_translada() {
        let src = [Point::new(1000.0, 0.0, 200, 100, 50, false)];
        let mut out = Vec::new();
        Transform {
            x: 5.0,
            y: 0.0,
            scale: 2.0,
            rot: 90.0,
            color: (1.0, 0.5, 0.0),
        }
        .apply(&src, &mut out);
        assert_eq!((out[0].x, out[0].y), (5, 2000));
        assert_eq!((out[0].r, out[0].g, out[0].b), (200, 50, 0));
        // identidade copia sem mexer
        Transform::default().apply(&src, &mut out);
        assert_eq!(out[0], src[0]);
    }

    #[test]
    fn ring_descarta_o_frame_velho() {
        let emu = Emulator::start(1800).unwrap();
        emu.record(false);
        let dac = EtherDream::connect(&format!("127.0.0.1:{}", emu.port), 1800).unwrap();
        let mut feed = Feed::start(Box::new(dac), 30_000, 2, Safety::default()).unwrap();
        let pts: Vec<Point> = (0..600)
            .map(|i| Point::new(i as f64 * 50.0 - 15000.0, 0.0, 255, 255, 255, false))
            .collect();
        for _ in 0..200 {
            feed.push(&pts);
        }
        std::thread::sleep(Duration::from_millis(300));
        feed.stop();
        let st = feed.stats();
        assert!(st.sent > 0, "nada foi enviado");
        assert!(
            st.dropped > 0,
            "o ring nao descartou nada com 200 frames de uma vez"
        );
        assert_eq!(st.errors, 0);
        // todo frame ou saiu ou foi descartado; no maximo os 2 do ring ficam para tras
        let contados = st.sent + st.dropped;
        assert!((198..=200).contains(&contados), "sent+dropped = {contados}");
    }

    #[test]
    fn safety_roda_antes_de_todo_envio() {
        let emu = Emulator::start(1800).unwrap();
        let dac = EtherDream::connect(&format!("127.0.0.1:{}", emu.port), 1800).unwrap();
        let mut feed = Feed::start(
            Box::new(dac),
            20_000,
            2,
            Safety {
                min_size: 2000,
                max_intensity: 255,
                zone: None,
            },
        )
        .unwrap();
        // figura de 100 unidades: ganho 100/2000 -> 255 vira 12
        let pts: Vec<Point> = (0..50)
            .map(|i| Point::new(i as f64 * 2.0, 0.0, 255, 255, 255, false))
            .collect();
        feed.push(&pts);
        std::thread::sleep(Duration::from_millis(200));
        feed.stop();
        let got = emu.points();
        assert_eq!(got.len(), 50);
        assert!(
            got.iter().all(|p| p.r == 12 * 257),
            "safety nao escureceu: {:?}",
            got[0]
        );
    }
}
