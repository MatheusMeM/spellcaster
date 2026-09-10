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

- Entregue como protótipo funcional, não como prancha: https://claude.ai/code/artifact/106e46a9-8070-4edd-9351-83ac4e7a5e2e. Fontes eram `design/rodada2/` (apagado da árvore em 09/09/2026; histórico no git, commit `6fe8b23`).
- Carcaça = vidro com silhueta cortada + gel (cores de gel reais Lee/Rosco) + geada opcional. No produto: Tauri `transparent`, sem decoração, acrílico via `window-vibrancy`.
- Entry = splash cracktro com jingle chiptune sintetizado (zero mídia), pulável, "nunca mais" honesto.
- Skins do WMP entram de verdade: leitor de `.wmz` (zip + XML + BMP) com silhueta por clippingColor, mapa de clique por cor, hover/down repintados; Play/Next → GO. Sliders e JScript ficam fora.
- Fita compacta (TAB) sobre o Resolume; NFO como about; copy com humor (toast), animação só onde há estado.
- Pendente: votos da rodada 2 (gel padrão, splash, silhueta, kitsch, jingle, .wmz, fita).
- 2026-09-09 — Interface e atalhos seguem Adobe Premiere e DaVinci Resolve (mapa em `design/SHORTCUTS.md`). Motivo: operador que edita vídeo opera sem aprender nada novo; gramática Ctrl/Shift/Alt fixa.
- 2026-09-09 — Carcaça reprovada de novo ("software quadradão"): o vidro passa a ser um fragment shader WebGL (SDF corpo + cúpula do GO, refração com dispersão R/G/B, fresnel, specular no mouse, varredura, sombra e cáustica do gel). Motivo: pedido explícito de vidro reconhecível e "espetaculoso"; GLSL liberado pelo Matheus. Splash cracktro aprovado ("cyber matrix ficou ótima").

- 2026-09-09 — Vidro 2D-SDF reprovado também (refs: Skins Factory wmpdesign, Shadertoy 4ll3R7 / 4s2GDV / XlscDH / dl3BRS / 4dSBDt). Nova direção aprovada pelo Matheus ("imagina se o menu fica nesse cubo flutuando e é isso a janela"): a janela é um cubo de vidro raymarched (760×470×220, cúpula do GO), flutuando e girando na frente do desktop; refração de duas faces com dispersão por canal, absorção do gel pela espessura, fogo volumétrico (gyroid fBm) dentro do cubo no GO, dois passes do shader (corpo abaixo do HTML, face da frente acima), UI em CSS 3D com a mesma matriz. Fluido MIP (tsKXR3) fica para o gel numa rodada futura: precisa de multi-buffer.
- 2026-09-09 — Voto da rodada 2 (Matheus): fita SIM · forma OBJETO · gel SEM GEL · jingle CHIP · kitsch MAIS · splash PRIMEIRA · wmz NÃO. Ressalvas: "não amei, mas um milhão de vezes melhor"; W (.wmz) quebrado com o cubo; fogo sem sentido num cubo azul ("use água ou gelo aqui, fogo em outra skin"); falta visão central; "crie temas e destrinche temas antes de personas e usos, aí builde as skins". Consequências: `.wmz` sai do protótipo; o cubo vira tema GELO (rachaduras que acendem no GO); gel padrão = sem gel; `design/TEMAS.md` nasce com a regra "cada tema é um material" e quatro temas (GELO, BRASA, TANQUE, CROMO) para aprovar antes de qualquer skin.
- 2026-09-09 — Matheus joga fora a UI da rodada 2 ("usa o conhecimento dela para as próximas"). Regras novas: tudo com a mesma cara do splash ao info; **função antes de UI** ("fica muito difícil fazer UI para software sem função"); funções pedidas: ILDA player, NDI→ILDA, orquestrador tipo Chataigne, cenas e cues DMX com menu de cenário interativo, companion tipo Clippy em todas as skins como menu principal; "be more wild". Resposta: `design/TEMAS.md` reescrito como mapa função ↔ tema (LASER, FÓSFORO, PATCHBAY, TEATRO DE PAPEL + Aprendiz); rodada 3 = ILDA player em tema LASER, `design/rodada3/ilda.html` (apagado da árvore; commit `6cc9bfa`).

