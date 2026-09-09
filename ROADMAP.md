# Spellcaster — roadmap (protótipo Python F0–F7; estado das fases Rust na seção 7)

Show-control portátil da Feitiçaria Industrial: timeline + sACN / Art-Net / OSC / ILDA (laser), GUI web com skins, MCP embutido, player standalone, versão Lite para Raspberry Pi (CLI pura por SSH). Repositório: github.com/MatheusMeM/spellcaster. Pacote Python `spellcaster`, CLI `spell`.

Origem: o gerador sACN de `show_medgrupo.py` (plenária MED GRUPO RJ, 09/2026). Tudo que já foi calibrado lá (pacote E1.31, perfis BSW/Sharpy/BX-402, geometria de grupo com histerese e rampa, cues por corte de vídeo) migra para cá como código de produção.

## 1. Decisões de stack

| Decisão | Escolha | Por quê |
|---|---|---|
| Linguagem | Python 3.13, stdlib primeiro | Reaproveita o que existe; roda nativo no Raspberry Pi; o time já opera em Python. |
| GUI | Web (HTML/CSS/JS vanilla) servida pelo próprio processo; janela via `pywebview` (WebView2, já presente no Windows 10/11) | Uma GUI serve o desktop, o Pi Lite (acessado pelo navegador) e o tablet na mesa. Skins = CSS. Zero framework JS. |
| Timeline | Editor em `<canvas>` próprio | Nenhuma lib pronta atende timeline de show (tracks heterogêneas, curvas, cues, markers de vídeo). |
| Empacotamento Windows | PyInstaller **onedir** numa pasta do pendrive + `Spellcaster.exe` | Onefile extrai em `%TEMP%` a cada abertura (lento, dispara antivírus). Onedir abre em < 1 s e não grava nada fora do pendrive. |
| Lite (Pi) | Mesmo pacote sem `pywebview`; `pip install` ou tarball; serviço `systemd` | Mesmo código, mesma GUI (pelo navegador), sem tela no Pi. |
| Arquivo de show | JSON legível (`.spell`) | Git-friendly; o MCP e o humano leem o mesmo arquivo. |
| MCP | Gerado automaticamente do registro de comandos do engine | AI-nativo por construção: cada comando existe uma vez e aparece em CLI, GUI, OSC-API e MCP. |
| Laser (ILDA) | Sem placa ILDA no PC: o engine gera frames ILDA e envia para DACs de rede/USB: Ether Dream (UDP/TCP aberto), IDN-Stream (padrão ILDA Digital Network, Ethernet), Helios (USB, via `libusb`/`pyusb`). Arquivos `.ild` como clipes. | São os protocolos abertos; cobrem os DACs que a Feitiçaria usa e o que um Pi consegue alimentar. Saída ILDA analógica só via DAC. |
| Modo CLI | `spell` funciona 100 % por terminal: `play`, `net`, `patch`, `tui` (monitor `curses`) | No Pi por SSH não há GUI; tudo que a GUI faz tem verbo de CLI porque ambos chamam o registry. |
| Dependências | `pywebview`, `mcp` (SDK oficial), opcionais `pyusb` (Helios), `python-rtmidi`, `numpy` | Só o que uma linha de stdlib não faz. |

## 2. Arquitetura

