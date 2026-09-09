# Decisões de design

Uma linha por decisão. Data, decisão, motivo. Agente nenhum re-decide o que está aqui; para mudar, adiciona linha nova que revoga a antiga.

- 2026-09-09 — Design system próprio ("Spellcaster DS", prefixo `--sc-`), com o da Feitiçaria como starter kit. Motivo: o produto precisa de cores funcionais (armado, ao vivo, ensaio, erro) e de voz mono que a identidade da firma não tem.
- 2026-09-09 — Palco `#000000` e Floral White `#F7F5EB` herdados da Feitiçaria; nunca `#FFFFFF`. Motivo: sala escura, continuidade com a firma.
- 2026-09-09 — Accent do produto é âmbar `#FFB000` (seleção, foco, ativo). Motivo: âmbar de VFD/LED de console, lê à distância, não é a cor da firma.
- 2026-09-09 — Verde-lima `#A8E05E` da Feitiçaria só em GO e "ok"; Wisteria `#B7AED9` só em modo ensaio; vermelho `#FF2D1F` em armado/ao vivo/erro. Motivo: cor = estado (princípio 2).
- 2026-09-09 — Tipografia: JetBrains Mono (voz, valores, timecode) e Barlow Condensed (rótulos, overlines em caps). Ambas OFL, embutidas no binário. Motivo: mono é o vocabulário CLI/MCP na tela; condensada cabe em widget 1×1.
- 2026-09-09 — Grade 8 px, unidade de widget 48 px, gap 8 px, raio 0. Motivo: densidade de console, alvo de toque ≥ 44 px.
- 2026-09-09 — Sem sombras. Estado "ao vivo" usa contorno 1 px + glow 12 px na cor do estado. Motivo: flat, glow só com significado.
- 2026-09-09 — Ícones próprios, grade 16 px, traço 1,5 px, cantos retos. Até existirem, placeholder é um quadrado tracejado com o nome do ícone.
- 2026-09-09 — Faces de referência: `performance` (kiosk, 4 widgets) e `editor` (áreas divisíveis como no Blender; graph ao centro como no TouchDesigner). Motivo: prova a separação Theme/Face antes da R5.
- 2026-09-09 — Cores das famílias de nó do Graph: entrada âmbar, lógica cinza-claro, comando Floral White, saída wisteria. Motivo: mesma ideia do TouchDesigner (família = cor fixa), sem copiar a paleta dele.

## 2026-09-09 · rodada 1 do moodboard (reprovada)

- Votos: A console NÃO · B bancada NÃO · C cápsula TALVEZ · D grimório NÃO · E cartaz NÃO · F fita NÃO.
- Brief novo do Matheus: como as skins do WMP mesmo que kitsch; nada de software quadradão; janela transparente feito vidro; estética keygen/cracktro com jingle no splash; UI responsiva, boa de usar e engraçada; a experiência segue o arco 5E.
- Consequência: `PRINCIPIOS.md` e `tokens/spellcaster.css` continuam valendo para o **dentro** da tela (leitura, estado, alvos). A **carcaça** é objeto: silhueta, vidro, gel. Decisões da rodada 2 entram abaixo quando aprovadas.

## 2026-09-09 · rodada 2 (protótipo, aguardando voto)