## 09/09/2026 · voto da rodada 3 e virada para o aparelho

Voto (moodboard/round3, 01:53): LASER **ajustar** · Aprendiz **outro personagem** · névoa **mais** · próxima **NDI → ILDA (FÓSFORO)**. Notas: info em camada legível (o transparente não funcionou); programa não quadrado, borda interessante; mascote com botões nele, mais Clippy que pixel art.

Diretriz nova do Matheus, na sequência: **o programa é o modelo 3D do próprio laser 10 W.** Traseira = menu (portas, botões, VFD). Preferências = a câmera sobe, os parafusos saem, a tampa abre, e cada componente é o seu ajuste (diodo = limite e curva; galvos = kpps; placa = buffer e velocidade; VFD = endereço DMX e conexões). Precisa parecer real, fotorrealista, não cartoon.

Consequências:
- **Pino** (`design/pino.js`, apagado da árvore; commit `7dacf8f`): cabo DMX com plugue XLR-5 na cabeça; os cinco pinos são botões (1 ILDA, 2 NDI→ILDA, 3 orquestrador, 4 cenas e cues, 5 info), a trava é "some". Vetor SVG, olhos que seguem o mouse, balão Win98. Substitui o Aprendiz em todas as skins. `design/build.py` inlina o arquivo na rodada para publicar.
- **Rodada 4 = `design/rodada4/projetor.html`** (apagado da árvore; commit `7dacf8f`): three.js r128 (jsdelivr; o cdnjs não tem o build UMD), PBR com PMREM de estúdio procedural, ACES, sombras PCF, alumínio escovado (normal + roughness procedurais), chapas com chanfro (ExtrudeGeometry), dicroicos em MeshPhysicalMaterial. A parede é o canvas 2D da rodada 3 como textura aditiva; os feixes saem da abertura do modelo. Painel de parâmetros opaco e chanfrado (lição do voto). Sala com clip-path chanfrado e degrau (borda não quadrada).
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
- Pino 3D substitui o Pino 2D (`design/pino.js`, hoje só no git): cabo DMX plugado no DMX OUT, ponta macho em pé no case, cinco pinos como botões na cena, balão Win98 ancorado na projeção da cabeça.
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

## 09/09/2026 · limpeza da árvore de design (0.1.4)

- 2026-09-09 — `design/funcoes-referencia` entra em `design/0.1.4` por merge: `design/FUNCOES/` (funções, regras transversais e auditorias em `fontes/`) passa a conviver com `design/laser/`. Motivo: função antes de UI, as duas metades na mesma branch.
- 2026-09-09 — `design/laser/INTEGRACAO.md` vira `design/FUNCOES/integracao-laser.md` e entra na tabela de `FUNCOES/README.md`. Motivo: era contrato de função, morava em `laser/` só porque a branch não tinha `FUNCOES/`.
- 2026-09-09 — Rodadas 2, 3 e 4 saem da árvore (`design/rodada2/`, `design/rodada3/`, `design/rodada4/`, `design/pino.js`); o histórico fica no git (commits `6fe8b23`, `6cc9bfa`, `7dacf8f`). Motivo: a rodada 2 foi reprovada no voto e as rodadas 3 e 4 foram absorvidas por `design/laser/`; código morto na árvore custa leitura e não guarda nada que o git já não guarde.

## 09/09/2026 · Nó estado e nó módulo no graph — aguarda voto

