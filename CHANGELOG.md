# Changelog

One entry per release. Dates in ISO. A `v*` tag on `main` fires the `build` workflow, which
attaches the Windows onedir, the Lite tarball and the `spellcore` binaries to the GitHub Release.

## v0.1.0 — 2026-09-10

First numbered release. It brings together the Rust core (R0, R1, R3, R4, R7, R8), the `serve` bus,
the `spellgui/web` pages and the program's main page: the 3D laser projector.

### The program is the projector (`spellgui/web/laser3d/`)

- PBR model of the 10 W projector: chassis with hinge, knurled knob, XLR-3 DMX, ARMED LED,
  fan as a texture, measured labels, firmware printed on the board, Feitiçaria Industrial vector
  brand (`brand/`), no USB.
- Interior: aluminium optical bench with M4 holes at 12.5 mm, X/Y galvo block in the 6215H
  standard, boards with soldered components and standoffs, PSU, DAC, cables on a real route,
  cable ties, ILDA twisted pair and earth wire. Nothing crosses the beam.
- SolidWorks-style camera, one law per view: SHOW free, REAR fixed, INSIDE restricted;
  critically damped spring; `Z`/`Shift+Z`, arrows, `F`; source manual in
  `design/FUNCOES/camera-solidworks.md`. Knob with the TouchDesigner gesture.
- HUD with title and live facts only (ENGINE, DAC/feed, sACN/Art-Net inputs, MIDI) with LEDs;
  drawer on the right (`Tab`) with the LASER, DMX, NET, INTERLOCK, BINDINGS, VIDEO and INFO tabs
  replacing the floating panel.
- Pino attached to the camera in the left corner, DMX cable with a Verlet rope (`rope.js`), balloon
  on the right, dismiss. It speaks only on click.
- VIDEO tab shaped like a game video menu: LOW/MEDIUM/HIGH/ULTRA presets,
  27 options actually wired into the render (scale, FOV, fps cap, FXAA/MSAA 4×, shadows and
  filter, anisotropy, reflections, bloom, tone mapping, exposure, wall resolution, trail,
  halo, beams, dust, haze, Pino rope, motion), PERFORMANCE block read from `renderer.info`,
  persistence in `localStorage`. Single table in `video.js`.
- Laser wall in a `WebGLRenderTarget` with `LineSegments` and time-based fading, replacing the 2D
  canvas with `shadowBlur`: 2.9 → 45–60 fps with a dense `.ild`. `.ild` parsing with
  `Uint8Array` and a pre-sized vector (142 → 6 ms).
- Key and MIDI bindings with learn, reset and manifest (`bind.js`); input map and
  interlock in the drawer.
- `bench.html`: grid of model views without `app.js`, to check each part.
- `<!doctype html>` and `<meta charset="utf-8">` on every page; `charset.test.js` watches over it.

### Engine, CLI and protocols (`spellcore/`)

- `netscan`: Ether Dream beacon listened for on each board's IP (the `bind` on `0.0.0.0:7654`
  fails on Windows with Ether Dream Sitter open), loopback included; with no beacon, the DAC is
  found by TCP 7765 status on the ARP neighbours. `laser_dacs` returns `via: beacon|tcp`.
- DMX input (sACN and Art-Net), keyframe recording (`rec_arm`/`rec_state`) and `.ild` import
  as a track.
- MIDI input and a key → command map in the `.spell` (`midi_*`), with a mapping page.
- Loop as a transport state over the In-Out range (`loop_set`).
- `serve`: static HTTP with `charset=utf-8` and font and media mime types, JSON-RPC WebSocket,
  binary DMX monitor, streamable MCP at `/mcp`.
- `spellcaster.exe` (crate `gui`, `tao` + `wry`/WebView2): native window with the bus in process,
  opening on the 3D laser page.
- Registry: a single name for the show path and the track name, a description on every argument,
  `spellcore commands <name>`. The README command table regenerated from the registry (59).

### Web pages (`spellgui/web/`)

- `nav.js`: page bar for TIMELINE, PATCHBAY, THEATER, FACE, LASER and HELP
  (`Shift+1`…`Shift+6`), ENGINE/OFFLINE indicator, editable show name.
- `help.html`: shortcuts from `design/SHORTCUTS.md` and the live registry, one form per command.
- Timeline: the missing shortcuts, local undo, In-Out that does not collapse, mouse wheel scrolls
  the tracks, 2D previz in the bottom strip (`viewer.js`: DMX, ILDA frame at `t`, patch plan).
- Patchbay: create a node from search, edit `cfg` in the Inspector, plain wheel scrolls with `ymax`.
- three.js r128 and the Michroma and Share Tech Mono fonts vendored: it runs without a network.

### Design and documentation

- `design/FUNCOES/`: timeline as a DAW (Ableton, Resolve), DAW interface, source digests,
  SolidWorks camera.
- ponytail audit of the workstreams: references fixed, tautological assertions removed,
  license and cost of the window crates in the README.
- `MANUAL.md`: usage and capability manual for the operator. `CHANGELOG.md`: this file.
- Whole project in English: UI, Pino lines, CLI/registry/MCP descriptions, docs, comments.
