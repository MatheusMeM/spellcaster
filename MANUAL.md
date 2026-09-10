# Operator manual — Spellcaster

The program is the device. The main screen is the 3D laser projector
(`spellgui/web/laser3d/app.html`): its rear panel is the menu, the open lid is the preferences,
and the back wall shows what goes out through the laser. The other pages (timeline, patchbay,
theater, face, midi, help) and the `spellcore` CLI make up the rest. This manual only describes what
exists in the code of this tree.

## Opening the program

**Windows, own window.** `spellcaster.exe` (crate `spellcore/gui`, built from source with
`cargo build --release -p gui`) starts the bus in process, on a free port on `127.0.0.1`, and
opens a window straight at `/spellgui/web/laser3d/app.html`. Closing the window kills the process and
the bus with it. Without WebView2 the window does not open. **From the release or from source**,
start the bus with `spellcore` (in the release, `spellcore-windows-x64.exe`) and open it in the
browser — `--dir` is the repository root, not `spellgui/web`, because the pages read
`../../design/tokens/`, `../../faces/` and `../../shows/`. The `Spellcaster.exe` from the USB-stick
zip is the Python prototype (`INSTALL.md` §1), not this page.

```
spellcaster.exe shows\medgrupo.spell
spellcore serve --port 8000 --dir . --show shows/medgrupo.spell
http://127.0.0.1:8000/spellgui/web/laser3d/app.html
```

**Opening shortcuts (URL hash).**

| URL | What it does |
|---|---|
| `app.html` | splash on the wall, then goes to the REAR view |
| `app.html#rear` | skips the splash, opens on the REAR view (menu), key switch disarmed |
| `app.html#inside` | skips the splash, arms the key switch and opens the INSIDE view |
| `app.html#laser` | skips the splash, arms the key switch and opens the SHOW view |

`#inside` and `#laser` arm the key switch in the page state, without sending `laser_open` to the
engine: to arm for real (open the DAC) use the key switch on the rear panel or the `S` key.

**No engine.** `app.html?offline=1` (or opening over `file://`) runs the whole page without a bus:
`bus.call` only echoes what it would do, nothing goes out on the network. The HUD writes
`ENGINE OFFLINE` and the drawer writes `engine offline` instead of the file list and the DACs. The
query comes before the hash: `app.html?offline=1#rear`. WebGL 2 is required only for MSAA 4× in the
VIDEO menu; without WebGL 2 that line falls back to FXAA on its own. `?perf=1` turns on the raw
counter in `window.__perf`.

## The screen

**Splash.** The galvo traces SPELLCASTER LASER on the wall, letter by letter, and at the end
everything glows and the camera flies to the rear panel: about 7.6 s. Any key or click skips it
(with a jingle).
**The device** is a 10 W projector in 3D: chassis, rear panel with ports, display, encoder,
key switch, power rocker, and the optical bench under the lid. Hovering lights up the part and shows
its label; clicking opens that part's tab in the drawer. Plate, fin, plug, PSU and fan are inert
parts: they do not light up and clicking does nothing.

**The three views.**

| View | Key | What it is | What it unlocks |
|---|---|---|---|
| REAR | `2` | the menu | ports, display, encoder, key switch, rocker, interlock |
| INSIDE | `3` | the preferences | lid opens; optical bench, R/G/B modules, shutter, galvos |
| SHOW | `1` | the free view | the visualizer: device, beam, haze and the wall |

The SHOW view only unlocks with the key switch armed; without the key switch the `1 SHOW` button is
disabled. Clicking a port (ILDA, DMX, NET) while in SHOW takes you to REAR; clicking a galvo, the
shutter or a colour module takes you to INSIDE.

**Key switch, power and interlock.** The POWER rocker (`P`) turns the device on: switched off, the
display goes dark and nothing else responds — not the key switch, not the interlock, not the encoder.
The key switch (`S`) arms emission and, with an engine, it is what sends `laser_open` to the chosen
DAC; disarming sends `laser_close`. The interlock (`I`) is an input: plug out = SCAN FAIL, shutter
closed, beam parked. The big ARMED LED on the chassis and the emission LED follow the four states:
dark with no power, red blinking when disarmed (the emission one blinks amber), solid red on SCAN
FAIL, solid green on LIVE. The NET port LEDs only light up with power.

