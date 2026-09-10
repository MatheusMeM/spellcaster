# Ableton Live 12 — digest do manual oficial (fonte para a timeline do Spellcaster)

Digest do **manual**, não do app: o Live não está instalado nesta máquina e não foi aberto.
Cada afirmação leva a página do PDF (`p.N`). O que não foi lido não está aqui.

## Fontes lidas

| Arquivo | O que é |
|---|---|
| `<scratchpad>/fontes/live12-manual-en.pdf` (96,7 MB, 1009 páginas) | "Ableton Live 12 Manual", baixado de https://cdn-resources.ableton.com/resources/manuals/live12-manual-en.pdf em 09/09/2026 |
| `<scratchpad>/fontes/live12-manual-en.txt` (1,47 MB) | texto extraído com PyMuPDF, um bloco `=== p.N ===` por página |
| `<scratchpad>/fontes/live12-manual-en.toc.txt` (42,5 KB) | sumário embutido do PDF, com número de página por seção |

`<scratchpad>` = `C:\Users\email\AppData\Local\Temp\claude\D--DRIVE-MNDS-...\6b3d20de-.../scratchpad`.
O PDF e o TXT **não entram no repositório** — ficam no scratchpad da sessão.

**Numeração de capítulo.** O pedido original citava capítulos de uma edição anterior. A numeração real do
Live 12 é outra e é a usada aqui:

| Pedido original | Capítulo real no Live 12 |
|---|---|
| 3.1-3.2 Control Bar / Status Bar | 3.1 (p.33) e 3.2 (p.34) — coincide |
| cap. 4 e 5 Browser | 4 "Working with the Browser" (p.60); o 5 é "Managing Files and Sets" (p.120), não é browser |
| cap. 6 Arrangement View | 6 (p.160) — coincide |
| cap. 7 e 8 Session/launch | 7 "Session View" (p.182), 8 é "Clip View" (p.195); lançamento de clip é o **cap. 16** (p.355) |
| cap. 17 Recording | 19 "Recording New Clips" (p.405) |
| cap. 19 Automation | 25 "Automation and Editing Envelopes" (p.491) |
| cap. 20 Clip envelopes | 26 (p.504) |
| cap. 21 Video | 27 (p.517) |
| cap. 27 MIDI and Key Remote | 33 (p.836) |
| cap. 36 Keyboard shortcuts | 41 "Live Keyboard Shortcuts" (p.984) |

**Lido**: p.33-35, p.60-63, p.83-84, p.115-119, p.160-181, p.182-194, p.355-362, p.405-415,
p.491-503, p.504-514, p.517-522, p.836-844, p.984-1004.
**Não lido**: os capítulos de áudio/MIDI/devices (9-15, 17, 18, 20-24, 28-32), Push (34, 35),
sincronização (36), acessibilidade (40). Nada deste digest fala deles.

---

## 1. Control Bar e Status Bar (3.1-3.2)

A Control Bar é a única barra fixa do Live e está dividida em **nove seções** (p.33-34), nesta ordem:

1. **Browser Options** — toggle mostrar/esconder browser + Browser Config Menu (expandir para altura total, mostrar Tuning e Groove Pool) (p.33).
2. **Tempo Settings and Metronome** — Link, tempo, fórmula de compasso, metrônomo, Tempo Follower (p.34).
3. **Scale Settings** — reflete a escala do clip selecionado; muda o clip selecionado e todos os criados depois (p.34).
4. **Follow and Arrangement Position** — o toggle Follow e a posição atual do Arrangement (p.34).
5. **Transport Controls** — play/stop e o start da gravação de Arrangement (p.34).
6. **Automation and Capture MIDI** — MIDI overdub, *arm* de automação, **re-enable automation** para parâmetros sobrescritos, Capture MIDI, e o botão de Session record (p.34).
7. **Arrangement Loop Settings** — ativa e configura o loop e os pontos de punch-in/punch-out (p.34).
8. **MIDI and CPU Settings** — Draw Mode, Computer MIDI Keyboard, **Key e MIDI map modes**, sample rate, CPU (p.34).
9. **View Selector** — alterna Session ↔ Arrangement (p.34).

A **Status Bar** (p.34-35) mostra erro e aviso; ao editar MIDI mostra localização, altura, velocity e
probabilidade da nota selecionada; **ao passar o mouse sobre um insert marker no Session ou no
Arrangement mostra a posição exata do marcador** (p.35). Ou seja: a barra de status é o display numérico
de precisão do que o mouse está tocando, não um lugar de mensagens só.

**Para o Spellcaster**: as seções 6 e 8 são as que faltam na nossa barra — um lugar fixo para
*arm de automação*, *re-enable automation*, e os toggles de **key map / MIDI map**. A seção 9 é a
troca de Face.

---

## 2. Browser e drag and drop (cap. 4)

### Layout (p.60-61)

Oito elementos numerados no manual:

1. **Sidebar** com as seções Collections, Library e Places e seus rótulos (p.60).
2. **Browse Back / Browse Forward** — histórico de estados de busca e navegação (p.60).
3. **Search field** (p.61).
4. **Show/Hide Filter** + Filter View menu (quais grupos de filtro aparecem; toggles de Tag Editor, Quick Tags, Auto Tags) (p.61).
5. **Filter View** — grupos de filtro com tags (p.61).
6. **Results bar** — aparece ao buscar/filtrar; tem o botão **Add Label**, que salva o resultado como rótulo customizado, e mostra quantos filtros estão aplicados (p.61).
7. **Content pane** (p.61).
8. **Preview tab** — desenha a forma de onda de samples e clips e toca quando o Preview está ligado (p.61).

Redimensionamento: arrastar a linha divisória do meio muda a proporção entre os dois painéis;
arrastar a borda direita ou a inferior aumenta o browser inteiro — e **arrastar a borda inferior fecha
automaticamente o Info View e o Clip/Device View** (p.61). Existe uma opção `Full-Height Browser` no
menu View que expande até o fundo **sem** fechar Info View e Clip/Device View (p.61).

### Content pane (p.62-63)

Lista dos itens do rótulo selecionado ou do resultado de busca. A coluna Name é sempre visível; outras
colunas vêm do **Content Options menu** (p.62). Colunas são reordenáveis por arraste do nome (p.63);
clicar no nome ordena ascendente/descendente (p.63); botão direito no nome também abre o Content
Options (p.63). Ali também se liga/desliga a exibição de extensões (p.63).

Busca: os termos são combinados com **E**, não OU — "electric bass" acha o que tem os dois (p.63).
`Ctrl+F` / `Cmd+F` muda para o rótulo "All" e põe o cursor na busca (p.63).

### Places (p.83-84)

Rótulos: **Packs** (Core Library + packs instalados + updates), **Splice**, **Cloud**, **Push**,
**User Library**, **Current Project** (todos os arquivos do projeto aberto), **User Folder** (pasta
qualquer do disco adicionada ao browser) e **Add Folder…** (p.83-84).

### Navegação (p.115)

- Rolar: setas ↑/↓, roda do mouse, ou **arrastar segurando `Ctrl+Alt` / `Cmd+Option`** (p.115).
- Abrir/fechar pastas e mover entre sidebar e content pane: setas ←/→ (p.115).
- Por padrão abrir uma pasta fecha a anterior; segurar `Ctrl`/`Cmd` mantém as duas abertas (p.115).

### Preview (p.115-118)

O toggle Preview fica ao lado da Preview Tab, no rodapé do browser (p.115). Com ele ligado, selecionar
um arquivo já toca; ↑/↓ anda entre arquivos ouvindo (p.116). **Mesmo com o toggle desligado**,
`Shift+Enter` ou seta → prevê o item selecionado (p.116). Durante o preview aparece a forma de onda na
Preview Tab e dá para clicar na scrub area para pular (p.116) — **não dá para fazer scrub em clip salvo
com Warp desligado** (p.116). O botão **Raw**: desligado (padrão), o Live toca o arquivo no início do
próximo compasso e em loop, sincronizado ao projeto; ligado, toca no tempo original, sem loop e sem
scrub (p.117). Volume do preview: knob Preview/Cue da Main track (p.117); com 4 saídas dá para prever
no fone enquanto a música continua na saída principal (p.118).

### O que cada drop cria (p.118-119)

- **Arrastar para uma track** (Session ou Arrangement) põe o item nela (p.118).
- **Arrastar para o espaço à direita das tracks do Session, ou abaixo das tracks do Arrangement, cria uma track nova** com o item dentro (p.118).
- **No Session**, duplo-clique ou `Enter` sobre um *device* do browser o carrega na track selecionada; sobre um *sample*, carrega num **Simpler** se a track é MIDI, ou **num clip slot** se a track é de áudio (p.118-119).
- **No Arrangement**, duplo-clique ou `Enter` sobre device ou sample carrega na track selecionada (p.119).
- Arquivos também podem ser soltos direto do Explorer/Finder (p.119).
- Arrastar **instrumento ou MIDI effect** para a *Mixer Drop Area* abaixo das tracks do Arrangement cria uma **track MIDI**; arrastar **audio effect** cria uma **track de áudio** (p.162).
- Arrastando **vários clips** de uma vez: por padrão o Live enfileira todos numa track só (vertical no Session, horizontal no Arrangement); segurar `Ctrl`/`Cmd` **antes de soltar** espalha em várias tracks (p.190). Vale para áudio e MIDI cru, **não** para Live Clips, porque estes podem trazer devices embutidos (p.190).

