# Design decisions

One line per decision. Date, decision, reason. No agent re-decides what is here; to change it, add a new line that revokes the old one.

- 2026-09-09 — Our own design system ("Spellcaster DS", prefix `--sc-`), with Feitiçaria's as the starter kit. Reason: the product needs functional colors (armed, live, rehearsal, error) and a mono voice that the firm's identity does not have.
- 2026-09-09 — Stage `#000000` and Floral White `#F7F5EB` inherited from Feitiçaria; never `#FFFFFF`. Reason: dark room, continuity with the firm.
- 2026-09-09 — The product accent is amber `#FFB000` (selection, focus, active). Reason: console VFD/LED amber, reads from a distance, is not the firm's color.
- 2026-09-09 — Feitiçaria's lime green `#A8E05E` only on GO and "ok"; Wisteria `#B7AED9` only in rehearsal mode; red `#FF2D1F` on armed/live/error. Reason: color = state (principle 2).
- 2026-09-09 — Typography: JetBrains Mono (voice, values, timecode) and Barlow Condensed (labels, overlines in caps). Both OFL, embedded in the binary. Reason: mono is the CLI/MCP vocabulary on screen; condensed fits in a 1×1 widget.
- 2026-09-09 — 8 px grid, 48 px widget unit, 8 px gap, radius 0. Reason: console density, touch target ≥ 44 px.
- 2026-09-09 — No shadows. The "live" state uses a 1 px outline + 12 px glow in the state's color. Reason: flat, glow only with meaning.
- 2026-09-09 — Our own icons, 16 px grid, 1.5 px stroke, square corners. Until they exist, the placeholder is a dashed square with the icon's name.
- 2026-09-09 — Reference Faces: `performance` (kiosk, 4 widgets) and `editor` (splittable areas as in Blender; graph in the center as in TouchDesigner). Reason: proves the Theme/Face separation before R5.
- 2026-09-09 — Colors of the Graph's node families: input amber, logic light gray, command Floral White, output wisteria. Reason: the same idea as TouchDesigner (family = fixed color), without copying its palette.

## 2026-09-09 · moodboard round 1 (rejected)

- Votes: A console NO · B workbench NO · C capsule MAYBE · D grimoire NO · E poster NO · F ribbon NO.
- New brief from Matheus: like WMP skins even if kitsch; nothing boxy-software; window transparent like glass; keygen/cracktro aesthetic with a jingle on the splash; responsive UI, good to use and funny; the experience follows the 5E arc.
- Consequence: `PRINCIPIOS.md` and `tokens/spellcaster.css` still hold for the **inside** of the screen (reading, state, targets). The **shell** is an object: silhouette, glass, gel. Round 2 decisions go below once approved.

## 2026-09-09 · round 2 (prototype, awaiting vote)

- Delivered as a working prototype, not as a drawing sheet: https://claude.ai/code/artifact/106e46a9-8070-4edd-9351-83ac4e7a5e2e. Sources were `design/rodada2/` (deleted from the tree on 2026-09-09; history in git, commit `6fe8b23`).
- Shell = glass with a cut silhouette + gel (real Lee/Rosco gel colors) + optional frost. In the product: Tauri `transparent`, no decoration, acrylic via `window-vibrancy`.
- Entry = cracktro splash with a synthesized chiptune jingle (zero media), skippable, with an honest "never again".
- WMP skins come in for real: a `.wmz` reader (zip + XML + BMP) with silhouette by clippingColor, click map by color, hover/down repainted; Play/Next → GO. Sliders and JScript stay out.
- Compact ribbon (TAB) over Resolume; NFO as the about box; copy with humor (toast), animation only where there is state.
- Pending: round 2 votes (default gel, splash, silhouette, kitsch, jingle, .wmz, ribbon).
- 2026-09-09 — Interface and shortcuts follow Adobe Premiere and DaVinci Resolve (map in `design/SHORTCUTS.md`). Reason: an operator who edits video operates without learning anything new; fixed Ctrl/Shift/Alt grammar.
- 2026-09-09 — Shell rejected again ("boxy software"): the glass becomes a WebGL fragment shader (body SDF + GO dome, refraction with dispersion R/G/B, fresnel, specular at the mouse, sweep, shadow and gel caustic). Reason: an explicit request for recognizable, "spectacular" glass; GLSL cleared by Matheus. Cracktro splash approved ("the cyber matrix came out great").

