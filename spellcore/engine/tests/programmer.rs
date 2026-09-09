//! Programmer (a camada manual do operador) num player vivo: `level_set` por cima da timeline
//! E da cue viva com HTP, `level_clear` devolvendo o canal, `cue_capture` virando cue e
//! `fixture_set` resolvendo nome de fixture + nome de canal do perfil. Binario proprio porque
//! mexe no `CURRENT` do player e no `OPEN` do registry, que sao globais do processo.
//!
//! Show sem `outputs`: nada de socket, o teste roda em maquina com firewall fechado.

use engine::hook::FrameHook;
use engine::registry::{base, Registry};
use engine::{Player, Show, Universes};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Gancho de frame que guarda o universo 1 como saiu (timeline + programmer).
struct Espelho(Arc<Mutex<[u8; 512]>>);

impl FrameHook for Espelho {
    fn frame(&mut self, _t: f64, uni: &mut Universes) {
        if let Some(u) = uni.get(1) {
            *self.0.lock().expect("espelho") = u.data;
        }
    }
}

fn lock(m: &Arc<Mutex<[u8; 512]>>) -> [u8; 512] {
    *m.lock().expect("espelho")
}

/// Espera ate `secs` por um frame que satisfaca `cond`; devolve o ultimo visto.
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

/// Um teste so': `CURRENT` (player) e `OPEN` (show do registry) sao globais do processo, e os
/// testes de um binario rodam em paralelo.
#[test]
fn programmer_htp_clear_captura_e_fixture_set() {
    let r = base();
    // sem player, todo comando do programmer devolve o erro padrao
    for (c, a) in [
        ("level_set", json!({"address": 1, "values": [10]})),
        ("level_clear", json!({})),
        ("level_get", json!({})),
        ("cue_capture", json!({})),
    ] {
        assert_eq!(r.call(c, a).unwrap_err(), "sem player em execucao", "{}", c);
    }

    let sh: Show = serde_json::from_value(json!({
        "name": "programmer", "fps": 60, "version": 1, "outputs": [],
        "tracks": [{"type": "dmx", "universe": 1, "address": 1, "keys": [[0, 100], [60, 100]]}],
        "cues": [{"name": "viva", "fade": 0, "values": {"1/30": [60]}}]
    }))
    .expect("show de teste");
    let esp = Arc::new(Mutex::new([0u8; 512]));
    let mut p = Player::new(sh, PathBuf::from("."), false).expect("player sem saidas");
    p.hook(Box::new(Espelho(esp.clone())));
    p.start(None).expect("start");
    p.handle().play();

    let d = espera(&esp, 2.0, |d| d[0] == 100);
    assert_eq!(d[0], 100, "a timeline nao chegou a escrever o canal 1");

    // 1. override acima da timeline: o programmer ganha
    assert_eq!(
        ok(&r, "level_set", json!({"address": 1, "values": [200]})),
        json!(1)
    );
    let d = espera(&esp, 2.0, |d| d[0] == 200);
    assert_eq!(d[0], 200, "level_set nao subiu o canal 1");

    // 2. HTP: override abaixo da timeline nao derruba o canal
    ok(&r, "level_set", json!({"address": 1, "values": [50]}));
    let d = espera(&esp, 0.3, |_| false);
    assert_eq!(d[0], 100, "HTP: 50 do programmer sob 100 da timeline");

    // 3. varios canais de uma vez, num endereco que a timeline nao toca
    ok(
        &r,
        "level_set",
        json!({"universe": 1, "address": 5, "values": [11, 22, 33]}),
    );
    let d = espera(&esp, 2.0, |d| d[4] == 11);
    assert_eq!(&d[4..7], &[11, 22, 33], "level_set com values");

    // 4. level_get no formato de values de cue
    assert_eq!(
        ok(&r, "level_get", json!({})),
        json!({"1/1": 50, "1/5": 11, "1/6": 22, "1/7": 33})
    );

    // 5. captura: vira cue no show aberto e solta o override
    let i = ok(&r, "cue_capture", json!({"name": "cena 1", "fade": 2.0}));
    assert_eq!(i, json!(0));
    let cue = &ok(&r, "show_get", json!({"full": true}))["cues"][0];
    assert_eq!(cue["name"], "cena 1");
    assert_eq!(cue["fade"], json!(2.0));
    assert_eq!(
        cue["values"],
        json!({"1/1": 50, "1/5": 11, "1/6": 22, "1/7": 33})
    );
    assert_eq!(
        ok(&r, "level_get", json!({})),
        json!({}),
        "captura solta o override"
    );

    // 6. soltar o override devolve o canal: a timeline volta a mandar, o resto zera
    let d = espera(&esp, 2.0, |d| d[4] == 0);
    assert_eq!(d[0], 100, "canal da timeline volta ao valor dela");
    assert_eq!(&d[4..7], &[0, 0, 0], "canal so' do programmer volta a zero");

    // 7. level_clear devolve quantos canais soltou
    ok(&r, "level_set", json!({"address": 20, "values": [1, 2]}));
    assert_eq!(ok(&r, "level_clear", json!({"universe": 1})), json!(2));
    assert_eq!(ok(&r, "level_clear", json!({})), json!(0));
    assert!(
        r.call("cue_capture", json!({})).is_err(),
        "programmer vazio nao vira cue"
    );

    // 8. fixture_set: nome da fixture + nome do canal do perfil viram universo/endereco
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
        "offset 1 do perfil RGB sobre o endereco 10"
    );
    let d = espera(&esp, 2.0, |d| d[10] == 180);
    assert_eq!(&d[9..12], &[0, 180, 0]);

    // canal so' por nome: numero nao resolve, e o erro explica o que o perfil tem
    assert!(r
        .call(
            "fixture_set",
            json!({"name": "par 1", "channel": "2", "value": 90})
        )
        .unwrap_err()
        .contains("nao tem canal"));
    let e = r
        .call(
            "fixture_set",
            json!({"name": "par 1", "channel": "tilt", "value": 1}),
        )
        .unwrap_err();
    assert!(
        e.contains("\"tilt\"") && e.contains("\"r\""),
        "erro sem os canais do perfil: {}",
        e
    );
    assert!(r
        .call(
            "fixture_set",
            json!({"name": "nao existe", "channel": "r", "value": 1})
        )
        .unwrap_err()
        .contains("nao esta no patch"));

    // 9. o operador sobrepoe a cue viva: o programmer roda DEPOIS de `CueList::update`.
    // HTP, entao o valor do operador tem que ser maior que o da cue para vencer.
    p.handle().cue_go(Some(0));
    let d = espera(&esp, 2.0, |d| d[29] == 60);
    assert_eq!(d[29], 60, "a cue nao chegou a escrever o canal 30");
    ok(&r, "level_set", json!({"address": 30, "values": [200]}));
    let d = espera(&esp, 2.0, |d| d[29] == 200);
    assert_eq!(d[29], 200, "cue viva reescreveu o canal do operador");

    p.close();
}
