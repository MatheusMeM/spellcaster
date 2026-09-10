# Changelog

Uma entrada por release. Datas em ISO. A tag `v*` no `main` dispara o workflow `build`, que
anexa à GitHub Release o onedir do Windows, o tarball Lite e os binários `spellcore`.

## v0.1.0 — 2026-09-10

Primeira release numerada. Reúne o core Rust (R0, R1, R3, R4, R7, R8), o barramento `serve`,
as páginas de `spellgui/web` e a página principal do programa: o projetor laser em 3D.

### O programa é o projetor (`spellgui/web/laser3d/`)

- Modelo PBR do projetor 10 W: chassi com dobradiça, knob estriado, XLR-3 DMX, LED de ARMED,
  ventoinha em textura, etiquetas medidas, firmware gravado na placa, marca vetorial da
  Feitiçaria Industrial (`brand/`), sem USB.
- Interior: mesa óptica de alumínio com furação M4 a 12,5 mm, bloco de galvos X/Y no padrão
  6215H, placas com componentes soldados e espaçadores, fonte, DAC, cabos com rota real,
  abraçadeiras, par trançado ILDA e fio de terra. Nada atravessa o feixe.
- Câmera no padrão SolidWorks, uma lei por vista: SHOW livre, TRASEIRA fixa, DENTRO restrita;
  mola criticamente amortecida; `Z`/`Shift+Z`, setas, `F`; manual da fonte em
  `design/FUNCOES/camera-solidworks.md`. Knob com gesto do TouchDesigner.
- HUD só com título e fatos ao vivo (ENGINE, DAC/feed, entradas sACN/Art-Net, MIDI) com LEDs;
  gaveta à direita (`Tab`) com as abas LASER, DMX, NET, INTERLOCK, BINDINGS, VÍDEO e INFO no
  lugar do painel flutuante.
- Pino preso à câmera no canto esquerdo, cabo DMX com corda de Verlet (`rope.js`), balão à
  direita, dispensar. Fala só no clique.
- Aba VÍDEO no formato de menu de vídeo de jogo: predefinições BAIXO/MÉDIO/ALTO/ULTRA,
  27 opções ligadas de verdade no render (escala, FOV, limite de fps, FXAA/MSAA 4×, sombras e
  filtro, anisotropia, reflexos, bloom, tone mapping, exposição, resolução da parede, rastro,
  halo, feixes, poeira, névoa, corda do Pino, movimento), bloco DESEMPENHO lido do `renderer.info`,
  persistência em `localStorage`. Tabela única em `video.js`.
- Parede do laser em `WebGLRenderTarget` com `LineSegments` e esmaecimento por tempo, no lugar do
  canvas 2D com `shadowBlur`: 2,9 → 45–60 fps com um `.ild` denso. Parse do `.ild` com
  `Uint8Array` e vetor pré-dimensionado (142 → 6 ms).
- Bindings de tecla e MIDI com learn, reset e manifesto (`bind.js`); mapa de entrada e
  interlock na gaveta.
- `bench.html`: grade de vistas do modelo sem `app.js`, para conferir cada peça.
- `<!doctype html>` e `<meta charset="utf-8">` em todas as páginas; `charset.test.js` vigia.

### Engine, CLI e protocolos (`spellcore/`)

- `netscan`: beacon do Ether Dream escutado no IP de cada placa (o `bind` em `0.0.0.0:7654`
  falha no Windows com o Ether Dream Sitter aberto), loopback incluído; sem beacon, o DAC é
  achado por status TCP 7765 nos vizinhos da ARP. `laser_dacs` devolve `via: beacon|tcp`.
- Entrada DMX (sACN e Art-Net), gravação de keyframes (`rec_arm`/`rec_state`) e importação de
  `.ild` como track.
- Entrada MIDI e mapa tecla → comando no `.spell` (`midi_*`), com página de mapeamento.
- Loop como estado do transporte no intervalo In-Out (`loop_set`).
- `serve`: HTTP estático com `charset=utf-8` e mime de fonte e mídia, WebSocket JSON-RPC,
  monitor DMX binário, MCP streamable em `/mcp`.
- `spellcaster.exe` (crate `gui`, `tao` + `wry`/WebView2): janela nativa com o barramento em
  processo, abrindo na página 3D do laser.
- Registry: um nome só para caminho do show e nome do track, descrição em todo argumento,
  `spellcore commands <nome>`. Tabela de comandos do README regerada do registry (59).

### Páginas web (`spellgui/web/`)

- `nav.js`: barra das páginas TIMELINE, PATCHBAY, TEATRO, FACE, LASER e AJUDA
  (`Shift+1`…`Shift+6`), indicador ENGINE/OFFLINE, nome do show editável.
- `help.html`: atalhos de `design/SHORTCUTS.md` e o registry vivo, um formulário por comando.
- Timeline: atalhos que faltavam, desfazer local, In-Out que não colapsa, roda do mouse rola as
  tracks, previz 2D na faixa de baixo (`viewer.js`: DMX, quadro ILDA em `t`, planta do patch).
- Patchbay: criar nó pela busca, editar `cfg` no Inspector, roda pura rola com `ymax`.
- three.js r128 e as fontes Michroma e Share Tech Mono vendorizados: roda sem rede.

### Design e documentação

- `design/FUNCOES/`: timeline como DAW (Ableton, Resolve), interface DAW, digests das fontes,
  câmera SolidWorks.
- Auditoria ponytail das frentes: referências corrigidas, asserções tautológicas removidas,
  licença e custo das crates de janela no README.
- `MANUAL.md`: manual de uso e capacidades do operador. `CHANGELOG.md`: este arquivo.
