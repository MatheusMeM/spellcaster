//! `spellcore` — CLI da R1. Os subcomandos NAO sao escritos a mao: sao construidos em runtime a
//! partir de `Registry::schema()`, exatamente como o `spellcaster/cli.py` faz com o registry
//! Python. Quem tem logica e o registry; a CLI so traduz argv <-> JSON e imprime.
//!
//!   spellcore play <show.spell> [--loop] [--osc-port N]
//!   spellcore net [--json] [--timeout N]
//!   spellcore commands

use clap::{Arg, ArgAction, ArgMatches};
use engine::registry::Registry;
use engine::{show, Ev, EventSink, Player, TransportState};
use protocols::{netscan, osc};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Subcomando da CLI -> comando do registry. UMA entrada: o verbo do produto chama-se
/// `play_show` (igual ao Python), o operador digita `play`.
const ALIAS: [(&str, &str); 1] = [("play", "play_show")];

fn as_cli(reg: &str) -> &str {
    ALIAS
        .iter()
        .find(|(_, r)| *r == reg)
        .map_or(reg, |(c, _)| *c)
}

fn as_reg(cli: &str) -> &str {
    ALIAS
        .iter()
        .find(|(c, _)| *c == cli)
        .map_or(cli, |(_, r)| *r)
}

// ------------------------------------------------------------------ comandos

#[derive(Deserialize, JsonSchema)]
struct PlayArgs {
    /// Caminho do arquivo .spell.
    file: String,
    /// Repete o show do inicio ao chegar no fim.
    #[serde(default, rename = "loop")]
    #[schemars(rename = "loop")]
    looping: bool,
    /// Porta do transporte remoto por OSC (sobrepoe transport.osc_port do .spell).
    #[serde(default)]
    osc_port: Option<u16>,
}

#[derive(Deserialize, JsonSchema)]
struct NetArgs {
    /// Segundos de escuta por protocolo.
    #[serde(default = "def_timeout")]
    timeout: f64,
    /// Devolve o resultado cru em JSON em vez do relatorio de texto.
    #[serde(default)]
    json: bool,
}

fn def_timeout() -> f64 {
    2.0
}

// ------------------------------------------------------------- sink de evento

/// Saida de evento do Graph: `Cmd` vai para o registry, `Osc` para a saida OSC do show, o
/// resto para o stderr em uma linha ASCII.
struct CliSink {
    reg: Registry,
    osc: Option<osc::OscOut>,
}

impl CliSink {
    // ponytail: o sink monta seu proprio `registry::base()` (o transporte age no player vivo por
    // `player::current()`, nao precisa do registry da CLI) ; passar o registry inteiro quando um
    // no `cmd` precisar de `net` ou `play_show`.
    fn new(target: Option<(String, u16)>) -> CliSink {
        let osc = target.and_then(|(h, p)| match osc::OscOut::new(&h, p) {
            Ok(o) => Some(o),
            Err(e) => {
                eprintln!("aviso: saida osc {}:{} indisponivel: {}", h, p, e);
                None
            }
        });
        CliSink {
            reg: engine::registry::base(),
            osc,
        }
    }
}

impl EventSink for CliSink {
    fn emit(&mut self, e: &Ev) {
        match e {
            Ev::Cmd { name, args } => {
                if let Err(err) = self.reg.call(name, args.clone()) {
                    eprintln!("cmd {}: {}", name, err);
                }
            }
            Ev::Osc { address, args } => match &self.osc {
                // float32 e' o que o `spellcaster/protocols/osc.py` emite para numero solto.
                Some(o) => {
                    let a: Vec<osc::Arg> = args.iter().map(|v| osc::Arg::Float(*v as f32)).collect();
                    o.send(address, &a);
                }
                None => eprintln!("osc {}: show sem saida osc", address),
            },
            Ev::Widget { id, prop, value } => eprintln!("widget {}.{}={}", id, prop, value),
            Ev::Param { target, value } => eprintln!("param {}={}", target, value),
            Ev::Notify { text } => eprintln!("notify {}", text),
        }
    }
}

/// Host/porta da saida `osc` do .spell. `show::OutputCfg` guarda so' o nome dos tipos que nao
/// conhece, entao o destino do `out.osc` do Graph sai do JSON cru.
// ponytail: reabre o arquivo so' para isso ; sair daqui quando `OutputCfg` ganhar variante Osc.
fn osc_target(path: &Path) -> Option<(String, u16)> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let c = v["outputs"].as_array()?.iter().find(|c| c["type"] == "osc")?;
    let host = c["host"].as_str().unwrap_or("127.0.0.1").to_string();
    Some((host, c["port"].as_u64()? as u16))
}

