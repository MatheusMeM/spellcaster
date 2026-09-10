# Audio and video — audio plays in the engine, video plays in the GUI

Matheus's request (2026-09-09): *"I want drag n drop of elements and outputs and media and laser and audio and video"*, and before that *"nor any way to visualize how these things move in time"*. Today Spellcaster plays neither sound nor image: `spellcore/Cargo.toml` has nine members and no audio dependency, and no page has `<audio>` or `<video>`.

This file says **where each medium plays** — and the answer is different for the two, for a reason that is not preference.

Sources: **Ableton Live 12** §27 (Working with Video) and §7 (Session View), cited by section. **DaVinci Resolve 20**, waveform on the timeline, cited by page. **Resolume Arena**, *Syphon & Spout* and *Screens*, for the second-output case.

| Item | Who solved it best | Why |
|---|---|---|
| Where audio plays | none of the three (all of them are the audio host themselves) | We are a show-control with our own clock (`spellcore/engine/src/clock.rs`); the decision is ours and it is in §3 |
| Who owns the clock | Ableton §27.2.3 | *"video clips are usually set as tempo leaders, while audio clips are left as tempo followers"* — declare who leads, instead of hoping they stay together |
| Video window | Ableton §27.2.2 | Floating window, always on top, double-click goes full screen on a second monitor, `Ctrl+Alt+V` shows/hides |
| Waveform | Resolve p.643 | Three declared options (rectified or not, full or with a divider, outline) — and the most useful one: track height is independent of the drawing mode |
| Second output as a "screen" | Resolume, *Screens* | Syphon/Spout/NDI *"can be treated like a separate physical screen"*: the video output is an object of the show, just like a universe |
| Video format | none | Ableton only accepts `.mov` (§27.1). We accept what the browser accepts, which is more |

## 1. The two new tracks

```json
{"type": "audio", "clips": [{"t0": 0, "len": 85.9, "src": "media/plenaria.mp3", "offset": 0}],
 "gain": [[0, 1.0]], "clock": true}
{"type": "video", "clips": [{"t0": 12.0, "len": 30.0, "src": "media/vinheta.mp4", "offset": 0}],
 "screen": 1}
```

- `clips[]` is the same as `daw-arranjo.md §4.1`, with no exception.
- `gain` is an automation lane like any other: a `keys` with its own name, interpolated the same way. Fade in/out is two keyframes, and that is why `daw-arranjo.md` C13 needs no fade object.
- `clock: true` marks the audio track that **is** the clock (§3). One per show; the second one errors on load, not silence.
- `screen` says on which video output the track appears (§4). Absent = GUI viewer.
- No `universe`, no `address`: they are not network tracks. `timeline.rs:376` (`resolvido`) does not recognize them, so `ignored()` reports them — and that is what `pontos-falhos.md` item 13 demands stop being invisible.

Formats: audio `.wav`, `.mp3`, `.flac`; video `.mp4`, `.webm`. **Declared divergence from Ableton**, §27.1: *"Live can import movies in Apple QuickTime format (.mov)"*. We do not copy it: video plays in a WebView `<video>`, and `.mov` is precisely what the WebView usually will not open. The criterion is the player's, not the competitor's.

**We also diverge from Ableton §27.1** on *"Live will only display video for video clips residing in the Arrangement View. Movie files that are loaded into the Session View are treated as audio clips"*. Here there is no such asymmetry: the Session View is a grid of cues (`daw-sessao.md`), not a second place where the media lives.

## 2. Audio plays in the engine

**Why in the engine and not in the GUI:** audio is the clock (§3), and the clock cannot depend on a window being open. `spell play` over SSH on a Raspberry Pi, with no GUI at all, has to play the show's sound. If the audio lived in the page's `<audio>`, a show with no GUI would be mute and the clock without a reference.

**A new dependency, and the ladder of `CLAUDE.md`.** There is no audio in the Rust stdlib, and none of the already-installed dependencies (`serde`, `axum`, `tokio`, `socket2`, `clap`, `rmcp`, `schemars`, `criterion`) plays sound. Only then: **`rodio`**, which decodes `wav`, `mp3`, `flac` and `ogg` and opens the system default device, and is the smallest thing that does the whole job. It comes in as a new workspace member or as a dependency of the `engine`, behind an `audio` feature — so the Pi build can ship without it if there is no sound card.

**New commands:**

