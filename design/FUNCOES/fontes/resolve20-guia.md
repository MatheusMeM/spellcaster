# DaVinci Resolve 20 — digest of the official Beginner's Guide (source for the Spellcaster timeline)

Digest of the **book**, not of the app: Resolve is not installed on this machine and was not opened.
Every statement carries the PDF page (`p.N`) — which is the file page, not the number printed in the
footer (the book has 25 front-matter pages before printed p.1). Only the Windows column of shortcuts.
What was not read is not here.

## Sources read

| File | What it is |
|---|---|
| `<scratchpad>/fontes/resolve20-beginners-guide.pdf` (643 pages) | "The Beginner's Guide to DaVinci Resolve 20", Blackmagic Design Learning Series, downloaded from https://documents.blackmagicdesign.com/DaVinciResolve/20/The%20Beginners%20Guide%20to%20DaVinci%20Resolve%2020.pdf on 2026-09-09 |
| `<scratchpad>/fontes/resolve20-beginners-guide.txt` (1.1 MB) | text extracted with PyMuPDF, one `=== p.N ===` block per page |
| `<scratchpad>/fontes/resolve20-beginners-guide.toc.txt` | table of contents embedded in the PDF |

`<scratchpad>` = `C:\Users\email\AppData\Local\Temp\claude\D--DRIVE-MNDS-...\6b3d20de-.../scratchpad`.
The PDF and the TXT **do not go into the repository**.

**Read**: p.39-42 (bins and new timeline), p.57-70 (In/Out, zoom, edit overlays, shuffle insert),
p.75-93 (Place on Top, source tape, backtiming, destination controls), p.101-118 (trim), p.127-131
(Inspector), p.148-149, p.169, p.175-198 (audio keyframes, voiceover, track order, cinema
viewer), p.432-440 (Keyboard Customization), p.455-467 (Fairlight), p.485-490 (scrollers, razor),
p.509-515 (keyframes in Fairlight).
**Not read**: the Fusion (9), Color (4-6) and delivery (10) lessons and the end-of-lesson reviews.

## What this book does NOT have — stated up front

- **There is no Cut page chapter.** The book is centered on the Edit page. Cut shows up only as a
  mention: it is the page Resolve opens on by default (p.23, p.362), and Edit has "Full Extent Zoom"
  and "Detail Zoom" which correspond to the upper and lower timelines of Cut (p.58). The Cut
  "mini-timeline" is mentioned in passing in a comparison on the Color page (p.217). **"Smart insert"
  does not exist anywhere in the book** — the search returns nothing. What does exist, and is
  analogous, on the Edit page, is **Shuffle Insert / Swap Insert** (p.65-66).
- **There is no track "arm" nor "record enable".** The search returns nothing. Recording in this book is
  only the **Voiceover** tool (p.193-196), which uses no per-track arm button — the target is chosen
  in the tool's own Record Track menu.
- **There is no mixer automation in Fairlight.** The book closes lesson 8 by stating explicitly that
  automation, Fairlight FX and ADR are left to "The Fairlight Audio Guide to DaVinci Resolve 20" (p.515).
  What the book covers as "automation" is **clip volume keyframing**, not fader automation.
- **There is no shortcut table.** The shortcuts appear scattered through the step-by-step; the list below
  was assembled by sweeping the text, with the page for each one.

---

## 1. Edit page timeline

### Create and organize (p.39-41)

New bin: `Shift+Ctrl+N`, created **inside the selected bin** (p.39). New timeline: `Ctrl+N`,
created **inside the selected bin** (p.40). A project holds many timelines; each one can have its
own settings, chosen at creation or changed later (p.41). The media pool has ordinary bins,
**smart bins** by keyword (the Smart Bins area is resizable by dragging the divider line,
p.68) and **subclips** (p.42).

### Timeline zoom — three modes (p.58)

| Mode | What it does |
|---|---|
| **Full Extent Zoom** | always shows the whole timeline in the window, readjusting by itself. Bird's eye view, navigation to any point |
| **Detail Zoom** | scales to a close view **centered on the playhead**. For getting into the timeline and adjusting an edit point |
| **Custom Zoom** | free scaling, by slider **or `Alt`+mouse scroll**, centered on the playhead |

