# Pontos falhos da interface atual — auditoria com evidência

Auditoria da GUI no commit base `dac3e0a`, feita com o engine vivo: `spellcore serve --port 8809 --dir . --show shows/medgrupo.spell`, cinco páginas capturadas em Chrome headless a 1280×800 (e `index.html` e `teatro.html` também a 900×600, porque metade dos defeitos de layout só aparece quando a janela encolhe). Os PNG ficam no scratchpad da sessão, não no repo:

| Página | PNG |
|---|---|
| `spellgui/web/index.html` (timeline) | `png/index.png`, `png/index-900.png` |
| `spellgui/web/patchbay.html` | `png/patchbay.png` |
| `spellgui/web/teatro.html` | `png/teatro.png`, `png/teatro-900.png` |
| `spellgui/web/face.html?face=quatro` | `png/face.png` |
| `spellgui/web/laser.html` | `png/laser.png` |

Cada linha abaixo tem: **o que o operador vê**, **causa no código** (`arquivo:linha`) e **a frente que corrige**. `P1` = o operador tropeça no primeiro minuto. Defeitos que `design/FUNCOES/timeline-daw.md` já levantou (follow ausente, loop que some com o player, `Shift` soltando o snap, `Shift+M` duplicando marcador, mute/solo sem runtime, marcador com dois formatos) **não** se repetem aqui; a lista abaixo é o que sobrou.

## 1. O que o dono apontou

| # | O operador vê | Causa | Frente | P |
|---|---|---|---|---|
| 1 | "não quero abrir no browser, quero uma GUI do programa" | não há crate de janela: `spellgui/` tem só `web/`, e `spellcore/Cargo.toml:3` lista nove membros, nenhum de GUI. A única forma de abrir é `serve` + navegador | `gui-janela` | **P1** |
| 2 | "não tenho como navegar entre as interfaces" | **nenhum `<a href>` entre as cinco páginas.** Grep por `href=` em `spellgui/web/*.html` só acha o `<link>` do tokens.css. Quem abre `face.html` não volta | `gui-janela` | **P1** |
| 3 | "não tenho como mudar o nome do show da interface" | o nome só aparece como texto no rodapé/toolbar (`timeline.js:201-202`). O engine já aceita a edição (`show_patch {ops:[{op:"add",path:"/name",...}]}`, `edit.rs:824`); é a página que não tem campo | `gui-janela` | **P1** |
| 4 | scroll da página faz os menus sumirem; a área de tracks vai para baixo | a barra é `display:flex` **sem `flex-wrap` e sem `overflow`** (`index.html:11-13`), e o `#msg` é texto livre que cresce (`index.html:60`, preenchido em `timeline.js:201`). A 1280 px o texto do nome do show quebra em cinco linhas e a barra vira 90 px de altura, empurrando o canvas; a 900 px os botões `+Track`, `-Track`, `Monitor` e `Salvar` **saem da tela e não há como alcançá-los** (`png/index-900.png`) | `timeline-ux` + `gui-janela` | **P1** |
| 5 | roda do mouse não anda nos tracks | `canvaskit.js:169-173`: roda nua dá zoom, `Shift`+roda rola vertical, nada anda de lado. O pedido é roda = tracks, `Shift` = lateral, `Ctrl` = zoom | `timeline-ux` | **P1** |
| 6 | "não tenho como inputar ou gravar DMX novo ou ILDA ou vídeo" | não há entrada nenhuma: nenhum `drop`/`dragover`/`dataTransfer` em `spellgui/web/*` (grep vazio), nenhuma rota de upload em `serve/src/lib.rs:338-345` (`/commands`, `/show`, `/ws`, `/mcp`, fallback estático), e nenhum comando de gravação no registry (a lista completa de 45 comandos não tem `rec_*`) | `browser-dnd` + `gravar-dmx` | **P1** |
| 7 | "nem como visualizar como essas coisas se mexem no tempo" | o único viewer que existe é o `TL.mon` de 512 barras (`timeline.js:649-667`), sobreposto no rodapé, do universo da lane focada. Nenhum quadro ILDA, nenhum vídeo, nenhuma forma de onda | `previz` + `audio-video` | **P1** |
| 8 | não dá para fazer MIDI mapping | nenhuma página lê `navigator.requestMIDIAccess` (grep vazio). O engine **já tem a metade de baixo**: `input {key}` aceita `"midi:144/60"` (`registry.rs:110-116`) e o nó `in.midi` já escuta essa chave (`script/src/graph.rs:284`). Falta a metade de cima: quem escuta o teclado MIDI e quem grava o par | `midi` + `mapping` | **P1** |
| 9 | o controle de loop ficou bugado | já em `timeline-daw.md` item 5 | `timeline-ux` | — |

