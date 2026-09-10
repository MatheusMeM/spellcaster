# Installation

Three ways to run Spellcaster: USB stick on Windows, Lite on the Raspberry Pi, and from source
(development). The Rust binary `spellcore` comes ready from CI for all three targets.

## 1. Windows · USB stick (operator)

1. Download `spellcaster-windows-latest.zip` from the latest [release](https://github.com/MatheusMeM/spellcaster/releases)
   (or from the `build` workflow artifact in Actions).
2. Unzip it at the root of the USB stick: you get `Spellcaster\Spellcaster.exe`, `spell.exe`, `shows\`, `profiles\`.
3. Double-click `Spellcaster.exe`. The window opens with the network scan; without WebView2 it opens in the default browser.
4. Antivirus complaining about the unsigned onedir: add the `Spellcaster\` folder as an exception.

Nothing is written outside the USB stick: `shows/`, `profiles/` and `config.json` sit next to the exe.

```
Spellcaster\spell.exe net --timeout 2
Spellcaster\spell.exe play_show medgrupo.spell
```

For the Rust core, the same release ships `spellcore-windows-x64.exe`:

```
spellcore-windows-x64.exe play shows\medgrupo.spell --osc-port 9000
```

## 2. Raspberry Pi · Lite (SSH, no GUI)

All it needs is the `python3` from Raspberry Pi OS (3.11+). No pip, no venv, no dependency.

```bash
tar xzf spellcaster-lite-<version>.tar.gz
cd spellcaster-lite-<version>
sudo ./install.sh              # copies to /opt/spellcaster, creates `spell`, enables the systemd service
spell net
spell tui
spell play_show medgrupo.spell
```

The `spellcaster` service starts `spell serve`; the GUI sits at `http://<hostname>.local:8000`
for anyone with a browser on the network. `install.sh <destination>` changes the folder.

Rust binary for the Pi, two of them in the release:

- `spellcore-linux-aarch64-static` — musl, no `NEEDED` in the ELF: runs on any aarch64 Raspberry Pi
  OS (or Alpine, or a `scratch` container) without depending on the glibc version. Use this one by
  default.
- `spellcore-linux-aarch64` — glibc, in case you need something from the system at runtime.

```bash
chmod +x spellcore-linux-aarch64-static
./spellcore-linux-aarch64-static play shows/medgrupo.spell --osc-port 9000
```

Not the static binary's fault: Helios (USB) is already a stub in `laser` in every build — it is not
a musl limitation.

## 3. From source (development)

### Python (reference prototype, frozen)

Windows: global Python 3.13, no venv.

```
C:\Python313\python.exe -m spellcaster.cli --version
C:\Python313\python.exe -m unittest discover -s tests -v
```

`pip install -e .` installs the `spell` command. The `gui` extra brings `pywebview` (native window).

### Rust (spellcore)

Toolchain via rustup (`rustc` ≥ 1.88, required by `rmcp`). The repo folder lives on Google Drive: the
`target/` NEVER stays inside it.

```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cd spellcore
cargo test --workspace
cargo build --release --workspace
# binary at %TEMP%\spellcore_target\release\spellcore.exe
```

To open the program from source: `spellcore serve --port 8000 --dir . --show shows/medgrupo.spell`
at the repo root and `http://127.0.0.1:8000/spellgui/web/laser3d/app.html` in the browser
(usage in `MANUAL.md`).

Details, bench and conformance: `spellcore/README.md` and `ARCHITECTURE.md`.

### Packaging

```powershell
pwsh -File packaging\build_win.ps1            # onedir at %TEMP%\spellcaster_build\dist\Spellcaster
sh packaging/build_lite.sh [output]          # Lite tarball (default /tmp)
```

### MCP (AI session)

Rust (`spellcore`, this is what the product uses):

```
spellcore mcp                              # MCP server over stdio, for Claude Desktop / Claude Code
spellcore mcp install --target desktop     # writes the entry to %APPDATA%\Claude\claude_desktop_config.json
spellcore mcp install --target code        # writes .mcp.json in the current directory (project)
spellcore mcp install --target code --yes  # without asking
```

`install` shows the entry it is about to write, makes a `.bak` backup and only writes after a
confirmation in the console. With no console (pipe), it aborts. The tools are the commands from `spellcore commands`;
the resources are `spell://show` and `spell://commands`.

Python (prototype, stdio only):

```
spell mcp                      # stdio, for Claude Desktop / Claude Code
spell mcp_install --target desktop       # writes the entry to claude_desktop_config.json (--target code = .mcp.json); asks for confirmation
```
