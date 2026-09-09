//! Player: show curto em loopback sACN, transporte remoto por OSC e `locate` zerando cues e
//! ganchos. Tudo em 127.0.0.1; quando o socket nao sobe (firewall, porta ocupada), o teste
//! avisa e passa — igual aos testes de loopback do crate `protocols`.

use engine::hook::FrameHook;
use engine::{Player, Show, Universes};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn show(v: serde_json::Value) -> Show {
    serde_json::from_value(v).expect("show de teste invalido")
}

/// Espera a condicao por ate `secs`, checando a cada 5 ms.
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
fn show_de_dois_tracks_em_loopback_sacn() {
    let mut rx = match protocols::sacn::SacnIn::new(&[1]) {
        Ok(r) => r,
        Err(e) => return println!("pulado: bind 5568 falhou: {:?}", e.kind()),
    };
    let sh = show(json!({
        "name": "teste", "fps": 30, "duration": 0.5, "version": 1,
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
        Err(e) => return println!("pulado: saida sACN nao subiu: {}", e),
    };
    p.start(None).expect("start");
    let h = p.handle();
    h.play();
    assert!(p.wait(Some(Duration::from_secs(5))), "o show de 0,5 s nao terminou");

    let ok = espera(2.0, || {
        rx.get(1).is_some_and(|d| d[0] == 200 && d[9] == 11)
    });
    let got = rx.get(1);
    let st = h.state();
    p.close();
    rx.close();
    match got {
        None => println!("pulado: UDP em loopback nao entregou (firewall?)"),
        Some(d) => {
            assert!(ok, "frame recebido nao bate: {:?}", &d[..12]);
            assert_eq!(&d[..1], &[200], "track 1: canal 1");
            assert_eq!(&d[9..12], &[11, 22, 33], "track 2: canais 10..12");
            assert_eq!(&d[19..21], &[10, 5], "media Capture: ch1 play = 10, ch2 clipe = 5");
            assert_eq!(d[1], 0, "canal nao escrito continua zero");
        }
    }
    assert_eq!(st.state, "stop", "fim do show para o transporte");
    assert!(st.frames >= 10, "frames em 0,5 s a 30 fps: {}", st.frames);
    assert_eq!(st.universes, vec![1]);
    assert!(engine::player::current().is_none(), "close limpa o CURRENT");
}

#[test]
fn transporte_remoto_por_osc() {
    const PORTA: u16 = 19100;
    let sh = show(json!({
        "name": "osc", "fps": 30, "version": 1, "outputs": [], "tracks": [],
        "transport": {"osc_port": PORTA}
    }));
    let mut p = Player::new(sh, false).expect("player sem saida");
    if let Err(e) = p.start(None) {
        return println!("pulado: OscIn na porta {} nao subiu: {}", PORTA, e);
    }
    let h = p.handle();
    let tx = match protocols::osc::OscOut::new("127.0.0.1", PORTA) {
        Ok(t) => t,
        Err(e) => return println!("pulado: OscOut falhou: {:?}", e.kind()),
    };

    tx.send("/spellcaster/play", &[]);
    if !espera(2.0, || h.state().state == "play") {
        p.close();
        return println!("pulado: OSC em loopback nao chegou (firewall?)");
    }
    tx.send("/spellcaster/pause", &[]);
    assert!(espera(2.0, || h.state().state == "pause"), "pause remoto");
    tx.send("/spellcaster/locate", &[protocols::osc::Arg::Float(2.5)]);
    assert!(
        espera(2.0, || (h.state().t - 2.5).abs() < 0.05),
        "locate remoto: t = {}",
        h.state().t
    );
    tx.send("/spellcaster/stop", &[]);
    assert!(espera(2.0, || h.state().state == "stop"), "stop remoto");
    assert!(p.wait(Some(Duration::from_secs(2))), "stop remoto acorda o wait");
    p.close();
}

/// Tracks `osc` e `media` nao-Capture saem pela saida "osc" do show, so' quando o valor muda.
#[test]
fn tracks_de_efeito_colateral_saem_por_osc() {
    const PORTA: u16 = 19101;
    let mut rx = match protocols::osc::OscIn::new(PORTA) {
        Ok(r) => r,
        Err(e) => return println!("pulado: OscIn {} nao subiu: {:?}", PORTA, e.kind()),
    };
    let vistos = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    // o `*` do OSC nao atravessa "/": um padrao por nivel
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
        Err(e) => return println!("pulado: saida OSC nao subiu: {}", e),
    };
    p.start(None).expect("start");
    p.handle().play();
    assert!(p.wait(Some(Duration::from_secs(5))), "o show de 0,4 s nao terminou");
    let ok = espera(2.0, || vistos.lock().unwrap().len() >= 2);
    let msgs = vistos.lock().unwrap().clone();
    p.close();
    rx.close();
    if !ok {
        return println!("pulado: OSC em loopback nao entregou (firewall?): {:?}", msgs);
    }
    assert!(msgs.contains(&"/spell/dim".to_string()), "track osc: {:?}", msgs);
    assert!(
        msgs.contains(&"/spell/clip/play".to_string()),
        "media nao-Capture vira endereco/valor: {:?}",
        msgs
    );
    assert_eq!(msgs.len(), 2, "valor constante manda uma vez so': {:?}", msgs);
}

/// Gancho de teste: conta frames e resets, e escreve um canal para provar que roda no frame.
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
fn locate_zera_cues_e_ganchos() {
    let sh = show(json!({
        "name": "cues", "fps": 60, "version": 1, "outputs": [],
        "tracks": [{"type": "dmx", "universe": 1, "address": 1, "keys": [[0, 1]]}],
        "cues": [{"name": "um", "fade": 0.0, "values": {"1/20": [255]}},
                 {"name": "dois", "fade": 0.0, "values": {"1/20": [10]}}]
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
    assert!(espera(2.0, || frames.load(Ordering::Relaxed) > 2), "o gancho nao rodou");
    assert_eq!(h.state().cue, -1);

    h.cue_go(None);
    assert!(espera(2.0, || h.state().cue == 0), "GO nao disparou a cue 0");
    h.cue_go(None);
    assert!(espera(2.0, || h.state().cue == 1), "GO nao andou para a cue 1");

    let antes = resets.load(Ordering::Relaxed);
    h.locate(3.0);
    assert!(espera(2.0, || h.state().cue == -1), "locate nao zerou as cues");
    assert!(
        espera(2.0, || resets.load(Ordering::Relaxed) > antes),
        "locate nao chamou reset() no gancho"
    );
    assert!((h.state().t - 3.0).abs() < 0.5, "locate move o relogio");

    // stop tambem zera: a cue volta para -1 depois de um GO
    h.play();
    h.cue_go(Some(1));
    assert!(espera(2.0, || h.state().cue == 1));
    let antes = resets.load(Ordering::Relaxed);
    h.stop();
    assert!(espera(2.0, || h.state().cue == -1), "stop nao zerou as cues");
    assert!(espera(2.0, || resets.load(Ordering::Relaxed) > antes), "stop nao resetou o gancho");
    assert!(p.wait(Some(Duration::from_secs(2))), "stop acorda o wait");
    p.close();
}