## 2. O que esta auditoria achou

| # | O operador vê | Causa | Frente | P |
|---|---|---|---|---|
| 10 | **mutar um track de laser não pega: o M acende e o arquivo continua com `mute: false`** | `timeline.js:308-312`: `commit()` percorre TODAS as lanes e faz `L.spec.mute = L.mute`. Um track de laser tem três lanes sobre o MESMO `spec` (principal, `.rot`, `.scale` — `timeline.js:190-196`), e as lanes de parâmetro carregam a cópia velha (`mkLane`, `:163`). A última lane escrita vence e desfaz o mute. Confirmado rodando: `spec.mute` volta a `false` depois de `TL.commit([])` com a lane principal em `mute: true`. O `show_patch` chega ao engine com `true`, o JSON local fica `false`, e o próximo `TL.reload()` apaga o mute. **Vale igual para `solo`** (`:311`) e para o mesmo caso em `fixture` (`:193`) | `timeline-ux` (a correção é `L.mute` virar leitura de `L.spec.mute`, uma fonte só) | **P1** |
| 11 | `Backspace` apaga o keyframe selecionado quando deveria voltar uma cue | `timeline.js:902`: `key === "Delete" \|\| key === "Backspace"` → `delSelected()`. `SHORTCUTS.md` fixa `Backspace` = "cue voltar", e `FUNCOES/README.md` regra 11 diz "Apagar é `Delete`". São dois donos da mesma tecla no mesmo painel | `timeline-ux` | **P1** |
| 12 | o show que a página mostra não é o que o engine toca | `index.html:63,102` carrega `../../shows/medgrupo.spell` por `fetch` e, no `onopen` do WS, `TL.reload()` busca `/show` (`timeline.js:96,413`): duas cargas em corrida. E o botão **Abrir** (`index.html:63`) troca o show **só na página** — não existe chamada de `load` em `spellgui/web/*` (grep vazio), embora o registry tenha `load {path}` (`registry.rs:201-212`). Pior no `patchbay.html:51`, que nasce apontando para `shows/patchbay_demo.spell` enquanto o engine está com `medgrupo.spell`: o `png/patchbay.png` mostra a página editando um show e o `/show` servindo outro | `comandos` + `gui-janela` | **P1** |
| 13 | tracks que nunca vão sair na rede parecem tracks normais | o engine imprime `aviso: tracks ignorados: laser, pyfx` **no stderr do serve**, e só lá (`Timeline::ignored`, `timeline.rs:404-410`; tipos resolvidos em `:376`). Na tela, `medgrupo.py` (pyfx) e `medgrupo_laser.ild` (laser) desenham igual aos demais (`png/index.png`, três das cinco lanes). O `load` até devolve `"ignored"` (`registry.rs:207-208`) e ninguém lê | `comandos` | P2 |
| 14 | um clipe de laser de 46,8 s aparece como uma lane vazia | `mkLane` só conhece `spec.keys` e `spec.<param>` (`timeline.js:157-181`). O track `{"type":"laser","clip":"medgrupo_laser.ild","fps":30,...}` tem arquivo e duração e a timeline não desenha nem retângulo nem nome de arquivo com extensão nem forma. É o buraco central que `daw-arranjo.md` fecha | `daw-arranjo` | **P1** |
| 15 | três clientes de WebSocket diferentes no mesmo produto | `bus.js` (usado por `patchbay.html:67`, `laser.html:85`, `face.html:49`), o BUS próprio de `timeline.js:64-106` e o BUS próprio de `teatro.js:71`. Os dois últimos têm `ponytail:` dizendo "trocar por bus.js quando ele existir" — ele existe. Um overlay de mapeamento em TODAS as páginas (o pedido `Ctrl+Shift+A`) não tem onde se plugar enquanto forem três | `mapping` (pré-requisito) | **P1** |
| 16 | a página não sabe onde mora o show | `GET /show` chama `show_get {full:true}`, que devolve o `.spell` **sem o caminho** (`registry.rs:221-223`: só o ramo não-`full` passa por `resumo(f, sh)`). Sem isso não há "pasta do show" para onde arrastar mídia, nem caminho relativo para gravar em `clips[]` | `browser-dnd` | **P1** |
| 17 | rolar os tracks passa do fim e a tela fica preta | `canvaskit.js:148` e `:171` limitam `view.y` por baixo (`Math.max(0, …)`) e não por cima. Com 5 lanes de 32 px cabendo em 800 px de canvas, dá para rolar para dentro do vazio e não há indicação de onde voltar. O `png/index.png` já mostra 600 px de nada abaixo da última lane, sem rolagem envolvida | `timeline-ux` | P2 |
| 18 | nome de track é ilegível e não editável | `timeline.js:503` corta em 22 caracteres (`medgrupo_laser.ild.sc…` no `png/index.png`) e não há campo de edição em lugar nenhum: `track_add` aceita `label` (`edit.rs:493`) e depois disso o nome só muda por `show_patch` à mão | `daw-arranjo` | P2 |
| 19 | `Rec arm` acende e não grava | `index.html:70-75` e `timeline.js:910` só viram um booleano de desenho; o próprio `ponytail:` em `index.html:69` admite. Nenhum comando de gravação existe no registry | `gravar-dmx` | P2 |
| 20 | o Patchbay tem 900 px de canvas e desenha a grade em 660 | `png/patchbay.png`: a grade e o retângulo do grupo `raiz` param em y≈670 e sobra uma faixa preta. O canvas é dimensionado pelo `ResizeObserver` do kit (`canvaskit.js:189`) mas o CSS da página não estica o elemento até o rodapé | `patchbay-2` | P2 |
| 21 | o Patchbay pergunta o caminho do show em texto e o resto não pergunta nada | `patchbay.html:51` tem campo de caminho; `teatro.html` e `laser.html` não têm nem isso; `face.html` decide pelo query string. Quatro políticas de "que show é este" em quatro páginas | `gui-janela` | P2 |
| 22 | o TEATRO mostra as cues como planilha, não como grade | `teatro.js:196-246` monta um `<table>` linha a linha, com `cue_go` no duplo-clique (`:241`). É a lista de GO; a grade cenas × tracks do Session View não existe em lugar nenhum | `daw-sessao` | P2 |
| 23 | a Face `quatro` ocupa a tela inteira e não tem saída | `png/face.png`: quatro botões, um rodapé de status, nenhum caminho de volta ao editor. `SHORTCUTS.md` promete `Tab` (editor ↔ performance) e `Esc` (fechar sem sair); `face.js` não implementa nenhum dos dois (grep por `"Tab"` e `"Escape"` em `face.js` vazio) | `gui-janela` | P2 |
| 24 | o painel do laser tem oito sliders e nenhum DAC | `png/laser.png`: `geo/*`, `limit/*`, `safe/*` desenhados e ativos com "sem feed aberto" no lado direito. Parâmetro sem alvo é widget que mente; `ilda-player.md §2` pede o estado "sem dispositivo / procurando / conectado / erro" antes de tudo | `ui-3d` | P2 |
| 25 | não há Inspector em nenhuma página do editor | `SHORTCUTS.md` reserva `Shift+7` para Inspector e `SHORTCUTS.md § Interface` copia o "Inspector à direita" do Resolve. `index.html` não tem painel à direita; `patchbay.html` tem um ("NADA SELECIONADO", `png/patchbay.png`) e ele é só leitura | `patchbay-2` + `daw-arranjo` | P2 |
| 26 | a régua mostra timecode de hora cheia num show de 86 s | `timeline.js:56-57`: `tc()` sempre imprime `hh:mm:ss:ff`, então cada rótulo gasta 11 caracteres para dizer `00:00:10:00`. A escada de grade também é de segundos (`STEPS`, `:28`), sem degrau de quadro | `daw-arranjo` | P3 |
| 27 | o intervalo In–Out do show inteiro parece um risco | `png/index.png`: com In=0 e Out=85,9 a barra cinza de 3 px (`timeline.js:622-623`) atravessa a régua toda e some contra a grade. Não há brace, não há sombreado fora do intervalo | `daw-arranjo` | P3 |

