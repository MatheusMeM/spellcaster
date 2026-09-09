//! `spellcore` — CLI da R0. Os subcomandos NAO sao escritos a mao: sao construidos em runtime a
//! partir de `Registry::schema()`, exatamente como o `spellcaster/cli.py` faz com o registry
//! Python. Quem tem logica e o registry; a CLI so traduz argv <-> JSON.
//!
//!   spellcore play <show.spell> [--loop]
//!   spellcore net [--json] [--timeout N]
//!   spellcore commands

use clap::{Arg, ArgAction, ArgMatches};
use engine::registry::Registry;
use engine::{show, Clock, Timeline, Universes};
use protocols::{artnet::ArtNetOut, netscan, sacn::SacnOut, Output};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::net::Ipv4Addr;
use std::path::Path;
use std::time::Duration;

// ------------------------------------------------------------------ comandos

#[derive(Deserialize, JsonSchema)]
struct PlayArgs {
    /// Caminho do arquivo .spell.
    file: String,
    /// Repete o show do inicio ao chegar no fim.
    #[serde(default, rename = "loop")]
    #[schemars(rename = "loop")]
    looping: bool,
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

/// Abre as saidas declaradas em `outputs`. Tipo desconhecido vira aviso, nao erro.
fn open_outputs(sh: &show::Show) -> Result<Vec<Box<dyn Output>>, String> {
    let mut outs: Vec<Box<dyn Output>> = Vec::new();
    for c in &sh.outputs {
        match c {
            show::OutputCfg::Sacn {
                universes,
                priority,
                source_name,
                interfaces,
            } => {
                let ifaces = interfaces.as_ref().map(|v| {
                    v.iter()
                        .filter_map(|s| s.parse::<Ipv4Addr>().ok())
                        .collect::<Vec<_>>()
                });
                let o = SacnOut::with(universes, *priority, source_name, ifaces)
                    .map_err(|e| format!("sacn: {}", e))?;
                outs.push(Box::new(o));
            }
            show::OutputCfg::ArtNet { targets, broadcast } => {
                let o = ArtNetOut::new(targets.clone(), *broadcast)
                    .map_err(|e| format!("artnet: {}", e))?;
                outs.push(Box::new(o));
            }
            show::OutputCfg::Unknown(t) => eprintln!("aviso: saida \"{}\" ignorada na R0", t),
        }
    }
    Ok(outs)
}

fn play(a: PlayArgs) -> Result<Value, String> {
    let path = Path::new(&a.file);
    let sh = show::load(path)?;
    let mut tl = Timeline::new(&sh)?;
    let ignored = tl.ignored().join(", ");
    if !ignored.is_empty() {
        eprintln!("aviso: tracks ignorados na R0: {}", ignored);
    }
    let mut outs = open_outputs(&sh)?;
    if outs.is_empty() {
        return Err("show sem saida utilizavel (outputs vazio)".into());
    }
    let mut uni = Universes::new();
    let clock = Clock::new(sh.fps);
    println!(
        "{}: {} tracks, {} fps, {:?}s, {} saidas",
        sh.name,
        tl.tracks.len(),
        sh.fps,
        sh.duration,
        outs.len()
    );

    // ponytail: todo universo escrito vai para TODAS as saidas do show ; separar os universos
    // por saida quando um show misturar tracks "dmx" e "artnet" no mesmo numero de universo.
    let mut frames: u64 = 0;
    loop {
        clock.run(
            |t| {
                tl.apply(&mut uni, t);
                for u in uni.iter() {
                    for o in outs.iter_mut() {
                        o.send(u.number, &u.data);
                    }
                }
                frames += 1;
            },
            sh.duration,
        );
        if !a.looping || sh.duration.is_none() {
            break;
        }
        clock.locate(0.0);
        clock.play();
    }
    for o in outs.iter_mut() {
        o.close();
    }
    let st = clock.stats();
    Ok(json!({"name": sh.name, "frames": frames, "jitter_p99_ms": st.p99 * 1e3,
              "jitter_max_ms": st.max * 1e3, "drift": st.drift}))
}

fn net(a: NetArgs) -> Result<Value, String> {
    let scan = netscan::scan_all(Duration::from_secs_f64(a.timeout.max(0.1)));
    if a.json {
        serde_json::to_value(&scan).map_err(|e| e.to_string())
    } else {
        Ok(Value::String(netscan::report(&scan)))
    }
}

fn registry() -> Registry {
    // O relogio do `locate` do engine e o do transporte remoto; o `play` cria o seu.
    // ponytail: `locate`/`pause`/`stop` so agem num player deste processo, que a R0 nao tem
    // ; ligar no transporte quando a R1 trouxer o Player com OSC.
    let mut r = engine::registry::base(Clock::new(30));
    r.add::<PlayArgs>("play", "Toca um show .spell ate o fim ou Ctrl+C.", play);
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
/// `boolean` vira flag `--nome`; o resto vira `--nome VALOR`.
fn subcommand(name: &str, doc: &str, params: &Value) -> clap::Command {
    let mut c = clap::Command::new(stat(name)).about(stat(doc));
    let req: Vec<&str> = params["required"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    if let Some(props) = params["properties"].as_object() {
        for (pname, p) in props {
            let id = stat(pname);
            let mut arg = Arg::new(id).help(stat(p["description"].as_str().unwrap_or("")));
            if kind(p) == "boolean" {
                arg = arg.long(id).action(ArgAction::SetTrue);
            } else if req.contains(&pname.as_str()) {
                arg = arg.required(true).value_name(stat(&pname.to_uppercase()));
            } else {
                arg = arg.long(id).value_name(stat(&pname.to_uppercase()));
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
        .about("Spellcaster core (R0): timeline, sACN, Art-Net, OSC, varredura de rede")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(clap::Command::new("commands").about("Lista o registry em JSON."));
    for e in &entries {
        app = app.subcommand(subcommand(
            e["name"].as_str().unwrap_or(""),
            e["doc"].as_str().unwrap_or(""),
            &e["params"],
        ));
    }

    let m = app.get_matches();
    let (name, sub) = m.subcommand().expect("subcomando obrigatorio");
    if name == "commands" {
        println!("{}", serde_json::to_string_pretty(&schema).unwrap_or_default());
        return;
    }
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
        assert!(nomes.contains(&"play") && nomes.contains(&"net") && nomes.contains(&"load"));

        let e = entries.iter().find(|e| e["name"] == "play").unwrap();
        let cmd = subcommand("play", "", &e["params"]);
        let m = cmd
            .try_get_matches_from(vec!["play", "shows/x.spell", "--loop"])
            .expect("play aceita posicional + --loop");
        assert_eq!(
            args_from(&m, &e["params"]).unwrap(),
            json!({"file": "shows/x.spell", "loop": true})
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
}
