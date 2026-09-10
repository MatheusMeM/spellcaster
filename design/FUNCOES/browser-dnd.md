# Browser and drag and drop — the left panel (`Alt+5`)

Matheus's request (2026-09-09): *"I want drag n drop of elements and outputs and media and laser and audio and video"*, and before that *"I have no way to input or record new DMX or ILDA or video"*. Today there is **no** media input at all: no `drop`/`dragover`/`dataTransfer` in `spellgui/web/*`, no write route in `spellcore/serve/src/lib.rs:338-345`, no file command among the registry's 30.

Sources: **Ableton Live 12** §4 (Working with the Browser), §4.10 (Adding Content), §5.3 (Live Clips), §41.22 (shortcuts), cited by section. **DaVinci Resolve 20**, Media Pool, cited by page of the local manual. **Resolume Arena**, online manual (Decks, Clips), for what happens when you drop onto what already exists.

| Item | Who solved it best | Why |
|---|---|---|
| Panel structure | Ableton §4 | Three named sections — Collections, Library, Places — and Places is where the user's folders go. Resolve's Media Pool is one bin per project, and our "library" is the show folder plus whatever the operator points at |
| Where a drop creates a track | Ableton §4.10 | *"into the space to the right of Session View tracks or below Arrangement View tracks will create a new track"* — a single rule, in both views |
| Drop coming from the file system | both | Ableton §4.10: *"Files can also be dropped directly into Live from the Explorer (Win)/Finder (Mac)"*; Resolve p.690: *"You can also drag a clip directly from your file system to the Timeline"* |
| Drop onto something that already exists | Resolume (Decks) | It is the only manual that declares the three cases: overlapping swaps position, dropping between two inserts, `Ctrl` on drop copies. Ableton does not describe it, Resolve tells you to use "Set Path" |
| Preview before using | Ableton §4.9 | Preview toggle, `Shift+Enter` auditions without turning the toggle on, and a `Raw` button that separates "play in show time" from "play as the file is" |
| Outputs in the browser | none | None of the three treats an output as a draggable item. It is our invention, justified in §4 |

## 1. Browser objects

One tree, three roots, in the order they appear:

| Root | Contains | Where it comes from |
|---|---|---|
| **Show** | what this `.spell` uses: media referenced by `clips[]`, patch profiles, graph modules, faces | read from the open show |
| **Folder** | the show folder and the folders the operator adds | Ableton's `Places` §4.7: *"you must first add them to the browser, either by dropping them directly into the Places section from the Explorer (Win)/Finder (Mac), or by using the Add Folder option"*; §4.7.8: *"Adding a user folder does not actually move the folder to a new location"* |
| **Outputs** | sACN/Art-Net universes, OSC targets, laser DACs, video windows | read from `show.outputs[]` |

Item types, with what each one becomes when dropped:

| Item | Extension | Becomes |
|---|---|---|
| Laser clip | `.ild` | `laser` track + clip |
| Audio | `.wav`, `.mp3`, `.flac` | `audio` track + clip (`audio-video.md §1`) |
| Video | `.mp4`, `.webm` | `video` track + clip |
| Effect | `.rhai` | `fx` track |
| Fixture profile | `profiles/*.json` | `patch_add {name, profile, universe, address}` |
| Graph module | `modules/*.json` | `module` node in the PATCHBAY (`DECISOES.md`, awaiting vote) |
| Face | `faces/*.json` | opens the Face |
| Output | — | see §4 |

Color by type is the same rule as `daw-arranjo.md` B7: derived from the type, never free.

## 2. Preview

Ableton §4.9 has three things and all three hold:

1. **Preview toggle** next to the tab. On, clicking the item auditions it.
2. **`Shift+Enter` auditions even with the toggle off** (*"You can preview files even when the Preview toggle is not enabled by pressing ShiftEnter or the right arrow key"*). It is the shortcut that saves whoever turned preview off so as not to blow the PA in the middle of the show.
3. **`Raw` button**: off, the preview waits for the next bar and loops; on, it plays at the original tempo, no loop, no scrub. Our equivalent: off = the preview respects the transport (comes in at the next marker); on = plays right away. **`Raw` comes in**, because it is the difference between hearing a file and rehearsing a cue-in.

Preview of `.ild` is the same gesture in the previz viewer (`previz` workstream); preview of `.rhai` does not exist (you do not audition a script).

## 3. What each drop does

This is the table the implementation follows. `target` = where the mouse drops.

