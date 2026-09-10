# DMX scenes and cues — `spell patch`, `spell scene`, `spell cue` (PAPER THEATER theme)

Patching fixtures into universes, recording values into scenes and cues, and firing them in order with a fade. Covers the Patch (`Shift+1`) and Outputs (`Shift+4`) panels and the cue list inside the Timeline (`Shift+2`).

| Item | Who solved it best | Why |
|---|---|---|
| Objects and verbs | MadMapper for the cue, Capture for the patch, Chataigne for the list | Cue = a list of (address, value) pairs with a per-entry fade; a fixture has four numbers and sequential batch numbering; the patch dialog says how many channels it consumes before recording; Conductor is the GO list |
| States | Resolume, with TouchDesigner's pending and MadMapper's overlay | `connected` is a five-state enum, not a boolean; `connect` is "button pressed", momentary and latch with one primitive; red = it is in the cue, orange = it is, but with another value |
| Screen zones | Capture, with Resolume's output panel | Fixture table navigable like a spreadsheet; universe view in Channels or Fixtures, levels in % or DMX; a panel that shows channel overlap |
| Shortcuts | Chataigne, with Resolume's grammar | `Ctrl+B` cue at the playhead, `Shift+PageUp/Down` previous/next cue; with no modifier it acts on the selected one, `Shift` raises the scope |
| File | MadMapper, corrected | The cue entry schema is the right one; the flattened grid, the embedded thumbnail and the scene as a separate class do not come in; Resolume's personality with no channel type is the example to avoid |

## 1. Objects and verbs

Vocabulary in Portuguese, from Capture's official translation with the corrections in the Spellcaster column [fontes/capture.md § EN → PT vocabulary]: **Aparelho** (fixture), **Patch**, **Universo** (universe), **Canal (ID)** (channel ID), **Unidade** (unit), **Endereço inicial** (start address), **Canais necessários** (channels required), **Facas** (shutters), **Zoom** (not "Zum"), **Gel**, **Instantâneo** (snapshot).

**Fixture.** Four numbers, different things [fontes/capture.md § Patch and universes]:

| Number | What it is |
|---|---|
| Patch | universe + start channel (DMX address) |
| Channel (ID) | the fixture's number on the console (identity, not address) |
| Unit | physical number on the bar or on the stage |
| Circuit | electrical circuit |

Plus `profile` (personality) + `mode`, and **per-instance** overrides: invert pan/tilt/zoom/iris, limit pan and tilt travel, intensity scale [fontes/capture.md § Patch and universes]. "That is how you fix a fixture hung upside down." Verbs: patch, unpatch, duplicate, replace, number in batch (Sequential for Unit, Circuit, Patch and Channel), group.

**Profile.** A list of channels **with a type**: dimmer, pan, tilt, color, strobe, gobo, macro; 16 bits (coarse/fine); ranges with a label ("0-7 closed, 8-134 strobe"). Resolume has none of that, only `ParamRange 0..255` with a free name, and the result in the user's files is `New Parameter 1..34` [fontes/resolume.md § What NOT to copy]. The type decides the fade (a dimmer interpolates, a gobo jumps), the resolution, and what Aprendiz checks. Resolume also shows that a fixture has to accept **two paths**: value per channel (scene) and surface sampling (pixel map, the `DmxSlice/InputRect`) [fontes/resolume.md § What NOT to copy]; the personality's pixel block (`width, height, color format, distribution, gamma`) comes in as a `pixels` channel type.

**Universe.** `name`, `patch base` (numeric position independent of the name), `style: indexed | continuous` (continuous = a single 1-2048 range in theater) [fontes/capture.md § Patch and universes]; per output: protocol, `rate ≤ 44 Hz` (TouchDesigner) [fontes/touchdesigner.md § Laser and NDI], `delay 0-150 ms` (Resolume) [fontes/resolume.md § Fixtures and DMX], sACN `priority`, ArtSync with timeout. Verbs: add (one or N with a base name), zero the levels, restart an external universe. **Declared merge policy** per channel type (HTP for dimmer, LTP for the rest), because "the last one wins by accident of implementation" is MadMapper's defect [fontes/madmapper.md § What NOT to copy].

