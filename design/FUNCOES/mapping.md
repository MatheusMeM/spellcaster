# Mapping mode — `Ctrl+Shift+A`

Request from Matheus (2026-09-09), literal: *"and a mapping system just like resolume Ctrl shift A"*. The key is fixed by the owner and is not up for discussion; what this file does is say what the mode shows, what it records and how that resolves into a command.

Source: **Resolume Arena**, online manual, pages *Keyboard Shortcuts*, *MIDI Shortcuts*, *DMX Shortcuts*, *OSC* and *Shortcuts* (table). Verbatim quotes. Where Resolume has four modes and we have one, the reason is written down.

| Item | Who solved it best | Why |
|---|---|---|
| Entering the mode | Resolume | It recolors the whole interface and says right to your face what is mappable: *"Everything blue can have a shortcut assigned to it."* A "pick the control from a list" dialog would be the same information hidden |
| Learn gesture | Resolume | *"click on it with the mouse. Now press the spacebar, and voila"* — click the control and play the input. Two actions, no text box |
| Mode panel | Resolume | A tab that **only exists inside the mode**, with the complete list, sortable, and duplicates marked in red |
| Delete | Resolume | `Backspace` or Delete in the context menu |
| Behavior (toggle, piano, range) | Resolume, with a cut | `Mode` = Toggle/Value/Mouse, `Piano` and `Invert` as toggles, `Range` as a min/max slider. We keep range and piano; the rest becomes graph (§5) |
| Textual address | Resolume | *"The addresses are all fixed and set up already... Unlike MIDI and keyboard shortcuts, that require you to first link a control to a specific shortcut."* It is rule 2 of `FUNCOES/README.md`, already ours |
| Four modes, one per protocol | none | **We do not copy it.** See §2 |
| Target scope (By Position / This Clip / Selected) | none | **We do not copy it for now.** See §7 |

## 1. What the mode does on screen

`Ctrl+Shift+A` turns it on and off. `Esc` also turns it off — Resolume, *Shortcuts*: `Stop Shortcut Editing | Esc`.

Inside the mode:

1. **The whole interface recolors**, on every page: timeline (`index.html`), patchbay, teatro, face, laser 3D, midi. Resolume, *Keyboard Shortcuts*: *"The interface will now turn partially blue."* Here the paint is the amber of `PRINCIPIOS.md §2`, because it is a state and a state has only one color. What does **not** recolor is not mappable, and that is the information.
2. **Every mappable control shows its ADDRESS in text**, on it or next to it. Resolume shows the already assigned shortcut (*"If the Bypass button was big enough, you could even read it had the spacebar assigned to it"*); we show the address **always**, because the address is the identity (rule 2) and because it is what the operator types in the CLI, sends over OSC and writes in a cue.
3. **Mode panel**, bottom right corner, only exists inside the mode (Resolume, *Keyboard Shortcuts*: *"This tab is only visible while you're in Shortcuts mode"*). Columns: input, address, type, range. Sortable by any column; **duplicates in red** (*"which makes it really easy to spot double assigned shortcuts. These will be marked in red"*).
4. **Learn**: click the control, then play the input — key, MIDI note/CC, OSC message. Whatever arrives first becomes the mapping.
5. **Delete**: with the row or the control selected, `Backspace` or Delete in the context menu (Resolume, identical text on all three pages).
6. **Leaving undoes nothing.** The map belongs to the show and is already saved when the mode closes.

The overlay is a single layer, written once, and for that reason it **requires every page to talk over the same bus**. Today there are three WebSocket clients: `bus.js` (patchbay, laser, face), the one in `timeline.js:64-106` and the one in `teatro.js:71`, the last two with a `ponytail:` telling us to swap them. Swapping is a prerequisite of this function, not an improvement (`pontos-falhos.md` item 15).

## 2. One mode, not four

Resolume has four entry doors, each with its own color: `Shift+Ctrl+K` keyboard, `Shift+Ctrl+M` MIDI, `Shift+Ctrl+O` OSC, `Shift+Ctrl+X` DMX (*Shortcuts*, table). The color changes per protocol (*MIDI Shortcuts*: *"The interface will now change color depending on the protocol"*; DMX is *"a nice pastelly yellow"*).

We have **one** mode, `Ctrl+Shift+A`, for the reasons below, and Resolume's three keys become a **filter inside the mode**:

