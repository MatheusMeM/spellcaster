//! Frame hooks and event output. R1 CONTRACT set by the orchestrator: this is where the
//! `script` crate (Rhai fx and Graph) plugs in without the engine knowing Rhai, and where the
//! Graph talks to the registry / the network / the GUI without the engine knowing any of the
//! three. Do not change it without changing `spellcore/README.md` (section "R1 — fixed contracts").

use crate::universe::Universes;

/// Writes into the Universes once per frame, after `Timeline::apply`.
pub trait FrameHook: Send {
    fn frame(&mut self, t: f64, uni: &mut Universes);

    /// Input event for the Graph. Key: "widget:go", "key:Space", "osc:/spell/go",
    /// "midi:144/60", "marker:pico". Queued; consumed on the next `frame()`.
    fn input(&mut self, _key: &str, _value: f64) {}

    /// locate/stop: clears the persistent state.
    fn reset(&mut self, _t: f64) {}
}

/// Event that leaves the Graph for the world.
// ponytail: Ev with String ; make it a catalog index if some graph starts emitting per frame
// (today an event is rare: a GO, an incoming OSC).
#[derive(Clone, Debug, PartialEq)]
pub enum Ev {
    /// `cmd` node: calls a registry entry.
    Cmd {
        name: String,
        args: serde_json::Value,
    },
    /// `out.osc`
    Osc { address: String, args: Vec<f64> },
    /// `out.widget`
    Widget {
        id: String,
        prop: String,
        value: f64,
    },
    /// `out.param` ("fixture.channel")
    Param { target: String, value: f64 },
    /// `out.notify`
    Notify { text: String },
}

/// Whoever receives the Graph events. Implemented by the CLI (registry, OscOut, stderr): the
/// engine only declares the trait because it knows neither the live registry nor the GUI.
pub trait EventSink: Send {
    fn emit(&mut self, e: &Ev);
}

/// Sink that drops everything. Useful in tests and in a show with no graph.
pub struct NullSink;

impl EventSink for NullSink {
    fn emit(&mut self, _e: &Ev) {}
}
