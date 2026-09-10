# spellcore — the Spellcaster core in Rust (R0, R1, R3, R4, R7)

Cargo workspace with the engine, the protocols, the CLI and the bench. No GUI, no Godot, no media.
The Python package `spellcaster/` is still the reference implementation: spellcore has to
reproduce its output byte for byte (fixtures in `tests/conformance/`).

```
spellcore/
  Cargo.toml        workspace (edition 2021, release: lto thin, codegen-units 1, panic abort)
  engine/           clock, universe, timeline (keys/curves), show (.spell v1), registry, edit (editing the open show)
  protocols/        trait Output, sacn, artnet, osc, netscan
  pixelmap/         frame sampling -> DMX bytes per universe (rayon); bin `map_bench`
  mcp/              MCP server (rmcp, stdio) + `mcp install`
  serve/            bus: HTTP + WebSocket JSON-RPC + DMX monitor + MCP at /mcp
  cli/              `spellcore` binary (play, net, commands, mcp, serve); the CLI and the full
                    `registry()` live in `lib.rs`, so that `gui` uses the SAME registry
  gui/              `spellcaster` binary: the window (tao + wry/WebView2) with the bus in
                    process — `spellcaster [show.spell] [--dir ROOT]`
  bench/            Criterion + `jitter` and `throughput` binaries
```

## Build — `target/` NEVER stays in the repo

The repo folder is inside Google Drive: `target/` takes the sync down and fills the account.
Export the target before any cargo command.

```powershell
# PowerShell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cargo test --workspace
cargo build --release
```

```bash
# bash (Git Bash)
export CARGO_TARGET_DIR="$TEMP/spellcore_target"
cargo test --workspace
```

The file `spellcore/.cargo/config.toml` (not versioned, `.gitignore`) pins the same path for
whoever forgets the variable. `target/` is in the repo `.gitignore` as a second barrier.

## Bench

```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cargo run --release -p bench --bin jitter        # Clock at 60 Hz for 10 s: p50/p99/max and drift
cargo run --release -p bench --bin throughput    # 16 sACN + 16 Art-Net at 60 Hz for 10 s: %CPU
cargo run --release -p pixelmap --bin map_bench   # 100 000 px of a 1080p: ms/frame, 2 ms gate
cargo bench -p bench                             # Criterion (curves, packet, timeline)
cargo bench -p pixelmap                          # Criterion (100k px nearest and bilinear)
```

`map_bench` accepts `--pixels N --frames N --width N --height N --bilinear` and exits with code 1
if the p99 per frame goes past 2 ms.

Targets of the PRD table (desktop x64) and what was measured on this machine (Windows 11, R0):

| Metric | Target | Measured |
|---|---|---|
| Jitter between frames, 60 Hz, p99 | < 1 ms | 0.42 ms (max 0.57 ms) |
| Drift over 10 s at 60 Hz | 0 frames | 0 |
| 16 sACN + 16 Art-Net at 60 Hz | < 3 % of one core | 0.78 % |
| Boot to the first DMX frame | < 2 s | 0.002 s |
| RSS at rest | < 60 MB | 5.6 MB |
| `spellcore.exe` release | < 20 MB | 0.95 MB (R0) / 3.8 MB with Rhai and rmcp |
| Pixel mapping, 100 000 px at 60 Hz (CPU) | < 2 ms per frame | 0.105 ms p50, 0.316 ms p99 |

Criterion: `Timeline::apply` of the whole show (219 tracks, 67 161 keyframes) in 3.73 us;
`Keys::eval` 862 ns; `sacn::packet` 44 ns; `artnet::artdmx` 37 ns.

Live conformance (the Rust binary against the Python fixture):

```
C:\Python313\python.exe tests/conformance/capture_sacn.py --secs 3
```

## Dependencies (one line of justification each)

| Crate | Where | Why |
|---|---|---|
| `serde` + `serde_json` | engine, protocols, cli | the `.spell` and `net --json` are JSON; nothing in the stdlib reads JSON |
| `schemars` | engine (registry) | JSON schema of each command, consumed by the CLI and by the MCP tools; re-exported in `engine::schemars` so that `cli` does not pin its own version |
| `clap` (feature `derive`) | cli | argument parser; five subcommands in fixed structs |
| `rmcp` + `tokio` | mcp | official Model Context Protocol SDK; it is async, and the `current_thread` runtime lives only inside `mcp::serve_stdio` |
| `midir` | protocols | MIDI input from any keyboard or control surface; it is RtMidi in Rust (winmm on Windows, ALSA on Linux, CoreMIDI on mac), and talking winmm by hand would be Windows FFI inside the protocol crate |
| `socket2` | protocols | `std::net::UdpSocket` exposes neither `IP_MULTICAST_IF` nor `SO_REUSEADDR`, both required by sACN |
| `criterion` | bench, laser, pixelmap (dev) | statistical measurement of jitter/latency required by the PRD |
| `rayon` | pixelmap | 100 000 px per frame over ~590 independent universes; a work pool without writing one |
| `tao` + `wry` | gui (only `cfg(windows)`) | the window and the WebView2 that Windows 11 already ships; it is Tauri without Tauri (one window, one webview, no native menu, updater or tray to justify the whole framework). Measured cost: ~200 new crates in `Cargo.lock`, all behind `cfg(windows)`. Licenses: `wry` Apache-2.0 OR MIT, `tao` **Apache-2.0 only** — the repository is MIT, and Apache-2.0 is compatible, but it asks for the attribution notice in the package (`packaging/`) |
| `axum` + `tokio` | serve | HTTP, WebSocket and the `tower::Service` of the streamable MCP in a single server; `rmcp` already required hyper/tower, and writing a WS handshake by hand in the server does not pay off |

Nothing else comes in without justification and without measuring the binary size.
The Windows API (`timeBeginPeriod`, `SetThreadPriority`, `GetProcessTimes`) is declared with
`extern "system"` directly — it avoids the whole of `windows-sys` for three symbols.

## Contracts between the crates (fixed before writing code)

### `engine::clock`

```rust
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State { Stop, Play, Pause }

#[derive(Clone)]                       // clonable: transport from another thread acts on the same clock
pub struct Clock { /* Arc<inner> */ }

impl Clock {
    pub fn new(fps: u32) -> Clock;
    pub fn fps(&self) -> u32;
    pub fn time(&self) -> f64;                       // seconds
    pub fn state(&self) -> State;
    pub fn play(&self);
    pub fn pause(&self);
    pub fn stop(&self);
    pub fn locate(&self, t: f64);
    /// Calls f(t) every 1/fps s until stop() or t >= duration. Blocks the calling thread.
    /// High priority + timeBeginPeriod(1) on Windows; sleeps up to the margin before the target, spins for the rest.
    /// The margin is CALIBRATED at runtime from the measured sleep overshoot (EWMA, clamped to 0.3-2 ms):
    /// a fixed 1 ms margin costs ~6 % of one core at 60 Hz in spin alone. It is the only knob of the clock.
    pub fn run<F: FnMut(f64)>(&self, f: F, duration: Option<f64>);
    /// Jitter of the last run: (p50, p99, max) in seconds, and drift = frames lost.
    pub fn stats(&self) -> Stats;
}

pub struct Stats { pub p50: f64, pub p99: f64, pub max: f64, pub frames: u64, pub drift: i64 }
```

### `engine::universe`

```rust
pub struct Universe { pub number: u16, pub data: [u8; 512] }
impl Universe {
    /// addr is 1-based; clamp 0..255; ignores whatever goes past 512 (same as Python).
    pub fn set(&mut self, addr: u16, values: &[f64]);
    pub fn set_bytes(&mut self, addr: u16, values: &[u8]);
}

pub struct Universes { /* Vec<Universe> sorted by number, no allocation on the hot path */ }
impl Universes {
    pub fn new() -> Universes;
    pub fn get_or_create(&mut self, number: u16) -> &mut Universe;
    pub fn get(&self, number: u16) -> Option<&Universe>;
    pub fn iter(&self) -> impl Iterator<Item = &Universe>;
    pub fn numbers(&self) -> Vec<u16>;
}
```

### `engine::timeline`

```rust
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Curve { Linear, Hold, In, Out, InOut, Bezier }   // curve of the segment that ARRIVES at the keyframe
impl Curve {
    pub fn from_str(s: &str) -> Curve;                    // unknown -> Linear
    pub fn ease(self, u: f64, c: (f64, f64)) -> f64;      // u in 0..1
}
pub const BEZ: (f64, f64) = (0.42, 0.58);

pub enum Value { Num(f64), List(Vec<f64>), Text(String) }

pub struct Keyframe { pub t: f64, pub value: Value, pub curve: Curve, pub c: (f64, f64) }

pub struct Keys { /* sorted keys + times: Vec<f64> */ }
impl Keys {
    pub fn new(keys: Vec<Keyframe>) -> Keys;
    pub fn len(&self) -> usize;
    /// Evaluation by partition_point (binary search). No allocation for Value::Num.
    pub fn value(&self, t: f64) -> Option<&Value>;        // step/text
    pub fn eval(&self, t: f64, out: &mut Vec<f64>) -> bool;  // interpolated; false if empty
    pub fn crossed(&self, t0: f64, t1: f64) -> &[Keyframe];
}

pub struct Track { pub kind: String, pub universe: u16, pub address: u16, pub keys: Keys, /* .. */ }

pub struct Timeline { pub fps: u32, pub duration: Option<f64>, pub tracks: Vec<Track> }
impl Timeline {
    pub fn new(show: &Show) -> Result<Timeline, String>;
    /// Writes the DMX tracks (types "dmx" and "artnet") into the Universes at instant t. Zero allocation.
    pub fn apply(&mut self, universes: &mut Universes, t: f64);
}
```

