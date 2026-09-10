# NDI → ILDA — `spell ilda from-ndi` (FÓSFORO theme)

Receive video over NDI, vectorize each frame into paths and deliver ILDA frames to the same laser output as `ilda-player.md`. Everything that is output (PPS, blanking, color, safety, arm, shutter) is there and is not repeated here; this file covers the input and the conversion.

| Item | Who solved it best | Why |
|---|---|---|
| Objects and verbs | MadMapper | The two algorithms (contours vs skeleton), the length filter, the "keep the N longest" limit and the vectorization error field are the complete set, with real keys in the saved project |
| States | TouchDesigner | The NDI In has no FPS parameter: it has six read channels that are the health panel of the link; and `updatemethod` names the flicker |
| Screen zones | MadMapper, with the Resolume Monitor | A single preview, with four modes (source, processed, processed + polylines, polylines only); preview and output are the same widget |
| Shortcuts | Blender | Toggle with an arrow for the Canny parameters; nothing modal |
| File | MadMapper, with the `failovername` of TouchDesigner | The vectorized output is published as internal media (loopback), consumable by the rest of the graph; the NDI source has a failover name |

## 1. Objects and verbs

**NDI source.** Parameters, from the NDI In TOP [fontes/touchdesigner.md § Laser and NDI]:

| Parameter | Type | Note |
|---|---|---|
| Source/Name | enum auto-populated by the discovered streams | discovery by mDNS, "usually limited to local networks" |
| Source/Extra IPs | list of IP | for sources outside multicast |
| Source/Bandwidth | enum high, low | two values only; low is the rehearsal mode on the Pi |
| Source/Failover | name in the format `MACHINE (Source)` | from the NDI Out `failovername`, read here on the receiver side: where to migrate if the source drops |
| Source/Groups | list | filters the listed sources |

No FPS parameter: the frame rate arrives as a reading (see States). Hardware decoding only applies to NDI|HX, and "software solutions only send native NDI", so the toggle stays hidden until the source is HX. Multicast is configured outside, in the NDI Access Manager; jumbo frames of 9014 bytes fixed frame drops on gigabit, and that is a sentence from the Aprendiz, not a parameter.

**Vectorizer.** Two algorithms, with the keys of the `Process` group of `Laser Example.mad` [fontes/madmapper.md § Laser]:

- **Contours** (Canny): `Threshold`, `Size`, `Blur`. Edges of a filled shape.
- **Paths** (skeleton): `Threshold`, `Thickness` (of the stroke in the source material), `Noise reduction`, `Maximum resolution`; underneath, Zhang-Suen thinning and recomposition by angle, color and distance tolerance. Line stroke.

TouchDesigner has a third path, raster: the Scan CHOP converts the image into an oscilloscope sweep, with `width`, `height`, brightness `levels`, automatic reduction to hold the frame rate, ordering by brightness and interlacing `sweep | evenodd | max` "to minimize flicker" [fontes/touchdesigner.md § Laser and NDI]. It is deprecated there, but it is the only documented raster→vector in the six apps. It stays as the third value of the `Algorithm` enum, for when the source is text or a filled logo.

**Path filter.** `Minimum` and `maximum length` as a percentage of the largest dimension of the media; `Limit: keep the N longest` [fontes/madmapper.md § Laser]. The engine gives up above 2 000 paths; that ceiling is a visible constant, not a surprise.

**Smoothing.** Between the contour and the frame come filters from the Chataigne mapping chain: `Damping`, `Lag`, `OneEuro`, `Speed`, `CurveMap`, each returning `CHANGED | UNCHANGED | STOP_HERE` [fontes/chataigne.md § Mappings and Actions]. They are graph nodes (`orquestrador.md`), not hidden parameters of the vectorizer; here we only declare that the vectorizer output is a port of type `frame`.

**Output frame.** What comes out of the vectorizer is the same object the player delivers: a list of points with a shape `id`. The Laser CHOP groups points by the `id` channel, and without it "each point is loose and disconnected" [fontes/touchdesigner.md § Laser and NDI]; MadMapper does the same with `shapeNumber`, "every time it changes, a new path starts" [fontes/madmapper.md § Laser]. One path = one `id`.

Verbs: connect, disconnect, choose algorithm, freeze frame, record (the Capture snapshot records DMX, video, laser and camera together, and on playback it overlays the external input) [fontes/capture.md § What to copy], publish the output as internal media (MadMapper loopback, `Dispatch Count` splits it into N media, one per projector) [fontes/madmapper.md § Laser].

## 2. States