// ------------------------------------------------------------ play (headless)

/// Uma linha por segundo, largura estavel no `t`, so' ASCII. Sem `\r` e sem barra de progresso:
/// a saida do play tem que sobreviver a um `ssh ... | tee` no Pi.
fn status_line(st: &TransportState, jit_p99_ms: f64) -> String {
    let u: Vec<String> = st.universes.iter().map(|n| n.to_string()).collect();
    format!(
        "t={:>7.2}s state={} cue={} frames={} jit_p99={:.2}ms u={}",
        st.t,
        st.state,
        st.cue,
        st.frames,
        jit_p99_ms,
        u.join(",")
    )
}

static INT: AtomicBool = AtomicBool::new(false);

/// Ctrl+C vira um flag; quem fecha o player e' o laco do `play`, na thread principal (chamar o
/// engine de dentro de um handler de sinal nao e' seguro).
#[cfg(windows)]
mod sig {
    use std::sync::atomic::Ordering;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetConsoleCtrlHandler(h: Option<extern "system" fn(u32) -> i32>, add: i32) -> i32;
    }
    extern "system" fn on_ctrl(_ty: u32) -> i32 {
        super::INT.store(true, Ordering::SeqCst);
        1 // TRUE: tratado, o processo continua vivo ate o `close()`
    }
    pub fn trap() {
        unsafe { SetConsoleCtrlHandler(Some(on_ctrl), 1) };
    }
}

#[cfg(not(windows))]
mod sig {
    use std::sync::atomic::Ordering;
    unsafe extern "C" {
        fn signal(sig: i32, h: usize) -> usize;
    }
    extern "C" fn on_sigint(_s: i32) {
        super::INT.store(true, Ordering::SeqCst);
    }
    pub fn trap() {
        unsafe { signal(2, on_sigint as usize) }; // SIGINT
    }
}

fn play(a: PlayArgs) -> Result<Value, String> {
    let path = Path::new(&a.file);
    let sh = show::load(path)?;
    let base = path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
    let sink: Box<dyn EventSink> = Box::new(CliSink::new(osc_target(path)));
    let hooks = script::hooks(&sh, &base, sink)?;
    let name = sh.name.clone();

    let mut p = Player::new(sh, base, a.looping)?;
    for h in hooks {
        p.hook(h);
    }
    p.start(a.osc_port)?;
    let h = p.handle();
    h.play();
    sig::trap();

    // Cabecalho sem universos: eles so' existem depois do primeiro frame; quem os mostra e' a
    // linha de status.
    let st = h.state();
    let dur = st.duration.map_or("sem fim".into(), |d| format!("{:.2}s", d));
    println!("{}: {} fps, {}", name, st.fps, dur);
    // ponytail: acorda a cada 200 ms so' para ver o Ctrl+C e imprimir o status ; virar condvar
    // do player se a linha de status precisar de resolucao melhor que 1 s.
    let mut last = Instant::now();
    while !p.wait(Some(Duration::from_millis(200))) {
        if INT.load(Ordering::SeqCst) {
            break;
        }
        if last.elapsed() >= Duration::from_secs(1) {
            last = Instant::now();
            // ponytail: jit_p99 e' o do ultimo `Clock::run` FECHADO (o Clock so' publica stats no
            // fim do run) ; virar contador vivo se o operador precisar do jitter durante o show.
            println!("{}", status_line(&h.state(), p.clock().stats().p99 * 1e3));
        }
    }
    let st = h.state();
    p.close();
    let s = p.clock().stats();
    Ok(json!({"name": name, "frames": st.frames, "jitter_p99_ms": s.p99 * 1e3,
              "jitter_max_ms": s.max * 1e3, "drift": s.drift}))
}

fn net(a: NetArgs) -> Result<Value, String> {
    let scan = netscan::scan_all(Duration::from_secs_f64(a.timeout.max(0.1)));
    if a.json {
        serde_json::to_value(&scan).map_err(|e| e.to_string())
    } else {
        Ok(Value::String(netscan::report(&scan)))
    }
}

