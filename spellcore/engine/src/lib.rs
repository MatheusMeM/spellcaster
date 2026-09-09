//! Engine do Spellcaster (R0): relogio, buffers DMX, timeline, arquivo de show e registry.
//! Nada aqui conhece rede nem GUI — `protocols` e `cli` sao clientes deste crate.

pub mod clock;
pub mod cues;
pub mod edit;
pub mod hook;
pub mod player;
pub mod registry;
pub mod show;
pub mod timeline;
pub mod universe;

/// Reexportado: quem registra comando no `Registry` deriva `JsonSchema`, e a versao do schemars
/// e' contrato do engine — `cli` e `mcp` nao pinam a sua.
pub use schemars;

pub use clock::{Clock, State, Stats};
pub use cues::{Cue, CueList};
pub use hook::{Ev, EventSink, FrameHook, NullSink};
pub use player::{Handle, Player, TransportState};
pub use registry::{Command, Registry};
pub use show::{OutputCfg, Show};
pub use timeline::{Curve, Keyframe, Keys, Timeline, Track, Value, BEZ};
pub use universe::{Universe, Universes};
