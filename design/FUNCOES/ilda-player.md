# ILDA player — `spell ilda play` (LASER theme)

Play an `.ild` on a DAC (EtherDream today, `spellcaster/protocols/ilda/etherdream.py`) with transport, loop, calibration and safety. The Python module already has `Point(x,y,r,g,b,blank)`, `optimize(dwell, blank_gap, max_step, angle)` and `safety(min_size, max_intensity, zone)` in `frame.py`; this file says how that becomes an interface.

| Item | Who solved it best | Why |
|---|---|---|
| Objects and verbs | MadMapper, with the player-as-device of Capture | The MadMapper laser output and laser surface have the complete, named set of physical parameters; Capture shows that the player is a patched device, with its own frame rate and two DMX channels |
| States | TouchDesigner | `debugchan` gives the state of each emitted point; `PlayBarPending` gives the "changed but not applied" state; and its omission (no safety state at all) shows what not to repeat |
| Screen zones | Blender, with MadMapper's "Arm your laser" | Six-button transport in a single `row`, with a bigger Pause while playing; the arming sits in the top right corner of the preview, not in a menu |
| Shortcuts | Blender, with the panel table of TouchDesigner | "Frames" keymap valid in every window; `I`/`Backspace` over the field; `Shift+Space` as the only play in performance |
| File | MadMapper | It declares that ILDA does not carry PPS, separates "record at fixed FPS" from "record as stream", and stores everything about the laser in readable name-value pairs |

## 1. Objects and verbs

**Clip** — an `.ild` (formats 0, 1, 2, 4, 5 already in `ild.py`). Verbs: add, remove, rename, duplicate. A clip does not know its PPS: the format does not carry a playback rate, "the hardware treats the file as a stream of points: a frame of 500 points lasts less than one of 700" [fontes/madmapper.md § Laser]. Therefore frame rate and PPS belong to the **player and the output**, never to the file. Capture confirms it from the other side: `ILDAFrameRate` is a property of the `MediaPlayer`, not of the media [fontes/capture.md § Patch and universes].

**Frame** and **Point** — what the clip contains. Verbs on the frame: go to, mark In/Out, loop. No verb on an isolated point in the GUI; a point is diagnostic data (see States).

**Player** — the transport. Verbs: play, pause, stop, locate, loop over the In/Out range, speed, requested frame rate. It is a **patched device**: it responds to two DMX channels (play/pause/stop/replay control and media selection) and to a playlist of 256 entries [fontes/capture.md § Patch and universes]. That is what makes `spell cue` fire a laser clip the way it fires a dimmer.

**Laser output** — the DAC. Parameters, with the names MadMapper writes in `customSettings` and TouchDesigner exposes in the Laser CHOP, grouped by `/` in the label (cross-cutting rule 1):

| Group/Parameter | Unit | Origin | Note |
|---|---|---|---|
| Device/Type | enum etherdream, helios, shownet | TD `type` | menu auto-populated with what is on the network [fontes/touchdesigner.md § Laser and NDI] |
| Device/Address | IP | TD `netaddress` | discovery is a known problem; list + field |
| Device/Queue | samples, frames or seconds | TD `queuetime` + `queueunits` | "it is usually useful to reduce it when sending few points" |
| Device/PPS | points/s | MM `Device/PPS`, TD `outputrate` | physical clamp of the galvo, useful show range kept separate (`min/max` vs `norm`, rule 1) |
| ILDA/Requested FPS | Hz | MM `ILDA/Desired FPS` | what the operator wants |
| ILDA/Actual FPS | Hz, read-only | MM `ILDA/ILDA FPS` | `= PPS / points per frame`; below 35 it flickers, above 45 the sweep is not visible |
| ILDA/Points per frame | count, read-only | MM `ILDA/Point Count` | the third variable of the formula, always next to the other two |
| ILDA/Update | enum all-drawn, every-frame | TD `updatemethod` | the parameter that explains flicker from too many points |
| Blanking/Colored step, Blanked step | distance per sample | TD `stepsize`, `bstepsize` | our `max_step` |
| Blanking/Corner hold min, max | samples | TD `mincornerhold`, `maxcornerhold` | interpolated by the angle; our `dwell` + `angle` |
| Blanking/Pre-on, Post-on, Pre-off, Post-off | ms | TD `preblankoff`, `postblankoff`, `preblankon`, `postblankon` | our `blank_gap`; MM condenses it into `Blank Delay/Smth/Curve` |
| Blanking/Start repeat, end repeat | points | MM `Start Repeat`, `End Repeat` | "the software never knows where the beam really is" |
| Blanking/In fade, out fade | 0..1 | MM `In Fade`, `Out Fade` | avoids the hot point at the start and the end of the path |
| Color/Scale R G B | 0..1 | TD `redscale`…, MM `Color Levels` | |
| Color/Minimum voltage R G B | 0..1 | MM `Min Voltage` | "so a dark gray does not turn red" |
| Color/Shift R G B | **ILDA points** | MM `Time Shift` | the right unit: the delay is in samples, not wall clock; TD uses ms (`colordelay`) and loses |
| Color/Curve R G B | curve | MM `Response` | |
| Safety/Scan area | 0..1 | MM `ILDA/Scan Area` | our `zone`; "listen to the scanner: the sound must be smooth" |
| Safety/Masks | polygons with opacity and inversion | MM `Masks` | applies to the test cursor as well |
| Safety/Minimum size, Maximum intensity | | our `safety()` | TD has neither of the two; inheriting the omission is forbidden |
| Geometry/Scale X Y, Rotation, Flip, Swap XY | | TD `xscale yscale rotate swap`, MM `scale rotation flip` | |
| Test/Test pattern, Level | | MM `Test Pattern` | |

