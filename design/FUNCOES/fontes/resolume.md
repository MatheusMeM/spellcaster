# Resolume Arena 7.21.1 — interface audit for Spellcaster

Audit made by reading only files from the installation and from the user's documents folder on this machine.
Arena was not opened. Nothing here comes from memory: every statement cites the file read.
Reference version in every user file: `versionInfo majorVersion=7 minorVersion=21 microVersion=1 revision=37851`.

## Sources read

| File | What it is |
|---|---|
| `C:\Program Files\Resolume Arena\default\Example.avc` (354,122 B) | factory composition, XML; 2,758 elements, 56 distinct tags |
| `C:\Users\email\Documents\Resolume Arena\Compositions\aniver 30.avc` (197,486 B) | the user's real show; 1,364 elements, 42 tags |
| `C:\Users\email\Documents\Resolume Arena\Compositions\climao.avc` (1,207,870 B) | the user's real show; 4,373 elements, 42 tags |
| `...\Shortcuts\Keyboard\Default.xml` | the whole keyboard map, 42 shortcuts; identical to the factory one in `default\Shortcuts\Keyboard\Default.xml` (only `presetId` and `versionInfo` differ) |
| `...\Shortcuts\MIDI\{Default,APC MINI,APC 40 MK II,Nano Kontrol 2}.xml` | MIDI mappings; `Default.xml` is the user's (7 faders of the APC40 mkII) |
| `...\Shortcuts\OSC\{Default,OSC_TD_TEST}.xml` | `Default` empty; `OSC_TD_TEST` with 3 output mappings |
| `...\Shortcuts\DMX\Default.xml` | `<ShortcutManager name="DMXShortcutManagerShortcuts"/>` — empty |
| `...\Shortcuts\activePresets.xml` | which preset is active per protocol |
| `...\Fixture Library\*.xml` (24 user fixtures) and `default\Fixture Library\LED 16.xml` (the only factory one) | personality schema |
| `...\Preferences\{config,recentLayout,dmx,midi,osc,AdvancedOutput,SimpleOutput,server}.xml` | preferences and panel layout |
| `...\Presets\Advanced Output\amiver 30.xml` (78,923 B) | the show's real DMX patch: 1 Lumiverse, 11 fixtures |
| `...\Presets\Interface\*.xml` (9) and `default\Presets\Envelopes\*.xml` | layout and envelope presets |
| `C:\Program Files\Resolume Arena\docs\help\English.xml` | 412 contextual-help entries (`<help components> → <title> + <text>`) |
| `C:\Program Files\Resolume Arena\docs\gui\English.txt` | 1 GUI strings file, format `"key"="value"` |
| `C:\Program Files\Resolume Arena\rest\docs\swagger.yaml` (208,471 B) | OpenAPI of the REST API: the cleanest data model the product exposes |

Skins/themes: **they do not exist**. A `find` for `*.css`, `*skin*`, `*theme*`, `*.qss` in `C:\Program Files\Resolume Arena` returns only
`rest\docs\index.css`, `rest\docs\swagger-ui.css` and `rest\landing\index.css` — style sheets of the Swagger UI and of the webserver landing, not of the application.
What Arena calls an "interface preset" is only a panel arrangement: `Presets\Interface\Panels Left & Right.xml` uses the same `<Layout>` schema as `recentLayout.xml`.

## Screen anatomy

Evidence: `...\Preferences\recentLayout.xml`. The layout is a `Window → FlexContainer(orientation) → TabbedContainer → <PanelX>` tree,
with `width_desired` / `height_desired` per panel — console layout, not responsive.

Panels instantiated in the user's real session (in the order they appear in the XML):

- `<LayersAndClips>` — **the clip grid and the layer strip are ONE single panel** (`recentLayout.xml`, `Window/FlexContainer/FlexContainer/FlexContainer/LayersAndClips`, `height_desired=318`). It takes the top half of the window.
- `<Toolbar>` (`height_desired=36`) — thin strip between the grid and the lower panels.
- Two `<Monitor>` stacked on the left: `instance=0 subject_type="Composition" checker_board=0 showUI=0` and `instance=1 subject_type="Preview" checker_board=1 showUI=1`. That is: **output and preview are the same parameterized widget**, not two components.
- `<TabbedContainer>` with `<Composition>` + `<Layer>` (property tabs for whatever is selected).
- `<TabbedContainer>` with `<Clip>`.
- `<TabbedContainer>` with `<Files>`, `<Compositions>`, `<Effects>`, `<Sources>`, `<Recording>` — the browser is a stack of tabs, not a tree.

In `<HiddenPanels>` (they exist, they are closed, and they remember the tab they came from via `unhide_panel_id` + `unhide_target_index`):
`RenderQueue`, `ClipTime` (with two `<Clock mode= show_time_remaining=>`), `Notes`, `Group`, `Slices`, `SMPTE` (two fields with `colorIndex`), `Shortcuts`.

Below, `<Params name="LayoutParams">` is a flat list of 33 `Show X` booleans — the whole interface is turned on and off by checkbox,
not by "mode" or "workspace": `Show Compositions`, `Show Layer`, `Show Clip`, `Show Composition`, `Show Group`, `Show Ableton Link`,
`Show Audio Controls`, `Show Layer Transport Controls`, `Show Layer Transition Controls`, `Show Crossfader`, `Show Help`, `Show Dashboard`,
`Show Autopilot`, `Show Cue Points`, `Show Beat Looper`, `Show FFT Gain`, `Show Undo Toolbar`, `Show Monitor`, `Show SMPTE`,
`Show Denon DJ StageLinQ`, `Show Pioneer DJ TCNet`, `Show FPS & Stats`, `Show Shortcuts`, `Show Effects`, `Show Sources`,
`Show Clip Time`, `Show Notes`, `Show Slices`, `Show Files`, `Show Render Queue`, `Show Recordings`, `ShowFileBrowserThumbnails`, `showClipTimeRemaining`.