- **Nó `state`.** Config `group` (padrão `"main"`) e `initial`; entradas `enter` e `exit`, saída `active`. Um ativo por grupo: pulso em `enter` liga este estado e desliga os outros do mesmo grupo, `exit` desliga, `locate`/`stop` volta ao `initial`. É a State Machine do Chataigne com a parte que cabe em `PRINCIPIOS.md`: sem transição com fade, sem sub-máquina, sem vários ativos no mesmo grupo.
- **Nó `module`.** Config `module`, que nomeia `modules/<nome>.json` ao lado do show — o `module.json` do Chataigne (`parameters`, `values`, `commands`), regra 14 de `FUNCOES/README.md`. Cada `parameter` é uma entrada (muda → `Ev::Param{target:"<módulo>/<path>"}`, `norm` mapeia 0..1 antes do clamp em `min`..`max`), cada `value` é uma saída de nível alimentada por `input {key:"module:<módulo>/<path>"}`, cada `command` é uma entrada de trigger. O PATCHBAY monta o nó lendo o arquivo, sem conhecer o app.
- **Chaves `"mute": true` e `"state": "<id>"` em qualquer nó.** As duas significam a mesma coisa no runtime: o nó não emite — saídas em 0, nenhum evento, `time.delay` pendente cancelado. Ao voltar, o valor guardado (toggle, latch, counter) continua lá, as bordas estão zeradas e a próxima saída de nível é reemitida (o "reemitir ao ativar" do Chataigne). É o Bypass do TouchDesigner sem verbo novo (regra 9).
- **Um verbo por conceito, e `lock` e `solo` ficam de fora do runtime.** `lock` é só edição (congela o valor no editor) e `solo` é o mute dos outros, calculado pelo PATCHBAY: nenhum dos dois precisa de código no engine.
- **O que o voto decide:** se o estado é exclusivo por grupo (aqui) ou vários ativos ao mesmo tempo (Chataigne), e se `norm` mapeia o sinal 0..1 para a faixa (aqui) ou é só a faixa do slider na GUI.
- Motivo: são as duas lacunas que `design/FUNCOES/orquestrador.md` aponta contra o Chataigne; sem elas, "no segundo ato este conjunto de rotas passa a valer" vira condição copiada em cada rota, e app separado não vira nó.

## 09/09/2026 · Nó módulo → comando: convenção de nome e paths sem implementação — aguarda voto

- **Convenção.** O nó `module` emite `Ev::Param{target:"<mod>/<path>"}` e `Ev::Cmd{name:"<mod>/<cmd>"}`; o sink da CLI roteia para `<mod>_param {feed:"<mod>", path, value}` e `<mod>_<cmd> {feed:"<mod>", ...}`. Ou seja: **`feed` = nome do módulo**. Com dois lasers abertos, os dois nós `module` teriam que se chamar `laser` e o roteamento colide — a saída é instância nomeada (`laser@palco`), e o voto decide se ela entra agora ou quando aparecer a segunda mesa.
- **Paths declarados sem implementação.** `modules/laser.json` declara `dev/type`, `dev/host`, `dev/pps`, `ilda/fps`, `curve/r|g|b`, `safe/zone`, `safe/armed` e `test/pattern`; `laser_param` não aceita nenhum deles (`dev/*` e `ilda/*` são argumento de `laser_open`/`laser_play`, o resto espera LUT de cor e `optimize` paramétrico). O manifesto é a declaração do app, não do comando: o voto decide se ele só declara o que já roda, ou se declara o alvo e o comando cresce até ele.
- **`shutter` está dos dois lados.** É `command` no `modules/laser.json` e `path` no `laser_param`. Uma das duas some.
- Motivo: a convenção está no código (seis linhas no sink da CLI, com comentário `ponytail:`) e funciona para um laser; registrar aqui evita que ela vire contrato por omissão.

## 09/09/2026 · A barra de navegação na Face (modo kiosk) — aguarda voto

