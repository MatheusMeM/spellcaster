//! MIDI mapping: uma porta de entrada aberta, um mapa `tecla -> comando` que vive no `.spell`,
//! e o evento entregue ao graph pela MESMA fila do comando `input`.
//!
//! Chave do evento: `"<status>/<data1>"` (`144/60` = note on canal 1 nota 60, `176/1` = CC 1),
//! igual a' do no' `in.midi` do graph. Cada evento recebido:
//!
//!   1. vira `input {key: "midi:<chave>", value}` nos ganchos do player vivo (o `in.midi`);
//!   2. se a chave estiver no mapa, chama o comando pelo registry — o mesmo caminho do WS.
//!
//! `value` e' `data2 / 127` (0..1). Nos `args` do mapa, `"$"` vira esse valor e `"$<n>"` vira
//! `round(valor * n)`: `"$127"` e' o byte cru do MIDI e `"$255"` e' o nivel DMX.
//!
//! Quem drena a fila e' o `pump()`, chamado no lugar do frame onde o `input` ja' e' consumido
//! (`player::Rt::drain`) — a ordem do frame nao muda. `midi_last` e `midi_learn` tambem drenam,
//! para o LEARN da GUI funcionar sem show tocando.

use crate::registry::{lock, NoArgs, Registry, OPEN};
use crate::show::Show;
use protocols::midi::MidiIn;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// A porta aberta neste processo.
// ponytail: uma porta por processo ; virar tabela como a `FEEDS` do laser quando alguem ligar
// teclado e surface ao mesmo tempo (a chave nao distingue a porta).
static IN: Mutex<Option<MidiIn>> = Mutex::new(None);

/// Ultimo evento recebido: (chave, valor 0..1, byte cru). E' o que o LEARN da GUI le.
static LAST: Mutex<Option<(String, f64, u8)>> = Mutex::new(None);

/// Conta eventos: `midi_learn` espera este numero mudar em vez de disputar a fila com o frame.
static SEQ: AtomicU64 = AtomicU64::new(0);

/// Segundos que o `midi_learn` espera pela proxima tecla.
const LEARN_MS: u64 = 5000;

// ------------------------------------------------------------------- registry do mapa

/// Como montar o registry que o mapa chama. A CLI instala o dela (com `play_show`, `net` e os
/// `laser_*`); sem instalacao, `registry::base()`.
static BUILD: Mutex<Option<fn() -> Registry>> = Mutex::new(None);

pub fn builder(f: fn() -> Registry) {
    *lock(&BUILD) = Some(f);
}

fn reg() -> &'static Registry {
    static R: OnceLock<Registry> = OnceLock::new();
    R.get_or_init(|| match *lock(&BUILD) {
        Some(f) => f(),
        None => crate::registry::base(),
    })
}

// ------------------------------------------------------------------------ mapa

