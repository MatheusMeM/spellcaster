//! MIDI map with no hardware. Its own binary because the open show (`OPEN`) is one per process.

use engine::registry::base;
use serde_json::json;

#[test]
fn the_map_in_the_show_is_what_the_event_fires() {
    let r = base();

    // a machine with no device at all returns an empty list, not an error
    let p = r
        .call("midi_ports", json!({}))
        .expect("midi_ports never fails");
    assert!(p["ports"].is_array(), "{}", p);
    assert_eq!(p["open"], json!(null));

    // with no open port, draining does nothing and `midi_last` invents no event
    engine::midi::pump();
    assert_eq!(r.call("midi_last", json!({})).unwrap()["key"], json!(null));

    // key and command are validated at mapping time, and not in the middle of the show
    assert!(r
        .call("midi_map", json!({"key": "144", "cmd": "resume"}))
        .is_err());
    assert!(r
        .call(
            "midi_map",
            json!({"key": "144/60", "cmd": "does_not_exist"})
        )
        .is_err());

    r.call("midi_map", json!({"key": "144/60", "cmd": "resume"}))
        .unwrap();
    let m = r
        .call(
            "midi_map",
            json!({"key": "176/1", "cmd": "level_set",
                   "args": {"address": 1, "values": ["$255"]}}),
        )
        .unwrap();
    assert_eq!(m["144/60"], json!({"cmd": "resume", "args": {}}));

    // the map lives in the open .spell, not in a parallel table
    let sh = r.call("show_get", json!({"full": true})).unwrap();
    assert_eq!(sh["midi"], m);
    assert_eq!(r.call("midi_maps", json!({})).unwrap(), m);

    // what the event fires: key -> (command, args), with the "$" already substituted
    let (cmd, args) = engine::midi::liga("176/1").expect("176/1 mapped");
    assert_eq!(cmd, "level_set");
    assert_eq!(
        engine::midi::expande(&args, 100.0 / 127.0),
        json!({"address": 1, "values": [201]}),
        "$255 = the DMX level of byte 100"
    );
    assert!(engine::midi::liga("144/61").is_none(), "an unmapped key");

    // the whole path of an event, with no hardware: CC 7 = 100 -> track_add address=100.
    // (`track_add` because it leaves a mark in the show; a transport command would need a player.)
    r.call(
        "midi_map",
        json!({"key": "176/7", "cmd": "track_add", "args": {"address": "$127"}}),
    )
    .unwrap();
    engine::midi::entrega(176, 7, 100);
    let sh = r.call("show_get", json!({"full": true})).unwrap();
    assert_eq!(
        sh["tracks"][0]["address"],
        json!(100),
        "the mapped command ran with the $ substituted: {}",
        sh["tracks"]
    );
    assert_eq!(
        r.call("midi_last", json!({})).unwrap()["key"],
        json!("176/7")
    );
    r.call("midi_unmap", json!({"key": "176/7"})).unwrap();

    r.call("midi_unmap", json!({"key": "144/60"})).unwrap();
    assert!(
        r.call("midi_unmap", json!({"key": "144/60"})).is_err(),
        "removing it twice is an error"
    );
    assert!(r
        .call("midi_maps", json!({}))
        .unwrap()
        .get("144/60")
        .is_none());
}
