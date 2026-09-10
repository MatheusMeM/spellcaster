//! `spellcore` — the R1/R7/R5 CLI. Five subcommands, written with `clap::Parser`:
//!
//!   spellcore play <show.spell> [--loop] [--osc-port N]
//!   spellcore net [--json] [--timeout N]
//!   spellcore commands [name]
//!   spellcore mcp [install --target desktop|code [--path P] [--yes]]
//!   spellcore serve [--port N] [--dir D] [--show x.spell]
//!
//! What holds the logic is the registry; the CLI only calls and prints. The subcommands are NO
//! longer generated at runtime from `Registry::schema()`: `load`, `pause`, `stop`, `locate`,
//! `cue_go`, `transport_state` and `show_get` act on the player ALIVE IN THIS PROCESS and made
//! no sense as a separate process. They stay in the registry, which is what the MCP exposes.
//!
//! `PlayArgs` and `NetArgs` serve both ends: `clap::Args` for the argv and `JsonSchema` +
//! `Deserialize` for the registry (and, through it, for the MCP tools).

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

// ------------------------------------------------------------------ commands

/// One source for the registry `doc` (`/commands`, MCP tools) and for the clap `about`
/// (`spellcore --help`, `spellcore play --help`): both sides read these constants.
const DOC_PLAY: &str = "Plays a .spell show to the end or Ctrl+C.";
const DOC_NET: &str =
    "Scans the network: interfaces, Art-Net nodes, sACN sources, Ether Dream and hints.";
const DOC_GRAPH_CHECK: &str =
    "Compiles the graph of the open show without running it. Returns {nodes, error}.";

#[derive(Args, Deserialize, JsonSchema)]
#[schemars(crate = "engine::schemars")]
struct PlayArgs {
    /// Path of the .spell file.
    file: String,
    /// Repeats the show from the start when it reaches the end.
    #[arg(long = "loop")]
    #[serde(default, rename = "loop")]
    #[schemars(rename = "loop")]
    looping: bool,
    /// Port of the remote transport over OSC (overrides transport.osc_port of the .spell).
    #[arg(long)]
    #[serde(default)]
    osc_port: Option<u16>,
}

#[derive(Args, Deserialize, JsonSchema)]
#[schemars(crate = "engine::schemars")]
struct NetArgs {
    /// Seconds of listening per protocol.
    #[arg(long, default_value_t = def_timeout())]
    #[serde(default = "def_timeout")]
    timeout: f64,
    /// Returns the raw result as JSON instead of the text report.
    #[arg(long)]
    #[serde(default)]
    json: bool,
}

fn def_timeout() -> f64 {
    2.0
}

// ------------------------------------------------------------------ event sink

/// Event output of the Graph: `Cmd` goes to the registry, `Osc` to the show OSC output, the
/// rest to the stderr in one ASCII line.
struct CliSink {
    reg: Registry,
    osc: Option<osc::OscOut>,
}

impl CliSink {
    // ponytail: the sink builds its own `registry::base()` (the transport acts on the live player
    // through `player::current()`, it does not need the CLI registry) ; pass the whole registry
    // when a `cmd` node needs `net` or `play_show`.
    fn new(target: Option<(String, u16)>) -> CliSink {
        let osc = target.and_then(|(h, p)| match osc::OscOut::new(&h, p) {
            Ok(o) => Some(o),
            Err(e) => {
                eprintln!("warning: osc output {}:{} unavailable: {}", h, p, e);
                None
            }
        });
        CliSink {
            reg: engine::registry::base(),
            osc,
        }
    }
}

/// The `Ev::Cmd` `args` plus `feed`, for the `module` node router.
fn com_feed(mut args: Value, feed: &str) -> Value {
    if let Some(o) = args.as_object_mut() {
        o.insert("feed".into(), Value::String(feed.into()));
    }
    args
}