**Link health**, six readings of the NDI In, always visible when the source is chosen [fontes/touchdesigner.md § Laser and NDI]:

| Reading | State it generates |
|---|---|
| `connected` | disconnected / connected |
| `receive_fps` | received FPS; compared to the actual laser FPS it gives "the galvo can't keep up" |
| `num_source` | zero = "no source on the network", a sentence from the Aprendiz |
| `queue_size` | growing queue = rising latency |
| `received_frames` | counter |
| `missed_frames` | rising = `warning`, the sentence "dropping frames" |

Plus the ones from Capture, `Requesting` and `Receiving`: the media says what it asked for and what it is receiving, two states before "connected" [fontes/capture.md § States and messages].

**Vectorization**: paths found / limit; points needed / `PPS ÷ FPS`; MadMapper's `Monitor/Info` as an error field in text [fontes/madmapper.md § Laser]. When points needed > points available, the state is the same `warning` "the galvo can't keep up" as the player, with the cause "vectorization" instead of "clip".

**Frozen**: the current frame stays, the source keeps arriving. Independent of the shutter and of the transport (MadMapper separates freezing the engine from freezing the output) [fontes/madmapper.md § States].

**Pending**: changing the algorithm applies on the next frame; red until then [fontes/touchdesigner.md § States].

**Latency**: NDI `queue_size` plus `Delay` per output (Resolume: 0–150 ms per device, "real hardware arrives out of phase") [fontes/resolume.md § What to copy]. The delay belongs to the DMX output and to the laser output separately; that is how the vectorized laser lines up with the DMX that came from the same video.

## 3. Screen zones

- **Left**: list of discovered NDI sources, with `receive_fps` next to the name. A long list becomes a search.
- **Center, viewer**: a single one, with MadMapper's mode selector: `Source image`, `Processed image`, `Processed + polylines`, `Polylines only` [fontes/madmapper.md § Laser]. Plus the player's per-point diagnostic mode. Masks and safe area on top in every mode.
- **Right, Inspector**: `Source/`, `Vectorization/`, `Filter/`, `Output/` (the same group as the player). The `Vectorize` toggle with an arrow: it turns on and opens the parameters of the chosen algorithm; switching algorithm switches the content of the popover [fontes/blender.md § Anatomy of an editor].
- **Bottom**: no timeline of its own; the ruler is the show's. The health panel (the six readings + paths + points + actual FPS) sits in the status bar, in the statistics slot [fontes/blender.md § States].

Panel menu: `View, Select, Add, Source`.

## 4. Shortcuts

Everything from the player applies. What is added:

| Action | Key | Origin | Conflict |
|---|---|---|---|
| Cycle the viewer mode | `V` | (ours; MadMapper has the selector with no key) | none in `SHORTCUTS.md` |
| Freeze frame | `F` | (ours) | none; `Shift+F` is full screen |
| Turn vectorization on/off | `Shift+V` | (ours) | none |
| Canny parameters | popover from the arrow next to the toggle | Blender | gesture |
| Threshold by value ladder | hold the middle button | TD Value Ladder | gesture |

Rule: no source shortcut changes output state. Arm, shutter and blackout are the player's.

## 5. File

In the `.spell`:

- `sources[]`: `{uid, kind: "ndi", name, extra_ips[], bandwidth, failover, groups[]}`. A source name is a weak identity (it changes with the machine that emits it); `failover` is the declared second attempt, never a heuristic.
- `vectorizer`: `{source, algorithm (stable id: contours | paths | raster), complete params of the three algorithms, filter: {min_len, max_len, keep_longest}, publish_as: "media/<name>"}`. Keep the parameters of all three algorithms, even the inactive one, to switch without losing the tuning; that is what TouchDesigner does with the four parameter modes kept at the same time [fontes/touchdesigner.md § What to copy].
- `outputs[]`: the player's, with `delay_ms` per output.

Outside the `.spell`: list of sources seen on the network, `receive_fps`, counters, frozen frame.

What the output publishes: a `frame` in the graph, addressable as `media/<name>` (the MadMapper loopback publishes the paths of an output with no destination as Live Input, and `Dispatch Count` splits it into N media) [fontes/madmapper.md § What to copy]. That way the orchestrator routes the same vector to two lasers without duplicating the vectorizer.

Recording: a single recorder for the four streams (DMX, video, laser, camera), and "Recall DMX" to freeze a state when there is no signal [fontes/capture.md § What to copy]; the recorded file is the one from `ilda-player.md § 5` in the "at fixed FPS" mode.
