//! O ILDA player como o operador o usa: os comandos `laser_*` do registry, vistos pelo MCP,
//! contra o `Emulator` Ether Dream do proprio crate `laser`.
//!
//! Binario proprio, e o servidor num processo separado, porque a tabela `FEEDS` e' global ao
//! processo (mesma razao de `engine/tests/edit.rs`). O emulador fica AQUI: assim o teste le' os
//! pontos que chegaram ao "DAC" e confere a geometria que saiu.

use laser::{ild, Emulator, Frame, Point};
use serde_json::json;

use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

mod common;
use common::Mcp;

// ------------------------------------------------------------------- utilidades

/// Quadrado de +-10000 unidades ILDA, aceso: bbox bem maior que o `min_size` da safety.
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

/// Maior |x| dos pontos acesos que o emulador recebeu a partir de `desde`.
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

// ------------------------------------------------------------------------ teste

#[test]
fn ilda_player_do_scan_ao_close() {
    let emu = Emulator::start(1800).expect("emulador Ether Dream");
    let porta = emu.port;

    // beacon do emulador em 127.0.0.1:7654 enquanto o `laser_dacs` do outro processo escuta
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
    std::fs::create_dir_all(&dir).expect("dir do teste");
    let ild = dir.join("quadrado.ild");
    let frames: Vec<Frame> = (0..3).map(|i| quadrado(&format!("q{}", i))).collect();
    ild::write(&ild, &frames, 5, "teste", "spell", None).expect("gravar .ild");
    let arquivo = ild.to_string_lossy().to_string();

    let mut m = Mcp::start();

    // ---- laser_files acha o .ild gravado
    let f = m.cmd("laser_files", json!({"dir": dir.to_string_lossy()}));
    assert!(
        f["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["name"] == "quadrado.ild"),
        "laser_files nao listou o .ild: {}",
        f
    );

    // ---- laser_dacs acha o emulador pelo beacon
    let d = m.cmd("laser_dacs", json!({"timeout": 1.5}));
    let achou = d
        .as_array()
        .expect("laser_dacs devolve lista")
        .iter()
        .any(|x| x["type"] == "etherdream" && x["id"] == "aa:bb:cc:00:00:01");
    run.store(false, Ordering::Relaxed);
    farol.join().ok();
    assert!(achou, "laser_dacs nao viu o beacon do emulador: {}", d);

    // ---- abre, toca, mede
    let o = m.cmd(
        "laser_open",
        json!({"dac": "etherdream", "host": format!("127.0.0.1:{}", porta), "kpps": 30}),
    );
    let feed = o["feed"].as_u64().expect("id do feed");
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

    assert!(espera(|| emu.count() > 0), "nada chegou ao emulador");
    let s = m.cmd("laser_stats", json!({"feed": feed}));
    assert!(
        s["stat/sent"].as_u64().unwrap() > 0,
        "laser_stats sem frames entregues: {}",
        s
    );
    assert_eq!(s["playing"], json!(true));
    assert_eq!(s["stat/errors"], json!(0));

    let cheio = largura(&emu, 0);
    assert!(
        (9000..=10000).contains(&cheio),
        "quadrado inteiro deu {}",
        cheio
    );

    // ---- geo/scale 0.5 encolhe a bbox do que sai
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
        "com geo/scale 0.5 a bbox deu {} (era {})",
        meio,
        cheio
    );

    // ---- shutter apaga sem parar o transporte
    m.cmd(
        "laser_param",
        json!({"feed": feed, "path": "shutter", "value": 1}),
    );
    let marca = emu.points().len();
    assert!(espera(|| emu.points().len() > marca + 400));
    let apagados = emu.points()[marca + 400..]
        .iter()
        .all(|p| (p.r | p.g | p.b) == 0);
    assert!(apagados, "shutter fechado e ainda saiu cor");
    assert_eq!(
        m.cmd("laser_stats", json!({"feed": feed}))["shutter"],
        json!(true)
    );
    m.cmd(
        "laser_param",
        json!({"feed": feed, "path": "shutter", "value": 0}),
    );

    // ---- safe/*: o clamp e' o do modules/laser.json (min_size 0..32767, max_intensity 0..255)
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

    // ---- path invalido nomeia os validos e nao mexe em nada
    let e = m.erro(
        "laser_param",
        json!({"feed": feed, "path": "curve/r", "value": 1}),
    );
    assert!(
        e.contains("geo/scale") && e.contains("shutter"),
        "erro pobre: {}",
        e
    );

    // ---- sem loop, o fim do arquivo desarma o transporte sozinho (bug 6 da revisao)
    m.cmd(
        "laser_play",
        json!({"feed": feed, "file": arquivo, "fps": 60, "loop": false}),
    );
    assert!(
        espera(|| m.cmd("laser_stats", json!({"feed": feed}))["playing"] == json!(false)),
        "arquivo sem loop terminou e laser_stats seguiu playing"
    );

    // ---- stop mantem o feed; close some com ele
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
        format!("feed {} nao existe", feed)
    );

    std::fs::remove_file(&ild).ok();
}

// ------------------------------------------------------- clip_frame (previz)

/// O quadro que o viewer da timeline desenha: escolha por `index` e por `t` a `fps`, e os pontos
/// ja' normalizados em -1..1. Um .ild de tres quadros, cada um com o ponto num canto diferente.
#[test]
fn clip_frame_escolhe_o_quadro_e_normaliza() {
    let dir = std::env::temp_dir().join("spellcore_clipframe_test");
    std::fs::create_dir_all(&dir).expect("dir do teste");
    let ild = dir.join("tres.ild");
    let frames: Vec<Frame> = (0..3)
        .map(|i| {
            Frame::new(
                vec![Point::new(32767.0, -32767.0, 255, 128, 0, i == 2)],
                &format!("f{}", i),
            )
        })
        .collect();
    ild::write(&ild, &frames, 5, "teste", "spell", None).expect("gravar .ild");
    let arquivo = ild.to_string_lossy().to_string();

    let mut m = Mcp::start();

    // index direto, e o clipe repete (index 4 de 3 quadros = 1)
    let a = m.cmd("clip_frame", json!({"clip": arquivo, "index": 4}));
    assert_eq!(a["index"], json!(1));
    assert_eq!(a["frames"], json!(3));
    assert_eq!(a["name"], json!("f1"));

    // t a fps: floor(t * fps) % quadros, a conta do player
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

    // ponto [x, y, r, g, b, blank] com x e y em -1..1
    let p = &m.cmd("clip_frame", json!({"clip": arquivo, "index": 2}))["points"][0];
    assert_eq!(p[0], json!(1.0));
    assert_eq!(p[1], json!(-1.0));
    assert_eq!(
        (p[2].as_u64(), p[3].as_u64(), p[4].as_u64()),
        (Some(255), Some(128), Some(0))
    );
    assert_eq!(p[5], json!(1), "blank do terceiro quadro");

    assert!(m
        .erro("clip_frame", json!({"clip": "nao_existe.ild"}))
        .contains("nao_existe.ild"));

    std::fs::remove_file(&ild).ok();
}