- 2026-09-09 — 2D-SDF glass rejected too (refs: Skins Factory wmpdesign, Shadertoy 4ll3R7 / 4s2GDV / XlscDH / dl3BRS / 4dSBDt). New direction approved by Matheus ("imagine if the menu lives in that floating cube and that is the window"): the window is a raymarched glass cube (760×470×220, with the GO dome), floating and rotating in front of the desktop; two-face refraction with per-channel dispersion, gel absorption by thickness, volumetric fire (gyroid fBm) inside the cube on GO, two shader passes (body below the HTML, front face above), UI in CSS 3D with the same matrix. MIP fluid (tsKXR3) is left for the gel in a future round: it needs multi-buffer.
- 2026-09-09 — Round 2 vote (Matheus): ribbon YES · form OBJECT · gel NO GEL · jingle CHIP · kitsch MORE · splash FIRST · wmz NO. Caveats: "did not love it, but a million times better"; W (.wmz) broken by the cube; fire makes no sense in a blue cube ("use water or ice here, fire in another skin"); no central vision; "create themes and take themes apart before personas and uses, then build the skins". Consequences: `.wmz` leaves the prototype; the cube becomes the ICE theme (cracks that light up on GO); default gel = no gel; `design/TEMAS.md` is born with the rule "every theme is a material" and four themes (ICE, EMBER, TANK, CHROME) to approve before any skin.
- 2026-09-09 — Matheus throws away the round 2 UI ("use what it taught for the next ones"). New rules: everything with the same look from splash to info; **function before UI** ("it is very hard to do UI for software with no function"); functions requested: ILDA player, NDI→ILDA, Chataigne-style orchestrator, DMX scenes and cues with an interactive stage-set menu, a Clippy-style companion in every skin as the main menu; "be more wild". Answer: `design/TEMAS.md` rewritten as a function ↔ theme map (LASER, FÓSFORO, PATCHBAY, PAPER THEATER + Aprendiz); round 3 = ILDA player in the LASER theme, `design/rodada3/ilda.html` (deleted from the tree; commit `6cc9bfa`).

## 2026-09-09 · round 3 vote and the turn toward the device

Vote (moodboard/round3, 01:53): LASER **adjust** · Aprendiz **another character** · haze **more** · next **NDI → ILDA (FÓSFORO)**. Notes: info on a readable layer (the transparent one did not work); a program that is not boxy, with an interesting edge; a mascot with buttons on it, more Clippy than pixel art.

New directive from Matheus, right after: **the program is the 3D model of the 10 W laser itself.** Rear panel = menu (ports, buttons, VFD). Preferences = the camera rises, the screws come out, the lid opens, and each component is its own setting (diode = limit and curve; galvos = kpps; board = buffer and speed; VFD = DMX address and connections). It has to look real, photorealistic, not cartoon.

Consequences:
- **Pino** (`design/pino.js`, deleted from the tree; commit `7dacf8f`): a DMX cable with an XLR-5 plug for a head; the five pins are buttons (1 ILDA, 2 NDI→ILDA, 3 orchestrator, 4 scenes and cues, 5 info), the latch means "go away". SVG vector, eyes that follow the mouse, Win98 speech balloon. Replaces the Aprendiz in every skin. `design/build.py` inlines the file into the round for publishing.
- **Round 4 = `design/rodada4/projetor.html`** (deleted from the tree; commit `7dacf8f`): three.js r128 (jsdelivr; cdnjs has no UMD build), PBR with a procedural studio PMREM, ACES, PCF shadows, brushed aluminium (procedural normal + roughness), plates with chamfer (ExtrudeGeometry), dichroics in MeshPhysicalMaterial. The wall is the round 3 2D canvas as an additive texture; the beams leave the model's aperture. Opaque, chamfered parameter panel (lesson from the vote). Room with a chamfered clip-path and a step (a non-boxy edge).
- **NDI → ILDA (FÓSFORO)** did not die: it becomes what shows up on the ETHER port (a rack monitor plugged in there). Left for round 5 if the vote confirms it.
- Round 3 (`ilda.html`) stays as a record; the LASER adjustment was absorbed by round 4.

## 2026-09-09 · round 5 (prototype published, awaiting vote)

