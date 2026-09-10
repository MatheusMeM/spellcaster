# Aprendiz — the main menu (all themes)

Aprendiz is the command palette, the contextual help, the warning panel and the status bar, with one voice. It is not a character that interrupts: it is the text surface that `PRINCIPIOS.md §4` demands of every screen, and the place where "the galvo cannot keep up" (`TEMAS.md`) is said.

| Item | Who solved it best | Why |
|---|---|---|
| Objects and verbs | Blender, with Resolume's help table | Search that indexes menu labels (not operators), favorites, "adjust last operation", "repeat last"; contextual help as data, one sentence per element |
| States | Chataigne, with TouchDesigner and Capture | Warnings registered by id with a clickable culprit; error and warning as a count per node; a message that names the probable cause and puts the log one click away |
| Screen zones | Blender | Status bar with four things in fixed order; the message never disappears, it only moves; Playback and Keying as popovers |
| Shortcuts | Blender | `F3`, `Q`, `F9`, `Shift+R`, `F2`, `Shift+Ctrl+C`; any menu becomes a search when you type; help with `Alt`+hover comes from TouchDesigner |
| File | Resolume, with TouchDesigner's `help` and Capture's `.c2l` | Help corpus as a data file, keyed by element; a tip is plain text; translation is `Section/Key`; everything generated from the registry, not parallel to it |

## 1. Objects and verbs

**Command.** The registry `@command`, with `label`, `help` (one sentence), `menu` (path `Panel/Menu/Item`), `shortcut`. A command is only findable if it is in a menu: in Blender, "the `text=` is the search key" [fontes/blender.md § Command palette]. Here the menu label **is** the registry name (rule 10), so the palette indexes both at once.

