//! Gravacao de DMX: uma saida sACN em 127.0.0.1 no universo 7 muda de valor, o player toca com o
//! track armado e os valores viram keyframes no show aberto. Binario proprio: `OPEN` (o show
//! aberto) e `CURRENT` (o player vivo) sao globais do processo.
//!
//! Tudo em loopback, sem hardware: o `SacnOut` manda unicast para 127.0.0.1:5568 alem do
//! multicast, e e' por esse unicast que o `SacnIn` do player recebe (o mesmo truque dos testes
//! de `protocols::sacn`).

use engine::registry::Registry;
use engine::{Player, Show};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

fn espera(secs: f64, mut cond: impl FnMut() -> bool) -> bool {
    let fim = Instant::now() + Duration::from_secs_f64(secs);
    while Instant::now() < fim {
        if cond() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    cond()
}

/// Valores de um track do show aberto, na ordem dos keyframes.
fn gravados(r: &Registry, track: usize) -> Vec<f64> {
    let sh = r.call("show_get", json!({"full": true})).expect("show_get");
    sh["tracks"][track]["keys"]
        .as_array()
        .map(|ks| {
            ks.iter()
                .filter_map(|k| k.get(1).and_then(Value::as_f64))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn track_armado_grava_o_universo_de_entrada() {
    let cfg = json!({
        "name": "gravacao", "fps": 30, "version": 1, "outputs": [],
        "inputs": [{"type": "sacn", "universe": 7}],
        "tracks": [{"type": "dmx", "universe": 7, "address": 1, "keys": []}]
    });
    let r = engine::registry::base();
    // sem player ainda: `input_get` responde como todo comando de transporte
    assert_eq!(
        r.call("input_get", json!({"universe": 7})).unwrap_err(),
        "sem player em execucao"
    );
    assert_eq!(
        r.call("rec_state", json!({})).unwrap(),
        json!({"recording": false, "tracks": []})
    );
    r.call("show_set", json!({ "data": cfg }))
        .expect("show_set");

    let sh: Show = serde_json::from_value(cfg).expect("show de teste");
    let mut p = match Player::new(sh, false) {
        Ok(p) => p,
        Err(e) => return println!("pulado: entrada sACN nao subiu: {}", e),
    };
    p.start(None).expect("start");
    let h = p.handle();

    let mut tx = match protocols::sacn::SacnOut::new(&[7], Some(vec!["127.0.0.1".parse().unwrap()]))
    {
        Ok(t) => t,
        Err(e) => {
            p.close();
            return println!("pulado: saida sACN nao subiu: {:?}", e.kind());
        }
    };

    r.call("rec_arm", json!({"track": 0, "on": true}))
        .expect("rec_arm");
    assert_eq!(
        r.call("rec_state", json!({})).unwrap(),
        json!({"recording": true, "tracks": [0]})
    );
    h.play();

    // 0,5 s de gravacao: tres valores distintos, cada um segurado por mais de um frame (30 fps).
    let esperados = [10u8, 20, 30];
    use protocols::Output;
    for v in esperados {
        let mut frame = [0u8; 512];
        frame[0] = v;
        for _ in 0..4 {
            tx.send(7, &frame);
            std::thread::sleep(Duration::from_millis(40));
        }
    }
    let chegou = espera(2.0, || gravados(&r, 0).len() >= 3);

    // em pausa o frame continua rodando com t congelado: NAO grava
    h.pause();
    std::thread::sleep(Duration::from_millis(100));
    let antes = gravados(&r, 0).len();
    let mut frame = [0u8; 512];
    frame[0] = 40;
    for _ in 0..4 {
        tx.send(7, &frame);
        std::thread::sleep(Duration::from_millis(40));
    }
    let em_pausa = gravados(&r, 0).len() - antes;

    // parar o transporte desarma
    h.stop();
    let desarmou = espera(1.0, || {
        r.call("rec_state", json!({})).unwrap()["recording"] == json!(false)
    });

    let keys = gravados(&r, 0);
    let entrada = h.input_get(7);
    tx.close();
    p.close();

    if entrada.is_none() {
        return println!("pulado: UDP em loopback nao entregou (firewall?)");
    }
    assert!(chegou, "gravou {} keyframes: {:?}", keys.len(), keys);
    // A entrada nasce em zero: o primeiro frame gravado pode ser o 0 antes do primeiro pacote.
    let vs: Vec<f64> = keys.into_iter().filter(|v| *v > 0.0).collect();
    assert_eq!(vs, vec![10.0, 20.0, 30.0], "valores gravados em ordem");
    assert_eq!(em_pausa, 0, "gravou em pausa");
    assert!(desarmou, "stop nao desarmou a gravacao");
}
