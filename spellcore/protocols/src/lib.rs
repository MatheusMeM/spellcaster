//! Protocolos de saida/entrada do Spellcaster: sACN (E1.31), Art-Net 4, OSC 1.0 e varredura de rede.
//!
//! Crate autocontido: nao depende de `engine`. A referencia de comportamento e o pacote Python
//! `spellcaster/protocols/` — os bytes na rede tem que ser identicos (fixtures em tests/conformance).

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub mod artnet;
pub mod netscan;
pub mod osc;
pub mod sacn;

/// Toda saida de protocolo do engine.
///
/// `close(&mut self)` e nao `close(self)` do PRD: `Box<dyn Output>` exige object safety,
/// e um metodo que consome `self` nao pode ser chamado atraves de um trait object.
// ponytail: close por &mut, dupla chamada e no-op ; so mudaria se o engine passasse a possuir
// as saidas por valor, o que quebraria Box<dyn Output>.
pub trait Output: Send {
    fn send(&mut self, universe: u16, data: &[u8; 512]);
    fn close(&mut self);
}

// --------------------------------------------------------------------- fila
// Fila de frames entre a thread do engine e a thread de I/O de uma saida.
// Maximo de 2 frames POR UNIVERSO; ao encher, o frame VELHO daquele universo e descartado
// (o novo sempre entra). `push` nunca bloqueia por espera — so pelo lock, que a thread de I/O
// solta antes de tocar no socket.
// ponytail: fila com Mutex+Condvar, nao lock-free ; trocar por ring SPSC se o bench de
// throughput acusar contencao (16+16 universos a 60 Hz = 1920 push/s, irrisorio para um mutex).

pub(crate) const DEPTH: usize = 2;

pub(crate) struct Queue {
    state: Mutex<QState>,
    cv: Condvar,
}

pub(crate) struct QState {
    frames: VecDeque<(u16, [u8; 512])>,
    running: bool,
}

impl Queue {
    pub(crate) fn new(universes: usize) -> Arc<Queue> {
        Arc::new(Queue {
            state: Mutex::new(QState {
                // capacidade fixa: DEPTH por universo (+1 folga para universo nao declarado).
                // Nunca realoca no caminho quente porque o push descarta antes de inserir.
                frames: VecDeque::with_capacity(DEPTH * universes.max(1) + 1),
                running: true,
            }),
            cv: Condvar::new(),
        })
    }

    pub(crate) fn push(&self, universe: u16, data: &[u8; 512]) {
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if !st.running {
            return;
        }
        if st.frames.iter().filter(|(u, _)| *u == universe).count() >= DEPTH {
            if let Some(i) = st.frames.iter().position(|(u, _)| *u == universe) {
                st.frames.remove(i); // descarta o mais velho desse universo
            }
        }
        st.frames.push_back((universe, *data));
        drop(st);
        self.cv.notify_one();
    }

    /// Bloqueia ate ter frame ou ate a fila ser fechada. `None` = hora de sair.
    pub(crate) fn pop(&self) -> Option<(u16, [u8; 512])> {
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            if let Some(f) = st.frames.pop_front() {
                return Some(f);
            }
            if !st.running {
                return None;
            }
            st = self.cv.wait(st).unwrap_or_else(|e| e.into_inner());
        }
    }

    pub(crate) fn stop(&self) {
        let mut st = self.state.lock().unwrap_or_else(|e| e.into_inner());
        st.running = false;
        drop(st);
        self.cv.notify_all();
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .frames
            .len()
    }
}

// --------------------------------------------------------------------- util
/// 16 bytes pseudo-aleatorios para o CID do sACN (equivalente ao uuid4() do Python).
// ponytail: hash de relogio+pid+contador, nao CSPRNG ; o CID so precisa ser unico na rede,
// nao imprevisivel. Trocar por getrandom se algum dia virar identidade de seguranca.
pub(crate) fn random16() -> [u8; 16] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static N: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let stack = &nanos as *const u64 as usize as u64; // ASLR: entropia entre processos
    let seed = nanos ^ ((std::process::id() as u64) << 32) ^ stack ^ N.fetch_add(1, Ordering::Relaxed);
    let mut out = [0u8; 16];
    for (i, half) in out.chunks_mut(8).enumerate() {
        let mut h = DefaultHasher::new();
        (seed, i as u64).hash(&mut h);
        half.copy_from_slice(&h.finish().to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_descarta_frame_velho() {
        let q = Queue::new(1);
        for i in 0..100u16 {
            let mut d = [0u8; 512];
            d[0] = i as u8;
            q.push(1, &d);
        }
        assert_eq!(q.len(), DEPTH, "fila deve ficar em 2 frames");
        // o que sobrou sao os DOIS mais novos, na ordem
        assert_eq!(q.pop().unwrap().1[0], 98);
        assert_eq!(q.pop().unwrap().1[0], 99);
        q.stop();
        assert!(q.pop().is_none());
    }

    #[test]
    fn queue_por_universo() {
        let q = Queue::new(3);
        for u in 1..=3u16 {
            for _ in 0..10 {
                q.push(u, &[0u8; 512]);
            }
        }
        assert_eq!(q.len(), DEPTH * 3);
    }

    #[test]
    fn cid_nao_repete() {
        assert_ne!(random16(), random16());
        assert_ne!(random16(), [0u8; 16]);
    }
}
