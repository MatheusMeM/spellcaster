# TouchDesigner — auditoria de interface a partir da instalação real

Auditoria feita só por leitura de arquivo, sem abrir o TouchDesigner. Versão instalada:
build 2025.30000 (rodapé de `Laser_Device_CHOP.htm`; snippets em `OPSnippets/Version.txt` = `990.681`).
Onde a ajuda offline não tem a página, está dito.

Raiz da ajuda offline (`$H` daqui em diante):
`C:\Program Files\Derivative\TouchDesigner\Samples\Learn\OfflineHelp\https.docs.derivative.ca\`
São 2118 arquivos `.htm` — é o wiki inteiro espelhado, não um subconjunto.

## Fontes lidas

Config:
- `C:\Program Files\Derivative\TouchDesigner\Config\PanelShortcuts.txt` (520 B, lido inteiro)
- `C:\Program Files\Derivative\TouchDesigner\Config\TouchShortcuts.txt` (4904 B, lido inteiro)
- `C:\Program Files\Derivative\TouchDesigner\Config\TouchColors` (21105 B, lido por grep: famílias, `parms.*`, `tile.*`, `playbar.*`)
- `C:\Program Files\Derivative\TouchDesigner\Config\MiscColors` (lido inteiro)
- `C:\Program Files\Derivative\TouchDesigner\Config\3DSceneColors` (lido inteiro)
- `C:\Program Files\Derivative\TouchDesigner\Config\opColorPalette.def`, `colorPalette.def` (lidos inteiros)

Python de UI:
- `C:\Program Files\Derivative\TouchDesigner\bin\Lib\TDJSON.py` (745 linhas; lidas 1-40, 160-270, 370-400)
- existem também `bin\Lib\TDFunctions.py` (1186 linhas) e `bin\Lib\TDStoreTools.py` (785 linhas) — não abertos além do `find`

Amostras:
- `C:\Program Files\Derivative\TouchDesigner\Samples\Learn\TouchDesignerTips.txt` (lido inteiro, 74 dicas)
- `Samples\Learn\OPSnippets\Snippets\{CHOP,TOP,DAT,COMP,POP,SOP,MAT}\` (listados: 112 CHOP, 93 TOP, ...)
- `Samples\Palette\` (listado: Generators, ImageFilters, Mapping, MetaQuest, POPs, TDAbleton, TDBitwig, TDSynchro, TDVR, TDVS, Techniques, ThreadManager, Tools, UI, Vive, WebRTC, `defaultUserPalette.json`, `template.tox`)
- `Samples\Palette\UI\Basic Widgets\` (listado inteiro)

Ajuda offline (`$H`), páginas convertidas para texto e lidas:
`Laser_CHOP.htm`, `Laser_Device_CHOP.htm`, `Scan_CHOP.htm`, `Lasers.htm`,
`NDI_In_TOP.htm`, `NDI_Out_TOP.htm`, `NDI.htm`, `DMX_Out_CHOP.htm`,
`Network_Editor.htm`, `Pane.htm`, `Timeline.htm`, `Component_Timeline.htm`,
`Parameter_Dialog.htm`, `Parameter_Dialog_Gadgets.htm`, `Parameter_Mode.htm`, `Value_Ladder.htm`,
`Custom_Parameters.htm`, `Page_Class.htm`, `Par_Class.htm`,
`Operator.htm`, `OP_Create_Dialog.htm`, `Flag.htm`, `Cook.htm`,
`Export.htm`, `Binding.htm`, `Perform_Mode.htm`, `Palette.htm`, `Widgets.htm`,
`Replicator_COMP.htm`, `Application_Shortcuts.htm`, `Keyboard_Shortcuts.htm`,
`Errors_Dialog.htm`, `File_Types.htm`, `Toeexpand.htm`, `Base_COMP.htm`, `Text_DAT.htm`,
`Palette-sceneChanger.htm`.

Não existe / não foi encontrado:
- Não há página "Keyboard Shortcuts" com tabela: `$H\Keyboard_Shortcuts.htm` tem 8 linhas e só aponta para `Application_Shortcuts.htm` e `Panel_Shortcuts.htm`. A tabela real está em `Application_Shortcuts.htm`, que reproduz o conteúdo de `Config\TouchShortcuts.txt`.
- Não existem `presets` nem `keyboardIn` em `Samples\Palette\Tools\` (listagem completa conferida). O único componente de teclado é `Tools\onScreenKeyboard.tox`. `sceneChanger.tox` existe.
- Não há snippet de ILDA/NDI-In: `OPSnippets\Snippets\CHOP\` tem `laserCHOP.tox`, `laserdeviceCHOP.tox`, `etherdreamCHOP.tox`, `dmxoutCHOP.tox`; `TOP\` tem só `ndioutTOP.tox`, não tem `ndiinTOP.tox`.
- `Config\SplashTips\` não tem texto: só `.tif` (logo, botões de olho/cadeado, tipos de licença). O texto de dica vive em `Samples\Learn\TouchDesignerTips.txt`.
- `Config\Help\` tem só `command.help` e a pasta `exprhelp` (não abertos).
- Não há página "Externalize"; a externalização está descrita nos parâmetros da Common page de COMP (`Base_COMP.htm`) e no `Sync to File` dos DATs (`Text_DAT.htm`).

## Anatomia da tela

O TouchDesigner é uma janela com **barra de menu no topo, Timeline no rodapé e Layout no meio**;
o Layout é feito de um ou mais Panes (`$H\Pane.htm`, definição de "Layout" no rodapé da página).

- **Pane**: área de trabalho; 9 tipos (Network Editor, Panel, Geometry Viewer, TOP Viewer, CHOP Viewer, Animation Editor, Parameters, Textport/DATs, Browser) — `$H\Pane.htm`.
- **Pane Bar**: no topo de todo Pane. No meio dela fica o **network path**; à esquerda e à direita, botões de viewer, bookmarks, histórico (back/forward), split, fullscreen e link entre panes (`$H\Pane.htm`). Clicar nos `/` do path navega por menus; clicar no espaço vazio permite digitar/colar um path. Botão direito em qualquer nome do path abre o menu do COMP daquele nível (`TouchDesignerTips.txt:19`, `:31`).
- **Palette**: aberta pelo botão à esquerda das opções de Pane Layout, embaixo do menu File, ou por `Dialogs -> Palette Browser`, ou `ui.openPaletteBrowser()` (`$H\Palette.htm`). Fecha para ganhar espaço (`TouchDesignerTips.txt:13`) — ou seja, **não é permanente**.
- **Parameter Dialog**: fica **à direita da network** ("Parameter dialogs are displayed on the right side of a network"), aparece e some com a tecla `p`, e pode existir em três lugares: dentro do Network Editor, como janela flutuante (RMB no nó → Parameters...) ou como tipo de Pane (`$H\Parameter_Dialog.htm`; `TouchDesignerTips.txt:4`, `:36`). Vários podem ficar abertos ao mesmo tempo via botão Sticky; o de cima é o OP corrente.
- **OP Create Dialog** (o "Tab menu"): abre com `Tab`, duplo-clique no fundo da network, botão `+` na Pane Bar ao lado do path, MMB/RMB nos conectores de entrada/saída, ou RMB num wire (`$H\OP_Create_Dialog.htm`). Tem busca por digitação: digitar `midi` acende em branco todos os tipos que casam. Geradores (0 entradas) aparecem num tom mais escuro da cor da família; filtros no tom mais claro.
- **Timeline** no rodapé: timecode (frames ou beats), campo `fps`, campo `frame`, e transporte Reset / Pause / Reverse Play / Play / Step Back / Step Forward; botões Range Limit (Loop ou Once); campos Start/End (comprimento total), RStart/REnd (working range, desenhado como barra colorida acima do índice de tempo), FPS, BPM, ResetF, T Sig (`$H\Timeline.htm`).
- **Timepath**: a Timeline do rodapé não é global por decreto — ela está *escopada* num Time COMP. Padrão é root (`/`). Botão `S` na mini-timeline de um componente escopa aquele tempo no rodapé, **e a Timeline muda de cor** para a cor daquele Component Time; o botão `[/]` volta pro root, cuja cor é sempre o azul escuro da interface (`$H\Timeline.htm`, `$H\Component_Timeline.htm`).
- **Component Timeline**: mini-timeline no rodapé do network editor de qualquer network que tenha Component Time, com dois botões: `I` (Run Independently) e `S` (Scope) (`$H\Component_Timeline.htm`).

**Perform Mode vs Designer Mode** (`$H\Perform_Mode.htm`):
- Perform Mode renderiza **um único Window COMP** e nada mais; a janela de edição de networks não existe. É otimizado: iniciando direto em Perform Mode, "a memória extra que a interface Designer requer não será usada".
- `F1` entra, `Esc` sai (`Shift+Esc` se o Window COMP tiver 'Close on Escape Key' desligado). `ui.performMode = True` em Python.
- Qual janela é a de perform se define no diálogo **Window Placement**, coluna "Perform Window". "Start in Perform Mode" faz o arquivo abrir já em performance.
- Full-screen exclusive (Windows): só se a janela for borderless e cobrir 100% do desktop em um monitor único/Mosaic. Sem isso, o compositor do Windows engole frames e você vê stutter mesmo rodando 60 FPS estáveis.
- Se o arquivo tem a opção Privacy ligada, **não dá para sair** do Perform Mode.
- Pausar a timeline em Perform Mode é `Shift+Space`, não `Space` (`$H\Perform_Mode.htm`; `TouchDesignerTips.txt:28`, `:52`).
- `F10`/`F9` com o cursor sobre qualquer elemento de UI mostra a network dele — inclusive em Perform Mode (`TouchDesignerTips.txt:7`).

## Objetos e verbos

**Sete famílias** (`$H\Operator.htm`): COMP (contêm networks), TOP (imagem, GPU), CHOP (canais: movimento, áudio, controle, protocolos), POP (pontos na GPU), DAT (texto e tabelas), MAT (materiais), SOP (geometria legada). Regra dura: **"Only operators of the same family (color) can be Wired together."** Referência entre famílias diferentes é feita por **Link** (parâmetro que aponta pra um OP) ou por **Export** (CHOP → parâmetro de qualquer OP).

Dentro de cada família: **generator** = 0 entradas; **filter** = 1+ entradas.

**Criar**: `Tab` → OP Create Dialog → clicar no nome → clicar na network para posicionar. Com `Ctrl` segurado dá pra colocar vários; com `Shift` segurado, os operadores saem **já cabeados em série**, e trocar de família começa um novo ramo (`$H\OP_Create_Dialog.htm`).

**Conectar / editar fiação** (`$H\Network_Editor.htm`, `TouchDesignerTips.txt`):
- Inserir um nó no meio de um wire: RMB no wire ou na saída do nó de origem (`:1`).
- Novo ramo a partir de uma saída já conectada: MMB na saída (`:5`).
- Trocar a entrada: clicar da saída de outro nó em qualquer ponto do wire existente (`:25`).
- MMB no wire mostra o que está passando; rollover mostra origem e destino (`:24`).
- **Wire com tracejado animado = a origem está cozinhando** (`:29`, `$H\Cook.htm`).
- Componentes 3D e componentes 2D de painel têm conectores em cima e embaixo, para hierarquia de parentesco e agrupamento de painéis — **sem fluxo de dados** por eles (`:22`, `:46`).

**Cook** (`$H\Cook.htm`): cozinhar = calcular. O TD não cozinha tudo todo frame. Um nó cozinha se tiver (1) um *pedido* de cook e (2) um *motivo*. Pedido vem de: nó a jusante quer cozinhar; nó que referencia por parâmetro quer cozinhar; um viewer está olhando; o alvo de um export quer cozinhar; `cook()` chamado. Motivo vem de: uma entrada cozinhou (entre outros). O que dispara a cadeia toda são os viewers visíveis, os painéis exibidos, e os nós de **saída** (Touch Out CHOP, OSC Out CHOP, NDI Out TOP, Audio Device Out CHOP). MMB num nó mostra tempo do último cook e contagem de cooks. `Dialogs -> Performance Monitor` mostra o que cozinhou num frame; o componente `probe` da Palette assiste ao vivo.

**Flags** (`$H\Flag.htm`) — estados binários na borda esquerda e inferior do nó, **não são parâmetros, não cozinham e não podem receber export**:
- em todos: Viewer, Viewer Active, Lock (congela o dado em memória e o salva no `.toe`/`.tox`), Bypass (entrada 0 passa direto; num COMP faz tudo dentro dele ser bypassado), Cooking (num COMP, impede o interior de cozinhar), Immune (nó imune ao clone), Current, Selected, Expose, Python.
- em objetos 3D: Render, Display, Pickable.
- em CHOPs: Export.
- em SOPs: Compare, Template.
- Em Table View (`Shift+T`) o conjunto completo de flags aparece como colunas (`$H\Flag.htm`; `TouchDesignerTips.txt:21`).

**Viewer Active** = o viewer dentro do nó fica interativo; com ele desligado, clicar/arrastar no nó move e seleciona o nó. `a` alterna nos nós selecionados; `Alt+a` põe todos em Viewer Active enquanto a tecla estiver pressionada (`TouchDesignerTips.txt:17`, `:69`). Com Viewer Active ligado, o campo de nome no rodapé do nó continua servindo para arrastar/info/menu (`:35`).

**Clone vs Replicator** (`$H\Replicator_COMP.htm`): Clone sincroniza o interior de um COMP com um mestre. **Replicator é o for-loop**: cria e destrói nós ("replicants") conforme as linhas de uma tabela DAT ou o parâmetro Number of Replicants. Nomeia por índice (`item1, item2...`) ou por coluna da tabela. Layout dos replicants em Off/Horizontal/Vertical/Grid. `Incremental Update` cria N replicants por frame (default 1) para não derrubar frames. Se só uma linha da tabela muda, os outros replicants **não são recriados**. Cada replicant pode passar por um callback que ajusta parâmetro, expressão ou modo (`c.par.display.mode = ParMode.EXPRESSION`). Exemplo citado: alimentar o Replicator direto com a tabela do Multi Touch In DAT para criar algo em cada dedo.

## Parâmetros (tipo → widget → gesto)

**Cabeçalho** do Parameter Dialog (`$H\Parameter_Dialog.htm`): topo com OP Type e OP Name; **o fundo do cabeçalho é a cor da família** do operador. Embaixo: Operator Help, Python Class Help, Operator Info, Comment, Clipboard, toggle Python/Tscript, Hide/Show Default Parameters.

**Páginas**: cada operador tem uma ou mais páginas (abas) de parâmetros; algumas próprias do tipo, outras comuns (Common). Selecionando vários OPs, a página escolhida no atual passa a ser a primeira aberta em cada um dos outros — poupa cliques ao percorrer muitos nós (`$H\Parameter_Dialog.htm`). Custom parameters aparecem numa **segunda fileira de páginas** (`$H\Custom_Parameters.htm`).

**Tipos** — a lista canônica é o conjunto de `append*` do `Page_Class` (`$H\Page_Class.htm`):
`Int, Float, XY, XYZ, XYZW, WH, UV, UVW, RGB, RGBA, Str, StrMenu, Menu, File, FileSave, Folder, Pulse, Momentary, Toggle, Python, Header, Sequence, ParGroup` mais as refs de OP: `OP, COMP, Object, PanelCOMP, TOP, CHOP, SOP, POP, MAT, DAT`.
`Par.style` devolve exatamente isso como string: `'Float'`, `'Int'`, `'Pulse'`, `'XYZ'` etc. (`$H\Par_Class.htm`).
Tamanho máximo de um tuplet numérico é 4 (`$H\Custom_Parameters.htm`).

| Tipo | Widget (`$H\Parameter_Dialog_Gadgets.htm`) | Gesto |
|---|---|---|
| Float / Int (size 1) | Single Number with Slider (e a variante com Scale) | digitar no campo; **MMB (ou LMB) segurado sobre o campo ou sobre o rótulo abre o Value Ladder** |
| Float/Int size 2-4 (XY, XYZ, RGB, WH...) | Multiple Numbers na mesma linha | ladder no **rótulo** move os 2/3/4 juntos; ladder no campo move só aquele |
| RGB / RGBA | Color Picker com tupla RGB | idem acima |
| Toggle | Check Box (single e multiple) | clique |
| Menu | Drop Down Menu (single e multiple) | clique |
| Str | Text Box | digitar |
| File / FileSave / Folder | File System Path com botão de abrir arquivo | digitar ou botão |
| OP/TOP/CHOP/... ref | Operator Path com botão "Jump To" | digitar path ou botão para pular pro nó |
| Menu de itens ordenados | Ordered List | — |
| Pulse | botão | dispara e volta: `pulse()` "seta o parâmetro para o valor, cozinha o operador, e restaura o valor anterior"; para tipo Pulse não se especifica valor nem tempo (`$H\Par_Class.htm`) |
| Momentary | botão | tipo distinto de Pulse e de Toggle; `Par.isMomentary` existe ao lado de `isPulse` e `isToggle` (`$H\Par_Class.htm`). **A ajuda offline não descreve a semântica de Momentary em prosa** — só a existência do tipo e do teste. |

**Value Ladder** (`$H\Value_Ladder.htm`, `TouchDesignerTips.txt:16`): segurar MMB (ou LMB) sobre o valor ou o rótulo; abre uma escada com `.01 .1 1 10 100`; ainda com o botão apertado, mover **verticalmente** escolhe o incremento e arrastar **para a direita/esquerda** aumenta/diminui naquele incremento. Sem botão do meio, `Alt+RMB` substitui (`TouchDesignerTips.txt:3`).

**Quatro modos de parâmetro** (`$H\Parameter_Mode.htm`, `$H\Parameter_Dialog.htm`):
clicar no rótulo (ou no `+` que aparece no hover) expande a linha, mostrando **o nome interno do parâmetro** (o nome que scripts usam) e quatro botões quadrados:

| Modo | Cor do botão na doc | Cor real no tema | O que é |
|---|---|---|---|
| Constant | cinza | `parms.const.bg 0.425 0.425 0.425` | valor digitado; default |
| Expression | azul | `parms.expr.bg 0.5 0.7 0.7` (ciano) | expressão Python/Tscript |
| Export | verde | `parms.override.bg 0.35 0.5 0.25` | dirigido por CHOP/DAT |
| Bind | roxo | `parms.bind.bg.enabled 0.77 0.7 1` | ligação bidirecional |

(A doc chama de "blue button for expression"; o `TouchColors` guarda o valor real como um ciano dessaturado. Export no `TouchColors` se chama `override`, não `export`.)

O ponto que interessa: **os quatro valores ficam salvos ao mesmo tempo no parâmetro**. Dá para ter um constante, uma expressão, um export e um bind no mesmo parâmetro e alternar livremente. Se existe expressão/export/bind configurado mas o modo não está selecionado, o botão daquele modo mostra **um quadradinho no canto inferior esquerdo**. Isso é o mecanismo de depuração: pular pra constante para testar um valor e voltar sem perder a lógica. Export só pode ser selecionado se já existe um export chegando.

**Expression vs Export vs Bind** (`$H\Export.htm`, `$H\Binding.htm`):
- Export = CHOP (ou DAT) empurra pro parâmetro, unidirecional. Gesto: Viewer Active no CHOP, arrastar o canal até o nó (esperar ele virar corrente e o painel de parâmetros abrir), continuar arrastando até o parâmetro, soltar, escolher "Export CHOP" (`TouchDesignerTips.txt:11`, `:55` — o `:55` acrescenta que dá pra pousar sobre a **aba da página** para trocar de página no meio do arrasto). Aparece na network como **data link pontilhado cinza com seta**, animada quando cozinha. O Export Flag do CHOP liga/desliga tudo de uma vez.
- Mass export: nomear canais como `caminho/do/op:parametro` e pôr Export Method = `Channel Name is Path:Parameter`. Um só gesto conecta dezenas de parâmetros.
- Expressão = o parâmetro puxa. Desde ~2017 as expressões são compiladas e **o desempenho é igual**; a escolha é semântica: expressão sobrevive a reordenação de canais (export não), permite matemática (`op('null1')[0] * 2 - 1`) e escolha condicional do que referenciar; export permite desligar em bloco e o mass-export.
- Bind = bidirecional, com um **bind master** que guarda o valor de verdade e bind references em cadeia ou muitos-para-um. Masters podem ser célula de tabela, canal de Bind CHOP, Panel Value ou objeto Dependency. Uma referência bind não usa constante/expressão/export. Gesto: **arrastar sempre do master para a referência**.
- `Ctrl+E` sobre uma expressão abre ela no editor de texto (`TouchDesignerTips.txt:2`).
- RMB → Copy Parameter, depois RMB → Paste Parameter Reference cria a referência ou o bind (`:15`). Arrastar de rótulo para rótulo funciona inclusive em parâmetros multi-campo como RGB (`:54`).

**Ajuda de parâmetro**: segurar `Alt` e passar o mouse sobre o rótulo mostra o help do wiki (`$H\Parameter_Dialog.htm`; `TouchDesignerTips.txt:27`). O corpo desse help está em `Config\TDParameterHelp.json` (5,45 MB) — não foi parseado aqui.

**Esquema JSON de parâmetro** (`bin\Lib\TDJSON.py`, `parameterToJSONPar`, linhas 186-190):
```
('name', 'tupletName', 'label', 'page', 'sequence', 'style',
 'size', 'defaultMode', 'default', 'defaultExpr', 'defaultBindExpr',
 'enable', 'startSection', 'cloneImmune', 'readOnly', 'enableExpr', 'help')
