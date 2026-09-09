# PRD — Spellcaster v1: core em Rust, previz em Godot, GUI web com skins

Você é o orquestrador. Leia este documento inteiro antes de despachar qualquer agente.
Modo ponytail sempre: menor código que funciona, stdlib e crates já presentes antes de
dependência nova, nada "para depois", `// ponytail: <limite> ; <quando trocar>` em toda
simplificação deliberada. Commits e pushes só na conta do autor do repo; proibido
`Co-Authored-By` ou qualquer crédito a IA no histórico.

## 1. Objetivo

Media server de show-control portátil, um binário por plataforma, sem instalação.
Timeline com saída simultânea de DMX (sACN, Art-Net), OSC, laser multi-feed (ILDA),
vídeo (decode, NDI in/out, RTSP in, Spout out), pixel mapping em GPU e previz 3D.
Controle por GUI, CLI, OSC e MCP, todos clientes do mesmo registry de comandos.

## 2. Não-objetivos

- Webcam virtual. NDI e Spout cobrem o caso.
- Framework JS no frontend. Vanilla HTML, CSS, JS, canvas.
- Banco de dados. O show é um arquivo `.spell` JSON.
- Editor de vídeo. O engine reproduz, mapeia e roteia; não corta nem colore.
- Plugins binários de terceiros. Efeitos custom são scripts embutidos.

## 3. Kernel de performance (aceite não negociável)

Medido por `bench/` em CI e em máquina real. Nada entra no `main` se regredir.

| Métrica | Alvo desktop x64 | Alvo Pi 4 / Pi 5 |
|---|---|---|
| Jitter entre frames do engine, p99 | < 1 ms | < 3 ms |
| Drift acumulado em 1 h a 60 Hz | 0 frames | 0 frames |
| 16 universos sACN + 16 Art-Net a 60 Hz | < 3 % de um núcleo | < 15 % |
| Pixel mapping, 100 000 pixels a 60 Hz, GPU | < 2 ms por frame | fallback CPU: 20 000 pixels a 30 Hz |
| Decode 1080p60 H.264 + NDI out | < 25 % de um núcleo (hardware decode) | 1080p30 no decoder V4L2 |
| 4 feeds laser a 30 kpps cada | < 1 % de um núcleo | igual |
| Boot até primeiro frame DMX | < 2 s | < 5 s |
| RSS em repouso, sem vídeo | < 60 MB | < 60 MB |
| Tamanho no pendrive | < 150 MB com GStreamer e NDI | binário estático < 20 MB sem mídia |
| GUI: latência clique → estado do engine refletido | < 16 ms | n/a |

Regras do kernel:
- Loop do engine em thread própria, prioridade alta, `timeBeginPeriod(1)` no Windows,
  sleep até 1 ms antes do alvo e spin no resto.
- Zero alocação no caminho quente: buffers por universo e por DAC pré-alocados;
  `Vec::with_capacity` fora do loop; sem `String` por frame; sem `clone` de show.
- Avaliação de keyframes por busca binária; caches por track invalidados só na edição.
- Processo do engine separado do processo da GUI. GUI lenta nunca atrasa DMX.
- Cada saída de rede em sua thread com fila `SPSC` de tamanho fixo; frame velho é descartado, nunca enfileirado.

## 4. Arquitetura

```
spellcore/          binário Rust, sem GUI
  engine/           clock, timeline, cues, registry de comandos
  protocols/        sacn, artnet, osc, netscan
  laser/            frame, optimize, safety, dac::{etherdream, helios, idn}
  media/            gstreamer pipeline, ndi in/out, rtsp in, spout out, texture pool
  pixelmap/         wgpu compute + fallback rayon
  ipc/              socket local, frames binários, memória compartilhada p/ preview
  mcp/              rmcp, tools/resources gerados do registry
  script/           rhai embutido (tracks fx)
  cli               spell play|net|serve|tui|mcp|export
spellgui/           Tauri: WebView2 + web/ (index, timeline.js em canvas, skins/)
spellviz/           projeto Godot 4 + gdext em Rust: palco 3D, fixtures, feixes, LED walls
shows/ profiles/ skins/ tests/ bench/
```

Contratos fixos:
- Universos 1-based. Art-Net converte para port-address internamente.
- Toda saída implementa `trait Output { fn send(&mut self, universe: u16, data: &[u8; 512]); fn close(self); }`.
- Todo comando do produto é uma entrada do `registry` com tipos `serde` + `schemars`.
  CLI, WS, OSC-API e MCP só chamam o registry; nunca têm lógica própria.
- `spellcore` não conhece GUI nem Godot. Só IPC.
- Formato `.spell` versionado; migração sempre para frente.

## 5. Exemplos mínimos

Comando no registry (gera CLI, WS, MCP sem código extra):
```rust
#[command(mcp)]
/// Carrega um show e deixa parado em t=0.
fn load(path: PathBuf) -> Result<ShowInfo>
```