Track types in R0: `dmx` and `artnet` (same resolver, different destinations).
`pyfx` is Python and does not exist in Rust — the MED GRUPO show comes in baked into `dmx` tracks
(see `tests/conformance/gen.py`). A track of an unknown type is ignored with a warning.

### `engine::show` (`.spell` version 1, the same file as Python)

```rust
#[derive(Serialize, Deserialize)]
pub struct Show {
    pub name: String, pub fps: u32, pub duration: Option<f64>,
    pub outputs: Vec<OutputCfg>, pub tracks: Vec<serde_json::Value>,
    pub version: u32, /* the rest preserved in #[serde(flatten)] extra */
}
pub enum OutputCfg { Sacn { universes: Vec<u16>, priority: u8, source_name: String, interfaces: Option<Vec<String>> },
                     ArtNet { targets: Option<Vec<String>>, broadcast: bool },
                     Unknown(String) }
pub fn load(path: &Path) -> Result<Show, String>;
pub fn save(path: &Path, show: &Show) -> Result<(), String>;
```

`load` accepts `version` <= 1; a higher `version` is an error. `save` writes JSON compatible with
Python.

### `engine::registry`

```rust
pub struct Command {
    pub name: String, pub doc: String, pub schema: serde_json::Value,
    pub f: Box<dyn Fn(serde_json::Value) -> Result<serde_json::Value, String> + Send + Sync>,
}
pub struct Registry { /* Vec<Command> in insertion order */ }
impl Registry {
    pub fn new() -> Registry;
    pub fn add<A: schemars::JsonSchema + serde::de::DeserializeOwned>(
        &mut self, name: &str, doc: &str,
        f: impl Fn(A) -> Result<serde_json::Value, String> + Send + Sync + 'static);
    pub fn get(&self, name: &str) -> Option<&Command>;
    pub fn schema(&self) -> serde_json::Value;                 // [{name, doc, params, mcp}]
    pub fn call(&self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, String>;
    pub fn iter(&self) -> impl Iterator<Item = &Command>;
}
```

`add` derives the schema with `schemars::schema_for!(A)`. Every product command goes through here;
CLI, OSC-API, GUI and MCP are clients. An error is a `String` — no `anyhow` in R0.

### `protocols`

```rust
pub trait Output: Send {
    fn send(&mut self, universe: u16, data: &[u8; 512]);
    fn close(&mut self);
}
```

`close(&mut self)` and not the `close(self)` of the PRD: `Box<dyn Output>` requires object safety.

```rust
// protocols::sacn
pub const PORT: u16 = 5568;
pub fn mcast(universe: u16) -> std::net::Ipv4Addr;                 // 239.255.{u>>8}.{u&255}
pub fn packet(universe: u16, data: &[u8], cid: &[u8; 16], seq: u8,
              source_name: &str, priority: u8) -> Vec<u8>;         // same as sacn_packet.bin
pub fn parse(pk: &[u8]) -> Option<Packet>;                          // data (vector 4) and discovery (8)
pub fn interfaces() -> Vec<Ipv4Addr>;                               // local IPv4 + 127.0.0.1
pub struct SacnOut;
impl SacnOut {
    pub fn new(universes: &[u16], ifaces: Option<Vec<Ipv4Addr>>) -> std::io::Result<SacnOut>;
    pub fn with(universes: &[u16], priority: u8, source_name: &str,
                ifaces: Option<Vec<Ipv4Addr>>) -> std::io::Result<SacnOut>;
    pub fn cid(&self) -> [u8; 16];
}
impl Output for SacnOut { .. }
pub struct SacnIn;
impl SacnIn { pub fn new(universes: &[u16]) -> io::Result<SacnIn>;
              pub fn get(&self, universe: u16) -> Option<[u8; 512]>;
              pub fn close(&mut self); }

// protocols::artnet
pub const PORT: u16 = 6454;
pub fn port_address(universe: u16) -> u16;                          // universe - 1
pub fn artdmx(universe: u16, data: &[u8], sequence: u8) -> Vec<u8>; // same as artnet_packet.bin
pub fn artpoll() -> Vec<u8>;
pub fn artsync() -> Vec<u8>;
pub fn parse(pkt: &[u8]) -> Option<Packet>;
pub struct ArtNetOut;
impl ArtNetOut { pub fn new(targets: Option<Vec<String>>, broadcast: bool) -> io::Result<ArtNetOut>;
                 pub fn sync(&mut self); }
impl Output for ArtNetOut { .. }

// protocols::osc
pub enum Arg { Int(i32), Long(i64), Float(f32), Double(f64), Str(String), Blob(Vec<u8>),
               Bool(bool), Nil, Impulse }
pub fn message(address: &str, args: &[Arg]) -> Vec<u8>;
pub fn bundle(elements: &[Vec<u8>], tt: u64) -> Vec<u8>;
pub const IMMEDIATE: u64 = 1;
pub fn timetag(secs: f64) -> u64;
pub fn parse(data: &[u8]) -> Option<Parsed>;                        // Msg | Bundle
pub fn matches(pattern: &str, address: &str) -> bool;               // * ? [a-z] [!a-z] {a,b}
pub struct OscOut; impl OscOut { pub fn new(host: &str, port: u16) -> io::Result<OscOut>;
                                 pub fn send(&self, address: &str, args: &[Arg]);
                                 pub fn close(&mut self); }
pub struct OscIn;  impl OscIn  { pub fn new(port: u16) -> io::Result<OscIn>;
                                 pub fn on(&mut self, pattern: &str,
                                           f: impl Fn(&str, &[Arg]) + Send + 'static);
                                 pub fn close(&mut self); }

// protocols::netscan
pub struct Iface { pub name: String, pub ip: String, pub mask: String, pub gateway: Option<String> }
pub fn interfaces() -> Vec<Iface>;                                  // ipconfig / ip -j addr
pub fn parse_ipconfig(text: &str) -> Vec<Iface>;
pub fn parse_ip_addr(text: &str, route_text: &str) -> Vec<Iface>;
pub fn scan_artnet(timeout: Duration, ifaces: &[Iface]) -> Vec<Node>;
pub fn scan_sacn(timeout: Duration, ifaces: &[Iface]) -> Vec<Source>;
pub fn scan_etherdream(timeout: Duration) -> Vec<Dac>;
pub fn suggest(ifaces: &[Iface]) -> Vec<String>;                    // rule 2.x / 10.x + netsh
pub fn scan_all(timeout: Duration) -> Scan;                         // Serialize for `net --json`
pub fn report(s: &Scan) -> String;
```

Each network output runs on its own thread with a queue of 2 frames per universe; when the queue
fills up, the old frame is dropped (`send` never blocks the engine).

### `serve` — bus

```
spellcore serve [--port 8000] [--dir .] [--show x.spell]
```

One process touches the hardware; every page and every AI talk to it. Port `0` = a free one; on
start-up, the server prints **on stderr** the line `serve http://127.0.0.1:<port>`.

Contract (the other work fronts read from here, word for word):

```
HTTP  GET /commands -> Registry::schema()   GET /show -> show_get full   GET /<file> -> <--dir>/<file>
WS    /ws  request  {"id":7,"cmd":"locate","args":{"t":12.5}}
           response {"id":7,"result":...,"rev":n} | {"id":7,"error":"text","rev":n}
           event    {"event":"transport","data":<TransportState>} | {"event":"show","data":{"rev":n}} | {"event":"log","data":{"text":...}} | {"event":"widget","data":{"id","prop","value"}}
           binary   topic:u8 | universe:u16 LE | 512 bytes   (topic 1 = output dmx,
                    topic 2 = INPUT dmx, the universes of `show.inputs`)
The `input {key, value}` command of the registry feeds FrameHook::input of the live player (keys "widget:go", "key:Space", "module:laser/stat/fps").
```