**The HUD (top right).** One line per live fact, each with its own LED; a line with no fact does not
appear.

| Line | When it appears | LED |
|---|---|---|
| `ENGINE · rev N · <show>` or `ENGINE OFFLINE` | always | lit with an engine |
| `DAC <type> <host> · feed N` | with an engine | lit with the feed open; blinking amber while searching; red on error |
| `sACN · in U` / `ART-NET · in U` / `MIDI IN · in U` | one per input declared in `show.inputs` | lights up with a frame received in the last 2 s |
| `MIDI · <n> in · <n> out` | with MIDI connected | lit with at least one input |
| `<n> kpps · <n> pts · <n> fps` and `<file> · frame n/N · DMX <addr>` | always | — |
| `<n> fps · <n> ms · <n> draw calls` | with HUD COUNTER = YES | — |
| `OFF` / `STANDBY · DISARMED` / `SCAN FAIL` / `LIVE · ARMED` | always | follows the state |

Below the HUD, in amber, sits the echo of the last command the page sent, in the form
`spell <command> --arg value`.

**The rear panel display.** Seven pages — STATUS, SHOW, DMX, NET, TEMP/ILK, ENGINE, ERROR — with one
big line you can read from across the room. An alarm (SCAN FAIL, error) inverts the block; there is
no new colour. The encoder turns pages; pressing it enters edit mode and steps from field to field;
BACK leaves the field, or goes back one page. Fields per page: STATUS = kpps; DMX = address and
universe; NET = sACN, Art-Net, NDI, Spout; ERROR = clear. SHOW, TEMP/ILK and ENGINE are informative
only. An engine error lands on the ERROR page and in Pino's mouth; nothing locks up.

**Pino.** A DMX cable with five button-pins, attached to the camera in the bottom left corner,
connected by its cable to the device's DMX OUT. Clicking it opens the menu of screens; each pin is a
screen.

| Pin | Goes to |
|---|---|
| 1 · LASER | this page (ILDA IN, on the rear panel) |
| 2 · FÓSFORO | the NET port (the converter does not exist yet) |
| 3 · PATCHBAY | `patchbay.html` |
| 4 · PAPER THEATER | `teatro.html` |
| 5 · INFO | the drawer's INFO tab |
| RECORDER | `index.html` (the timeline) |
| CONSOLE | `face.html?face=quatro` |

