//! Engine do Spellcaster (R0): relogio, buffers DMX, timeline, arquivo de show e registry.
//! Nada aqui conhece rede nem GUI — `protocols` e `cli` sao clientes deste crate.

pub mod clock;
pub mod registry;
pub mod show;
pub mod timeline;
pub mod universe;

pub use clock::{Clock, State, Stats};
pub use registry::{Command, Registry};
pub use show::{OutputCfg, Show};
pub use timeline::{Curve, Keyframe, Keys, Timeline, Track, Value, BEZ};
pub use universe::{Universe, Universes};