- Prototype: https://claude.ai/code/artifact/8a913f8b-8ea7-4621-b57a-88d7738dbafd · design system: https://claude.ai/code/artifact/2bada8a5-b991-43b7-a086-1b72b0a42232. Sources in `design/laser/` (modules `ilda.js`, `cam.js`, `bind.js`, `mat.js`, `body.js`, `optics.js`, `beam.js`, `pino3d.js`, `app.js`, page `app.html`, `tokens.css`, `SISTEMA.md`, `sistema.html`). `design/build.py` inlines any local `<script src>` and `<link>` to publish as a single file.
- Matheus's requests for this round stay verbatim in `design/laser/PEDIDOS.md` (all marked as done; the vote decides what gets adjusted).
- Rear panel inspired by the Kvant Clubmax, without copying it: powerCON TRUE1, rocker, key switch, EMISSION LED, a single interlock, no fuse, ILDA IN/OUT DB25, DMX IN/OUT XLR-5, NET RJ45, USB, OLED with encoder and BACK, a 60 mm axial fan modeled for real (ring, hub, 7 blades, wire grille), serial plate.
- Inside, "nothing flying": aluminium optical bench (a single silver block, M4 drilling), three modules on mounts, two dichroics and the fold mirror in kinematic holders, solenoid shutter, galvo block on an angle bracket (X vertical, Y at 90°, mirrors that follow the galvo), diode and galvo drivers and the DAC board on the walls, dark; 48 V PSU; nine cables routed from point to point.
- Beam in GLSL: instanced cylinders (core + halo), alpha by dot(N,V), dust by 1D noise, additive, bloom. Outside: aperture → lit points on the wall. Inside: optical path module → dichroic → fold → shutter → galvo X → galvo Y → aperture, lights up with the key switch and the shutter cuts it.
- Splash = the camera aims at the output: it starts on a static dot, the focus goes to the wall ignoring the laser, the galvo traces SPELLCASTER LASER (marching squares over the text in Michroma), each closed letter is revealed, everything glows, the laser goes out, the room darkens and the camera lands on the rear panel. No ILDA on the wall during the splash.
- Flow: the SHOW view stays locked until the key switch arms (settings first, show after).
- Camera in the SolidWorks standard (`cam.js`): MMB orbits around the clicked point, Ctrl+MMB pans, Shift+MMB zooms, wheel at the cursor (SolidWorks direction with a toggle), arrows 15°/Shift 90°/Ctrl pan, F frames, Ctrl+1..7 views.
- Bindings (`bind.js`): every action has an id; key or MIDI (note/CC with channel) with LEARN, output feedback to the controller, persisted in localStorage. Mirrors MadMapper/Resolume.
- Pino 3D replaces Pino 2D (`design/pino.js`, today only in git): a DMX cable plugged into DMX OUT, male end standing on the case, five pins as buttons in the scene, Win98 speech balloon anchored to the projection of the head.
- A design system for this tool only: `tokens.css` (LASER/amber/red/OLED colors, Michroma + Share Tech Mono, scale, chamfers, glows) + `SISTEMA.md` + `sistema.html`.
- Pending: round 5 vote (rear panel, inside, splash, camera, bindings, Pino 3D) in `moodboard/round5`.

## 2026-09-09 · round 6 (integration with the orchestrator)

- Prototype republished into the same round 5 artifact (the `moodboard/round5` vote intact). Sources: `design/FUNCOES/integracao-laser.md`, `design/laser/module.json`, `graph.json`, `bind.js`, `app.js`; test `tests/test_laser_graph.py`. Branch `design/0.1.3`.
- 2026-09-09 — The projector is the graph's `laser/1` module; every key/MIDI binding is a `input → filter → address` route. Addresses: `laser/1/arm`, `power`, `play`, `shutter`, `kpps` (value, and trigger with `{step}`), `clip {file}`, `net/ndi|spout|artnet|sacn` (commands); `limit/r|g|b`, `curve/r|g|b`, `geo/scale`, `dmx/addr`, `queue` (parameters); `interlock`, `temp`, `fps`, `points`, `emitting` (read-only values). Name = the CLI's, grouped by `/` like the label in `ilda-player.md §1`. Reason: rule 2 of `FUNCOES/README.md`, an address is the identity of everything.
- 2026-09-09 — Stays out of the registry: camera (a viewer gesture; only `cam/view` and `cam/fog` survive, in the `view` block of the `.spell`), splash, Pino, OLED/encoder/BACK (the pages are already addresses), ILDA OUT/DMX OUT/USB, mechanics (bench, dichroics, fold, PCBs, PSU), driver speed (previz calibration). Reason: none of that changes the show.
- 2026-09-09 — A graph port is written `<uid>/<port>` (`midi/cc:1:7`, `laser/1/kpps`), not `uid.port` as in `orquestrador.md §5`: the port name already carries `/` and this way the port is the registry address itself. Reason: one single name.
- 2026-09-09 — A continuous CC goes through `filter.lag` (80 ms); note and key are a direct `trigger`; `trigger → value` requires an argument; `number → trigger` has no filter in PRD §10 (it stays a route with no filter, the test does not cover it). The manifest's `dependency` carries `target` (§5 omits it).
- Open issue: a **state node** does not exist in PRD §10 (`orquestrador.md §1`, "State"). `states[]` comes out empty from `Bind.graph()`; "in the second act these routes hold and those stop" is a condition on every route today. Pending decision: adopt the whole Chataigne semantics (a container of routes with `active`, `on load`, transitions) and add `state` to the PRD's node catalog.
- Open issue: a threshold filter (`number → trigger`) for a CC on a trigger action; `bind.js` already treats CC > 63 as a trigger, the graph cannot say so.

## 2026-09-09 · design tree cleanup (0.1.4)

- 2026-09-09 — `design/funcoes-referencia` goes into `design/0.1.4` by merge: `design/FUNCOES/` (functions, cross-cutting rules and audits in `fontes/`) now lives alongside `design/laser/`. Reason: function before UI, both halves on the same branch.
- 2026-09-09 — `design/laser/INTEGRACAO.md` becomes `design/FUNCOES/integracao-laser.md` and enters the table in `FUNCOES/README.md`. Reason: it was a function contract, and it lived in `laser/` only because the branch had no `FUNCOES/`.
- 2026-09-09 — Rounds 2, 3 and 4 leave the tree (`design/rodada2/`, `design/rodada3/`, `design/rodada4/`, `design/pino.js`); the history stays in git (commits `6fe8b23`, `6cc9bfa`, `7dacf8f`). Reason: round 2 was rejected in the vote and rounds 3 and 4 were absorbed by `design/laser/`; dead code in the tree costs reading and keeps nothing that git does not already keep.

