# Resolume — official Quickstart Tutorial (what the product teaches first)

Short companion to `resolume.md`. That file is the audit of the files of the installed Arena 7.21.1
(`.avc` compositions, `swagger.yaml`, shortcuts, fixtures, contextual help) and covers the whole data
model. This one is something else: **the text of the official first-steps tutorial**, which
says not what the product has, but what the manufacturer chose to teach in the first five minutes —
and in that order. Nothing is repeated from `resolume.md`; where the subject is already there, this file points to it.

## Sources read

- `<scratchpad>/fontes/resolume-quickstart.txt` (4,818 bytes) — full text of
  https://resolume.com/support/en/quickstart (v7 version), captured on 2026-09-09.
  The file **does not go into the repository**.
- Short document: one page, five sections (`Trigger Clips`, `Mixing`, `Effects`, `Have a Play!`).
  It was read in full. There is not a single keyboard shortcut in the text, no mention of DMX, output, full
  screen, MIDI/OSC mapping or Advanced Output. Whoever wants that: `resolume.md`.

## What the tutorial teaches, in order

### 1. Composition is the unit of work

First definition given: "a composition is a complete Resolume setup — each composition can
include sets of clips, pre-programmed effects and all the other settings needed for
a performance". A fresh install **already comes with an example composition** — the user never
faces an empty screen.

For us: the `.spell` is the composition, and **Spellcaster must open with an example show loaded**,
not with an empty timeline. Low cost, and it is the difference between "click a thumbnail and something happens" and
"read the manual".

### 2. A Layer is a horizontal row, and it plays one clip at a time

The text introduces the grid like this: below the menu bar there is a set of **horizontal rows**, each
one with controls on the left and a row of thumbnails; each thumbnail is a clip. Then: "each one
of these horizontal rows is a **layer**. Each layer plays **one clip at a time**."

The mixing rule is taught by contrast, in two clicks:

- clicking another thumbnail **on the same layer** → at the start of the next bar, the output switches;
- clicking a thumbnail **on another layer** → the old clip keeps playing and the two mix together.

That is: **exclusivity within the row, sum across rows.** It is the same rule Ableton uses in the
Session View (one clip per track, `ableton12.md` §4) and it is the rule Spellcaster needs for
"one universe, one scene at a time; different universes sum".

The data model behind it (`Layer`, `Clip`, `Column`, `Deck`, `connected` as a 5-state enum)
is in `resolume.md` § Objects and verbs and § States.

### 3. Triggering is quantized by default, and the tutorial warns before the user complains

"these clips are set to sync to the BPM, so the clip may not start
instantly — it will wait for the start of the next bar. Don't worry, if you want to
launch clips instantly, you can set them up for that."

Two good things here, both of UX and not of function: the **default** behavior is the synced one, and the
tutorial **explains the delay in the exact paragraph where it happens**, before the user thinks it
froze. An unexplained delay reads as a bug.

### 4. Clip transport: three buttons and a draggable wedge

Selecting the **Clip** tab gives the **Transport** section: **Forwards, Backwards and Pause** icons to play
and stop, and — the gesture that matters — "you can also grab the **moving blue wedge**
directly to scratch the clip".

And the consequence is stated: doing that **puts the clip out of phase with the BPM** — the tempo stays
right, the phase does not. To resync, **click the thumbnail again**: it restarts at the beginning of the
next bar.

For us: (a) the segment's position indicator is **grabbable**, it is not just a drawing; (b) going out of
sync is a visible state, reversible by an obvious gesture, not an error. Ableton solves the
same problem with Legato and with Nudge (`ableton12.md` §4); Resolume solves it with "click the
clip again".

### 5. Per-layer sliders: A, V and M

To the left of each layer's thumbnails, two vertical sliders **A** and **V**: A fades the layer's audio,
V does the same for the video. And the **M (master)** controls both at the same time.

This is the only point of the quickstart already covered in `resolume.md` (§ Objects and verbs: the
`audio`, `video` and `master` fields of the `Layer` in the swagger). What the quickstart adds is the
placement: **the three sit on the left edge of the row, next to the layer name**, always
visible, without opening a panel.

### 6. Browser: tabs, not a tree — and the drop target has four colored corners

On the right of the interface there are the **Files**, **Compositions**, **Effects** and **Sources** tabs
(the `recentLayout.xml` shows there is a fifth, `Recording` — `resolume.md` § Screen anatomy).
Selecting **Effects** lists the installed video effects.

The drag and drop gesture, which is the reason this file exists for the `browser-dnd` workstream:

1. pick an effect from the list in the Effects tab;
2. drag it to the left, up to the **Composition** tab;
3. drop it **on the area that says "Drop effect or mask here"**;
4. "you know you are in the right place when you see **four colored corners** appear around the
   Composition tab".

Three decisions in one sentence: **the drop zone is labeled with text when it is empty**, the valid target
lights up **before** dropping, and the highlight is **the four corners**, not a full outline (it works over
any content, without covering what is underneath).

The effect appears in the output immediately. Effects stack: "each effect takes the output of the previous one and
processes it" — the stack order is the chain. To remove: **click the `x` to the right of the effect name**.

### 7. Every effect has Opacity

"All video effects have the **Opacity** slider — it serves to mix the processed video with the
original." Beyond that, most have their own parameters (the example is Bendoscope, with a slider for
the number of divisions).

A cross-cutting rule, and a cheap one: **every Spellcaster patchbay module exposes a "how much" beyond its own
parameters**, with the same name and in the same place in all of them. It makes "turn off without removing" and
"half of the effect" exist without each module inventing its own way.

### 8. Contextual Help window in the bottom right corner

"A useful feature is the **Help** window in the bottom right corner of the interface. It shows short tips
about whatever the mouse pointer is currently over."

It is the same mechanism `resolume.md` already identified from the other side, on disk: `docs\help\English.xml`
with 412 `element → title + one sentence` entries. The quickstart shows **where that appears on screen** and
that it is the last item taught — after the user has already made something work.

---

## Adaptation to Spellcaster

| Resolume quickstart | Spellcaster | Workstream |
|---|---|---|
| A fresh install opens with an example composition | example `.spell` loaded on the first run | — |
| Layer = horizontal row, **one clip at a time**; layers sum | one scene per universe, universes sum | `daw-sessao` |
| Clicking the thumbnail triggers, quantized, and the tutorial explains the delay right there | cue triggering with a time grid, with the delay shown (not silent) | `daw-sessao` |
| **Grabbable blue wedge** = scratch | the segment's position indicator is grabbable | `daw-arranjo` |
| Clicking the thumbnail again resyncs | one obvious gesture for "get back in phase" | `daw-sessao` |
| A / V / M sliders on the left edge of the layer | per-show-line level always visible, without opening a panel | `daw-sessao` |
| Files / Compositions / Effects / Sources tabs | browser in tabs, not in a tree | `browser-dnd` |
| **Labeled drop zone + four colored corners on hover** | valid target highlight before dropping | `browser-dnd` |
| Effect stack: each one takes the output of the previous; `x` removes | patchbay module chain | `mapping` |
| **Opacity on every effect** | a standard "how much" on every module | `mapping` |
| Contextual Help window in the bottom right corner | Aprendiz, one sentence per element under the cursor | — |

### What the quickstart does not cover and must not be inferred from it

No keyboard shortcut, no cue, no GO, no DMX, no arming the output, no output/full screen, no
MIDI/OSC mapping. Those subjects are audited in `resolume.md`, including the product's real
absences (§ "What NOT to copy": no arming the output, no rehearsal mode, no GO on the keyboard, no show cue).
