# Modo de mapeamento — `Ctrl+Shift+A`

Pedido do Matheus (09/09/2026), literal: *"e um sistema de mapping que nem o resolume Ctrl shift A"*. A tecla está fixada pelo dono e não se discute; o que este arquivo faz é dizer o que o modo mostra, o que ele grava e como isso se resolve num comando.

Fonte: **Resolume Arena**, manual online, páginas *Keyboard Shortcuts*, *MIDI Shortcuts*, *DMX Shortcuts*, *OSC* e *Shortcuts* (tabela). Citações verbatim. Onde o Resolume tem quatro modos e nós temos um, o motivo está escrito.

| Item | Quem resolveu melhor | Por quê |
|---|---|---|
| Entrar no modo | Resolume | Recolore a interface inteira e diz na cara o que é mapeável: *"Everything blue can have a shortcut assigned to it."* Um diálogo de "escolha o controle numa lista" seria a mesma informação escondida |
| Gesto de aprender | Resolume | *"click on it with the mouse. Now press the spacebar, and voila"* — clicar o controle e tocar a entrada. Duas ações, nenhuma caixa de texto |
| Painel do modo | Resolume | Aba que **só existe dentro do modo**, com a lista completa, ordenável, e duplicata marcada em vermelho |
| Apagar | Resolume | `Backspace` ou Delete no menu de contexto |
| Comportamento (toggle, piano, faixa) | Resolume, com corte | `Mode` = Toggle/Value/Mouse, `Piano` e `Invert` como toggles, `Range` como slider de min/max. Nós ficamos com faixa e com piano; o resto vira graph (§5) |
| Endereço textual | Resolume | *"The addresses are all fixed and set up already... Unlike MIDI and keyboard shortcuts, that require you to first link a control to a specific shortcut."* É a regra 2 de `FUNCOES/README.md`, já nossa |
| Quatro modos por protocolo | nenhum | **Não copiamos.** Ver §2 |
| Escopo do alvo (By Position / This Clip / Selected) | nenhum | **Não copiamos agora.** Ver §7 |

## 1. O que o modo faz na tela

`Ctrl+Shift+A` liga e desliga. `Esc` também desliga — Resolume, *Shortcuts*: `Stop Shortcut Editing | Esc`.

Dentro do modo:

1. **Toda a interface recolore**, em todas as páginas: timeline (`index.html`), patchbay, teatro, face, laser 3D, midi. Resolume, *Keyboard Shortcuts*: *"The interface will now turn partially blue."* Aqui a tinta é o âmbar de `PRINCIPIOS.md §2`, porque é um estado e estado tem uma cor só. O que **não** recolore não é mapeável, e isso é a informação.
2. **Cada controle mapeável mostra o seu ENDEREÇO em texto**, sobre ele ou ao lado. Resolume mostra o atalho já atribuído (*"If the Bypass button was big enough, you could even read it had the spacebar assigned to it"*); nós mostramos o endereço **sempre**, porque o endereço é a identidade (regra 2) e porque é o que o operador digita na CLI, manda por OSC e escreve numa cue.
3. **Painel do modo**, canto inferior direito, só existe dentro do modo (Resolume, *Keyboard Shortcuts*: *"This tab is only visible while you're in Shortcuts mode"*). Colunas: entrada, endereço, tipo, faixa. Ordenável por qualquer coluna; **duplicata em vermelho** (*"which makes it really easy to spot double assigned shortcuts. These will be marked in red"*).
4. **Aprender**: clique no controle, depois toque a entrada — tecla, nota/CC do MIDI, mensagem OSC. O que chegar primeiro vira o mapeamento.
5. **Apagar**: com a linha ou o controle selecionado, `Backspace` ou Delete no menu de contexto (Resolume, texto idêntico nas três páginas).
6. **Sair não desfaz nada.** O mapa é do show e já está gravado quando o modo fecha.

O overlay é uma camada só, escrita uma vez, e por isso **exige que todas as páginas falem pelo mesmo bus**. Hoje são três clientes de WebSocket: `bus.js` (patchbay, laser, face), o de `timeline.js:64-106` e o de `teatro.js:71`, os dois últimos com `ponytail:` mandando trocar. Trocar é pré-requisito desta função, não melhoria (`pontos-falhos.md` item 15).

