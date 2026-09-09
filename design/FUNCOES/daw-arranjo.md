# Timeline como Arrangement View — `spell timeline` (painel `Shift+2`)

Pedido do Matheus (09/09/2026): *"quero uma interface de uso e funcionalidade e usabilidade e interface que nem o ABLETON, profissional e com features e segmentos pensados"*, e *"quero drag n drop de elementos e saídas e mídias e laser e áudio e vídeo"*.

`timeline-daw.md` (branch `frente/timeline-daw`) já leu os dois manuais para **navegar, marcar e automatizar**: roda/zoom/pan, follow, loop, snap, marcador, nudge, lane de parâmetro. Aquele documento tem 38 comportamentos e não se repete aqui. O buraco que ele mesmo declara é o item 37: *"a nossa timeline não tem clip com borda"*. **Este arquivo é o clipe.** O que ele acrescenta: faixa de visão geral, régua com brace e locators, cabeçalho de track de verdade (nome, arm/solo/mute, lock, dobra, altura, reordenar, cor), clipe como objeto de primeira classe (laser, áudio, vídeo, fx, osc), seletor de lane de automação, seleção de tempo × seleção de objeto, Inspector à direita e transporte.

Fontes. **Ableton Live 12**, manual oficial online (`ableton.com/en/live-manual/12/`), citado por seção. **DaVinci Resolve 20**, manual local `C:\Program Files\Blackmagic Design\DaVinci Resolve\Documents\DaVinci Resolve.pdf` (4140 páginas), citado por página. Nada de memória.

Código. Linhas de `spellgui/web/*` e `spellcore/*` no commit base `dac3e0a`. A coluna **hoje** descreve o que existe; `pontos-falhos.md` traz a auditoria com screenshot.

| Item | Quem resolveu melhor | Por quê |
|---|---|---|
| Clipe na timeline | Resolve | Clipe é mídia com In/Out de origem e In/Out de linha; `Ctrl+E` corta no ponto do clique, arrastar a borda apara. O Ableton tem o mesmo, com warp e compasso atrás, que não entram (`timeline-daw.md` item 38) |
| Cabeçalho de track | Resolve | Nome editável (p.645), cor por menu (p.621), lock/R/S/M com arraste sobre vários (p.3681), Tracks Index como lista paralela (p.646, p.649) |
| Reordenar track | Resolve | Arrastar no Index com uma linha branca mostrando onde entra (p.3681) — sem isso, "mover track" vira menu Up/Down (p.3574), que é pior |
| Faixa de visão geral | Ableton | §6.1: arrastar horizontal rola, vertical dá zoom, duplo-clique enquadra tudo. O Resolve não tem overview, tem Zoom Slider |
| Seletor de lane de automação | Ableton | §25.5: dois seletores (device, parâmetro), LED no que está automatizado, "Show Automated Parameters Only". É exatamente o nosso problema de cinco lanes por track |
| Loop brace e locators | Ableton | §6.6 e §6.4: brace arrastável nas bordas e no meio; locator dispara playback e é mapeável |
| Seleção de tempo × objeto | Ableton | §6.9: "Arrangement editing is selection-based"; clique no fundo põe insert marker, arrasto faz intervalo |
| Inspector | Resolve | p.412: painéis por aspecto (Video, Audio, Effects, Transition, Image, File), painel que não se aplica fica cinza — não some |
| Consolidate | nenhum | **Não entra.** Ableton §6.13 grava um sample novo por track em `Samples/Processed/Consolidate`; nós não renderizamos mídia e não vamos escrever arquivo derivado no show |

## 1. Objetos e verbos

**Clipe** — um pedaço de mídia posto no tempo. É o objeto que falta. Quatro campos: `t0` (onde começa na linha), `len` (quanto dura na linha), `src` (arquivo, relativo à pasta do show) e `offset` (de que ponto do arquivo se lê). Verbos: **inserir** (drop), **mover**, **aparar** (trim das duas bordas), **cortar** (`Ctrl+E`), **duplicar** (`Ctrl+D`), **copiar/colar**, **apagar** (`Delete`). Um clipe não tem envelope próprio: automação é do track (§3.4).