Consequence for Spellcaster: each of these `Show X` is a show line the operator turns on as the night requires, and the saved layout is a named preset (`Presets\Interface\Big Monitor.xml`, `Dual Display.xml`, `Many Layers.xml`, `Triple Display.xml`, ...). That is exactly the Face/Theme separation of `design/PRINCIPIOS.md §5`, only without the Theme.

## Objects and verbs

Canonical model in `rest\docs\swagger.yaml`, section `components/schemas` (line 5174 onwards).

| Object | Where | Verbs / fields observed |
|---|---|---|
| `Composition` | swagger:6037 | `bypassed`, `master`, `speed`, `cliptarget`, `cliptriggerstyle`, `clipbeatsnap`, `dashboard`, `audio`, `video`, `crossfader`, `decks[]`, `layers[]`, `columns[]`, `layergroups[]`, `tempo_controller`; endpoints `POST /composition/action` (`undo`/`redo`, swagger:109) and `POST /composition/disconnect-all` (swagger:138) |
| `Deck` | swagger:5724 | `closed` (bool, read-only), `name`, `colorid`, `selected`, `scrollx`; `/decks/{i}/select`, `/decks/add`, `/duplicate`, `/by-id/{id}/open`, `/by-id/{id}/close` |
| `Column` | swagger:5682 | `name`, `colorid`, `connected` (**ChoiceParameter**, read-only); `/columns/{i}/connect`, `/columns/add`, `/duplicate` |
| `Layer` | swagger:5947 | `bypassed`, `solo`, `crossfadergroup`, `master`, `maskmode`, `ignorecolumntrigger`, `faderstart`, `dashboard`, `audio`, `video` (`VideoTrackLayer` with `autosize`), `transition`, `clips[]`, `autopilot`; `/select`, `/clear`, `/clearclips`, `/add`, `/duplicate`, `/effects/video/{add,move,offset}` |
| `LayerGroup` | swagger:5995 | everything from Layer minus `maskmode`/`faderstart`, plus `speed`, `layers[]`, `/move-layer`, `/add-layer`, `/layergroups/{i}/columns/{c}/connect` (**column inside the group**) |
| `Clip` | swagger:5612 | `name`, `colorid`, `selected`, `connected` (Choice), `target`, `triggerstyle`, `ignorecolumntrigger`, `faderstart`, `beatsnap`, `transporttype`, `transport`, `dashboard`, `audio`, `video`, `thumbnail`; `/connect`, `/select`, `/open`, `/openfile`, `/clear`, `/thumbnail/{last-updated}` |
| `AutoPilot` | swagger:5555 | a single field: `target` (ChoiceParameter). The rest of the autopilot (duration, action) lives in the clip: help `Duration` = "how long this clip will play when the autopilot is active", `Autopilot Action` = "Set the behaviour of the clip when in Autopilot mode", `Loops` = "Determine how many times the clip repeats" |
| `TransportTimeline` / `TransportBPMSync` | swagger:5590 and 5562 | `position` + `controls`: `playdirection`, `playmode`, `playmodeaway`, `duration`, `speed`; the BPM version adds `bpm`, `syncmode`, `beatloop` |
| `LayerTransition` | swagger:5874 | `duration`, `blend_mode` |
| `CrossFader` | swagger:5701 | `phase`, `behaviour`, `curve`, `sidea` (`ParamEvent`), `sideb` (`ParamEvent`), `mixer` |
| `TempoController` | swagger:5933 | `tempo` (Range) + four `ParamEvent`: `tempo_pull`, `tempo_push`, `tempo_tap`, `resync` |
| `Effect` / `VideoEffect` / `AudioEffect` | swagger:5751, 5463 and 5445 | `idstring`, `name`, `display_name`, `bypassed` (nullable — "primary Transform for example is not allowed to be bypassed"), `mixer`, `params`, `effect`, `presets[]` |
| `Source` | swagger:5802 | `idstring`, `name`, `presets[]` |
| `Screen` / `Slice` / `Lumiverse` / `Fixture` | not in the REST API | they exist only in Advanced Output (see §Fixtures and DMX) |

Verbs that exist only as a shortcut path, not as a REST resource (read in `Shortcuts\Keyboard\Default.xml`):
`/composition/connectnextcolumn`, `/connectprevcolumn`, `/selectnextdeck`, `/selectprevdeck`,
`/composition/selectedlayer/connectnextclip`, `/connectprevclip`, `/composition/selectedlayer/clear`, `/composition/disconnectall`.

Ejection vocabulary, straight from the help (`docs\help\English.xml`): `Clear All Layers` = "Eject ALL the clips. From ALL the layers.";
`Clear Layer` = "Clear the playing clip from this layer."; `Layer Solo` = "Show this layer, the whole layer and nothing but this layer.";
`Column Trigger` = "Start all clips in this column at once."; `Decks` = "Decks are like your records, and your composition is your record bag."

## States

