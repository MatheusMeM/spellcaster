//! Live modules through the registry: loading `modules/laser.json`, add/list/get/del, a rejected
//! manifest and the resolution of `modules/`. Its own binary because `MODULES` and `OPEN` are one
//! per process and the lib tests run in parallel in the same binary.

use engine::module::{self, Module};
use engine::registry::base;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const RAIZ: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const SPELL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../shows/medgrupo.spell");

fn laser() -> PathBuf {
    Path::new(RAIZ).join("modules").join("laser.json")
}

#[test]
fn laser_json_passes_the_check() {
    let m: Module = module::load(&laser()).expect("modules/laser.json");
    assert_eq!(m.name, "laser");
    assert!(module::check(&m).is_empty(), "{:?}", module::check(&m));
    assert!(m.parameters.contains_key("geo/scale"));
    assert!(m.values.contains_key("stat/fps"));
    assert!(m.commands.contains_key("play"));
    // stable order on write: BTreeMap, so the JSON comes back sorted by address
    let v = serde_json::to_value(&m).unwrap();
    let ks: Vec<&str> = v["parameters"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut ord = ks.clone();
    ord.sort_unstable();
    assert_eq!(ks, ord);
}

/// A single test for everything that touches `MODULES` and `OPEN`: the tests of one binary run
/// in parallel, and both tables are one per process.
#[test]
fn registry_of_live_modules() {
    let r = base();
    let c = |n: &str, a: Value| r.call(n, a);

    let f = laser().display().to_string();
    let a = c("module_add", json!({ "file": f })).unwrap();
    assert_eq!(a, json!({"name": "laser", "version": "0.1.0"}));

    let l = c("module_list", json!({})).unwrap();
    assert_eq!(l.as_array().unwrap().len(), 1);
    assert_eq!(l[0]["name"], json!("laser"));
    assert_eq!(l[0]["type"], json!("laser"));

    let g = c("module_get", json!({"name": "laser"})).unwrap();
    assert_eq!(g["parameters"]["geo/scale"]["max"], json!(4.0));
    assert_eq!(g["commands"]["shutter"]["context"], json!("both"));
    assert!(c("module_get", json!({"name": "does_not_exist"})).is_err());

    // the same name replaces, it does not duplicate
    c(
        "module_add",
        json!({"data": {"name": "laser", "version": "9.9.9",
                        "parameters": {"geo/scale": {"type": "float"}}}}),
    )
    .unwrap();
    let l = c("module_list", json!({})).unwrap();
    assert_eq!(l.as_array().unwrap().len(), 1, "{}", l);
    assert_eq!(l[0]["version"], json!("9.9.9"));
    assert!(c("module_get", json!({"name": "laser"})).unwrap()["values"]
        .as_object()
        .unwrap()
        .is_empty());

    let d = c("module_del", json!({"name": "laser"})).unwrap();
    assert_eq!(d["version"], json!("9.9.9"));
    assert_eq!(c("module_list", json!({})).unwrap(), json!([]));
    assert!(c("module_del", json!({"name": "laser"})).is_err());
    assert!(c("module_add", json!({})).is_err(), "no file and no data");

    // ---- a wrong manifest: the errors come all together and name the path
    let ruim: Module = serde_json::from_value(json!({"name": "bad", "parameters": {
        "geo/scale": {"type": "float", "min": 2, "max": 1},
        "with space/x": {"type": "int"},
        "mode": {"type": "enum"},
        "out/of_range": {"type": "float", "min": 0, "max": 1, "default": 5},
        "out/of_options": {"type": "enum", "options": ["x", "y"], "default": "z"},
        "wrong/type": {"type": "quaternion"},
        "geo//scale": {"type": "float"},
        "/scale": {"type": "float"}
    }}))
    .expect("bad manifest");

    let e = module::check(&ruim);
    assert_eq!(e.len(), 8, "{:?}", e);
    let tem = |t: &str| e.iter().any(|s| s.contains(t));
    assert!(tem("parameters/geo/scale") && tem("min 2"), "{:?}", e);
    assert!(tem("with space/x"), "{:?}", e);
    // an empty segment in the middle and at the start: `endereco_ok` refuses both
    assert!(tem("parameters/geo//scale"), "{:?}", e);
    assert!(tem("parameters//scale"), "{:?}", e);
    assert!(tem("parameters/mode"), "{:?}", e);
    assert!(
        tem("parameters/out/of_range") && tem("default 5"),
        "{:?}",
        e
    );
    assert!(
        tem("parameters/out/of_options") && tem("is not in options"),
        "{:?}",
        e
    );
    assert!(
        tem("parameters/wrong/type") && tem("\"quaternion\""),
        "{:?}",
        e
    );

    let err = r
        .call("module_add", json!({ "data": ruim }))
        .expect_err("a rejected one does not get in");
    assert!(err.contains("parameters/geo/scale"), "{}", err);
    assert_eq!(
        r.call("module_list", json!({})).unwrap(),
        json!([]),
        "a rejected one does not get in"
    );

    // ---- modules_dir: `shows/` and `modules/` are siblings in the repo: the one-level-up rule finds the folder
    assert_eq!(
        module::modules_dir(SPELL).canonicalize().ok(),
        laser().parent().unwrap().canonicalize().ok()
    );

    // a folder next to the .spell beats the one above
    let d = std::env::temp_dir().join("spellcaster_test_modules");
    std::fs::create_dir_all(d.join("modules")).expect("temp");
    let spell = d.join("x.spell");
    assert_eq!(
        module::modules_dir(&spell.display().to_string()),
        d.join("modules")
    );

    // with a show open, `file` without an extension resolves through the show folder
    r.call("show_get", json!({ "file": SPELL })).unwrap();
    assert_eq!(
        r.call("module_add", json!({"file": "laser"})).unwrap()["name"],
        json!("laser")
    );
    r.call("module_del", json!({"name": "laser"})).unwrap();
}
