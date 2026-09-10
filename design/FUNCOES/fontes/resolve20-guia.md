# DaVinci Resolve 20 — digest do Beginner's Guide oficial (fonte para a timeline do Spellcaster)

Digest do **livro**, não do app: o Resolve não está instalado nesta máquina e não foi aberto.
Cada afirmação leva a página do PDF (`p.N`) — que é a página do arquivo, não o número impresso no
rodapé (o livro tem 25 páginas de frente antes da p.1 impressa). Só a coluna Windows dos atalhos.
O que não foi lido não está aqui.

## Fontes lidas

| Arquivo | O que é |
|---|---|
| `<scratchpad>/fontes/resolve20-beginners-guide.pdf` (643 páginas) | "The Beginner's Guide to DaVinci Resolve 20", Blackmagic Design Learning Series, baixado de https://documents.blackmagicdesign.com/DaVinciResolve/20/The%20Beginners%20Guide%20to%20DaVinci%20Resolve%2020.pdf em 09/09/2026 |
| `<scratchpad>/fontes/resolve20-beginners-guide.txt` (1,1 MB) | texto extraído com PyMuPDF, um bloco `=== p.N ===` por página |
| `<scratchpad>/fontes/resolve20-beginners-guide.toc.txt` | sumário embutido do PDF |

`<scratchpad>` = `C:\Users\email\AppData\Local\Temp\claude\D--DRIVE-MNDS-...\6b3d20de-.../scratchpad`.
O PDF e o TXT **não entram no repositório**.

**Lido**: p.39-42 (bins e timeline nova), p.57-70 (In/Out, zoom, overlays de edição, shuffle insert),
p.75-93 (Place on Top, source tape, backtiming, destination controls), p.101-118 (trim), p.127-131
(Inspector), p.148-149, p.169, p.175-198 (keyframes de áudio, voiceover, ordem de track, cinema
viewer), p.432-440 (Keyboard Customization), p.455-467 (Fairlight), p.485-490 (scrollers, razor),
p.509-515 (keyframes no Fairlight).
**Não lido**: as lições de Fusion (9), Color (4-6), entrega (10) e as revisões de fim de lição.

## O que este livro NÃO tem — declarado antes de tudo

- **Não há capítulo da página Cut.** O livro é centrado na página Edit. A Cut aparece só como
  menção: é a página em que o Resolve abre por padrão (p.23, p.362), e a Edit tem "Full Extent Zoom"
  e "Detail Zoom" que correspondem à timeline de cima e à de baixo da Cut (p.58). A "mini-timeline"
  da Cut é citada de passagem numa comparação na página Color (p.217). **"Smart insert" não existe
  em lugar nenhum do livro** — a busca não retorna nada. O que existe e é análogo, na Edit, é o
  **Shuffle Insert / Swap Insert** (p.65-66).
- **Não há "arm" nem "record enable" de track.** A busca não retorna nada. A gravação neste livro é
  só a ferramenta **Voiceover** (p.193-196), que não usa botão de arm por track — o alvo é escolhido
  no menu Record Track da própria ferramenta.
- **Não há automação de mixer no Fairlight.** O livro fecha a lição 8 dizendo explicitamente que
  automação, Fairlight FX e ADR ficam para "The Fairlight Audio Guide to DaVinci Resolve 20" (p.515).
  O que o livro cobre de "automação" é **keyframe de volume no clip**, não automação de fader.
- **Não há tabela de atalhos.** Os atalhos aparecem espalhados no passo a passo; a lista abaixo foi
  montada varrendo o texto, com a página de cada um.

---

## 1. Timeline da página Edit

### Criar e organizar (p.39-41)

Bin novo: `Shift+Ctrl+N`, criado **dentro do bin selecionado** (p.39). Timeline nova: `Ctrl+N`,
criada **dentro do bin selecionado** (p.40). Um projeto contém muitas timelines; cada uma pode ter
configuração própria, escolhida na criação ou mudada depois (p.41). O media pool tem bins comuns,
**smart bins** por palavra-chave (a área de Smart Bins é redimensionável arrastando a linha
divisória, p.68) e **subclipes** (p.42).

