# Instalação

Três jeitos de rodar o Spellcaster: pendrive no Windows, Lite no Raspberry Pi, e a partir do
código-fonte (desenvolvimento). O binário Rust `spellcore` sai pronto do CI para os três alvos.

## 1. Windows · pendrive (operador)

1. Baixe `spellcaster-windows-latest.zip` da última [release](https://github.com/MatheusMeM/spellcaster/releases)
   (ou do artefato do workflow `build` em Actions).
2. Descompacte na raiz do pendrive: fica `Spellcaster\Spellcaster.exe`, `spell.exe`, `shows\`, `profiles\`.
3. Duplo clique em `Spellcaster.exe`. A janela abre com a análise de rede; sem WebView2 ela abre no navegador padrão.
4. Antivírus reclamando do onedir sem assinatura: adicione a pasta `Spellcaster\` como exceção.

Nada é gravado fora do pendrive: `shows/`, `profiles/` e `config.json` ficam ao lado do exe.

```
Spellcaster\spell.exe net --timeout 2
Spellcaster\spell.exe play_show medgrupo.spell
```

Para o core Rust, o mesmo release traz `spellcore-windows-x64.exe`:

```
spellcore-windows-x64.exe play shows\medgrupo.spell --osc-port 9000
```

## 2. Raspberry Pi · Lite (SSH, sem GUI)

Precisa só do `python3` do Raspberry Pi OS (3.11+). Sem pip, sem venv, sem dependência.

```bash
tar xzf spellcaster-lite-<versão>.tar.gz
cd spellcaster-lite-<versão>
sudo ./install.sh              # copia para /opt/spellcaster, cria `spell`, liga o serviço systemd
spell net
spell tui
spell play_show medgrupo.spell
```

O serviço `spellcaster` sobe `spell serve`; a GUI fica em `http://<hostname>.local:8000`
para quem tiver navegador na rede. `install.sh <destino>` muda a pasta. Tarball `--no-gui` é o
mesmo pacote sem os assets da GUI.

Binário Rust para o Pi: `spellcore-linux-aarch64` do release (glibc; o build estático musl é
pendência do R8).

## 3. A partir do código-fonte (desenvolvimento)

### Python (protótipo de referência, congelado)

Windows: Python 3.13 global, sem venv.

```
C:\Python313\python.exe -m spellcaster.cli --version
C:\Python313\python.exe -m unittest discover -s tests -v
```

`pip install -e .` instala o comando `spell`. Extra `gui` traz `pywebview` (janela nativa).

### Rust (spellcore)

Toolchain via rustup (`rustc` ≥ 1.75). A pasta do repo mora no Google Drive: o `target/`
NUNCA fica dentro dela.

```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cd spellcore
cargo test --workspace
cargo build --release --workspace
# binário em %TEMP%\spellcore_target\release\spellcore.exe
```

Detalhes, bench e conformidade: `spellcore/README.md` e `ARCHITECTURE.md`.

### Empacotar

```powershell
pwsh -File packaging\build_win.ps1            # onedir em %TEMP%\spellcaster_build\dist\Spellcaster
sh packaging/build_lite.sh [--no-gui] [saída] # tarball Lite (default /tmp)
```

### MCP (sessão de IA)

```
spell mcp                      # stdio, para Claude Desktop / Claude Code
spell mcp_install --target desktop       # grava a entrada em claude_desktop_config.json (--target code = .mcp.json); pede confirmação
```
