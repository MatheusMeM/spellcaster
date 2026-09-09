//! Ganchos de frame e saida de evento. CONTRATO da R1 posto pelo orquestrador: e' por aqui que
//! o crate `script` (Rhai fx e Graph) se pluga sem o engine conhecer Rhai, e por aqui que o
//! Graph fala com o registry / a rede / a GUI sem o engine conhecer nenhum dos tres.
//! Nao mudar sem mudar `spellcore/README.md` (secao "R1 — contratos fixados").

use crate::universe::Universes;

/// Escreve nos Universes uma vez por frame, depois de `Timeline::apply`.
pub trait FrameHook: Send {
    fn frame(&mut self, t: f64, uni: &mut Universes);

    /// Evento de entrada para o Graph. Chave: "widget:go", "key:Space", "osc:/spell/go",
    /// "midi:144/60", "marker:pico". Enfileirado; consumido no proximo `frame()`.
    fn input(&mut self, _key: &str, _value: f64) {}

    /// locate/stop: zera o estado persistente.
    fn reset(&mut self, _t: f64) {}
}

/// Evento que sai do Graph para o mundo.
// ponytail: Ev com String ; virar indice no catalogo se algum graph passar a emitir por frame
// (hoje evento e' raro: um GO, um OSC de entrada).
#[derive(Clone, Debug, PartialEq)]
pub enum Ev {
    /// No `cmd`: chama uma entrada do registry.
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
    /// `out.param` ("fixture.canal")
    Param { target: String, value: f64 },
    /// `out.notify`
    Notify { text: String },
}

/// Quem recebe os eventos do Graph. Implementado pela CLI (registry, OscOut, stderr): o engine
/// so' declara o trait porque nao conhece o registry vivo nem a GUI.
pub trait EventSink: Send {
    fn emit(&mut self, e: &Ev);
}

/// Sink que descarta tudo. Util em teste e em show sem graph.
pub struct NullSink;

impl EventSink for NullSink {
    fn emit(&mut self, _e: &Ev) {}
}
