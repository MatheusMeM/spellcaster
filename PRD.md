# PRD — Spellcaster v1: core in Rust, previz in Godot, interface through Theme + Face + Graph

You are the orchestrator. Read this whole document before dispatching any agent.
ponytail mode always: the smallest code that works, stdlib and already-present crates before a
new dependency, nothing "for later", `// ponytail: <limit> ; <when to change it>` on every
deliberate simplification. Commits and pushes only from the repo author's account; `Co-Authored-By`
or any AI credit in the history is forbidden.

## 1. Goal

A portable show-control media server, one binary per platform, no installation.
Timeline with simultaneous output of DMX (sACN, Art-Net), OSC, multi-feed laser (ILDA),
video (decode, NDI in/out, RTSP in, Spout out), pixel mapping on the GPU and 3D previz.
Control by GUI, CLI, OSC and MCP, all of them clients of the same command registry.

## 2. Non-goals

- Virtual webcam. NDI and Spout cover the case.
- A JS framework in the frontend. Vanilla HTML, CSS, JS, canvas.
- A database. The show is a `.spell` JSON file.
- A video editor. The engine plays, maps and routes; it does not cut or colour-grade.
- Third-party binary plugins. Custom effects are embedded scripts.

## 3. Performance kernel (non-negotiable acceptance)

Measured by `bench/` in CI and on a real machine. Nothing lands in `main` if it regresses.

| Metric | x64 desktop target | Pi 4 / Pi 5 target |
|---|---|---|
| Jitter between engine frames, p99 | < 1 ms | < 3 ms |
| Accumulated drift over 1 h at 60 Hz | 0 frames | 0 frames |
| 16 sACN universes + 16 Art-Net at 60 Hz | < 3 % of one core | < 15 % |
| Pixel mapping, 100,000 pixels at 60 Hz, GPU | < 2 ms per frame | CPU fallback: 20,000 pixels at 30 Hz |
| 1080p60 H.264 decode + NDI out | < 25 % of one core (hardware decode) | 1080p30 on the V4L2 decoder |
| 4 laser feeds at 30 kpps each | < 1 % of one core | same |
| Boot to the first DMX frame | < 2 s | < 5 s |
| RSS at rest, no video | < 60 MB | < 60 MB |
| Size on the USB stick | < 150 MB with GStreamer and NDI | static binary < 20 MB without media |
| GUI: click → engine state reflected latency | < 16 ms | n/a |

Kernel rules:
- Engine loop in its own thread, high priority, `timeBeginPeriod(1)` on Windows,
  sleep until 1 ms before the target and spin for the rest.
- Zero allocation on the hot path: per-universe and per-DAC buffers pre-allocated;
  `Vec::with_capacity` outside the loop; no `String` per frame; no `clone` of the show.
- Keyframe evaluation by binary search; per-track caches invalidated only on edit.
- The engine process is separate from the GUI process. A slow GUI never delays DMX.
- Each network output on its own thread with a fixed-size `SPSC` queue; an old frame is dropped, never queued.

## 4. Architecture

```
spellcore/          Rust binary, no GUI
  engine/           clock, timeline, cues, command registry
  protocols/        sacn, artnet, osc, netscan
  laser/            frame, optimize, safety, dac::{etherdream, helios, idn}
  media/            gstreamer pipeline, ndi in/out, rtsp in, spout out, texture pool
  pixelmap/         wgpu compute + rayon fallback
  ipc/              local socket, binary frames, shared memory for preview
  mcp/              rmcp, tools/resources generated from the registry
  script/           embedded rhai: fx tracks and the compiled Graph (section 10)
  cli               spell play|net|serve|tui|mcp|export|agent
spellgui/           Tauri: WebView2 + web/ (index, canvaskit.js, timeline.js, graph.js, face.js, widgets/, themes/)
faces/              operation surfaces (.face.json) — layout, views, widgets
themes/             looks (theme.json + theme.css) — what today sits in spellcaster/gui/web/skins/
spellviz/           Godot 4 project + gdext in Rust: 3D stage, fixtures, beams, LED walls
shows/ profiles/ tests/ bench/
```

Fixed contracts:
- 1-based universes. Art-Net converts to port-address internally.
- Every output implements `trait Output { fn send(&mut self, universe: u16, data: &[u8; 512]); fn close(self); }`.
- Every product command is a `registry` entry with `serde` + `schemars` types.
  CLI, WS, OSC-API and MCP only call the registry; they never have logic of their own.
- `spellcore` knows nothing about the GUI or Godot. Only IPC.
- Versioned `.spell` format; migration always forward.

## 5. Minimal examples