Mensagem IPC / WebSocket:
```
{"id": 7, "cmd": "locate", "args": {"t": 12.5}}
{"id": 7, "result": {"t": 12.5, "state": "paused"}}
```
Broadcast binário de monitor: `topic:u8 | universe:u16 | 512 bytes`.

Track de pixel mapping no .spell:
```json
{"type": "pixelmap", "source": "media:1", "map": "profiles/ledwall_96x54.json",
 "out": {"protocol": "sacn", "first_universe": 10, "order": "grb"}}
```

Track de laser:
```json
{"type": "laser", "dac": "etherdream:192.168.0.50", "clip": "shows/logo.ild",
 "keys": {"scale": [[0, 0.2], [4, 1.0, "easeInOut"]], "rot": [[0, 0], [8, 360]]},
 "safety": {"min_size": 2000, "max_intensity": 200, "zone": [-1, -0.2, 1, 1]}}
```

Skin (mantida do que já existe em web/skins/):
```json
{"name": "quicksilver", "chrome": {"transport": "round", "visualizer": "scope"},
 "vars": {"--bg": "#1c1e21", "--accent": "#39d0ff", "--bevel": "1px"}}
```

Previz (Godot recebe do engine por IPC, nunca lê o .spell sozinho):
```
frame_dmx(universe, [u8; 512])  → cada fixture patcheada atualiza pan/tilt/cor/beam
frame_video(texture_id)         → LED wall mostra o mapping em tempo real
```

## 6. Fases e aceite

- **R0 — Core e protocolos.** `spellcore play show.spell` reproduz os fixtures de conformidade em `tests/` byte a byte (sACN e Art-Net). `bench/jitter` dentro do alvo. CLI `net`.
- **R1 — Timeline, cues, .spell, script fx, transporte por OSC.** Player headless no Pi por SSH.
- **R2 — Mídia:** GStreamer decode com hardware, NDI in/out, RTSP in, Spout out. Preview por memória compartilhada. Aceite: 1080p60 dentro do alvo de CPU.
- **R3 — Pixel mapping** wgpu + fallback rayon. Aceite: 100 000 pixels a 60 Hz < 2 ms.
- **R4 — Laser multi-feed:** Ether Dream, Helios, IDN; safety no engine; 4 feeds simultâneos.
- **R5 — GUI Tauri:** timeline canvas (tracks, keyframes, curvas, snapping em markers, scrub, zoom, loop, record arm), painéis Patch, Outputs, Network, Log; skins existentes funcionando com troca em runtime. Aceite: montar um show de 3 min do zero na GUI.
- **R6 — Previz Godot:** palco, fixtures com feixe volumétrico, LED walls com o mapping, laser projetado. Aceite: 60 fps com 64 fixtures e 2 LED walls.
- **R7 — MCP com rmcp** (stdio e HTTP), `spell mcp install`. Aceite: de uma sessão de IA, escanear rede, patchear, criar timeline e dar play sem tocar na GUI.
- **R8 — Empacotamento:** Windows onedir no pendrive, Linux x64, Pi aarch64 estático; CI com `bench/` como gate.

Cada fase entrega algo usável em obra. Ordem fixa. Fase só fecha com o bench verde.

## 7. Regras para os agentes

- Um agente por diretório de primeiro nível; nenhum agente edita arquivo de outro. O orquestrador faz o wiring entre módulos.
- Todo módulo com lógica não trivial deixa um teste `cargo test` e, se tocar no caminho quente, um `bench/` com Criterion.
- Dependência nova só com justificativa de uma linha no PR e tamanho do binário medido.
- Licenças: GStreamer e ffmpeg LGPL em ligação dinâmica; NDI SDK sob EULA com redistribuição; sem código GPL (x264 fora; encode por openh264 ou hardware).
- Windows: console cp1252, testes só ASCII; binários pesados gerados em `%TEMP%` e movidos.
- Relatório de agente: no máximo 25 linhas, com números do bench, simplificações `ponytail:` e o que ficou de fora.

## 8. Riscos e resposta

| Risco | Resposta |
|---|---|
| Timeline em canvas JS bate no teto com dezenas de milhares de keyframes | Virtualizar por viewport primeiro; WebGL depois; egui só se medido |
| GStreamer + NDI incham o pendrive | Conjunto mínimo de plugins listado em `media/PLUGINS.md`; medir a cada release |
| Godot vira projeto próprio | Escopo travado em previz; nada de edição de show dentro do Godot |
| GPU ausente no Pi | Fallback CPU obrigatório e testado em CI com `WGPU_BACKEND=none` |
| Laser queima ou ofusca | Safety no engine, nunca na GUI; testes de figura mínima e ponto parado antes de R4 fechar |

## 9. Relação com o protótipo Python (`spellcaster/`)

O pacote Python `spellcaster/` (F0–F6) fica no repo como implementação de referência e gerador dos fixtures de conformidade. Não recebe funcionalidade nova. `spellcore` tem que reproduzir byte a byte a saída sACN/Art-Net do `shows/medgrupo.spell` gerada pelo Python. As skins de `spellcaster/gui/web/skins/` são reaproveitadas pela GUI Tauri sem alteração de formato.