**Menu.** `View, Select, Add, <panel object>` in every panel (rule 6); app bar with `File, Edit, Show, Window, Help` (Blender's topbar is `Blender, File, Edit, Render, Window, Help`) [fontes/blender.md § Anatomy of an editor]. A menu with more than 8 items becomes a search on the first character (`SEARCH_ON_KEY_PRESS`). Capture's 16 tool panes in a column with no search are the counter-example [fontes/capture.md § What NOT to copy].

**Palette.** `F3`. The result shows the label, the menu path where it lives, the shortcut and the help sentence. Searching by the internal operator name sits behind "developer mode" in Blender [fontes/blender.md § Command palette]; here there is no difference between the two names, so there is no mode.

**Favorites.** `Q`, the menu the user builds button by button [fontes/blender.md § Command palette]. It lives in the profile, not in the show.

**Last operation.** `F9` opens the HUD panel with the parameters of the last command, editable: "after a GO, the operator wants to fix the fade without repeating the cue" [fontes/blender.md § What to copy]. `Shift+R` repeats.

**Contextual help.** Table `element → one sentence`: the 412 entries in Resolume's `docs\help\English.xml` are "Aprendiz's body, ready-made" [fontes/resolume.md § What to copy]. Every parameter carries `help` in the schema (TouchDesigner `TDJSON.py`) [fontes/touchdesigner.md § File]; `Alt`+hover over the label shows it [fontes/touchdesigner.md § Parameters].

**Tip.** Plain text, one per line, the `TouchDesignerTips.txt` [fontes/touchdesigner.md § Sources read]. It shows up in the splash and in the statistics slot when there is no warning.

**Warning.** `{target (address), id, message, severity}`, registered by any object with `setWarningMessage(msg, id)` and cleared by id [fontes/chataigne.md § Visual states]. The panel line resolves to the culprit. Examples, with the source of each number:

| Aprendiz sentence | Origin of the criterion |
|---|---|
| "Real FPS 28 Hz: the galvo cannot keep up. PPS 30 000 / 1 070 points." | MadMapper, `ILDA FPS = PPS / Point Count`, blinks below 35 [fontes/madmapper.md § Laser] |
| "Dropping frames from the NDI source (missed_frames rising)." | TouchDesigner, NDI In info channels [fontes/touchdesigner.md § Laser and NDI] |
| "Universe 3 with no answer for 4 s. Possibly blocked by firewall. Open log." | Capture `PotentiallyBlocked` + `OpenLogFolder` [fontes/capture.md § States and messages] |
| "This patch consumes 39 channels and overflows universe 1 by 7. Continue on 2?" | Capture `ChannelsRequired` + `OverflowWithContinue` [fontes/capture.md § What to copy] |
| "Fixtures 4 and 5 overlap channels 40-48." | Resolume, DMX Output panel [fontes/resolume.md § What to copy] |
| "Art-Net output rate above 44 Hz." | TouchDesigner DMX Out [fontes/touchdesigner.md § Laser and NDI] |
| "1 830 vectorized paths, limit 2 000." | MadMapper `Monitor/Info` [fontes/madmapper.md § Laser] |
| "OSC preset 'OutputAllMessages' does not exist; using the default." | Resolume, broken reference by name [fontes/resolume.md § What NOT to copy] |

The warning arrives when the cost rises, not when the ceiling bursts (Capture's `TooManySmokeObjects` arrives late) [fontes/capture.md § What NOT to copy].

**Log.** `{time, source, severity, text}`, 2 000 entries in memory, optional file writing [fontes/chataigne.md § Visual states]. TouchDesigner's Error DAT records errors over time "to hunt intermittents" [fontes/touchdesigner.md § States].

**Watch.** "Watch this parameter and plot the history" (Detective) [fontes/chataigne.md § Visual states]. A verb of any field.

Verbs: search and run, explain, warn, go to the culprit, repeat, adjust, copy CLI command, favorite, open log folder, watch, dismiss.

## 2. States

The four inks of Blender's State block, and only those: `error`, `warning`, `info`, `success` [fontes/blender.md § States]. A warning is `warning`; whatever stops the show is `error`; "cue recorded" is `success` for two seconds; a tip is `info`.

- **Warning active / cleared**: by id; when the target is fixed, the warning disappears on its own [fontes/chataigne.md § Visual states].
- **Count per node**: `warnings` and `errors` as a number on each node, read by the GUI, by the CLI and by Aprendiz from the same source (rule 5).
- **Task running**: progress bar in the third slot of the status bar [fontes/blender.md § States].
- **Activity is not working**: green is traffic; Aprendiz only says "it works" when there is an answer [fontes/capture.md § States and messages].
- **The message never disappears**: if the status bar is hidden, the warning banner and the task banner migrate to the top [fontes/blender.md § States].
- **Fail early, with a name**: Capture separates seven startup failures (configuration, license, network, video, connectivity, resources, real time) [fontes/capture.md § States and messages]. `spell` reports the same at Pi boot.

Not included: a dialog apologizing for a bug (`DisableAdaptiveQualityWarning`) [fontes/capture.md § What NOT to copy]; a modal of any kind for a warning (`SHORTCUTS.md`: dialog only for open/save).

## 3. Screen zones

- **Status bar** (footer, always): four things, in this order, and nothing else [fontes/blender.md § States]: (1) what the mouse buttons do right now, in context; (2) the last message; (3) task running; (4) statistics or a tip. Item 1 is the software's learning line and teaches the modifier grammar of `cenario-interativo.md § 4` gesture by gesture.
- **Log panel (`Shift+6`)**: tabs `Log | Warnings | Help`, Chataigne's bottom-right corner [fontes/chataigne.md § Anatomy of the screen]. Warnings with a clickable culprit; Help shows the sentence for the element under the mouse or selected.
- **Palette**: floating in the center, only while `F3` is open; closes with `Esc` or on running.
- **Last-operation HUD**: footer of the focused panel, appears with `F9`, disappears when you click outside (Blender's `HUD` region) [fontes/blender.md § Anatomy of an editor].
- **Popovers in the header**: a toggle with an arrow instead of a window; Playback and Keying in Blender's Timeline are popovers [fontes/blender.md § What to copy].
- **Menu**: app bar at the top; panel menus in each panel's header, collapsible.

Aprendiz as a character (the Clippy of `TEMAS.md`) lives in slot 2 of the status bar and in the Help tab. It never covers the viewer, never blocks.

## 4. Shortcuts

| Action | Key | Origin | Conflict with `SHORTCUTS.md` |
|---|---|---|---|
| Command palette | `F3` | Blender `wm.search_menu` [fontes/blender.md § Shortcuts] | none |
| Favorites | `Q` | Blender `SCREEN_MT_user_menu` | none |
| Adjust last operation | `F9` | Blender `screen.redo_last` | none; TD uses `F9` to show the network under the cursor, irrelevant |
| Repeat last operation | `Shift+R` | Blender | none; `R` is record arm |
| Rename active item / in batch | `F2` / `Ctrl+F2` | Blender | none |
| Copy the CLI command of the field under the mouse | `Shift+Ctrl+C` | Blender `copy_data_path` | none |
| Help for the element under the mouse | `Alt`+hover; `F1` pins the help in the tab | TD [fontes/touchdesigner.md § Parameters]; `F1` ours | TD uses `F1` for Perform Mode; here the Face switches with `Tab` |
| Filter the list under the cursor | `Ctrl+F` | Blender | none |
| Any menu becomes a search | type with the menu open | Blender `SEARCH_ON_KEY_PRESS` | gesture |
| Go to the warning's culprit | click the line, or `Enter` with the line selected in the Warnings panel | Chataigne, TD Errors Dialog | `Enter` is GO outside the panel; the focused panel decides |
| Close palette / HUD | `Esc` tap | `SHORTCUTS.md` | held is blackout |
| Change the panel's editor type | not included | Blender `Shift+F1..F12` | Faces are named presets, not swappable panels |

Disabling a shortcut: keep the line in `config.json`, erase the command (rule 8). The key map needs a test that compares it with the behavior: `TouchShortcuts.txt` has `forward left` and `backward right` inverted relative to the wiki "since forever" [fontes/touchdesigner.md § What NOT to copy]; `spell keys check` lists the map and fails if a command does not exist in the registry.

## 5. File

Everything in Aprendiz is generated from the registry or is text next to it; nothing is parallel.

- **Help**: the `help` field of each `@command` and of each parameter is the source. `spell help --export` generates `help/pt-BR.json` in the format `{ "registry.name": "one sentence" }`, Resolume's `element → sentence` [fontes/resolume.md § Sources read], for review and translation. An entry with no matching command is a build error.
- **Translation**: one file per language, keyed by registry name, the `Section/Key` of the `.c2l` [fontes/capture.md § Sources read]. A shortcut is never a translatable string: in Capture "no Phrase contains Ctrl, Shift or F1" [fontes/capture.md § Shortcuts].
- **Tips**: `tips/pt-BR.txt`, one per line.
- **Favorites, key map, Face layout**: profile `config.json`, never in the `.spell`.
- **Warnings**: are not saved; they are reborn from state on open.
- **Log**: `logs/<date>.log` when enabled; "Open log folder" is a command.
- **Last operation**: session memory; `F9` edits what is still on the undo stack.

Source corpus for writing the sentences, already on disk: Resolume's 412 entries, TouchDesigner's 74 tips, Capture's named messages (`.c2l`, 1 885 lines) and the Laser/NDI/DMX pages of TouchDesigner's offline wiki, all cited in `fontes/`.
