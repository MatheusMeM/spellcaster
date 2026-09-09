# Spellcaster

Spellcaster é um media server de luz e laser portátil da Feitiçaria Industrial: timeline, cues,
sACN, Art-Net, OSC e ILDA (Ether Dream, Helios, IDN), GUI web com skins, MCP embutido, player
standalone e versão Lite para Raspberry Pi operada por CLI via SSH.

O repo tem duas camadas:

- `spellcaster/` — protótipo Python 3.13 (stdlib), fases F0–F6 concluídas: CLI `spell`, GUI web
  com 6 skins, timeline em canvas, MCP (stdio), TUI, empacotamento portátil e Lite.
  Hoje é a implementação de referência e o gerador dos fixtures de conformidade; não recebe
  funcionalidade nova.
- `spellcore/` — o core do produto em Rust (PRD v1.1 em `PRD.md`). Fases R0 (engine, protocolos,
  bench), R1 (cues, `.spell` completo, `fx` em Rhai, Graph runtime, player com transporte OSC),
  R3 (pixel mapping), R4 (laser multi-feed com safety), R7 (MCP em stdio) e R8 (empacotamento)
  concluídas e conformes byte a byte com o Python. `spellgui/web/` tem a base da GUI (R5).

Arquivo de show: `.spell` (JSON, versão 1), o mesmo para os dois lados.

Instalação (pendrive Windows, Lite no Pi, código-fonte): `INSTALL.md`. Licença MIT (`LICENSE`).

## Estado atual

| Fase | O que é | Estado |
|---|---|---|
| F0–F6 (Python) | protocolos, netscan, perfis/patch, timeline, GUI + skins, MCP, portátil/Lite | concluídas, 102 testes |
| R0 (Rust) | `engine`, `protocols`, `cli net/play`, `bench` jitter/throughput | concluída, dentro do alvo |
| R1 (Rust) | cues, `.spell` completo, `fx` Rhai, Graph, player + OSC, CLI headless | concluída, conformidade ao vivo 89/89 |
| R3 (Rust) | `pixelmap`: amostragem nearest/bilinear com rayon, 100 000 px | concluída, 0,316 ms p99 por frame |
| R4 (Rust) | `laser`: optimize/safety, `.ild`, Ether Dream/Helios/IDN, 4 feeds | concluída, 0,83 % de cpu |
| R5 base | `spellgui/web`: `canvaskit.js` (pan, zoom, seleção, hit-test) e timeline em canvas | base pronta; falta Tauri, painéis, Theme/Face |
| R7 (Rust) | `mcp`: rmcp em stdio, uma tool por comando do registry, `spellcore mcp install` | concluída em stdio |
| R8 | onedir Windows, Linux, `spellcore` estático para Pi (musl), CI com bench como gate | concluída |
| R2 mídia, R6 previz Godot, R9 editores | ver `PRD.md` §6 | pendentes (SDKs, Godot, R5) |

Design em `design/` (tokens, princípios, atalhos de Premiere/Resolve) e nas branches `design/*`:
seis rodadas de protótipo do departamento de design, em que o programa é o modelo 3D do próprio
aparelho (rodadas 4–6: o projetor de laser em three.js, traseira como menu, bindings de tecla e MIDI,
Pino como menu, e o laser como módulo `laser/1` do orquestrador). Estado por rodada em `ROADMAP.md` §8.

## Como rodar no Windows

Sem venv. Use o interpretador global.

```
C:\Python313\python.exe -m spellcaster.cli --version
C:\Python313\python.exe -m spellcaster.cli commands
C:\Python313\python.exe -m spellcaster.cli play shows\medgrupo.py --fps 30
C:\Python313\python.exe -m spellcaster.cli play shows\medgrupo.py --loop --universes 1,2
C:\Python313\python.exe -m spellcaster.cli net --timeout 2
C:\Python313\python.exe -m spellcaster.protocols.ilda.generators saida.ild
C:\Python313\python.exe -m spellcaster.protocols.ilda.generators saida.ild 20000 10000
```

- `play` toca um show `.py` por sACN. Opções: `--fps` (padrão 30), `--loop`, `--universes` (lista separada por vírgula, padrão `1`). O show precisa definir `look(t)`; `DUR` é opcional.
- `commands` imprime o schema do registry em JSON.
- `net` aceita `--timeout` (segundos, padrão 2), imprime o relatório e devolve o dict do scan (a chave `report` é esse texto); a GUI e o MCP leem o mesmo comando. Também roda como `-m spellcaster.protocols.netscan [--json]`.
- `generators` grava o laser MED GRUPO em `.ild`. Os dois argumentos opcionais são a meia-largura e a meia-altura da tela em unidades ILDA (padrão 20000 e 10000).