## 2. Um modo, não quatro

O Resolume tem quatro portas de entrada, cada uma com sua cor: `Shift+Ctrl+K` teclado, `Shift+Ctrl+M` MIDI, `Shift+Ctrl+O` OSC, `Shift+Ctrl+X` DMX (*Shortcuts*, tabela). A cor muda por protocolo (*MIDI Shortcuts*: *"The interface will now change color depending on the protocol"*; DMX é *"a nice pastelly yellow"*).

Nós temos **um** modo, `Ctrl+Shift+A`, pelos motivos abaixo, e as três teclas do Resolume viram **filtro dentro do modo**:

- O protocolo já vem na própria entrada. Uma tecla é uma tecla, uma nota MIDI é uma nota, uma mensagem OSC é uma mensagem — não há ambiguidade a resolver com um modo separado. O Resolume precisa dos quatro porque no DMX não existe "tocar o controle" (por isso lá há *DMX Learn* e "Create DMX Shortcut" no botão direito).
- Quatro cores contra `PRINCIPIOS.md §2` ("um acento, e ele significa estado"). Quatro modos com quatro cores é quatro estados para a mesma coisa.
- O dono fixou uma tecla. Quatro entradas seriam quatro teclas.

Dentro do modo, `K`, `M` e `O` **restringem a fonte que o próximo aprendizado aceita** — útil quando há um teclado MIDI mandando clock e o operador quer mapear uma tecla do computador sem capturar a primeira nota que passar. Teclas nuas, painel focado, e a tabela do painel filtra junto. É a única coisa que as três teclas do Resolume compram aqui, e é o suficiente para justificá-las.

Não há modo DMX de entrada: DMX-in não existe no engine (não há `in.dmx` no catálogo fechado de `script/src/graph.rs`). Quando existir, entra como fonte `dmx:` na mesma tabela, sem modo novo.

## 3. Conflito de tecla: `Ctrl+Shift+A`

`design/SHORTCUTS.md` já usa `Ctrl+Shift+A` para **"Selecionar nada"** (par de `Ctrl+A`, origem "ambos": Premiere e Resolve).

O dono fixou `Ctrl+Shift+A` para o modo de mapeamento. Então "selecionar nada" muda, e muda para **`Alt+A`**, porque a própria gramática de `SHORTCUTS.md` já diz o que fazer: *"Ctrl = comando, Shift = estende/amplia, **Alt = variante/limpa**"*, e o mapa já aplica isso três vezes (`Alt+I`, `Alt+O`, `Alt+X` limpam In/Out). Limpar a seleção com `Alt` é a regra da casa aplicada a si mesma, não uma exceção.

(Nota de fonte, para o registro: no Resolume `Ctrl+Shift+A` é *"Open Advanced Output"*, não o modo de atalhos — os modos de lá são `Shift+Ctrl+K/M/O/X`. O dono pediu a tecla, não a página; a tecla fica como ele pediu.)

`A` sozinho é modo automação (`daw-arranjo.md` D1) e `Alt+A` não colide com ele.

## 4. Endereços

Um endereço é o nome do registry, agrupado por `/` (regra 2 de `FUNCOES/README.md`; convenção já fixada em `DECISOES.md` 09/09: *"Porta do graph escreve-se `<uid>/<porta>`... o nome da porta já leva `/` e assim a porta é o próprio endereço do registry"*).

