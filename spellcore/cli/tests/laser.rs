//! The ILDA player as the operator uses it: the `laser_*` commands of the registry, seen through
//! the MCP, against the Ether Dream `Emulator` of the `laser` crate itself.
//!
//! Its own binary, and the server in a separate process, because the `FEEDS` table is global to
//! the process (the same reason as `engine/tests/edit.rs`). The emulator stays HERE: that way the
//! test reads the points that reached the "DAC" and checks the geometry that went out.

use laser::{ild, Emulator, Frame, Point};
use serde_json::json;

use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

mod common;
use common::Mcp;

// ---------------------------------------------------------------------- helpers

/// A +-10000 ILDA unit square, lit: a bbox much larger than the safety `min_size`.
fn quadrado(nome: &str) -> Frame {
    let c = [
        (-10000.0, -10000.0),
        (10000.0, -10000.0),
        (10000.0, 10000.0),
        (-10000.0, 10000.0),
    ];
    let mut p: Vec<Point> = Vec::new();
    for (a, b) in c.iter().zip(c.iter().cycle().skip(1)) {
        for i in 0..25 {
            let k = i as f64 / 25.0;
            p.push(Point::new(
                a.0 + (b.0 - a.0) * k,
                a.1 + (b.1 - a.1) * k,
                255,
                255,
                255,
                false,
            ));
        }
    }
    Frame::new(p, nome)
}

/// Largest |x| of the lit points the emulator received starting at `desde`.
fn largura(emu: &Emulator, desde: usize) -> i32 {
    emu.points()
        .iter()
        .skip(desde)
        .filter(|p| (p.r | p.g | p.b) != 0)
        .map(|p| (p.x as i32).abs())
        .max()
        .unwrap_or(0)
}