The `–` button on the balloon (or the latch on Pino's body) sends Pino away: he goes down and the icon
left behind brings him back (address `pino.hide`). There is no cable between Pino and the device.

### The drawer (Tab)

Every menu and every setting lives in a drawer on the right. `Tab` opens and closes it, `Esc` closes
it, the handle on the edge works too; inside it `Tab` goes back to being the keyboard's. Seven tabs:

**LASER.** The `.ild` file and the optical path. ILDA IN: name and frame count of the file, plus
`CHOOSE .ILD`, `DEMO` and `PLAY/PAUSE`; SIZE from 30 % to 130 %; the list of `.ild` files on the
engine's disk (`laser_files`) with the size in kB, and clicking one of them plays the file on the DAC
and draws it on the wall. X/Y GALVOS: KPPS 5,000 to 40,000 (step 500), BUFFER 1 to 8, SPEED 0.40× to
1.60×. R/G/B MODULES (638 / 520 / 445 nm): limit 0–100 % and γ curve 0.50–2.50 per colour, with the
graph of the three curves. SHUTTER: OPEN or CLOSED — it closes with no key switch or no interlock.
ILDA OUT is informative (daisy-chained: none).

**DMX.** Address 1–512, universe 1–512, the 16-channel mode listed, and the last reading received on
the bus (`U<universe> ch<address> = <value>`) or `no frame received`. **NET.** Four state buttons
labelled ON/OFF: NDI, SPOUT, ART-NET, SACN. Below them, the DAC: type, host, open feed, the
`spell laser_open` line, the editable HOST field and the `FIND DACS` button (which calls
`laser_dacs` with a 2 s deadline). Each DAC found becomes a `TYPE host` button; clicking picks who
receives the beam.

**INTERLOCK.** The interlock is an input, at address `laser/1/interlock`. The tab shows the current
state and where to map whatever triggers it: KEY (learn), MIDI (learn), OSC (the address), ART-NET
(universe and channel, `≥ 128` = closed), MQTT (topic) and the CLI line. Only key and MIDI are
actually wired — the OSC, Art-Net and MQTT map is kept in the browser and the matching line is marked
"coming soon". The `SIMULATE OPENING` / `PUT THE PLUG BACK` button flips the state.

**BINDINGS.** A `MIDI:` button with the connection state, a `WHEEL: SOLIDWORKS / NORMAL` button
(mouse wheel direction), a `RESET` button and the table of every action: label, key, MIDI and `×` to
clear.

**VIDEO.** The visualizer's video menu — its own section below. **INFO.** What the program is, the
engine state (`on · rev N` or `offline`), the open feed and two CLI lines; `N` closes it.

## Camera

The map is the SolidWorks one; what changes per view is not the keyboard, it is the camera law.

| Gesture | SHOW (free) | REAR (fixed) | INSIDE (restricted) |
|---|---|---|---|
| drag middle button | orbits around the clicked point | — | orbits within the limits |
| `Ctrl` / `Shift` / `Alt` + middle | pan · zoom · roll | — | — |
| left drag on empty space | orbits | — | orbits within the limits |
| left drag on a part | belongs to the part, never to the camera | same | same |
| wheel | dolly towards the focal point | — | — |
| wheel over the encoder | turns the encoder | turns the encoder | — |
| arrows / `Shift`+arrows / `Ctrl`+arrows / `Alt`+arrows | 15° · 90° · pan · roll | — | — |
| `F`, `Ctrl+1..7`, `Z`, `Shift+Z` | as in SolidWorks | — | — |
| `1` `2` `3` | switches view | same | same |

`Z` moves away and `Shift+Z` moves closer — that is the order in the SolidWorks manual. `F` frames
the device and `Ctrl+1..7` are the standard views (front, rear, left, right, top, bottom,
isometric). All those keys work **only in the SHOW view**: in the other two the camera law refuses
the movement.
**REAR** is a fixed pose, computed from the rear panel normal and recomputed when the window is
resized. No dragging, no wheel zoom, no arrows. The only movement is a ±2° breathing that follows the
mouse and does not change the distance — it can be switched off at address `cam.breathe`.
**INSIDE** is an orbit locked to the centre of the optical bench: yaw ±60° from the entry pose,
pitch from 20° to 80°, distance locked. **SHOW** is the whole of SolidWorks, with limits: the camera
never enters the device, nor goes through the floor or the wall, and the pitch stops at ±85°;
sensitivity is proportional to distance.

**The encoder knob** uses the TouchDesigner gesture: press and drag the mouse up to increase, down to
decrease — one step every 6 px, or 24 px with `Shift` (fine adjustment). Releasing without moving
3 px counts as a click, which is OK. The mouse wheel over the knob also turns the encoder.
`WHEEL: SOLIDWORKS` (default, in the BINDINGS tab) inverts the zoom relative to the browser; the
state is kept in the browser.

## Keyboard and MIDI

Every action has a textual address, the same one for key and for MIDI. Default table, in the order of
the BINDINGS tab.

| Address | Label | Default key |
|---|---|---|
| `cam.show` | SHOW view | `1` |
| `cam.rear` | REAR view · menu | `2` |
| `cam.inside` | INSIDE view · preferences | `3` |
| `key.toggle` | key switch: arm | `S` |
| `lock.toggle` | interlock | `I` |
| `power.toggle` | power | `P` |
| `play.toggle` | play / pause | `Space` |
| `kpps.down` | kpps −1k | `[` |
| `kpps.up` | kpps +1k | `]` |
| `kpps` / `size` / `lim.r` / `lim.g` / `lim.b` | faders: kpps, size, limit for each colour | no default key (learn it in BINDINGS) |
| `file.open` | open .ild | `O` |
| `demo` | demo.ild | `D` |
| `net.ndi` / `net.spout` / `net.artnet` / `net.sacn` | network: NDI / SPOUT / ART-NET / SACN | no default key (learn it in BINDINGS) |
| `oled.up` / `oled.down` / `oled.ok` | OLED: encoder + / − / OK | no default key (learn it in BINDINGS) |
| `oled.back` | OLED: BACK | `Backspace` |
| `nfo` | info | `N` |
| `bind` | bindings | `B` |
| `drawer` | drawer: open and close | `Tab` |
| `esc` | closes the drawer | `Escape` |
| `dacs` / `midi.connect` / `bind.reset` | find DACs, connect MIDI, reset the bindings | no default key (learn it in BINDINGS) |
| `cam.reverse` / `cam.breathe` | wheel direction, REAR view parallax | no default key (learn it in BINDINGS) |
| `cam.rotL` / `cam.rotR` / `cam.rotU` / `cam.rotD` | camera: rotate 15° | `←` `→` `↑` `↓` |
| `cam.rot90L` / `cam.rot90R` / `cam.rot90U` / `cam.rot90D` | camera: rotate 90° | `Shift+←` `Shift+→` `Shift+↑` `Shift+↓` |
| `cam.panL` / `cam.panR` / `cam.panU` / `cam.panD` | camera: pan | `Ctrl+←` `Ctrl+→` `Ctrl+↑` `Ctrl+↓` |
| `cam.rollL` / `cam.rollR` | camera: roll | `Alt+←` / `Alt+→` |
| `cam.fit` | camera: fit | `F` |
| `cam.front` … `cam.iso` | standard views | `Ctrl+1` … `Ctrl+7` |
| `cam.zoomIn` | camera: zoom + | `Shift+Z` |
| `cam.zoomOut` | camera: zoom − | `Z` |
| `pino.hide` | Pino: leaves the screen / comes back | no default key (learn it in BINDINGS) |
| `video`, `video.preset`, `video.defaults`, `video.fullscreen` and one `video.<id>` per line of the VIDEO menu | video menu, preset, restore defaults, fullscreen and each option | no default key (learn it in BINDINGS) |

**Key learn.** Open BINDINGS (`B`) and click the button in the key column, on the action's row: it
writes `KEY…` and the next key becomes the binding. `Esc` cancels. One key serves one action only:
on learning, it is taken away from whoever had it.
**MIDI learn.** Same path, MIDI column: the button writes `MIDI…` and the next message becomes the
binding. Before that you have to connect: the `MIDI:` button in the same tab (it uses the browser's
Web MIDI; without Web MIDI the button writes `no Web MIDI`). Note on, note off and control change are
accepted; the key stores the channel (`cc:1:7`, `note:1:60`). No SysEx, no NRPN, no MIDI clock. A `cc`
action (the faders) receives the 0–127 value as 0–1; a button action fires on note on or on a CC above
63. If the controller has an output, Spellcaster sends back the state of every mapped action.

The `×` on each row clears the key and the MIDI of that action; `RESET` puts everything back to the
factory default. All of it lives in the browser's `localStorage`: `sc-laser-bind` (bindings),
`sc-laser` (kpps and last file), `sc-laser-video` (video menu), `sc-laser-ilk` (interlock map),
`sc-laser-wheel` and `sc-laser-breathe` (camera). None of it goes into the `.spell`. With a text field
focused the device keyboard is switched off; with a fader focused, only the arrows and Home/End go to
the fader.

## Video

The VIDEO tab is a game menu: PRESET at the top, the option groups, the PERFORMANCE block and
RESTORE DEFAULTS at the bottom. Every line applies immediately — there is no APPLY button, no page
reload, not even for MSAA.

**Presets:** LOW, MEDIUM, HIGH, ULTRA and CUSTOM. The default is HIGH, or MEDIUM on a machine with
four cores or fewer. CUSTOM is not a choice: it is what the menu writes when some value leaves the
preset — putting the value back by hand returns to the preset name on its own. With
`prefers-reduced-motion` on in the system, the animations start switched off and the trail shorter.

The whole table comes out of `video.js`. Groups: SCREEN (how many pixels the program draws, and how
many times per second), QUALITY (aliasing, shadow and texture), POST-PROCESSING (what the composer
does after the scene), LASER (wall, trail, beams, haze) and SCENE (the device and what moves on its
own). The `FULLSCREEN` button sits at the end of the SCREEN group.

| Group | id | Label | Values | What it changes |
|---|---|---|---|---|
| SCREEN | `scale` | RESOLUTION SCALE | 50 / 75 / 100 / 150 / 200 % | multiplies the devicePixelRatio (2× ceiling): 200 % is true supersampling |
| SCREEN | `fov` | FIELD OF VIEW | 30° to 90° | the camera fov; the fixed views reframe from it |
| SCREEN | `fpsMax` | FPS CAP | 30 / 60 / 120 / UNLIMITED | skips the frame's work by the clock, without letting go of the rAF |
| SCREEN | `hudFps` | HUD COUNTER | YES / NO | adds the fps · ms · draw calls line to the HUD |
| QUALITY | `aa` | ANTI-ALIASING | OFF / FXAA / MSAA 4× | FXAA is a composer pass; MSAA swaps the composer target for a multisampled one (requires WebGL 2, otherwise it falls back to FXAA). The two do not add up |
| QUALITY | `shadows` | SHADOWS | OFF / 1024 / 2048 / 4096 | turns on the spot shadow and the map size |
| QUALITY | `shadowType` | SHADOW FILTER | BASIC / PCF / PCF SOFT / VSM | the shadow map filter; changing it recreates the map |
| QUALITY | `aniso` | ANISOTROPY | 1× to 16× | texture anisotropy, capped by the GPU maximum |
| QUALITY | `reflections` | ENVIRONMENT REFLECTIONS | YES / NO | turns the scene environment map on and off |
| QUALITY | `reflectionInt` | REFLECTION INTENSITY | 0 to 2× | multiplies the `envMapIntensity` each material already carries |
| POST-PROCESSING | `bloom` | BLOOM | YES / NO | turns on the bloom pass |
| POST-PROCESSING | `bloomStrength` | BLOOM STRENGTH | 0 to 2 | glow intensity |
| POST-PROCESSING | `bloomRadius` | BLOOM RADIUS | 0 to 1 | spread |
| POST-PROCESSING | `bloomThreshold` | BLOOM THRESHOLD | 0 to 1 | from which luminance a pixel glows |
| POST-PROCESSING | `bloomRes` | BLOOM RESOLUTION | ¼ / ½ / 1× of the screen | the size of the bloom target |
| POST-PROCESSING | `tone` | TONE MAPPING | NONE / LINEAR / REINHARD / CINEON / ACES | the renderer's tone curve (recompiles the materials) |
| POST-PROCESSING | `exposure` | EXPOSURE | 0.20 to 3.00 | tone mapping exposure |
| LASER | `wall` | WALL RESOLUTION | 512×320 / 1024×640 / 2048×1280 | the target where the trail lives; changing it clears the wall |
| LASER | `trail` | TRAIL PERSISTENCE | 0.30 to 0.95 | how much of the previous frame survives every 1/60 s |
| LASER | `halo` | STROKE HALO | 0 to 0.40 | halo opacity per point |
| LASER | `haloPx` | HALO SIZE | 4 to 16 px | size of the halo sprite |
| LASER | `beams` | EXTERNAL BEAMS | 40 / 80 / 160 / 320 | how many beam segments leave the aperture towards the wall |
| LASER | `dust` | BEAM DUST | YES / NO | particles inside the beam |
| LASER | `haze` | HAZE | 0 to 1 | beam gain in the air and cloud opacity |
| LASER | `puffs` | HAZE PUFFS | NONE / 14 / 28 | how many haze sprites stay visible |
| SCENE | `motion` | ANIMATIONS (FAN, LED) | YES / NO | fan spinning and LED blinking; off by default with `prefers-reduced-motion` |

**PERFORMANCE.** A read-only block, measured and not estimated, refreshed four times a second: fps
and ms/frame (half-second average), draw calls and triangles, geometries and textures, screen
resolution in pixels with the dpr, wall resolution, WebGL 1 or 2, MSAA active, maximum anisotropy and
the GPU renderer name. **RESTORE DEFAULTS** goes back to the machine's preset. Everything is kept in
`localStorage` (`sc-laser-video`); a value out of range or off the list is refused at the input, and
the line keeps the default.

## Laser

**Opening an `.ild`.** Three ways: `CHOOSE .ILD` (or `O`), dragging the file onto the page, or
clicking a file from the engine's list in the LASER tab. `D` goes back to `demo.ild`. ILDA formats 0,
1, 4 and 5 are read; format 2 (palette only) is skipped. A file brought in by hand only draws on the
wall: to go out on the DAC it has to be in `shows/`, on the engine's disk.

**kpps, size and limits.** KPPS from 5,000 to 40,000 (also `[` and `]`). The frame rate is
kpps ÷ points: below 25 fps the figure flickers, and above 32 kpps the galvo cannot keep up and the
corners turn into curves — Pino warns you in both cases. SIZE goes to `laser_param geo/scale`; the R,
G and B limits go to `laser_param limit/r|g|b`. The γ curve and the BUFFER are local: there is no
`laser_param` for them today. The kpps is local too: it is an opening argument, so it only reaches
the DAC on the next `laser_open` — disarming and arming the key switch applies it.

**Finding the DAC.** `FIND DACS` in the NET tab calls `laser_dacs` with 2 s. The Ether Dream
announces a 36-byte UDP beacon on `255.255.255.255:7654`, at 1 Hz, and Spellcaster listens on the IP
of each network board, not just on the wildcard. If no beacon arrives within half the deadline, it
asks for the status over TCP on port 7765 from the neighbours in the ARP table — an Ether Dream
answers 22 bytes when it accepts the connection. Each hit carries `via: beacon` or `via: tcp`; a hit
found by TCP carries neither `buffer` nor `max_pps`, because only the beacon carries those fields.
IDN is looked for by scan.

**Ether Dream Sitter open.** On Windows, with the Sitter open, the `bind` on `0.0.0.0:7654` is
refused with `WSAEACCES` (10013), even with `SO_REUSEADDR`. In that case the beacon does not arrive:
close the Sitter, or type the DAC's IP into the HOST field of the NET tab and arm the key switch —
opening by host does not depend on discovery.

**Safety.** `laser_open` receives `safety` (`min_size`, `max_intensity`, `zone`) and it can **never
be switched off**: there is no command to remove it. The interlock acts on top of it, through
`laser_param shutter` (1 closes, 0 opens), and the shutter also closes by itself with no key switch.
With no DAC chosen there is nothing to arm: with no engine the key switch arms only the model on the
screen.
With the feed open, the page asks for `laser_stats` once a second — points sent, dropped, errors,
jitter, cpu and the current safety — and shows the result on the ENGINE page of the display.

## DMX and network

**Output.** The show declares its outputs in `outputs`, and the player opens them: sACN, Art-Net and
OSC. Universes are numbered from 1; Art-Net converts to port-address internally.

**Input.** The show declares `inputs` (`{"type": "sacn", "universe": 1}`, `artnet`, `midi`). The bus
publishes the input frames as topic 2, and that is what lights up the LEDs of the input lines in the
HUD — only with a frame actually received. The DMX tab shows the last value read at the chosen
address.

**NDI and Spout.** The four buttons in the NET tab mark the source on the page and light up the NET
port LEDs. NDI → ILDA and Spout → ILDA are FÓSFORO, the converter that does not exist yet: turning on
NDI or Spout today converts nothing, and Pino says so.

**`net`.** The network scan is in the CLI and in the registry (`spellcore net --timeout 2 [--json]`):
interfaces, Art-Net, sACN and Ether Dream nodes, plus the configuration suggestions.

## The other pages

They all live in `spellgui/web` and are served by the same `spellcore serve`. Five of them carry the
`nav.js` bar at the top, with `Shift+1` to `Shift+6` and the `?` key for help. The 3D laser page does
**not** load that bar: its navigation is Pino.

- **TIMELINE** (`index.html`, `Shift+1`) — the canvas timeline: tracks, keyframes, cues, loop in the
  engine, recording (`R`) and 2D previz in the bottom strip (`Alt+M`).
- **PATCHBAY** (`patchbay.html`, `Shift+2`) — the graph editor: create a node (`Shift+A`), wire,
  group and undo, all through `show_patch`.
- **THEATER** (`teatro.html`, `Shift+3`) — DMX patch, scenes and cues, with GO and a clickable set.
- **FACE** (`face.html?face=quatro`, `Shift+4`) — the console: the show's big buttons, in kiosk mode.
  `Enter` = cue GO, `Esc` held = blackout, `Shift+F` = fullscreen.
- **LASER** (`Shift+5`) — the bar's tab points to `laser.html`, the 2D ILDA player page (DAC, kpps,
  file, play/stop, sliders, shutter, stats), not to the 3D page.
