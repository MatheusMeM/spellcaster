# spellgui/web — the Spellcaster pages

Plain HTML and JS, no build, no framework, no npm. Each page is a flat folder of files that
`spellcore serve` (or any static server) hands out. Color and typography come only from
`design/tokens/spellcaster.css`; no page writes a literal color.

| File | What it is |
|---|---|
| `nav.js` | the top bar of the five pages (one `<script src="nav.js">` line and one `<div id="nav">`): tabs TIMELINE/PATCHBAY/THEATER/FACE/LASER with `Shift+1`..`Shift+5`, ENGINE/OFFLINE indicator with `rev`, and the editable show name (writes on Enter or on blur, never on a key) |
| `bus.js` | bus client: WS `{"id","cmd","args"}` with a promise per id, reconnection, events (`transport`, `show`, `log`, `widget`), binary frames `topic\|universe\|512` decoded into `{topic, universe, data}` (topic 1 = output, topic 2 = input). Offline mode built in |
| `widgets.js` | typed parameter → widget (`WG.kindOf`), fields → request `args` (`WG.args`), form of a registry command (`WG.form`). Classes for the page to style: `.wg`, `.wg-<type>`, `.wg-lab`, `.wg-num`, `.wg-form`, `.wg-doc`, `.wg-go`, `.wg-out` |
| `face.js` + `face.html` | Face runtime: `faces/<name>.face.json` becomes a widget grid in kiosk mode |
| `canvaskit.js` | pan, zoom, hit-test, marquee, DPR, dirty flag; what mounts a canvas with it is `timeline.js` and `graph.js` (`teatro.js` only uses `CK.colors`). The mouse wheel behaves the same in both: **wheel** scrolls the content, **Shift+wheel** moves along the time axis, **Ctrl+wheel** zooms at the cursor, the middle button drags. Every event carries `preventDefault` (`passive:false`): the page never scrolls and Ctrl+wheel does not zoom the browser. The bottom limit of `view.y` is 0; the top one is `k.ymax`, written by whoever knows the content height |
| `timeline.js` + `index.html` | canvas timeline. Layout of three fixed bands in the window height (toolbar, canvas, status bar) with `overflow:hidden`: the page does not scroll. Loop is engine state (`loop_set`), not client state; undo/redo is a local stack of 20 copies of the show (`show_set`). `R` (header button, key and bar button) calls `rec_arm` in the engine and the arm state comes from `rec_state`; `+Track` opens a type menu (`dmx`, `laser` with the `laser_files` list, `fx`). Shortcuts: `design/SHORTCUTS.md` |
| `help.js` + `help.html` | help: the "Default map" table of `design/SHORTCUTS.md` (table parser in `HELP.table`, no markdown library) and the live registry from `bus.commands()`, one `WG.form` per command to run it. `HELP.bindKey()` binds the `?` key on any page that loads `help.js` |
| `viewer.js` | 2D previz in the bottom strip of the timeline (`Alt+M`): DMX bars of the universe of the focused lane — the INPUT one when it is armed —, the ILDA frame of each laser track at `t` (`clip_frame` of the engine, with the track `scale`/`rot`) and the patch plan in a grid by address, with the fixture color coming from the pure part of `teatro.js` (the same maths in both plans). A column with no data writes “no laser” / “no patch” |
| `midi.js` + `midi.html` | MIDI mapping: ports, open/close, `key -> command` table of the `.spell`, LEARN and the last key live. Only a client of the `midi_*` commands |
| `laser3d/` | **the main page of the program**: the 10 W laser projector in 3D (three.js r128 + PBR, GLSL beam with bloom, optical table, camera in the SolidWorks convention, splash on the wall, key+MIDI bindings with learn, Pino as the menu of the screens). `engine.js` is the state → command map (`cmdFor`); `bind.js` holds the bindings and exposes `Bind.manifest()`; `tokens.css` is the design system of this tool only; `video.js` is the option table of the VIDEO tab (presets, ranges, default and persistence in `localStorage`), the single source the tab draws itself from and where `app.js` reads what to apply in three.js. The model, the beam, the camera and the Pino come from the design prototype that was voted, copied untouched |
| `vendor/` | three.js r128 (`build` + the post-processing passes) and the two `.woff2` fonts (Michroma, Share Tech Mono). Vendored on purpose: the program runs at an event, **with no network** — no page asks a CDN |
| `dev/commands.json` | `Registry::schema()` frozen, used in offline mode |
| `test/*.test.js` | `node --test spellgui/web/test/*.test.js`. Every page declares `<meta charset="utf-8">` in the first 1024 bytes because not every server sends a charset in the `Content-Type` (`python -m http.server`, the Tauri asset protocol, `file://`) and without it an accent turns into mojibake; `charset.test.js` watches that and the BOM. |