---

## 3. Arrangement View (cap. 6)

### Layout (p.160-162)

Numerado no manual, 16 elementos. Os que importam:

1. **Overview** — mostra o arranjo inteiro; o contorno preto é a parte visível. **Arrastar horizontal rola; arrastar vertical dá zoom**; duplo-clique dentro do contorno volta ao arranjo inteiro (p.160-161).
2. **Beat-time ruler** — tempo em compasso-batida-semicolcheia; arrastar horizontal rola, arrastar vertical dá zoom; **duplo-clique dá zoom na seleção atual, e sem seleção volta para o arranjo inteiro** (p.161).
3. **Scrub area** — clicar dispara playback dali; **segurar o botão do mouse num ponto faz aquele trecho tocar em loop, na quantização global** (p.161).
4. **Locators** — marcadores lançáveis, adicionáveis em qualquer ponto da scrub area, para organizar a peça em seções disparáveis (p.161).
5. **Set Locator** — cria locator durante playback ou gravação; quando um locator está selecionado o mesmo botão vira **Delete Locator** (p.161).
6. **Previous / Next Locator** — o salto entre locators é quantizado pela quantização global de launch (p.161).
7. **Automation Mode toggle** — mostra/esconde as automation lanes (p.161).
8. **Lock Envelopes** — trava os envelopes na posição da música em vez de no clip; permite mover clips sem mover a automação (p.161).
9. **Main lane** de cada track, onde ficam os clips (p.162).
10. **Arrangement Track Controls** — volume, pan, I/O; quais aparecem é escolhido no submenu Arrangement Track Controls do menu View (p.162).
11. Tracks empilhadas verticalmente, reordenáveis por arraste acima/abaixo (p.162).
12. **Mixer Drop Area** abaixo das tracks (ver §2) (p.162).
13. **Optimize Height / Optimize Width** — encaixam todas as tracks na altura ou largura atual; teclas `H` e `W` (p.162).
14. **Waveform Vertical Zoom Level** — slider que aumenta só o desenho da forma de onda, sem mexer no gain; vale para todas as tracks de áudio e para clips novos gravados depois (p.162).
15. **Time ruler** — segunda régua, em minutos-segundos-milissegundos; arrastar rola (p.162).
16. **Mixer** — abre pelo menu View ou pelo toggle no canto inferior direito (p.162).

Duas réguas de tempo simultâneas (compasso e relógio) é o detalhe que o Spellcaster precisa: um show
tem cue em tempo de relógio e keyframe em quadro.

### Navegação e zoom (p.163)

- Zoom progressivo em torno da seleção: teclas `+` e `-`, ou **roda do mouse com `Ctrl`/`Cmd`** (p.163).
- **Pan do display: arrastar segurando `Ctrl+Alt` / `Cmd+Option`** (p.163).
- `Z` = zoom completo na seleção de tempo; `X` volta um passo de zoom, e pode ser pressionado várias vezes para desfazer vários `Z` (p.163).
- Selecionar tempo num clip do Arrangement **faz o editor do Clip View dar zoom no mesmo tempo** (p.163).
- **Zoom vertical de uma track: roda do mouse com `Alt`/`Option` dentro da main lane.** Se houver seleção de tempo, todas as tracks com conteúdo selecionado dão zoom vertical juntas (p.163).
- **Follow**: liga na Control Bar ou no menu Options. **Pausa sozinho** se você editar, rolar horizontalmente, ou clicar na beat-time ruler; **volta** ao parar/reiniciar o playback ou ao clicar no Arrangement ou na scrub area do clip (p.163).

### Transporte (p.163-165)

Play/Stop na Control Bar ou barra de espaço (p.163). O **insert marker** azul piscante determina onde
o playback começa (p.164). Clicar em qualquer ponto de uma track move o insert marker (p.164).
Duplo-clique no Stop, ou `Home`, devolve o insert marker ao início (p.164). `Shift+Space` continua de
onde parou em vez de voltar ao insert marker (p.164). Os campos de Arrangement Position aceitam arraste
vertical, digitação + `Enter`, e setas ↑/↓ — e mexer neles move o insert marker (p.164).

**Permanent Scrub Areas** é padrão ligado (Display & Input Settings): clicar em qualquer ponto da scrub
area toca dali (p.164). O salto entre pontos é quantizado pela quantização da Control Bar (p.165).
Mesmo com a opção desligada, `Shift`+clique na scrub area ou na beat-time ruler ainda faz scrub (p.165).

### Locators (p.165-167)

- **Set Locator** funciona com transporte rodando (quantizado) e parado (cria no insert marker ou no início da seleção) (p.166). Também existe "Add Locator" no menu de contexto da scrub area e no menu Create (p.166).
- Pular: clicar no locator, ou Previous/Next. Depois do primeiro/último locator, os botões pulam para o **início/fim do arranjo** (p.166).
- **Locators podem ser disparados por MIDI/key mapping** (p.167).
- Com o transporte parado, **duplo-clique num locator seleciona e começa a tocar dali** (p.167).
- Mover: arraste ou setas. Renomear: `Ctrl+R`/`Cmd+R` (p.167). Existe **info text** próprio por locator (Edit Info Text) (p.167).
- **Loop to Next Locator** no menu de contexto faz loop entre dois locators com um comando (p.167).
- **Set Song Start Time Here** sobrepõe o comportamento padrão "playback começa na seleção": com ele marcado, o playback começa naquele locator (p.167).

### Loop brace (p.169-170)

Toggle na Control Bar. Sem seleção, a brace cobre o arranjo inteiro (p.169). Campos numéricos
**Loop Start** e **Loop Length** na Control Bar (p.169). `Ctrl+L`/`Cmd+L` = Loop Selection: liga o loop
e põe a brace na seleção de tempo atual; com clip ou tempo selecionado a mesma tecla também liga/desliga
o loop (p.169).

Ajuste da brace (p.170):

- `←` / `→` — empurram a brace pelo passo da grade;
- `↑` / `↓` — deslocam a brace de um comprimento de loop;
- `Ctrl`+`←`/`→` — encurtam/alongam pelo passo da grade;
- `Ctrl`+`↑`/`↓` — dobram/dividem o comprimento;
- arrastar a borda esquerda/direita muda início/fim; arrastar a barra move sem mudar o comprimento.

O menu de contexto da brace também tem **Set Song Start Time Here** (p.170).

### Editar clips

**Mover e redimensionar** (p.170-171): arrastar o clip muda posição e track; arrastar a borda
esquerda/direita muda o comprimento. **Só a barra do clip é arrastável — não dá para arrastar pela
forma de onda ou pelo desenho MIDI** (p.171). Clips grudam na grade **e** nas bordas de outros clips,
locators e mudanças de fórmula de compasso (p.171). Para deslizar o conteúdo dentro do clip:
`Ctrl+Shift` (Win) arrastando a forma de onda; para ignorar a grade nesse gesto, `Ctrl+Alt+Shift` (p.171).

**Fades e crossfades** (p.172-174): as alças ficam nas bordas dos clips de áudio e **só aparecem se a
track estiver alta o bastante**; se a track está dobrada ou pequena, é preciso aumentar a altura (p.172).
Com Automation Mode ligado, **segurar `F`** mostra momentaneamente os controles de fade enquanto o mouse
está sobre uma automation lane (p.172). Fade In Start / Fade Out End mudam a duração sem mexer no pico;
a borda não passa do pico; a **Fade Curve handle** molda a curva (p.172). **Fades são propriedade do
clip, não da track, e são independentes dos envelopes de automação** (p.174).

**Seleção** (p.174-175):

- clicar num clip seleciona o clip;
- clicar no fundo põe um **insert marker**, movível com ←/→ (tempo) e ↑/↓ (entre tracks). **`Ctrl`+←/→ faz o insert marker pular para locators e bordas de clip** da(s) track(s) selecionada(s) (p.174);
- clicar e arrastar seleciona um intervalo de tempo;
- para chegar ao tempo *dentro* de um clip, é preciso desdobrar a track (p.174);
- arrastar dentro da forma de onda seleciona tempo dentro do clip (p.174);
- **clicar na loop brace equivale a Select Loop** (seleciona tudo dentro do loop) (p.175);
- `Shift`+clique estende a seleção na mesma track ou entre tracks; `Shift`+setas estende/encurta (p.175);
- **`0` desativa a seleção** de material, mesmo com vários clips; com o cabeçalho de track selecionado, `0` desativa a track (p.175);
- `R` inverte a seleção de áudio (não funciona se houver clip MIDI na seleção) (p.175);
- ←/→ empurram a seleção (p.175).

