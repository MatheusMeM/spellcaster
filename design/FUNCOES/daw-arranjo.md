# Timeline as an Arrangement View — `spell timeline` (panel `Shift+2`)

Request from Matheus (2026-09-09): *"I want an interface of use and functionality and usability and interface just like ABLETON, professional and with features and segments that were thought through"*, and *"I want drag n drop of elements and outputs and media and laser and audio and video"*.

`timeline-daw.md` (branch `frente/timeline-daw`) has already read both manuals for **navigating, marking and automating**: wheel/zoom/pan, follow, loop, snap, marker, nudge, parameter lane. That document has 38 behaviors and they are not repeated here. The hole it declares itself is item 37: *"our timeline has no clip with an edge"*. **This file is the clip.** What it adds: overview strip, ruler with brace and locators, a real track header (name, arm/solo/mute, lock, fold, height, reorder, color), the clip as a first-class object (laser, audio, video, fx, osc), automation lane selector, time selection x object selection, Inspector on the right and transport.

Sources. **Ableton Live 12**, official online manual (`ableton.com/en/live-manual/12/`), cited by section. **DaVinci Resolve 20**, local manual `C:\Program Files\Blackmagic Design\DaVinci Resolve\Documents\DaVinci Resolve.pdf` (4140 pages), cited by page. Nothing from memory.

Code. Lines of `spellgui/web/*` and `spellcore/*` at base commit `dac3e0a`. The **today** column describes what exists; `pontos-falhos.md` carries the audit with screenshots.

| Item | Who solved it best | Why |
|---|---|---|
| Clip on the timeline | Resolve | A clip is media with a source In/Out and a timeline In/Out; `Ctrl+E` cuts at the click point, dragging the edge trims. Ableton has the same, with warp and bars behind it, which do not come in (`timeline-daw.md` item 38) |
| Track header | Resolve | Editable name (p.645), color by menu (p.621), lock/R/S/M with a drag over several (p.3681), Tracks Index as a parallel list (p.646, p.649) |
| Reorder track | Resolve | Drag in the Index with a white line showing where it lands (p.3681) — without that, "move track" becomes an Up/Down menu (p.3574), which is worse |
| Overview strip | Ableton | §6.1: dragging horizontally scrolls, vertically zooms, double-click frames everything. Resolve has no overview, it has a Zoom Slider |
| Automation lane selector | Ableton | §25.5: two selectors (device, parameter), an LED on whatever is automated, "Show Automated Parameters Only". It is exactly our problem of five lanes per track |
| Loop brace and locators | Ableton | §6.6 and §6.4: brace draggable at the edges and in the middle; a locator launches playback and is mappable |
| Time selection x object selection | Ableton | §6.9: "Arrangement editing is selection-based"; clicking the background puts an insert marker, dragging makes a range |
| Inspector | Resolve | p.412: panels per aspect (Video, Audio, Effects, Transition, Image, File), a panel that does not apply goes gray — it does not disappear |
| Consolidate | none | **Does not come in.** Ableton §6.13 records a new sample per track in `Samples/Processed/Consolidate`; we do not render media and we are not going to write a derived file into the show |

## 1. Objects and verbs

**Clip** — a piece of media placed in time. It is the object that is missing. Four fields: `t0` (where it starts on the timeline), `len` (how long it lasts on the timeline), `src` (file, relative to the show folder) and `offset` (from which point of the file it is read). Verbs: **insert** (drop), **move**, **trim** (both edges), **cut** (`Ctrl+E`), **duplicate** (`Ctrl+D`), **copy/paste**, **delete** (`Delete`). A clip has no envelope of its own: automation belongs to the track (§3.4).

**Track** — the line of the `.spell` (`tracks[]`). It gains verbs it does not have today: **rename**, **reorder**, **lock**, **fold**, **change height**. Types that draw a clip: `laser` (`.ild`), `audio` (`.wav/.mp3/.flac`), `video` (`.mp4/.webm`), `fx` (`.rhai`), `osc`. Types that draw keyframes only: `dmx`, `artnet`, `fixture`. Type `cue` draws a vertical mark (`timeline-daw.md` item 36).

