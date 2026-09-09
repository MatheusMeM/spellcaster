//! Relogio unico do engine: tick de fase fixa, transporte play/pause/stop/locate e jitter medido.
//! Transporte igual ao `spellcaster/core/clock.py`; a espera e' que muda — fase fixa, sleep+spin.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Stop,
    Play,
    Pause,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stats {
    pub p50: f64,
    pub p99: f64,
    pub max: f64,
    pub frames: u64,
    pub drift: i64,
}

struct Tr {
    state: State,
    pos: f64,  // posicao quando parado/pausado
    t0: Instant, // instante correspondente a t=0 quando tocando
}

struct Inner {
    fps: u32,
    tr: Mutex<Tr>,
    stats: Mutex<Stats>,
}

/// Clonavel: o transporte de outra thread age no mesmo relogio.
#[derive(Clone)]
pub struct Clock {
    i: Arc<Inner>,
}

impl Clock {
    pub fn new(fps: u32) -> Clock {
        Clock {
            i: Arc::new(Inner {
                fps: fps.max(1), // ponytail: fps 0 vira 1 em vez de erro ; validar no .spell quando houver schema de show
                tr: Mutex::new(Tr {
                    state: State::Stop,
                    pos: 0.0,
                    t0: Instant::now(),
                }),
                stats: Mutex::new(Stats::default()),
            }),
        }
    }

    pub fn fps(&self) -> u32 {
        self.i.fps
    }

    pub fn time(&self) -> f64 {
        let tr = self.i.tr.lock().unwrap();
        if tr.state == State::Play {
            tr.t0.elapsed().as_secs_f64()
        } else {
            tr.pos
        }
    }

    pub fn state(&self) -> State {
        self.i.tr.lock().unwrap().state
    }

    pub fn play(&self) {
        let mut tr = self.i.tr.lock().unwrap();
        if tr.state != State::Play {
            tr.t0 = Instant::now() - Duration::from_secs_f64(tr.pos.max(0.0));
            tr.state = State::Play;
        }
    }

    pub fn pause(&self) {
        let mut tr = self.i.tr.lock().unwrap();
        if tr.state == State::Play {
            tr.pos = tr.t0.elapsed().as_secs_f64();
        }
        tr.state = State::Pause;
    }

    pub fn stop(&self) {
        let mut tr = self.i.tr.lock().unwrap();
        tr.state = State::Stop;
        tr.pos = 0.0;
    }

    pub fn locate(&self, t: f64) {
        let mut tr = self.i.tr.lock().unwrap();
        tr.pos = t;
        tr.t0 = Instant::now() - Duration::from_secs_f64(t.max(0.0));
    }

    /// Chama f(t) a cada 1/fps s ate stop() ou t >= duration. Bloqueia a thread chamadora.
    /// Fase fixa (`next += period`): atraso nao reancora o relogio, so' pula frames e conta drift.
    pub fn run<F: FnMut(f64)>(&self, mut f: F, duration: Option<f64>) {
        if self.state() == State::Stop {
            self.play();
        }
        let pd = Duration::from_secs_f64(1.0 / self.i.fps as f64);
        // Quanto o `sleep` desta maquina passa do pedido, medido em obra e nao chutado: dormir
        // ate 1 ms fixo antes do alvo queima ~1 ms de spin por frame (6 % de um nucleo a 60 Hz)
        // quando o overshoot real aqui e' ~0,45 ms. A margem persegue a media do overshoot; o
        // spin cobre so' o que sobra.
        // ponytail: EWMA simples com fator 0,05 e teto de 1,3x, limitada a 0,3..2 ms — e' a
        // calibracao, o unico botao ; trocar por quantil real se um alvo (Pi, VM) tiver cauda
        // larga o bastante para o p99 de jitter passar de 1 ms.
        let mut over_avg = 8e-4_f64; // chute inicial conservador: 0,8 ms
        let mut margin = Duration::from_secs_f64(over_avg * 1.3);
        // ponytail: sem duration, guarda 10 min de amostras e para de amostrar ; trocar por
        // histograma de baldes se o run passar a durar horas com stats ligado.
        let cap = duration
            .map(|d| (d * self.i.fps as f64).ceil() as usize + 2)
            .unwrap_or(self.i.fps as usize * 600);
        let mut jit: Vec<f64> = Vec::with_capacity(cap);
        let mut frames = 0u64;
        let mut drift = 0i64;
        let _boost = rt::Boost::on();
        let mut next = Instant::now();
        while self.state() != State::Stop {
            // Tempo travado no quadro: n/fps exato, como o gerador do fixture (t = i/fps). Sem isso um fx
            // continuo (sin) amostrado no tempo medido (i/fps + overshoot) diverge em +-1 na truncagem.
            let t = (self.time() * self.i.fps as f64).round() / self.i.fps as f64;
            if let Some(d) = duration {
                if t >= d {
                    break;
                }
            }
            f(t);
            frames += 1;
            next += pd;
            loop {
                let now = Instant::now();
                if now >= next {
                    break;
                }
                let rem = next - now;
                if rem > margin {
                    let want = rem - margin;
                    std::thread::sleep(want);
                    let over = now.elapsed().saturating_sub(want).as_secs_f64();
                    over_avg += (over - over_avg) * 0.05;
                    margin = Duration::from_secs_f64((over_avg * 1.3).clamp(3e-4, 2e-3));
                } else {
                    std::hint::spin_loop();
                }
            }
            let now = Instant::now();
            if jit.len() < cap {
                jit.push((now - next).as_secs_f64());
            }
            while next + pd <= now {
                next += pd;
                drift += 1;
            }
        }
        drop(_boost);
        jit.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let pick = |q: f64| -> f64 {
            if jit.is_empty() {
                0.0
            } else {
                jit[(((jit.len() as f64) * q) as usize).min(jit.len() - 1)]
            }
        };
        *self.i.stats.lock().unwrap() = Stats {
            p50: pick(0.50),
            p99: pick(0.99),
            max: jit.last().copied().unwrap_or(0.0),
            frames,
            drift,
        };
    }