impl EventSink for CliSink {
    fn emit(&mut self, e: &Ev) {
        match e {
            // The graph `module` node emits `<mod>/<cmd>`; the registered command is
            // `<mod>_<cmd>`, with `feed` = the module name.
            // ponytail: convention feed = module name ; a named instance when there are two lasers.
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
                // float32 is what `spellcaster/protocols/osc.py` emits for a bare number.
                Some(o) => {
                    let a: Vec<osc::Arg> =
                        args.iter().map(|v| osc::Arg::Float(*v as f32)).collect();
                    o.send(address, &a);
                }
                None => eprintln!("osc {}: show with no osc output", address),
            },
            // `out.widget` belongs to the GUI: it goes to the bus (nothing happens without `serve`
            // up) and to the stderr, which is the only monitor of `spellcore play`.
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

/// Host/port of the `osc` output of the .spell — the destination of the Graph `out.osc`.
fn osc_target(sh: &show::Show) -> Option<(String, u16)> {
    sh.outputs.iter().find_map(|o| match o {
        show::OutputCfg::Osc(o) => Some((o.host.clone(), o.port)),
        _ => None,
    })
}

// ------------------------------------------------------------ play (headless)

/// One line per second, stable width on `t`, ASCII only. No `\r` and no progress bar: the play
/// output has to survive an `ssh ... | tee` on the Pi.
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

/// False while the MCP server runs: there the stdout is the JSON-RPC channel and one status line
/// takes the protocol down (Python solves it with `contextlib.redirect_stdout`).
static STDOUT_LIVRE: AtomicBool = AtomicBool::new(true);

fn saida(l: &str) {
    if STDOUT_LIVRE.load(Ordering::SeqCst) {
        println!("{}", l);
    } else {
        eprintln!("{}", l);
    }
}

static INT: AtomicBool = AtomicBool::new(false);

/// Ctrl+C becomes a flag; what closes the player is the `play` loop, on the main thread (calling
/// the engine from inside a signal handler is not safe).
#[cfg(windows)]
mod sig {
    use std::sync::atomic::Ordering;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetConsoleCtrlHandler(h: Option<extern "system" fn(u32) -> i32>, add: i32) -> i32;
    }
    extern "system" fn on_ctrl(_ty: u32) -> i32 {
        super::INT.store(true, Ordering::SeqCst);
        1 // TRUE: handled, the process stays alive until the `close()`
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

    // Header with no universes: they only exist after the first frame; what shows them is the
    // status line.
    let st = h.state();
    let dur = st
        .duration
        .map_or("no end".into(), |d| format!("{:.2}s", d));
    saida(&format!("{}: {} fps, {}", name, st.fps, dur));
    // ponytail: it wakes every 200 ms just to check the Ctrl+C and print the status ; make it a
    // player condvar if the status line needs better resolution than 1 s.
    let mut last = Instant::now();
    while !p.wait(Some(Duration::from_millis(200))) {
        if INT.load(Ordering::SeqCst) {
            break;
        }
        if last.elapsed() >= Duration::from_secs(1) {
            last = Instant::now();
            // ponytail: jit_p99 is the one of the last CLOSED `Clock::run` (the Clock only
            // publishes stats at the end of the run) ; make it a live counter if the operator
            // needs the jitter during the show.
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

/// Compiles the graph of the open show without running it: how many nodes it has, or the error.
/// It lives here because only the CLI knows the `script` crate — `graph_get`, which is pure
/// editing, stays in the engine.
fn graph_check(_: NoArgs) -> Result<Value, String> {
    let g = engine::edit::graph();
    match script::graph::Graph::new(&g, Box::new(engine::NullSink)) {
        Ok(gr) => Ok(json!({"nodes": gr.nodes(), "error": Value::Null})),
        Err(e) => Ok(json!({"nodes": 0, "error": e})),
    }
}

/// `play_show`, `net` and `graph_check` live here because only the CLI knows `script` and
/// `protocols`; the rest of the transport comes from `registry::base()`, which acts on the live
/// player (`player::current()`). It is this registry that the MCP exposes as tools.
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
    about = "Spellcaster core: timeline, cues, fx, graph, sACN, Art-Net, OSC, network, MCP",
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Args)]
struct ServeArgs {
    /// HTTP port; 0 = a free one (the `serve http://...` line on the stderr says which).
    #[arg(long, default_value_t = 8000)]
    port: u16,
    /// Root of the static files; default = the repo root, because the `spellgui/web` pages
    /// reference `../../design/tokens` and `../../shows`.
    #[arg(long, default_value = ".")]
    dir: String,
    /// .spell opened at boot, with the player stopped at t=0.
    #[arg(long)]
    show: Option<String>,
}

#[derive(Subcommand)]
enum Cmd {
    #[command(about = DOC_PLAY)]
    Play(PlayArgs),
    #[command(about = DOC_NET)]
    Net(NetArgs),
    /// Lists the registry as JSON: name, doc and schema of each command. With a name, only that
    /// command - it is the `spellcore <cmd> --help` of the verbs that did not become subcommands.
    Commands {
        /// Name of the registry command; empty = all of them.
        name: Option<String>,
    },
    /// MCP server over stdio; `spellcore mcp install` registers the server in Claude.
    Mcp(McpArgs),
    /// Bus: HTTP + WebSocket + MCP at /mcp. It is the process that drives the hardware.
    Serve(ServeArgs),
}

#[derive(Args)]
struct McpArgs {
    #[command(subcommand)]
    cmd: Option<McpCmd>,
}

#[derive(Subcommand)]
enum McpCmd {
    /// Writes the "spellcaster" entry into the Claude config (it asks for confirmation).
    Install {
        /// desktop = claude_desktop_config.json; code = .mcp.json of the current directory.
        #[arg(long, default_value = "desktop")]
        target: String,
        /// Path of the config; empty = the target default.
        #[arg(long, default_value = "")]
        path: String,
        /// Writes without asking.
        #[arg(long)]
        yes: bool,
    },
}

pub fn main() {
    // The MIDI map calls the FULL registry (with `play_show`, `net` and the `laser_*`), not just
    // the engine `base()`.
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
                .ok_or_else(|| format!("unknown command: {}", n)),
        },
        Cmd::Serve(a) => {
            // the stdout of a server is not a data channel: status and log go to the stderr
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
        Ok(Value::Null) => {} // MCP server closed, or install aborted: it has explained itself
        // a string = a finished report (net without --json); the rest goes out as JSON
        Ok(Value::String(s)) => println!("{}", s),
        Ok(v) => println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default()),
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn clap_well_formed() {
        Cli::command().debug_assert();
    }

    /// The argv contract: the five subcommands and the flags the operator types today.
    #[test]
    fn argv_of_the_five_subcommands() {
        let c = Cli::try_parse_from([
            "spellcore",
            "play",
            "shows/x.spell",
            "--loop",
            "--osc-port",
            "9000",
        ])
        .expect("play accepts a positional, --loop and --osc-port");
        match c.cmd {
            Cmd::Play(a) => {
                assert_eq!(a.file, "shows/x.spell");
                assert!(a.looping);
                assert_eq!(a.osc_port, Some(9000));
            }
            _ => panic!("expected play"),
        }

        match Cli::try_parse_from(["spellcore", "net", "--timeout", "0.5", "--json"])
            .expect("net accepts --timeout and --json")
            .cmd
        {
            Cmd::Net(a) => {
                assert_eq!(a.timeout, 0.5);
                assert!(a.json);
            }
            _ => panic!("expected net"),
        }
        match Cli::try_parse_from(["spellcore", "net"]).unwrap().cmd {
            Cmd::Net(a) => assert_eq!(a.timeout, 2.0, "the --timeout default"),
            _ => panic!("expected net"),
        }

        match Cli::try_parse_from(["spellcore", "commands"]).unwrap().cmd {
            Cmd::Commands { name } => assert!(name.is_none(), "`commands` alone = all of them"),
            _ => panic!("expected commands"),
        }
        match Cli::try_parse_from(["spellcore", "commands", "cue_go"])
            .unwrap()
            .cmd
        {
            Cmd::Commands { name } => assert_eq!(name.as_deref(), Some("cue_go")),
            _ => panic!("expected commands"),
        }
        match Cli::try_parse_from(["spellcore", "mcp"]).unwrap().cmd {
            Cmd::Mcp(m) => assert!(m.cmd.is_none(), "`mcp` alone = the stdio server"),
            _ => panic!("expected mcp"),
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
            _ => panic!("expected mcp install"),
        }
        match Cli::try_parse_from(["spellcore", "serve", "--port", "0", "--show", "x.spell"])
            .expect("serve accepts --port, --dir and --show")
            .cmd
        {
            Cmd::Serve(a) => {
                assert_eq!(a.port, 0);
                assert_eq!(a.dir, ".", "the --dir default is the repo root");
                assert_eq!(a.show.as_deref(), Some("x.spell"));
            }
            _ => panic!("expected serve"),
        }
        assert!(
            Cli::try_parse_from(["spellcore"]).is_err(),
            "no subcommand = help"
        );
    }

    /// The operator types `play`; the registry (and the MCP) only knows `play_show`. The schema
    /// of the parameters is what becomes the tool `inputSchema`.
    #[test]
    fn registry_exposes_play_show_and_net_with_schema() {
        let reg = registry();
        assert!(reg.get("play_show").is_some());
        assert!(reg.get("net").is_some());
        assert!(reg.get("play").is_none(), "no duplicated command");
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
            "the JSON field is called loop: {}",
            props
        );
        assert!(props["osc_port"].is_object());
        assert_eq!(
            e["params"]["required"].as_array().unwrap(),
            &vec![json!("file")]
        );
    }

    /// One source: the subcommand `about` and the registry `doc` are the same constant.
    #[test]
    fn clap_about_and_registry_doc_are_the_same_text() {
        let c = Cli::command();
        let about = |n: &str| {
            c.find_subcommand(n)
                .unwrap_or_else(|| panic!("subcommand {}", n))
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
    fn status_line_is_stable() {
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
        // stable width on the `t` field: the line does not change shape from one second to the next
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
