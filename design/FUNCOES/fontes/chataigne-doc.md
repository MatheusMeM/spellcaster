# Chataigne — digest of the official documentation (Notion)

Companion to `chataigne.md`. That file audits the **installed app and the C++ source code**
(`Sequence.h`, `Mapping.h`, `Action.h`, `Parrot.h`, `OrganicMainComponentCommands.cpp`, ...) and is the
authority on structures, enums, parameters and command shortcuts. **This file repeats none of
that.** Here is only what the documentation adds: the mouse gestures (which exist in no command
table anywhere), the author's conceptual framing, the `module.json` format, Multiplex,
Conductor, Morpher, the Web Dashboard — and the pages that were never written.

Capture: **Browser panel**, because Notion renders through JavaScript and returns nothing to a
`WebFetch`. Date: **2026-09-09**.

## Sources read

Root: https://benkuper.notion.site/The-Amazing-Chataigne-Documentation-079bd5a0b7e648bbbfe34c3c869a3985
Text of each page saved to `<scratchpad>/fontes/chataigne/<slug>.txt` (21 files, 53,355 bytes),
each with the source URL and the date in the header. **They do not enter the repository.**

| File | Notion pages covered |
|---|---|
| `00-index.txt` | root + the raw `a[href]` list (24 internal subpages + 1 external) |
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
| `sequence-layers.txt` | Sequence Layers **+ the 6 subpages** (Trigger, Mapping, Mapping 2D, Color, Audio, Sequence Layer) |
| `scripting-introduction.txt` | Introduction to Scripts |
| `scripting-reference.txt` | Scripting Reference (**truncated**, see below) |
| `custom-variables.txt` | Introduction to Custom Variables + Example 1 + Example 2 |
| `morpher.txt` | The Morpher : 2D Interpolations made fun |
| `dashboard.txt` | Introduction to the Dashboard + Web Dashboard |
| `detective-e-parrot.txt` | Introduction to the Detective + Introduction to the Parrot (**both empty**) |

**24 pages listed at the root, 24 visited, 21 files** (short pages on the same topic were
grouped into a single file). Adding the 6 Sequence Layers subpages, which do not appear in the root
index and were only discovered by re-reading the `a[href]` entries inside `main`: **30 pages captured**.

### Failures and gaps, declared

1. **"Introduction to the Detective" and "Introduction to the Parrot" are empty.** Both exist in the
   index, they open, and the body has **zero** paragraphs, lists, tables or images — `get_page_text`
   returns only the title, and the programmatic check confirmed it (`{blocks: 1, imgs: 2, len: 26}`).
   Consequence: **everything known about Detective and Parrot comes from `chataigne.md`**
   (`Detective.h:12` with `watchControllable(c)`; `Parrot.h:22-44` with `status ∈ {IDLE, RECORDING,
   PLAYING}`, `targetsCC`, `loop`, `forceValueAtStartRecord`, `trimToFirstData`/`trimToLastData`), not
   from this documentation. This matters for the `gravar-dmx` workstream: **the product closest to
   what we want to build did not document the feature**.
2. **"Scripting Reference" was truncated** at the capture's 30,000-character limit, in the middle of the
   "Util object" section. The script-type subpages (Module, Condition, Consequence, Mapping Filter,
   Mapping Output) **were not captured**. This digest uses only what came before the cut.
3. **The "Parameters" section of the Mappings page is empty in the source** — the literal text is
   `To fill`. It is a gap in the documentation, not in the capture.
4. The Raspberry Pi page lives in **another Notion workspace**
   (`sugar-ocean-c6b.notion.site`) and was not visited — it is outside the requested scope.
5. The root itself warns: *"as the software is in constant evolution, this documentation may not
   always be up to date"*. Where the documentation and `chataigne.md` diverge, **the source code wins**.

---

## 1. The declared philosophy (and why it matters to us)

The author defines the product like this: in artistic projects with technology, the creator uses several
pieces of software and devices — one for audio, one for video, one for mapping, one for light, an Arduino for motors —
and the communication between them becomes the problem. Chataigne "wants to be the **conductor of that
technological orchestra**: on its own it does almost nothing visible to the audience, but it is the one that sees the whole picture and
makes sure each one gets what it needs".

Two consequences the author himself draws, and that Spellcaster should adopt in writing:

- **"Since Chataigne is a conductor and does not play an instrument, it is useless if you do not have
  an orchestra."** It is honest and it is a scope decision: the product does not try to also be the generator.
- **Two central concepts, not one**: the **State Machine** handles real-time interaction; the **Time
  Machine** handles time-based control. And about the built-in audio timeline, he says himself:
  *"if you need complex audio processing, use a dedicated software and control it from
  Chataigne"*. Acknowledging the limit of your own feature, in the documentation, is a pattern worth copying in
  `design/FUNCOES/README.md`.

Origin of the name: "chataigne" is chestnut in French; the story involves a love affair and the author found it
funny. Recorded because it explains `.noisette` ("hazelnut") as the project file extension.

---

## 2. The screen: seven panels, and one layout decision

The documentation enumerates the panels, which is what `chataigne.md` covers from the code side
(`ShapeShifterManager`). What the doc adds is **the declared role of each one**:

| # | Panel | Role, in the doc's words |
|---|---|---|
| 1 | **Module** | "your starting point". When creating a project, you list here **all** the software and devices you are going to interact with, before any rule |
| 2 | **State Machine** | the real-time interaction rules |
| 3 | **Time Machine** (or Sequence Editor) | timeline sequences: triggers and parameter animation over time |
| 4 | **Inspector** | "your main editing panel, you will spend a lot of time there". **Anything selectable in the software shows up there in detail**; changing the selection changes the content |
| 5 | **Logger** | "your verbose friend". It says what worked and what failed, and shows useful things such as **the machine's IP addresses when a network module is created**. The user can also log their own messages and values |
| 6 | **Help** | shows useful hotkeys and how to use **the item under the cursor** |
| 7 | **Warnings** | used **when loading a file**, to check what broke: missing files, script errors, broken links |

The UI framework is called **Organic UI** and has a **ShapeShifter** mechanism: the panel layout is
freely rearrangeable and **savable as different layouts depending on what you are doing**.

Three things to take away:

1. **The working order is embedded in the panel numbering**: declare the devices first,
   then the rules, then time. Our equivalent is: patchbay → mapping → timeline.
2. **The Logger spits out the machine's IPs by itself** when a network module is born. Near-zero cost,
   it answers the most frequent question of any networked show — and Spellcaster has sACN, Art-Net and
   OSC, all three with the same problem.
3. **The Warnings panel is explicitly "for when you open a file"**. A `.spell` that opens with a
   missing fixture, a vanished `.ild` file or a duplicated universe needs exactly that, and
   `chataigne.md` already details the mechanism behind it (`WarningTarget`, `WarningReporter`).

---

## 3. Cheat Sheet — the copied table

`chataigne.md` already has the **menu commands** with keys (`Ctrl+N`, `Ctrl+B` creates a cue, `Shift+PageUp`
navigates cues, `Ctrl+E` dashboard edit mode, ...), read from
`OrganicMainComponentCommands.cpp` and `TimelineAppCommands.cpp`. Not repeated here.

What **only** exists in the documentation are the **mouse gestures and the drag modifiers** — which do not
go through the JUCE command system and therefore appear in no shortcut source file.
It is the most valuable part of this whole page.

### Command line

```
./Chataigne [-r] [-f file] [-headless] [-forceGL / -forceNoGL] [<file>]
```

| Flag | Effect |
|---|---|
| `-r` | **resets the preferences** |
| `-f <file>` | opens the file (also works by just putting the name at the end, without `-f`) |
| `-headless` | runs **with no GUI, no window** |
| `-forceGL` / `-forceNoGL` | forces the value of "use opengl renderer"; `-forceNoGL` is there to work around a problematic video driver |

For Spellcaster Lite on the Raspberry Pi, `-headless` and "open a file from the positional argument, with no
flag" are the minimum contract — and `-r` is the panic button every product with saved preferences
needs and almost none has.

### Editing parameters — the product's general rule

> "Right-click on any parameter in the Inspector to reveal a new world of possibilities!"

And what shows up: **change the parameter's range** (when allowed), **send it to the Dashboard**,
**copy its script or OSC control address**. The doc generalizes: *"in general, in the software,
trying to right-click and see whether there are more options is a good idea"*.