- The protocol already comes with the input itself. A key is a key, a MIDI note is a note, an OSC message is a message — there is no ambiguity to resolve with a separate mode. Resolume needs all four because in DMX there is no "play the control" (that is why there is *DMX Learn* and "Create DMX Shortcut" on the right button).
- Four colors go against `PRINCIPIOS.md §2` ("one accent, and it means state"). Four modes with four colors is four states for the same thing.
- The owner fixed one key. Four entries would be four keys.

Inside the mode, `K`, `M` and `O` **restrict the source that the next learn accepts** — useful when a MIDI keyboard is sending clock and the operator wants to map a computer key without capturing the first note that goes by. Bare keys, focused panel, and the panel table filters along. That is the only thing Resolume's three keys buy here, and it is enough to justify them.

There is no DMX input mode: DMX-in does not exist in the engine (there is no `in.dmx` in the closed catalog of `script/src/graph.rs`). When it exists, it enters as a `dmx:` source in the same table, with no new mode.

## 3. Key conflict: `Ctrl+Shift+A`

`design/SHORTCUTS.md` already uses `Ctrl+Shift+A` for **"Select none"** (the pair of `Ctrl+A`, origin "both": Premiere and Resolve).

The owner fixed `Ctrl+Shift+A` for the mapping mode. So "select none" moves, and it moves to **`Alt+A`**, because the grammar of `SHORTCUTS.md` itself already says what to do: *"Ctrl = command, Shift = extends/widens, **Alt = variant/clears**"*, and the map already applies that three times (`Alt+I`, `Alt+O`, `Alt+X` clear In/Out). Clearing the selection with `Alt` is the house rule applied to itself, not an exception.

(Source note, for the record: in Resolume `Ctrl+Shift+A` is *"Open Advanced Output"*, not the shortcuts mode — the modes there are `Shift+Ctrl+K/M/O/X`. The owner asked for the key, not the page; the key stays as he asked.)

`A` alone is automation mode (`daw-arranjo.md` D1) and `Alt+A` does not collide with it.

## 4. Addresses

An address is the registry name, grouped by `/` (rule 2 of `FUNCOES/README.md`; convention already fixed in `DECISOES.md` 09/09: *"A graph port is written `<uid>/<port>`... the port name already carries `/` and so the port is the registry address itself"*).

| Address | Control | Type | Resolves to |
|---|---|---|---|
| `transport/play` | play button | trigger | `resume` |
| `transport/pause` | pause button | trigger | `pause` |
| `transport/stop` | stop button | trigger | `stop` |
| `transport/locate` | ruler, position field | value (s) | `locate {t: $<duration>}` |
| `transport/loop` | loop button | toggle | window state (`daw-arranjo.md §4.2`) |
| `track/<i>/mute` | M of the header | toggle | `show_patch {ops:[{op:"add", path:"/tracks/<i>/mute", value:…}]}` |
| `track/<i>/solo` | S of the header | toggle | same, `/solo` |
| `track/<i>/lock` | padlock | toggle | same, `/lock` |
| `cue/<i>/go` | cue row, grid cell | trigger | `cue_go {index:<i>}` |
| `cue/go` | GO button | trigger | `cue_go {}` (next) |
| `level/<u>/<ch>` | programmer fader | value 0..255 | `level_set {universe:<u>, address:<ch>, value:"$255"}` |
| `level/clear` | release button | trigger | `level_clear` |
| `fixture/<name>/<channel>` | fixture fader | value | `fixture_set {name, channel, value:"$255"}` |
| `laser/1/arm`, `power`, `play`, `shutter` | buttons of the 3D model | trigger | `laser_*` (`integracao-laser.md`, already in `DECISOES.md`) |
| `laser/1/kpps` | slider | value | `laser_param {feed:"laser", path:"kpps", value:"$"}` |
| `laser/1/limit/r|g|b`, `curve/r|g|b`, `geo/scale`, `dmx/addr` | panel sliders | value | `laser_param` |
| `ilda/fps`, `dev/pps` | player panel | value | `ilda-player.md §1` |
| `<mod>/<path>` | module parameter | value | `<mod>_param {feed:"<mod>", path, value}` (`DECISOES.md` 09/09) |
| `marker/<name>` | marker on the ruler | trigger | `input {key:"marker:<name>", value:1}` |
| `widget/<id>` | Face widget | value | `input {key:"widget:<id>", value}` |

The `laser/*` and `ilda/*` names are not invented here: they come from `integracao-laser.md` and from `ilda-player.md §1`, and `DECISOES.md` (09/09) already recorded the list. This file only adds `transport/*`, `track/*`, `cue/*` and `level/*`, which are the addresses of the controls that the timeline and the teatro draw.