- **O que existe.** `spellgui/web/nav.js` põe a mesma barra (abas TIMELINE/PATCHBAY/TEATRO/FACE/LASER, `Shift+1`..`Shift+5`, ENGINE/`rev` e o nome do show editável) no topo das cinco páginas, `face.html` inclusive. Sem isso a Face é a única página sem saída: quem abre nela não tem como voltar.
- **O conflito.** O PRD §10 e `spellgui/web/README.md` descrevem a Face como kiosk: sem chrome, sem nada editável, alvo de toque. Uma barra com o nome do show num campo de texto é chrome, e é editável.
- **O que o voto decide:** a barra some da Face (e o operador volta pelo `Shift+1`, que continua valendo), ou some só no modo `performance` (e fica no `editor`), ou fica como está.
- Motivo: é decisão de produto, não de implementação — as três saídas custam a mesma linha de código.

## 09/09/2026 · x/y de fixture no patch — aguarda voto

- O previz da timeline (`spellgui/web/viewer.js`) desenha a planta do patch, e o patch não tem
  onde a fixture está: `patch_add` grava `{name, profile, universe, address}` e nada mais. Enquanto
  isso, a planta é uma grade em ordem de endereço — lê o rig, não a sala.
- **O que o voto decide:** se a entrada do patch ganha `x`/`y` (planta em metros, com o palco na
  origem) e se eles entram no `.spell` ou num arquivo de planta ao lado dele.
- Não implementado e não removido até o voto: a grade por endereço fica, e vira posição real no dia
  em que o patch souber dizer onde a fixture está.

## 09/09/2026 · Entrada DMX e gravação: formato de `inputs` e forma do keyframe gravado — aguarda voto

- **`inputs` é lista de `{type, universe}`, irmã de `outputs`.** `[{"type":"sacn","universe":1},{"type":"artnet","universe":2}]`: um universo por entrada, sem `interfaces` e sem prioridade. A alternativa era espelhar `outputs` (`{"type":"sacn","universes":[1,2],"interfaces":[...]}`), que casa com o que já existe no arquivo mas repete configuração de rede que a entrada não usa (multicast entra em todos os grupos declarados; Art-Net chega por broadcast). O voto decide qual das duas formas vira contrato do `.spell` v1 — trocar depois quebra show gravado.
- **O que a gravação escreve.** Um keyframe `linear` por MUDANÇA de valor, sem thinning: um fader andando a 60 fps deixa 60 keyframes por segundo no track. A alternativa é gravar reduzido (Douglas-Peucker no fim do take) ou em degrau (`hold`, que reproduz a mesa byte a byte mas não interpola em fps diferente). O voto decide o padrão; o código está com `linear` e um `ponytail:` apontando a redução.
- **A largura do track (quantos canais gravam) vem do keyframe que já existe**, não de um campo. Track vazio grava um canal só, no `address`. A alternativa é um campo `channels` no track `dmx` — mais um campo no `.spell` para o caso "armei um track novo de 4 canais".
- **Fora de escopo, e por quê:** merge HTP entrada→saída (passthrough) e gravação de laser/OSC ficam para depois; vídeo (NDI/GStreamer) está bloqueado pelos SDKs não instalados (ROADMAP R2), não por decisão de design.

## 09/09/2026 · MIDI de entrada (frente `midi`) — o que ficou fora, aguarda voto

- **Chave do evento é `"<status>/<data1>"`** (`144/60` = note on canal 1 nota 60, `176/1` = CC 1), e não o `note:1:60` / `cc:1:7` do protótipo `design/laser/bind.js`: é a chave que o nó `in.midi` do graph já usa (PRD §10) e o canal já vem no status. Quando o `bind.js` virar produto os dois formatos precisam virar um só; o voto decide qual.
- **Saída MIDI e feedback de superfície (LED, fader motorizado) ficam fora** — aguarda voto. É o que faz o controlador mostrar o estado do show; o `feedback()` do `bind.js` já manda note/CC de volta, então a função existe no protótipo e não no engine.
- **MTC / MIDI clock e MIDI Show Control ficam fora** — aguarda voto. São o caminho para o Spellcaster receber GO (ou timecode) de uma mesa de som ou de vídeo; é função de produto, não detalhe de implementação.
- Uma porta MIDI por processo: teclado e surface ao mesmo tempo colidem, porque a chave não diz de qual porta o evento veio. Sai da frente como limite anotado (`// ponytail:` em `spellcore/engine/src/midi.rs`); o voto decide se a porta entra na chave ou se cada superfície vira um módulo.

