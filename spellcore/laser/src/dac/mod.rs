//! Saidas de laser. Um DAC = um destino fisico (Ether Dream, Helios, projetor IDN).

pub mod etherdream;
pub mod helios;
pub mod idn;

use std::io;

use crate::frame::Point;

/// Contrato de todo DAC. Espelha o `trait Output` de `protocols` (send + close), mas em
/// pontos e nao em universos DMX.
///
/// O controle de fluxo (buffer do DAC, chunking) e responsabilidade do proprio DAC: o
/// `Feed` so entrega pontos ja passados pela safety.
// ponytail: sem `open()` no trait ; cada DAC abre no construtor e devolve io::Result, entao
// um DAC no trait object ja esta conectado. Entra `reopen()` quando houver reconexao a quente.
pub trait Dac: Send {
    /// Nome curto para log (`etherdream:192.168.0.50`).
    fn name(&self) -> String;
    /// Prepara e comeca a tocar a `pps` pontos por segundo.
    fn begin(&mut self, pps: u32) -> io::Result<()>;
    /// Envia pontos. Pode bloquear enquanto o buffer do DAC estiver cheio.
    fn send(&mut self, points: &[Point]) -> io::Result<()>;
    /// Para a saida e apaga. Idempotente.
    fn stop(&mut self);
}