All three are timeline toolbar buttons. **Track height**: Timeline View Options menu, or
**`Shift`+scroll** inside the video or audio area of the timeline (p.58).

Zoom shortcuts (p.58): `Ctrl+=` zooms in at the playhead, `Ctrl+-` zooms out, **`Shift+Z` toggles between
fitting the whole timeline in the window and going back to the previous zoom**.

This is the most usable detail of the book for us: **three named zoom modes, with a toggle for
"fit everything / back to where I was"**, is a better answer than a loose zoom slider.

### Snapping (p.113, p.188, p.466)

Toolbar button, or the **`N`** key. And the behaviour worth copying: **turning snapping off with `N`
in the middle of a drag turns snapping back on by itself when the mouse button is released** (p.113, p.188). The
modifier becomes momentary without needing a modifier.

### Markers (p.116, p.455-457)

`M` adds a marker at the playhead position — on the source clip, on the timeline clip, or on the
timeline itself (p.116). **`M` a second time with the playhead over the marker opens the marker window**
to rename it, change its color and add a comment/keyword (p.456); double-clicking the marker does the same
(p.116). With no clip selected (`Shift+Ctrl+A` clears the selection), the marker goes to the **timeline**;
with a clip selected it goes to the clip (p.455).
Navigate between markers: **`Shift+↑` / `Shift+↓`** (p.116, p.461).
The marker list lives in the **Markers tab of the Index** (p.457, p.462, p.490) and clicking an item in the
list jumps to that point.

### In / Out (p.57, p.84-85)

`I` marks In, `O` marks Out (p.57). They apply in the source viewer **and** in the timeline (p.78). `↓` jumps to the
next edit point (p.84).
**The Resolve playhead is inclusive of the current frame**: In lands at the head of the frame, Out lands at
the tail — the shortest markable duration is 1 frame (p.84).
Clearing: `Alt+I` (Clear In), `Alt+O` (Clear Out), **`Alt+X` clears both** (p.85).
**Backtiming**: with **only** an Out in the source and In+Out in the timeline, Resolve aligns the two Outs — the
shot ends where it was asked to, instead of starting there (p.85).

### Edit overlays (p.60-62, p.69-70, p.75-79)

Dragging the clip from the source viewer **to the timeline viewer without releasing** brings up the set of
edit overlays; the default is **Overwrite** (p.60). It is the only way to choose the edit type
with the mouse. The ones the book uses:

| Overlay | Effect | Shortcut |
|---|---|---|
| **Overwrite** | lands at the playhead position, overwriting whatever is in the way (non-destructive — it can be untrimmed back later) | `F10` (p.62, p.65) |
| **Insert** | lands pushing the rest of the timeline forward | (p.69) |
| **Append at End** | lands after the last clip of the timeline | (p.70) |
| **Replace** | uses **the position of the two playheads** (source and timeline) to align, instead of In/Out; swaps the existing clip | `F11` (p.115-118) |
| **Place on Top** | lands on the track above, creating new video and audio tracks if needed | `F12` (p.75-77, p.118) |

There is also a **"video only" overlay** in the source viewer: dragging from it onto the destination overlay edits
video only, without the audio — **and this works only by dragging**, not by shortcut nor by toolbar button
(p.79). To get the same with a shortcut, the audio track's **destination control** is turned off
(p.80). Two paths to the same intent, with different rules: it is exactly the kind of
inconsistency not to copy.

**Reordering without dragging** (p.65-66): Swap Clips Towards Left = `Shift+Ctrl+,` and Swap Clips Towards
Right = `Shift+Ctrl+.`. It works with several clips selected. This is the "Shuffle Insert".

### Trim (p.101-114)

Two modes: **Selection mode** (default) and **Trim Edit mode**, toolbar button or the **`T`** key; the
button turns **red** and the cursor changes (p.102).

Trim Edit mode is **contextual — the function depends on where the cursor is over the clip**:

| Cursor over | Operation | What it does |
|---|---|---|
| middle of the clip | **slip** | slides the content inside the clip's In/Out, without moving the clip on the timeline (p.103) |
| center of an edit point | **roll** | moves the cut point, trimming both neighboring clips at once, leaving no gap (p.106-107). It works **also** in Selection mode (p.106) |
| side of an edit point | **ripple** | the duration change propagates through the whole rest of the timeline (p.109-112) |
| the clip's **name bar** | **slide** | the clip slides between its two neighbours, adjusting the outgoing one and the incoming one (p.112-113) |