## 09/09/2026 · Timeline DAW: marcador com nome, lock de track, cor e altura — aguarda voto

Origem: `design/FUNCOES/timeline-daw.md` §3 (leitura de Ableton Live 12 e do manual do DaVinci Resolve instalado nesta máquina).

- **Marcador vira objeto.** Hoje há dois formatos no mesmo campo: a GUI grava `markers: [12.5, 40.0]` (números, `spellgui/web/timeline.js:198,864`) e o engine já tem teste com `markers: [{"t": 1.0, "name": "um"}]` (`spellcore/engine/tests/patch.rs:44`); nenhum dos dois falha, porque `show_patch` aceita qualquer JSON. Recomendação: **objeto `{t, name, note}`**, com número aceito na leitura e convertido na carga (migração local, `FUNCOES/README.md §12`). Motivo: marcador sem nome não serve para nada às 23h, e o Resolve mostra por que — nome, nota e um ponto no marcador que tem nota (manual p.548, p.784).
- **`lock` por track.** `tracks[i].lock: true`, um booleano que o engine ignora: track travado não deixa mover, apagar nem selecionar keyframe. Recomendação: **entra**. Já é coerente com a decisão de 09/09 ("lock é só edição, sem runtime").
- **Cor por track: não entra como campo.** O Ableton deixa o usuário pintar cada track; `PRINCIPIOS.md §2` proíbe cor decorativa. Recomendação: a cor sai do `type` do track (`dmx`, `laser`, `fx`, `cue`, `media`), como as famílias de nó do Graph — zero campo novo. Pela mesma razão, marcador **não** ganha `color` (`SHORTCUTS.md` já fixou marcador cinza).
- **Altura de faixa e dobra de lane: não entram no `.spell`.** É estado de janela, proibido dentro do show por `FUNCOES/README.md §12`; vão para o `config.json` da GUI. `loop` idem (estado de transporte); se um dia precisar persistir, o lugar é `transport.loop`, que já existe no arquivo.
- **Não é formato, é runtime, e precisa entrar:** `mute` de track não faz nada no engine Rust (não há `mute` nem `solo` em `spellcore/engine/`; o `mute` de `spellcore/script/src/graph.rs:200` é o do nó do graph, outra coisa; mute de track só no protótipo Python, `spellcaster/gui/api.py:168-171`). Hoje os botões M e S da timeline são pintura: o DMX continua saindo. `solo` continua sendo "mute dos outros" calculado no cliente.

## 09/09/2026 · interface DAW (frente `daw-pesquisa`) — aguarda voto

Seis documentos novos em `design/FUNCOES/` (`daw-arranjo`, `daw-sessao`, `browser-dnd`, `mapping`, `audio-video`, `pontos-falhos`), lidos dos manuais do Ableton Live 12, do DaVinci Resolve 20 e do Resolume Arena. O que eles decidiram está lá; o que eles **não** decidem está aqui.

**Formato do `.spell`**

