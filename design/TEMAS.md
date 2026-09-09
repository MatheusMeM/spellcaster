# Temas do Spellcaster

Ordem de trabalho combinada com o Matheus (voto da rodada 2, 09/09/2026): **temas → destrinchar temas → personas e usos → só então buildar skins**. Este arquivo é o passo 1 e 2. Nada aqui vira código antes de aprovado.

Regra que amarra tudo (a "visão central" que faltou no cubo com fogo): **cada tema é um material, e tudo na skin obedece ao material.** A forma da janela, o que acontece dentro dela, como o GO responde, que luz reflete, que som faz, que gel cabe. Fogo dentro de um cubo azul quebrou a regra; gelo rachando não quebra.

## O que é um tema

Um tema não é uma paleta. É um objeto físico que a janela finge ser. Ele responde cinco perguntas, sempre as mesmas:

| Pergunta | O que decide |
|---|---|
| **Matéria** | De que a carcaça é feita; como a luz entra, reflete, atravessa (o shader). |
| **Fenômeno** | O que vive dentro ou em cima do material quando o show está parado (idle). |
| **GO** | O que o material faz quando o operador aperta GO. Sempre um evento físico, nunca "pisca". |
| **Estado** | Como armado / ao vivo / ensaio / erro aparecem no material (PRINCIPIOS §2: cor é estado). |
| **Som** | Timbre do jingle de entrada e do blip do GO. Mesmo motor Web Audio, instrumento diferente. |

O que **não** muda entre temas (PRINCIPIOS §5): posição e tamanho dos widgets, atalhos (`SHORTCUTS.md`), nomes de comando, densidade. Um tema veste qualquer Face.

## Os temas

### 1. GELO — cubo de gelo (o tema atual, corrigido)

- **Matéria**: bloco de gelo grosso, bordas redondas, cúpula do GO. Refração dupla, dispersão fraca, tinta ciano pela espessura. Geada opcional (o toggle GEADA vira "gelo mais velho").
- **Fenômeno**: rachaduras e bolhas congeladas por dentro, quase invisíveis; reflexo de estúdio frio (softboxes azuladas).
- **GO**: o gelo racha. Uma onda de luz corre pelas fissuras a partir da cúpula e apaga em 2 s. Cúpula acende verde-gelo.
- **Estado**: armado = gelo limpo; ao vivo = fissuras acesas fracas o tempo todo; erro = uma trinca vermelha fixa atravessa o bloco.
- **Som**: chiptune com onda triangular e reverb curto (cristal); blip do GO = estalo.
- **Gel**: sem gel por padrão (voto). Gels frios (Lee 181 Congo, 117 Steel) funcionam; quentes ficam feios de propósito.
- **Serve para**: operador que quer a tela mais limpa possível. Show parado é um bloco transparente em cima do Resolume.

### 2. BRASA — forja

- **Matéria**: vidro fumê escuro e grosso, quase obsidiana, cantos chanfrados (cubo Enscape). Reflexo quente.
- **Fenômeno**: brasa fraca no fundo do bloco, fumaça lenta. É o lar do fogo volumétrico que tirei do gelo.
- **GO**: as chamas sobem pelo bloco inteiro e a face da frente esquenta (bordas laranja por 2 s).
- **Estado**: armado = brasa; ao vivo = chama baixa constante; erro = a chama apaga e sobra fumaça.
- **Som**: chiptune com onda quadrada e distorção; blip = sopro.
- **Gel**: Lee 158 Deep Orange, Rosco 27 Red. Frios não cabem.
- **Serve para**: show de alto impacto (festival, palco de banda). O operador quer sentir o GO.

### 3. TANQUE — água

- **Matéria**: aquário de acrílico, paredes finas, água até 70% da altura com superfície ondulando devagar. Refração da água diferente da do acrílico.
- **Fenômeno**: partículas em suspensão, cáustica da superfície projetada no fundo (o desktop).
- **GO**: uma gota cai na superfície: ondas concêntricas, cáustica corre pelo desktop. O tempo do cue vira boia que sobe.
- **Estado**: armado = água parada; ao vivo = ondulação leve; erro = água turva.
- **Som**: chiptune com senoide e portamento (subaquático); blip = gota.
- **Gel**: Lee 139 Green, 117 Steel. Vira "tanque com corante".
- **Serve para**: instalação, museu, show contemplativo. Quem opera de longe e olha de vez em quando.

### 4. CROMO — skin de WMP de 2001

- **Matéria**: metal cromado com bisel, botões redondos com LED, LCD verde. É o tema Skins Factory literal, o mais kitsch dos quatro.
- **Fenômeno**: reflexo do ambiente (softboxes) deslizando no cromo conforme o cubo gira; LED de standby pulsando.
- **GO**: o botão afunda de verdade (profundidade no shader), o LCD dá flash, o VU estoura.
- **Estado**: LEDs. Armado = âmbar; ao vivo = vermelho; ensaio = azul; erro = todos piscando.
- **Som**: chiptune 8-bit puro (pulse 12,5 %); blip = clique de relé.
- **Gel**: não tem. Cromo não aceita gel; a cor vem dos LEDs.
- **Serve para**: quem entrou no brief pelo WMP. Também o tema mais fácil de ler à distância.

### 5. FITA — pill compacta (não é tema, é Face)

TAB hoje já reduz a janela a uma fita. Isso é uma **Face** (quais widgets), não um tema: cada tema veste a fita do seu jeito (gelo fino, barra de brasa, tubo de água, régua cromada). Fica registrado para não virar "quinto tema".

## Como destrinchar cada tema (antes de codar)

Para cada tema, uma página com:

1. **Referência física**: foto ou shader de referência do material (Shadertoy IDs em `DECISOES.md`).
2. **Tabela das cinco perguntas** preenchida como acima, sem adjetivo solto.
3. **Idle / GO / erro** em três frames desenhados (canvas do Claude Design ou captura do protótipo).
4. **Custo**: passos de raymarching, buffers extras. Água precisa de 1 buffer para a superfície; fogo e gelo não precisam de nenhum. Fluido MIP (tsKXR3) é 4 buffers: só entra se algum tema justificar.
5. **Fita** do tema (a Face compacta vestida).

## Personas e usos (passo 3, depois dos temas aprovados)

Esboço para não perder: operador de mesa às 23h (PRINCIPIOS); VJ com Resolume ao lado; artista de instalação que liga e vai embora; técnico no Pi por SSH (não vê tema nenhum). Cada persona escolhe um tema padrão e uma Face. Detalhar só depois do OK nos temas.

## Passo 4: buildar skins

Ordem proposta: GELO (já existe, ajustar) → CROMO (mais kitsch, mais pedido) → BRASA (reaproveita o fogo) → TANQUE (único que precisa de buffer). Uma por rodada, cada uma com voto.
