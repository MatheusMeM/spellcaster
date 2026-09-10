//! Editing the show through the registry (the Python `test_gui_api.py`): open, edit
//! track/keyframe, cue and patch, save and reopen. Its own binary because `OPEN` is one per
//! process and the `registry.rs` tests run in parallel in the same lib binary.

use engine::registry::base;
use serde_json::{json, Value};

const SPELL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../shows/medgrupo.spell");

#[test]
fn open_edit_save_and_reopen() {
    let r = base();
    let c = |n: &str, a: Value| r.call(n, a);

    // a new show: what the MCP sees before any file
    let sh = c("show_new", json!({})).unwrap();
    assert_eq!(sh["name"], json!("new show"));
    assert_eq!(sh["outputs"][0]["type"], json!("sacn"));
    assert_eq!(sh["tracks"], json!([]));
    assert_eq!(sh["patch"], json!([]));
    assert_eq!(
        c("show_get", json!({"full": true})).unwrap()["cues"],
        json!([])
    );

    // medgrupo + a new track with out-of-order keyframes
    c("show_get", json!({ "file": SPELL })).unwrap();
    let i = c(
        "track_add",
        json!({"type": "dmx", "universe": 2, "address": 10, "name": "test"}),
    )
    .unwrap();
    assert_eq!(i, json!(3));
    // `label` is the old name of `name` and stays accepted for one round (deprecated)
    let j = c("track_add", json!({"label": "old"})).unwrap();
    let tr = c("show_get", json!({"full": true})).unwrap();
    assert_eq!(tr["tracks"][j.as_u64().unwrap() as usize]["name"], "old");
    c("track_del", json!({ "index": j })).unwrap();
    c("key_set", json!({"track": 3, "t": 0, "value": [0, 0, 0]})).unwrap();
    c(
        "key_set",
        json!({"track": 3, "t": 2.5, "value": "[255, 128, 0]", "curve": "inout"}),
    )
    .unwrap();
    let ks = c(
        "key_set",
        json!({"track": 3, "t": 1, "value": [10, 10, 10]}),
    )
    .unwrap();
    let ts: Vec<f64> = ks
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k[0].as_f64().unwrap())
        .collect();
    assert_eq!(ts, vec![0.0, 1.0, 2.5], "sorted by time");
    assert_eq!(
        ks[2],
        json!([2.5, [255, 128, 0], "inout"]),
        "JSON text becomes a list"
    );

    // the same t replaces; key_del counts; an invalid curve and track are errors
    let ks = c(
        "key_set",
        json!({"track": 3, "t": 1, "value": 10, "curve": "hold"}),
    )
    .unwrap();
    assert_eq!(ks[1], json!([1.0, 10, "hold"]));
    assert_eq!(ks.as_array().unwrap().len(), 3);
    assert_eq!(c("key_del", json!({"track": 3, "t": 1})).unwrap(), json!(1));
    assert_eq!(c("key_del", json!({"track": 3, "t": 1})).unwrap(), json!(0));
    assert!(c("key_set", json!({"track": 3, "t": 0, "curve": "spring"}))
        .unwrap_err()
        .contains("spring"));
    assert!(c("key_set", json!({"track": 9, "t": 0}))
        .unwrap_err()
        .contains("track 9"));
    assert_eq!(
        c("key_set", json!({"track": 3, "t": 5})).unwrap()[2],
        json!([5.0, 0, "linear"]),
        "no value = 0"
    );

    // cues: create, replace, invalid key, remove
    assert_eq!(
        c(
            "cue_set",
            json!({"name": "a", "fade": 2, "values": {"1/100": [255, 0], "7": 10}}),
        )
        .unwrap(),
        json!(0)
    );
    assert_eq!(
        c("cue_set", json!({"name": "b", "follow": true})).unwrap(),
        json!(1)
    );
    assert_eq!(
        c("cue_set", json!({"index": 0, "name": "a2", "fade": 3})).unwrap(),
        json!(0)
    );
    assert!(c("cue_set", json!({"index": 5})).is_err());
    assert!(c("cue_set", json!({"values": {"x/y": 1}}))
        .unwrap_err()
        .contains("x/y"));
    let full = c("show_get", json!({"full": true})).unwrap();
    assert_eq!(full["cues"][0]["name"], json!("a2"));
    assert_eq!(full["cues"][1]["follow"], json!(true));
    let resumo = c("show_get", json!({})).unwrap();
    assert_eq!(resumo["cues"][0]["fade"], json!(3.0));
    assert_eq!(
        c("cue_del", json!({"index": 0})).unwrap()["name"],
        json!("a2")
    );
    assert!(c("cue_del", json!({"index": 1})).is_err());

    // patch: 17 ch at 300 and 316 overlap; at 317 they do not
    let bsw = |n: &str, a: u16| json!({"name": n, "profile": "bsw_scorpio_17", "address": a});
    let rows = c("patch_add", bsw("bsw_1", 300)).unwrap();
    assert_eq!(rows[0]["channels"], json!(17));
    let e = c("patch_add", bsw("bsw_2", 316)).unwrap_err();
    assert!(e.contains("overlap") && e.contains("316"), "{}", e);
    let rows = c("patch_add", bsw("bsw_2", 317)).unwrap();
    assert_eq!(
        rows.as_array().unwrap().len(),
        2,
        "the refused one did not stay in the patch"
    );
    assert!(c("patch_add", bsw("bsw_2", 400))
        .unwrap_err()
        .contains("is already in"));
    assert!(c(
        "patch_add",
        json!({"name": "x", "profile": "dimmer_1", "address": 513})
    )
    .unwrap_err()
    .contains("512"));
    assert!(c(
        "patch_add",
        json!({"name": "x", "profile": "does_not_exist", "address": 1})
    )
    .unwrap_err()
    .contains("not found"));
    let ck = c("patch_check", json!({})).unwrap();
    assert_eq!(ck["error"], Value::Null);
    assert_eq!(ck["rows"].as_array().unwrap().len(), 2);
    assert_eq!(
        c("show_get", json!({})).unwrap()["patch"],
        json!(["bsw_1", "bsw_2"])
    );
    assert!(c("profiles", json!({}))
        .unwrap()
        .as_array()
        .unwrap()
        .contains(&json!("bsw_scorpio_17")));
    assert_eq!(
        c("patch_del", json!({"name": "bsw_1"})).unwrap()["address"],
        json!(300)
    );
    assert!(c("patch_del", json!({"name": "bsw_1"})).is_err());

    // save, clear, reopen: identical
    let out = std::env::temp_dir().join("spellcore_test_edit.spell");
    let f = out.to_string_lossy().to_string();
    assert_eq!(c("show_save", json!({ "file": f })).unwrap(), json!(f));
    let antes = c("show_get", json!({"full": true})).unwrap();
    assert_eq!(
        c("track_del", json!({"index": 0})).unwrap()["type"],
        json!("pyfx")
    );
    c("show_new", json!({})).unwrap();
    assert!(
        c("show_save", json!({})).unwrap_err().contains("no path"),
        "a new show has no path"
    );
    let depois = c("show_get", json!({"file": f, "full": true})).unwrap();
    std::fs::remove_file(&out).ok();
    assert_eq!(depois, antes);
    assert_eq!(depois["tracks"].as_array().unwrap().len(), 4);
    assert_eq!(depois["patch"][0]["name"], json!("bsw_2"));

    // show_set validates and replaces
    let got = c(
        "show_set",
        json!({"data": {"name": "x", "fps": 25, "duration": 3.0,
                        "tracks": [{"type": "dmx", "keys": [[0, 1]]}], "_dir": "junk"}}),
    )
    .unwrap();
    assert_eq!(got["fps"], json!(25));
    assert_eq!(got["version"], json!(1));
    assert!(got.get("_dir").is_none());
    assert_eq!(
        c("show_set", json!({"data": "{\"name\": \"txt\"}"})).unwrap()["name"],
        json!("txt")
    );
    assert!(c("show_set", json!({"data": {"tracks": 3}})).is_err());
    assert!(c("show_set", json!({"data": [1, 2]})).is_err());
    assert!(c("show_set", json!({"data": {"version": 9}}))
        .unwrap_err()
        .contains("version 9"));
}
