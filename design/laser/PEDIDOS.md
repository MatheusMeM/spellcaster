# Matheus's requests for the LASER tool (round 5 onward)

Verbatim record of what was asked for, in the order it arrived. Nothing here is optional; each item becomes an `[x]` once it is in the published prototype.

## 2026-09-09 · about round 4 ("I liked the program a lot as a whole, let's iterate on top of it")

- [x] **Realistic rear panel** like a real show laser (references: Kvant Clubmax rear panel; Chinese RGB laser diagram). "Take inspiration, don't copy everything." No element without a function: **one** interlock only, **no fuse**. The fan can stay, but "you modelled it all wrong" (real axial fan: frame ring, hub, 7 curved blades, wire guard).
- [x] Search the internet for laser images, 3D models and assets to build ours (external assets do not load in the artifact; the model is procedural, inspired by the photos).
- [x] **The inside "is terrible"**: use **GLSL** to model the laser beam; add the **diodes** and the **dichroic mirrors**; model the **optical path trace** efficiently and eye-candy.
- [x] **Menu mascot (Pino)** "isn't very good yet".
- [x] **Opening flow**: the settings screen first, only then can you navigate to the show screen.
- [x] **Triple AAA** result in the model, in the interface and in the usability.
- [x] Generate this as **a design system for this laser and ILDA tool only** (`tokens.css` + `SISTEMA.md` + `sistema.html` page).
- [x] **Key binding magic** like MadMapper / Resolume: anything becomes a MIDI event, **in and out** (MIDI learn, output feedback).
- [x] **Camera**: the current orbit is uncomfortable; use the **SolidWorks standard** for manipulation (MMB orbits around the clicked point, Ctrl+MMB pan, Shift+MMB zoom, wheel zooms at the cursor, arrows 15°, Shift+arrows 90°, Ctrl+arrows pan, F frames, Ctrl+1..7 standard views).

## 2026-09-09 · splash and camera

- [x] The splash is the **camera aimed straight at the output** (the wall). When the splash ends, the camera **focuses on the rear of the laser**; menus and interactivity come after that.
- [x] Splash detail: on the wall, and **only there** (no ILDA, no show laser on the output), the laser draws the **outline of the name SPELLCASTER LASER**. While the laser draws the outline in a **dynamic movement**, the letters are **revealed** and stay on the wall; **they all glow**; then the camera goes **straight to the menu** (rear panel) and the laser **goes dark**, dimming the surrounding room light a little.

## 2026-09-09 · the inside (with a screenshot of round 4)

- [x] **Nothing may be floating or out of context.** Look for images of show lasers opened up.
- [x] **Model the galvos** (X/Y block with the two motors at 90°, mirrors, cables).
- [x] **Model the optical table system** (base plate with a drilling pattern, modules, kinematic mounts).
- [x] Make the optical assembly **look like a single block**, different from the other components.
- [x] **Simulate the PCB better** than this, **but do not make it prominent** (driver board and PSU sit to the side, dark, discreet).

## Permanent rules (already in memory, repeated here for safety)

- Commits and pushes only on Matheus's account, no credit to Claude. Branch `design/0.1.3` (round 6; previously `design/0.1.2`).
- Agents always on Opus.
- Ponytail full; Python only in `.py`; UTF-8 without BOM; heavy binaries outside the Drive folder.
- Function before UI; one material per theme; vote inside the prototype; be wild; no boxy software.

## 2026-09-09 · camera focus in the splash

- [x] In the splash the camera focus **leaves the static point** it is aiming at and **goes to the screen (the wall) itself**, ignoring the laser and its rear panel. Only after that does it go and focus on something else: the menu (rear panel) or the preferences (lid).

## 2026-09-09 · Round 6 · integration with the orchestrator

Literal request (the "Context:" block that came with it belongs to an earlier task, the merge of `design/funcoes-referencia`, and was left out of this round):

