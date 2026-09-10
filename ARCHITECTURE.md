# Spellcaster — architecture (Python prototype F0–F6; spellcore Rust R0, R1, R3, R4, R7)

## Tree

```
spellcaster/
  __init__.py            __version__
  cli.py                 spell: play, commands (generated from the registry)
  core/
    clock.py             Clock
    engine.py            Engine
    registry.py          @command, call, schema, REGISTRY
    universe.py          Universe, Universes
  protocols/
    sacn.py              E1.31 out/in
    artnet.py            Art-Net 4 out/in
    osc.py               OSC 1.0 out/in
    netscan.py           network scan (__main__)
    ilda/
      frame.py           Point, Frame, optimize, safety
      ild.py             read, write
      generators.py      figures, medgrupo, render (__main__)
      etherdream.py      EtherDream, Emulator
shows/
  medgrupo.py            look(t), DUR
tests/                   test_artnet, test_clock, test_engine, test_ilda, test_netscan,
                         test_osc, test_registry, test_sacn
```

## Data flow

```
show .py  ──look(t)──▶  Engine.apply  ──▶  Universes  ──bytes──▶  out.send(universe, data)
                ▲                                                  (SacnOut | ArtNetOut)
             Clock.run
```

1. `Clock.run(fn, duration)` calls `fn(t)` every `1/fps` s. `t` comes from `perf_counter`.
2. `Engine.tick(look, t)` calls `look(t)`. The return value is a dict `{addr: [values]}` (universe 1) or `{(universe, addr): [values]}`.
3. `Engine.apply(frame)` writes each list into `Universes.get_or_create(u).set(addr, values)`.
4. For each universe, `Engine.tick` converts the buffer to `bytes` and calls `out.send(u.number, data)` on every output.

### Public signatures (`spellcaster/core`)

- `Clock(fps=30)`: `.time`, `.state` ("stop" | "play" | "pause"), `play()`, `pause()`, `stop()`, `locate(t)`, `run(fn, duration=None)`.
- `Universe(number)`: `.number`, `.data` (512-byte bytearray), `set(addr, values)`. 1-based address, clamp 0-255, ignores anything past 512.
- `Universes(dict)`: `get_or_create(number)`.
- `Engine(outputs, fps=30)`: `.outputs`, `.universes`, `.clock`, `apply(frame)`, `tick(look, t)`, `run(look, duration=None)`.
- `registry.command(name=None, mcp=True)`: decorator. `registry.call(name, **kwargs)`: coerces by annotation and calls. `registry.schema()`: list of `{name, doc, params: [{name, type, default}], mcp}`.

### Registry → CLI → MCP

`cli.main()` walks `schema()` and creates one subparser per command. A parameter with no default becomes a positional argument. A `bool` parameter becomes a `--name` flag (`store_true`). The rest become `--name` with the default. `call()` converts argparse strings to the annotated type (`int`, `float`, `str`, `bool`). The same `schema()` will feed the MCP generator (F5): each entry with `mcp=True` becomes a tool with a JSON schema; `mcp=False` entries sit behind a generic `run_command` tool.

## Protocols

Every output exposes `send(universe: int, data: bytes)` and `close()`.

### sacn (`protocols/sacn.py`)

- Port 5568. Multicast per universe `239.255.{u>>8}.{u&255}`. Discovery on `239.255.250.214`.
- `packet(universe, data, cid, seq, source_name="Spellcaster", priority=100) -> bytes`: pure function, the same bytes as the MED GRUPO generator.
- `parse(pk) -> dict | None`: data packet (vector 4) or discovery (vector 8).
- `interfaces() -> list[str]`: local IPv4 IPs via `getaddrinfo(gethostname())` plus `127.0.0.1`.
- `SacnOut(universes=(1,), priority=100, source_name="Spellcaster", interfaces=None)`: one socket per IP with `IP_MULTICAST_IF`. `send` sends multicast on each socket and unicast on `127.0.0.1`. `close()`.
- `SacnIn(universes=(1,))`: daemon thread. `get(universe) -> bytes | None`, `.sources`, `close()`.
- No `ponytail:`. Not implemented: E1.31 sync, priority merge on input. Network discovery is scanned by `netscan.scan_sacn`, which decodes with this `parse`.