/// `play_show` e `net` moram aqui porque so' a CLI conhece `script` e `protocols`; o resto do
/// transporte vem de `registry::base()`, que age no player vivo (`player::current()`).
fn registry() -> Registry {
    let mut r = engine::registry::base();
    r.add::<PlayArgs>("play_show", "Toca um show .spell ate o fim ou Ctrl+C.", play);
    r.add::<NetArgs>(
        "net",
        "Varre a rede: interfaces, nos Art-Net, fontes sACN, Ether Dream e sugestoes.",
        net,
    );
    r
}

// -------------------------------------------------------- argv <-> JSON

/// Tipo declarado no schema JSON da propriedade ("string" | "number" | "integer" | "boolean").
fn kind(p: &Value) -> &str {
    match &p["type"] {
        Value::String(s) => s.as_str(),
        Value::Array(a) => a
            .iter()
            .filter_map(|v| v.as_str())
            .find(|s| *s != "null")
            .unwrap_or("string"),
        _ => "string",
    }
}

/// Converte o texto do argparse para o tipo do schema (o `_coerce` do registry Python).
fn coerce(k: &str, s: &str) -> Result<Value, String> {
    match k {
        "integer" => s
            .parse::<i64>()
            .map(Value::from)
            .map_err(|_| format!("valor inteiro invalido: {}", s)),
        "number" => s
            .parse::<f64>()
            .map(Value::from)
            .map_err(|_| format!("valor numerico invalido: {}", s)),
        "boolean" => Ok(Value::Bool(matches!(
            s.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on" | "sim"
        ))),
        _ => Ok(Value::String(s.to_string())),
    }
}

