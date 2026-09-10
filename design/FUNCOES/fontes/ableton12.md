# Ableton Live 12 — digest of the official manual (source for the Spellcaster timeline)

Digest of the **manual**, not of the app: Live is not installed on this machine and was never opened.
Every statement carries the PDF page (`p.N`). What was not read is not here.

## Sources read

| File | What it is |
|---|---|
| `<scratchpad>/fontes/live12-manual-en.pdf` (96.7 MB, 1009 pages) | "Ableton Live 12 Manual", downloaded from https://cdn-resources.ableton.com/resources/manuals/live12-manual-en.pdf on 2026-09-09 |
| `<scratchpad>/fontes/live12-manual-en.txt` (1.47 MB) | text extracted with PyMuPDF, one `=== p.N ===` block per page |
| `<scratchpad>/fontes/live12-manual-en.toc.txt` (42.5 KB) | the PDF's embedded table of contents, with a page number per section |

`<scratchpad>` = `C:\Users\email\AppData\Local\Temp\claude\D--DRIVE-MNDS-...\6b3d20de-.../scratchpad`.
The PDF and the TXT **do not go into the repository** — they stay in the session scratchpad.

**Chapter numbering.** The original request quoted chapters from an earlier edition. The real numbering in
Live 12 is different, and it is the one used here:

| Original request | Real chapter in Live 12 |
|---|---|
| 3.1-3.2 Control Bar / Status Bar | 3.1 (p.33) and 3.2 (p.34) — matches |
| ch. 4 and 5 Browser | 4 "Working with the Browser" (p.60); 5 is "Managing Files and Sets" (p.120), not browser |
| ch. 6 Arrangement View | 6 (p.160) — matches |
| ch. 7 and 8 Session/launch | 7 "Session View" (p.182), 8 is "Clip View" (p.195); clip launching is **ch. 16** (p.355) |
| ch. 17 Recording | 19 "Recording New Clips" (p.405) |
| ch. 19 Automation | 25 "Automation and Editing Envelopes" (p.491) |
| ch. 20 Clip envelopes | 26 (p.504) |
| ch. 21 Video | 27 (p.517) |
| ch. 27 MIDI and Key Remote | 33 (p.836) |
| ch. 36 Keyboard shortcuts | 41 "Live Keyboard Shortcuts" (p.984) |

**Read**: p.33-35, p.60-63, p.83-84, p.115-119, p.160-181, p.182-194, p.355-362, p.405-415,
p.491-503, p.504-514, p.517-522, p.836-844, p.984-1004.
**Not read**: the audio/MIDI/devices chapters (9-15, 17, 18, 20-24, 28-32), Push (34, 35),
synchronization (36), accessibility (40). Nothing in this digest speaks about them.

---

## 1. Control Bar and Status Bar (3.1-3.2)

The Control Bar is Live's only fixed bar and is split into **nine sections** (p.33-34), in this order:

1. **Browser Options** — show/hide browser toggle + Browser Config Menu (expand to full height, show Tuning and Groove Pool) (p.33).
2. **Tempo Settings and Metronome** — Link, tempo, time signature, metronome, Tempo Follower (p.34).
3. **Scale Settings** — reflects the scale of the selected clip; changes the selected clip and every clip created afterwards (p.34).
4. **Follow and Arrangement Position** — the Follow toggle and the current Arrangement position (p.34).
5. **Transport Controls** — play/stop and the start of Arrangement recording (p.34).
6. **Automation and Capture MIDI** — MIDI overdub, automation *arm*, **re-enable automation** for overridden parameters, Capture MIDI, and the Session record button (p.34).
7. **Arrangement Loop Settings** — enables and configures the loop and the punch-in/punch-out points (p.34).
8. **MIDI and CPU Settings** — Draw Mode, Computer MIDI Keyboard, **Key and MIDI map modes**, sample rate, CPU (p.34).
9. **View Selector** — switches Session ↔ Arrangement (p.34).

The **Status Bar** (p.34-35) shows error and warning; while editing MIDI it shows location, pitch, velocity and
probability of the selected note; **hovering the mouse over an insert marker in the Session or in the
Arrangement shows the exact position of the marker** (p.35). That is: the status bar is the precision numeric
display of whatever the mouse is touching, not just a place for messages.

**For Spellcaster**: sections 6 and 8 are the ones missing from our bar — a fixed place for
*automation arm*, *re-enable automation*, and the **key map / MIDI map** toggles. Section 9 is the
Face switch.

---

## 2. Browser and drag and drop (ch. 4)

### Layout (p.60-61)

Eight elements numbered in the manual:

1. **Sidebar** with the Collections, Library and Places sections and their labels (p.60).
2. **Browse Back / Browse Forward** — history of search and navigation states (p.60).
3. **Search field** (p.61).
4. **Show/Hide Filter** + Filter View menu (which filter groups appear; Tag Editor, Quick Tags, Auto Tags toggles) (p.61).
5. **Filter View** — filter groups with tags (p.61).
6. **Results bar** — appears when searching/filtering; has the **Add Label** button, which saves the result as a custom label, and shows how many filters are applied (p.61).
7. **Content pane** (p.61).
8. **Preview tab** — draws the waveform of samples and clips and plays when Preview is on (p.61).

Resizing: dragging the middle divider line changes the ratio between the two panes;
dragging the right or the bottom edge grows the whole browser — and **dragging the bottom edge automatically
closes the Info View and the Clip/Device View** (p.61). There is a `Full-Height Browser` option in the
View menu that expands to the bottom **without** closing Info View and Clip/Device View (p.61).

### Content pane (p.62-63)

Lists the items of the selected label or of the search result. The Name column is always visible; other
columns come from the **Content Options menu** (p.62). Columns are reorderable by dragging the name (p.63);
clicking the name sorts ascending/descending (p.63); right-click on the name also opens the Content
Options (p.63). That is also where showing extensions is turned on/off (p.63).

Search: terms are combined with **AND**, not OR — "electric bass" finds what has both (p.63).
`Ctrl+F` / `Cmd+F` switches to the "All" label and puts the cursor in the search field (p.63).

### Places (p.83-84)

Labels: **Packs** (Core Library + installed packs + updates), **Splice**, **Cloud**, **Push**,
**User Library**, **Current Project** (all files of the open project), **User Folder** (any folder
on disk added to the browser) and **Add Folder…** (p.83-84).

### Navigation (p.115)

- Scroll: ↑/↓ arrows, mouse wheel, or **dragging while holding `Ctrl+Alt` / `Cmd+Option`** (p.115).
- Open/close folders and move between sidebar and content pane: ←/→ arrows (p.115).
- By default opening a folder closes the previous one; holding `Ctrl`/`Cmd` keeps both open (p.115).

### Preview (p.115-118)

The Preview toggle sits next to the Preview Tab, at the bottom of the browser (p.115). With it on, selecting
a file already plays it; ↑/↓ walks through files listening (p.116). **Even with the toggle off**,
`Shift+Enter` or the → arrow previews the selected item (p.116). During preview the waveform appears in the
Preview Tab and you can click the scrub area to jump (p.116) — **you cannot scrub a clip saved
with Warp off** (p.116). The **Raw** button: off (default), Live plays the file at the start of the
next bar and looped, synced to the project; on, it plays at the original tempo, without loop and without
scrub (p.117). Preview volume: Preview/Cue knob of the Main track (p.117); with 4 outputs you can preview
on headphones while the music continues on the main output (p.118).

### What each drop creates (p.118-119)