**Lane** — a drawn line. A track gives one main lane and one lane per automated parameter (`timeline.js:157-196`). New: the parameter lanes **fold** and exist on demand, not the five fixed ones of `LANE_PARAMS` (`timeline.js:190-196`).

**Time selection** — a range with no object. It does not exist today: `CK.sel()` stores keyframes (`canvaskit.js:152-168`).

**Locator** — a marker that fires. In Ableton §6.4 a locator is a point in the scrub area that launches playback; we already have `markers[]` and the `cue` track. **A locator does not become a new object**: see §5.

## 2. Screen segments

From top to bottom, full width, no overlap (that is complaint 4 of `pontos-falhos.md`):

| Segment | Height | Content | Origin |
|---|---|---|---|
| Transport bar | 32 px fixed, with wrapping or overflow into a menu | position (timecode + seconds), play/stop, rec, loop, follow, snap, editable show name | Ableton (Control Bar) / Resolve p.625 (toolbar) |
| Overview strip | 24 px | the whole show in miniature, outline of the current window | Ableton §6.1 |
| Ruler | 24 px (`TL.rulerH`, `timeline.js:33`) | adaptive timecode, loop brace, markers, In/Out | Ableton §6.1 / Resolve p.484 |
| Tracks | whatever is left, scrollable | 192 px header (`TL.headW`) + lanes | both |
| Inspector | 280 px on the right, collapsible | of whatever is selected | Resolve p.412 |
| Footer | 20 px | grid spacing + bus state | Ableton §6.10 ("displayed above the time ruler in the lower right corner") |

The Inspector on the right and the browser on the left (`browser-dnd.md`) form the layout that `SHORTCUTS.md § Interface` already describes: *"Inspector on the right, Media Pool on the left"*.

## 3. Behavior table

`today` = base commit `dac3e0a`. P1 = missing in the first minute. No line repeats `timeline-daw.md`.

### 3.1 Overview and ruler

| # | Behavior | Origin | Gesture | Effect on our model | Today | P |
|---|---|---|---|---|---|---|
| A1 | Overview strip: dragging horizontally scrolls, vertically zooms, double-click inside the outline frames everything | Ableton §6.1 ("drag left or right to scroll... drag vertically to zoom in or out... double-click anywhere within the black outline") | drag / double-click | Only the view (`CK.view`), nothing in the `.spell` | missing | **P1** |
| A2 | The outline marks the current window inside the strip | Ableton §6.1 | — | Same | missing | **P1** |
| A3 | Clicking the ruler plays from there | Ableton §6.1 ("Clicking anywhere in the scrub area launches playback from that point") | click | `locate {t}` + `resume` | partial — clicking the ruler only drags the playhead (`timeline.js:704-712,747`) | P2 |
| A4 | Loop brace with three handles: left edge, right edge, middle | Ableton §6.6 ("dragging from the left or right edge adjusts the loop start/end points, while dragging the brace bar horizontally moves the loop without changing its length") | drag | The handles are In/Out (`timeline-daw.md` item 7); the middle is item 8 over there. **New here**: draw it as a brace (two serifs and a bar), not as a scratch | the drawing is missing — `timeline.js:622-623` makes a 3 px gray rectangle that disappears against the grid (`pontos-falhos.md` item 27) | P2 |
| A5 | Clicking the brace selects what is inside it | Ableton §6.9 ("Clicking on the loop brace is a shortcut for executing the Edit menu's Select Loop command") | click on the brace | Selects keyframes and clips between In and Out | missing | P3 |
| A6 | Arrows move the brace along the grid; `Ctrl+←/→` shortens/lengthens; `Ctrl+↑/↓` doubles/halves | Ableton §6.6 | keyboard | Rewrites `in`/`out` | missing | P3 |
| A7 | Adaptive timecode: with the whole show on screen the label is `mm:ss`; with one second on screen it is `ss:ff` | both (Ableton §6.10 shows the spacing; Resolve p.647 has the three zoom presets) | automatic | Drawing only | wrong — `tc()` always prints `hh:mm:ss:ff` (`timeline.js:56-57`): 11 characters to say 10 seconds | P2 |
| A8 | The grid ladder has a frame step | both | automatic | `STEPS` gains `1/fps` and `5/fps` as its first steps, read from `show.fps` | missing — the smallest step is 0.04 s (`timeline.js:28`), which at 30 fps is neither a frame nor a multiple of a frame | P2 |

