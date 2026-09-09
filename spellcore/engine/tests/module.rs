//! Modulos vivos pelo registry: carregar `modules/laser.json`, add/list/get/del, manifesto
//! reprovado e a resolucao de `modules/`. Binario proprio porque `MODULES` e `OPEN` sao um por
//! processo e os testes da lib rodam em paralelo no mesmo binario.

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
fn laser_json_passa_no_check() {
    let m: Module = module::load(&laser()).expect("modules/laser.json");
    assert_eq!(m.name, "laser");
    assert!(module::check(&m).is_empty(), "{:?}", module::check(&m));
    assert!(m.parameters.contains_key("geo/scale"));
    assert!(m.values.contains_key("stat/fps"));
    assert!(m.commands.contains_key("play"));
    // ordem estavel ao gravar: BTreeMap, entao o JSON volta ordenado por endereco
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

/// Um teste so' para tudo que mexe em `MODULES` e `OPEN`: os testes de um binario rodam em
/// paralelo, e as duas tabelas sao uma por processo.
#[test]
fn registro_de_modulos_vivos() {
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
    assert!(c("module_get", json!({"name": "nao_existe"})).is_err());

    // mesmo nome substitui, nao duplica
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
    assert!(c("module_add", json!({})).is_err(), "sem file nem data");

    // ---- manifesto errado: os erros vem todos juntos e citam o path
    let ruim: Module = serde_json::from_value(json!({"name": "ruim", "parameters": {
        "geo/scale": {"type": "float", "min": 2, "max": 1},
        "sem espaco/x": {"type": "int"},
        "modo": {"type": "enum"},
        "fora/faixa": {"type": "float", "min": 0, "max": 1, "default": 5},
        "fora/options": {"type": "enum", "options": ["x", "y"], "default": "z"},
        "tipo/errado": {"type": "quaternion"}
    }}))
    .expect("manifesto ruim");

    let e = module::check(&ruim);
    assert_eq!(e.len(), 6, "{:?}", e);
    let tem = |t: &str| e.iter().any(|s| s.contains(t));
    assert!(tem("parameters/geo/scale") && tem("min 2"), "{:?}", e);
    assert!(tem("sem espaco/x"), "{:?}", e);
    assert!(tem("parameters/modo"), "{:?}", e);
    assert!(tem("parameters/fora/faixa") && tem("default 5"), "{:?}", e);
    assert!(
        tem("parameters/fora/options") && tem("nao esta em options"),
        "{:?}",
        e
    );
    assert!(
        tem("parameters/tipo/errado") && tem("\"quaternion\""),
        "{:?}",
        e
    );

    let err = r
        .call("module_add", json!({ "data": ruim }))
        .expect_err("reprovado nao entra");
    assert!(err.contains("parameters/geo/scale"), "{}", err);
    assert_eq!(
        r.call("module_list", json!({})).unwrap(),
        json!([]),
        "reprovado nao entra"
    );

    // ---- modules_dir: `shows/` e `modules/` sao irmaos no repo: a regra do nivel acima acha a pasta
    assert_eq!(
        module::modules_dir(SPELL).canonicalize().ok(),
        laser().parent().unwrap().canonicalize().ok()
    );

    // pasta ao lado do .spell ganha da de cima
    let d = std::env::temp_dir().join("spellcaster_test_modules");
    std::fs::create_dir_all(d.join("modules")).expect("temp");
    let spell = d.join("x.spell");
    assert_eq!(
        module::modules_dir(&spell.display().to_string()),
        d.join("modules")
    );

    // com show aberto, `file` sem extensao resolve pela pasta do show
    r.call("show_get", json!({ "file": SPELL })).unwrap();
    assert_eq!(
        r.call("module_add", json!({"file": "laser"})).unwrap()["name"],
        json!("laser")
    );
    r.call("module_del", json!({"name": "laser"})).unwrap();
}