> You are the Spellcaster design agent. Ponytail full, protocolo-padrao, answer in Portuguese, verdict before explanation. Agents only on `opus`. Commits and pushes only on Matheus's account: `Co-Authored-By`, "Generated with Claude" or any credit to Claude is forbidden. Python only in a `.py` file (`C:\Python313\python.exe`), never `python -c`. UTF-8 without BOM. No heavy binaries in the Drive folder: generate them in the scratchpad.
> 
> ## What exists
> 
> - **Round 5 (branch `design/0.1.2`, commit `b90d1b0`)**: the program is the 10 W laser projector in 3D. Sources in `design/laser/`: `app.html` + `app.js` (state `S`, wall, splash, panels, OLED, tick), `body.js` (case and rear panel: ports with `userData.key` = power, keyswitch, interlock, ilda, ildathru, dmxin, dmxout, rj45, usb, oled, enc, back, fan), `optics.js` (optical bench, r/g/b modules, dichroics, shutter, galvos, PCBs, DAC, PSU), `beam.js` (GLSL beam), `cam.js` (SolidWorks camera), `bind.js` (action registration: `Bind.def(id, label, fn, {key, midi, type: "btn"|"cc", get})`, MIDI learn in/out, feedback, `localStorage sc-laser-bind`), `pino3d.js` (menu mascot: pin 3 = "Orchestrator", today it only opens the NET panel with a message), `tokens.css` + `SISTEMA.md` + `sistema.html` (design system), `PEDIDOS.md` (verbatim log of the requests). `design/build.py` inlines the modules into a single HTML. Published prototype: https://claude.ai/code/artifact/8a913f8b-8ea7-4621-b57a-88d7738dbafd (vote in `moodboard/round5`; do not touch the vote).
> - **Functions (working tree of branch `design/funcoes-referencia`, `design/FUNCOES/`)**: `README.md` (11 cross-cutting rules: a typed parameter generates a widget; **a textual address is the identity of everything**; trigger/toggle/value are distinct types; state is paint; an error is data; one verb per concept...), `orquestrador.md` (`spell graph`, PATCHBAY theme: a module with `hasInput/hasOutput`, parameters, values, commands with `context: action|mapping|both`; a route = inputs → filters → outputs + scope + in/out range + return path for the LED; filters Lag/Damping/OneEuro/CurveMap...; multiplex; state; a cable only between ports of compatible type `trigger|bool|number|color|xy|frame|dmx`; `module.json` manifest; the `graph` key of the `.spell` with `nodes[]`, `wires[]`, `states[]`, `view`), `ilda-player.md` (complete table of laser output parameters with names grouped by `/`, armed/shutter/DAC states, shortcuts), `fontes/` (audits with `path:line`). If `design/FUNCOES/` is still uncommitted, it belongs to another session: **read it, do not edit it, do not commit it**.
> 
> ## What to do
> 
> **Goal**: the round-5 projector becomes an **orchestrator module**, and the round-5 bindings become **routes** of the graph. No new PATCHBAY UI yet: the integration is one of contract and data, with the minimum interface for it to be visible in the prototype.
> 
> 1. **`design/FUNCOES/integracao-laser.md`** (new; if you cannot write in `FUNCOES/`, use `design/laser/INTEGRACAO.md`). Table: every port, component and action of round 5 (all the `Bind.def` ids in `app.js` and all the `userData.key` values in `body.js`/`optics.js`) → registry address (`laser/1/kpps`, `laser/1/shutter`, `laser/1/limit/r`, `laser/1/dmx/addr`, `laser/1/net/sacn`, `laser/1/cam/view`...) → type (`trigger|toggle|value`) → `context` (`action|mapping|both`) → graph port type → return path (LED/fader). Use the names grouped by `/` from `ilda-player.md §1` where they already exist; do not invent a new name for a parameter the table already names. Say what from round 5 does **not** become an address (camera, splash, Pino) and why.
> 2. **Module manifest** `design/laser/module.json`, in the format of `orquestrador.md §5` (`parameters`, read-only `values` such as temperature, real fps, points per frame; `commands` with `context`; `dependency` for what only appears when armed).
> 3. **`bind.js`**: `Bind.def` gains `addr` (the registry address) and `ctx`; the bindings table (`Bind.html()`) shows the address next to the label, because the address is the identity (rule 2). `Bind.manifest()` generates the `module.json` from the definitions; `Bind.graph()` returns the `graph` section of the `.spell` with the node `laser/1` and the current bindings as `wires[]` from `midi/<port>` → filter → `laser/1/...` (a continuous CC goes through `Lag`; a note is a direct `trigger`), with `view` in a separate block. Without duplicating the action list: a single source, in `app.js`.
> 4. **In the prototype**: Pino's pin 3 ("Orchestrator") opens an `ORCHESTRATOR` panel (wide) with three blocks: the manifest, the active routes (one line per binding: input → filter → output, with the activity blink lighting up when the action runs) and the `graph` of the `.spell` in a `<pre>` with a COPY button (download is blocked in the artifact). The equivalent command in the corner: `spell graph add laser/1` and `spell graph wire midi/cc:1:7 laser/1/kpps --filter lag`. No more UI than that.
> 5. **Record**: `design/laser/PEDIDOS.md` gains the section "Round 6 · integration with the orchestrator" with this literal request; `design/DECISOES.md` gains the dated entry with what was decided (addresses, what was left out, open issue: a state node does not exist in PRD §10). `design/TEMAS.md` only if something changes theme.
> 
> ## How to verify
> 
> Build: `C:\Python313\python.exe design\build.py design\laser\app.html <scratchpad>\spellcaster-laser.html`. Syntax: `node <scratchpad>\jscheck3.js` (reads `spellcaster-laser.html`, `new Function` per `<script>`). Runtime: `errwrap.py` in the scratchpad generates `err-laser.html` with an `onerror` hook; run headless Chrome with `--dump-dom` and look for `id="ERR"`:
> 
> ```
> "C:/Program Files/Google/Chrome/Application/chrome.exe" --headless=new --no-first-run --user-data-dir=<tmp> --use-angle=swiftshader --enable-unsafe-swiftshader --hide-scrollbars --window-size=1240,1200 --virtual-time-budget=12000 --screenshot=<png> "file:///<scratchpad>/spellcaster-laser.html#tras"
> ```
> 
> The hashes `#tras`, `#dentro`, `#laser` skip the splash. One visual correction pass, not a loop. `module.json` and the generated `graph` need **one** `unittest` test in `tests/` that validates: every `wire` connects ports of compatible type and every address exists in the manifest.
> 
> ## Delivery
> 
> Republish the prototype **in the same artifact** (`url` = the link above, `capabilities` omitted). Commit on branch **`design/0.1.3`** created from `design/0.1.2`, via a temporary `git worktree` in the scratchpad (the project folder is on another branch with a live session: never switch its branch). Message in Portuguese, one subject line, no credit to Claude. Push. In the report: verdict, what became an address and what was left out, link, commit, and what was not done.

- [x] `design/FUNCOES/integracao-laser.md` (written as `design/laser/INTEGRACAO.md` and moved in the 0.1.4 merge): table of port/component/action → address → type → context → graph port → return; what stays out and why.
- [x] `design/laser/module.json` in the format of `orquestrador.md §5`, generated by `Bind.manifest()`.
- [x] `bind.js`: `addr`, `ctx` (+ `arg`, `t`, `range`, `unit`, `needs`), address next to the label, `manifest()`, `graph()`; single source in `app.js`.
- [x] Pin 3 opens the ORCHESTRATOR panel (manifest, routes with blink, `graph` with COPY); command `spell graph add laser/1` in the corner.
- [x] `tests/test_laser_graph.py`; `DECISOES.md`; `TEMAS.md` unchanged.