| Detail | Rule |
|---|---|
| header | every HTTP response carries `Cache-Control: no-store` |
| `--dir` | default `.` = the **repository root**, not `spellgui/web`: the pages reference `../../design/tokens/spellcaster.css` and `../../shows/*.spell`, so the page opens at `/spellgui/web/index.html` and what it references comes from the same server. In exchange, the whole repository (`.git` included) is readable on 127.0.0.1 — acceptable while the socket is loopback only |
| static | plain relative segments only: `..`, an empty segment, `\` and `:` are refused with 403 before touching the disk; the path is not percent-decoded. `/` = `index.html` |
| command | each WS request calls `Registry::call`; a command error comes back as `{"id","error"}` and does **not** drop the connection |
| `play_show` | it is in the `BACKGROUND` list of the `mcp` crate: it runs on a thread and the response comes back at once, with `"<cmd> started in the background"`; an error becomes a `log` event |
| `rev` | a single counter, the engine one (`engine::edit::rev()`). There is no list of read commands: `serve` reads `rev()` before and after each call, **every** response carries the `rev` from after, and the broadcast `{"event":"show","data":{"rev":n}}` goes out **only** when the number changed. `load` and `show_get {file}` swap the whole show and therefore bump it |
| `transport` | on every state change and at 10 Hz while the player runs (polling of `player::current()`) |
| `widget` | `out.widget` of the graph (the `Ev::Widget` the CLI sink receives) becomes `{"event":"widget","data":{"id","prop","value"}}` |
| monitor | a global `FrameHook` copies the universes of the frame, at 40 Hz at most and only when there is a WS client; it goes out as a 515-byte binary frame. Under the same ceiling goes **topic 2**, the last frame received on each universe of `show.inputs` (`Handle::input_frames`) |
| `show` with no command | the recording (`rec.rs`) edits the show INSIDE the player frame, without going through any request. A task polls `edit::rev()` at **4 Hz** and emits `{"event":"show","data":{"rev":n}}` when the number changed without a WS command having already announced it (`St::visto` avoids the doubled event) |
| `/mcp` | `StreamableHttpService` of `rmcp` (feature `transport-streamable-http-server`) over the same `Spell` as the stdio one, with no session and with a JSON response: a `POST /mcp` of an `initialize` returns the JSON-RPC directly |
| `--show` | `load` (it is what `GET /show` reads) and the player stopped at `t=0`; it is released with the `resume` command |

`serve` is a CLI subcommand, not a registry command: it is the one that builds the `Registry`
(with `play_show` and `net`) and passes it to `serve::serve`. The `serve` crate has no product
logic — only transport.

**Out of scope, and why:** authentication and TLS (that is why the socket opens **on 127.0.0.1
only**: with no token, opening the LAN would hand the hardware to whoever is on the wifi —
`--host` comes in together with the token); mDNS; more than one show per process (one `OPEN`, one
`CURRENT`).

### Conformance fixtures (`tests/conformance/`, generated by `gen.py`)

| File | Content |
|---|---|
| `medgrupo_u1.bin` | `u32 LE nframes` + `nframes x 512` bytes of universe 1, t = n/30 |
| `sacn_packet.bin` | E1.31 packet: universe 1, seq 0, CID `00..0f`, "Spellcaster", prio 100, data `[0..255,0..255]` |
| `artnet_packet.bin` | ArtDmx: universe 1, seq 0, same data |
| `../../shows/medgrupo_r0.spell` | the same show with the pyfx baked into 219 `dmx` tracks (67 161 keyframes) |

To regenerate: `C:\Python313\python.exe tests/conformance/gen.py`.

## `laser::trace` — PHOSPHOR (bitmap -> contour -> ILDA)

The half of NDI->ILDA that does not depend on an SDK (`design/FUNCOES/ndi-ilda.md`, the
"Contornos" algorithm). Pure Rust, no new dependency; the input is a raw RGBA buffer, whether it
comes from NDI, from Spout or from a file.

```rust
/// `color: None` samples the source pixel under each point.
pub struct Opts { threshold: u8, epsilon: f32, max_points: usize, invert: bool, color: Option<(u8, u8, u8)> }
/// Paths in ILDA coordinates, one per object, closed; before blanking/optimize/safety.
pub fn paths(rgba: &[u8], w: usize, h: usize, o: &Opts) -> Vec<Vec<Point>>;
/// Frame ready for the `Feed`: paths + blanked jump + `optimize` + default `safety`.
pub fn trace(rgba: &[u8], w: usize, h: usize, o: &Opts) -> Vec<Point>;
```

Mask by BT.601 luma - outer contour by Moore boundary following, one path per object (a hole does
not become a path) - Ramer-Douglas-Peucker simplification with `epsilon` in pixels of the input -
proportional trimming per path down to `max_points` (counted **before** `optimize`) - ordering by
nearest neighbour - ceiling of `MAX_PATHS = 2000` paths per frame. The image comes in centred with
the aspect ratio preserved, in the ILDA range +-32767 with Y up. Deterministic: same input, same
bytes.

```
trace in.rgba WxH out.ild [--threshold N] [--epsilon F] [--max-points N] [--invert] [--sampled]
```

Measured in release: **4.4 ms** per 1920x1080 frame with 20 objects (goal 8 ms). The cost is in the
two whole-frame passes (mask and candidate sweep, 2 Mpx each); the vectorization itself only
touches the contour pixels — one object or twenty takes the same time. Test: `laser/tests/trace.rs`,
which generates the RGBA inputs in the file itself.

## CLI

```
spellcore play <show.spell> [--loop] [--osc-port N]   plays the show (sACN / Art-Net as per "outputs")
spellcore net [--json] [--timeout N]                  network scan + hints
spellcore commands                                    lists the registry (name, doc, schema)
spellcore mcp                                         MCP server over stdio
spellcore mcp install --target desktop|code [--yes]   registers the server with Claude
spellcore serve [--port N] [--dir D] [--show S]       HTTP + WebSocket + MCP bus
```

The five subcommands are fixed `clap::Args` structs. `PlayArgs` and `NetArgs` serve both ends:
`clap` for the argv and `JsonSchema` + `Deserialize` for the registry. The transport (`load`,
`show_get`, `pause`, `stop`, `locate`, `cue_go`, `transport_state`) stays in the registry but is
**not** a subcommand: it acts on the live player IN THIS process, and a second process has no
player at all. The ones who use them are the MCP (and, later, the GUI).

---

# R1 — fixed contracts (orchestrator, before any code)

Scope of R1: cues, the complete `.spell`, `fx` track in Rhai, Graph runtime, Player with remote
transport over OSC, headless CLI `play`. No GUI, no media, no laser (another agent writes
`spellcore/laser/`; nothing here touches that directory).

New workspace member: `script/` (crate `script`). Dependency graph of R1:

```
protocols  (self-contained)
engine     -> protocols, serde, serde_json, schemars
script     -> engine, rhai, serde_json
cli        -> engine, protocols, script, clap
bench      -> engine, protocols, script
```

`engine -> protocols` is new and deliberate: the Player opens the outputs of the `.spell` and
listens to the remote transport over OSC, exactly as `spellcaster/player/player.py` imports
`..protocols`. `protocols` still does not know `engine`; there is no cycle. The engine still does
not know GUI, MCP, Rhai and laser.

## Evaluation order of one frame (fixed; it is what the conformance measures)

0. The input queue: the `cue_go`/`locate`/`input` requests **and** the MIDI events
   (`midi::pump()`). It is not an evaluation step — it is what arrives from outside before the
   frame starts.
1. `Timeline::apply(&mut universes, t)` — `dmx` and `artnet` tracks.
2. Each `FrameHook` in the order it was registered (`fx` tracks in `.spell` order, then the Graph).
3. Side effect: the **recording** (`rec::tick` — an armed track reads the input universe and writes
   a keyframe when the value changes), then `osc` and non-Capture `media` (they send when the value
   changes) and `cue` (`crossed(prev, t)` fires `CueList::go`).
4. `CueList::update(t)` writes the current snapshot into the Universes.
5. The programmer (`player::Prog`): the manual operator override, HTP per channel, on top of the
   timeline **and** of the live cue — the operator overrides what the cue is holding.
6. The **global** hooks (`player::hook_global`, the `serve` monitor): the frame already complete,
   with the programmer inside, before it goes out on the network.
7. I/O: each universe written goes to all the outputs.

Same as the `_tick` + `_side` of Python.

## Track types in the `.spell` (R1)

| `type` | Fields | Semantics |
|---|---|---|
| `dmx` | `universe`, `address`, `keys` | R0, unchanged |
| `artnet` | same | R0, unchanged |
| `osc` | `address` (text), `keys`, `args?` | sends over OSC when the value changes (`last != v`) |
| `media` | `universe`, `address`, `clip`, `keys` (text `play`/`replay`/`stop`) | Capture: ch1 = 10/20/28, ch2 = `clip`. With a `"player"` other than `"capture"` it becomes an OSC track (`address/value`) |
| `cue` | `keys` (text `GO` or a number = index) | `crossed(prev, t)` fires the cue |
| `fx` | `script` (path relative to the `.spell`), `universe` | Rhai script with state persisted between frames |
| `fixture` | reserved | parsed and ignored with a warning (the F2 of Python resolves it; Rust does not in R1) |

A track of an unknown type is still ignored with a warning.

## DMX input (`engine::input`) and recording (`engine::rec`)

The `.spell` gains `inputs`, sibling of `outputs`:

```json
"inputs": [{"type": "sacn", "universe": 1}, {"type": "artnet", "universe": 2}]
```

`Player::new` opens the inputs together with the outputs: one `SacnIn` with all the declared
`sacn` universes (multicast + the unicast on 127.0.0.1 that `SacnOut` also sends) and, if there is
`artnet`, an `ArtNetIn` on 6454. An unknown type becomes a warning, not an error; with no `inputs`
no socket comes up. The input is a PARALLEL buffer: there is no HTP merge with the output — the
passthrough is on the "later" list of the ROADMAP.

| Command | Does | Returns |
|---|---|---|
| `input_get(universe=1)` | last frame received on that input universe | `{universe, data:[512]}` |
| `rec_arm(track, on=true)` | arms/disarms the recording of a `dmx`/`artnet` track of the OPEN show | `{track, on, tracks}` |
| `rec_state()` | tracks armed in this process | `{recording, tracks}` |

With the transport playing, each frame reads the channels of the track on the input universe and,
**only when the value changes**, writes a keyframe through `edit::key_put` — the same funnel as
the `key_set` command, so `rev` goes up and every bus client receives `show {rev}`. The width (how
many channels) comes from the first list keyframe of the track; with no list, one channel. Curve of
the recorded keyframe: `linear`. Stopping the transport disarms, on the entering edge into Stop
(the transport thread) and in `Player::close`. Arming with the transport stopped survives until
play — it is the normal operator path. If writing the keyframe fails (the track vanished in a
`track_del`, or the show was swapped), the track is disarmed right there, instead of repeating the
error every frame.

The arming lives in the process, not in the `.spell`: `rec_arm` is not an edit and does not bump
`rev`.

Video (NDI/GStreamer) does **not** come in here: R2 is blocked until the SDKs are installed.

Tests: `engine/tests/rec.rs` (its own binary, `OPEN` and `CURRENT` are process globals) records a
loopback sACN source on universe 7 and checks the values in `show_get`; `protocols` has the
`loopback_out_in` of `ArtNetIn`.

## `engine::hook` (new module)

```rust
use crate::universe::Universes;

