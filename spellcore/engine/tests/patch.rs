//! `show_patch` (JSON Patch), `graph_get`, `face_get` e o contador `rev`. Binario
//! proprio porque `OPEN` e `REV` sao um por processo; e um `#[test]` so' porque os testes de um
//! mesmo binario rodam em paralelo e todos aqui mexem nesse estado.

use engine::edit::rev;
use engine::registry::base;
use engine::Registry;
use serde_json::{json, Value};

const SPELL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../shows/medgrupo.spell");
const OUTRO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../shows/medgrupo_r0.spell");

/// `show_patch` com uma lista de ops, sem checagem de revisao.
fn ops(r: &Registry, o: Value) -> Result<Value, String> {
    r.call("show_patch", json!({ "ops": o }))
}

fn full(r: &Registry) -> Value {
    r.call("show_get", json!({"full": true})).unwrap()
}

#[test]
fn patch_graph_e_face() {
    let r = base();
    tudo_ou_nada(&r);
    show_continua_show(&r);
    rev_velha_e_recusada(&r);
    trocar_de_show_sobe_rev(&r);
    graph_e_face(&r);
}

fn tudo_ou_nada(r: &Registry) {
    r.call("show_get", json!({ "file": SPELL })).unwrap();
    let antes = full(r);
    let rev0 = rev();

    // add em /tracks/-, replace em /fps, add de uma chave de `extra`
    let out = ops(
        r,
        json!([
            {"op": "add", "path": "/tracks/-", "value": {"type": "dmx", "universe": 9,
                                                         "address": 1, "keys": []}},
            {"op": "replace", "path": "/fps", "value": 25},
            {"op": "add", "path": "/markers", "value": [{"t": 1.0, "name": "um"}]},
        ]),
    )
    .unwrap();
    assert_eq!(out["rev"], json!(rev0 + 1));
    assert_eq!(rev(), rev0 + 1);
    let d = full(r);
    assert_eq!(d["fps"], json!(25));
    assert_eq!(d["markers"][0]["name"], json!("um"));
    let n = d["tracks"].as_array().unwrap().len();
    assert_eq!(n, antes["tracks"].as_array().unwrap().len() + 1);

    // o undo ja' vem na ordem de aplicacao: mandar de volta como veio restaura byte a byte.
    // "/tracks/-" virou "/tracks/<indice>" no inverso: "-" nao serve para remover.
    let u = out["undo"].clone();
    assert_eq!(u[2]["path"], json!(format!("/tracks/{}", n - 1)));
    ops(r, u).unwrap();
    assert_eq!(full(r), antes, "undo devolve o show identico");
    assert_eq!(rev(), rev0 + 2, "desfazer tambem e' edicao");

    // reordenar e' remove + add (nao ha' op `move`)
    let out = ops(
        r,
        json!([
            {"op": "remove", "path": "/tracks/0"},
            {"op": "remove", "path": "/tracks/1"},
            {"op": "add", "path": "/tracks/0", "value": antes["tracks"][2]},
        ]),
    )
    .unwrap();
    let d = full(r);
    assert_eq!(d["tracks"].as_array().unwrap().len(), n - 2);
    assert_eq!(d["tracks"][0], antes["tracks"][2], "remove + add reordena");
    ops(r, out["undo"].clone()).unwrap();
    assert_eq!(full(r), antes, "undo de remove + add");

    // test que falha: a op anterior, que ja' tinha passado, nao fica
    let e = ops(
        r,
        json!([
            {"op": "replace", "path": "/name", "value": "nao devia ficar"},
            {"op": "test", "path": "/fps", "value": 999},
        ]),
    )
    .unwrap_err();
    assert!(e.contains("test") && e.contains("999"), "{}", e);
    assert_eq!(full(r), antes, "op que falha cancela as anteriores");

    // op fora do subconjunto, no meio da lista
    let e = ops(
        r,
        json!([
            {"op": "replace", "path": "/name", "value": "nem esta"},
            {"op": "move", "path": "/fps", "from": "/version"},
            {"op": "replace", "path": "/fps", "value": 1},
        ]),
    )
    .unwrap_err();
    assert!(e.contains("move"), "{}", e);
    assert_eq!(full(r), antes);

    // path que nao existe, indice fora da lista, pointer sem a barra
    for (o, txt) in [
        (
            json!([{"op": "remove", "path": "/nao_existe"}]),
            "nao existe",
        ),
        (json!([{"op": "remove", "path": "/tracks/99"}]), "indice 99"),
        (
            json!([{"op": "add", "path": "fps", "value": 1}]),
            "JSON Pointer",
        ),
        (
            json!([{"op": "replace", "path": "/tracks/0/x/y", "value": 1}]),
            "nao existe",
        ),
    ] {
        let e = ops(r, o).unwrap_err();
        assert!(e.contains(txt), "esperava {:?} em {:?}", txt, e);
    }
    assert_eq!(full(r), antes);

    // um `test` que passa nao entra no undo
    let out = ops(
        r,
        json!([{"op": "test", "path": "/fps", "value": 30},
               {"op": "replace", "path": "/fps", "value": 50}]),
    )
    .unwrap();
    assert_eq!(out["undo"].as_array().unwrap().len(), 1);
    ops(r, out["undo"].clone()).unwrap();
    assert_eq!(full(r), antes);
}