```
Se `isNumber`, acrescenta `NUMATTRS = ('min','max','normMin','normMax','clampMin','clampMax')` (`TDJSON.py:22`).
Se `isMenu`, acrescenta `('menuSource',)` ou `('menuNames','menuLabels')` (`TDJSON.py:213-215`).
Na volta (`addParameterFromJSONDict`), **as chaves obrigatórias são só três**: `{'page', 'style', 'name'}` (`TDJSON.py:379`).
Um tuplet vira **um único dicionário** com `size` = comprimento do tuplet (`TDJSON.py:255-257`).
Distinção importante: `min`/`max` são o clamp duro (com `clampMin`/`clampMax` ligando ou não), `normMin`/`normMax` são **só o alcance do slider** (`$H\Custom_Parameters.htm`).

**Convenção de nome** (`$H\Custom_Parameters.htm`): parâmetro nativo é todo minúsculo (`brightness`); parâmetro custom começa com maiúscula e o resto minúsculo (`Divisions`) — se a primeira letra não for maiúscula, **a criação falha com erro**. Sem underscore. Máximo recomendado de 12 caracteres, de preferência 10, "pois ficam ilegíveis quando você abre o parâmetro com o ícone +". Rótulo em Title Case menos preposições: `Rotate to X Axis`.

## Estados

**Cor por família** — `Config\TouchColors:1-14`, valores RGB 0-1:

| Família | RGB | Leitura |
|---|---|---|
| CHOP | 0.385 0.55 0.275 | verde |
| COMP | 0.19 0.19 0.19 | cinza quase preto |
| DAT | 0.575 0.36 0.50 | magenta acinzentado |
| MAT | 0.625 0.58 0.28 | ocre |
| POP | 0.315 0.305 0.75 | azul-violeta |
| SOP | 0.29 0.5 0.7 | azul |
| TOP | 0.41 0.36 0.575 | roxo |

Cada família tem `.hilite` (versão clara para seleção) e algumas `.editbg`. Note que **todas as sete são dessaturadas**, luminância parecida; nada satura. Generator vs filter é o mesmo matiz em dois tons (`$H\OP_Create_Dialog.htm`).

`opColorPalette.def` é outra coisa: são as **24 cores que o usuário pode aplicar a um nó** (tecla `c`), 18 matizes num círculo de saturação idêntica (~0.8 no canal dominante) mais 6 cinzas. `colorPalette.def` são 44 cores para uso geral em 4 fileiras (11 cinzas, 11 escuras, 11 pastéis, 11 saturadas).

**Erro e aviso** (`Config\MiscColors`, `Config\TouchColors`):
- `ErrorFlag 0.9 0.1 0.1`, `WarningFlag 1 1 0`, `MessageFlag 0.8 0.8 0.8`, `FilteredFlag 0.6 0.6 0.6` — "usados para as cores do indicador no botão de info".
- `tile.error 1 0 0`, `tile.warning 1 1 0` — o nó inteiro.
- `parms.err.bg 1 0 0` com `parms.err.fg 0.8 0.8 0.8`; `parms.disabled.err.bg 0.5 0 0` — erro **no campo do parâmetro**, não só no nó.
- `default.error 1 0 0`, `dialog.error 1 0 0`.
- O `Errors Dialog` lista os erros e **clicar leva ao operador ofensor** (`$H\Errors_Dialog.htm` — a página tem uma frase só). O Error DAT registra erros e avisos ao longo do tempo, para caçar intermitente (`TouchDesignerTips.txt:66`).
- Info channels de qualquer operador incluem `warnings` e `errors` como contagem numérica, além de `cook_time`, `total_cooks`, `cooked_this_frame` (`$H\Laser_Device_CHOP.htm`, seção Common Operator Info Channels). Ou seja: **o estado de erro é legível como dado**, não só como pintura.

**Estado no nó** (`Config\TouchColors`, prefixo `tile.`):
`tile.current 0 1 0` (verde puro — o nó corrente), `tile.picked 0.85 0.85 0` (amarelo — selecionado), `tile.connection.hilite1 1 1 0`, `tile.commented 0.3 0.36 0.6`, `tile.ghost 0.3 0.3 0.3`.
Flags coloridas individualmente: `tile.flag.display 0.195 0.49 1` (azul), `tile.flag.render 0.53 0.402 1` (roxo), `tile.flag.export 0.525 0.75 0.375` (verde), `tile.flag.clone 1 0.3 0.3` (vermelho), `tile.flag.clonechild 0.5 0.15 0.15`, `tile.flag.expose 1 0 0`, `tile.flag.hardlock 1 1 0`, `tile.flag.template 0.871 0.377 0.892`, `tile.flag.pickable 0.885 0.557 0.097`, `tile.flag.bypass.cross 0.8 0 0`, `tile.flagv.cloneimmune 0.9 0.45 0`.
Em `MiscColors` há o par ligado/desligado de cada flag, sempre a mesma cor em duas luminâncias: `DisplayOnColor .3 .5 1` / `DisplayOffColor .2 .3 .6`; `BypassOnColor 1 .5 0` / `BypassOffColor .5 .25 0`; `ExposeOnColor 1 0 0` / `ExposeOffColor .6 0 0`; `CurrentColor 0.25 0.85 0.25`.

**Transporte** (`Config\MiscColors`, `TouchColors:473-476`):
`PlayBarOnColor 0.05 0.8 0.05` (verde tocando) / `PlayBarOffColor 0.1 0.1 0.1` (preto parado) / `PlayBarDisabledColor 0.4 0.4 0.4` / `PlayBarResetColor 1 1 0` (amarelo).
Há também um par explícito de **pendente**: `PlayBarPending 0.8 0 0` e `PlayBarNoPending 0 0.8 0` — vermelho quando há mudança não aplicada, verde quando não há. `PendingColor 0.75 0.0 0.0` no bloco de parâmetros diz o mesmo: **mudança pendente é vermelha**.

**Estado de keyframe** (`Config\MiscColors`, com o comentário do próprio arquivo):
`IsKeyColor 0.0 0.55 0.25` ("Keyframe!"), `IsSoftKeyColor 0.0 0.25 0.55`, `IsNotKeyColor 0.80 0.80 0.0` ("tem canal mas não está num key"), `LockedColor 0.75 0.55 0.60`. O cabeçalho do arquivo declara a regra: "as luminâncias da maioria dessas cores devem ser parecidas, para que nenhuma se destaque demais e para que o texto nos campos de parâmetro fique legível".

**Cooking**: não é uma barra de tempo na UI — é o **wire tracejado animado** (`$H\Cook.htm`, `TouchDesignerTips.txt:29`) mais o popup de MMB no nó (tempo do último cook, contagem) e o Performance Monitor. A ajuda offline não descreve nenhuma barra de progresso de cook.

## Atalhos

Fonte definitiva: `Config\TouchShortcuts.txt`, lido na inicialização (`$H\Application_Shortcuts.htm`). Três colunas: label, key, command. `000` = nenhuma tecla definida. Override do usuário: um arquivo com o mesmo formato em `app.preferencesFolder\TouchShortcuts.txt`; para desabilitar um atalho, mantenha a linha e **apague o comando**. Em Perform Mode dá para sobrescrever travando o DAT em `/local/shortcuts`.

Global (`TouchShortcuts.txt`, bloco `general.*`):

| Ação | Tecla |
|---|---|
| Pause/play da timeline | `Space` (`general.pause`) |
| Passo de frame | `←` / `→` (`general.forward` / `general.backward` — os rótulos estão trocados em relação ao movimento descrito em `Application_Shortcuts.htm`, que diz seta direita = avança um frame) |
| Ligar/desligar cooking global | `Ctrl+Space` (`general.cooking`, comando `offon`) |
| F1..F12 e Alt+F1..F12 | reservados como slots nomeados (`F1` é Perform Mode) |

Network editor (`TouchShortcuts.txt`, bloco `network.*`):

| Ação | Tecla |
|---|---|
| Criar operador (Tab menu) | `Tab` |
| Null rápido a partir do nó atual | `Alt+N` |
| Entrar no componente | `Enter` ou `i` |
| Subir um nível | `u` |
| Cancelar | `Esc` |
| Mostrar/esconder parâmetros | `p` |
| Frame all / frame selected | `f` / `Shift+F` |
| Home / home selected | `h` / `Shift+H` |
| Overview (mapa da network) | `o` |
| Modo tabela (Table View) | `Shift+T` |
| Ativar viewers dos selecionados | `a` |
| Toggle Render / Bypass / Display | `r` / `b` / `d` |
| Paleta de cor do nó | `c` |
| Renomear | `n` |
| Estilo de conexão | `s` |
| Mostrar data links (exports) | `x` |
| Editar/expor | `e` |
| Carregar `.tox` | `Shift+X` |
| Comentário / annotate / network box | `Shift+C` / `Shift+A` / `Shift+B` |
| Agrupar / abrir grupos | `Shift+G` / `Ctrl+G` |
| Selecionar tudo | `Ctrl+A`; deselecionar um nó: `Ctrl+clique` |
| Copiar / colar / colar no mouse / recortar | `Ctrl+C` / `Ctrl+V` / `Ctrl+Shift+V` / `Ctrl+X` |
| Apagar | `Del` ou `Backspace` |
| Find / browser / search | `Ctrl+F` / `Ctrl+B` / `Alt+S` |
| Zoom | `Ctrl+=` / `Ctrl+-`; scroll com MMB; box-zoom com `Ctrl`+MMB |
| Scroll da view | `Ctrl+↑↓←→` |
| Histórico da network | `Alt+←` / `Alt+→` |
| Navegar na lista (Table View) | `j` `k` `,` `.` |
| Editar/rodar DAT | `Ctrl+E` / `Ctrl+R` |
| Novo projeto / import / export movie | `Ctrl+P` / `Ctrl+I` / `Ctrl+M` |

Troca de tipo de Pane (`network.switchto.*`): `Alt+1` net, `Alt+2` panel, `Alt+3` geoview, `Alt+4` topview, `Alt+5` chopview, `Alt+6` keyframer, `Alt+7` parm, `Alt+8` opbrowser, `Alt+9` textport.
Panes: `Alt+[` split L/R, `Alt+]` split T/B, `` Alt+` `` fullscreen, `Alt+Z` fechar, `Alt+Shift+C` clonar, `Alt++` / `Alt+-` link.
Diálogos: `Alt+P` preferências, `Alt+L` palette, `Alt+O` operator browser, `Alt+T` textport, `Alt+H` help, `Alt+B` bookmarks, `Alt+C` console, `Alt+F` explorer, `Alt+Y` performance, `Alt+D` MIDI mapper, `Alt+K` license, `Alt+W` window placement.
App: `Ctrl+S` salvar, `Ctrl+Shift+S` salvar como, `Ctrl+O` abrir, `Ctrl+Q` sair, `Ctrl+Z` undo, `Ctrl+Y` redo.