- **A connected clip is not a boolean.** `Clip.connected` and `Column.connected` are read-only `ChoiceParameter` (swagger:5633 and 5697), not `ParamBoolean`. The help confirms how many states: in `MIDI Out Status` — "for instance clip triggers can have 5 different states! Colors galore!" (`docs\help\English.xml`). Each state has its own LED color on the controller.
- `selected` is a **read-only** `ParamBoolean` in Clip, Layer, LayerGroup and Deck. Selection is a consequence of `/select`, not an editable field.
- `Deck.closed` is a read-only bool; the deck list appears in the `.avc` as `<DeckInfo name id closed>` (e.g.: `aniver 30.avc`, three `DeckInfo` with `closed="0"`).
- `bypassed` and `solo` are `ParamBoolean` in Layer and LayerGroup; `bypassed` also exists in the Composition (the "Master Bypass" = "Makes all output go black.").
- Tempo state: the four events of the `TempoController` are `ParamEvent` — they hold no value, they only fire. The `.avc` stores only `Tempo` (`Example.avc` = 128; `climao.avc` = 129.69098774160480048; `aniver 30.avc` does not even serialize it, it sits at the default 120).
- Fixture state: **two enable levels**, `Enable Lumiverse` ("Turn off the fixtures in this Lumiverse in one fell swoop") and `Enable Fixture` ("This will stop Resolume from sending DMX data to this fixture") — both in `docs\help\English.xml`.
- **There is no "armed" state nor "rehearsal" state anywhere.** Not in the swagger, not in `recentLayout.xml`, not in the help. As soon as a Lumiverse exists in `Presets\Advanced Output\amiver 30.xml`, it sends. The only brake is the `Enabled` of the screen/fixture.

## Shortcuts and mapping

### Factory keyboard map (42 shortcuts)

`Shortcuts\Keyboard\Default.xml`. Each `<RawInputMessage key="N">` is `keycode << 32 | modifiers`;
the keycodes are ASCII for ordinary keys and `0x10025`/`0x10027` (65573/65575) for left/right arrow.
Modifiers observed: `0` (none), `1` (Shift), `3` (Shift + a second modifier).

| Key | Target (`path` in the XML) |
|---|---|
| `1` `2` `3` | `/composition/layers/{1,2,3}/select` |
| `Shift`+`1`…`9` | `/composition/columns/{1..9}/connect` |
| `Q W E R T Y U I` | `/composition/selectedlayer/clips/{1..8}/connect` |
| `←` / `→` | `/composition/selectedlayer/connectprevclip` / `connectnextclip` |
| `Shift`+`←` / `→` | `/composition/connectprevcolumn` / `connectnextcolumn` |
| `Shift`+mod2+`←` / `→` | `/composition/selectprevdeck` / `selectnextdeck` |
| `Space` | `/composition/tempocontroller/tempotap` |
| `,` / `.` | `/composition/tempocontroller/tempo` with `Subtarget type="1" optionIndex="8"` / `optionIndex="7"` (nudge down/up) |
| `/` | `/composition/tempocontroller/resync` |
| `B` / `Shift`+`B` | `/composition/selectedlayer/bypassed` / `/composition/bypassed` |
| `S` | `/composition/selectedlayer/solo` |
| `X` / `Shift`+`X` | `/composition/selectedlayer/clear` / `/composition/disconnectall` |
| `M` / `Shift`+`M` | `/composition/selectedlayer/master` / `/composition/master` |
| `V` | `/composition/selectedlayer/video/opacity` |
| `A` | `/composition/selectedlayer/audio/volume`, with `ValueRange min=0.0630957…` (logarithmic fader, −24 dB) |
| `P` | `/composition/selectedclip/video/effects/transform/positionx` **and** `positiony`, on the same `P`, with a different `behaviour` (4098 vs 12290) and `ValueRange 0.4375..0.5625` |
| `G` | `/composition/objects/1505384963468/video/effects/addsubtract/effect/g` — shortcut tied to an object by numeric id |

The grammar that comes out of it: **no modifier acts on the selected one, `Shift` climbs one scope level** (layer→column, layer→composition),
top-row letters are the clip column of the selected layer, and the same `P` becomes the X or the Y axis according to the `behaviour`
(horizontal/vertical mouse, see strings `"Mouse"`, `"Horizontal"`, `"Vertical"`, `"Sticky"` in `docs\gui\English.txt:427-430`).
There is no `Space` for play/pause: `Space` is the BPM tap. There is no GO shortcut, no cue, no blackout, no armed-output shortcut.

### XML grammar of a mapping (the same in all four protocols)

```
<Shortcut uniqueId behaviour paramNodeName inputDeviceName outputDeviceName hasCustomOutputPath>
  <ShortcutPath name="InputPath"          path="/composition/..." translationType allowedTranslationTypes/>
  <ShortcutPath name="OutputPath"         path="..."/>   <!-- feedback: OSC address or MIDI device -->
  <ShortcutPath name="InputSiblingPath"   path=".../selected"/>  <!-- where to read the state from -->
  <ShortcutPath name="OutputSiblingPath"  path=".../selected"/>  <!-- where to send the state back to -->
  <Subtarget type optionIndex/>
  <ValueRange min max/>
  <RawInputMessage key value numSteps/>
  <NamedValues><Value first="Off" second="0"/><Value first="On" second="1"/></NamedValues>
</Shortcut>
```

