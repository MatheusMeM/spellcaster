//! DMX recording: an sACN output on 127.0.0.1 on universe 7 changes value, the player plays with
//! the track armed and the values become keyframes in the open show. Its own binary: `OPEN` (the
//! open show) and `CURRENT` (the live player) are process globals.
//!
//! All in loopback, with no hardware: `SacnOut` sends unicast to 127.0.0.1:5568 besides the
//! multicast, and it is over that unicast that the player `SacnIn` receives (the same trick as
//! the `protocols::sacn` tests).

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

/// Values of a track of the open show, in keyframe order.
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
fn armed_track_records_the_input_universe() {
    let cfg = json!({
        "name": "recording", "fps": 30, "version": 1, "outputs": [],
        "inputs": [{"type": "sacn", "universe": 7}],
        "tracks": [{"type": "dmx", "universe": 7, "address": 1, "keys": []}]
    });
    let r = engine::registry::base();
    // no player yet: `input_get` answers like every transport command
    assert_eq!(
        r.call("input_get", json!({"universe": 7})).unwrap_err(),
        "no player running"
    );
    assert_eq!(
        r.call("rec_state", json!({})).unwrap(),
        json!({"recording": false, "tracks": []})
    );
    r.call("show_set", json!({ "data": cfg }))
        .expect("show_set");

    let sh: Show = serde_json::from_value(cfg).expect("test show");
    let mut p = match Player::new(sh, false) {
        Ok(p) => p,
        Err(e) => return println!("skipped: sACN input did not come up: {}", e),
    };
    p.start(None).expect("start");
    let h = p.handle();

    let mut tx = match protocols::sacn::SacnOut::new(&[7], Some(vec!["127.0.0.1".parse().unwrap()]))
    {
        Ok(t) => t,
        Err(e) => {
            p.close();
            return println!("skipped: sACN output did not come up: {:?}", e.kind());
        }
    };

    r.call("rec_arm", json!({"track": 0, "on": true}))
        .expect("rec_arm");
    assert_eq!(
        r.call("rec_state", json!({})).unwrap(),
        json!({"recording": true, "tracks": [0]})
    );
    h.play();

    // 0.5 s of recording: three distinct values, each held for more than one frame (30 fps).
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

    // while paused the frame keeps running with t frozen: it does NOT record
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

    // stopping the transport disarms
    h.stop();
    let desarmou = espera(1.0, || {
        r.call("rec_state", json!({})).unwrap()["recording"] == json!(false)
    });

    let keys = gravados(&r, 0);
    let entrada = h.input_get(7);
    tx.close();
    p.close();

    if entrada.is_none() {
        return println!("skipped: loopback UDP did not deliver (firewall?)");
    }
    assert!(chegou, "recorded {} keyframes: {:?}", keys.len(), keys);
    // The input is born at zero: the first frame recorded may be the 0 before the first packet.
    let vs: Vec<f64> = keys.into_iter().filter(|v| *v > 0.0).collect();
    assert_eq!(vs, vec![10.0, 20.0, 30.0], "values recorded in order");
    assert_eq!(em_pausa, 0, "it recorded while paused");
    assert!(desarmou, "stop did not disarm the recording");
}
