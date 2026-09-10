# design/FUNCOES — the practical part of each function

Matheus's rule (2026-09-09): you do not design UI for software with no function. Before any skin, every Spellcaster function needs five questions answered, and the answer comes from reading how Blender, TouchDesigner, Resolume Arena, MadMapper, Capture and Chataigne solved the same thing, in the files installed on this machine, not from impression.

One file per function, always with the same sections:

1. **Objects and verbs** — what the function calls a thing and what you do with each one.
2. **States** — stopped, armed, live, error, pending; how they show up.
3. **Screen zones** — what sits where by default and what is always visible.
4. **Shortcuts** — what is added beyond `design/SHORTCUTS.md`, with the source.
5. **File** — what the function saves into the `.spell`, and what stays out.

And at the top of each one, the verdict table: which app solved each item best.

| Function | File | Command | Theme |
|---|---|---|---|
| ILDA player | `ilda-player.md` | `spell ilda play` | LASER |
| NDI → ILDA | `ndi-ilda.md` | `spell ilda from-ndi` | FÓSFORO |
| Orchestrator | `orquestrador.md` | `spell graph` | PATCHBAY |
| DMX scenes and cues | `cenas-cues-dmx.md` | `spell cue`, `spell scene`, `spell patch` | PAPER THEATER |
| Interactive set | `cenario-interativo.md` | (Face over the patch) | PAPER THEATER |
| Aprendiz (main menu) | `aprendiz-menu.md` | command palette | all |
| Laser as a graph module | `integracao-laser.md` | `spell graph add laser/1` | LASER |
| Timeline as a DAW | `timeline-daw.md` | `spell timeline` (panel `Shift+2`) | all |
| Timeline as Arrangement View | `daw-arranjo.md` | `spell timeline` | (editor) |
| Session View (cues in a grid) | `daw-sessao.md` | `spell cue` | PAPER THEATER |
| Browser and drag and drop | `browser-dnd.md` | (panel `Alt+5`) | (editor) |
| Mapping mode | `mapping.md` | `Ctrl+Shift+A` | all |
| Audio and video | `audio-video.md` | `spell play` | (editor) |
| Failure points of the current interface | `pontos-falhos.md` | (audit) | — |

The last six came from the `daw-pesquisa` workstream: Matheus's request for *"an interface of use and functionality and usability and interface just like ABLETON"*, with drag and drop of media and outputs and mapping *"just like resolume Ctrl shift A"*. Their sources are the manuals of **Ableton Live 12** (online, cited by section), **DaVinci Resolve 20** (local PDF, cited by page) and **Resolume Arena** (online, cited by page); the citation sits on the line itself, as in `timeline-daw.md`, and there is no file in `fontes/` for them.

The per-app audits, with `path:line` on every claim, are in `fontes/` (`blender.md`, `touchdesigner.md`, `resolume.md`, `madmapper.md`, `capture.md`, `chataigne.md`). The function files cite the sources as `[fontes/app.md § Section]`; the evidence lives there, not here. Exception: `timeline-daw.md` cites Ableton Live and DaVinci Resolve on the table line itself (manual section, or page of the installed PDF), because each claim is worth one line and an audit file would be the same table again.

Beyond the installed-app audits, there are digests of **official documentation**, with `p.N` or a section on every claim. The PDFs and the extracted texts **do not enter the repository** — they stay in the session scratchpad, and the path is in the header of each digest. All downloaded on **2026-09-09**:

- `fontes/ableton12.md` — "Ableton Live 12 Manual", 1009 pages. https://cdn-resources.ableton.com/resources/manuals/live12-manual-en.pdf
- `fontes/resolve20-guia.md` — "The Beginner's Guide to DaVinci Resolve 20", Blackmagic Design Learning Series, 643 pages. https://documents.blackmagicdesign.com/DaVinciResolve/20/The%20Beginners%20Guide%20to%20DaVinci%20Resolve%2020.pdf
- `fontes/resolume-quickstart.md` — "Quickstart Tutorial" of Resolume 7, one page. https://resolume.com/support/en/quickstart
- `fontes/chataigne-doc.md` — "The Amazing Chataigne Documentation" (Notion, 30 pages captured by the Browser panel; the Detective and Parrot ones are empty at the source). https://benkuper.notion.site/The-Amazing-Chataigne-Documentation-079bd5a0b7e648bbbfe34c3c869a3985

