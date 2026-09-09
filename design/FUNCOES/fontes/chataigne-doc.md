# Chataigne — digest da documentação oficial (Notion)

Complemento de `chataigne.md`. Aquele arquivo audita o **app instalado e o código-fonte C++**
(`Sequence.h`, `Mapping.h`, `Action.h`, `Parrot.h`, `OrganicMainComponentCommands.cpp`, ...) e é a
autoridade sobre estruturas, enums, parâmetros e atalhos de comando. **Este arquivo não repete nada
disso.** Aqui está só o que a documentação acrescenta: os gestos de mouse (que não existem em tabela
de comando nenhuma), o enquadramento conceitual do autor, o formato `module.json`, Multiplex,
Conductor, Morpher, o Web Dashboard — e as páginas que nunca foram escritas.

Captura: **painel Browser**, porque o Notion renderiza por JavaScript e não devolve nada a um
`WebFetch`. Data: **09/09/2026**.

## Fontes lidas

Raiz: https://benkuper.notion.site/The-Amazing-Chataigne-Documentation-079bd5a0b7e648bbbfe34c3c869a3985
Texto de cada página salvo em `<scratchpad>/fontes/chataigne/<slug>.txt` (21 arquivos, 53.355 bytes),
cada um com a URL de origem e a data no cabeçalho. **Não entram no repositório.**

| Arquivo | Páginas do Notion cobertas |
|---|---|
| `00-index.txt` | raiz + a lista bruta de `a[href]` (24 subpáginas internas + 1 externa) |
| `history-and-philosophy.txt` | History and Philosophy of Chataigne |
| `the-interface.txt` | The Interface |
| `the-ultimate-cheat-sheet.txt` | The Ultimate Cheat Sheet |
| `the-modules.txt` | The Modules |
| `making-your-own-module.txt` | Making your own Module |
| `the-module-router.txt` | The Module Router |
| `state-machine-introduction.txt` | Introduction to the State Machine |
| `actions.txt` | Actions |
| `mappings.txt` | Mappings |
| `multiplex.txt` | Multiplex |
| `conductor.txt` | Conductor |
| `time-machine-introduction.txt` | Introduction to the Time Machine |
| `time-machine-interface-and-navigation.txt` | Interface and Navigation |
| `sequence-layers.txt` | Sequence Layers **+ as 6 subpáginas** (Trigger, Mapping, Mapping 2D, Color, Audio, Sequence Layer) |
| `scripting-introduction.txt` | Introduction to Scripts |
| `scripting-reference.txt` | Scripting Reference (**truncada**, ver abaixo) |
| `custom-variables.txt` | Introduction to Custom Variables + Example 1 + Example 2 |
| `morpher.txt` | The Morpher : 2D Interpolations made fun |
| `dashboard.txt` | Introduction to the Dashboard + Web Dashboard |
| `detective-e-parrot.txt` | Introduction to the Detective + Introduction to the Parrot (**as duas vazias**) |

**24 páginas listadas na raiz, 24 visitadas, 21 arquivos** (páginas curtas de mesmo tema foram
agrupadas num arquivo só). Somando as 6 subpáginas de Sequence Layers, que não aparecem no índice da
raiz e só foram descobertas relendo os `a[href]` de dentro de `main`: **30 páginas capturadas**.

### Falhas e lacunas, declaradas

1. **"Introduction to the Detective" e "Introduction to the Parrot" estão vazias.** As duas existem no
   índice, abrem, e o corpo tem **zero** parágrafos, listas, tabelas ou imagens — `get_page_text`
   devolve só o título, e a checagem programática confirmou (`{blocks: 1, imgs: 2, len: 26}`).
   Consequência: **tudo o que se sabe sobre Detective e Parrot vem de `chataigne.md`**
   (`Detective.h:12` com `watchControllable(c)`; `Parrot.h:22-44` com `status ∈ {IDLE, RECORDING,
   PLAYING}`, `targetsCC`, `loop`, `forceValueAtStartRecord`, `trimToFirstData`/`trimToLastData`), não
   desta documentação. Isto é relevante para a frente `gravar-dmx`: **o produto que mais se parece com
   o que queremos fazer não documentou a função**.
2. **"Scripting Reference" foi truncada** no limite de 30.000 caracteres da captura, no meio da seção
   "Util object". As subpáginas de tipos de script (Module, Condition, Consequence, Mapping Filter,
   Mapping Output) **não foram capturadas**. Este digest só usa o que veio antes do corte.
3. **A seção "Parameters" da página Mappings está vazia na fonte** — o texto literal é
   `To fill`. É lacuna da documentação, não da captura.
4. A página do Raspberry Pi mora em **outro workspace do Notion**
   (`sugar-ocean-c6b.notion.site`) e não foi visitada — está fora do escopo pedido.
5. A própria raiz avisa: *"como o software está em evolução constante, esta documentação pode não
   estar sempre atualizada"*. Onde a documentação e `chataigne.md` divergirem, **o código-fonte manda**.

---

## 1. A filosofia declarada (e por que ela importa para nós)

O autor define o produto assim: em projetos artísticos com tecnologia, o criador usa vários softwares
e dispositivos — um para áudio, um para vídeo, um para mapping, um para luz, um Arduino para motores —
e a comunicação entre eles vira o problema. O Chataigne "quer ser o **maestro dessa orquestra
tecnológica**: sozinho ele quase não faz nada visível para o público, mas é quem vê o quadro inteiro e
garante que cada um receba o que precisa".

Duas consequências que ele mesmo tira, e que o Spellcaster deveria assumir por escrito:

- **"Como o Chataigne é um maestro e não toca um instrumento, ele não serve para nada se você não tem
  uma orquestra."** É honesto e é uma decisão de escopo: o produto não tenta ser também o gerador.
- **Dois conceitos centrais, não um**: a **State Machine** cuida da interação em tempo real; a **Time
  Machine** cuida do controle baseado em tempo. E sobre a timeline de áudio embutida, ele mesmo diz:
  *"se você precisa de tratamento de áudio complexo, use um software dedicado e controle-o pelo
  Chataigne"*. Reconhecer o limite do próprio recurso, na documentação, é padrão que vale copiar em
  `design/FUNCOES/README.md`.