/// Writes into the Universes once per frame, after Timeline::apply.
/// It is through here that `script` (Rhai fx and Graph) plugs in without the engine knowing Rhai.
pub trait FrameHook: Send {
    fn frame(&mut self, t: f64, uni: &mut Universes);
    /// Input event for the Graph. Key: "widget:go", "key:Space", "osc:/spell/go",
    /// "midi:144/60", "marker:pico". Queued; consumed on the next frame(). Default: ignores.
    fn input(&mut self, _key: &str, _value: f64) {}
    /// locate/stop: clears the persistent state. Default: no-op.
    fn reset(&mut self, _t: f64) {}
}

/// Event output of the Graph to the world. One implementation in `cli` (registry command,
/// OscOut, stderr); the engine only declares the trait because it knows neither the live registry
/// nor the GUI.
#[derive(Clone, Debug, PartialEq)]
pub enum Ev {
    Cmd { name: String, args: serde_json::Value },   // `cmd` node
    Osc { address: String, args: Vec<f64> },         // `out.osc`
    Widget { id: String, prop: String, value: f64 }, // `out.widget`
    Param { target: String, value: f64 },            // `out.param` ("fixture.channel")
    Notify { text: String },                         // `out.notify`
}
pub trait EventSink: Send { fn emit(&mut self, e: &Ev); }
```

`Ev` allocates a `String`: events are rare (a GO, an incoming OSC), they do not happen per frame.
`// ponytail: Ev with String ; make it a catalog index if some graph starts emitting per frame.`

## `engine::cues` (port of `spellcaster/timeline/cues.py`)

```rust
pub struct Cue { pub name: String, pub fade: f64, pub wait: f64, pub follow: bool,
                 pub values: Vec<((u16, u16), Vec<f64>)> }   // JSON order preserved
pub struct CueList { /* state, index, current cue, from, pending */ }
impl CueList {
    pub fn new(specs: &[serde_json::Value]) -> CueList;
    pub fn len(&self) -> usize;
    pub fn index(&self) -> i32;                       // -1 = none fired
    /// Fires the next cue (or the one at the given index). The fade starts after its `wait`.
    pub fn go(&mut self, t: f64, index: Option<usize>) -> bool;
    /// Advances the fade and writes the current snapshot into the Universes. Zero allocation per frame.
    pub fn update(&mut self, t: f64, uni: &mut Universes);
    pub fn reset(&mut self);
}
```

Key `"1/100"` -> `(1, 100)`; `"100"` -> `(1, 100)` (the `key()` of Python). Linear fade, `u = 1`
when `fade <= 0`; on reaching `u >= 1` with `follow`, it fires the next one.

## `engine::player`

```rust
pub struct Player { /* timeline, cues, hooks, outputs, Arc<Shared> */ }

#[derive(Clone)]
pub struct Handle { /* Arc<Shared>: clock + cue queue + counters */ }

#[derive(Serialize)]
pub struct TransportState { pub t: f64, pub state: &'static str, // "stop"|"play"|"pause"
                            pub cue: i32, pub frames: u64, pub fps: u32,
                            pub duration: Option<f64>, pub universes: Vec<u16> }

impl Player {
    /// Loads timeline + cues and opens the outputs of `show.outputs` (sacn, artnet, osc).
    /// `base` = directory of the .spell (paths of `fx`/clips are relative to it).
    pub fn new(show: Show, base: PathBuf, looping: bool) -> Result<Player, String>;
    /// Script/graph hooks, before `start()`, in the order they should run.
    pub fn hook(&mut self, h: Box<dyn FrameHook>);
    pub fn handle(&self) -> Handle;
    pub fn clock(&self) -> Clock;
    /// Brings the transport thread up and, if `osc_port` (argument or `transport.osc_port`),
    /// the OscIn with /spellcaster/play|pause|stop|locate f. Registers this player as the CURRENT.
    pub fn start(&mut self, osc_port: Option<u16>) -> Result<(), String>;
    pub fn wait(&self, timeout: Option<Duration>) -> bool;   // blocks until stop/end
    pub fn close(&mut self);                                  // idempotent; clears the CURRENT
}

impl Handle {
    pub fn play(&self); pub fn pause(&self); pub fn stop(&self);
    pub fn locate(&self, t: f64);
    pub fn cue_go(&self, index: Option<usize>);
    pub fn state(&self) -> TransportState;
}

/// Live player in this process (the `CURRENT` of Python). Used by the registry commands.
pub fn current() -> Option<Handle>;
```

`locate`/`stop` call `reset()` on every hook and `CueList::reset()`. `looping` with no `duration`
does not repeat (there is no end). The remote transport over OSC never touches the Universes
directly.

## Registry — the 59 commands in one table

Audit of September 2026, updated in the integration of round 2 (`spellcore commands`). One line
per command: the arguments with type and default (the schema `schemars` generates from the `Args`
struct, the same one that comes out in `GET /commands`, in the MCP tools and in
`spellcore commands <name>`), what it does, what it returns and what fires the command in the GUI.
The sections below are still the explanation; this table is the index.

Reading rules: `arg:type` is required, `arg:type?` is optional with no default, `arg:type=v` has
default `v`. "Fires in the GUI" names the page (`index.html` = TIMELINE, `teatro.html` = TEATRO,
`patchbay.html` = PATCHBAY, `laser.html` = LASER, `face.html` = FACE, `midi.html` = MIDI,
`help.html` = AJUDA) and the key from `design/SHORTCUTS.md` when there is one; `—` is a command
that today only the AI (MCP), OSC and `help.html` call.