```
spellcaster/
  core/
    clock.py        relógio único (perf_counter), tick 30/60 Hz, transporte play/pause/stop/locate
    registry.py     @command: nome, args tipados, doc → CLI + OSC-API + MCP + GUI
    universe.py     buffers DMX (N universos), merge HTP/LTP por fonte
    engine.py       loop: timeline → parâmetros → fixtures → universos → saídas
  protocols/
    sacn.py         E1.31 out/in, discovery (239.255.250.214), prioridade, sync
    artnet.py       ArtDmx out/in, ArtPoll/ArtPollReply, ArtSync
    osc.py          OSC 1.0 out/in (UDP), bundles, timetag, pattern matching
    netscan.py      interfaces, ArtPoll, sACN discovery, mDNS _osc._udp, Ether Dream/IDN discovery, sugestão de IP/subrede
    ilda/
      frame.py      ponto ILDA (x, y, r, g, b, blank), frame, otimização (blanking, dwell, interpolação, limite de scan)
      ild.py        leitura/escrita .ild (formatos 0/1/2/4/5) — migra de ilda_gen.py
      etherdream.py Ether Dream: broadcast de descoberta, TCP stream de pontos, buffer
      idn.py        IDN-Stream (ILDA Digital Network) sobre UDP
      helios.py     Helios USB (opcional, pyusb)
  fixtures/
    profile.py      perfil por nome de canal (pan16, tilt16, color, shutter…), faixas e rodas
    fixture.py      Fixture(perfil, universo, addr).set(dimmer=…, color="amarelo", pan_deg=…)
    group.py        Group + geometria (reto, sinal lateral, amplitude), histerese de pan, rampa
    library/        BSW, Sharpy, BX-402, BX-940, WLED bar, laser Butrym… (JSON)
  timeline/
    model.py        Sequence → Track → Clip/Keyframe; curvas (linear, ease, hold, bezier); markers
    tracks.py       tipos: fixture-param, dmx-raw, osc, artnet-raw, laser (clipe .ild ou gerador), cue-trigger, media-control, python-fx
    cues.py         cue list (GO / follow / wait), snapshots, fades
    render.py       avalia a timeline em t → dicionário de parâmetros
  player/
    player.py       modo standalone/headless: carrega .spell, roda, aceita OSC/HTTP para transporte
  gui/
    server.py       HTTP + WebSocket (stdlib `http.server` + handshake WS manual ou `websockets`)
    web/            index.html, timeline.js, patch.js, monitor.js, skins/
    window.py       pywebview (só desktop)
  mcp/
    server.py       MCP stdio + HTTP streamable, tools/resources/prompts gerados do registry
  cli.py            spell play|stop|net|patch|calib|laser|serve|tui|mcp|export — cada verbo é um comando do registry
  tui.py            monitor curses: transporte, universos, fontes, laser, log — para SSH no Pi
  show.py           load/save .spell, migrações de versão
```

Princípio: **o engine não sabe que existe GUI**. GUI, CLI, OSC-API e MCP são clientes do mesmo registro de comandos via WebSocket/stdio. Isso é o que torna o Lite e o MCP baratos.

## 3. Fases

Cada fase termina com algo usável em trabalho real. Ordem fixa; prazos são estimativa para uma pessoa em dedicação parcial.

### F0 — Fundação e protocolos (semana 1–2)
- Migrar `sacn` de `show_medgrupo.py` para `protocols/sacn.py` com N universos e entrada.
- `artnet.py` out/in + ArtPoll; `osc.py` out/in.
- `clock.py`, `registry.py`, `engine.py` com uma `look(t)` Python como track (compatibilidade com o show do MED GRUPO).
- CLI: `spell play shows/medgrupo.py` reproduz o show atual sem GUI.
- Aceite: Capture recebe sACN e Art-Net idênticos; `ffmpeg`-style loopback test compara pacotes byte a byte com fixtures gravadas.

### F1 — Análise de rede (semana 2)
- `spell net`: interfaces, IP/máscara, nós Art-Net (ArtPollReply), fontes sACN (discovery + escuta), dispositivos OSC (mDNS), latência.
- Sugestão automática: "Art-Net exige 2.x.x.x ou 10.x.x.x; sua placa está em 192.168…" com comando `netsh` pronto (não executa sozinho).
- Aceite: relatório em texto e JSON na GUI e no MCP.

### F2 — Perfis e patch (semana 3)
- Formato de perfil JSON por nome de canal, com faixas (zoom 80–255 = 13–36°), rodas nomeadas, 16 bit.
- Importar os perfis já calibrados; importador GDTF fica no backlog.
- Patch: universo/endereço, detecção de sobreposição (o bug dos 17 ch em espaçamento de 16 vira erro na hora).
- `Group` com geometria calibrável e ferramenta `spell calib hold|sweep` (o `bsw_hold`/`bsw_multi` de hoje).
- Aceite: show MED GRUPO reescrito em fixtures nomeadas, sem índice de canal no código.