- **Dragging onto a track** (Session or Arrangement) puts the item in it (p.118).
- **Dragging into the space to the right of the Session tracks, or below the Arrangement tracks, creates a new track** with the item inside (p.118).
- **In the Session**, double-click or `Enter` on a *device* from the browser loads it into the selected track; on a *sample*, it loads into a **Simpler** if the track is MIDI, or **into a clip slot** if the track is audio (p.118-119).
- **In the Arrangement**, double-click or `Enter` on a device or sample loads it into the selected track (p.119).
- Files can also be dropped straight from Explorer/Finder (p.119).
- Dragging an **instrument or MIDI effect** onto the *Mixer Drop Area* below the Arrangement tracks creates a **MIDI track**; dragging an **audio effect** creates an **audio track** (p.162).
- Dragging **several clips** at once: by default Live queues them all into a single track (vertically in the Session, horizontally in the Arrangement); holding `Ctrl`/`Cmd` **before dropping** spreads them across several tracks (p.190). It applies to audio and raw MIDI, **not** to Live Clips, because those can carry embedded devices (p.190).

---

## 3. Arrangement View (ch. 6)

### Layout (p.160-162)

Numbered in the manual, 16 elements. The ones that matter:

1. **Overview** — shows the whole arrangement; the black outline is the visible part. **Dragging horizontally scrolls; dragging vertically zooms**; double-clicking inside the outline returns to the whole arrangement (p.160-161).
2. **Beat-time ruler** — time in bar-beat-sixteenth; dragging horizontally scrolls, dragging vertically zooms; **double-click zooms to the current selection, and with no selection returns to the whole arrangement** (p.161).
3. **Scrub area** — clicking starts playback from there; **holding the mouse button on a point makes that stretch play in a loop, at the global quantization** (p.161).
4. **Locators** — launchable markers, addable at any point of the scrub area, to organize the piece into launchable sections (p.161).
5. **Set Locator** — creates a locator during playback or recording; when a locator is selected the same button becomes **Delete Locator** (p.161).
6. **Previous / Next Locator** — the jump between locators is quantized by the global launch quantization (p.161).
7. **Automation Mode toggle** — shows/hides the automation lanes (p.161).
8. **Lock Envelopes** — locks the envelopes to the song position instead of to the clip; lets you move clips without moving the automation (p.161).
9. **Main lane** of each track, where the clips live (p.162).
10. **Arrangement Track Controls** — volume, pan, I/O; which ones appear is chosen in the Arrangement Track Controls submenu of the View menu (p.162).
11. Tracks stacked vertically, reorderable by dragging above/below (p.162).
12. **Mixer Drop Area** below the tracks (see §2) (p.162).
13. **Optimize Height / Optimize Width** — fit all tracks into the current height or width; keys `H` and `W` (p.162).
14. **Waveform Vertical Zoom Level** — slider that grows only the waveform drawing, without touching the gain; applies to all audio tracks and to new clips recorded afterwards (p.162).
15. **Time ruler** — a second ruler, in minutes-seconds-milliseconds; dragging scrolls (p.162).
16. **Mixer** — opens from the View menu or from the toggle in the bottom right corner (p.162).

Two simultaneous time rulers (bar and clock) is the detail Spellcaster needs: a show
has cues in clock time and keyframes in frames.

### Navigation and zoom (p.163)

- Progressive zoom around the selection: keys `+` and `-`, or **mouse wheel with `Ctrl`/`Cmd`** (p.163).
- **Panning the display: dragging while holding `Ctrl+Alt` / `Cmd+Option`** (p.163).
- `Z` = full zoom on the time selection; `X` steps one zoom level back, and can be pressed several times to undo several `Z` (p.163).
- Selecting time inside an Arrangement clip **makes the Clip View editor zoom to the same time** (p.163).
- **Vertical zoom of one track: mouse wheel with `Alt`/`Option` inside the main lane.** If there is a time selection, all tracks with selected content zoom vertically together (p.163).
- **Follow**: turned on in the Control Bar or in the Options menu. **It pauses on its own** if you edit, scroll horizontally, or click the beat-time ruler; **it comes back** when you stop/restart playback or click the Arrangement or the clip's scrub area (p.163).

### Transport (p.163-165)

Play/Stop in the Control Bar or the space bar (p.163). The blinking blue **insert marker** determines where
playback starts (p.164). Clicking anywhere on a track moves the insert marker (p.164).
Double-clicking Stop, or `Home`, returns the insert marker to the beginning (p.164). `Shift+Space` continues from
where it stopped instead of returning to the insert marker (p.164). The Arrangement Position fields accept vertical
dragging, typing + `Enter`, and ↑/↓ arrows — and touching them moves the insert marker (p.164).

**Permanent Scrub Areas** is on by default (Display & Input Settings): clicking anywhere in the scrub
area plays from there (p.164). The jump between points is quantized by the Control Bar quantization (p.165).
Even with the option off, `Shift`+click on the scrub area or on the beat-time ruler still scrubs (p.165).

### Locators (p.165-167)

- **Set Locator** works with the transport running (quantized) and stopped (creates at the insert marker or at the start of the selection) (p.166). There is also "Add Locator" in the scrub area's context menu and in the Create menu (p.166).
- Jumping: clicking the locator, or Previous/Next. Past the first/last locator, the buttons jump to the **beginning/end of the arrangement** (p.166).
- **Locators can be triggered by MIDI/key mapping** (p.167).
- With the transport stopped, **double-clicking a locator selects it and starts playing from there** (p.167).
- Moving: dragging or arrows. Renaming: `Ctrl+R`/`Cmd+R` (p.167). There is a dedicated **info text** per locator (Edit Info Text) (p.167).
- **Loop to Next Locator** in the context menu loops between two locators with one command (p.167).
- **Set Song Start Time Here** overrides the default "playback starts at the selection" behavior: with it checked, playback starts at that locator (p.167).

### Loop brace (p.169-170)

Toggle in the Control Bar. With no selection, the brace covers the whole arrangement (p.169). Numeric fields
**Loop Start** and **Loop Length** in the Control Bar (p.169). `Ctrl+L`/`Cmd+L` = Loop Selection: turns the loop on
and puts the brace on the current time selection; with a clip or time selected the same key also toggles
the loop (p.169).

Adjusting the brace (p.170):

- `←` / `→` — push the brace by the grid step;
- `↑` / `↓` — shift the brace by one loop length;
- `Ctrl`+`←`/`→` — shorten/lengthen by the grid step;
- `Ctrl`+`↑`/`↓` — double/halve the length;
- dragging the left/right edge changes start/end; dragging the bar moves it without changing the length.

The brace's context menu also has **Set Song Start Time Here** (p.170).

### Editing clips

**Moving and resizing** (p.170-171): dragging the clip changes position and track; dragging the
left/right edge changes the length. **Only the clip bar is draggable — you cannot drag by the
waveform or by the MIDI drawing** (p.171). Clips snap to the grid **and** to the edges of other clips,
locators and time signature changes (p.171). To slide the content inside the clip:
`Ctrl+Shift` (Win) while dragging the waveform; to ignore the grid in that gesture, `Ctrl+Alt+Shift` (p.171).

**Fades and crossfades** (p.172-174): the handles sit at the edges of audio clips and **only appear if the
track is tall enough**; if the track is folded or small, you have to increase its height (p.172).
With Automation Mode on, **holding `F`** momentarily shows the fade controls while the mouse
is over an automation lane (p.172). Fade In Start / Fade Out End change the duration without touching the peak;
the edge never goes past the peak; the **Fade Curve handle** shapes the curve (p.172). **Fades are a property of the
clip, not of the track, and are independent of the automation envelopes** (p.174).

**Selection** (p.174-175):

- clicking a clip selects the clip;
- clicking the background places an **insert marker**, movable with ←/→ (time) and ↑/↓ (between tracks). **`Ctrl`+←/→ makes the insert marker jump to locators and clip edges** of the selected track(s) (p.174);
- clicking and dragging selects a time range;
- to reach the time *inside* a clip, you have to unfold the track (p.174);
- dragging inside the waveform selects time inside the clip (p.174);
- **clicking the loop brace is equivalent to Select Loop** (selects everything inside the loop) (p.175);
- `Shift`+click extends the selection on the same track or across tracks; `Shift`+arrows extends/shrinks (p.175);
- **`0` deactivates the selection** of material, even with several clips; with the track header selected, `0` deactivates the track (p.175);
- `R` reverses the audio selection (does not work if there is a MIDI clip in the selection) (p.175);
- ←/→ nudge the selection (p.175).