| Command | Arguments | Does | Returns | Fires in the GUI |
|---|---|---|---|---|
| `load` | `file:string` | opens the `.spell` **and validates the timeline** | `{name, fps, duration, tracks, ignored}` | PATCHBAY: path field + `Abrir`; `serve --show` |
| `show_get` | `file:string=""`, `full:boolean=false` | opens (or reuses) and summarizes; `full` returns the whole `.spell` | summary or the show | TIMELINE, PATCHBAY and TEATRO at boot; `GET /show` |
| `resume` | — | releases the paused player | transport state | TIMELINE: `Play`, `Space`, `L` |
| `pause` | — | pauses the player | transport state | TIMELINE: `Pause`, `Space`, `K` |
| `stop` | — | stops the player | transport state | TIMELINE: `Stop` |
| `locate` | `t:number` | jumps to `t` seconds | transport state | TIMELINE: ruler, `←`/`→`, `Home`/`End` |
| `cue_go` | `index:integer?` | fires the next cue, or the one at the given index | transport state | TEATRO: `GO` (`Enter`) |
| `transport_state` | — | reads the transport without touching anything | `{t, state, cue, frames, fps, duration, universes}` | — (the GUI receives the `transport` event) |
| `loop_set` | `on:boolean` | turns the player loop on/off over the In-Out range of the open show | transport state | TIMELINE: `Loop` (`Ctrl+L`) |
| `input` | `key:string`, `value:number=0` | delivers an event to the hooks of the player (the Graph) | `{key, value}` | FACE: every widget |
| `input_get` | `universe:integer=1` | last frame received on the INPUT universe (`show.inputs`) | `{universe, data:[512]}` | — (the previz uses the binary frame of the bus) |
| `rec_arm` | `track:integer`, `on:boolean=true` | arms/disarms the recording of a `dmx` track of the OPEN show | `{track, on, tracks}` | TIMELINE: `Rec arm`, `R` |
| `rec_state` | — | tracks armed in this process | `{recording, tracks}` | TIMELINE: at boot and on every `show` event |
| `show_new` | — | clears the open show (sACN on universe 1, 60 s) | the whole show | — |
| `show_set` | `data:any` | **imports** a whole show (object or JSON text) | the whole show | — |
| `show_save` | `file:string=""` | saves; with no `file`, at the path of the last one opened | the path | TIMELINE: `Salvar`, `Ctrl+S` |
| `track_add` | `type:string="dmx"`, `universe:integer=1`, `address:integer=1`, `name:string=""` | appends an empty track | the index | TIMELINE: `+Track` |
| `track_del` | `index:integer` | removes the track | the removed track | TIMELINE: `-Track` |
| `key_set` | `track:integer`, `t:number`, `value:any=null`, `curve:string="linear"` | creates or replaces the keyframe at `t` | the keys of the track | TIMELINE: `Ctrl+K`, drag, `Ctrl+V` |
| `key_del` | `track:integer`, `t:number` | deletes the keyframe at `t` (1 ms tolerance) | how many were removed | TIMELINE: `Delete`, `Ctrl+X` |
| `cue_set` | `index:integer?`, `name:string=""`, `fade:number=0`, `wait:number=0`, `follow:boolean=false`, `values:object={}` | creates (with no `index`) or replaces a cue | the index | TEATRO: scene list |
| `cue_del` | `index:integer` | removes the cue | the removed cue | TEATRO: delete scene |
| `patch_add` | `name:string`, `profile:string`, `universe:integer=1`, `address:integer=1` | patches and revalidates the whole patch | the patch grid | TEATRO: `Adicionar` |
| `patch_del` | `name:string` | takes the fixture out of the patch | the removed entry | TEATRO: delete fixture |
| `patch_check` | — | patch grid + the first error | `{rows, error}` | TEATRO: the grid |
| `profiles` | — | names of the `.json` in `profiles/` | list of names | TEATRO: profile select |
| `show_patch` | `ops:array`, `rev:integer?` | JSON Patch (RFC 6902) on the open show; all or nothing | `{rev, undo}` | TIMELINE, PATCHBAY and TEATRO: **every** edit |
| `graph_get` | — | the `graph` of the show | `{nodes, edges}` | — (the PATCHBAY reads the graph through `show_get full`) |
| `face_get` | — | the inline face, or `faces/<name>.face.json` | the face or `null` | — (the FACE today fetches the `.face.json` directly) |
| `profile_get` | `name:string` | the whole profile (channels, `ranges`, `wheel`) | the JSON of the profile | TEATRO: fixture widgets |
| `level_set` | `universe:integer=1`, `address:integer`, `values:array=[]` | writes into the programmer override (HTP) | how many channels | — (the TEATRO writes through `fixture_set`) |
| `level_clear` | `universe:integer?` | releases the override of one universe, or of all | how many channels were freed | TEATRO: `Solta` |
| `level_get` | `universe:integer?` | the current override | `{"u/end": v}` | — |
| `cue_capture` | `name:string=""`, `fade:number=0`, `wait:number=0`, `follow:boolean=false` | the override becomes a new cue and the override is released | the index | TEATRO: `Capturar` |
| `fixture_set` | `name:string`, `channel:string`, `value:number` | resolves fixture + profile channel and calls `level_set` | `{universe, address, value}` | TEATRO: fixture sliders |
| `module_add` | `file:string=""`, `data:any=null` | validates a `module.json` and puts it in the table of live modules | `{name, version}` | — |
| `module_del` | `name:string` | takes the module out of the table | the removed manifest | — |
| `module_list` | — | live modules | `[{name, type, version}]` | PATCHBAY: catalog of the `module` node |
| `module_get` | `name:string` | the whole manifest | `{name, type, version, parameters, values, commands}` | PATCHBAY: the `module` node |
| `midi_ports` | — | MIDI input ports of the machine and which one is open | `{ports, open}` | MIDI: `Portas` |
| `midi_open` | `port:string=""` | opens by name, part of the name or index; writes `midi_port` into the open show | `{open}` | MIDI: `Abrir` |
| `midi_close` | — | closes the port and removes `midi_port` from the open show | `{open:null}` | MIDI: `Fechar` |
| `midi_map` | `key:string`, `cmd:string`, `args:any?` | binds a key/CC to a command in the open show | the map | MIDI: `LEARN` and the table row |
| `midi_unmap` | `key:string` | unbinds the key in the open show | the map | MIDI: delete the row |
| `midi_maps` | — | the `key -> command` map of the open show | the map | MIDI: the table |
| `midi_last` | — | last MIDI key received | `{key, value, raw, seq}` | MIDI: the live key (4 Hz) |
| `midi_learn` | — | waits up to 5 s for the next key and returns its key | `{key, value}` | MIDI: `LEARN` |
| `play_show` | `file:string`, `loop:boolean=false`, `osc_port:integer?` | **brings up** a player and plays to the end or Ctrl+C | `{name, frames, jitter_p99_ms, jitter_max_ms, drift}` | TIMELINE: `Play` with no live player; CLI `spellcore play` |
| `net` | `timeout:number=2`, `json:boolean=false` | scans the network (interfaces, Art-Net, sACN, Ether Dream) | text report, or the raw scan | CLI `spellcore net` |
| `graph_check` | — | compiles the graph of the open show without running it | `{nodes, error}` | PATCHBAY: on every graph edit |
| `laser_dacs` | `timeout:number=2` | looks for DACs (Ether Dream by UDP beacon and, with no beacon, by TCP status; IDN by scan) | `[{type, id, host, via}]` | LASER: `Procurar` |
| `laser_open` | `dac:string`, `host:string=""`, `kpps:number=30`, `safety:any=null` | opens the DAC and brings the feed up (the safety never switches off) | `{feed, dac, pps}` | LASER: `Abrir` |
| `laser_play` | `feed:integer`, `file:string`, `fps:number=30`, `loop:boolean=false` | pushes the frames of the `.ild` to the DAC | `{feed, file, frames, fps, loop}` | LASER: `Play` |
| `laser_stop` | `feed:integer` | stops the playback; the DAC stays open | `{feed, playing:false}` | LASER: `Stop` |
| `laser_close` | `feed:integer` | stops and closes (shuts the DAC down) | `{feed, dac, closed}` | LASER: `Fechar` |
| `laser_param` | `feed:integer`, `path:string`, `value:number` | one parameter of the feed (geo, limit, safe, shutter) | `{feed, path, value, shutter}` | LASER: sliders and `Shutter` |
| `laser_stats` | `feed:integer` | state of the feed | `{playing, file, stat/*, jitter, cpu, safety}` | LASER: stats panel (4 Hz) |
| `laser_files` | `dir:string=""` | lists the `.ild` of the directory (empty = `shows/`) | `{dir, files:[{name, path, bytes}]}` | LASER: `Listar` |
| `clip_frame` | `clip:string`, `index:integer?`, `t:number=0`, `fps:number=30` | one frame of the `.ild` to draw, by `index` or by `t` at `fps` | points `[x, y, r, g, b, blank]`, `x`/`y` in -1..1 | TIMELINE: playhead previz (`Alt+M`) |

### What the audit fixed, and what stood

- **`load(path)` became `load(file)`.** The same `.spell` path was called `path` in `load` and
  `file` in `show_get`, `show_save`, `play_show` and `module_add`. `path` stays accepted for one
  round (`#[serde(alias = "path")]`) and the `doc` of the command warns that it is deprecated;
  updated clients: `spellgui/web/graph.js`, `serve::abre`, `cli/tests/serve.rs`,
  `engine/tests/patch.rs`. Inside the repository no caller of the old name was left: the alias
  stays only for scripts and MCP sessions already written, and goes out in round 3. The Python
  `spellcaster/` does not change because it has its own registry
  (`spellcaster/core/registry.py`) and never calls the Rust one.
- **`track_add(label)` became `track_add(name)`.** The field it writes in the `.spell` is called
  `name`, and `patch_add` already used `name` for the same idea. `label` stays accepted for one
  round, by the same mechanism; updated client: `spellgui/web/timeline.js`.
- **Nothing was removed.** The four suspicious pairs have a client and a reason:
  `load` x `show_get{file}` (one validates the timeline, the other only summarizes — the `doc`
  now says so), `resume` x `play_show` (one releases the live player, the other brings one up),
  `show_new` x `show_set` (one clears, the other imports), `laser_stop` x `laser_close` (one
  stops the file, the other shuts the DAC down).
- **Stood on purpose:** `track_del(index)`/`cue_del(index)` name the target of the command and
  `key_set(track, t)` names the container of the keyframe — different names for different things.
  `resume`/`pause`/`stop` are still three verbs with no argument, and not a `transport(state=…)`,
  because that is how they show up in the shortcut map, in the MCP tools and in the OSC addresses.
- **Description per argument:** the twelve arguments that still had no `description`
  (`track_add.universe`, `key_del.track`, `cue_set.name`, `cue_del.index`, `patch_add.universe`,
  `patch_del.name`, `level_set.universe`, the four of `cue_capture` and `laser_param.value`) got
  theirs. `spellgui/web/test/help.test.js` fails if a new command or argument comes in with no
  text.
- **One source for the text:** `DOC_PLAY`, `DOC_NET` and `DOC_GRAPH_CHECK` in `cli/src/main.rs`
  are at the same time the `doc` of the registry and the `about` of clap, so `spellcore play
  --help` and `GET /commands` say the same sentence (test:
  `clap_about_and_registry_doc_are_the_same_text`). For the commands that are **not** a CLI
  subcommand, `spellcore commands <name>` prints the `doc` and the schema of one alone — it is
  their `spellcore <cmd> --help`.

## `engine::registry::base()`

