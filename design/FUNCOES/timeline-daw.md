# Timeline como DAW — `spell timeline` (painel `Shift+2`)

Pedido do Matheus (09/09/2026): "Copie a usabilidade das DAW tipo Ableton ou do DaVinci Resolve, de como fazer um bom sistema de timeline". Este arquivo lê os dois, linha por linha, e diz o que entra na nossa timeline (tracks dmx/laser/fx com keyframes, cues, marcadores, In/Out), com que gesto, e o que já existe no código.

Fontes. **DaVinci Resolve 18.6** está instalado nesta máquina (`Resolve.exe`, versão 18.6.6.7): toda afirmação sobre ele cita a página do manual em `C:\Program Files\Blackmagic Design\DaVinci Resolve\Documents\DaVinci Resolve.pdf` (o PDF que veio com essa versão; todos os números de página abaixo são dele). **Ableton Live 12** não está instalado: as afirmações citam a seção do manual oficial (`ableton.com/en/live-manual/12/`). Não há arquivo em `fontes/` porque cada afirmação vale uma linha da tabela e a citação cabe na própria linha.

Código. As linhas de `spellgui/web/*` são do commit base `dac3e0a`. A frente `timeline-ux` está editando `timeline.js`, `canvaskit.js` e `index.html` agora; a coluna "hoje" descreve o que existia quando isto foi escrito, e o §4 assume a base dela pronta (layout fixo, roda/Shift/Ctrl, loop no engine, undo local, atalhos de `SHORTCUTS.md`).

| Item | Quem resolveu melhor | Por quê |
|---|---|---|
| Navegação (roda, zoom, pan) | Ableton | Roda = tracks, `Ctrl`+roda = zoom, `Shift`+roda = lateral é o que o Matheus já fixou; o Resolve usa `Option` para zoom e `Command` para andar, e isso briga com a gramática de `SHORTCUTS.md` |
| Enquadramento | Resolve | Três presets declarados (Full Extent, Detail, Custom) e `Shift+Z` que volta ao zoom anterior; o Ableton só tem o arraste vertical na régua |
| Não perder o playhead | Ableton | `Follow` rola a vista e **pausa sozinho** quando o operador edita ou rola; o Fixed Playhead do Resolve prende o playhead no centro e briga com o scrub |
| Automação | Ableton | Envelope por parâmetro em lane própria, breakpoint criado no clique da curva, `Alt` solta a grade, `Shift` afina o valor — é exatamente o nosso keyframe |
| Marcador | Resolve | Marcador tem nome, nota e navegação por tecla; "marcador com nota aparece com um ponto" é o estado sem cor decorativa |
| Cabeçalho de track | Resolve | Lock com arraste sobre vários cabeçalhos; mute/solo/arm o Ableton também tem, a trava não |
| Precisão sem mouse | Resolve | `,` e `.` empurram a seleção 1 quadro, com `Shift` 5 |
| Grade de cenas | Ableton (Session View) | É o nosso cue list, e ele já está especificado em `cenas-cues-dmx.md`; aqui só entra o que a timeline desenha |
| Tempo musical (tap tempo, quantize, warp) | nenhum | Não entra: o show é relógio e timecode, não compasso. Ver §2, última linha |

## 1. Objetos e verbos

Sem vocabulário novo. **Track** é a linha do `.spell` (`tracks[]`); **lane** é uma linha desenhada — um track vira uma lane principal (`spec.keys`) e uma lane por parâmetro animado (`spec.scale`, `spec.rot`, `spec.x`…), `timeline.js:157-196`. **Keyframe** é o breakpoint do Ableton: `[t, valor, curva]`. **Marcador** é ponto na régua. **In/Out** é o par que define o intervalo de trabalho (`SHORTCUTS.md`), e é ele que faz o papel do *loop brace* do Ableton — não há um segundo par de alças. **Cue** é o track de tipo `cue` (`spellcore/engine/src/timeline.rs:354,369,393`): a lista de cues no tempo já é um track, não um objeto novo.

Verbos que a tabela do §2 acrescenta: **enquadrar** (fit, detalhe, anterior), **seguir** (follow), **empurrar** (nudge), **travar** (lock), **dobrar** (fold das lanes de parâmetro), **selecionar tempo** (intervalo, não pontos).

