//! `spellgui/web/dev/commands.json` e' o `Registry::schema()` congelado: e' dele que o
//! `widgets.js` monta formulario e que o `bus.js` valida comando no modo offline (pagina aberta
//! sem engine). Este teste e' o que impede o arquivo de envelhecer.
//!
//! Regerar: `spellcore commands > spellgui/web/dev/commands.json`.

use serde_json::Value;
use std::process::Command;

const DEV: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../spellgui/web/dev/commands.json"
);

#[test]
fn dev_commands_json_em_dia() {
    let out = Command::new(env!("CARGO_BIN_EXE_spellcore"))
        .arg("commands")
        .output()
        .expect("spellcore commands");
    let vivo: Value = serde_json::from_slice(&out.stdout).expect("stdout de `commands` e' JSON");
    let txt = std::fs::read_to_string(DEV).expect(DEV);
    let disco: Value = serde_json::from_str(&txt).expect("dev/commands.json e' JSON");
    // Subconjunto de proposito: comando novo no registry nao quebra a pagina; comando que sumiu
    // ou que mudou de schema, sim — o formulario da GUI sai daqui.
    let vivos = vivo.as_array().expect("lista de comandos");
    for c in disco.as_array().expect("lista de comandos") {
        let n = &c["name"];
        let v = vivos.iter().find(|x| x["name"] == *n).unwrap_or_else(|| {
            panic!("{} saiu do registry; regere com `spellcore commands`", n)
        });
        assert_eq!(
            v, c,
            "schema de {} mudou; regere com `spellcore commands`",
            n
        );
    }
}