- `translationType` / `allowedTranslationTypes` encode the scope of the target. The help (`Shortcut Target`) names the three: *"Selected..."* (the clip/layer/group selected at that moment), *"This..."* (that specific object, wherever it goes) and *"By Position"* (by index: always the first layer, even after reordering). In the XML: `1` for absolute targets (`/composition/columns/3/connect`), `8` for selected-layer targets, `2`/`4` for per-object/per-selected-clip targets; `allowed` is the mask of what that target accepts (`1`, `3`, `7`, `11`).
- `RawInputMessage key` is an 8-byte packet: `[type:1][device hash:4][data1:2][status:1]`.
  Verified: APC MINI `0x0100000000000790` = type 1, device 0 (any), note `0x07`, status `0x90` (NoteOn ch1);
  `Shortcuts\MIDI\Default.xml` `0x0204cc397d0007b2` = type 2, device `04cc397d` (APC40 mkII), CC `0x0007`, status `0xB2` (CC ch3);
  OSC `0x0600000000000000` = type 6 and nothing else — matching is by address, the `path` is already the key.
  The keyboard is type 0 with keycode and modifier in the low 32 bits.
- `numSteps="128"` appears in every MIDI mapping: the controller resolution belongs to the mapping, not to the parameter.
- `NamedValues` is the LED color table (`<Value first="On" second="0.039370078740157479769"/>` = velocity 5/127 on the APC MINI).
  Help `MIDI Out Velocity`: "This will set the color of the pad."

### Shortcut modes

Read in `docs\gui\English.txt:409-430 and :467`: `Absolute`, `Button`, `Relative`, `Fake relative`, `Piano`, `Value`, `Toggle`;
plus `Mouse` / `Horizontal` / `Vertical` / `Sticky`, `Invert Value`, `Range`, `14-Bit`, `CC` / `CC Fine`, `Loop`, `Step Size`,
`Select Next Item`, `Select Previous Item`, `Random Item`, `Jump To Playhead`.
Help `Shortcut Mode`: "The two most important ones to know about are Toggle and Value. Toggle will let you switch between two values with each press. Value will always set the parameter to a fixed value."
Help `Piano Mode`: "On key down the parameter will jump to the max value. When the key is released the parameter will jump back to the min value." (= momentary)
Help `MIDI Controller Mode`: "Endless dials should be set to 'Relative'. Fixed MIDI controllers to 'Absolute'."
In the XML this becomes the `behaviour` field (values seen: `0`, `8`, `26`, `1024`, `1028`, `4098`, `12290`, `14338`) — undocumented bitfield.

### DMX input mapping

`Shortcuts\DMX\Default.xml` is **empty**. The grammar is only in the help: `DMX Lumiverse` ("Choose which internal Resolume universe this DMX shortcut is part of"),
`DMX Channel` and `DMX Channel Offset` ("shift all the assigned DMX channels up or down"); and in the strings `"Universe:"`, `"Channel:"`, `"With offset"`, `"Send DMX channel to assign it to this interface element."` (`docs\gui\English.txt`).
`Preferences\dmx.xml` on this machine: `<DmxController automapEnabled="1" artNetName="Arena nautilus" bindAdapter="ethernet_32768"><Inputs/></DmxController>` — no input configured.

### Preset state on this machine

`Shortcuts\activePresets.xml` holds four ids (`DMXShortcutPreset`, `KeyboardShortcutPreset`, `MidiShortcutPreset`, `OSCShortcutPreset`).
The compositions reference presets **by name**: `climao.avc` has `Param name="OscShortcutPreset" value="OutputAllMessages"` — and there is no
`OutputAllMessages.xml` in `Shortcuts\OSC\` (only `Default.xml` and `OSC_TD_TEST.xml`). Reference by name that breaks without warning.

## Fixtures and DMX

### Schema of a personality

25 files read (24 from the user + the factory `LED 16`). The complete schema, without exception, is:

```
<Fixture uuid="<32 hex>" fixtureName="...">
  <versionInfo .../>
  <Params>
    <ParamRange storage="0" name="Dimmer" T="DOUBLE" default="0" value="255">
      <PhaseSourceStatic phase="1"/><BehaviourDouble/>
      <ValueRange name="defaultRange" min="0" max="255"/>
      <ValueRange name="minMax"       min="0" max="255"/>
      <ValueRange name="startStop"    min="0" max="255"/>
    </ParamRange>
    ...                                   <!-- 1 ParamRange = 1 DMX channel -->
    <ParamFixturePixels name="Pixels">    <!-- at most one per fixture -->
      <ParamRange name="Width"/> <ParamRange name="Height"/>   <!-- 1..512 -->
      <ParamChoice name="Color Format" default="rgb" value="rgbw"/>
      <ParamChoice name="Distribution" T="INT32" value="170"/>
      <ParamRange  name="Gamma" default="2.5" min="1" max="3"/>
    </ParamFixturePixels>
  </Params>
