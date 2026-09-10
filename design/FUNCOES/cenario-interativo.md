# Interactive set — the scale model (PAPER THEATER theme)

The central viewer as a scale model of the stage: the fixtures, the laser and the outputs drawn over a floor plan or a photo, clickable. It is the Face over the patch; it has no command of its own because each click fires a command that already exists.

| Item | Who solved it best | Why |
|---|---|---|
| Objects and verbs | Capture | Focus mode solves the central ambiguity: the same click cannot mean "I want this fixture" and "point over here"; a layer with `locked`, `not selectable` and `no simulation` kept separate; camera positions in a catalog |
| States | Resolume, with Blender and Capture | Five states per cell, one accent; feedback from the real state, not from the command sent; solo dims without removing; simulation switchable off without deleting |
| Screen zones | Capture, with MadMapper | A view named by its use, not Alpha/Beta/Gamma; control panel only in Live; input view and output view are two views of the same object |
| Shortcuts | Capture | The 3D view's modifier table is complete and consistent; Blender gives `H`/`Shift+H`/`Alt+H` |
| File | Capture | Set position kept separate from channel value; import with column mapping; Chataigne's dashboard shows what not to record (appearance in the item) |

## 1. Objects and verbs

**Scale model.** A view over the patch. Background: a reference image that does not go out on the output (MadMapper's output `background`, and the background image of Chataigne's Morpher) [fontes/madmapper.md § Objects and verbs], [fontes/chataigne.md § Objects and verbs]. Grid and image guides, Resolume's two `ScreenGuide` [fontes/resolume.md § Fixtures and DMX].

**Cutout.** A fixture, group, laser output or cue placed on the scale model. It knows how to become an item from any controllable (`createDashboardItem()`) [fontes/chataigne.md § Objects and verbs]: dragging from the patch table onto the scale model creates the cutout. Fields: target (registry address), position, size, layer, label. Appearance (color, outline, font) is **not** a field of the cutout [fontes/chataigne.md § What NOT to copy]; it belongs to the Theme.

**Layer.** Not just visibility: `locked`, `not selectable`, `include in reports`, `simulate` [fontes/capture.md § Objects and verbs]. "You can turn off the simulation of a whole layer without deleting it": it is the performance button at show time.

**View.** Named by its use: `stage`, `house`, `bars`; never Alpha, Beta, Gamma [fontes/capture.md § What NOT to copy]. Camera positions stored in a catalog and triggerable by DMX [fontes/capture.md § Objects and verbs].

**Set position.** Capture's "Scene": position and visibility of objects, with no channel value [fontes/capture.md § Objects and verbs]. It keeps "where things are in the second act"; the cue keeps "how much light". Different names because they are different things.

**Focus point.** Where the selected fixtures point.

Verbs, from Capture's scene context menu [fontes/capture.md § Objects and verbs] and from MadMapper's edit mode:

| Verb | Gesture | Origin |
|---|---|---|
| Select | click; `Shift` adds; `Ctrl` toggles; a box from left to right catches only what is entirely inside, from right to left also what it touches; double click descends one level into the group | Capture [fontes/capture.md § 3D scene and interaction] |
| Point (Focus mode) | with the mode on, clicking a point **does not select**: it points the already-selected fixtures there | Capture, Focus mode |
| Move / rotate | drag inside the outline; `Shift` locks orthogonal and 5°; `Ctrl` turns snapping off; the outer region of the triangle rotates each one on its own axis | Capture |
| Pan/tilt fan | `Ctrl`+drag scales, `Alt`+drag offsets | Capture |
| Assign | drag a gel or gobo onto the selection; here, drag a cue or scene onto the cutout | Capture `Common/Assign` |
| Cue | in cue edit mode, clicking an already-selected cutout includes/excludes its geometry in the cue (MadMapper cues "Input Geometry" in the input view, "Output Geometry" in the output view) | MadMapper [fontes/madmapper.md § Scenes and cues] |
| Isolate | solo | Blender Outliner, "Isolate first, alone" [fontes/blender.md § Objects and verbs] |
| Hide / hide the rest / reveal | `H` / `Shift+H` / `Alt+H` | Blender |
| Store camera | the current position becomes a catalog entry | Capture |
| Measure | a click starts, a click ends, `Shift`+click adds a point, `Esc` clears | Capture |
| Talkback | clicking the scale model returns pan/tilt to the source controlling the fixture (console over OSC, CITP) | Capture, DMX talkback [fontes/capture.md § What to copy] |
| Morph | X/Y cursor between scenes placed as points, weight by Voronoi | Chataigne Morpher [fontes/chataigne.md § What to copy] |