### 3.2 Track

| # | Behavior | Origin | Gesture | Effect on our model | Today | P |
|---|---|---|---|---|---|---|
| B1 | Track name editable in the header | Resolve p.645 ("click the default 'Video X' or 'Audio X' track name to select it, then type your preferred name and press the Return key") | double-click, type, `Enter` | `show_patch {ops:[{op:"add", path:"/tracks/3/name", value:"..."}]}`. The `name` field is already written by `track_add` when `label` comes in (`edit.rs:692-694`) | missing — the name is `spec.file`/`spec.clip` truncated at 22 characters (`timeline.js:503`) | **P1** |
| B2 | Arm / Solo / Mute actually take effect | both (Resolve p.3681: *"you can use the Lock, Record, Solo, and Mute controls to quickly enable or disable multiple tracks by clicking and dragging up or down"*) | click on R/S/M, or drag over several | `mute` goes into the `.spell` and the Rust player obeys it (`timeline-daw.md` item 30, already in `DECISOES.md`) | **broken** — `commit()` rewrites `spec.mute` from each lane and the parameter lane undoes the mute (`timeline.js:308-312`; `pontos-falhos.md` item 10) | **P1** |
| B3 | Lock locks the track | Resolve p.3635 ("Click any track's lock control and drag over the lock controls of other tracks") | `Shift+L`, or drag over the padlocks | `tracks[i].lock` (proposed in `timeline-daw.md §3`, awaiting vote) | missing | P2 |
| B4 | Fold the parameter lanes | Ableton §6.9 (`U`) and §25.5 ("Using the left and right arrow keys on a main track will fold/unfold its automation lanes") | `U`, or `←`/`→` on the focused header | Window state. **Course correction**: the lanes now exist only for a parameter that has a keyframe | missing — five fixed lanes per track (`timeline.js:190-196`) | **P1** |
| B5 | Height per track, by dragging the divider | Resolve p.643 ("any track in the Timeline can be individually resized by dragging its top divider in the Track Header area") | drag the top edge of the header | Window state. **Divergence**: `timeline-daw.md` item 19 proposed a global height (`TL.rowH`); Resolve p.643 is per track, and that is what you expect from an audio track next to a DMX one. It stays per track, with `Alt`+drag applying it to all (Ableton §6.9: *"hold Alt while resizing a single track"* resizes all of them) | missing | P2 |
| B6 | Reorder by dragging, with a line at the destination | Resolve p.3681 ("As you drag, a white line shows you where that track will be inserted when you release it") | drag the header | **Rewrites the order of `tracks[]`.** There is no order field: the order of the array is the order on screen. It needs `track_move` (§6) | missing | P2 |
| B7 | Color per track family | Resolve p.621 ("Each track can be color-coded with one of 16 different colors") | — | **Rejected as free color** (`PRINCIPIOS.md §2`; already decided in `timeline-daw.md` item 31). It comes in derived from the `type`: `dmx`/`artnet` amber, `laser` red, `audio` green, `video` blue, `fx`/`osc` gray. It is not a field of the `.spell` | missing | P3 |
| B8 | New track by dropping into the empty space below the tracks | Ableton §4.10 ("Dragging and dropping content from the browser into the space... below Arrangement View tracks will create a new track and place the new item(s) there") and §6.1 (Mixer Drop Area) | drop media below the last track | `track_add {kind, label}` + the clip. Full contract in `browser-dnd.md §3` | missing | **P1** |
| B9 | Delete empty tracks in one go | Resolve p.511 ("Delete Empty Tracks") | header menu | `track_del` in series | missing | P3 |

### 3.3 Clip