Keyframer (`keyframer.*`): `h`/`Shift+H` home, `Shift+F`/`Shift+V` home horizontal/vertical, `n` nomes longos, `e` escala do handle, `t` amarrar selecionados, `Del` apagar key, `Ctrl+C`/`Ctrl+V` copiar/colar key, `Ctrl+←`/`Ctrl+→` key anterior/próximo, **`Alt+LMB` adiciona key no mais próximo**, **`Alt+MMB` adiciona nos selecionados**, `Ctrl+J` key nos selecionados no playhead, `Ctrl+K` key em todos no playhead, `Ctrl+=`/`Ctrl+-` zoom.

CHOP viewer (`chopviewer.*`): `h` home, `Shift+H`/`Shift+V` adapt horizontal/vertical, `t` time bar, `c` time scroll, `l` labels, `x` extend, `d` dots, `n` handles, `g` grid, `u` units, `e` menu de edição, `s` menu de scope, `p` preciso.

`Config\PanelShortcuts.txt` é uma tabela **diferente e muito menor** (32 linhas): é o conjunto que vale dentro de um Panel/em Perform Mode. Só tem `space`, `left`, `right` mapeados para *nada* (comando vazio = desabilitado) e **`shift.space` → `space`, `shift.left` → `left`, `shift.right` → `right`**, mais `ctrl.s` → `toewrite -s` e os slots F1..F12/Alt+F1..F12. Ou seja: **dentro de um painel, o transporte exige Shift** — de propósito, para o operador não pausar o show ao digitar. `Application_Shortcuts.htm` confirma: "(As teclas são diferentes em Panel Shortcuts.)" e Preferences tem "Enable Playbar Shortcuts" para desligar isso de vez (`TouchDesignerTips.txt:58`).