**Fold, height, unfold** (p.174): the unfold button sits next to the track name; `U` unfolds the
selected tracks. The height changes by dragging the divider line below the button, or with
`Alt`+`+` / `Alt`+`-`. Holding `Alt` and pinching on the trackpad also resizes.
**Holding `Alt` while resizing one track resizes all of them.** `Alt`+click on the unfold
button, or `Alt+U`, unfolds all (p.174).

**Grid and snap** (p.175): the grid can be **zoom-adaptive or fixed**; the width of both is set in the
context menu of the main lane or of the MIDI Note Editor.
`Ctrl+1` narrows (doubles the density), `Ctrl+2` widens, `Ctrl+3` toggles triplets, `Ctrl+4` toggles
snap, `Ctrl+5` switches fixed/adaptive. The current spacing appears **above the time ruler, in the bottom
right corner** (p.175). **`Alt` (Win) / `Cmd` (Mac) held during an action ignores the snap; with the
grid off, the same modifier turns the snap on temporarily** (p.175).

**"…Time" commands** (p.176): unlike normal Cut/Copy/Paste, they act on **all tracks**,
inserting and removing time, and they affect the time signature markers in the stretch.
Cut Time, Paste Time, Duplicate Time, Delete Time, Insert Silence.

**Split** (p.176): clicking inside the waveform / MIDI drawing and `Ctrl+E`, or the Split item in the
context menu, splits the clip there. Dragging over a range and using the same command isolates that stretch
as a new clip.

**Consolidate** (p.177): joins the selected material of several adjacent clips into a new clip; it works
per track and across several tracks. When consolidating audio, it **creates a new sample per track**, recording the
output of the warp engine **before** the effects chain and the mixer — the sample carries clip attenuation,
warp, pitch and clip envelopes, but **not** the effects (p.177). The samples go to the project's
`Samples/Processed/Consolidate` (p.177).

**Linked-track editing** (p.178-180): tracks can be linked with "Link Tracks" in the context menu of the
header, including inside a Group Track (p.178). Each track belongs to only one link instance
(p.179); hovering the indicator highlights the linked tracks; clicking the indicator selects all of them
(p.179). What then applies simultaneously (p.179-180): moving and resizing clips; selecting clips
and time; "…Time" commands; split and consolidate; creating and editing audio fades (**only fades that start at
the same time position**); arming and disarming tracks; renaming/inserting/deleting take lanes.

**Colors**: in the Arrangement the manual does not describe track color; in the Session, the clip context menu and the
scene one have a **color palette** (p.184, p.186), and the browser accepts `1`…`7` to color items and `0` to
reset (p.1002).

---

## 4. Session View and clip launching (ch. 7 and 16)

### Grid, slots, scenes (p.183-191)

- Every Session clip has a **triangular button** on the left edge. Clicking launches it; or pre-select it by clicking the name and launch with `Enter`; afterwards you walk through the neighbors with the arrows (p.183).
- The square **Clip Stop** stops the running clip, either in the slot or in the Track Status field below the grid (p.183).
- `0` deactivates the selected clip(s) (p.183).
- Clips can be mapped **to MIDI note ranges to be played chromatically** (p.183; how to do it: p.841).
- The clip layout **does not determine the order**: the grid is random access (p.183).
- Even with every clip stopped, the Control Bar's Play stays lit and the position fields keep running — musical time is continuous, independent of what the clips do (p.183). Two clicks on Stop return to 1.1.1 and stop everything (p.184).
- Renaming: Edit menu or context menu; **several clips at once**; there is **info text** per clip; and the context menu has a color palette (p.184).
- Reordering by dragging; adjacent multiple selection with `Shift`, non-adjacent with `Ctrl` (p.184).
- **Group Track slots** show a shaded area when some track of the group has a clip in that scene, in the color of the leftmost clip; the group slot has a launch button that fires all the clips, and turns into stop when there are no clips (p.184).
- **Session tracks are resizable in width** by dragging the title edge, down to only the launch button being left; `Alt` while resizing resizes all of them (p.185).
- `0` over a track header deactivates the track (p.185).
- **Scene** = horizontal row. The Scene Launch sits in the rightmost column (Main track). "Cancel Scene Launch" exists in the Main track's context menu (p.185).
- **The scene below the launched one is automatically selected as the next one**, unless "Select Next Scene on Launch" is Off (p.185).
- Rename scenes with `Tab` to jump to the next one (p.185-186); info text and color palette per scene (p.186).
- Dragging a selection of **non-adjacent** scenes collapses them together; to move without collapsing, `Ctrl`+↑/↓ (p.186).
- **The scene number is positional**: moving the scene changes the number (p.186).
- **Scene Tempo and Scene Time Signature** appear by dragging the left edge of the Main track header; hidden by default. The project adjusts to them when the scene is launched (p.186). A scene with an assigned tempo/time signature has a **colored launch button** (p.187).
- **Scene View** (p.187-189): the scene's property panel — tempo, time signature and **the scene's Follow Actions**. It opens when you select scene(s), click a scene tempo control, or click the Main track title. With several scenes selected, the title shows the count instead of the name.

**Track Status fields** (p.189-190) — the per-track state indicator:
pie icon = looping clip, with the loop length in beats on the right and the number of repetitions
played on the left; progress bar = one-shot clip, with the remaining time in minutes:seconds;
microphone = audio track monitoring input, keyboard = MIDI track monitoring; a miniature of the arrangement
= the track is playing the Arrangement (p.189-190).

**Stop buttons**: `Ctrl+E` adds/removes the Clip Stop of a slot. Removing the stop from the scene 3/track 4 slot
is how you pre-configure "scene 3 does not touch track 4" (p.191).

**Insert Scene** (`Ctrl+I`) inserts an empty scene below the selection. **Capture and Insert Scene**
(`Ctrl+Shift+I`) inserts a new scene below, **copies the clips playing right now into it and launches
the new scene with no audible interruption** (p.191). This is the "capture" our `gravar-dmx` wants for scenes.

**Session → Arrangement** (p.192-194): with Arrangement Record on, Live records into the arrangement the launched
clips, changes to the properties of those clips, mixer and device changes (automation) and
tempo/time signature changes (p.192). **The recording creates no new audio, only clips** (p.192). Session and Arrangement of
the same track are mutually exclusive: launching a Session clip takes over that track's Arrangement; and
**"Back to Arrangement"** — a button that lights up to remind you that what you hear differs from the arrangement — gives it back
(p.192-193). "Stop All Clips" on the Main track deactivates all Arrangement clips (p.193).
You can also move clips between the two views by copy/paste, by dragging over the view selectors,
or by dragging between windows with Second Window (`Ctrl+Shift+W`) (p.194).
**Consolidate Time to New Scene** (Create menu or Arrangement context menu) consolidates the selected time
range into one clip per track and drops it into a new scene (p.194).

### Launch modes (ch. 16, p.355-362)

The launch controls are in the **clip panel with the launch button icon**, and they **only apply to Session
clips** — Arrangement clips are not launched, they play by position (p.355). You can edit the launch of
several clips at once by selecting them first (p.356).

**Launch Mode** (p.356) — four modes, valid for mouse, key and MIDI note:

| Mode | Behavior |
|---|---|
| Trigger | *down* starts; *up* ignored |
| Gate | *down* starts; *up* stops |
| Toggle | *down* starts; *up* ignored; the next *down* stops |
| Repeat | while held, it fires repeatedly at the clip's quantization rate |