**Patch dialog.** It shows `channels required` before confirming; universe overflow is a question with "continue until complete", not an error [fontes/capture.md § What to copy]. `spell patch` prints the same thing before writing.

**Cue.** MadMapper's central finding, confirmed in the file [fontes/madmapper.md § Scenes and cues]:

```
cue = { uid, name, comment, fade: {type, duration},
        entries: [ { address, value, fade?: {type, duration} } ] }
```

A cue is a list of (registry address, value) pairs with a per-entry fade, optionally different from the cue's fade ("fade the opacity but not the RGB"). The value can be a number or a reference to another object (`/medias/9`). If every widget is a node with an address, the cue is a diff over the graph and needs no structure of its own. Per channel type, Chataigne's interpolation mode: `interpolate | change at the end | change at the start | none` [fontes/chataigne.md § Objects and verbs].

Verbs: record from the current situation ("Update from current values" in a multi-selection; "CUE ALL" on the fixture) [fontes/madmapper.md § Scenes and cues]; enter edit mode and click the widget to include/exclude; GO, go back, go to; `momentary` (the cue holds while the key is pressed, Resolume's `connect`) [fontes/resolume.md § What to copy]; `follow` (auto-follow: MadMapper's `auto_play` is global with a per-column override; Chataigne's `ConductorCue` ties a sequence with `autoStart`, `autoNext`) [fontes/chataigne.md § Objects and verbs]; `condition` (a `ChataigneCue` is only active if the conditions match) [fontes/chataigne.md § Sequences]; `action on arrival: nothing | pause | jump to` (the `TimeCue`).

**Scene.** A cue with the `exclusive` flag: it records everything and, when fired, whatever is not in it goes to zero. Not a separate class with its own rules on the first row of the grid, as in MadMapper [fontes/madmapper.md § What NOT to copy]. And not Capture's "Scene", which stores the position and visibility of a set object; that is a "set position" and lives in `cenario-interativo.md` [fontes/capture.md § What NOT to copy].

**Cue list.** Conductor: current cue, next, loop, previous/current triggers [fontes/chataigne.md § Objects and verbs]. One list per show or several; MadMapper's 16×8 grid is a Face over the same list, not another object.

**Apply to N.** Multiplex: one cue or route instantiated N times with an index [fontes/chataigne.md § What to copy]. `spell cue set --fixtures 1-24 dimmer 80`.

**Rehearsal without a timeline.** Parrot: records whatever the operator touches in a set of parameters and plays it back (`IDLE | RECORDING | PLAYING`, loop, trims) [fontes/chataigne.md § Visual states]. Morpher: scenes as points on a plane, weight by Voronoi, X/Y cursor mixes [fontes/chataigne.md § What to copy]; it is the PAPER THEATER scale model mixing wing flats.

**A parameter with three values at once.** Constant, cue value, cable/timeline value, stored together, with "has content" in the inactive mode [fontes/touchdesigner.md § What to copy]. "Take a fixture out of the timeline's control to test a fixed value at 11 pm and give it back later without reprogramming." Resolume does the same thing by swapping the parameter's child (`PhaseSourceStatic | Timeline | TransportTimeline | DashboardLink`) without changing type or address [fontes/resolume.md § Parameter types].

## 2. States

**Universe / output**: `no signal`, `searching`, `connected` (Capture's `(no)`, `Searching..`, `(auto)`) [fontes/capture.md § What to copy]; `blocked by firewall` as a named cause; **not armed / armed** (none of the six apps has it; Resolume sends as soon as the Lumiverse exists) [fontes/resolume.md § States]; **rehearsal** (the engine computes, it does not send). Two enable levels as in Resolume: the whole universe and the fixture [fontes/resolume.md § States].

**Cue**: an enum, not a boolean. Resolume's `Clip.connected` has five states and "each state has its own LED color" [fontes/resolume.md § States]. Here: `empty`, `loaded`, `next` (standby), `live`, `fading` (progress bar in the cell itself, reflecting the longest transition) [fontes/madmapper.md § States], `error`. A new transition on the same parameter discards the previous one.

**Entry in edit** (cue edit mode): a red outline = it is in the cue; orange = it is, but with a value different from the current one, or absent in part of the multi-selection [fontes/madmapper.md § Scenes and cues]. Zero modals.

**Fixture field**, inks of Blender's State block [fontes/blender.md § States]: `overridden by a cue` (the `inner_overridden`), `with a keyframe at this time` (`inner_key`), `animated, no key here` (`inner_anim`), `driven by a cable` (`inner_driven`), `different from the scene` (`inner_changed`), each one with a selected pair. Three levels of not-acting: a fixture with a muted cue = dimmed and editable (`active`); an output not armed in rehearsal = locked (`enabled`); a universe that does not answer = red (`alert`) [fontes/blender.md § What to copy]. Divergent values in a multi-selection: `Mixed values`, a string of its own [fontes/capture.md § States and messages].

**Pending**: a cue edited and not recorded, red [fontes/touchdesigner.md § States].

**Preview/blind** is an engine mode (rehearsal × live), not a little box per universe like Capture's `BlindLevelsMode` [fontes/capture.md § What NOT to copy].

## 3. Screen zones

- **Patch (`Shift+1`)**: a fixture table like a spreadsheet, "navigated and edited with the arrows and Enter", sortable by header, with search [fontes/capture.md § Shortcuts]. Fixed columns: Name, Profile, Mode, Universe, Address, Channels, Channel (ID), Unit, Circuit, Group. Not seven columns configurable by preference [fontes/blender.md § What NOT to copy]. Below the table, the universe view: `Mode: Channels | Fixtures`, `Levels: % | DMX` [fontes/capture.md § Patch and universes], showing occupancy and channel **overlap**, "the only way for the operator to see that they patched two fixtures on top of each other before the show" [fontes/resolume.md § What to copy]. The Resolume user wrote "40 - 68" in the slice name because the UI did not show the range [fontes/resolume.md § Fixtures and DMX].
- **Outputs (`Shift+4`)**: universes with state, VU, rate, delay, priority, arm per universe.
- **Timeline (`Shift+2`)**: cues as markers with an action on the ruler (the `TimeCue`) and, in the track header, the cue list with current and next highlighted (Conductor). A single panel for the clip grid and the layer strip, like Resolume's `LayersAndClips` [fontes/resolume.md § Screen anatomy].
- **Inspector (`Shift+7`)**: parameters of the selected fixture grouped by type (Intensity/, Position/, Color/, Beam/); in cue edit mode, the red/orange overlay on top; a "record from the current situation" button.
- **Central viewer**: universe VUs or the scale model (`cenario-interativo.md`).
- **Status bar**: armed/rehearsal, current cue → next, master, blackout.

Menus: `View, Select, Add, Fixture` in the Patch; `View, Select, Add, Cue` in the Timeline. `Add` with search on typing.

Performance Face: a big GO (Blender's Pause at double the width) [fontes/blender.md § Anatomy of an editor], cue list, master, blackout, armed. Or MadMapper's cell grid in Live mode, "press the cell, no editing, made for a touch screen" [fontes/madmapper.md § Scenes and cues]. Nothing editable.

## 4. Shortcuts

`SHORTCUTS.md` already has `Enter` GO, `Backspace` go back, `Ctrl+G` go to cue, `Ctrl+Shift+Enter` arm, `Ctrl+Shift+R` rehearsal, `Esc` held blackout, `Ctrl+K` keyframe, `R` record arm, `Shift+D`/`Shift+S` mute/solo. What comes in:

| Action | Key | Origin | Conflict |
|---|---|---|---|
| Cue at the playhead position | `Ctrl+B` | Chataigne `TimelineAppCommands.cpp:47` [fontes/chataigne.md § Shortcuts] | none |
| Previous / next cue (selection, without firing) | `Shift+PageUp` / `Shift+PageDown` | Chataigne | none; `Ctrl+Shift+←/→` remains marker |
| Time step | `PageUp` / `PageDown` | Chataigne | none |
| Cue edit mode | `Shift+E` | (ours; MadMapper uses `Cmd+Shift+C`, which collides with copy CLI command) | none |
| Change a value without recording it into the cue, in edit mode | hold `Shift` while moving it | MadMapper [fontes/madmapper.md § Scenes and cues] | gesture |
| Remove a parameter from the selected cue | `Backspace` over the field, in edit mode | MadMapper | `Backspace` outside a field = previous cue; the UI keymap only holds over a field |
| Keyframe on the field under the mouse / delete / clear animation | `I` / `Alt+I` / `Shift+Alt+I` | Blender keymap "User Interface" [fontes/blender.md § Shortcuts] | `I` = In outside a field |
| Return the field to the scene value | `Backspace` over the field, outside edit mode | Blender | same |
| Copy the field's CLI command | `Shift+Ctrl+C` | Blender | none |
| Number in batch | menu command `Sequential`, no key | Capture | |
| Play/pause in the performance Face | `Shift+Space` | TD | rule 8 |
| Momentary cue | hold the cue's key | Resolume `connect`, Piano Mode | gesture |
| Duplicate a cue to another position | `Alt` + drag | MadMapper | gesture |

Resolume's grammar: with no modifier it acts on the selected one (`B` bypasses the selected layer), `Shift` goes up one level (`Shift+B` bypasses the composition) [fontes/resolume.md § Shortcuts and mapping]. Here: `Shift+D` mutes the focused track; `Ctrl+Shift+D` mutes the track's universe (proposal, ours). Controller mapping by position in the grid **or** by cue identity (`by_cell` vs `by_name`); the second one "survives reorganizing the show at 11 pm" [fontes/madmapper.md § What to copy]; the mapping also declares the target's scope (`Selected | This | By position`) [fontes/resolume.md § What to copy].

## 5. File

In the `.spell`:

- `profiles[]`: a profile with `{id, version, name, modes: {name: channels[]}}`, channel = `{name, type, bits, default, ranges[{from, to, label}]}`. The user's library in a text folder, referenced by `id + version`. Resolume's anonymous copy (`fixtureName=""`, a new uuid, "fixing the personality does not fix the show") is what not to do [fontes/resolume.md § What NOT to copy].
- `fixtures[]`: `{uid, name, profile, mode, universe, address, id, unit, circuit, group, overrides: {invert_pan, limit_pan, intensity_scale, …}}`.
- `universes[]`: `{uid, name, base, style, outputs: [{protocol, rate, delay_ms, priority, artsync}], merge: {dimmer: htp, default: ltp}}`.
- `cues[]`: the schema of §1, with its own `uid` and position as an attribute (`list`, `index`; or `bank`, `col`, `row` for the grid Face). Not the flattened `cues[128]` grid with an implicit index [fontes/madmapper.md § What NOT to copy]. No thumbnail (one cue in MadMapper's example carries 17 658 bytes of PNG). `exclusive`, `momentary`, `follow`, `conditions`, `action`.
- `cuelists[]`: `{uid, name, cues[uid], loop, current}`.
- `mappings[]` with `target: {by: uid | cell, scope: selected | this | position}`.

Outside the `.spell`: live levels, connection state, the operator's programmer (Parrot records into its own file when asked).

Saving: automatic backup copy and an offer of it on a corrupted open (`FileErrorUseBackup`, `ChecksumError`) [fontes/capture.md § States and messages]. Importing fixtures from CSV with column mapping and a report of `updated / added / skipped rows` [fontes/capture.md § File] becomes `spell patch import`.