Mouse no Network Editor (`$H\Network_Editor.htm`): LMB seleciona e arrasta o nó; LMB no vazio **faz pan sem mudar o zoom**; `Shift`+arrasto LMB ou RMB faz box-select; `Ctrl`+clique adiciona à seleção; MMB arrastando controla o zoom; scroll = zoom; `Ctrl`+MMB da esquerda para a direita = box zoom in, direita para esquerda = zoom out; `f` volta ao zoom 1. Zoom para dentro de um COMP até entrar nele, zoom para fora até sair. MMB no nó abre o popup de info.

## Laser e NDI (parâmetros reais)

### Cinco caminhos para laser (`$H\Lasers.htm`)
EtherDream (Ethernet → ILDA), Helios (serial/USB, e IDN sobre Ethernet), ShowNET (Ethernet, DAC embarcado), LaserAnimation Sollinger AVB (via Audio Device Out CHOP num dispositivo AVB de baixa latência; o AVB2ILDA dá 24 bits em X/Y e cor, mascaramento eletrônico de zonas e delay de cor por canal), e Pangolin Beyond (Pangolin CHOP; o Beyond é quem gerencia máscara de segurança e desligamento). Em todos menos Pangolin: `CHOP/POP/SOP → Laser CHOP → Laser Device CHOP`.