### artnet (`protocols/artnet.py`)

- Port 6454. Broadcast on `2.255.255.255`, `10.255.255.255`, `255.255.255.255`.
- `port_address(universe) -> int`: `universe - 1`, 15 bits.
- `artdmx(universe, data, sequence, physical=0) -> bytes`: data 2..512 bytes, even length.
- `artpoll(flags=0x06, priority=0x10) -> bytes`, `artsync() -> bytes`.
- `parse(packet) -> dict | None`: ArtDmx, ArtPoll, ArtPollReply, ArtSync.
- `ArtNetOut(targets=None, broadcast=True, port=6454)`: `send(universe, data)`, `close()`. Sequence 1..255.
- `ArtNetIn(universes, host="0.0.0.0", port=6454)`: thread. `.frames[universe] -> bytes`, `close()`.
- The network ArtPoll is fired by `netscan.scan_artnet`, which decodes the reply with this `parse`.
- Not implemented: emitting ArtPollReply (Spellcaster does not show up as a node to other controllers).

### osc (`protocols/osc.py`)

- UDP, port chosen by the caller. No default port.
- `message(address, *args) -> bytes`. Types: `int` (i/h), `float` (f), `str` (s), `bytes`/`Blob` (b), `True`/`False`, `None`, `Ellipsis` (impulse), tuple `("d"|"h"|"t", value)` for an explicit type.
- `bundle(elements, tt=IMMEDIATE) -> bytes`, `timetag(t=None) -> int` (64-bit NTP).
- `parse(data) -> (address, args) | ("#bundle", timetag, elements)`.
- `pattern_re(pattern) -> re.Pattern`: supports `*`, `?`, `[a-z]`, `[!a-z]`, `{a,b}`.
- `OscOut(host, port)`: `send(address, *args)`, `bundle(msgs, tt=IMMEDIATE)`, `close()`. This `send` does not follow the `(universe, data)` contract; OSC is not a DMX output.
- `OscIn(port, host="0.0.0.0")`: thread. `on(pattern, fn)` calls `fn(address, *args)`. `close()`.
- `osc.py:153` ponytail: a future timetag in a bundle is ignored and the content runs right away. Missing: scheduling on the `Clock` once the engine exposes an event queue.

### netscan (`protocols/netscan.py`)

- Executable: `python -m spellcaster.protocols.netscan [--json] [--timeout N]`.
- `interfaces() -> list[{name, ip, mask, gateway}]`: `ipconfig` on Windows, `ip -j addr` + `ip -j route` on Linux, `parse_ip_addr` as a fallback. No loopback.
- `parse_ipconfig(text)`, `parse_ip_addr(text, route_text="")`: pure parsers, tested against pt-BR and en output.
- `scan_artnet(timeout=2, ifaces=None)`: ArtPoll on the global broadcast, 2.x, 10.x and each subnet's broadcast; decodes with `artnet.parse`.
- `scan_sacn(timeout=3, ifaces=None)`: joins the discovery multicast on each interface; decodes with `sacn.parse`.
- `scan_etherdream(timeout=2)`: UDP 7654 beacons.
- `suggest(ifaces, windows=WINDOWS) -> list[str]`: subnet rules for Art-Net, a ready `netsh` command (text, it does not run), warning about interfaces on the same subnet.
- `scan_all(timeout=2) -> dict`: the three scans in threads. `report(d) -> str`.
- No `ponytail:`. One decoder per protocol: the packets come from `artnet.parse` and `sacn.parse`.
- Not implemented (ROADMAP F1): mDNS `_osc._udp`, IDN discovery, latency measurement. The `spell net` verb (single: returns the dict with `report`, also used by the GUI and by the MCP) is in `cli.py`.

### Two `interfaces()`

| Function | Returns | Source | Used for |
|---|---|---|---|
| `sacn.interfaces()` | `list[str]` of IPs, includes `127.0.0.1` | `socket.getaddrinfo` | choosing `IP_MULTICAST_IF` in `SacnOut` |
| `netscan.interfaces()` | `list[dict]` with `name`, `ip`, `mask`, `gateway`, no loopback | `ipconfig` / `ip` | network report, per-subnet broadcast, suggestions |

