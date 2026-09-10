# Resolume — Quickstart Tutorial oficial (o que o produto ensina primeiro)

Complemento curto de `resolume.md`. Aquele arquivo é a auditoria dos arquivos do Arena 7.21.1
instalado (composições `.avc`, `swagger.yaml`, atalhos, fixtures, ajuda contextual) e cobre o modelo
de dados inteiro. Este aqui é outra coisa: **o texto do tutorial oficial de primeiros passos**, que
diz não o que o produto tem, mas o que o fabricante escolheu ensinar nos primeiros cinco minutos —
e nessa ordem. Nada é repetido de `resolume.md`; onde o assunto já está lá, este arquivo aponta.

## Fontes lidas

- `<scratchpad>/fontes/resolume-quickstart.txt` (4.818 bytes) — texto integral de
  https://resolume.com/support/en/quickstart (versão v7), capturado em 09/09/2026.
  O arquivo **não entra no repositório**.
- Documento curto: uma página, cinco seções (`Trigger Clips`, `Mixing`, `Effects`, `Have a Play!`).
  Foi lido inteiro. Não há atalho de teclado nenhum no texto, nenhuma menção a DMX, output, tela
  cheia, mapeamento MIDI/OSC ou Advanced Output. Quem quiser isso: `resolume.md`.

## O que o tutorial ensina, na ordem

### 1. Composition é a unidade de trabalho

Primeira definição dada: "uma composition é um setup completo do Resolume — cada composition pode
incluir conjuntos de clips, efeitos pré-programados e todas as outras configurações necessárias para
uma apresentação". Uma instalação nova **já vem com uma composition de exemplo** — o usuário nunca
encara tela vazia.

Para nós: o `.spell` é a composition, e **o Spellcaster deve abrir com um show de exemplo carregado**,
não com timeline vazia. Custo baixo, e é a diferença entre "clique num thumbnail e algo acontece" e
"leia o manual".

### 2. Layer é linha horizontal, e toca um clip por vez

O texto apresenta o grid assim: abaixo da barra de menu há um conjunto de **linhas horizontais**, cada
uma com controles à esquerda e uma fileira de thumbnails; cada thumbnail é um clip. Depois: "cada uma
dessas linhas horizontais é um **layer**. Cada layer toca **um clip por vez**."

A regra de mistura é ensinada por contraste, em dois cliques:

- clicar outro thumbnail **no mesmo layer** → no início do próximo compasso, a saída troca;
- clicar um thumbnail **em outro layer** → o clip antigo continua e os dois se misturam.

Ou seja: **exclusividade dentro da linha, soma entre linhas.** É a mesma regra que o Ableton usa no
Session View (um clip por track, `ableton12.md` §4) e é a regra que o Spellcaster precisa para
"um universo, uma cena por vez; universos diferentes somam".

O modelo de dados por trás (`Layer`, `Clip`, `Column`, `Deck`, `connected` como enum de 5 estados)
está em `resolume.md` § Objetos e verbos e § Estados.

### 3. Disparo é quantizado por padrão, e o tutorial avisa antes de o usuário reclamar

"esses clips estão configurados para sincronizar com o BPM, então o clip pode não começar
instantaneamente — ele vai esperar o início do próximo compasso. Não se preocupe, se você quiser
lançar clips instantaneamente, pode configurá-los para isso."

Duas coisas boas aqui, as duas de UX e não de função: o comportamento **padrão** é o sincronizado, e o
tutorial **explica o atraso no exato parágrafo em que ele acontece**, antes de o usuário achar que
travou. Um atraso não explicado lê como bug.

### 4. Transport do clip: três botões e uma cunha arrastável

Selecionar a aba **Clip** dá a seção **Transport**: ícones **Forwards, Backwards e Pause** para tocar
e parar, e — o gesto que interessa — "você também pode agarrar a **cunha azul em movimento**
diretamente para fazer scratch no clip".

E a consequência é declarada: mexer assim **tira o clip de fase com o BPM** — o andamento continua
certo, a fase não. Para ressincronizar, **clicar o thumbnail de novo**: ele reinicia no começo do
próximo compasso.

Para nós: (a) o indicador de posição do trecho é **agarrável**, não é só um desenho; (b) sair de
sincronismo é um estado visível e reversível por um gesto óbvio, não um erro. O Ableton resolve o
mesmo problema com Legato e com o Nudge (`ableton12.md` §4); o Resolume resolve com "clique de novo
no clip".

### 5. Sliders por layer: A, V e M

À esquerda dos thumbnails de cada layer, dois sliders verticais **A** e **V**: o A faz fade do áudio
do layer, o V faz o mesmo para o vídeo. E o **M (master)** controla os dois ao mesmo tempo.