/// O patch passa pela mesma porta do `show_set`: o resultado ainda tem que ser um Show.
fn show_continua_show(r: &Registry) {
    let antes = full(r);
    for (o, txt) in [
        (
            json!([{"op": "replace", "path": "/version", "value": 9}]),
            "versao 9",
        ),
        (
            json!([{"op": "replace", "path": "/tracks", "value": 3}]),
            "show_patch",
        ),
        (
            json!([{"op": "replace", "path": "/fps", "value": "trinta"}]),
            "show_patch",
        ),
    ] {
        let e = ops(r, o).unwrap_err();
        assert!(e.contains(txt), "esperava {:?} em {:?}", txt, e);
        assert_eq!(full(r), antes);
    }
}

/// Duas GUIs no mesmo show: quem manda com a revisao velha leva erro em vez de sobrescrever.
fn rev_velha_e_recusada(r: &Registry) {
    let v = rev();
    r.call(
        "show_patch",
        json!({"rev": v, "ops": [{"op": "replace", "path": "/fps", "value": 24}]}),
    )
    .unwrap();
    let e = r
        .call(
            "show_patch",
            json!({"rev": v, "ops": [{"op": "replace", "path": "/fps", "value": 12}]}),
        )
        .unwrap_err();
    assert_eq!(e, format!("rev {} != {}", v, v + 1));
    assert_eq!(full(r)["fps"], json!(24), "a edicao recusada nao entrou");
}

/// `load` e `show_get {file}` trocam o show inteiro: a revisao que o cliente segurava nao pode
/// valer no show novo, senao o patch dele entra no arquivo errado.
fn trocar_de_show_sobe_rev(r: &Registry) {
    for (cmd, arg) in [("load", "path"), ("show_get", "file")] {
        r.call("show_get", json!({ "file": SPELL })).unwrap();
        let v = rev();
        r.call(cmd, json!({ arg: OUTRO })).unwrap();
        assert_ne!(rev(), v, "{} deixou a rev parada", cmd);
        let e = r
            .call(
                "show_patch",
                json!({"rev": v, "ops": [{"op": "replace", "path": "/fps", "value": 12}]}),
            )
            .unwrap_err();
        assert!(e.contains("rev"), "{}: {:?}", cmd, e);
    }
}

fn graph_e_face(r: &Registry) {
    r.call("show_new", json!({})).unwrap();
    assert_eq!(
        r.call("graph_get", json!({})).unwrap(),
        json!({"nodes": [], "edges": []})
    );
    assert_eq!(r.call("face_get", json!({})).unwrap(), Value::Null);

    // o graph se edita por show_patch: inteiro de uma vez, ou um no' de cada vez
    let g = json!({"nodes": [{"id": "k", "type": "in.key", "key": "Space"},
                             {"id": "c", "type": "cmd", "cmd": "cue_go"}],
                   "edges": [["k.down", "c.trigger"]]});
    let v0 = rev();
    ops(r, json!([{"op": "add", "path": "/graph", "value": g}])).unwrap();
    assert_eq!(rev(), v0 + 1);
    assert_eq!(r.call("graph_get", json!({})).unwrap(), g);
    assert_eq!(full(r)["graph"], g, "o graph mora no show, nao ao lado");

    ops(
        r,
        json!([{"op": "add", "path": "/graph/nodes/-",
                   "value": {"id": "t", "type": "logic.toggle"}}]),
    )
    .unwrap();
    assert_eq!(
        r.call("graph_get", json!({})).unwrap()["nodes"][2]["id"],
        json!("t")
    );

    // face inline sai como veio
    ops(
        r,
        json!([{"op": "add", "path": "/face", "value": {"widgets": [{"id": "go"}]}}]),
    )
    .unwrap();
    assert_eq!(
        r.call("face_get", json!({})).unwrap()["widgets"][0]["id"],
        json!("go")
    );

    // face por nome vira faces/<nome>.face.json; sem o arquivo, o erro diz qual e'
    ops(
        r,
        json!([{"op": "replace", "path": "/face", "value": "nao_existe"}]),
    )
    .unwrap();
    let e = r.call("face_get", json!({})).unwrap_err();
    assert!(e.contains("nao_existe.face.json"), "{}", e);
}