- **`clips[]` por track**: `{"t0", "len", "src", "offset"}` em segundos, `src` relativo à pasta do show. Convive com `keys[]` e com as lanes de parâmetro, não substitui nada. `migrate()` converte o `clip` (singular) do track laser sem subir `VERSION`. Motivo: hoje um clipe de laser de 46,8 s desenha uma lane vazia, porque a timeline só conhece `keys` (`daw-arranjo.md §4.1`).
- **Tipos de track `audio` e `video`**, com `clips[]`, lane `gain` e sem `universe`. A alternativa é serem saídas, e não são: têm posição no tempo (`audio-video.md §1`).
- **Ordem de `tracks[]` é a ordem da tela**, então reordenar reescreve o array e qualquer índice guardado (cue, mapeamento, endereço `track/3/mute`) passa a apontar para outro track. A alternativa é **`uid` por track**, que é o que `FUNCOES/README.md §12` já manda para referência entre objetos, resolve de vez o "Shortcut Target" do Resolume (`mapping.md §7`) e custa um campo. **É a decisão de maior alcance desta rodada.**
- **`marker.go: "<endereço>"`** transforma marcador em locator (Ableton §6.4) sem objeto novo, e **`cue.fires: ["<endereço>", ...]`** é o que dá célula de grade aos tracks de mídia, cujos valores `cue.values` (só `endereço DMX → número`) não alcançam. O voto decide se os dois se chamam igual (lista nos dois) ou se o marcador fica com string.
- **`"midi"` vira `"map"`, com a fonte no prefixo da chave** (`"key:Space"`, `"midi:144/60"`, `"osc:/spell/go"`, `"widget:go"`) — o mesmo vocabulário que `input {key}` já documenta e que `chave()` do graph já produz. `midi_map` vira `map_set`; `migrate()` prefixa o bloco antigo. É o que impede duas fontes de verdade entre a frente `midi` e o modo de mapeamento (`mapping.md §6`). O `bind.js` da rodada 5 (chave `note:1:60`, persistência em `localStorage`) perde as duas coisas: a chave vira a do engine e o mapa vai para o `.spell`.
- **`show.outputs[]` ganha `{"type":"screen","monitor":N}`** para a segunda janela de vídeo no projetor (`audio-video.md §4`), e o track de vídeo aponta para ela por `screen`.

**Atalhos**

- **`Ctrl+Shift+A` passa a ser o modo de mapeamento** (o dono fixou a tecla). Consequência: "selecionar nada" sai de `Ctrl+Shift+A` e vai para **`Alt+A`**, pela própria gramática de `SHORTCUTS.md` (*"Alt = variante/limpa"*, como `Alt+I`/`Alt+O`/`Alt+X` já fazem). Nota de fonte: no Resolume `Ctrl+Shift+A` é o Advanced Output; os modos de atalho de lá são `Shift+Ctrl+K/M/O/X`. A tecla fica como o dono pediu.
- **`Tab` continua sendo a troca de Face** (editor ↔ performance, PRD §10). Arrangement ↔ Session é a **segunda batida do `Shift+2`**, na mesma lógica de `Shift+Z` (enquadra, bate de novo e volta) e de `M` (cria marcador, bate de novo e edita). Terceira vez que `Tab` é disputada; fica decidido e sai dos pontos abertos de `FUNCOES/README.md`.
- **`Ctrl+E` age no objeto selecionado**: clipe corta (Ableton §6.12), keyframe abre o menu de easing (`SHORTCUTS.md`). Um atalho, dois objetos.
- **Altura de faixa por track** (Resolve p.643) contra a altura global proposta em `timeline-daw.md` item 19.

**Recusas deliberadas, registradas para não voltarem por esquecimento**

