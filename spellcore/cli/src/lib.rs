//! `spellcore` — CLI da R1/R7/R5. Cinco subcomandos, escritos com `clap::Parser`:
//!
//!   spellcore play <show.spell> [--loop] [--osc-port N]
//!   spellcore net [--json] [--timeout N]
//!   spellcore commands [nome]
//!   spellcore mcp [install --target desktop|code [--path P] [--yes]]
//!   spellcore serve [--port N] [--dir D] [--show x.spell]
//!
//! Quem tem logica e' o registry; a CLI so' chama e imprime. Os subcomandos NAO sao mais
//! gerados em runtime a partir do `Registry::schema()`: `load`, `pause`, `stop`, `locate`,
//! `cue_go`, `transport_state` e `show_get` agem no player VIVO NESTE PROCESSO e nao faziam
//! sentido como processo separado. Eles continuam no registry, que e' o que o MCP expoe.
//!
//! `PlayArgs` e `NetArgs` servem as duas pontas: `clap::Args` para o argv e `JsonSchema` +
//! `Deserialize` para o registry (e, por ele, para as tools do MCP).

mod laser_cmd;

use clap::{Args, Parser, Subcommand};
use engine::registry::{NoArgs, Registry};
use engine::schemars::JsonSchema;
use engine::{show, Ev, EventSink, Player, TransportState};
use protocols::{netscan, osc};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

// ------------------------------------------------------------------ comandos

/// Uma fonte para o `doc` do registry (`/commands`, tools MCP) e para o `about` do clap
/// (`spellcore --help`, `spellcore play --help`): os dois lados leem estas constantes.
const DOC_PLAY: &str = "Toca um show .spell ate o fim ou Ctrl+C.";
const DOC_NET: &str =
    "Varre a rede: interfaces, nos Art-Net, fontes sACN, Ether Dream e sugestoes.";
const DOC_GRAPH_CHECK: &str = "Compila o graph do show aberto sem rodar. Devolve {nodes, error}.";

#[derive(Args, Deserialize, JsonSchema)]
#[schemars(crate = "engine::schemars")]
struct PlayArgs {
    /// Caminho do arquivo .spell.
    file: String,
    /// Repete o show do inicio ao chegar no fim.
    #[arg(long = "loop")]
    #[serde(default, rename = "loop")]
    #[schemars(rename = "loop")]
    looping: bool,
    /// Porta do transporte remoto por OSC (sobrepoe transport.osc_port do .spell).
    #[arg(long)]
    #[serde(default)]
    osc_port: Option<u16>,
}