The signature becomes `pub fn base() -> Registry` (with no `Clock`: the transport acts on the live
player). Commands: `load` (R0), `resume`, `pause`, `stop`, `locate`, `cue_go`, `transport_state`
and, since R7, `show_get`; plus the ones of `engine::edit`, `engine::module` and `engine::midi`,
which register themselves in `base()`. The transport ones use `player::current()`; with no live
player they return `Err("no player running")`. `show_get(file="", full=false)` opens the `.spell`
(or reuses the last one opened in this process, the `OPEN` of `spellcaster/mcp/tools.py`) and
summarizes name, fps, duration, outputs, patch, tracks, cues and the live transport; it is the one
that feeds the `spell://show` resource. `full=true` returns the whole `.spell` (what the GUI
draws).
`resume()` is the pair of `pause` (the `Handle::play`): without it, whoever paused through the
registry could only play again by bringing up another player. It is called `resume` and not `play`
because `play` is the CLI subcommand that BRINGS UP a player — that one is `play_show`.
`play_show` does **not** come in here: it builds the `script` hooks and is registered by the CLI,
like `play` and `net` in R0.

### `engine::edit` — editing the open show

Port of `spellcaster/gui/api.py` (plus the footprint check of `fixtures/patch.py`), wired into
`base()` by `edit::register`. Every command acts on the `OPEN`; with no open show, it opens a new
one (the `SHOW = NEW` of Python), so the AI can call `track_add` before any file.

| Command | Does | Returns |
|---|---|---|
| `show_new` | clears: "novo show", sACN on universe 1, 60 s, empty `patch`/`cues`/`markers` | the whole show |
| `show_set(data)` | replaces it with the given JSON (object or JSON text); `migrate`; `_x` keys are dropped | the whole show |
| `show_save(file="")` | saves (with no `file`, at the path of the last `load`/`show_get`/`show_save`) | the path |
| `track_add(type="dmx", universe=1, address=1, label="", clip="", script="")` | empty track at the end; `clip` is the `.ild` of the `laser` track, `script` the `.rhai` of the `fx` track | index |
| `track_del(index)` | removes | the track |
| `key_set(track, t, value=0, curve="linear")` | creates or replaces the keyframe at `t` (`\|Δt\| < 1 µs`); a `value` text that is JSON becomes JSON; sorted list | keys of the track |
| `key_del(track, t)` | deletes at `t` (1 ms tolerance) | how many were removed |
| `cue_set(index?, name, fade, wait, follow, values)` | creates (with no `index`) or replaces; `values` = `{"u/end": v \| [v...]}`, key validated by `cues::key` | index |
| `cue_del(index)` | removes | the cue |
| `patch_add(name, profile, universe=1, address=1)` | appends and validates the whole patch; an overlap or running past 512 is refused and rolled back | the grid |
| `patch_del(name)` | removes by name | the entry |
| `patch_check()` | grid (`name`, `profile`, `universe`, `address`, `channels`) + `error` of the first fixture that does not fit | `{rows, error}` |
| `profiles()` | names of the `.json` in `profiles/` | list |
| `show_patch(ops, rev?)` | JSON Patch (RFC 6902: `add`, `remove`, `replace`, `test`) over the open show; it applies on a copy and only commits if all of them pass **and** the result still deserializes into `Show`; a `rev` different from the current one is refused (`"rev 3 != 5"`) | `{rev, undo}` |
| `graph_get()` | the `graph` of the show (section 10 of the PRD) | `{nodes, edges}` |
| `face_get()` | inline `face`, or `faces/<name>.face.json` when `face` is text | the face or `null` |
| `profile_get(name)` | the whole profile (channels with `offset`, `fine`, `ranges`, `wheel`) for the client to build a widget | the JSON of the profile |

Transport, besides the ones of R0/R7: `resume()` continues the paused player (the missing half of
`pause`; it is called `resume` because `play` is the CLI subcommand and the registry already has
`play_show`) and `input(key, value)` delivers the event to `FrameHook::input` of the live player —
it is through it that widget, key and module feed the Graph.

`profiles/` is the first one that exists among: next to the `.spell`, one level above it (`shows/`
and `profiles/` as siblings, as in the repo and on the USB stick), the cwd and the folder of the
executable. The profile is read for name and footprint (`max(offset, fine) + 1`); `fixture_set`
resolves the channel name in the raw JSON of the profile itself; ranges and wheel only travel raw
in `profile_get`. Test: `engine/tests/edit.rs`, its own binary because `OPEN` is one per process.

`show_patch` is the preferred way to edit: the client sends the list of ops and receives
`{rev, undo}`, where `undo` already comes **in application order** — sending it back as it came
undoes the edit byte for byte. The undo stack belongs to the client; the engine only keeps the
counter `rev` (`engine::edit::rev()`), which goes up on every successful edit and is what the bus
broadcasts. Two GUIs on the same show: whoever sends with an old `rev` gets an error instead of
overwriting the other one's edit. `show_set` still exists in order to **import** a whole show.
Test: `engine/tests/patch.rs`.

`graph_get` is a read and lives in the engine; editing the graph is `show_patch` on `/graph`.
**Compiling** the graph is `graph_check`, registered by the CLI (only it knows the `script`
crate), which compiles the graph of the open show and returns `{nodes, error}` — `error` is text,
not an exception, so that the editor can show it next to the node.

### `engine::module` — a module is a declared app

An app (the laser, the media player, a Pi on the network) is not core code: it is a `module.json`
that declares `parameters` (what is sent to it), `values` (what it returns, read only) and
`commands` (`context: action | mapping | both`, the `CommandContext` of Chataigne). The textual
address `group/name` is the identity (rule 2 of `design/FUNCOES/README.md`) and the `type` of the
parameter is what generates the widget (rule 1): `float`, `int`, `bool`, `trigger`, `color`,
`string`, `enum` (with `options`); `min`/`max` is the physical clamp and `norm` is the useful range
of the slider, separate from it. The core only keeps the table of the modules alive in this
process; the PATCHBAY builds the node from it without knowing the app, and the live values arrive
through the `input` command with the key `module:<name>/<path>`. This section is the specification
of the format; the live example is `modules/laser.json`, the first declared module.

| Command | Does | Returns |
|---|---|---|
| `module_add(file="", data={})` | reads the manifest (`file` = a name in `modules/` or the path of a `.json`; or the whole `data`), validates it (address `a/b` with no space, known `type` and `context`, `min < max`, `default` inside the range, `enum` with `options`) and puts it in the table; the same `name` replaces. A refusal returns all the errors at once, each one naming the path | `{name, version}` |
| `module_del(name)` | takes it out of the table | the removed manifest |
| `module_list()` | live modules | `[{name, type, version}]` |
| `module_get(name)` | the whole manifest | `{name, type, version, parameters, values, commands}` |

`modules/` resolves by the same rule as `profiles/` (`edit::recurso_dir`). Foreign fields of the
Chataigne manifest (`hasInput`, `dependency`, `label`, `unit`, `args`) are read and discarded.
Test: `engine/tests/module.rs`, its own binary because the table is one per process.

### `laser_*` — ILDA player (registered by the CLI)

`spellcore/cli/src/laser_cmd.rs`. It lives in the CLI, and not in the engine, for the same reason
as `play_show` and `net`: the engine does not know the `laser` crate. One feed = one open DAC; the
`FEEDS` table is to the laser what `player::current()` is to the transport (one process, N feeds).

| Command | Does | Returns |
|---|---|---|
| `laser_dacs(timeout=2)` | Ether Dream by beacon and, if no beacon arrives within half the deadline, by TCP status on the ARP neighbours (`netscan`); IDN by scan | list of `{type, id, host, via}`, `via` = `beacon` or `tcp` (one found by TCP does not carry `buffer`/`max_pps`: only the beacon carries them) |
| `laser_open(dac, host="", kpps=30, safety?)` | opens the DAC and brings the `Feed` up; `safety` = `{min_size, max_intensity, zone}`, never switchable off | `{feed, dac, pps}` |
| `laser_play(feed, file, fps=30, loop=false)` | a thread that reads the `.ild` and does `feed.push` at the rate (the `.ild` carries no rate); with no `loop`, the end of the file disarms the transport and `laser_stats` goes back to `playing:false` | `{feed, file, frames, fps, loop}` |
| `laser_stop(feed)` | stops the playback; the DAC stays open | `{feed, playing:false}` |
| `laser_close(feed)` | stops and closes (the `Drop` of the `Feed` shuts the DAC down) | `{feed, dac, closed}` |
| `laser_param(feed, path, value)` | one parameter of the feed (table below) | `{feed, path, value, shutter}` |
| `laser_stats(feed)` | `playing`, file, `stat/sent`, `stat/dropped`, `stat/errors`, jitter, cpu and the current safety | object |
| `laser_files(dir="shows")` | the `.ild` of the directory | `{dir, files:[{name, path, bytes}]}` |
| `clip_frame(clip, t=0, index?, fps=30)` | one frame of the `.ild` to draw (the timeline previz): `index` picks it directly, otherwise it is `floor(t*fps)` with the clip repeating, the player's arithmetic. A `clip` with no path resolves in the folder of the open `.spell`; it touches no DAC | `{clip, index, frames, name, points:[[x, y, r, g, b, blank]]}` with `x` and `y` normalized to -1..1 |