| # | Item | Target | Effect | Origin |
|---|---|---|---|---|
| 1 | media | empty area below the last track | **creates a track of the media's type** and puts the clip at the time of the drop point | Ableton §4.10 |
| 2 | media | compatible track, at a time | puts a clip there. Compatible = the track type matches the extension | Ableton §4.10 ("Items can be dragged and dropped from the browser into tracks") |
| 3 | media | incompatible track | refuses, with the refusal cursor. Does not convert, does not create a hidden track | (ours; the silent alternative is the defect `PRINCIPIOS.md` calls a widget lie) |
| 4 | media | on top of an existing clip | **inserts alongside**, pushing: dropping on the left half inserts before, on the right inserts after | Resolume, Decks: *"You can also drag a clip just to the right or left of another clip. This will insert the dragged clip next to it, and shift over the others to make room for it."* Replacing by drop **does not come in**: Resolume does not have it either (the path there is the Media Manager, "Set Path") |
| 5 | clip already on the timeline | elsewhere | moves (that is `daw-arranjo.md` C2) |
| 6 | clip already on the timeline | elsewhere, with `Alt` | copies | `timeline-daw.md` item 15; Resolume uses `Ctrl` on drop, we already fixed `Alt` |
| 7 | multiple files | any target | **a single track**, stacked in time, unless `Ctrl` is held on drop, which spreads them across tracks | Ableton §7.4: *"Live defaults to arranging them in one track... Hold down Ctrl (Win) / Cmd (Mac) prior to dropping them so as to lay the clips out in multiple tracks instead"* |
| 8 | fixture profile | Patch panel | `patch_add` with the file name as `name` | (ours; `cenas-cues-dmx.md`) |
| 9 | output | a track header | **assigns the output to the track**: writes `universe`/`address` (sACN/Art-Net), `feed` (laser), `address` (OSC) | see §4 |
| 10 | output | empty area | **creates the output in the show** (`show.outputs[]`) | see §4 |
| 11 | file from Explorer | any target | same as 1-4, after uploading the file | see §5 |
| 12 | folder from Explorer | browser's **Folder** root | adds the folder to the browser, **copying nothing** | Ableton §4.7.8 |

Cross-cutting drop rule, taken from Ableton §4.10 and valid for all of them: **double-click or `Enter` on the item does the same thing as dropping onto the selected track.** Without it, the browser is unusable from the keyboard.

## 4. Draggable output: why, and the contract

None of the three manuals has it — in Resolume the output is a `screen` in the Advanced Output (`Ctrl+Shift+A` there), in Ableton it is the routing in the mixer, in Resolve it is the Deliver page. But the request is literal (*"drag n drop of elements and **outputs** and media"*) and our output **is** an object of the show: `show.outputs[]` already exists and already has a shape (`shows/medgrupo.spell`: `{"type":"sacn","universes":[1],"priority":100,"source_name":"Spellcaster"}` and `{"type":"laser","dac":null,"port":7765,"pps":25000,"safety":{...}}`).

Contract:

- **An output item in the browser = one entry of `show.outputs[]`**, plus the candidates discovered on the network that are not in the show yet (an EtherDream DAC that answered, an announced Art-Net node). A candidate shows up dimmed; dragging it into the empty area is what adds it.
- **Dropping an output on a track header** writes into the track what links the two. For `dmx`/`artnet`: `universe` and `address`. For `laser`: the `feed`. For `osc`: the base `address`. It is a one-line `show_patch`.
- **Dropping an output on an empty area** adds it to `show.outputs[]`.
- **One physical output serves one target only.** Resolume, Screens: *"Every output can only have a single screen associated with it"* — when you pick an output already in use, Resolume sends the other one back to virtual. Here: a universe already assigned to another track is a warning, not an error (two tracks on the same universe is legitimate use with declared merge, `cenas-cues-dmx.md § Universe`); a laser DAC already opened by another track **is an error**, because one DAC plays one stream.
- **One panic button.** Resolume, Screens: `Ctrl+Shift+D` disables all outputs. `SHORTCUTS.md` already has `Ctrl+Shift+Enter` (arm real outputs) and `Ctrl+Shift+R` (rehearsal mode); the missing half is disarming everything, and it is the same arm key hit again.

**Commands missing in the registry.** Today an output is only edited through `show_patch` with the path `/outputs/0/pps`, which forces the GUI to know the index and the schema:

| Command | Arguments | Does |
|---|---|---|
| `output_add` | `kind, ...` | Adds to `show.outputs[]`; returns the index |
| `output_set` | `index, <fields>` | Edits |
| `output_del` | `index` | Removes |
| `outputs` | — | Lists the show's **and** those discovered on the network, with state |

`outputs` is what fills the browser. Without it the outputs panel is a read of `/show` with no state, which is defect 24 of `pontos-falhos.md` (the laser panel has eight sliders and no DAC).

## 5. File from Explorer: the `POST /files` contract

A file drop from the system arrives through HTML5: the `drop` event carries `dataTransfer.files`, a `FileList` of `File` objects. **In the wry window there is no path** — `File.name` is the name, `File.path` is an Electron extension and does not exist there. Therefore the content has to be uploaded.

New route in `spellcore/serve/src/lib.rs`, next to the four that exist (`/commands`, `/show`, `/ws`, `/mcp`, `:338-345`):

```
POST /files/<name>
  body: the file bytes
  200 -> {"path": "media/medgrupo_laser.ild", "bytes": 41232}
  400 -> name refused
  413 -> larger than the limit
  409 -> already exists
```

Rules, all mandatory:

