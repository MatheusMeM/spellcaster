# Chataigne — interface audit (Benjamin Kuperberg, JUCE, open source)

Functional audit, not aesthetic. Feeds Spellcaster's function 3 ("orchestrator", `design/TEMAS.md:13`) and the rule "the graph is the interface" (`design/PRINCIPIOS.md` §1). Chataigne v1.9.24 (version installed on this machine, `Chataigne.settings` `lastVersion`).

## Sources read

Local (read in full):

- `C:\Users\email\Documents\Chataigne\layouts\default.chalayout` (JSON, 135 lines)
- `C:\Users\email\Documents\Chataigne\layouts\_lastSession.chalayout` (identical to default except for the central panel's `currentContent`: `Morpher` instead of `State Machine`)
- `C:\Users\email\AppData\Roaming\Chataigne\Chataigne.settings` (JUCE XML `PROPERTIES`, with `globalSettings` as escaped JSON)
- `C:\Users\email\Documents\Chataigne\dashboard\index.html` + `dashboard\assets\` (20 CSS, 29 JS, Ember app `chataigne-web-dashboard` v1.2.9) + `dashboard\fonts\` (icomoon)
- `C:\Users\email\Documents\Chataigne\modules\` — empty (no community module installed)
- Search `es.exe ext:noisette` and `es.exe -path "D:\DRIVE_MNDS" noisette`: **zero results**. There is no `.noisette` session on this machine; the schema below comes from the code, not from a real file.

Remote — source code (via `raw.githubusercontent.com`, branch `master`):

- `benkuper/Chataigne`: `Source/ChataigneEngine.cpp`, `Source/MainComponent.h`, `Source/MainComponent.cpp`, `Source/Module/Module.h`, `Source/Module/Module.cpp`, `Source/Module/ui/ModuleUI.h`, `Source/Module/Routing/ModuleRouter.h`, `Source/StateMachine/State/State.h`, `Source/StateMachine/State/State.cpp`, `Source/StateMachine/StateManager.h`, `Source/StateMachine/Transition/StateTransition.h`, `Source/StateMachine/Transition/StateTransition.cpp`, `Source/Common/Processor/Processor.h`, `Source/Common/Processor/Action/Action.h`, `Source/Common/Processor/Action/Condition/Condition.h`, `.../ActivationCondition/ActivationCondition.h`, `.../Consequence/Consequence.h`, `Source/Common/Processor/Mapping/Mapping.h`, `.../Input/MappingInput.h`, `.../Filter/MappingFilter.h`, `.../Output/MappingOutput.h`, `Source/Common/Processor/Multiplex/Multiplex.h`, `Source/Common/Processor/Conductor/Conductor.h`, `.../ConductorCue.h`, `Source/Common/Command/BaseCommand.h`, `CommandDefinition.h`, `CommandContext.h`, `Source/TimeMachine/Sequence/ChataigneSequence.h`, `.../Cue/ChataigneCue.h`, `.../layers/trigger/ChataigneTimeTrigger.h`, `.../layers/mapping/MappingLayer.h`, `Source/CustomVariables/CVGroup.h`, `.../Preset/CVPreset.h`, `.../Preset/Morpher/Morpher.h`, `Source/Module/modules/dmx/DMXModule.h`, and the complete `Source/` tree (342 headers) via the GitHub API.
- `benkuper/juce_organicui` (submodule, `.gitmodules`): `controllable/Controllable.h/.cpp`, `controllable/parameter/Parameter.h/.cpp`, `Trigger.h`, `TargetParameter.h`, `EnumParameter.h`, `ControllableFactory.h/.cpp`, `FloatParameter.h/.cpp`, `IntParameter.cpp`, `BoolParameter.cpp`, `StringParameter.h/.cpp`, `ColorParameter.cpp`, `Point2DParameter.cpp`, `Point3DParameter.cpp`, `ui/ParameterUI.h`, `controllable/ControllableContainer.cpp`, `manager/BaseItem.h`, `manager/BaseManager.h`, `automation/Automation.h`, `AutomationKey.h`, `easing/Easing.h`, `dashboard/Dashboard.h`, `DashboardItem.h`, `DashboardManager.h`, `DashboardIFrameItem.h`, `controllable/dashboard/DashboardControllableItem.h`, `controllable/detective/Detective.h`, `warning/WarningReporter.h`, `WarningTarget.h`, `parrot/Parrot.h`, `logger/CustomLogger.h`, `outliner/Outliner.h`, `ui/shapeshifter/*.h` + `ShapeShifter.cpp`, `ShapeShifterContainer.cpp`, `ShapeShifterPanel.cpp`, `app/OrganicMainComponent.cpp`, `app/OrganicMainComponentCommands.cpp`, `engine/Engine.cpp`, `engine/EngineFileDocument.cpp`, `remotecontrol/OSCRemoteControl.h`.
- `benkuper/juce_timeline` (submodule): `timeline/Sequence/Sequence.h`, `Layer/SequenceLayer.h`, `Layer/layers/Trigger/TimeTrigger.h`, `Cue/TimeCue.h`, `TimelineAppCommands.h/.cpp`.
- `tommag/Sample-Chataigne-module` → `module.json` (994 bytes, read in full) and `benkuper/LaunchpadX-Chataigne-Module` → `module.json` (11.7 KB).

Failures recorded (nothing was asserted from them):

- `https://benjamin.kuperberg.fr/chataigne/docs/` responds **301** to `https://benkuper.notion.site/The-Amazing-Chataigne-Documentation-079bd5a0b7e648bbbfe34c3c869a3985`. That page is a Notion SPA: `WebFetch` returns only the string "Notion", with no content. Same failure at `benkuper.notion.site/Making-your-own-Module-f892f4150bc74dc1b851ba1a3c99510d`. **No statement in this document comes from the official documentation.**
- `github.com/benkuper/Chataigne-ModuleLibrary` returns **404** on the API (no repository exists under that name). The live repository is `benkuper/Chataigne-community-modules`, which hosts only `modules.json` (an index of URLs). The example `module.json` files came from the two module repositories cited above.
- `github.com/benkuper/OrganicUI` returns **404**; the real name is `juce_organicui`.

---

## Anatomy of the screen

There is no fixed layout: the screen is a tree of docks serialized in JSON (`.chalayout`), loaded from `Documents/Chataigne/layouts/`. `ShapeShifterManager` holds `appLayoutExtension` and `appSubFolder = "layouts"` (`ui/shapeshifter/ShapeShifterManager.h:29-31`) and writes `_lastSession` on every exit (`saveLastLayout`, same class line 42).

Schema of `.chalayout` (`ShapeShifter.cpp:59-71`, `ShapeShifterContainer.cpp:250-262`, `ShapeShifterPanel.cpp:329-345`):

```
{ "mainLayout": <shifter>, "windows": null }
<shifter> = { "type": 0|1, "width": int, "height": int,
              // type 1 (CONTAINER):
              "direction": 1|2, "shifters": [<shifter>...]
              // type 0 (PANEL):
              "currentContent": "<name>", "tabs": [{"name":"<name>"}...] }
```

`type` is `ShapeShifter::Type { PANEL=0, CONTAINER=1 }` (`ShapeShifter.h:19`); `direction` is `ShapeShifterContainer::Direction { NONE=0, HORIZONTAL=1, VERTICAL=2 }` (`ShapeShifterContainer.h:40`). There are no absolute coordinates: only nesting, direction and preferred size; the gap between panels is fixed at 6 px (`ShapeShifterContainer.h:51`) and resizing is done by `GapGrabber`.

Default layout of this installation (`default.chalayout`, screen 2561×1350), two stacked bands (VERTICAL):

- Top band (751 px tall, HORIZONTAL):
  - left column 307 px (VERTICAL): **Modules** (382 px) over **Custom Variables** (362 px) — lines 26-47.
  - center 1814 px: a single panel with four tabs — **State Machine**, **Dashboard**, **Module Router**, **Morpher** (lines 50-69). In `_lastSession` the active tab is `Morpher`.
  - right 428 px: **Inspector**, alone (lines 70-80).
- Bottom band (593 px, HORIZONTAL): **Sequences** (list, 178 px) | **Sequence Editor** (timeline, 1897 px) | panel with tabs **Help / Logger / Warnings** (474 px, active on Logger) — lines 83-128.

Reading pattern: narrow item list on the left, big editor in the middle, Inspector fixed on the right, timeline at the bottom, log/help in the corner. Same as the Premiere/Resolve pair described in `design/SHORTCUTS.md:57-58`.

Available panels (registered in `ShapeShifterFactory`), eight generic ones from the toolkit — **Inspector, Outliner, Dashboard, Parrots, The Detective, Help, Warnings, Logger** (`app/OrganicMainComponent.cpp:63-92`) — and nine specific to Chataigne — **Module Router, Modules, Custom Variables, Morpher, Sequences, Sequence Editor, States, State Machine, Command Templates** (`Source/MainComponent.cpp:28-40`). Note that `States` (the list) and `State Machine` (the node view) are distinct panels, and that the default layout does not open `States`, `Outliner`, `Parrots`, `The Detective` or `Command Templates`.

The Inspector is generic: each object answers `getEditorInternal(bool isRoot, Array<Inspectable*>)` (`Controllable.h:141`, `manager/BaseItem.h:105`) and the panel only shows the editor for the current selection. Editing N items at once is native — the `Inspectable*` array in the signature exists for that.

There is a "maximize panel": `toggleTemporaryFullContent`, which stores the old layout in `ghostLayout` and restores it afterwards (`ShapeShifterManager.h:40-41, 85`).

## Objects and verbs

The whole hierarchy descends from two classes: `ControllableContainer` (node with children) and `Controllable` (leaf with a value). `BaseItem` = a container that is also a list item, with `miniMode`, `listUISize`, `viewUIPosition`, `viewUISize`, `isUILocked`, `itemColor` (`manager/BaseItem.h:26-31`) — that is, **position and color on the canvas are saved parameters of the object itself**, not separate UI state. `BaseManager<T>` is the generic list: add, remove, reorder, copy/paste through a JSON text clipboard, undo (`manager/BaseManager.h:437, 585, 919-930`).

**Module** (`Source/Module/Module.h`). A module is: `moduleParams` (configuration container), `valuesCC` (container of received values, line 46), `defManager` (list of `CommandDefinition`, line 45), `templateManager` (user command templates, line 60), `logIncomingData`/`logOutgoingData` (lines 36-37), `inActivityTrigger`/`outActivityTrigger` (lines 40-41, outside the hierarchy so as not to generate a listener storm) and `connectionFeedbackRef` (line 43). `hasInput`/`hasOutput` define whether it appears as a source and/or a destination (`setupIOConfiguration`, line 64). Verbs: add from the menu, enable/disable, log incoming/outgoing, test a command (`ModuleCommandTester`, line 52), route values to another module (`canHandleRouteValues`, line 71). Built-in modules, by folder under `Source/Module/modules/`: audio (with FFT and pitch detection), ble, dmx, gpio, http, midi, mqtt, osc (+ dlight, HeavyM, Live, Millumin, Powerpoint, QLab, Reaper, Resolume profiles), oscquery (+ MadMapper), posistagenet, serial, tcp (client, server, PJLink, Watchout), udp, websocket (client and server), abletonlink, gamepad/joycon/wiimote/kinect/keyboard/mouse/streamdeck/loupedeck/ajazz, metronome, signal generator, sequence, state, customvariables, multiplex, os, time, generic, empty. The DMX module has `dmxType`, `sendRate`, `sendOnChangeOnly`, `useMulticast`, `inputUniverseManager`/`outputUniverseManager` and a `thruManager` (`Source/Module/modules/dmx/DMXModule.h:23-42`) — DMX output is multi-universe with "thru", exactly what Spellcaster needs for sACN.

**Custom Variables** (`Source/CustomVariables/CVGroup.h`). A group has `values` (parameters created by the user), `pm` (list of presets) and `morpher`. `controlMode` ∈ {FREE, WEIGHTS, VORONOI, GRADIENT_BAND} (line 28). Verbs: `addItemFromParameter` (line 59 — creates the variable from any parameter in the app), `setValuesToPreset`, `lerpPresets`, `goToPreset(preset, time, curve)` (line 69: go to a preset in N seconds following an easing curve), `randomizeValues` (line 72). `CVPreset` has `defaultLoadTime`, `loadTrigger`, `updateTrigger` (`Preset/CVPreset.h:94-96`) and each parameter inside the preset has `interpolationMode` ∈ {INTERPOLATE, CHANGE_AT_END, CHANGE_AT_START, NONE} (`Preset/CVPreset.h:24-25`). The **Morpher** is a 2D plane: each preset is a point, the `targetPosition` cursor generates weights by a Voronoi diagram or by a gradient band, with an optional background image and physical "attraction" of the cursor (`Preset/Morpher/Morpher.h:28-51`). This is literally a "morphable scene" — an X/Y pad of scenes.

**State** (`Source/StateMachine/State/State.h`) — see its own section below.

**Sequence** (`juce_timeline` + `Source/TimeMachine/`) — see its own section.

**Processor** is the common superclass of everything that "does": `enum ProcessorType { ACTION, MAPPING, MULTIPLEX, CONDUCTOR }` (`Common/Processor/Processor.h:19`). A `ProcessorManager` lives inside each State, each Multiplex and each Conductor. This is the key to the architecture: **state, mapping, action and cue are the same thing in four flavors**, and all of them can be disabled as a block by `setForceDisabled(value, force, fromActivation)` (line 29).

**Multiplex** (`Common/Processor/Multiplex/Multiplex.h`): `count` + `previewIndex` + lists; any Action/Mapping inside it is instantiated N times with an index. It is the "do this for the 24 fixtures" without duplicating 24 mappings.

**Conductor / ConductorCue** (`Common/Processor/Conductor/`): a cue list with `currentCueIndex`, `nextCueIndex`, `loop`, `currentCueName`/`nextCueName`, triggers `triggerPrevious`/`triggerCurrent`, and UI colors for the current and the next cue (`Conductor.h:22-42`). Each `ConductorCue` can tie a sequence (`linkedSequenceParam`) with `autoStart`, `forceStartFrom0`, `autoStop`, `autoNext` (`ConductorCue.h:29-34`). It is the lighting console's GO list, written as an Action.

**Command / CommandDefinition** (`Common/Command/`). A `CommandDefinition` is `{container, menuPath, commandType, params, createFunc, context}` (`CommandDefinition.h:24-36`); `CommandContext { ACTION, MAPPING, BOTH }` (`CommandContext.h:13`) declares whether the command can be triggered (Action) or receive a continuous value (Mapping) or both. An instantiated `BaseCommand` exposes `trigger(multiplexIndex)` and `setValue(var, multiplexIndex)` (`BaseCommand.h:56-59`) — the same object serves both "fire this" and "send this value to this". Commands can be abstracted into a reusable `CommandTemplate`.

**Dashboard item** (`dashboard/`): `Dashboard` is a `BaseItem` with `itemManager`, `isBeingEdited` and an optional password (`Dashboard.h:22-25`). Item types in the repository: `DashboardControllableItem` (parameter/trigger), `DashboardCCItem` (whole container), `DashboardGroupItem`, `DashboardCommentItem`, `DashboardLinkItem`, `DashboardIFrameItem`, `SharedTextureDashboardItem`, `DashboardInspectableItem`. Creation verb: any `Controllable` knows how to turn itself into an item (`Controllable::createDashboardItem()`, `Controllable.h:112`) — dragging a parameter to the dashboard is a method call, not a special case.

## Parameter types (OrganicUI)

Canonical enum: `Controllable::Type { CUSTOM, TRIGGER, FLOAT, INT, BOOL, STRING, ENUM, POINT2D, POINT3D, TARGET, COLOR, TYPE_MAX }` (`controllable/Controllable.h:24`). The factory registers exactly eleven creatable types, in this order: Trigger, Boolean, Float, Integer, Enum, String, File, Point2D, Point3D, Target, Color (`controllable/ControllableFactory.cpp:17-27`).

Default widget for each type (`createDefaultUI`):

| Type | Widget | Source |
|---|---|---|
| Trigger | button; image and "blink" variants | `Trigger.h:24-27` |
| Boolean | toggle | `BoolParameter.cpp:38-40` |
| Float | slider if it has a range, otherwise label; `UIType { NONE, SLIDER, STEPPER, LABEL, TIME }` | `FloatParameter.cpp:50-53`, `FloatParameter.h:29` |
| Integer | stepper | `IntParameter.cpp:120-127` |
| String | text field, optionally multiline; `UIType { TEXT, FILE }` | `StringParameter.cpp:46-56`, `StringParameter.h:22` |
| Enum | dropdown (or button bar, `EnumParameterButtonBarUI`) | `EnumParameter.cpp:473-475`, `EnumParameter.h:14` |
| Color | color picker | `ColorParameter.cpp:196-199` |
| Point2D | two coupled sliders (`DoubleSliderUI`) | `Point2DParameter.cpp:222-227` |
| Point3D | three sliders (`TripleSliderUI`) | `Point3DParameter.cpp:246-251` |
| Target | control address picker, with type filtering | `TargetParameter.h:106-107, 43-44` |
| File | field with browse | `FileParameter.h` (registered in `ControllableFactory.cpp:23`) |

Declarable attributes (exact name accepted in JSON and by script). `Controllable::getValidAttributes()` returns `{ "enabled", "canBeDisabled", "targetType", "searchLevel", "allowedTypes", "excludedTypes", "root", "labelLevel", "saveValueOnly" }` (`Controllable.cpp:402-405`); `Parameter` adds `"alwaysNotify"` (`Parameter.cpp:411-416`). The setter also accepts `"description"` and `"readonly"`/`"readOnly"`, the latter mapped to `setControllableFeedbackOnly(value)` (`Controllable.cpp:359-382`). That is:

- `enabled` — bool, field `Controllable::enabled` (`Controllable.h:37`).
- `readOnly` → `isControllableFeedbackOnly` (`Controllable.h:41`): the widget shows the value but does not allow editing. That is how a "received value" differs from a "configuration parameter", and not by a different widget.
- `alwaysNotify` — notifies even when the new value equals the old one (`Parameter.h:63`). Necessary for OSC/DMX where repeating matters.
- `hideInEditor` is not a `Controllable` attribute: it is a `ControllableContainer` field, saved in the JSON only when true (`ControllableContainer.cpp:983`); in `module.json` Chataigne itself turns it on by itself when the parameter container ends up empty (`Module.cpp:214`).
- Range: `setupFromJSONData` reads `min`, `max` and `default` (`Parameter.cpp:646-657`).

Beyond the value, every `Parameter` has a **control mode**: `ControlMode { MANUAL, EXPRESSION, REFERENCE, AUTOMATION }` (`Parameter.h:36-41`). Any parameter can be switched, from the context menu, to "JS expression", "reference to another parameter" or "automation with its own curve" — without there being a patch node for it. There is also `colorStatusMap` (`Parameter.h:79`): a value→color map, that is, the widget changes color according to the value. And `ValueInterpolator` (`Parameter.h:235-279`), which fades any parameter to a target value in N seconds on its own thread — fade is a primitive of the parameter, not of the scene.

## State Machine

A `State` is a `BaseItem` with a `ProcessorManager` inside it (`State.h:23-35`). It therefore contains its own Actions, Mappings, Multiplexes and Conductors.

State parameters:

- `active` (bool) — "if active, this state's actions and mappings take effect; otherwise this state does nothing" (`State.cpp:23`).
- `loadActivationBehavior` ∈ {Restore last state, Activate, Deactivate} (`State.h:25-26`, `State.cpp:24-25`). The "activate on start" is this enum, not a boolean.
- `checkTransitionsOnActivate` (bool) — on activation, evaluates transitions that are already true; without it, a condition has to become false and true again to fire (`State.cpp:26-27`).
- `focusOnLastActionTriggered` (bool) — scrolls the view to the last action fired (`State.cpp:30`).

Diagram of what happens:

```
activate state S
  └─ S.pm.setForceDisabled(false)            → S's Actions/Mappings take effect again
  └─ notifies listeners (UI lights up)
  └─ S.pm.checkAllActivateActions()          → Actions with role ACTIVATE run now
  └─ for each transition T leaving S:
        T.forceCheck(false)                  → validity state without firing
        T's ActivationCondition conditions receive valid = (type == ON_ACTIVATE)
        if checkTransitionsOnActivate and T.cdm valid:
              T.triggerConsequences(true); break

deactivate state S
  └─ ActivationConditions of S's outgoing transitions become invalid
  └─ S.pm.checkAllDeactivateActions()        → Actions with role DEACTIVATE run
  └─ notifies listeners
  └─ S.pm.setForceDisabled(true)             → S's Actions/Mappings stop taking effect

transition T (source → dest), when its conditions become true
  └─ if source.active:
        T.triggerConsequences(true)          → the transition's consequences
        source.active = false                (turn the source off first)
        dest.active   = true                 (then turn the destination on)
```

Source: `State.cpp:62-124` and `StateTransition.cpp:63-74`. The comment in the code explains the order: turn the source off first "in case the destination instantly reactivates this one" (`StateTransition.cpp:70`).

A **transition is an Action** (`StateTransition.h:15-17`) with `sourceState`/`destState` and with activation definitions enabled in its condition manager (`StateTransition.cpp:23`). Practical consequence: a transition has conditions and consequences like any action — you can send OSC "on the way" between two states. In the file, it is saved by short name: `"sourceState"` and `"destState"` store the `shortName` (`StateTransition.cpp:41-45`) — a fragile reference by name, not by UID.

Multi-activation is allowed: `StateManager::checkStartActivationOverlap` and `getLinkedStates` exist precisely because several states can be active at the same time (`StateManager.h:52, 68`). It is not a single-state machine.

## Sequences

`Sequence` is a `BaseItem` + `Thread` + `AudioIODeviceCallback` (`juce_timeline/timeline/Sequence/Sequence.h:17-22`) — the same sequence can be pinned to the audio clock.

Transport and parameters (`Sequence.h:30-55`): `startAtLoad`, `totalTime`, `currentTime`, `playSpeed`, `loopParam`, `fps`, `autoSnap`, `bpmPreview`, `beatsPerBar`, `evaluateOnSeek` ∈ {NEVER, ONLY_PLAYING, ONLY_NOT_PLAYING, ALWAYS}, and the triggers `playTrigger`, `pauseTrigger`, `stopTrigger`, `finishTrigger`, `togglePlayTrigger`, `prevCue`, `nextCue`, plus `isPlaying`. `viewStartTime`/`viewEndTime`/`viewFollowTime` are saved parameters: the timeline zoom is part of the document.

**Cues**: `TimeCue` has `time` and `cueAction` ∈ {NOTHING, PAUSE, LOOP_JUMP}, plus `loopCue` (the jump target) and the `playFromHere` trigger (`Cue/TimeCue.h:21-26`). This is the "pause on trigger" and the "loop" asked for. In Chataigne, `ChataigneCue` adds a `ConditionManager` (`ChataigneCue.h:22`): the cue is only active if the conditions match — a conditional cue.

**Layers** (`SequenceLayerManager`), existing types: Trigger, Audio, Mapping (1D, 2D, Color) and SequenceBlock (blocks that play another sequence). Each layer is a `BaseItem` with a saved `uiHeight` (`Layer/SequenceLayer.h:27`) and knows how to answer `selectAllItemsBetween`, `getRemoveTimespan`, `getInsertTimespan`, `getSnapTimes` (lines 31-40) — inserting/removing time is a layer operation, with undo.

- **Trigger layer**: `TimeTrigger` = `time` + `isTriggered` + `flagY` (the flag's vertical position in the UI) (`Trigger/TimeTrigger.h:20-24`). In Chataigne, `ChataigneTimeTrigger` carries a `ConsequenceManager` (`ChataigneTimeTrigger.h:22`): the marker on the timeline fires the same consequence list as an Action.
- **Mapping layer**: contains a whole `Mapping` and an automation; parameters `alwaysUpdate`, `sendOnPlay`, `sendOnStop`, `sendOnSeek` (`MappingLayer.h:24-27`), method `getValueAtPosition(float)` and `exportBakedValues` (lines 38-39).
- **Automation**: `Automation` is a `BaseManager<AutomationKey>` with `position`, `length`, `value`, `valueRange`, `viewValueRange`, `rangeRemapMode` ∈ {ABSOLUTE, PROPORTIONAL} (`automation/Automation.h:23-35`). Each `AutomationKey` has `position`, `value` and `easingType` (`AutomationKey.h:21-25`). Easing types: `LINEAR, BEZIER, HOLD, SINE, ELASTIC, BOUNCE, STEPS, NOISE, PERLIN` (`easing/Easing.h:20`) — note NOISE and PERLIN, which turn a curve segment into a generator. The automation also has interactive simplification of a hand-drawn stroke (`addFromPointsAndSimplifyBezier`, `launchInteractiveSimplification`, `Automation.h:46-52`) and an `AutomationRecorder`.

Sync (only in Chataigne, `ChataigneSequence.h:29-51`): master audio module, MTC sent and received with `mtcFPS` and `resetTimeOnMTCStopped`, LTC with `ltcSyncTolerance`, `ltcOutOfRangeMode` ∈ {DO_NOTHING, JUMP_TO_CLOSEST, JUMP_TO_START, JUMP_TO_END}, `ltcMode` ∈ {RECEIVE, SEND, BOTH}, `syncOffset` and `reverseOffset`.

Comparison with `design/SHORTCUTS.md`: Chataigne's transport is poorer than Spellcaster's map. There is no J/K/L, no In/Out, no separate cue markers, and `Space` only plays if `useSpaceBarAsPlayPause` is on (`TimelineAppCommands.h:19`). What Chataigne has and `SHORTCUTS.md` does not yet define: `Ctrl+B` creates a cue at the position, `Shift+PageUp/PageDown` navigates cue by cue, `PageUp/PageDown` steps in time increments, `Ctrl+←/→` moves the selected item one frame (`TimelineAppCommands.cpp:11-57`).

## Mappings and Actions

**Mapping** is a chain with four stages, each one its own `BaseManager`: `im` (inputs), `mappingParams`, `fm` (filters), `om` (outputs), plus `outValuesCC` (`Mapping.h:25-29`).

- **Input** (`MappingInput.h`): `StandardMappingInput` points a `TargetParameter` at any parameter in the app (line 76); `ManualMappingInput` creates a loose parameter to operate by hand (line 107). `triggersProcess` (line 21) decides whether that input triggers the recalculation — several inputs, only one of them as the trigger.
- **Filters** (`Filter/filters/`, complete list): Delay, Script, Time; color: ColorRemap, ColorShift; condition: Condition; conversion: Conversion, Merge, SimpleConversion; number: Crop, CurveMap, Damping, Freeze, Inverse, Lag, Math, OneEuro, SimpleRemap, SimpleSmooth, Speed; string: String. Each filter returns `ProcessResult { CHANGED, UNCHANGED, STOP_HERE }` (`MappingFilter.h:28`) — a filter can interrupt the chain, and that is how "Condition" becomes a gate. Filters can also exclude channels (`excludedChannels`, line 26) and have their own parameters linkable to other parameters (`ParamLinkContainer`, line 29).
- **Output** (`MappingOutput.h:15-17`): it is a `BaseCommandHandler`, that is, a module command in MAPPING context receiving `setValue`.
- Processing mode: `ProcessMode { VALUE_CHANGE, MANUAL, TIMER }` with `updateRate`, and the keys `sendOnInputChangeOnly`, `sendOnOutputChangeOnly`, `sendAfterLoad`, `sendOnActivate` (`Mapping.h:31-39`). `sendOnActivate` is worth copying: on entering a state, the mapping re-emits the current value, so the light does not stay "stuck" at the previous state's value.

**Action** (`Action.h`): `cdm` (conditions), `csmOn` and `csmOff` (consequences for true and for false), `triggerOn`, `triggerOff`, `triggerPreview` (lines 34-40). `Role { ACTIVATE, DEACTIVATE }` (line 26) defines whether the action runs when activating or when deactivating the state that contains it. `autoTriggerWhenAllConditionAreActives` (line 29) allows using the Action as a mere indicator, without automatic firing.

Available conditions (`Action/Condition/conditions/`): StandardCondition (with comparators per type — Bool, Enum, Number, Point2D, Point3D, String), ConditionGroup (nested AND/OR), ManualCondition, ScriptCondition, MultiplexIndexCondition and ActivationCondition, whose `Type { ON_ACTIVATE, ON_DEACTIVATE }` (`ActivationCondition.h:20`) is what allows "on entering this state, do".

A Consequence (`Consequence.h:15-17`) is also a `BaseCommandHandler`. Therefore **a mapping output and an action consequence are the same object**, only the `CommandContext` changes. Spellcaster gains the same if `send(universe, data)` and "cue" are clients of the same registry.

An authoring shortcut worth copying: right-clicking any parameter on screen offers "Add & Link to Custom Variable..." and "Add & Link to Sequence..." — the second creates the mapping layer in the chosen sequence, already with the output pointed at that parameter and with the range copied (`Source/MainComponent.cpp:64-141`). Mapping does not start in the mapping panel; it starts at the widget.

**Module Router** (`Module/Routing/ModuleRouter.h`): source, destination, list of source values and the triggers `selectAllValues`, `deselectAllValues`, `routeAllValues` (lines 24-35). It is the shortcut for "throw everything from this module into that one" without writing N mappings.

## Visual states

- **Live module**: two `TriggerImageUI` in the header, one for input and one for output, fed by `inActivityTrigger`/`outActivityTrigger`, and a connection `BoolToggleUI` (`ModuleUI.h:23-27`; triggers in `Module.h:40-43`). It blinks per event, not by polling.
- **Value feedback**: `Controllable::isControllableFeedbackOnly` (`Controllable.h:41`) draws the widget as read-only; `ParameterUI` repaints through `UITimerTarget`/`handlePaintTimer` (`ui/ParameterUI.h:16, 49-50`), with grouped timers instead of a repaint per event.
- **Color by value**: `Parameter::colorStatusMap` (`Parameter.h:79`) and `ColorStatusUI` — a value can light the widget in a declared color.
- **Warnings**: any object can inherit `WarningTarget` and call `setWarningMessage(msg, id)` / `clearWarning(id)` (`warning/WarningTarget.h:29-30`); `WarningReporter` is a singleton that indexes target→address→message and emits WARNING_REGISTERED/UNREGISTERED events (`warning/WarningReporter.h:20-21, 40`). The Warnings panel lists everything and each line resolves to the guilty object (`warningResolveInspectable`, `WarningTarget.h:22`). The state has `showWarningInUI = true` explicitly (`State.cpp:44`).
- **Log**: `CustomLogger` keeps at most 2000 entries (`logger/CustomLogger.h:4`); each entry has time, content, source and severity, and there is optional writing to a file (`FileWriter`, lines 38-46).
- **Detective**: singleton `BaseManager<ControllableDetectiveWatcher>` with `watchControllable(c)` (`controllable/detective/Detective.h:12`). It is a "watch this parameter" that plots the value history in a panel — a signal debugger, not a code one.
- **Parrot**: generic recorder/player. It has `targetsCC` (list of watched parameters), `recordManager`, `status` ∈ {IDLE, RECORDING, PLAYING}, `playProgression`, `loop`, `forceValueAtStartRecord`, `trimToFirstData`, `trimToLastData` and the start/stop record, play/pause/stop triggers (`parrot/Parrot.h:22-44`). It records any set of parameters and plays them back, with no timeline. `Controllable::isControlledByParrot` (`Controllable.h:50`) marks the parameter being driven by the recording.
- **Dashboard as state**: `DashboardControllableItem` has `showLabel`, `textColor`, `textSize`, `opaqueBackground`, `contourColor`, `contourThickness`, `customLabel`, `forceReadOnly` (`DashboardControllableItem.h:13-21`). Appearance is item data, not theme data.

## Shortcuts

From the code; `commandModifier` is Ctrl on Windows. Source: `juce_organicui/app/OrganicMainComponentCommands.cpp:55-250` and `juce_timeline/TimelineAppCommands.cpp:11-57`.

| Action | Key | Source |
|---|---|---|
| New | `Ctrl+N` | `OrganicMainComponentCommands.cpp:62` |
| Open | `Ctrl+O` | `:67` |
| Open last document | `Ctrl+Shift+O` | `:72` |
| Save | `Ctrl+S` | `:77` |
| Save as | `Ctrl+Shift+S` | `:82` |
| Save copy | `Ctrl+Alt+S` | `:87` |
| Project settings | `Ctrl+;` | `:97` |
| Preferences | `Ctrl+,` | `:102` |
| Full screen (kiosk) | `F11` | `:111` |
| Undo | `Ctrl+Z` | `:120` |
| Redo | `Ctrl+Shift+Z` or `Ctrl+Y` | `:126-127` |
| Copy / cut / paste | `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | `:137, :148, :157` |
| Duplicate | `Ctrl+D` | `:168` |
| Delete | `Delete` or `Backspace` | `:179` |
| Select all | `Ctrl+A` | `:187` |
| Previous / next item | `↑` / `↓` (with `Shift` extends) | `:195, :207` |
| Move item up / down | `Alt+↑` / `Alt+↓` | `:219, :231` |
| Toggle Dashboard edit mode | `Ctrl+E` | `:243` |
| Sequence play/pause | `Space` (only if `useSpaceBarAsPlayPause`) | `TimelineAppCommands.cpp:12` |
| Previous / next cue | `Shift+PageUp` / `Shift+PageDown` | `:17, :22` |
| Previous / next time step | `PageUp` / `PageDown` | `:27, :32` |
| Start / end | `Home` / `End` | `:37, :42` |
| Add cue at the position | `Ctrl+B` | `:47` |
| Move one frame forward / back | `Ctrl+→` / `Ctrl+←` | `:52, :57` |

Panels have command IDs reserved from `0x31000` onwards (`ShapeShifterManager.h:89-90`), so each panel can get a shortcut through the menu, but none comes mapped by default. This machine's `Chataigne.settings` confirms it: `keyMappings` is empty (`<KEYMAPPINGS basedOnDefaults="1"/>`), that is, the user can remap everything through the JUCE mechanism and nobody remapped anything.

## File (.noisette, module.json)

**`.noisette`** is pure JSON. The extension is declared in the constructor: `Engine("Chataigne", ".noisette")` (`ChataigneEngine.cpp:17`).

Root (`juce_organicui/engine/EngineFileDocument.cpp:471-488`):

```
{ "metaData": { "version": "...", "versionNumber": n },
  "projectSettings": {...},        // only if non-empty
  "dashboardManager": {...},       // only if non-empty
  "parrots": {...},                // shortName of the ParrotManager
  "layout": {...},                 // the same format as .chalayout, embedded in the show
  ...engine containers... }
```

Plus Chataigne's five keys, each being the `shortName` of the corresponding manager (`ChataigneEngine.cpp:133-146`): **modules** (`BaseManager<Module>("Modules")`), **states** (`"States"`), **sequences**, **routers** (`"Routers"`) and the variable group (`CVGroupManager`). All are loaded in that same order in `loadJSONDataInternalEngine` (lines 162-182).

Each manager serializes `{"items": [...]}` (`manager/BaseManager.h:919-930`); each item serializes `{"type": ..., "niceName": ..., "parameters": [...], "containers": {...}}` with `hideInEditor`, `editorIsCollapsed`, `removable` and `customData` written only when different from the default (`ControllableContainer.cpp:972-1016`). A parameter is only written if it was overridden (`Parameter::shouldBeSaved`, `isOverriden`, `forceSaveValue`, `Parameter.h:89-92`) — the file stores the delta against the default, not the complete state.

`Engine` keeps a list of `breakingChangesVersions` and a `convertURL` to migrate old shows by a script on the server (`ChataigneEngine.cpp:25-38`). Fifteen versions broke compatibility between 1.6.12 and 1.9.17.

**`module.json`** — the custom module format, confirmed by two real files and by the parser (`Source/Module/Module.cpp:212-341`):

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

(source: `tommag/Sample-Chataigne-module/module.json`, read in full).

Parser rules: a node's `type` can be `"Container"`, and then it becomes a recursive subcontainer with optional `index` and `collapsed` (`Module.cpp:301-315`); any other `type` falls through to `ControllableFactory` by the type name (`ControllableFactory.cpp:116-126`), accepting `min`/`max`/`default` and the attributes listed in the parameters section. Each command can declare `"context": "action" | "mapping" | "both"` (`Module.cpp:236-241`). There is `dependency`, with `{source, value, check, action}` where `check` ∈ {equals, notEquals, lessThan, greaterThan} and `action` ∈ {show, enable} (`Module.h:86-93`, `Module.cpp:328-341`): a parameter shows or enables according to another's value, declaratively, without scripting. The Launchpad X `module.json` (11.7 KB) uses this and shows that the tree can have hundreds of generated parameters (81 pad colors in `Row 1..8` containers).

## What Spellcaster should copy

- **A typed parameter is the unit of the interface, and the widget is derived from the type.** Eleven types, one `createDefaultUI` function per type, declarative attributes (`readOnly`, `enabled`, `min/max/default`, `alwaysNotify`). That fulfills `PRINCIPIOS.md` §1 ("every widget is a visible node") without writing one widget per command. It benefits all five functions, but above all the **orchestrator** and the **Aprendiz** (which then knows how to talk about any parameter by its address). Source: `Controllable.h:24`, `ControllableFactory.cpp:17-27`.
- **A single "command" object serving both triggering and continuous value, distinguished by `CommandContext { ACTION, MAPPING, BOTH }`.** In Spellcaster that is the registry: each `@command` declares whether it accepts GO, whether it accepts a value, or both — and cue, mapping and CLI become clients of the same object. It benefits **DMX scenes and cues** and the **orchestrator**. Source: `CommandContext.h:13`, `BaseCommand.h:56-59`, `MappingOutput.h:15`, `Consequence.h:15`.
- **`sendOnActivate` on the mapping and `Role {ACTIVATE, DEACTIVATE}` on the action.** On entering a state, everything re-emits; on leaving, the exit list runs. It is what keeps the light from being stuck at the previous state's value. It benefits **DMX scenes and cues** and the **interactive stage set**. Source: `Mapping.h:36`, `Action.h:26`, `State.cpp:71, 108`.
- **A filter chain with `ProcessResult { CHANGED, UNCHANGED, STOP_HERE }`.** A filter that interrupts the chain is the conditional gate without inventing a new node type; `Damping`, `Lag`, `OneEuro`, `Speed` and `CurveMap` are exactly what is missing between a sensor and a galvo. It benefits **NDI → ILDA** (smoothing a contour before it becomes a frame) and the **interactive stage set**. Source: `MappingFilter.h:28`, folder `Filter/filters/`.
- **Creating the mapping from the widget, with a right-click.** "Add & Link to Sequence" creates the layer, the output and copies the range in one click. Applied to Spellcaster: click a channel in the Patch and choose "create a keyframe in this timeline" or "tie to this scene". It benefits **DMX scenes and cues** and the **orchestrator**. Source: `Source/MainComponent.cpp:80-141`.
- **A cue with a declared action (`NOTHING / PAUSE / LOOP_JUMP` + loop target) and a conditional cue.** It covers "pause on trigger", "loop between two points" and "only stop here if X" without logic in the GUI. It benefits the **ILDA player** (frame loop) and **DMX scenes and cues**. Source: `Cue/TimeCue.h:23-26`, `ChataigneCue.h:22`.
- **Multiplex: one action instantiated N times with an index.** One mapping for 24 fixtures instead of 24 mappings. It benefits **DMX scenes and cues** and the **orchestrator**. Source: `Multiplex.h:17-27`.
- **WarningReporter as a central service, with the guilty object clickable.** A single panel listing "universe not responding", "galvo not keeping up", "kpps ÷ points below the limit" — exactly the lines foreseen for the Aprendiz in `design/TEMAS.md:15`. It benefits the **Aprendiz** and the **ILDA player**. Source: `warning/WarningReporter.h:28-29`, `WarningTarget.h:22, 29-30`.
- **Detective and Parrot.** "Watch this parameter and show me the history" and "record what I move and play it back" are two rehearsal tools that no cheap competitor has. The Parrot solves show rehearsal without a timeline. It benefits the **interactive stage set** and **DMX scenes and cues**. Source: `detective/Detective.h:12`, `parrot/Parrot.h:22-44`.
- **Morpher: presets as points on a plane, weight by Voronoi.** It is an X/Y pad of scenes that already exists and works; it fits the **PAPER THEATER** (moving the cursor across the scale model mixes wing flats). It benefits **DMX scenes and cues**. Source: `Morpher.h:46-51`, `CVGroup.h:28`.

## What NOT to copy

- **Free dock layout (ShapeShifter) as the default.** The container tree with draggable gaps lets the operator dismantle the screen and never find the button again — the opposite of `PRINCIPIOS.md` §3 ("nothing elastic, nothing that moves on its own", 8 px grid, discrete sizes). Copying the *serialized format* (tree, direction, preferred size) for the Faces is useful; copying the *free dragging* is not. Source: `ShapeShifterContainer.h:40-51`.
- **Reference between objects by short name.** `StateTransition` writes `sourceState`/`destState` as `shortName` (`StateTransition.cpp:41-45`) and `TargetParameter` stores a text address with a repair routine (`tryFixBrokenLink`, `useGhosting`, `ghostValue`, `TargetParameter.h:33-34, 86`). Renaming a state or a fixture breaks the show, and the patch is heuristic. In Spellcaster: stable UID in the `.spell`, name for display only.
- **Four control modes per parameter (MANUAL / EXPRESSION / REFERENCE / AUTOMATION) hidden in the context menu.** Each one is invisible logic in the graph: the value changes and there is no node explaining why. It violates `PRINCIPIOS.md` §1 ("forbids a magic shortcut with no corresponding node"). If Spellcaster wants expressions, they need to be a visible node. Source: `Parameter.h:36-41`.
- **Appearance saved inside each item.** `itemColor`, `viewUIPosition`, `viewUISize`, `listUISize`, `miniMode` in `BaseItem` (`manager/BaseItem.h:26-31`) and the whole tab color palette inside `DashboardManager` (`DashboardManager.h:52-59`) mix data and theme. `PRINCIPIOS.md` §5 forbids it: "a Face that fixes a color". Position belongs to the Face; color belongs to the Theme; neither belongs to the show object.
- **The web dashboard as a separate application.** It is 29 JS bundles and 20 CSS from an Ember app versioned separately (`dashboard/index.html`, `dashboard/assets/`), downloaded over HTTP from the author's server (`DashboardManager.h:78-86`) — a second code base, a second UI, and the item only recognizes part of the types (the installed bundle names only `DashboardGroupItem`, `DashboardCommentItem` and `DashboardLinkItem`, although the C++ defines eight). If Spellcaster has a native web GUI with skins, the "dashboard" should be one more Face, not a second product.
- **A sequence dependent on the audio clock and on the audio thread.** `Sequence` inherits `AudioIODeviceCallback` and `timeIsDrivenByAudio()` (`Sequence.h:20, 120`) — good for a show with a soundtrack, but it ties the transport to the audio device, and on a Pi with no sound card that is a problem. Spellcaster Lite needs its own clock, with audio as an optional synchronizer.
- **Fifteen compatibility-breaking versions "solved" by a remote converter.** `breakingChangesVersions` + `convertURL` pointing at a PHP on the author's site (`ChataigneEngine.cpp:25-38`) means that opening an old show depends on a third-party server being up. `.spell` needs a version number in the file and local migration, inside the binary itself.
- **`Space` as play/pause conditioned on a global boolean** (`useSpaceBarAsPlayPause`, `TimelineAppCommands.h:19`) and no panel shortcut mapped by default. `design/SHORTCUTS.md` already solves it better: `Space` always plays, panels on `Shift+1..7`, and the menu shows the shortcut next to the item.