## 2026-09-09 · State node and module node in the graph — awaiting vote

- **`state` node.** Config `group` (default `"main"`) and `initial`; inputs `enter` and `exit`, output `active`. One active per group: a pulse on `enter` turns this state on and the others in the same group off, `exit` turns it off, `locate`/`stop` returns to `initial`. It is Chataigne's State Machine with the part that fits `PRINCIPIOS.md`: no transition with fade, no sub-machine, no multiple actives in the same group.
- **`module` node.** Config `module`, which names `modules/<name>.json` next to the show — Chataigne's `module.json` (`parameters`, `values`, `commands`), rule 14 of `FUNCOES/README.md`. Each `parameter` is an input (on change → `Ev::Param{target:"<module>/<path>"}`, `norm` maps 0..1 before the clamp on `min`..`max`), each `value` is a level output fed by `input {key:"module:<module>/<path>"}`, each `command` is a trigger input. The PATCHBAY builds the node by reading the file, without knowing the app.
- **Keys `"mute": true` and `"state": "<id>"` on any node.** Both mean the same thing at runtime: the node does not emit — outputs at 0, no events, a pending `time.delay` canceled. On the way back, the stored value (toggle, latch, counter) is still there, the edges are cleared and the next level output is re-emitted (Chataigne's "re-emit on activate"). It is TouchDesigner's Bypass with no new verb (rule 9).
- **One verb per concept, and `lock` and `solo` stay out of the runtime.** `lock` is editing only (it freezes the value in the editor) and `solo` is muting the others, computed by the PATCHBAY: neither needs code in the engine.
- **What the vote decides:** whether the state is exclusive per group (here) or several can be active at once (Chataigne), and whether `norm` maps the 0..1 signal to the range (here) or is just the slider range in the GUI.
- Reason: these are the two gaps that `design/FUNCOES/orquestrador.md` points out against Chataigne; without them, "in the second act this set of routes starts to hold" becomes a condition copied onto every route, and a separate app never becomes a node.

## 2026-09-09 · Module node → command: naming convention and paths with no implementation — awaiting vote

- **Convention.** The `module` node emits `Ev::Param{target:"<mod>/<path>"}` and `Ev::Cmd{name:"<mod>/<cmd>"}`; the CLI sink routes to `<mod>_param {feed:"<mod>", path, value}` and `<mod>_<cmd> {feed:"<mod>", ...}`. That is: **`feed` = the module's name**. With two lasers open, both `module` nodes would have to be called `laser` and the routing collides — the way out is a named instance (`laser@stage`), and the vote decides whether it comes in now or when the second console shows up.
- **Paths declared with no implementation.** `modules/laser.json` declares `dev/type`, `dev/host`, `dev/pps`, `ilda/fps`, `curve/r|g|b`, `safe/zone`, `safe/armed` and `test/pattern`; `laser_param` accepts none of them (`dev/*` and `ilda/*` are arguments to `laser_open`/`laser_play`, the rest expect a color LUT and a parametric `optimize`). The manifest is the app's declaration, not the command's: the vote decides whether it only declares what already runs, or declares the target and the command grows into it.
- **`shutter` is on both sides.** It is a `command` in `modules/laser.json` and a `path` in `laser_param`. One of the two goes away.
- Reason: the convention is in the code (six lines in the CLI sink, with a `ponytail:` comment) and works for one laser; recording it here keeps it from becoming a contract by omission.

## 2026-09-09 · The navigation bar in the Face (kiosk mode) — awaiting vote

- **What exists.** `spellgui/web/nav.js` puts the same bar (TIMELINE/PATCHBAY/THEATER/FACE/LASER tabs, `Shift+1`..`Shift+5`, ENGINE/`rev` and the editable show name) at the top of the five pages, `face.html` included. Without it the Face is the only page with no way out: whoever opens on it has no way back.
- **The conflict.** PRD §10 and `spellgui/web/README.md` describe the Face as kiosk: no chrome, nothing editable, touch targets. A bar with the show name in a text field is chrome, and it is editable.
- **What the vote decides:** the bar disappears from the Face (and the operator gets back via `Shift+1`, which still holds), or it disappears only in `performance` mode (and stays in `editor`), or it stays as it is.
- Reason: it is a product decision, not an implementation one — the three outcomes cost the same line of code.

## 2026-09-09 · Fixture x/y in the patch — awaiting vote

- The timeline previz (`spellgui/web/viewer.js`) draws the patch plan, and the patch has no
  record of where the fixture is: `patch_add` writes `{name, profile, universe, address}` and nothing else. Meanwhile,
  the plan is a grid in address order — it reads the rig, not the room.
- **What the vote decides:** whether the patch entry gains `x`/`y` (a plan in meters, with the stage at the
  origin) and whether they go into the `.spell` or into a plan file next to it.
- Not implemented and not removed until the vote: the address grid stays, and becomes a real position the day
  the patch can say where the fixture is.

## 2026-09-09 · DMX input and recording: format of `inputs` and shape of the recorded keyframe — awaiting vote

- **`inputs` is a list of `{type, universe}`, sibling of `outputs`.** `[{"type":"sacn","universe":1},{"type":"artnet","universe":2}]`: one universe per input, no `interfaces` and no priority. The alternative was to mirror `outputs` (`{"type":"sacn","universes":[1,2],"interfaces":[...]}`), which matches what is already in the file but repeats network configuration the input does not use (multicast joins every declared group; Art-Net arrives by broadcast). The vote decides which of the two shapes becomes the `.spell` v1 contract — swapping later breaks a recorded show.
- **What recording writes.** One `linear` keyframe per value CHANGE, with no thinning: a fader moving at 60 fps leaves 60 keyframes per second in the track. The alternative is to record reduced (Douglas-Peucker at the end of the take) or stepped (`hold`, which reproduces the console byte for byte but does not interpolate at a different fps). The vote decides the default; the code is on `linear` with a `ponytail:` pointing at the reduction.
- **The track's width (how many channels record) comes from the keyframe that already exists**, not from a field. An empty track records a single channel, at the `address`. The alternative is a `channels` field in the `dmx` track — one more field in the `.spell` for the "I armed a new 4-channel track" case.
- **Out of scope, and why:** HTP merge input→output (passthrough) and laser/OSC recording are left for later; video (NDI/GStreamer) is blocked by the SDKs not being installed (ROADMAP R2), not by a design decision.

## 2026-09-09 · MIDI input (`midi` workstream) — what stayed out, awaiting vote

- **The event key is `"<status>/<data1>"`** (`144/60` = note on channel 1 note 60, `176/1` = CC 1), and not the `note:1:60` / `cc:1:7` of the `design/laser/bind.js` prototype: it is the key the graph's `in.midi` node already uses (PRD §10) and the channel already comes in the status. When `bind.js` becomes product the two formats have to become one; the vote decides which.
- **MIDI output and surface feedback (LED, motorized fader) stay out** — awaiting vote. It is what makes the controller show the show's state; `bind.js`'s `feedback()` already sends note/CC back, so the function exists in the prototype and not in the engine.
- **MTC / MIDI clock and MIDI Show Control stay out** — awaiting vote. They are the path for Spellcaster to receive GO (or timecode) from a sound or video console; it is a product function, not an implementation detail.
- One MIDI port per process: a keyboard and a surface at the same time collide, because the key does not say which port the event came from. It leaves the workstream as a noted limit (`// ponytail:` in `spellcore/engine/src/midi.rs`); the vote decides whether the port enters the key or whether each surface becomes a module.

## 2026-09-09 · DAW timeline: named marker, track lock, color and height — awaiting vote

Origin: `design/FUNCOES/timeline-daw.md` §3 (a reading of Ableton Live 12 and of the DaVinci Resolve manual installed on this machine).

- **A marker becomes an object.** Today there are two formats in the same field: the GUI writes `markers: [12.5, 40.0]` (numbers, `spellgui/web/timeline.js:198,864`) and the engine already has a test with `markers: [{"t": 1.0, "name": "um"}]` (`spellcore/engine/tests/patch.rs:44`); neither one fails, because `show_patch` accepts any JSON. Recommendation: **an object `{t, name, note}`**, with a number accepted on read and converted on load (local migration, `FUNCOES/README.md §12`). Reason: a marker with no name is useless at 11 p.m., and Resolve shows why — name, note and a dot on the marker that has a note (manual p.548, p.784).
- **`lock` per track.** `tracks[i].lock: true`, a boolean the engine ignores: a locked track will not let a keyframe be moved, deleted or selected. Recommendation: **it goes in**. It is already consistent with the 09-09 decision ("lock is editing only, no runtime").
- **Color per track: does not go in as a field.** Ableton lets the user paint each track; `PRINCIPIOS.md §2` forbids decorative color. Recommendation: the color comes from the track's `type` (`dmx`, `laser`, `fx`, `cue`, `media`), like the Graph's node families — zero new fields. For the same reason, a marker does **not** get a `color` (`SHORTCUTS.md` already fixed markers as gray).
- **Track height and lane folding: do not go into the `.spell`.** That is window state, forbidden inside the show by `FUNCOES/README.md §12`; they go to the GUI's `config.json`. `loop` likewise (transport state); if it ever needs persisting, the place is `transport.loop`, which already exists in the file.
- **Not format, runtime, and it has to go in:** track `mute` does nothing in the Rust engine (there is no `mute` and no `solo` in `spellcore/engine/`; the `mute` in `spellcore/script/src/graph.rs:200` is the graph node's, another thing; track mute only in the Python prototype, `spellcaster/gui/api.py:168-171`). Today the timeline's M and S buttons are paint: the DMX keeps going out. `solo` remains "mute the others", computed on the client.

## 2026-09-09 · DAW interface (`daw-pesquisa` workstream) — awaiting vote

Six new documents in `design/FUNCOES/` (`daw-arranjo`, `daw-sessao`, `browser-dnd`, `mapping`, `audio-video`, `pontos-falhos`), read from the Ableton Live 12, DaVinci Resolve 20 and Resolume Arena manuals. What they decided is there; what they do **not** decide is here.

**`.spell` format**

- **`clips[]` per track**: `{"t0", "len", "src", "offset"}` in seconds, `src` relative to the show folder. It coexists with `keys[]` and with the parameter lanes, it replaces nothing. `migrate()` converts the laser track's `clip` (singular) without bumping `VERSION`. Reason: today a 46.8 s laser clip draws an empty lane, because the timeline only knows `keys` (`daw-arranjo.md §4.1`).
- **Track types `audio` and `video`**, with `clips[]`, a `gain` lane and no `universe`. The alternative is for them to be outputs, and they are not: they have a position in time (`audio-video.md §1`).
- **The order of `tracks[]` is the order on screen**, so reordering rewrites the array and any stored index (cue, mapping, address `track/3/mute`) starts pointing at another track. The alternative is a **`uid` per track**, which is what `FUNCOES/README.md §12` already requires for references between objects, settles Resolume's "Shortcut Target" for good (`mapping.md §7`) and costs one field. **It is the widest-reaching decision of this round.**
- **`marker.go: "<address>"`** turns a marker into a locator (Ableton §6.4) with no new object, and **`cue.fires: ["<address>", ...]`** is what gives media tracks a grid cell, which the `cue.values` (only `DMX address → number`) cannot reach. The vote decides whether both are named alike (a list in both) or whether the marker keeps a string.
- **`"midi"` becomes `"map"`, with the source in the key's prefix** (`"key:Space"`, `"midi:144/60"`, `"osc:/spell/go"`, `"widget:go"`) — the same vocabulary that `input {key}` already documents and that the graph's `chave()` already produces. `midi_map` becomes `map_set`; `migrate()` prefixes the old block. It is what keeps the `midi` workstream and the mapping mode from having two sources of truth (`mapping.md §6`). Round 5's `bind.js` (key `note:1:60`, persistence in `localStorage`) loses both: the key becomes the engine's and the map goes into the `.spell`.
- **`show.outputs[]` gains `{"type":"screen","monitor":N}`** for the second video window on the projector (`audio-video.md §4`), and the video track points at it via `screen`.

**Shortcuts**

- **`Ctrl+Shift+A` becomes the mapping mode** (the owner fixed the key). Consequence: "select nothing" leaves `Ctrl+Shift+A` and goes to **`Alt+A`**, by `SHORTCUTS.md`'s own grammar (*"Alt = variant/clears"*, as `Alt+I`/`Alt+O`/`Alt+X` already do). Source note: in Resolume `Ctrl+Shift+A` is the Advanced Output; its shortcut modes are `Shift+Ctrl+K/M/O/X`. The key stays as the owner asked.
- **`Tab` remains the Face switch** (editor ↔ performance, PRD §10). Arrangement ↔ Session is the **second hit of `Shift+2`**, in the same logic as `Shift+Z` (fit, hit again and go back) and `M` (create a marker, hit again and edit). Third time `Tab` has been contested; it is decided and leaves the open issues in `FUNCOES/README.md`.
- **`Ctrl+E` acts on the selected object**: a clip splits (Ableton §6.12), a keyframe opens the easing menu (`SHORTCUTS.md`). One shortcut, two objects.
- **Track height per track** (Resolve p.643) against the global height proposed in `timeline-daw.md` item 19.

**Deliberate refusals, recorded so they do not come back by forgetfulness**

- **Consolidate** (Ableton §6.13): writes a new sample in `Samples/Processed/Consolidate`. We do not render media and we do not write a derived file into the show folder.
- **Follow actions** (Ableton §16.7): two actions with probability, ten types, `Jump Target`, a loop multiplier. Our cue already has `follow: bool` (= the `Next` Follow Action), and the rest is a state machine hidden in the cue list — and the state machine is already being designed in the right place (the graph's `state` node).
- **Toggle, latch, counter and threshold in the direct map**: they go to the graph, which already has `logic.*`, `math.*`, `time.*` and `state` in its closed catalog. The flat map holds key → command and no state.
- **Four mapping modes per protocol** (Resolume, `Shift+Ctrl+K/M/O/X`, one color each): a single mode, because the protocol already comes with the input and four colors go against `PRINCIPIOS.md §2`. `K`/`M`/`O` survive as a source filter **inside** the mode.
- **NDI and Spout**: a licensing block (a registered SDK, against `FUNCOES/README.md §14`) and a GPU-context block in the WebView, not an effort block. The real case — getting an image to the projector — is solved with the second window.
- **Free color per track** (Resolve p.621, 16 colors), **red automation × blue modulation** (Ableton §26.3), **colored marker**: `PRINCIPIOS.md §2`, color means state.
- **`.mov` as a video format** (Ableton §27.1): the criterion is the player's, and the player is the WebView.

**Proven defect, for whichever workstream fixes it**

- `spellgui/web/timeline.js:308-312`: `commit()` rewrites `spec.mute` from **each** lane, and the parameter lanes of the same track carry the old copy — the last write wins and undoes the mute the operator has just turned on. The same holds for `solo`. Reproduced with `shows/medgrupo.spell` (the laser track has `.rot` and `.scale` lanes over the same `spec`). The fix is a single source: `L.mute` becomes a read of `L.spec.mute`.

## 2026-09-09 · round 2 integration

- 2026-09-09 — **The plain mouse wheel scrolls the content in the PATCHBAY too**, and `graph.js` now
  writes `k.ymax` (`desenha()`, from the lowest box). Reason: one single gesture on both screens
  (`wheel` scrolls, `Shift`+wheel pans, `Ctrl`+wheel zooms), and writing `ymax` costs two lines against
  the capture listener that would be needed to give zoom back to the plain wheel in the graph. An
  integration decision, **reversible**: if the vote says a node graph must zoom on the plain wheel (like
  Blender and TouchDesigner), `canvaskit.js` gets the branch and `ymax` keeps serving the clamp.

## 2026-09-09 · In-Out loop with an armed track overwrites the take — awaiting vote

- Transport loop (`loop_set`, In–Out) and recording (`rec_arm`) are independent states: with
  both on, every pass of the loop records over what the previous pass recorded. There is no bug; there are
  two possible semantics and neither one has been chosen.
- **What the vote decides:** (a) the loop's wrap **disarms** the track (one pass, one take, like Pro Tools'
  punch), or (b) it stays as it is and the documentation says that loop + arm overwrites
  (like destructive overdub), or (c) each pass becomes a new take — which is a new field in the `.spell` and
  does not come for free.
- Not implemented and not removed until the vote: today it is (b), with no warning on screen.

## 2026-09-09 · The program window's home page — awaiting vote

- **What the vote decides:** when the Spellcaster window opens, what shows up first — the **device** (`spellgui/web/laser3d/app.html`, the 3D laser projector, with the splash, the rear panel as the menu and Pino as navigation) or the **timeline** (`spellgui/web/index.html`).
- **Recommendation: the device.** It was the literal request ("I do not want to open in the browser, I want a GUI of the program" · "let the UI already be wild and with 3D and with GLSL shaders and cool to operate"), and it is the "function before UI" rule actually applied: the program is the device, the timeline is the device's recorder. Opening at the recorder inverts the metaphor and gives back the boxy software.
- **What already stands in both cases:** `laser3d/` is the main page and runs wired to the registry (`laser_open`, `laser_play`, `laser_stop`, `laser_close`, `laser_param`, `laser_stats`, `laser_files`, `resume`/`pause`, `show_get`) through `bus.js`, with three.js and the fonts vendored in `spellgui/web/vendor/` — no CDN, the event has no network. With no engine the page is still whole, in local mode.
- **What stays out while the vote is pending:** the native window (Tauri) belongs to the `gui-janela` workstream; this decision is only about which URL it loads first.
- Reason: it is a product choice, not a code one — swapping the home page is one line, but it defines what the program **is** when it opens.

## 2026-09-09 · What goes into Pino's menu — awaiting vote

- **The five pins remain the five screens** (1 laser · 2 phosphor · 3 patchbay · 4 theater · 5 info). The speech balloon gained **two items that are not pins**: the **RECORDER** (the timeline, `index.html`) and the **CONSOLE** (the Face, `face.html`). Each carries the sentence that justifies the piece: the timeline is the device's tape, the Face is the big buttons the operator hits during the show.
- **What the vote decides:** whether a piece without a pin may live in the speech balloon, or whether each one has to become a pin — which would demand a Pino with seven pins (XLR-7 does not exist) or a second cable.
- Reason: the rule is "nothing shows up out of software convenience". Two items with no pin are the exception the balloon is opening; either it is accepted with the justification, or the device has to grow a connector.

## 2026-09-10 · chassis-4 · The lid hinge goes at the FRONT, not at the back

- **The request said** "a real hinge, at `z = D/2`" (rear). **It ended up at `z = −D/2 + 4.5 mm`** (front), at the center of the front edge's radius.
- **Reason:** what opens the lid is `app.js`, which writes a **negative** angle into `lid.rotation.x`. With the axis at the back, a negative angle throws the plate down and backwards: it goes through the rear panel and the flight case — exactly the "lid clipping" that was reported. With the axis at the front, the same negative angle opens the lid up and forward, sweeping nothing between 0 and −1.9 rad, and with no need for an artificial travel limit.
- **The alternative was to edit `app.js`** (flip the sign), and `app.js` is not part of this workstream. If the integrator prefers the hinge at the back, the fix is one line in `app.js` (`rotation.x = +angle`) plus moving `lid.position.z` back to `D/2 − 4.5 mm`.
- **Side effect left to the integrator:** `app.js` raises the lid screws by 50 mm (`s.position.y = .004 + sT * .05`). They are children of the lid and follow the rotation, but the travel is exaggerated; 8 mm would do.

## 2026-09-10 · `Z` and `Shift+Z` swapped, to match the SolidWorks manual

- It was `Z` = zoom **in** and `Shift+Z` = zoom **out**. Dassault's official quick reference
  (`quick_reference.pdf`, p. 1, `SWQRCENG06060`) says the opposite: **`Z` zooms out, `Shift+Z` zooms in**.
  Since the SHOW view's camera now copies SolidWorks wholesale (`design/FUNCOES/camera-solidworks.md`),
  keeping half the map inverted would be the worst of the two options: whoever knows the CAD gets it wrong, and whoever does not
  know it gains nothing. Both remain addresses (`cam.zoomIn`, `cam.zoomOut`), remappable.
- It was not in the request; it was decided here because the request said to follow the manual and the manual disagrees
  with what was there. **Reversible in one line** (the two keys in `app.js`'s `Bind.def`) if the vote says
  the "Z zooms in" intuition is worth more than compatibility with the CAD.

## 2026-09-10 · hud-4 workstream · The name of the command that maps an input — awaiting vote

- The **interlock stopped being a button** on the laser page: it is an **input** of the device, and the drawer's INTERLOCK tab is where you declare **who triggers it**. The identity is a textual address, `laser/1/interlock`, the same across key, MIDI, OSC, Art-Net, MQTT and CLI (FUNCOES/README rule 2).
- **What already works today:** key and MIDI, through the `Bind` the page already had (`Bind.def("lock.toggle")`), and in the engine MIDI through `spell midi_map --key 176/1 --cmd laser_param`. The `SIMULAR ABERTURA` button triggers the input with no hardware.
- **What the vote decides:** the name of the registry command that maps a **non-MIDI** source to an address. The proposal is one single form for all of them: `spell map <source> <address>` — `spell map osc laser/1/interlock`, `spell map artnet 1/512 laser/1/interlock`, `spell map mqtt spell/laser/1/interlock laser/1/interlock`. The alternative is one command per protocol (`osc_map`, `artnet_map`, `mqtt_map`), like the `midi_map` that already exists.
- **What stays out while the vote is pending:** the command itself. The tab shows the address, the universe/channel and the topic (stored in `localStorage`, ready to become arguments) and writes **"coming soon"** on the line of the command that is missing. No pretending to map.
- Reason: `midi_map` exists and sets a precedent for the second form; the first is a single CLI line for N protocols. It is a choice of product command name, not of code — and a command name is a contract.

## 2026-09-10 · hud-4 workstream · The HUD's Art-Net/sACN line: a declaration lights the text, a frame lights the LED

- The HUD's statistics block writes only **engine fact**. The line for an Art-Net or sACN input appears **if, and only if**, `show_get {full:true}` returns that input in `show.inputs`; its LED lights **only when a real frame arrives** (binary topic 2 of the bus), and goes out on its own when the frames stop.
- `shows/medgrupo.spell` declares no `inputs`: that is why today **there is no Art-Net/sACN line on screen** — and that is how it has to be. An unlit LED next to a protocol nobody configured is noise; an invented protocol is a lie.
- Same rule on the chassis plate: the firmware version comes from the engine, and with no engine the plate writes `FIRMWARE OFFLINE` instead of a number.

## 2026-09-10 · The interior light blows out any material — awaiting vote

- Measured on the bench (`bench.html?v=optica`, `sun` spot at intensity 50) and checked in `app.js`
  (the same spot at intensity **90**, with no `physicallyCorrectLights`): every diffuse surface facing
  up saturates. A black anodized aluminium with albedo 0x0d1013 (0.012 linear) comes out of the render
  light gray; a red wire 0x4d130e comes out pink; a board with solder mask 0x05130c comes out a blaring
  green. With that gain there is no such thing as a dark albedo: the material only goes dark again if it is
  metallic (`metalness >= .85`), because then there is no diffuse to blow out.
- **What this has already cost in `optics.js`:** the whole palette had to become metal (bench, mounts,
  motors, heatsinks) and the colors of the wires and the boards had to be darkened twice just
  so they would not shine more than the optical bench. It is a workaround, not a fix — the part is compensating for the light.
- **What the vote decides:** (a) lower the spot and raise `toneMappingExposure` in `app.js`/`bench.html`
  until a dark albedo reads dark (it is one line in each file, but it changes the look of every
  workstream at once), or (b) turn on `renderer.physicallyCorrectLights = true` and recalibrate the three
  lights in candela, or (c) keep it as it is and accept that every interior material is metallic.
- Reason: `app.js` and `bench.html` are not part of this workstream, and touching the light changes the body, Pino and the
  wall all at once. Recorded with a measured number for whoever calibrates it.