A command in the registry (generates CLI, WS, MCP with no extra code):
```rust
#[command(mcp)]
/// Loads a show and leaves it stopped at t=0.
fn load(path: PathBuf) -> Result<ShowInfo>
```

IPC / WebSocket message:
```
{"id": 7, "cmd": "locate", "args": {"t": 12.5}}
{"id": 7, "result": {"t": 12.5, "state": "paused"}}
```
Binary monitor broadcast: `topic:u8 | universe:u16 | 512 bytes`.

Pixel mapping track in the .spell:
```json
{"type": "pixelmap", "source": "media:1", "map": "profiles/ledwall_96x54.json",
 "out": {"protocol": "sacn", "first_universe": 10, "order": "grb"}}
```

Laser track:
```json
{"type": "laser", "dac": "etherdream:192.168.0.50", "clip": "shows/logo.ild",
 "keys": {"scale": [[0, 0.2], [4, 1.0, "easeInOut"]], "rot": [[0, 0], [8, 360]]},
 "safety": {"min_size": 2000, "max_intensity": 200, "zone": [-1, -0.2, 1, 1]}}
```

Theme (today's `skin.json`, renamed; format kept):
```json
{"name": "quicksilver", "chrome": {"transport": "round", "visualizer": "scope", "shape": "chrome.svg"},
 "vars": {"--bg": "#1c1e21", "--accent": "#39d0ff", "--bevel": "1px"}}
```

Face (a show's operation surface; several views like the WMP `<VIEW>`s):
```json
{"name": "operador_medgrupo", "theme": "quicksilver", "mode": "performance",
 "views": {
   "full":    {"grid": "12x8", "widgets": ["transport", "go", "cues", "vu"]},
   "compact": {"grid": "4x1",  "widgets": ["go", "transport"], "shape": "pill"}},
 "widgets": [
   {"id": "go",    "type": "button", "label": "GO", "size": "xl", "at": [0, 0, 4, 4], "bind": "graph:go.press"},
   {"id": "cues",  "type": "cuelist", "at": [4, 0, 8, 6]},
   {"id": "vu",    "type": "universes", "universes": [1, 2], "at": [4, 6, 8, 2]},
   {"id": "transport", "type": "transport", "at": [0, 4, 4, 1]}]}
```

Graph (behaviour; lives in the `.spell`, runs in the engine, with or without a GUI):
```json
{"nodes": [
   {"id": "go",    "type": "in.widget", "widget": "go"},
   {"id": "k",     "type": "in.key", "key": "Space"},
   {"id": "any",   "type": "logic.or"},
   {"id": "next",  "type": "cmd", "cmd": "cue_go"},
   {"id": "flash", "type": "out.widget", "widget": "go", "prop": "glow", "hold_ms": 300}],
 "edges": [["go.press", "any.a"], ["k.down", "any.b"], ["any.out", "next.trigger"], ["next.done", "flash.in"]]}
```

Previz (Godot receives from the engine over IPC, it never reads the .spell by itself):
```
frame_dmx(universe, [u8; 512])  → every patched fixture updates pan/tilt/colour/beam
frame_video(texture_id)         → the LED wall shows the mapping in real time
```

## 6. Phases and acceptance

- **R0 — Core and protocols.** `spellcore play show.spell` reproduces the conformance fixtures in `tests/` byte for byte (sACN and Art-Net). `bench/jitter` within target. CLI `net`.
- **R1 — Timeline, cues, .spell, fx script, Graph runtime, OSC transport.** The Graph (section 10) compiles to Rhai and runs in the engine; OSC/MIDI/keyboard/timer sources work with no GUI. Headless player on the Pi over SSH. Acceptance: a 500-node graph evaluated in < 0.1 ms per frame.
- **R2 — Media:** GStreamer decode with hardware, NDI in/out, RTSP in, Spout out. Preview over shared memory. Acceptance: 1080p60 within the CPU target.
- **R3 — Pixel mapping** wgpu + rayon fallback. Acceptance: 100,000 pixels at 60 Hz < 2 ms.
- **R4 — Multi-feed laser:** Ether Dream, Helios, IDN; safety in the engine; 4 simultaneous feeds.
- **R5 — Tauri GUI:** `canvaskit.js` (pan, zoom, selection, hit-test by bisect, dirty flag, DPR) shared by timeline and graph; canvas timeline (tracks, keyframes, curves, snapping to markers, scrub, zoom, loop, record arm); Patch, Outputs, Network, Log panels; **Theme and Face runtime**: widget catalogue, views switched by shortcut, performance mode (kiosk, fullscreen, touch, nothing editable), frameless window with a shape from the theme's SVG; the 6 existing themes with runtime switching. Acceptance: build a 3-min show from scratch in the GUI; open `faces/operador_medgrupo.face.json` in performance mode and operate the show from it alone.
- **R6 — Godot previz:** stage, fixtures with volumetric beams, LED walls with the mapping, projected laser. Acceptance: 60 fps with 64 fixtures and 2 LED walls.
- **R7 — MCP with rmcp** (stdio and HTTP), `spell mcp install`; Theme/Face/Graph tools (`face_get`, `face_patch` with JSON Patch, `graph_get`, `graph_patch`, `theme_set`), resources `spell://face`, `spell://graph`, `spell://ui/screenshot`, `spell://ui/events`. Acceptance: from an AI session, scan the network, patch, build a timeline, build a 4-button Face wired by Graph to cues and hit play without touching the GUI.
- **R8 — Packaging:** Windows onedir on the USB stick, Linux x64, static Pi aarch64; CI with `bench/` as the gate.
- **R9 — Face and Graph editors + Agent panel.** Face editor (drag widgets on the grid, properties, views, live theme preview), canvas Graph editor (catalogue nodes, wires, search by type, collapse into a subgraph, live values on the pins), unlimited undo via inverse JSON Patch, an AI "proposal" shown as a diff before applying, rehearsal mode (the graph armed only on virtual/previz outputs). Agent panel: a chat that starts `claude` (CLI) as a subprocess with the Spellcaster MCP registered and streams the conversation; no agent loop of its own. Acceptance: an untrained operator builds a show Face in 10 min; the AI, from a prompt in the panel, generates a Face + Graph for `medgrupo.spell` and the operator applies it after seeing the diff.

Each phase delivers something usable on a job. Fixed order. A phase only closes with the bench green.

## 7. Rules for the agents

- One agent per top-level directory; no agent edits another's file. The orchestrator does the wiring between modules.
- Every module with non-trivial logic leaves a `cargo test` and, if it touches the hot path, a `bench/` with Criterion.
- A new dependency only with a one-line justification in the PR and the binary size measured.
- Licenses: GStreamer and ffmpeg LGPL with dynamic linking; NDI SDK under an EULA with redistribution; no GPL code (x264 is out; encode via openh264 or hardware).
- Windows: cp1252 console, ASCII-only tests; heavy binaries generated in `%TEMP%` and moved.
- Agent report: 25 lines at most, with the bench numbers, the `ponytail:` simplifications and what was left out.

## 8. Risks and response

| Risk | Response |
|---|---|
| The JS canvas timeline hits its ceiling with tens of thousands of keyframes | Virtualize by viewport first; WebGL later; egui only if measured |
| GStreamer + NDI bloat the USB stick | A minimal plugin set listed in `media/PLUGINS.md`; measure every release |
| Godot turns into a project of its own | Scope locked to previz; no show editing inside Godot |
| No GPU on the Pi | Mandatory CPU fallback, tested in CI with `WGPU_BACKEND=none` |
| The laser burns or dazzles | Safety in the engine, never in the GUI; minimum-figure and stopped-point tests before R4 closes |
| The node editor turns into a TouchDesigner | Closed catalogue (section 10): nodes only for input, logic, command and output; no signal, no video, no render. A new request becomes a track, not a node |
| A Graph edited by the AI fires the laser or a blackout live | Every MCP edit is a proposal with a diff; applying requires a click; rehearsal mode by default on an armed show |
| Custom widgets without end | A single `canvas` widget with a Rhai drawing script; no user-defined widgets in v1 |

## 9. Relation to the Python prototype (`spellcaster/`)

The Python package `spellcaster/` (F0–F6) stays in the repo as the reference implementation and the conformance-fixture generator. It gets no new functionality. `spellcore` has to reproduce byte for byte the sACN/Art-Net output of `shows/medgrupo.spell` generated by the Python. The skins in `spellcaster/gui/web/skins/` are reused by the Tauri GUI with no format change.

## 10. Interface: Theme, Face and Graph

The original idea: skins that change the behaviour of the interface, one per show, inspired by the Windows Media Player skins, plus a node-based interface editor usable by people and by AI, with an embedded agent. The right reading of the WMP skins is that `skin.xml` declared **views, buttons, sliders and what each of them did**, not just colours. A "skin" was three things glued together. Here they come apart, because each layer changes for a different reason and is edited with a different tool:

| Layer | What it is | Where it lives | Who edits it |
|---|---|---|---|
| **Theme** | Look: colour tokens, typography, bevel, glow, window shape (SVG), transport and visualizer style | `themes/<name>/theme.json` + `theme.css` | Designer; AI via `theme_set` |
| **Face** | Surface: which widgets exist, where, in which views, what is editable | `faces/<name>.face.json`, referenced by the show (`"face": ...`) or inline | Operator in the Face editor; AI via `face_patch` |
| **Graph** | Behaviour: what each widget, key, OSC, MIDI, timer or marker does | `"graph"` inside the `.spell` | Node editor; AI via `graph_patch` |

Rules:
- **One Theme dresses any Face; one Face accepts any Theme.** Theme precedence: user preference > `face.theme` > `feiticaria` (default).
- **The Face is a view of the Graph; the Graph lives in the engine.** A Face button is an `in.widget` node. The same Graph runs on the Pi with no GUI, with `in.osc`, `in.midi`, `in.key`, `in.timer`, `in.marker` as sources. There is no second runtime in the GUI; the GUI only renders state and sends events over IPC.
- **Everything is JSON, diffable, with JSON Patch.** The canvas editors are renderers/editors of that JSON. Undo = inverse patch. The AI edits through the same path as the human.
- **Two Faces by default in every show:** `editor` (the full application) and `performance` (only what the operator needs; kiosk, fullscreen, touch, nothing editable, no menu). Switch with one key. That is what "each project has its own skin" means in practice: the designer builds it in the editor, the operator gets a four-button screen.
- **Views per Face** (full, compact, touch, …) like the WMP `<VIEW>`s: same Face, different arrangements, a shortcut to switch. Frameless window, shape by `clip-path` from a Theme SVG, transparency via Tauri: this is where the interface gets wild without costing performance.

Widget catalogue (closed in v1): `button`, `toggle`, `fader`, `knob`, `xy`, `color`, `label`, `lcd`, `meter`, `universes`, `timecode`, `transport`, `cuelist`, `timeline`, `netscan`, `log`, `visualizer` (bars/scope/script), `canvas` (drawing by Rhai script, the only "custom" one). Each widget is `{id, type, at:[col,row,w,h], props, bind}`; `bind` points at a Graph pin.

Node catalogue (closed in v1): inputs `in.widget | in.key | in.osc | in.midi | in.timer | in.marker | in.state` (time, current cue, universe, fixture); logic `logic.and|or|not|latch|toggle|debounce|counter|select`, `math.map|curve|expr` (expr = one line of Rhai), `time.delay|hold`; command `cmd` (any registry entry, typed by `schemars`); outputs `out.widget` (a widget prop), `out.osc`, `out.param` (fixture.channel via the patch), `out.notify`. No signal, audio, video or render: that is a track. A subgraph can be collapsed into a node with pins; it is the only reuse mechanism.

Three additions to the catalogue, **this round's proposal** (they close the two gaps `design/FUNCOES/orquestrador.md` points out against Chataigne; awaiting a vote in `design/DECISOES.md`):

- **`state`** — state machine. Config `group` (default `"main"`) and `initial`; inputs `enter`/`exit`, output `active`. One active per group: a pulse on `enter` turns this one on and the others in the group off, `exit` turns it off, `locate`/`stop` returns to `initial`.
- **`module`** — an app declared in `modules/<name>.json` (Chataigne's `module.json`). One input per `parameter` (change → `Ev::Param{target:"<module>/<path>"}`, `norm` maps 0..1 before the clamp to `min`..`max`), one level output per `value` (fed by `input {key:"module:<module>/<path>"}`), one trigger input per `command` (→ `Ev::Cmd{name:"<module>/<cmd>"}`). This is what makes a separate app interoperable without the PATCHBAY knowing the app.
- **`"mute": true` and `"state": "<id>"` on any node** — the node does not emit: outputs at 0, no events, a pending `time.delay` cancelled. It is TouchDesigner's Bypass and Chataigne's state container, with no new verb (rule 9 of `design/FUNCOES/README.md`).

Embedded agent: the Agent panel implements no agent loop. It starts `claude` (the Claude Code CLI) as a subprocess with the Spellcaster MCP registered, streams the conversation and shows every `face_patch`/`graph_patch` proposal as a diff with an Apply button. `// ponytail: claude subprocess ; own loop over the API only if the CLI is not installed on the operator's machine`. The AI sees what it did through the `spell://ui/screenshot` resource and sees the operator through `spell://ui/events` (the last 200 widget events).

Show safety: laser, blackout and output-arming commands go through the same Graph, but the engine has the last word (safety, section 8). A show armed on real outputs enters **rehearsal mode** by default for Graph edits: the new graph runs against virtual outputs (previz) until the operator arms it.