Origem do nome: "chataigne" é castanha em francês; a história envolve um caso amoroso e o autor achou
engraçado. Registrado porque explica `.noisette` ("avelã") como extensão do arquivo de projeto.

---

## 2. A tela: sete painéis, e uma decisão de layout

A documentação enumera os painéis, que é o que `chataigne.md` cobre pelo lado do código
(`ShapeShifterManager`). O que a doc acrescenta é **o papel declarado de cada um**:

| # | Painel | Papel, nas palavras da doc |
|---|---|---|
| 1 | **Module** | "seu ponto de partida". Ao criar um projeto, lista-se aqui **todos** os softwares e dispositivos com que se vai interagir, antes de qualquer regra |
| 2 | **State Machine** | as regras de interação em tempo real |
| 3 | **Time Machine** (ou Sequence Editor) | sequências em timeline: triggers e animação de parâmetro no tempo |
| 4 | **Inspector** | "seu painel principal de edição, você vai passar muito tempo aí". **Qualquer coisa selecionável no software aparece nele em detalhe**; trocar a seleção troca o conteúdo |
| 5 | **Logger** | "seu amigo verboso". Diz o que deu certo e o que falhou, e mostra coisas úteis como **os endereços IP da máquina quando um módulo de rede é criado**. O usuário também pode logar mensagens e valores próprios |
| 6 | **Help** | mostra hotkeys úteis e como usar **o item sob o cursor** |
| 7 | **Warnings** | usado **ao carregar um arquivo**, para conferir o que quebrou: arquivos faltando, erro de script, links quebrados |

O framework de UI se chama **Organic UI** e tem um mecanismo **ShapeShifter**: o layout de painéis é
livremente rearranjável e **salvável em layouts diferentes conforme o que se está fazendo**.

Três coisas para levar:

1. **A ordem de trabalho está embutida na numeração dos painéis**: declarar os dispositivos primeiro,
   depois as regras, depois o tempo. O nosso equivalente é: patchbay → mapping → timeline.
2. **O Logger cospe os IPs da máquina sozinho** quando um módulo de rede nasce. Custo quase zero,
   resolve a pergunta mais frequente de qualquer show em rede — e o Spellcaster tem sACN, Art-Net e
   OSC, os três com o mesmo problema.
3. **O painel Warnings é explicitamente "para quando você abre um arquivo"**. Um `.spell` que abre com
   fixture faltando, arquivo `.ild` sumido ou universo duplicado precisa exatamente disso, e
   `chataigne.md` já detalha o mecanismo por trás (`WarningTarget`, `WarningReporter`).

---

## 3. Cheat Sheet — a tabela copiada

`chataigne.md` já tem os **comandos de menu** com tecla (`Ctrl+N`, `Ctrl+B` cria cue, `Shift+PageUp`
navega cue, `Ctrl+E` modo de edição do dashboard, ...), lidos de
`OrganicMainComponentCommands.cpp` e `TimelineAppCommands.cpp`. Não repetido aqui.

O que **só** existe na documentação são os **gestos de mouse e os modificadores de arraste** — que não
passam pelo sistema de comandos do JUCE e portanto não aparecem em nenhum arquivo de código de
atalhos. É a parte mais valiosa desta página inteira.

### Linha de comando

```
./Chataigne [-r] [-f arquivo] [-headless] [-forceGL / -forceNoGL] [<arquivo>]
```

| Flag | Efeito |
|---|---|
| `-r` | **reseta as preferências** |
| `-f <arquivo>` | abre o arquivo (funciona também só pondo o nome no fim, sem `-f`) |
| `-headless` | roda **sem GUI, sem janela** |
| `-forceGL` / `-forceNoGL` | força o valor de "use opengl renderer"; `-forceNoGL` serve para contornar driver de vídeo problemático |

Para o Spellcaster Lite no Raspberry Pi, `-headless` e "abrir arquivo pelo argumento posicional, sem
flag" são o contrato mínimo — e `-r` é o botão de pânico que todo produto com preferências salvas
precisa ter e quase nenhum tem.

### Editar parâmetros — a regra geral do produto

> "Botão direito em qualquer parâmetro do Inspector para revelar um novo mundo de possibilidades!"

E o que aparece: **mudar a faixa** do parâmetro (quando permitido), **mandá-lo para o Dashboard**,
**copiar o endereço de script ou de controle OSC dele**. A doc generaliza: *"em geral, no software,
tentar clicar com o botão direito e ver se há mais opções é uma boa ideia"*.