**Track** — a linha do `.spell` (`tracks[]`). Ganha verbos que hoje não tem: **renomear**, **reordenar**, **travar**, **dobrar**, **mudar altura**. Tipos que desenham clipe: `laser` (`.ild`), `audio` (`.wav/.mp3/.flac`), `video` (`.mp4/.webm`), `fx` (`.rhai`), `osc`. Tipos que desenham só keyframes: `dmx`, `artnet`, `fixture`. Tipo `cue` desenha marca vertical (`timeline-daw.md` item 36).

**Lane** — linha desenhada. Um track dá uma lane principal e uma lane por parâmetro automatizado (`timeline.js:157-196`). Novo: as lanes de parâmetro **dobram** e existem sob demanda, não as cinco fixas de `LANE_PARAMS` (`timeline.js:190-196`).

**Seleção de tempo** — intervalo sem objeto. Hoje não existe: `CK.sel()` guarda keyframes (`canvaskit.js:152-168`).

**Locator** — marcador que dispara. No Ableton §6.4 locator é ponto na scrub area que lança o playback; nós já temos `markers[]` e o track `cue`. **Locator não vira objeto novo**: ver §5.

## 2. Segmentos da tela

De cima para baixo, largura total, sem sobreposição (é a queixa 4 de `pontos-falhos.md`):

| Segmento | Altura | Conteúdo | Origem |
|---|---|---|---|
| Barra de transporte | 32 px fixos, com quebra ou overflow em menu | posição (timecode + segundos), play/stop, rec, loop, follow, snap, nome do show editável | Ableton (Control Bar) / Resolve p.625 (toolbar) |
| Faixa de visão geral | 24 px | o show inteiro em miniatura, contorno da janela atual | Ableton §6.1 |
| Régua | 24 px (`TL.rulerH`, `timeline.js:33`) | timecode adaptativo, brace de loop, marcadores, In/Out | Ableton §6.1 / Resolve p.484 |
| Tracks | o que sobra, rolável | cabeçalho de 192 px (`TL.headW`) + lanes | ambos |
| Inspector | 280 px à direita, dobrável | do que está selecionado | Resolve p.412 |
| Rodapé | 20 px | espaçamento da grade + estado do bus | Ableton §6.10 ("displayed above the time ruler in the lower right corner") |

O Inspector à direita e o browser à esquerda (`browser-dnd.md`) formam o layout que `SHORTCUTS.md § Interface` já descreve: *"Inspector à direita, Media Pool à esquerda"*.

## 3. Tabela de comportamentos

`hoje` = commit base `dac3e0a`. P1 = falta no primeiro minuto. Nenhuma linha repete `timeline-daw.md`.

### 3.1 Visão geral e régua