This matches what `chataigne.md` found in the code (`MainComponent.cpp:64-141`: "Add & Link to Custom
Variable…", "Add & Link to Sequence…"). Together they give the rule: **every parameter on screen is the starting
point for mapping, animating, exposing and addressing.** None of that starts in a dedicated panel.

### Selection and content transfer

| Gesture | Effect |
|---|---|
| `Ctrl` + click on an item | **toggles** that item's selection state |
| `Shift` + click on an item | selects everything **up to** it |
| `Alt+O` | **imports a LilNut file** and adds the content to the existing project |
| `Alt+S` | **exports the current selection** to a LilNut file |

The content that travels in a LilNut is declared: **Modules, States, Custom Variables, Module Router and
Sequences**. That is: a partial exchange format, carrying pieces of a project between projects, with a
closed list of what is transportable. `.spell` needs the analogue — export "only these scenes and this
patch" without exporting the whole show.

### Inspector

| Gesture | Effect |
|---|---|
| `Shift` + click on a Container header | folds/unfolds **all children** |
| **`Alt` + drag** on a slider or numeric label | **decreases** the drag sensitivity |
| **`Shift` + drag** on a slider or numeric label | **increases** the drag sensitivity |

Note the difference against Ableton and Resolve, where only "`Shift` = fine" exists
(`ableton12.md` §10.6; `resolve20-guia.md` §4): Chataigne gives **both sides**, fine and coarse, with
two modifiers. For a 0-255 DMX parameter where you sometimes want 1 step and sometimes 100,
that is not a luxury.

### State Machine (the node view)

| Gesture | Effect |
|---|---|
| **Middle button drag**, or `Alt` + drag | pan the canvas |
| **`F`** | frames the view on the center of **all** States |
| **`H`** | goes back to the **absolute center** of the view (home) |
| Mouse wheel | scrolls up and down ("may change in the future") |
| **`Shift` + wheel** | zoom |
| `Shift+Enter` inside a comment | line break |
| `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | copy, paste, duplicate — **valid for all items in lists and views**: States, Mappings, Actions, Modules, Sequences |

**`F` and `H` are two deliberately different things**: "frame what exists" and "go back to the origin". A
patchbay without both is a patchbay where you lose your work. And note the inversion against
Ableton and Resolve: here the **bare wheel scrolls** and **`Shift`+wheel zooms**, on the node canvas.

### Time Machine — timeline manipulation

| Gesture | Effect |
|---|---|
| **Dragging the blue bar** horizontally / vertically | **zoom and time focus at the same time** |
| **Right-click on the blue bar** | resets the view to the whole sequence |
| Right-click **dragging** on the blue bar | selective zoom on a span (absolute) |
| Right-click dragging **on the numbers** of the ruler | selective zoom on a span (relative) |
| **`Shift` + dragging the playhead** | **snaps** the playhead to timeline elements (cues, triggers, other mapping keys…) |
| **Double-click on the ruler numbers** | **creates a Time Cue** |
| `Shift` + dragging a cue | moves with snap (time bar, triggers, other mapping keys…) |

The "blue bar" is the same object as Ableton's Overview (`ableton12.md` §3) and what Resolume calls
the wedge (`resolume-quickstart.md` §4): **a strip above the timeline that is at once the map and
the zoom control, with the same two-axis drag gesture**. Three independent products arrived
at it. There is no reason to invent something else.

What is **exclusive to Chataigne** and worth more: **right-click resets the view**, and **right-click
dragging zooms into the span**. The ruler's right-click is a whole vocabulary that Ableton and
Resolve leave empty.

### Mapping Layer (and Mapping 2D) — curve editing

| Gesture | Effect |
|---|---|
| **Double-click on empty space** | creates a key at that position |
| **Double-click on the curve** | adds a point **keeping the overall shape intact** |
| **`Shift` + dragging a key** | keeps the **value**, moves only the position |
| **`Alt` + dragging a key** | keeps the **position**, moves only the value |
| `Shift+Alt` + dragging a key | moves only the position, **with snap** to elements of the other layers (time bar, cues, triggers, other keys) |
| **`Ctrl` + click on the curve** | **switches the easing type** of that segment |
| **`Ctrl+Shift` + drag** | **draws the curve by hand** |

This is the most directly usable table in the whole file, for the `daw-arranjo` workstream:

- **"adds a point keeping the shape intact"** is an operation distinct from "creates a key". Two
  intents, two gestures, the same double-click with a different target. Ableton does not have the first
  (`ableton12.md` §6: double-click on the background creates a breakpoint, and that is it).
- **`Shift` locks the value, `Alt` locks the position** — two axes, two modifiers, symmetric and
  memorable. Ableton uses `Shift` for "constrain to one axis", without saying which: worse.
- **`Ctrl`+click cycles the easing directly on the segment**, no menu, no inspector. And the existing types
  are in `chataigne.md` (`LINEAR, BEZIER, HOLD, SINE, ELASTIC, BOUNCE, STEPS, NOISE, PERLIN`).
- **`Ctrl+Shift`+drag draws**, and the stroke becomes editable automation — it is Ableton's Draw Mode
  (key `B`) without needing a mode.

### Modules

| Gesture | Effect |
|---|---|
| **Dragging the module into a State** | opens a menu to use it **automatically as the input or output** of an Action or a Mapping |
| Clicking the module's **activity arrows** | toggles Log Incoming / Log Outgoing for that module |

The first is the gesture the `browser-dnd` workstream needs to understand: **dragging does not create "a copy of
the module"; it opens a menu asking which role it will have at the destination**. Drop with disambiguation, instead
of drop with guessed behavior.

---

## 4. Modules and Router — what the doc adds

### Declared anatomy of a module

Six fixed sections in the Inspector, in order: **Header** (enable/disable — a disabled module "does not
update or send anything" — plus the Log Incoming/Outgoing toggles), **Parameters** (host, port, device
name…), **Values** (what the module receives; **some modules have none** if they do not receive or if
the Input is disabled), **Scripts**, **Command Tester** and **Templates**.

Two points:

- **Command Tester**: "sends commands manually to check whether the communication is working;
  **it does not affect the rest of the software**". A Trigger button, plus the **Auto Trigger** option,
  which resends the command every time one of its parameters changes. A test bench that does not dirty the show is exactly what
  almost every lighting software lacks — and it is the answer to "is the projector responding?"
  without arming output.
- **Templates**: customizing a module for a specific use **without writing your own module** —
  you create a Template from a base command, choose **which fields are editable and which
  are not**, and define the default mapping behavior. It is the "command preset with locked fields",
  which is how a team keeps the night's operator from changing what they should not.

### Module catalog, as declared

- **Protocol**: OSC, OSCQuery, MIDI, DMX, Serial, UDP, TCP Client, TCP Server, HTTP, Websocket
  Client, Websocket Server, MQTT, PJLink, PosiStageNet, Ableton Link
- **Hardware**: Sound Card, Wiimote, JoyCon, Keyboard, Mouse, Gamepad, Kinect V2, StreamDeck,
  LoupeDeck, GPIO
- **Software**: DLight, HeavyM, MadMapper, Millumin, QLab, Reaper, Resolume, Watchout, Powerpoint
- **Generator**: Metronome, Signal — **generators are modules**, not a separate category
- **System**: Time, OS

Note what is **not** here: **sACN and Art-Net do not appear in the list**. Only "DMX". That is real space.

### Module Router

"A useful tool when you have many mappings to make from one module to another." **One router
links one input to one output only**, but you can create as many as you want. And the warning: to transfer data
directly between **modules of the same type**, use the module's **pass-through** feature, which is optimized and
simpler.

Two tools for the same problem, with the documentation saying which to use when. The practice is worth copying,
not just the feature.

### `module.json` — the custom module format

Folder at `<Documents>/Chataigne/modules/<name>/`, with a mandatory `module.json`, a `.js` logic script
almost always, and optionally a **32×32** `icon.png`. After touching the JSON:
**File > Reload custom modules — and delete and recreate the module**. There are also **local modules**: a
`modules` folder next to the `.noisette` file makes that module exist only while that project
is open.

Fields, summarized:

| Group | Keys |
|---|---|
| Metadata | `name`, **`type`** (which base module to extend), `path` (submenu), `version`, `description`, `url`, `downloadURL` |
| Base override | `hasInput`, `hasOutput`, `defaults`, **`hideDefaultParameters`** (array of short names), `hideDefaultCommands`, `alwaysShowValues` |
| Content | `parameters`, `values`, `commands`, `scripts` |
| Per command | `menu`, **`callback`** (script function), **`setupCallback`** (dynamic command creation), `parameters` |
| Data types | `Container`, `Boolean`, `Float`, `Integer`, `Enum`, `String`, `File`, `Target`, `Color`, `Point2D`, `Point3D` |
| Per datum | `readOnly`, `shortName`, `description`, `min`/`max`, `default`, **`dependency`** |
| Float UI | `ui`: `stepper`, `slider`, `label`, **`time`** |

Three findings that apply to our fixture editor (`fixtures/`) and to the patchbay:

1. **`type` = inherit from a base module.** A custom module is a declarative specialization of an
   existing one, not a plugin from scratch. A fixture personality should work the same way: it inherits from
   "16-channel moving head" and overrides.
2. **`dependency`** — every datum can declare `{source, value, check ∈ {equals, notEquals, lessThan,
   greaterThan}, action ∈ {show, enable}}`. **Conditional UI is file data, not code.** It is how
   a "gobo rotation" channel appears only when "gobo" is at a certain value, without writing a line.
3. **`ui: "time"` as a float widget type.** Time is not just any float.

And the practical tip, which reveals the architecture: *"to find out a parameter's short name, hover
the mouse over it and look at the control address"*. **Every parameter has a textual address, and the UI
shows it.** The same namespace for script, OSC and JSON — it is the `spellcaster.core.registry` of
`CLAUDE.md`, and it is what `orquestrador.md` already assumes.

---

## 5. State Machine — what the doc adds

`chataigne.md` covers `Action.h`, `Condition`, `Consequence`, `Mapping` and the filter list. Here only
what the documentation says and the code does not show.

### The State Network rule

> "At any moment, there is **only 1 active state inside a state network**."

And the declared consequence: **not linking states to each other** is what allows having several states active at
the same time — as many state networks as you want. That is: **exclusivity is opt-in, expressed by the
transitions, not a global property**. A disconnected graph is an independent group.

This directly answers an open question in Spellcaster: a lighting scene in different universes
has to be able to be exclusive within the universe and independent across universes. **The link is the
declaration of exclusivity.** It is the same rule as Resolume's layer by another route
(`resolume-quickstart.md` §2).

Transitions have two simultaneous declared roles: transferring activation from one state to another
(acting **as an Action, including with its own consequences**, to give different behavior
depending on where you came from) and linking states into a network.

### Actions — the 5 conditions, named

**From Input Value** (the most used), **Scripts**, **Group**, **On Activate**, **On Deactivate**.

The detail that exists only in the doc: **`onActivate` conditions fire when the project is loaded, if the
state that contains them is active** — and that is why "they can be used as an initialization action". A
single mechanism covers "on entering this state" and "on opening the file". Cheap and right.

And: `onActivate`/`onDeactivate` react to **activated/deactivated**, not to **enabled/disabled** — the
doc stresses the difference. Two distinct axes, like visibility × mute in Resolve
(`resolve20-guia.md` §1).

Declared visual feedback: **each validated condition turns green and goes back to gray when invalidated**.
The rule's state is readable in the rule itself, without opening a log.

Consequences: as many as you want, all fired at once ("synchronized control of different modules"),
with two timing options — **delay** after validation and **stagger**, which spaces the
firing of each consequence at a regular interval. Stagger on a lighting cascade is halfway to a
chase effect without writing any effect at all.

### Mappings — string wildcards

What the doc adds to what `chataigne.md` already brings from `Mapping.h`:

- **The lightning symbol next to each input decides whether that input triggers the mapping.** Several
  inputs, one trigger — the doc spells out what it is for.
- **Filters choose which channels they act on**, through the "Channels" menu in the filter's header.
- **Wildcards in string parameters**: `{input:1}`, `{input:2}`… are replaced by the input
  values. `"My value is {input:1}, and second value is {input:2}"` becomes
  `"My value is 0.53, and second value is 127"`.

Template string interpolation in the parameter is the difference between "I can send a dynamic OSC" and
"I need a script". Cost: one regex.

### Multiplex — the find of the page

The problem: several values that need the same filter, or a row of buttons/faders that need
the same treatment — and the alternative would be duplicating N actions and N mappings.

How it works: a **Multiplex** has a **count** parameter that defines how many iterations it handles.
Inside it you create **lists**, all with the length of the count. Actions and Mappings placed inside
the Multiplex **gain new capabilities**: a list can be a condition Input, and a parameter of a
Consequence, a Mapping Output or a Mapping Filter can be linked to an element of the list.

The mechanism: when **one** element of a list changes, it fires the processing of the Action/Mapping that
has that list as input — **and the process carries the index of the element that fired it**, end to end.
That way "the same treatment for many items, with a specific output for each one".

Multiplex wildcards, in string parameters:

| Wildcard | Becomes |
|---|---|
| `{index}` | the index, 1-based |
| `{index0}` | the index, 0-based |
| `{input:1}` | the first value of the mapping's input |
| `{list:names}` | the element with the same index in the "Names" list (camelCase conversion) |

`"Hello {list:names}, you're patient number {index}"` → `"Hello Leon, you're patient number 5"`.

And the fill helper: **Fill… > From expression**, with the path of the first item copied by
right-click → "Copy value" and the index swapped for a wildcard —
`"/modules/OSC/values/track{index}_x"` fills the whole list in order.

**This is the piece Spellcaster needs most and that none of the other five audited apps have.**
A rule written once, applied to 24 identical fixtures, with the index propagating all the way to the output —
it is the difference between patching a rig of 24 pars and patching one par 24 times. Record as a candidate
for `orquestrador.md`.

### Conductor — the cue list Chataigne has

A Conductor creates a **cue list of sequential actions**. Each cue can fire several consequences
(like an Action) **or be linked to a sequence**. The Conductor keeps the **current cue (purple background)** and
the **next cue (orange background)**; when fired, it fires the next cue and increments both.

- **If there is a condition on the Conductor, that is what fires the next cue** — the doc gives the examples:
  a key, or an OSC trigger. That is, **GO is a condition, not a special button**.
- Cue linked to a sequence: when fired, **it plays the linked sequence**. With the
  **"Auto Next On Finish"** option, when the sequence ends it **fires the next cue by itself**.

`chataigne.md` § Sequences states, comparing against `design/SHORTCUTS.md`, that "there are no separate
cue markers" and that the transport is poorer than ours. **That remains true for the sequence**,
but the Conductor is the cue list object missing from that map — it lives in the State
Machine, not in the Time Machine. Scope correction, not contradiction.

For us, the model is exactly what the "cues" function in `design/TEMAS.md` asks for: ordered list,
current cue and next cue highlighted by color, GO as a mappable condition, and "cue = trigger + optional
sequence, with automatic chaining at the end".

---

## 6. Time Machine — and the comparison with Ableton

### What the doc adds

Sequences: "as many as you want, controlled independently" from the Sequence panel. A sequence
is "a time-based object, with its own timeline, containing a group of layers".

**Cues** (Interface and Navigation): marks on the timeline used to **play from there** or **go to the
next**. And the declared use of the pause option: *"pausing the timeline when reaching a cue, which is
convenient in semi-interactive shows — waiting for the actor to finish the line, or the dancer to enter the stage,
before continuing"*. This is the `cueAction = PAUSE` of `chataigne.md` (`TimeCue.h:21-26`) with the
justification that was missing. **It is the "theatrical show" model against the "timed show" one, and the
difference is a three-value enum.**

**Zoom and scroll**: clicking and dragging the blue rectangle above the timeline, up/down and
left/right, navigates and zooms (detailed in §3).

### The six layers, with what only the doc says

| Layer | What the doc adds |
|---|---|
| **Trigger Layer** | "triggers that are like actions, but fire at the instant where they are placed". The doc admits: "they will gain more features in future versions" |
| **Mapping Layer** | contains **an automation — a curve-based animation** — used as the **input of a mapping**. "You can use this layer as a state mapping and add filters and outputs to it" |
| **Mapping 2D Layer** | animates **a point along a 2D path**. The path is created separately; the timeline animates **the position over it** |
| **Color Layer** | animate color over time; **two interpolation types, Linear and Hold**, toggled by **`Ctrl`+click on the keys** |
| **Audio Layer** | several audio clips played over time. "Support is limited"; **it requires a Sound Card module** in the project to play |
| **Sequence Layer** | "control sequences inside sequences. inside sequences. inside sequences." — nesting with no declared limit |

**The most important sentence of the whole Time Machine**: the Mapping Layer **is a mapping**. The curve does not
"control a parameter"; it is the **input** of an input → filters → outputs chain identical to the
State Machine's. Practical consequence: the same curve can go through Damping, Crop, Math, OneEuro,
and come out in several commands at once. **Automation and mapping are not two subsystems.**

For Spellcaster: the **timeline keys must be input to the patchbay**, not a parallel route to
the output. That solves "the timeline sends 0-255 but I want a dimmer curve" without inventing a
second place for curves.

**Mapping 2D separates the path from the position over the path** — and it is exactly the right model for
laser: the `.ild` figure is the path, the timeline animates the progress over it.

### The Recorder — Chataigne's "record dmx", documented

On the Mapping Layer: pick an **Input value**, enable the **Arm** parameter, and start playing. The value
**appears in red on the layer** while recording and, **when the sequence is stopped, it automatically converts
into an editable curve**. The doc's example is recording perlin noise from the Generator module.

Four ready-made decisions for the `gravar-dmx` workstream:

1. **Arm is a layer parameter**, not a global mode. You record one line at a time, explicitly.
2. **Red during recording**, and the red is the still-raw data.
3. **The conversion into an editable curve happens on stop**, not during — you do not edit what is still
   being recorded.
4. **The result is an editable curve, not an immutable track.** Combined with the interactive simplification that
   `chataigne.md` finds in the code (`launchInteractiveSimplification`, `Automation.h:46-52`) and with
   Ableton's Simplify Envelope (`ableton12.md` §6, p.500), the three products agree: **high-rate
   recording has to become few editable points, or it is useless.**

### Ableton Live 12 × Chataigne — the timeline, side by side

| | Ableton Live 12 | Chataigne | For us |
|---|---|---|---|
| Ruler | two at once: bar-beat-16th and minute-second-ms (p.160-162) | one; `fps` and `bpmPreview` are sequence parameters (`chataigne.md`) | we need both: cue in clock time, key in frames |
| Map/overview | Overview: dragging horizontally scrolls, vertically zooms; double-click goes back to the whole (p.160-161) | blue bar: drag on both axes; **right-click resets**; right-click dragging zooms into the span | the bar's right-click is Chataigne's net gain |
| Keyboard zoom | `+`/`-`, `Z` on the selection, `X` steps back one (p.163, p.999) | none; mouse gesture only | copy Ableton's `Z`/`X` |
| Scroll | `Shift`+wheel horizontal, `Ctrl`+wheel zoom, `Alt`+wheel track height (p.163, p.991) | in the State Machine: wheel scrolls, `Shift`+wheel zooms | **real conflict**: the two conventions are opposite. Decide one and apply it on both screens |
| Follow / centered playhead | Follow switch, which **pauses on edit and comes back on restart** (p.163) | `viewFollowTime` is a saved sequence parameter (`chataigne.md`) | Ableton's auto-pause behavior is what Chataigne lacks |
| Markers | **Locators**, triggerable, mappable, with "Loop to Next Locator" and "Set Song Start Time Here" (p.166-167) | **Time Cues**, with `cueAction ∈ {NOTHING, PAUSE, LOOP_JUMP}` and `playFromHere`; in Chataigne the cue also carries **conditions** (`ChataigneCue.h:22`) | **the conditional cue is Chataigne's and has no parallel**; the "loop between two markers" is Ableton's |
| Create marker | Set Locator button, or menu (p.166) | **double-click on the ruler numbers**, or `Ctrl+B` | double-click on the ruler is cheaper |
| Loop | loop brace with `Ctrl+L`, arrows adjust, `Ctrl+↑/↓` doubles/halves (p.169-170) | `loopParam` on the sequence; the loop jump is **a cue with `LOOP_JUMP`** | Ableton's explicit brace for rehearsal; cue with jump for show structure |
| Snap | grid `Ctrl+1..5`; **`Alt` inverts the snap during the action** (p.175) | `autoSnap` on the sequence; **`Shift`+drag snaps the playhead and the cues** | Ableton: snap to a time grid. Chataigne: snap **to elements**. We need both |
| Create key | double-click on the background creates a breakpoint (p.497) | double-click on empty space creates a key; **double-click on the curve adds a point without changing the shape** | the second operation does not exist in Ableton and should |
| Move key | `Shift` constrains to one axis, without saying which (p.498-499) | **`Shift` locks the value, `Alt` locks the position, `Shift+Alt` = position with snap** | copy Chataigne: two axes, two modifiers |
| Segment curve | **`Alt`+drag curves it; `Alt`+double-click goes back to a straight line** — one curve shape only (p.499) | **`Ctrl`+click cycles the easing**: `LINEAR, BEZIER, HOLD, SINE, ELASTIC, BOUNCE, STEPS, NOISE, PERLIN` | Chataigne's easing vocabulary; **`NOISE` and `PERLIN` turn a segment into a generator** |
| Draw | Draw Mode `B` (and holding `B` is momentary) (p.496) | **`Ctrl+Shift`+drag**, with no mode | the modifier removes the need for the mode; the key removes the need for the modifier. Having both is reasonable |
| Simplify | **Simplify Envelope** over a selection (p.500) | interactive simplification after recording (`chataigne.md`) | mandatory on both sides |
| Record | global Automation Arm + Session Record; touch (mouse) and latch (controller) modes (p.491-494) | **Arm per layer**, value in red, becomes a curve on stop | Arm per layer is clearer; Ableton's touch/latch modes are the refinement |
| Audio layer | it is the whole product | "limited support", and the doc tells you to use dedicated software | same stance as Spellcaster with audio |
| Nesting | does not exist: a clip does not contain an arrangement | **Sequence Layer**: sequence inside sequence, no limit | Chataigne's gain — "this scene is a mini-timeline" |
| Cue list | **Follow Actions** per clip/scene, with Chance A/B, Linked/Unlinked, Jump (p.361-362) | **Conductor**: current cue purple, next orange, GO as a condition, Auto Next On Finish | Follow Action is probabilistic; Conductor is deterministic. **A show wants the Conductor** |
| Scope of what you hear | Session ↔ Arrangement mutually exclusive, with "Back to Arrangement" (p.192-193) | active State × playing sequence are independent | Ableton's "Back to Arrangement" is the solution to the conflict; we will have the same problem |

---

## 7. Custom Variables and Morpher

**Custom Variables** are "the place to store, modify and retrieve your own data". Structured into
**Groups**, and **each group has presets**. The doc admits: "the concept is a bit abstract for non-programmers",
and that is why it gives two examples:

1. **Game logic**: a group with `Score` and `HighScore`; one action increments `Score` on each button
   press; another action checks whether it reached 10, shows "YOU WIN" and resets. Show state that is not
   device state.
2. **Particle system presets**: 4 variables in a group, 2 presets, and **a Mapping that takes an
   input signal and uses it to interpolate between the two presets**.

The second is the direct model for "crossfade between two lighting scenes" — and the interpolation is done by
**Special Module commands**, that is, by a module, with the same grammar as everything else.

**Morpher**: putting the Custom Variables group's **Control Mode** into **2D Voronoi** opens the Morpher
panel, where the presets are **positioned on a 2D plane**. A **white target** defines the weight of each
preset according to the distance to it **and to the global layout** (Voronoi proximity algorithm).
Then there are **attraction** and **decay** for "even more unpredictable behaviors".

Declared origin: it was a separate project, a 2D interpolator between multiple presets, that ended up
merged into the Custom Variables system.

For us: **an XY pad that mixes N scenes by proximity** is a whole show control in one widget.
And the architecture teaches more than the feature: **the Morpher is not a new object — it is a "Control Mode" of
a variable group.** Changing a container's control mode changes the UI and the semantics without creating
a new type. Combined with `module.json`'s `dependency` (§4), it is the same principle twice.

---

## 8. Dashboard

"A way to create a custom interface by importing, positioning and styling any
component of your composition on a canvas. You can create as many dashboards as you want."
**`Ctrl+E` toggles Edit mode ↔ Play mode** (already in `chataigne.md`).

The doc is short and even warns: *"a new dashboard, the 'Golden Board', is on the roadmap"*.

### Web Dashboard — the useful part

In **File > Project Settings**, you enable the **Dashboard Server**. With it on, the dashboards are
accessible **from any browser on the network**, by the machine's IP (or `127.0.0.1` on the machine itself) and the port.
**Default port: 9999** — `http://127.0.0.1:9999`.

Addressing and URL parameters:

| URL | Effect |
|---|---|
| `http://<ip>:<port>/` | opens the dashboard server |
| `http://<ip>:<port>/#/<dashboard>` | opens **a specific dashboard** directly |
| `...?disableMenu` | hides the menu |
| `...?disableList` | hides the dashboard page list |
| `...?disableMenu&disableList` | both — the "operator tablet" mode |

This is literally what Spellcaster wants for the web GUI with skins: **the same screen, served over
HTTP, with the chrome removable by query string**, so a tablet on stage shows only the buttons. Cost:
two flags. And port 9999 is a collision candidate — note it in `design/DECISOES.md` together with the ports
of the workstreams.

---

## 9. Scripts — the minimum, since the reference was truncated

Five places where scripts exist, and the doc is explicit that **the available methods and callbacks change
depending on where the script is**: **Module Scripts, Condition Scripts, Consequence Script, Mapping Filter
Scripts, Mapping Output Script**.

What is worth recording from the authoring flow (and it is what Spellcaster should imitate if it ever has scripting):

- When you create a script, the file **is born already filled with content generated dynamically according to the
  object it was created from** — a script created in an OSC module comes with the generic part, plus the module-specific
  part, plus the OSC-specific part.
- **Real-time compilation and interpretation**: saving the file reloads it in the app.
- **The script's state is visible in two places**: a **green** dot in the script's Inspector when it compiles,
  a **red** dot plus a warning when it fails — and the Logger shows **the error and the line**.
- Common functions, none mandatory (the app "optimizes the script according to which ones exist"): `init()`
  right after loading; `update(deltaTime)` called at the rate of the "Update rate" parameter — which **only
  appears in the UI when the function exists** —, with `deltaTime` in seconds and the rate in Hz, adjustable by
  `script.setUpdateRate(rate)`; `messageBoxCallback(id, result)`.
- The referenced types: Trigger and Parameters (Float, Integer, Boolean, String, Color, Target, Enum,
  File, Point2D, Point3D), Container, Manager, States, Automation, Commands, and the objects `script`,
  `root`, `local`, `util`.

**"The parameter only appears in the UI when the function that uses it exists"** is the script version of
`module.json`'s `dependency`. Third time the same principle appears: **the UI is a declared consequence
of the data, never written by hand.**

---

## Adaptation to Spellcaster

Cross-reference: `design/FUNCOES/orquestrador.md` (`spell graph`, PATCHBAY theme, "Chataigne mode").

### What goes into the orchestrator / patchbay

| From the documentation | What it becomes in Spellcaster |
|---|---|
| **Module panel as panel no. 1**, before the rules | the patchbay starts by declaring outputs and inputs; no rule exists before that |
| **Dragging the module into a State opens an "input or output?" menu** | a drop on the patchbay asks the role; it never guesses (`browser-dnd` workstream) |
| **Command Tester that does not affect the rest of the software** | "test this output" without arming the show — it solves the hole `resolume.md` points at ("a Lumiverse existing already means sending") |
| **Command Templates with locked fields** | a command preset where the night's operator only touches what was unlocked |
| **Module Router: one input, one output, as many as you want; and pass-through for the trivial case** | two declared tools, with the doc saying which to use when |
| **`module.json`: `type` inherits from a base module; `dependency` makes conditional UI; `ui:"time"`** | a fixture personality inheriting from a type, with channels that appear according to the value of another channel — it is what Resolume lacks (`resolume.md` § What NOT to copy) |
| **Textual address visible on hovering any parameter** | the registry exposed in the UI; one namespace for GUI, OSC, CLI and MCP |
| **Right-click on a parameter = change the range, send to the Dashboard, copy the address** | mapping and exposing start at the widget, not in a panel |
| **Multiplex: one rule, N items, index propagated to the output** | one rule for 24 identical fixtures, with a specific output per index. **No other audited app has this** |
| **Wildcards `{index}`, `{index0}`, `{input:1}`, `{list:name}` in string parameters** | OSC address and target name assembled by template, without scripting |
| **State Network rule: exclusivity is the link, not a global property** | a scene exclusive within the universe, independent across universes, expressed by the graph |
| **`onActivate` also fires when the file is loaded** | one mechanism covers "on entering the scene" and "on opening the show" |
| **A validated condition turns green and goes back to gray** | the rule's state readable in the rule |
| **Consequences with `delay` and `stagger`** | a lighting cascade without writing an effect |
| **Conductor: current cue purple, next orange, GO as a condition, cue→sequence, Auto Next On Finish** | Spellcaster's cue list, entire |
| **`-headless`, file as a positional argument, `-r` to reset preferences** | the CLI contract for Spellcaster Lite on the Pi |
| **Web Dashboard: port 9999, `#/<name>`, `?disableMenu&disableList`** | web GUI served over the network, with removable chrome by query string, for the stage tablet |
| **Logger spits out the machine's IPs when a network module is created** | sACN/Art-Net/OSC saying which interface they are on, without the operator asking |
| **Warnings panel explicitly "for when you open a file"** | a `.spell` that opens with a missing `.ild` or a duplicated universe warns in a clickable list |

### What goes into the timeline (`daw-arranjo`)

Curve gestures, in full: double-click on empty space creates a key; **double-click on the curve adds a point
without changing the shape**; **`Shift` locks the value, `Alt` locks the position, `Shift+Alt` = position with snap to
elements of the other layers**; **`Ctrl`+click cycles the segment's easing**; **`Ctrl+Shift`+drag
draws by hand**. Plus: **right-click on the zoom bar resets the view, right-click dragging zooms
into the span**, **double-click on the numbers creates a cue**, **`Shift`+dragging the playhead snaps to elements**.

Concepts: **the mapping layer is a mapping** — the curve is the input of an input→filters→outputs chain,
not a parallel route to the output; **Mapping 2D separates the path from the position over it** (the model for
`.ild`); **Sequence Layer nests sequences**; **a cue that pauses the timeline** for a semi-interactive show.

### What goes into `gravar-dmx`

**Arm per layer** (not a global mode), **value in red during recording**, **conversion into an editable
curve on stop**, and **simplification afterwards**. And the record that **the Parrot documentation is
empty** — the only source on it is `chataigne.md`.

### What NOT to copy

- **Leaving two documentation pages empty** in the software most similar to ours. Detective and
  Parrot are the two features Spellcaster would most need to study, and there is not one line.
- **"Parameters — To fill"** published on the Mappings page.
- **`Ctrl+;` for Project Settings and `Ctrl+,` for Preferences**: two adjacent keyboard shortcuts
  for two similar dialogs with opposite scope (one saves in the file, the other on the machine). Swapping one
  for the other is a guaranteed error. If Spellcaster has both, let them be far apart and labeled.
- **Changing `module.json` requiring "Reload custom modules" *and* deleting and recreating the module.** A reload that does not
  reload.
- **The mouse wheel with inverted semantics between screens** — in the State Machine the wheel scrolls and `Shift`+wheel
  zooms; in Ableton and Resolve it is the opposite. The doc even admits "it may change in the future". Choose
  one convention and apply it in the whole app, and write it in `design/SHORTCUTS.md`.

### Open issue for `design/DECISOES.md`

Mouse wheel: `Shift`+wheel = **zoom** (Chataigne, node canvas) or = **horizontal scroll** (Ableton
p.991, Resolve p.58 uses `Shift`+wheel for track height)? The three diverge, and Spellcaster has
a node canvas **and** a timeline. Not decided here.
