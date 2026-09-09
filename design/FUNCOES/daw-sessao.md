# Session View — a lista de cues como grade cenas × tracks (painel `Shift+2`, segunda batida)

A outra metade do pedido do Matheus: a interface "que nem o ABLETON" tem duas vistas, e a segunda é a grade. `daw-arranjo.md` é o tempo escrito; este arquivo é o tempo disparado. Nós já temos o objeto — a cue (`cue_set`, `cue_go`, `cue_capture`, `cue_del`, `edit.rs:746,751,874`), especificada em `cenas-cues-dmx.md`. O que falta é a **forma**: hoje a cue é uma linha de planilha (`teatro.js:196-246`), e o Ableton mostra por que a grade é melhor.

Fonte: **Ableton Live 12**, manual oficial online, §7 (Session View) e §16 (Launching Clips), citado por seção. Ableton é a única fonte aqui: o Resolve não tem Session View, e o Resolume (que tem, e chama de deck/column/layer) já foi lido em `fontes/resolume.md` e em `cenas-cues-dmx.md`.

| Item | Quem resolveu melhor | Por quê |
|---|---|---|
| Forma da grade | Ableton §7.2 | Linha = cena, coluna = track, célula = o que aquela cena faz naquele track. Uma tabela de cues com uma coluna "5 ch" (`teatro.js:236`) esconde exatamente o que o operador precisa ver |
| Disparo | Ableton §7.1 e §7.2 | Botão triangular por célula, botão por cena na coluna da direita, pré-selecionar pelo nome e disparar com `Enter` |
| Estado do track | Ableton §7.3 (Track Status field) | Uma pizza com "quantas voltas" e "tamanho do loop" diz mais que um LED |
| Quantização de disparo | Ableton §16.4 | Existe e é global; no nosso caso vira `wait`, que já está no `.spell` |
| Follow actions | nenhum | **Não entram.** Ver §3 |
| Cancelar cena já disparada | Ableton §7.2 ("Cancel Scene Launch") | O `wait` da cue cria a mesma janela e precisa do mesmo botão |

## 1. O que é uma "célula" no nosso caso

No Ableton a célula é um clip: mídia própria, com launch mode e follow action. No Spellcaster **a cue é a linha inteira** — `cue.values` é um mapa achatado `{"universo/endereço": valor}` (`edit.rs:541-543`), não um objeto por track.

Logo:

> **Uma cena = uma cue. Uma célula = a fatia de `cue.values` cujos endereços caem dentro daquele track.** A célula é *derivada*, não armazenada.

Isso não é limitação: é o que faz a grade valer a pena sem mudar formato. Para um track `dmx` com `universe: 1, address: 10` e 4 canais, a célula da cena 3 mostra os valores que a cue 3 escreve em `1/10..1/13` — cheia, parcial ou vazia. É o "vermelho = está na cue, laranja = está mas com outro valor" que `cenas-cues-dmx.md` já tirou do MadMapper, agora com lugar na tela.

Para track `fixture`, a fatia é por canal do perfil (`fixture_set {name, channel}`, `edit.rs:627-631`), não por endereço cru.

Para track de mídia (`laser`, `audio`, `video`, `fx`, `osc`) **a célula está vazia hoje e não há como preenchê-la**: `cue.values` só carrega número por endereço DMX. É a lacuna real desta função, e ela tem uma saída de uma linha:

```json
{"name": "abertura", "fade": 2, "wait": 0, "follow": false,
 "values": {"1/10": 255},
 "fires": ["laser/1/play", "track/4/clip/2"]}
```

`fires[]` é uma lista de **endereços do registry** disparados junto com a cue — a mesma identidade textual da regra 2 de `FUNCOES/README.md`, a mesma lista que `mapping.md` resolve e a mesma que `daw-arranjo.md §5` propõe para o marcador com `go`. Um campo, três funções. **Aguarda voto.**

## 2. Zonas da tela

| Zona | Conteúdo | Origem |
|---|---|---|
| Grade | linhas = cues na ordem do `.spell`; colunas = tracks na ordem do `.spell` (a mesma ordem de `daw-arranjo.md` B6) | Ableton §7.2 |
| Coluna da direita | botão de disparo da cena inteira, nome da cue, `fade`, `wait` | Ableton §7.2 ("The Scene Launch buttons are located in the rightmost column, which represents the Main track") |
| Linha abaixo da grade | por track: estado (parado / em fade / no valor), botão de soltar | Ableton §7.3 (Track Status field) |
| Botão GO | grande, separado, dispara a próxima cue | já existe: `teatro.js:246` |
| Programmer | os canais capturados que ainda não viraram cue | já existe: `level_set` / `level_clear` / `cue_capture` |