    /// Jitter do ultimo run: (p50, p99, max) em segundos, frames emitidos e frames perdidos.
    pub fn stats(&self) -> Stats {
        *self.i.stats.lock().unwrap()
    }
}

#[cfg(windows)]
mod rt {
    // Tres simbolos declarados na mao: `windows-sys` inteiro nao paga a pena (README).
    #[link(name = "winmm")]
    unsafe extern "system" {
        fn timeBeginPeriod(u: u32) -> u32;
        fn timeEndPeriod(u: u32) -> u32;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentThread() -> isize;
        fn SetThreadPriority(h: isize, p: i32) -> i32;
        fn GetThreadPriority(h: isize) -> i32;
    }
    const TIME_CRITICAL: i32 = 15;
    const ERROR_RETURN: i32 = 0x7fff_ffff;

    /// timeBeginPeriod(1) + prioridade da thread; restaura tudo no Drop.
    pub struct Boost(i32);

    impl Boost {
        pub fn on() -> Boost {
            unsafe {
                timeBeginPeriod(1);
                let h = GetCurrentThread();
                let old = GetThreadPriority(h);
                SetThreadPriority(h, TIME_CRITICAL);
                Boost(old)
            }
        }
    }

    impl Drop for Boost {
        fn drop(&mut self) {
            unsafe {
                if self.0 != ERROR_RETURN {
                    SetThreadPriority(GetCurrentThread(), self.0);
                }
                timeEndPeriod(1);
            }
        }
    }
}

#[cfg(not(windows))]
mod rt {
    /// ponytail: no-op fora do Windows ; o Pi ganha SCHED_FIFO/nice na R1.
    pub struct Boost;
    impl Boost {
        pub fn on() -> Boost {
            Boost
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transporte() {
        let c = Clock::new(30);
        assert_eq!(c.state(), State::Stop);
        assert_eq!(c.time(), 0.0);
        c.locate(5.0);
        assert_eq!(c.time(), 5.0);
        assert_eq!(c.state(), State::Stop);
        c.play();
        assert!(c.time() >= 5.0);
        c.pause();
        assert_eq!(c.state(), State::Pause);
        let a = c.time();
        std::thread::sleep(Duration::from_millis(5));
        assert_eq!(c.time(), a, "pausado nao anda");
        c.stop();
        assert_eq!(c.time(), 0.0);
    }

    #[test]
    fn run_conta_frames_e_para_na_duracao() {
        let c = Clock::new(100);
        let mut n = 0u32;
        c.run(|_| n += 1, Some(0.2));
        // 20 frames em 0,2 s; a folga para baixo cobre a maquina engasgada (frames pulados).
        assert!((15..=21).contains(&n), "frames={}", n);
        let s = c.stats();
        assert_eq!(s.frames as u32, n);
        assert!(s.max >= 0.0 && s.p50 <= s.p99 && s.p99 <= s.max);
    }

    #[test]
    fn stop_de_outra_thread_encerra_o_run() {
        let c = Clock::new(50);
        let c2 = c.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(60));
            c2.stop();
        });
        c.run(|_| {}, None);
        assert_eq!(c.state(), State::Stop);
    }
}