Visual feedback: during slip and slide the timeline viewer becomes a **four-frame preview** — on top,
In and Out of the clip being adjusted; below, the last frame of the previous clip and the first of the next one
(p.103-104, p.114). During roll, it becomes **two frames** (p.108). And on the timeline, a **white outline**
shows the available *handles*, that is, the part of the clip that exists in the file but is not in use
(p.102, p.104). The tooltip shows the delta in `±SS:FF` (p.101, p.107, p.113).

**Selecting several edit points**: `Ctrl`+click adds a second cut point to the selection, and the
two are trimmed together (p.111). It solves the case of "I rippled here but the shot above stayed
behind".

**Linked Selection** (p.108, p.110): toolbar button, **`Shift+Ctrl+L`**. On, selecting video
selects the linked audio along with it; linked clips have a **chain icon before the name**. Turning it off is what
allows the **split edit** (J-cut / L-cut): rolling only the video cut, leaving the audio where it is
(p.108-109).

**Gap**: even with no clip, "the outgoing side of the gap" is selectable and trimmable (p.110).

**Razor**: `Ctrl+B`, or the scissors button on the toolbar, splits the clip at the playhead (p.489).

**Undo**: `Ctrl+Z`; and there is a **history window** with the full list of what can be undone
and redone, in Edit > History > Open History Window (p.111).

### Inspector (p.127-131)

Button at the top right; it opens to the right of the timeline viewer. It controls the clip: Zoom, Position,
Rotation, Speed Change, stabilization, and the Video / Effects / Transition / Settings tabs according to
what is selected (p.127, p.134, p.143, p.152, p.161). The **Expand** button makes the Inspector take the
full height of the interface (p.127).

**The rule for which clip the Inspector shows** (p.128-129), which is the interesting part:

1. clicking a clip selects it, and the Inspector shows the selected one;
2. **with nothing selected, the Inspector automatically shows the clip of the highest track under the
   playhead**;
3. selection **overrides** the automatic choice — with a clip selected, moving the playhead does not change the
   Inspector.

The clip name appears at the top of the Inspector to confirm whose controls these are (p.129).

Numeric values: drag the wheel, or type and press `Enter` (p.130).

**Keyframing in the Inspector**: the book **does not teach video keyframing on the Edit page**. It says, in a
note (p.155), that the graph can be animated with keyframes instead of Dynamic Zoom and that "you will
learn more later" — and "later" is the **Fusion** lesson (p.542-543), where the gesture is: click the
**gray Keyframe button to the right of the parameter** in the Inspector to create the first one, move in time,
change the value (or click the button again) to create the next one, and refine the curves in the
**Keyframes Editor**, a separate panel opened by the Keyframes button at the top right (p.543).
That is: **in Resolve the keyframe is born in the Inspector and edited in a separate curve editor** — not
on the timeline. Only the **audio** keyframe lives on the timeline.

### Timeline View Options (p.58, p.148, p.169, p.461, p.485-487)

Toolbar menu that controls the drawing of the timeline, not the content: track height (p.58, p.169),
viewer background (Checkerboard / Black, p.148-149), **Track Display Options** — in Fairlight, showing
the video tracks at the top of the timeline (p.461) — and the **scrollers** (p.485-487, see §3).

### Track order (p.197)

Right-click on the track controls → **Move Track Up/Down**. Or open the **Index**, **Tracks** tab, and
drag. The Tracks tab also has the **per-track visibility (eye) button**: hiding tracks
**does not mute them** — they keep sounding, they only leave the screen to simplify the work (p.510-511).
Separating visibility from mute is a distinction Spellcaster needs to have as well.

### Cinema Viewer (p.198)

Workspace > Viewer Mode > Cinema Viewer, or **`Ctrl+F`**. Full screen with navigation controls in
an overlay that **disappear by themselves after a few seconds**; the normal shortcuts keep working
(`Home`, `J`/`K`/`L`); `Esc` exits.

### Segment preview

`/` (slash) plays the segment around the current point to check the mix (p.191, p.192).

---

