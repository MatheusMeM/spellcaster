# Laser integration with the orchestrator (round 6)

The round 5 projector becomes the `laser/1` module of the graph (`orquestrador.md`), and each key/MIDI binding becomes a **route** `input → filter → address`. No new PATCHBAY UI: this is contract and data. Single source of the actions: `app.js` (`Bind.def(id, label, fn, {addr, ctx, …})`); `Bind.manifest()` generates `module.json` and `Bind.graph()` generates the `graph` block of the `.spell`. The two versioned files in `design/laser/` (`module.json`, `graph.json`) are the prototype dump with the default bindings, and `tests/test_laser_graph.py` validates both.

## Rules

- **Address** = `laser/<instance>/<name>`. The name is the CLI one (`--kpps`, `limit --r`), grouped by `/` like the label in the table of `ilda-player.md §1` (`Color/Scale R` → `limit/r`, `Device/Queue` → `queue`, `Geometry/Scale` → `geo/scale`, `ILDA/Actual FPS` → `fps`, `ILDA/Points per frame` → `points`). A new name only for what the table does not name (`arm`, `play`, `clip`, `net/*`, `dmx/addr`, `temp`, `emitting`).
- **Type**: `trigger` fires, `toggle` inverts (or takes a bool), `value` takes a number (or a trigger with an argument, e.g.: `kpps {step}`). Rule 3 of `README.md`.
- **`context`** (Chataigne): `action` only accepts firing, `mapping` only accepts a continuous value, `both` accepts both. Plus two contexts of our own that do **not** go into `commands`: `value` (read-only → `values` of the manifest) and `view` (previz only → `view` block of the `.spell`).
- **Port types** of the graph: `trigger`, `toggle` (bool), `number`. A cable only connects compatible types: `midi/note:*` and `key/*` are `trigger`; `midi/cc:*` is `number`. `number → number` goes through `filter.lag` (continuous CC); `trigger → toggle` inverts; `trigger → number` requires an argument (`step`, `file`); `number → trigger` does not exist without a threshold filter (PRD §10 has none; open issue).
- **Port** is written `<uid>/<port>` (`midi/cc:1:7`, `laser/1/kpps`), not `uid.port` as in `orquestrador.md §5`: the port name already carries `/` (`limit/r`), and this way the port **is** the registry address (rule 2). Recorded in `DECISOES.md`.
- **Feedback** (controller LED/fader): `Bind.feedback` sends `get()` back through the same binding: bool → note/CC 127|0 (LED), number → CC 0..127 (motorized fader).

## Table: port or component → address

Column "from where": `B` = `Bind.def` id in `app.js`, `K` = `userData.key` in `body.js`/`optics.js`, `P` = pin of the Pino (`pino3d.js`).

| Port / component / action | From where | Address | Type | context | Graph port | Feedback |
|---|---|---|---|---|---|---|
| key switch (KEY) | K `keyswitch` · B `key.toggle` | `laser/1/arm` | toggle | both | trigger/toggle | LED |
| rocker + powerCON (AC IN) | K `power` · B `power.toggle` | `laser/1/power` | toggle | both | trigger/toggle | LED |
| play/pause | B `play.toggle` | `laser/1/play` | toggle | both (`dependency: arm`) | trigger/toggle | LED |
| shutter | K `shutter` · B `shutter` (new) | `laser/1/shutter` | toggle | both (`dependency: arm`) | trigger/toggle | LED |
| galvos (kpps) | K `galvo` · B `kpps` (cc), `kpps.up`, `kpps.down` | `laser/1/kpps` | value | both (`{step}` on firing) | number (Lag) / trigger+args | fader |
| ILDA IN (file) | K `ilda` · B `demo` | `laser/1/clip` | trigger `{file}` | action | trigger+args | — |
| ILDA IN (dialog) | B `file.open` | — | | | | | 
| R/G/B module: limit | K `r` `g` `b` · B `lim.r` `lim.g` `lim.b` | `laser/1/limit/r` `g` `b` | value | mapping | number (Lag) | fader |
| R/G/B module: curve | K `r` `g` `b` · B `curve.r` `curve.g` `curve.b` (new) | `laser/1/curve/r` `g` `b` | value | mapping | number (Lag) | fader |
| size | B `size` | `laser/1/geo/scale` | value | mapping | number (Lag) | fader |
| DMX IN (address) | K `dmxin` · B `dmx.addr` (new) | `laser/1/dmx/addr` | value int 1..512 | mapping | number (Lag) | fader |
| galvo driver (buffer) | K `galvodrv` · B `queue` (new) | `laser/1/queue` | value int 1..8 | mapping | number (Lag) | fader |
| NET (RJ45) | K `rj45` · B `net.ndi` `net.spout` `net.artnet` `net.sacn` | `laser/1/net/ndi` `spout` `artnet` `sacn` | toggle | both | trigger/toggle | LED |
| interlock | K `interlock` · B `lock.toggle` | `laser/1/interlock` | bool, read-only | value | (no input) | LED |
| fan / temperature | K `fan` · B `temp` (new) | `laser/1/temp` | float °C, read-only | value | — | — |
| actual fps | OLED OUTPUT · B `fps` (new) | `laser/1/fps` | float Hz, read-only | value | — | — |
| points per frame | OLED OUTPUT · B `points` (new) | `laser/1/points` | int, read-only | value | — | — |
| emitting (BEAM RELEASED) | HUD · B `emitting` (new) | `laser/1/emitting` | bool, read-only | value | — | LED |
| camera view | B `cam.show` `cam.rear` `cam.inside` · K `lid` | `laser/1/cam/view` | enum show/rear/inside | view | — (`view` block) | — |
| haze | B `fog` | `laser/1/cam/fog` | float 0..1 | view | — (`view` block) | — |