### Zoom da timeline — três modos (p.58)

| Modo | O que faz |
|---|---|
| **Full Extent Zoom** | mostra sempre a timeline inteira na janela, reajustando sozinho. Visão de pássaro, navegação para qualquer ponto |
| **Detail Zoom** | escala para uma visão de perto **centrada no playhead**. Para entrar na timeline e ajustar um edit point |
| **Custom Zoom** | escala livre, por slider **ou `Alt`+scroll do mouse**, centrado no playhead |

Os três são botões da toolbar da timeline. **Altura das tracks**: menu Timeline View Options, ou
**`Shift`+scroll** dentro da área de vídeo ou de áudio da timeline (p.58).

Atalhos de zoom (p.58): `Ctrl+=` aproxima no playhead, `Ctrl+-` afasta, **`Shift+Z` alterna entre
caber a timeline inteira na janela e voltar ao zoom anterior**.

Este é o detalhe mais aproveitável do livro para nós: **três modos nomeados de zoom, com um toggle de
"cabe tudo / volta ao que eu estava"**, é uma resposta melhor do que um slider de zoom solto.

### Snapping (p.113, p.188, p.466)

Botão na toolbar, ou tecla **`N`**. E o comportamento que vale copiar: **desligar o snapping com `N`
no meio de um arraste religa o snapping sozinho quando o botão do mouse é solto** (p.113, p.188). O
modificador vira momentâneo sem precisar de modificador.

### Marcadores (p.116, p.455-457)

`M` adiciona marcador na posição do playhead — no clip do source, no clip da timeline, ou na própria
timeline (p.116). **`M` uma segunda vez com o playhead sobre o marcador abre a janela do marcador**
para renomear, mudar a cor e pôr comentário/keyword (p.456); duplo-clique no marcador faz o mesmo
(p.116). Sem clip selecionado (`Shift+Ctrl+A` limpa a seleção), o marcador vai para a **timeline**;
com clip selecionado vai para o clip (p.455).
Navegar entre marcadores: **`Shift+↑` / `Shift+↓`** (p.116, p.461).
A lista de marcadores fica na aba **Markers do Index** (p.457, p.462, p.490) e clicar num item da
lista pula para aquele ponto.

### In / Out (p.57, p.84-85)

`I` marca In, `O` marca Out (p.57). Valem no source viewer **e** na timeline (p.78). `↓` pula para o
próximo edit point (p.84).
**O playhead do Resolve é inclusivo do quadro atual**: In entra na cabeça do quadro, Out entra na
cauda — a duração mínima marcável é 1 quadro (p.84).
Limpar: `Alt+I` (Clear In), `Alt+O` (Clear Out), **`Alt+X` limpa os dois** (p.85).
**Backtiming**: com **só** um Out no source e In+Out na timeline, o Resolve alinha os dois Out — o
plano termina onde se pediu, em vez de começar (p.85).

### Overlays de edição (p.60-62, p.69-70, p.75-79)

Arrastar o clip do source viewer **para o timeline viewer sem soltar** faz aparecer o conjunto de
overlays de edição; o padrão é **Overwrite** (p.60). É a única maneira de escolher o tipo de edição
com o mouse. As que o livro usa:

| Overlay | Efeito | Atalho |
|---|---|---|
| **Overwrite** | entra na posição do playhead, sobrescrevendo o que estiver no caminho (não destrutivo — dá para destrimar de volta depois) | `F10` (p.62, p.65) |
| **Insert** | entra empurrando o resto da timeline para a frente | (p.69) |
| **Append at End** | entra depois do último clip da timeline | (p.70) |
| **Replace** | usa **a posição dos dois playheads** (source e timeline) para alinhar, em vez de In/Out; troca o clip existente | `F11` (p.115-118) |
| **Place on Top** | entra na track acima, criando track de vídeo e de áudio novas se preciso | `F12` (p.75-77, p.118) |