## 2. Cut page — only what differs and is worth it

The book's entire content about Cut:

- **It is the page Resolve opens on by default** (p.23, p.362).
- Cut has **two stacked timelines**: the upper one is the whole timeline, the lower one is the close-up
  view. The book uses this to explain Full Extent Zoom and Detail Zoom of the Edit page, which are
  the button equivalents (p.58).
- Cut has a **mini-timeline** — a strip of bars whose width is proportional to the clip duration; the
  same thing shows up on the Color page (p.217).

**Source Tape** (p.83) belongs to the Edit page, not to Cut, despite being the tool the request
associated with Cut. Instead of opening clip by clip: clicking the **Source Tape** button in the source viewer opens
**all the clips of the selected bin(s) spliced together**, in the current sort order, as if they were a
single tape. In and Out can be marked in there normally. Turning on **Timeline mode** after Source
Tape, the tape opens **as a timeline**, with greater navigation and precision. **`Q`** toggles between
the Source Tape timeline and the edit timeline. The **Source Clip** button returns to the single-clip view.

For Spellcaster: Source Tape is the answer to "I want to skim 200 `.ild` files without opening them
200 times". It is worth more than the whole Cut page.

---

## 3. Fairlight — only what the book covers

`Shift+7` switches to the Fairlight page (p.457). **It is the same timeline**: fades, transitions,
keyframes, track names, colors and markers made in Edit are already there, and the reverse too (p.458).

### Zoom and height, without the buttons (p.459)

Fairlight **does not have** the Full Extent and Detail Zoom buttons. The gestures are the same as in Edit:
`Alt`+scroll zooms horizontally, `Shift`+scroll changes the track height, **`Shift+Z` fits**
(p.458, p.459, p.462). To zoom in height **centered on a track**, the track header is clicked
first — **and this also automatically selects the clip of that track under the playhead** (p.459).

### Bus 1 (p.459)

Below the tracks there is one more "track" that does not exist in Edit: **Bus 1**, the stereo output of the
timeline. Channel count in Fairlight > Bus Format. The rest of bussing is left to the Fairlight guide.

### Sample precision (p.465-466)

The video clip is trimmed at the frame; audio is trimmed at the **sample** (48 kHz ≅ 2000 samples per frame at
24 fps). Zooming in enough, **the individual sample points appear**. And by holding the button
during the trim, Fairlight **draws the waveform of the handles** — you see what exists outside the
cut before deciding (p.466).

The book's practical rule for fine work: **snapping and Linked Selection off** and deep zoom
(p.467).

### Scrollers (p.485-488)

Timeline View Options → **Display Video Scroller**: opens, below the timeline, a strip of the **individual
video frames**, with a **red line in the center** marking the frame under the playhead. Clicking
a frame to the left or to the right moves the playhead to the start of that frame.
→ **Show Audio Scroller 1**: opens below the video scroller the waveform of **one chosen
track** in a Display menu of its own. Dragging the waveform in the scroller moves the playhead.
It serves to match a sound effect with action on screen — the book says it is much easier than judging by the
timeline (p.488).

This is the most transferable find of Fairlight for us: **a secondary strip, under the timeline, that
shows the content frame by frame around the playhead, with a fixed center mark.** It is the right answer
for "aligning a laser key with a video frame".

### Track formats and locked height (p.463-464, p.512)

Right-click on the header → **Change Track Type To > Mono/Stereo**; a mono track plays only the first channel
of the clip (p.464-465). Right-click on the controls → **Lock Track Height to > Mini** shrinks the track and
keeps it accessible; **> None** gives the free height back (p.512).

### What the book calls "automation" (p.509-515)

It is not fader automation. It is **clip gain keyframing**, the same gesture as in Edit: `Alt`+click on the
gain line creates a keyframe, dragging the segment between two keyframes changes the level (p.513-514).
Noted difference: **in Fairlight the tooltip shows the absolute level and the relative one as a Δ value**
(p.514), while in Edit it shows only the adjustment (p.176).
What replaces automation, in the book, is the **Ducker** — a mixer processor that lowers one track as a
function of another track's audio, turned on by an Enable button on the mixer strip (p.507, p.510). The book compares
the two paths on purpose: Ducker is automatic, keyframing is fine control (p.509).

---