Com o pacote instalado (`pip install -e .`), `spell` substitui `C:\Python313\python.exe -m spellcaster.cli`.

## Testes

```
C:\Python313\python.exe -m unittest discover -s tests -v
```

102 testes Python e 120 Rust (`cargo test --workspace`). Não rodar os dois ao mesmo tempo: ambos
usam sACN em loopback na porta 5568 e um rouba os pacotes do outro. Loopback UDP em 127.0.0.1 faz o papel de mock. Os testes não imprimem caracteres fora de ASCII.

## Contratos fixos

- Universos numerados a partir de 1 (sACN). Art-Net converte para port-address internamente.
- Toda saída de protocolo expõe `send(universe: int, data: bytes)` e `close()`.
- Todo comando do produto passa pelo `spellcaster.core.registry` (`@command`). CLI, OSC-API, GUI e MCP são clientes do registry e não implementam lógica própria.
- O engine não conhece GUI. Nada em `core`, `protocols`, `fixtures`, `timeline` importa de `gui` ou `mcp`.
- Stdlib antes de dependência. `pyproject.toml` declara zero dependências.
- Toda lógica não trivial deixa um teste `unittest` em `tests/`.
- Simplificação deliberada leva comentário `# ponytail: <limite> ; <quando trocar>`.

## spellcore (Rust)

A pasta do repo está no Google Drive, então o `target/` do cargo fica fora dela.

```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\spellcore_target"
cd spellcore
cargo test --workspace
cargo build --release --workspace

# binario em %TEMP%/spellcore_target/release/spellcore.exe
spellcore play ..\shows\medgrupo.spell            # fx em Rhai + laser; Ctrl+C para
spellcore play ..\shows\medgrupo.spell --osc-port 9000   # transporte por /spellcaster/play|pause|stop|locate
spellcore net --timeout 2 [--json]
spellcore commands                                 # registry em JSON (o mesmo que vira MCP)

# bench (gate do PRD)
cargo run --release -p bench --bin jitter
cargo run --release -p bench --bin throughput
cargo run --release -p laser --bin feeds -- --secs 30
cargo bench -p bench
```

Fixtures de conformidade (regerar com `C:\Python313\python.exe tests/conformance/gen.py`):
`tests/conformance/medgrupo_u1.bin`, `sacn_packet.bin`, `artnet_packet.bin` e o show assado
`shows/medgrupo_r0.spell`. Para validar o binário Rust ao vivo contra o Python:

```
C:\Python313\python.exe tests/conformance/capture_sacn.py --secs 3
C:\Python313\python.exe tests/conformance/capture_sacn.py --secs 3 --show shows/medgrupo.spell
```

Árvore, contratos, tabela de conformidade e números do bench em `ARCHITECTURE.md`; dependências
e justificativa em `spellcore/README.md`.

## Deploy no GitHub

`.github/workflows/build.yml` roda em todo push em `main` e em todo PR: testes Python nas três
plataformas, `clippy -D warnings` + testes + bench do `spellcore`, e publica como artefatos o
onedir do Windows (zip), o tarball Lite (x64 e aarch64) e o binário `spellcore` (Windows x64,
Linux x64, Linux aarch64).

Release = tag. O mesmo workflow, ao receber uma tag `v*`, cria a GitHub Release com esses
artefatos anexados e notas geradas do histórico:

```bash
git tag -a v0.1.0 -m "Spellcaster 0.1.0: R0, R1 e R4 do spellcore; protótipo Python F0–F6"
git push origin main --tags
```

Regras: commits e pushes só na conta do dono do repo, mensagem em português, sem crédito a
ferramenta nenhuma; nunca commitar `target/`, `build/`, `dist/`; o bench é o gate.

## Documentos

- `PRD.md`: produto v1 (core Rust, previz Godot, Theme/Face/Graph), tabela de performance, fases R0–R9.
- `ARCHITECTURE.md`: árvore, fluxo de dados, contratos, conformidade e números medidos.
- `ROADMAP.md`: decisões de stack, fases F0–F7 do protótipo, estado das fases R0–R9, design e o que falta.
- `INSTALL.md`: pendrive Windows, Lite no Raspberry Pi, build a partir do código.
- `design/`: `DECISOES.md`, `PRINCIPIOS.md`, `SHORTCUTS.md`, `TEMAS.md`, `tokens/`, `canvas/`; protótipos nas branches `design/*`.
- `CLAUDE.md`: regras do repositório.
- `LICENSE`: MIT.