## 3. O que NÃO entra, e por quê

- **Follow actions.** Ableton §16.7: duas ações (A e B) com `Chance A`/`Chance B` em porcentagem, dez ações possíveis, `Linked`/`Unlinked` com multiplicador de loops, `Jump Target`, um botão global `Enable Follow Actions Globally`, e ainda a regra *"Follow Actions in scenes always take precedence once they are triggered"*. Nós já temos `follow: bool` na cue (`edit.rs:538-540`): ao terminar, dispara a próxima. É o Follow Action `Next` sem probabilidade, sem salto e sem multiplicador — e é o que um console de luz faz. Acrescentar o resto é uma máquina de estados escondida na lista de cues, e a máquina de estados já está sendo desenhada no lugar certo (nó `state` do graph, `DECISOES.md`, aguarda voto). **Follow actions ficam fora, e o motivo fica escrito aqui para não voltarem por esquecimento.**
- **Launch modes por célula** (Ableton §16.2: Trigger / Gate / Toggle / Repeat). Trigger/toggle/valor já são tipos declarados no registry (regra 3 de `FUNCOES/README.md`) e o modo de disparo é do **mapeamento**, não da cue: `mapping.md §4` põe piano/toggle no atalho, que é onde o Resolume também põe. Duas cópias do mesmo conceito seria a regra 9 violada.
- **Legato** (Ableton §16.3). Sem sentido: cue não tem posição de reprodução.
- **Velocity Amount** (Ableton §16.5). Entra pelo mapeamento MIDI (`mapping.md`), não pela cue.
- **Decks** (Resolume) / conjuntos nomeados de cenas. Uma lista de cues por show basta; quando um show precisar de duas listas, isso é dois shows.
- **Clip Stop por célula** (Ableton §7.4.2, `Ctrl+E` adiciona/remove o botão de stop). Nossa cue não "toca", ela chega a um valor; parar não é um verbo dela. O que existe é `level_clear`, que já é o botão da linha de baixo.

## 4. `Tab`: o conflito, e como fica

Ableton §41.1: `Tab` alterna Session ↔ Arrangement, e §41.25 diz que ele é *momentarily latchable* — segurar cerca de 500 ms alterna e soltar volta.

`design/SHORTCUTS.md` já deu `Tab` para outra coisa: **alternar Face `editor` ↔ `performance`, "(segurar para espiar, toque para trocar)"**. É literalmente o mesmo gesto do Ableton, aplicado a um escopo maior.

**Resolução: `Tab` fica com a Face.** Motivos, na ordem:

1. Escopo. Face `performance` é o que o operador vê no palco; Arrangement e Session são dois painéis do mesmo editor. A tecla mais barata vai para a troca mais cara.
2. `SHORTCUTS.md` cita o PRD §10 para esse `Tab`; mudar exigiria mudar o PRD, e nada aqui justifica.
3. Já existe padrão para "segunda batida" nesta base: `Shift+Z` enquadra e, batido de novo, volta ao zoom anterior (`timeline-daw.md` item 22); `M` cria marcador e, batido de novo, edita (item 26). **Arrangement ↔ Session é a segunda batida do `Shift+2`**, que é a tecla do painel Timeline em `SHORTCUTS.md`. Zero tecla nova, zero conflito, e a regra ("bater de novo na tecla do painel troca a vista dele") vale para qualquer painel que ganhe uma segunda vista depois.
4. O que se perde do Ableton é o *latch* momentâneo entre as duas vistas. Perde-se pouco: espiar a grade enquanto se edita o arranjo é o caso que o Ableton resolve com dois monitores, e nós resolvemos com a Face (`FUNCOES/README.md §13`, layout salvo como preset nomeado).

`FUNCOES/README.md § Pontos abertos` já registra `Tab` como tecla disputada (entrar/sair de grupo no graph teve de ir para `Ctrl+]`/`Ctrl+[` pelo mesmo motivo). Esta é a terceira vez que `Tab` é defendida; fica decidido aqui e some da lista de pontos abertos.

