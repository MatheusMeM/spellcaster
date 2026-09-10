# Session View — the cue list as a scenes x tracks grid (panel `Shift+2`, second press)

The other half of Matheus's request: the interface "just like ABLETON" has two views, and the second one is the grid. `daw-arranjo.md` is written time; this file is fired time. We already have the object — the cue (`cue_set`, `cue_go`, `cue_capture`, `cue_del`, `edit.rs:746,751,874`), specified in `cenas-cues-dmx.md`. What is missing is the **form**: today the cue is a spreadsheet row (`teatro.js:196-246`), and Ableton shows why the grid is better.

Source: **Ableton Live 12**, official online manual, §7 (Session View) and §16 (Launching Clips), cited by section. Ableton is the only source here: Resolve has no Session View, and Resolume (which does, and calls it deck/column/layer) was already read in `fontes/resolume.md` and in `cenas-cues-dmx.md`.

| Item | Who solved it best | Why |
|---|---|---|
| Form of the grid | Ableton §7.2 | Row = scene, column = track, cell = what that scene does on that track. A cue table with a "5 ch" column (`teatro.js:236`) hides exactly what the operator needs to see |
| Firing | Ableton §7.1 and §7.2 | A triangular button per cell, a button per scene in the right-hand column, pre-select by the name and fire with `Enter` |
| Track state | Ableton §7.3 (Track Status field) | A pie chart with "how many laps" and "loop length" says more than an LED |
| Launch quantization | Ableton §16.4 | It exists and it is global; in our case it becomes `wait`, which is already in the `.spell` |
| Follow actions | none | **They do not come in.** See §3 |
| Cancel an already fired scene | Ableton §7.2 ("Cancel Scene Launch") | The `wait` of the cue creates the same window and needs the same button |

## 1. What a "cell" is in our case

In Ableton the cell is a clip: its own media, with a launch mode and a follow action. In Spellcaster **the cue is the whole row** — `cue.values` is a flattened map `{"universe/address": value}` (`edit.rs:541-543`), not one object per track.

Therefore:

> **One scene = one cue. One cell = the slice of `cue.values` whose addresses fall inside that track.** The cell is *derived*, not stored.

That is not a limitation: it is what makes the grid worth it without changing the format. For a `dmx` track with `universe: 1, address: 10` and 4 channels, the cell of scene 3 shows the values that cue 3 writes into `1/10..1/13` — full, partial or empty. It is the "red = it is in the cue, orange = it is but with another value" that `cenas-cues-dmx.md` already took from MadMapper, now with a place on screen.

For a `fixture` track, the slice is per profile channel (`fixture_set {name, channel}`, `edit.rs:627-631`), not per raw address.

For a media track (`laser`, `audio`, `video`, `fx`, `osc`) **the cell is empty today and there is no way to fill it**: `cue.values` only carries a number per DMX address. It is the real gap of this function, and it has a one-line way out:

```json
{"name": "abertura", "fade": 2, "wait": 0, "follow": false,
 "values": {"1/10": 255},
 "fires": ["laser/1/play", "track/4/clip/2"]}
```

`fires[]` is a list of **registry addresses** fired along with the cue — the same textual identity of rule 2 of `FUNCOES/README.md`, the same list that `mapping.md` resolves and the same one that `daw-arranjo.md §5` proposes for the marker with `go`. One field, three functions. **Awaiting vote.**

## 2. Screen zones

| Zone | Content | Origin |
|---|---|---|
| Grid | rows = cues in the order of the `.spell`; columns = tracks in the order of the `.spell` (the same order as `daw-arranjo.md` B6) | Ableton §7.2 |
| Right-hand column | fire button for the whole scene, cue name, `fade`, `wait` | Ableton §7.2 ("The Scene Launch buttons are located in the rightmost column, which represents the Main track") |
| Row below the grid | per track: state (stopped / fading / at value), release button | Ableton §7.3 (Track Status field) |
| GO button | big, separate, fires the next cue | already exists: `teatro.js:246` |
| Programmer | the captured channels that have not become a cue yet | already exists: `level_set` / `level_clear` / `cue_capture` |