### F3 — Timeline engine + arquivo de show + Player (semana 4–5)
- Modelo Sequence/Track/Keyframe, curvas, markers importados de vídeo (`ffmpeg` scene detect) e de áudio (beats).
- Tracks: parâmetro de fixture, DMX cru, OSC, Art-Net cru, cue-trigger, media-control (Capture media player, VLC via OSC), python-fx (função `f(t)` embutida para efeitos gerativos).
- Cue list com GO/follow/wait e fades entre snapshots.
- Laser: track `laser` com clipes `.ild` e geradores (formas, texto, scanner de figuras como o do MED GRUPO), transformações (posição, escala, rotação, cor), safety (limite de tamanho mínimo, zona proibida, intensidade máxima) e saída para Ether Dream / IDN / Helios em thread própria a 20–30 kpps.
- `spell play show.spell` headless = **Player standalone** pronto; transporte por OSC (`/spellcaster/play`), HTTP e teclado.
- Aceite: o show MED GRUPO expresso 100 % em `.spell`, saída idêntica à versão Python; `medgrupo_laser.ild` tocando num Ether Dream (ou no emulador de rede) sincronizado à timeline.

### F4 — GUI (semana 6–9)
- Layout inspirado em Chataigne (painéis dockáveis: Patch, Timeline, Outputs, Network, Inspector, Log) e Adobe (timeline com tracks, keyframes, curvas, snapping em markers, scrub, zoom, in/out, loop, régua de tempo/timecode).
- Timeline em canvas: seleção múltipla, arrastar, copiar/colar, easing por keyframe, solo/mute por track, gravação ao vivo de parâmetros (record arm).
- Monitor de saída: universos como grade, VU de canais, fontes de entrada.
- Skins estilo Windows Media Player: pasta `skins/<nome>/` com `skin.json` + `skin.css` + imagens; cromo customizável (bordas, botões de transporte, visualizador); tema claro/escuro; skin padrão Feitiçaria (design system).
- Aceite: montar do zero, na GUI, um show de 3 minutos com 8 movings, 2 universos, OSC para um player de vídeo, e gravar/reproduzir.

### F5 — MCP e AI-nativo (semana 9–10)
- Gerador: percorre `registry` e emite tools MCP com schema JSON tipado a partir das assinaturas; resources: show atual, patch, rede, log; prompts: "monte um show a partir deste vídeo", "calibre este grupo".
- Transportes: stdio (Claude Desktop/Code) e HTTP streamable (remoto, Pi Lite).
- Comando `spell mcp install` grava a entrada em `claude_desktop_config.json` / `.mcp.json` (pede confirmação).
- Eventos do engine (frame, cue, erro) como notificações MCP.
- Aceite: de uma sessão Claude, sem tocar na GUI: escanear rede, patchear, criar timeline com cues nos cortes de um vídeo, dar play e ler o monitor de saída.

### F6 — Empacotamento portátil e Lite (semana 10–11)
- Windows: PyInstaller onedir → `Spellcaster/` no pendrive com `Spellcaster.exe`, `shows/`, `skins/`, `profiles/`, `config.json`. Tudo relativo ao executável; nada em `%APPDATA%`. Primeiro boot mostra a análise de rede.
- Assinatura de código (certificado) para reduzir alarme de antivírus; sem ela, documentar exceção.
- Lite: `pip install spellcaster[lite]` ou tarball `spellcaster-lite-aarch64.tar.gz`; `spell serve --headless` como serviço `systemd`; GUI pelo navegador em `http://spellcaster.local:8000` **ou só CLI por SSH**: `spell play show.spell`, `spell tui` (monitor curses), `spell net`, `spell laser test`. Sem GUI instalada, sem X, sem navegador. Imagem de cartão SD opcional via `pi-gen`.
- CI (GitHub Actions): build Windows x64, Linux x64, Linux aarch64; testes de protocolo em loopback.
- Aceite: pendrive em PC limpo → duplo clique → show rodando em < 10 s. Pi Zero 2 W entregando 8 universos sACN + Art-Net + OSC a 40 Hz e um laser Ether Dream a 20 kpps, operado só por SSH.

