//! Programmer (the operator manual layer) on a live player: `level_set` on top of the timeline
//! AND of the live cue with HTP, `level_clear` handing the channel back, `cue_capture` becoming a
//! cue and `fixture_set` resolving fixture name + profile channel name. Its own binary because it
//! touches the player `CURRENT` and the registry `OPEN`, which are process globals.
//!
//! What the test watches is the frame OUTPUT, not the buffer halfway through it: `Espelho` is a
//! test `Output`, the last step of the tick. A show with no network `outputs`: no socket, the
//! test runs on a machine with the firewall shut.

use engine::registry::{base, Registry};
use engine::{Player, Show};
use protocols::Output;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Test output that keeps universe 1 as it was sent (timeline + cues + programmer).
struct Espelho(Arc<Mutex<[u8; 512]>>);

impl Output for Espelho {
    fn send(&mut self, universe: u16, data: &[u8; 512]) {
        if universe == 1 {
            *self.0.lock().expect("mirror") = *data;
        }
    }

    fn close(&mut self) {}
}

fn lock(m: &Arc<Mutex<[u8; 512]>>) -> [u8; 512] {
    *m.lock().expect("mirror")
}

/// Waits up to `secs` for a frame that satisfies `cond`; returns the last one seen.
fn espera(m: &Arc<Mutex<[u8; 512]>>, secs: f64, cond: impl Fn(&[u8; 512]) -> bool) -> [u8; 512] {
    let fim = Instant::now() + Duration::from_secs_f64(secs);
    loop {
        let d = lock(m);
        if cond(&d) || Instant::now() >= fim {
            return d;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn ok(r: &Registry, cmd: &str, args: Value) -> Value {
    r.call(cmd, args)
        .unwrap_or_else(|e| panic!("{}: {}", cmd, e))
}

/// A single test: `CURRENT` (player) and `OPEN` (registry show) are process globals, and the
/// tests of one binary run in parallel.
#[test]
fn programmer_htp_clear_capture_and_fixture_set() {
    let r = base();
    // with no player, every programmer command returns the standard error
    for (c, a) in [
        ("level_set", json!({"address": 1, "values": [10]})),
        ("level_clear", json!({})),
        ("level_get", json!({})),
        ("cue_capture", json!({})),
    ] {
        assert_eq!(r.call(c, a).unwrap_err(), "no player running", "{}", c);
    }

    let sh: Show = serde_json::from_value(json!({
        "name": "programmer", "fps": 60, "version": 1, "outputs": [],
        "tracks": [{"type": "dmx", "universe": 1, "address": 1, "keys": [[0, 100], [60, 100]]}],
        "cues": [{"name": "live", "fade": 0, "values": {"1/30": [60]}}]
    }))
    .expect("test show");
    let esp = Arc::new(Mutex::new([0u8; 512]));
    let mut p = Player::new(sh, false).expect("player with no outputs");
    p.output(Box::new(Espelho(esp.clone())));
    p.start(None).expect("start");
    p.handle().play();

    let d = espera(&esp, 2.0, |d| d[0] == 100);
    assert_eq!(d[0], 100, "the timeline never got to write channel 1");

    // 1. override above the timeline: the programmer wins
    assert_eq!(
        ok(&r, "level_set", json!({"address": 1, "values": [200]})),
        json!(1)
    );
    let d = espera(&esp, 2.0, |d| d[0] == 200);
    assert_eq!(d[0], 200, "level_set did not raise channel 1");

    // an address outside 1..512 is an error, not silence
    for a in [0, 513] {
        let e = r
            .call("level_set", json!({"address": a, "values": [1]}))
            .expect_err("an out-of-range address has to be refused");
        assert!(e.contains("outside 1..512"), "{}", e);
    }

    // 2. HTP: an override below the timeline does not pull the channel down
    ok(&r, "level_set", json!({"address": 1, "values": [50]}));
    let d = espera(&esp, 0.3, |_| false);
    assert_eq!(
        d[0], 100,
        "HTP: 50 from the programmer under 100 from the timeline"
    );

    // 3. several channels at once, at an address the timeline does not touch
    ok(
        &r,
        "level_set",
        json!({"universe": 1, "address": 5, "values": [11, 22, 33]}),
    );
    let d = espera(&esp, 2.0, |d| d[4] == 11);
    assert_eq!(&d[4..7], &[11, 22, 33], "level_set with values");

    // 4. level_get in the cue values format
    assert_eq!(
        ok(&r, "level_get", json!({})),
        json!({"1/1": 50, "1/5": 11, "1/6": 22, "1/7": 33})
    );

    // 5. capture: it becomes a cue in the open show and releases the override
    let i = ok(&r, "cue_capture", json!({"name": "scene 1", "fade": 2.0}));
    assert_eq!(i, json!(0));
    let cue = &ok(&r, "show_get", json!({"full": true}))["cues"][0];
    assert_eq!(cue["name"], "scene 1");
    assert_eq!(cue["fade"], json!(2.0));
    assert_eq!(
        cue["values"],
        json!({"1/1": 50, "1/5": 11, "1/6": 22, "1/7": 33})
    );
    assert_eq!(
        ok(&r, "level_get", json!({})),
        json!({}),
        "the capture releases the override"
    );

    // 6. releasing the override hands the channel back: the timeline rules again, the rest zeroes
    let d = espera(&esp, 2.0, |d| d[4] == 0);
    assert_eq!(d[0], 100, "a timeline channel goes back to its value");
    assert_eq!(
        &d[4..7],
        &[0, 0, 0],
        "a programmer-only channel goes back to zero"
    );

    // 7. level_clear returns how many channels it released
    ok(&r, "level_set", json!({"address": 20, "values": [1, 2]}));
    assert_eq!(ok(&r, "level_clear", json!({"universe": 1})), json!(2));
    assert_eq!(ok(&r, "level_clear", json!({})), json!(0));
    assert!(
        r.call("cue_capture", json!({})).is_err(),
        "an empty programmer does not become a cue"
    );

    // 8. fixture_set: fixture name + profile channel name become universe/address
    let prof = concat!(env!("CARGO_MANIFEST_DIR"), "/../../profiles/par_rgb_3.json");
    ok(&r, "show_new", json!({}));
    ok(
        &r,
        "patch_add",
        json!({"name": "par 1", "profile": prof, "universe": 1, "address": 10}),
    );

    assert_eq!(
        ok(
            &r,
            "fixture_set",
            json!({"name": "par 1", "channel": "g", "value": 180})
        ),
        json!({"universe": 1, "address": 11, "value": 180.0}),
        "offset 1 of the RGB profile over address 10"
    );
    let d = espera(&esp, 2.0, |d| d[10] == 180);
    assert_eq!(&d[9..12], &[0, 180, 0]);

    // channel by name only: a number does not resolve, and the error explains what the profile has
    assert!(r
        .call(
            "fixture_set",
            json!({"name": "par 1", "channel": "2", "value": 90})
        )
        .unwrap_err()
        .contains("has no channel"));
    let e = r
        .call(
            "fixture_set",
            json!({"name": "par 1", "channel": "tilt", "value": 1}),
        )
        .unwrap_err();
    assert!(
        e.contains("\"tilt\"") && e.contains("\"r\""),
        "error without the profile channels: {}",
        e
    );
    assert!(r
        .call(
            "fixture_set",
            json!({"name": "does not exist", "channel": "r", "value": 1})
        )
        .unwrap_err()
        .contains("is not in the patch"));

    // 9. the operator overrides the live cue: the programmer runs AFTER `CueList::update`.
    // HTP, so the operator value has to be higher than the cue one to win.
    p.handle().cue_go(Some(0));
    let d = espera(&esp, 2.0, |d| d[29] == 60);
    assert_eq!(d[29], 60, "the cue never got to write channel 30");
    ok(&r, "level_set", json!({"address": 30, "values": [200]}));
    let d = espera(&esp, 2.0, |d| d[29] == 200);
    assert_eq!(d[29], 200, "the live cue rewrote the operator channel");

    p.close();
}