## How to open

The pages read `../../design/tokens/spellcaster.css`, `../../faces/` and `../../shows/` (it is what
`index.html` already did): **serve the repo root**, not `spellgui/web`.

With an engine (the normal case — it is the only process that touches hardware):

```
spellcore serve --port 8000 --dir . --show shows/medgrupo.spell
# http://127.0.0.1:8000/spellgui/web/laser3d/app.html      <- the main page
# http://127.0.0.1:8000/spellgui/web/face.html?face=quatro
```

`laser3d/app.html` opens through the splash; the hashes `#rear`, `#inside` and `#laser` jump
straight to the menu, the preferences and the show view. `Tab` opens the tab drawer; the Pino stays
attached to the camera. Operator manual: `MANUAL.md` at the repo root.

With no engine (only to draw the page; no DMX output):

```
python -m http.server 8000
# http://127.0.0.1:8000/spellgui/web/face.html?face=quatro&offline=1
```

In offline mode there is no engine: `bus.call` echoes `{offline, cmd, args}` and writes in the log
what it would do. The page mounts, the buttons respond, nothing goes out on the network.

## Face

`faces/<name>.face.json` (PRD §10). The widget declares **where** it sits and **what it fires**; the
behaviour lives in the Graph, inside the `.spell`:

```json
{"id": "go", "type": "button", "label": "GO", "at": [0, 0, 2, 2], "cmd": "cue_go", "args": {}}
{"id": "blackout", "type": "button", "label": "BLACK", "at": [3, 1, 1, 1], "input": "widget:blackout"}
```

- `cmd` + `args` → `Registry::call` over the bus.
- `input` → `input {key, value}` command; an `in.widget` node with `"widget": "blackout"` in the
  show graph is what decides what that does. Without that node the `input` has no effect — on
  purpose: the page implements no behaviour.
- The way back is the `widget` event (`out.widget` of the Graph): `{"id","prop","value"}`. `prop`
  becomes a class of the same name on the widget, on when the value is not zero; `face.html` paints
  `glow`/`on`, `alert`/`live` and `off`, and the rest is left for the page to style.
- `views` rearrange the same widgets (`grid` `[columns, rows]`, list of `widgets`, `at` per id);
  `?view=compact`. `views` is required.
- Types today: `button`, `toggle`, `fader`, `label`. The rest of the PRD §10 catalogue comes in with
  the Face editor (R9).

Face shortcuts (design/SHORTCUTS.md): `Enter` = cue GO, `Esc` held 0.5 s = blackout, `Shift+F` =
full screen.

## Regenerating `dev/commands.json`

It comes out of the registry, never written by hand:

```
spellcore commands > spellgui/web/dev/commands.json
```

`spellcore/cli/tests/commands_json.rs` fails when a command leaves the registry or changes schema.

## How to open without a browser

`spellcaster.exe` (crate `spellcore/gui`) brings the bus up in process, on a free port, and opens
these same pages in a window of the program. It is the normal mode on Windows:

```
spellcaster shows/medgrupo.spell
```