| # | Behavior | Origin | Gesture | Effect on our model | Today | P |
|---|---|---|---|---|---|---|
| C1 | Clip drawn as a rectangle with a title bar, file name and content | Resolve p.625 (Filmstrip / Thumbnail / Minimized) / Ableton §6.7 | — | One `clips[]` per track (§4). Content: waveform for `audio` (`audio-video.md §2`), frame thumbnail for `laser`/`video`, solid color for `fx`/`osc` | **entirely missing** — the track `{"type":"laser","clip":"medgrupo_laser.ild"}` of `shows/medgrupo.spell` draws an empty lane (`timeline.js:157-181` only knows `keys` and `spec.<param>`; `pontos-falhos.md` item 14) | **P1** |
| C2 | Only the title bar drags the clip | Ableton §6.7 ("only the clip bar is draggable, it is not possible to drag from the clip's waveform or MIDI display") | drag the bar | Changes `t0`, and the track if it moves to another line | missing | **P1** |
| C3 | Dragging the edge trims | Ableton §6.7 ("Dragging a clip's left or right edge changes the clip's length") | drag the edge | The right edge changes `len`; the left edge changes `t0` **and** `offset` by the same amount | missing | **P1** |
| C4 | Slide the content inside the clip | Ableton §6.7 (`Ctrl+Shift`+drag on the waveform) | `Ctrl+Shift`+drag on the body | Changes `offset` only | missing | P3 |
| C5 | The clip snaps to the grid **and** to the edge of another clip, to a marker and to the playhead | Ableton §6.7 ("Clips snap to the editing grid, as well as... the edges of other clips, locators and time signature changes") / Resolve p.546 | automatic | `TL.snaps` gains the clip edges. `Alt` releases it (already in `timeline-daw.md` item 11) | partial — `snapT` exists and only knows the grid and markers | P2 |
| C6 | Cut at the clicked point | Ableton §6.12 (`Ctrl+E`: *"click anywhere within a clip's waveform or MIDI display and then use the shortcut"*) | `Ctrl+E` | One clip becomes two: `{t0,len,src,offset}` → `{t0, d, src, offset}` + `{t0+d, len-d, src, offset+d}`. It cuts at the **click**, not at the playhead: it avoids moving the transport in order to edit | missing | **P1** |
| C7 | Duplicate | Ableton §41.5 (`Ctrl+D`) | `Ctrl+D` | A copy right after: `t0' = t0 + len` | missing | P2 |
| C8 | Duplicate by dragging with `Alt` | Resolve / MadMapper (gesture already fixed in `timeline-daw.md` item 15) | `Alt`+drag | Same, wherever you drop it. **Conscious collision**: `Alt` also releases the grid (item 11 over there); in a clip drag both things apply together, and that is how it is in Ableton |missing | P2 |
| C9 | Copy / paste | both | `Ctrl+C` / `Ctrl+V` | Pastes at the playhead, on the focused track | missing | P2 |
| C10 | Deactivate without deleting | Ableton §6.9 ("Pressing the 0 key deactivates a selection of material") | `0` | `clips[i].mute: true`. **Divergence**: `SHORTCUTS.md` uses `Shift+D` for track mute; `0` stays only for the selected clip | missing | P3 |
| C11 | Consolidate adjacent clips into one | Ableton §6.13 (`Ctrl+J`) | — | **DOES NOT COME IN.** In Ableton *"a new sample is created for every track in the selection"*, written to `Samples/Processed/Consolidate`. We do not render media, and a derived file inside the show goes against the spirit of `FUNCOES/README.md §12` (the `.spell` references originals, not products) | — | — |
| C12 | "…Time" commands (insert/delete time on all tracks) | Ableton §6.11 (`Ctrl+Shift+X/C/V/Delete`; `Ctrl+I` inserts silence) | — | **Does not come in now.** It requires a ripple in the `keys[]` of every track and there is no request for it. Recorded because it is the difference between editing a clip and editing the timeline | — | P3 |
| C13 | Fade in/out on the audio clip | Ableton §6.8 (`Ctrl+Alt+F`; `F` over the lane toggles the controls) | — | **Does not come in now.** Audio volume is an automation lane like any other; a fade would be a shortcut for two keyframes. Re-evaluate with `audio-video.md` | — | P3 |