| Endereço | Controle | Tipo | Resolve para |
|---|---|---|---|
| `transport/play` | botão play | trigger | `resume` |
| `transport/pause` | botão pause | trigger | `pause` |
| `transport/stop` | botão stop | trigger | `stop` |
| `transport/locate` | régua, campo de posição | valor (s) | `locate {t: $<duração>}` |
| `transport/loop` | botão loop | toggle | estado de janela (`daw-arranjo.md §4.2`) |
| `track/<i>/mute` | M do cabeçalho | toggle | `show_patch {ops:[{op:"add", path:"/tracks/<i>/mute", value:…}]}` |
| `track/<i>/solo` | S do cabeçalho | toggle | idem, `/solo` |
| `track/<i>/lock` | cadeado | toggle | idem, `/lock` |
| `cue/<i>/go` | linha da cue, célula da grade | trigger | `cue_go {index:<i>}` |
| `cue/go` | botão GO | trigger | `cue_go {}` (próxima) |
| `level/<u>/<ch>` | fader do programmer | valor 0..255 | `level_set {universe:<u>, address:<ch>, value:"$255"}` |
| `level/clear` | botão soltar | trigger | `level_clear` |
| `fixture/<nome>/<canal>` | fader de aparelho | valor | `fixture_set {name, channel, value:"$255"}` |
| `laser/1/arm`, `power`, `play`, `shutter` | botões do modelo 3D | trigger | `laser_*` (`integracao-laser.md`, já em `DECISOES.md`) |
| `laser/1/kpps` | slider | valor | `laser_param {feed:"laser", path:"kpps", value:"$"}` |
| `laser/1/limit/r|g|b`, `curve/r|g|b`, `geo/scale`, `dmx/addr` | sliders do painel | valor | `laser_param` |
| `ilda/fps`, `dev/pps` | painel do player | valor | `ilda-player.md §1` |
| `<mod>/<path>` | parâmetro de módulo | valor | `<mod>_param {feed:"<mod>", path, value}` (`DECISOES.md` 09/09) |
| `marker/<nome>` | marcador na régua | trigger | `input {key:"marker:<nome>", value:1}` |
| `widget/<id>` | widget de Face | valor | `input {key:"widget:<id>", value}` |

Os nomes de `laser/*` e `ilda/*` não são inventados aqui: vêm de `integracao-laser.md` e de `ilda-player.md §1`, e `DECISOES.md` (09/09) já registrou a lista. Este arquivo só acrescenta `transport/*`, `track/*`, `cue/*` e `level/*`, que são os endereços dos controles que a timeline e o teatro desenham.

**Endereço não existe sem controle.** Um endereço na tabela e nenhum widget na tela é a "declaração sem implementação" que `DECISOES.md` já apontou como problema no `modules/laser.json`. A prova disso é um teste (§8).

## 5. Tipos: o que o mapeamento faz, e o que o graph faz

O Resolume tem, por atalho: `Mode` (Toggle / Value / Mouse, e Velocity em nota MIDI), `Piano`, `Invert`, `Range` com min e max, e mais quatro modos só para CC (Absolute / Button / Relative / Fake Relative). É muito, e a razão é que o Resolume não tem um graph atrás.

Nós temos. A divisão:

**O mapa direto (flat) faz duas coisas, sem estado:**

- **trigger** — a entrada chega, o comando roda. `144/60` → `cue_go`.
- **valor** — a entrada chega com um número, e ele entra nos argumentos. É o `"$"` que a frente `midi` já implementou (`spellcore/engine/src/midi.rs:85-107`): `"$"` vira o valor 0..1, `"$<n>"` vira `round(valor * n)` inteiro (`"$127"` = byte MIDI cru, `"$255"` = nível DMX). Recursivo dentro de lista e de objeto.

**Faixa (Range).** Resolume, *Keyboard Shortcuts*: *"With the Range option, you can see what values the slider should jump to when the button is pressed and released"*; e para CC absoluto, *"If you want, you can Invert this behaviour, or set a specific Range."* Cabe numa terceira forma do mesmo token, sem campo novo: **`"$<min>..<max>"`** → `min + valor * (max - min)`, arredondado como `"$<n>"` já é. Invert é `"$255..0"` — o próprio `min > max`, que é como o Ableton também resolve (§33.2.3: *"You can reverse this behavior by setting a Min value that is higher than its corresponding Max value"*). Um `match` a mais em `expande()`, e nenhuma chave nova no arquivo.

**Piano** (segurar liga, soltar desliga). Resolume: *"they will be on for as long as you hold the key down, and turn off when you release"*. **Não precisa de campo nenhum em MIDI**: note-on e note-off são status diferentes (`144/60` e `128/60`), então são duas linhas do mapa, e piano é mapear as duas. Para tecla de computador e para OSC, a chave ganha o sufixo `^` (`"key:Space^"` = ao soltar), que é uma linha no despachante e nenhuma estrutura nova.