## 3. O que não é defeito e parece

- **`canvaskit.js:181` só redesenha com `dirty`.** Certo: é o que segura 60 fps com show grande. Quem esquecer `k.dirty = true` numa alteração nova é que quebra.
- **`serve` entrega a raiz do repo inteira em 127.0.0.1** (`serve/src/lib.rs:99-101`, com `ponytail:` declarado). Não entra na lista porque é limite conhecido e escrito, e porque `POST /files` (`browser-dnd.md §5`) vai mexer exatamente aí — a frente que abrir escrita fecha a leitura junto.
- **`face.html` sem Inspector.** É o desenho: `ilda-player.md §3` fixa "Face performance: sem Inspector".

## 4. Resumo por frente

| Frente | Itens |
|---|---|
| `gui-janela` | 1, 2, 3, 4, 12, 21, 23 |
| `timeline-ux` | 4, 5, 10, 11, 17 |
| `daw-arranjo` | 14, 18, 25, 26, 27 |
| `daw-sessao` | 22 |
| `browser-dnd` | 6, 16 |
| `mapping` | 8, 15 |
| `audio-video` | 7 |
| `previz` | 7 |
| `gravar-dmx` | 6, 19 |
| `patchbay-2` | 20, 25 |
| `comandos` | 12, 13 |
| `ui-3d` | 24 |