#[derive(Args, Deserialize, JsonSchema)]
#[schemars(crate = "engine::schemars")]
struct NetArgs {
    /// Segundos de escuta por protocolo.
    #[arg(long, default_value_t = def_timeout())]
    #[serde(default = "def_timeout")]
    timeout: f64,
    /// Devolve o resultado cru em JSON em vez do relatorio de texto.
    #[arg(long)]
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

/// `args` do `Ev::Cmd` mais `feed`, para o roteador do no `module`.
fn com_feed(mut args: Value, feed: &str) -> Value {
    if let Some(o) = args.as_object_mut() {
        o.insert("feed".into(), Value::String(feed.into()));
    }
    args
}

impl EventSink for CliSink {
    fn emit(&mut self, e: &Ev) {
        match e {
            // O no `module` do graph emite `<mod>/<cmd>`; o comando registrado e' `<mod>_<cmd>`,
            // com `feed` = nome do modulo.
            // ponytail: convencao feed = nome do modulo ; instancia nomeada quando houver dois lasers.
            Ev::Cmd { name, args } => {
                let (nome, args) = match name.split_once('/') {
                    Some((m, c)) => (format!("{}_{}", m, c), com_feed(args.clone(), m)),
                    None => (name.clone(), args.clone()),
                };
                if let Err(err) = self.reg.call(&nome, args) {
                    eprintln!("cmd {}: {}", nome, err);
                }
            }
            Ev::Osc { address, args } => match &self.osc {
                // float32 e' o que o `spellcaster/protocols/osc.py` emite para numero solto.
                Some(o) => {
                    let a: Vec<osc::Arg> =
                        args.iter().map(|v| osc::Arg::Float(*v as f32)).collect();
                    o.send(address, &a);
                }
                None => eprintln!("osc {}: show sem saida osc", address),
            },
            // `out.widget` e' da GUI: vai para o barramento (nada acontece sem `serve` no ar) e
            // para o stderr, que e' o unico monitor do `spellcore play`.
            Ev::Widget { id, prop, value } => {
                serve::widget(id, prop, *value);
                eprintln!("widget {}.{}={}", id, prop, value);
            }
            Ev::Param { target, value } => match target.split_once('/') {
                Some((m, path)) => {
                    let a = json!({"feed": m, "path": path, "value": value});
                    if let Err(err) = self.reg.call(&format!("{}_param", m), a) {
                        eprintln!("param {}: {}", target, err);
                    }
                }
                None => eprintln!("param {}={}", target, value),
            },
            Ev::Notify { text } => eprintln!("notify {}", text),
        }
    }
}

/// Host/porta da saida `osc` do .spell — o destino do `out.osc` do Graph.
fn osc_target(sh: &show::Show) -> Option<(String, u16)> {
    sh.outputs.iter().find_map(|o| match o {
        show::OutputCfg::Osc(o) => Some((o.host.clone(), o.port)),
        _ => None,
    })
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

/// Falso enquanto o servidor MCP roda: la' o stdout e' o canal JSON-RPC e uma linha de status
/// derruba o protocolo (o Python resolve com `contextlib.redirect_stdout`).
static STDOUT_LIVRE: AtomicBool = AtomicBool::new(true);

fn saida(l: &str) {
    if STDOUT_LIVRE.load(Ordering::SeqCst) {
        println!("{}", l);
    } else {
        eprintln!("{}", l);
    }
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
        fn signal(sig: i32, h: extern "C" fn(i32)) -> usize;
    }
    extern "C" fn on_sigint(_s: i32) {
        super::INT.store(true, Ordering::SeqCst);
    }
    pub fn trap() {
        unsafe { signal(2, on_sigint) }; // SIGINT
    }
}

fn play(a: PlayArgs) -> Result<Value, String> {
    let path = Path::new(&a.file);
    let sh = show::load(path)?;
    let base = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let sink: Box<dyn EventSink> = Box::new(CliSink::new(osc_target(&sh)));
    let hooks = script::hooks(&sh, &base, sink)?;
    let name = sh.name.clone();

    let mut p = Player::new(sh, a.looping)?;
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
    let dur = st
        .duration
        .map_or("sem fim".into(), |d| format!("{:.2}s", d));
    saida(&format!("{}: {} fps, {}", name, st.fps, dur));
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
            saida(&status_line(&h.state(), p.clock().stats().p99 * 1e3));
        }
    }
    let st = h.state();
    p.close();
    let s = p.clock().stats();
    Ok(
        json!({"name": name, "frames": st.frames, "jitter_p99_ms": s.p99 * 1e3,
              "jitter_max_ms": s.max * 1e3, "drift": s.drift}),
    )
}

fn net(a: NetArgs) -> Result<Value, String> {
    let scan = netscan::scan_all(Duration::from_secs_f64(a.timeout.max(0.1)));
    if a.json {
        serde_json::to_value(&scan).map_err(|e| e.to_string())
    } else {
        Ok(Value::String(netscan::report(&scan)))
    }
}

/// Compila o graph do show aberto sem rodar: quantos nos ele tem, ou o erro. Mora aqui porque
/// so' a CLI conhece o crate `script` — `graph_get`, que e' edicao pura, fica no engine.
fn graph_check(_: NoArgs) -> Result<Value, String> {
    let g = engine::edit::graph();
    match script::graph::Graph::new(&g, Box::new(engine::NullSink)) {
        Ok(gr) => Ok(json!({"nodes": gr.nodes(), "error": Value::Null})),
        Err(e) => Ok(json!({"nodes": 0, "error": e})),
    }
}

/// `play_show`, `net` e `graph_check` moram aqui porque so' a CLI conhece `script` e
/// `protocols`; o resto do
/// transporte vem de `registry::base()`, que age no player vivo (`player::current()`).
/// E' este registry que o MCP expoe como tools.
pub fn registry() -> Registry {
    let mut r = engine::registry::base();
    r.add::<PlayArgs>("play_show", DOC_PLAY, play);
    r.add::<NetArgs>("net", DOC_NET, net);
    r.add::<NoArgs>("graph_check", DOC_GRAPH_CHECK, graph_check);
    laser_cmd::register(&mut r);
    r
}

// ------------------------------------------------------------------- argv