- **MIDI** (`midi.html`) — ports, open and close, the `key → command` table from the `.spell`, LEARN
  and the last key live. It has no tab on the bar; open it by URL.
- **HELP** (`help.html`, `Shift+6`, or `?` on any page with the bar) — the shortcut table from
  `design/SHORTCUTS.md` and the live registry, with one form per command to run it.

Details of each file: `spellgui/web/README.md`. Key map for the whole product, with the state column
(done / missing / n.a.): `design/SHORTCUTS.md`.

## CLI and MCP

One process touches the hardware; every page and every AI session talks to it.
```
spellcore play <show.spell> [--loop] [--osc-port N]   plays the show (sACN / Art-Net as per "outputs")
spellcore net [--json] [--timeout N]                  network scan + suggestions
spellcore commands                                    the registry as JSON (name, doc, schema)
spellcore serve [--port N] [--dir D] [--show S]       HTTP + WebSocket + MCP bus
spellcore mcp                                         MCP server over stdio
spellcore mcp install --target desktop|code [--yes]   registers the server with Claude
```

`serve` only opens on `127.0.0.1`, with no authentication and no TLS — and that is why it does not
accept `--host`. Port `0` picks a free one and the line `serve http://127.0.0.1:<port>` goes to
stderr. `--show` loads the show with the player stopped at `t=0`; the `resume` command releases it.
**MCP.** `spellcore mcp` starts the server over stdio; `mcp install` writes the entry in Claude
Desktop (`--target desktop`) or a `.mcp.json` in the current directory (`--target code`), always
showing what it is about to write, with a `.bak` backup, and only after confirming in the console.
The same MCP is also served at `/mcp` by `serve`. An AI session sees **one tool per registry
command** — 59 of them today, including `load`, `show_get`, `resume`, `pause`, `locate`, `cue_go`,
the editing ones (`track_add`, `key_set`, `cue_set`, `patch_add`, `show_patch`), the `midi_*`, `net`,
`play_show` and the `laser_*` (`laser_dacs`, `laser_open`, `laser_play`, `laser_stop`, `laser_close`,
`laser_param`, `laser_stats`, `laser_files`, `clip_frame`) — plus four resources: `spell://show`,
`spell://commands`, `spell://graph` and `spell://face`. With that, the session scans the network,
patches, edits the timeline and hits play without a GUI. Full table of commands and parameters:
`spellcore/README.md`.