## 2. Tabela de comportamentos

`hoje` = estado no commit base. P1 = o operador sente falta no primeiro minuto.

| # | Comportamento | Origem | Gesto | O que faz na nossa timeline | Hoje | P |
|---|---|---|---|---|---|---|
| 1 | Roda anda nos tracks; `Ctrl`+roda dá zoom; `Shift`+roda anda de lado | Ableton §6.2 (`Ctrl`+roda = zoom) / Resolve p.648 (roda = tracks) | roda, `Ctrl`+roda, `Shift`+roda | Idem. **Divergência**: o Resolve p.648 usa `Option` para zoom e `Command` para andar; ganha o Ableton, porque bate com a ordem já fixada pelo Matheus e com `Ctrl` = comando de `SHORTCUTS.md` | errado — `canvaskit.js:169-173`: roda nua dá zoom, `Shift` rola vertical; nada anda de lado | P1 (frente `timeline-ux`) |
| 2 | Zoom ancorado no cursor | Resolve p.647 ("Zoom Around Mouse Pointer", opcional lá) | `Ctrl`+roda sobre o ponto | Idem, sempre ligado: opção de preferência para isto é `PRINCIPIOS.md §3` proibido ("nada que muda de lugar sozinho" vale dobrado para o modo de zoom) | feito — `canvaskit.js:100-105` | — |
| 3 | Pan arrastando com o botão do meio | Resolve p.648 ("middle-clicking and dragging in any direction") | botão do meio | Idem, nos dois eixos | feito — `canvaskit.js:129-149`; já está em `SHORTCUTS.md` | — |
| 4 | Follow: a vista acompanha o playhead e **pausa sozinha** quando o operador rola ou edita | Ableton §6.2 | ligado por padrão; `Alt+Shift+F` liga/desliga | Tocando, quando o playhead sai da janela a vista salta uma página. Qualquer pan, zoom ou edição desliga o follow até o próximo `locate`/play. **Divergência**: o Fixed Playhead do Resolve (p.545) prende o playhead no centro e rola o fundo; rejeitado, porque o nosso scrub arrasta o playhead e os dois gestos brigam | falta — `setT` só move `TL.t` (`timeline.js:340-343`); tocando com zoom o playhead sai da tela e não volta | **P1** |
| 5 | Loop no intervalo In–Out, com o wrap valendo também quando quem toca é o engine | Ableton §6.6 (loop brace) / Resolve p.614 ("loops back to the beginning when the end of that command's range is reached") | `Ctrl+L`, alças de In/Out na régua | Idem. Hoje o wrap está escrito no relógio local e some quando existe player: `frame()` devolve antes (`timeline.js:390`) e o loop só viaja como argumento de `play_show` (`:365`), então ligar/desligar durante a execução não faz nada — é o "controle de loop bugado" do feedback | parcial — `timeline.js:36,365,391,861` | **P1** (frente `timeline-ux`) |
| 6 | Loop na seleção de tempo | Ableton (Loop Selection, `Ctrl+L`) | `Ctrl+L` com um intervalo selecionado | Com seleção de tempo ativa, `Ctrl+L` põe In/Out nas bordas da seleção e liga o loop; sem seleção, continua ligando/desligando. Mesma tecla, dois casos — não é tecla nova | falta | P2 |
| 7 | Alças de In/Out na régua, barra entre elas | Resolve p.482 ("Mark In/Out") / Ableton §6.6 | arrastar as alças | Idem | feito — desenho `timeline.js:622-629`, arraste `:704-712,748-749`, gravação `:330-335` | — |
| 8 | Arrastar a barra In–Out inteira, sem mudar a duração | Ableton §6.6 (o loop brace move inteiro) | arrastar o meio da barra | Move o intervalo de trabalho preservando `out - in` | falta — só as duas alças pegam (`timeline.js:706-708`) | P2 |
| 9 | Grade adaptativa ao zoom | ambos (Ableton §6.2; Resolve, Zoom Slider p.647) | automático | Idem, com um defeito: a grade é uma escada de segundos (`STEPS`, `timeline.js:28`) e o menor degrau é 0,04 s, que não é quadro em nenhum fps do show. O degrau fino tem que ser `1/fps` e seus múltiplos | parcial — `timeline.js:28,530-538` | P2 |
| 10 | Snap liga/desliga por tecla | Premiere `S` (já em `SHORTCUTS.md`) / Resolve p.546 (`N`) | `S` | Idem. **Divergência**: o Resolve usa `N`; ganha o `S` do Premiere, porque `SHORTCUTS.md` já o fixou e `N` não tem parentesco com nada | feito — `timeline.js:907` | — |
| 11 | Segurar um modificador solta a grade durante o arrasto, e o snap volta ao soltar | Ableton (Automação: "Hold `Alt` while dragging horizontally to bypass grid snapping") / Resolve p.546 (o `N` no meio do arrasto é temporário) | segurar `Alt` ao arrastar | Idem. Hoje quem solta o snap é o `Shift` (`timeline.js:753`), e `Shift` já é "estende a seleção" no mesmo canvas: duas coisas na mesma tecla. Passa para `Alt` ("variante", `SHORTCUTS.md`) | errado — `timeline.js:753` | **P1** |
| 12 | Segurar `Shift` afina o valor ao arrastar na vertical | Ableton (Automação: "finer resolution") | segurar `Shift`, arrastar | Arrasto vertical do keyframe passa a andar 1/10 por pixel. Só existe depois que o item 11 liberar o `Shift` | falta — `timeline.js:757-759` | P2 |
| 13 | O playhead e o scrub também grudam nos marcadores | Resolve p.546 ("clip in and out points, markers, and the playhead all snap") | arrastar na régua | `snapT` já existe e já inclui marcadores, In/Out e keyframes visíveis; o scrub simplesmente não o chama | falta — `timeline.js:747` chama `x2t(p.x)` cru; `snapT` em `:460-465` | P2 |
| 14 | Empurrar a seleção quadro a quadro pelo teclado | Resolve p.533 e p.625 (`,` / `.`; `Shift` = 5 quadros) | `,` / `.` e `Shift+,` / `Shift+.` | Move os keyframes selecionados 1 quadro (`1/fps`) ou 5. Hoje não existe jeito de mover um keyframe com precisão: só arrastando com o mouse. As setas continuam movendo o playhead (Premiere), sem conflito | falta | **P1** |
| 15 | Duplicar a seleção arrastando com `Alt` | Resolve/MadMapper (gesto já adotado em `cenas-cues-dmx.md §4`) | `Alt`+arrastar | Copia os keyframes selecionados para onde soltar. Convive com o item 11: `Alt` sozinho no arrasto de keyframe duplica **e** solta a grade; são o mesmo gesto, não dois | falta | P2 |
| 16 | Criar keyframe clicando na curva | Ableton (Automação: "Click on a line segment to create a breakpoint"; duplo-clique no vazio) | duplo-clique na lane | Cria keyframe no tempo do clique com o valor interpolado (`TL.valueAt`), sem mexer no playhead. `Ctrl+K` continua criando no playhead | falta — só `Ctrl+K` (`timeline.js:871-877`) | P2 |
| 17 | Draw mode: desenhar a automação arrastando | Ableton (Automação: `B`) | segurar `B` e arrastar | Escreve um keyframe por degrau da grade enquanto o mouse anda. Vale para chase de DMX; não vale a complexidade antes de tudo acima estar de pé | falta | P3 |
| 18 | Lane de automação por parâmetro, dobrável | Ableton §6.1 (`U`, Fold/Unfold; altura por `Alt++`/`Alt+-`) | `U` no track focado | Já temos uma lane por parâmetro (`timeline.js:190-196`); falta esconder as lanes de parâmetro e deixar só a principal. Com 24 fixtures a lista fica ilegível sem isto | falta | P2 |
| 19 | Altura das faixas | Resolve p.648 (`Shift`+roda) / Ableton (`Alt++`, `Alt+-`) | `Alt`+roda, `Alt++` / `Alt+-` | Muda `TL.rowH` (global, não por track — ver §3). **Divergência**: o Resolve usa `Shift`+roda, que aqui é andar de lado; fica `Alt` | falta — `rowH` fixo em 32 (`timeline.js:34`) | P2 |
| 20 | Faixa de visão geral clicável no topo | Ableton §6.1 (arrastar horizontal rola, arrastar vertical dá zoom, duplo-clique enquadra tudo) | arrastar / duplo-clique na faixa | Uma faixa de ~24 px acima da régua com o show inteiro e o retângulo da janela visível. Duplo-clique = enquadrar tudo | falta | P2 |
| 21 | Zoom por arrastar na régua | Ableton §6.1 | arrastar vertical na régua | **Rejeitado.** Na régua o gesto é scrub (Resolve, e é o que o editor de vídeo espera). Zoom fica em `Ctrl`+roda, `=`/`-` e nos presets do item 22 | (a régua já faz scrub — `timeline.js:704-712`) | — |
| 22 | Presets de zoom: tudo, detalhe, anterior | Resolve p.626 e p.647 (Full Extent / Detail / Custom; `Shift+Z` enquadra tudo e **volta** ao zoom anterior) | `\` ou `Shift+Z`; `Shift+Z` de novo volta | "Tudo" já existe (`TL.fit`). Falta guardar o zoom anterior para o segundo `Shift+Z` — é assim que se sai de um ponto do show, olha o todo e volta para outro ponto. "Detalhe" = zoom em que 1 quadro tem largura clicável, centrado no playhead | parcial — `timeline.js:870,916-919` | P2 |
| 23 | A vista rola sozinha quando o arrasto chega na borda | ambos | arrastar um keyframe até a borda | Idem, na horizontal e na vertical | falta | P2 |
| 24 | J/K/L com aceleração | ambos (Resolve p.545, p.664) | `J` / `K` / `L` | Feito no relógio local, com teto 8× | parcial — `timeline.js:840-844`; com player do engine `J`/`L` viram pause/play (`:357-358`), porque o registry não tem comando de rate | P2 |
| 25 | `K+J` / `K+L` quadro a quadro | Premiere (já na gramática de `SHORTCUTS.md`) | segurar `K`, tocar `J`/`L` | Idem | falta | P3 |
| 26 | Marcador com nome e nota | Resolve p.548 e p.784 (duplo-clique ou `M` de novo abre o diálogo: Time, Name, Notes, Keywords; "markers with custom notes appear with a dot") | `M` cria; `M` no mesmo tempo, ou `Shift+M`, edita | Marcador vira objeto com nome e nota (§3). O ponto do Resolve é a marca certa: informação sem cor decorativa (`PRINCIPIOS.md §2`) | falta, e com bug: `Shift+M` cai no ramo do `M` (`timeline.js:863`, o teste é `kb === "m" && !ctrl`) e cria um marcador em cima do outro em vez de editar | **P1** (o bug) / P2 (o diálogo) |
| 27 | Marcador colorido | Resolve p.5, p.782 (paleta de cores de marcador) | menu do marcador | **Rejeitado.** `PRINCIPIOS.md §2`: cor só significa estado, e `SHORTCUTS.md` já fixou "marcadores são cinza". Quem separa marcador é o nome, que é texto e é buscável (`PRINCIPIOS.md §4`) | — | — |
| 28 | Navegar de marcador em marcador | Resolve p.548 (lá é `↑`/`↓`) | `Ctrl+Shift+←` / `Ctrl+Shift+→` | Feito. **Divergência**: no Resolve `↑`/`↓` andam entre marcadores; aqui `↑`/`↓` andam entre keyframes do track focado (Premiere, edit points), que é o equivalente do "edit" para nós | feito — `timeline.js:825-833,845-850` | — |
| 29 | Arrastar o marcador na régua | Resolve p.548 ("Drag a marker to another frame in the Timeline Ruler") | arrastar | Idem | falta — os marcadores são desenhados (`timeline.js:616-621`) e não têm hit-test | P2 |
| 30 | Track com mute, solo e record arm no cabeçalho, e o estado valendo de verdade | ambos (Ableton: Activator/S/Arm; Resolve p.546) | clique em M / S / R, ou `Shift+D` / `Shift+S` / `R` | Os três botões existem e o `.spell` guarda mute e solo. **O engine Rust não lê nenhum dos dois**: `solo` não existe em `spellcore/`, e o único `mute` que existe é o do **nó do graph** (`spellcore/script/src/graph.rs:200`), outra coisa — em `spellcore/engine/` não há nenhum dos dois. Quem aplica mute de track é só o protótipo Python (`spellcaster/gui/api.py:168-171`). Mutar um track hoje é pintura: o DMX continua saindo | parcial/mentiroso — `timeline.js:507-514,718-730`; engine: nada | **P1** |
| 31 | Cor por track | Ableton (cor de track é livre e os clips herdam) | menu do cabeçalho | **Rejeitado como cor livre.** `PRINCIPIOS.md §2` proíbe cor decorativa. Entra como cor **por família de track** (`dmx`, `laser`, `fx`, `cue`, `media`), derivada do `type`, igual às famílias de nó do graph (`DECISOES.md`, 09/09) — e sem campo novo no `.spell` | falta — tudo desenha em `col.fg`/`col.fg2` (`timeline.js:554,598`) | P2 |
| 32 | Lock de track | Resolve p.546 e p.3635 (clicar e arrastar o cadeado sobre vários cabeçalhos) | `Shift+L` no track focado; arrastar sobre os cadeados | Track travado: keyframe não move, não apaga, não entra em marquee; cabeçalho hachurado. Já decidido que lock é só edição, sem runtime (`DECISOES.md`, 09/09, "Um verbo por conceito") | falta | P2 |
| 33 | Seleção de tempo (intervalo) separada da seleção de objeto | Ableton §6.9 ("Clicking into the Arrangement background selects a point in time… insert marker") | arrastar no vazio da lane / da régua | Hoje o marquee já pega os keyframes do retângulo (`timeline.js:776-790`), que resolve 90% do caso. Falta o intervalo como coisa: alimentar `Ctrl+L` (item 6), apagar/copiar "tudo entre 12 s e 16 s nestes tracks" e mostrar as bordas | parcial | P2 |
| 34 | Keyframe no Inspector, ao lado do valor, aceso quando o playhead está em cima | Resolve p.1035 e p.571 (botão cinza vira laranja, com setas de anterior/próximo) | clique no botão | É o `inner_key` do Blender que `FUNCOES/README.md §4` já fixou, e o âmbar é o nosso accent. Depende de existir painel Inspector (`Shift+7`) | falta — não há Inspector na página (`index.html`) | P3 |
| 35 | Keyframe gravado durante a execução | Resolve p.1040 ("Keyframes can be added directly on clips in the Timeline while playing back") | `Ctrl+K` tocando | Já funciona: `Ctrl+K` não olha o transporte | feito — `timeline.js:871-877` | — |
| 36 | Cue na régua, com rótulo | Resolve (marcador com ação) / Ableton Session View (cena = linha disparada) | — | O track `cue` já existe no engine (`timeline.rs:354,369,393`) e a lista de cues é `cenas-cues-dmx.md`. Aqui: desenhar os keyframes do track `cue` como rótulo (`GO`, nome) em vez de losango, e marcá-los na régua. Zero campo novo no `.spell` | falta — `timeline.js` não sabe o que é cue | P2 |
| 37 | Trim das bordas do clip com `Ctrl`/`Alt` | Resolve p.625 (Trim Edit Mode, ripple/roll/slip/slide) | arrastar a borda | **Não entra agora.** A nossa timeline não tem clip com borda: tem ponto (keyframe). Entra quando o track de laser/vídeo tiver clip com in/out próprio (frentes `gravar-dmx` e `previz`); aí ripple e roll ganham sentido | — | P3 |
| 38 | Tap tempo, quantize, warp, grade em compassos | Ableton | — | **Não entra.** O show é relógio: timecode, `fps` e segundos (`show.fps`, `show.duration`). Compasso só entraria com um track de tempo, e nenhum pedido do Matheus pede isso | — | — |

## 3. Modelo de dados

Muda o `.spell`:

- **Marcador com nome e nota** (item 26). Hoje a GUI grava `markers: [12.5, 40.0]` (números, `timeline.js:198-199,864`) e o engine já tem teste com `markers: [{"t": 1.0, "name": "um"}]` (`spellcore/engine/tests/patch.rs:44-52`). São dois formatos no mesmo campo, e nenhum dos dois é erro hoje porque `show_patch` aceita qualquer JSON (`edit.rs:825`). Recomendação: **objeto** `{t, name, note}`, com número aceito na leitura e convertido (migração local, `FUNCOES/README.md §12`). Sem `color` (item 27).
- **`lock` por track**: `tracks[i].lock: true`. Um booleano; o engine ignora, como já foi decidido para lock. Recomendação: **entra**.
- **Cor por track**: **não entra como campo.** A cor sai do `type` (item 31).
- **Altura de faixa e dobra de lane**: **não entram no `.spell`.** É estado de janela, e `FUNCOES/README.md §12` proíbe estado de janela dentro do show. Vão para o `config.json` da GUI.
- **Loop**: **não entra no `.spell`.** É estado de transporte. Se um dia precisar sobreviver ao fechamento, o lugar é `transport.loop` (o objeto `transport` já existe: `shows/medgrupo.spell`).

Não muda o `.spell`, é só UI: follow, presets de zoom, faixa de visão geral, nudge, duplicar, seleção de tempo, snap do scrub, `Alt` soltando a grade, edge-scroll, arrastar o marcador, desenho de cue na régua.

Não é formato, é runtime: **mute e solo de track precisam valer no player Rust** (item 30). `solo` continua sendo "mute dos outros" calculado no cliente (`DECISOES.md`, 09/09); só `mute` desce para o engine.

As duas mudanças de formato (marcador objeto; `lock`) e as duas recusas (cor por track; altura/dobra no arquivo) estão em `design/DECISOES.md` como **aguarda voto**.

## 4. Plano de implementação

Ordem de execução. A base é a frente `timeline-ux` (layout fixo, roda/Shift/Ctrl, loop no engine, undo local, atalhos). Regra para a frente que implementar: **toda regra nova entra como função pura exportada em `TL`, e o handler de mouse/tecla só a chama** — é a única forma de o teste `node --test` cobrir sem DOM. O harness de `spellgui/web/test/timeline.test.js:13-28` precisa de dois ajustes de uma linha cada: `CK.near` e `CK.sel` reais (hoje são `() => -1` e `() => ({})`), copiados de `canvaskit.js:26-36,58-72`.

**P1**

1. **Follow** (item 4). `TL.follow(t, viewX, zoom, w, gutter)` → novo `viewX`, ou `null` se o playhead está na janela; `frame()` aplica; `onDown`/`wheel`/`zoomAt` põem `TL.followOff = true`, `TL.locate` e `TL.play` limpam. Teste: playhead no meio → `null`; playhead 1 px depois da borda direita → salta uma página (o novo `viewX` deixa o playhead na margem esquerda); playhead antes da borda esquerda → volta.
2. **Loop com player** (item 5). `TL.loopWrap(t, inT, outT, loop)` → `t` corrigido, usado no `frame()` local **e** no ramo do player (hoje `timeline.js:390` devolve antes de olhar o loop): com engine, ao cruzar o Out manda `locate(in)`. Teste: `t > out` com loop → `in`; sem loop → `t`; `out <= in` → `t` (nunca laço infinito).
3. **`Alt` solta a grade, `Shift` afina** (itens 11 e 12). `onMove` (`timeline.js:750-760`): `!p.shift` vira `!p.alt`; `p.shift` passa a dividir o passo vertical por 10. Uma linha e meia, sem teste próprio — o comportamento do snap já é o do item 5 da lista de testes abaixo.
4. **Nudge** (item 14). `TL.moveSel(dt)` → aplica em `TL.lanes` e devolve a lista de edições (`{k:"move", …}`) que vai para `commit`; `onKey` liga `,` / `.` / `Shift+,` / `Shift+.` com `dt = ±1/fps` e `±5/fps`. Teste: monta `TL.lanes` com duas lanes na mão, seleciona um keyframe, chama `TL.moveSel(1/30)`, confere o `ts` novo e o `{k:"move", from, t}` devolvido; e confere que `TL.ops` transforma isso em `key_del` + `key_set` (já coberto por `timeline.test.js:45`).
5. **Snap no scrub** (item 13). `onMove` do modo `scrub` passa por `snapT`. Teste: `TL.snaps = [0, 3, 10]`, `TL.snap = true`, `TL.k.view.zoom = 100`; `TL.snapT(3.02)` → `3`; `TL.snap = false` → `3.02`.
6. **`Shift+M` não cria marcador** (item 26). `onKey`: o ramo do `M` ganha `&& !shift`, e `Shift+M` (e `M` sobre um marcador existente) chama `TL.markerEdit(t)`. Junto vem `TL.markerT(m)` (aceita número ou `{t}`) e `TL.markerList(show)` (normaliza e ordena), usados por `jumpMarker`, pelo desenho e pelo snap. Teste: `TL.markerList({markers:[3, {t:1, name:"pico"}, 2]})` → ordenado, três objetos, `name` preservado; `TL.markerT(3)` → `3`.
7. **Mute valendo no player** (item 30). `spellcore/engine/src/timeline.rs`: track com `"mute": true` não escreve nos universos. Teste Rust `#[test]` ao lado dos que já existem em `timeline.rs:570-590`: dois tracks no mesmo endereço, um mutado, o frame sai com o valor do não-mutado.

