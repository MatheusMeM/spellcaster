# Shortcuts and interface grammar

Reference for R5 (GUI) and R9 (editors). Source: Adobe Premiere Pro and DaVinci Resolve; since `design/FUNCOES/timeline-daw.md`, **Ableton Live** is a valid origin too, for what a DAW solves better than an NLE (automation, follow, loop on the selection). Rule: whoever edits video operates Spellcaster without learning anything new; what does not exist there (cues, outputs, laser) follows the same modifier logic. Shortcuts are remappable in `config.json` (`"keys": {...}`), but the map below is the default and what the documentation teaches.

## Grammar (inherited from both)

- Space plays and stops. J/K/L is the shuttle: J rewinds, L goes forward, repeating speeds up (×2, ×4, ×8), K stops; K+J / K+L steps frame by frame (Premiere).
- One I/O pair defines the work range; everything that is "loop", "render", "export" uses that range.
- Arrows move 1 frame; Shift multiplies by 5 (Premiere) — here 1 frame = 1/fps of the show.
- Ctrl = command, Shift = extends/widens, Alt = variant/clears. Never invent a fourth combination.
- Panels have focus; the shortcut acts on the focused panel (Premiere). A focused panel has a 1 px border in the accent (design/PRINCIPIOS.md §2).
- Zoom with `=` / `-`, fit everything with `Shift+Z` (Premiere) or `\` (Resolve: both are valid).
- Markers: `M` creates one at the playhead, `Shift+M` edits, `Ctrl+Shift+←/→` navigates (Resolve). Markers imported from a video edit are ordinary markers, gray.
- The wheel scrolls the tracks, `Ctrl`+wheel zooms, `Shift`+wheel scrolls sideways, `Alt`+wheel changes track height (Ableton; Resolve uses `Option` for zoom and `Command` for scroll, and that would clash with `Ctrl` = command). There is never page scroll.
- During a drag, `Alt` releases the snap and `Shift` fine-tunes the value (Ableton). The snap comes back on its own when the button is released (Resolve).

## Default map

The **state** column is what the code does today (base `dac3e0a`, pages under `spellgui/web`), not
what is intended: `done` = the key is wired on the page that owns the action; `missing` = the action
exists in the product and the key does not; `n/a` = the screen or the mode the key controls does not
exist yet. Whoever touches the key maintains the column — `spellgui/web/help.html` reads this table
and shows it to the operator.

| Action | Key | Origin | State |
|---|---|---|---|
| Play / pause | `Space` | both | done |
| Shuttle rewind / stop / forward | `J` / `K` / `L` | both | done |
| Previous / next frame | `←` / `→` | both | done |
| 5 frames | `Shift+←` / `Shift+→` | Premiere | done |
| Start / end | `Home` / `End` | both | done |
| Previous / next keyframe (focused track) | `↑` / `↓` | Premiere (edit points) | done |
| Mark In / Out | `I` / `O` | both | done |
| Clear In / Out / both | `Alt+I` / `Alt+O` / `Alt+X` | Premiere | done |
| Go to In / Out | `Shift+I` / `Shift+O` | Premiere | done |
| Loop on the In–Out range | `Ctrl+L` | Premiere | done |
| Loop on the time selection (In/Out become the selection) | `Ctrl+L` with an active selection | Ableton (Loop Selection) | missing |
| Follow: the view tracks the playhead | `Alt+Shift+F` | Ableton | missing |
| Marker at the playhead | `M` | both | done |
| Edit marker | `Shift+M` | both | missing |
| Edit the marker under the playhead | `M` again | Resolve (manual p.548, p.781) | missing |
| Previous / next marker | `Ctrl+Shift+←` / `Ctrl+Shift+→` | Resolve | done |
| Previz of the playhead: DMX of the focused track (input when armed), ILDA frame and patch plan | `Alt+M` | (ours; `M` is already marker, `Ctrl+M` is already export) | done |
| Zoom in / out / fit | `=` / `-` / `Shift+Z` or `\` | Premiere / Resolve | done |
| Back to the previous zoom | `Shift+Z` again | Resolve (manual p.647) | missing |
| Fit everything in the overview strip | double-click on the strip | Ableton §6.1 | missing |
| Scroll the tracks (vertical) | wheel | (ours; the page does not scroll — the tracks do) | done |
| Pan the timeline (horizontal) | `Shift`+wheel, or drag with the middle button | both | done |
| Zoom at the cursor | `Ctrl`+wheel | both | done |
| Track height | `Alt`+wheel, or `Alt++` / `Alt+-` | Resolve p.648 (there it is `Shift`) / Ableton | missing |
| Fold / unfold the lanes of the focused track | `U` | Ableton (Fold/Unfold) | missing |
| Lock / unlock the focused track | `Shift+L` | (ours; pairs with `Shift+D` and `Shift+S`) | missing |
| Keyframe at the playhead (focused track) | `Ctrl+K` | Premiere (add edit) | done |
| Keyframe on the curve, at the time of the click | double-click on the lane | Ableton (Automation) | missing |
| Add / remove keyframe per parameter | `Ctrl+Click` on the diamond | Resolve | missing |
| Nudge the selection 1 frame / 5 frames | `,` / `.` and `Shift+,` / `Shift+.` | Resolve (manual p.533, p.625) | missing |
| Duplicate the selection | `Alt` + drag | Resolve / MadMapper | missing |
| Release the snap in the middle of a drag | hold `Alt` | Ableton (Automation) | missing |
| Fine value adjustment while dragging | hold `Shift` | Ableton (Automation) | missing |
| Draw automation (draw mode) | hold `B` | Ableton | missing |
| Select all / none | `Ctrl+A` / `Ctrl+Shift+A` | both | done |
| Search and create a node at the cursor (PATCHBAY) | `Shift+A` | Blender (Add) | done |
| Copy / paste / cut / delete | `Ctrl+C` / `Ctrl+V` / `Ctrl+X` / `Delete` | both | done |
| Undo / redo | `Ctrl+Z` / `Ctrl+Shift+Z` | both | done |
| Easing of the selected keyframe: menu | `Ctrl+E` | (ours) | missing |
| Easing of the selected keyframe: cycle linear→in→out→inout→hold | `Ctrl+Shift+E` | (ours) | done |
| Mute / solo of the focused track | `Shift+D` / `Shift+S` | Premiere (disable) / (ours) | done |
| Snapping on/off | `S` | Premiere | done |
| Record arm of the focused track | `R` | Resolve (Fairlight) | done |
| New / open / save as | `Ctrl+N` / `Ctrl+O` / `Ctrl+Shift+S` | both | missing |
| Save | `Ctrl+S` | both | done |
| Import video/audio markers | `Ctrl+I` | Premiere (import) | missing |
| Export (show render) | `Ctrl+M` | Premiere | n/a |
| Help (this table + the registry commands) | `?` | (ours) | done |
| Page: TIMELINE, PATCHBAY, THEATER, FACE, LASER | `Shift+1` … `Shift+5` | Resolve (Shift+2..7 pages) | done |
| HELP page | `Shift+6` | Resolve (pages) | done |
| Panel focus inside the page | `Shift+7` … | Premiere (Shift+1..9) | n/a |
| Maximize the focused panel | `` Ctrl+` `` | Premiere | n/a |
| Workspace (Face `editor`) 1..9 | `Alt+Shift+1` … `Alt+Shift+9` | Premiere | n/a |
| Toggle Face `editor` ↔ `performance` | `Tab` (hold to peek, tap to switch) | (ours, PRD §10) | n/a |
| Fullscreen / kiosk | `Shift+F` | Resolve | missing |
| Cue GO | `Enter` | (ours; lighting consoles) | done |
| Cue back / jump to cue | `Backspace` / `Ctrl+G` | (ours; lighting consoles) | missing |
| Arm real outputs / rehearsal mode | `Ctrl+Shift+Enter` / `Ctrl+Shift+R` | (ours, PRD §10) | n/a |
| Blackout (hold) | `Esc` held 0.5 s | (ours) | missing |
| Close without quitting (back to the editor) | `Esc` | both | n/a |