/// clap exige `&'static str` em nome/long/value_name. O schema so' e' conhecido em runtime.
// ponytail: vaza um punhado de strings curtas uma vez no boot (< 1 KB, vida = a do processo)
// ; trocar por `Box<Registry>` vazado inteiro se algum dia a CLI reconstruir os subcomandos.
fn stat(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

/// Um subcomando clap por comando do registry. Parametro sem default vira posicional;
/// `boolean` vira flag `--nome`; o resto vira `--nome VALOR`. Convencao das duas pontas:
/// JSON em snake_case (`osc_port`), argv com traco (`--osc-port`).
fn subcommand(name: &str, doc: &str, params: &Value) -> clap::Command {
    let mut c = clap::Command::new(stat(name)).about(stat(doc));
    let req: Vec<&str> = params["required"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    if let Some(props) = params["properties"].as_object() {
        for (pname, p) in props {
            let id = stat(pname);
            let long = stat(&pname.replace('_', "-"));
            let mut arg = Arg::new(id).help(stat(p["description"].as_str().unwrap_or("")));
            if kind(p) == "boolean" {
                arg = arg.long(long).action(ArgAction::SetTrue);
            } else if req.contains(&pname.as_str()) {
                arg = arg.required(true).value_name(stat(&pname.to_uppercase()));
            } else {
                arg = arg.long(long).value_name(stat(&pname.to_uppercase()));
            }
            c = c.arg(arg);
        }
    }
    c
}

fn args_from(m: &ArgMatches, params: &Value) -> Result<Value, String> {
    let mut out = Map::new();
    if let Some(props) = params["properties"].as_object() {
        for (pname, p) in props {
            if kind(p) == "boolean" {
                if m.get_flag(pname) {
                    out.insert(pname.clone(), Value::Bool(true));
                }
            } else if let Some(s) = m.get_one::<String>(pname) {
                out.insert(pname.clone(), coerce(kind(p), s)?);
            }
        }
    }
    Ok(Value::Object(out))
}

fn main() {
    let reg = registry();
    let schema = reg.schema();
    let entries = schema.as_array().cloned().unwrap_or_default();

    let mut app = clap::Command::new("spellcore")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Spellcaster core (R1): timeline, cues, fx, graph, sACN, Art-Net, OSC, rede")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(clap::Command::new("commands").about("Lista o registry em JSON."));
    for e in &entries {
        app = app.subcommand(subcommand(
            as_cli(e["name"].as_str().unwrap_or("")),
            e["doc"].as_str().unwrap_or(""),
            &e["params"],
        ));
    }

    let m = app.get_matches();
    let (cli_name, sub) = m.subcommand().expect("subcomando obrigatorio");
    if cli_name == "commands" {
        println!("{}", serde_json::to_string_pretty(&schema).unwrap_or_default());
        return;
    }
    let name = as_reg(cli_name);
    let params = entries
        .iter()
        .find(|e| e["name"] == name)
        .map(|e| e["params"].clone())
        .unwrap_or(Value::Null);

    let r = args_from(sub, &params).and_then(|a| reg.call(name, a));
    match r {
        // string = relatorio pronto (net sem --json); o resto sai como JSON
        Ok(Value::String(s)) => println!("{}", s),
        Ok(v) => println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default()),
        Err(e) => {
            eprintln!("erro: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coerce_por_tipo() {
        assert_eq!(coerce("integer", "7").unwrap(), json!(7));
        assert_eq!(coerce("number", "1.5").unwrap(), json!(1.5));
        assert_eq!(coerce("boolean", "sim").unwrap(), json!(true));
        assert_eq!(coerce("boolean", "nao").unwrap(), json!(false));
        assert_eq!(coerce("string", "x").unwrap(), json!("x"));
        assert!(coerce("integer", "x").is_err());
    }

    #[test]
    fn kind_aceita_option() {
        assert_eq!(kind(&json!({"type": "number"})), "number");
        assert_eq!(kind(&json!({"type": ["string", "null"]})), "string");
        assert_eq!(kind(&json!({"type": ["integer", "null"]})), "integer");
        assert_eq!(kind(&json!({})), "string");
    }

    /// O contrato da CLI: todo comando do registry vira subcomando, e o argv volta como o
    /// JSON que o registry espera. Se isso quebrar, `spellcore play` para de existir.
    #[test]
    fn subcomandos_saem_do_registry() {
        let reg = registry();
        let schema = reg.schema();
        let entries = schema.as_array().unwrap();
        let nomes: Vec<&str> = entries.iter().map(|e| e["name"].as_str().unwrap()).collect();
        assert!(nomes.contains(&"play_show"), "registry sem play_show: {:?}", nomes);
        assert!(nomes.contains(&"net") && nomes.contains(&"load"));

        let e = entries.iter().find(|e| e["name"] == "play_show").unwrap();
        let cmd = subcommand("play", "", &e["params"]);
        let m = cmd
            .try_get_matches_from(vec!["play", "shows/x.spell", "--loop", "--osc-port", "9000"])
            .expect("play aceita posicional, --loop e --osc-port");
        assert_eq!(
            args_from(&m, &e["params"]).unwrap(),
            json!({"file": "shows/x.spell", "loop": true, "osc_port": 9000})
        );

        let e = entries.iter().find(|e| e["name"] == "net").unwrap();
        let cmd = subcommand("net", "", &e["params"]);
        let m = cmd
            .try_get_matches_from(vec!["net", "--timeout", "0.5", "--json"])
            .expect("net aceita --timeout e --json");
        assert_eq!(
            args_from(&m, &e["params"]).unwrap(),
            json!({"timeout": 0.5, "json": true})
        );
    }

    /// O operador digita `play`; o registry (e o MCP depois) so' conhece `play_show`.
    #[test]
    fn alias_play_vira_play_show() {
        assert_eq!(as_reg("play"), "play_show");
        assert_eq!(as_cli("play_show"), "play");
        assert_eq!(as_reg("net"), "net");
        assert_eq!(as_cli("net"), "net");
        let reg = registry();
        assert!(reg.get("play_show").is_some());
        assert!(reg.get("play").is_none(), "sem comando duplicado");
    }

    #[test]
    fn linha_de_status() {
        let st = TransportState {
            t: 12.345,
            state: "play",
            cue: 3,
            frames: 372,
            fps: 30,
            duration: Some(60.0),
            universes: vec![1, 2],
        };
        assert_eq!(
            status_line(&st, 0.41),
            "t=  12.35s state=play cue=3 frames=372 jit_p99=0.41ms u=1,2"
        );
        // largura do campo `t` estavel: a linha nao muda de forma entre um segundo e o proximo
        let parado = TransportState {
            t: 0.0,
            state: "stop",
            cue: -1,
            frames: 0,
            fps: 30,
            duration: None,
            universes: Vec::new(),
        };
        let l = status_line(&parado, 0.0);
        assert_eq!(l, "t=   0.00s state=stop cue=-1 frames=0 jit_p99=0.00ms u=");
        assert!(l.is_ascii() && !l.contains('\r'));
    }
}