Há também um **overlay "video only"** no source viewer: arrastar dele para o overlay de destino edita
só o vídeo, sem o áudio — **e isso só funciona arrastando**, não por atalho nem por botão da toolbar
(p.79). Para conseguir o mesmo com atalho, desliga-se o **destination control** da track de áudio
(p.80). Dois caminhos para a mesma intenção, com regras diferentes: é exatamente o tipo de
inconsistência que não se deve copiar.

**Reordenar sem arrastar** (p.65-66): Swap Clips Towards Left = `Shift+Ctrl+,` e Swap Clips Towards
Right = `Shift+Ctrl+.`. Funciona com vários clips selecionados. É o "Shuffle Insert".

### Trim (p.101-114)

Dois modos: **Selection mode** (padrão) e **Trim Edit mode**, botão na toolbar ou tecla **`T`**; o
botão fica **vermelho** e o cursor muda (p.102).

O Trim Edit mode é **contextual — a função depende de onde o cursor está sobre o clip**:

| Cursor sobre | Operação | O que faz |
|---|---|---|
| meio do clip | **slip** | desliza o conteúdo dentro dos In/Out do clip, sem mover o clip na timeline (p.103) |
| centro de um edit point | **roll** | move o ponto de corte, aparando os dois clips vizinhos ao mesmo tempo, sem deixar buraco (p.106-107). Funciona **também** no Selection mode (p.106) |
| lado de um edit point | **ripple** | a mudança de duração propaga por todo o resto da timeline (p.109-112) |
| **barra do nome** do clip | **slide** | o clip desliza entre os dois vizinhos, ajustando o de saída e o de entrada (p.112-113) |

Feedback visual: durante slip e slide o timeline viewer vira **preview em quatro quadros** — em cima,
In e Out do clip sendo ajustado; embaixo, o último quadro do clip anterior e o primeiro do seguinte
(p.103-104, p.114). Durante roll, vira **dois quadros** (p.108). E na timeline, um **contorno branco**
mostra os *handles* disponíveis, isto é, a parte do clip que existe no arquivo mas não está em uso
(p.102, p.104). O tooltip mostra o delta em `±SS:FF` (p.101, p.107, p.113).

**Selecionar vários edit points**: `Ctrl`+clique adiciona um segundo ponto de corte à seleção, e os
dois são aparados juntos (p.111). Resolve o caso de "rippleei aqui mas o plano de cima ficou para
trás".

**Linked Selection** (p.108, p.110): botão da toolbar, **`Shift+Ctrl+L`**. Ligado, selecionar vídeo
seleciona o áudio ligado junto; clips ligados têm **ícone de corrente antes do nome**. Desligar é o
que permite o **split edit** (J-cut / L-cut): rolar só o corte de vídeo, deixando o áudio onde está
(p.108-109).

**Gap**: mesmo sem clip, "o lado de saída do buraco" é selecionável e aparável (p.110).

**Razor**: `Ctrl+B`, ou o botão de tesoura da toolbar, divide o clip no playhead (p.489).

**Undo**: `Ctrl+Z`; e existe uma **janela de histórico** com a lista completa do que dá para desfazer
e refazer, em Edit > History > Open History Window (p.111).

### Inspector (p.127-131)

Botão no alto à direita; abre à direita do timeline viewer. Controla o clip: Zoom, Position,
Rotation, Speed Change, estabilização, e as abas Video / Effects / Transition / Settings conforme o
que está selecionado (p.127, p.134, p.143, p.152, p.161). Botão **Expand** faz o Inspector ocupar a
altura inteira da interface (p.127).

**A regra de qual clip o Inspector mostra** (p.128-129), que é a parte que interessa:

1. clicar num clip o seleciona, e o Inspector mostra o selecionado;
2. **sem nada selecionado, o Inspector mostra automaticamente o clip da track mais alta sob o
   playhead**;
3. seleção **sobrepõe** a escolha automática — com um clip selecionado, mover o playhead não muda o
   Inspector.

O nome do clip aparece no topo do Inspector para confirmar de quem são os controles (p.129).

Valores numéricos: arrastar a rodinha, ou digitar e `Enter` (p.130).