**An address does not exist without a control.** An address in the table and no widget on screen is the "declaration without implementation" that `DECISOES.md` already flagged as a problem in `modules/laser.json`. The proof of that is a test (§8).

## 5. Types: what the mapping does, and what the graph does

Resolume has, per shortcut: `Mode` (Toggle / Value / Mouse, and Velocity on a MIDI note), `Piano`, `Invert`, `Range` with min and max, plus four more modes for CC only (Absolute / Button / Relative / Fake Relative). It is a lot, and the reason is that Resolume does not have a graph behind it.

We do. The split:

**The direct (flat) map does two things, with no state:**

- **trigger** — the input arrives, the command runs. `144/60` → `cue_go`.
- **value** — the input arrives with a number, and it goes into the arguments. It is the `"$"` that the `midi` workstream already implemented (`spellcore/engine/src/midi.rs:85-107`): `"$"` becomes the value 0..1, `"$<n>"` becomes `round(value * n)` as an integer (`"$127"` = raw MIDI byte, `"$255"` = DMX level). Recursive inside a list and inside an object.

**Range.** Resolume, *Keyboard Shortcuts*: *"With the Range option, you can see what values the slider should jump to when the button is pressed and released"*; and for absolute CC, *"If you want, you can Invert this behaviour, or set a specific Range."* It fits in a third form of the same token, with no new field: **`"$<min>..<max>"`** → `min + value * (max - min)`, rounded as `"$<n>"` already is. Invert is `"$255..0"` — `min > max` itself, which is how Ableton solves it too (§33.2.3: *"You can reverse this behavior by setting a Min value that is higher than its corresponding Max value"*). One more `match` in `expande()`, and no new key in the file.

**Piano** (hold turns it on, release turns it off). Resolume: *"they will be on for as long as you hold the key down, and turn off when you release"*. **It needs no field at all in MIDI**: note-on and note-off are different statuses (`144/60` and `128/60`), so they are two rows of the map, and piano is mapping both. For a computer key and for OSC, the key gains the suffix `^` (`"key:Space^"` = on release), which is one line in the dispatcher and no new structure.

**Toggle, latch, counter, ramp, threshold, delay.** **They do not go into the map.** They go to the graph, which already has the nodes in the closed catalog (`logic.*`, `math.*`, `time.*`, `state`, `cmd` — `script/src/graph.rs:1-70`), and the route is `in.midi` → `logic.toggle` → `cmd`. The mapping mode offers a "send to the graph" button that creates that route and opens the PATCHBAY on it. Reason: keeping state in two places is the double source that this document exists to avoid, and the graph is the place that was already designed for it (`orquestrador.md`).

**Out, and why:**

- **Mouse mode** (Resolume: *"you can use the mouse to control a parameter while you have the shortcut pressed"*). A pretty gesture, with no request behind it, and it would fight with clip dragging.
- **Relative / Fake Relative** (endless encoder, with Steps, Step Size, Loop). It comes in when there is a surface with an endless encoder on stage; today there is none and the `midi` workstream already recorded "one MIDI port per process" as a limit.
- **Velocity** (MIDI note, force becomes value). One line when a sensitive pad exists: it is `"$"` reading `data2` instead of firing 1. Recorded, not implemented.
- **Shortcut Groups** and *Select Next/Previous/Random Item* (Resolume). That is the graph with `logic` and `state`.

## 6. How this is saved, and how it unifies with the `midi` workstream

The `midi` workstream (branch `frente/midi`, commit `39b2c4d`) already created this, and it is right:

```json
"midi": { "144/60": {"cmd": "resume", "args": {}},
          "176/1":  {"cmd": "laser_param", "args": {"feed":"laser","path":"kpps","value":"$"}} }
```

— an `extra` block of the show (`midi.rs:62-68`), key `"<status>/<data1>"` validated (`midi.rs:110-122`), commands `midi_map` / `midi_unmap` / `midi_maps` / `midi_learn` / `midi_last`.

**The unification is one line: the same block, with the source in the key prefix.**

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

Why this is not a new format: **the prefix is already the vocabulary of the engine.** `input {key}` documents exactly those keys (`registry.rs:110-112`: *"widget:go", "key:Space", "osc:/spell/go", "module:laser/stat/fps"*), and `chave()` in the graph produces the same five (`script/src/graph.rs:280-290`). The `"midi"` block of the `midi` workstream is that map with the prefix implicit.

Consequences, and this is what avoids two sources of truth:

