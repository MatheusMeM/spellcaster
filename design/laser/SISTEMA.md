# Spellcaster LASER — design system

System of the laser and ILDA tool. This is not the Feiticaria Industrial design system: it applies only here, inside the laser module. Source of truth for the values: `tokens.css` in this folder. This document says what each token means and when to use it.

---

## 1. Principle

The program **is** the photorealistic 3D model of the RGB 10 W projector itself. It is not a window with a 3D viewport inside: the interface is the device.

- **Rear of the device = menu.** Every control lives in a connector, button, switch or display that exists on the real equipment. Clicking the rear panel is operating the rear panel.
- **Lid open = preferences.** Fine adjustment, calibration, kpps, scan limits, ILDA in/out: things that on a real laser you touch with the lid off.
- **The wall is the output.** What leaves the device appears projected on the wall of the room, in haze. An ILDA preview is not a UI rectangle: it is a beam.
- Nothing appears on screen out of software convenience. If it has no function on the device, it does not exist.

Practical consequence: **function before UI**. Every new element has to answer "which part of the laser is this and what does it do". If the answer is "it is a card", cut it.

## 2. Single material

One material, one accent, two traffic lights.

- **Black anodized aluminium** — the whole body, every panel, every background. `--bg:#050608`, `--panel:#07090C`, edges `--line:#1A2026` and `--line-2:#2A333C`. Matte surface, no decorative gradient, no glass on top of the UI.
- **Beam in haze** — the only light source of the scene. Every glow in the system comes from the beam, never from a card shadow.
- **Green accent of the beam `#38FF5C` (`--las`)** — active, LIVE, armed, selected, current numeric value. Green on the screen means "this is emitting or ready to emit".
- **Amber `#FFB000` (`--amb`)** — warning, pending action, LEARN in progress, echoed CLI. Amber is never a final state.
- **Red `#FF2A1A` (`--red`)** — danger, disarmed, e-stop, interlock open, SCAN FAIL.
- **OLED `#9FF5D0` (`--oled`)** — and only that — the little display on the rear panel. No other element uses this color; it identifies "this is the physical screen of the device".
- **Blue `#3A6BFF` (`--blue`)** — reserved for external ILDA/DMX (a signal coming from outside). Rare use; it is not a second accent.

Text: `--fg:#E6EBEF` for reading, `--dim:#7C8791` for label and metadata.

## 3. Tokens (`tokens.css`)

Read the file, do not memorise values. Summary of the contract:

**Color** — `--bg --panel --fg --dim --line --line-2 --las --amb --red --oled --blue --pino-on`.

**Typography** — `--font-disp: Michroma` (title, title HUD, `h1/h2/h3`, save button) and `--font-mono: Share Tech Mono` (everything else: label, value, CLI, table, button). Two families, no third.

**Scale** — `--fs-0:10px --fs-1:11px --fs-2:12px --fs-3:13px --fs-4:15px --fs-5:18px --fs-6:34px`. 10 and 11 are for panel label and warning HUD; 12-13 is the body of the device UI; 15 is text read from a distance; 34 only for a screen title.

**Spacing** — `--s1:4 --s2:8 --s3:12 --s4:16 --s6:24 --s8:32`. No improvised 6, 10, 18.

**Chamfer** — `--chamfer:14px` on a panel (`clip-path` cutting opposite corners) and `--chamfer-room:30px` in the room. A rectangular panel is wrong; it is a machined part.

**Glow** — `--glow` (accent at rest), `--glow-on` (active), `--glow-red` (danger). Glow belongs to the material, it is not an elevation `box-shadow`.

**Time** — `--t-fast:120ms` (button feedback), `--t-ui:220ms` (open/close panel, state change), `--t-cam:700ms` (camera movement). Three timings, no fourth.

## 4. Components