| Command | Arguments | Does |
|---|---|---|
| `audio_devices` | — | Lists the output devices, with the default marked |
| `audio_open` | `device?` | Opens; with no argument, the default |
| `audio_close` | — | Closes |
| `audio_peaks` | `file, n` | Returns `n` normalized `[min, max]` pairs from the file — this is the waveform |
| `audio_pos` | — | Real device position, in seconds (§3) |

`audio_peaks {file, n}` is read **once** by the GUI when loading the clip and drawn on the canvas; it does not go through the WebSocket every frame. `n` is the number of pixels of the clip on screen, so the cost does not grow with the file duration. Resolve p.643 gives the drawing options (rectified or mirrored, with or without a divider) and the rule that matters: *"Track heights in the Edit page are independent of the Thumbnail and Waveform view settings"* — the waveform adapts to the height, not the other way around.

`play_show` (which lives in the CLI, not in the registry) now opens the audio together with the network outputs, and closes it together.

## 3. Audio is the clock, and how that is done without rewriting the clock

Ableton §27.2.3 declares who leads: *"video clips are usually set as tempo leaders, while audio clips are left as tempo followers"*. We invert it, for a physical reason: **a sound card has its own crystal and cannot be pushed; a browser `<video>` has a `currentTime` that can be pushed.** Therefore audio leads and video follows.

The engine's clock already exists and does not need to change shape. `Clock` keeps `t0: Instant` and returns `t0.elapsed()` (`clock.rs:26,63`); `locate` already knows how to reposition the origin: `tr.t0 = Instant::now() - Duration::from_secs_f64(t)` (`clock.rs:98`).

So drift correction is **the `locate` that already exists, called now and then with the card's value**:

1. With no `clock: true` track, nothing changes: the clock is the `Instant`, as today.
2. With a `clock: true` track, every second the engine compares `clock.time()` with `audio_pos()`.
3. Difference below tolerance: nothing. Above: reposition the origin, without skipping or repeating a frame (the next `frame()` already comes out in the right place).
4. **Tolerance: 20 ms**, with the number in the `ponytail:` comment and the criterion written down — 20 ms is below the threshold at which the ear separates two transients, and above the jitter that `Clock` itself already measures and reports (`Stats { p50, p99, max, drift }`, `clock.rs:14-21`). If the card drifts more than that per second, the problem is the card and `drift` already shows up in the panel.

It is one function (`Clock::sync(pos)`), three lines, reusing `locate`. No second clock, which is what rule 9 of `FUNCOES/README.md` forbids.

## 4. Video plays in the GUI

**Why in the GUI:** decoding H.264 in Rust is a heavy dependency (or a system `ffmpeg`) to deliver pixels that would have to go back to the screen. The WebView already has a video decoder, hardware acceleration and an element with a writable `currentTime`. Writing a decoder next to it is the definition of code that does not need to exist.

**Viewer, inside the editor window.** A `<video>` in the place `SHORTCUTS.md § Interface` has already reserved (*"large central viewer (here: previz or universe VUs)"*), sharing space with the previz of the `previz` workstream.

**Sync, the contract:**

- The `<video>` does **not** play on its own following its own clock. On every `transport` event from the bus, the page compares `video.currentTime + clip.t0 - clip.offset` with the engine's `t`.
- Difference below tolerance: let it run (letting the video run on its own is what gives a fluid image).
- Difference between the tolerance and a larger limit: correct with `playbackRate` (0.97 to 1.03) until it closes — this is what avoids the visible jolt of a `seek`.
- Above the larger limit: `currentTime = ...`, and the jolt is accepted because the alternative is a wrong image.
- **Tolerance 40 ms** (a bit more than one frame at 25 fps) and **limit 250 ms**, both with `ponytail:` and a criterion: below 40 ms nobody sees it; above 250 ms the `playbackRate` would take longer than the jump itself to close the gap.
- Transport stopped: the video stays paused on the right frame, and `locate` does a `seek`. Scrubbing the ruler does a `seek` without playing.

**Second window, on the projector's monitor.** Ableton §27.2.2: *"a separate, floating window that always remains above Live's main window"*, and *"The video can be shown in full screen (and optionally on a second monitor) by double-clicking in the Video Window"*, with `Ctrl+Alt+V` showing and hiding (§41.1). Resolume, *Screens*: an output is an object of the show and *"Every output can only have a single screen associated with it"*.

