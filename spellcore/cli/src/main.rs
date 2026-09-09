//! O binario `spellcore`. A CLI inteira mora em `lib.rs` porque a janela
//! (`spellcore/gui`, binario `spellcaster`) precisa do MESMO registry: `registry()` acrescenta
//! `play_show`, `net`, `graph_check` e os `laser_*` ao `engine::registry::base()`, e um segundo
//! registry montado a mao ficaria para tras no primeiro comando novo.

fn main() {
    cli::main();
}