## Capabilities and limits

| Area | State |
|---|---|
| Engine, protocols (sACN, Art-Net, OSC), `net` | done (R0) |
| Timeline, cues, `.spell`, `fx` in Rhai, Graph, player with OSC transport | done (R1) |
| Pixel mapping (100,000 px) | done (R3), but the crate stands alone: wiring the frame source to the player waits on media |
| Multi-feed laser: Ether Dream, IDN, safety, 4 feeds | done (R4) |
| MCP over stdio + `/mcp` in `serve` | done (R7) |
| Packaging: Windows onedir, Linux, static Pi, CI with the bench as a gate | done (R8) |
| GUI: own window (`spellcaster.exe`) opening the 3D laser page, wired to the registry | base done (R5); panels, Theme and Face missing |
| Media: GStreamer, NDI, RTSP, Spout | does not exist (R2) — blocked by the SDKs not being installed |
| FÓSFORO (NDI/Spout → ILDA) | only the `laser::trace` core (bitmap → outline → ILDA, 4.4 ms with 20 objects). The R2 NDI source is missing; the NDI/SPOUT buttons in the NET tab convert nothing |
| Godot previz | does not exist (R6) — Godot not installed |
| Face and Graph editors, Agent panel | do not exist (R9) — they depend on the GUI |
| Helios (USB DAC) | stub in every build, not a limitation of the static binary |
| `face_patch`, `graph_patch`, `theme_set` | do not exist: the `.spell` does not serialize Theme or Graph yet. What edits is `show_patch` |
| Export / render of the show | does not exist in the registry; `Ctrl+M` is reserved |

**On the 3D page, what still does not act on the engine.** BUFFER and the γ curves of the modules are
local (there is no `laser_param` for them, and BUFFER today does not even change the drawing on the
wall); the UNIVERSE in the DMX tab only shows up on the display; the interlock's OSC, Art-Net and MQTT
map is kept in the browser and does not become a command — only key and MIDI trigger the input; ILDA
OUT (daisy chain) is informative.