Each of those lines is a registry `@command` with a declared type (rule 3): PPS is a value, Test pattern is a toggle, Arm is a trigger with confirmation.

**Calibration** — the existing `calib_*`; MadMapper does it by photographing what the laser draws with a camera and warping it to the laser's point of view [fontes/madmapper.md § Objects and verbs]. It stays a verb of the Outputs panel, not of the player.

**Shutter** — its own verb, with its own state. TouchDesigner does implicit blanking when the three color scales are zero, "a state that appears nowhere in the UI" [fontes/touchdesigner.md § What NOT to copy]. Here the shutter is named. Holding the key is a momentary shutter, by the same mechanism as Resolume's boolean `connect`: "analogous to holding the mouse down" [fontes/resolume.md § What to copy].

## 2. States

One accent, mixed into the background of the field (rule 4). The states the player must show, in order of severity:

1. **Not armed / armed.** No equivalent in Resolume ("as soon as a Lumiverse exists, it sends") or TouchDesigner. MadMapper has "Arm your laser!" in the top right corner of the previews [fontes/madmapper.md § States]. Here: `Ctrl+Shift+Enter` arms, and it is the only state that changes the color of the whole preview frame.
2. **Shutter closed / open.** Named, with its own indicator, never inferred from zero color.
3. **DAC: no device, searching, connected, error.** The three universe states of Capture, `(no)`, `Searching..`, `(auto)`, plus error [fontes/capture.md § States and messages]; "activity does not guarantee operation" applies here too.
4. **Transport: stopped, playing, paused, looping, and pending.** The four from TouchDesigner (`PlayBarOff`, `On`, `Reset`, `Pending` red) [fontes/touchdesigner.md § States]. Pending = PPS or queue changed and the DAC has not applied it yet.
5. **The galvo can't keep up.** Actual FPS below 35 Hz, or points per frame above `PPS / requested FPS`. It is a `warning`, not an `error`: the laser keeps going, it flickers. The Aprendiz speaks up here (`aprendiz-menu.md`).
6. **Outside the safe area.** Points cut by `zone` or a mask: count per frame, `warning` if greater than zero.

Per-point diagnostics: the engine emits, per output sample, the same enum as the Laser CHOP's `debugchan`: `-1` start of frame, `0` color, `1` corner hold, `2` hold of the first point, `3`/`4` pre and post blank-on, `5` blanking, `6`/`7` pre and post blank-off [fontes/touchdesigner.md § Laser and NDI]. The preview paints the point by state when the "diagnostics" toggle is on; `spell ilda monitor` prints the same. It is what turns "the stroke has a tail" into a number.

Freezing: MadMapper separates freezing the engine from freezing each output (`laserOutputsFrozen`) and warns that freezing the output does not stop playback [fontes/madmapper.md § States]. Here: pausing the transport is one thing; closing the shutter is another; both are shown.

## 3. Screen zones

As per `SHORTCUTS.md § Interface` (Media Pool on the left, viewer in the center, Inspector on the right, timeline at the bottom), filled in like this:

- **Left, Media Pool**: list of `.ild` clips, with frames and duration per clip. A list longer than 8 items becomes a search-as-you-type (rule 10).
- **Center, viewer**: the current frame. Preview and output are **the same parameterized widget** (Resolume: `Monitor subject_type=Composition|Preview`) [fontes/resolume.md § Anatomy of the screen]. Viewer modes: clip frame, optimized frame, per-point diagnostics. Masks and scan area drawn on top. In the top right corner: **Arm** and the shutter indicator [fontes/madmapper.md § States].
- **Right, Inspector**: the groups of the table above, collapsible; the `ILDA/` group always open because it shows the formula. Toggle with an arrow for blanking and color: the button turns it on, the arrow opens the adjustment popover, "never two separate buttons" [fontes/blender.md § Anatomy of an editor].
- **Bottom, Timeline**: frame ruler of the clip, In/Out, markers. Transport header = a single `row` with rewind, previous frame, reverse play, play, next frame, fast-forward; while playing, the two plays become a Pause of double the width [fontes/blender.md § Anatomy of an editor].
- **Footer, status bar**: what the mouse buttons do right now, last warning, running task, statistics (PPS, actual FPS, points) [fontes/blender.md § States].

Panel menu: `View, Select, Add, Frame` (rule 6). Nothing floating: TouchDesigner's Palette that closes itself "to gain space" is on the do-not-copy list [fontes/touchdesigner.md § What NOT to copy].

Performance Face: big viewer, transport, Arm, shutter, PPS and actual FPS. No Inspector.

## 4. Shortcuts

Everything in `SHORTCUTS.md` applies (Space, J/K/L, arrows, Home/End, I/O, `Ctrl+L` loop, `M` marker, `Ctrl+Shift+Enter` arm, `Ctrl+Shift+R` rehearsal, `Esc` held blackout). What is added:

| Action | Key | Origin | Conflict |
|---|---|---|---|
| Play/pause on the performance Face | `Shift+Space`; `Space` alone disabled | TD `PanelShortcuts.txt` [fontes/touchdesigner.md § Shortcuts] | none; it is the new column of `SHORTCUTS.md` (rule 8) |
| Momentary shutter | holding `Esc` closes; tapping `Esc` closes the panel | `SHORTCUTS.md` blackout, Resolume `connect` held | none |
| Keyframe on the field under the mouse | `I` | Blender keymap "User Interface" [fontes/blender.md § Shortcuts] | `I` = In when the mouse is not over a field; the UI keymap only applies over a field, as in Blender |
| Return the field to the scene value | `Backspace` over the field | Blender | `Backspace` = previous cue outside a field; disabled on the performance Face |
| Copy the CLI command of the field | `Shift+Ctrl+C` | Blender `copy_data_path` | none |
| Test pattern | `Shift+T` | (ours; MadMapper has the button with no key) | none |
| Choose the order of magnitude and drag | hold the middle button over the number, move vertically, drag horizontally | TD Value Ladder [fontes/touchdesigner.md § Parameters] | gesture, not key; `Alt+right button` with no middle button |
| Ladder on the label moves X and Y together; on the field, only one | same | TD | |

Grammar: no modifier acts on the selected item, `Shift` goes one level up (Resolume: layer → composition) [fontes/resolume.md § Shortcuts and mapping]; it is the same "Shift extends" of `SHORTCUTS.md`. Not included: configurable `Space`, numpad, `X` to delete (rule 11).

## 5. File

In the `.spell`:

- `clips[]`: `{uid, name, path}` with `path` relative to the show and `relpath` declared; a safety copy of the `.ild` inside the show package when exported (TD `savebackup`) [fontes/touchdesigner.md § File]. No PPS in the clip.
- `player`: `{clip, fps, loop, in, out, speed, dmx: {universe, channel}}`. The two DMX channels of Capture.
- `outputs[]` (shared with `patch`): every parameter of the table in §1, **complete**, not only the delta (rule 12), with an enum by stable identifier, not by label (`"ILDA/Mode": "Preserve Image Quality"` of MadMapper is the mistake) [fontes/madmapper.md § What NOT to copy]. Masks as normalized polygons with opacity and inversion.
- `calibration` per output, from `calib_*`.

Outside the `.spell`: discovered IP (goes to the profile's `config.json`), viewer zoom, list of devices seen on the network, frame thumbnails.

Export: two modes, MadMapper's "Movie Mode" ones, because ILDA does not carry PPS: **at fixed FPS** (to play back in Spellcaster) and **as stream** (for the laser's SD card or another DAC) [fontes/madmapper.md § Laser]. The mode becomes a flag of `spell ilda export`.

Version: opening and playing differently is worse than refusing. TouchDesigner 2025.30000 started generating at 192 000 and resampling, removed two parameters, and warned only in the wiki [fontes/touchdesigner.md § What NOT to copy]. `"version"` at the top, local migration, loud failure.
