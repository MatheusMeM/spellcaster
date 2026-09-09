# PRD — Spellcaster v1: core em Rust, previz em Godot, interface por Theme + Face + Graph

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
  script/           rhai embutido: tracks fx e o Graph compilado (seção 10)
  cli               spell play|net|serve|tui|mcp|export|agent
spellgui/           Tauri: WebView2 + web/ (index, canvaskit.js, timeline.js, graph.js, face.js, widgets/, themes/)
faces/              superfícies de operação (.face.json) — layout, views, widgets
themes/             looks (theme.json + theme.css) — o que hoje está em spellcaster/gui/web/skins/
spellviz/           projeto Godot 4 + gdext em Rust: palco 3D, fixtures, feixes, LED walls
shows/ profiles/ tests/ bench/
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

Theme (o `skin.json` atual, renomeado; formato mantido):
```json
{"name": "quicksilver", "chrome": {"transport": "round", "visualizer": "scope", "shape": "chrome.svg"},
 "vars": {"--bg": "#1c1e21", "--accent": "#39d0ff", "--bevel": "1px"}}
```

Face (superfície de operação de um show; várias views como as `<VIEW>` do WMP):
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

Graph (comportamento; vive no `.spell`, roda no engine, com ou sem GUI):
```json
{"nodes": [
   {"id": "go",    "type": "in.widget", "widget": "go"},
   {"id": "k",     "type": "in.key", "key": "Space"},
   {"id": "any",   "type": "logic.or"},
   {"id": "next",  "type": "cmd", "cmd": "cue_go"},
   {"id": "flash", "type": "out.widget", "widget": "go", "prop": "glow", "hold_ms": 300}],
 "edges": [["go.press", "any.a"], ["k.down", "any.b"], ["any.out", "next.trigger"], ["next.done", "flash.in"]]}
```

Previz (Godot recebe do engine por IPC, nunca lê o .spell sozinho):
```
frame_dmx(universe, [u8; 512])  → cada fixture patcheada atualiza pan/tilt/cor/beam
frame_video(texture_id)         → LED wall mostra o mapping em tempo real
```

## 6. Fases e aceite

- **R0 — Core e protocolos.** `spellcore play show.spell` reproduz os fixtures de conformidade em `tests/` byte a byte (sACN e Art-Net). `bench/jitter` dentro do alvo. CLI `net`.
- **R1 — Timeline, cues, .spell, script fx, Graph runtime, transporte por OSC.** O Graph (seção 10) compila para Rhai e roda no engine; fontes OSC/MIDI/teclado/timer funcionam sem GUI. Player headless no Pi por SSH. Aceite: graph de 500 nós avaliado em < 0,1 ms por frame.
- **R2 — Mídia:** GStreamer decode com hardware, NDI in/out, RTSP in, Spout out. Preview por memória compartilhada. Aceite: 1080p60 dentro do alvo de CPU.
- **R3 — Pixel mapping** wgpu + fallback rayon. Aceite: 100 000 pixels a 60 Hz < 2 ms.
- **R4 — Laser multi-feed:** Ether Dream, Helios, IDN; safety no engine; 4 feeds simultâneos.
- **R5 — GUI Tauri:** `canvaskit.js` (pan, zoom, seleção, hit-test por bisect, dirty-flag, DPR) compartilhado por timeline e graph; timeline canvas (tracks, keyframes, curvas, snapping em markers, scrub, zoom, loop, record arm); painéis Patch, Outputs, Network, Log; **runtime de Theme e Face**: catálogo de widgets, views com troca por atalho, modo performance (kiosk, fullscreen, touch, nada editável), janela sem moldura com forma por SVG do theme; os 6 themes existentes com troca em runtime. Aceite: montar um show de 3 min do zero na GUI; abrir `faces/operador_medgrupo.face.json` em modo performance e operar o show só por ele.
- **R6 — Previz Godot:** palco, fixtures com feixe volumétrico, LED walls com o mapping, laser projetado. Aceite: 60 fps com 64 fixtures e 2 LED walls.
- **R7 — MCP com rmcp** (stdio e HTTP), `spell mcp install`; tools de Theme/Face/Graph (`face_get`, `face_patch` com JSON Patch, `graph_get`, `graph_patch`, `theme_set`), resources `spell://face`, `spell://graph`, `spell://ui/screenshot`, `spell://ui/events`. Aceite: de uma sessão de IA, escanear rede, patchear, criar timeline, criar uma Face de 4 botões ligada por Graph a cues e dar play sem tocar na GUI.
- **R8 — Empacotamento:** Windows onedir no pendrive, Linux x64, Pi aarch64 estático; CI com `bench/` como gate.
- **R9 — Editores de Face e Graph + painel Agent.** Editor de Face (arrastar widgets na grade, propriedades, views, preview do theme ao vivo), editor de Graph em canvas (nós do catálogo, fios, busca por tipo, colapsar em subgraph, valores ao vivo nos pinos), undo ilimitado por JSON Patch inverso, "proposta" da IA mostrada como diff antes de aplicar, modo ensaio (graph armado só em saídas virtuais/previz). Painel Agent: chat que sobe `claude` (CLI) como subprocesso com o MCP do Spellcaster registrado e faz stream da conversa; sem loop de agente próprio. Aceite: um operador sem treino monta uma Face de show em 10 min; a IA, por prompt no painel, gera Face + Graph para o `medgrupo.spell` e o operador aplica após ver o diff.

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
| Editor de nós vira um TouchDesigner | Catálogo fechado (seção 10): nós só de entrada, lógica, comando e saída; sem sinal, sem vídeo, sem render. Pedido novo vira track, não nó |
| Graph editado pela IA dispara laser ou blackout ao vivo | Toda edição por MCP é proposta com diff; aplicar exige clique; modo ensaio por padrão em show armado |
| Widget custom sem fim | Um só widget `canvas` com script Rhai de desenho; nada de widgets definidos pelo usuário em v1 |