Isto é o único ponto do quickstart que já está coberto em `resolume.md` (§ Objetos e verbos: os
campos `audio`, `video` e `master` do `Layer` no swagger). O que o quickstart acrescenta é o
posicionamento: **os três ficam na aresta esquerda da linha, ao lado do nome do layer**, sempre
visíveis, sem abrir painel.

### 6. Browser: abas, não árvore — e o alvo do drop tem quatro cantos coloridos

À direita da interface há as abas **Files**, **Compositions**, **Effects** e **Sources**
(o `recentLayout.xml` mostra que há uma quinta, `Recording` — `resolume.md` § Anatomia da tela).
Selecionar **Effects** lista os efeitos de vídeo instalados.

O gesto de drag and drop, que é o motivo de este arquivo existir para a frente `browser-dnd`:

1. escolher um efeito na lista da aba Effects;
2. arrastar para a esquerda, até a aba **Composition**;
3. soltar **na área que diz "Drop effect or mask here"**;
4. "você sabe que está no lugar certo quando vê **quatro cantos coloridos** aparecerem em volta da aba
   Composition".

Três decisões numa frase: **a zona de drop é rotulada com texto quando está vazia**, o alvo válido
acende **antes** de soltar, e o realce é **os quatro cantos**, não um contorno inteiro (funciona sobre
qualquer conteúdo, sem tapar o que está por baixo).

O efeito aparece imediatamente na saída. Efeitos empilham: "cada efeito pega a saída do anterior e
processa" — a ordem da pilha é a cadeia. Remover: **clicar o `x` à direita do nome do efeito**.

### 7. Todo efeito tem Opacity

"Todos os efeitos de vídeo têm o slider **Opacity** — ele serve para misturar o vídeo processado com o
original." Além dele, a maioria tem parâmetros próprios (o exemplo é o Bendoscope, com um slider de
número de divisões).

Regra transversal, e barata: **todo módulo do patchbay do Spellcaster expõe um "quanto" além dos seus
parâmetros próprios**, com o mesmo nome e no mesmo lugar em todos. Faz "desligar sem remover" e
"metade do efeito" existirem sem que cada módulo invente o seu jeito.

### 8. Janela de Help contextual no canto inferior direito

"Um recurso útil é a janela **Help** no canto inferior direito da interface. Ela mostra dicas curtas
sobre como usar aquilo sobre o que o ponteiro do mouse está no momento."

É o mesmo mecanismo que `resolume.md` já identificou do outro lado, no disco: `docs\help\English.xml`
com 412 verbetes `elemento → título + uma frase`. O quickstart mostra **onde isso aparece na tela** e
que é o último item ensinado — depois de o usuário já ter feito algo funcionar.

---

## Adaptação ao Spellcaster

| Quickstart do Resolume | Spellcaster | Frente |
|---|---|---|
| Instalação nova abre com composition de exemplo | `.spell` de exemplo carregado na primeira execução | — |
| Layer = linha horizontal, **um clip por vez**; layers somam | uma cena por universo, universos somam | `daw-sessao` |
| Clique no thumbnail dispara, quantizado, e o tutorial explica o atraso na hora | disparo de cue com grade de tempo, com o atraso mostrado (não silencioso) | `daw-sessao` |
| **Cunha azul agarrável** = scratch | indicador de posição do trecho é agarrável | `daw-arranjo` |
| Clicar o thumbnail de novo ressincroniza | um gesto óbvio para "volte à fase" | `daw-sessao` |
| Sliders A / V / M na aresta esquerda do layer | nível por linha de show sempre visível, sem abrir painel | `daw-sessao` |
| Abas Files / Compositions / Effects / Sources | browser em abas, não em árvore | `browser-dnd` |
| **Zona de drop rotulada + quatro cantos coloridos ao passar por cima** | realce de alvo válido antes de soltar | `browser-dnd` |
| Pilha de efeitos: cada um pega a saída do anterior; `x` remove | cadeia de módulos do patchbay | `mapping` |
| **Opacity em todo efeito** | um "quanto" padrão em todo módulo | `mapping` |
| Janela Help contextual no canto inferior direito | Aprendiz, uma frase por elemento sob o cursor | — |

### O que o quickstart não cobre e não se deve inferir dele

Sem atalho de teclado, sem cue, sem GO, sem DMX, sem armar saída, sem output/tela cheia, sem
mapeamento MIDI/OSC. Esses assuntos estão auditados em `resolume.md`, inclusive as ausências reais do
produto (§ "O que NÃO copiar": sem armar saída, sem modo ensaio, sem GO no teclado, sem cue de show).