Talkback changes the nature of the node: the scale model is an **input** of the graph, "just like an OSC module" [fontes/capture.md § What to copy]. Selection synchronized both ways with the console (CITP/FSEL, EOS/OSC) is the same mechanism.

## 2. States

One accent per cutout, with Resolume's five-state enum ("clip triggers can have 5 different states") [fontes/resolume.md § What to copy]: `empty` (no patch), `patched`, `armed`, `live`, `error`. Plus `selected` as a pair of each one (rule 4).

**Feedback from the real state.** The cutout lights up with what the fixture is receiving, read from the engine, not with the command the scale model sent; it is Resolume's `OutputSiblingPath`, "the cutout on the scale model lights up with the real state, not with the command sent" [fontes/resolume.md § What to copy].

**Driven by**: `inner_driven` ink when the value comes from a cable, a cue or Parrot (`isControlledByParrot`) [fontes/chataigne.md § Visual states]; the operator sees that touching it there will not help.

**Solo** dims the toggles of the other cutouts without removing them [fontes/blender.md § Objects and verbs]. **Layer with no simulation**: cutouts dimmed, present, clickable (`active` false) [fontes/blender.md § States]. **Locked layer**: `enabled` false. **The cutout's universe with no answer**: `alert`.

**Active mode** (Focus, Measure, Edit cue): an indicator in the status bar and on the cursor; never on the cursor alone. Capture leaves Focus through the Selection Navigator button [fontes/capture.md § 3D scene and interaction]; here it is `Esc`.

**Mixed values** in a multi-selection, with a string of their own [fontes/capture.md § States and messages].

**Visible cost**: `Rate | Detail`, adaptive quality, resolution limit, performance information [fontes/capture.md § 3D scene and interaction]. A warning when the cost rises, not when the ceiling bursts (`TooManySmokeObjects` arrives late) [fontes/capture.md § What NOT to copy].

## 3. Screen zones

- **Center, viewer**: the scale model. It is the "large central viewer" of `SHORTCUTS.md § Interface`. Named view tabs at the top of the viewer (`stage | house | bars`), like Blender's workspace tabs [fontes/blender.md § File]. Two views of the same object when it makes sense: input (what comes in) and output (what goes out), side by side or just one, with MadMapper's toggle [fontes/madmapper.md § Anatomy of the screen].
- **Left**: layers, with the four toggles per row, and the camera position catalog.
- **Right, Inspector**: that of the selected fixture (`cenas-cues-dmx.md § 3`). Capture's Selection Navigator, which "appears next to the selection" [fontes/capture.md § Anatomy of the screen], does not come in as a floating panel (`PRINCIPIOS.md §3`); its verbs go to the context menu and to the palette, and what it shows goes to the Inspector, which does not move.
- **Control panel** by type of selected fixture, "only exists in Live mode" [fontes/capture.md § 3D scene and interaction]: here, only in the performance Face, with Light (turns the selection on/off), Home, and the type's parameters.
- **Status bar**: what the mouse buttons do right now, in the current mode [fontes/blender.md § States]. It is where the modifier table of §4 is taught, gesture by gesture.