**Fold, altura, unfold** (p.174): o botão de desdobrar fica ao lado do nome da track; `U` desdobra as
tracks selecionadas. A altura muda arrastando a linha divisória abaixo do botão, ou por
`Alt`+`+` / `Alt`+`-`. Segurar `Alt` e usar pinça no trackpad também redimensiona.
**Segurar `Alt` enquanto redimensiona uma track redimensiona todas.** `Alt`+clique no botão de
desdobrar, ou `Alt+U`, desdobra todas (p.174).

**Grade e snap** (p.175): a grade pode ser **zoom-adaptive ou fixa**; a largura das duas se define pelo
menu de contexto da main lane ou do MIDI Note Editor.
`Ctrl+1` estreita (dobra a densidade), `Ctrl+2` alarga, `Ctrl+3` alterna tercinas, `Ctrl+4` liga/desliga
o snap, `Ctrl+5` alterna fixa/adaptativa. O espaçamento atual aparece **acima da time ruler, no canto
inferior direito** (p.175). **`Alt` (Win) / `Cmd` (Mac) segurado durante uma ação ignora o snap; com a
grade desligada, o mesmo modificador liga o snap temporariamente** (p.175).

**Comandos "…Time"** (p.176): ao contrário de Cut/Copy/Paste normais, agem sobre **todas as tracks**,
inserindo e removendo tempo, e afetam os marcadores de fórmula de compasso no trecho.
Cut Time, Paste Time, Duplicate Time, Delete Time, Insert Silence.

**Split** (p.176): clicar dentro da forma de onda / desenho MIDI e `Ctrl+E`, ou o item Split no menu de
contexto, divide o clip ali. Arrastando sobre uma faixa e usando o mesmo comando, isola aquele trecho
como clip novo.

**Consolidate** (p.177): junta o material selecionado de vários clips adjacentes num clip novo; funciona
por track e através de várias tracks. Ao consolidar áudio, **cria um sample novo por track**, gravando a
saída do motor de warp **antes** da cadeia de efeitos e do mixer — o sample carrega atenuação de clip,
warp, pitch e clip envelopes, mas **não** os efeitos (p.177). Os samples vão para
`Samples/Processed/Consolidate` do projeto (p.177).

**Linked-track editing** (p.178-180): tracks podem ser ligadas por "Link Tracks" no menu de contexto do
cabeçalho, inclusive dentro de um Group Track (p.178). Cada track só pertence a uma instância de link
(p.179); passar o mouse no indicador destaca as tracks ligadas; clicar no indicador seleciona todas
(p.179). O que passa a valer simultaneamente (p.179-180): mover e redimensionar clips; selecionar clips
e tempo; comandos "…Time"; split e consolidate; criar e editar fades de áudio (**só fades que começam na
mesma posição de tempo**); armar e desarmar tracks; renomear/inserir/apagar take lanes.

**Cores**: no Arrangement o manual não descreve cor de track; no Session, o menu de contexto do clip e o
do scene têm **paleta de cor** (p.184, p.186), e o browser aceita `1`…`7` para colorir itens e `0` para
resetar (p.1002).

---

## 4. Session View e lançamento de clips (cap. 7 e 16)

### Grid, slots, scenes (p.183-191)

- Cada clip do Session tem um **botão triangular** na borda esquerda. Clicar lança; ou pré-selecionar clicando no nome e lançar com `Enter`; depois anda-se pelos vizinhos com as setas (p.183).
- O **Clip Stop** quadrado para o clip rodando, seja no slot, seja no Track Status field abaixo do grid (p.183).
- `0` desativa o(s) clip(s) selecionado(s) (p.183).
- Clips podem ser mapeados **a faixas de notas MIDI para tocar cromaticamente** (p.183; como fazer: p.841).
- O layout dos clips **não determina a ordem**: o grid é acesso aleatório (p.183).
- Mesmo com todos os clips parados, o Play da Control Bar continua aceso e os campos de posição continuam correndo — o tempo musical é contínuo, independente do que os clips fazem (p.183). Dois cliques no Stop voltam a 1.1.1 e param tudo (p.184).
- Renomear: menu Edit ou de contexto; **vários clips de uma vez**; existe **info text** por clip; e o menu de contexto tem paleta de cor (p.184).
- Reordenar por arraste; seleção múltipla adjacente com `Shift`, não adjacente com `Ctrl` (p.184).
- **Group Track slots** mostram área sombreada quando alguma track do grupo tem clip naquela scene, com a cor do clip mais à esquerda; o slot de grupo tem botão de launch que dispara todos os clips, e vira stop quando não há clips (p.184).
- **Tracks do Session são redimensionáveis em largura** arrastando a borda do título, até sobrar só o botão de launch; `Alt` enquanto redimensiona redimensiona todas (p.185).
- `0` sobre um cabeçalho de track desativa a track (p.185).
- **Scene** = linha horizontal. O Scene Launch fica na coluna mais à direita (Main track). "Cancel Scene Launch" existe no menu de contexto da Main track (p.185).
- **A scene abaixo da lançada é selecionada automaticamente como a próxima**, a menos que "Select Next Scene on Launch" esteja Off (p.185).
- Renomear scenes com `Tab` para pular para a próxima (p.185-186); info text e paleta de cor por scene (p.186).
- Arrastar seleção de scenes **não adjacentes** colapsa elas juntas; para mover sem colapsar, `Ctrl`+↑/↓ (p.186).
- **O número da scene é posicional**: mover a scene muda o número (p.186).
- **Scene Tempo e Scene Time Signature** aparecem arrastando a borda esquerda do cabeçalho da Main track; escondidos por padrão. O projeto se ajusta a eles ao lançar a scene (p.186). Scene com tempo/compasso atribuído tem o **botão de launch colorido** (p.187).
- **Scene View** (p.187-189): painel de propriedades da scene — tempo, fórmula de compasso e **Follow Actions da scene**. Abre ao selecionar scene(s), clicar num controle de tempo de scene, ou clicar no título da Main track. Com várias scenes selecionadas, o título mostra a contagem em vez do nome.

**Track Status fields** (p.189-190) — o indicador de estado por track:
ícone de pizza = clip em loop, com o comprimento do loop em batidas à direita e o número de repetições
tocadas à esquerda; barra de progresso = clip one-shot, com o tempo restante em minutos:segundos;
microfone = track de áudio monitorando entrada, teclado = track MIDI monitorando; miniatura do arranjo
= a track está tocando o Arrangement (p.189-190).

**Stop buttons**: `Ctrl+E` adiciona/remove o Clip Stop de um slot. Remover o stop do slot scene 3/track 4
é como se pré-configura "a scene 3 não mexe na track 4" (p.191).

**Insert Scene** (`Ctrl+I`) insere scene vazia abaixo da seleção. **Capture and Insert Scene**
(`Ctrl+Shift+I`) insere uma scene nova abaixo, **copia os clips que estão tocando agora para ela e lança
a scene nova sem interrupção audível** (p.191). É o "capture" que a nossa `gravar-dmx` quer para cena.

**Session → Arrangement** (p.192-194): com o Arrangement Record ligado, o Live grava no arranjo os clips
lançados, mudanças de propriedade desses clips, mudanças de mixer e device (automação) e mudanças de
tempo/compasso (p.192). **A gravação não cria áudio novo, só clips** (p.192). Session e Arrangement da
mesma track são mutuamente exclusivos: lançar um clip do Session para o Arrangement daquela track; e
**"Back to Arrangement"** — botão que acende para lembrar que o que se ouve difere do arranjo — devolve
(p.192-193). "Stop All Clips" na Main track desativa todos os clips do Arrangement (p.193).
Também dá para mover clips entre as duas views por copiar/colar, arrastando sobre os seletores de view,
ou arrastando entre janelas com Second Window (`Ctrl+Shift+W`) (p.194).
**Consolidate Time to New Scene** (menu Create ou contexto do Arrangement) consolida a faixa de tempo
selecionada em um clip por track e joga numa scene nova (p.194).

### Launch modes (cap. 16, p.355-362)

Os controles de launch estão no **clip panel com o ícone do botão de launch**, e **só valem para clips do
Session** — clips do Arrangement não são lançados, tocam pela posição (p.355). Dá para editar o launch de
vários clips ao mesmo tempo selecionando antes (p.356).

**Launch Mode** (p.356) — quatro modos, valem para mouse, tecla e nota MIDI:

| Modo | Comportamento |
|---|---|
| Trigger | *down* começa; *up* ignorado |
| Gate | *down* começa; *up* para |
| Toggle | *down* começa; *up* ignorado; o próximo *down* para |
| Repeat | enquanto segurado, dispara repetidamente na taxa de quantização do clip |