### Laser CHOP (`$H\Laser_CHOP.htm`)
Substitui o Scan CHOP, que está marcado DEPRECATED (`$H\Scan_CHOP.htm`). Taxa típica declarada: **10.000 a 96.000 amostras/s**.

Entrada CHOP: canais `x`, `y` obrigatórios; opcionais `z`, `r`, `g`, `b`, `id`. **`id` agrupa pontos numa forma**: id=0 é a primeira forma, id=1 a segunda; sem `id`, cada ponto é solto e desconectado. Qualquer outro canal é tratado como cor e recebe blanking — o que permite projetores com mais diodos que RGB. Canais extras reconhecidos: `lascorner`, `lascornerholdadd`, `lascornerholdlookupfactor`.

**Corner points vs guide points** (novidade de 2025.30000; antes disso todo ponto era corner). Atributo booleano `LasCorner` em SOP/POP, canal `lascorner` em CHOP. Corner define começo/fim de um segmento e leva pontos de hold repetidos em função do ângulo; guide point só ajuda a curva e **é emitido uma vez, nunca repetido**. A repetição é `NumRepeatPoints = HoldAdd + HoldLookupFactor * H`.

Página Laser:
`active`; `source` (menu `sop` / `chop` / `pop`); `sop` / `chop` / `pop` (path); **`outputrate`** — Output Sample Rate, amostras/s, **default 48000; a 60 fps isso dá 800 pares posição+cor por frame**; `swap` (troca eixos X/Y); `xscale`; `yscale`; `rotate`; `camera` (Camera COMP para desenhar um SOP da vista da câmera); `updatemethod` (menu `alldrawn` / `everyframe`); `startpulse` (Frame Start Pulse — insere uma amostra com todas as cores em **-1** no início do frame); `debugchan`; `cornerattr` (default `LasCorner`); `cornerholdaddattr` (default `LasCornerHoldAdd`); `cornerholdfactorattr` (default `LasCornerHoldLookupFactor`).