Menu: `View, Select, Add, Cutout`. Select brings Capture's ten criteria (by layer, by location, by model, by type, by group, …) [fontes/capture.md § Objects and verbs]. No floating window (report, floor plan, console patch): "at 11 pm in a dark room it is a window lost behind another" [fontes/capture.md § What NOT to copy].

## 4. Shortcuts

Modifiers on the scale model, Capture's table, whole [fontes/capture.md § Shortcuts]:

| Action | Input |
|---|---|
| Orbit / pan | middle button, or `Alt`+drag; holding `Shift` swaps one for the other |
| Rotate without moving the camera | `Ctrl` |
| Zoom by moving the focal point / by changing the field of view | `Shift`+wheel / `Ctrl`+wheel |
| Add to selection / toggle item | `Shift`+click / `Ctrl`+click |
| Box: only fully inside / also what it touches | drag → / drag ← |
| Descend one level into the group | double click |
| Move orthogonally, rotate by 5°, fine slider adjustment | `Shift` |
| Turn snapping off | `Ctrl` |
| Fan scale / fan offset | `Ctrl`+drag / `Alt`+drag |

Keys that come in:

| Action | Key | Origin | Conflict |
|---|---|---|---|
| Focus the viewer (the scale model) | `Shift+8` | (ours) | `SHORTCUTS.md` stops at `Shift+7`; closes the README's open issue |
| Focus mode | `F` | (ours; Capture has no key) | none; in `ndi-ilda.md` `F` freezes the frame, but there the viewer is another one and the shortcut acts on the focused panel |
| Leave the mode | `Esc` tap | Capture, `SHORTCUTS.md` | held is blackout |
| Hide / hide the rest / reveal | `H` / `Shift+H` / `Alt+H` | Blender [fontes/blender.md § Shortcuts] | none |
| Solo | `Shift+S` | `SHORTCUTS.md` | |
| Camera position 1..9 | `Alt+1` … `Alt+9` | (ours; Blender uses the numpad, forbidden by rule 11) | none; `Alt+Shift+1..9` remains workspace |
| Store camera position | `Ctrl+Alt+1..9` | (ours) | none |
| Select all / none / invert | `Ctrl+A` / `Ctrl+Shift+A` / `Ctrl+I` | `SHORTCUTS.md` + Blender | `Ctrl+I` is import markers in `SHORTCUTS.md`; in the focused viewer, invert; issue to register |
| Frame all / selection | `Shift+Z` / `\` | `SHORTCUTS.md` | |
| Measure | `Shift+M` in the viewer | (ours) | `Shift+M` edits a marker on the timeline; the focused panel decides |

Capture rule that does not come in: shortcuts only with `Ctrl`+letter, "no bare key" [fontes/capture.md § What NOT to copy].

## 5. File

In the `.spell`, key `stage`:

- `views[]`: `{uid, name, background: {path, relpath, backup}, camera: {…}, guides}`.
- `items[]`: `{uid, target (address), view, layer, x, y, w, h, rotation, label}`. The cutout's geometry is show data because it is the physical floor plan, like `PositionX/Y/Z` of `Object` in Capture [fontes/capture.md § Patch and universes]. Color, outline, font, background opacity: never (Chataigne's `DashboardControllableItem` records `textColor`, `contourColor`, `opaqueBackground`, and mixes data with theme) [fontes/chataigne.md § Visual states].
- `layers[]`: `{uid, name, visible, locked, selectable, simulate}`.
- `positions[]` (set positions): `{uid, name, items: {uid: {x, y, visible}}}`, separate from `cues[]`.
- `cameras[]`: `{uid, name, view, …}`, triggerable by a cue like any address.

Outside the `.spell`: selection, current zoom, active mode, adaptive quality.

Import: a CSV of fixtures with column mapping for position, rotation, unit, pan/tilt focus, mode channels, identifying the fixture by a chosen property, with a report of what was updated, added and skipped [fontes/capture.md § File]. Export: floor plan and report (equipment, anchor point, cable, fixture, group, location) [fontes/capture.md § Objects and verbs]; floor-plan symbols from SVG.