### 3.4 Automation

| # | Behavior | Origin | Gesture | Effect on our model | Today | P |
|---|---|---|---|---|---|---|
| D1 | Automation mode toggles with `A` | Ableton §25.5 ("enable Automation Mode by clicking the toggle button above the track headers, or using the A shortcut") | `A` | Shows/hides all the parameter lanes. Window state | missing | P2 |
| D2 | Lane selector with two fields and an LED on whatever is automated | Ableton §25.5 (Device chooser + Automation Control chooser; *"showing an LED next to their labels"*) | menu in the lane header | The first field is the **target** (the track, or the graph module), the second is the **parameter** — and the pair is the textual address of rule 2 of `FUNCOES/README.md`: `track/3/scale`, `laser/1/kpps`. It is the same address that `mapping.md` maps | missing — the five fixed lanes of `LANE_PARAMS` (`timeline.js:190`) are a selector with no menu | **P1** |
| D3 | "Show Automated Parameters Only" | Ableton §25.5 | selector option | Default **on**: only a parameter lane that has a keyframe shows up. It is what fixes B4 | missing | **P1** |
| D4 | A button that sends the envelope to its own lane; with `Alt`, sends all the automated ones | Ableton §25.5 | click / `Alt`+click | Window state | missing | P3 |
| D5 | Hiding the lane does not deactivate the envelope | Ableton §25.5 ("hiding a lane from view does not deactivate its envelope") | — | A rule, not a gesture: folding never touches `keys`. It is the rule that prevents repeating defect B2 | — | **P1** |
| D6 | Envelope locked to the music or to the clip (Lock Envelopes) | Ableton §6.1 | toggle | **Does not come in.** Our keyframes belong to the track and live in absolute time; there is no second mode | — | — |
| D7 | Automation red, modulation blue | Ableton §26.3 | — | **Rejected.** `PRINCIPIOS.md §2`: one accent only, color means state. The two are told apart by the lane they are in | — | — |
| D8 | Simplify envelope | Ableton §25.5.4 ("calculates the optimal number of breakpoints... and removes any unnecessary breakpoints") | menu | It becomes necessary when `gravar-dmx` records 30 keyframes per second. Recorded for that workstream, not for this one | missing | P3 |
| D9 | Ready-made automation shapes (sine, ramp, ADSR) over the time selection | Ableton §25.5.5 | context menu | **Does not come in**: it is what `fx` (`.rhai`) and the graph do better. Recorded so it is not reinvented | — | — |

### 3.5 Selection, transport and Inspector

| # | Behavior | Origin | Gesture | Effect on our model | Today | P |
|---|---|---|---|---|---|---|
| E1 | Clicking the background puts an insert marker; dragging makes a time selection | Ableton §6.9 | click / drag on empty space | The time selection is `{t0, t1, tracks[]}`, separate from the object selection. `Ctrl+L` loops over it (`timeline-daw.md` item 6) | missing — dragging on empty space makes a keyframe marquee (`canvaskit.js:152-168`) | **P1** |
| E2 | Selection-based editing | Ableton §6.9 ("you select something and then execute a command") | — | Rule: every editing command asks "time selection or object selection?", never both at the same time | — | **P1** |
| E3 | `Z` frames the time selection, `X` goes back on the zoom | Ableton §6.2 | `Z` / `X` | **Divergence**: `SHORTCUTS.md` already has `Shift+Z` (frame everything) and `timeline-daw.md` item 22 already fixed the second press of `Shift+Z` as going back. `Z`/`X` stay out; framing the selection is `Shift+Z` with an active selection, the same logic as `Ctrl+L` | missing | P2 |
| E4 | Transport: position in timecode **and** in seconds, play/stop, rec, loop, follow, snap | both | — | `locate`, `resume`, `pause`, `stop` already exist in the registry (`registry.rs:243,251,257`) | partial — `index.html` has the buttons and no editable position field | **P1** |
| E5 | Stopping twice goes back to the start | Ableton §7.1 ("pressing the Control Bar's Stop button twice") | `Space` twice while stopped | `stop` + `locate {t:0}` | missing | P3 |
| E6 | Inspector on the right, in panels per aspect; an inapplicable panel goes gray and does not disappear | Resolve p.412 ("Inspector panels that are not applicable to your clip or selection are grayed out") | `Shift+7` (already in `SHORTCUTS.md`) | Shows the selected item: track (name, type, universe, address, output), clip (`src`, `t0`, `len`, `offset`), keyframe (t, value, curve). Every field shows **the textual address** next to the label — it is what `mapping.md` maps and what `Shift+Ctrl+C` copies (`FUNCOES/README.md §7`) | missing — no Inspector in `index.html` (`pontos-falhos.md` item 25) | **P1** |
| E7 | Switch Arrangement ↔ Session | Ableton §41.1 (`Tab`) | — | Solved in `daw-sessao.md §4`: `Tab` is already the Face switch in `SHORTCUTS.md` | — | — |