| # | Comportamento | Origem | Gesto | Efeito no nosso modelo | Hoje | P |
|---|---|---|---|---|---|---|
| A1 | Faixa de visão geral: arrastar horizontal rola, vertical dá zoom, duplo-clique dentro do contorno enquadra tudo | Ableton §6.1 ("drag left or right to scroll... drag vertically to zoom in or out... double-click anywhere within the black outline") | arraste / duplo-clique | Só a vista (`CK.view`), nada no `.spell` | falta | **P1** |
| A2 | Contorno marca a janela atual dentro da faixa | Ableton §6.1 | — | Idem | falta | **P1** |
| A3 | Clicar na régua toca dali | Ableton §6.1 ("Clicking anywhere in the scrub area launches playback from that point") | clique | `locate {t}` + `resume` | parcial — o clique na régua só arrasta o playhead (`timeline.js:704-712,747`) | P2 |
| A4 | Brace de loop com três alças: borda esquerda, borda direita, meio | Ableton §6.6 ("dragging from the left or right edge adjusts the loop start/end points, while dragging the brace bar horizontally moves the loop without changing its length") | arrastar | As alças são In/Out (`timeline-daw.md` item 7); o meio é o item 8 de lá. **Novo aqui**: desenhar como brace (duas serifas e uma barra), não como risco | falta o desenho — `timeline.js:622-623` faz um retângulo cinza de 3 px que some contra a grade (`pontos-falhos.md` item 27) | P2 |
| A5 | Clicar no brace seleciona o que está dentro | Ableton §6.9 ("Clicking on the loop brace is a shortcut for executing the Edit menu's Select Loop command") | clique no brace | Seleciona keyframes e clipes entre In e Out | falta | P3 |
| A6 | Setas movem o brace pela grade; `Ctrl+←/→` encurta/alonga; `Ctrl+↑/↓` dobra/divide | Ableton §6.6 | teclado | Reescreve `in`/`out` | falta | P3 |
| A7 | Timecode adaptativo: com o show inteiro na tela o rótulo é `mm:ss`; com um segundo na tela é `ss:ff` | ambos (Ableton §6.10 mostra o espaçamento; Resolve p.647 tem os três presets de zoom) | automático | Só desenho | errado — `tc()` sempre imprime `hh:mm:ss:ff` (`timeline.js:56-57`): 11 caracteres para dizer 10 segundos | P2 |
| A8 | A escada de grade tem degrau de quadro | ambos | automático | `STEPS` ganha `1/fps` e `5/fps` como primeiros degraus, lidos de `show.fps` | falta — menor degrau é 0,04 s (`timeline.js:28`), que a 30 fps não é quadro nem múltiplo de quadro | P2 |

### 3.2 Track

| # | Comportamento | Origem | Gesto | Efeito no nosso modelo | Hoje | P |
|---|---|---|---|---|---|---|
| B1 | Nome do track editável no cabeçalho | Resolve p.645 ("click the default 'Video X' or 'Audio X' track name to select it, then type your preferred name and press the Return key") | duplo-clique, digitar, `Enter` | `show_patch {ops:[{op:"add", path:"/tracks/3/name", value:"..."}]}`. O campo `name` já é escrito por `track_add` quando vem `label` (`edit.rs:692-694`) | falta — o nome é `spec.file`/`spec.clip` cortado em 22 caracteres (`timeline.js:503`) | **P1** |
| B2 | Arm / Solo / Mute valem de verdade | ambos (Resolve p.3681: *"you can use the Lock, Record, Solo, and Mute controls to quickly enable or disable multiple tracks by clicking and dragging up or down"*) | clique em R/S/M, ou arrastar sobre vários | `mute` vai para o `.spell` e o player Rust obedece (`timeline-daw.md` item 30, já em `DECISOES.md`) | **quebrado** — `commit()` reescreve `spec.mute` a partir de cada lane e a lane de parâmetro desfaz o mute (`timeline.js:308-312`; `pontos-falhos.md` item 10) | **P1** |
| B3 | Lock trava o track | Resolve p.3635 ("Click any track's lock control and drag over the lock controls of other tracks") | `Shift+L`, ou arrastar sobre os cadeados | `tracks[i].lock` (proposto em `timeline-daw.md §3`, aguarda voto) | falta | P2 |
| B4 | Dobrar as lanes de parâmetro | Ableton §6.9 (`U`) e §25.5 ("Using the left and right arrow keys on a main track will fold/unfold its automation lanes") | `U`, ou `←`/`→` no cabeçalho focado | Estado de janela. **Correção de rumo**: as lanes passam a existir só para parâmetro que tem keyframe | falta — cinco lanes fixas por track (`timeline.js:190-196`) | **P1** |
| B5 | Altura por track, arrastando a divisória | Resolve p.643 ("any track in the Timeline can be individually resized by dragging its top divider in the Track Header area") | arrastar a borda de cima do cabeçalho | Estado de janela. **Divergência**: `timeline-daw.md` item 19 propôs altura global (`TL.rowH`); o Resolve p.643 é por track, e é o que se espera de um track de áudio ao lado de um de DMX. Fica por track, com `Alt`+arrastar aplicando a todos (Ableton §6.9: *"hold Alt while resizing a single track"* redimensiona todos) | falta | P2 |
| B6 | Reordenar arrastando, com linha no destino | Resolve p.3681 ("As you drag, a white line shows you where that track will be inserted when you release it") | arrastar o cabeçalho | **Reescreve a ordem de `tracks[]`.** Não há campo de ordem: a ordem do array é a ordem da tela. Precisa de `track_move` (§6) | falta | P2 |
| B7 | Cor por família de track | Resolve p.621 ("Each track can be color-coded with one of 16 different colors") | — | **Rejeitado como cor livre** (`PRINCIPIOS.md §2`; já decidido em `timeline-daw.md` item 31). Entra derivada do `type`: `dmx`/`artnet` âmbar, `laser` vermelho, `audio` verde, `video` azul, `fx`/`osc` cinza. Não é campo do `.spell` | falta | P3 |
| B8 | Track novo pelo drop no vazio abaixo dos tracks | Ableton §4.10 ("Dragging and dropping content from the browser into the space... below Arrangement View tracks will create a new track and place the new item(s) there") e §6.1 (Mixer Drop Area) | soltar mídia abaixo do último track | `track_add {kind, label}` + o clipe. Contrato completo em `browser-dnd.md §3` | falta | **P1** |
| B9 | Apagar tracks vazios de uma vez | Resolve p.511 ("Delete Empty Tracks") | menu do cabeçalho | `track_del` em série | falta | P3 |

