# Orchestrator — `spell graph` (PATCHBAY theme, "Chataigne mode")

Connect sources (OSC, MIDI, vectorized NDI, incoming DMX, keyboard, time) to destinations (universes, lasers, cues, player) through routes with filters, without code. It is the Graph panel of `SHORTCUTS.md` (`Shift+3`) and the realization of `PRINCIPIOS.md §1`: the graph is the interface.

| Item | Who solved it best | Why |
|---|---|---|
| Objects and verbs | Chataigne | Module, route, action, multiplex and cue list are the same `Processor` in four flavors; a command accepts firing, a value, or both through `CommandContext` |
| States | Chataigne, with the cook of TouchDesigner | Two activity blinkers per module, fed by event; a central `WarningReporter` with the culprit clickable; the animated dashed cable says the data is flowing; error and warning are a count per node |
| Screen zones | Chataigne | The default layout is exactly Premiere/Resolve: list on the left, canvas in the center, Inspector on the right, sequences and log at the bottom |
| Shortcuts | Blender (node editor) | Enter/leave group, delete and reconnect, mute a cable without deleting it, Add menu with search-as-you-type |
| File | Chataigne `module.json`, with the externalization of TouchDesigner | Module declared in text (`parameters, values, commands, context, dependency`); external reference with a safety copy and `relpath` declared |

## 1. Objects and verbs

The whole hierarchy descends from two types: **container** (node with children) and **controllable** (leaf with a value), and every controllable has an address [fontes/chataigne.md § Objects and verbs]. In Spellcaster the address is the registry name (rule 2).

**Module** — a source, a destination, or both (`hasInput`/`hasOutput`). It has `parameters` (configuration), `values` (what it receives, read-only), `commands` (what it accepts), two activity triggers (input, output) and a connection state [fontes/chataigne.md § Objects and verbs]. Verbs: add from the menu, enable/disable, log input/output, test a command, route all values to another module (the Module Router: source, destination, "route all") [fontes/chataigne.md § Mappings and Actions]. Chataigne's DMX module is multi-universe with `thru`, `sendRate` and `sendOnChangeOnly`; TouchDesigner's DMX Out adds the warning "Rate ≤ 44 Hz" and, for sACN, `cid`, name and **priority** for merging with other sources [fontes/touchdesigner.md § Laser and NDI].

**Command** — the `@command`. It declares `context: action | mapping | both`: whether it accepts GO, a continuous value, or both [fontes/chataigne.md § Objects and verbs]. The same object serves "fire this" and "send this value to this"; the output of a route and the consequence of an action are the same class, only the context changes.

**Route** (mapping) — a chain of four stages: `inputs` (any address; several, only one triggers the recalculation), `filters`, `outputs` (commands in mapping context), plus `mode` (`on change | manual | timer`) and `re-emit on activation` [fontes/chataigne.md § Mappings and Actions]. Resolume adds to the same object what Chataigne does not have: **target scope** (`Selected | This | By position`), input and output ranges separate from the parameter (`in/out` vs `min/max`), and the **feedback path** for a controller LED in the same object (`OutputPath`, `NamedValues` = color table) [fontes/resolume.md § Shortcuts and mapping]. A useful range separate from the physical one is what avoids a scaling node in every route [fontes/resolume.md § What to copy].

**Filter** — a node with a single input and a single output that returns `CHANGED | UNCHANGED | STOP_HERE`. Chataigne's list: Delay, Script, Time; ColorRemap, ColorShift; Condition; Conversion, Merge, SimpleConversion; Crop, CurveMap, Damping, Freeze, Inverse, Lag, Math, OneEuro, SimpleRemap, SimpleSmooth, Speed; String [fontes/chataigne.md § Mappings and Actions]. Plus MadMapper's generators, which are filters between control and value: `time_base` (integrates speed; it exists because `sin(speed*TIME)` jumps when you move the speed), `damper`, `adsr`, `ease`, `incrementer`, `pass_thru` (reads any address of the app) [fontes/madmapper.md § Material/module parameters]. `Condition` with `STOP_HERE` is the gate: there is no "if" node.

**Action** — conditions + consequences for true and for false + `role: on activation | on deactivation` [fontes/chataigne.md § Mappings and Actions]. Conditions: comparison by type, AND/OR group, manual, script, multiplex index, activation.

**Multiplex** — `count` + index: a route or action instantiated N times. "One mapping for 24 fixtures instead of 24 mappings" [fontes/chataigne.md § What to copy].

**Cue list** (Conductor) — current cue, next, loop, previous/current triggers; each cue can bind a sequence with `autoStart`, `forceStartFrom0`, `autoStop`, `autoNext` [fontes/chataigne.md § Objects and verbs]. It is the GO list of a lighting console written as an action; detailed in `cenas-cues-dmx.md`.