- Entregue como protótipo funcional, não como prancha: https://claude.ai/code/artifact/106e46a9-8070-4edd-9351-83ac4e7a5e2e. Fontes em `design/rodada2/`.
- Carcaça = vidro com silhueta cortada + gel (cores de gel reais Lee/Rosco) + geada opcional. No produto: Tauri `transparent`, sem decoração, acrílico via `window-vibrancy`.
- Entry = splash cracktro com jingle chiptune sintetizado (zero mídia), pulável, "nunca mais" honesto.
- Skins do WMP entram de verdade: leitor de `.wmz` (zip + XML + BMP) com silhueta por clippingColor, mapa de clique por cor, hover/down repintados; Play/Next → GO. Sliders e JScript ficam fora.
- Fita compacta (TAB) sobre o Resolume; NFO como about; copy com humor (toast), animação só onde há estado.
- Pendente: votos da rodada 2 (gel padrão, splash, silhueta, kitsch, jingle, .wmz, fita).
- 2026-09-09 — Interface e atalhos seguem Adobe Premiere e DaVinci Resolve (mapa em `design/SHORTCUTS.md`). Motivo: operador que edita vídeo opera sem aprender nada novo; gramática Ctrl/Shift/Alt fixa.
- 2026-09-09 — Carcaça reprovada de novo ("software quadradão"): o vidro passa a ser um fragment shader WebGL (SDF corpo + cúpula do GO, refração com dispersão R/G/B, fresnel, specular no mouse, varredura, sombra e cáustica do gel). Motivo: pedido explícito de vidro reconhecível e "espetaculoso"; GLSL liberado pelo Matheus. Splash cracktro aprovado ("cyber matrix ficou ótima").

- 2026-09-09 — Vidro 2D-SDF reprovado também (refs: Skins Factory wmpdesign, Shadertoy 4ll3R7 / 4s2GDV / XlscDH / dl3BRS / 4dSBDt). Nova direção aprovada pelo Matheus ("imagina se o menu fica nesse cubo flutuando e é isso a janela"): a janela é um cubo de vidro raymarched (760×470×220, cúpula do GO), flutuando e girando na frente do desktop; refração de duas faces com dispersão por canal, absorção do gel pela espessura, fogo volumétrico (gyroid fBm) dentro do cubo no GO, dois passes do shader (corpo abaixo do HTML, face da frente acima), UI em CSS 3D com a mesma matriz. Fluido MIP (tsKXR3) fica para o gel numa rodada futura: precisa de multi-buffer.
- 2026-09-09 — Voto da rodada 2 (Matheus): fita SIM · forma OBJETO · gel SEM GEL · jingle CHIP · kitsch MAIS · splash PRIMEIRA · wmz NÃO. Ressalvas: "não amei, mas um milhão de vezes melhor"; W (.wmz) quebrado com o cubo; fogo sem sentido num cubo azul ("use água ou gelo aqui, fogo em outra skin"); falta visão central; "crie temas e destrinche temas antes de personas e usos, aí builde as skins". Consequências: `.wmz` sai do protótipo; o cubo vira tema GELO (rachaduras que acendem no GO); gel padrão = sem gel; `design/TEMAS.md` nasce com a regra "cada tema é um material" e quatro temas (GELO, BRASA, TANQUE, CROMO) para aprovar antes de qualquer skin.
- 2026-09-09 — Matheus joga fora a UI da rodada 2 ("usa o conhecimento dela para as próximas"). Regras novas: tudo com a mesma cara do splash ao info; **função antes de UI** ("fica muito difícil fazer UI para software sem função"); funções pedidas: ILDA player, NDI→ILDA, orquestrador tipo Chataigne, cenas e cues DMX com menu de cenário interativo, companion tipo Clippy em todas as skins como menu principal; "be more wild". Resposta: `design/TEMAS.md` reescrito como mapa função ↔ tema (LASER, FÓSFORO, PATCHBAY, TEATRO DE PAPEL + Aprendiz); rodada 3 = ILDA player em tema LASER, `design/rodada3/ilda.html`.

## 09/09/2026 · voto da rodada 3 e virada para o aparelho

Voto (moodboard/round3, 01:53): LASER **ajustar** · Aprendiz **outro personagem** · névoa **mais** · próxima **NDI → ILDA (FÓSFORO)**. Notas: info em camada legível (o transparente não funcionou); programa não quadrado, borda interessante; mascote com botões nele, mais Clippy que pixel art.

Diretriz nova do Matheus, na sequência: **o programa é o modelo 3D do próprio laser 10 W.** Traseira = menu (portas, botões, VFD). Preferências = a câmera sobe, os parafusos saem, a tampa abre, e cada componente é o seu ajuste (diodo = limite e curva; galvos = kpps; placa = buffer e velocidade; VFD = endereço DMX e conexões). Precisa parecer real, fotorrealista, não cartoon.