## 5. `teatro.html` é a base? Sim, com duas condições

**É a base.** `spellgui/web/teatro.js` já é o único cliente que fala a linguagem de cue inteira: `cue_set` com todos os campos (`teatro.js:210-220`), `cue_del` (`:240`), `cue_go` no duplo-clique (`:241`) e no botão GO (`:246`), `cue_capture` (`:248`), `level_clear` (`:250`), mais o patch (`teatro.js:~185`). Reescrever isso noutra página seria jogar fora o único código de cue que existe.

Duas condições antes de virar Session View:

1. **Transpor.** `teatro.js:196-246` monta um `<table>` linha a linha com um `<input>` por campo. A grade é o mesmo dado com os eixos trocados: cue vira linha, track vira coluna, e os campos `name`/`fade`/`wait`/`follow` saem da linha para a coluna da direita (§2). O `cue_set` não muda: continua mandando o objeto inteiro (`teatro.js:211-219` já faz isso, porque `cue_set` substitui e não mescla).
2. **Trocar o BUS.** `teatro.js:71` tem um cliente WebSocket próprio de ~40 linhas, com `ponytail:` dizendo para trocar por `bus.js` quando ele existir — ele existe (`patchbay.html:67`, `laser.html:85`, `face.html:49`). Enquanto forem dois clientes, o overlay de mapeamento de `mapping.md` não tem onde se plugar nesta página (`pontos-falhos.md` item 15). A troca é pré-requisito, não melhoria.

O que **não** é base: a tabela de patch da mesma página fica onde está, é `cenas-cues-dmx.md`, e a frente `patchbay-2` mexe nela.

## 6. Atalhos

| Ação | Tecla | Origem | Conflito |
|---|---|---|---|
| Alternar Arrangement ↔ Session | `Shift+2` de novo | Ableton §41.1 (lá é `Tab`) | resolvido em §4 |
| Disparar a cue selecionada | `Enter` | Ableton §7.1 ("pre-select a clip by clicking on its name, and launch it using the computer's Enter key") | nenhum: `SHORTCUTS.md` já tem `Enter` = "Cue GO" |
| Navegar entre células | setas | Ableton §7.1 ("You can then move on to the neighboring clips using the arrow keys") | as setas na Session movem seleção; na Arrangement movem o playhead. Painel focado decide (`SHORTCUTS.md § Gramática`) |
| Cue anterior / próxima | `Shift+PageUp` / `Shift+PageDown` | já proposto em `cenas-cues-dmx.md` | nenhum |
| Cancelar a cena disparada e ainda em `wait` | `Esc` toque | Ableton §7.2 ("Cancel Scene Launch") | `SHORTCUTS.md`: `Esc` segurado 0,5 s é blackout, toque é fechar. Aqui o toque cancela o `wait` pendente **quando há um**; sem `wait` pendente, continua fechando |

## 7. Aguarda voto

1. **`cue.fires: ["<endereço>", ...]`** (§1), que é o que dá célula aos tracks de mídia. É o mesmo campo que `daw-arranjo.md §5` propõe como `marker.go`; o voto decide se os dois se chamam igual (`fires` em ambos, lista) ou se marcador fica com um só (`go`, string).
2. **Follow actions ficam fora** (§3): registrado como recusa deliberada, com o motivo, para não voltar.
3. **`Tab` fica com a Face; Arrangement ↔ Session é a segunda batida do `Shift+2`** (§4).

## 8. Testes

| Função | Entrada | Saída esperada |
|---|---|---|
| `TEATRO.celula(cue, track)` | cue com `{"1/10":255,"1/14":0}`, track `{universe:1, address:10, canais:4}` | `{n: 1, de: 4}` — parcial |
| `TEATRO.celula(cue, track)` | mesma cue, track em `universe:2` | `{n: 0, de: 4}` — vazia |
| `TEATRO.grade(show)` | show com 3 cues e 5 tracks | matriz 3×5, sem consultar o engine |
| `TEATRO.celula` com track de mídia | qualquer cue sem `fires` | `null` (prova que a lacuna de §1 é explícita, não um zero disfarçado) |

Prova visual: screenshot headless de `teatro.html` com `shows/medgrupo.spell` mostrando a grade, e a comparação com `png/teatro.png` (a planilha de hoje).