**Toggle, latch, contador, rampa, limiar, atraso.** **Não entram no mapa.** Vão para o graph, que já tem os nós no catálogo fechado (`logic.*`, `math.*`, `time.*`, `state`, `cmd` — `script/src/graph.rs:1-70`), e a rota é `in.midi` → `logic.toggle` → `cmd`. O modo de mapeamento oferece um botão "mandar para o graph" que cria essa rota e abre o PATCHBAY nela. Motivo: guardar estado em dois lugares é a fonte dupla que este documento existe para evitar, e o graph é o lugar que já foi desenhado para isso (`orquestrador.md`).

**Fora, e por quê:**

- **Mouse mode** (Resolume: *"you can use the mouse to control a parameter while you have the shortcut pressed"*). Gesto bonito, sem pedido, e brigaria com o arraste de clipe.
- **Relative / Fake Relative** (encoder infinito, com Steps, Step Size, Loop). Entra quando existir superfície com encoder infinito no palco; hoje não há e a frente `midi` já registrou "uma porta MIDI por processo" como limite.
- **Velocity** (nota MIDI, força vira valor). Uma linha quando existir pad sensível: é `"$"` lendo `data2` em vez de disparar 1. Registrado, não implementado.
- **Shortcut Groups** e *Select Next/Previous/Random Item* (Resolume). É o graph com `logic` e `state`.

## 6. Como isso se grava, e como se unifica com a frente `midi`

A frente `midi` (branch `frente/midi`, commit `39b2c4d`) já criou, e está certo:

```json
"midi": { "144/60": {"cmd": "resume", "args": {}},
          "176/1":  {"cmd": "laser_param", "args": {"feed":"laser","path":"kpps","value":"$"}} }
```

— um bloco `extra` do show (`midi.rs:62-68`), chave `"<status>/<data1>"` validada (`midi.rs:110-122`), comandos `midi_map` / `midi_unmap` / `midi_maps` / `midi_learn` / `midi_last`.

**A unificação é de uma linha: o mesmo bloco, com a fonte no prefixo da chave.**

```json
"map": {
  "key:Space":     {"cmd": "resume"},
  "key:Space^":    {"cmd": "pause"},
  "midi:144/60":   {"cmd": "cue_go"},
  "midi:176/1":    {"cmd": "laser_param", "args": {"feed":"laser","path":"kpps","value":"$0..40000"}},
  "osc:/spell/go": {"cmd": "cue_go"},
  "widget:go":     {"cmd": "cue_go"}
}
```

Por que isso não é um formato novo: **o prefixo já é o vocabulário do engine.** `input {key}` documenta exatamente essas chaves (`registry.rs:110-112`: *"widget:go", "key:Space", "osc:/spell/go", "module:laser/stat/fps"*), e `chave()` no graph produz as mesmas cinco (`script/src/graph.rs:280-290`). O bloco `"midi"` da frente `midi` é esse mapa com o prefixo implícito.

Consequências, e é isso que evita duas fontes de verdade:

1. `"midi"` vira `"map"`, e a chave passa a ser `"midi:144/60"`. `migrate()` (`show.rs`) faz a conversão prefixando; nenhum show existente quebra.
2. `midi_map {key, cmd, args}` vira `map_set {key, cmd, args}`, com a mesma validação: comando existe no registry, chave bem formada por fonte (`chave_ok` de `midi.rs:110-122` vira o ramo `midi:`). `midi_map` pode ficar como apelido que prefixa, ou sair — decide a frente `comandos`.
3. `midi_learn` vira `map_learn {source}` e é o que o overlay chama.
4. **Um despachante só.** Hoje `midi::liga(key)` (`midi.rs:73-82`) lê o bloco e chama o registry a partir do pump MIDI. Passa a ser `map::liga(key)`, chamado de três lugares: pump MIDI, `onkeydown` da GUI e o receptor OSC. `input {key, value}` continua sendo a porta única para o graph, e o mapa é consultado **antes**: se a chave está no mapa, roda o comando; sempre, o evento também vai para os ganchos do player (é o que `midi.rs:pump` já faz).
5. **A regra que fecha o assunto:** o mapa direto e o graph nunca guardam a mesma coisa. Mapa = chave → comando, sem estado. Graph = tudo com estado. Uma chave pode estar nos dois (dispara o comando e alimenta a rota), e isso é deliberado, não duplicação: são efeitos diferentes da mesma entrada.