</Fixture>
```

Count across the 25 files: `ParamRange` 123×, `ParamFixturePixels` 23×, `ParamChoice` 46×, `ValueRange` 357×. **No other node type.**

What this means in practice:

- **There is no channel taxonomy.** There is no `dimmer`, `pan`, `tilt`, `strobe`, `color wheel` type. Only `ParamRange 0..255` with a free-form `name`. The user's real fixtures prove it: `core - PARLED` has `Dimmer` + `Pixels(rgbw)`; `core - ribalta43ch tilt` has `Dimmer `, `Shutter`, `Pixels(l)`; `srg - strobo fita 11chrgb` has `Dimmer`, `Shutter`, `Pixels(rgb)` and two more channels called `New Parameter N`.
- **There is no 16-bit**, no fine/coarse channel, no labeled ranges (nothing like "0-7 = closed, 8-134 = strobe"). `teste_parametros.xml` is 34 identical `ParamRange` called `New Parameter 1..34` — that is how the user patched a 34-channel moving light.
- **Order = address.** The order of the elements in `<Params>` is the channel order. The `DMX Parameter` help says "You can drag parameters around to change their order."
- The pixel block is the heart of it: `Color Format` observed with values `rgb`, `rgbw`, `gbr`, `brg`, `l` (luminance); `Distribution=170` in 100% of the files (help `Distribution`: "The open arrow signifies the first channel, the direction of the subsequent arrows define how the channels 'snake' through the fixture"). The factory `LED 16` uses the old key `Color Space="1"` instead of `Color Format` — format drift between versions.

### The patch: Lumiverse, Screen, Slice

`Preferences\AdvancedOutput.xml` is only a pointer (`<ScreenSetup presetFile="amiver 30"/>`); the whole patch is in
`Presets\Advanced Output\amiver 30.xml`. Real structure of the show:

```
ScreenSetup
 └ CurrentCompositionTextureSize width=1920 height=1080
 └ screens
    └ DmxScreen name="artnet" uniqueId="1784918734030" LumiverseId="1"
       ├ Params: Name, Enabled, Hidden, Auto Span, Align Output
       ├ Params Output: Opacity, Brightness, Contrast, Red, Green, Blue   (-1..1)
       ├ guides: 2× ScreenGuide (grid and reference image)
       ├ OutputDevice → OutputDeviceDmx name="Lumiverse" deviceId="Lumiverse"
       │    Framerate (1..40, default 30, value 30) · Delay (0..150 ms, value 40)
       │    Subnet=0 · Universe=1
       └ layers: 11× DmxSlice
            Params Common : Name, Enabled
            Params Input  : Input Source · Fixture (uuid) · Start Channel (1..131072) · Filter Mode
            Params Output : Flip, Brightness, Contrast, Red, Green, Blue, Soft Edge
            InputRect  : 4 vertices in composition pixels    (e.g.: 400.5,810.75 → 693.5,1009.75)
            OutputRect : 4 normalized vertices (-0.5..0.5)
            FixtureInstance → Fixture (whole copy of the personality with the live values)