- **Consolidate** (Ableton §6.13): grava sample novo em `Samples/Processed/Consolidate`. Não renderizamos mídia e não escrevemos arquivo derivado na pasta do show.
- **Follow actions** (Ableton §16.7): duas ações com probabilidade, dez tipos, `Jump Target`, multiplicador de loops. Nossa cue já tem `follow: bool` (= o Follow Action `Next`), e o resto é máquina de estados escondida na lista de cues — a máquina de estados já está sendo desenhada no lugar certo (nó `state` do graph).
- **Toggle, latch, contador e limiar no mapa direto**: vão para o graph, que já tem `logic.*`, `math.*`, `time.*` e `state` no catálogo fechado. O mapa flat guarda chave → comando e nada de estado.
- **Quatro modos de mapeamento por protocolo** (Resolume, `Shift+Ctrl+K/M/O/X`, uma cor cada): um modo só, porque o protocolo já vem na entrada e quatro cores contra `PRINCIPIOS.md §2`. `K`/`M`/`O` sobrevivem como filtro de fonte **dentro** do modo.
- **NDI e Spout**: bloqueio de licença (SDK registrado, contra `FUNCOES/README.md §14`) e de contexto GPU no WebView, não de esforço. O caso real — levar imagem ao projetor — resolve-se com a segunda janela.
- **Cor livre por track** (Resolve p.621, 16 cores), **automação vermelha × modulação azul** (Ableton §26.3), **marcador colorido**: `PRINCIPIOS.md §2`, cor significa estado.
- **`.mov` como formato de vídeo** (Ableton §27.1): o critério é o do reprodutor, e o reprodutor é o WebView.

**Defeito provado, para a frente que corrigir**

- `spellgui/web/timeline.js:308-312`: `commit()` reescreve `spec.mute` a partir de **cada** lane, e as lanes de parâmetro do mesmo track carregam a cópia velha — a última escrita vence e desfaz o mute que o operador acabou de ligar. Vale igual para `solo`. Reproduzido com `shows/medgrupo.spell` (o track laser tem lanes `.rot` e `.scale` sobre o mesmo `spec`). A correção é uma fonte só: `L.mute` vira leitura de `L.spec.mute`.

## 09/09/2026 · integração da rodada 2

- 2026-09-09 — **Roda pura do mouse rola o conteúdo também no PATCHBAY**, e o `graph.js` passa a
  escrever `k.ymax` (`desenha()`, a partir da caixa mais baixa). Motivo: um gesto só nas duas telas
  (`roda` rola, `Shift`+roda anda, `Ctrl`+roda dá zoom), e escrever `ymax` custa duas linhas contra
  o listener de captura que seria preciso para devolver o zoom à roda pura no graph. Decisão de
  integração, **reversível**: se o voto disser que graph de nós tem que dar zoom na roda pura (como
  Blender e TouchDesigner), o `canvaskit.js` ganha o desvio e o `ymax` continua servindo ao clamp.

## 09/09/2026 · Loop In-Out com track armado reescreve o take — aguarda voto

- Loop de transporte (`loop_set`, In–Out) e gravação (`rec_arm`) são estados independentes: com os
  dois ligados, cada volta do loop grava por cima do que a volta anterior gravou. Não há erro; há
  duas semânticas possíveis e nenhuma está escolhida.
- **O que o voto decide:** (a) a volta do loop **desarma** o track (uma passada, um take, como o
  punch do Pro Tools), ou (b) fica como está e a documentação diz que loop + arme sobrescreve
  (como o overdub destrutivo), ou (c) cada volta vira um take novo — que é campo novo no `.spell` e
  não sai de graça.
- Não implementado e não removido até o voto: hoje é (b), sem aviso na tela.

## 09/09/2026 · A página inicial da janela do programa — aguarda voto

- **O que o voto decide:** ao abrir a janela do Spellcaster, o que aparece primeiro — o **aparelho** (`spellgui/web/laser3d/app.html`, o projetor laser em 3D, com a splash, a traseira como menu e o Pino como navegação) ou a **timeline** (`spellgui/web/index.html`).
- **Recomendação: o aparelho.** Foi o pedido literal ("não quero abrir no browser, quero uma GUI do programa" · "que a UI seja já wild e com 3D e com shaders GLSL e que seja cool de operar"), e é a regra "função antes de UI" aplicada de verdade: o programa é o aparelho, a timeline é o gravador do aparelho. Abrir pelo gravador inverte a metáfora e devolve o software quadradão.
- **O que já está de pé nos dois casos:** o `laser3d/` é a página principal e roda ligado ao registry (`laser_open`, `laser_play`, `laser_stop`, `laser_close`, `laser_param`, `laser_stats`, `laser_files`, `resume`/`pause`, `show_get`) por `bus.js`, com three.js e as fontes vendorizados em `spellgui/web/vendor/` — nada de CDN, o evento não tem rede. Sem engine a página continua inteira, em modo local.
- **O que fica fora enquanto o voto não sai:** a janela nativa (Tauri) é da frente `gui-janela`; esta decisão é só qual URL ela carrega primeiro.
- Motivo: é escolha de produto, não de código — trocar a página inicial é uma linha, mas define o que o programa **é** quando abre.