**Legato** (p.357): the launched clip takes over the playback position of the previous clip of that track — it lets you
switch clips at any moment without losing sync, even with quantization off.

**Clip Launch Quantization** (p.358): per clip; "None" turns it off; "Global" uses the Control Bar's one
(`Ctrl+6..0`). Any value other than "None" also quantizes triggering coming from a Follow Action.

**Velocity Amount** (p.359): how much the MIDI note velocity affects the clip volume; 0 = no
influence, 100% = the weakest notes play in silence.

**Nudge** (p.359-360): Backward/Forward buttons jump inside the clip in increments of the size of the
global quantization. **They are mappable**; and **in MIDI Map Mode a scrub control appears between them**, which
can be assigned to a rotary encoder for continuous scrubbing (p.360).

**Follow Actions** (p.361-362): they define what happens to the other clips of the same *group* (clips in
successive slots of the same track, separated by empty slots) after the clip plays. They also exist for
scenes, in the Scene View. Controls: enable button (`Shift+Enter`), two choosers A and B, **Chance A/Chance
B** in percent with a slider between them, **Linked/Unlinked** switch (Linked = fires at the end of the clip or
after N loops; Unlinked = after the Follow Action Time), and **Follow Action Time** in bar-beat-sixteenth,
with a draggable marker in the editor (p.361-362). The ten actions: No Action, Stop, Play Again, Previous,
Next, First, Last, Any, Other, **Jump** (with a target slider, to choose slot or scene) (p.362).
Clips and scenes with a Follow Action have a **striped launch button** (p.362). Follow Actions **bypass
the global quantization but not the clip quantization** (p.362).

---

## 5. Recording (ch. 19)

- **Input choice** (p.405-406): the track records whatever is on its In/Out (View menu → In/Out). In the Arrangement, you have to unfold and resize the track to see the whole section. An audio track records mono from external input 1 or 2 by default; a MIDI track records all MIDI from the active input devices; **the computer keyboard can be enabled as a pseudo-MIDI device** (p.405).
- **Arm** (p.406): clicking the Arm of a track **disarms all the others**, unless `Ctrl`/`Cmd` is held. With several tracks selected, arming one arms them all. **Arming selects the track.** Armed tracks are monitored by default ("auto-monitoring"). On a natively supported control surface, arming a MIDI track locks the surface to that track's instrument (p.406).
- **Arrangement Record** (p.407): the behavior depends on "Start Playback with Record" (Record, Warp & Launch Settings) — on, it records as soon as you press it; off, it only records when Play is pressed or a Session clip is launched. **`Shift` on Arrangement Record inverts the behavior** (p.407). The recording creates new clips on all armed tracks (p.407).
- **MIDI Arrangement Overdub**: the new clip mixes what was already there with the new input; **it only applies to MIDI tracks** (p.407).
- **Punch-In / Punch-Out** (p.407): dedicated switches. **The punch-in is the start position of the Arrangement Loop and the punch-out is the end.** It protects what you do not want to re-record and gives you pre-roll time.
- **Loop recording** (p.407): recording inside the loop, Live keeps the audio of **every pass**. You can "unroll" it with repeated Undo or graphically: double-clicking the new clip shows in the Sample Editor a long sample with everything that was recorded; the Clip View loop brace delimits the last pass, and moving the markers to the left auditions the earlier ones (p.407).
- **Recording into Session slots** (p.408): global quantization other than "None" so the clips come out cut right; arm the tracks (**Clip Record buttons** appear in the empty slots); **Session Record** records into the selected scene on all armed tracks — the launch button turns red while recording; pressing Session Record again goes straight from recording into loop; alternatively clicking a Clip Record records only into that slot, and that clip's launch button closes the recording. **The "New" button** stops the clips of all armed tracks and selects (or creates) a scene for the next take — **and it only exists in Key Map Mode and MIDI Map Mode** (p.408).
- By default launching a scene does **not** trigger recording in the armed empty slots; "Start Recording on Scene Launch" changes that (p.408-409).
- **Overdub MIDI** (p.409): with global quantization at 1 bar and a Record Quantization chosen, double-clicking a slot creates an empty 1-bar clip; arm; Session Record; the clip overdubs on every pass, layer by layer; pressing Session Record again pauses the recording without stopping playback, and the next press resumes recording. **`Alt`+double-click on the empty slot already arms the track and launches the clip** (p.409).
- **Step recording** (p.409-410): with the transport stopped, hold notes on the controller and press `→` to advance the insert marker by the grid step, inserting the notes; keep holding and press `→` again to extend the duration; `←` erases what was just recorded. **The step recording navigators are MIDI-mappable** (p.410, p.415).
- **Metronome** (p.411-412): volume via the Preview Volume knob; drop-down menu next to the switch with count-in, tick sound, **Rhythm** (beat division; "Auto" follows the time signature denominator; divisions that do not fit the bar appear disabled) and **Enable Only While Recording** (it stays highlighted with the transport running but only sounds while recording; with Punch-In active, only after the punch point) (p.412).
- **Record Quantization** (p.412): in the Edit menu; recording into the Arrangement, quantization is **a separate step in the undo history** — you can undo only the quantization and keep the recording. It cannot be changed mid-recording (Session or Arrangement); in overdub with the Clip View loop active, it changes on the fly and is not undone separately.
- **Remote control of recording** (p.414-415): mappable are the Arrangement Record, the transport controls, the per-track Arm buttons, the Session Record, the New button, the individual slots, the relative navigation controls (Scene Up/Down) and the step recording navigators. The manual gives the standard: one key to jump scene and another to start/end recording on that track (p.414-415).

---

## 6. Automation and envelopes (ch. 25)

**Recording in the Arrangement** (p.491): two ways — changing parameters by hand while recording, or recording a
Session performance that contains automation. In a Session→Arrangement recording, the automation of the Session
clips **always** goes to the Arrangement, along with the manual changes on the tracks being
recorded. For direct manual changes, what rules is the **Automation Arm**: with it on, every control
change during Arrangement Record becomes automation (p.491). The automated control gets an **LED
on the slider thumb**; on pan and Track Activator the LED appears in the top left corner (p.491).

**Recording in the Session** (p.492-494): turn on Automation Arm; arm the tracks; Session Record. There is a
**Session Automation Recording** switch in the Settings that records automation into **every playing clip**,
armed or not — that way you can overdub automation into an existing MIDI clip without recording notes (p.493).
Session automation **becomes track automation** when the clips are recorded or copied into the
Arrangement (p.494).

**Automation recording modes in the Session** (p.494): with the **mouse**, recording stops the instant
the button is released — "touch" behavior. With a **MIDI controller knob or fader**, recording continues
while the control is being moved and, on release, runs to the end of the clip loop and punches out on its own —
"latch" behavior.

**Deleting** (p.494): right-click on the control → Delete Automation, or `Ctrl+Backspace`. The LED disappears and the
value stays constant across the whole timeline and in every Session clip.

**Override and Re-Enable** (p.494-495): touching an automated control **outside** recording turns the LED off —
that control's automation becomes inactive and the manual value rules. Whenever any control is in that
state, the Control Bar's **Re-Enable Automation** button lights up; clicking it returns everything to what is
recorded (p.494). You can re-enable **a single parameter**, from its context menu; and in the Session,
**relaunching the clip that contains the automation already re-enables it** (p.495).

**Automation lanes** (p.495-496), numbered in the manual:

1. **Automation Mode**: toggle above the track headers, or the `A` key. `A` again turns it off (p.495).
2. Clicking a mixer or device control of the track **shows that control's envelope on the clip's track** (p.495).
3. The envelopes appear in the **main lane, "on top of" the waveform / MIDI drawing** — good for aligning breakpoints with the content. Vertical axis = value, horizontal = time. For switches and radio buttons the value axis is **discrete** (p.495).
4. **Device chooser**: chooses the track mixer, a device, or "None" to hide. **It has an LED next to the devices that have automation**, and the "Show Automated Parameters Only" option (p.495-496).
5. **Automation Control chooser**: chooses the control inside the device; automated controls have an LED (p.496).
6. Button that **moves the envelope into its own lane below the clip**, freeing the choosers to see another parameter at the same time. `Alt` + that button moves **the selected one and all the automated ones** into their own lanes. If the Device chooser is at "None", the button disappears (p.496).
7. Button that **hides the lane** — hiding **does not deactivate** the envelope. `Alt` + that button removes the selected lane and all the following ones of that track (p.496).
8. Toggle that shows/hides all the extra lanes (p.496).

Right-click on a lane header opens extra display options and **commands to clear all the
automation of the track or of a device** (p.496). `←` from a lane returns to the main track and
folds all the lanes; `←`/`→` on the main track fold/unfold the lanes (p.496).

**Draw Mode** (p.496-497): Options menu, switch in the Control Bar, or the `B` key. **Holding `B` while editing
with the mouse turns Draw Mode on momentarily** (p.496). Drawing creates steps the width of the visible grid;
`Shift` while dragging vertically gives fine resolution on the step value; hiding the grid (`Ctrl+4`) gives free
drawing, and **`Alt` held during drawing gives temporary free drawing with the grid visible** (p.497).

**Breakpoints** (with Draw Mode off) (p.497-499):

- clicking on a segment creates a breakpoint there; **double-clicking anywhere on the background creates a breakpoint there**; clicking a breakpoint deletes it (p.497);
- the numeric value appears on creation, on hover and while dragging; hovering over a selected segment, it shows the value of the breakpoint closest to the cursor (p.497);
- dragging a breakpoint that is inside the selection moves **all** of the ones in the selection along with it; a thin black vertical line shows the position relative to the grid (p.498);
- right-click on the breakpoint → **Edit Value** to type an exact value; with several selected, all of them move relatively. **Add Value** creates a breakpoint with an exact value from a preview breakpoint (p.498);
- clicking near a segment (or `Shift`+click on it) selects the segment and lets you drag it; if the segment is inside the time selection, Live **inserts breakpoints at the selection edges** and moves the whole segment (p.498);
- a breakpoint created near a grid line **snaps to it**; `Alt` while dragging horizontally ignores the snap; breakpoints and segments also snap to neighboring breakpoint positions, and **continuing to drag "over" a neighbor removes it** (p.498);
- `Shift` while dragging constrains the movement to one axis; `Shift` vertically gives fine resolution (p.498-499);
- **`Alt` while dragging a segment curves the segment; `Alt`+double-click returns it to a straight line** (p.499).

**Stretching and skewing** (p.499-500): hovering the mouse over a time selection reveals handles at the edges.
The top and bottom handles stretch on the vertical axis (a rectangle shows how much; it snaps at the limits and
when the corners cross; `Shift` refines; going past the limits **clips** the envelope). The middle
left/right handles stretch horizontally (dragging over breakpoints outside the selection **removes** them;
`Shift` moves them proportionally; `Alt` ignores the snap). The corner handles **skew**; `Alt` mirrors the
movement on the opposite handle.

**Simplify Envelope** (p.500): over a time selection, it computes the optimal number of breakpoints and removes
the unnecessary ones, replacing them with straight lines or curves. It is the right command after recording automation.

**Automation Shapes** (p.501): right-click on a time selection → shape. Top row:
sine, triangle, sawtooth, inverted sawtooth, square — scaled horizontally to the
selection and vertically to the parameter range; **with no selection, they scale to the grid size**.
Bottom row: two sets of ramps and an **ADSR** — these **link to the automation value before or
after the selection**, indicated by the dotted line.

**Lock Envelopes** (p.502): normally moving an Arrangement clip moves the automation with it; the switch
(Control Bar or Options menu) locks the envelopes to the song position.

**Edit commands inside lanes** (p.502): Cut/Copy/Duplicate/Delete applied to a selection **inside
a lane** only affect that envelope — the clip and the other automations at the same time stay intact.
You can work on several lanes at once. For the edit to reach the clip **and** all the
envelopes, Lock Envelopes has to be off and the selection has to be on the clip's track (p.502).
**Copying and pasting envelope movement from one parameter to another is allowed**, even between unrelated
parameters (p.502).

**Tempo is automation like any other** (p.502-503): unfold the Main track, Device chooser = "Mixer",
Control chooser = "Song Tempo". The two fields below the choosers scale the value axis (minimum and
maximum in BPM) — **and those two fields also determine the range of a MIDI controller assigned to the
tempo** (p.503).

---

## 7. Clip envelopes — only the difference (ch. 26)

A **clip envelope** lives inside the clip, in the Envelopes tab of the Clip View, with the same two choosers
(Device / Control) and the same drawing and breakpoint gestures as automation (p.504-505).
The differences that matter:

- **Automation sets the absolute value; modulation only influences that value.** That is why both coexist on the same parameter. Automation is drawn in **red**, modulation in **blue**; on a knob, automation moves the needle and modulation appears as a blue segment on the ring (p.508).
- **A Session clip has both** (two toggles, Automation and Modulation, below the choosers). **An Arrangement clip only has modulation** — its automation lives in the track lane (p.505, p.509).
- LEDs in the Control chooser: red = it has automation, blue = it has modulation, both = both (p.509-510).
- A clip envelope is **non-destructive**: hundreds of clips can use the same sample and sound different (p.505).
- Deleting: right-click in the envelope editor or `Ctrl+Backspace` → Clear Envelope (p.505).
- "MIDI Envelope Auto-Reset" (Options menu) resets certain MIDI control messages at the start of each clip (p.505).
- **Two envelopes affect volume**: Clip Gain and Track Volume; the second is the mixer's gain stage, therefore post-effect. A small dot below the slider thumb shows the real modulated volume (p.510).
- **MIDI Controller clip envelopes** (p.513): Device chooser = "MIDI Ctrl", Control chooser = the controller number. It supports up to controller 119. Controllers that already have an envelope show an LED.
- **Unlink** (p.513-515): the clip envelope can have its **own loop/region, independent of the clip**. On unlinking, the envelope's loop braces become colored and the loop/region controls of the Envelopes tab become active: the sample can keep looping while the envelope plays "one-shot", and vice versa. It serves to program an 8-bar fade-out over a 1-bar loop (p.514), to turn a short loop into a long one by superimposing an 8-bar looped envelope (p.514), and to use the envelope as an LFO (p.515).

---

## 8. Video (ch. 27)

- Format: **QuickTime (.mov) only** (p.517). The files appear in the browser and come in by dragging.
- **Live only draws video for clips that are in the Arrangement.** A movie file loaded into the Session is treated as an audio clip (p.517).
- In the Arrangement a video clip is just like an audio clip, except for the "sprocket holes" on the title bar (p.517). It can be trimmed by dragging the edges. **Consolidate, Reverse and Crop replace the video clip with an audio clip** (internally; the original file is never altered) (p.518).
- **Video Window** (p.518): a separate floating window that stays **always above Live's main window**, never covered. Draggable, resizable from the bottom right corner, with visibility in the View menu. **The size and position do not belong to the Set** — they are restored the next time a video is opened. Double-click = full screen (optionally on a second monitor); `Alt`+double-click returns to the video's original size.
- A stretch with no video in the file = black screen; a stretch with no audio = silence (p.519).
- **Tempo Leader** (p.519): when scoring video, the video clip is the *tempo leader* and the audio clips are *followers* — this is the default for Arrangement clips. In that arrangement, **the video clip's Warp Markers define the "hit points"** the music syncs to. Warp on the video clip has to be on for it to be able to be leader. Only the **lowest** playing leader clip is the effective leader (p.519).
- **Dragging a Warp Marker updates the Video Window to the corresponding frame** (p.519) — this is video scrubbing by marker. The **QuickTime markers embedded in the file are displayed by Live** and serve as a visual reference (p.519).
- Full workflow in seven steps (p.520): `Tab` switches the views on a single monitor; dragging the .mov onto an audio track makes the Video Window appear; dragging audio onto the drop area creates a track; unfold both; in the video's Clip View turn Warp on and set Leader; add Warp Markers; optionally turn on the Arrangement Loop over a section; and **Export Audio/Video** exports audio and video together.
- **Pre-roll trick** (p.520-522): movies arrive with a "two-beep" before the action. You drop the movie at 1.1.1, drag the clip's **Start Marker** to the right until the beginning of the action — action and music then start at 1.1.1 / 00:00:00:00. At the end, you select everything, drag the composition a few seconds to the right, select only the video clip and drag its **left edge** to the left, revealing the pre-roll back. Since Export uses the duration of the Arrangement selection by default, the file comes out with the exact duration of the original.