**Keyframe no Inspector**: o livro **não ensina keyframe de vídeo na página Edit**. Ele diz, numa
nota (p.155), que dá para animar o gráfico com keyframes em vez de Dynamic Zoom e que "você vai
aprender mais adiante" — e o "adiante" é a lição de **Fusion** (p.542-543), onde o gesto é: clicar o
**botão cinza de Keyframe à direita do parâmetro** no Inspector para criar o primeiro, mover no tempo,
mexer no valor (ou clicar o botão de novo) para criar o próximo, e refinar as curvas no
**Keyframes Editor**, um painel separado aberto pelo botão Keyframes no alto à direita (p.543).
Ou seja: **no Resolve o keyframe nasce no Inspector e é editado num editor de curvas separado** — não
na timeline. Só o keyframe **de áudio** vive na timeline.

### Timeline View Options (p.58, p.148, p.169, p.461, p.485-487)

Menu da toolbar que controla o desenho da timeline, não o conteúdo: altura das tracks (p.58, p.169),
fundo do viewer (Checkerboard / Black, p.148-149), **Track Display Options** — no Fairlight, mostrar
as tracks de vídeo no topo da timeline (p.461) — e os **scrollers** (p.485-487, ver §3).

### Ordem das tracks (p.197)

Botão direito nos controles da track → **Move Track Up/Down**. Ou abrir o **Index**, aba **Tracks**, e
arrastar. A aba Tracks também tem o **botão de visibilidade (olho) por track**: esconder tracks
**não as muta** — elas continuam soando, só saem da tela para simplificar o trabalho (p.510-511).
Separar visibilidade de mute é distinção que o Spellcaster precisa ter também.

### Cinema Viewer (p.198)

Workspace > Viewer Mode > Cinema Viewer, ou **`Ctrl+F`**. Tela cheia com controles de navegação em
overlay que **somem sozinhos depois de alguns segundos**; os atalhos normais continuam valendo
(`Home`, `J`/`K`/`L`); `Esc` sai.

### Preview de trecho

`/` (barra) toca o trecho em torno do ponto atual para conferir a mistura (p.191, p.192).

---

## 2. Página Cut — só o que difere e vale

Único conteúdo do livro sobre a Cut:

- **É a página em que o Resolve abre por padrão** (p.23, p.362).
- A Cut tem **duas timelines empilhadas**: a de cima é a timeline inteira, a de baixo é a visão
  aproximada. O livro usa isso para explicar Full Extent Zoom e Detail Zoom da página Edit, que são
  os equivalentes em botão (p.58).
- A Cut tem **mini-timeline** — uma tira de barras cuja largura é proporcional à duração do clip; a
  mesma coisa aparece na página Color (p.217).

**Source Tape** (p.83) é da página Edit, não da Cut, apesar de ser a ferramenta que o pedido
associava à Cut. Em vez de abrir clip por clip: clicar o botão **Source Tape** no source viewer abre
**todos os clips do(s) bin(s) selecionado(s) emendados**, na ordem de sort atual, como se fossem uma
fita só. Dá para marcar In e Out ali dentro normalmente. Ligando o **Timeline mode** depois do Source
Tape, a fita abre **como timeline**, com navegação e precisão maiores. **`Q`** alterna entre a
timeline do Source Tape e a timeline de edição. O botão **Source Clip** volta à visão de clip único.

Para o Spellcaster: Source Tape é a resposta a "quero passar o olho em 200 arquivos `.ild` sem abrir
200 vezes". Vale mais que a página Cut inteira.

---

## 3. Fairlight — só o que o livro cobre

`Shift+7` troca para a página Fairlight (p.457). **É a mesma timeline**: fades, transições,
keyframes, nomes de track, cores e marcadores feitos na Edit já estão lá, e o inverso também (p.458).

### Zoom e altura, sem os botões (p.459)

O Fairlight **não tem** os botões Full Extent e Detail Zoom. Os gestos são os mesmos da Edit:
`Alt`+scroll dá zoom horizontal, `Shift`+scroll muda a altura das tracks, **`Shift+Z` faz caber**
(p.458, p.459, p.462). Para dar zoom de altura **centrado numa track**, clica-se no header da track
primeiro — **e isso também seleciona automaticamente o clip daquela track sob o playhead** (p.459).

