//! Mapa MIDI sem hardware. Binario proprio porque o show aberto (`OPEN`) e' um por processo.

use engine::registry::base;
use serde_json::json;

#[test]
fn mapa_no_show_e_o_que_o_evento_dispara() {
    let r = base();

    // maquina sem dispositivo nenhum devolve lista vazia, nao erro
    let p = r
        .call("midi_ports", json!({}))
        .expect("midi_ports nunca falha");
    assert!(p["ports"].is_array(), "{}", p);
    assert_eq!(p["open"], json!(null));

    // sem porta aberta, drenar nao faz nada e `midi_last` nao inventa evento
    engine::midi::pump();
    assert_eq!(r.call("midi_last", json!({})).unwrap()["key"], json!(null));

    // chave e comando sao validados na hora de mapear, e nao no meio do show
    assert!(r
        .call("midi_map", json!({"key": "144", "cmd": "resume"}))
        .is_err());
    assert!(r
        .call("midi_map", json!({"key": "144/60", "cmd": "nao_existe"}))
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

    // o mapa mora no .spell aberto, nao numa tabela paralela
    let sh = r.call("show_get", json!({"full": true})).unwrap();
    assert_eq!(sh["midi"], m);
    assert_eq!(r.call("midi_maps", json!({})).unwrap(), m);

    // o que o evento dispara: chave -> (comando, args), com o "$" ja' substituido
    let (cmd, args) = engine::midi::liga("176/1").expect("176/1 mapeado");
    assert_eq!(cmd, "level_set");
    assert_eq!(
        engine::midi::expande(&args, 100.0 / 127.0),
        json!({"address": 1, "values": [201]}),
        "$255 = nivel DMX do byte 100"
    );
    assert!(engine::midi::liga("144/61").is_none(), "tecla sem mapa");

    // o caminho inteiro de um evento, sem hardware: CC 7 = 100 -> track_add address=100.
    // (`track_add` porque ele deixa marca no show; comando de transporte precisaria de player.)
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
        "o comando do mapa rodou com o $ substituido: {}",
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
        "tirar duas vezes e' erro"
    );
    assert!(r
        .call("midi_maps", json!({}))
        .unwrap()
        .get("144/60")
        .is_none());
}