**P2**, na ordem: presets de zoom com volta (22) · dobrar lanes de parâmetro `U` (18) · lock de track (32) · altura de faixa (19) · cor por família (31) · marcador com diálogo e arraste (26, 29) · cue na régua (36) · duplo-clique cria keyframe (16) · `Alt`+arrastar duplica (15) · seleção de tempo e `Ctrl+L` na seleção (33, 6) · barra In–Out arrastável inteira (8) · faixa de visão geral (20) · edge-scroll (23) · grade em quadros (9) · `Shift` afinando o valor (12) · J/K/L com rate no engine (24).

Cada um leva um teste do mesmo feitio (função pura + `assert`): `TL.zoomBack()` guarda e devolve o par `{x, zoom}`; `TL.fold(li)` devolve a lista de lanes visíveis; `TL.locked(li)` é consultado por `delSelected`, `onMarquee` e `onMove`; `TL.laneColor(type)` devolve o token; `TL.timeSel(r)` devolve `{t0, t1, lanes}`.

**P3**: draw mode `B` (17) · `K+J`/`K+L` (25) · Inspector com botão de keyframe (34) · trim de clip (37).

## 5. Atalhos

O que este documento acrescenta a `design/SHORTCUTS.md` (nada foi removido de lá; "Ableton" passa a ser origem válida, ao lado de Premiere e Resolve):