```

The show's 11 slices, with name, source and start channel:

| Slice (name given by the user) | Input Source | Fixture (uuid) | Start Channel |
|---|---|---|---|
| `40 - 68 core - atomic39ch color` | `3:2` | `13ed8267…` | 40 |
| `80 - 108 core - atomic39ch color` | `3:2` | `13ed8267…` | 80 |
| `120 - 148 core - atomic39ch color` | `3:3` | `13ed8267…` | 120 |
| `170 - 198 core - atomic39ch color` | `3:3` | `13ed8267…` | 170 |
| `240 - 268 core - atomic39ch color` | `3:5` | `13ed8267…` | 240 |
| `300 - 304 core - PARLED` … `330 - 334` | `3:1` / `3:4` | `b4e49617…` | 300, 310, 320, 330 |
| `500 - 502 FOG` / `505 - 507 FOG` | `3:7` / `3:6` | `649146e3…` | 500, 505 |

Direct readings:

- **DMX in Arena is sampled video.** A fixture does not receive values: it receives the pixels that fall inside its `InputRect` on the composition texture, converted by the `Color Format`/`Distribution`/`Gamma` of the personality. Help `Fixture Output Preview`: "Preview of the RGB data that this fixture is sending out."
- **Lumiverse ≠ universe.** `Start Channel` runs from 1 to 131,072 = 256 × 512. The Lumiverse is a continuous address space that the `OutputDeviceDmx` slices into Art-Net universes (`Subnet` 0-15 × `Universe` 0-15, per the `Subnet Number` and `Universe` help). Help `DMX Output`: "Here you can visually re-arrange the start channels of your fixtures or check for any overlap."
- **The user addresses by the slice name.** No XML field stores "40 to 68"; he wrote that in the `Name` because the UI does not show the occupied range in the list. Symptom of a gap in the interface, not style.
- The personality is **copied** into the patch (`FixtureInstance/Fixture`), with **`fixtureName=""`** and a new `uuid` — the link to the library is only the slice's `ParamChoice name="Fixture"`. Editing the personality in the library does not update the show, and the show no longer knows the name of what it loads.
- The only temporal adjustment of DMX is the `Delay` (0-150 ms) on the output device, applied to the whole Lumiverse. There is no per-fixture delay, no fade, no per-channel dimmer curve.

## Parameter types

Two grammars for the same model. The REST one (clean) and the file XML one (real).

### REST (`swagger.yaml`, `components/schemas`, line 5174+)

| Type (`valuetype`) | Own fields | Widget the API suggests |
|---|---|---|
| `ParamBoolean` | `value: bool` | toggle |
| `ParamChoice` (and `ParamState`) | `value: string`, `index: int`, `options: [string]` | `choice_buttons` or `choice_combobox` |
| `ParamColor` | `value: "#rrggbb[aa]"`, `palette: [string]` | `color_picker`, `color_pallette`, `slider_color_*` |
| `ParamEvent` | **none** — it only fires | trigger button |
| `ParamNumber` | `value: int64` | `spinner` |
| `ParamRange` | `min`, `max`, **`in`**, **`out`**, `value` | `slider` / `rotary` |
| `ParamString` | `value: string` | `text` |
| `ParamText` | `value: multiline string` | `text_multiline` |

They all carry `id: int64` and `view: ParameterView`. The `ParameterView` (swagger:5202) is the presentation layer, separate from the value:
`suffix` (e.g.: `%`), `step`, `multiplier`, `display_units` ∈ `{real, integer, percent, degrees, decibels, frames_per_second, milliseconds, seconds, beats, fractions}`,
`control_type` ∈ `{based_on_param, choice_buttons, choice_combobox, spinner, duration_spinner, slider, slider_color_red…alpha…opacity, color_pallette, color_picker, rotary, text, text_multiline}`.

Three details worth more than the rest:

1. **`in` / `out` in the `ParamRange`** — "The lowest/highest value we clamped the range to". The useful range is separate from the physical range. That is what makes the keyboard shortcut `A` map volume in `0.0630957..1` without altering the parameter.
2. **`ParamEvent` is a type, not a `bool` that goes back to zero.** Trigger and toggle are different things from the schema up: `CrossFader.sidea`/`sideb` and the four of the `TempoController` are `ParamEvent`.
3. **`ParameterCollection`** is a map `name → parameter of any type` (swagger:5431). `dashboard`, `mixer`, `params`, `effect` and `sourceparams` use it. There is no closed effect schema: the effect describes its own parameters.

### File XML (`.avc`, fixtures, presets)

Different names for the same model: `<Param T="STRING|BOOL|UINT32|INT32|UINT8|COLOR|DOUBLE">`, `<ParamRange>`, `<ParamChoice storeChoices>`,
`<ParamColor channelmode paletteEnabled color interpolated>`, `<ParamText>`, `<ParamPixels>`, `<ParamFixturePixels>`.
A `ParamRange` always carries three `<ValueRange>`: `defaultRange`, `minMax`, `startStop`.

And it carries **the animation source as a child**, which is the most interesting mechanism in the file:

| Node | What it is | Occurrences |
|---|---|---|
| `PhaseSourceStatic phase=` | static value | 361 / 191 / 755 (Example / aniver / climao) |
| `PhaseSourceTimeline` | follows the clip timeline | 3 / 15 / 31 |
| `PhaseSourceTransportTimeline` | follows the transport, with `defaultBeatsDuration` and `defaultMillisecondsDuration` | 47 / 17 / 63 |
| `PhaseSourceDashboardLink linkId="/link1" linkName="RGB"` | follows a Dashboard dial | 8 / 0 / 0 |
| `Modifier → ModifierEnvelope → points → point x y curve` | drawn envelope | 2 in `Example.avc` |

That is: **any numeric parameter can become animated by swapping the child**, without changing the parameter type.
The envelopes are presettable: `default\Presets\Envelopes\ADSR.xml` is `<Preset uniqueId="MOD_ENVELOPE" className="Envelope">` with 5 `point x/y/curve`
(`curve` an integer: 1, 12, 33 seen). Help `Envelope`: "Double click to add and remove keyframes, right click each keyframe to change its interpolation."

## File (.avc)

An `.avc` is pure XML, root `<Composition>`, no compression.

```
Composition (name uniqueId numDecks currentDeckIndex numLayers numColumns compositionIsRelative)
 ├ versionInfo
 ├ CompositionInfo (name description width height)
 ├ Params  : Name · Speed · Beat Snap · KeyboardShortcutPreset · MidiShortcutPreset · OscShortcutPreset · DmxShortcutPreset
 ├ Params name="Dashboard" : Link 1..Link 8
 ├ TempoController → Params → ParamRange Tempo
 ├ CrossFader · ClipTransition · CompositionView(FoldParams/FoldState) · Notes(Note)
 ├ composition VideoTrack / AudioTrack (chained RenderPass)
 ├ DeckInfo × N  (name id closed)
 ├ Deck (uniqueId closed numLayersWithContent numColumnsWithContent numLayers numColumns deckIndex)
 │   └ Clip (uniqueId layerIndex columnIndex)
 │        ├ Params : Name · LoadProgress · TransportType
 │        ├ PreloadData → VideoFile / AudioFile
 │        ├ Transport → Params → ParamRange Position → DurationSource
 │        │                                          → PhaseSourceTransportTimeline → Beats_d / Beats_double
 │        ├ ClipView → FoldParams → FoldState
 │        └ VideoTrack (manualDuration) : Width Height RScale GScale BScale AScale
 │             ├ RenderPass(type=RenderPassChain) → RenderPass(type=TransformEffect|Alpha|…)
 │             ├ ChoosableMixer name="Blend" → ParamChoice "Blend Mode"
 │             └ PrimarySource → VideoSource(type width height) → VideoFormatReaderSource | GeneratorVideoSource
 ├ Column × N (uniqueId columnIndex) [+ ColumnAttributes index name — only in Example.avc]
 └ Layer × N (uniqueId layerIndex) : Params Name · ClipTransition · LayerView · AudioTrack · VideoTrack