## 4. Audio keyframes on the timeline (p.175-179, p.190-192)

The complete gesture, because it is what most resembles what Spellcaster needs:

1. `Shift`+scroll over the audio area to enlarge the tracks and see the waveform (p.175).
2. **`Alt`+click on the volume bar creates a keyframe** at that point (p.176).
3. Clicking and holding the volume bar shows, in the tooltip, **the current adjustment in dB** (p.176).
4. Dragging the volume bar between two keyframes changes the level of that segment; the tooltip shows the value
   (p.177).
5. **Holding `Shift` during the drag gives fine precision** (p.177).
6. The usage pattern is **in pairs**: two keyframes mark the edge of a region, and the segment between them
   is what gets raised or lowered (p.176-178, p.192). To remove a pop, three keyframes and pull the
   middle one down (p.178-179).

There is, in the book, no curve, easing or interpolation type for audio keyframes — only the segment between
two points.

---

## 5. Voiceover (p.193-196)

The only recording the book teaches. Timeline > Record Voiceover, or the **Voiceover button at the top of
the timeline** (p.194). The window has:

- **File Name** — the name of the file to record (p.194);
- **Audio Input** — which input to use, when there is more than one (p.194);
- **Record Track** — which track to record on; **"Auto" lets Resolve choose** according to the current track
  layout (p.194);
- an Options menu (…) with **Mute Timeline Audio While Recording**, to avoid feedback (p.195), and
  **Stereo Input** (the default is mono) (p.195);
- a **Record** button, which fires a **countdown** before starting (p.195); pressing it again,
  or `Esc`, ends it (p.196).

The result lands **as a new clip on a new timeline track and, at the same time, in the selected bin
of the media pool** (p.196). Subsequent takes **overwrite the timeline clip** but **pile up in the
bin** — nothing is lost (p.196).

For us: countdown before recording, "Auto" destination, and "the last take rules the timeline but
all of them are kept" are three ready-made decisions for `gravar-dmx`.

---

## 6. Shortcuts cited in the book

Windows only. Every line has the page where it appears.

### Timeline and navigation

| Action | Windows | p. |
|---|---|---|
| New bin | `Shift+Ctrl+N` | 39 |
| New timeline | `Ctrl+N` | 40 |
| Mark In / Out | `I` / `O` | 57 |
| Clear In / Out / both | `Alt+I` / `Alt+O` / `Alt+X` | 85 |
| Next edit point | `↓` | 84 |
| Zoom in / out at the playhead | `Ctrl+=` / `Ctrl+-` | 58 |
| Dynamic Custom Zoom | `Alt`+scroll | 58, 459 |
| Track height | `Shift`+scroll | 58, 175, 459 |
| **Fit the timeline / back to the previous zoom** | `Shift+Z` | 58, 458, 462 |
| Snapping on/off (turns back on by itself when the mouse is released) | `N` | 113, 188, 466 |
| Marker (2nd time opens the marker window) | `M` | 116, 456 |
| Previous / next marker | `Shift+↑` / `Shift+↓` | 116, 461 |
| Deselect all | `Shift+Ctrl+A` | 455 |
| Undo | `Ctrl+Z` | 111 |
| Segment preview | `/` | 191 |
| Full screen (Cinema Viewer) | `Ctrl+F` | 198 |
| Exit full screen / stop recording | `Esc` | 196, 198 |
| Fairlight page | `Shift+7` | 457 |

### Editing

| Action | Windows | p. |
|---|---|---|
| **Overwrite** | `F10` | 62, 65 |
| **Replace** | `F11` | 118 |
| **Place on Top** | `F12` | 77, 81-89 |
| Trim Edit mode | `T` | 102, 117 |
| Linked Selection on/off | `Shift+Ctrl+L` | 110 |
| Add edit point to the selection | `Ctrl`+click | 111 |
| Razor / split at the playhead | `Ctrl+B` | 489 |
| Swap Clips Towards Left / Right | `Shift+Ctrl+,` / `Shift+Ctrl+.` | 66 |
| Toggle Source Tape ↔ edit timeline | `Q` | 83 |
| Enable Clip (turns the clip on/off) | `D` | 435 |
| Bypass all grades | `Shift+D` | 436 |
| Volume keyframe | `Alt`+click on the volume bar | 176, 513 |
| Fine precision while dragging volume | `Shift` | 177 |
| Paste Attributes | `Alt+V` | 145, 183 |
| Create Subclip | `Alt+B` | 416 |
| Keyboard Customization | `Alt+Ctrl+K` | 433 |