## Cross-cutting rules

They showed up in more than one app and hold for all six functions. Each function file assumes these and only adds what is its own.

1. **A typed parameter generates the widget; nobody draws a widget per command.** Chataigne has eleven types and one `createDefaultUI` per type; TouchDesigner serializes a parameter with `name, label, page, style, size, default, min/max, normMin/normMax, enable, readOnly, help`; MadMapper declares `LABEL, TYPE, MIN, MAX, DEFAULT, FLAGS` and the `LABEL` with `/` builds the group tree; Resolume separates the value (`ParamRange`) from the presentation (`ParameterView`: `suffix, step, display_units, control_type`). Spellcaster's minimum schema, per registry parameter: `name` (the CLI one), `label`, `type` (trigger, bool, int, float, enum, string, color, xy, target, file), `min/max` (physical clamp), `norm` (slider range, separate from the clamp), `default`, `unit`, `readonly`, `enabled`, `group` (via `/` in the label), `flags` (button, momentary, spinbox), `help` (one sentence). [fontes/chataigne.md § Parameter types], [fontes/touchdesigner.md § File], [fontes/madmapper.md § Material/module parameters], [fontes/resolume.md § Parameter types]
2. **A textual address is the identity of everything.** In Resolume `/composition/layers/1/clips/3/connect` is at the same time a keyboard shortcut, an OSC address, a MIDI target and a REST route; in MadMapper the cue stores `/fixtures/9/color/red` and the shader reads `/custom/BPM/bpmPos`; in Chataigne every `Controllable` has an address and the `TargetParameter` points at it. It is `spellcaster.core.registry` validated three times over: CLI, OSC, cue, mapping, MCP and Aprendiz use the same name, and the GUI shows that name. [fontes/resolume.md § What to copy], [fontes/madmapper.md § Scenes and cues]
3. **Trigger, toggle and value are different types starting at the schema.** Resolume has `ParamEvent` (fires only), `ParamBoolean` and `ParamRange`; Chataigne has `Trigger` separate from `Bool`, and `CommandContext {ACTION, MAPPING, BOTH}` says whether a command accepts a trigger, a value or both; TouchDesigner has `Pulse`, `Momentary` and `Toggle` as distinct styles. Each registry `@command` declares which it is. [fontes/resolume.md § Parameter types], [fontes/chataigne.md § Objects and verbs]
4. **State is ink mixed into the field background, with a selected pair.** Blender has a theme block only for state (`error, warning, info, success, inner_anim, inner_key, inner_driven, inner_overridden, inner_changed`) with a `blend` factor and a `_sel` version of each; the widget does not change shape, only color. Three levels of "does not act now": `active` (dimmed, editable), `enabled` (locked), `alert` (red). TouchDesigner adds the state missing from `PRINCIPIOS.md §2`: **pending** (change not applied), red. [fontes/blender.md § States], [fontes/touchdesigner.md § States]
5. **Error and warning are data, not just paint.** Every TouchDesigner node exposes `warnings` and `errors` as counts; Chataigne has a central `WarningReporter` where any object registers `setWarningMessage(msg, id)` and the panel line takes you to the culprit. In Spellcaster the engine is the single source: panel, log, `spell` over SSH and Aprendiz all read the same. [fontes/touchdesigner.md § States], [fontes/chataigne.md § Visual states]
6. **Fixed menu order: View, Select, Add, <panel object>.** The View menu starts with the region toggles and ends with the area block (split, maximize, close). Verified across eleven Blender editors. [fontes/blender.md § Anatomy of an editor]
7. **One global UI keymap holds over any field of any panel.** In Blender `I` inserts a keyframe on the field under the mouse, `Backspace` goes back to the default, `Shift+Ctrl+C` copies the data path. In Spellcaster: `I` records the field into the focused scene or timeline, `Backspace` goes back to the scene value, `Shift+Ctrl+C` copies the CLI command of that field. [fontes/blender.md § Shortcuts]
8. **Two shortcut tables, editor and performance.** TouchDesigner disables `Space` inside a panel and in Perform Mode; only `Shift+Space` pauses, on purpose. `SHORTCUTS.md` gains a "Performance Face" column where transport requires `Shift`. Disabling a shortcut = keep the line and erase the command, so the map stays auditable. [fontes/touchdesigner.md § Shortcuts]
9. **One verb per concept.** Blender uses `mute/hide/disable/protect/lock/restrict` for the same thing across four editors. Here: `mute`, `lock`, `solo`, and nothing else. Solo dims the toggle of the others but does not remove it. [fontes/blender.md § What NOT to copy]
10. **The command palette indexes the menu label, and the label is the registry name.** Blender's F3 indexes menus, not operators; `SEARCH_ON_KEY_PRESS` turns any long menu into a search on the first character. A menu with more than 8 items becomes a search. [fontes/blender.md § Command palette]
11. **No numpad, no `X` to delete, no configurable `Space`.** Delete is `Delete`. `Space` is play, full stop. [fontes/blender.md § What NOT to copy]
12. **`.spell` file: complete JSON, versioned, with UID.** Do not write only the delta against the default (Resolume and Chataigne do that and the file becomes unauditable); `"version"` at the top and local migration, not a remote converter (Chataigne) nor silent key renaming (Resolume `Beats_d → Beats_double`); reference between objects by UID, never by short name (Chataigne `sourceState`, Resolume `OscShortcutPreset`); no thumbnail (Resolume, MadMapper), no window state (MadMapper `previewsData`, Chataigne `layout`) inside the show. A reference to an external file carries a backup copy and a declared `relpath` (TouchDesigner `savebackup`). [fontes/resolume.md § File], [fontes/madmapper.md § File], [fontes/chataigne.md § File], [fontes/touchdesigner.md § File]
13. **A saved layout is a named preset with declared content, not 33 checkboxes.** Resolume turns each panel on with a `Show X` boolean; Chataigne serializes a dock tree with direction and preferred size. Copy the tree format for the Faces; do not copy free dragging nor the checkboxes. [fontes/resolume.md § Screen anatomy], [fontes/chataigne.md § Anatomy of the screen]
14. **Module, profile and help are readable text.** MadMapper encrypts the factory modules (`main.ldat`); Chataigne declares a module in `module.json` with `parameters, values, commands, dependency`; Resolume keeps the contextual help as `element → one sentence` in XML. Everything Spellcaster loads is versionable text. [fontes/madmapper.md § What NOT to copy], [fontes/chataigne.md § File], [fontes/resolume.md § What to copy]

