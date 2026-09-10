//! The `spellcore` binary. The whole CLI lives in `lib.rs` because the window
//! (`spellcore/gui`, binary `spellcaster`) needs the SAME registry: `registry()` adds
//! `play_show`, `net`, `graph_check` and the `laser_*` to `engine::registry::base()`, and a
//! second registry built by hand would fall behind on the first new command.

fn main() {
    cli::main();
}