**Ether Dream discovery (`laser_dacs`, `net`).** The DAC announces a 36-byte UDP beacon on
`255.255.255.255:7654`, at 1 Hz. On Windows, with the **Ether Dream Sitter** open, the `bind` on
`0.0.0.0:7654` is refused with `WSAEACCES` (10013) even with `SO_REUSEADDR` — Windows only shares
the datagram if **both** sockets ask for it, and the Sitter does not. Measured on this machine
(Sitter on PID 53580, DAC at 169.254.207.140, PC at 169.254.86.236/16): `bind 0.0.0.0` fails,
`bind 169.254.86.236:7654` passes **and receives the broadcast** (4 beacons in 4 s). That is why
`netscan` listens on the IP of each card, and not only on the wildcard; and, if nothing arrives
within half the deadline, it asks for the status over TCP 7765 from the neighbours of the ARP table
(`arp -a` / `ip neigh`, text that is read, never executed) — an Ether Dream answers 22 bytes
(`ack` + echoed command + `dac_status`) when it accepts the connection.

`path` of `laser_param` (the same paths as `modules/laser.json`, the declaration of the laser
module); the `stat/*` keys of `laser_stats` are the `values` of the same file, only the ones
`FeedStats` counts:

| path | field | range |
|---|---|---|
| `geo/x`, `geo/y` | `Transform.x`, `Transform.y` | ILDA units, +-32767 |
| `geo/scale` | `Transform.scale` | 0..4 |
| `geo/rot` | `Transform.rot` | degrees, +-180 |
| `limit/r`, `limit/g`, `limit/b` | `Transform.color.0/.1/.2` | 0..1 |
| `safe/min_size` | `Safety.min_size` | ILDA units, 0..32767 |
| `safe/max_intensity` | `Safety.max_intensity` | 0..255 |
| `shutter` | zeroes `max_intensity` and gives back the value kept at open time | 0 or 1 |

`curve/r|g|b`, `Blanking/*` and `Cor/Time Shift` of the `ilda-player` table stay out: they come in
when the `Feed` has a color LUT and `optimize` is parameterizable at runtime.

Page: `spellgui/web/laser.html` + `laser.js` (DAC, kpps, file, play/stop, `geo/*` and `limit/*`
sliders, shutter button, stats by polling at 4 Hz). Test: `spellcore/cli/tests/laser.rs`, its own
binary, brings the Ether Dream `Emulator` of the `laser` crate up and talks to the registry through
the MCP server in another process (the `FEEDS` table is one per process).

### Programmer — the manual operator layer (TEATRO DE PAPEL theme)

`Prog`, in `player.rs`: one `Option<u8>` per channel (value and "touched" mask in the same
structure), applied **after `CueList::update` and before the I/O**, HTP per channel: the operator
overrides the live cue on the same channel. Releasing a channel clears the value held in the buffer
on the next frame, before the timeline, so that what the timeline owns is valid again. The
programmer does not go into the `.spell`: what records is the cue.

| Command | Does | Returns |
|---|---|---|
| `level_set(universe=1, address, values)` | writes into the override from `address` on; HTP over the timeline and the live cue; an empty list writes zero | how many channels |
| `level_clear(universe?)` | releases one universe, or all of them | how many channels were freed |
| `level_get(universe?)` | the current override, in the `values` format of a cue | `{"u/end": v}` |
| `cue_capture(name, fade, wait, follow)` | the override becomes a new cue at the end of the list (the same path as `cue_set`) and the override is released | index |
| `fixture_set(name, channel, value)` | resolves the patched fixture + the channel name in the profile and calls `level_set` | `{universe, address, value}` |

With no live player, the five return `no player running`. Test: `engine/tests/programmer.rs`, its
own binary (`CURRENT` and `OPEN` are process globals).

### MIDI — mapping a key to a command (`engine::midi` + `protocols::midi`)

"Do the MIDI mapping of the keys for any keyboard or control surface": one open input port, a
`key -> command` map that lives in the `.spell`, and the same key delivered to the graph.

Event key: **`"<status>/<data1>"`** — `144/60` = note on channel 1 note 60, `128/60` = the same key
released, `176/1` = CC 1 of channel 1. The channel is already in the status (channel 2 = 145 /
177), so it is a single key, the same one the `in.midi` node of the graph already used. Note on
with velocity 0 becomes note off (`0x8n`): half of the keyboards release the key that way, and
without this a release would fire the command again.

In the `.spell`:

```json
{"midi_port": "MPK mini 3",
 "midi": {"144/60": {"cmd": "resume"},
          "144/62": {"cmd": "cue_go", "args": {}},
          "176/1":  {"cmd": "level_set", "args": {"address": 1, "values": ["$255"]}}}}
```

`"$"` in the args becomes the value of the key (0..1, that is `data2/127`) and `"$<n>"` becomes
`round(value x n)`: `"$127"` is the raw MIDI byte and `"$255"` is the DMX level. It holds inside a
list and inside an object. `"midi_port"` reconnects the control surface when the show comes up
(`Player::new`); a port that does not show up becomes a warning on stderr, not an error — a show
does not stop because the controller stayed in its case.

| Command | Does | Returns |
|---|---|---|
| `midi_ports()` | input ports of the machine; a machine with no MIDI returns an empty list, never an error | `{ports, open}` |
| `midi_open(port="")` | opens by name, part of the name or index as text; empty = the first one. Writes `midi_port` into the open show | `{open}` |
| `midi_close()` | closes the port and removes `midi_port` from the show | `{open: null}` |
| `midi_map(key, cmd, args={})` | binds the key to the command in the open show; the key and the command name are validated **here**, not in the middle of the show | the map |
| `midi_unmap(key)` | unbinds the key | the map |
| `midi_maps()` | the map of the open show | the map |
| `midi_last()` | last key received; it is with it that the GUI shows live | `{key, value, raw, seq}` |
| `midi_learn()` | waits up to 5 s for the next key and returns its key | `{key, value, raw, seq}` |

Where the event is consumed: `midi::pump()` on the first line of `Rt::drain`, the same point where
the queue of the `input` command is consumed — **the evaluation order of the frame does not
change**. Each event becomes (1) `input {key: "midi:<key>", value}` on the hooks of the live
player, for the `in.midi` node, and (2) the call of the mapped command through the registry, the
same path as the WS. `midi_last` and `midi_learn` also drain the queue, so that the LEARN of the
GUI works with no show playing.

`protocols::midi` (crate `midir`, RtMidi in Rust: winmm on Windows, ALSA on Linux, CoreMIDI on mac
— it is what makes "any keyboard" work with no vendor driver):

```rust
pub fn ports() -> Vec<String>;                       // empty when there is no MIDI on the machine
pub struct MidiIn { /* connection + queue */ }
impl MidiIn {
    pub fn open(port: &str) -> Result<MidiIn, String>;   // name, fragment, index, or "" = the first one
    pub fn name(&self) -> &str;
    pub fn try_recv(&self) -> Option<(u8, u8, u8)>;      // (status, data1, data2)
}
pub fn evento(m: &[u8]) -> Option<(u8, u8, u8)>;     // raw message -> channel event
```

The callback thread of the driver pushes into a `sync_channel(256)`: when it is full, the new event
is dropped and the driver never blocks. `Ignore::All` discards SysEx, clock and active sensing
before the queue.

Out of the scope of this work front: MIDI **output**, MTC/clock, MIDI Show Control and LED feedback
on the control surface (`design/DECISOES.md`, "aguarda voto").

Deliberate limits:

- one port per process (`// ponytail:` in `engine/src/midi.rs`) — the key does not tell the port
  apart;
- the mapped command runs on the thread that drained it (the frame, in the normal case): mapping
  `net` or `play_show` to a key blocks the frame while the command runs;
- with no player and no MIDI page open, nobody drains the queue — it fills up and stops growing.

Page: `spellgui/web/midi.html` + `midi.js` (ports, open/close, map table, LEARN, live last key,
preview of the `"$"`). Tests: `engine/tests/midi.rs` (map and `$`, with no hardware),
`protocols::midi` (message parsing and port choice), `spellgui/web/test/midi.test.js` (arguments of
the page).

## `script` (new crate)

```rust
/// Track {"type":"fx","script":"medgrupo.rhai","universe":N}. Compiles the .rhai once,
/// keeps the state between frames, exposes `set(universe, addr, values)` to the script.
pub struct Fx;
impl Fx { pub fn new(path: &Path, universe: u16) -> Result<Fx, String>; }
impl engine::FrameHook for Fx { }

/// Graph of section 10 of the PRD, compiled from the JSON of `show["graph"]`.
pub struct Graph;
impl Graph {
    pub fn new(spec: &serde_json::Value, sink: Box<dyn engine::EventSink>) -> Result<Graph, String>;
    /// Same, with the directory of the show: it is from there that the `module` node reads `modules/<name>.json`.
    pub fn new_in(spec: &serde_json::Value, sink: Box<dyn engine::EventSink>, base: &Path)
        -> Result<Graph, String>;
    pub fn nodes(&self) -> usize;
}
impl engine::FrameHook for Graph { }

/// Hooks of a show, in execution order: one Fx per "fx" track + the Graph, if there is one.
pub fn hooks(show: &engine::Show, base: &Path, sink: Box<dyn engine::EventSink>)
    -> Result<Vec<Box<dyn engine::FrameHook>>, String>;
```

