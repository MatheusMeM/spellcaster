//! Spellcaster output/input protocols: sACN (E1.31), Art-Net 4, OSC 1.0, MIDI input
//! and network scan.
//!
//! Self-contained crate: does not depend on `engine`. The behavior reference is the Python
//! package `spellcaster/protocols/` - the bytes on the wire must be identical (fixtures in
//! tests/conformance).

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};

pub mod artnet;
pub mod midi;
pub mod netscan;
pub mod osc;
pub mod sacn;

/// Every protocol output of the engine.
///
/// `close(&mut self)` and not the PRD's `close(self)`: `Box<dyn Output>` requires object
/// safety, and a method that consumes `self` cannot be called through a trait object.
// ponytail: close takes &mut, a second call is a no-op ; would only change if the engine
// came to own the outputs by value, which would break Box<dyn Output>.
pub trait Output: Send {
    fn send(&mut self, universe: u16, data: &[u8; 512]);
    fn close(&mut self);
}

// -------------------------------------------------------------------- queue
// Frame queue between the engine thread and the I/O thread of an output.
// At most 2 frames PER UNIVERSE; when full, the OLD frame of that universe is dropped
// (the new one always gets in). `push` never blocks waiting - only on the lock, which the
// I/O thread releases before touching the socket.
// ponytail: queue with Mutex+Condvar, not lock-free ; switch to an SPSC ring if the
// throughput bench shows contention (16+16 universes at 60 Hz = 1920 push/s, trivial for a
// mutex).

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
                // fixed capacity: DEPTH per universe (+1 slack for an undeclared universe).
                // Never reallocates on the hot path because push drops before inserting.
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
                st.frames.remove(i); // drop the oldest of this universe
            }
        }
        st.frames.push_back((universe, *data));
        drop(st);
        self.cv.notify_one();
    }

    /// Blocks until there is a frame or the queue is closed. `None` = time to exit.
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
/// 16 pseudo-random bytes for the sACN CID (equivalent to Python's uuid4()).
// ponytail: hash of clock+pid+counter, not a CSPRNG ; the CID only needs to be unique on
// the network, not unpredictable. Switch to getrandom if it ever becomes a security identity.
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
    let stack = &nanos as *const u64 as usize as u64; // ASLR: entropy across processes
    let seed =
        nanos ^ ((std::process::id() as u64) << 32) ^ stack ^ N.fetch_add(1, Ordering::Relaxed);
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
    fn queue_drops_old_frame() {
        let q = Queue::new(1);
        for i in 0..100u16 {
            let mut d = [0u8; 512];
            d[0] = i as u8;
            q.push(1, &d);
        }
        assert_eq!(q.len(), DEPTH, "queue must stay at 2 frames");
        // what is left are the TWO newest, in order
        assert_eq!(q.pop().unwrap().1[0], 98);
        assert_eq!(q.pop().unwrap().1[0], 99);
        q.stop();
        assert!(q.pop().is_none());
    }

    #[test]
    fn queue_per_universe() {
        let q = Queue::new(3);
        for u in 1..=3u16 {
            for _ in 0..10 {
                q.push(u, &[0u8; 512]);
            }
        }
        assert_eq!(q.len(), DEPTH * 3);
    }

    #[test]
    fn cid_does_not_repeat() {
        assert_ne!(random16(), random16());
        assert_ne!(random16(), [0u8; 16]);
    }
}