## 09/09/2026 · O que entra no menu do Pino — aguarda voto

- **Os cinco pinos continuam sendo as cinco telas** (1 laser · 2 fósforo · 3 patchbay · 4 teatro · 5 info). O balão ganhou **dois itens que não são pino**: o **GRAVADOR** (a timeline, `index.html`) e a **MESA** (a Face, `face.html`). Cada um leva a frase que justifica a peça: a timeline é a fita do aparelho, a Face são os botões grandes que o operador aperta no show.
- **O que o voto decide:** se peça sem pino pode morar no balão, ou se cada uma precisa virar um pino — o que exigiria um Pino com sete pinos (XLR-7 não existe) ou um segundo cabo.
- Motivo: a regra é "nada aparece por conveniência de software". Dois itens sem pino são a exceção que o balão está abrindo; ou ela é aceita com a justificativa, ou o aparelho precisa crescer um conector.

## 10/09/2026 · chassi-4 · A dobradiça da tampa fica na FRENTE, não atrás

- **O pedido dizia** "dobradiça de verdade, em `z = D/2`" (traseira). **Ficou em `z = −D/2 + 4,5 mm`** (frente), no centro do raio da aresta dianteira.
- **Motivo:** quem abre a tampa é `app.js`, que escreve um ângulo **negativo** em `lid.rotation.x`. Com o eixo atrás, ângulo negativo joga a chapa para baixo e para trás: ela atravessa o painel traseiro e o flightcase — é exatamente o "tampa clipando" reclamado. Com o eixo na frente, o mesmo ângulo negativo abre a tampa para cima e para a frente, sem varrer nada entre 0 e −1,9 rad, e sem precisar de limite de curso artificial.
- **A alternativa era editar `app.js`** (inverter o sinal), e `app.js` não é desta frente. Se o integrador preferir a dobradiça atrás, o conserto é uma linha em `app.js` (`rotation.x = +ângulo`) mais mover `lid.position.z` de volta para `D/2 − 4,5 mm`.
- **Efeito colateral que fica para o integrador:** `app.js` sobe os parafusos da tampa 50 mm (`s.position.y = .004 + sT * .05`). Eles são filhos da tampa e acompanham o giro, mas o curso é exagerado; 8 mm bastaria.

## 10/09/2026 · `Z` e `Shift+Z` trocados de lado, para bater com o manual do SolidWorks

- Estava `Z` = zoom **+** e `Shift+Z` = zoom **−**. A referência rápida oficial da Dassault
  (`quick_reference.pdf`, p. 1, `SWQRCENG06060`) diz o contrário: **`Z` afasta, `Shift+Z` aproxima**.
  Como a câmera da vista SHOW passou a copiar o SolidWorks inteiro (`design/FUNCOES/camera-solidworks.md`),
  ficar com metade do mapa invertida seria a pior das duas opções: quem conhece o CAD erra, e quem não
  conhece não ganha nada. Os dois continuam sendo endereços (`cam.zoomIn`, `cam.zoomOut`), remapeáveis.
- Não estava no pedido; foi decidido aqui porque o pedido mandou seguir o manual e o manual discorda
  do que havia. **Reversível em uma linha** (as duas teclas no `Bind.def` de `app.js`) se o voto disser
  que a intuição "Z aproxima" vale mais que a compatibilidade com o CAD.