**HUD `.hud`** — text over the scene, `pointer-events:none`, `text-shadow:0 0 8px currentColor`. Variants: `.title` (Michroma, text color), `.cli` (amber, echoes the equivalent `spell` command), `.warn` (10px, amber), `.danger` (red). A HUD never receives a click — what receives a click is the part of the device.

**Laser button `.lb`** — accent outline, near-black background, fill only when active.
- default: green border and text, `--glow`;
- `.on`: filled `rgba(56,255,92,.18)`, `--glow-on` — the on state is visible from a distance;
- `.red` / `.red.on`: danger and danger triggered;
- `.amb`: warning/pending;
- `:disabled`: `opacity:.35`, no glow, `cursor:not-allowed` — disabled does not glow;
- `:focus-visible`: white 2px outline, 2px offset;
- `small` inside the button = shortcut or unit, `opacity:.6`.

**Panel `.panel`** — opaque (never translucent), accent border, 14px chamfer, `z-index:5`. Fixed structure: `h3` (Michroma) + `.sub` (11px, dim) + `.row` lines on the grid **label · control · value** (`96px 1fr 58px`), value aligned right, green, `tabular-nums`. `.x` closes at the top right corner. `.btns` groups smaller `.lb` at the panel footer. `pre` inside the panel = CLI block.

**Tooltip `.tip`** — solid green background, black text, no border, no arrow, 11px, `display:none` until hover. It is a bench label, not a balloon.

**Pino's balloon `.bal`** — literal Win98: `#FFFFE1`, 1px black border, hard shadow `2px 2px 0 #000`, Tahoma 12px, arrow in `::after`, `x` in the corner. It is the only "software" element of the scene, and that is deliberate — Pino is the menu mascot, it is not part of the device. Option list in `ul/li` with hover `#0A246A` inverted, `small` for the shortcut. Short voice.

**Bindings table** — inside `.panel`, `td.k` (action, text color), `td.b` (current binding, dim, `nowrap`) and an amber `.lb.learn` with `animation:pulse 1s infinite`. While it pulses, the next keyboard or MIDI event is captured. `prefers-reduced-motion` turns the pulse off — the button stays amber.

**Room `.room`** — pure black background `#000`, `overflow:hidden`, **non-rectangular** `clip-path`: a 30px chamfer on the four corners and a trapezoidal cutout at the center of the base (the mark of the device on the floor). Everything that is scene lives inside it.

## 5. States of the device

Four states, and nothing between them. The state is read simultaneously in the beam, in the OLED and in the emission LED.

| State | Beam | OLED (`--oled`) | Emission LED | UI |
|---|---|---|---|---|
| **OFF** | absent | dark | dark | panels `disabled`; dark room; only the power switch responds |
| **STANDBY** | absent | `STANDBY / INTERLOCK OK` | amber blinking slowly | controls editable, ARM available; amber HUD "no emission" |
| **SCAN FAIL** | cut immediately | `SCAN FAIL` inverted | steady red | `.lb.red.on` on ARM, all output locked until reset; Pino's balloon explains |
| **LIVE** | visible, green/RGB in the haze | `LIVE · 30 kpps` | steady green | green HUD, CLI echoing, e-stop always reachable |

Rules: never go from OFF straight to LIVE; SCAN FAIL is only left by an explicit reset; any loss of interlock falls to SCAN FAIL, not to STANDBY.

## 6. Camera — SolidWorks standard

The orbit is the SolidWorks one, with no invention:

- **MMB drag** — orbits around the **clicked point** (not around the center of the scene).
- **Ctrl + MMB** — pan.
- **Shift + MMB** — zoom.
- **Wheel** — zoom at the cursor, **inverted direction by default**, with a toggle in the preferences.
- **Arrows** — orbit 15°; **Shift + arrows** — 90°; **Ctrl + arrows** — pan.
- **F** — frames the selection (or the scene, if nothing is selected).
- **Ctrl + 1..7** — standard views (front, rear, left, right, top, bottom, isometric).
- **Dragging with the left button on empty space** also orbits — whoever has no middle button is not left out.