---

## 9. MIDI and Key remote control (ch. 33)

**What is mappable** (p.836): Session slots (**the assignment belongs to the slot, not to the clip in it**),
switches and buttons (Track/Device Activator, tap tempo, metronome, transport), **radio buttons** (a group of
mutually exclusive options, e.g. crossfader assignment), continuous controls (volume, pan, sends) and
the crossfader.

**Important note** (p.836): a MIDI key used in a mapping **stops playing the instrument** of the MIDI
track — it now belongs only to the mapped control.

**Control surfaces** (p.837-839): up to six simultaneously, defined in Link, Tempo & MIDI (`Ctrl+,`).
Natively supported surfaces get **Instant Mappings** — automatic mappings by control family, which
**remap themselves to the selected device** (p.837). Instant Mappings **do not
appear in the Mapping Browser** (p.840). A surface can be **locked to a device** (right-click on the
device title bar → "Lock to…"), and a **hand icon** on the title bar marks the locked device;
by default, arming a MIDI track locks the surface to its instrument (p.838). Unsupported surface:
just turn on the input port's **Remote** switch in the MIDI Ports table; any number of ports can be
used, and Live mixes the MIDI from all of them (p.838-839). For a surface with feedback (motorized fader,
LED) you also have to turn on Remote for the **output** port (p.839).

**Takeover Mode** (p.839-840) — what to do when the physical value and the on-screen value diverge
(bank switching):

| Mode | Behavior |
|---|---|
| None | the new value goes straight to the destination; abrupt jump |
| Pick-Up | moving does nothing until the physical control reaches the destination value; from there it follows 1:1. Smooth, but it is hard to guess where the pick-up happens |
| Value Scaling | compares the two values and computes a smooth convergence as the control is moved; when they match, it follows 1:1 |

**Mapping Browser** (p.840): hidden until one of the three mapping modes is turned on, and then it lists
**the mappings of the current mode**. Each row has: the control element, the **path** to the mapped
parameter, the parameter name, and **Min and Max**. The ranges are editable at any time and there is a
context command to **invert** them (Min > Max). Deleting a mapping: `Backspace`.

**Making the MIDI mapping** (p.841): 1) `Ctrl+M` turns on MIDI Map Mode — the mappable elements turn
**blue** and the Mapping Browser appears (`Ctrl+Alt+B` opens the browser if it is closed); 2) click the
parameter; 3) send the MIDI message; 4) `Ctrl+M` exits.

**MIDI notes** (p.841): on Session slots, Note On/Off act according to the clip's **Launch Mode**;
on switches, Note On toggles the state; on radio buttons, Note On cycles through the options; on variable
parameters, **a single note** toggles between Min and Max, and **a note range** distributes discrete values
evenly spaced across the parameter range. A slot can be mapped to a **note range to be played
chromatically**: you play the root key first (which plays the clip at the default transposition) and, holding it,
one key below and one above to define the limits (p.841).

**Absolute 0-127 controllers** (p.841-842): on slots, value ≥64 = Note On, ≤63 = Note Off;
on **track activators and device on/off buttons**, values **inside** the Min-Max range turn on and values
outside turn off — **and setting Min greater than Max inverts that**; on the other switches (transport etc.), ≥64
turns on and <64 turns off; on radio buttons the 0-127 range is mapped across the options; on continuous controls,
across the parameter range. Pitch bend and **14-bit** controllers (0-16383) are also supported,
centered at 8191/8192 (p.842).

**Relative controllers** (p.842-843): four types — Signed Bit, Signed Bit 2, Bin Offset, Twos
Complement — each of them also in "linear" mode. Live tries to detect the type and whether there is acceleration;
**moving the control slowly to the left while creating the assignment improves the detection**, and the type appears
in the "mode" chooser of the Status Bar, editable by hand (p.842). On slots, increment = Note On and decrement =
Note Off; on switches, increment turns on and decrement turns off; on radio buttons, it advances/goes back one option
(p.843).

**Relative Session navigation** (p.843) — in both mapping modes **a strip of assignable controls
appears below the grid** of the Session, with five numbered targets:

1. buttons to move the highlighted scene up/down;
2. a numeric scene field, ideal for an **endless encoder**, to run through the scenes;
3. a button to launch the highlighted scene (with "Select Next Scene on Launch" on, it advances by itself);
4. a button to cancel the launch of a triggered scene;
5. buttons to launch the clip of the highlighted scene, per track.

The gain: **Live keeps the highlighted scene always in the center of the Session View**, so a large Set is
navigable with few controls (p.843).

**Clip View** (p.843-844): the Clip View shows the selected clip — and multi-selection too. Therefore
**mapping a Clip View control can affect any clip in the Set**; the manual recommends using
**relative** controllers for those controls, to avoid jumps.

**Computer keyboard** (p.844): `Ctrl+K` turns on Key Map Mode; the mappable elements turn **red**
(against the blue of MIDI). Same four steps. Effects: on slots, according to the Launch Mode; on switches,
it toggles; on radio buttons, it cycles through the options. Not to be confused with the **Computer MIDI Keyboard** (`M` key),
which generates notes.

---

## 10. Keyboard shortcuts (ch. 41) — copied tables

Windows column only. Source of each block: the section cited.

### 41.1 Showing and hiding views (p.984-985)

| Action | Windows |
|---|---|
| Full screen | `F11` |
| Second window | `Ctrl+Shift+W` |
| **Toggle Session/Arrangement** | `Tab` |
| Toggle Device/Clip View | `Shift+Tab` or `F12` |
| Show Device **and** Clip View | `Alt`+click on the view selector |
| Hot-Swap | `Q` |
| Drum Rack / last pad | `D` |
| Info View | `Shift+?` |
| **Video Window** | `Ctrl+Alt+V` |
| Browser | `Ctrl+Alt+B` or `Ctrl+Alt+5` |
| Overview | `Ctrl+Alt+O` |
| In/Out | `Ctrl+Alt+I` |
| Sends | `Ctrl+Alt+S` |
| Mixer | `Ctrl+Alt+M` |
| Clip View | `Ctrl+Alt+3` |
| Device View | `Ctrl+Alt+4` |
| Groove Pool | `Ctrl+Alt+6` |
| Learn View | `Ctrl+Alt+7` |
| Open Settings | `Ctrl+,` |
| Close window/dialog | `Esc` |

### 41.2 Keyboard focus (p.985-986)

| Move focus to | Windows |
|---|---|
| Control Bar | `Alt+0` |
| Session View | `Alt+1` |
| Arrangement View | `Alt+2` |
| Clip View | `Alt+3` |
| Device View | `Alt+4` |
| Browser | `Alt+5` |
| Groove Pool | `Alt+6` |
| Learn View | `Alt+7` |
| Selected Clip Panel | `Alt+8` |
| Clip Panels | `Alt+Shift+P` |

With "Use Tab to Move Focus" on: `Tab` / `Shift+Tab` walk between focusable controls;
`Ctrl+Tab` / `Ctrl+Shift+Tab` walk between neighbors of the current control (p.986).

