# Spellcaster functions and themes

Matheus's rule (2026-09-09): **you do not design UI for software with no function.** Every screen is born from a concrete function; the theme is the material that function wears; splash, work screen and info all look the same. Everything made in rounds 1 and 2 (CSS glass, 2D SDF, raymarched cube) is thrown away as UI and kept as technique (see "Salvage").

The rule that binds it: **a theme is a material, and everything in the skin obeys the material** (shape, what lives inside, what GO does, how state shows up, what sound it makes).

## The five functions

| # | Function | What it does (and the command) | Theme (material) | Round |
|---|---|---|---|---|
| 1 | **ILDA player** | Opens `.ild` (formats 0/1/4/5), plays frames, controls kpps, size, color, shutter. `spell ilda play show.ild --kpps 30`. Ether Dream / Helios output. | **LASER**: the UI is the photorealistic 3D model of the 10 W projector itself in a hazy room. Rear panel = menu (ports, VFD, buttons); lid open = preferences (diodes, galvos, board, PSU). The wall shows the frame with real physics: beams leave the aperture, the galvo lags, the frame flickers if kpps ÷ points drops. | 3 → 4 |
| 2 | **NDI → ILDA** | Takes NDI video in, extracts contours (Canny + simplification), turns it into an ILDA frame in real time. `spell ilda from-ndi "RESOLUME (out)" --kpps 25`. | **FÓSFORO** (phosphor): a 1970s broadcast rack. On the left a raster CRT (the NDI), on the right a vector oscilloscope (the ILDA). Green phosphor, Tektronix knobs, line noise. | 4 |
| 3 | **Orchestrator** | The Chataigne mode: modules (sACN, Art-Net, OSC, MIDI, NDI, ILDA), states, sequences, mappings. It is the Graph from `PRINCIPIOS.md §1` seen head-on. `spell graph`. | **PATCHBAY**: a 1960 telephone exchange. Bakelite, brass jacks, cloth cables with physics, Dymo labels. To map = to plug a cable. State = valve lamp. | 5 |
| 4 | **DMX scenes and cues + interactive stage set** | Programs scenes (values per fixture), cues (scene + fade + follow), and a stage-set menu where you click the fixture in the scale model. `spell cue`, `spell scene`, `spell patch`. | **PAPER THEATER**: a cardboard stage model. Fixtures are cutouts that light up; scenes are wing flats that slide; cues are pages of the libretto. GO turns the page. | 6 |
| 5 | **Pino** (companion) | The Spellcaster's Clippy and the **main menu** of every skin. A DMX cable with an XLR-5 plug for a head: the five pins are buttons (1 ILDA, 2 NDI→ILDA, 3 orchestrator, 4 scenes and cues, 5 info), the latch sends him away. 3D (`design/laser/pino3d.js`), eyes that follow the mouse, Win98 speech balloon. He knows the context: he warns when the galvo cannot keep up, when the frame flickers, when a universe stops answering. He remembers the last session. He replaced the pixel Aprendiz in the round 3 vote. | Has no theme: he is the same in all of them, the way Clippy was the same across every Office. | 4 onward |

Build order: 3 → 4 (the 3D projector) → 5 → 6, one function per round, each with a vote. Pino grows every round. NDI → ILDA (FÓSFORO) shows up on the projector's ETHER port.

## The five questions per theme

| | LASER | FÓSFORO | PATCHBAY | PAPER THEATER |
|---|---|---|---|---|
| **Matter** | Beam in haze; wall as screen; vector with glow and persistence | P31 phosphor on curved glass; brushed aluminium; screen printing | Black bakelite; brass; cloth cable; felt | Cardboard, kraft paper, gouache paint, candlelight |
| **Phenomenon** | The beam's standing dot; haze in motion; corners rounded off by the galvo | 60 Hz line noise; retrace; burn-in of the previous image | Cables swing; lamps flicker when signal passes | Paper ripples in the air; candle shadow |
| **GO** | Shutter opens, the frame appears with the trace running | Trigger fires: the CRT freezes, the vector draws | The relay clacks, the route lamp lights | The libretto page turns; the wing flat slides |
| **State** | Armed = standing dot; live = trace running; error = SCAN FAIL, shutter closes | Armed = empty green screen; error = screen full of snow | Armed = amber lamp; error = blown fuse | Armed = curtain closed; error = the candle goes out |
| **Sound** | Chip square wave, fast arpeggio (the galvo's "song") | Chip sine + 60 Hz hum | Chip pulse + relay clacks | Chip triangle + crumpling paper |

## 5E arc in every function

Always the same sequence, dressed in the theme: **Excitement** = power on (the material wakes up: standing beam, CRT warming, test lamps, closed curtain) · **Entry** = splash with the logo drawn by the material + a jingle in the theme's timbre · **Engagement** = the work screen · **Exit** = the material goes dark (shutter, screen collapsing to a dot, cables unplug, curtain) with the closing chord · **Extension** = the Aprendiz remembers and comments on the next launch.

## Salvage from rounds 1 and 2 (reusable technique, UI discarded)

- Cracktro splash with a synthesized Web Audio jingle (zero media): approved, becomes the Entry standard.
- WebGL fragment shader for physical material (raymarching, refraction, dispersion, absorption): kept for FÓSFORO (the CRT glass) and PATCHBAY (the brass).
- Face in CSS 3D with the same matrix as the shader: kept for any screen with a 3D object.
- `.wmz` reader: discarded (vote). The reader lives only in git (`design/rodada2/wmzparse.py`, commit `6fe8b23`).
- Lee/Rosco gels as a physical filter: kept for PAPER THEATER (a real gel in front of the cutout).
- Vote inside the prototype with `db`: the standard for every round.

## Round 5 · the LASER theme became a complete device

- The LASER theme now has its own design system (`design/laser/tokens.css`, `SISTEMA.md`, `sistema.html`): the matter is the brushed aluminium of the case, the natural-aluminium optical bench, the beam in haze and the aqua-green OLED. Everything that is 2D UI (HUD, panels, speech balloon) is screen printing on the object.
- Pino 3D is the mascot-menu of every theme from here on: a real cable plugged into a port of each theme's device (on LASER, DMX OUT).
- The SolidWorks camera and the key/MIDI bindings are cross-cutting: they hold for FÓSFORO, PATCHBAY and PAPER THEATER with no change.
