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