**Legato** (p.357): o clip lançado assume a posição de reprodução do clip anterior daquela track — permite
trocar de clip a qualquer momento sem perder o sync, mesmo com quantização desligada.

**Clip Launch Quantization** (p.358): por clip; "None" desliga; "Global" usa a da Control Bar
(`Ctrl+6..0`). Qualquer valor diferente de "None" também quantiza o disparo vindo de Follow Action.

**Velocity Amount** (p.359): quanto a velocity da nota MIDI afeta o volume do clip; 0 = nenhuma
influência, 100% = as notas mais fracas tocam em silêncio.

**Nudge** (p.359-360): botões Backward/Forward pulam dentro do clip em incrementos do tamanho da
quantização global. **São mapeáveis**; e **em MIDI Map Mode aparece um controle de scrub entre eles**, que
pode ser atribuído a um encoder rotativo para scrub contínuo (p.360).

**Follow Actions** (p.361-362): definem o que acontece com os outros clips do mesmo *group* (clips em
slots sucessivos da mesma track, separados por slots vazios) depois que o clip toca. Também existem para
scenes, na Scene View. Controles: botão de ativar (`Shift+Enter`), dois choosers A e B, **Chance A/Chance
B** em porcentagem com slider entre eles, switch **Linked/Unlinked** (Linked = dispara no fim do clip ou
após N loops; Unlinked = após a Follow Action Time), e **Follow Action Time** em compasso-batida-semicolcheia,
com um marcador arrastável no editor (p.361-362). As dez ações: No Action, Stop, Play Again, Previous,
Next, First, Last, Any, Other, **Jump** (com slider de alvo, para escolher slot ou scene) (p.362).
Clips e scenes com Follow Action têm o **botão de launch listrado** (p.362). Follow Actions **furam a
quantização global mas não a quantização do clip** (p.362).

---

## 5. Recording (cap. 19)

- **Escolha de entrada** (p.405-406): a track grava o que está no seu In/Out (menu View → In/Out). No Arrangement, é preciso desdobrar e redimensionar a track para ver a seção inteira. Track de áudio grava mono da entrada externa 1 ou 2 por padrão; track MIDI grava todo MIDI dos dispositivos de entrada ativos; **o teclado do computador pode ser ativado como dispositivo pseudo-MIDI** (p.405).
- **Arm** (p.406): clicar no Arm de uma track **desarma todas as outras**, a menos que `Ctrl`/`Cmd` esteja segurado. Com várias tracks selecionadas, armar uma arma todas. **Armar seleciona a track.** Tracks armadas são monitoradas por padrão ("auto-monitoring"). Numa control surface nativa, armar uma track MIDI trava a surface no instrumento daquela track (p.406).
- **Arrangement Record** (p.407): o comportamento depende de "Start Playback with Record" (Record, Warp & Launch Settings) — ligado, grava ao apertar; desligado, só grava quando o Play for apertado ou um clip do Session for lançado. **`Shift` no Arrangement Record inverte o comportamento** (p.407). A gravação cria clips novos em todas as tracks armadas (p.407).
- **MIDI Arrangement Overdub**: o clip novo mistura o que já estava com a entrada nova; **só vale para tracks MIDI** (p.407).
- **Punch-In / Punch-Out** (p.407): switches próprios. **O punch-in é a posição de início do Arrangement Loop e o punch-out é o fim.** Protege o que não se quer regravar e dá tempo de pre-roll.
- **Loop recording** (p.407): gravando dentro do loop, o Live guarda o áudio de **cada passada**. Dá para "desenrolar" com Undo repetido ou graficamente: duplo-clique no clip novo mostra no Sample Editor um sample longo com tudo o que foi gravado; a loop brace do Clip View delimita a última passada, e mover os marcadores para a esquerda audiciona as anteriores (p.407).
- **Gravar em slots do Session** (p.408): quantização global diferente de "None" para os clips saírem cortados certo; armar as tracks (aparecem **Clip Record buttons** nos slots vazios); **Session Record** grava na scene selecionada em todas as tracks armadas — o botão de launch fica vermelho enquanto grava; apertar Session Record de novo passa direto de gravação para loop; alternativamente clicar num Clip Record grava só naquele slot, e o botão de launch daquele clip fecha a gravação. **O botão "New"** para os clips de todas as tracks armadas e seleciona (ou cria) uma scene para a próxima take — **e só existe em Key Map Mode e MIDI Map Mode** (p.408).
- Por padrão lançar uma scene **não** dispara gravação nos slots vazios armados; "Start Recording on Scene Launch" muda isso (p.408-409).
- **Overdub MIDI** (p.409): com quantização global em 1 compasso e Record Quantization escolhida, duplo-clique num slot cria clip vazio de 1 compasso; armar; Session Record; o clip sobregrava a cada volta, camada por camada; apertar Session Record de novo pausa a gravação sem parar o playback, e o próximo aperto volta a gravar. **`Alt`+duplo-clique no slot vazio já arma a track e lança o clip** (p.409).
- **Step recording** (p.409-410): com o transporte parado, segurar notas no controlador e apertar `→` avança o insert marker pelo passo da grade, inserindo as notas; continuar segurando e apertar `→` de novo estende a duração; `←` apaga o que acabou de gravar. **Os navegadores de step recording são mapeáveis por MIDI** (p.410, p.415).
- **Metrônomo** (p.411-412): volume pelo knob Preview Volume; menu suspenso ao lado do switch com count-in, som do tick, **Rhythm** (divisão de batida; "Auto" segue o denominador da fórmula de compasso; divisões que não cabem no compasso aparecem desabilitadas) e **Enable Only While Recording** (fica destacado com o transporte rodando mas só soa gravando; com Punch-In ativo, só depois do ponto de punch) (p.412).
- **Record Quantization** (p.412): no menu Edit; gravando no Arrangement, a quantização é **um passo separado no histórico de undo** — dá para desfazer só a quantização e manter a gravação. Não pode ser mudada no meio da gravação (Session ou Arrangement); em overdub com o loop do Clip View ativo, muda na hora e não é desfeita separadamente.
- **Remote control da gravação** (p.414-415): são mapeáveis o Arrangement Record, os controles de transporte, os botões Arm por track, o Session Record, o botão New, os slots individuais, os controles de navegação relativa (Scene Up/Down) e os navegadores de step recording. O manual dá o padrão: uma tecla para pular de scene e outra para começar/terminar a gravação naquela track (p.414-415).

---

## 6. Automação e envelopes (cap. 25)

**Gravar no Arrangement** (p.491): duas formas — mudar parâmetros à mão enquanto grava, ou gravar uma
performance do Session que contenha automação. Na gravação Session→Arrangement, a automação dos clips do
Session **sempre** vai para o Arrangement, junto com as mudanças manuais nas tracks que estão sendo
gravadas. Para mudanças manuais diretas, quem manda é o **Automation Arm**: com ele ligado, toda mudança
de controle durante o Arrangement Record vira automação (p.491). O controle automatizado ganha um **LED
no thumb do slider**; em pan e Track Activator o LED aparece no canto superior esquerdo (p.491).

**Gravar no Session** (p.492-494): ligar Automation Arm; armar as tracks; Session Record. Existe um
switch **Session Automation Recording** nas Settings que grava automação em **todos os clips tocando**,
armados ou não — assim dá para sobregravar automação num clip MIDI existente sem gravar notas (p.493).
Automação do Session **vira automação de track** quando os clips são gravados ou copiados para o
Arrangement (p.494).

**Modos de gravação de automação no Session** (p.494): com o **mouse**, a gravação para no instante em que
o botão é solto — comportamento "touch". Com **knob ou fader de controlador MIDI**, a gravação continua
enquanto o controle for mexido e, ao soltar, segue até o fim do loop do clip e faz punch-out sozinho —
comportamento "latch".

**Apagar** (p.494): botão direito no controle → Delete Automation, ou `Ctrl+Backspace`. O LED some e o
valor fica constante em toda a timeline e em todos os clips do Session.

**Override e Re-Enable** (p.494-495): mexer num controle automatizado **fora** da gravação apaga o LED —
a automação daquele controle fica inativa e o valor manual manda. Quando há qualquer controle nesse
estado, o botão **Re-Enable Automation** da Control Bar acende; clicar nele devolve tudo ao que está
gravado (p.494). Dá para re-habilitar **um parâmetro só**, pelo menu de contexto dele; e no Session,
**relançar o clip que contém a automação já re-habilita** (p.495).

**Automation lanes** (p.495-496), numerado no manual:

1. **Automation Mode**: toggle acima dos cabeçalhos de track, ou a tecla `A`. `A` de novo desliga (p.495).
2. Clicar num controle de mixer ou device da track **mostra o envelope daquele controle na track do clip** (p.495).
3. Os envelopes aparecem na **main lane, "por cima" da forma de onda / desenho MIDI** — bom para alinhar breakpoints com o conteúdo. Eixo vertical = valor, horizontal = tempo. Para switches e radio buttons o eixo de valor é **discreto** (p.495).
4. **Device chooser**: escolhe o mixer da track, um device, ou "None" para esconder. **Tem LED ao lado dos devices que têm automação**, e a opção "Show Automated Parameters Only" (p.495-496).
5. **Automation Control chooser**: escolhe o controle dentro do device; controles automatizados têm LED (p.496).
6. Botão que **move o envelope para uma lane própria abaixo do clip**, liberando os choosers para ver outro parâmetro ao mesmo tempo. `Alt` + esse botão move **o selecionado e todos os automatizados** para lanes próprias. Se o Device chooser está em "None", o botão some (p.496).
7. Botão que **esconde a lane** — esconder **não desativa** o envelope. `Alt` + esse botão remove a lane selecionada e todas as seguintes daquela track (p.496).
8. Toggle que mostra/esconde todas as lanes extras (p.496).

Botão direito no cabeçalho de uma lane abre opções extras de visualização e **comandos para limpar toda a
automação da track ou de um device** (p.496). `←` a partir de uma lane volta para a track principal e
dobra todas as lanes; `←`/`→` na track principal dobram/desdobram as lanes (p.496).

**Draw Mode** (p.496-497): menu Options, switch na Control Bar, ou tecla `B`. **Segurar `B` enquanto edita
com o mouse liga o Draw Mode momentaneamente** (p.496). Desenhar cria degraus da largura da grade visível;
`Shift` arrastando vertical dá resolução fina no valor do degrau; esconder a grade (`Ctrl+4`) dá desenho
livre, e **`Alt` segurado durante o desenho dá desenho livre temporário com a grade visível** (p.497).

**Breakpoints** (com Draw Mode desligado) (p.497-499):

- clicar sobre um segmento cria breakpoint ali; **duplo-clique em qualquer lugar do fundo cria breakpoint ali**; clicar num breakpoint apaga (p.497);
- o valor numérico aparece ao criar, ao passar o mouse e ao arrastar; passando sobre um segmento selecionado, mostra o valor do breakpoint mais próximo do cursor (p.497);
- arrastar um breakpoint que está na seleção move **todos** os da seleção junto; uma linha vertical preta fina mostra a posição em relação à grade (p.498);
- botão direito no breakpoint → **Edit Value** para digitar valor exato; com vários selecionados, todos se movem relativamente. **Add Value** cria breakpoint com valor exato a partir de um breakpoint de preview (p.498);
- clicar perto de um segmento (ou `Shift`+clique em cima dele) seleciona o segmento e permite arrastar; se o segmento está dentro da seleção de tempo, o Live **insere breakpoints nas bordas da seleção** e move o segmento inteiro (p.498);
- breakpoint criado perto de uma linha de grade **gruda nela**; `Alt` arrastando horizontal ignora o snap; breakpoints e segmentos também grudam nas posições de breakpoints vizinhos, e **continuar arrastando "por cima" de um vizinho o remove** (p.498);
- `Shift` arrastando restringe o movimento a um eixo; `Shift` vertical dá resolução fina (p.498-499);
- **`Alt` arrastando um segmento curva o segmento; `Alt`+duplo-clique volta a reta** (p.499).

**Esticar e enviesar** (p.499-500): passando o mouse sobre uma seleção de tempo aparecem alças nas bordas.
As alças de cima e de baixo esticam no eixo vertical (um retângulo mostra o quanto; gruda nos limites e
quando os cantos se cruzam; `Shift` afina; passar dos limites **corta** o envelope). As alças do meio
esquerda/direita esticam no horizontal (arrastar por cima de breakpoints fora da seleção os **remove**;
`Shift` os move proporcionalmente; `Alt` ignora o snap). As alças de canto **enviesam**; `Alt` espelha o
movimento na alça oposta.

**Simplify Envelope** (p.500): sobre uma seleção de tempo, calcula o número ótimo de breakpoints e remove
os desnecessários, substituindo por retas ou curvas. É o comando certo depois de gravar automação.

**Automation Shapes** (p.501): botão direito numa seleção de tempo → forma. Linha de cima:
seno, triângulo, dente de serra, dente de serra invertido, quadrada — escalados horizontalmente para a
seleção e verticalmente para a faixa do parâmetro; **sem seleção, escalam para o tamanho da grade**.
Linha de baixo: dois conjuntos de rampas e um **ADSR** — estes se **ligam ao valor da automação antes ou
depois da seleção**, indicado pela linha pontilhada.

**Lock Envelopes** (p.502): normalmente mover um clip do Arrangement move a automação junto; o switch
(Control Bar ou menu Options) trava os envelopes na posição da música.

**Comandos de Edit dentro de lanes** (p.502): Cut/Copy/Duplicate/Delete aplicados a uma seleção **dentro
de uma lane** só afetam aquele envelope — o clip e as outras automações no mesmo tempo ficam intactos.
Dá para trabalhar em várias lanes ao mesmo tempo. Para que a edição atinja o clip **e** todos os
envelopes, o Lock Envelopes tem que estar desligado e a seleção tem que ser na track do clip (p.502).
**Copiar e colar movimento de envelope de um parâmetro para outro é permitido**, mesmo entre parâmetros
sem relação (p.502).

**Tempo é automação como qualquer outra** (p.502-503): desdobrar a Main track, Device chooser = "Mixer",
Control chooser = "Song Tempo". Os dois campos abaixo dos choosers escalam o eixo de valor (mínimo e
máximo em BPM) — **e esses dois campos também determinam a faixa de um controlador MIDI atribuído ao
tempo** (p.503).

---

## 7. Clip envelopes — só a diferença (cap. 26)

Um **clip envelope** vive dentro do clip, na aba Envelopes do Clip View, com os mesmos dois choosers
(Device / Control) e os mesmos gestos de desenho e breakpoint da automação (p.504-505).
As diferenças que importam:

- **Automação define o valor absoluto; modulação só influencia esse valor.** Por isso as duas convivem no mesmo parâmetro. Automação é desenhada em **vermelho**, modulação em **azul**; num knob, a automação move a agulha e a modulação aparece como segmento azul no anel (p.508).
- **Clip do Session tem os dois** (dois toggles, Automation e Modulation, abaixo dos choosers). **Clip do Arrangement só tem modulação** — a automação dele mora na lane da track (p.505, p.509).
- LEDs no Control chooser: vermelho = tem automação, azul = tem modulação, os dois = ambos (p.509-510).
- Clip envelope é **não destrutivo**: centenas de clips podem usar o mesmo sample e soar diferente (p.505).
- Apagar: botão direito no editor de envelope ou `Ctrl+Backspace` → Clear Envelope (p.505).
- "MIDI Envelope Auto-Reset" (menu Options) reseta certas mensagens de controle MIDI no início de cada clip (p.505).
- **Dois envelopes afetam volume**: Clip Gain e Track Volume; o segundo é o estágio de ganho do mixer, portanto pós-efeito. Um pontinho abaixo do thumb do slider mostra o volume modulado real (p.510).
- **MIDI Controller clip envelopes** (p.513): Device chooser = "MIDI Ctrl", Control chooser = o número do controlador. Suporta até o controlador 119. Controladores que já têm envelope aparecem com LED.
- **Unlink** (p.513-515): o clip envelope pode ter **loop/região próprios, independentes do clip**. Ao desvincular, as loop braces do envelope ficam coloridas e os controles de loop/região da aba Envelopes ficam ativos: o sample pode continuar em loop enquanto o envelope toca "one-shot", e vice-versa. Serve para programar um fade-out de 8 compassos sobre um loop de 1 compasso (p.514), para transformar um loop curto num longo superpondo um envelope de 8 compassos em loop (p.514), e para usar o envelope como LFO (p.515).

---

## 8. Video (cap. 27)