### F7 — Backlog (depois de usar em obra)
Timecode LTC/MTC in, MIDI in/out, GDTF/MVR import, LaserCube/outros DACs proprietários, editor gráfico de frames laser, entrada Art-Net/sACN com merge para "passthrough + overlay", NDI/vídeo nativo no player, macOS build, sincronização multi-máquina (master/slave por clock UDP), scripting Lua/Python ao vivo, undo ilimitado com histórico visual.

## 4. O que não fazer agora
- Framework JS (React/Electron): dobra o tamanho e o tempo de boot do pendrive; o canvas próprio dá o controle que uma timeline exige.
- Banco de dados: o show é um JSON.
- Plugins binários: tudo Python; efeitos custom são tracks `python-fx`.
- Vídeo dentro do engine: no F3 o player controla players externos (Capture, VLC, Resolume) por DMX/OSC. Vídeo nativo é F7.

## 5. Riscos
| Risco | Mitigação |
|---|---|
| Jitter do loop Python a 60 Hz com muitos universos | Loop em thread própria com `perf_counter` e compensação de deriva; envio em lote por universo; medido em F0 antes de escolher 30 ou 60 Hz padrão. |
| WebView2 ausente em PC muito antigo | Fallback: abre no navegador padrão (`spell serve --browser`). |
| Antivírus no pendrive | Onedir + assinatura; instruções de exceção no `LEIA-ME`. |
| Timeline em canvas virar um projeto em si | Escopo do F4 travado nos verbos listados; o resto vai para F7. |
| Laser: ponto parado ou figura pequena demais queima/ofusca | Safety no engine, não na GUI: limite de kpps, tamanho mínimo de figura, zona de exclusão por DAC, shutter por software; testes em loopback antes de F3 fechar. |
| MCP gerar tools demais e confundir o modelo | Registry marca `mcp=True` só nos comandos de alto nível; os de baixo nível ficam em um tool genérico `run_command`. |

## 6. Primeiro passo
F0 começa extraindo `protocols/sacn.py` e `core/clock.py` de `show_medgrupo.py`, com o show do MED GRUPO como teste de regressão: mesma saída, byte a byte.

## 7. Fases Rust (PRD v1.1) — estado em 09/09/2026

O plano acima (F0–F7) foi o do protótipo Python e está concluído até F6. O produto segue o
`PRD.md`: core em Rust, previz em Godot, GUI Tauri com Theme/Face/Graph.

| Fase | Aceite (PRD §6) | Estado | Bloqueio |
|---|---|---|---|
| R0 core e protocolos | fixtures byte a byte, `bench/jitter` no alvo, CLI `net` | concluída | — |
| R1 timeline, cues, .spell, fx, Graph, OSC | graph de 500 nós < 0,1 ms/frame; player headless | concluída (8,4 µs) | — |
| R2 mídia (GStreamer, NDI, RTSP, Spout) | 1080p60 no alvo de CPU | pendente | SDKs não instalados (GStreamer, NDI) |
| R3 pixel mapping (rayon; wgpu depois) | 100 000 px a 60 Hz < 2 ms | concluída (0,105 ms p50 / 0,316 ms p99 por frame; bilinear 0,196 / 0,493) | crate autônomo: ligar a fonte de frame ao player espera a R2 |
| R4 laser multi-feed | Ether Dream, Helios, IDN; safety no engine; 4 feeds | concluída (0,83 % cpu) | — |
| R5 GUI Tauri | show de 3 min do zero; Face em modo performance | pendente | voto das rodadas 5 e 6 do design |
| R6 previz Godot | 60 fps, 64 fixtures, 2 LED walls | pendente | Godot não instalado |
| R7 MCP com rmcp | sessão de IA monta e toca um show sem GUI | pendente | nenhum: registry pronto |
| R8 empacotamento | onedir, Linux, Pi estático; CI com bench como gate | parcial | Pi estático (musl) pendente; só CI |
| R9 editores de Face/Graph + painel Agent | operador monta uma Face em 10 min | pendente | depende de R5 |