#[derive(Parser)]
#[command(
    name = "spellcore",
    version,
    about = "Spellcaster core: timeline, cues, fx, graph, sACN, Art-Net, OSC, rede, MCP",
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Args)]
struct ServeArgs {
    /// Porta HTTP; 0 = uma livre (a linha `serve http://...` no stderr diz qual).
    #[arg(long, default_value_t = 8000)]
    port: u16,
    /// Raiz do estatico; padrao = raiz do repo, porque as paginas de `spellgui/web`
    /// referenciam `../../design/tokens` e `../../shows`.
    #[arg(long, default_value = ".")]
    dir: String,
    /// .spell aberto no boot, com o player parado em t=0.
    #[arg(long)]
    show: Option<String>,
}

#[derive(Subcommand)]
enum Cmd {
    #[command(about = DOC_PLAY)]
    Play(PlayArgs),
    #[command(about = DOC_NET)]
    Net(NetArgs),
    /// Lista o registry em JSON: nome, doc e schema de cada comando. Com um nome, so' esse
    /// comando - e' o `spellcore <cmd> --help` dos verbos que nao viraram subcomando.
    Commands {
        /// Nome do comando do registry; vazio = todos.
        name: Option<String>,
    },
    /// Servidor MCP em stdio; `spellcore mcp install` registra o servidor no Claude.
    Mcp(McpArgs),
    /// Barramento: HTTP + WebSocket + MCP em /mcp. E' o processo que toca o hardware.
    Serve(ServeArgs),
}

#[derive(Args)]
struct McpArgs {
    #[command(subcommand)]
    cmd: Option<McpCmd>,
}

#[derive(Subcommand)]
enum McpCmd {
    /// Grava a entrada "spellcaster" no config do Claude (pede confirmacao).
    Install {
        /// desktop = claude_desktop_config.json; code = .mcp.json do diretorio corrente.
        #[arg(long, default_value = "desktop")]
        target: String,
        /// Caminho do config; vazio = o padrao do target.
        #[arg(long, default_value = "")]
        path: String,
        /// Grava sem perguntar.
        #[arg(long)]
        yes: bool,
    },
}