**State** — a container of routes and actions with `active`, `on load: restore | activate | deactivate`, `check transitions on activation`. A transition is an action with a source and a destination; it turns the source off before turning the destination on; several states active at the same time [fontes/chataigne.md § State Machine]. **The node catalog of PRD §10 has no state node.** Without it, "when the second act starts, this set of routes takes effect and that one stops" requires a condition hack in every route. Open issue for `DECISOES.md`; the proposed semantics is Chataigne's, whole.

**Cable** — it only connects ports of compatible types. TouchDesigner only cables within the same family and uses Link/Export to cross over [fontes/touchdesigner.md § Objects and verbs]; here the families are the port types (`trigger`, `bool`, `number`, `color`, `xy`, `frame`, `dmx`) and the conversion is a visible filter node, never implicit coercion.

**Group** — a navigable container. `Tab` enters and leaves in Blender (meta-strip, node group) [fontes/blender.md § What to copy]; here it is `Ctrl+]`/`Ctrl+[` (see Shortcuts).

Authoring verbs that are born in the widget, not in the panel: right-click on any parameter offers "Add and link to a sequence" (creates the layer, the output and copies the track) and "Add and link to a variable" [fontes/chataigne.md § Mappings and Actions]; any controllable knows how to become a dashboard item (`createDashboardItem()`) [fontes/chataigne.md § Objects and verbs]; dragging a CHOP channel onto the parameter creates the export, and hovering over the tab switches page mid-drag [fontes/touchdesigner.md § Parameters].

A parameter driven by a cable keeps the constant: TouchDesigner keeps constant, expression, export and bind at the same time, with a little square on the button of the inactive mode that has content [fontes/touchdesigner.md § Parameters]. Here: muting the cable (not deleting it) returns the parameter to the constant, and the field shows the little "has a cable" square. What is not included: Chataigne's four modes hidden in the context menu, "invisible logic in the graph" [fontes/chataigne.md § What NOT to copy]; an expression is a node.

## 2. States

- **Module**: enabled/disabled; connected/disconnected; input and output blinker **by event, not by polling** [fontes/chataigne.md § Visual states]. Capture: green is activity, "activity does not guarantee operation", and the probable cause comes named (`Potentially blocked by firewall`) [fontes/capture.md § States and messages].
- **Cable**: animated dashes while the data flows (TouchDesigner: "a wire with animated dashes = the source is cooking") [fontes/touchdesigner.md § Objects and verbs]; middle button on the cable shows the value going through. Muted cable: dimmed, present.
- **Node**: `mute` (input passes straight through, TD's Bypass), `lock` (freezes the output value, TD's Lock, saved in the file), `solo` (only this one emits; it clears the mute of the others without removing them) [fontes/blender.md § What to copy]. One verb per concept (rule 9).
- **Error and warning as a number**: each node exposes `warnings` and `errors` as a count [fontes/touchdesigner.md § States]; the warnings panel lists them all and each line leads to the culprit (`WarningReporter`, `warningResolveInspectable`) [fontes/chataigne.md § Visual states]. Error on the parameter field, not only on the node (`parms.err.bg`).
- **Active state** (State): lit; the last triggered action can scroll the view to it (`focusOnLastActionTriggered`) [fontes/chataigne.md § State Machine].
- **Watch** (Detective): "watch this parameter and plot the history" [fontes/chataigne.md § Visual states]. A signal debugger, not a code debugger.
- **Pending**: route edited and not applied; red [fontes/touchdesigner.md § States].

Not included: color by node family (seven desaturated hues in TouchDesigner: "when everything is colored, nothing is state") [fontes/touchdesigner.md § What NOT to copy]; family is shape or label. Nor `itemColor` saved in the item [fontes/chataigne.md § What NOT to copy].

## 3. Screen zones

Chataigne's `default.chalayout`, which is already the design of `SHORTCUTS.md § Interface` [fontes/chataigne.md § Anatomy of the screen]:

- **Left**: list of modules (with the two blinkers per line), and below it the show variables.
- **Center**: graph canvas. Chataigne's center tabs: State Machine, Dashboard, Router, Morpher; here only one, the Graph, with states as containers inside it.
- **Right**: generic Inspector, "each object answers `getEditor`", editing N items at the same time is native [fontes/chataigne.md § Anatomy of the screen]. One Inspector, one place; not the three of TouchDesigner [fontes/touchdesigner.md § What NOT to copy].
- **Bottom**: sequences on the left, timeline in the middle, `Help | Log | Warnings` tabs on the right. Log with 2 000 entries and optional recording to file [fontes/chataigne.md § Visual states].