Where reading the column is not obvious:

- **Page `Shift+1..6`** belongs to `spellgui/web/nav.js`, the bar at the top of the six pages; the
  `?` key works on all of them because that same bar loads `help.js`.
- **Export** is `n/a` because there is no show render in the registry; `Ctrl+M` stays reserved.
- **Blackout** is `missing`, not `n/a`, because the action exists: it is the Face's `blackout`
  widget, which sends `input {key:"widget:blackout"}` to the Graph. What is missing is the key.

## Interface (what to copy from each)

- **Premiere**: timeline with a ruler on top, stacked tracks with a header on the left (name, mute, solo, lock, record arm), red playhead crossing everything, markers on the ruler, In/Out as a gray bar, Inspector panel ("Effect Controls") with keyframes next to the value. Named workspaces as tabs at the top.
- **Resolve**: pages as fixed tabs in the footer (here: Patch, Timeline, Graph, Outputs, Network, Log), a large central viewer (here: previz or universe VU), Inspector on the right, "Media Pool" on the left (here: shows, profiles, laser clips, faces). Page buttons are uppercase text, no icon — matches design/PRINCIPIOS.md §4.
- **From both**: zero modals for operation; a dialog only for open/save. Everything that changes state has a shortcut and shows up in the menu with the shortcut next to it.