The `lock.toggle` binding still exists in the prototype (it simulates pulling the plug), but does not become a route: interlock is a sensor, it enters the `.spell` only as a value.

## What does not become an address, and why

| Thing | From where | Why |
|---|---|---|
| camera: orbit, pan, zoom, frame, standard views, wheel direction | B `cam.rot*` `cam.rot90*` `cam.pan*` `cam.fit` `cam.front…iso` `cam.zoomIn/Out` `cam.reverse` | viewer gesture, not show state; what gets saved is only the view (`cam/view`) and the haze, in the `view` block, which PRD §10 and `orquestrador.md §5` keep apart from the purpose logic |
| splash | `wallSplash`, `skipSplash` | Entry of the 5E arc; the `.spell` does not know it exists |
| Pino and its pins | P `pino`, `pino.ilda` `pino.ndi` `pino.orq` `pino.cues` `pino.nfo` `pino.bye` | program menu; no pin changes the show |
| OLED, encoder, BACK | K `oled` `enc` `back` · B `oled.up` `oled.down` `oled.ok` `oled.back` | physical panel menu; its pages are already addresses (`dmx/addr`, `net/*`, `kpps`) |
| ILDA OUT, DMX OUT, USB | K `ildathru` `dmxout` `usb` | signal pass-through and firmware: no state in the show (`--chain` is a CLI flag, no state in the prototype) |
| aperture, front, side, lid, screws | K `aperture` `front` `side` `lid` | information panels; `lid` only switches the view |
| bench, dichroics, fold mirror, diode driver, DAC board, PSU | K `bench` `dichro` `fold` `pcb` `dac` `psu` | mechanics and service; the DAC board is the `laser/1` node itself; PSU telemetry stays out of v1 |
| galvo driver speed | `speed` slider in `galvodrv` | calibrates the previz low-pass (simulated galvo response), it is not a device parameter |
| bindings, info, Esc, MIDI connect, reset, orchestrator | B `bind` `nfo` `esc` `midi.connect` `bind.reset` `orq` | prototype UI |

## Manifest and graph

- `module.json`: format of `orquestrador.md §5` (`parameters`, `values` with `readOnly`, `commands` with `context`, `dependency`). Difference: each `dependency` carries `target` (§5 omits it because in Chataigne the dependency lives inside the parameter).
- `graph.json`: `nodes[]` (`key`, `midi`, `laser/1` with complete `params`, `lag/N`), `wires[]` (`{from, filter?, to, args?, muted}`), empty `states[]` (PRD §10 has no state node; `DECISOES.md`), `view` in a separate block with positions and the previz `cam`/`fog`.
- Equivalent CLI: `spell graph add laser/1` · `spell graph wire midi/cc:1:7 laser/1/kpps --filter lag`.
- To regenerate both: open the prototype with `#orq` and copy the `<pre>` of the panel; or `errwrap.py` + headless Chrome `--dump-dom` (pre `id=DUMP`).