fn espera<F: FnMut() -> bool>(mut f: F) -> bool {
    for _ in 0..100 {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

// ------------------------------------------------------------------------- test

#[test]
fn ilda_player_from_scan_to_close() {
    let emu = Emulator::start(1800).expect("Ether Dream emulator");
    let porta = emu.port;

    // emulator beacon on 127.0.0.1:7654 while the `laser_dacs` of the other process listens
    let run = Arc::new(AtomicBool::new(true));
    let (b, r) = (emu.beacon([0xaa, 0xbb, 0xcc, 0, 0, 1]), run.clone());
    let farol = std::thread::spawn(move || {
        let s = match UdpSocket::bind("127.0.0.1:0") {
            Ok(s) => s,
            Err(_) => return,
        };
        while r.load(Ordering::Relaxed) {
            let _ = s.send_to(&b, "127.0.0.1:7654");
            std::thread::sleep(Duration::from_millis(100));
        }
    });

    let dir = std::env::temp_dir().join("spellcore_laser_test");
    std::fs::create_dir_all(&dir).expect("test dir");
    let ild = dir.join("quadrado.ild");
    let frames: Vec<Frame> = (0..3).map(|i| quadrado(&format!("q{}", i))).collect();
    ild::write(&ild, &frames, 5, "test", "spell", None).expect("write the .ild");
    let arquivo = ild.to_string_lossy().to_string();

    let mut m = Mcp::start();

    // ---- laser_files finds the .ild that was written
    let f = m.cmd("laser_files", json!({"dir": dir.to_string_lossy()}));
    assert!(
        f["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["name"] == "quadrado.ild"),
        "laser_files did not list the .ild: {}",
        f
    );

    // ---- laser_dacs finds the emulator by its beacon
    let d = m.cmd("laser_dacs", json!({"timeout": 1.5}));
    let achou = d
        .as_array()
        .expect("laser_dacs returns a list")
        .iter()
        .any(|x| x["type"] == "etherdream" && x["id"] == "aa:bb:cc:00:00:01");
    run.store(false, Ordering::Relaxed);
    farol.join().ok();
    assert!(achou, "laser_dacs did not see the emulator beacon: {}", d);

    // ---- open, play, measure
    let o = m.cmd(
        "laser_open",
        json!({"dac": "etherdream", "host": format!("127.0.0.1:{}", porta), "kpps": 30}),
    );
    let feed = o["feed"].as_u64().expect("feed id");
    assert!(
        o["dac"].as_str().unwrap().starts_with("etherdream:"),
        "{}",
        o
    );

    let p = m.cmd(
        "laser_play",
        json!({"feed": feed, "file": arquivo, "fps": 60, "loop": true}),
    );
    assert_eq!(p["frames"], json!(3));

    assert!(espera(|| emu.count() > 0), "nothing reached the emulator");
    let s = m.cmd("laser_stats", json!({"feed": feed}));
    assert!(
        s["stat/sent"].as_u64().unwrap() > 0,
        "laser_stats with no delivered frames: {}",
        s
    );
    assert_eq!(s["playing"], json!(true));
    assert_eq!(s["stat/errors"], json!(0));

    let cheio = largura(&emu, 0);
    assert!(
        (9000..=10000).contains(&cheio),
        "the whole square gave {}",
        cheio
    );

    // ---- geo/scale 0.5 shrinks the bbox of what goes out
    let marca = emu.points().len();
    let r = m.cmd(
        "laser_param",
        json!({"feed": feed, "path": "geo/scale", "value": 0.5}),
    );
    assert_eq!(r["shutter"], json!(false));
    assert!(espera(|| largura(&emu, marca + 400) > 0));
    let meio = largura(&emu, marca + 400);
    assert!(
        (4000..=5100).contains(&meio),
        "with geo/scale 0.5 the bbox gave {} (it was {})",
        meio,
        cheio
    );

    // ---- the shutter blanks the output without stopping the transport
    m.cmd(
        "laser_param",
        json!({"feed": feed, "path": "shutter", "value": 1}),
    );
    let marca = emu.points().len();
    assert!(espera(|| emu.points().len() > marca + 400));
    let apagados = emu.points()[marca + 400..]
        .iter()
        .all(|p| (p.r | p.g | p.b) == 0);
    assert!(apagados, "the shutter is closed and color still went out");
    assert_eq!(
        m.cmd("laser_stats", json!({"feed": feed}))["shutter"],
        json!(true)
    );
    m.cmd(
        "laser_param",
        json!({"feed": feed, "path": "shutter", "value": 0}),
    );

    // ---- safe/*: the clamp is the one from modules/laser.json (min_size 0..32767,
    // max_intensity 0..255)
    m.cmd(
        "laser_param",
        json!({"feed": feed, "path": "safe/min_size", "value": 99999}),
    );
    m.cmd(
        "laser_param",
        json!({"feed": feed, "path": "safe/max_intensity", "value": 900}),
    );
    let st = m.cmd("laser_stats", json!({"feed": feed}));
    assert_eq!(st["safe/min_size"], json!(32767), "{}", st);
    assert_eq!(st["safe/max_intensity"], json!(255), "{}", st);
    m.cmd(
        "laser_param",
        json!({"feed": feed, "path": "safe/min_size", "value": 2000}),
    );

    // ---- an invalid path names the valid ones and touches nothing
    let e = m.erro(
        "laser_param",
        json!({"feed": feed, "path": "curve/r", "value": 1}),
    );
    assert!(
        e.contains("geo/scale") && e.contains("shutter"),
        "poor error: {}",
        e
    );

    // ---- with no loop, the end of the file disarms the transport by itself (bug 6 of the review)
    m.cmd(
        "laser_play",
        json!({"feed": feed, "file": arquivo, "fps": 60, "loop": false}),
    );
    assert!(
        espera(|| m.cmd("laser_stats", json!({"feed": feed}))["playing"] == json!(false)),
        "the file with no loop ended and laser_stats kept playing"
    );

    // ---- stop keeps the feed; close takes it away
    assert_eq!(
        m.cmd("laser_stop", json!({"feed": feed}))["playing"],
        json!(false)
    );
    assert_eq!(
        m.cmd("laser_stats", json!({"feed": feed}))["playing"],
        json!(false)
    );
    assert_eq!(
        m.cmd("laser_close", json!({"feed": feed}))["closed"],
        json!(true)
    );
    assert_eq!(
        m.erro("laser_stats", json!({"feed": feed})),
        format!("feed {} does not exist", feed)
    );

    std::fs::remove_file(&ild).ok();
}

// ------------------------------------------------------- clip_frame (previz)

/// The frame the timeline viewer draws: pick by `index` and by `t` at `fps`, and the points
/// already normalized to -1..1. An .ild of three frames, each with the point in a different
/// corner.
#[test]
fn clip_frame_picks_the_frame_and_normalizes() {
    let dir = std::env::temp_dir().join("spellcore_clipframe_test");
    std::fs::create_dir_all(&dir).expect("test dir");
    let ild = dir.join("tres.ild");
    let frames: Vec<Frame> = (0..3)
        .map(|i| {
            Frame::new(
                vec![Point::new(32767.0, -32767.0, 255, 128, 0, i == 2)],
                &format!("f{}", i),
            )
        })
        .collect();
    ild::write(&ild, &frames, 5, "test", "spell", None).expect("write the .ild");
    let arquivo = ild.to_string_lossy().to_string();

    let mut m = Mcp::start();

    // a direct index, and the clip repeats (index 4 of 3 frames = 1)
    let a = m.cmd("clip_frame", json!({"clip": arquivo, "index": 4}));
    assert_eq!(a["index"], json!(1));
    assert_eq!(a["frames"], json!(3));
    assert_eq!(a["name"], json!("f1"));

    // t at fps: floor(t * fps) % frames, the player's arithmetic
    for (t, fps, i) in [
        (0.0, 30.0, 0),
        (0.05, 30.0, 1),
        (0.07, 30.0, 2),
        (0.1, 30.0, 0),
        (0.3, 10.0, 0),
        (0.5, 10.0, 2),
    ] {
        let v = m.cmd("clip_frame", json!({"clip": arquivo, "t": t, "fps": fps}));
        assert_eq!(v["index"], json!(i), "t={} fps={}", t, fps);
    }

    // a point [x, y, r, g, b, blank] with x and y in -1..1
    let p = &m.cmd("clip_frame", json!({"clip": arquivo, "index": 2}))["points"][0];
    assert_eq!(p[0], json!(1.0));
    assert_eq!(p[1], json!(-1.0));
    assert_eq!(
        (p[2].as_u64(), p[3].as_u64(), p[4].as_u64()),
        (Some(255), Some(128), Some(0))
    );
    assert_eq!(p[5], json!(1), "blank of the third frame");

    assert!(m
        .erro("clip_frame", json!({"clip": "no_such_file.ild"}))
        .contains("no_such_file.ild"));

    std::fs::remove_file(&ild).ok();
}