/// O mapa do show aberto, ou vazio.
fn mapa() -> Map<String, Value> {
    lock(&OPEN)
        .as_ref()
        .and_then(|(_, sh)| sh.extra.get("midi"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

/// O que a chave dispara: (comando, args crus). Le e SOLTA o show antes de qualquer chamada —
/// o comando do mapa pode ser um que edita o show. Publica para `tests/midi.rs` provar que o
/// que `midi_map` grava e' o que o evento le.
pub fn liga(key: &str) -> Option<(String, Value)> {
    let e = lock(&OPEN)
        .as_ref()
        .and_then(|(_, sh)| sh.extra.get("midi"))
        .and_then(|m| m.get(key))
        .cloned()?;
    let cmd = e.get("cmd").and_then(Value::as_str)?.to_string();
    let args = e.get("args").cloned().unwrap_or_else(|| json!({}));
    Some((cmd, args))
}

/// `"$"` vira o valor 0..1; `"$<n>"` vira `round(valor * n)` (`"$127"` = o byte cru do MIDI,
/// `"$255"` = nivel DMX). Recursivo: vale dentro de lista e de objeto.
///
/// `"$<n>"` sai INTEIRO, nao float: `address` e `index` de comando sao `u16`/`usize`, e o serde
/// recusa `100.0` onde espera inteiro (float entra em `f64` sem reclamar, o contrario nao).
pub fn expande(v: &Value, valor: f64) -> Value {
    match v {
        Value::String(s) => match s.strip_prefix('$') {
            Some("") => json!(valor),
            Some(n) => match n.parse::<f64>() {
                Ok(n) => json!((valor * n).round() as i64),
                Err(_) => v.clone(),
            },
            None => v.clone(),
        },
        Value::Array(a) => Value::Array(a.iter().map(|x| expande(x, valor)).collect()),
        Value::Object(o) => Value::Object(
            o.iter()
                .map(|(k, x)| (k.clone(), expande(x, valor)))
                .collect(),
        ),
        _ => v.clone(),
    }
}

/// Chave valida: `<status 128..239>/<data1 0..127>`. O teto e' 239 (0xEF) porque so' mensagem de
/// CANAL vira evento (`protocols::midi::canal` recusa 0xF0..0xFF, que e' system common e realtime):
/// aceitar `240/0` seria aceitar uma chave que nunca dispara.
fn chave_ok(k: &str) -> Result<(), String> {
    let erro = || format!("chave \"{}\": esperava <status>/<data1>, ex. 144/60", k);
    let (s, d) = k.split_once('/').ok_or_else(erro)?;
    let s: u16 = s.parse().map_err(|_| erro())?;
    let d: u16 = d.parse().map_err(|_| erro())?;
    if !(128..240).contains(&s) || d > 127 {
        return Err(erro());
    }
    Ok(())
}

// ------------------------------------------------------------------------ pump

/// Drena a fila do driver. Cada evento vai para os ganchos do player vivo e, se estiver no mapa,
/// para o registry.
// ponytail: o comando do mapa roda na thread que chamou o pump (o frame, no caso normal) ; virar
// thread se alguem mapear comando lento (`net` varre a rede, `play_show` toca o show inteiro).
pub fn pump() {
    loop {
        // pega UM evento e solta a porta: `midi_close` tambem quer este lock.
        let e = {
            let g = lock(&IN);
            match g.as_ref() {
                Some(m) => m.try_recv(),
                None => return,
            }
        };
        match e {
            Some((status, d1, d2)) => entrega(status, d1, d2),
            None => return,
        }
    }
}

/// Um evento: vira `input {key: "midi:<chave>"}` nos ganchos do player vivo e, se a chave estiver
/// no mapa, chamada do comando pelo registry. Publica porque e' o caminho inteiro do MIDI e o
/// teste precisa dele sem hardware (no Windows nao ha porta virtual para injetar nota).
pub fn entrega(status: u8, d1: u8, d2: u8) {
    let key = format!("{}/{}", status, d1);
    let valor = d2 as f64 / 127.0;
    *lock(&LAST) = Some((key.clone(), valor, d2));
    SEQ.fetch_add(1, Ordering::Relaxed);
    if let Some(h) = crate::player::current() {
        h.input(&format!("midi:{}", key), valor);
    }
    if let Some((cmd, args)) = liga(&key) {
        if let Err(e) = reg().call(&cmd, expande(&args, valor)) {
            eprintln!("midi {} -> {}: {}", key, cmd, e);
        }
    }
}

/// Abre a porta (nome, trecho do nome ou indice em texto) e devolve o nome de verdade.
fn abrir(port: &str) -> Result<String, String> {
    let m = MidiIn::open(port)?;
    let n = m.name().to_string();
    *lock(&IN) = Some(m);
    Ok(n)
}

/// `"midi_port": "<nome>"` no `.spell`: religa a superficie quando o show sobe. Falha vira aviso
/// — show nao para porque o controlador ficou no estojo.
pub fn auto(sh: &Show) {
    let Some(p) = sh
        .extra
        .get("midi_port")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    else {
        return;
    };
    if lock(&IN).as_ref().is_some_and(|m| m.name() == p) {
        return; // ja' aberta nesta porta
    }
    match abrir(p) {
        Ok(n) => eprintln!("midi: {}", n),
        Err(e) => eprintln!("aviso: midi_port \"{}\": {}", p, e),
    }
}

fn aberta() -> Value {
    match lock(&IN).as_ref() {
        Some(m) => json!(m.name()),
        None => Value::Null,
    }
}

fn ultima() -> Value {
    match lock(&LAST).as_ref() {
        Some((k, v, r)) => {
            json!({"key": k, "value": v, "raw": r, "seq": SEQ.load(Ordering::Relaxed)})
        }
        None => json!({"key": Value::Null, "value": 0.0, "raw": 0, "seq": 0}),
    }
}

// ---------------------------------------------------------------------- comandos

#[derive(Deserialize, JsonSchema)]
pub struct PortArgs {
    /// Nome da porta (ou um trecho dele), indice em texto ("0"), ou vazio = a primeira.
    #[serde(default)]
    pub port: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct MapArgs {
    /// Chave do evento: "<status>/<data1>" (144/60 = note on nota 60, 176/1 = CC 1).
    pub key: String,
    /// Nome do comando do registry.
    pub cmd: String,
    /// Argumentos do comando. "$" vira o valor 0..1 e "$<n>" vira round(valor * n).
    #[serde(default)]
    pub args: Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct KeyArgs {
    /// Chave do evento a remover.
    pub key: String,
}

pub fn register(r: &mut Registry) {
    r.add::<NoArgs>(
        "midi_ports",
        "Lista as portas MIDI de entrada da maquina e qual esta' aberta.",
        |_| Ok(json!({"ports": protocols::midi::ports(), "open": aberta()})),
    );
    r.add::<PortArgs>(
        "midi_open",
        "Abre uma porta MIDI de entrada (nome, trecho do nome ou indice) e grava midi_port no show aberto.",
        |a| {
            let n = abrir(&a.port)?;
            crate::edit::com(|_, sh| {
                sh.extra.insert("midi_port".into(), json!(n));
                Ok(json!({"open": n}))
            })
        },
    );
    r.add::<NoArgs>(
        "midi_close",
        "Fecha a porta MIDI e tira o midi_port do show aberto.",
        |_| {
            *lock(&IN) = None;
            crate::edit::com(|_, sh| {
                sh.extra.remove("midi_port");
                Ok(json!({"open": Value::Null}))
            })
        },
    );
    r.add::<MapArgs>(
        "midi_map",
        "Liga uma tecla/CC a um comando no show aberto. Devolve o mapa.",
        |a| {
            chave_ok(&a.key)?;
            if reg().get(&a.cmd).is_none() {
                return Err(format!("comando desconhecido: {}", a.cmd));
            }
            let args = match a.args {
                Value::Null => json!({}),
                // texto que e' JSON vira JSON (campo de formulario, CLI), como o `valor` do edit
                Value::String(s) => serde_json::from_str(&s).map_err(|e| format!("args: {}", e))?,
                v => v,
            };
            if !args.is_object() {
                return Err("args: esperava um objeto JSON".into());
            }
            crate::edit::com(|_, sh| {
                let m = sh
                    .extra
                    .entry("midi")
                    .or_insert_with(|| json!({}))
                    .as_object_mut()
                    .ok_or("midi: a chave \"midi\" do show nao e' um objeto")?;
                m.insert(a.key.clone(), json!({"cmd": a.cmd, "args": args}));
                Ok(Value::Object(m.clone()))
            })
        },
    );
    r.add::<KeyArgs>("midi_unmap", "Desliga uma tecla do show aberto.", |a| {
        crate::edit::com(|_, sh| {
            let m = sh
                .extra
                .get_mut("midi")
                .and_then(Value::as_object_mut)
                .ok_or("midi: o show nao tem mapa")?;
            m.remove(&a.key)
                .ok_or_else(|| format!("chave {} nao esta' no mapa", a.key))?;
            Ok(Value::Object(m.clone()))
        })
    });
    r.add::<NoArgs>(
        "midi_maps",
        "O mapa tecla -> comando do show aberto.",
        |_| Ok(Value::Object(mapa())),
    );
    r.add::<NoArgs>(
        "midi_last",
        "Ultima tecla MIDI recebida: {key, value, raw, seq}. E' com ela que a GUI mostra ao vivo.",
        |_| {
            pump();
            Ok(ultima())
        },
    );
    r.add::<NoArgs>(
        "midi_learn",
        "Espera ate' 5 s pela proxima tecla MIDI e devolve a chave dela.",
        |_| {
            let base = SEQ.load(Ordering::Relaxed);
            let fim = Instant::now() + Duration::from_millis(LEARN_MS);
            while Instant::now() < fim {
                pump(); // sem player tocando, ninguem mais drena a fila
                if SEQ.load(Ordering::Relaxed) != base {
                    return Ok(ultima());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(format!("nenhum evento MIDI em {} s", LEARN_MS / 1000))
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expande_cifrao() {
        let v = json!({"universe": 1, "address": 3, "values": ["$255"], "u": "$", "n": "$127"});
        let e = expande(&v, 100.0 / 127.0);
        assert_eq!(e["universe"], json!(1), "numero passa igual");
        assert_eq!(e["values"], json!([201]), "$255 = nivel DMX");
        assert_eq!(e["n"], json!(100), "$127 = o byte cru do MIDI");
        assert!(e["n"].is_i64(), "$<n> sai inteiro, senao `u16` recusa");
        assert!((e["u"].as_f64().unwrap() - 100.0 / 127.0).abs() < 1e-12);
        // zero e cheio nas duas pontas
        assert_eq!(expande(&json!("$255"), 0.0), json!(0));
        assert_eq!(expande(&json!("$255"), 1.0), json!(255));
        assert_eq!(expande(&json!("$127"), 1.0), json!(127));
        // texto que nao e' cifrao fica como esta'
        assert_eq!(expande(&json!("$x"), 0.5), json!("$x"));
        assert_eq!(expande(&json!("go"), 0.5), json!("go"));
        assert_eq!(expande(&json!(true), 0.5), json!(true));
    }

    #[test]
    fn chave_valida() {
        for k in ["144/60", "128/0", "176/1", "239/127"] {
            assert!(chave_ok(k).is_ok(), "{}", k);
        }
        // 240 (0xF0) para cima e' system common/realtime: `protocols::midi::canal` nao emite
        for k in [
            "", "144", "60/144", "144/128", "x/1", "144/", "-1/1", "240/0", "255/127", "256/1",
        ] {
            assert!(chave_ok(k).is_err(), "{}", k);
        }
    }
}