## 9. Relação com o protótipo Python (`spellcaster/`)

O pacote Python `spellcaster/` (F0–F6) fica no repo como implementação de referência e gerador dos fixtures de conformidade. Não recebe funcionalidade nova. `spellcore` tem que reproduzir byte a byte a saída sACN/Art-Net do `shows/medgrupo.spell` gerada pelo Python. As skins de `spellcaster/gui/web/skins/` são reaproveitadas pela GUI Tauri sem alteração de formato.

## 10. Interface: Theme, Face e Graph

A ideia de origem: skins que mudam o comportamento da interface, cada show com a sua, inspiradas nas skins do Windows Media Player, mais um editor de interface por nós usável por gente e por IA, com agente embutido. A leitura correta das skins do WMP é que `skin.xml` declarava **views, botões, sliders e o que cada um fazia**, não só cores. Uma "skin" era três coisas coladas. Aqui elas se separam, porque cada camada muda por um motivo diferente e é editada por uma ferramenta diferente:

| Camada | O que é | Onde vive | Quem edita |
|---|---|---|---|
| **Theme** | Look: tokens de cor, tipografia, bisel, brilho, forma da janela (SVG), estilo do transporte e do visualizador | `themes/<nome>/theme.json` + `theme.css` | Designer; IA por `theme_set` |
| **Face** | Superfície: quais widgets existem, onde, em que views, o que é editável | `faces/<nome>.face.json`, referenciada pelo show (`"face": ...`) ou inline | Operador no editor de Face; IA por `face_patch` |
| **Graph** | Comportamento: o que cada widget, tecla, OSC, MIDI, timer ou marker faz | `"graph"` dentro do `.spell` | Editor de nós; IA por `graph_patch` |