pub fn main() {
    // O mapa MIDI chama o registry COMPLETO (com `play_show`, `net` e os `laser_*`), nao so' o
    // `base()` do engine.
    engine::midi::builder(registry);
    let r = match Cli::parse().cmd {
        Cmd::Play(a) => play(a),
        Cmd::Net(a) => net(a),
        Cmd::Commands { name } => match name {
            None => Ok(registry().schema()),
            Some(n) => registry()
                .schema()
                .as_array()
                .and_then(|a| a.iter().find(|c| c["name"] == n.as_str()).cloned())
                .ok_or_else(|| format!("comando desconhecido: {}", n)),
        },
        Cmd::Serve(a) => {
            // o stdout de um servidor nao e' canal de dado: status e log vao para o stderr
            STDOUT_LIVRE.store(false, Ordering::SeqCst);
            serve::serve(registry(), a.port, a.dir.into(), a.show, |_| {}).map(|_| Value::Null)
        }
        Cmd::Mcp(m) => match m.cmd {
            Some(McpCmd::Install { target, path, yes }) => {
                mcp::install::install(&target, &path, yes)
            }
            None => {
                STDOUT_LIVRE.store(false, Ordering::SeqCst);
                mcp::serve_stdio(registry()).map(|_| Value::Null)
            }
        },
    };
    match r {
        Ok(Value::Null) => {} // servidor MCP encerrado, ou install abortado: ja' se explicou
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
    use clap::CommandFactory;

    #[test]
    fn clap_bem_formado() {
        Cli::command().debug_assert();
    }

    /// O contrato do argv: os cinco subcomandos e as flags que o operador digita hoje.
    #[test]
    fn argv_dos_cinco_subcomandos() {
        let c = Cli::try_parse_from([
            "spellcore",
            "play",
            "shows/x.spell",
            "--loop",
            "--osc-port",
            "9000",
        ])
        .expect("play aceita posicional, --loop e --osc-port");
        match c.cmd {
            Cmd::Play(a) => {
                assert_eq!(a.file, "shows/x.spell");
                assert!(a.looping);
                assert_eq!(a.osc_port, Some(9000));
            }
            _ => panic!("esperava play"),
        }

        match Cli::try_parse_from(["spellcore", "net", "--timeout", "0.5", "--json"])
            .expect("net aceita --timeout e --json")
            .cmd
        {
            Cmd::Net(a) => {
                assert_eq!(a.timeout, 0.5);
                assert!(a.json);
            }
            _ => panic!("esperava net"),
        }
        match Cli::try_parse_from(["spellcore", "net"]).unwrap().cmd {
            Cmd::Net(a) => assert_eq!(a.timeout, 2.0, "default do --timeout"),
            _ => panic!("esperava net"),
        }

        match Cli::try_parse_from(["spellcore", "commands"]).unwrap().cmd {
            Cmd::Commands { name } => assert!(name.is_none(), "`commands` sozinho = todos"),
            _ => panic!("esperava commands"),
        }
        match Cli::try_parse_from(["spellcore", "commands", "cue_go"])
            .unwrap()
            .cmd
        {
            Cmd::Commands { name } => assert_eq!(name.as_deref(), Some("cue_go")),
            _ => panic!("esperava commands"),
        }
        match Cli::try_parse_from(["spellcore", "mcp"]).unwrap().cmd {
            Cmd::Mcp(m) => assert!(m.cmd.is_none(), "`mcp` sozinho = servidor stdio"),
            _ => panic!("esperava mcp"),
        }
        match Cli::try_parse_from(["spellcore", "mcp", "install", "--target", "code", "--yes"])
            .unwrap()
            .cmd
        {
            Cmd::Mcp(McpArgs {
                cmd: Some(McpCmd::Install { target, path, yes }),
            }) => {
                assert_eq!(target, "code");
                assert!(path.is_empty());
                assert!(yes);
            }
            _ => panic!("esperava mcp install"),
        }
        match Cli::try_parse_from(["spellcore", "serve", "--port", "0", "--show", "x.spell"])
            .expect("serve aceita --port, --dir e --show")
            .cmd
        {
            Cmd::Serve(a) => {
                assert_eq!(a.port, 0);
                assert_eq!(a.dir, ".", "default do --dir e' a raiz do repo");
                assert_eq!(a.show.as_deref(), Some("x.spell"));
            }
            _ => panic!("esperava serve"),
        }
        assert!(
            Cli::try_parse_from(["spellcore"]).is_err(),
            "sem subcomando = ajuda"
        );
    }

    /// O operador digita `play`; o registry (e o MCP) so' conhece `play_show`. O schema dos
    /// parametros e' o que vira `inputSchema` da tool.
    #[test]
    fn registry_expoe_play_show_e_net_com_schema() {
        let reg = registry();
        assert!(reg.get("play_show").is_some());
        assert!(reg.get("net").is_some());
        assert!(reg.get("play").is_none(), "sem comando duplicado");
        let sc = reg.schema();
        let e = sc
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["name"] == "play_show")
            .unwrap();
        let props = &e["params"]["properties"];
        assert!(props["file"].is_object());
        assert!(
            props["loop"].is_object(),
            "o campo JSON chama-se loop: {}",
            props
        );
        assert!(props["osc_port"].is_object());
        assert_eq!(
            e["params"]["required"].as_array().unwrap(),
            &vec![json!("file")]
        );
    }

    /// Uma fonte: o `about` do subcomando e o `doc` do registry sao a mesma constante.
    #[test]
    fn about_do_clap_e_doc_do_registry_sao_o_mesmo_texto() {
        let c = Cli::command();
        let about = |n: &str| {
            c.find_subcommand(n)
                .unwrap_or_else(|| panic!("subcomando {}", n))
                .get_about()
                .expect("about")
                .to_string()
        };
        assert_eq!(about("play"), DOC_PLAY);
        assert_eq!(about("net"), DOC_NET);
        let reg = registry();
        assert_eq!(reg.get("play_show").unwrap().doc, DOC_PLAY);
        assert_eq!(reg.get("net").unwrap().doc, DOC_NET);
        assert_eq!(reg.get("graph_check").unwrap().doc, DOC_GRAPH_CHECK);
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
            looping: true,
            loop_in: 1.0,
            loop_out: 2.0,
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
            looping: false,
            loop_in: 0.0,
            loop_out: 0.0,
        };
        let l = status_line(&parado, 0.0);
        assert_eq!(l, "t=   0.00s state=stop cue=-1 frames=0 jit_p99=0.00ms u=");
        assert!(l.is_ascii() && !l.contains('\r'));
    }
}
