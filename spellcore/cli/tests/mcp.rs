//! O servidor MCP como o Claude o ve: `spellcore mcp` em stdio, uma mensagem JSON-RPC por linha.
//! Sobe o binario de verdade (nao o `Spell` em memoria) porque o que quebra na pratica e' o
//! empacotamento: subcomando errado, stdout sujo, registry incompleto.

use serde_json::{json, Value};

mod common;
use common::{texto, Mcp};

const SHOW: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../shows/medgrupo.spell");

#[test]
fn handshake_tools_e_resources() {
    let mut m = Mcp::cru(); // sem o handshake do harness: e' ele que se confere aqui

    let init = m.rpc(
        "initialize",
        json!({"protocolVersion": "2025-06-18", "capabilities": {},
               "clientInfo": {"name": "teste", "version": "0"}}),
    );
    assert_eq!(init["serverInfo"]["name"], json!("spellcaster"));
    assert!(init["capabilities"]["tools"].is_object());
    assert!(init["capabilities"]["resources"].is_object());
    assert!(init["instructions"].as_str().unwrap().contains("show_get"));
    m.notifica("notifications/initialized");

    // tools/list: uma tool por comando do registry da CLI, com o schema do schemars
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
        assert!(nomes.contains(&n), "tool {} ausente: {:?}", n, nomes);
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

    // tools/call show_get no show de conformidade
    let r = m.tool("show_get", json!({"file": SHOW}));
    assert_eq!(r["isError"], json!(false));
    let d: Value = serde_json::from_str(&texto(&r)).expect("show_get devolve JSON");
    assert_eq!(d["aberto"], json!(true));
    assert_eq!(d["fps"], json!(30));
    assert!(!d["tracks"].as_array().unwrap().is_empty());
    assert!(d["outputs"].as_array().unwrap().iter().any(|o| o == "sacn"));

    // erro do comando volta como erro de TOOL (o cliente le o texto), nao como erro JSON-RPC
    let r = m.tool("pause", json!({}));
    assert_eq!(r["isError"], json!(true));
    assert_eq!(texto(&r), "sem player em execucao");

    // resources: spell://commands e' o registry inteiro; spell://show, o show que acabou de abrir
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
    let txt = c["contents"][0]["text"].as_str().expect("texto");
    let cmds: Value = serde_json::from_str(txt).expect("commands devolve JSON");
    let nomes: Vec<&str> = cmds
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap_or(""))
        .collect();
    assert!(nomes.contains(&"play_show") && nomes.contains(&"show_get"));
    assert!(cmds[0]["params"].is_object(), "cada comando leva o schema");

    let s = m.rpc("resources/read", json!({"uri": "spell://show"}));
    let d: Value = serde_json::from_str(s["contents"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(
        d["aberto"],
        json!(true),
        "o show aberto pelo show_get continua aberto"
    );
    assert_eq!(d["transport"], Value::Null, "nenhum player rodando");

    // graph: o resource le o graph_get do registry; o medgrupo nao tem graph, entao vem vazio
    let g = m.rpc("resources/read", json!({"uri": "spell://graph"}));
    let g: Value = serde_json::from_str(g["contents"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(g, json!({"nodes": [], "edges": []}));

    // show_patch em /graph + resource: o que a IA grava e' o que o resource devolve
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

    // graph_check compila o graph aberto: 3 nos, sem erro
    let r = m.tool("graph_check", json!({}));
    let c: Value = serde_json::from_str(&texto(&r)).expect("graph_check devolve JSON");
    assert_eq!(c["nodes"], json!(3), "{}", texto(&r));
    assert_eq!(c["error"], Value::Null);

    // ciclo: erro no campo, nao excecao (o editor mostra o texto ao lado dos nos)
    let ciclo = json!({"nodes": [{"id": "a", "type": "logic.not"},
                                 {"id": "b", "type": "logic.not"}],
                       "edges": [["a.out", "b.in"], ["b.out", "a.in"]]});
    m.tool(
        "show_patch",
        json!({"ops": [{"op": "replace", "path": "/graph", "value": ciclo}]}),
    );
    let r = m.tool("graph_check", json!({}));
    let c: Value = serde_json::from_str(&texto(&r)).expect("graph_check devolve JSON");
    let e = c["error"].as_str().unwrap_or("");
    assert!(
        e.contains("cycle") && e.contains('a') && e.contains('b'),
        "{}",
        e
    );

    // show_patch: edita e devolve as ops de undo
    let r = m.tool(
        "show_patch",
        json!({"ops": [{"op": "replace", "path": "/fps", "value": 25}]}),
    );
    let d: Value = serde_json::from_str(&texto(&r)).expect("show_patch devolve JSON");
    assert!(d["rev"].as_u64().unwrap_or(0) > 0);
    assert_eq!(
        d["undo"],
        json!([{"op": "replace", "path": "/fps", "value": 30}])
    );
}

/// `play_show` bloqueia ate o fim do show: pelo MCP ele roda em thread e a tool volta na hora.
/// E, sobretudo, a linha de status do `play` NAO pode sair no stdout — la' passa o JSON-RPC.
#[test]
fn play_show_em_background_nao_suja_o_stdout() {
    let p = std::env::temp_dir().join("spellcore_mcp_play.spell");
    std::fs::write(
        &p,
        r#"{"name":"curto","fps":30,"duration":0.4,"version":1,"outputs":[],
            "tracks":[{"type":"dmx","universe":1,"address":1,"keys":[[0,10],[0.4,200]]}]}"#,
    )
    .expect("escrever o show");
    let mut m = Mcp::start();

    let f = p.to_string_lossy().to_string();
    let r = m.tool("play_show", json!({"file": f}));
    assert_eq!(r["isError"], json!(false));
    assert!(
        texto(&r).starts_with("play_show iniciado em background"),
        "{}",
        texto(&r)
    );

    // o show tem 0,4 s; ate ele acabar, `transport_state` responde — e cada resposta que chega
    // inteira aqui prova que a linha de status do play foi para o stderr.
    let mut viu_play = false;
    for _ in 0..9 {
        let r = m.tool("transport_state", json!({}));
        if r["isError"] == json!(false) && texto(&r).contains("\"state\": \"play\"") {
            viu_play = true;
            break;
        }
    }
    assert!(viu_play, "o player nao chegou a tocar");
    m.tool("stop", json!({}));
    std::fs::remove_file(&p).ok();
}