## 8. Design — rodadas e branches (09/09/2026)

O departamento de design trabalha em branches próprias e publica cada rodada como protótipo
HTML (three.js) num artifact; o voto do dono decide o que entra. Regra: função antes de UI, e
o programa é o modelo 3D fotorrealista do aparelho que ele controla.

| Rodada | O quê | Branch | Estado |
|---|---|---|---|
| 1 | moodboard estático | `design/0.1.2` | reprovada |
| 2 | vidro em GLSL, cubo raymarched, splash, skins `.wmz`, tema GELO | `design/0.1.2` | votada |
| 3 | ILDA player em tema LASER, Aprendiz como menu, `TEMAS.md` mapa função→tema | `design/0.1.2` | votada; virada para o aparelho |
| 4 | o programa é o projetor 3D (PBR), traseira = menu, tampa = preferências, Pino no lugar do Aprendiz | `design/0.1.2` | votada |
| 5 | traseira real, mesa óptica e feixe em GLSL, splash na parede, câmera SolidWorks, bindings tecla + MIDI, Pino 3D, design system do laser (`design/laser/SISTEMA.md`) | `design/0.1.2` | publicada, aguardando voto |
| 6 | o laser como módulo `laser/1` do orquestrador: endereços, `module.json`, `graph.json`, painel ORQUESTRADOR, teste em `tests/test_laser_graph.py` | `design/0.1.3` | publicada, aguardando voto |
| FUNCOES | funções por referência (Blender, TouchDesigner, Resolume, MadMapper, Capture, Chataigne): `ilda-player`, `ndi-ilda`, `orquestrador`, `cenas-cues-dmx`, `cenario-interativo`, `aprendiz-menu` | `design/funcoes-referencia` | em uso pelas rodadas |

Pendências de design: merge de `design/funcoes-referencia` e `design/0.1.3` numa base única;
mover `design/laser/INTEGRACAO.md` para `design/FUNCOES/integracao-laser.md`; rodada 7 = FÓSFORO
(conversor NDI/Spout → ILDA na porta NET) e PATCHBAY (UI do orquestrador), após o voto.

## 9. O que falta, e o que roda em paralelo agora

Sem bloqueio externo, cada linha é um agente independente (código novo em crate próprio ou em
CI, sem tocar no que já está conforme):

| Frente | Entrega | Aceite | Depende de |
|---|---|---|---|
| R7 MCP | crate `mcp` com rmcp (stdio + HTTP), tools do registry, `spellcore mcp install` | sessão de IA escaneia, patcheia, cria timeline e dá play | nada |
| R8 Pi estático | job de CI `aarch64-unknown-linux-musl`, artefato `spellcore-linux-aarch64-static` | binário roda num Pi limpo sem glibc da versão | nada |
| R5 base | `canvaskit.js` (pan, zoom, seleção, hit-test por bisect, dirty-flag, DPR) + timeline canvas portada do Python | testes headless no Chrome; timeline abre `medgrupo.spell` | nada (design só define o cromo) |
| Design | merge das branches de design; rodada 7 | voto do dono | voto das rodadas 5 e 6 |

Bloqueadas até instalar SDK (decisão do dono, não de agente): R2 (GStreamer + NDI SDK),
R6 (Godot 4). R9 espera R5.

Já feito: F0–F6, R0, R1, R3, R4, CI com release por tag, docs (README, INSTALL, LICENSE, ARCHITECTURE, PRD).
