# Spellcaster — roadmap (Python prototype F0–F7; state of the Rust phases in section 7)

Portable show control by Feitiçaria Industrial: timeline + sACN / Art-Net / OSC / ILDA (laser), web GUI with skins, embedded MCP, standalone player, Lite version for the Raspberry Pi (pure CLI over SSH). Repository: github.com/MatheusMeM/spellcaster. Python package `spellcaster`, CLI `spell`.

Origin: the sACN generator in `show_medgrupo.py` (MED GRUPO RJ plenary, 09/2026). Everything already calibrated there (E1.31 packet, BSW/Sharpy/BX-402 profiles, group geometry with hysteresis and ramp, cues on video cuts) migrates here as production code.

## 1. Stack decisions

| Decision | Choice | Why |
|---|---|---|
| Language | Python 3.13, stdlib first | Reuses what exists; runs natively on the Raspberry Pi; the team already works in Python. |
| GUI | Web (vanilla HTML/CSS/JS) served by the process itself; window via `pywebview` (WebView2, already present on Windows 10/11) | One GUI serves the desktop, the Pi Lite (reached through a browser) and the tablet on the console. Skins = CSS. Zero JS framework. |
| Timeline | Our own `<canvas>` editor | No off-the-shelf library covers a show timeline (heterogeneous tracks, curves, cues, video markers). |
| Windows packaging | PyInstaller **onedir** in a folder on the USB stick + `Spellcaster.exe` | Onefile extracts into `%TEMP%` on every start (slow, sets off antivirus). Onedir opens in < 1 s and writes nothing outside the USB stick. |
| Lite (Pi) | Same package without `pywebview`; `pip install` or tarball; `systemd` service | Same code, same GUI (through the browser), with no screen on the Pi. |
| Show file | Readable JSON (`.spell`) | Git-friendly; the MCP and the human read the same file. |
| MCP | Generated automatically from the engine's command registry | AI-native by construction: each command exists once and shows up in CLI, GUI, OSC-API and MCP. |
| Laser (ILDA) | No ILDA board in the PC: the engine generates ILDA frames and sends them to network/USB DACs: Ether Dream (open UDP/TCP), IDN-Stream (ILDA Digital Network standard, Ethernet), Helios (USB, via `libusb`/`pyusb`). `.ild` files as clips. | These are the open protocols; they cover the DACs Feitiçaria uses and what a Pi can feed. Analogue ILDA output only through a DAC. |
| CLI mode | `spell` works 100 % from a terminal: `play`, `net`, `patch`, `tui` (`curses` monitor) | On the Pi over SSH there is no GUI; everything the GUI does has a CLI verb because both call the registry. |
| Dependencies | `pywebview`, `mcp` (official SDK), optional `pyusb` (Helios), `python-rtmidi`, `numpy` | Only what a line of stdlib cannot do. |

## 2. Architecture

```
spellcaster/
  core/
    clock.py        single clock (perf_counter), 30/60 Hz tick, play/pause/stop/locate transport
    registry.py     @command: name, typed args, doc → CLI + OSC-API + MCP + GUI
    universe.py     DMX buffers (N universes), HTP/LTP merge per source
    engine.py       loop: timeline → parameters → fixtures → universes → outputs
  protocols/
    sacn.py         E1.31 out/in, discovery (239.255.250.214), priority, sync
    artnet.py       ArtDmx out/in, ArtPoll/ArtPollReply, ArtSync
    osc.py          OSC 1.0 out/in (UDP), bundles, timetag, pattern matching
    netscan.py      interfaces, ArtPoll, sACN discovery, mDNS _osc._udp, Ether Dream/IDN discovery, IP/subnet suggestion
    ilda/
      frame.py      ILDA point (x, y, r, g, b, blank), frame, optimization (blanking, dwell, interpolation, scan limit)
      ild.py        .ild read/write (formats 0/1/2/4/5) — migrated from ilda_gen.py
      etherdream.py Ether Dream: discovery broadcast, TCP point stream, buffer
      idn.py        IDN-Stream (ILDA Digital Network) over UDP
      helios.py     Helios USB (optional, pyusb)
  fixtures/
    profile.py      profile by channel name (pan16, tilt16, color, shutter…), ranges and wheels
    fixture.py      Fixture(profile, universe, addr).set(dimmer=…, color="amarelo", pan_deg=…)
    group.py        Group + geometry (straight, side signal, amplitude), pan hysteresis, ramp
    library/        BSW, Sharpy, BX-402, BX-940, WLED bar, Butrym laser… (JSON)
  timeline/
    model.py        Sequence → Track → Clip/Keyframe; curves (linear, ease, hold, bezier); markers
    tracks.py       types: fixture-param, dmx-raw, osc, artnet-raw, laser (.ild clip or generator), cue-trigger, media-control, python-fx
    cues.py         cue list (GO / follow / wait), snapshots, fades
    render.py       evaluates the timeline at t → parameter dictionary
  player/
    player.py       standalone/headless mode: loads a .spell, runs it, accepts OSC/HTTP for transport
  gui/
    server.py       HTTP + WebSocket (stdlib `http.server` + manual WS handshake or `websockets`)
    web/            index.html, timeline.js, patch.js, monitor.js, skins/
    window.py       pywebview (desktop only)
  mcp/
    server.py       MCP stdio + streamable HTTP, tools/resources/prompts generated from the registry
  cli.py            spell play|stop|net|patch|calib|laser|serve|tui|mcp|export — every verb is a registry command
  tui.py            curses monitor: transport, universes, sources, laser, log — for SSH on the Pi
  show.py           .spell load/save, version migrations
```