### 41.3 Set and program (p.986)

`Ctrl+N` new · `Ctrl+O` open · `Ctrl+S` save · `Ctrl+Shift+S` save as ·
`Ctrl+Q` quit · `Ctrl+Shift+R` **Export Audio/Video** · `Ctrl+Shift+E` export MIDI.

### 41.5 Editing (p.987-988)

| Action | Windows |
|---|---|
| Cut / copy / paste | `Ctrl+X` / `Ctrl+C` / `Ctrl+V` |
| Duplicate | `Ctrl+D` |
| Delete | `Delete` |
| Undo / redo | `Ctrl+Z` / `Ctrl+Y` |
| Rename | `Ctrl+R` |
| Select all | `Ctrl+A` |
| Select several items | `Ctrl`+click |
| Select from first to last | `Shift`+click |
| Next track/scene while renaming | `Tab` |
| **Ignore the grid while dragging** | `Alt` |

Modifiers that extend the reach of the commands above (p.988): `Shift` = clips and slots of **all
tracks**; `Shift` = time on **all tracks**; `Alt` = **the selected part of the envelope**.

### 41.6 Adjusting values (p.988-989)

↑/↓ arrows decrement/increment · `Shift`+arrows in octaves or fine adjustment ·
`Shift` while dragging = fine resolution · `Delete` returns to the default · `0`…`9` types ·
`.` and `,` walk between fields (bar/beat/16th) · `Esc` cancels · `Enter` confirms.

### 41.7 Breakpoint envelopes (p.989)

| Action | Windows |
|---|---|
| **Automation Mode** | `A` |
| Fine resolution while dragging | `Shift` |
| **Create curved segment** | `Alt` |
| Show fade controls momentarily | `F` |
| Delete selected envelope | `Ctrl+Delete` |
| Ignore grid while dragging | `Alt` |

### 41.8 Loop brace and start/end markers (p.990)

| Action | Windows |
|---|---|
| Set Start Marker | `Ctrl+F9` |
| Set Loop Brace Start | `Ctrl+F10` |
| Set Loop Brace End | `Ctrl+F11` |
| Set End Marker | `Ctrl+F12` |

With the brace or the markers **selected**: `Ctrl`+click moves the Start Marker to the position;
`Ctrl+Shift`+click moves the End Marker; `←`/`→` push the brace; `↑`/`↓` move the brace by one
loop length; `Ctrl`+`↑`/`↓` double/halve the length; `Ctrl`+`←`/`→` shorten/lengthen;
`Ctrl+Shift+L` selects the material inside the loop (p.990).

### 41.9 Zoom, display and selection (p.990-991)

| Action | Windows |
|---|---|
| Window zoom in/out | `Ctrl++` / `Ctrl+-` |
| Time ruler zoom in/out | `+` / `-` |
| Scroll following the playback | `Alt+Shift+F` |
| **Scroll horizontally** | `Shift`+wheel |
| Pan left/right of the selection | `Ctrl+Alt` |
| Add items to the selection | `Shift`+click or drag |
| Add adjacent clips/tracks/scenes | `Shift`+click |
| Add non-adjacent ones | `Ctrl`+click |

### 41.13 Grid and drawing (p.995)

| Action | Windows |
|---|---|
| **Draw Mode** | `B` |
| Narrower grid | `Ctrl+1` |
| Wider grid | `Ctrl+2` |
| Triplets | `Ctrl+3` |
| **Snap to Grid on/off** | `Ctrl+4` |
| Fixed / zoom-adaptive grid | `Ctrl+5` |
| **Ignore snap while dragging** | `Alt` |

### 41.14 Global quantization (p.995)

`Ctrl+6` sixteenth · `Ctrl+7` eighth · `Ctrl+8` quarter · `Ctrl+9` 1 bar · `Ctrl+0` off.

### 41.15 Session View (p.996-997)

| Action | Windows |
|---|---|
| **Launch selected clip/slot** | `Enter` |
| Select neighboring clip/slot | arrows |
| Select all clips/slots | `Ctrl+A` |
| Copy clips | `Ctrl`+drag |
| **Add/remove Stop Button** | `Ctrl+E` |
| Stop the clips of the track with the selected slot | `Ctrl+Enter` |
| Insert MIDI clip | `Ctrl+Shift+M` |
| **Insert Scene** | `Ctrl+I` |
| **Insert Captured Scene** | `Ctrl+Shift+I` |
| Walk between scenes, one by one | `↑` / `↓` |
| Walk between scenes, eight by eight | `PageUp` / `PageDown` |
| **Record into the Session** | `Ctrl+Shift+F9` |
| Toggle Follow Actions of the selected clips | `Shift+Enter` |
| Create Follow Action Chain | `Ctrl+Shift+Enter` |
| Move the selected track | `Ctrl`+`←`/`→` |
| Move non-adjacent scenes without collapsing | `Ctrl`+`↑`/`↓` |
| Drop clips from the browser as one scene | `Ctrl` |
| **Deactivate selected clip** | `0` |
| Go to the title of the highlighted track | `Esc` |
| Go to the first/last track of the scene | `Home` / `End` |
| Solo of the selected chain | `S` |

### 41.16 Arrangement View (p.997-999)

| Action | Windows |
|---|---|
| **Split at the selection** | `Ctrl+E` |
| **Consolidate** | `Ctrl+J` |
| Crop the selected clips | `Ctrl+Shift+J` |
| Resize the clip with the insert marker at the edge | `Enter` + `←`/`→` |
| Slide the waveform | `Shift+Alt`+drag |
| Stretch warped clip | `Shift`+drag on the title bar |
| Select time inside the clip | `Shift+Alt`+drag on the title bar |
| Create fade/crossfade | `Ctrl+Alt+F` |
| Delete fades of the selected clips | `Ctrl+Alt+Backspace` |
| Show fade handles momentarily | `F` |
| **Loop brace on/off** | `Ctrl+L` |
| Adjust the brace length | `Ctrl`+`←`/`→` |
| Select the content of the brace | `Ctrl+Shift+L` |
| Insert silence | `Ctrl+I` |
| **Cut / Copy / Paste / Duplicate / Delete Time** | `Ctrl+Shift+X` / `+C` / `+V` / `+D` / `+Delete` |
| **Fold/unfold selected tracks** | `U` or `←`/`→` |
| Unfold all | `Alt+U` |
| **Height of the selected tracks/clips** | `Alt++` / `Alt+-` |
| Scroll following the playback | `Ctrl+Shift+F` |
| Scroll left/right of the selection | `Ctrl+Alt`+drag |
| **Optimize Arrangement Height** | `H` |
| **Optimize Arrangement Width** | `W` |
| Deactivate selection | `0` |
| Nudge the selection | `←` / `→` |
| Reverse selected audio clip | `R` |
| **Zoom to the time selection** | `Z` |
| **Step back from the zoom** | `X` |
| Play from the insert marker in the selected clip | `Ctrl+Space` |
| Move the insert marker to the playhead | `Ctrl+Shift+Space` |
| Go to the title of the highlighted track | `Esc` |
| Move focus to the Mixer | `Alt+Shift+M` |

### 41.19 Tracks (p.1000-1001)

| Action | Windows |
|---|---|
| Insert audio / MIDI / return track | `Ctrl+T` / `Ctrl+Shift+T` / `Ctrl+Alt+T` |
| Rename selected track | `Ctrl+R` |
| Next track while renaming | `Tab` |
| Group / ungroup | `Ctrl+G` / `Ctrl+Shift+G` |
| Show / hide grouped tracks | `+` / `-` |
| Collapse/expand group | `U` |
| Show/hide return tracks | `Ctrl+Alt+R` |
| Move non-adjacent tracks without collapsing | `Ctrl`+arrows |
| **Arm selected tracks** | `C` |
| **Solo the selected tracks** | `S` |
| Add device from the browser | `Enter` |
| **Deactivate selected track** | `0` |
| Freeze/unfreeze | `Ctrl+Alt+Shift+F` |
| Delete track from the title bar | `Delete` |