- Formato: **só QuickTime (.mov)** (p.517). Os arquivos aparecem no browser e entram arrastando.
- **O Live só desenha vídeo para clips que estão no Arrangement.** Arquivo de filme carregado no Session é tratado como clip de áudio (p.517).
- No Arrangement o clip de vídeo é igual a um clip de áudio, exceto pelos "furos de bobina" na barra de título (p.517). Pode ser aparado arrastando as bordas. **Consolidate, Reverse e Crop substituem o clip de vídeo por um clip de áudio** (internamente; o arquivo original nunca é alterado) (p.518).
- **Video Window** (p.518): janela flutuante separada que fica **sempre acima da janela principal** do Live, nunca é coberta. Arrastável, redimensionável pelo canto inferior direito, com visibilidade no menu View. **O tamanho e a posição não pertencem ao Set** — são restaurados na próxima vez que um vídeo for aberto. Duplo-clique = tela cheia (opcionalmente num segundo monitor); `Alt`+duplo-clique volta ao tamanho original do vídeo.
- Trecho sem vídeo no arquivo = tela preta; trecho sem áudio = silêncio (p.519).
- **Tempo Leader** (p.519): ao musicar vídeo, o clip de vídeo é o *tempo leader* e os clips de áudio são *followers* — este é o padrão dos clips do Arrangement. Nesse arranjo, **os Warp Markers do clip de vídeo definem os "hit points"** aos quais a música se sincroniza. O Warp do clip de vídeo precisa estar ligado para ele poder ser leader. Só o clip leader mais **abaixo** que estiver tocando é o leader efetivo (p.519).
- **Arrastar um Warp Marker atualiza a Video Window para o quadro correspondente** (p.519) — é o scrub de vídeo por marcador. Os **marcadores QuickTime embutidos no arquivo são exibidos pelo Live** e servem de referência visual (p.519).
- Fluxo completo em sete passos (p.520): `Tab` alterna as views num monitor só; arrastar o .mov para uma track de áudio faz a Video Window aparecer; arrastar áudio para a drop area cria track; desdobrar as duas; no Clip View do vídeo ligar Warp e pôr Leader; adicionar Warp Markers; opcionalmente ligar o Arrangement Loop numa seção; e **Export Audio/Video** exporta áudio e vídeo juntos.
- **Truque de pre-roll** (p.520-522): filmes chegam com "two-beep" antes da ação. Solta-se o filme em 1.1.1, arrasta-se o **Start Marker** do clip para a direita até o início da ação — ação e música passam a começar em 1.1.1 / 00:00:00:00. No fim, seleciona-se tudo, arrasta-se a composição alguns segundos para a direita, seleciona-se só o clip de vídeo e arrasta-se a **borda esquerda** dele para a esquerda, revelando o pre-roll de volta. Como o Export usa por padrão a duração da seleção do Arrangement, o arquivo sai com a duração exata do original.

---

## 9. MIDI and Key remote control (cap. 33)

**O que é mapeável** (p.836): slots do Session (**a atribuição é do slot, não do clip que está nele**),
switches e botões (Track/Device Activator, tap tempo, metrônomo, transporte), **radio buttons** (grupo de
opções mutuamente exclusivas, ex.: atribuição de crossfader), controles contínuos (volume, pan, sends) e
o crossfader.

**Nota importante** (p.836): uma tecla MIDI usada em mapeamento **deixa de tocar o instrumento** da track
MIDI — ela passa a pertencer só ao controle mapeado.

**Control surfaces** (p.837-839): até seis simultâneas, definidas em Link, Tempo & MIDI (`Ctrl+,`).
Surfaces suportadas nativamente ganham **Instant Mappings** — mapeamentos automáticos por família de
controle, que **se remapeiam sozinhos para o device selecionado** (p.837). Instant Mappings **não
aparecem no Mapping Browser** (p.840). Uma surface pode ser **travada num device** (botão direito na
barra de título do device → "Lock to…"), e um **ícone de mão** na barra de título marca o device travado;
por padrão, armar uma track MIDI trava a surface no instrumento dela (p.838). Surface não suportada:
basta ligar o switch **Remote** da porta de entrada na tabela MIDI Ports; qualquer número de portas pode
ser usado, e o Live mistura o MIDI de todas (p.838-839). Para surface com feedback (fader motorizado,
LED) é preciso ligar o Remote da porta de **saída** também (p.839).

**Takeover Mode** (p.839-840) — o que fazer quando o valor físico e o valor na tela divergem
(bank switching):

| Modo | Comportamento |
|---|---|
| None | o valor novo vai direto para o destino; salto abrupto |
| Pick-Up | mexer não faz nada até o físico alcançar o valor do destino; a partir daí segue 1:1. Suave, mas é difícil adivinhar onde o pick-up acontece |
| Value Scaling | compara os dois valores e calcula uma convergência suave conforme o controle é movido; quando se igualam, segue 1:1 |

**Mapping Browser** (p.840): escondido até um dos três modos de mapeamento ser ligado, e então lista
**os mapeamentos do modo atual**. Cada linha tem: o elemento de controle, o **caminho** até o parâmetro
mapeado, o nome do parâmetro, e **Min e Max**. Os ranges são editáveis a qualquer momento e há um
comando de contexto para **inverter** (Min > Max). Apagar mapeamento: `Backspace`.

**Fazer o mapeamento MIDI** (p.841): 1) `Ctrl+M` liga o MIDI Map Mode — os elementos mapeáveis ficam
**azuis** e o Mapping Browser aparece (`Ctrl+Alt+B` abre o browser se estiver fechado); 2) clicar no
parâmetro; 3) mandar a mensagem MIDI; 4) `Ctrl+M` sai.

**Notas MIDI** (p.841): em slots do Session, Note On/Off agem conforme o **Launch Mode** do clip;
em switches, Note On alterna o estado; em radio buttons, Note On percorre as opções; em parâmetros
variáveis, **uma nota só** alterna entre Min e Max, e **uma faixa de notas** distribui valores discretos
igualmente espaçados pela faixa do parâmetro. Um slot pode ser mapeado a uma **faixa de notas para tocar
cromaticamente**: toca-se primeiro a tecla raiz (que toca o clip na transposição padrão) e, segurando-a,
uma tecla abaixo e uma acima para definir os limites (p.841).

**Controladores absolutos 0-127** (p.841-842): em slots, valor ≥64 = Note On, ≤63 = Note Off;
em **track activators e botões on/off de device**, valores **dentro** da faixa Min-Max ligam e valores
fora desligam — **e pôr Min maior que Max inverte isso**; nos demais switches (transporte etc.), ≥64
liga e <64 desliga; em radio buttons a faixa 0-127 é mapeada sobre as opções; em controles contínuos,
sobre a faixa do parâmetro. Pitch bend e controladores de **14 bits** (0-16383) também são suportados,
com centro em 8191/8192 (p.842).

**Controladores relativos** (p.842-843): quatro tipos — Signed Bit, Signed Bit 2, Bin Offset, Twos
Complement — cada um também em modo "linear". O Live tenta detectar o tipo e se há aceleração;
**mover o controle devagar para a esquerda ao criar a atribuição melhora a detecção**, e o tipo aparece
no chooser "mode" da Status Bar, editável à mão (p.842). Em slots, incremento = Note On e decremento =
Note Off; em switches, incremento liga e decremento desliga; em radio buttons, avança/volta uma opção
(p.843).

**Navegação relativa do Session** (p.843) — nos dois modos de mapeamento aparece **uma tira de controles
atribuíveis abaixo do grid** do Session, com cinco alvos numerados:

1. botões para mover a scene destacada para cima/baixo;
2. um campo numérico de scene, ideal para **encoder infinito**, para percorrer as scenes;
3. botão para lançar a scene destacada (com "Select Next Scene on Launch" ligado, anda sozinho);
4. botão para cancelar o lançamento de uma scene disparada;
5. botões para lançar o clip da scene destacada, por track.

O ganho: **o Live mantém a scene destacada sempre no centro do Session View**, então um Set grande é
navegável com poucos controles (p.843).

**Clip View** (p.843-844): o Clip View mostra o clip selecionado — e também multi-seleção. Portanto
**mapear um controle do Clip View pode afetar qualquer clip do Set**; o manual recomenda usar
controladores **relativos** para esses controles, para evitar saltos.

**Teclado do computador** (p.844): `Ctrl+K` liga o Key Map Mode; os elementos mapeáveis ficam **vermelhos**
(contra o azul do MIDI). Mesmos quatro passos. Efeitos: em slots, conforme o Launch Mode; em switches,
alterna; em radio buttons, percorre as opções. Não confundir com o **Computer MIDI Keyboard** (tecla `M`),
que gera notas.

---

## 10. Atalhos de teclado (cap. 41) — tabelas copiadas

Só a coluna Windows. Fonte de cada bloco: a seção citada.

### 41.1 Mostrar e esconder views (p.984-985)

| Ação | Windows |
|---|---|
| Tela cheia | `F11` |
| Segunda janela | `Ctrl+Shift+W` |
| **Alternar Session/Arrangement** | `Tab` |
| Alternar Device/Clip View | `Shift+Tab` ou `F12` |
| Mostrar Device **e** Clip View | `Alt`+clique no seletor de view |
| Hot-Swap | `Q` |
| Drum Rack / último pad | `D` |
| Info View | `Shift+?` |
| **Video Window** | `Ctrl+Alt+V` |
| Browser | `Ctrl+Alt+B` ou `Ctrl+Alt+5` |
| Overview | `Ctrl+Alt+O` |
| In/Out | `Ctrl+Alt+I` |
| Sends | `Ctrl+Alt+S` |
| Mixer | `Ctrl+Alt+M` |
| Clip View | `Ctrl+Alt+3` |
| Device View | `Ctrl+Alt+4` |
| Groove Pool | `Ctrl+Alt+6` |
| Learn View | `Ctrl+Alt+7` |
| Abrir Settings | `Ctrl+,` |
| Fechar janela/diálogo | `Esc` |

### 41.2 Foco de teclado (p.985-986)