## 3. What does NOT come in, and why

- **Follow actions.** Ableton §16.7: two actions (A and B) with `Chance A`/`Chance B` as a percentage, ten possible actions, `Linked`/`Unlinked` with a loop multiplier, `Jump Target`, a global `Enable Follow Actions Globally` button, and on top of that the rule *"Follow Actions in scenes always take precedence once they are triggered"*. We already have `follow: bool` in the cue (`edit.rs:538-540`): when it ends, it fires the next one. It is the `Next` Follow Action with no probability, no jump and no multiplier — and it is what a lighting console does. Adding the rest is a state machine hidden in the cue list, and the state machine is already being designed in the right place (`state` node of the graph, `DECISOES.md`, awaiting vote). **Follow actions stay out, and the reason stays written here so they do not come back by forgetfulness.**
- **Per-cell launch modes** (Ableton §16.2: Trigger / Gate / Toggle / Repeat). Trigger/toggle/value are already declared types in the registry (rule 3 of `FUNCOES/README.md`) and the firing mode belongs to the **mapping**, not to the cue: `mapping.md §4` puts piano/toggle in the shortcut, which is where Resolume puts it too. Two copies of the same concept would be rule 9 violated.
- **Legato** (Ableton §16.3). Meaningless: a cue has no playback position.
- **Velocity Amount** (Ableton §16.5). It comes in through the mapping (`mapping.md`), not through the cue.
- **Decks** (Resolume) / named sets of scenes. One cue list per show is enough; when a show needs two lists, that is two shows.
- **Clip Stop per cell** (Ableton §7.4.2, `Ctrl+E` adds/removes the stop button). Our cue does not "play", it reaches a value; stopping is not a verb of its own. What exists is `level_clear`, which is already the button of the bottom row.

## 4. `Tab`: the conflict, and how it ends up

Ableton §41.1: `Tab` switches Session ↔ Arrangement, and §41.25 says it is *momentarily latchable* — holding it about 500 ms switches and releasing goes back.

`design/SHORTCUTS.md` already gave `Tab` to something else: **switch Face `editor` ↔ `performance`, "(hold to peek, tap to switch)"**. It is literally the same gesture as Ableton's, applied to a larger scope.

**Resolution: `Tab` stays with the Face.** Reasons, in order:

1. Scope. The `performance` Face is what the operator sees on stage; Arrangement and Session are two panels of the same editor. The cheapest key goes to the most expensive switch.
2. `SHORTCUTS.md` cites PRD §10 for that `Tab`; changing it would require changing the PRD, and nothing here justifies that.
3. There is already a pattern for "second press" in this codebase: `Shift+Z` frames and, pressed again, goes back to the previous zoom (`timeline-daw.md` item 22); `M` creates a marker and, pressed again, edits it (item 26). **Arrangement ↔ Session is the second press of `Shift+2`**, which is the Timeline panel key in `SHORTCUTS.md`. Zero new keys, zero conflicts, and the rule ("pressing the panel key again switches its view") holds for any panel that gains a second view later.
4. What is lost from Ableton is the momentary *latch* between the two views. Little is lost: peeking at the grid while editing the arrangement is the case Ableton solves with two monitors, and we solve with the Face (`FUNCOES/README.md §13`, layout saved as a named preset).

`FUNCOES/README.md § Open issues` already records `Tab` as a disputed key (entering/leaving a group in the graph had to move to `Ctrl+]`/`Ctrl+[` for the same reason). This is the third time `Tab` has been defended; it is decided here and it drops off the open issues list.

## 5. Is `teatro.html` the base? Yes, with two conditions

