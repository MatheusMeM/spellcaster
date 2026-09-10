# Spellcaster

Spellcaster is a portable light and laser media server by Feitiçaria Industrial: timeline, cues,
sACN, Art-Net, OSC and ILDA (Ether Dream, IDN), web GUI with skins, embedded MCP, standalone
player and a Lite version for the Raspberry Pi driven by CLI over SSH.

The repo has two layers:

- `spellcaster/` — Python 3.13 prototype (stdlib), phases F0–F6 done: `spell` CLI, web GUI
  with 6 skins, canvas timeline, MCP (stdio), TUI, portable packaging and Lite.
  Today it is the reference implementation and the conformance-fixture generator; it gets no
  new functionality.
- `spellcore/` — the product core in Rust (PRD v1.1 in `PRD.md`). Phases R0 (engine, protocols,
  bench), R1 (cues, full `.spell`, `fx` in Rhai, Graph runtime, player with OSC transport),
  R3 (pixel mapping), R4 (multi-feed laser with safety), R7 (MCP over stdio) and R8 (packaging)
  are done and byte-for-byte conformant with the Python. `spellgui/web/` is the GUI (R5): the main
  page is the 3D laser projector (`laser3d/`), served by `spellcore serve` or opened by the
  native `spellcaster.exe` window.

Show file: `.spell` (JSON, version 1), the same on both sides.

Installation (Windows USB stick, Lite on the Pi, source): `INSTALL.md`. Operator manual:
`MANUAL.md`. What went into each release: `CHANGELOG.md`. MIT license (`LICENSE`).

## Current state

| Phase | What it is | State |
|---|---|---|
| F0–F6 (Python) | protocols, netscan, profiles/patch, timeline, GUI + skins, MCP, portable/Lite | done, 102 tests |
| R0 (Rust) | `engine`, `protocols`, `cli net/play`, `bench` jitter/throughput | done, within target |
| R1 (Rust) | cues, full `.spell`, `fx` Rhai, Graph, player + OSC, headless CLI | done, live conformance 89/89 |
| R3 (Rust) | `pixelmap`: nearest/bilinear sampling with rayon, 100,000 px | done, 0.316 ms p99 per frame |
| R4 (Rust) | `laser`: optimize/safety, `.ild`, Ether Dream/IDN, 4 feeds | done, 0.83 % of cpu |
| R5 GUI | `spellgui/web`: 3D projector page (`laser3d/`: SolidWorks camera, HUD, LASER/DMX/NET/INTERLOCK/BINDINGS/VIDEO/INFO drawer, Pino, key+MIDI bindings, video menu), timeline with 2D previz, patchbay, theater, Face, MIDI, help; `spellcaster.exe` (native window) | in use; Theme and the Face/Graph editors (R9) still missing |
| R7 (Rust) | `mcp`: rmcp over stdio, one tool per registry command, `spellcore mcp install` | done over stdio |
| R8 | Windows onedir, Linux, static `spellcore` for the Pi (musl), CI with the bench as a gate | done |
| R2 media, R6 Godot previz, R9 editors | see `PRD.md` §6 | pending (SDKs, Godot, R5) |

Design lives in `design/` (tokens, principles, Premiere/Resolve shortcuts) and in the `design/*`
branches: six prototype rounds from the design department, in which the program is the 3D model of
the device itself (rounds 4–6: the laser projector in three.js, the rear panel as the menu, key and
MIDI bindings, Pino as the menu, and the laser as the orchestrator module `laser/1`). State per
round in `ROADMAP.md` §8.

## How to open the program

```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cargo build --release -p cli --manifest-path spellcore\Cargo.toml
& "$env:TEMP\spellcore_target\release\spellcore.exe" serve --port 8000 --dir . --show shows\medgrupo.spell
# http://127.0.0.1:8000/spellgui/web/laser3d/app.html
```

Or `spellcaster.exe shows\medgrupo.spell` (crate `spellcore/gui`), which opens the same page in a
window of the program. Full usage in `MANUAL.md`.

## Python prototype

No venv. Use the global interpreter.

```
C:\Python313\python.exe -m spellcaster.cli --version
C:\Python313\python.exe -m spellcaster.cli commands
C:\Python313\python.exe -m spellcaster.cli play shows\medgrupo.py --fps 30
C:\Python313\python.exe -m spellcaster.cli play shows\medgrupo.py --loop --universes 1,2
C:\Python313\python.exe -m spellcaster.cli net --timeout 2
C:\Python313\python.exe -m spellcaster.protocols.ilda.generators output.ild
C:\Python313\python.exe -m spellcaster.protocols.ilda.generators output.ild 20000 10000
```

