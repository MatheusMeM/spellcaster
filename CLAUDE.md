# Spellcaster — repository rules

Portable show control: timeline + sACN / Art-Net / OSC / ILDA, web GUI with skins, embedded MCP, standalone player, Lite for Raspberry Pi (CLI over SSH). Plan in `ROADMAP.md`; architecture in `ARCHITECTURE.md`.

## Working mode: ponytail (always)

- The smallest code that works. Stdlib before a dependency; an already-installed dependency before a new one; one line before fifty.
- No abstraction with a single implementation, no factory, no config for a value that never changes, no scaffolding "for later".
- A deliberate simplification carries a `# ponytail: <limit> ; <when to change it>` comment.
- Every non-trivial piece of logic leaves ONE `unittest` test in `tests/`. No pytest, no fixtures, no heavy mocks: UDP loopback on 127.0.0.1 is the mock.
- Bug fix = root cause at the point every caller goes through.

## Fixed contracts

- Package `spellcaster`, CLI `spell` (`spellcaster.cli:main`). Show file `.spell` (JSON).
- Universes numbered from 1 (sACN). Art-Net converts to port-address internally.
- Every protocol output exposes `send(universe: int, data: bytes)` and `close()`.
- Every product command goes through `spellcaster.core.registry` (`@command`). CLI, OSC-API, GUI and MCP are clients of the registry; they never implement logic of their own.
- The engine does not know about the GUI. Nothing in `spellcaster/core`, `protocols`, `fixtures`, `timeline` imports from `gui` or `mcp`.
- Python 3.13, no venv. Run with `C:\Python313\python.exe` on Windows, `python3` on the Pi.

## Language

- The whole project is in English: UI, docs, comments, commit messages, Pino lines, registry and MCP descriptions.
- File names stay as they are (`design/DECISOES.md`, `design/FUNCOES/…`, `MANUAL.md`), even the Portuguese ones: renaming would break links and `help.js`.

## Environment (Windows, folder on Google Drive)

- Python only in `.py` files; never `python -c` with nested quotes.
- UTF-8 files without BOM. The console is cp1252: tests do not print characters outside ASCII.
- Do not write heavy binaries (exe, mp4, SD image) straight into this folder; generate them in `%TEMP%` and move them.
- Tests: `C:\Python313\python.exe -m unittest discover -s tests -v`.

## Git

- Branch `main`, remote `origin` = github.com/MatheusMeM/spellcaster. Commit messages in English, imperative, one subject line.
- Do not commit `build/`, `dist/`, `*.log`, `__pycache__/`.

- Commits and pushes only from Matheus's account. `Co-Authored-By`, "Generated with Claude" or any credit to Claude in the git history is forbidden.

## Spellcaster v1 (Rust)

Specification in `PRD.md`. Rust core in `spellcore/`, Tauri GUI in `spellgui/`, Godot previz in `spellviz/`, benches in `bench/`. The Python package `spellcaster/` is the reference implementation and the conformance-fixture generator; it gets no new functionality. Toolchain: `cargo` (rustup, user install, no UAC).
