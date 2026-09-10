//! `show_patch` (JSON Patch), `graph_get`, `face_get` and the `rev` counter. Its own binary
//! because `OPEN` and `REV` are one per process; and a single `#[test]` because the tests of one
//! binary run in parallel and every one here touches that state.

use engine::edit::rev;
use engine::registry::base;
use engine::Registry;
use serde_json::{json, Value};

const SPELL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../shows/medgrupo.spell");
const OUTRO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../shows/medgrupo_r0.spell");

/// `show_patch` with a list of ops, with no revision check.
fn ops(r: &Registry, o: Value) -> Result<Value, String> {
    r.call("show_patch", json!({ "ops": o }))
}

fn full(r: &Registry) -> Value {
    r.call("show_get", json!({"full": true})).unwrap()
}

#[test]
fn patch_graph_and_face() {
    let r = base();
    all_or_nothing(&r);
    show_stays_a_show(&r);
    stale_rev_is_refused(&r);
    swapping_show_bumps_rev(&r);
    graph_and_face(&r);
}

fn all_or_nothing(r: &Registry) {
    r.call("show_get", json!({ "file": SPELL })).unwrap();
    let antes = full(r);
    let rev0 = rev();

    // add at /tracks/-, replace at /fps, add of an `extra` key
    let out = ops(
        r,
        json!([
            {"op": "add", "path": "/tracks/-", "value": {"type": "dmx", "universe": 9,
                                                         "address": 1, "keys": []}},
            {"op": "replace", "path": "/fps", "value": 25},
            {"op": "add", "path": "/markers", "value": [{"t": 1.0, "name": "one"}]},
        ]),
    )
    .unwrap();
    assert_eq!(out["rev"], json!(rev0 + 1));
    assert_eq!(rev(), rev0 + 1);
    let d = full(r);
    assert_eq!(d["fps"], json!(25));
    assert_eq!(d["markers"][0]["name"], json!("one"));
    let n = d["tracks"].as_array().unwrap().len();
    assert_eq!(n, antes["tracks"].as_array().unwrap().len() + 1);

    // the undo already comes in application order: sending it back as it came restores it byte
    // for byte. "/tracks/-" became "/tracks/<index>" in the inverse: "-" is no good for removing.
    let u = out["undo"].clone();
    assert_eq!(u[2]["path"], json!(format!("/tracks/{}", n - 1)));
    ops(r, u).unwrap();
    assert_eq!(full(r), antes, "undo gives back an identical show");
    assert_eq!(rev(), rev0 + 2, "an undo is an edit too");

    // reordering is remove + add (there is no `move` op)
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
    assert_eq!(d["tracks"][0], antes["tracks"][2], "remove + add reorders");
    ops(r, out["undo"].clone()).unwrap();
    assert_eq!(full(r), antes, "undo of remove + add");

    // a failing test: the previous op, which had already passed, does not stay
    let e = ops(
        r,
        json!([
            {"op": "replace", "path": "/name", "value": "should not stay"},
            {"op": "test", "path": "/fps", "value": 999},
        ]),
    )
    .unwrap_err();
    assert!(e.contains("test") && e.contains("999"), "{}", e);
    assert_eq!(full(r), antes, "a failing op cancels the previous ones");

    // an op outside the subset, in the middle of the list
    let e = ops(
        r,
        json!([
            {"op": "replace", "path": "/name", "value": "not this one either"},
            {"op": "move", "path": "/fps", "from": "/version"},
            {"op": "replace", "path": "/fps", "value": 1},
        ]),
    )
    .unwrap_err();
    assert!(e.contains("move"), "{}", e);
    assert_eq!(full(r), antes);

    // a path that does not exist, an index outside the list, a pointer with no slash
    for (o, txt) in [
        (
            json!([{"op": "remove", "path": "/does_not_exist"}]),
            "does not exist",
        ),
        (json!([{"op": "remove", "path": "/tracks/99"}]), "index 99"),
        (
            json!([{"op": "add", "path": "fps", "value": 1}]),
            "JSON Pointer",
        ),
        (
            json!([{"op": "replace", "path": "/tracks/0/x/y", "value": 1}]),
            "does not exist",
        ),
    ] {
        let e = ops(r, o).unwrap_err();
        assert!(e.contains(txt), "expected {:?} in {:?}", txt, e);
    }
    assert_eq!(full(r), antes);

    // a `test` that passes does not enter the undo
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

/// The patch goes through the same door as `show_set`: the result still has to be a Show.
fn show_stays_a_show(r: &Registry) {
    let antes = full(r);
    for (o, txt) in [
        (
            json!([{"op": "replace", "path": "/version", "value": 9}]),
            "version 9",
        ),
        (
            json!([{"op": "replace", "path": "/tracks", "value": 3}]),
            "show_patch",
        ),
        (
            json!([{"op": "replace", "path": "/fps", "value": "thirty"}]),
            "show_patch",
        ),
    ] {
        let e = ops(r, o).unwrap_err();
        assert!(e.contains(txt), "expected {:?} in {:?}", txt, e);
        assert_eq!(full(r), antes);
    }
}

/// Two GUIs on the same show: whoever sends with the stale revision gets an error instead of
/// overwriting.
fn stale_rev_is_refused(r: &Registry) {
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
    assert_eq!(full(r)["fps"], json!(24), "the refused edit did not land");
}

/// `load` and `show_get {file}` swap the whole show: the revision the client was holding cannot
/// stay valid on the new show, or its patch lands on the wrong file.
fn swapping_show_bumps_rev(r: &Registry) {
    for (cmd, arg) in [("load", "file"), ("show_get", "file")] {
        r.call("show_get", json!({ "file": SPELL })).unwrap();
        let v = rev();
        r.call(cmd, json!({ arg: OUTRO })).unwrap();
        assert_ne!(rev(), v, "{} left the rev where it was", cmd);
        let e = r
            .call(
                "show_patch",
                json!({"rev": v, "ops": [{"op": "replace", "path": "/fps", "value": 12}]}),
            )
            .unwrap_err();
        assert!(e.contains("rev"), "{}: {:?}", cmd, e);
    }
}

fn graph_and_face(r: &Registry) {
    r.call("show_new", json!({})).unwrap();
    assert_eq!(
        r.call("graph_get", json!({})).unwrap(),
        json!({"nodes": [], "edges": []})
    );
    assert_eq!(r.call("face_get", json!({})).unwrap(), Value::Null);

    // the graph is edited by show_patch: whole in one go, or one node at a time
    let g = json!({"nodes": [{"id": "k", "type": "in.key", "key": "Space"},
                             {"id": "c", "type": "cmd", "cmd": "cue_go"}],
                   "edges": [["k.down", "c.trigger"]]});
    let v0 = rev();
    ops(r, json!([{"op": "add", "path": "/graph", "value": g}])).unwrap();
    assert_eq!(rev(), v0 + 1);
    assert_eq!(r.call("graph_get", json!({})).unwrap(), g);
    assert_eq!(
        full(r)["graph"],
        g,
        "the graph lives in the show, not beside it"
    );

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

    // an inline face comes out as it came in
    ops(
        r,
        json!([{"op": "add", "path": "/face", "value": {"widgets": [{"id": "go"}]}}]),
    )
    .unwrap();
    assert_eq!(
        r.call("face_get", json!({})).unwrap()["widgets"][0]["id"],
        json!("go")
    );

    // a face by name becomes faces/<name>.face.json; with no file, the error says which one
    ops(
        r,
        json!([{"op": "replace", "path": "/face", "value": "does_not_exist"}]),
    )
    .unwrap();
    let e = r.call("face_get", json!({})).unwrap_err();
    assert!(e.contains("does_not_exist.face.json"), "{}", e);
}