### Bus 1 (p.459)

Abaixo das tracks aparece uma "track" a mais que não existe na Edit: **Bus 1**, a saída estéreo da
timeline. Número de canais em Fairlight > Bus Format. O resto de bussing fica para o guia do Fairlight.

### Precisão de sample (p.465-466)

O clip de vídeo se apara no quadro; o áudio se apara no **sample** (48 kHz ≅ 2000 samples por quadro a
24 fps). Dando zoom o bastante **aparecem os pontos dos samples individuais**. E ao segurar o botão
durante o trim, o Fairlight **desenha a forma de onda dos handles** — vê-se o que existe fora do
corte antes de decidir (p.466).

Regra prática do livro para trabalho fino: **snapping e Linked Selection desligados** e zoom fundo
(p.467).

### Scrollers (p.485-488)

Timeline View Options → **Display Video Scroller**: abre, abaixo da timeline, uma tira dos **quadros
de vídeo individuais**, com uma **linha vermelha no centro** marcando o quadro sob o playhead. Clicar
num quadro à esquerda ou à direita move o playhead para o início daquele quadro.
→ **Show Audio Scroller 1**: abre abaixo do scroller de vídeo a forma de onda de **uma track
escolhida** num menu Display próprio. Arrastar a forma de onda no scroller move o playhead.
Serve para casar efeito sonoro com ação na tela — o livro diz que é muito mais fácil que julgar pela
timeline (p.488).

Este é o achado mais transferível do Fairlight para nós: **uma tira secundária, sob a timeline, que
mostra o conteúdo quadro a quadro em torno do playhead, com marca central fixa.** É a resposta certa
para "alinhar um key de laser com um quadro de vídeo".

### Track formats e altura travada (p.463-464, p.512)

Botão direito no header → **Change Track Type To > Mono/Stereo**; track mono toca só o primeiro canal
do clip (p.464-465). Botão direito nos controles → **Lock Track Height to > Mini** encolhe a track e
a mantém acessível; **> None** devolve a altura livre (p.512).

### O que o livro chama de "automação" (p.509-515)

Não é automação de fader. É **keyframe de ganho no clip**, o mesmo gesto da Edit: `Alt`+clique na
linha de ganho cria keyframe, arrastar o trecho entre dois keyframes muda o nível (p.513-514).
Diferença anotada: **no Fairlight o tooltip mostra o nível absoluto e o relativo como valor Δ**
(p.514), enquanto na Edit mostra só o ajuste (p.176).
O que substitui automação, no livro, é o **Ducker** — um processador do mixer que abaixa uma track em
função do áudio de outra, ligado por um botão Enable na tira do mixer (p.507, p.510). O livro compara
os dois caminhos de propósito: Ducker é automático, keyframe é controle fino (p.509).

---

## 4. Keyframes de áudio na timeline (p.175-179, p.190-192)

O gesto completo, porque é o que mais se parece com o que o Spellcaster precisa:

1. `Shift`+scroll sobre a área de áudio para aumentar as tracks e enxergar a forma de onda (p.175).
2. **`Alt`+clique na volume bar cria um keyframe** naquele ponto (p.176).
3. Clicar e segurar a volume bar mostra, no tooltip, **o ajuste atual em dB** (p.176).
4. Arrastar a volume bar entre dois keyframes muda o nível daquele trecho; o tooltip mostra o valor
   (p.177).
5. **`Shift` segurado durante o arraste dá precisão fina** (p.177).
6. O padrão de uso é **em pares**: dois keyframes marcam a borda de uma região, e o trecho entre eles
   é o que se levanta ou se abaixa (p.176-178, p.192). Para tirar um estalo, três keyframes e puxar o
   do meio para baixo (p.178-179).

Não há, no livro, curva, easing ou tipo de interpolação para keyframe de áudio — só o segmento entre
dois pontos.

---

## 5. Voiceover (p.193-196)

A única gravação que o livro ensina. Timeline > Record Voiceover, ou o **botão Voiceover no topo da
timeline** (p.194). A janela tem:

- **File Name** — o nome do arquivo a gravar (p.194);
- **Audio Input** — qual entrada usar, quando há mais de uma (p.194);
- **Record Track** — em qual track gravar; **"Auto" deixa o Resolve escolher** conforme o layout atual
  de tracks (p.194);
- menu Options (…) com **Mute Timeline Audio While Recording**, para não realimentar (p.195), e
  **Stereo Input** (o padrão é mono) (p.195);
- botão **Record**, que dispara uma **contagem regressiva** antes de começar (p.195); apertar de novo,
  ou `Esc`, encerra (p.196).

O resultado entra **como clip novo numa track nova da timeline e, ao mesmo tempo, no bin selecionado
do media pool** (p.196). Tomadas seguintes **sobrescrevem o clip da timeline** mas **se acumulam no
bin** — nada é perdido (p.196).

Para nós: contagem regressiva antes de gravar, destino "Auto", e "a última toma manda na timeline mas
todas ficam guardadas" são três decisões prontas para `gravar-dmx`.

---

## 6. Atalhos citados no livro

Só Windows. Cada linha tem a página onde aparece.

### Timeline e navegação

| Ação | Windows | p. |
|---|---|---|
| Novo bin | `Shift+Ctrl+N` | 39 |
| Nova timeline | `Ctrl+N` | 40 |
| Marcar In / Out | `I` / `O` | 57 |
| Limpar In / Out / ambos | `Alt+I` / `Alt+O` / `Alt+X` | 85 |
| Próximo edit point | `↓` | 84 |
| Zoom in / out no playhead | `Ctrl+=` / `Ctrl+-` | 58 |
| Custom Zoom dinâmico | `Alt`+scroll | 58, 459 |
| Altura das tracks | `Shift`+scroll | 58, 175, 459 |
| **Caber a timeline / voltar ao zoom anterior** | `Shift+Z` | 58, 458, 462 |
| Snapping on/off (religa sozinho ao soltar o mouse) | `N` | 113, 188, 466 |
| Marcador (2ª vez abre a janela do marcador) | `M` | 116, 456 |
| Marcador anterior / próximo | `Shift+↑` / `Shift+↓` | 116, 461 |
| Desselecionar tudo | `Shift+Ctrl+A` | 455 |
| Desfazer | `Ctrl+Z` | 111 |
| Preview do trecho | `/` | 191 |
| Tela cheia (Cinema Viewer) | `Ctrl+F` | 198 |
| Sair da tela cheia / parar gravação | `Esc` | 196, 198 |
| Página Fairlight | `Shift+7` | 457 |

### Edição

| Ação | Windows | p. |
|---|---|---|
| **Overwrite** | `F10` | 62, 65 |
| **Replace** | `F11` | 118 |
| **Place on Top** | `F12` | 77, 81-89 |
| Trim Edit mode | `T` | 102, 117 |
| Linked Selection on/off | `Shift+Ctrl+L` | 110 |
| Adicionar edit point à seleção | `Ctrl`+clique | 111 |
| Razor / dividir no playhead | `Ctrl+B` | 489 |
| Swap Clips Towards Left / Right | `Shift+Ctrl+,` / `Shift+Ctrl+.` | 66 |
| Alternar Source Tape ↔ timeline de edição | `Q` | 83 |
| Enable Clip (liga/desliga o clip) | `D` | 435 |
| Bypass de todos os grades | `Shift+D` | 436 |
| Keyframe de volume | `Alt`+clique na volume bar | 176, 513 |
| Precisão fina no arraste de volume | `Shift` | 177 |
| Paste Attributes | `Alt+V` | 145, 183 |
| Create Subclip | `Alt+B` | 416 |
| Keyboard Customization | `Alt+Ctrl+K` | 433 |

### Keyboard Customization (p.432-439) — o que copiar do sistema, não dos atalhos

- Abre por `Alt+Ctrl+K` (p.433).
- Traz **presets que emulam outros NLEs**; o livro avisa que a emulação nunca é 100%, porque funções
  que existem num sistema não existem no outro (p.433-434).
