# Open issues of the current interface — audit with evidence

Audit of the GUI at base commit `dac3e0a`, done with the engine live: `spellcore serve --port 8809 --dir . --show shows/medgrupo.spell`, five pages captured in headless Chrome at 1280x800 (and `index.html` and `teatro.html` also at 900x600, because half of the layout defects only show up when the window shrinks). The PNGs stay in the session scratchpad, not in the repo:

| Page | PNG |
|---|---|
| `spellgui/web/index.html` (timeline) | `png/index.png`, `png/index-900.png` |
| `spellgui/web/patchbay.html` | `png/patchbay.png` |
| `spellgui/web/teatro.html` | `png/teatro.png`, `png/teatro-900.png` |
| `spellgui/web/face.html?face=quatro` | `png/face.png` |
| `spellgui/web/laser.html` | `png/laser.png` |

Each line below has: **what the operator sees**, **cause in the code** (`file:line`) and **the workstream that fixes it**. `P1` = the operator trips over it in the first minute. Defects already raised by `design/FUNCOES/timeline-daw.md` (missing follow, loop that disappears with the player, `Shift` releasing the snap, `Shift+M` duplicating a marker, mute/solo with no runtime, marker with two formats) are **not** repeated here; the list below is what was left.

## 1. What the owner pointed out

| # | The operator sees | Cause | Workstream | P |
|---|---|---|---|---|
| 1 | "I don't want to open it in the browser, I want a GUI of the program" | there is no window crate: `spellgui/` has only `web/`, and `spellcore/Cargo.toml:3` lists nine members, none of them GUI. The only way to open it is `serve` + browser | `gui-janela` | **P1** |
| 2 | "I have no way to navigate between the interfaces" | **no `<a href>` between the five pages.** Grep for `href=` in `spellgui/web/*.html` only finds the `<link>` of tokens.css. Whoever opens `face.html` does not come back | `gui-janela` | **P1** |
| 3 | "I have no way to change the show name from the interface" | the name only appears as text in the footer/toolbar (`timeline.js:201-202`). The engine already accepts the edit (`show_patch {ops:[{op:"add",path:"/name",...}]}`, `edit.rs:824`); it is the page that has no field | `gui-janela` | **P1** |
| 4 | scrolling the page makes the menus disappear; the track area goes down | the bar is `display:flex` **with no `flex-wrap` and no `overflow`** (`index.html:11-13`), and `#msg` is free text that grows (`index.html:60`, filled in `timeline.js:201`). At 1280 px the show name text wraps into five lines and the bar becomes 90 px tall, pushing the canvas; at 900 px the `+Track`, `-Track`, `Monitor` and `Salvar` buttons **go off screen and there is no way to reach them** (`png/index-900.png`) | `timeline-ux` + `gui-janela` | **P1** |
| 5 | the mouse wheel does not move through the tracks | `canvaskit.js:169-173`: bare wheel zooms, `Shift`+wheel scrolls vertically, nothing moves sideways. The request is wheel = tracks, `Shift` = sideways, `Ctrl` = zoom | `timeline-ux` | **P1** |
| 6 | "no way to input or record new DMX or ILDA or video" | there is no input at all: no `drop`/`dragover`/`dataTransfer` in `spellgui/web/*` (empty grep), no upload route in `serve/src/lib.rs:338-345` (`/commands`, `/show`, `/ws`, `/mcp`, static fallback), and no recording command in the registry (the full list of 45 commands has no `rec_*`) | `browser-dnd` + `gravar-dmx` | **P1** |
| 7 | "nor to see how those things move in time" | the only viewer that exists is `TL.mon` with 512 bars (`timeline.js:649-667`), overlaid in the footer, for the universe of the focused lane. No ILDA frame, no video, no waveform | `previz` + `audio-video` | **P1** |
| 8 | MIDI mapping is not possible | no page reads `navigator.requestMIDIAccess` (empty grep). The engine **already has the bottom half**: `input {key}` accepts `"midi:144/60"` (`registry.rs:110-116`) and the `in.midi` node already listens to that key (`script/src/graph.rs:284`). What is missing is the top half: who listens to the MIDI keyboard and who records the pair | `midi` + `mapping` | **P1** |
| 9 | the loop control got buggy | already in `timeline-daw.md` item 5 | `timeline-ux` | — |

## 2. What this audit found