Consequências:
- **Pino** (`design/pino.js`): cabo DMX com plugue XLR-5 na cabeça; os cinco pinos são botões (1 ILDA, 2 NDI→ILDA, 3 orquestrador, 4 cenas e cues, 5 info), a trava é "some". Vetor SVG, olhos que seguem o mouse, balão Win98. Substitui o Aprendiz em todas as skins. `design/build.py` inlina o arquivo na rodada para publicar.
- **Rodada 4 = `design/rodada4/projetor.html`**: three.js r128 (jsdelivr; o cdnjs não tem o build UMD), PBR com PMREM de estúdio procedural, ACES, sombras PCF, alumínio escovado (normal + roughness procedurais), chapas com chanfro (ExtrudeGeometry), dicroicos em MeshPhysicalMaterial. A parede é o canvas 2D da rodada 3 como textura aditiva; os feixes saem da abertura do modelo. Painel de parâmetros opaco e chanfrado (lição do voto). Sala com clip-path chanfrado e degrau (borda não quadrada).
- **NDI → ILDA (FÓSFORO)** não morreu: vira o que aparece na porta ETHER (um monitor de rack ligado ali). Fica para a rodada 5 se o voto confirmar.
- Rodada 3 (`ilda.html`) fica como registro; o ajuste do LASER foi absorvido pela 4.

## 09/09/2026 · rodada 5 (protótipo publicado, aguardando voto)

- Protótipo: https://claude.ai/code/artifact/8a913f8b-8ea7-4621-b57a-88d7738dbafd · design system: https://claude.ai/code/artifact/2bada8a5-b991-43b7-a086-1b72b0a42232. Fontes em `design/laser/` (módulos `ilda.js`, `cam.js`, `bind.js`, `mat.js`, `body.js`, `optics.js`, `beam.js`, `pino3d.js`, `app.js`, página `app.html`, `tokens.css`, `SISTEMA.md`, `sistema.html`). `design/build.py` inlina qualquer `<script src>` e `<link>` local para publicar como um arquivo só.
- Pedidos do Matheus desta rodada ficam literais em `design/laser/PEDIDOS.md` (todos marcados como feitos; o voto decide o que ajusta).
- Traseira inspirada no Kvant Clubmax, sem copiar: powerCON TRUE1, rocker, chave, LED EMISSION, um interlock só, sem fusível, ILDA IN/OUT DB25, DMX IN/OUT XLR-5, NET RJ45, USB, OLED com encoder e BACK, ventoinha axial de 60 mm modelada de verdade (aro, cubo, 7 pás, grade de arame), placa de série.
- Dentro, "nada voando": mesa óptica de alumínio (bloco silver único, furação M4), três módulos em bases, dois dicroicos e o espelho de dobra em suportes cinemáticos, obturador de solenoide, bloco de galvos em cantoneira (X vertical, Y a 90°, espelhos que seguem o galvo), drivers de diodo e de galvo e a placa DAC nas paredes, escuros; fonte 48 V; nove cabos roteados de um ponto a outro.
- Feixe em GLSL: cilindros instanciados (núcleo + halo), alfa por dot(N,V), poeira por ruído 1D, aditivo, bloom. Fora: abertura → pontos acesos da parede. Dentro: caminho óptico módulo → dicroico → dobra → obturador → galvo X → galvo Y → abertura, acende com a chave e o obturador corta.
- Splash = a câmera mira o output: começa num ponto estático, o foco vai para a parede ignorando o laser, o galvo contorna SPELLCASTER LASER (marching squares sobre o texto em Michroma), cada letra fechada é revelada, tudo brilha, o laser apaga, a sala escurece e a câmera pousa na traseira. Sem ILDA na parede durante a splash.
- Fluxo: a vista SHOW fica trancada até armar a chave (settings primeiro, show depois).
- Câmera no padrão SolidWorks (`cam.js`): MMB gira em torno do ponto clicado, Ctrl+MMB pan, Shift+MMB zoom, roda no cursor (sentido SolidWorks com toggle), setas 15°/Shift 90°/Ctrl pan, F enquadra, Ctrl+1..7 vistas.
- Bindings (`bind.js`): toda ação tem id; tecla ou MIDI (note/CC com canal) com LEARN, feedback de saída para o controlador, persistido em localStorage. Espelha o MadMapper/Resolume.
- Pino 3D substitui o Pino 2D (`design/pino.js` fica como referência): cabo DMX plugado no DMX OUT, ponta macho em pé no case, cinco pinos como botões na cena, balão Win98 ancorado na projeção da cabeça.
- Design system só desta ferramenta: `tokens.css` (cores LASER/âmbar/vermelho/OLED, Michroma + Share Tech Mono, escala, chanfros, glows) + `SISTEMA.md` + `sistema.html`.
- Pendente: voto da rodada 5 (traseira, dentro, splash, câmera, bindings, Pino 3D) em `moodboard/round5`.