- A metade de cima é um **teclado interativo**: teclas sem função aparecem escuras, com função
  aparecem claras, e **um número no canto inferior direito da tecla indica que ela tem função em mais
  de uma página** do app (p.434). Clicar (ou segurar a tecla física) mostra, na área Active de baixo,
  **a função e em qual painel ela vale** (p.435). Clicar nos modificadores remapeia o desenho inteiro
  do teclado para aquele modificador (p.435-436).
- A metade de baixo é **busca por comando**: escolher o grupo (All Commands, ou um menu/painel
  específico), digitar, e clicar na coluna Keystroke para atribuir apertando as teclas (p.436-438).
- **Um comando aceita vários atalhos** (botão `+`); cada um removível pelo `x`; e há uma **seta de
  reset por comando**, visível ao passar o mouse (p.438).
- **Conflito é avisado, não silencioso**: se a combinação já pertence a outro comando, o Resolve
  pergunta; escolhendo Assign, o atalho é **removido do comando original**. Resetar devolve (p.438).
- **Os presets padrão não podem ser alterados** — mexer obriga a salvar um preset novo com nome
  (p.438-439). O menu Options (…) exporta, importa e apaga presets (p.439).

Este é o modelo certo para o nosso `design/SHORTCUTS.md` virar tela: teclado desenhado, busca por
comando, aviso de conflito com quem perde o atalho, preset do usuário separado do preset de fábrica.

---

## Adaptação ao Spellcaster

| Resolve 20 | Spellcaster | Referência |
|---|---|---|
| Timeline da página Edit | timeline de show | p.58 |
| Clip numa track | trecho de `.ild` / áudio / vídeo / fx | p.101 |
| **Full Extent / Detail / Custom Zoom** | os três botões de zoom da nossa régua | p.58 |
| `Shift+Z` | "caber tudo" com volta ao zoom anterior | p.58 |
| **Trim Edit mode contextual** (`T`) | um modo, quatro operações pela posição do cursor | p.102-113 |
| **Handles em contorno branco** | mostrar quanto do arquivo existe fora do corte | p.102, 466 |
| Preview de quatro quadros no slip/slide | preview do quadro de entrada e saída ao aparar | p.103-104 |
| **Linked Selection** (`Shift+Ctrl+L`) | ligar/desligar o vínculo entre a linha de laser e a de áudio do mesmo trecho | p.108-110 |
| Marcador (`M`, `M` de novo edita) | marcador de timeline com nome, cor e comentário | p.116, 456 |
| **Index > aba Tracks, com olho de visibilidade** | lista de linhas do show; esconder ≠ mutar | p.197, 510-511 |
| **Index > aba Markers** | lista de marcadores clicável | p.457, 462 |
| Keyframe de volume (`Alt`+clique, pares) | keys de parâmetro desenhados direto na track | p.176-178 |
| Inspector, com regra de "clip da track mais alta sob o playhead" | painel de propriedades do trecho selecionado | p.128-129 |
| **Scrollers de vídeo e de áudio** | tira quadro a quadro sob a timeline, com marca central | p.485-488 |
| **Voiceover tool** (contagem, Record Track = Auto, tomadas acumuladas no bin) | `gravar dmx` | p.193-196 |
| **Keyboard Customization** | tela de atalhos do Spellcaster | p.432-439 |
| Source Tape (`Q`) | passar o olho em muitos arquivos como se fossem um só | p.83 |
| Playhead inclusivo do quadro; duração mínima 1 quadro | regra de borda da nossa seleção de tempo | p.84 |

### O que NÃO se aplica

Páginas Color, Fusion e Deliver, grades e nodes, bussing e mixer de áudio multicanal, formatos de
track mono/estéreo, speed change e estabilização — o Spellcaster não faz pós de imagem nem mixagem, e
o que aqui é "canal de áudio" lá é universo DMX, que não tem estéreo nem sample rate.

### Ponto aberto

`N` (snapping) e `T` (trim mode) do Resolve colidem com nada em `design/SHORTCUTS.md` hoje, mas
`M` (marcador) e o `M` do Ableton (Computer MIDI Keyboard) apontam para lados opostos — ver
`ableton12.md` § Adaptação. Não decidido aqui.
