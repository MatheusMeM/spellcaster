//! Spellcaster engine (R0): clock, DMX buffers, timeline, show file and registry.
//! Nothing here knows the network or the GUI — `protocols` and `cli` are clients of this crate.

pub mod clock;
pub mod cues;
pub mod edit;
pub mod hook;
pub mod input;
pub mod midi;
pub mod module;
pub mod player;
pub mod rec;
pub mod registry;
pub mod show;
pub mod timeline;
pub mod universe;

/// Re-exported: whoever registers a command in the `Registry` derives `JsonSchema`, and the
/// schemars version is an engine contract — `cli` and `mcp` do not pin their own.
pub use schemars;

pub use clock::{Clock, State, Stats};
pub use cues::{Cue, CueList};
pub use hook::{Ev, EventSink, FrameHook, NullSink};
pub use input::Inputs;
pub use module::Module;
pub use player::{Handle, Player, TransportState};
pub use registry::{Command, Registry};
pub use show::{OutputCfg, Show};
pub use timeline::{Curve, Keyframe, Keys, Timeline, Track, Value, BEZ};
pub use universe::{Universe, Universes};