Every camera movement uses `--t-cam` (700 ms) with ease-out. No infinite inertia, no "floating".

## 7. Key binding

Any action of the system becomes a key **or** a MIDI event, in and out.

- A **LEARN** button per table row; while it pulses, it captures the next event (key, note on, CC).
- **MIDI out** exists: the state of the control goes back to the surface (the pad LED lights up when ARM is on). A binding is bidirectional by default.
- Persistence in `localStorage`, key **`sc-laser-bind`**, JSON `{ action: {key, midi} }`.
- An action without a binding is valid; a binding without an action does not exist.
- Every bindable action has an equivalent CLI — it is the same verb as the registry.

## 8. Movement

Splash, once, at startup:

1. The camera focus **leaves the static point** where it was and **goes to the wall** — the output — ignoring the laser and its rear panel. Room in haze, device out of frame (or out of focus in the foreground).
2. The laser **outlines** `SPELLCASTER LASER` on the wall, in a dynamic, continuous movement — no ILDA, no show, only the outline.
3. The letters are **revealed** as the beam passes and **stay** on the wall.
4. They all **glow** together when the outline closes.
5. Only then does the camera **fly to the rear panel** (the menu) — or to the open lid (preferences), `--t-cam`.
6. The laser **goes dark** and the room light **drops** a little. From here on, interactivity.

Outside the splash: panel transition in `--t-ui`, button feedback in `--t-fast`. `prefers-reduced-motion` cuts the splash to the final frame and removes pulses.

## 9. Sound

- **Jingle** of square wave in the splash — short, chiptune, no sample.
- **Blip** when each letter is revealed and when a LEARN is captured.
- **Click** of a dry physical button on `.lb`.
- **Descending chord** when disarming / entering SCAN FAIL.
Sound is confirmation of a physical action, never a soundtrack. Mute is a default that is respected and persisted.

## 10. Text

**Pino's** voice: short, technical, dry humour. One-line sentence. No "Oops!", no double exclamation mark, no tutorial.

- Good: `Interlock open. No interlock, no beam.`
- Bad: `Oops! Looks like something went wrong with your interlock :(`

**CLI always visible.** Every control shows the equivalent command in a `.cli` HUD or in a `pre` inside the panel:

```
spell ilda play show.ild --kpps 30
```

Whoever learns the GUI learns the CLI for free. Names on the screen = names in the registry.

## 11. Accessibility

- **Visible focus** is mandatory: `:focus-visible` with a white 2px outline and 2px offset on every control, including inside the 3D scene. Tab order follows the reading of the panel.
- **`prefers-reduced-motion`**: no animated splash, no LEARN pulse, no panel transition; all the information remains present through color and text.
- **Contrast** — approved pairs over `--bg`/`--panel`: `--fg`, `--las`, `--amb`, `--oled` and `--dim` (only for a label, never for critical information). `--red` over black is approved for icon and border; long text in red, no. State is **never** communicated by color alone: always color + label (LIVE, STANDBY, SCAN FAIL) + position.
- Minimum click target of 24px in the scene, even when the modelled part is smaller.

## 12. What NOT to do

- **Boxy software** — a grid of cards, a sidebar, an app title bar. If it looks like a dashboard, it is wrong.
- **Transparent panel** — glass, blur, translucency in the UI. The only material that transmits light is the haze. A panel is aluminium: opaque.
- **Element without a function** — two interlocks, a decorative fuse, an LED that indicates nothing, a connector that connects nothing. One interlock. No fuse.
- **Cartoon** — thick stroke, pastel color, rounded icon, cute mascot. Pino is dry Win98, it is not a critter.
- A third font, a fourth accent color, a spacing value outside the scale, a timing outside the three.
- Glow as card elevation. Glow is beam.
- Color alone carrying state.