Principle: **the engine does not know a GUI exists**. GUI, CLI, OSC-API and MCP are clients of the same command registry over WebSocket/stdio. That is what makes Lite and MCP cheap.

## 3. Phases

Each phase ends with something usable in real work. Fixed order; the timings are an estimate for one person working part time.

### F0 — Foundation and protocols (week 1–2)
- Migrate `sacn` from `show_medgrupo.py` into `protocols/sacn.py` with N universes and input.
- `artnet.py` out/in + ArtPoll; `osc.py` out/in.
- `clock.py`, `registry.py`, `engine.py` with a Python `look(t)` as a track (compatible with the MED GRUPO show).
- CLI: `spell play shows/medgrupo.py` plays the current show without a GUI.
- Acceptance: Capture receives identical sACN and Art-Net; an `ffmpeg`-style loopback test compares packets byte for byte against recorded fixtures.

### F1 — Network scan (week 2)
- `spell net`: interfaces, IP/mask, Art-Net nodes (ArtPollReply), sACN sources (discovery + listening), OSC devices (mDNS), latency.
- Automatic suggestion: "Art-Net requires 2.x.x.x or 10.x.x.x; your board is on 192.168…" with a ready `netsh` command (it does not run by itself).
- Acceptance: report in text and JSON in the GUI and in the MCP.