| Levar foco para | Windows |
|---|---|
| Control Bar | `Alt+0` |
| Session View | `Alt+1` |
| Arrangement View | `Alt+2` |
| Clip View | `Alt+3` |
| Device View | `Alt+4` |
| Browser | `Alt+5` |
| Groove Pool | `Alt+6` |
| Learn View | `Alt+7` |
| Clip Panel selecionado | `Alt+8` |
| Clip Panels | `Alt+Shift+P` |

Com "Use Tab to Move Focus" ligado: `Tab` / `Shift+Tab` andam entre controles focáveis;
`Ctrl+Tab` / `Ctrl+Shift+Tab` andam entre vizinhos do controle atual (p.986).

### 41.3 Set e programa (p.986)

`Ctrl+N` novo · `Ctrl+O` abrir · `Ctrl+S` salvar · `Ctrl+Shift+S` salvar como ·
`Ctrl+Q` sair · `Ctrl+Shift+R` **Export Audio/Video** · `Ctrl+Shift+E` exportar MIDI.

### 41.5 Edição (p.987-988)

| Ação | Windows |
|---|---|
| Recortar / copiar / colar | `Ctrl+X` / `Ctrl+C` / `Ctrl+V` |
| Duplicar | `Ctrl+D` |
| Apagar | `Delete` |
| Desfazer / refazer | `Ctrl+Z` / `Ctrl+Y` |
| Renomear | `Ctrl+R` |
| Selecionar tudo | `Ctrl+A` |
| Selecionar vários itens | `Ctrl`+clique |
| Selecionar do primeiro ao último | `Shift`+clique |
| Próxima track/scene ao renomear | `Tab` |
| **Ignorar a grade ao arrastar** | `Alt` |

Modificadores que estendem o alcance dos comandos acima (p.988): `Shift` = clips e slots de **todas as
tracks**; `Shift` = tempo em **todas as tracks**; `Alt` = **a parte selecionada do envelope**.

### 41.6 Ajustar valores (p.988-989)

Setas ↑/↓ decrementam/incrementam · `Shift`+setas em oitavas ou ajuste fino ·
`Shift` arrastando = resolução fina · `Delete` volta ao default · `0`…`9` digita ·
`.` e `,` andam entre campos (compasso/batida/16º) · `Esc` cancela · `Enter` confirma.

### 41.7 Envelopes de breakpoint (p.989)

| Ação | Windows |
|---|---|
| **Automation Mode** | `A` |
| Resolução fina ao arrastar | `Shift` |
| **Criar segmento curvo** | `Alt` |
| Mostrar controles de fade momentaneamente | `F` |
| Apagar envelope selecionado | `Ctrl+Delete` |
| Ignorar grade ao arrastar | `Alt` |

### 41.8 Loop brace e marcadores de início/fim (p.990)

| Ação | Windows |
|---|---|
| Set Start Marker | `Ctrl+F9` |
| Set Loop Brace Start | `Ctrl+F10` |
| Set Loop Brace End | `Ctrl+F11` |
| Set End Marker | `Ctrl+F12` |

Com a brace ou os marcadores **selecionados**: `Ctrl`+clique move o Start Marker para a posição;
`Ctrl+Shift`+clique move o End Marker; `←`/`→` empurram a brace; `↑`/`↓` movem a brace por um
comprimento de loop; `Ctrl`+`↑`/`↓` dobram/dividem o comprimento; `Ctrl`+`←`/`→` encurtam/alongam;
`Ctrl+Shift+L` seleciona o material dentro do loop (p.990).

### 41.9 Zoom, display e seleção (p.990-991)

| Ação | Windows |
|---|---|
| Zoom in/out da janela | `Ctrl++` / `Ctrl+-` |
| Zoom in/out da time ruler | `+` / `-` |
| Rolar acompanhando o playback | `Alt+Shift+F` |
| **Rolar horizontalmente** | `Shift`+roda |
| Pan à esquerda/direita da seleção | `Ctrl+Alt` |
| Adicionar itens à seleção | `Shift`+clique ou arraste |
| Adicionar clips/tracks/scenes adjacentes | `Shift`+clique |
| Adicionar não adjacentes | `Ctrl`+clique |

### 41.13 Grade e desenho (p.995)

| Ação | Windows |
|---|---|
| **Draw Mode** | `B` |
| Grade mais estreita | `Ctrl+1` |
| Grade mais larga | `Ctrl+2` |
| Tercinas | `Ctrl+3` |
| **Snap to Grid on/off** | `Ctrl+4` |
| Grade fixa / adaptativa ao zoom | `Ctrl+5` |
| **Ignorar snap enquanto arrasta** | `Alt` |

### 41.14 Quantização global (p.995)

`Ctrl+6` semicolcheia · `Ctrl+7` colcheia · `Ctrl+8` semínima · `Ctrl+9` 1 compasso · `Ctrl+0` desligada.

### 41.15 Session View (p.996-997)

| Ação | Windows |
|---|---|
| **Lançar clip/slot selecionado** | `Enter` |
| Selecionar clip/slot vizinho | setas |
| Selecionar todos os clips/slots | `Ctrl+A` |
| Copiar clips | `Ctrl`+arraste |
| **Adicionar/remover Stop Button** | `Ctrl+E` |
| Parar clips da track com slot selecionado | `Ctrl+Enter` |
| Inserir clip MIDI | `Ctrl+Shift+M` |
| **Inserir Scene** | `Ctrl+I` |
| **Inserir Captured Scene** | `Ctrl+Shift+I` |
| Andar entre scenes, uma a uma | `↑` / `↓` |
| Andar entre scenes, oito a oito | `PageUp` / `PageDown` |
| **Gravar no Session** | `Ctrl+Shift+F9` |
| Ligar/desligar Follow Actions dos clips selecionados | `Shift+Enter` |
| Criar Follow Action Chain | `Ctrl+Shift+Enter` |
| Mover a track selecionada | `Ctrl`+`←`/`→` |
| Mover scenes não adjacentes sem colapsar | `Ctrl`+`↑`/`↓` |
| Soltar clips do browser como uma scene | `Ctrl` |
| **Desativar clip selecionado** | `0` |
| Ir para o título da track destacada | `Esc` |
| Ir para a primeira/última track da scene | `Home` / `End` |
| Solo da chain selecionada | `S` |

### 41.16 Arrangement View (p.997-999)

| Ação | Windows |
|---|---|
| **Split na seleção** | `Ctrl+E` |
| **Consolidate** | `Ctrl+J` |
| Crop dos clips selecionados | `Ctrl+Shift+J` |
| Redimensionar o clip com o insert marker na borda | `Enter` + `←`/`→` |
| Deslizar a forma de onda | `Shift+Alt`+arraste |
| Esticar clip warped | `Shift`+arraste na barra de título |
| Selecionar tempo dentro do clip | `Shift+Alt`+arraste na barra de título |
| Criar fade/crossfade | `Ctrl+Alt+F` |
| Apagar fades dos clips selecionados | `Ctrl+Alt+Backspace` |
| Mostrar alças de fade momentaneamente | `F` |
| **Loop brace on/off** | `Ctrl+L` |
| Ajustar comprimento da brace | `Ctrl`+`←`/`→` |
| Selecionar conteúdo da brace | `Ctrl+Shift+L` |
| Inserir silêncio | `Ctrl+I` |
| **Cut / Copy / Paste / Duplicate / Delete Time** | `Ctrl+Shift+X` / `+C` / `+V` / `+D` / `+Delete` |
| **Dobrar/desdobrar tracks selecionadas** | `U` ou `←`/`→` |
| Desdobrar todas | `Alt+U` |
| **Altura das tracks/clips selecionados** | `Alt++` / `Alt+-` |
| Rolar acompanhando o playback | `Ctrl+Shift+F` |
| Rolar à esquerda/direita da seleção | `Ctrl+Alt`+arraste |
| **Optimize Arrangement Height** | `H` |
| **Optimize Arrangement Width** | `W` |
| Desativar seleção | `0` |
| Empurrar a seleção | `←` / `→` |
| Inverter clip de áudio selecionado | `R` |
| **Zoom na seleção de tempo** | `Z` |
| **Voltar do zoom** | `X` |
| Tocar a partir do insert marker no clip selecionado | `Ctrl+Space` |
| Mover o insert marker para o playhead | `Ctrl+Shift+Space` |
| Ir para o título da track destacada | `Esc` |
| Levar foco para o Mixer | `Alt+Shift+M` |

### 41.19 Tracks (p.1000-1001)