Regras:
- **Um Theme veste qualquer Face; uma Face aceita qualquer Theme.** Precedência do theme: preferência do usuário > `face.theme` > `feiticaria` (padrão).
- **A Face é uma vista do Graph; o Graph vive no engine.** Um botão da Face é um nó `in.widget`. O mesmo Graph roda no Pi sem GUI com `in.osc`, `in.midi`, `in.key`, `in.timer`, `in.marker` como fontes. Não existe segundo runtime na GUI; a GUI só renderiza estado e envia eventos por IPC.
- **Tudo é JSON, diffável, com JSON Patch.** Os editores em canvas são renderizadores/editores desse JSON. Undo = patch inverso. A IA edita pelo mesmo caminho que o humano.
- **Duas Faces por padrão em todo show:** `editor` (a aplicação completa) e `performance` (só o que o operador precisa; kiosk, fullscreen, touch, nada editável, sem menu). Alternar com uma tecla. É isso que "cada projeto tem a sua skin" significa na prática: o designer monta no editor, o operador recebe uma tela de quatro botões.
- **Views por Face** (full, compact, touch, …) como as `<VIEW>` do WMP: mesma Face, arranjos diferentes, atalho para trocar. Janela sem moldura, forma por `clip-path` de um SVG do Theme, transparência via Tauri: é aqui que a interface fica selvagem sem custar performance.

Catálogo de widgets (fechado em v1): `button`, `toggle`, `fader`, `knob`, `xy`, `color`, `label`, `lcd`, `meter`, `universes`, `timecode`, `transport`, `cuelist`, `timeline`, `netscan`, `log`, `visualizer` (barras/scope/script), `canvas` (desenho por script Rhai, o único "custom"). Cada widget é `{id, type, at:[col,row,w,h], props, bind}`; `bind` aponta para um pino do Graph.

Catálogo de nós (fechado em v1): entradas `in.widget | in.key | in.osc | in.midi | in.timer | in.marker | in.state` (tempo, cue atual, universo, fixture); lógica `logic.and|or|not|latch|toggle|debounce|counter|select`, `math.map|curve|expr` (expr = Rhai de uma linha), `time.delay|hold`; comando `cmd` (qualquer entrada do registry, tipado pelo `schemars`); saídas `out.widget` (prop de widget), `out.osc`, `out.param` (fixture.canal via patch), `out.notify`. Nada de sinal, áudio, vídeo, render: isso é track. Um subgraph pode ser colapsado em um nó com pinos; é o único mecanismo de reuso.

Três acréscimos ao catálogo, **proposta desta rodada** (fecham as duas lacunas apontadas por `design/FUNCOES/orquestrador.md` contra o Chataigne; aguardam voto em `design/DECISOES.md`):

- **`state`** — máquina de estados. Config `group` (padrão `"main"`) e `initial`; entradas `enter`/`exit`, saída `active`. Um ativo por grupo: pulso em `enter` liga este e desliga os outros do grupo, `exit` desliga, `locate`/`stop` volta ao `initial`.
- **`module`** — app declarado em `modules/<nome>.json` (o `module.json` do Chataigne). Uma entrada por `parameter` (muda → `Ev::Param{target:"<módulo>/<path>"}`, `norm` mapeia 0..1 antes do clamp em `min`..`max`), uma saída de nível por `value` (alimentada por `input {key:"module:<módulo>/<path>"}`), uma entrada de trigger por `command` (→ `Ev::Cmd{name:"<módulo>/<cmd>"}`). É o que torna app separado interoperável sem o PATCHBAY conhecer o app.
- **`"mute": true` e `"state": "<id>"` em qualquer nó** — o nó não emite: saídas em 0, nenhum evento, `time.delay` pendente cancelado. É o Bypass do TouchDesigner e o container de estado do Chataigne, sem verbo novo (regra 9 de `design/FUNCOES/README.md`).

Agent embutido: o painel Agent não implementa loop de agente. Sobe `claude` (CLI do Claude Code) como subprocesso com o MCP do Spellcaster registrado, faz stream da conversa e mostra cada proposta de `face_patch`/`graph_patch` como diff com botão Aplicar. `// ponytail: subprocesso do claude ; loop próprio via API só se o CLI não estiver instalado na máquina do operador`. A IA vê o que fez pelo resource `spell://ui/screenshot` e vê o operador pelo `spell://ui/events` (últimos 200 eventos de widget).

Segurança de show: comandos de laser, blackout e armar saídas passam pelo mesmo Graph, mas o engine tem a palavra final (safety, seção 8). Show armado em saídas reais entra em **modo ensaio** por padrão para edições de Graph: o graph novo roda contra saídas virtuais (previz) até o operador armar.