Minimum API exposed to the Rhai script (fixed; the rest is the crate's choice, documented here
later): `set(universe, addr, values)` — `values` an array of numbers or a bare number; same
semantics as `Universe::set` (clamp 0..255, truncation toward zero, 1-based, ignores whatever goes
past 512).

Rhai comes in with `default-features = false` and the minimum set of features that makes the show
run. The size of the release `spellcore.exe` is measured before and after and goes into the
dependency table. Reference measurement before Rhai (Windows x64, release profile of the
workspace): **970 240 bytes**.

### Graph: closed catalog (section 10 of the PRD)

`in.widget | in.key | in.osc | in.midi (stub) | in.timer | in.marker | in.state` ·
`logic.and|or|not|latch|toggle|debounce|counter|select` · `math.map|curve|expr` ·
`time.delay|hold` · `cmd` · `out.widget|out.osc|out.param|out.notify` ·
`state` · `module` (the last two are a proposal of this round, `design/DECISOES.md`).

Semantics of `state`, `module` and `mute`: `design/DECISOES.md` (and the header of
`script/src/graph.rs`, which is the same table as the runtime).

JSON: `{"nodes":[{"id","type",...}], "edges":[["node.pin","node.pin"], ...]}`. It compiles to a
list of nodes in topological order with pins indexed by integer; evaluation per frame with no
allocation; `math.expr` compiles the Rhai once. A cycle in the graph is a compile error. Inputs
arrive through a pre-allocated `in.*` queue (`FrameHook::input`); outputs go out through a
pre-allocated `out.*` queue, drained to the `EventSink` at the end of the frame. Acceptance: 500
nodes in less than 0.1 ms per frame (`bench/benches/graph.rs`).

#### PATCHBAY — the graph editor (`spellgui/web/patchbay.html`)

The same catalog, as data, in `spellgui/web/catalog.js`: config and pins of each type, plus the
**port type** (`trigger, bool, number, color, xy, frame, dmx`), which is a rule of the editor — the
runtime carries everything as `f64`. A cable only connects compatible types (`trigger` and `bool`
are the same wire); a conversion is a visible node (`math.map`, `logic.toggle`), never a hidden
coercion. A `module.json` (work front `module`) becomes a node through `CATALOG.moduleDef`.
`spellgui/web/graph.js` has the pure model (`GM`: JSON Patch with inverse, node/cable/key ops,
`Shift+Delete` rewiring, closed group) and the page. Every edit goes out as `show_patch {ops}` on
`/graph/...` and the `undo` response is stacked; with no engine the same list runs locally. `x`,
`y`, `group`, `mute`, `lock`, `label` live in the node (the runtime ignores unknown keys).
Demonstration: `shows/patchbay_demo.spell`. Tests:
`node --test spellgui/web/test/graph.test.js`.

## CLI

```
spellcore play <show.spell> [--loop] [--osc-port N]
```

`play` is the name of the subcommand; the registry command is called `play_show` (from R7 on they
are two literal names in the CLI code, no longer an alias table). The CLI builds the `Player`, asks
`script::hooks` for the hooks, passes its own `EventSink` (`Ev::Cmd` -> registry, `Ev::Osc` ->
OscOut, `Ev::Notify`/`Widget`/`Param` -> stderr) and prints, **one line per second, ASCII only**:

```
t=  12.35s state=play cue=3 frames=372 jit_p99=0.41ms u=1,2
```

`--osc-port` overrides `transport.osc_port` of the `.spell`. Ctrl+C closes the player and the
outputs.

The CLI also registers `graph_check(graph?)` -> `{nodes, error}`: it compiles the graph with
`script::graph::Graph::new` without running it. It stays here for the same reason as `play_show`
and `net` — the engine does not see `script`.

## The MED GRUPO show in R1

`shows/medgrupo.spell` gains a track `{"type":"fx","script":"medgrupo.rhai","universe":1}` **next
to** the `pyfx` track, which stays in the file: Python ignores `fx` and Rust ignores `pyfx`, and
that way `tests/conformance/gen.py` keeps regenerating the fixtures from the same file.
`shows/medgrupo_r0.spell` (219 baked `dmx` tracks) is still the show of the offline conformance and
of the Criterion bench.

Acceptance of the `fx`: `shows/medgrupo.rhai`, ported from `shows/medgrupo.py`, reproduces
`tests/conformance/medgrupo_u1.bin` on the 2577 frames of universe 1, byte for byte. Any
floating-point divergence that survives is documented byte for byte (frame, channel, value,
reason).

## Environment of the agents

```bash
export PATH="$PATH:/c/Users/email/.cargo/bin"
export CARGO_TARGET_DIR="C:/Users/email/AppData/Local/Temp/spellcore_target"
```

Worktree: `C:\Users\email\AppData\Local\Temp\spellcaster-main`. Never `target/` inside it.
No `git commit`, no `git push`. Tests and bench output in ASCII only (cp1252 console).

---

# R7 — MCP (crate `mcp`)

MCP server over the official SDK `rmcp` 3.2. Port of `spellcaster/mcp/server.py`: **the tools come
from the registry**, no product logic in the crate.

```
spellcore mcp                                       # stdio: one JSON-RPC message per line
spellcore mcp install --target desktop              # %APPDATA%\Claude\claude_desktop_config.json
spellcore mcp install --target code [--path P]      # .mcp.json of the current directory
```

| Surface | Content |
|---|---|
| tools | one per command of `Registry::iter()`: `load`, `show_get`, `resume`, `pause`, `stop`, `locate`, `cue_go`, `transport_state`, `input`, `input_get`, `rec_arm`, `rec_state`, the editing ones of `engine::edit` (`show_new`, `show_set`, `show_save`, `track_add`, `track_del`, `key_set`, `key_del`, `cue_set`, `cue_del`, `patch_add`, `patch_del`, `patch_check`, `profiles`, `show_patch`, `graph_get`, `face_get`, `profile_get`, `level_set`, `level_clear`, `level_get`, `cue_capture`, `fixture_set`), the `module_*` of `engine::module` (`module_add`, `module_del`, `module_list`, `module_get`), the `midi_*` of `engine::midi` (`midi_ports`, `midi_open`, `midi_close`, `midi_map`, `midi_unmap`, `midi_maps`, `midi_last`, `midi_learn`), `play_show`, `net`, `graph_check` and the `laser_*` of `cli/src/laser_cmd.rs` (`laser_dacs`, `laser_open`, `laser_play`, `laser_stop`, `laser_close`, `laser_param`, `laser_stats`, `laser_files`, `clip_frame`). `inputSchema` = the schema `schemars` generated from the argument struct |
| resources | `spell://show` (the open `.spell`: fps, duration, outputs, patch, tracks, cues, live transport), `spell://commands` (the whole registry in JSON), `spell://graph` (the `graph_get`) and `spell://face` (the `face_get`). Each resource is one registry command call: the `mcp` crate has no product logic |
| error | a command error comes back as `isError: true` with the text (the client reads it); only a route that does not exist becomes a JSON-RPC error |
| `play_show` | it blocks until the end of the show, so it runs on a thread and the tool comes back at once (the `BACKGROUND` of Python). While the MCP runs, the status line of `play` goes to **stderr**: on stdio the stdout is the JSON-RPC channel |

Out for now, and why:

- **`face_patch`/`graph_patch`/`theme_set`** (PRD §10). `face_get` and `graph_get` exist (they read
  what is in the `.spell`), but `engine::show` has no serialized Theme and the Graph only exists
  compiled inside `script`; with no structure to apply JSON Patch to, those three would have no
  backend — the one that edits the whole show by JSON Patch is `show_patch`.
- **`mcp_install` as a registry command.** In Python it is `@command` and therefore a tool. Here it
  is not: an AI session must not rewrite its own configuration — the one who installs is the
  operator, from the terminal.

Test: `spellcore/cli/tests/mcp.rs` brings the real binary up on stdio, does `initialize`,
`tools/list`, `tools/call show_get` on `shows/medgrupo.spell`, reads `spell://commands` and
`spell://show`, and checks that `play_show` comes back at once without dirtying the stdout.

## `spellgui/web` — the command catalog as a widget

The pages (`face.html`, `index.html`) know no command at all: they read `Registry::schema()` and
build the form from the schema of each `Args` (`widgets.js`: `number` with `min`/`max` becomes a
slider, `integer` becomes a spin, `boolean` becomes a toggle, `enum` becomes a select, a command
with no property becomes a button). The frozen copy of that schema is
`spellgui/web/dev/commands.json`, which the page uses when it opens with no engine.

| Command | Does | Returns |
|---|---|---|
| `commands` (CLI subcommand) | prints `Registry::schema()`; it is the source of `spellgui/web/dev/commands.json` | the list of commands with doc and schema |
| `commands <name>` | that command alone — it is the `spellcore <cmd> --help` of the verbs that did not become a CLI subcommand | `{name, doc, params}` |

```
spellcore commands > spellgui/web/dev/commands.json
```

`cli/tests/commands_json.rs` fails if some command leaves the registry or changes schema without
the JSON being regenerated, and `spellgui/web/test/help.test.js` fails if a command or argument
comes in with no help text. The page that shows all of this to the operator is
`spellgui/web/help.html` (shortcuts of `design/SHORTCUTS.md` + the live registry, each command with
a form that runs it). The rest (bus, Face, how to open) is in `spellgui/web/README.md`.