### F2 — Profiles and patch (week 3)
- JSON profile format by channel name, with ranges (zoom 80–255 = 13–36°), named wheels, 16 bit.
- Import the already-calibrated profiles; a GDTF importer stays in the backlog.
- Patch: universe/address, overlap detection (the 17 ch bug on a 16-channel spacing becomes an error right away).
- `Group` with calibratable geometry and the `spell calib hold|sweep` tool (today's `bsw_hold`/`bsw_multi`).
- Acceptance: the MED GRUPO show rewritten with named fixtures, no channel index in the code.

### F3 — Timeline engine + show file + Player (week 4–5)
- Sequence/Track/Keyframe model, curves, markers imported from video (`ffmpeg` scene detect) and from audio (beats).
- Tracks: fixture parameter, raw DMX, OSC, raw Art-Net, cue-trigger, media-control (Capture media player, VLC over OSC), python-fx (an embedded `f(t)` function for generative effects).
- Cue list with GO/follow/wait and fades between snapshots.
- Laser: `laser` track with `.ild` clips and generators (shapes, text, figure scanner like the MED GRUPO one), transforms (position, scale, rotation, colour), safety (minimum size limit, forbidden zone, maximum intensity) and output to Ether Dream / IDN / Helios in its own thread at 20–30 kpps.
- `spell play show.spell` headless = **standalone Player** ready; transport over OSC (`/spellcaster/play`), HTTP and keyboard.
- Acceptance: the MED GRUPO show expressed 100 % in `.spell`, output identical to the Python version; `medgrupo_laser.ild` playing on an Ether Dream (or on the network emulator) synced to the timeline.

### F4 — GUI (week 6–9)
- Layout inspired by Chataigne (dockable panels: Patch, Timeline, Outputs, Network, Inspector, Log) and Adobe (timeline with tracks, keyframes, curves, snapping to markers, scrub, zoom, in/out, loop, time/timecode ruler).
- Canvas timeline: multiple selection, dragging, copy/paste, per-keyframe easing, solo/mute per track, live parameter recording (record arm).
- Output monitor: universes as a grid, channel VU, input sources.
- Windows Media Player-style skins: `skins/<name>/` folder with `skin.json` + `skin.css` + images; customizable chrome (borders, transport buttons, visualizer); light/dark theme; default Feitiçaria skin (design system).
- Acceptance: build a 3-minute show from scratch in the GUI, with 8 moving lights, 2 universes, OSC to a video player, and record/play it back.

### F5 — MCP and AI-native (week 9–10)
- Generator: walks the `registry` and emits MCP tools with a typed JSON schema from the signatures; resources: current show, patch, network, log; prompts: "build a show from this video", "calibrate this group".
- Transports: stdio (Claude Desktop/Code) and streamable HTTP (remote, Pi Lite).
- The `spell mcp install` command writes the entry into `claude_desktop_config.json` / `.mcp.json` (asks for confirmation).
- Engine events (frame, cue, error) as MCP notifications.
- Acceptance: from a Claude session, without touching the GUI: scan the network, patch, build a timeline with cues on a video's cuts, hit play and read the output monitor.

### F6 — Portable packaging and Lite (week 10–11)
- Windows: PyInstaller onedir → `Spellcaster/` on the USB stick with `Spellcaster.exe`, `shows/`, `skins/`, `profiles/`, `config.json`. Everything relative to the executable; nothing in `%APPDATA%`. The first boot shows the network scan.
- Code signing (certificate) to reduce antivirus alarm; without it, document the exception.
- Lite: `pip install spellcaster[lite]` or the `spellcaster-lite-aarch64.tar.gz` tarball; `spell serve --headless` as a `systemd` service; GUI through the browser at `http://spellcaster.local:8000` **or CLI only over SSH**: `spell play show.spell`, `spell tui` (curses monitor), `spell net`, `spell laser test`. No GUI installed, no X, no browser. Optional SD card image via `pi-gen`.
- CI (GitHub Actions): build Windows x64, Linux x64, Linux aarch64; protocol tests on loopback.
- Acceptance: USB stick in a clean PC → double click → show running in < 10 s. A Pi Zero 2 W delivering 8 sACN universes + Art-Net + OSC at 40 Hz and an Ether Dream laser at 20 kpps, operated over SSH alone.

### F7 — Backlog (after using it on a job)
LTC/MTC timecode in, MIDI in/out, GDTF/MVR import, LaserCube and other proprietary DACs, graphical laser frame editor, Art-Net/sACN input with merge for "passthrough + overlay", native NDI/video in the player, macOS build, multi-machine sync (master/slave over a UDP clock), live Lua/Python scripting, unlimited undo with visual history.

## 4. What not to do now
- A JS framework (React/Electron): it doubles the size and the boot time from the USB stick; our own canvas gives the control a timeline demands.
- A database: the show is a JSON file.
- Binary plugins: all Python; custom effects are `python-fx` tracks.
- Video inside the engine: in F3 the player controls external players (Capture, VLC, Resolume) over DMX/OSC. Native video is F7.

## 5. Risks
| Risk | Mitigation |
|---|---|
| Jitter of the Python loop at 60 Hz with many universes | Loop in its own thread with `perf_counter` and drift compensation; batched sending per universe; measured in F0 before choosing 30 or 60 Hz as the default. |
| WebView2 missing on a very old PC | Fallback: it opens in the default browser (`spell serve --browser`). |
| Antivirus on the USB stick | Onedir + signing; exception instructions in the `LEIA-ME`. |
| The canvas timeline turning into a project of its own | The F4 scope is locked to the listed verbs; the rest goes to F7. |
| Laser: a stopped point or too small a figure burns or dazzles | Safety in the engine, not in the GUI: kpps limit, minimum figure size, exclusion zone per DAC, software shutter; loopback tests before F3 closes. |
| The MCP generating too many tools and confusing the model | The registry marks `mcp=True` only on the high-level commands; the low-level ones sit in a generic `run_command` tool. |

## 6. First step
F0 starts by extracting `protocols/sacn.py` and `core/clock.py` from `show_medgrupo.py`, with the MED GRUPO show as the regression test: same output, byte for byte.

## 7. Rust phases (PRD v1.1) — state on 10/09/2026

The plan above (F0–F7) was the Python prototype's and is done up to F6. The product follows
`PRD.md`: core in Rust, previz in Godot, Tauri GUI with Theme/Face/Graph.

| Phase | Acceptance (PRD §6) | State | Blocker |
|---|---|---|---|
| R0 core and protocols | fixtures byte for byte, `bench/jitter` on target, CLI `net` | done | — |
| R1 timeline, cues, .spell, fx, Graph, OSC | 500-node graph < 0.1 ms/frame; headless player | done (8.4 µs) | — |
| R2 media (GStreamer, NDI, RTSP, Spout) | 1080p60 within the CPU target | pending | SDKs not installed (GStreamer, NDI) |
| R3 pixel mapping (rayon; wgpu later) | 100,000 px at 60 Hz < 2 ms | done (0.105 ms p50 / 0.316 ms p99 per frame; bilinear 0.196 / 0.493) | standalone crate: wiring the frame source to the player waits on R2 |
| R4 multi-feed laser | Ether Dream, IDN; safety in the engine; 4 feeds | done (0.83 % cpu) | — |
| R5 GUI (own window) | 3-min show from scratch; Face in performance mode | in use since v0.1.0: `spellcaster.exe` (crate `spellcore/gui`, `tao` + `wry`/WebView2, no Tauri) starts the bus in process and opens at `spellgui/web/laser3d/app.html` — the 3D laser projector (full PBR model, SolidWorks camera per view, HUD with LEDs, tabbed drawer, Pino on the camera, key+MIDI bindings, VIDEO menu with presets, wall in WebGL), wired to the registry by `bus.js`, without a network; the other pages wired by the `nav.js` bar; Theme and the editors (R9) missing | language selector (UI is English now); usability pass |
| R6 Godot previz | 60 fps, 64 fixtures, 2 LED walls | pending | Godot not installed |
| R7 MCP with rmcp | an AI session builds and plays a show without a GUI | done over stdio; show editing (patch, track, key, cue) in the registry (`engine::edit`) | `spell://face`/`spell://graph` pending; streamable HTTP transport at `/mcp` delivered by F1 (`serve`) |
| R8 packaging | onedir, Linux, static Pi; CI with the bench as a gate | done | — |
| R9 Face/Graph editors + Agent panel | an operator builds a Face in 10 min | pending | depends on R5 |

## 8. Design — rounds and branches (10/09/2026)

The design department works in its own branches and publishes each round as an HTML prototype
(three.js) in an artifact; the owner's vote decides what goes in. Rule: function before UI, and
the program is the photorealistic 3D model of the device it controls.

| Round | What | Branch | State |
|---|---|---|---|
| 1 | static moodboard | `design/0.1.2` | rejected |
| 2 | glass in GLSL, raymarched cube, splash, `.wmz` skins, ICE theme | `design/0.1.2` | voted |
| 3 | ILDA player in the LASER theme, Aprendiz as the menu, `TEMAS.md` function→theme map | `design/0.1.2` | voted; turned towards the device |
| 4 | the program is the 3D projector (PBR), rear panel = menu, lid = preferences, Pino in place of Aprendiz | `design/0.1.2` | voted |
| 5 | real rear panel, optical bench and beam in GLSL, splash on the wall, SolidWorks camera, key + MIDI bindings, 3D Pino, laser design system (`design/laser/SISTEMA.md`) | `design/0.1.2` | published, awaiting vote |
| 6 | the laser as the orchestrator module `laser/1`: addresses, `module.json`, `graph.json`, ORCHESTRATOR panel, test in `tests/test_laser_graph.py` | `design/0.1.3` | published, awaiting vote |
| FUNCOES | functions by reference (Blender, TouchDesigner, Resolume, MadMapper, Capture, Chataigne): `ilda-player`, `ndi-ilda`, `orquestrador`, `cenas-cues-dmx`, `cenario-interativo`, `aprendiz-menu` | `design/funcoes-referencia` | in use by the rounds |
| 0.1.4 | single base: merge of `funcoes-referencia` + `0.1.3`, `INTEGRACAO` became `design/FUNCOES/integracao-laser.md`, rounds 2–4 and the 2D Pino out of the tree | `design/0.1.4`, in `main` | done |

Editing by JSON Patch (workstream `patch`): `show_patch` (`add`, `remove`, `replace`, `test`,
all-or-nothing, returns the `undo` ops and the revision `rev`), `graph_get`/`face_get` in the registry,
`graph_check` in the CLI and the resources `spell://graph` and `spell://face` in the MCP. The graph
becomes AI-editable before any canvas exists.

Integration rounds in `main` (branches `integracao-2`, `integracao-4`, 09–10/09/2026): rounds 5 and 6
became product code in `spellgui/web/laser3d/`, workstream by workstream (camera, chassis, optics,
HUD, Pino, DAC, `.ild` performance, charset, video), each with a headless gate on the three views and
`node --test` tests. Detail per release in `CHANGELOG.md`.

Design pending: round 7 = FÓSFORO (NDI/Spout → ILDA converter on the NET port) and PATCHBAY
(orchestrator UI).

## 9. What is missing, and what runs in parallel right now

With no external blocker, each line is an independent agent (new code in its own crate or in CI,
without touching what is already conformant):

| Workstream | Delivery | Acceptance | Depends on |
|---|---|---|---|
| Design | round 7 (FÓSFORO, PATCHBAY) | owner's vote | vote on rounds 5 and 6 |
| F1 serve | crate `spellcore/serve` and `spellcore serve`: HTTP (`/commands`, `/show`, static), JSON-RPC WebSocket with `show`/`transport`/`log`/`widget` events, binary DMX monitor at 40 Hz, streamable MCP at `/mcp`, `input` and `resume` commands; `--dir` = repo root | done: `cli/tests/serve.rs` starts the binary and closes the contract end to end | — |
| F2 patch | fixture profiles and patch in the engine (`patch_add`, `patch_del`, `patch_check`, `profiles`, `profile_get`) and `show_patch`: JSON Patch (RFC 6902) over the open show, with `rev` and `undo` | done: `engine/tests/patch.rs` and `engine/tests/edit.rs` green | — |
| F4 module | `module.json`: `engine::module` (Module/Param/Cmd, `load`, `check`), commands `module_add/del/list/get`, `modules/laser.json` | `engine/tests/module.rs` green | — |
| Graph runtime | `state` and `module` nodes and the `mute`/`state` keys on any node (`script/graph.rs`) | 500 nodes still < 0.1 ms/frame (8.7 µs); semantics in `design/DECISOES.md` | owner's vote; `module.json` format agreed with the `module` workstream |
| F9 LASER app | `laser_*` in the registry (dacs, open, play, stop, close, param, stats, files) + page `spellgui/web/laser.html` | `cli/tests/laser.rs` green against the Ether Dream `Emulator` | bus contract (F1) for the page |
| F8 PAPER THEATER | programmer in the engine (`level_set`, `level_clear`, `level_get`, `cue_capture`, `fixture_set`, `profile_get`) and the page `spellgui/web/teatro.*`: patch, cues with GO and a clickable set | `engine/tests/programmer.rs` and `node --test spellgui/web/test/teatro.test.js` green | the page waits on the F1 bus and the F2 `show_patch` |
| FÓSFORO core | `laser::trace`: RGBA → outlines → ILDA frame (`paths`, `trace`, bin `trace`) | a 1080p frame in < 8 ms in release; square/circle/two objects/max_points in the tests | — (done: 4.4 ms with 20 objects; the R2 NDI is still missing to feed it) |
| F5 face | `spellgui/web/bus.js` (bus client), `widgets.js` (typed parameter → widget: rules 1 and 3 of `design/FUNCOES/README.md`), `face.js`/`face.html` and `faces/quatro.face.json` | the 4-button Face opens in kiosk mode and operates the show through the registry alone | the `input` command and the bus (F1) |
| F7 sequencer | `spellgui/web/timeline.js` wired to the engine: editing via `key_set`/`key_del`/`show_patch`, transport and DMX monitor over the bus, offline mode intact | dragging a keyframe changes the show in the engine and the playhead follows the player | the `serve` contract (F1) |
| patchbay | graph editor in `spellgui/web` (`catalog.js`, `graph.js`, `patchbay.html`), editing via `show_patch` with undo | open `shows/patchbay_demo.spell`, create a node, wire, group, undo | `serve` (bus) and `show_patch`/`graph_check` to leave offline mode |

Blocked until an SDK is installed (owner's decision, not an agent's): R2 (GStreamer + NDI SDK),
R6 (Godot 4). R9 waits on R5.

Already done: F0–F6, R0, R1, R3, R4, R5 base, R7 (stdio), R8, design merge into `0.1.4`, ponytail
audit (−2,900 lines), CI with release by tag, docs (README, INSTALL, LICENSE, ARCHITECTURE, PRD).

R7 delivered the `mcp` crate (rmcp 3.2, stdio), one tool per registry command, the resources
`spell://show` and `spell://commands` and `spellcore mcp install`. Of the PRD §6 acceptance — "scan
the network, patch, build a timeline and hit play" — the Rust core has **scan** (`net`), **play**
(`play_show` + transport) and, since `engine::edit`, **patch** (`patch_add`/`patch_del`/
`patch_check`/`profiles`) and **timeline editing** (`show_new`/`show_set`/`show_save`, `track_add`/
`track_del`, `key_set`/`key_del`, `cue_set`/`cue_del`); Python is no longer the only editor.
Theme/Face/Graph still have no serialized structure in `engine::show` for `face_patch`/`graph_patch`
to operate on. Missing, in this order: `spell://face`/`spell://graph` and the streamable HTTP
transport.
