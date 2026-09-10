//! The MCP server as Claude sees it: `spellcore mcp` over stdio, one JSON-RPC message per line.
//! It brings the real binary up (not the `Spell` in memory) because what breaks in practice is
//! the packaging: a wrong subcommand, a dirty stdout, an incomplete registry.

use serde_json::{json, Value};

mod common;
use common::{texto, Mcp};

const SHOW: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../shows/medgrupo.spell");

#[test]
fn handshake_tools_and_resources() {
    let mut m = Mcp::cru(); // without the harness handshake: it is the thing being checked here

    let init = m.rpc(
        "initialize",
        json!({"protocolVersion": "2025-06-18", "capabilities": {},
               "clientInfo": {"name": "test", "version": "0"}}),
    );
    assert_eq!(init["serverInfo"]["name"], json!("spellcaster"));
    assert!(init["capabilities"]["tools"].is_object());
    assert!(init["capabilities"]["resources"].is_object());
    assert!(init["instructions"].as_str().unwrap().contains("show_get"));
    m.notifica("notifications/initialized");

    // tools/list: one tool per command of the CLI registry, with the schemars schema
    let tools = m.rpc("tools/list", json!({}));
    let nomes: Vec<&str> = tools["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .map(|t| t["name"].as_str().unwrap_or(""))
        .collect();
    for n in [
        "load",
        "show_get",
        "pause",
        "stop",
        "locate",
        "cue_go",
        "transport_state",
        "show_new",
        "show_save",
        "track_add",
        "key_set",
        "cue_set",
        "patch_add",
        "patch_check",
        "show_patch",
        "graph_get",
        "graph_check",
        "face_get",
        "profile_get",
        "level_set",
        "cue_capture",
        "fixture_set",
        "play_show",
        "net",
        "laser_open",
        "laser_play",
    ] {
        assert!(nomes.contains(&n), "tool {} missing: {:?}", n, nomes);
    }
    let play = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "play_show")
        .unwrap();
    assert_eq!(play["inputSchema"]["type"], json!("object"));
    assert!(play["inputSchema"]["properties"]["loop"].is_object());
    assert_eq!(play["inputSchema"]["required"], json!(["file"]));

    // tools/call show_get on the conformance show
    let r = m.tool("show_get", json!({"file": SHOW}));
    assert_eq!(r["isError"], json!(false));
    let d: Value = serde_json::from_str(&texto(&r)).expect("show_get returns JSON");
    assert_eq!(d["aberto"], json!(true));
    assert_eq!(d["fps"], json!(30));
    assert!(!d["tracks"].as_array().unwrap().is_empty());
    assert!(d["outputs"].as_array().unwrap().iter().any(|o| o == "sacn"));

    // a command error comes back as a TOOL error (the client reads the text), not as a JSON-RPC
    // error
    let r = m.tool("pause", json!({}));
    assert_eq!(r["isError"], json!(true));
    assert_eq!(texto(&r), "no player running");

    // resources: spell://commands is the whole registry; spell://show, the show just opened
    let rs = m.rpc("resources/list", json!({}));
    let uris: Vec<&str> = rs["resources"]
        .as_array()
        .expect("resources")
        .iter()
        .map(|r| r["uri"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(
        uris,
        vec![
            "spell://show",
            "spell://commands",
            "spell://graph",
            "spell://face"
        ]
    );

    let c = m.rpc("resources/read", json!({"uri": "spell://commands"}));
    let txt = c["contents"][0]["text"].as_str().expect("text");
    let cmds: Value = serde_json::from_str(txt).expect("commands returns JSON");
    let nomes: Vec<&str> = cmds
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap_or(""))
        .collect();
    assert!(nomes.contains(&"play_show") && nomes.contains(&"show_get"));
    assert!(
        cmds[0]["params"].is_object(),
        "every command carries the schema"
    );

    let s = m.rpc("resources/read", json!({"uri": "spell://show"}));
    let d: Value = serde_json::from_str(s["contents"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(
        d["aberto"],
        json!(true),
        "the show opened by show_get stays open"
    );
    assert_eq!(d["transport"], Value::Null, "no player running");

    // graph: the resource reads the registry graph_get; the medgrupo has no graph, so it comes
    // back empty
    let g = m.rpc("resources/read", json!({"uri": "spell://graph"}));
    let g: Value = serde_json::from_str(g["contents"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(g, json!({"nodes": [], "edges": []}));

    // show_patch at /graph + resource: what the AI writes is what the resource returns
    let g = json!({"nodes": [{"id": "k", "type": "in.key", "key": "Space"},
                             {"id": "t", "type": "logic.toggle"},
                             {"id": "c", "type": "cmd", "cmd": "cue_go"}],
                   "edges": [["k.down", "t.in"], ["t.out", "c.trigger"]]});
    let r = m.tool(
        "show_patch",
        json!({"ops": [{"op": "add", "path": "/graph", "value": g}]}),
    );
    assert_eq!(r["isError"], json!(false));
    let lido = m.rpc("resources/read", json!({"uri": "spell://graph"}));
    let lido: Value = serde_json::from_str(lido["contents"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(lido, g);

    // graph_check compiles the open graph: 3 nodes, no error
    let r = m.tool("graph_check", json!({}));
    let c: Value = serde_json::from_str(&texto(&r)).expect("graph_check returns JSON");
    assert_eq!(c["nodes"], json!(3), "{}", texto(&r));
    assert_eq!(c["error"], Value::Null);

    // a cycle: an error in the field, not an exception (the editor shows the text beside the nodes)
    let ciclo = json!({"nodes": [{"id": "a", "type": "logic.not"},
                                 {"id": "b", "type": "logic.not"}],
                       "edges": [["a.out", "b.in"], ["b.out", "a.in"]]});
    m.tool(
        "show_patch",
        json!({"ops": [{"op": "replace", "path": "/graph", "value": ciclo}]}),
    );
    let r = m.tool("graph_check", json!({}));
    let c: Value = serde_json::from_str(&texto(&r)).expect("graph_check returns JSON");
    let e = c["error"].as_str().unwrap_or("");
    assert!(
        e.contains("cycle") && e.contains('a') && e.contains('b'),
        "{}",
        e
    );

    // show_patch: it edits and returns the undo ops
    let r = m.tool(
        "show_patch",
        json!({"ops": [{"op": "replace", "path": "/fps", "value": 25}]}),
    );
    let d: Value = serde_json::from_str(&texto(&r)).expect("show_patch returns JSON");
    assert!(d["rev"].as_u64().unwrap_or(0) > 0);
    assert_eq!(
        d["undo"],
        json!([{"op": "replace", "path": "/fps", "value": 30}])
    );
}

/// `play_show` blocks until the end of the show: through the MCP it runs in a thread and the
/// tool comes back at once. And, above all, the `play` status line must NOT go out on the stdout
/// — the JSON-RPC goes through there.
#[test]
fn play_show_in_the_background_does_not_dirty_the_stdout() {
    let p = std::env::temp_dir().join("spellcore_mcp_play.spell");
    std::fs::write(
        &p,
        r#"{"name":"short","fps":30,"duration":0.4,"version":1,"outputs":[],
            "tracks":[{"type":"dmx","universe":1,"address":1,"keys":[[0,10],[0.4,200]]}]}"#,
    )
    .expect("write the show");
    let mut m = Mcp::start();

    let f = p.to_string_lossy().to_string();
    let r = m.tool("play_show", json!({"file": f}));
    assert_eq!(r["isError"], json!(false));
    assert!(
        texto(&r).starts_with("play_show started in the background"),
        "{}",
        texto(&r)
    );

    // the show is 0.4 s long; until it ends, `transport_state` answers — and every answer that
    // arrives whole here proves the play status line went to the stderr.
    let mut viu_play = false;
    for _ in 0..9 {
        let r = m.tool("transport_state", json!({}));
        if r["isError"] == json!(false) && texto(&r).contains("\"state\": \"play\"") {
            viu_play = true;
            break;
        }
    }
    assert!(viu_play, "the player never got to play");
    m.tool("stop", json!({}));
    std::fs::remove_file(&p).ok();
}