`sacn` does not depend on subprocess and works without `ipconfig`. `netscan` needs the mask to compute broadcast and subnet. Keep both until F2 decides on a single source.

### ilda (`protocols/ilda/`)

- Coordinates ±32767. Colours 0-255.
- `frame.Point(x, y, r=0, g=0, b=0, blank=False)`: dataclass with clamp. `.lit`.
- `frame.Frame(points=None, name="")`: `len()`, iteration, `bbox(lit_only=True)`.
- `frame.optimize(frame, dwell=2, blank_gap=4, max_step=1200, angle=25) -> Frame`: dwell at vertices, blanked points at the transitions, jump interpolation.
- `frame.safety(frame, min_size=2000, max_intensity=255, zone=None) -> Frame`: dims a figure smaller than `min_size`, caps intensity, blanks outside the zone.
- `ild.read(path) -> list[Frame]`, `ild.write(path, frames, fmt=5, name="", company="spell", palette=None)`. Formats 0, 1, 2 (palette), 4, 5.
- `generators`: `ellipse`, `circle`, `polyline`, `rect`, `blank_to`, `ease`, `medgrupo(t, sx=20000, sy=10000) -> Frame`, `render(fn, fps=25, dur=46.8, **kw) -> list[Frame]`.
- `etherdream`: UDP 7654 beacon, TCP 7765 stream, little-endian.
  - `EtherDream(ip, port=7765, capacity=1800)`: `connect(timeout=2)`, `prepare()`, `begin(pps, low_water=0)`, `send(points)`, `ping()`, `play(frames, pps=20000, chunk=None) -> Thread`, `stop()`, `close()`. `send` here is the DAC's `d` command, not the DMX contract.
  - `Emulator(host="127.0.0.1", port=0, capacity=1800)`: TCP server for tests. `start()`, `stop()`, `beacon()`, `.received`, `.commands`.
  - `parse_beacon(b)`, `parse_status(b)`, `parse_response(b)`, `encode_data(points)`.
- `etherdream.py:107` ponytail: `_loop` waits for the buffer to drain with `sleep` + `ping` before queueing. Missing: flow by `low_water` and reconnection on a drop.
- Not implemented (ROADMAP): `idn.py` (IDN-Stream), `helios.py` (USB).

### core

- `clock.py:59` ponytail: when a tick is late, `run` re-anchors `nxt` at the current instant and does not accumulate the delay. Missing: measuring jitter at 60 Hz with many universes (risk listed in the ROADMAP).

## Where the next phases plug in

| Phase | Goes into | Depends on |
|---|---|---|
| F2 profiles and patch | `fixtures/profile.py`, `fixture.py`, `group.py`, `library/*.json`; `spell calib` verb | `Universe.set`, registry |
| F3 timeline, `.spell`, player | `timeline/model.py`, `tracks.py`, `cues.py`, `render.py`; `show.py`; `player/player.py`; the `laser` track uses `ilda.frame`, `ild`, `etherdream` | `Clock`, `Engine.apply` receives the dict from `timeline.render` instead of `look(t)` |
| F4 GUI | `gui/server.py` (HTTP + WebSocket), `gui/web/`, `gui/window.py` (pywebview) | registry over WebSocket; `netscan.scan_all` for the Network panel |
| F5 MCP | `mcp/server.py` (stdio) generates tools from `registry.schema()`; resources: show, patch, network, log | registry, `net` command |
| F6 portable and Lite | PyInstaller onedir; `spell serve --headless`, `spell tui` (curses); aarch64 tarball | everything above without `pywebview` |

## spellcore (Rust) — state at R1 + R3 + R4 + R7

The product core rewritten in Rust (PRD v1.1). The Python package `spellcaster/` stays in the repo
as the reference implementation and the conformance-fixture generator; it gets no new functionality.
`spellcore` reproduces byte for byte the sACN/Art-Net output of `shows/medgrupo.spell` and the bytes
and numbers of `spellcaster/protocols/ilda/`.