### 41.20 Transport (p.1001)

| Action | Windows |
|---|---|
| **Play from the Start Marker / stop** | `Space` |
| **Continue from where it stopped** | `Shift+Space` |
| Stop at the end of the selection | `Ctrl+Space` |
| Play the Arrangement selection | `Space` |
| Insert marker to the beginning | `Home` |
| **Record** | `F9` |
| Arm Arrangement recording | `Shift+F9` |
| Record into the Session | `Ctrl+Shift+F9` |
| **Back to Arrangement** | `F10` |
| Activate/deactivate tracks 1…8 | `F1`…`F8` |
| Metronome | `O` |

### 41.22 Browser (p.1002)

| Action | Windows |
|---|---|
| Scroll | ↑/↓ arrows |
| Show/hide browser | `Ctrl+Alt+B` |
| Close/open folders | `←` / `→` |
| Load selected item | `Enter` |
| **Preview selected file** | `Shift+Enter` or `→` |
| Search | `Ctrl+F` |
| Go to the results | `↓` or `Enter` |
| **Assign color to the selected items** | `1`…`7` |
| Reset color | `0` |
| Similarity search | `Ctrl+Shift+F` |
| Browser history | `Ctrl+[` / `Ctrl+]` |
| Filter View | `Ctrl+Alt+G` |
| Tag Editor | `Ctrl+Shift+E` |

### 41.24 Key/MIDI Map Mode and Computer MIDI Keyboard (p.1003)

| Action | Windows |
|---|---|
| **MIDI Map Mode** | `Ctrl+M` |
| **Key Map Mode** | `Ctrl+K` |
| Computer MIDI Keyboard | `M` |
| Computer keyboard octave | `X` / `Z` |
| Computer keyboard velocity | `C` / `V` |

With the Computer MIDI Keyboard on, **the single-letter shortcuts keep working with `Shift`**
(e.g. `Shift+S` for solo) (p.1004).

### 41.25 Momentary latch (p.1004)

Holding the key for ~500 ms toggles the action **while held** and returns to the previous state on release.
Keys with latch: `A` (Automation Mode), `B` (Draw Mode), `S` (solo of the selected track),
`Z` (zoom to the Arrangement selection), `F1`…`F8` (activator of the first eight tracks),
`Tab` (Arrangement ↔ Session). Disableable with `-DisableHotKeyLatching` in `Options.txt`.

---

## Adaptation to Spellcaster

### Concept table

| Ableton Live 12 | Spellcaster | Note |
|---|---|---|
| **Clip** (audio/MIDI) on an Arrangement track | **segment** of `.ild`, audio, video or fx on a timeline track | move, trim by the edge, split (`Ctrl+E`), consolidate (`Ctrl+J`), fades at the ends, own color and name (p.170-177) |
| **Scene** (Session row, with Scene Launch) | **cue** | launch the whole row; the next one gets pre-selected; `Ctrl+I` inserts, `Ctrl+Shift+I` captures the current state as a new cue (p.185, p.191) |
| Empty **clip slot** with Stop Button | cue slot that **does not touch** that show line | removing the stop button (`Ctrl+E`) pre-configures "this cue does not play this track" (p.191) |
| **Track** | show line (universe, laser, video) | fold `U`, height `Alt+±`, `H`/`W` to fit everything, `0` deactivates, `C` arms, `S` solos (p.174, p.998, p.1000) |
| **Device** in the track chain | **patchbay module / node** | the automation lane's Device chooser is what gives "which module, which parameter" (p.495-496) |
| **Automation envelope** in the track lane | **keys** (keyframes) of the parameter on the timeline | breakpoint, curve with `Alt`, drawing with `B`, Simplify, shapes (p.496-501) |
| **Clip envelope / modulation** | relative modulation over the absolute value of the key | red = absolute, blue = relative; the distinction solves "the cue says 50%, the timeline modulates ±20%" (p.508) |
| **Locator** in the scrub area | timeline **marker**, triggerable | mappable by MIDI/key, "Loop to Next Locator", "Set Song Start Time Here" (p.166-167) |
| **Loop brace** | rehearsal loop | `Ctrl+L` turns it on over the selection; arrows adjust; `Ctrl+↑/↓` doubles/halves (p.169-170) |
| Track **Arm** + Session Record | `gravar dmx` armed per line | arming selects; `Ctrl`+click arms without disarming the others; `C` arms the selected ones (p.406, p.1000) |
| **Punch-in / punch-out** | protected recording window | they are the loop edges, not separate fields (p.407) |
| **MIDI/Key Map Mode + Mapping Browser** | `Ctrl+Shift+A` (our mapping) | to copy: mode with colored highlight, browser listing control → path → parameter → editable and invertible Min/Max (p.840-841) |
| **Takeover Mode** | external fader behavior when switching bank | Pick-Up and Value Scaling are the right answer for a surface with banks (p.839-840) |
| **Follow Action** | automatic cue chaining | Chance A/B, Linked/Unlinked, Jump to a target — it is a cue list with probability (p.361-362) |
| **Video Window** | laser/video preview | floating window always on top, size **outside** the show file (p.518) |
| **Overview** + two rulers (bar and clock) | dual timeline ruler | a show has cues in clock time and keys in frames; Live already shows both (p.160-162) |

### Gestures worth copying literally

1. **Dragging vertically on the ruler = zoom; dragging horizontally = scroll.** It applies on the Overview and on the beat-time ruler (p.160-161). It answers the owner's request without inventing a modifier.
2. **`Ctrl`+wheel = zoom, `Shift`+wheel = horizontal scroll, `Alt`+wheel inside the track = track height** (p.163, p.991). It is exactly the request "mouse scroll moves up and down only on the tracks; shift+scroll goes side to side; ctrl+scroll zooms in or out" — with `Alt` for height added on top.
3. **`Alt` held ignores the snap; with snap off, `Alt` turns the snap on** (p.175). A single modifier, symmetric.
4. **Holding a key for ~500 ms becomes a momentary toggle** (p.1004). `A`, `B`, `S`, `Z`, `Tab`. An operator holds `Z` to take a look and releases.
5. **Follow pauses on its own when you edit and comes back when playback restarts** (p.163). It is the difference between a usable "centered playhead" and an unbearable one.
6. **Resizing one track with `Alt` resizes all of them** (p.174).
7. **A button that moves an envelope into its own lane, and `Alt` on the same button moves all the automated ones** (p.496). One key solves "I want to see only this one" and "I want to see everything".
8. **Hiding a lane ≠ deactivating the envelope** (p.496). Never confuse visibility with state.
9. **`Ctrl`+`←`/`→` makes the insert marker jump from marker to marker and from clip edge to clip edge** (p.174).
10. **Simplify Envelope** after recording (p.500) — it is mandatory for `gravar-dmx`: a recording at 44 Hz becomes an editable curve.
11. **The Status Bar shows the exact position of whatever is under the mouse** (p.35).

### What does not apply

Warp, tempo/BPM, musical quantization, Racks, Grooves, Scale Awareness, Capture MIDI, comping and Push:
Spellcaster has neither musical time nor elastic audio material — the ruler is clock and frame, the
"launch quantization" becomes a time/timecode grid if it is ever missed, and what Live solves with a
device chain we solve with the patchbay.

### Open issue for `design/DECISOES.md`

`Tab` in Live toggles Session ↔ Arrangement, and in `design/SHORTCUTS.md` `Tab` is already the Face switch —
the two uses coincide in intent. But Live also uses `Tab` for focus navigation when
"Use Tab to Move Focus" is on (p.986), and it uses `U` both for track fold in the Arrangement and
to collapse a group (p.998, p.1000). Not decided here.