### Keyboard Customization (p.432-439) — copy the system, not the shortcuts

- Opens with `Alt+Ctrl+K` (p.433).
- It ships **presets that emulate other NLEs**; the book warns that the emulation is never 100%, because functions
  that exist in one system do not exist in the other (p.433-434).
- The top half is an **interactive keyboard**: keys with no function appear dark, keys with a function
  appear light, and **a number in the lower right corner of the key indicates that it has a function on more
  than one page** of the app (p.434). Clicking (or holding the physical key) shows, in the Active area below,
  **the function and in which panel it applies** (p.435). Clicking the modifiers remaps the whole drawing
  of the keyboard for that modifier (p.435-436).
- The bottom half is **search by command**: choose the group (All Commands, or a specific menu/panel),
  type, and click the Keystroke column to assign by pressing the keys (p.436-438).
- **One command accepts several shortcuts** (`+` button); each one removable by the `x`; and there is a **per-command
  reset arrow**, visible on mouse-over (p.438).
- **A conflict is announced, not silent**: if the combination already belongs to another command, Resolve
  asks; choosing Assign, the shortcut is **removed from the original command**. Resetting gives it back (p.438).
- **The stock presets cannot be altered** — touching them forces saving a new named preset
  (p.438-439). The Options menu (…) exports, imports and deletes presets (p.439).

This is the right model for our `design/SHORTCUTS.md` to become a screen: a drawn keyboard, search by
command, conflict warning naming who loses the shortcut, user preset separate from the factory preset.

---

## Adaptation to Spellcaster

| Resolve 20 | Spellcaster | Reference |
|---|---|---|
| Edit page timeline | show timeline | p.58 |
| Clip on a track | segment of `.ild` / audio / video / fx | p.101 |
| **Full Extent / Detail / Custom Zoom** | the three zoom buttons of our ruler | p.58 |
| `Shift+Z` | "fit everything" with a return to the previous zoom | p.58 |
| **Contextual Trim Edit mode** (`T`) | one mode, four operations by cursor position | p.102-113 |
| **Handles as a white outline** | show how much of the file exists outside the cut | p.102, 466 |
| Four-frame preview on slip/slide | preview of the incoming and outgoing frame while trimming | p.103-104 |
| **Linked Selection** (`Shift+Ctrl+L`) | turn on/off the link between the laser line and the audio line of the same segment | p.108-110 |
| Marker (`M`, `M` again edits) | timeline marker with name, color and comment | p.116, 456 |
| **Index > Tracks tab, with a visibility eye** | list of show lines; hiding ≠ muting | p.197, 510-511 |
| **Index > Markers tab** | clickable marker list | p.457, 462 |
| Volume keyframe (`Alt`+click, pairs) | parameter keys drawn straight on the track | p.176-178 |
| Inspector, with the "clip of the highest track under the playhead" rule | properties panel of the selected segment | p.128-129 |
| **Video and audio scrollers** | frame-by-frame strip under the timeline, with a center mark | p.485-488 |
| **Voiceover tool** (countdown, Record Track = Auto, takes piled up in the bin) | `gravar dmx` | p.193-196 |
| **Keyboard Customization** | Spellcaster shortcuts screen | p.432-439 |
| Source Tape (`Q`) | skim many files as if they were a single one | p.83 |
| Playhead inclusive of the frame; minimum duration 1 frame | edge rule of our time selection | p.84 |

### What does NOT apply

Color, Fusion and Deliver pages, grades and nodes, bussing and multichannel audio mixer, mono/stereo
track formats, speed change and stabilization — Spellcaster does no image post nor mixing, and
what is an "audio channel" here is a DMX universe there, which has neither stereo nor sample rate.

### Open issue

Resolve's `N` (snapping) and `T` (trim mode) collide with nothing in `design/SHORTCUTS.md` today, but
`M` (marker) and Ableton's `M` (Computer MIDI Keyboard) point in opposite directions — see
`ableton12.md` § Adaptation. Not decided here.