```
spellcore/
  Cargo.toml     workspace, edition 2021; release: lto "thin", codegen-units 1, panic abort
  engine/        clock, universe, timeline, show, registry, cues, hook, player
  protocols/     lib.rs (trait Output + queue), sacn, artnet, osc, netscan
  script/        lib.rs (Fx: `fx` track in Rhai), graph.rs (Graph runtime from PRD section 10)
  laser/         frame (optimization + safety), ild, feed (multi-feed), dac/{etherdream,helios,idn}
                 bin/feeds (4-feed bench), benches/laser.rs, tests/conformance.rs + fixtures
  pixelmap/      lib.rs (Order, Fixture, PixelMap, Frame, Sampling, Mapper, grid), rayon
                 bin/map_bench (2 ms gate), benches/pixelmap.rs, tests/conformance.rs
  mcp/           lib.rs (MCP server over `rmcp`, stdio), install.rs (`mcp install`)
  cli/           `spellcore` binary: play, net, commands, mcp (clap derive, fixed structs)
  bench/         Criterion (benches/core.rs, benches/graph.rs) + jitter and throughput binaries
tests/conformance/
  gen.py         generates the fixtures from the Python package
  medgrupo_u1.bin, sacn_packet.bin, artnet_packet.bin
  capture_sacn.py  validates the Rust binary against the fixture, live, on 127.0.0.1
shows/
  medgrupo.spell   pyfx (Python) + fx (Rust, `medgrupo.rhai`) + laser tracks; each side ignores the other
  medgrupo_r0.spell  219 baked dmx tracks (67,161 hold keys): offline conformance and Criterion bench
```

Dependency graph: `protocols` self-contained; `engine -> protocols`; `script -> engine, rhai`;
`laser -> protocols` (only the Ether Dream beacon); `pixelmap -> rayon, serde` (depends on no
workspace crate); `mcp -> engine, rmcp, tokio`; `cli -> engine, protocols, script, mcp`;
`bench -> engine, protocols, script`. The engine knows nothing about GUI, MCP, Rhai, laser or pixelmap.