1. `"midi"` becomes `"map"`, and the key becomes `"midi:144/60"`. `migrate()` (`show.rs`) does the conversion by prefixing; no existing show breaks.
2. `midi_map {key, cmd, args}` becomes `map_set {key, cmd, args}`, with the same validation: the command exists in the registry, the key is well formed per source (`chave_ok` of `midi.rs:110-122` becomes the `midi:` branch). `midi_map` can stay as an alias that prefixes, or go away — the `comandos` workstream decides.
3. `midi_learn` becomes `map_learn {source}` and it is what the overlay calls.
4. **One single dispatcher.** Today `midi::liga(key)` (`midi.rs:73-82`) reads the block and calls the registry from the MIDI pump. It becomes `map::liga(key)`, called from three places: the MIDI pump, the GUI's `onkeydown` and the OSC receiver. `input {key, value}` remains the single door to the graph, and the map is consulted **first**: if the key is in the map, the command runs; always, the event also goes to the player hooks (which is what `midi.rs:pump` already does).
5. **The rule that closes the matter:** the direct map and the graph never store the same thing. Map = key → command, no state. Graph = everything with state. A key can be in both (it fires the command and feeds the route), and that is deliberate, not duplication: they are different effects of the same input.

`design/laser/bind.js` (round 5 prototype) has a third format, with the key `note:1:60` / `cc:1:7` and persistence in `localStorage`. `DECISOES.md` (09/09, `midi` workstream) already recorded the conflict and sent it to a vote. The answer of this file: **the engine key wins** (`midi:144/60`), because it is the one the graph's `in.midi` already uses and the one PRD §10 fixed; and **`bind.js` stops persisting in `localStorage`**, because mapping belongs to the show and `FUNCOES/README.md §12` does not allow show state outside the `.spell`.

## 7. What stays out

- **Shortcut Target** (Resolume: *By Position* / *This Clip, Layer or Group* / *Selected*). It is the real problem of "I mapped the mute of track 3 and then reordered the tracks", and Resolume warns: *"When you delete that specific clip, layer or group, of course the shortcut disappears with it!"*. Here that is the same item as `daw-arranjo.md §4.3` (index versus `uid`). **It is not decided here**: if the track gains a `uid`, the address becomes `track/<uid>/mute` and the problem disappears with no scope mode at all. It goes into the vote together.
- **Mapping presets** (Resolume: separate XML, swappable by dropdown). The map belongs to the show. When the same surface exists in two shows, the way is for the show to import a map file, not for the GUI to keep presets.
- **OSC output / feedback** (Resolume: *"By right clicking, you can enable OSC output for this and only this button"*; and the `midi` workstream already recorded LED and motorized fader as out, awaiting vote). It stays out here too, for the same reason: it is the return half and it needs a product decision.
- **DMX-in** (Resolume, *DMX Shortcuts*): with no `in.dmx` in the catalog, there is nothing to map.
- **Fixed OSC addresses with no mapping** (Resolume, *OSC*: *"The addresses are all fixed and set up already"*). That we **already have and it is better**: every registry command is an OSC address by construction, and `GET /commands` lists them. The mapping mode is for the opposite direction (a physical input → an address), not for inventing an address.

## 8. Tests

| What it proves | How |
|---|---|
| `expande("$", 0.5)` | `0.5` |
| `expande("$255", 0.5)` | `128` (integer, not `127.5`) |
| `expande("$0..40000", 0.5)` | `20000` |
| `expande("$255..0", 0.25)` | `191` (invert by `min > max`) |
| `map_set` with a key with no prefix | error, with the list of accepted prefixes |
| `map_set` with a nonexistent `cmd` | error (the `midi` workstream already has this test: `tests/midi.rs:26`) |
| Migration | a show with a `"midi"` block loads with `"map"` and `midi:` keys |
| Dispatcher | `map::liga("key:Space")` returns `("resume", {})` after `map_set` |
| Piano | `map_set("key:Space^")` and `map_set("key:Space")` coexist and fire different commands |
| **Every address has a control** | walk the table of §4 and check that each address appears in a `data-addr` of some page of `spellgui/web/`; whatever does not appear, fails |
| **Every control has an address** | the inverse: every clickable element of the five pages has `data-addr`, or is in an exception list declared in the test |

The last two are the ones that keep the mapping mode from becoming a pretty screen with half the controls dead.

Visual proof: headless screenshot of `index.html`, `patchbay.html`, `teatro.html`, `face.html` and `laser.html` with the mode on, showing the overlay and the addresses — and the count of mappable controls per page in the panel footer.