Panel menu: `View, Select, Add, Module`. `Add` opens with search on the first character (TouchDesigner's Tab menu lights up the matching types as you type; Blender does the same with `SEARCH_ON_KEY_PRESS`) [fontes/touchdesigner.md § Anatomy of the screen], [fontes/blender.md § Command palette]. No dockable palette.

Canvas: click on empty space pans without changing zoom; `Shift`+drag is box selection; the wheel is zoom; middle button on the node opens the info popup [fontes/touchdesigner.md § Shortcuts]. A network edge map is not necessary; `Shift+Z` frames.

Performance Face: no canvas. Modules as a list with blinkers, warnings and the active state.

## 4. Shortcuts

`SHORTCUTS.md` applies (`Shift+3` focuses the Graph, `Shift+Z` frames, `Ctrl+C/V/X`, `Delete`, `Shift+D` mute, `Shift+S` solo, `Ctrl+A`). What is added, from Blender's node editor [fontes/blender.md § Shortcuts] and TouchDesigner's Network Editor [fontes/touchdesigner.md § Shortcuts]:

| Action | Key | Origin | Conflict |
|---|---|---|---|
| Add node (menu with search) | `Shift+A` | Blender | none |
| Enter the group / go up one level | `Ctrl+]` / `Ctrl+[` | (ours) | Blender uses `Tab`, which in `SHORTCUTS.md` is the Face switch; TD uses `Enter`/`u`, and `Enter` is GO |
| Delete and reconnect both sides | `Shift+Delete` | Blender `delete_reconnect` | Blender uses `Ctrl+X`, which is cut |
| Mute a cable without deleting it | `Ctrl+Alt` + right-button drag over the cable | Blender `links_mute` | gesture |
| Cut cables | `Ctrl` + right-button drag | Blender `links_cut` | gesture |
| New branch from an already connected output | middle button on the output | TD | gesture |
| Insert a node in the middle of the cable | right button on the cable | TD | gesture |
| Create several nodes already cabled in series | hold `Shift` while creating | TD | gesture |
| Change the input of a node | drag from another node's output over the existing cable | TD | gesture |
| Node lock | `Shift+L` | (ours) | none |
| Rename | `F2` | Blender | none |
| Show the value on the cable | middle button on the cable | TD | gesture |
| Turn global cooking on/off | `Ctrl+Space` | TD | Blender uses `Ctrl+Space` to maximize; `SHORTCUTS.md` maximizes with `` Ctrl+` ``, so it is free |

Grammar of the modifiers on the canvas, from Blender: no modifier = the action; `Shift` = the same over the complement; `Alt` = the inverse or clear; `Ctrl` = the strong variant [fontes/blender.md § Shortcuts]. It matches `SHORTCUTS.md § Grammar`.

## 5. File

**Module manifest**, text, one per folder, Chataigne's `module.json` [fontes/chataigne.md § File]:

```json
{ "name": "...", "type": "osc", "version": "1.0.0",
  "hasInput": true, "hasOutput": true,
  "parameters": { "port": { "type": "int", "default": 9001 } },
  "values":     { "level": { "type": "float", "readOnly": true } },
  "commands":   { "go": { "context": "action", "parameters": {} } },
  "dependency": [ { "source": "mode", "check": "equals", "value": "advanced", "action": "show" } ] }
```

`dependency` shows or enables a parameter according to another, without a script. An encrypted module (MadMapper's `main.ldat`) is the counter-example: "whoever buys the software cannot read or version what runs in their own show" [fontes/madmapper.md § What NOT to copy].

**In the `.spell`**, key `graph`:

- `nodes[]`: `{uid, type, name, params (complete), mute, lock, enabled}`. Reference between nodes by `uid`, never by short name (Chataigne's `sourceState`/`destState` break on rename) [fontes/chataigne.md § What NOT to copy].
- `wires[]`: `{from: uid.port, to: uid.port, muted}`.
- `states[]`: `{uid, name, active, on_load: restore | activate | deactivate, check_on_activate, children[]}`.
- `view`: `{uid: {x, y, collapsed}}`, in a block separate from the nodes, so that the logic diff does not carry the position diff. Never color; that belongs to the Theme (`PRINCIPIOS.md §5`).
- External module: `{path, relpath: show | module, backup: <embedded copy>}`. "A show that references external modules needs to carry a safety copy of each one, otherwise it dies on a thumb drive that does not have the folder" [fontes/touchdesigner.md § What to copy].

Lock saves the frozen value inside the file, like TouchDesigner's Lock flag [fontes/touchdesigner.md § File].

Outside the `.spell`: log, warnings, received values, device list, blinker state, dock layout (Chataigne embeds `layout` in the `.noisette`; no) [fontes/chataigne.md § File].

Tie-break between competing sources: MadMapper's Cue Scheduler checks the clock at 1 Hz and "the last module in the list wins" [fontes/madmapper.md § What to copy]. Here: for a trigger, the last in the list; for a DMX value, the merge policy declared in the patch (`cenas-cues-dmx.md § 1`).