`pixelmap` (R3) samples an RGB/RGBA frame at a normalized coordinate `(u, v)` per physical pixel
and writes a 512-channel buffer per universe — the `(universe, &[u8; 512])` pair that
`protocols::Output::send` receives. `Mapper::new` groups the fixtures by universe and allocates the
buffers once; `render` does not allocate and parallelizes per universe with `rayon`. Nearest
sampling by default, bilinear optional. Universe 0, channel 0 and any channel above 512 are dropped
at compile time; a channel that overruns the 512 at the end (510 + RGBW) is truncated; `u`/`v`
outside `0..1` clamp at the edge. The `.spell` keeps the map in the `pixelmaps` block (outside
`tracks`, preserved by the `Show`'s `extra`), with `{name, source: "media/<id>", fixtures:
[{universe, channel, order, u, v}]}`; the link to the player waits on R2 (media), so the crate still
runs on its own, over a test `&[u8]`. The PRD's wgpu version arrives when `map_bench` no longer
closes the 2 ms.

### Contracts

- `Clock::new(fps)`, `Clock::run(FnMut(f64), Option<f64>)`, `stats() -> Stats {p50,p99,max,frames,drift}`.
  High-priority thread, `timeBeginPeriod(1)` on Windows, fixed phase (`next += period`).
  Spin margin calibrated at runtime from the measured `sleep` overshoot (EWMA, 0.3–2 ms).
  **The `t` handed to the frame is quantized to the frame** (`round(t·fps)/fps`): it is the same
  `i/fps` as the fixture generator, and without it a continuous `fx` (sine) sampled at the measured
  time diverges by ±1.
- `Universes` 1-based, pre-allocated `[u8; 512]` buffers, `get_or_create` by binary search.
- `Timeline::apply(&mut Universes, t)` with no per-frame allocation; keys via `partition_point`.
  Curves: linear, hold, in, out, inout, bezier — the same formulas as `timeline/model.py`.
- Fixed frame order: `Timeline::apply` → each `FrameHook` of the show in registration order (`fx`
  tracks, then the Graph) → `osc`/`media`/`cue` tracks → `CueList::update` → programmer → global
  hooks (`hook_global`, the `serve` monitor) → I/O. The same as Python's `_tick`.
- `engine::hook`: `trait FrameHook { frame(t, &mut Universes); input(key, value); reset(t) }`,
  `enum Ev { Cmd, Osc, Widget, Param, Notify }`, `trait EventSink { emit(&Ev) }`. This is how
  `script` plugs in without the engine knowing about Rhai.
- `engine::cues`: `CueList::new(&[Value])`, `go(t, Option<usize>)`, `update(t, &mut Universes)`,
  `reset()`. Key `"1/100"` → `(1, 100)`; linear fade; `follow` chains.
- `engine::player`: `Player::new(show, base, looping)`, `hook(Box<dyn FrameHook>)`,
  `start(osc_port)` (transport thread + OscIn `/spellcaster/play|pause|stop|locate`),
  `Handle {play, pause, stop, locate, cue_go, state}`, `player::current()` is the live player of the process.
- `Registry::add::<A: JsonSchema + DeserializeOwned>(name, doc, fn)`; the error is a `String`.
  `registry::base()` with no `Clock`: `load`, `show_get`, `pause`, `stop`, `locate`, `cue_go`,
  `transport_state`; the transport ones act on `player::current()`. `play_show` and `net` are
  registered by the CLI, which is the one that knows `script` and `protocols`.
- `engine::edit` (wired in `base()`): editing of the open show (`OPEN`, one per process) —
  `show_new`/`show_set`/`show_save`, `track_add`/`track_del`, `key_set`/`key_del`,
  `cue_set`/`cue_del`, `patch_add`/`patch_del`/`patch_check`/`profiles`. A port of
  `gui/api.py` + the footprint of `fixtures/patch.py`; this is how the Tauri GUI and the MCP edit.
- `mcp::Spell` (crate `mcp`) implements `rmcp::ServerHandler` over a `Registry`: one tool per
  command, with the `inputSchema` that `schemars` generated; resources `spell://show` (the `show_get`)
  and `spell://commands` (the `Registry::schema()`). A command error comes back as `isError`, not as a
  JSON-RPC error. `play_show` blocks, so it runs in a thread and the tool returns immediately; while
  the MCP runs, the `play` status line goes to stderr (over stdio, stdout is the JSON-RPC channel).
  `mcp::install` writes the "spellcaster" entry into Claude's config — it is not a registry command,
  so that the AI cannot rewrite its own configuration. stdio only: `rmcp`'s streamable HTTP is a
  `tower::Service` and would still require axum/hyper.
- `trait Output { fn send(&mut self, universe: u16, data: &[u8; 512]); fn close(&mut self); }`.
  Each output has its own thread and a 2-frame queue per universe; `send()` never blocks the engine.
- `script::Fx::new(path, universe)`: compiles the `.rhai` once, state persists between frames; API
  exposed to the script: `set(universe, addr, values)`, `set(addr, values)`, `st(i)`/`st(i, v)`
  (state). Rhai with `default-features = false` + `std, sync, no_custom_syntax, no_time, only_i64,
  no_module, no_closure`; `max_expr_depths(256, 256)` because the default (64/32) refuses the show.
- `script::Graph::new(&Value, Box<dyn EventSink>)`: closed catalogue (`in.*`, `logic.*`, `math.*`,
  `time.*`, `cmd`, `out.*`), topological order at compile time, pins by index, zero per-frame allocation.
- `laser`: `Point`, `Frame`, `optimize(&[Point], Safety) -> Vec<Point>` (dwell, blanking,
  interpolation, kpps limit, minimum figure size), `ild::read/write` (formats 0, 1, 2, 4, 5),
  `trait Dac` for Ether Dream (TCP, with `Emulator`) and IDN (UDP) — Helios (USB) has only the
  frame encoder, no driver,
  `Feed::start(Box<dyn Dac>, pps, buffer_frames, Safety)` with its own thread per DAC.
  Safety is mandatory in the `Feed`: there is no path to the DAC without it.