O `updatemethod` é o parâmetro que descreve o **flicker por excesso de pontos**: se o desenho não cabe num frame na taxa configurada, "o efeito será visível como a imagem do laser piscando". `alldrawn` (default) só pega novos dados quando terminou de desenhar tudo; `everyframe` atualiza sempre.

`debugchan` gera um canal com o **estado de cada ponto**, e essa é uma máquina de estados pronta:
`-1` Frame Start Pulse, `0` Color, `1` Corner Hold Point, `2` Start Point Hold Time, `3` Pre Blank On, `4` Post Blank On, `5` Blanking, `6` Pre Blank Off, `7` Post Blank Off.

Página Scanning: `stepsize` (distância que x,y pode mudar por amostra **enquanto emite cor**); `bstepsize` (Blanking Step Size — a mesma distância **enquanto está apagado**); `mincornerhold` e `maxcornerhold` (o hold do ponto é interpolado linearmente entre os dois **conforme a abertura do ângulo**: 180° dá o mínimo, 0° dá o máximo; se max < min, max é clampado para cima); `cornerholdchop` (CHOP como curva de lookup customizada em lugar da interpolação linear); `closedoverlap` (Closed Shape Overlap, **em milissegundos**, sobreposição de início/fim em forma fechada para a interpolação de cor fechar uniforme).

Página Color — **todos os quatro delays de blanking são em ms**:
`redscale`, `greenscale`, `bluescale`; `preblankon` (espera antes de **desligar** a cor), `postblankon` (espera depois de desligar), `preblankoff` (espera antes de **ligar**), `postblankoff` (espera depois de ligar); `starthold` (Start-Point Hold Time, ms, espera no primeiro ponto de um frame novo); `colordelay` (atraso dos canais de cor na saída, ms); `interpcolors`; `brightnesscurvechop`.

A razão física está escrita na página: "como os espelhos do laser são movidos por motores, o dado posicional enviado provavelmente está adiantado em relação à posição real do espelho — o espelho precisa alcançar o dado. O dado de cor, porém, está em tempo, e o resultado pode ser rabos visíveis nos pontos onde o laser desliga a cor." Os parâmetros de blanking existem para compensar isso.

Nota de compatibilidade explícita: em 2025.30000 sumiram **Start Point Hold Time** e **Input Rate**; a geração de pontos agora acontece sempre a **192000** e depois é resampleada para o Output Rate, o que muda o efeito de `stepsize` e `bstepsize` em arquivos antigos.

O Laser CHOP **não expõe canais de info específicos** — só os comuns de CHOP e de operador (`$H\Laser_CHOP.htm`, seção Info CHOP Channels). Não há canal de "pontos por frame" nem de "kpps efetivo".

**Não há parâmetro de safe zone, nem de max points, no Laser CHOP nem no Laser Device CHOP.** A segurança é externa: o aviso em caixa alta na página ("LASERS ARE DANGEROUS...", exigindo Laser Safety Officer certificado, botão de emergência ao alcance, ninguém na área de projeção, nenhuma superfície reflexiva) e o mascaramento feito pelo AVB2ILDA ou pelo Pangolin Beyond (`$H\Lasers.htm`). Nenhuma página lê "kpps" — a unidade do TD é sempre samples/segundo.

### Laser Device CHOP (`$H\Laser_Device_CHOP.htm`)
Canais de entrada/saída, com faixa declarada: `x` e `y` **entre -1 e 1**; `r`, `g`, `b`, `i` **entre 0 e 1**; `user1`..`user4` opcionais (**`user3` e `user4` não são suportados pelo EtherDream**).

`active`; `type` (menu `etherdream` / `helios` / `shownet`); `device` (menu que **se auto-popula** com Helios e ShowNET conectados; ShowNET atualiza sozinho); `scan` (pulse — varre por Helios; a página avisa que **não pode haver conexão Helios ativa durante o scan, senão ela é fechada pelo processo**); `netaddress` e `port` (EtherDream); `localaddress` (escolhe a NIC quando há várias); `queuetime` + `queueunits` (menu `samples` / `frames` / `seconds`) — "determina o tamanho da fila do buffer de pontos do Helios/EtherDream e o tempo correspondente para drená-la; costuma ser útil reduzir esse valor ao enviar poucos pontos"; `xscale`, `yscale`, `redscale`, `greenscale`, `bluescale`, `intensityscale`.

Blanking aqui é implícito: "acontece quando os canais RGB de entrada são todos zero, **ou** quando Red Scale, Green Scale e Blue Scale são todos zero" — isto é, os três scales em zero funcionam como shutter.

Descoberta de IP do EtherDream é problema conhecido e a solução é o EtherDream DAT (`$H\Laser_Device_CHOP.htm`, `$H\Lasers.htm`).

### Scan CHOP — depreciado, mas é o único caminho raster→vetor documentado (`$H\Scan_CHOP.htm`)
Interessa porque é literalmente NDI→ILDA: converte **TOP** (imagem) em ondas de controle x/y.
Página Scan: `source` (menu `top` / `sop` / `chop`); `rate` (Sample Rate, amostras/s); `swap`; `xscale`; `yscale`; `rotate` (graus); `randomize` (emite as amostras em ordem aleatória — "cria uma imagem difusa e caótica no osciloscópio"); `color` (liga os canais r,g,b); `redscale`, `greenscale`, `bluescale`; `blankingcount` (**contagem de posições apagadas** — entre primitivas de geometria no caso SOP, entre varreduras completas no caso TOP; exige Output Color ligado).
Página TOP: `top`; `width` (colunas a reamostrar); `height` (linhas); `level` (**número de níveis de brilho que cada pixel pode ter**); `limit` (Auto Reduce — reduz linhas e colunas dinamicamente para manter o frame rate de saída constante); `layered` (emite os pixels **em ordem de brilho**, em vez de esquerda-para-direita por linha); `interleave` (menu `sweep` / `evenodd` / `max` — "controla a ordem em que as linhas são emitidas para minimizar o flicker").
Página SOP: `sop`; `vertexorder`; `limitstep` ("quebra saltos longos de x,y em vários saltos incrementais menores"); `stepsize`; `vertexrepeat`; `camera`.
O texto do TD sobre o modo TOP: "a luminância é controlada por quanto tempo cada amostra é 'desenhada' no scope. Como a saída é de baixa banda, opções diferentes de reamostragem e ordenação estão disponíveis para minimizar (ou realçar) o flicker."

### NDI In TOP (`$H\NDI_In_TOP.htm`)
`active`; `name` (**Source Name**, menu de streams descobertos); `extraips` ("por padrão o NDI busca por mDNS, que costuma ser limitado a redes locais" — lista de IPs separados por espaço para fontes fora do alcance de multicast); `bandwidth` (menu **`high` / `low`**, dois valores só); `hwdecode` (só funciona para NDI|HX, que é H264 — "o NDI Out TOP e outras soluções NDI em software só conseguem enviar NDI nativo, não NDI|HX"); `inputpixelformat` (menu `native` / `fixed8`); `inputcolorspace` (menu longo, `automatic` por padrão); `inputreferencewhite`; `grouptable` (DAT com nomes de grupos para filtrar as fontes listadas); `audiobuflen` (segundos — "a saída de áudio é atrasada por esse valor").
**Não há parâmetro de FPS no NDI In.** O frame rate chega como leitura, via Info CHOP: `connected`, `receive_fps`, `num_source`, `queue_size`, `received_frames`, `missed_frames`. Esses seis são o painel de saúde do link.

