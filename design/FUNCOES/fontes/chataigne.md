# Chataigne — auditoria de interface (Benjamin Kuperberg, JUCE, open source)

Auditoria funcional, não estética. Alimenta a função 3 do Spellcaster ("orquestrador", `design/TEMAS.md:13`) e a regra "o graph é a interface" (`design/PRINCIPIOS.md` §1). Chataigne v1.9.24 (versão instalada nesta máquina, `Chataigne.settings` `lastVersion`).

## Fontes lidas

Locais (lidos integralmente):

- `C:\Users\email\Documents\Chataigne\layouts\default.chalayout` (JSON, 135 linhas)
- `C:\Users\email\Documents\Chataigne\layouts\_lastSession.chalayout` (idêntico ao default exceto `currentContent` do painel central: `Morpher` em vez de `State Machine`)
- `C:\Users\email\AppData\Roaming\Chataigne\Chataigne.settings` (XML JUCE `PROPERTIES`, com `globalSettings` como JSON escapado)
- `C:\Users\email\Documents\Chataigne\dashboard\index.html` + `dashboard\assets\` (20 CSS, 29 JS, app Ember `chataigne-web-dashboard` v1.2.9) + `dashboard\fonts\` (icomoon)
- `C:\Users\email\Documents\Chataigne\modules\` — vazia (nenhum módulo comunitário instalado)
- Busca `es.exe ext:noisette` e `es.exe -path "D:\DRIVE_MNDS" noisette`: **zero resultados**. Não há sessão `.noisette` nesta máquina; o esquema abaixo vem do código, não de arquivo real.

Remotas — código-fonte (via `raw.githubusercontent.com`, branch `master`):

- `benkuper/Chataigne`: `Source/ChataigneEngine.cpp`, `Source/MainComponent.h`, `Source/MainComponent.cpp`, `Source/Module/Module.h`, `Source/Module/Module.cpp`, `Source/Module/ui/ModuleUI.h`, `Source/Module/Routing/ModuleRouter.h`, `Source/StateMachine/State/State.h`, `Source/StateMachine/State/State.cpp`, `Source/StateMachine/StateManager.h`, `Source/StateMachine/Transition/StateTransition.h`, `Source/StateMachine/Transition/StateTransition.cpp`, `Source/Common/Processor/Processor.h`, `Source/Common/Processor/Action/Action.h`, `Source/Common/Processor/Action/Condition/Condition.h`, `.../ActivationCondition/ActivationCondition.h`, `.../Consequence/Consequence.h`, `Source/Common/Processor/Mapping/Mapping.h`, `.../Input/MappingInput.h`, `.../Filter/MappingFilter.h`, `.../Output/MappingOutput.h`, `Source/Common/Processor/Multiplex/Multiplex.h`, `Source/Common/Processor/Conductor/Conductor.h`, `.../ConductorCue.h`, `Source/Common/Command/BaseCommand.h`, `CommandDefinition.h`, `CommandContext.h`, `Source/TimeMachine/Sequence/ChataigneSequence.h`, `.../Cue/ChataigneCue.h`, `.../layers/trigger/ChataigneTimeTrigger.h`, `.../layers/mapping/MappingLayer.h`, `Source/CustomVariables/CVGroup.h`, `.../Preset/CVPreset.h`, `.../Preset/Morpher/Morpher.h`, `Source/Module/modules/dmx/DMXModule.h`, e a árvore completa de `Source/` (342 headers) via API do GitHub.
- `benkuper/juce_organicui` (submódulo, `.gitmodules`): `controllable/Controllable.h/.cpp`, `controllable/parameter/Parameter.h/.cpp`, `Trigger.h`, `TargetParameter.h`, `EnumParameter.h`, `ControllableFactory.h/.cpp`, `FloatParameter.h/.cpp`, `IntParameter.cpp`, `BoolParameter.cpp`, `StringParameter.h/.cpp`, `ColorParameter.cpp`, `Point2DParameter.cpp`, `Point3DParameter.cpp`, `ui/ParameterUI.h`, `controllable/ControllableContainer.cpp`, `manager/BaseItem.h`, `manager/BaseManager.h`, `automation/Automation.h`, `AutomationKey.h`, `easing/Easing.h`, `dashboard/Dashboard.h`, `DashboardItem.h`, `DashboardManager.h`, `DashboardIFrameItem.h`, `controllable/dashboard/DashboardControllableItem.h`, `controllable/detective/Detective.h`, `warning/WarningReporter.h`, `WarningTarget.h`, `parrot/Parrot.h`, `logger/CustomLogger.h`, `outliner/Outliner.h`, `ui/shapeshifter/*.h` + `ShapeShifter.cpp`, `ShapeShifterContainer.cpp`, `ShapeShifterPanel.cpp`, `app/OrganicMainComponent.cpp`, `app/OrganicMainComponentCommands.cpp`, `engine/Engine.cpp`, `engine/EngineFileDocument.cpp`, `remotecontrol/OSCRemoteControl.h`.
- `benkuper/juce_timeline` (submódulo): `timeline/Sequence/Sequence.h`, `Layer/SequenceLayer.h`, `Layer/layers/Trigger/TimeTrigger.h`, `Cue/TimeCue.h`, `TimelineAppCommands.h/.cpp`.
- `tommag/Sample-Chataigne-module` → `module.json` (994 bytes, lido inteiro) e `benkuper/LaunchpadX-Chataigne-Module` → `module.json` (11,7 KB).

Falhas registradas (nada foi afirmado a partir delas):

- `https://benjamin.kuperberg.fr/chataigne/docs/` responde **301** para `https://benkuper.notion.site/The-Amazing-Chataigne-Documentation-079bd5a0b7e648bbbfe34c3c869a3985`. Essa página é SPA Notion: o `WebFetch` retorna só a string "Notion", sem conteúdo. Mesma falha em `benkuper.notion.site/Making-your-own-Module-f892f4150bc74dc1b851ba1a3c99510d`. **Nenhuma afirmação deste documento vem da documentação oficial.**
- `github.com/benkuper/Chataigne-ModuleLibrary` retorna **404** na API (repositório não existe com esse nome). O repositório vivo é `benkuper/Chataigne-community-modules`, que hospeda apenas `modules.json` (índice de URLs). Os `module.json` de exemplo vieram dos dois repositórios de módulo citados acima.
- `github.com/benkuper/OrganicUI` retorna **404**; o nome real é `juce_organicui`.

---

## Anatomia da tela

Não existe layout fixo: a tela é uma árvore de docks serializada em JSON (`.chalayout`), carregada de `Documents/Chataigne/layouts/`. `ShapeShifterManager` guarda `appLayoutExtension` e `appSubFolder = "layouts"` (`ui/shapeshifter/ShapeShifterManager.h:29-31`) e grava `_lastSession` a cada saída (`saveLastLayout`, mesma classe linha 42).

Esquema do `.chalayout` (`ShapeShifter.cpp:59-71`, `ShapeShifterContainer.cpp:250-262`, `ShapeShifterPanel.cpp:329-345`):

```
{ "mainLayout": <shifter>, "windows": null }
<shifter> = { "type": 0|1, "width": int, "height": int,
              // type 1 (CONTAINER):
              "direction": 1|2, "shifters": [<shifter>...]
              // type 0 (PANEL):
              "currentContent": "<nome>", "tabs": [{"name":"<nome>"}...] }
```

`type` é `ShapeShifter::Type { PANEL=0, CONTAINER=1 }` (`ShapeShifter.h:19`); `direction` é `ShapeShifterContainer::Direction { NONE=0, HORIZONTAL=1, VERTICAL=2 }` (`ShapeShifterContainer.h:40`). Não há coordenadas absolutas: só aninhamento, direção e tamanho preferido; o gap entre painéis é fixo em 6 px (`ShapeShifterContainer.h:51`) e o redimensionamento é feito por `GapGrabber`.

Layout padrão desta instalação (`default.chalayout`, tela 2561×1350), duas faixas empilhadas (VERTICAL):

- Faixa superior (751 px de altura, HORIZONTAL):
  - coluna esquerda 307 px (VERTICAL): **Modules** (382 px) sobre **Custom Variables** (362 px) — linhas 26-47.
  - centro 1814 px: um único painel com quatro abas — **State Machine**, **Dashboard**, **Module Router**, **Morpher** (linhas 50-69). Em `_lastSession` a aba ativa é `Morpher`.
  - direita 428 px: **Inspector**, sozinho (linhas 70-80).
- Faixa inferior (593 px, HORIZONTAL): **Sequences** (lista, 178 px) | **Sequence Editor** (timeline, 1897 px) | painel com abas **Help / Logger / Warnings** (474 px, ativo em Logger) — linhas 83-128.

Padrão de leitura: lista de itens estreita à esquerda, editor grande no meio, Inspector fixo à direita, timeline embaixo, log/ajuda no canto. Igual ao par Premiere/Resolve descrito em `design/SHORTCUTS.md:57-58`.

Painéis disponíveis (registro no `ShapeShifterFactory`), oito genéricos do toolkit — **Inspector, Outliner, Dashboard, Parrots, The Detective, Help, Warnings, Logger** (`app/OrganicMainComponent.cpp:63-92`) — e nove específicos do Chataigne — **Module Router, Modules, Custom Variables, Morpher, Sequences, Sequence Editor, States, State Machine, Command Templates** (`Source/MainComponent.cpp:28-40`). Repare que `States` (a lista) e `State Machine` (a vista de nós) são painéis distintos, e que o layout padrão não abre `States`, `Outliner`, `Parrots`, `The Detective` nem `Command Templates`.

O Inspector é genérico: cada objeto responde `getEditorInternal(bool isRoot, Array<Inspectable*>)` (`Controllable.h:141`, `manager/BaseItem.h:105`) e o painel só mostra o editor da seleção corrente. Editar N itens ao mesmo tempo é nativo — o array de `Inspectable*` na assinatura existe para isso.

Existe "maximizar painel": `toggleTemporaryFullContent`, que guarda o layout antigo em `ghostLayout` e restaura depois (`ShapeShifterManager.h:40-41, 85`).

## Objetos e verbos

Toda a hierarquia desce de duas classes: `ControllableContainer` (nó com filhos) e `Controllable` (folha com valor). `BaseItem` = container que também é item de lista, com `miniMode`, `listUISize`, `viewUIPosition`, `viewUISize`, `isUILocked`, `itemColor` (`manager/BaseItem.h:26-31`) — ou seja, **posição e cor no canvas são parâmetros salvos do próprio objeto**, não estado de UI separado. `BaseManager<T>` é a lista genérica: add, remove, reorder, copy/paste por clipboard de texto JSON, undo (`manager/BaseManager.h:437, 585, 919-930`).

**Module** (`Source/Module/Module.h`). Um módulo é: `moduleParams` (container de configuração), `valuesCC` (container de valores recebidos, linha 46), `defManager` (lista de `CommandDefinition`, linha 45), `templateManager` (comandos-modelo do usuário, linha 60), `logIncomingData`/`logOutgoingData` (linhas 36-37), `inActivityTrigger`/`outActivityTrigger` (linhas 40-41, fora da hierarquia para não gerar tempestade de listeners) e `connectionFeedbackRef` (linha 43). `hasInput`/`hasOutput` definem se ele aparece como fonte e/ou destino (`setupIOConfiguration`, linha 64). Verbos: adicionar pelo menu, habilitar/desabilitar, logar entrada/saída, testar comando (`ModuleCommandTester`, linha 52), rotear valores para outro módulo (`canHandleRouteValues`, linha 71). Módulos embutidos, por pasta em `Source/Module/modules/`: audio (com FFT e detecção de pitch), ble, dmx, gpio, http, midi, mqtt, osc (+ perfis dlight, HeavyM, Live, Millumin, Powerpoint, QLab, Reaper, Resolume), oscquery (+ MadMapper), posistagenet, serial, tcp (client, server, PJLink, Watchout), udp, websocket (client e server), abletonlink, gamepad/joycon/wiimote/kinect/keyboard/mouse/streamdeck/loupedeck/ajazz, metronome, signal generator, sequence, state, customvariables, multiplex, os, time, generic, empty. O módulo DMX tem `dmxType`, `sendRate`, `sendOnChangeOnly`, `useMulticast`, `inputUniverseManager`/`outputUniverseManager` e um `thruManager` (`Source/Module/modules/dmx/DMXModule.h:23-42`) — a saída DMX é multi-universo com "thru", exatamente o que o Spellcaster precisa para sACN.

**Custom Variables** (`Source/CustomVariables/CVGroup.h`). Um grupo tem `values` (parâmetros criados pelo usuário), `pm` (lista de presets) e `morpher`. `controlMode` ∈ {FREE, WEIGHTS, VORONOI, GRADIENT_BAND} (linha 28). Verbos: `addItemFromParameter` (linha 59 — cria a variável a partir de qualquer parâmetro do app), `setValuesToPreset`, `lerpPresets`, `goToPreset(preset, time, curve)` (linha 69: ir para um preset em N segundos seguindo uma curva de easing), `randomizeValues` (linha 72). O `CVPreset` tem `defaultLoadTime`, `loadTrigger`, `updateTrigger` (`Preset/CVPreset.h:94-96`) e cada parâmetro dentro do preset tem `interpolationMode` ∈ {INTERPOLATE, CHANGE_AT_END, CHANGE_AT_START, NONE} (`Preset/CVPreset.h:24-25`). O **Morpher** é um plano 2D: cada preset é um ponto, o cursor `targetPosition` gera pesos por diagrama de Voronoi ou por banda de gradiente, com imagem de fundo opcional e "atração" física do cursor (`Preset/Morpher/Morpher.h:28-51`). Isto é literalmente um "cena morfável" — um X/Y pad de cenas.

**State** (`Source/StateMachine/State/State.h`) — ver seção própria abaixo.

**Sequence** (`juce_timeline` + `Source/TimeMachine/`) — ver seção própria.

**Processor** é a superclasse comum de tudo que "faz": `enum ProcessorType { ACTION, MAPPING, MULTIPLEX, CONDUCTOR }` (`Common/Processor/Processor.h:19`). Um `ProcessorManager` vive dentro de cada State, de cada Multiplex e de cada Conductor. Isto é a chave da arquitetura: **estado, mapeamento, ação e cue são a mesma coisa em quatro sabores**, e todos podem ser desabilitados em bloco por `setForceDisabled(value, force, fromActivation)` (linha 29).

**Multiplex** (`Common/Processor/Multiplex/Multiplex.h`): `count` + `previewIndex` + listas; qualquer Action/Mapping dentro dele é instanciado N vezes com índice. É o "faça isto para os 24 fixtures" sem duplicar 24 mapeamentos.

**Conductor / ConductorCue** (`Common/Processor/Conductor/`): lista de cues com `currentCueIndex`, `nextCueIndex`, `loop`, `currentCueName`/`nextCueName`, gatilhos `triggerPrevious`/`triggerCurrent`, e cores de UI para o cue atual e o próximo (`Conductor.h:22-42`). Cada `ConductorCue` pode amarrar uma sequência (`linkedSequenceParam`) com `autoStart`, `forceStartFrom0`, `autoStop`, `autoNext` (`ConductorCue.h:29-34`). É a lista de GO de mesa de luz, escrita como Action.

**Command / CommandDefinition** (`Common/Command/`). Uma `CommandDefinition` é `{container, menuPath, commandType, params, createFunc, context}` (`CommandDefinition.h:24-36`); `CommandContext { ACTION, MAPPING, BOTH }` (`CommandContext.h:13`) declara se o comando pode ser disparado (Action) ou receber um valor contínuo (Mapping) ou ambos. Uma `BaseCommand` instanciada expõe `trigger(multiplexIndex)` e `setValue(var, multiplexIndex)` (`BaseCommand.h:56-59`) — o mesmo objeto serve para "dispare isto" e "mande este valor para isto". Comandos podem ser abstraídos em `CommandTemplate` reutilizável.

**Dashboard item** (`dashboard/`): `Dashboard` é um `BaseItem` com `itemManager`, `isBeingEdited` e senha opcional (`Dashboard.h:22-25`). Tipos de item no repositório: `DashboardControllableItem` (parâmetro/trigger), `DashboardCCItem` (container inteiro), `DashboardGroupItem`, `DashboardCommentItem`, `DashboardLinkItem`, `DashboardIFrameItem`, `SharedTextureDashboardItem`, `DashboardInspectableItem`. Verbo de criação: qualquer `Controllable` sabe se transformar em item (`Controllable::createDashboardItem()`, `Controllable.h:112`) — arrastar um parâmetro para o dashboard é uma chamada de método, não um caso especial.

## Tipos de parâmetro (OrganicUI)

Enum canônico: `Controllable::Type { CUSTOM, TRIGGER, FLOAT, INT, BOOL, STRING, ENUM, POINT2D, POINT3D, TARGET, COLOR, TYPE_MAX }` (`controllable/Controllable.h:24`). A fábrica registra exatamente onze tipos criáveis, nesta ordem: Trigger, Boolean, Float, Integer, Enum, String, File, Point2D, Point3D, Target, Color (`controllable/ControllableFactory.cpp:17-27`).

Widget padrão de cada tipo (`createDefaultUI`):

| Tipo | Widget | Fonte |
|---|---|---|
| Trigger | botão; variantes imagem e "blink" | `Trigger.h:24-27` |
| Boolean | toggle | `BoolParameter.cpp:38-40` |
| Float | slider se tem range, senão label; `UIType { NONE, SLIDER, STEPPER, LABEL, TIME }` | `FloatParameter.cpp:50-53`, `FloatParameter.h:29` |
| Integer | stepper | `IntParameter.cpp:120-127` |
| String | campo de texto, multilinha opcional; `UIType { TEXT, FILE }` | `StringParameter.cpp:46-56`, `StringParameter.h:22` |
| Enum | dropdown (ou barra de botões, `EnumParameterButtonBarUI`) | `EnumParameter.cpp:473-475`, `EnumParameter.h:14` |
| Color | seletor de cor | `ColorParameter.cpp:196-199` |
| Point2D | dois sliders acoplados (`DoubleSliderUI`) | `Point2DParameter.cpp:222-227` |
| Point3D | três sliders (`TripleSliderUI`) | `Point3DParameter.cpp:246-251` |
| Target | seletor de endereço de controle, com filtro de tipos | `TargetParameter.h:106-107, 43-44` |
| File | campo com browse | `FileParameter.h` (registrado em `ControllableFactory.cpp:23`) |

Atributos declaráveis (nome exato aceito em JSON e por script). `Controllable::getValidAttributes()` devolve `{ "enabled", "canBeDisabled", "targetType", "searchLevel", "allowedTypes", "excludedTypes", "root", "labelLevel", "saveValueOnly" }` (`Controllable.cpp:402-405`); `Parameter` acrescenta `"alwaysNotify"` (`Parameter.cpp:411-416`). O setter aceita ainda `"description"` e `"readonly"`/`"readOnly"`, este último mapeado para `setControllableFeedbackOnly(value)` (`Controllable.cpp:359-382`). Ou seja:

- `enabled` — bool, campo `Controllable::enabled` (`Controllable.h:37`).
- `readOnly` → `isControllableFeedbackOnly` (`Controllable.h:41`): o widget mostra o valor mas não deixa editar. É assim que um "valor recebido" difere de um "parâmetro de configuração", e não por widget diferente.
- `alwaysNotify` — notifica mesmo quando o valor novo é igual ao antigo (`Parameter.h:63`). Necessário para OSC/DMX onde repetir importa.
- `hideInEditor` não é atributo de `Controllable`: é campo de `ControllableContainer`, salvo no JSON só quando verdadeiro (`ControllableContainer.cpp:983`); em `module.json` o próprio Chataigne o liga sozinho quando o container de parâmetros fica vazio (`Module.cpp:214`).
- Range: `setupFromJSONData` lê `min`, `max` e `default` (`Parameter.cpp:646-657`).

Além do valor, todo `Parameter` tem um **modo de controle**: `ControlMode { MANUAL, EXPRESSION, REFERENCE, AUTOMATION }` (`Parameter.h:36-41`). Qualquer parâmetro pode ser trocado, pelo menu de contexto, para "expressão JS", "referência a outro parâmetro" ou "automação com curva própria" — sem que exista um nó de patch para isso. Também há `colorStatusMap` (`Parameter.h:79`): mapa valor→cor, isto é, o widget muda de cor conforme o valor. E `ValueInterpolator` (`Parameter.h:235-279`), que faz fade de qualquer parâmetro para um valor-alvo em N segundos numa thread própria — fade é primitiva do parâmetro, não da cena.

## State Machine

Um `State` é um `BaseItem` com um `ProcessorManager` dentro (`State.h:23-35`). Ele contém, portanto, Actions, Mappings, Multiplexes e Conductors próprios.

Parâmetros do estado:

- `active` (bool) — "se ativo, as ações e mapeamentos deste estado têm efeito; senão este estado não faz nada" (`State.cpp:23`).
- `loadActivationBehavior` ∈ {Restore last state, Activate, Deactivate} (`State.h:25-26`, `State.cpp:24-25`). O "activate on start" é este enum, não um booleano.
- `checkTransitionsOnActivate` (bool) — ao ativar, avalia transições já verdadeiras; sem ele, uma condição precisa ficar falsa e voltar a ser verdadeira para disparar (`State.cpp:26-27`).
- `focusOnLastActionTriggered` (bool) — rola a vista até a última ação disparada (`State.cpp:30`).

Diagrama do que acontece:

```
ativar estado S
  └─ S.pm.setForceDisabled(false)            → Actions/Mappings de S voltam a valer
  └─ notifica listeners (UI acende)
  └─ S.pm.checkAllActivateActions()          → Actions com papel ACTIVATE rodam agora
  └─ para cada transição T que sai de S:
        T.forceCheck(false)                  → estado de validade sem disparar
        condições ActivationCondition de T recebem valid = (tipo == ON_ACTIVATE)
        se checkTransitionsOnActivate e T.cdm válido:
              T.triggerConsequences(true); break

desativar estado S
  └─ ActivationConditions das transições de saída viram inválidas
  └─ S.pm.checkAllDeactivateActions()        → Actions com papel DEACTIVATE rodam
  └─ notifica listeners
  └─ S.pm.setForceDisabled(true)             → Actions/Mappings de S param de valer

transição T (source → dest), quando suas condições ficam verdadeiras
  └─ se source.active:
        T.triggerConsequences(true)          → consequências da transição
        source.active = false                (primeiro desliga a origem)
        dest.active   = true                 (depois liga o destino)
```

Fonte: `State.cpp:62-124` e `StateTransition.cpp:63-74`. O comentário no código explica a ordem: desliga a origem primeiro "caso o destino reative instantaneamente esta" (`StateTransition.cpp:70`).

Uma **transição é uma Action** (`StateTransition.h:15-17`) com `sourceState`/`destState` e com definições de ativação habilitadas no seu gerenciador de condições (`StateTransition.cpp:23`). Consequência prática: transição tem condições e consequências como qualquer ação — pode-se enviar OSC "no caminho" entre dois estados. No arquivo, ela é salva por nome curto: `"sourceState"` e `"destState"` guardam `shortName` (`StateTransition.cpp:41-45`) — referência frágil por nome, não por UID.

Multi-ativação é permitida: `StateManager::checkStartActivationOverlap` e `getLinkedStates` existem justamente porque vários estados podem estar ativos ao mesmo tempo (`StateManager.h:52, 68`). Não é uma máquina de um estado só.

## Sequências

`Sequence` é um `BaseItem` + `Thread` + `AudioIODeviceCallback` (`juce_timeline/timeline/Sequence/Sequence.h:17-22`) — a mesma sequência pode ser cravada no relógio de áudio.

Transporte e parâmetros (`Sequence.h:30-55`): `startAtLoad`, `totalTime`, `currentTime`, `playSpeed`, `loopParam`, `fps`, `autoSnap`, `bpmPreview`, `beatsPerBar`, `evaluateOnSeek` ∈ {NEVER, ONLY_PLAYING, ONLY_NOT_PLAYING, ALWAYS}, e os triggers `playTrigger`, `pauseTrigger`, `stopTrigger`, `finishTrigger`, `togglePlayTrigger`, `prevCue`, `nextCue`, mais `isPlaying`. `viewStartTime`/`viewEndTime`/`viewFollowTime` são parâmetros salvos: o zoom da timeline faz parte do documento.

**Cues**: `TimeCue` tem `time` e `cueAction` ∈ {NOTHING, PAUSE, LOOP_JUMP}, mais `loopCue` (alvo do salto) e o trigger `playFromHere` (`Cue/TimeCue.h:21-26`). Isto é o "pause on trigger" e o "loop" pedidos. No Chataigne, `ChataigneCue` acrescenta um `ConditionManager` (`ChataigneCue.h:22`): o cue só está ativo se as condições baterem — um cue condicional.

**Camadas** (`SequenceLayerManager`), tipos existentes: Trigger, Audio, Mapping (1D, 2D, Color) e SequenceBlock (blocos que tocam outra sequência). Cada camada é um `BaseItem` com `uiHeight` salvo (`Layer/SequenceLayer.h:27`) e sabe responder `selectAllItemsBetween`, `getRemoveTimespan`, `getInsertTimespan`, `getSnapTimes` (linhas 31-40) — inserir/remover tempo é operação da camada, com undo.

- **Trigger layer**: `TimeTrigger` = `time` + `isTriggered` + `flagY` (posição vertical do bandeirola na UI) (`Trigger/TimeTrigger.h:20-24`). No Chataigne, `ChataigneTimeTrigger` carrega um `ConsequenceManager` (`ChataigneTimeTrigger.h:22`): o marcador na timeline dispara a mesma lista de consequências de uma Action.
- **Mapping layer**: contém um `Mapping` inteiro e uma automação; parâmetros `alwaysUpdate`, `sendOnPlay`, `sendOnStop`, `sendOnSeek` (`MappingLayer.h:24-27`), método `getValueAtPosition(float)` e `exportBakedValues` (linhas 38-39).
- **Automação**: `Automation` é um `BaseManager<AutomationKey>` com `position`, `length`, `value`, `valueRange`, `viewValueRange`, `rangeRemapMode` ∈ {ABSOLUTE, PROPORTIONAL} (`automation/Automation.h:23-35`). Cada `AutomationKey` tem `position`, `value` e `easingType` (`AutomationKey.h:21-25`). Tipos de easing: `LINEAR, BEZIER, HOLD, SINE, ELASTIC, BOUNCE, STEPS, NOISE, PERLIN` (`easing/Easing.h:20`) — note NOISE e PERLIN, que transformam um segmento de curva em gerador. A automação também tem simplificação interativa de traço à mão (`addFromPointsAndSimplifyBezier`, `launchInteractiveSimplification`, `Automation.h:46-52`) e um `AutomationRecorder`.

Sincronismo (só no Chataigne, `ChataigneSequence.h:29-51`): módulo de áudio mestre, MTC enviado e recebido com `mtcFPS` e `resetTimeOnMTCStopped`, LTC com `ltcSyncTolerance`, `ltcOutOfRangeMode` ∈ {DO_NOTHING, JUMP_TO_CLOSEST, JUMP_TO_START, JUMP_TO_END}, `ltcMode` ∈ {RECEIVE, SEND, BOTH}, `syncOffset` e `reverseOffset`.

Comparação com `design/SHORTCUTS.md`: o transporte do Chataigne é mais pobre que o mapa do Spellcaster. Não há J/K/L, não há In/Out, não há marcadores separados de cues, e `Space` só toca se `useSpaceBarAsPlayPause` estiver ligado (`TimelineAppCommands.h:19`). O que o Chataigne tem e o `SHORTCUTS.md` ainda não define: `Ctrl+B` cria cue na posição, `Shift+PageUp/PageDown` navega cue a cue, `PageUp/PageDown` anda em passo de tempo, `Ctrl+←/→` move o item selecionado um quadro (`TimelineAppCommands.cpp:11-57`).

## Mappings e Actions

**Mapping** é uma cadeia com quatro estágios, cada um um `BaseManager` próprio: `im` (inputs), `mappingParams`, `fm` (filtros), `om` (outputs), mais `outValuesCC` (`Mapping.h:25-29`).

- **Input** (`MappingInput.h`): `StandardMappingInput` aponta um `TargetParameter` para qualquer parâmetro do app (linha 76); `ManualMappingInput` cria um parâmetro solto para operar à mão (linha 107). `triggersProcess` (linha 21) decide se aquele input dispara o recálculo — vários inputs, só um deles como gatilho.
- **Filtros** (`Filter/filters/`, lista completa): Delay, Script, Time; cor: ColorRemap, ColorShift; condição: Condition; conversão: Conversion, Merge, SimpleConversion; número: Crop, CurveMap, Damping, Freeze, Inverse, Lag, Math, OneEuro, SimpleRemap, SimpleSmooth, Speed; string: String. Cada filtro devolve `ProcessResult { CHANGED, UNCHANGED, STOP_HERE }` (`MappingFilter.h:28`) — um filtro pode interromper a cadeia, e é assim que "Condition" vira um gate. Filtros também podem excluir canais (`excludedChannels`, linha 26) e ter seus próprios parâmetros linkáveis a outros parâmetros (`ParamLinkContainer`, linha 29).
- **Output** (`MappingOutput.h:15-17`): é um `BaseCommandHandler`, ou seja, um comando de módulo em contexto MAPPING recebendo `setValue`.
- Modo de processamento: `ProcessMode { VALUE_CHANGE, MANUAL, TIMER }` com `updateRate`, e as chaves `sendOnInputChangeOnly`, `sendOnOutputChangeOnly`, `sendAfterLoad`, `sendOnActivate` (`Mapping.h:31-39`). Vale a pena copiar `sendOnActivate`: ao entrar num estado, o mapeamento reemite o valor atual, então a luz não fica "presa" no valor do estado anterior.

**Action** (`Action.h`): `cdm` (condições), `csmOn` e `csmOff` (consequências para verdadeiro e para falso), `triggerOn`, `triggerOff`, `triggerPreview` (linhas 34-40). `Role { ACTIVATE, DEACTIVATE }` (linha 26) define se a ação roda ao ativar ou ao desativar o estado que a contém. `autoTriggerWhenAllConditionAreActives` (linha 29) permite usar a Action como mero indicador, sem disparo automático.

Condições disponíveis (`Action/Condition/conditions/`): StandardCondition (com comparadores por tipo — Bool, Enum, Number, Point2D, Point3D, String), ConditionGroup (E/OU aninhado), ManualCondition, ScriptCondition, MultiplexIndexCondition e ActivationCondition, cujo `Type { ON_ACTIVATE, ON_DEACTIVATE }` (`ActivationCondition.h:20`) é o que permite "ao entrar neste estado, faça".

Consequência (`Consequence.h:15-17`) também é um `BaseCommandHandler`. Portanto **saída de mapping e consequência de ação são o mesmo objeto**, só muda o `CommandContext`. O Spellcaster ganha o mesmo se `send(universe, data)` e "cue" forem clientes do mesmo registry.

Atalho de autoria que vale copiar: clicar com o botão direito em qualquer parâmetro da tela oferece "Add & Link to Custom Variable..." e "Add & Link to Sequence..." — o segundo cria a camada de mapping na sequência escolhida, já com o output apontado para aquele parâmetro e com o range copiado (`Source/MainComponent.cpp:64-141`). Mapear não começa no painel de mapping; começa no widget.

**Module Router** (`Module/Routing/ModuleRouter.h`): fonte, destino, lista de valores da fonte e os gatilhos `selectAllValues`, `deselectAllValues`, `routeAllValues` (linhas 24-35). É o atalho para "jogue tudo deste módulo naquele" sem escrever N mapeamentos.

## Estados visuais

- **Módulo ao vivo**: dois `TriggerImageUI` no cabeçalho, um para entrada e outro para saída, alimentados por `inActivityTrigger`/`outActivityTrigger`, e um `BoolToggleUI` de conexão (`ModuleUI.h:23-27`; triggers em `Module.h:40-43`). Pisca por evento, não por polling.
- **Feedback de valor**: `Controllable::isControllableFeedbackOnly` (`Controllable.h:41`) desenha o widget como somente-leitura; `ParameterUI` repinta por `UITimerTarget`/`handlePaintTimer` (`ui/ParameterUI.h:16, 49-50`), com timers agrupados em vez de repaint por evento.
- **Cor por valor**: `Parameter::colorStatusMap` (`Parameter.h:79`) e `ColorStatusUI` — um valor pode acender o widget numa cor declarada.
- **Warnings**: qualquer objeto pode herdar `WarningTarget` e chamar `setWarningMessage(msg, id)` / `clearWarning(id)` (`warning/WarningTarget.h:29-30`); o `WarningReporter` é um singleton que indexa alvo→endereço→mensagem e emite eventos WARNING_REGISTERED/UNREGISTERED (`warning/WarningReporter.h:20-21, 40`). O painel Warnings lista tudo e cada linha resolve para o objeto culpado (`warningResolveInspectable`, `WarningTarget.h:22`). O estado tem `showWarningInUI = true` explicitamente (`State.cpp:44`).
- **Log**: `CustomLogger` guarda no máximo 2000 entradas (`logger/CustomLogger.h:4`); cada entrada tem hora, conteúdo, fonte e severidade, e há gravação opcional em arquivo (`FileWriter`, linhas 38-46).
- **Detective**: singleton `BaseManager<ControllableDetectiveWatcher>` com `watchControllable(c)` (`controllable/detective/Detective.h:12`). É um "vigia este parâmetro" que plota o histórico do valor num painel — depurador de sinal, não de código.
- **Parrot**: gravador/reprodutor genérico. Tem `targetsCC` (lista de parâmetros observados), `recordManager`, `status` ∈ {IDLE, RECORDING, PLAYING}, `playProgression`, `loop`, `forceValueAtStartRecord`, `trimToFirstData`, `trimToLastData` e os triggers start/stop record, play/pause/stop (`parrot/Parrot.h:22-44`). Grava qualquer conjunto de parâmetros e toca de volta, sem timeline. `Controllable::isControlledByParrot` (`Controllable.h:50`) marca o parâmetro que está sendo dirigido pela gravação.
- **Dashboard como estado**: `DashboardControllableItem` tem `showLabel`, `textColor`, `textSize`, `opaqueBackground`, `contourColor`, `contourThickness`, `customLabel`, `forceReadOnly` (`DashboardControllableItem.h:13-21`). Aparência é dado do item, não do tema.

## Atalhos

Do código; `commandModifier` é Ctrl no Windows. Fonte: `juce_organicui/app/OrganicMainComponentCommands.cpp:55-250` e `juce_timeline/TimelineAppCommands.cpp:11-57`.

| Ação | Tecla | Fonte |
|---|---|---|
| Novo | `Ctrl+N` | `OrganicMainComponentCommands.cpp:62` |
| Abrir | `Ctrl+O` | `:67` |
| Abrir último documento | `Ctrl+Shift+O` | `:72` |
| Salvar | `Ctrl+S` | `:77` |
| Salvar como | `Ctrl+Shift+S` | `:82` |
| Salvar cópia | `Ctrl+Alt+S` | `:87` |
| Configurações do projeto | `Ctrl+;` | `:97` |
| Preferências | `Ctrl+,` | `:102` |
| Tela cheia (kiosk) | `F11` | `:111` |
| Desfazer | `Ctrl+Z` | `:120` |
| Refazer | `Ctrl+Shift+Z` ou `Ctrl+Y` | `:126-127` |
| Copiar / recortar / colar | `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | `:137, :148, :157` |
| Duplicar | `Ctrl+D` | `:168` |
| Apagar | `Delete` ou `Backspace` | `:179` |
| Selecionar tudo | `Ctrl+A` | `:187` |
| Item anterior / próximo | `↑` / `↓` (com `Shift` estende) | `:195, :207` |
| Mover item para cima / para baixo | `Alt+↑` / `Alt+↓` | `:219, :231` |
| Alternar modo de edição do Dashboard | `Ctrl+E` | `:243` |
| Play/pause da sequência | `Space` (só se `useSpaceBarAsPlayPause`) | `TimelineAppCommands.cpp:12` |
| Cue anterior / próximo | `Shift+PageUp` / `Shift+PageDown` | `:17, :22` |
| Passo de tempo anterior / próximo | `PageUp` / `PageDown` | `:27, :32` |
| Início / fim | `Home` / `End` | `:37, :42` |
| Adicionar cue na posição | `Ctrl+B` | `:47` |
| Mover um quadro à frente / atrás | `Ctrl+→` / `Ctrl+←` | `:52, :57` |

Painéis têm IDs de comando reservados a partir de `0x31000` (`ShapeShifterManager.h:89-90`), então cada painel pode receber atalho pelo menu, mas nenhum vem mapeado por padrão. O `Chataigne.settings` desta máquina confirma: `keyMappings` está vazio (`<KEYMAPPINGS basedOnDefaults="1"/>`), isto é, o usuário pode remapear tudo pelo mecanismo do JUCE e ninguém remapeou.

## Arquivo (.noisette, module.json)

**`.noisette`** é JSON puro. A extensão é declarada no construtor: `Engine("Chataigne", ".noisette")` (`ChataigneEngine.cpp:17`).

Raiz (`juce_organicui/engine/EngineFileDocument.cpp:471-488`):

```
{ "metaData": { "version": "...", "versionNumber": n },
  "projectSettings": {...},        // só se não-vazio
  "dashboardManager": {...},       // só se não-vazio
  "parrots": {...},                // shortName do ParrotManager
  "layout": {...},                 // o mesmo formato do .chalayout, embutido no show
  ...containers do engine... }
```

Mais as cinco chaves do Chataigne, cada uma sendo o `shortName` do gerenciador correspondente (`ChataigneEngine.cpp:133-146`): **modules** (`BaseManager<Module>("Modules")`), **states** (`"States"`), **sequences**, **routers** (`"Routers"`) e o grupo de variáveis (`CVGroupManager`). Todos são carregados nessa mesma ordem em `loadJSONDataInternalEngine` (linhas 162-182).

Cada gerenciador serializa `{"items": [...]}` (`manager/BaseManager.h:919-930`); cada item serializa `{"type": ..., "niceName": ..., "parameters": [...], "containers": {...}}` com `hideInEditor`, `editorIsCollapsed`, `removable` e `customData` gravados só quando diferentes do padrão (`ControllableContainer.cpp:972-1016`). Um parâmetro só é gravado se foi sobrescrito (`Parameter::shouldBeSaved`, `isOverriden`, `forceSaveValue`, `Parameter.h:89-92`) — o arquivo guarda o delta em relação ao padrão, não o estado completo.

`Engine` mantém uma lista de `breakingChangesVersions` e um `convertURL` para migrar shows antigos por script no servidor (`ChataigneEngine.cpp:25-38`). Quinze versões quebraram compatibilidade entre 1.6.12 e 1.9.17.

**`module.json`** — formato do módulo customizado, confirmado por dois arquivos reais e pelo parser (`Source/Module/Module.cpp:212-341`):

```json
{
  "name": "My custom module", "type": "OSC", "path": "Custom",
  "version": "1.0.0", "description": "...", "url": "...", "downloadURL": "...",
  "hasInput": true, "hasOutput": true,
  "hideDefaultCommands": false,
  "defaults": { "autoAdd": false, "oscInput": { "localPort": 9001 } },
  "parameters": { "Module param": { "type": "Integer" } },
  "hideDefaultParameters": ["autoAdd", "oscInput/localPort"],
  "scripts": ["moduleScript.js"],
  "values": { "Module value": { "type": "Float" } },
  "commands": { "Custom command": { "menu": "", "callback": "customCmd",
      "parameters": { "Value": { "type": "Integer", "min": 0, "max": 100, "default": 0 } } } }
}
```

(fonte: `tommag/Sample-Chataigne-module/module.json`, lido inteiro).

Regras do parser: `type` de um nó pode ser `"Container"`, e aí ele vira subcontainer recursivo com `index` e `collapsed` opcionais (`Module.cpp:301-315`); qualquer outro `type` cai na `ControllableFactory` pelo nome do tipo (`ControllableFactory.cpp:116-126`), aceitando `min`/`max`/`default` e os atributos listados na seção de parâmetros. Cada comando pode declarar `"context": "action" | "mapping" | "both"` (`Module.cpp:236-241`). Existe `dependency`, com `{source, value, check, action}` onde `check` ∈ {equals, notEquals, lessThan, greaterThan} e `action` ∈ {show, enable} (`Module.h:86-93`, `Module.cpp:328-341`): um parâmetro aparece ou habilita conforme o valor de outro, declarativamente, sem script. O `module.json` do Launchpad X (11,7 KB) usa isso e mostra que a árvore pode ter centenas de parâmetros gerados (81 cores de pad em containers `Row 1..8`).

## O que o Spellcaster deve copiar

- **Parâmetro tipado é a unidade da interface, e o widget é derivado do tipo.** Onze tipos, uma função `createDefaultUI` por tipo, atributos declarativos (`readOnly`, `enabled`, `min/max/default`, `alwaysNotify`). Isso realiza `PRINCIPIOS.md` §1 ("todo widget é um nó visível") sem escrever um widget por comando. Beneficia todas as cinco funções, mas sobretudo o **orquestrador** e o **Aprendiz** (que passa a saber falar sobre qualquer parâmetro pelo endereço). Fonte: `Controllable.h:24`, `ControllableFactory.cpp:17-27`.
- **Um só objeto "comando" servindo disparo e valor contínuo, distinguidos por `CommandContext { ACTION, MAPPING, BOTH }`.** No Spellcaster isso é o registry: cada `@command` declara se aceita GO, se aceita valor, ou os dois — e cue, mapeamento e CLI viram clientes do mesmo objeto. Beneficia **cenas e cues DMX** e o **orquestrador**. Fonte: `CommandContext.h:13`, `BaseCommand.h:56-59`, `MappingOutput.h:15`, `Consequence.h:15`.
- **`sendOnActivate` no mapeamento e `Role {ACTIVATE, DEACTIVATE}` na ação.** Ao entrar num estado, tudo reemite; ao sair, roda a lista de saída. É o que impede a luz de ficar presa no valor do estado anterior. Beneficia **cenas e cues DMX** e **cenário interativo**. Fonte: `Mapping.h:36`, `Action.h:26`, `State.cpp:71, 108`.
- **Cadeia de filtros com `ProcessResult { CHANGED, UNCHANGED, STOP_HERE }`.** Um filtro que interrompe a cadeia é o gate condicional sem inventar tipo de nó novo; `Damping`, `Lag`, `OneEuro`, `Speed` e `CurveMap` são exatamente o que falta entre um sensor e um galvo. Beneficia **NDI → ILDA** (suavizar contorno antes de virar frame) e **cenário interativo**. Fonte: `MappingFilter.h:28`, pasta `Filter/filters/`.
- **Criar o mapeamento a partir do widget, com botão direito.** "Add & Link to Sequence" cria a camada, o output e copia o range num clique. Aplicado ao Spellcaster: clicar num canal no Patch e escolher "criar keyframe nesta timeline" ou "amarrar a esta cena". Beneficia **cenas e cues DMX** e o **orquestrador**. Fonte: `Source/MainComponent.cpp:80-141`.
- **Cue com ação declarada (`NOTHING / PAUSE / LOOP_JUMP` + alvo de loop) e cue condicional.** Cobre "pause on trigger", "loop entre dois pontos" e "só pare aqui se X" sem lógica na GUI. Beneficia o **ILDA player** (loop de frames) e **cenas e cues DMX**. Fonte: `Cue/TimeCue.h:23-26`, `ChataigneCue.h:22`.
- **Multiplex: uma ação instanciada N vezes com índice.** Um mapeamento para 24 fixtures em vez de 24 mapeamentos. Beneficia **cenas e cues DMX** e o **orquestrador**. Fonte: `Multiplex.h:17-27`.
- **WarningReporter como serviço central, com o objeto culpado clicável.** Painel único que lista "universo não responde", "galvo não acompanha", "kpps ÷ pontos abaixo do limite" — exatamente as falas previstas para o Aprendiz em `design/TEMAS.md:15`. Beneficia o **Aprendiz** e o **ILDA player**. Fonte: `warning/WarningReporter.h:28-29`, `WarningTarget.h:22, 29-30`.
- **Detective e Parrot.** "Vigie este parâmetro e me mostre o histórico" e "grave o que eu mexer e toque de volta" são duas ferramentas de ensaio que nenhum concorrente barato tem. O Parrot resolve ensaio de show sem timeline. Beneficia **cenário interativo** e **cenas e cues DMX**. Fonte: `detective/Detective.h:12`, `parrot/Parrot.h:22-44`.
- **Morpher: presets como pontos num plano, peso por Voronoi.** É um X/Y pad de cenas que já existe e funciona; casa com o **TEATRO DE PAPEL** (mover o cursor pela maquete mistura bastidores). Beneficia **cenas e cues DMX**. Fonte: `Morpher.h:46-51`, `CVGroup.h:28`.

## O que NÃO copiar

- **Layout de dock livre (ShapeShifter) como padrão.** A árvore de containers com gaps arrastáveis permite ao operador desmontar a tela e não achar mais o botão — o oposto de `PRINCIPIOS.md` §3 ("nada elástico, nada que muda de lugar sozinho", grade de 8 px, tamanhos discretos). Copiar o *formato serializado* (árvore, direção, tamanho preferido) para as Faces é útil; copiar o *arrastar livre* não é. Fonte: `ShapeShifterContainer.h:40-51`.
- **Referência entre objetos por nome curto.** `StateTransition` grava `sourceState`/`destState` como `shortName` (`StateTransition.cpp:41-45`) e `TargetParameter` guarda um endereço de texto com rotina de conserto (`tryFixBrokenLink`, `useGhosting`, `ghostValue`, `TargetParameter.h:33-34, 86`). Renomear um estado ou um fixture quebra o show, e o remendo é heurístico. No Spellcaster: UID estável no `.spell`, nome só para exibição.
- **Quatro modos de controle por parâmetro (MANUAL / EXPRESSION / REFERENCE / AUTOMATION) escondidos no menu de contexto.** Cada um é lógica invisível no graph: o valor muda e não há nó que explique por quê. Fere `PRINCIPIOS.md` §1 ("proíbe atalho mágico sem nó correspondente"). Se o Spellcaster quiser expressão, ela precisa ser um nó visível. Fonte: `Parameter.h:36-41`.
- **Aparência gravada dentro de cada item.** `itemColor`, `viewUIPosition`, `viewUISize`, `listUISize`, `miniMode` em `BaseItem` (`manager/BaseItem.h:26-31`) e a paleta inteira de cores das abas dentro do `DashboardManager` (`DashboardManager.h:52-59`) misturam dado e tema. `PRINCIPIOS.md` §5 proíbe: "Face que fixa cor". Posição pertence à Face; cor pertence ao Theme; nenhuma das duas pertence ao objeto do show.
- **Dashboard web como aplicação separada.** São 29 bundles JS e 20 CSS de um app Ember versionado à parte (`dashboard/index.html`, `dashboard/assets/`), baixado por HTTP do servidor do autor (`DashboardManager.h:78-86`) — segunda base de código, segunda UI, e o item só reconhece parte dos tipos (o bundle instalado nomeia apenas `DashboardGroupItem`, `DashboardCommentItem` e `DashboardLinkItem`, embora o C++ defina oito). Se o Spellcaster tem GUI web nativa com skins, o "dashboard" deve ser mais uma Face, não um segundo produto.
- **Sequência dependente do relógio de áudio e da thread de áudio.** `Sequence` herda `AudioIODeviceCallback` e `timeIsDrivenByAudio()` (`Sequence.h:20, 120`) — bom para show com trilha, mas amarra o transporte ao dispositivo de áudio, e no Pi sem placa de som isso é problema. O Spellcaster Lite precisa de relógio próprio, com áudio como sincronizador opcional.
- **Quinze versões com quebra de compatibilidade "resolvidas" por conversor remoto.** `breakingChangesVersions` + `convertURL` apontando para um PHP no site do autor (`ChataigneEngine.cpp:25-38`) significa que abrir um show antigo depende de um servidor de terceiro estar no ar. O `.spell` precisa de número de versão no arquivo e migração local, dentro do próprio binário.
- **`Space` como play/pause condicionado a um booleano global** (`useSpaceBarAsPlayPause`, `TimelineAppCommands.h:19`) e nenhum atalho de painel mapeado por padrão. `design/SHORTCUTS.md` já resolve melhor: `Space` sempre toca, painéis em `Shift+1..7`, e o menu mostra o atalho ao lado.
