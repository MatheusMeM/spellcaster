//! Laser outputs. One DAC = one physical destination (Ether Dream, Helios, IDN projector).

pub mod etherdream;
pub mod helios;
pub mod idn;

use std::io;

use crate::frame::Point;

/// Contract of every DAC. Mirrors the `trait Output` of `protocols` (send + close), but in
/// points and not in DMX universes.
///
/// Flow control (DAC buffer, chunking) is the DAC's own responsibility: the `Feed` only
/// delivers points that already went through safety.
// ponytail: no `open()` in the trait ; each DAC opens in the constructor and returns
// io::Result, so a DAC behind the trait object is already connected. `reopen()` arrives when
// there is hot reconnection.
pub trait Dac: Send {
    /// Short name for the log (`etherdream:192.168.0.50`).
    fn name(&self) -> String;
    /// Prepares and starts playing at `pps` points per second.
    fn begin(&mut self, pps: u32) -> io::Result<()>;
    /// Sends points. May block while the DAC buffer is full.
    fn send(&mut self, points: &[Point]) -> io::Result<()>;
    /// Stops the output and blanks. Idempotent.
    fn stop(&mut self);
}