Here that is: **a second wry window, with no bar, on the chosen monitor, showing only the video tracks whose `screen` points at it.** The two windows talk to the same engine over the same bus, so the sync is the same contract above, running twice. `show.outputs[]` gains `{"type":"screen","monitor":1}`, and the track's `screen` (§1) is the index of that output; whoever drags the output onto the track is `browser-dnd.md §4`, with no new gesture.

**This depends entirely on the `gui-janela` workstream.** Today there is no window crate at all (`spellcore/Cargo.toml:3`), so there is no first window, let alone a second. What comes in **now**, without waiting: the `video` track, the clip, the drawing on the timeline and the viewer inside the page. What waits: the second window and `screen`.

## 5. NDI and Spout: what blocks, exactly

`ndi-ilda.md` already specifies the whole NDI function (source, vectorization, link health) and it still holds. What this file adds is why it does not come in now, and what comes in instead.

- **NDI** needs the NewTek/Vizrt SDK, which is a registered download with its own license; there is no crate that can be vendored into the repo, and `FUNCOES/README.md §14` requires everything we load to be versionable text.
- **Spout** (Windows) and **Syphon** (macOS) are GPU texture sharing. Resolume, *Syphon & Spout*: input *"always enabled"*, output turned on through the Output Menu, and the announced name is `App Name` + `Server Name`. For us that would require a GPU context shared between wry and the process — which is exactly the thing the WebView does not expose.
- **What comes in instead, and solves the real case:** the second wry window full screen on the projector (§4). The reason NDI/Spout exists in a show is to take an image from here to the projector or to another piece of software; for the projector, the window solves it, and with no SDK at all.
- **What stays blocked:** sending an image to *another program* (Resolume, OBS) and receiving an image *from* another program. That is real NDI/Spout and it is written down as a license blocker, not an effort one.

## 6. What stays out

- **`.mov`** (Ableton §27.1). See §1: the criterion is the player's.
- **Consolidate / Reverse / Crop swapping video for audio** (Ableton §27.2.1: *"This replacement only occurs internally — your original movie files are never altered"*). We have none of the three; `daw-arranjo.md` C11 already refused Consolidate for the same reason — we do not render media.
- **Warp markers on the video defining hit points** (Ableton §27.2.3.1). That is musical time, and `timeline-daw.md` item 38 already refused bars.
- **Audio mixer, sends, returns.** An audio track with `gain` and one output device is enough for a show. A mixer is another product.
- **Audio input (recording from the microphone, beat detection).** No request. When there is one, the door is `input {key:"audio:level"}`, which the graph already knows how to read.
- **Per-track audio with routing to different outputs.** One device, one stereo pair. The limit sits in a `ponytail:` on `audio_open`.

## 7. Shortcuts

| Action | Key | Origin | Conflict |
|---|---|---|---|
| Show / hide the video window | `Ctrl+Alt+V` | Ableton §41.1 | none |
| Full screen for the video window | double-click on it | Ableton §27.2.2 | gesture |
| Back to original size | `Alt`+double-click | Ableton §27.2.2 | gesture; and it is the `Alt = variant` of our grammar |

## 8. Tests

Rust, next to the tests that already exist in `engine/tests/`:

| What it proves | How |
|---|---|
| `Clock::sync` corrects | clock at `t=10.0`, `sync(10.030)` → `time()` becomes `≈10.030`; `sync(10.005)` → does not move |
| `Clock::sync` with no `clock` track | is never called; the `Instant` stays in charge |
| Two tracks with `clock: true` | error on load, with the index of both |
| `audio_peaks` | a 1 s file with a sine wave, `n=10` → 10 pairs, all with `max > 0.9` and `min < -0.9` |
| `audio_peaks` with `n` larger than the number of samples | does not blow up; returns `n` pairs |
| `audio` track with no device open | the show plays in silence and **warns**, it does not fail (it is the "no feed" state of `ilda-player.md §2`) |

`node`, in the GUI:

| What it proves | How |
|---|---|
| `AV.corrige(dt)` | `dt = 0.02` → `{acao:"nada"}`; `dt = 0.1` → `{acao:"rate", rate:1.03}`; `dt = 0.5` → `{acao:"seek"}` |
| `AV.tempoDoVideo(clip, t)` | `t` outside the clip → `null` (the `<video>` stays paused, not on a wrong frame) |

Visual proof: headless screenshot of `index.html` with an `audio` track showing the waveform and a `video` track showing the clip, and the viewer with the frame matching the playhead.