### 3.3 Clipe

| # | Comportamento | Origem | Gesto | Efeito no nosso modelo | Hoje | P |
|---|---|---|---|---|---|---|
| C1 | Clipe desenhado como retângulo com barra de título, nome do arquivo e conteúdo | Resolve p.625 (Filmstrip / Thumbnail / Minimized) / Ableton §6.7 | — | Um `clips[]` por track (§4). Conteúdo: forma de onda para `audio` (`audio-video.md §2`), miniatura de quadro para `laser`/`video`, cor sólida para `fx`/`osc` | **falta inteiro** — o track `{"type":"laser","clip":"medgrupo_laser.ild"}` de `shows/medgrupo.spell` desenha uma lane vazia (`timeline.js:157-181` só conhece `keys` e `spec.<param>`; `pontos-falhos.md` item 14) | **P1** |
| C2 | Só a barra de título arrasta o clipe | Ableton §6.7 ("only the clip bar is draggable, it is not possible to drag from the clip's waveform or MIDI display") | arrastar a barra | Muda `t0`, e o track se mudar de linha | falta | **P1** |
| C3 | Arrastar a borda apara | Ableton §6.7 ("Dragging a clip's left or right edge changes the clip's length") | arrastar borda | Borda direita muda `len`; borda esquerda muda `t0` **e** `offset` na mesma quantidade | falta | **P1** |
| C4 | Deslizar o conteúdo dentro do clipe | Ableton §6.7 (`Ctrl+Shift`+arrastar no waveform) | `Ctrl+Shift`+arrastar no corpo | Muda só `offset` | falta | P3 |
| C5 | Clipe gruda na grade **e** na borda de outro clipe, em marcador e no playhead | Ableton §6.7 ("Clips snap to the editing grid, as well as... the edges of other clips, locators and time signature changes") / Resolve p.546 | automático | `TL.snaps` ganha as bordas de clipe. `Alt` solta (já em `timeline-daw.md` item 11) | parcial — `snapT` existe e só conhece grade e marcador | P2 |
| C6 | Cortar no ponto clicado | Ableton §6.12 (`Ctrl+E`: *"click anywhere within a clip's waveform or MIDI display and then use the shortcut"*) | `Ctrl+E` | Um clipe vira dois: `{t0,len,src,offset}` → `{t0, d, src, offset}` + `{t0+d, len-d, src, offset+d}`. Corta no **clique**, não no playhead: evita mover o transporte para editar | falta | **P1** |
| C7 | Duplicar | Ableton §41.5 (`Ctrl+D`) | `Ctrl+D` | Cópia logo depois: `t0' = t0 + len` | falta | P2 |
| C8 | Duplicar arrastando com `Alt` | Resolve / MadMapper (gesto já fixado em `timeline-daw.md` item 15) | `Alt`+arrastar | Idem, onde soltar. **Colisão consciente**: `Alt` também solta a grade (item 11 de lá); num arrasto de clipe as duas coisas valem juntas, e é assim no Ableton |falta | P2 |
| C9 | Copiar / colar | ambos | `Ctrl+C` / `Ctrl+V` | Cola no playhead, no track focado | falta | P2 |
| C10 | Desativar sem apagar | Ableton §6.9 ("Pressing the 0 key deactivates a selection of material") | `0` | `clips[i].mute: true`. **Divergência**: `SHORTCUTS.md` usa `Shift+D` para mute de track; `0` fica só para o clipe selecionado | falta | P3 |
| C11 | Consolidar clipes adjacentes num só | Ableton §6.13 (`Ctrl+J`) | — | **NÃO ENTRA.** No Ableton *"a new sample is created for every track in the selection"*, gravado em `Samples/Processed/Consolidate`. Nós não renderizamos mídia, e arquivo derivado dentro do show contraria o espírito de `FUNCOES/README.md §12` (o `.spell` referencia originais, não produtos) | — | — |
| C12 | Comandos "…Time" (inserir/apagar tempo em todos os tracks) | Ableton §6.11 (`Ctrl+Shift+X/C/V/Delete`; `Ctrl+I` insere silêncio) | — | **Não entra agora.** Exige ripple em `keys[]` de todos os tracks e não há pedido. Registrado porque é a diferença entre editar clipe e editar linha | — | P3 |
| C13 | Fade in/out no clipe de áudio | Ableton §6.8 (`Ctrl+Alt+F`; `F` sobre a lane alterna os controles) | — | **Não entra agora.** Volume de áudio é lane de automação como qualquer outra; fade seria atalho para dois keyframes. Reavaliar com `audio-video.md` | — | P3 |

