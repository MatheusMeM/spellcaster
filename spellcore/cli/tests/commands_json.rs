//! `spellgui/web/dev/commands.json` is the frozen `Registry::schema()`: it is what `widgets.js`
//! builds the form from and what `bus.js` validates a command against in offline mode (a page
//! opened with no engine). This test is what keeps the file from going stale.
//!
//! To regenerate: `spellcore commands > spellgui/web/dev/commands.json`.

use serde_json::Value;

mod common;

const DEV: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../spellgui/web/dev/commands.json"
);

#[test]
fn dev_commands_json_is_up_to_date() {
    let out = common::bin()
        .arg("commands")
        .output()
        .expect("spellcore commands");
    let vivo: Value = serde_json::from_slice(&out.stdout).expect("the `commands` stdout is JSON");
    let txt = std::fs::read_to_string(DEV).expect(DEV);
    let disco: Value = serde_json::from_str(&txt).expect("dev/commands.json is JSON");
    // A subset on purpose: a new command in the registry does not break the page; a command that
    // went away or that changed schema does — the GUI form comes from here.
    let vivos = vivo.as_array().expect("list of commands");
    for c in disco.as_array().expect("list of commands") {
        let n = &c["name"];
        let v = vivos.iter().find(|x| x["name"] == *n).unwrap_or_else(|| {
            panic!(
                "{} left the registry; regenerate with `spellcore commands`",
                n
            )
        });
        assert_eq!(
            v, c,
            "the schema of {} changed; regenerate with `spellcore commands`",
            n
        );
    }
}
