//! `laser` — R4 do Spellcaster: frames ILDA, otimizacao de scan, safety obrigatoria,
//! arquivos `.ild` e saida multi-feed para Ether Dream, Helios e IDN.
//!
//! Crate autocontido: depende de `protocols` so pelo beacon Ether Dream ja implementado
//! em `protocols::netscan`. Nao conhece engine, GUI nem timeline.
//!
//! A referencia de comportamento e o pacote Python `spellcaster/protocols/ilda/`: os
//! numeros de `optimize`/`safety` e os bytes do `.ild` tem que ser identicos (fixtures em
//! `tests/fixtures/`, geradas por `tests/gen_fixtures.py`).
//!
//! ```no_run
//! use laser::{Dac, EtherDream, Feed, Safety, ild};
//! let dac = EtherDream::connect("192.168.0.50", 1800).unwrap();
//! let feed = Feed::start(Box::new(dac), 30_000, 2, Safety::default()).unwrap();
//! for f in ild::read(std::path::Path::new("shows/logo.ild")).unwrap() {
//!     feed.push(&f.points);
//! }
//! ```

pub mod dac;
pub mod feed;
pub mod frame;
pub mod ild;
pub mod trace;

pub use dac::etherdream::{Emulator, EtherDream};
pub use dac::idn::Idn;
pub use dac::Dac;
pub use feed::{Feed, FeedStats, Transform};
pub use frame::{bbox, optimize, optimize_into, Frame, Point, Safety};