| Ação | Windows |
|---|---|
| Inserir track de áudio / MIDI / return | `Ctrl+T` / `Ctrl+Shift+T` / `Ctrl+Alt+T` |
| Renomear track selecionada | `Ctrl+R` |
| Próxima track ao renomear | `Tab` |
| Agrupar / desagrupar | `Ctrl+G` / `Ctrl+Shift+G` |
| Mostrar / esconder tracks agrupadas | `+` / `-` |
| Colapsar/expandir grupo | `U` |
| Mostrar/esconder return tracks | `Ctrl+Alt+R` |
| Mover tracks não adjacentes sem colapsar | `Ctrl`+setas |
| **Armar tracks selecionadas** | `C` |
| **Solo das tracks selecionadas** | `S` |
| Adicionar device do browser | `Enter` |
| **Desativar track selecionada** | `0` |
| Freeze/unfreeze | `Ctrl+Alt+Shift+F` |
| Apagar track a partir da barra de título | `Delete` |

### 41.20 Transporte (p.1001)

| Ação | Windows |
|---|---|
| **Tocar do Start Marker / parar** | `Space` |
| **Continuar do ponto onde parou** | `Shift+Space` |
| Parar no fim da seleção | `Ctrl+Space` |
| Tocar a seleção do Arrangement | `Space` |
| Insert marker para o início | `Home` |
| **Gravar** | `F9` |
| Armar a gravação no Arrangement | `Shift+F9` |
| Gravar no Session | `Ctrl+Shift+F9` |
| **Back to Arrangement** | `F10` |
| Ativar/desativar tracks 1…8 | `F1`…`F8` |
| Metrônomo | `O` |

### 41.22 Browser (p.1002)

| Ação | Windows |
|---|---|
| Rolar | setas ↑/↓ |
| Mostrar/esconder browser | `Ctrl+Alt+B` |
| Fechar/abrir pastas | `←` / `→` |
| Carregar item selecionado | `Enter` |
| **Prever arquivo selecionado** | `Shift+Enter` ou `→` |
| Buscar | `Ctrl+F` |
| Ir para os resultados | `↓` ou `Enter` |
| **Atribuir cor aos itens selecionados** | `1`…`7` |
| Resetar cor | `0` |
| Busca por similaridade | `Ctrl+Shift+F` |
| Histórico do browser | `Ctrl+[` / `Ctrl+]` |
| Filter View | `Ctrl+Alt+G` |
| Tag Editor | `Ctrl+Shift+E` |

### 41.24 Key/MIDI Map Mode e Computer MIDI Keyboard (p.1003)

| Ação | Windows |
|---|---|
| **MIDI Map Mode** | `Ctrl+M` |
| **Key Map Mode** | `Ctrl+K` |
| Computer MIDI Keyboard | `M` |
| Oitava do teclado do computador | `X` / `Z` |
| Velocity do teclado do computador | `C` / `V` |

Com o Computer MIDI Keyboard ligado, **os atalhos de uma letra continuam funcionando com `Shift`**
(ex.: `Shift+S` para solo) (p.1004).

### 41.25 Latch momentâneo (p.1004)

Segurar a tecla por ~500 ms alterna a ação **enquanto segura** e volta ao estado anterior ao soltar.
Teclas com latch: `A` (Automation Mode), `B` (Draw Mode), `S` (solo da track selecionada),
`Z` (zoom na seleção do Arrangement), `F1`…`F8` (activator das oito primeiras tracks),
`Tab` (Arrangement ↔ Session). Desligável por `-DisableHotKeyLatching` no `Options.txt`.

---

## Adaptação ao Spellcaster

### Tabela de conceitos

| Ableton Live 12 | Spellcaster | Observação |
|---|---|---|
| **Clip** (audio/MIDI) num track do Arrangement | **trecho** de `.ild`, áudio, vídeo ou fx num track da timeline | mover, aparar pela borda, split (`Ctrl+E`), consolidate (`Ctrl+J`), fade nas pontas, cor e nome próprios (p.170-177) |
| **Scene** (linha do Session, com Scene Launch) | **cue** | lançar a linha toda; a próxima fica pré-selecionada; `Ctrl+I` insere, `Ctrl+Shift+I` captura o estado atual como cue nova (p.185, p.191) |
| **Clip slot** vazio com Stop Button | slot de cue que **não mexe** naquela linha de show | remover o stop button (`Ctrl+E`) pré-configura "esta cue não toca esta track" (p.191) |
| **Track** | linha de show (universo, laser, vídeo) | fold `U`, altura `Alt+±`, `H`/`W` para caber tudo, `0` desativa, `C` arma, `S` solo (p.174, p.998, p.1000) |
| **Device** na cadeia da track | **módulo / nó do patchbay** | o Device chooser da automation lane é o que dá "qual módulo, qual parâmetro" (p.495-496) |
| **Automation envelope** na lane da track | **keys** (keyframes) do parâmetro na timeline | breakpoint, curva com `Alt`, desenho com `B`, Simplify, shapes (p.496-501) |
| **Clip envelope / modulation** | modulação relativa sobre o valor absoluto do key | vermelho = absoluto, azul = relativo; a distinção resolve "cue diz 50%, timeline modula ±20%" (p.508) |
| **Locator** na scrub area | **marcador** de timeline, disparável | mapeável por MIDI/tecla, "Loop to Next Locator", "Set Song Start Time Here" (p.166-167) |
| **Loop brace** | loop de ensaio | `Ctrl+L` liga na seleção; setas ajustam; `Ctrl+↑/↓` dobra/divide (p.169-170) |
| **Arm** de track + Session Record | `gravar dmx` armado por linha | armar seleciona; `Ctrl`+clique arma sem desarmar as outras; `C` arma as selecionadas (p.406, p.1000) |
| **Punch-in / punch-out** | janela de gravação protegida | são as bordas do loop, não campos separados (p.407) |
| **MIDI/Key Map Mode + Mapping Browser** | `Ctrl+Shift+A` (nosso mapping) | copiar: modo com highlight colorido, browser listando controle → caminho → parâmetro → Min/Max editáveis e invertíveis (p.840-841) |
| **Takeover Mode** | comportamento de fader externo ao trocar de banco | Pick-Up e Value Scaling são a resposta certa para surface com banco (p.839-840) |
| **Follow Action** | encadeamento automático de cue | Chance A/B, Linked/Unlinked, Jump para alvo — é lista de cue com probabilidade (p.361-362) |
| **Video Window** | preview do laser/vídeo | janela flutuante sempre acima, tamanho **fora** do arquivo de show (p.518) |
| **Overview** + duas réguas (compasso e relógio) | régua dupla da timeline | show tem cue em relógio e key em quadro; o Live já mostra as duas (p.160-162) |

### Gestos que valem a cópia literal

1. **Arrastar vertical na régua = zoom; arrastar horizontal = scroll.** Vale na Overview e na beat-time ruler (p.160-161). Resolve o pedido do dono sem inventar modificador.
2. **`Ctrl`+roda = zoom, `Shift`+roda = scroll horizontal, `Alt`+roda dentro da track = altura da track** (p.163, p.991). É exatamente o pedido "scroll do mouse mexe cima pra baixo somente nas tracks; shift+scroll anda side to side; ctrl+scroll dá ou tira zoom" — com o acréscimo do `Alt` para altura.
3. **`Alt` segurado ignora o snap; com snap desligado, `Alt` liga o snap** (p.175). Um modificador só, simétrico.
4. **Segurar uma tecla ~500 ms vira toggle momentâneo** (p.1004). `A`, `B`, `S`, `Z`, `Tab`. Um operador segura `Z` para dar uma olhada e solta.
5. **Follow pausa sozinho quando você edita e volta quando o playback reinicia** (p.163). É a diferença entre "playhead centrado" utilizável e insuportável.
6. **Redimensionar uma track com `Alt` redimensiona todas** (p.174).
7. **Botão que move um envelope para lane própria, e `Alt` no mesmo botão move todos os automatizados** (p.496). Uma tecla resolve "quero ver só este" e "quero ver tudo".
8. **Esconder lane ≠ desativar envelope** (p.496). Nunca confundir visibilidade com estado.
9. **`Ctrl`+`←`/`→` faz o insert marker pular de marcador em marcador e de borda de clip em borda de clip** (p.174).
10. **Simplify Envelope** depois de gravar (p.500) — é obrigatório para `gravar-dmx`: a gravação a 44 Hz vira uma curva editável.
11. **A Status Bar mostra a posição exata do que está sob o mouse** (p.35).

### O que NÃO se aplica

Warp, tempo/BPM, quantização musical, Racks, Grooves, Scale Awareness, Capture MIDI, comping e Push:
o Spellcaster não tem tempo musical nem material de áudio elástico — a régua é relógio e quadro, o
"launch quantization" vira grade de tempo/timecode se algum dia fizer falta, e o que o Live resolve com
device chain nós resolvemos com o patchbay.

### Ponto aberto para `design/DECISOES.md`

`Tab` no Live alterna Session ↔ Arrangement, e no `design/SHORTCUTS.md` `Tab` já é troca de Face —
os dois usos coincidem em intenção. Mas o Live também usa `Tab` para navegação de foco quando
"Use Tab to Move Focus" está ligado (p.986), e usa `U` tanto para fold de track no Arrangement quanto
para colapsar grupo (p.998, p.1000). Não decidido aqui.