## 4. Data model

### 4.1 The clip enters the `.spell`

```json
{"type": "laser", "universe": 1,
 "clips": [{"t0": 0, "len": 46.8, "src": "medgrupo_laser.ild", "offset": 0}],
 "scale": [[0, 1.0], [46.8, 1.0]]}
```

Rules:

- `clips[]` **coexists with** `keys[]` and with the parameter lanes. It replaces nothing: `keys` is the value in time of a `dmx` track; `clips` is the media in time of a track that has a file.
- `t0`, `len` and `offset` in **seconds**, like `duration` and like the keys of `keys` (`timeline.rs`, `parse_key`). Not in frames: `fps` belongs to the show, and a 30 fps `.ild` in a 25 fps show can exist.
- `src` is a path **relative to the show folder**, always with `/` and never with `\`, as the `estatico()` of serve already requires (`serve/src/lib.rs:106-108`).
- An absent `offset` is 0; an absent `len` is "to the end of the file", resolved by the GUI on loading.
- Audio and video tracks are new types: `{"type":"audio","clips":[...]}` and `{"type":"video","clips":[...]}` — no `universe`, no `address`. Contract in `audio-video.md §1`.
- **Migration of what already exists**: today's laser track is `{"type":"laser","clip":"x.ild","fps":30}`. The engine ignores the `laser` type (`timeline.rs:404-410`), so `clip` (singular) only lives in the GUI. `migrate()` (`show.rs`) converts `clip` → `clips:[{t0:0, len:<file duration>, src:<clip>, offset:0}]` **without** bumping `VERSION`: it is a compatible addition, and whoever reads `clips` and does not find it reads `clip`.

### 4.2 What is window state and does not go into the show

`FUNCOES/README.md §12` forbids window state in the `.spell`. These go into the GUI's `config.json`: height per track, lane fold, zoom and view position, follow, snap on, automation mode (`A`), width of the Inspector and of the browser, last show opened. In/Out stays in the `.spell` (`timeline.js:330-335` writes `show.in`/`show.out`) and loop stays out, as `timeline-daw.md §3` already decided.

### 4.3 Awaiting vote

Goes to `design/DECISOES.md`, without implementing or removing:

1. **`clips[]` as a track field** and the migration from `clip` → `clips`, in the format of §4.1.
2. **Types `audio` and `video`** as show tracks (the alternative is for them to be outputs, and they are not: they have a position in time).
3. **The order of `tracks[]` is the order on screen** (B6): reordering rewrites the array, and any index stored in a cue or a mapping starts pointing at another track. The alternative is a `uid` per track (`FUNCOES/README.md §12`: *"reference between objects by UID, never by short name"*), which solves it once and for all and costs one field.
4. **Height per track** (B5) against the global height proposed in `timeline-daw.md` item 19.

## 5. Locator: why it does not come in as an object

Ableton §6.4 has a locator: it launches playback, it is mappable, it has a name, `Ctrl+R` renames it. We already have two things that do that: `markers[]` (a point on the ruler with a name and a note, `timeline-daw.md` item 26) and the `cue` track (`timeline.rs:354`). A third object breaks rule 9 of `FUNCOES/README.md` (*"one verb per concept"*).

Proposed decision: **the marker gains an optional `go` field**, which is a registry address.

```json
{"t": 12.5, "name": "pico", "note": "", "go": "cue/3/go"}
```

A marker with no `go` is a marker. A marker with `go` is a locator: a triangle on the ruler, it fires when the transport passes and when it gets a double-click. The engine already has the bottom half — `in.marker` is a graph node and `input {key:"marker:pico"}` is the input (`script/src/graph.rs:280-290`). **It is also awaiting vote**: it is a format change.

## 6. Commands missing from the registry

Today's 30 commands (`registry.rs` + `edit.rs`) have none for clips. What this function needs, in the `<object>_<verb>` pattern:

| Command | Arguments | Does |
|---|---|---|
| `clip_add` | `track, t0, len, src, offset` | Inserts; returns the index |
| `clip_set` | `track, index, t0?, len?, offset?, mute?` | Moves, trims, slides |
| `clip_del` | `track, index` | Removes |
| `clip_split` | `track, index, t` | Cuts in two (C6) |
| `track_move` | `from, to` | Reorders `tracks[]` (B6) |

`track_set {index, name?, mute?, lock?}` does not need to exist: `show_patch` already does it (`edit.rs:824`) and the path is `/tracks/3/name`. The `comandos` workstream decides whether `clip_*` also becomes `show_patch`; the difference is that `clip_split` has logic (recomputing `offset`) and `show_patch` has nowhere to put logic.

## 7. Shortcuts

What this file adds to `SHORTCUTS.md` and to `timeline-daw.md §5`:

| Action | Key | Origin | Conflict |
|---|---|---|---|
| Cut the clip at the click point | `Ctrl+E` | Ableton §6.12 | **collides** with the `Ctrl+E` of `SHORTCUTS.md` ("Easing of the selected keyframe opens a menu"). Resolution: `Ctrl+E` acts on the selected object — a clip cuts, a keyframe opens easing. One shortcut, two objects; that is rule E2 |
| Duplicate the selection | `Ctrl+D` | Ableton §41.5 | none |
| Rename the focused track | `Ctrl+R` | Ableton §6.4 (there it is renaming a locator) | none |
| Automation mode | `A` | Ableton §25.5 | none; bare key on the focused panel (`FUNCOES/README.md § Open issues`) |
| Deactivate the selected clip | `0` | Ableton §6.9 | none |
| Frame the time selection | `Shift+Z` with an active selection | Ableton §6.2 (there it is `Z`) | none; the same key as frame everything, with a selection |

## 8. Tests that prove each piece

One per non-trivial rule, in the shape of `timeline-daw.md §4` (pure function + `assert`, running in `node`):

| Function | Input | Expected output |
|---|---|---|
| `TL.clipSplit(c, t)` | `{t0:0,len:10,src:"a",offset:2}`, `t=4` | `[{t0:0,len:4,src:"a",offset:2}, {t0:4,len:6,src:"a",offset:6}]` |
| `TL.clipTrim(c, "L", dt)` | same clip, `dt=+3` | `{t0:3,len:7,offset:5}` (the left edge moves `t0` and `offset` together) |
| `TL.clipTrim(c, "R", dt)` | `dt=-2` | `{t0:0,len:8,offset:2}` |
| `TL.snapsDe(track)` | track with two clips | the four edges, sorted |
| `TL.lanesDe(spec)` | spec with `scale` and without `rot` | one main lane and **one** parameter lane (proves D3) |
| `TL.commit` with a parameter lane in divergent mute | the case of `pontos-falhos.md` item 10 | `spec.mute === true`; the test that fails today already exists in `scratchpad/prova_mute.js` |
| `TL.migraClip(track)` | `{"type":"laser","clip":"x.ild"}` | `clips:[{t0:0,src:"x.ild",offset:0}]` |
| Rust, `timeline.rs` | track with `"mute": true` | no write to the universe |

Visual proof, in the shape of `pontos-falhos.md`: headless screenshot of `index.html` with `shows/medgrupo.spell`, showing `medgrupo_laser.ild` as a 46.8 s rectangle with a name, and the `scale` lane folded.