### NDI Out TOP (`$H\NDI_Out_TOP.htm`)
`active`; `name` (Source Name); `failovername` (formato `MACHINENAME (SourceName)` — para onde os receptores migram se esta fonte cair); **`fps`** — e a nota importante: "o NDI usa o FPS parcialmente como guia para controlar como comprimir os frames. Quanto maior o FPS, mais comprimidos os frames. Enviar a 1 FPS resulta em qualidade de imagem maior do que enviar a 30 FPS"; `lowperformancebehavior` (menu `stallmainthread` / `skipframes` — ou a thread principal trava e cede recurso ao sender, ou ela segue e o sender fica faminto e cai de FPS: **os dois caminhos perdem, e a UI força a escolha**); `outputpixelformat` (menu `fixed8` / `fixed16`); `includealpha`; `grouptable`; `audiochop`; `metadata` (DAT em tabela ou XML); `outputcolorspace`.

### NDI, protocolo (`$H\NDI.htm`)
Vídeo em YUV 4:2:2, comprimido com SpeedHQ (CPU), 8 ou 16 bits, alpha opcional. Sem limite de FPS ou resolução além do hardware. Descoberta por mDNS. Multicast é configurado **fora do TouchDesigner**, pelo NDI Access Manager. Metadata é XML; uma tabela DAT é convertida automaticamente para `<TouchDesignerFormat>`. Recomendação de rede: Jumbo Frames em 9014 bytes resolveu queda de frames em resoluções altas em NIC/switch gigabit.

### DMX Out CHOP — para as cenas/cues (`$H\DMX_Out_CHOP.htm`)
`interface` (menu `serial` / `enttecusbpro` / `enttecusbpromk2` / `artnet` / `sacn` / `kinet`); `format` (menu **`packetpersample`** = cada canal é um endereço DMX, 512 canais por universo; **`packetperchan`** = cada amostra de um canal é um endereço DMX, então 1 canal com 512 amostras = 1 universo, e universos viram canais); `rate` com aviso embutido: "dispositivos DMX512 têm taxa máxima de refresh de 44 Hz. Recomenda-se Rate <= 44".
Página Network: `net` (0-127), `subnet` (0-15), `universe` (0-15) para Art-Net; `multicast` para sACN ("constrói o IP automaticamente a partir de Net, Subnet e Universe"); `netaddress` (default `255.255.255.255` = broadcast); `localaddress` (escolhe NIC); `localport` (-1 = o SO escolhe); `customport` + `netport` (default 6454); `sendartsync` + `artsynctimeout` (ms — espera todos os ArtDmx saírem antes do ArtSync; se estourar o timeout **o ArtSync não é enviado** e um novo frame começa); `cid` (ID único do sender), `source` (nome atribuído pelo usuário, informativo), `priority` (prioridade quando há múltiplas fontes) — os três de sACN.
Existe também uma **Routing Table em DAT**, onde cada linha é um canal e especifica net, subnet e universo.

## Arquivo

Três tipos nativos (`$H\File_Types.htm`): `.toe` = o projeto inteiro (TOuch Environment file); `.tox` = um componente salvo, "para reuso e portabilidade de bibliotecas de componentes"; `.tog` = geometria em formato nativo.

`.toe` é binário, mas **existe conversor oficial para texto**: `toeexpand` expande o `.toe` numa coleção de arquivos ASCII legíveis, e `toecollapse` reverte; ambos em `C:\Program Files\Derivative\TouchDesigner\bin` (`$H\Toeexpand.htm`).

O que o Lock flag guarda: "o dado que o nó emite é congelado em memória **e salvo no `.toe` `.tox`**" (`$H\Flag.htm`). Ou seja, o arquivo carrega dado cozido, não só a descrição da rede.

**Externalização de COMP** — parâmetros da Common page de qualquer COMP (`$H\Base_COMP.htm`):
`externaltox` (path do `.tox` em disco que fornece o conteúdo do COMP na abertura do `.toe` — "permite que componentes contenham redes que podem ser atualizadas independentemente"; se o `.tox` não for achado, carrega o que estava no `.toe`); `enableexternaltox` (ligado por padrão; desligar carrega do `.toe` e deixa o recarregamento manual); `enableexternaltoxpulse` (recarrega agora); `reloadcustom` e `reloadbuiltin` (se os parâmetros de topo voltam ao valor do `.tox` no reload — **os parâmetros dos nós de dentro sempre voltam**); `savebackup` (guarda uma cópia dentro do `.toe` para o caso do `.tox` sumir ou o projeto rodar em outra máquina); `subcompname` (entra no `.tox` e promove um COMP interno a topo); `relpath` (menu: paths relativos ao `.toe`, ao `.tox`, ou herdados do pai).

**Externalização de DAT** — `Text_DAT.htm`: `file` (path); `syncfile` (**Sync to File**: "o arquivo é monitorado, de modo que qualquer mudança feita no arquivo atualiza o DAT, e qualquer mudança feita no DAT é escrita no arquivo imediatamente"; se o arquivo não existe, é criado; se é removido, o DAT mantém o conteúdo); `loadonstart` + `loadonstartpulse`; `write` (Write on Toe Save) + `writepulse`; `language`; `extension` (extensão exposta a editores externos).

**Esquema JSON de parâmetro**: ver a seção de Parâmetros acima. `TDJSON.py` também serializa **blocos de Sequence** separadamente, com `blockParNames`, `numBlocks` e um dicionário `blocks` indexado por string do índice, cada bloco guardando `val`, `expr`, `bindExpr`, `mode`, `readOnly`, `enable` por parâmetro-base (`TDJSON.py:224-248`). `LISTATTRS` (`TDJSON.py:23-27`) é o conjunto de atributos que viram lista quando o parâmetro é um tuplet.

## O que o Spellcaster deve copiar