1. **The destination is the show folder, `media/` subfolder.** Never `--dir`, never a path coming from the client.
2. **`<name>` goes through the same filter as `estatico()`** (`serve/src/lib.rs:102-108`): refuses `:`, `\`, an empty segment, `.` and `..`, and does no percent-decode. The filter already exists and is already tested; reuse it, do not rewrite it.
3. **Extension on a whitelist**: `ild`, `wav`, `mp3`, `flac`, `mp4`, `webm`, `rhai`, `json`, `spell`. Off the list, 400. Without a whitelist, `POST /files/x.exe` into the show folder is a ready-made vector.
4. **Size limit**, with the value in a `ponytail:` comment. Proposal: 512 MB, which covers a show video and does not cover a disk. Above it, 413.
5. **Already exists = 409**, and the GUI asks. Overwriting media used by another clip is silent loss.
6. **`127.0.0.1` only.** `serve` is already local (`serve/src/lib.rs`), but the write route is the first one that makes this a security decision and not an accident; a `ponytail:` comment says it falls when `--host` and the token exist, together with the read of the repo root already declared at `:99-101`.
7. **The response returns the path relative to the show**, which is exactly what goes into `clips[].src` (`daw-arranjo.md §4.1`).

**Blocker to solve first:** the GUI does not know where the show lives. `GET /show` calls `show_get {full:true}`, and that branch returns only `serde_json::to_value(sh)` — with no path (`registry.rs:221-223`; only the non-`full` branch goes through `resumo(f, sh)`). With no path there is no "show folder" for the `POST` nor for resolving `src`. Minimum fix: `show_get {full:true}` returning `{"file": "<caminho>", "show": {...}}`, or `GET /show` sending the path in a header. It is `pontos-falhos.md` item 16, and it is a prerequisite of this function.

**Second blocker, smaller:** `mime()` knows four types (`serve/src/lib.rs:87-96`) and returns `application/octet-stream` for everything else. An `<audio src="media/x.mp3">` or `<video src="media/x.mp4">` served as octet-stream plays in no browser at all. `mime()` gains `mp3`, `wav`, `flac`, `mp4`, `webm` — five lines, and the file's own `ponytail:` already anticipates it ("imagem e fonte entram quando alguma pagina trouxer uma").

## 6. What stays out

- **Hot-swap** (Ableton §23.2.4, key `Q`). Swapping a clip's file without dropping it is useful, but the gesture is "audition and swap live", which only makes sense with instant preview of video and audio already running. It comes back after `audio-video.md`.
- **Favorites and collection colors** (Ableton §4.5, keys `1`-`7`). The keys `1`-`7` are expensive and `PRINCIPIOS.md §2` does not allow decorative color. A favorite item is an item in a folder.
- **Media Manager / "Set Path"** (Resolume) and relinking lost media. It comes in when there is a show that traveled between machines; today the path is relative to the show folder and the folder travels with it.
- **Combining audio and video in a single clip** (Resolume, Decks: dropping audio onto a slot with video transposes the video to the audio's duration). We have two tracks; combining is syncing, and syncing is `audio-video.md §3`.
- **Arithmetic in fields** (Resolume, Input Selection: typing `/3` in a width field). Good, and it belongs to the Inspector, not the browser.

## 7. Shortcuts

| Action | Key | Origin | Conflict |
|---|---|---|---|
| Show / hide the browser | `Alt+5` | Ableton §41.2 ("Move Focus to the Browser \| Alt5") | none; `SHORTCUTS.md` uses `Shift+1..7` for panels and `Alt+Shift+1..9` for workspaces |
| Search in the browser | `Ctrl+F` | Ableton §41.22 | none |
| Load the selected item into the selected track | `Enter` | Ableton §41.22 | `Enter` is GO in the performance Face; here it is the focused panel |
| Audition the selected item | `Shift+Enter` | Ableton §41.22 | none |
| Open/close folder, go into the content | `←` / `→` | Ableton §4.8 | focused panel |
| Spread across several tracks on drop | hold `Ctrl` before dropping | Ableton §7.4 | gesture |

## 8. Tests

Pure functions, `node`, one `assert` each:

| Function | Input | Expected output |
|---|---|---|
| `BR.tipoDe("x.ILD")` | — | `"laser"` (extension, case-insensitive) |
| `BR.tipoDe("x.exe")` | — | `null` |
| `BR.alvoDrop(y, tracks)` | `y` below the last track | `{acao:"novo-track"}` |
| `BR.alvoDrop(y, tracks)` | `y` over an `audio` track, item `.ild` | `{acao:"recusa"}` (rule 3) |
| `BR.alvoDrop` over a clip, x on the left half | — | `{acao:"insere", lado:"antes"}` (rule 4) |
| `BR.espalha(files, ctrl)` | 3 files, `ctrl=false` | one track, three clips in sequence |
| `BR.espalha(files, ctrl)` | 3 files, `ctrl=true` | three tracks |

In Rust, next to the `serve` tests: `POST /files/../x.ild` → 400; `POST /files/x.exe` → 400; `POST /files/a.ild` twice → 200 and 409; a file above the limit → 413.