```

### The 40 most frequent elements (`Example.avc`, 2,758 elements)

`ParamRange` 418 · `PhaseSourceStatic` 361 · `Params` 275 · `Param` 274 · `RenderPass` 202 · `FoldState` 164 · `ParamChoice` 67 ·
`ChoosableMixer` 66 · `ValueRange` 65 · `PrimarySource` 65 · `View` 60 · `VideoTrack` 52 · `FoldParams` 51 · `DurationSource` 50 ·
`Beats_d` 50 · `Clip` 47 · `Transport` 47 · `PhaseSourceTransportTimeline` 47 · `ClipView` 47 · `VideoSource` 47 · `PreloadData` 45 ·
`VideoFile` 33 · `VideoFormatReaderSource` 33 · `AudioTrack` 23 · `AudioEffectChain` 23 · `ParamColor` 19 · `AudioFile` 18 ·
`AudioFileSource` 18 · `EmbeddedThumbnail` 12 · `Column` 9 · `PhaseSourceDashboardLink` 8 · `ColumnAttributes` 7 · `point` 6 ·
`DeckInfo` 4 · `Deck` 4 · `Choice` 4 · `Layer` 3 · `ClipTransition` 3 · `LayerView` 3 · `PhaseSourceTimeline` 3.
Example of `RenderPass` attributes: `{name: RenderPassChain, type: RenderPassChain, uniqueTypeId: RenderPassChain, uniqueId: 1621406654259, baseType: RenderPassChain}`.
Example of `Clip`: `{name: Clip, uniqueId: 1621342719991, layerIndex: 0, columnIndex: 0}`.

### What changes between `Example.avc` and the user's compositions

| | Example.avc | aniver 30.avc | climao.avc |
|---|---|---|---|
| decks / layers / columns | 4 / 3 / 9 | 3 / 7 / 10 | 3 / 7 / 14 |
| `<Clip>` elements | 47 | 70 | 98 |
| clips with content | 47 | 17 | 63 |
| `compositionIsRelative` | `1` | `0` | `0` |
| video sources | `VideoFormatReaderSource` 33, `GeneratorVideoSource` 13, `CompositionRouterVideoSource` 1 | `GeneratorVideoSource` 17 | `GeneratorVideoSource` 63 |
| effects (`RenderPass type`) | 22 distinct types, with `TextGenerator`, `WireGenerator`, `Metaballs` | 6 types: Transform, SolidColor, Alpha, Add, Lines | 10 types, with `StroboscopeGenerator` 8× and `Spiral` 8× |
| exclusive nodes | `AudioFile`, `AudioFileSource`, `Beats_d`, `Choice`, `ColorPalette`, `ColumnAttributes`, `Modifier`, `ModifierEnvelope`, `ParamText`, `PhaseSourceDashboardLink`, `SliceInputs`, `VideoFile`, `VideoFormatReaderSource`, `WireRenderPass`, `point`, `points` | — | — |
| nodes only the user's two files have | — | `Beats_double`, `Notes`, `Note` | same |

Four facts that matter for the `.spell` format:

1. **Only what differs from the default is serialized.** None of the three files stores `Target`, `Trigger Style`, `Beat Snap` or `Fader Start` in the clip, even though the swagger declares all four. Small file, but impossible to audit without the binary that knows the defaults.
2. **The whole grid on disk.** `aniver 30.avc` has 70 `<Clip>` = 7 layers × 10 columns of the first deck: the empty cells are serialized with `Transport`, `VideoTrack`, `RenderPass` and everything. `climao.avc`: 98 = 7 × 14. `Example.avc` writes only clips with content. Different behaviour between versions, in the same format.
3. **Node name drift between versions.** `Beats_d` (Example, 2021) became `Beats_double` (2026 files); `Color Space` became `Color Format` in the fixture. Element names are part of the contract and changed with no alias.
4. **Embedded thumbnail.** 12 `<EmbeddedThumbnail encoding="LZF" width=160 height=120 bitdepth=32 data="base64…">` in `Example.avc`, 11 of 320×240 in `aniver 30.avc`. That is where the 1.2 MB of `climao.avc` comes from, for 98 clips that are nearly all solid-color generators.

## What Spellcaster should copy

- **`connect` as a button press, not as an event.** The body of `POST /composition/layers/{l}/clips/{c}/connect` is a boolean: "analogous to whether the mouse is pressed down on the clip. If omitted, true and false are both send" (swagger:3946). That gives momentary and latch with a single primitive. It serves directly the **DMX scenes and cues** (a cue that holds while the key is pressed) and the **ILDA player** (momentary shutter).
- **Clip state as a 5-value enumeration, not a boolean** (`Clip.connected` is a `ChoiceParameter`, swagger:5633; help `MIDI Out Status`: "clip triggers can have 5 different states"). `design/PRINCIPIOS.md §2` says color means state — so the state needs more than two values for the color to have anything to say. It serves the **interactive set** (device empty / loaded / armed / live / error) and the **Aprendiz**, which can only warn "the galvo is not keeping up" if the state has that step.
- **`in`/`out` separate from `min`/`max` in the `ParamRange`** (swagger:5369). A parameter has a physical range and a useful range, and the mapping remaps without touching the parameter (`<ValueRange min="0.0630957" max="1"/>` in the keyboard shortcut `A`). It serves the **orchestrator**: an sACN→parameter mapping needs this so as not to require a scaling node in every route.
- **A single textual path as the identity of everything** (`/composition/selectedlayer/clips/3/connect`). The same path is a keyboard shortcut key, an OSC address, a MIDI target and a REST route — four protocols, one namespace (`Shortcuts\Keyboard\Default.xml`, `Shortcuts\OSC\OSC_TD_TEST.xml`, `swagger.yaml`). It is literally the `spellcaster.core.registry` of `CLAUDE.md`, and what makes `design/PRINCIPIOS.md §4` ("whoever learns the GUI already knows how to operate over SSH") work. It serves the **orchestrator** and the **Aprendiz** as a command palette.
- **Target scope declared in the mapping, with three modes** — *Selected / This / By Position* (help `Shortcut Target`; `translationType` field). It is what allows an 8-fader controller to operate 40 layers. It serves the **DMX scenes and cues** (an encoder that always takes the selected fixture) and the **orchestrator**.
- **Return path in the same mapping object** (`OutputPath`, `InputSiblingPath`, `OutputSiblingPath`, `NamedValues`). LED feedback is not a separate feature: it is a field of the mapping. It serves the **orchestrator** (MIDI console with correct LEDs) and the **interactive set** (the cutout on the scale model lights up with the real state, not with the command sent).
- **Animation source as a child of the parameter** (`PhaseSourceStatic` / `Timeline` / `TransportTimeline` / `DashboardLink`, and `ModifierEnvelope` with `point x y curve`). Swapping the child turns a fixed value into an animated value without changing type or address. It serves the **DMX timeline/cues** and the **ILDA player** (kpps or size following the timeline with the same syntax).
- **Delay and framerate per output, not global** (`OutputDeviceDmx`: `Framerate` 1-40, `Delay` 0-150 ms, in `Presets\Advanced Output\amiver 30.xml`). Real hardware arrives out of phase. It serves the **NDI→ILDA** (compensating NDI latency against DMX) and the **DMX scenes**.
- **An output panel that shows channel overlap** (help `DMX Output`: "visually re-arrange the start channels of your fixtures or check for any overlap"). It serves the **DMX scenes and cues**: it is the only way for the operator to see that two fixtures were patched on top of each other before the show.
- **Contextual help as a data file, one sentence per element** (`docs\help\English.xml`, 412 entries `components → title + text`). That is the body of the **Aprendiz** ready-made: the companion needs no model, it needs a table `element → one sentence`, loaded per Face, and a place to hang the state warnings.

## What NOT to copy

- **Personality with no channel type.** The 25 fixtures read are lists of `ParamRange 0..255` with a free-form name; there is no `pan`, `tilt`, `strobe`, `16-bit`, nor labeled range. The result is in the user's files: `teste_parametros.xml` with 34 channels called `New Parameter 1..34`, and `srg - strobo fita 11chrgb.xml` with two anonymous channels in the middle. Without a channel type there is no correct fade (a dimmer interpolates, a color wheel does not), there is no 16-bit for pan/tilt, and the Aprendiz has nothing to check. The Spellcaster editor needs a per-channel type from the first commit.
- **Personality copied into the patch, and with the name lost.** `FixtureInstance/Fixture` in `amiver 30.xml` has `fixtureName=""` and a new `uuid`, different from the library one. Fixing the personality does not fix the show, and the show does not know the name of what it loads. Reference by stable id + version, not an anonymous copy.
- **No output arming and no rehearsal mode.** No field in the swagger, `recentLayout.xml` or the help corresponds to it; for a Lumiverse to exist is already to send. `design/SHORTCUTS.md` already reserves `Ctrl+Shift+Enter` / `Ctrl+Shift+R` — keep it, it is what separates a VJ tool from a show tool.
- **No GO, no cue, no blackout on the keyboard.** The 42 shortcuts of `Shortcuts\Keyboard\Default.xml` are all clip, column, deck and BPM triggers. There is not one single sequence shortcut. The "Cue Points" of the help are six markers **inside a video clip**, not show cues. A show cue is function 4 of `design/TEMAS.md` and has no equivalent here — there is nothing to copy, only something to avoid assuming.
- **State color mixed with organization color.** Every object has `colorid` (`ChoiceParameter` in Clip, Column, Layer, LayerGroup, Deck — swagger) *and* a state with a color (`connected`, `bypassed`, `solo`). Two color semantics in the same cell. `design/PRINCIPIOS.md §2` forbids it: one accent, and it means state. A coloured organization label needs another channel (border, text, icon), never the fill.
- **33 `Show X` booleans in place of Faces.** `recentLayout.xml/Params[@name='LayoutParams']`. Recovering a screen means remembering 33 boxes and nine named presets (`Big Monitor`, `Many Monitors Left`, `Panels On Top`...). A named Face with declared content (`design/PRINCIPIOS.md §5`), not a checkbox per widget.
- **A format that stores only what differs from the default.** The three `.avc` omit `Target`, `Trigger Style`, `Beat Snap` and `Fader Start` from the clips even though the REST schema declares them. A `.spell` that stores only the diff is indistinguishable from a corrupted `.spell`, and default differences between versions become a silent change of show.
- **Thumbnail embedded in the show file.** `EmbeddedThumbnail` LZF base64 (12× in `Example.avc`, 11× of 320×240 in `aniver 30.avc`) is what makes `climao.avc` 1.2 MB for 98 solid-color clips. Cache alongside, with a hash — never inside the `.spell`.
- **Preset referenced by name.** `climao.avc` asks for `OscShortcutPreset="OutputAllMessages"`, which does not exist in `Shortcuts\OSC\`. Reference by id + declared fallback, and the Aprendiz warns on opening.
- **Node name as contract, with no alias.** `Beats_d` → `Beats_double`, `Color Space` → `Color Format`: silent renames between versions inside the same file extension. If the `.spell` is JSON, version the schema and keep a reader for the old names.
- **DMX as video sampling, and only that.** In Arena a fixture receives the rectangle of pixels that falls on top of it (`DmxSlice/InputRect` in composition coordinates). It works beautifully for pixel mapping and is useless for a gobo, macro or pan-speed channel. Spellcaster needs both paths: per-channel value (scene) *and* surface sampling (pixel map) — and the data model has to accept both in the same fixture.