## 09/09/2026 · rodada 6 (integração com o orquestrador)

- Protótipo republicado no mesmo artifact da rodada 5 (voto `moodboard/round5` intacto). Fontes: `design/FUNCOES/integracao-laser.md`, `design/laser/module.json`, `graph.json`, `bind.js`, `app.js`; teste `tests/test_laser_graph.py`. Branch `design/0.1.3`.
- 2026-09-09 — O projetor é o módulo `laser/1` do graph; cada binding de tecla/MIDI é uma rota `entrada → filtro → endereço`. Endereços: `laser/1/arm`, `power`, `play`, `shutter`, `kpps` (valor, e disparo com `{step}`), `clip {file}`, `net/ndi|spout|artnet|sacn` (comandos); `limit/r|g|b`, `curve/r|g|b`, `geo/scale`, `dmx/addr`, `queue` (parâmetros); `interlock`, `temp`, `fps`, `points`, `emitting` (valores somente leitura). Nome = o da CLI, agrupado por `/` como o rótulo de `ilda-player.md §1`. Motivo: regra 2 de `FUNCOES/README.md`, um endereço é a identidade de tudo.
- 2026-09-09 — Fica fora do registry: câmera (gesto do viewer; só `cam/view` e `cam/fog` sobrevivem, no bloco `view` do `.spell`), splash, Pino, OLED/encoder/BACK (as páginas já são endereços), ILDA OUT/DMX OUT/USB, mecânica (mesa, dicroicos, dobra, PCBs, fonte), velocidade do driver (calibração do previz). Motivo: nada disso muda o show.
- 2026-09-09 — Porta do graph escreve-se `<uid>/<porta>` (`midi/cc:1:7`, `laser/1/kpps`), não `uid.port` como em `orquestrador.md §5`: o nome da porta já leva `/` e assim a porta é o próprio endereço do registry. Motivo: um nome só.
- 2026-09-09 — CC contínuo passa por `filter.lag` (80 ms); nota e tecla são `trigger` direto; `trigger → value` exige argumento; `number → trigger` não tem filtro no PRD §10 (fica como rota sem filtro, o teste não cobre). `dependency` do manifesto leva `target` (o §5 omite).
- Ponto aberto: **nó de estado** não existe no PRD §10 (`orquestrador.md §1`, "Estado"). `states[]` sai vazio do `Bind.graph()`; "no segundo ato estas rotas valem e aquelas param" hoje é condição em cada rota. Decisão pendente: adotar a semântica do Chataigne inteira (container de rotas com `ativo`, `ao carregar`, transições) e acrescentar `state` ao catálogo de nós do PRD.
- Ponto aberto: filtro de limiar (`number → trigger`) para CC em ação de disparo; o `bind.js` já trata CC > 63 como disparo, o graph não sabe dizer isso.