### Conformance (what "byte for byte" measures)

| Test | Reference | Result |
|---|---|---|
| `cargo test -p engine` (R0 timeline) | `medgrupo_u1.bin` (2577 frames × 512) | equal |
| `cargo test -p script --test medgrupo` (`medgrupo.rhai`) | `medgrupo_u1.bin` | equal across the 2577 frames |
| `cargo test -p laser --test conformance` | `optimize`/`safety`/`.ild` fixtures generated from the Python | equal |
| `capture_sacn.py` (live, `medgrupo_r0.spell`) | sACN packets received on 127.0.0.1 | 89/89 equal |
| `capture_sacn.py --show shows/medgrupo.spell` (live, Rhai fx) | same | 89/89 equal |

### How to build

The repo folder is on Google Drive: `target/` can never be born inside it.

```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cd spellcore
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --workspace
cargo run --release -p bench --bin jitter
cargo run --release -p bench --bin throughput
cargo run --release -p laser --bin feeds -- --secs 30
cargo run --release -p pixelmap --bin map_bench
cargo bench -p bench
```

`spellcore/.cargo/config.toml` (not versioned) pins the same path for whoever forgets the variable;
`target/` is in `.gitignore` as a second barrier. Dependencies and their justification: `spellcore/README.md`.

### Measured numbers (x64 desktop, Windows 11, release)

| Metric | PRD target | Measured |
|---|---|---|
| Jitter between frames, 60 Hz, p99 | < 1 ms | 0.22 ms (max 0.28 ms) |
| Drift over 10 s at 60 Hz | 0 frames | 0 |
| 16 sACN + 16 Art-Net at 60 Hz | < 3 % of one core | 0.2 % (2.7 % on the first run, cold cache) |
| Boot to the first DMX frame | < 2 s | 0.002 s |
| RSS at rest | < 60 MB | 5.6 MB |
| `Timeline::apply` for the whole show | — | 3.73 µs |
| 500-node Graph per frame | < 0.1 ms | 8.4 µs (2.0 µs without `math.expr`) |
| 4 laser feeds × 30 kpps, cpu of the feed threads | < 1 % of one core | 0.83 % over 30 s (GetThreadTimes quantizes at 15.6 ms: run for ≥ 30 s) |
| Pixel mapping, 100,000 px at 60 Hz (CPU, rayon) | < 2 ms per frame | 0.105 ms p50, 0.316 ms p99 (bilinear: 0.196 / 0.493) |
| `spellcore.exe` release binary | < 20 MB | 3.8 MB (2.5 MB before rmcp; 0.95 MB before Rhai) |

## spellgui/web (Tauri GUI) — R5 base

```
spellgui/web/
  canvaskit.js   canvas kit: view {x, zoom, y}, world<->screen, bisect/near (hit-test),
                 sparse selection, marquee, pan, zoom at the cursor, dirty flag, DPR, rAF loop
  timeline.js    timeline on top of the kit: lanes from the .spell, keyframes in parallel arrays,
                 curves (linear, hold, in, out, inout, bezier) the same as the engine's, ruler with
                 timecode, snapping to markers/keyframes, scrub, In/Out, loop, shortcuts from
                 design/SHORTCUTS.md
  index.html     minimal page: opens shows/medgrupo.spell (?show=<path> switches), bar and menu
tests/test_spellgui_timeline.py   headless Chrome --dump-dom over a page that runs the assertions in JS
```

The Tauri app does not exist yet: `spellgui/web/` is the R5 canvas base, served by any static HTTP
server and tested in headless Chrome. `canvaskit.js` is the only place that knows about pan, zoom,
selection, marquee, DPR and dirty flag; the timeline (and the graph, in R9) only draw and respond to
`k.on.{down,move,up,marquee,menu,frame}`. Every hit-test is `CK.bisect`/`CK.near` over lists ordered
by time, and drawing walks only the visible lanes, as in the Python prototype. There is no WebSocket
and no engine: the transport is a local clock and editing goes back into the show's JSON in memory
(`TL.commit`). Colours only by token from `design/tokens/spellcaster.css`.