## Open issues (not decided here)

- `SHORTCUTS.md` has `Shift+1..7` for seven panels; `cenario-interativo.md § Shortcuts` proposes `Shift+8` for the viewer/scale model.
- The node catalog of PRD §10 has no **state** node (State Machine); `orquestrador.md § Objects and verbs` shows what Chataigne does and what is missing. The decision goes to `DECISOES.md`.
- `Tab` is the Face switch in `SHORTCUTS.md`, so entering/leaving a group in the graph goes to `Ctrl+]`/`Ctrl+[`; `Ctrl+X` is cut, so delete-and-reconnect goes to `Shift+Delete` (`orquestrador.md § Shortcuts`).
- `Ctrl+B` (cue at the playhead position, Chataigne) and `Shift+PageUp/PageDown` (previous/next cue) are not in `SHORTCUTS.md` and conflict with nothing; proposed in `cenas-cues-dmx.md`, together with `Shift+E` (cue edit mode).
- `F3` (palette), `Q` (favorites), `F9` (adjust last operation), `Shift+R` (repeat), `F1` (element help) come from Blender/TD, conflict with nothing, and are the Aprendiz keyboard (`aprendiz-menu.md`).
- Bare keys per focused panel: `V` (viewer mode) and `F` (freeze) in `ndi-ilda.md`; `F` (Focus mode), `H`/`Shift+H`/`Alt+H`, `Alt+1..9` (cameras) in `cenario-interativo.md`; `Shift+T` (test pattern) in `ilda-player.md`. `Ctrl+I` collides (import markers × invert selection) and needs a decision.
- Round 4 in `design/0.1.2` renamed Aprendiz to Pino; these files still say Aprendiz.