- `play` runs a `.py` show over sACN. Options: `--fps` (default 30), `--loop`, `--universes` (comma-separated list, default `1`). The show must define `look(t)`; `DUR` is optional.
- `commands` prints the registry schema as JSON.
- `net` takes `--timeout` (seconds, default 2), prints the report and returns the scan dict (the `report` key is that text); the GUI and the MCP read the same command. It also runs as `-m spellcaster.protocols.netscan [--json]`.
- `generators` writes the MED GRUPO laser to `.ild`. The two optional arguments are the half-width and half-height of the screen in ILDA units (default 20000 and 10000).

With the package installed (`pip install -e .`), `spell` replaces `C:\Python313\python.exe -m spellcaster.cli`.

## Tests

```
C:\Python313\python.exe -m unittest discover -s tests -v
```

102 Python tests, 168 Rust (`cargo test --workspace`) and 119 from the web pages
(`node --test spellgui/web/test/*.test.js`). Do not run Python and Rust at the same time: both
use sACN on loopback port 5568 and one steals the other's packets. UDP loopback on 127.0.0.1 plays the part of the mock. The tests do not print characters outside ASCII.

## Fixed contracts

- Universes numbered from 1 (sACN). Art-Net converts to port-address internally.
- Every protocol output exposes `send(universe: int, data: bytes)` and `close()`.
- Every product command goes through `spellcaster.core.registry` (`@command`). CLI, OSC-API, GUI and MCP are clients of the registry and implement no logic of their own.
- The engine does not know about the GUI. Nothing in `core`, `protocols`, `fixtures`, `timeline` imports from `gui` or `mcp`.
- Stdlib before a dependency. `pyproject.toml` declares zero dependencies.
- Every non-trivial piece of logic leaves a `unittest` test in `tests/`.
- A deliberate simplification carries a `# ponytail: <limit> ; <when to change it>` comment.

## spellcore (Rust)

The repo folder is on Google Drive, so the cargo `target/` stays outside it.

```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cd spellcore
cargo test --workspace
cargo build --release --workspace

# binary at %TEMP%/spellcore_target/release/spellcore.exe
spellcore play ..\shows\medgrupo.spell            # fx in Rhai + laser; Ctrl+C stops
spellcore play ..\shows\medgrupo.spell --osc-port 9000   # transport over /spellcaster/play|pause|stop|locate
spellcore net --timeout 2 [--json]
spellcore commands                                 # registry as JSON (the same one that becomes MCP)

# bench (PRD gate)
cargo run --release -p bench --bin jitter
cargo run --release -p bench --bin throughput
cargo run --release -p laser --bin feeds -- --secs 30
cargo bench -p bench
```

Conformance fixtures (regenerate with `C:\Python313\python.exe tests/conformance/gen.py`):
`tests/conformance/medgrupo_u1.bin`, `sacn_packet.bin`, `artnet_packet.bin` and the baked show
`shows/medgrupo_r0.spell`. To validate the Rust binary live against the Python:

```
C:\Python313\python.exe tests/conformance/capture_sacn.py --secs 3
C:\Python313\python.exe tests/conformance/capture_sacn.py --secs 3 --show shows/medgrupo.spell
```

Tree, contracts, conformance table and bench numbers in `ARCHITECTURE.md`; dependencies and their
justification in `spellcore/README.md`.

## Releases

`.github/workflows/build.yml` runs on every push to `main` and on every PR: Python tests on the
three platforms, `clippy -D warnings` + tests + bench for `spellcore`, and publishes as artifacts
the Windows onedir (zip), the Lite tarball (x64 and aarch64) and the `spellcore` binary (Windows
x64, Linux x64, Linux aarch64).

Release = tag. The same workflow, on receiving a `v*` tag, creates the GitHub Release with those
artifacts attached and notes generated from the history:

```powershell
git tag -a v0.1.0 -m "Spellcaster 0.1.0"
git push origin main --tags
```

The version's `CHANGELOG.md` entry becomes the release notes (`gh release edit vX.Y.Z --notes-file`).

Rules: commits and pushes only from the repo owner's account, message in English, no credit to any
tool; never commit `target/`, `build/`, `dist/`; the bench is the gate.

## Documents

- `PRD.md`: product v1 (Rust core, Godot previz, Theme/Face/Graph), performance table, phases R0–R9.
- `ARCHITECTURE.md`: tree, data flow, contracts, conformance and measured numbers.
- `ROADMAP.md`: stack decisions, prototype phases F0–F7, state of phases R0–R9, design and what is missing.
- `INSTALL.md`: Windows USB stick, Lite on the Raspberry Pi, build from source.
- `MANUAL.md`: usage and capability manual for the operator (screen, camera, keyboard and MIDI, video, laser, DMX, CLI and MCP).
- `CHANGELOG.md`: one entry per release.
- `design/`: `DECISOES.md`, `PRINCIPIOS.md`, `SHORTCUTS.md`, `TEMAS.md`, `tokens/`, `canvas/`; prototypes in the `design/*` branches.
- `CLAUDE.md`: repository rules.
- `LICENSE`: MIT.
