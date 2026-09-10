//! Player: short show over sACN loopback, remote transport over OSC and `locate` clearing cues
//! and hooks. All on 127.0.0.1; when the socket does not come up (firewall, busy port), the test
//! reports it and passes — same as the loopback tests of the `protocols` crate.

use engine::hook::FrameHook;
use engine::{Player, Show, Universes};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn show(v: serde_json::Value) -> Show {
    serde_json::from_value(v).expect("invalid test show")
}

/// Waits for the condition for up to `secs`, checking every 5 ms.
fn espera(secs: f64, mut cond: impl FnMut() -> bool) -> bool {
    let fim = Instant::now() + Duration::from_secs_f64(secs);
    while Instant::now() < fim {
        if cond() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    cond()
}

#[test]
fn two_track_show_over_sacn_loopback() {
    let mut rx = match protocols::sacn::SacnIn::new(&[1]) {
        Ok(r) => r,
        Err(e) => return println!("skipped: bind 5568 failed: {:?}", e.kind()),
    };
    let sh = show(json!({
        "name": "test", "fps": 30, "duration": 0.5, "version": 1,
        "outputs": [{"type": "sacn", "universes": [1], "interfaces": ["127.0.0.1"]}],
        "tracks": [
            {"type": "dmx", "universe": 1, "address": 1, "keys": [[0, 200], [9, 200]]},
            {"type": "dmx", "universe": 1, "address": 10,
             "keys": [[0, [11, 22, 33]], [9, [11, 22, 33]]]},
            {"type": "media", "universe": 1, "address": 20, "clip": 5, "keys": [[0, "play"]]}
        ]
    }));
    let mut p = match Player::new(sh, false) {
        Ok(p) => p,
        Err(e) => return println!("skipped: sACN output did not come up: {}", e),
    };
    p.start(None).expect("start");
    let h = p.handle();
    h.play();
    assert!(
        p.wait(Some(Duration::from_secs(5))),
        "the 0.5 s show did not finish"
    );

    let ok = espera(2.0, || rx.get(1).is_some_and(|d| d[0] == 200 && d[9] == 11));
    let got = rx.get(1);
    let st = h.state();
    p.close();
    rx.close();
    match got {
        None => println!("skipped: loopback UDP did not deliver (firewall?)"),
        Some(d) => {
            assert!(ok, "the received frame does not match: {:?}", &d[..12]);
            assert_eq!(&d[..1], &[200], "track 1: channel 1");
            assert_eq!(&d[9..12], &[11, 22, 33], "track 2: channels 10..12");
            assert_eq!(
                &d[19..21],
                &[10, 5],
                "Capture media: ch1 play = 10, ch2 clip = 5"
            );
            assert_eq!(d[1], 0, "an unwritten channel stays zero");
        }
    }
    assert_eq!(st.state, "stop", "the end of the show stops the transport");
    assert!(st.frames >= 10, "frames in 0.5 s at 30 fps: {}", st.frames);
    assert_eq!(st.universes, vec![1]);
    assert!(
        engine::player::current().is_none(),
        "close clears the CURRENT"
    );
}

#[test]
fn remote_transport_over_osc() {
    const PORTA: u16 = 19100;
    let sh = show(json!({
        "name": "osc", "fps": 30, "version": 1, "outputs": [], "tracks": [],
        "transport": {"osc_port": PORTA}
    }));
    let mut p = Player::new(sh, false).expect("player with no output");
    if let Err(e) = p.start(None) {
        return println!("skipped: OscIn on port {} did not come up: {}", PORTA, e);
    }
    let h = p.handle();
    let tx = match protocols::osc::OscOut::new("127.0.0.1", PORTA) {
        Ok(t) => t,
        Err(e) => return println!("skipped: OscOut failed: {:?}", e.kind()),
    };

    tx.send("/spellcaster/play", &[]);
    if !espera(2.0, || h.state().state == "play") {
        p.close();
        return println!("skipped: loopback OSC did not arrive (firewall?)");
    }
    tx.send("/spellcaster/pause", &[]);
    assert!(espera(2.0, || h.state().state == "pause"), "remote pause");
    tx.send("/spellcaster/locate", &[protocols::osc::Arg::Float(2.5)]);
    assert!(
        espera(2.0, || (h.state().t - 2.5).abs() < 0.05),
        "remote locate: t = {}",
        h.state().t
    );
    tx.send("/spellcaster/stop", &[]);
    assert!(espera(2.0, || h.state().state == "stop"), "remote stop");
    assert!(
        p.wait(Some(Duration::from_secs(2))),
        "a remote stop wakes the wait"
    );
    p.close();
}

/// `osc` tracks and non-Capture `media` go out through the show "osc" output, only when the
/// value changes.
#[test]
fn side_effect_tracks_go_out_over_osc() {
    const PORTA: u16 = 19101;
    let mut rx = match protocols::osc::OscIn::new(PORTA) {
        Ok(r) => r,
        Err(e) => return println!("skipped: OscIn {} did not come up: {:?}", PORTA, e.kind()),
    };
    let vistos = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    // the OSC `*` does not cross "/": one pattern per level
    for pat in ["/spell/*", "/spell/clip/*"] {
        let v = vistos.clone();
        rx.on(pat, move |a, _g| {
            v.lock().unwrap().push(a.to_string());
        });
    }

    let sh = show(json!({
        "name": "osc-out", "fps": 30, "duration": 0.4, "version": 1,
        "outputs": [{"type": "osc", "host": "127.0.0.1", "port": PORTA}],
        "tracks": [
            {"type": "osc", "address": "/spell/dim", "keys": [[0, 1], [10, 1]]},
            {"type": "media", "player": "resolume", "address": "/spell/clip/",
             "keys": [[0, "play"]]}
        ]
    }));
    let mut p = match Player::new(sh, false) {
        Ok(p) => p,
        Err(e) => return println!("skipped: OSC output did not come up: {}", e),
    };
    p.start(None).expect("start");
    p.handle().play();
    assert!(
        p.wait(Some(Duration::from_secs(5))),
        "the 0.4 s show did not finish"
    );
    let ok = espera(2.0, || vistos.lock().unwrap().len() >= 2);
    let msgs = vistos.lock().unwrap().clone();
    p.close();
    rx.close();
    if !ok {
        return println!(
            "skipped: loopback OSC did not deliver (firewall?): {:?}",
            msgs
        );
    }
    assert!(
        msgs.contains(&"/spell/dim".to_string()),
        "osc track: {:?}",
        msgs
    );
    assert!(
        msgs.contains(&"/spell/clip/play".to_string()),
        "non-Capture media becomes address/value: {:?}",
        msgs
    );
    assert_eq!(
        msgs.len(),
        2,
        "a constant value is sent just once: {:?}",
        msgs
    );
}

/// Test hook: it counts frames and resets, and writes a channel to prove it runs in the frame.
struct Conta {
    frames: Arc<AtomicUsize>,
    resets: Arc<AtomicUsize>,
}

impl FrameHook for Conta {
    fn frame(&mut self, _t: f64, uni: &mut Universes) {
        self.frames.fetch_add(1, Ordering::Relaxed);
        uni.get_or_create(1).set(100, &[7.0]);
    }

    fn reset(&mut self, _t: f64) {
        self.resets.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn locate_clears_cues_and_hooks() {
    let sh = show(json!({
        "name": "cues", "fps": 60, "version": 1, "outputs": [],
        "tracks": [{"type": "dmx", "universe": 1, "address": 1, "keys": [[0, 1]]}],
        "cues": [{"name": "one", "fade": 0.0, "values": {"1/20": [255]}},
                 {"name": "two", "fade": 0.0, "values": {"1/20": [10]}}]
    }));
    let (frames, resets) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let mut p = Player::new(sh, false).expect("player");
    p.hook(Box::new(Conta {
        frames: frames.clone(),
        resets: resets.clone(),
    }));
    p.start(None).expect("start");
    let h = p.handle();
    h.play();
    assert!(
        espera(2.0, || frames.load(Ordering::Relaxed) > 2),
        "the hook did not run"
    );
    assert_eq!(h.state().cue, -1);

    h.cue_go(None);
    assert!(espera(2.0, || h.state().cue == 0), "GO did not fire cue 0");
    h.cue_go(None);
    assert!(
        espera(2.0, || h.state().cue == 1),
        "GO did not move on to cue 1"
    );

    let antes = resets.load(Ordering::Relaxed);
    h.locate(3.0);
    assert!(
        espera(2.0, || h.state().cue == -1),
        "locate did not clear the cues"
    );
    assert!(
        espera(2.0, || resets.load(Ordering::Relaxed) > antes),
        "locate did not call reset() on the hook"
    );
    assert!((h.state().t - 3.0).abs() < 0.5, "locate moves the clock");

    // stop clears it too: the cue goes back to -1 after a GO
    h.play();
    h.cue_go(Some(1));
    assert!(espera(2.0, || h.state().cue == 1));
    let antes = resets.load(Ordering::Relaxed);
    h.stop();
    assert!(
        espera(2.0, || h.state().cue == -1),
        "stop did not clear the cues"
    );
    assert!(
        espera(2.0, || resets.load(Ordering::Relaxed) > antes),
        "stop did not reset the hook"
    );
    assert!(p.wait(Some(Duration::from_secs(2))), "stop wakes the wait");
    p.close();
}

/// The loop is ENGINE state, over the In-Out range of the show: with In=1 and Out=2 the
/// transport jumps back to the In on its own, and turning the loop off with the player running
/// releases the time in the same second.
#[test]
fn loop_repeats_the_in_out_range() {
    let sh = show(json!({
        "name": "loop", "fps": 60, "duration": 10.0, "version": 1,
        "outputs": [], "tracks": [], "in": 1.0, "out": 2.0
    }));
    let mut p = Player::new(sh, true).expect("player with no output");
    p.start(None).expect("start");
    let h = p.handle();
    let st = h.state();
    assert!(
        st.looping,
        "the player was born with the loop asked for in play_show"
    );
    assert_eq!(
        (st.loop_in, st.loop_out),
        (1.0, 2.0),
        "the loop range is the In-Out of the show"
    );

    h.locate(1.5);
    h.play();
    let voltou = espera(4.0, || h.state().t < 1.4);
    let t = h.state().t;
    assert!(
        voltou,
        "the transport did not jump back to the In: t = {}",
        t
    );
    assert!(t > 0.9, "it jumped back to before the In: t = {}", t);

    // turned off with the player running (the registry `loop_set`), time runs past the Out
    h.set_loop(false, 1.0, 2.0);
    let passou = espera(4.0, || h.state().t > 2.3);
    let t = h.state().t;
    p.close();
    assert!(
        passou,
        "the loop turned off still held the transport at {}",
        t
    );
}