**It is the base.** `spellgui/web/teatro.js` is already the only client that speaks the whole cue language: `cue_set` with every field (`teatro.js:210-220`), `cue_del` (`:240`), `cue_go` on double-click (`:241`) and on the GO button (`:246`), `cue_capture` (`:248`), `level_clear` (`:250`), plus the patch (`teatro.js:~185`). Rewriting that on another page would be throwing away the only cue code that exists.

Two conditions before it becomes the Session View:

1. **Transpose.** `teatro.js:196-246` builds a `<table>` row by row with an `<input>` per field. The grid is the same data with the axes swapped: a cue becomes a row, a track becomes a column, and the `name`/`fade`/`wait`/`follow` fields move out of the row into the right-hand column (§2). `cue_set` does not change: it keeps sending the whole object (`teatro.js:211-219` already does that, because `cue_set` replaces and does not merge).
2. **Swap the BUS.** `teatro.js:71` has its own WebSocket client of ~40 lines, with a `ponytail:` saying to swap it for `bus.js` when that exists — it exists (`patchbay.html:67`, `laser.html:85`, `face.html:49`). While there are two clients, the mapping overlay of `mapping.md` has nowhere to plug into on this page (`pontos-falhos.md` item 15). The swap is a prerequisite, not an improvement.

What is **not** the base: the patch table of the same page stays where it is, it is `cenas-cues-dmx.md`, and the `patchbay-2` workstream touches it.

## 6. Shortcuts

| Action | Key | Origin | Conflict |
|---|---|---|---|
| Switch Arrangement ↔ Session | `Shift+2` again | Ableton §41.1 (there it is `Tab`) | resolved in §4 |
| Fire the selected cue | `Enter` | Ableton §7.1 ("pre-select a clip by clicking on its name, and launch it using the computer's Enter key") | none: `SHORTCUTS.md` already has `Enter` = "Cue GO" |
| Navigate between cells | arrows | Ableton §7.1 ("You can then move on to the neighboring clips using the arrow keys") | the arrows in the Session move the selection; in the Arrangement they move the playhead. The focused panel decides (`SHORTCUTS.md § Grammar`) |
| Previous / next cue | `Shift+PageUp` / `Shift+PageDown` | already proposed in `cenas-cues-dmx.md` | none |
| Cancel the fired scene while still in `wait` | `Esc` tap | Ableton §7.2 ("Cancel Scene Launch") | `SHORTCUTS.md`: `Esc` held 0.5 s is blackout, a tap is close. Here the tap cancels the pending `wait` **when there is one**; with no pending `wait`, it keeps closing |

## 7. Awaiting vote

1. **`cue.fires: ["<address>", ...]`** (§1), which is what gives media tracks a cell. It is the same field that `daw-arranjo.md §5` proposes as `marker.go`; the vote decides whether the two are named the same (`fires` in both, a list) or the marker keeps a single one (`go`, a string).
2. **Follow actions stay out** (§3): recorded as a deliberate refusal, with the reason, so they do not come back.
3. **`Tab` stays with the Face; Arrangement ↔ Session is the second press of `Shift+2`** (§4).

## 8. Tests

| Function | Input | Expected output |
|---|---|---|
| `TEATRO.celula(cue, track)` | cue with `{"1/10":255,"1/14":0}`, track `{universe:1, address:10, canais:4}` | `{n: 1, de: 4}` — partial |
| `TEATRO.celula(cue, track)` | same cue, track on `universe:2` | `{n: 0, de: 4}` — empty |
| `TEATRO.grade(show)` | show with 3 cues and 5 tracks | 3x5 matrix, without asking the engine |
| `TEATRO.celula` with a media track | any cue with no `fires` | `null` (proof that the gap of §1 is explicit, not a disguised zero) |

Visual proof: headless screenshot of `teatro.html` with `shows/medgrupo.spell` showing the grid, and the comparison with `png/teatro.png` (today's spreadsheet).