| Ação | Tecla | Origem | Conflito |
|---|---|---|---|
| Empurrar a seleção 1 quadro / 5 quadros | `,` / `.` e `Shift+,` / `Shift+.` | Resolve (manual p.533, p.625) | nenhum; as setas continuam movendo o playhead |
| Soltar o snap durante o arrasto | segurar `Alt` | Ableton (Automação) | tira do `Shift`, que é estender seleção |
| Ajuste fino do valor ao arrastar | segurar `Shift` | Ableton (Automação) | só depois da linha acima |
| Duplicar a seleção | `Alt` + arrastar | Resolve / MadMapper (`cenas-cues-dmx.md §4`) | mesmo gesto da linha "soltar o snap" |
| Follow (a vista acompanha o playhead) | `Alt+Shift+F` | Ableton | nenhum; `Shift+F` continua tela cheia |
| Loop na seleção de tempo | `Ctrl+L` com seleção ativa | Ableton (Loop Selection) | nenhum; sem seleção `Ctrl+L` continua ligando/desligando |
| Voltar ao zoom anterior | `Shift+Z` de novo | Resolve (manual p.647) | nenhum; é a mesma tecla, segunda batida |
| Altura das faixas | `Alt` + roda, ou `Alt++` / `Alt+-` | Resolve p.648 (lá é `Shift`) / Ableton | `Shift`+roda aqui é andar de lado |
| Dobrar / desdobrar as lanes do track focado | `U` | Ableton (Fold/Unfold) | nenhum |
| Travar / destravar o track focado | `Shift+L` | (nosso; par de `Shift+D` mute e `Shift+S` solo) | `Ctrl+L` é loop |
| Editar o marcador sob o playhead | `M` de novo (ou `Shift+M`) | Resolve (manual p.548, p.781) | `Shift+M` já estava no mapa; a segunda batida do `M` é nova |
| Criar keyframe na curva | duplo-clique na lane | Ableton (Automação) | gesto |
| Enquadrar tudo na faixa de visão geral | duplo-clique na faixa | Ableton §6.1 | gesto |
| Draw mode (desenhar automação) | segurar `B` | Ableton | P3; entra com o item 17 |