### 3.4 Automação

| # | Comportamento | Origem | Gesto | Efeito no nosso modelo | Hoje | P |
|---|---|---|---|---|---|---|
| D1 | Modo automação liga/desliga com `A` | Ableton §25.5 ("enable Automation Mode by clicking the toggle button above the track headers, or using the A shortcut") | `A` | Mostra/esconde todas as lanes de parâmetro. Estado de janela | falta | P2 |
| D2 | Seletor de lane com dois campos e LED no que está automatizado | Ableton §25.5 (Device chooser + Automation Control chooser; *"showing an LED next to their labels"*) | menu no cabeçalho da lane | Primeiro campo é o **alvo** (o track, ou o módulo do graph), segundo é o **parâmetro** — e o par é o endereço textual da regra 2 de `FUNCOES/README.md`: `track/3/scale`, `laser/1/kpps`. É o mesmo endereço que `mapping.md` mapeia | falta — as cinco lanes fixas de `LANE_PARAMS` (`timeline.js:190`) são um seletor sem menu | **P1** |
| D3 | "Show Automated Parameters Only" | Ableton §25.5 | opção do seletor | Padrão **ligado**: só aparece lane de parâmetro que tem keyframe. É o que corrige B4 | falta | **P1** |
| D4 | Botão que manda o envelope para lane própria; com `Alt`, manda todos os automatizados | Ableton §25.5 | clique / `Alt`+clique | Estado de janela | falta | P3 |
| D5 | Esconder a lane não desativa o envelope | Ableton §25.5 ("hiding a lane from view does not deactivate its envelope") | — | Regra, não gesto: dobrar nunca mexe em `keys`. É a regra que impede repetir o defeito B2 | — | **P1** |
| D6 | Envelope preso à música ou ao clipe (Lock Envelopes) | Ableton §6.1 | toggle | **Não entra.** Nossos keyframes são do track e vivem em tempo absoluto; não há segundo modo | — | — |
| D7 | Automação vermelha, modulação azul | Ableton §26.3 | — | **Rejeitado.** `PRINCIPIOS.md §2`: um acento só, cor significa estado. As duas se distinguem pela lane em que estão | — | — |
| D8 | Simplificar envelope | Ableton §25.5.4 ("calculates the optimal number of breakpoints... and removes any unnecessary breakpoints") | menu | Vira necessário quando `gravar-dmx` gravar 30 keyframes por segundo. Registrado para aquela frente, não para esta | falta | P3 |
| D9 | Formas prontas de automação (seno, rampa, ADSR) sobre a seleção de tempo | Ableton §25.5.5 | menu de contexto | **Não entra**: é o que o `fx` (`.rhai`) e o graph fazem melhor. Registrado para não ser reinventado | — | — |