`design/laser/bind.js` (protótipo da rodada 5) tem um terceiro formato, com chave `note:1:60` / `cc:1:7` e persistência em `localStorage`. `DECISOES.md` (09/09, frente `midi`) já registrou o conflito e mandou para voto. A resposta deste arquivo: **ganha a chave do engine** (`midi:144/60`), porque é a que `in.midi` do graph já usa e a que o PRD §10 fixou; e **o `bind.js` para de persistir em `localStorage`**, porque mapeamento é do show e `FUNCOES/README.md §12` não deixa estado de show fora do `.spell`.

## 7. O que fica de fora

- **Shortcut Target** (Resolume: *By Position* / *This Clip, Layer or Group* / *Selected*). É o problema real de "mapeei o mute do track 3 e depois reordenei os tracks", e o Resolume avisa: *"When you delete that specific clip, layer or group, of course the shortcut disappears with it!"*. Aqui isso é o mesmo item de `daw-arranjo.md §4.3` (índice contra `uid`). **Não se decide aqui**: se o track ganhar `uid`, o endereço vira `track/<uid>/mute` e o problema some sem modo de escopo nenhum. Vai junto no voto.
- **Presets de mapeamento** (Resolume: XML separado, trocável por dropdown). O mapa é do show. Quando existir a mesma superfície em dois shows, o caminho é o show importar um arquivo de mapa, não a GUI guardar presets.
- **OSC output / feedback** (Resolume: *"By right clicking, you can enable OSC output for this and only this button"*; e a frente `midi` já registrou LED e fader motorizado como fora, aguardando voto). Fica fora aqui também, pelo mesmo motivo: é a metade de volta e ela precisa de decisão de produto.
- **DMX-in** (Resolume, *DMX Shortcuts*): sem `in.dmx` no catálogo, não há o que mapear.
- **Endereços OSC fixos sem mapear** (Resolume, *OSC*: *"The addresses are all fixed and set up already"*). Isso nós **já temos e é melhor**: todo comando do registry é um endereço OSC por construção, e `GET /commands` lista. O modo de mapeamento é para o caminho contrário (uma entrada física → um endereço), não para inventar endereço.

## 8. Testes

| O que prova | Como |
|---|---|
| `expande("$", 0.5)` | `0.5` |
| `expande("$255", 0.5)` | `128` (inteiro, não `127.5`) |
| `expande("$0..40000", 0.5)` | `20000` |
| `expande("$255..0", 0.25)` | `191` (invert por `min > max`) |
| `map_set` com chave sem prefixo | erro, com a lista de prefixos aceitos |
| `map_set` com `cmd` inexistente | erro (a frente `midi` já tem este teste: `tests/midi.rs:26`) |
| Migração | show com bloco `"midi"` carrega com `"map"` e chaves `midi:` |
| Despachante | `map::liga("key:Space")` devolve `("resume", {})` depois de `map_set` |
| Piano | `map_set("key:Space^")` e `map_set("key:Space")` coexistem e disparam comandos diferentes |
| **Todo endereço tem controle** | percorrer a tabela de §4 e conferir que cada endereço aparece num `data-addr` de alguma página de `spellgui/web/`; o que não aparecer, falha |
| **Todo controle tem endereço** | o inverso: todo elemento clicável das cinco páginas tem `data-addr`, ou está numa lista de exceções declarada no teste |

Os dois últimos são os que impedem o modo de mapeamento de virar uma tela bonita com metade dos controles apagados.

Prova visual: screenshot headless de `index.html`, `patchbay.html`, `teatro.html`, `face.html` e `laser.html` com o modo ligado, mostrando o overlay e os endereços — e a contagem de controles mapeáveis por página no rodapé do painel.