| # | The operator sees | Cause | Workstream | P |
|---|---|---|---|---|
| 10 | **muting a laser track does not take: the M lights up and the file keeps `mute: false`** | `timeline.js:308-312`: `commit()` walks over ALL lanes and does `L.spec.mute = L.mute`. A laser track has three lanes over the SAME `spec` (main, `.rot`, `.scale` — `timeline.js:190-196`), and the parameter lanes carry the stale copy (`mkLane`, `:163`). The last lane written wins and undoes the mute. Confirmed by running it: `spec.mute` goes back to `false` after `TL.commit([])` with the main lane at `mute: true`. The `show_patch` reaches the engine with `true`, the local JSON stays `false`, and the next `TL.reload()` wipes the mute. **Applies the same way to `solo`** (`:311`) and to the same case in `fixture` (`:193`) | `timeline-ux` (the fix is `L.mute` becoming a read of `L.spec.mute`, one single source) | **P1** |
| 11 | `Backspace` deletes the selected keyframe when it should go back one cue | `timeline.js:902`: `key === "Delete" \|\| key === "Backspace"` → `delSelected()`. `SHORTCUTS.md` fixes `Backspace` = "cue back", and `FUNCOES/README.md` rule 11 says "Delete is `Delete`". That is two owners of the same key in the same panel | `timeline-ux` | **P1** |
| 12 | the show the page shows is not the one the engine plays | `index.html:63,102` loads `../../shows/medgrupo.spell` by `fetch` and, in the WS `onopen`, `TL.reload()` fetches `/show` (`timeline.js:96,413`): two loads racing. And the **Abrir** button (`index.html:63`) swaps the show **only in the page** — there is no `load` call in `spellgui/web/*` (empty grep), although the registry has `load {path}` (`registry.rs:201-212`). Worse in `patchbay.html:51`, which starts pointing at `shows/patchbay_demo.spell` while the engine is on `medgrupo.spell`: `png/patchbay.png` shows the page editing one show while `/show` serves another | `comandos` + `gui-janela` | **P1** |
| 13 | tracks that will never go out on the network look like normal tracks | the engine prints `aviso: tracks ignorados: laser, pyfx` **on the stderr of serve**, and only there (`Timeline::ignored`, `timeline.rs:404-410`; types resolved at `:376`). On screen, `medgrupo.py` (pyfx) and `medgrupo_laser.ild` (laser) draw the same as the rest (`png/index.png`, three of the five lanes). `load` even returns `"ignored"` (`registry.rs:207-208`) and nobody reads it | `comandos` | P2 |
| 14 | a 46.8 s laser clip shows up as an empty lane | `mkLane` only knows `spec.keys` and `spec.<param>` (`timeline.js:157-181`). The track `{"type":"laser","clip":"medgrupo_laser.ild","fps":30,...}` has a file and a duration and the timeline draws neither a rectangle nor a file name with extension nor a shape. It is the central hole that `daw-arranjo.md` closes | `daw-arranjo` | **P1** |
| 15 | three different WebSocket clients in the same product | `bus.js` (used by `patchbay.html:67`, `laser.html:85`, `face.html:49`), the own BUS of `timeline.js:64-106` and the own BUS of `teatro.js:71`. The last two have a `ponytail:` saying "swap for bus.js when it exists" — it exists. A mapping overlay on ALL pages (the `Ctrl+Shift+A` request) has nowhere to plug in while there are three | `mapping` (prerequisite) | **P1** |
| 16 | the page does not know where the show lives | `GET /show` calls `show_get {full:true}`, which returns the `.spell` **without the path** (`registry.rs:221-223`: only the non-`full` branch goes through `resumo(f, sh)`). Without it there is no "show folder" to drag media into, and no relative path to write in `clips[]` | `browser-dnd` | **P1** |
| 17 | scrolling the tracks goes past the end and the screen goes black | `canvaskit.js:148` and `:171` clamp `view.y` from below (`Math.max(0, …)`) and not from above. With 5 lanes of 32 px fitting in 800 px of canvas, you can scroll into the void and there is no indication of how to get back. `png/index.png` already shows 600 px of nothing below the last lane, with no scrolling involved | `timeline-ux` | P2 |
| 18 | the track name is illegible and not editable | `timeline.js:503` truncates at 22 characters (`medgrupo_laser.ild.sc…` in `png/index.png`) and there is no edit field anywhere: `track_add` accepts `label` (`edit.rs:493`) and after that the name only changes by `show_patch` by hand | `daw-arranjo` | P2 |
| 19 | `Rec arm` lights up and records nothing | `index.html:70-75` and `timeline.js:910` only become a drawing boolean; the `ponytail:` in `index.html:69` admits it itself. No recording command exists in the registry | `gravar-dmx` | P2 |
| 20 | the Patchbay has 900 px of canvas and draws the grid in 660 | `png/patchbay.png`: the grid and the rectangle of the `raiz` group stop at y≈670 and a black strip is left over. The canvas is sized by the kit's `ResizeObserver` (`canvaskit.js:189`) but the page CSS does not stretch the element down to the footer | `patchbay-2` | P2 |
| 21 | the Patchbay asks for the show path as text and the rest asks nothing | `patchbay.html:51` has a path field; `teatro.html` and `laser.html` do not even have that; `face.html` decides by the query string. Four policies for "which show is this" across four pages | `gui-janela` | P2 |
| 22 | the TEATRO shows the cues as a spreadsheet, not as a grid | `teatro.js:196-246` builds a `<table>` line by line, with `cue_go` on double-click (`:241`). It is the GO list; the scenes x tracks grid of the Session View does not exist anywhere | `daw-sessao` | P2 |
| 23 | the `quatro` Face takes the whole screen and has no way out | `png/face.png`: four buttons, a status footer, no path back to the editor. `SHORTCUTS.md` promises `Tab` (editor ↔ performance) and `Esc` (close without leaving); `face.js` implements neither (grep for `"Tab"` and `"Escape"` in `face.js` empty) | `gui-janela` | P2 |
| 24 | the laser panel has eight sliders and no DAC | `png/laser.png`: `geo/*`, `limit/*`, `safe/*` drawn and active with "sem feed aberto" on the right-hand side. A parameter with no target is a widget that lies; `ilda-player.md §2` asks for the "no device / searching / connected / error" state before anything else | `ui-3d` | P2 |
| 25 | there is no Inspector on any page of the editor | `SHORTCUTS.md` reserves `Shift+7` for the Inspector and `SHORTCUTS.md § Interface` copies the "Inspector on the right" from Resolve. `index.html` has no right-hand panel; `patchbay.html` has one ("NADA SELECIONADO", `png/patchbay.png`) and it is read-only | `patchbay-2` + `daw-arranjo` | P2 |
| 26 | the ruler shows full-hour timecode in an 86 s show | `timeline.js:56-57`: `tc()` always prints `hh:mm:ss:ff`, so each label spends 11 characters to say `00:00:10:00`. The grid ladder is in seconds too (`STEPS`, `:28`), with no frame step | `daw-arranjo` | P3 |
| 27 | the In–Out range of the whole show looks like a scratch | `png/index.png`: with In=0 and Out=85.9 the 3 px gray bar (`timeline.js:622-623`) crosses the whole ruler and disappears against the grid. There is no brace, there is no shading outside the range | `daw-arranjo` | P3 |

## 3. What is not a defect and looks like one

- **`canvaskit.js:181` only redraws with `dirty`.** Correct: it is what holds 60 fps with a big show. Whoever forgets `k.dirty = true` in a new change is the one who breaks it.
- **`serve` serves the whole repo root on 127.0.0.1** (`serve/src/lib.rs:99-101`, with a declared `ponytail:`). It is not on the list because it is a known and written limit, and because `POST /files` (`browser-dnd.md §5`) will touch exactly that — the workstream that opens writing closes reading along with it.
- **`face.html` with no Inspector.** That is the design: `ilda-player.md §3` fixes "performance Face: no Inspector".

## 4. Summary by workstream

| Workstream | Items |
|---|---|
| `gui-janela` | 1, 2, 3, 4, 12, 21, 23 |
| `timeline-ux` | 4, 5, 10, 11, 17 |
| `daw-arranjo` | 14, 18, 25, 26, 27 |
| `daw-sessao` | 22 |
| `browser-dnd` | 6, 16 |
| `mapping` | 8, 15 |
| `audio-video` | 7 |
| `previz` | 7 |
| `gravar-dmx` | 6, 19 |
| `patchbay-2` | 20, 25 |
| `comandos` | 12, 13 |
| `ui-3d` | 24 |