### 3.5 Seleção, transporte e Inspector

| # | Comportamento | Origem | Gesto | Efeito no nosso modelo | Hoje | P |
|---|---|---|---|---|---|---|
| E1 | Clicar no fundo põe um insert marker; arrastar faz seleção de tempo | Ableton §6.9 | clique / arraste no vazio | Seleção de tempo é `{t0, t1, tracks[]}`, separada da seleção de objetos. `Ctrl+L` faz loop nela (`timeline-daw.md` item 6) | falta — arrastar no vazio faz marquee de keyframes (`canvaskit.js:152-168`) | **P1** |
| E2 | Edição baseada em seleção | Ableton §6.9 ("you select something and then execute a command") | — | Regra: todo comando de edição pergunta "seleção de tempo ou seleção de objeto?", nunca as duas ao mesmo tempo | — | **P1** |
| E3 | `Z` enquadra a seleção de tempo, `X` volta o zoom | Ableton §6.2 | `Z` / `X` | **Divergência**: `SHORTCUTS.md` já tem `Shift+Z` (enquadrar tudo) e `timeline-daw.md` item 22 já fixou a segunda batida do `Shift+Z` como voltar. `Z`/`X` ficam de fora; enquadrar a seleção é `Shift+Z` com seleção ativa, mesma lógica do `Ctrl+L` | falta | P2 |
| E4 | Transporte: posição em timecode **e** em segundos, play/stop, rec, loop, follow, snap | ambos | — | `locate`, `resume`, `pause`, `stop` já existem no registry (`registry.rs:243,251,257`) | parcial — `index.html` tem os botões e nenhum campo de posição editável | **P1** |
| E5 | Parar duas vezes volta ao início | Ableton §7.1 ("pressing the Control Bar's Stop button twice") | `Space` duas vezes parado | `stop` + `locate {t:0}` | falta | P3 |
| E6 | Inspector à direita, em painéis por aspecto; painel inaplicável fica cinza e não some | Resolve p.412 ("Inspector panels that are not applicable to your clip or selection are grayed out") | `Shift+7` (já em `SHORTCUTS.md`) | Mostra o selecionado: track (nome, tipo, universo, endereço, saída), clipe (`src`, `t0`, `len`, `offset`), keyframe (t, valor, curva). Todo campo mostra **o endereço textual** ao lado do rótulo — é o que `mapping.md` mapeia e o que `Shift+Ctrl+C` copia (`FUNCOES/README.md §7`) | falta — nenhum Inspector no `index.html` (`pontos-falhos.md` item 25) | **P1** |
| E7 | Alternar Arrangement ↔ Session | Ableton §41.1 (`Tab`) | — | Resolvido em `daw-sessao.md §4`: `Tab` já é a troca de Face em `SHORTCUTS.md` | — | — |

## 4. Modelo de dados