Isto casa com o que `chataigne.md` achou no código (`MainComponent.cpp:64-141`: "Add & Link to Custom
Variable…", "Add & Link to Sequence…"). Junto, dá a regra: **todo parâmetro da tela é o ponto de
partida de mapear, animar, expor e endereçar.** Nada disso começa num painel dedicado.

### Seleção e transferência de conteúdo

| Gesto | Efeito |
|---|---|
| `Ctrl` + clicar num item | **alterna** o estado de seleção daquele item |
| `Shift` + clicar num item | seleciona tudo **até** ele |
| `Alt+O` | **importa um arquivo LilNut** e adiciona o conteúdo ao projeto existente |
| `Alt+S` | **exporta a seleção atual** para um arquivo LilNut |

O conteúdo que trafega no LilNut é declarado: **Modules, States, Custom Variables, Module Router e
Sequences**. Ou seja: um formato de troca parcial, que carrega pedaços de projeto entre projetos, com
lista fechada do que é transportável. O `.spell` precisa do análogo — exportar "só estas cenas e este
patch" sem exportar o show inteiro.

### Inspector

| Gesto | Efeito |
|---|---|
| `Shift` + clique no cabeçalho de um Container | dobra/desdobra **todos os filhos** |
| **`Alt` + arraste** num slider ou label numérico | **diminui** a sensibilidade do arraste |
| **`Shift` + arraste** num slider ou label numérico | **aumenta** a sensibilidade do arraste |

Note a diferença em relação ao Ableton e ao Resolve, onde só existe "`Shift` = fino"
(`ableton12.md` §10.6; `resolve20-guia.md` §4): o Chataigne dá **os dois lados**, fino e grosso, com
dois modificadores. Para um parâmetro DMX de 0-255 em que às vezes se quer 1 passo e às vezes 100,
isso não é luxo.

### State Machine (a view de nós)

| Gesto | Efeito |
|---|---|
| **Botão do meio arrastando**, ou `Alt` + arraste | navegar no canvas |
| **`F`** | enquadra a view no centro de **todos** os States |
| **`H`** | volta ao **centro absoluto** da view (home) |
| Roda do mouse | rola para cima e para baixo ("pode mudar no futuro") |
| **`Shift` + roda** | zoom |
| `Shift+Enter` dentro de um comentário | quebra de linha |
| `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | copiar, colar, duplicar — **vale para todos os itens de listas e views**: States, Mappings, Actions, Modules, Sequences |

**`F` e `H` são duas coisas diferentes de propósito**: "enquadre o que existe" e "volte à origem". Um
patchbay sem os dois é um patchbay em que se perde o trabalho. E note a inversão em relação ao
Ableton e ao Resolve: aqui a **roda pura rola** e **`Shift`+roda dá zoom**, no canvas de nós.

### Time Machine — manipulação da timeline

| Gesto | Efeito |
|---|---|
| **Arrastar a barra azul** na horizontal / vertical | **zoom e foco de tempo ao mesmo tempo** |
| **Botão direito na barra azul** | reseta a view para a sequência inteira |
| Botão direito **arrastando** na barra azul | zoom seletivo num trecho (absoluto) |
| Botão direito arrastando **nos números** da régua | zoom seletivo num trecho (relativo) |
| **`Shift` + arrastar a agulha do tempo** | **snap** da agulha nos elementos da timeline (cues, triggers, outras chaves de mapping…) |
| **Duplo-clique nos números da régua** | **cria um Time Cue** |
| `Shift` + arrastar um cue | move com snap (barra de tempo, triggers, outras chaves de mapping…) |

A "barra azul" é o mesmo objeto que o Overview do Ableton (`ableton12.md` §3) e o Resolume chama de
cunha (`resolume-quickstart.md` §4): **uma tira acima da timeline que é ao mesmo tempo o mapa e o
controle de zoom, com o mesmo gesto de arrastar em dois eixos**. Três produtos independentes chegaram
nisso. Não há motivo para inventar outra coisa.

O que é **exclusivo do Chataigne** e vale mais: **botão direito reseta a view**, e **botão direito
arrastando faz zoom no trecho**. O botão direito da régua é um vocabulário inteiro que Ableton e
Resolve deixam vazio.

### Camada de Mapping (e Mapping 2D) — edição de curva

| Gesto | Efeito |
|---|---|
| **Duplo-clique no vazio** | cria uma chave naquela posição |
| **Duplo-clique sobre a curva** | adiciona um ponto **mantendo a forma geral intacta** |
| **`Shift` + arrastar uma chave** | mantém o **valor**, move só a posição |
| **`Alt` + arrastar uma chave** | mantém a **posição**, move só o valor |
| `Shift+Alt` + arrastar uma chave | move só a posição, **com snap** nos elementos das outras camadas (barra de tempo, cues, triggers, outras chaves) |
| **`Ctrl` + clique na curva** | **troca o tipo de easing** daquele segmento |
| **`Ctrl+Shift` + arraste** | **desenha a curva à mão** |

Esta é a tabela mais diretamente aproveitável do arquivo inteiro, para a frente `daw-arranjo`:

- **"adiciona um ponto mantendo a forma intacta"** é uma operação distinta de "cria uma chave". Duas
  intenções, dois gestos, o mesmo duplo-clique com alvo diferente. O Ableton não tem a primeira
  (`ableton12.md` §6: duplo-clique no fundo cria breakpoint, e ponto).
- **`Shift` trava valor, `Alt` trava posição** — dois eixos, dois modificadores, simétricos e
  memorizáveis. O Ableton usa `Shift` para "restringe a um eixo", sem dizer qual: pior.
- **`Ctrl`+clique cicla o easing direto no segmento**, sem menu, sem inspector. E os tipos existentes
  estão em `chataigne.md` (`LINEAR, BEZIER, HOLD, SINE, ELASTIC, BOUNCE, STEPS, NOISE, PERLIN`).
- **`Ctrl+Shift`+arraste desenha**, e o traço vira automação editável — é o Draw Mode do Ableton
  (tecla `B`) sem precisar de modo.

### Módulos

| Gesto | Efeito |
|---|---|
| **Arrastar o módulo para dentro de um State** | abre menu para usá-lo **automaticamente como input ou output** de uma Action ou de um Mapping |
| Clicar nas **setas de atividade** do módulo | alterna Log Incoming / Log Outgoing daquele módulo |

O primeiro é o gesto que a frente `browser-dnd` precisa entender: **arrastar não cria "uma cópia do
módulo"; abre um menu perguntando qual papel ele vai ter no destino**. Drop com desambiguação, em vez
de drop com comportamento adivinhado.

---

## 4. Módulos e Router — o que a doc acrescenta

### Anatomia declarada de um módulo

Seis seções fixas no Inspector, na ordem: **Header** (enable/disable — módulo desabilitado "não
atualiza nem envia nada" — mais os toggles Log Incoming/Outgoing), **Parameters** (host, porta, nome
do device…), **Values** (o que o módulo recebe; **alguns módulos não têm nenhum** se não recebem ou se
o Input está desativado), **Scripts**, **Command Tester** e **Templates**.

Dois pontos:

- **Command Tester**: "envia comandos manualmente para verificar se a comunicação está funcionando;
  **não afeta o resto do software**". Botão Trigger, mais a opção **Auto Trigger**, que reenvia o
  comando toda vez que um parâmetro dele muda. Um banco de teste que não suja o show é exatamente o
  que falta a quase todo software de luz — e é a resposta à pergunta "o projetor está respondendo?"
  sem armar saída.
- **Templates**: customizar um módulo para um uso específico **sem escrever um módulo próprio** —
  cria-se um Template a partir de um comando base, escolhe-se **quais campos são editáveis e quais
  não**, e define-se o comportamento padrão de mapeamento. É o "preset de comando com campos
  travados", que é como uma equipe evita que o operador da noite mude o que não deve.

### Catálogo de módulos, como declarado

- **Protocolo**: OSC, OSCQuery, MIDI, DMX, Serial, UDP, TCP Client, TCP Server, HTTP, Websocket
  Client, Websocket Server, MQTT, PJLink, PosiStageNet, Ableton Link
- **Hardware**: Sound Card, Wiimote, JoyCon, Keyboard, Mouse, Gamepad, Kinect V2, StreamDeck,
  LoupeDeck, GPIO
- **Software**: DLight, HeavyM, MadMapper, Millumin, QLab, Reaper, Resolume, Watchout, Powerpoint
- **Generator**: Metronome, Signal — **geradores são módulos**, não uma categoria à parte
- **System**: Time, OS

Note o que **não** está aqui: **sACN e Art-Net não aparecem na lista**. Só "DMX". É espaço real.

### Module Router

"Uma ferramenta útil quando você tem muitos mapeamentos a fazer de um módulo para outro." **Um router
liga uma entrada a uma saída só**, mas dá para criar quantos quiser. E o aviso: para transferir dados
direto entre **módulos do mesmo tipo**, use o recurso **pass-through** do módulo, que é otimizado e
mais simples.

Duas ferramentas para o mesmo problema, com a documentação dizendo qual usar quando. Vale copiar a
prática, não só o recurso.

### `module.json` — o formato de módulo customizado

Pasta em `<Documents>/Chataigne/modules/<nome>/`, com `module.json` obrigatório, um script `.js` de
lógica quase sempre, e opcionalmente um `icon.png` de **32×32**. Depois de mexer no JSON:
**File > Reload custom modules — e apagar e recriar o módulo**. Também existe **módulo local**: uma
pasta `modules` ao lado do arquivo `.noisette` faz aquele módulo existir só enquanto aquele projeto
está aberto.

Campos, resumidos:

| Grupo | Chaves |
|---|---|
| Metadados | `name`, **`type`** (qual módulo base estender), `path` (submenu), `version`, `description`, `url`, `downloadURL` |
| Sobrescrita do base | `hasInput`, `hasOutput`, `defaults`, **`hideDefaultParameters`** (array de nomes curtos), `hideDefaultCommands`, `alwaysShowValues` |
| Conteúdo | `parameters`, `values`, `commands`, `scripts` |
| Por comando | `menu`, **`callback`** (função do script), **`setupCallback`** (criação dinâmica de comandos), `parameters` |
| Tipos de dado | `Container`, `Boolean`, `Float`, `Integer`, `Enum`, `String`, `File`, `Target`, `Color`, `Point2D`, `Point3D` |
| Por dado | `readOnly`, `shortName`, `description`, `min`/`max`, `default`, **`dependency`** |
| UI de float | `ui`: `stepper`, `slider`, `label`, **`time`** |

Três achados que valem para o nosso editor de fixture (`fixtures/`) e para o patchbay:

1. **`type` = herdar de um módulo base.** Um módulo customizado é uma especialização declarativa de um
   existente, não um plugin do zero. Uma personality de fixture deveria funcionar igual: herda de
   "moving head 16 canais" e sobrescreve.
2. **`dependency`** — todo dado pode declarar `{source, value, check ∈ {equals, notEquals, lessThan,
   greaterThan}, action ∈ {show, enable}}`. **UI condicional é dado do arquivo, não código.** É como
   um canal "gobo rotation" só aparece quando "gobo" está num certo valor, sem escrever uma linha.
3. **`ui: "time"` como tipo de widget de float.** Tempo não é um float qualquer.

E a dica prática, que revela a arquitetura: *"para descobrir o nome curto de um parâmetro, passe o
mouse por cima dele e olhe o control address"*. **Todo parâmetro tem endereço textual, e a UI o
exibe.** Mesmo espaço de nomes para script, OSC e JSON — é o `spellcaster.core.registry` do
`CLAUDE.md`, e é o que `orquestrador.md` já assume.

---

## 5. State Machine — o que a doc acrescenta

`chataigne.md` cobre `Action.h`, `Condition`, `Consequence`, `Mapping` e a lista de filtros. Aqui só o
que a documentação diz e o código não mostra.

### A regra do State Network

> "A qualquer momento, há **apenas 1 state ativo dentro de um state network**."

E a consequência declarada: **não ligar states entre si** é o que permite ter vários states ativos ao
mesmo tempo — quantos state networks se quiser. Ou seja: **exclusividade é opt-in, expressa pelas
transições, não uma propriedade global**. Um grafo desconectado é um grupo independente.

Isto responde diretamente uma pergunta em aberto do Spellcaster: cena de luz em universos diferentes
tem que poder ser exclusiva dentro do universo e independente entre universos. **A ligação é a
declaração de exclusividade.** É a mesma regra do layer do Resolume por outro caminho
(`resolume-quickstart.md` §2).

Transições têm dois papéis simultâneos, declarados: transferir a ativação de um state para outro
(agindo **como uma Action, inclusive com consequências próprias**, para dar comportamento diferente
conforme de onde se veio) e ligar states num network.

### Actions — as 5 condições, nomeadas

**From Input Value** (a mais usada), **Scripts**, **Group**, **On Activate**, **On Deactivate**.

O detalhe que só está na doc: **as condições `onActivate` disparam quando o projeto é carregado, se o
state que as contém estiver ativo** — e por isso "podem ser usadas como ação de inicialização". Um
único mecanismo cobre "ao entrar neste estado" e "ao abrir o arquivo". Barato e certo.

E: `onActivate`/`onDeactivate` reagem a **ativado/desativado**, não a **habilitado/desabilitado** — a
doc grifa a diferença. Dois eixos distintos, como visibilidade × mute no Resolve
(`resolve20-guia.md` §1).

Feedback visual declarado: **cada condição validada fica verde e volta a cinza ao ser invalidada**.
O estado da regra é legível na própria regra, sem abrir log.

Consequências: quantas se quiser, todas disparadas de uma vez ("controle sincronizado de módulos
diferentes"), com duas opções de tempo — **delay** depois da validação e **stagger**, que espaça o
disparo de cada consequência num intervalo regular. Stagger em cascata de luz é meio caminho de um
efeito de chase sem escrever efeito nenhum.

### Mappings — wildcards de string

O que a doc acrescenta ao que `chataigne.md` já traz de `Mapping.h`:

- **O símbolo de raio ao lado de cada input decide se aquele input dispara o mapeamento.** Vários
  inputs, um gatilho — a doc explicita para que serve.
- **Filtros escolhem em quais canais agem**, pelo menu "Channels" do cabeçalho do filtro.
- **Wildcards em parâmetros de string**: `{input:1}`, `{input:2}`… são substituídos pelos valores de
  entrada. `"My value is {input:1}, and second value is {input:2}"` vira
  `"My value is 0.53, and second value is 127"`.

Interpolação de string por template no parâmetro é a diferença entre "posso mandar um OSC dinâmico" e
"preciso de um script". Custo: um regex.

### Multiplex — o achado da página

O problema: vários valores que precisam do mesmo filtro, ou uma fileira de botões/faders que precisam
do mesmo tratamento — e a alternativa seria duplicar N actions e N mappings.

Como funciona: um **Multiplex** tem um parâmetro **count** que define quantas iterações ele trata.
Dentro dele criam-se **listas**, todas com o comprimento do count. Actions e Mappings colocados dentro
do Multiplex **ganham capacidades novas**: uma lista pode ser Input de condição, e um parâmetro de
Consequence, de Mapping Output ou de Mapping Filter pode ser ligado a um elemento da lista.

O mecanismo: quando **um** elemento de uma lista muda, dispara o processamento da Action/Mapping que
tem aquela lista como input — **e o processo carrega o índice do elemento que disparou**, de ponta a
ponta. Assim "o mesmo tratamento para muitos itens, com saída específica para cada um".

Wildcards do Multiplex, em parâmetros de string:

| Wildcard | Vira |
|---|---|
| `{index}` | o índice, base 1 |
| `{index0}` | o índice, base 0 |
| `{input:1}` | o primeiro valor da entrada do mapping |
| `{list:names}` | o elemento de mesmo índice na lista "Names" (conversão camelCase) |

`"Hello {list:names}, you're patient number {index}"` → `"Hello Leon, you're patient number 5"`.

E o helper de preenchimento: **Fill… > From expression**, com o caminho do primeiro item copiado por
botão direito → "Copy value" e o índice trocado por wildcard —
`"/modules/OSC/values/track{index}_x"` preenche a lista inteira na ordem.

**Isto é a peça que o Spellcaster mais precisa e que nenhum dos outros cinco apps auditados tem.**
Uma regra escrita uma vez, aplicada a 24 fixtures idênticas, com o índice propagando até a saída —
é a diferença entre patchear um rig de 24 pares e patchear um par 24 vezes. Registrar como candidata
para `orquestrador.md`.

### Conductor — a lista de cues que o Chataigne tem

Um Conductor cria uma **cue list de ações sequenciais**. Cada cue pode disparar várias consequências
(como uma Action) **ou ser ligado a uma sequência**. O Conductor guarda o **cue atual (fundo roxo)** e
o **próximo cue (fundo laranja)**; ao ser disparado, ele dispara o próximo cue e incrementa os dois.

- **Se houver uma condição no Conductor, ela é o que dispara o próximo cue** — a doc dá os exemplos:
  uma tecla, ou um trigger OSC. Ou seja, **o GO é uma condição, não um botão especial**.
- Cue ligado a sequência: ao ser disparado, **toca a sequência ligada**. Com a opção
  **"Auto Next On Finish"**, ao terminar a sequência ele **dispara o próximo cue sozinho**.

`chataigne.md` § Sequências afirma, comparando com `design/SHORTCUTS.md`, que "não há marcadores
separados de cues" e que o transporte é mais pobre que o nosso. **Isso continua verdade para a
sequência**, mas o Conductor é o objeto de cue list que faltava naquele mapa — ele mora na State
Machine, não na Time Machine. Correção de escopo, não contradição.

Para nós, o modelo é exatamente o que a função "cues" do `design/TEMAS.md` pede: lista ordenada,
cue atual e próximo cue destacados por cor, GO como condição mapeável, e "cue = disparo + sequência
opcional, com encadeamento automático no fim".

---

## 6. Time Machine — e a comparação com o Ableton

### O que a doc acrescenta

Sequências: "quantas quiser, controladas independentemente" a partir do painel Sequence. Uma sequência
é "um objeto baseado em tempo, com timeline própria, contendo um grupo de layers".

**Cues** (Interface and Navigation): marcas na timeline usadas para **tocar dali** ou **ir para o
próximo**. E o uso declarado da opção de pausar: *"pausar a timeline ao atingir um cue, o que é
conveniente em shows semi-interativos — esperar o ator terminar a fala, ou a bailarina entrar em cena,
antes de continuar"*. Este é o `cueAction = PAUSE` de `chataigne.md` (`TimeCue.h:21-26`) com a
justificativa que faltava. **É o modelo de "show teatral" contra o de "show cronometrado", e a
diferença é um enum de três valores.**

**Zoom e scroll**: clicar e arrastar o retângulo azul acima da timeline, para cima/baixo e
esquerda/direita, navega e dá zoom (detalhado em §3).

### As seis camadas, com o que só a doc diz

| Camada | O que a doc acrescenta |
|---|---|
| **Trigger Layer** | "triggers que são como actions, mas disparam no instante em que estão postos". A doc admite: "vão ganhar mais recursos em versões futuras" |
| **Mapping Layer** | contém **uma automação — uma animação baseada em curva** — usada como **input de um mapping**. "Você pode usar essa camada como um mapping de state e adicionar filtros e outputs a ela" |
| **Mapping 2D Layer** | anima **um ponto ao longo de um caminho 2D**. O caminho é criado à parte; a timeline anima **a posição sobre ele** |
| **Color Layer** | animar cor no tempo; **dois tipos de interpolação, Linear e Hold**, alternados por **`Ctrl`+clique nas chaves** |
| **Audio Layer** | vários clips de áudio tocados no tempo. "O suporte é limitado"; **exige um módulo Sound Card** no projeto para tocar |
| **Sequence Layer** | "controlar sequências dentro de sequências. dentro de sequências. dentro de sequências." — aninhamento sem limite declarado |

**A frase mais importante da Time Machine inteira**: a Mapping Layer **é um mapping**. A curva não
"controla um parâmetro"; ela é a **entrada** de uma cadeia input → filtros → outputs idêntica à do
State Machine. Consequência prática: a mesma curva pode passar por Damping, Crop, Math, OneEuro,
e sair em vários comandos ao mesmo tempo. **Automação e mapeamento não são dois subsistemas.**

Para o Spellcaster: as **keys da timeline devem ser entrada do patchbay**, não um caminho paralelo até
a saída. Isso resolve "a timeline manda 0-255 mas eu quero uma curva de dimmer" sem inventar um
segundo lugar para curvas.

**Mapping 2D separa o caminho da posição sobre o caminho** — e é exatamente o modelo certo para
laser: a figura `.ild` é o caminho, a timeline anima o avanço sobre ele.

### The Recorder — o "gravar dmx" do Chataigne, documentado

Na Mapping Layer: escolher um **Input value**, ativar o parâmetro **Arm**, e começar a tocar. O valor
**aparece em vermelho na camada** enquanto grava e, **ao parar a sequência, converte-se automaticamente
numa curva editável**. O exemplo da doc é gravar um ruído perlin do módulo Generator.

Quatro decisões prontas para a frente `gravar-dmx`:

1. **Arm é parâmetro da camada**, não modo global. Grava-se numa linha por vez, explicitamente.
2. **Vermelho durante a gravação**, e o vermelho é o dado ainda cru.
3. **A conversão em curva editável acontece ao parar**, não durante — não se edita o que ainda está
   sendo gravado.
4. **O resultado é curva editável, não trilha imutável.** Combinado com a simplificação interativa que
   `chataigne.md` acha no código (`launchInteractiveSimplification`, `Automation.h:46-52`) e com o
   Simplify Envelope do Ableton (`ableton12.md` §6, p.500), os três produtos concordam: **gravação a
   taxa alta tem que virar poucos pontos editáveis, ou não serve.**

### Ableton Live 12 × Chataigne — a timeline, lado a lado

| | Ableton Live 12 | Chataigne | Para nós |
|---|---|---|---|
| Régua | duas simultâneas: compasso-batida-16º e minuto-segundo-ms (p.160-162) | uma; `fps` e `bpmPreview` são parâmetros da sequência (`chataigne.md`) | precisamos das duas: cue em relógio, key em quadro |
| Mapa/overview | Overview: arrastar horizontal rola, vertical dá zoom; duplo-clique volta ao todo (p.160-161) | barra azul: arrastar nos dois eixos; **botão direito reseta**; botão direito arrastando dá zoom no trecho | o botão direito da barra é ganho líquido do Chataigne |
| Zoom por teclado | `+`/`-`, `Z` na seleção, `X` volta um passo (p.163, p.999) | não há; só gesto de mouse | copiar o `Z`/`X` do Ableton |
| Scroll | `Shift`+roda horizontal, `Ctrl`+roda zoom, `Alt`+roda altura de track (p.163, p.991) | na State Machine: roda rola, `Shift`+roda dá zoom | **conflito real**: as duas convenções são opostas. Decidir uma e valer nas duas telas |
| Follow / playhead centrado | switch Follow, que **pausa ao editar e volta ao reiniciar** (p.163) | `viewFollowTime` é parâmetro salvo da sequência (`chataigne.md`) | o comportamento de auto-pausa do Ableton é o que falta ao Chataigne |
| Marcadores | **Locators**, disparáveis, mapeáveis, com "Loop to Next Locator" e "Set Song Start Time Here" (p.166-167) | **Time Cues**, com `cueAction ∈ {NOTHING, PAUSE, LOOP_JUMP}` e `playFromHere`; no Chataigne o cue ainda carrega **condições** (`ChataigneCue.h:22`) | **cue condicional é do Chataigne e não tem paralelo**; o "loop entre dois marcadores" é do Ableton |
| Criar marcador | botão Set Locator, ou menu (p.166) | **duplo-clique nos números da régua**, ou `Ctrl+B` | duplo-clique na régua é mais barato |
| Loop | loop brace com `Ctrl+L`, setas ajustam, `Ctrl+↑/↓` dobra/divide (p.169-170) | `loopParam` na sequência; o pulo de loop é **um cue com `LOOP_JUMP`** | brace explícita do Ableton para ensaio; cue com salto para estrutura de show |
| Snap | grade `Ctrl+1..5`; **`Alt` inverte o snap durante a ação** (p.175) | `autoSnap` na sequência; **`Shift`+arraste dá snap na agulha e nos cues** | Ableton: snap por grade de tempo. Chataigne: snap **em elementos**. Precisamos dos dois |
| Criar key | duplo-clique no fundo cria breakpoint (p.497) | duplo-clique no vazio cria chave; **duplo-clique na curva adiciona ponto sem mudar a forma** | a segunda operação não existe no Ableton e deveria existir |
| Mover key | `Shift` restringe a um eixo, sem dizer qual (p.498-499) | **`Shift` trava o valor, `Alt` trava a posição, `Shift+Alt` = posição com snap** | copiar o Chataigne: dois eixos, dois modificadores |
| Curva do segmento | **`Alt`+arraste curva; `Alt`+duplo-clique volta a reta** — uma forma de curva só (p.499) | **`Ctrl`+clique cicla o easing**: `LINEAR, BEZIER, HOLD, SINE, ELASTIC, BOUNCE, STEPS, NOISE, PERLIN` | o vocabulário de easing do Chataigne; **`NOISE` e `PERLIN` transformam um segmento em gerador** |
| Desenhar | Draw Mode `B` (e `B` segurado é momentâneo) (p.496) | **`Ctrl+Shift`+arraste**, sem modo | o modificador dispensa o modo; a tecla dispensa o modificador. Ter os dois é razoável |
| Simplificar | **Simplify Envelope** sobre uma seleção (p.500) | simplificação interativa após gravação (`chataigne.md`) | obrigatório dos dois lados |
| Gravar | Automation Arm global + Session Record; modos touch (mouse) e latch (controlador) (p.491-494) | **Arm por camada**, valor em vermelho, vira curva ao parar | Arm por camada é mais claro; os modos touch/latch do Ableton são o refinamento |
| Camada de áudio | é o produto inteiro | "suporte limitado", e a doc manda usar software dedicado | mesma postura do Spellcaster com áudio |
| Aninhamento | não existe: um clip não contém arranjo | **Sequence Layer**: sequência dentro de sequência, sem limite | ganho do Chataigne — "esta cena é uma mini-timeline" |
| Cue list | **Follow Actions** por clip/scene, com Chance A/B, Linked/Unlinked, Jump (p.361-362) | **Conductor**: cue atual roxo, próximo laranja, GO como condição, Auto Next On Finish | Follow Action é probabilístico; Conductor é determinístico. **Show quer o Conductor** |
| Escopo do que se ouve | Session ↔ Arrangement mutuamente exclusivos, com "Back to Arrangement" (p.192-193) | State ativo × sequência tocando são independentes | o "Back to Arrangement" do Ableton é a solução do conflito; nós teremos o mesmo problema |

---

## 7. Custom Variables e Morpher

**Custom Variables** são "o lugar para guardar, modificar e recuperar dados próprios". Estruturadas em
**Groups**, e **cada group tem presets**. A doc admite: "o conceito é meio abstrato para quem não
programa", e por isso dá dois exemplos:

1. **Lógica de jogo**: um group com `Score` e `HighScore`; uma action incrementa `Score` a cada aperto
   de botão; outra action verifica se chegou a 10, mostra "YOU WIN" e zera. Estado de show que não é
   estado de aparelho.
2. **Presets de sistema de partículas**: 4 variáveis num group, 2 presets, e **um Mapping que pega um
   sinal de entrada e o usa para interpolar entre os dois presets**.

O segundo é o modelo direto de "crossfade entre duas cenas de luz" — e a interpolação é feita por
**comandos de Special Module**, isto é, por um módulo, com a mesma gramática de tudo.

**Morpher**: pondo o **Control Mode** do group de Custom Variables em **2D Voronoi**, abre-se o painel
Morpher, onde os presets são **posicionados num plano 2D**. Um **alvo branco** define o peso de cada
preset conforme a distância até ele **e conforme o layout global** (algoritmo de proximidade por
Voronoi). Depois há **attraction** e **decay** para "comportamentos ainda mais imprevisíveis".

Origem declarada: era um projeto à parte, um interpolador 2D entre múltiplos presets, que acabou
fundido no sistema de Custom Variables.

Para nós: **um pad XY que mistura N cenas por proximidade** é um controle de show inteiro num widget.
E a arquitetura ensina mais que o recurso: **o Morpher não é um objeto novo — é um "Control Mode" de
um group de variáveis.** Trocar o modo de controle de um container muda a UI e a semântica sem criar
tipo novo. Combinado com o `dependency` do `module.json` (§4), é o mesmo princípio duas vezes.

---

## 8. Dashboard

"Uma maneira de criar uma interface customizada importando, posicionando e estilizando qualquer
componente da sua composição num canvas. Você pode criar quantos dashboards quiser."
**`Ctrl+E` alterna Edit mode ↔ Play mode** (já em `chataigne.md`).

A doc é curta e ainda avisa: *"um novo dashboard, o 'Golden Board', está no roadmap"*.

### Web Dashboard — a parte útil

Em **File > Project Settings**, habilita-se o **Dashboard Server**. Com ele ligado, os dashboards são
acessíveis **de qualquer navegador da rede**, pelo IP da máquina (ou `127.0.0.1` na própria) e a porta.
**Porta padrão: 9999** — `http://127.0.0.1:9999`.

Endereçamento e parâmetros de URL:

| URL | Efeito |
|---|---|
| `http://<ip>:<porta>/` | abre o dashboard server |
| `http://<ip>:<porta>/#/<dashboard>` | abre **um dashboard específico** direto |
| `...?disableMenu` | esconde o menu |
| `...?disableList` | esconde a lista de páginas de dashboard |
| `...?disableMenu&disableList` | os dois — o modo "tablet do operador" |

Isto é literalmente o que o Spellcaster quer para a GUI web com skins: **a mesma tela, servida por
HTTP, com o chrome removível por query string**, para um tablet no palco mostrar só os botões. Custo:
duas flags. E a porta 9999 é candidata a colisão — anotar no `design/DECISOES.md` junto com as portas
das frentes.

---

## 9. Scripts — o mínimo, já que a referência foi truncada

Cinco lugares onde script existe, e a doc é explícita que **os métodos e callbacks disponíveis mudam
conforme onde o script está**: **Module Scripts, Condition Scripts, Consequence Script, Mapping Filter
Scripts, Mapping Output Script**.

O que vale registrar do fluxo de autoria (e é o que o Spellcaster deve imitar se um dia tiver script):

- Ao criar um script, o arquivo **já nasce preenchido com conteúdo gerado dinamicamente conforme o
  objeto de onde foi criado** — um script criado num módulo OSC vem com o genérico, mais o específico
  de módulo, mais o específico de OSC.
- **Compilação e interpretação em tempo real**: salvar o arquivo recarrega no app.
- **Estado do script é visível em dois lugares**: ponto **verde** no Inspector do script quando compila,
  ponto **vermelho** mais um warning quando falha — e o Logger mostra **o erro e a linha**.
- Funções comuns, nenhuma obrigatória (o app "otimiza o script conforme quais existem"): `init()`
  logo após carregar; `update(deltaTime)` chamado na taxa do parâmetro "Update rate" — que **só
  aparece na UI quando a função existe** —, com `deltaTime` em segundos e taxa em Hz, ajustável por
  `script.setUpdateRate(rate)`; `messageBoxCallback(id, result)`.
- Os tipos referenciados: Trigger e Parameters (Float, Integer, Boolean, String, Color, Target, Enum,
  File, Point2D, Point3D), Container, Manager, States, Automation, Commands, e os objetos `script`,
  `root`, `local`, `util`.

**"O parâmetro só aparece na UI quando a função que o usa existe"** é a versão em script do
`dependency` do `module.json`. Terceira vez que o mesmo princípio aparece: **a UI é consequência
declarada dos dados, nunca escrita à mão.**

---

## Adaptação ao Spellcaster

Referência cruzada: `design/FUNCOES/orquestrador.md` (`spell graph`, tema PATCHBAY, "modo Chataigne").

### O que entra no orquestrador / patchbay

| Da documentação | O que vira no Spellcaster |
|---|---|
| **Painel Module como painel nº 1**, antes das regras | o patchbay começa declarando saídas e entradas; nenhuma regra existe antes disso |
| **Arrastar o módulo para dentro de um State abre menu "input ou output?"** | drop no patchbay pergunta o papel; nunca adivinha (frente `browser-dnd`) |
| **Command Tester que não afeta o resto do software** | "testar esta saída" sem armar o show — resolve o furo que `resolume.md` aponta ("existir Lumiverse já é enviar") |
| **Templates de comando com campos travados** | preset de comando em que o operador da noite só mexe no que foi liberado |
| **Module Router: uma entrada, uma saída, quantos quiser; e pass-through para o caso trivial** | duas ferramentas declaradas, com a doc dizendo qual usar quando |
| **`module.json`: `type` herda de um módulo base; `dependency` faz UI condicional; `ui:"time"`** | personality de fixture herdando de um tipo, com canais que aparecem conforme o valor de outro canal — é o que falta ao Resolume (`resolume.md` § O que NÃO copiar) |
| **Endereço textual visível ao passar o mouse em qualquer parâmetro** | o registry exposto na UI; um espaço de nomes para GUI, OSC, CLI e MCP |
| **Botão direito em parâmetro = mudar faixa, mandar ao Dashboard, copiar endereço** | mapear e expor começam no widget, não num painel |
| **Multiplex: uma regra, N itens, índice propagado até a saída** | uma regra para 24 fixtures idênticas, com saída específica por índice. **Nenhum outro app auditado tem isto** |
| **Wildcards `{index}`, `{index0}`, `{input:1}`, `{list:nome}` em parâmetros de string** | endereço OSC e nome de alvo montados por template, sem script |
| **Regra do State Network: exclusividade é a ligação, não uma propriedade global** | cena exclusiva dentro do universo, independente entre universos, expressa pelo grafo |
| **`onActivate` dispara também ao carregar o arquivo** | um mecanismo cobre "ao entrar na cena" e "ao abrir o show" |
| **Condição validada fica verde e volta a cinza** | estado da regra legível na regra |
| **Consequências com `delay` e `stagger`** | cascata de luz sem escrever efeito |
| **Conductor: cue atual roxo, próximo laranja, GO como condição, cue→sequência, Auto Next On Finish** | a cue list do Spellcaster, inteira |
| **`-headless`, arquivo como argumento posicional, `-r` para resetar preferências** | contrato de CLI do Spellcaster Lite no Pi |
| **Web Dashboard: porta 9999, `#/<nome>`, `?disableMenu&disableList`** | GUI web servida na rede, com chrome removível por query string, para o tablet do palco |
| **Logger cospe os IPs da máquina ao criar módulo de rede** | sACN/Art-Net/OSC dizendo em qual interface estão, sem o operador perguntar |
| **Painel Warnings explicitamente "para quando você abre um arquivo"** | `.spell` que abre com `.ild` faltando ou universo duplicado avisa numa lista clicável |

### O que entra na timeline (`daw-arranjo`)

Gestos de curva, na íntegra: duplo-clique no vazio cria chave; **duplo-clique na curva adiciona ponto
sem alterar a forma**; **`Shift` trava valor, `Alt` trava posição, `Shift+Alt` = posição com snap nos
elementos das outras camadas**; **`Ctrl`+clique cicla o easing do segmento**; **`Ctrl+Shift`+arraste
desenha à mão**. Mais: **botão direito na barra de zoom reseta a view, botão direito arrastando dá zoom
no trecho**, **duplo-clique nos números cria cue**, **`Shift`+arraste da agulha dá snap em elementos**.

Conceitos: **a camada de mapping é um mapping** — a curva é entrada de uma cadeia input→filtros→outputs,
não um caminho paralelo até a saída; **Mapping 2D separa o caminho da posição sobre ele** (modelo do
`.ild`); **Sequence Layer aninha sequências**; **cue que pausa a timeline** para show semi-interativo.

### O que entra em `gravar-dmx`

**Arm por camada** (não modo global), **valor em vermelho durante a gravação**, **conversão em curva
editável ao parar**, e **simplificação depois**. E o registro de que **a documentação do Parrot está
vazia** — a única fonte sobre ele é `chataigne.md`.

### O que NÃO copiar

- **Deixar duas páginas da documentação vazias** no software mais parecido com o nosso. Detective e
  Parrot são os dois recursos que o Spellcaster mais precisaria estudar, e não há uma linha.
- **"Parameters — To fill"** publicado na página de Mappings.
- **`Ctrl+;` para Project Settings e `Ctrl+,` para Preferences**: dois atalhos adjacentes no teclado
  para dois diálogos parecidos e de escopo oposto (um salva no arquivo, o outro na máquina). Trocar um
  pelo outro é erro garantido. Se o Spellcaster tiver os dois, que sejam distantes e rotulados.
- **Mudar `module.json` exigir "Reload custom modules" *e* apagar e recriar o módulo.** Recarga que não
  recarrega.
- **A roda do mouse com semântica invertida entre telas** — na State Machine a roda rola e `Shift`+roda
  dá zoom; no Ableton e no Resolve é o contrário. A doc ainda admite "pode mudar no futuro". Escolher
  uma convenção e valer no app inteiro, e escrever em `design/SHORTCUTS.md`.

### Ponto aberto para `design/DECISOES.md`

Roda do mouse: `Shift`+roda = **zoom** (Chataigne, canvas de nós) ou = **scroll horizontal** (Ableton
p.991, Resolve p.58 usa `Shift`+roda para altura de track)? Os três divergem, e o Spellcaster tem
canvas de nós **e** timeline. Não decidido aqui.