- **Os quatro modos de parâmetro guardados ao mesmo tempo, com indicador de "tem coisa aqui"** (`$H\Parameter_Mode.htm`). Um valor de cena guarda simultaneamente a constante, a expressão e o binding; alternar não destrói o outro, e o botão do modo não-ativo mostra o quadradinho no canto quando tem conteúdo. Isso é exatamente o que falta numa mesa de luz: poder tirar um fixture do controle do timeline para testar um valor fixo às 23h e devolver depois sem reprogramar. Beneficia **cenas/cues DMX** e o **orquestrador**.
- **A distinção `min`/`max` (clamp duro) versus `normMin`/`normMax` (só o alcance do slider)** (`TDJSON.py:22`, `$H\Custom_Parameters.htm`). Um dimmer tem faixa 0-255 fixa mas o slider útil pode ser 0-180 no ensaio. Duas propriedades, não uma. Beneficia **cenas/cues DMX** e **ILDA player** (kpps tem limite físico do galvo e faixa de trabalho do show).
- **Value Ladder no lugar de slider fino** (`$H\Value_Ladder.htm`, `TouchDesignerTips.txt:16`): segurar e escolher a ordem de grandeza verticalmente, depois arrastar horizontalmente. Resolve o problema real do console — o mesmo campo precisa de precisão de 0.01 e de curso de 100 sem trocar de widget e sem alvo de 3 px. Beneficia **ILDA player** (kpps, escala, rotação) e **NDI→ILDA**.
- **Ladder no rótulo move o grupo, ladder no campo move o componente** (`$H\Parameter_Dialog.htm`). XY, XYZ, RGB. Um gesto, dois alcances. Beneficia **ILDA player** (X/Y scale juntos) e **cenário interativo** (posição de um aparelho na maquete).
- **O canal de debug com estado por ponto** (`$H\Laser_CHOP.htm`, `debugchan`, valores -1 a 7). O Spellcaster deve emitir a mesma coisa: um estado por amostra de saída ILDA (frame-start, cor, corner hold, pre/post blank on, blanking, pre/post blank off). É o que transforma "o traço está com rabo" em diagnóstico. Beneficia **ILDA player**, **NDI→ILDA** e é a matéria-prima do aviso do **Aprendiz**.
- **Os seis canais de info do NDI In como painel de saúde do link** (`$H\NDI_In_TOP.htm`): `connected`, `receive_fps`, `num_source`, `queue_size`, `received_frames`, `missed_frames`. `missed_frames` subindo é a frase do **Aprendiz**; `receive_fps` medido contra o kpps configurado é o cálculo de se o galvo acompanha. Beneficia **NDI→ILDA** e o **Aprendiz**.
- **Erro e aviso como número lido do próprio nó, não só como pintura** (`warnings`, `errors` nos Common Operator Info Channels, `$H\Laser_Device_CHOP.htm`). Se todo nó do Graph expõe contagem de erro e aviso como dado, o painel de estado, o log, a CLI e o **Aprendiz** leem a mesma fonte, e o `spell` por SSH mostra o mesmo que a GUI. Beneficia **orquestrador** e **Aprendiz**.
- **O split entre atalho de aplicação e atalho de painel, com o transporte exigindo Shift dentro do painel** (`Config\PanelShortcuts.txt`, `$H\Application_Shortcuts.htm`). Em Perform Mode, `Space` sozinho está desabilitado e só `Shift+Space` pausa. Isso protege o show de um espaço acidental — e o `SHORTCUTS.md` atual não tem essa camada: `Space` é play/pause em tudo. Vale copiar o par de tabelas: uma para o editor, outra para a Face `performance`. Beneficia **cenas/cues DMX** e todas as funções ao vivo.
- **Desabilitar atalho apagando o comando, não removendo a linha** (`$H\Application_Shortcuts.htm`). O mapa de teclas continua completo e auditável mesmo com teclas desligadas; o override do usuário é um arquivo com o mesmo formato do default. Encaixa direto no `"keys": {...}` do `config.json`. Beneficia todas.
- **Externalização com backup embutido** (`$H\Base_COMP.htm`, `savebackup`). Um `.spell` que referencia módulos externos precisa carregar uma cópia de segurança de cada um, senão o show morre num pendrive que não tem a pasta. E `relpath` explícito: relativo ao show ou ao módulo, escolha declarada. Beneficia **orquestrador** e a portabilidade Windows→Pi.
- **A escada de estados do transporte com "pendente" vermelho** (`Config\MiscColors`: `PlayBarOnColor` verde, `PlayBarOffColor` preto, `PlayBarResetColor` amarelo, `PlayBarPending` vermelho / `PlayBarNoPending` verde). Quatro estados, não dois: parado, tocando, reset, e **mudança não aplicada**. O `PRINCIPIOS.md §2` já pede accent = estado; "pendente" é o estado que falta na lista (armado, ao vivo, GO, ensaio, erro). Beneficia **cenas/cues DMX**.

## O que NÃO copiar

- **A Palette como painel que se abre e se fecha para "ganhar espaço"** (`$H\Palette.htm`, `TouchDesignerTips.txt:13`). Se a biblioteca de módulos precisa sumir para o trabalho caber na tela, o layout está errado. Contraria `PRINCIPIOS.md §3` (nada que muda de lugar sozinho, o operador acha de olhos fechados).
- **O Parameter Dialog em três lugares diferentes ao mesmo tempo** (dentro da network, flutuante, e como Pane — `$H\Parameter_Dialog.htm`), com botão Sticky permitindo vários abertos e "o de cima é o corrente". Três lugares para a mesma informação é três lugares para procurar às 23h. Um Inspector, posição fixa (`SHORTCUTS.md`: Inspector à direita, à la Resolve).
- **Cor de família dessaturada em sete matizes** (`Config\TouchColors:1-14`). Sete cores permanentes na tela, todas ligadas o tempo todo, mata a regra do accent único: quando tudo é colorido, nada é estado. `PRINCIPIOS.md §2` proíbe isso. A informação "que família é este nó" deve ser forma ou rótulo, não cor.
- **Nome de parâmetro limitado a 10-12 caracteres porque a UI não cabe** (`$H\Custom_Parameters.htm`: "mantenha abaixo de 12, de preferência 10, pois ficam ilegíveis quando você abre o parâmetro com o ícone +"). É restrição de layout virando restrição de vocabulário. Contraria `PRINCIPIOS.md §4` — o nome na GUI tem que ser o nome do registry, inteiro.
- **Nome custom obrigado a começar com maiúscula, sob pena de erro na criação** (`$H\Custom_Parameters.htm`). Convenção de capitalização carregando informação semântica (nativo vs custom) é a coisa que quebra quando o usuário digita rápido. Se a distinção importa, é campo, não maiúscula.
- **Duas tabelas de shortcut com o mesmo label significando teclas diferentes** — `general.pause` existe em `TouchShortcuts.txt` e em `PanelShortcuts.txt` com mapeamento diferente. Copiar o *conceito* (contextos separados) sim; copiar a *colisão de nomes* não. Nomeie os contextos no próprio label.
- **Rótulos de atalho que mentem sobre o que fazem**: `TouchShortcuts.txt` traz `general.forward left` e `general.backward right`, enquanto `$H\Application_Shortcuts.htm` diz que a seta direita avança um frame. Um dos dois está errado desde sempre e ninguém corrigiu. Se o mapa de teclas é documentação, ele precisa de um teste que o compare com o comportamento.
- **Timeline global que muda de cor conforme o Component Time escopado** (`$H\Timeline.htm`, `$H\Component_Timeline.htm`). A ideia de escopar o tempo de um subcomponente no transporte principal é boa; usar **cor** para dizer qual está escopado não, porque a cor está reservada para estado de show. O escopo é texto: o Timepath já está lá.
- **Segurança de laser inteiramente fora do software.** Nem o Laser CHOP nem o Laser Device CHOP têm safe zone, limite de pontos ou watchdog; a única barreira é o aviso em caixa alta na página e o mascaramento feito pelo hardware AVB2ILDA ou pelo Pangolin Beyond (`$H\Laser_CHOP.htm`, `$H\Lasers.htm`). O Spellcaster não deve herdar essa omissão: shutter fechado por padrão, blackout que corta de verdade, e limite de zona no engine — não só no DAC de terceiro.
- **Blanking implícito por "todos os scales em zero"** (`$H\Laser_Device_CHOP.htm`). Shutter fechado por coincidência aritmética de três parâmetros de cor é um estado que não aparece em lugar nenhum da UI. Shutter é um estado nomeado, com indicador próprio.
- **Mudança silenciosa de contrato entre versões**: em 2025.30000 o Laser CHOP passou a gerar sempre a 192000 e reamostrar, e sumiram `Start Point Hold Time` e `Input Rate` — arquivos antigos abrem e produzem saída diferente, com um aviso só na página do wiki (`$H\Laser_CHOP.htm`). Um arquivo `.spell` que abre e toca diferente é pior do que um que se recusa a abrir. Versione o formato e falhe alto.