### 4.1 O clipe entra no `.spell`

```json
{"type": "laser", "universe": 1,
 "clips": [{"t0": 0, "len": 46.8, "src": "medgrupo_laser.ild", "offset": 0}],
 "scale": [[0, 1.0], [46.8, 1.0]]}
```

Regras:

- `clips[]` **convive com** `keys[]` e com as lanes de parâmetro. Não substitui nada: `keys` é o valor no tempo de um track `dmx`; `clips` é a mídia no tempo de um track que tem arquivo.
- `t0`, `len` e `offset` em **segundos**, como `duration` e como as chaves de `keys` (`timeline.rs`, `parse_key`). Não em quadros: `fps` é do show, e um `.ild` de 30 fps num show de 25 pode existir.
- `src` é caminho **relativo à pasta do show**, sempre com `/` e nunca com `\`, como o `estatico()` do serve já exige (`serve/src/lib.rs:106-108`).
- `offset` ausente vale 0; `len` ausente vale "até o fim do arquivo", resolvido pela GUI ao carregar.
- Track de áudio e de vídeo são tipos novos: `{"type":"audio","clips":[...]}` e `{"type":"video","clips":[...]}` — sem `universe`, sem `address`. Contrato em `audio-video.md §1`.
- **Migração do que já existe**: o track laser de hoje é `{"type":"laser","clip":"x.ild","fps":30}`. O engine ignora o tipo `laser` (`timeline.rs:404-410`), então `clip` (singular) só vive na GUI. `migrate()` (`show.rs`) converte `clip` → `clips:[{t0:0, len:<duração do arquivo>, src:<clip>, offset:0}]` **sem** subir `VERSION`: é acréscimo compatível, e quem lê `clips` e não acha lê `clip`.

### 4.2 O que é estado de janela e não vai para o show

`FUNCOES/README.md §12` proíbe estado de janela no `.spell`. Vão para o `config.json` da GUI: altura por track, dobra de lane, zoom e posição da vista, follow, snap ligado, modo automação (`A`), largura do Inspector e do browser, último show aberto. In/Out continua no `.spell` (`timeline.js:330-335` grava `show.in`/`show.out`) e loop continua fora, como `timeline-daw.md §3` já decidiu.

### 4.3 Aguarda voto

Vai para `design/DECISOES.md`, sem implementar nem remover:

1. **`clips[]` como campo de track** e a migração de `clip` → `clips`, no formato de §4.1.
2. **Tipos `audio` e `video`** como tracks do show (a alternativa é serem saídas, e não são: têm posição no tempo).
3. **Ordem de `tracks[]` é a ordem da tela** (B6): reordenar reescreve o array, e qualquer índice guardado em cue ou mapeamento passa a apontar para outro track. A alternativa é `uid` por track (`FUNCOES/README.md §12`: *"referência entre objetos por UID, nunca por nome curto"*), que resolve de vez e custa um campo.
4. **Altura por track** (B5) contra a altura global proposta em `timeline-daw.md` item 19.

## 5. Locator: por que não entra como objeto

Ableton §6.4 tem locator: dispara playback, é mapeável, tem nome, `Ctrl+R` renomeia. Nós já temos duas coisas que fazem isso: `markers[]` (ponto na régua com nome e nota, `timeline-daw.md` item 26) e o track `cue` (`timeline.rs:354`). Um terceiro objeto quebra a regra 9 de `FUNCOES/README.md` (*"um verbo por conceito"*).

Decisão proposta: **o marcador ganha um campo opcional `go`**, que é um endereço do registry.

```json
{"t": 12.5, "name": "pico", "note": "", "go": "cue/3/go"}
```

Marcador sem `go` é marcador. Marcador com `go` é locator: triângulo na régua, dispara quando o transporte passa e quando recebe duplo-clique. O engine já tem a metade de baixo — `in.marker` é nó do graph e `input {key:"marker:pico"}` é a entrada (`script/src/graph.rs:280-290`). **Também aguarda voto**: é mudança de formato.

## 6. Comandos que faltam no registry

Os 30 comandos de hoje (`registry.rs` + `edit.rs`) não têm nenhum de clipe. O que esta função precisa, no padrão `<objeto>_<verbo>`:

| Comando | Argumentos | Faz |
|---|---|---|
| `clip_add` | `track, t0, len, src, offset` | Insere; devolve o índice |
| `clip_set` | `track, index, t0?, len?, offset?, mute?` | Move, apara, desliza |
| `clip_del` | `track, index` | Remove |
| `clip_split` | `track, index, t` | Corta em dois (C6) |
| `track_move` | `from, to` | Reordena `tracks[]` (B6) |

`track_set {index, name?, mute?, lock?}` não precisa existir: `show_patch` já faz (`edit.rs:824`) e o caminho é `/tracks/3/name`. A frente `comandos` decide se `clip_*` também vira `show_patch`; a diferença é que `clip_split` tem lógica (recalcular `offset`) e `show_patch` não tem onde pôr lógica.

## 7. Atalhos

O que este arquivo acrescenta a `SHORTCUTS.md` e a `timeline-daw.md §5`:

| Ação | Tecla | Origem | Conflito |
|---|---|---|---|
| Cortar o clipe no ponto do clique | `Ctrl+E` | Ableton §6.12 | **colide** com `Ctrl+E` de `SHORTCUTS.md` ("Easing do keyframe selecionado abre menu"). Resolução: `Ctrl+E` age no objeto selecionado — clipe corta, keyframe abre easing. Um atalho, dois objetos; é a regra E2 |
| Duplicar a seleção | `Ctrl+D` | Ableton §41.5 | nenhum |
| Renomear o track focado | `Ctrl+R` | Ableton §6.4 (lá é renomear locator) | nenhum |
| Modo automação | `A` | Ableton §25.5 | nenhum; tecla nua no painel focado (`FUNCOES/README.md § Pontos abertos`) |
| Desativar o clipe selecionado | `0` | Ableton §6.9 | nenhum |
| Enquadrar a seleção de tempo | `Shift+Z` com seleção ativa | Ableton §6.2 (lá é `Z`) | nenhum; mesma tecla de enquadrar tudo, com seleção |

## 8. Testes que provam cada peça

Um por regra não trivial, no feitio de `timeline-daw.md §4` (função pura + `assert`, rodando em `node`):

| Função | Entrada | Saída esperada |
|---|---|---|
| `TL.clipSplit(c, t)` | `{t0:0,len:10,src:"a",offset:2}`, `t=4` | `[{t0:0,len:4,src:"a",offset:2}, {t0:4,len:6,src:"a",offset:6}]` |
| `TL.clipTrim(c, "L", dt)` | mesmo clipe, `dt=+3` | `{t0:3,len:7,offset:5}` (a borda esquerda move `t0` e `offset` juntos) |
| `TL.clipTrim(c, "R", dt)` | `dt=-2` | `{t0:0,len:8,offset:2}` |
| `TL.snapsDe(track)` | track com dois clipes | as quatro bordas, ordenadas |
| `TL.lanesDe(spec)` | spec com `scale` e sem `rot` | uma lane principal e **uma** de parâmetro (prova D3) |
| `TL.commit` com lane de parâmetro em mute divergente | o caso de `pontos-falhos.md` item 10 | `spec.mute === true`; o teste que falha hoje já existe em `scratchpad/prova_mute.js` |
| `TL.migraClip(track)` | `{"type":"laser","clip":"x.ild"}` | `clips:[{t0:0,src:"x.ild",offset:0}]` |
| Rust, `timeline.rs` | track com `"mute": true` | nenhuma escrita no universo |

Prova visual, na forma de `pontos-falhos.md`: screenshot headless de `index.html` com `shows/medgrupo.spell`, mostrando `medgrupo_laser.ild` como retângulo de 46,8 s com nome, e a lane `scale` dobrada.
