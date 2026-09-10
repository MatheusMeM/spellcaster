# MadMapper 5.6.8 — interface audit (primary source)

Survey done by reading the files installed in `C:\Program Files\MadMapper 5.6.8\` and `C:\Users\email\AppData\Roaming\GarageCube\MadMapper\`. Nothing here comes from memory: every statement cites the file, the PDF page or the project key. The application was not opened.

## Sources read

PDFs in `C:\Program Files\MadMapper 5.6.8\Resources\Guides\` (text extracted with PyMuPDF):

- `Introduction to the User Interface.pdf` (29 p.)
- `Scenes and Cues.pdf` (10 p.)
- `Cue Scheduler.pdf` (5 p.)
- `MadLaser Guide.pdf` (20 p.)
- `Laser Scanner Guide.pdf` (7 p.)
- `Laser Materials Documentation.pdf` (5 p.)
- `Materials Documentation.pdf` (15 p.)
- `OSC Channels.pdf` (6 p.)
- `Modules.pdf` (10 p.)
- `Preferences.pdf` (12 p.)
- `Control Surface Module Guide.pdf` (3 p.)
- `Stream Deck Plugin Guide.pdf` (4 p.)
- `Masking.pdf` (9 p.)
- `My First Video Mapping.pdf` (15 p.)
- `MadLaser AVB Support.pdf` (6 p.)
- `Realtime Reactive Visuals.pdf` (7 p.)
- `Knowledge Base.pdf` and `The MadMapper FAQ.pdf` (35 and 26 p., almost only licensing; only the "Controls" section of the Knowledge Base p. 4 yielded anything)

Binary and data files:

- `Resources\SampleProjects\*.mad` (6 projects). Format discovered and decoded by a script of our own (`madparse.py`, in the session scratchpad); **the 6 files were read 100%** (final byte = the file's byte), so the schema below is complete, not sampled.
- `Resources\Materials\All Widgets Template\All Widgets Template.fs`, `Resources\LaserMaterials\Beam Patterns 1\Beam Patterns 1.fs`, `Resources\FactorySurfaceLaserLineFX\Beams\Beams.fs`, and the `.info` of those same folders.
- `Resources\app.css` (Qt Widgets QSS stylesheet).
- `Resources\Translations\mm_fr.qm` (UTF-16 strings extracted: menus and actions).
- `C:\Users\email\AppData\Roaming\GarageCube\MadMapper\LEDFixtureLib.mfl` (XML).
- Listing of `Resources\FactoryModules\`, `SystemModules\`, `FactoryGenerators\`, `FactoryLaserGenerators\`, `Macro\` and of `C:\Users\email\Documents\MadMapper\`.

Not read due to opacity: the `main.ldat` of factory modules and generators (`Resources\FactoryModules\Cue Scheduler\main.ldat` etc.) are encrypted/compressed binaries — `od` shows high entropy from the first byte, with no readable header and no extractable strings. Therefore, **the internal format of a MadMapper module could not be read**; what is known about modules comes from the PDF and from the project serialization.

## Anatomy of the screen

Four fixed zones (`Introduction to the User Interface.pdf` p. 2):

1. **Media Panel** — the whole **right** side. Split into Media Bin (thumbnail grid or list) and Media Inspector below, with information and parameters of the selected media (p. 3). 2. **Views** — the center, split in two: **Input View** (left, chooses which part of the media is mapped — the UVs) and **Output Preview** (right, positions the surface and corrects the perspective) (p. 9–10). 3. **Project Tabs** — top left corner, five tabs: Surfaces, Light Fixtures, Projectors (Outputs), Modules, Master Settings (p. 13). Each tab is a list on top + Inspector below. 4. **Tool Panels** — bottom panel, chosen by dropdown: Scenes and Cues, Control List, Library, MadLight Recorder, DMX Monitor, Fixtures Editor, Code Editor (p. 25–28).

Always visible: the tooltip status bar at the bottom, which shows contextual information about the element under the mouse (p. 2). The declared flow is bidirectional, "from the left (surfaces, fixtures, modules) to the right (media) and vice versa" (p. 2).

The tool panel is **the only resizable part** by splitter, and it is the only one that can be detached into its own window, with a "keep on top of other windows" button (`Introduction to the User Interface.pdf` p. 25).

Toolbar above the views (p. 11–12): side by side / input only / output only, toggle vertical-horizontal orientation, X/Y position of the selected handle, selection groups named on the spot, undo/redo, lock affine transformations, snapping, "see only the selected", zoom in/out, fit to window, zoom on the selected, and (output only) the "Master" and "Selection" preview groups.

The UI is **classic Qt Widgets**, not QML: `Resources\app.css` is a QSS that styles `QMenuBar`, `QMenu`, `QSlider`, `QSpinBox`, `QTableView`, `QSplitter`, `QStatusBar` and its own classes (`CueCell`, `CueGridViewWidget`, `CueEditorToolBar`, `CueEditorInspector`, `CueBankComboBox`, `QMappableButtonGrid`, `MediaGridWidget`, `QAttributeWidget`, `HandlePosWidget`, `DMXUniverseDialog`, `LaserScannerDialog`, `LEDScannerDialog`). The theme is a short list of `$` variables: `$toolbar_background`, `$groupbox_background`, `$list_background`, `$list_item_background`, `$text_color`, `$disabled_text_color`, `$selection_blue`, `$active_selection_blue`, `$disabled_selection_blue`, `$cue_color`, `$preview_group_green`, `$slider_spinbox_etc_background`. That is: **two accents (selection blue, preview group green) and a separate token only for the cue color**.

## Objects and verbs

**Surfaces** (`Introduction…` p. 14–17): Quad, Line, Triangle, Circle, Mask, 3D Surface (.OBJ), Group. Verbs per list item: rename (click the name), show/hide, lock, move in the list — and the list order is layer order, the first one is in front. The Inspector brings blend mode, opacity, position, perspective, colors, FX, soft-edge/feathering, mesh warping and bezier mesh warping. Selecting several with Shift, the Inspector shows only the common parameters.

**Fixtures / light mapping** (p. 18–20): DMX Fixture, DMX Line, DMX Circle, Group. In MadLight you only work in the input view. A Fixture **has no opacity** — the one above overwrites the one below, it does not blend. Inspector: fixture definition (from the library, with an Edit button for the Fixture Editor), DMX channel and DIP switch, filtering, **response curve** (how the fixture's brightness responds to the DMX value), brightness, color, position, size, rotation, flip.

**Media** (p. 5–7), by category: Generators, Materials, ISFs, Quartz Composer, Images, Movies, Images Folders (with a folder watchdog), Live Input, Syphon/Spout/NDI. The distinction that matters: a *generator* is rendered once into a fixed-size texture; a *material* is a shader rendered separately **on each surface** that uses it, with more precision and more cost. Dragging a material to the Generators category converts it into a texture. Each media shows on how many surfaces it is applied, and clicking that number selects all of them.

**Outputs** (p. 20–22): projectors, with no layer order. Inspector: output size with aspect ratio lock and a button to capture the destination resolution, flip, Destination, background image (does not go to the output) and .png mask (does go). Tools: show/hide the preview window, turn on the test pattern, publish the live content over Syphon or NDI.

**Modules** (`Modules.pdf` p. 2–10): instantiable several times, renamable, with an enable/disable button per instance. The factory ones on disk (`Resources\FactoryModules\`): Audio Player, Calendar Scheduler, Control Surface, Controls Combiner, Cue Scheduler, DMX Router, Device Activity, Firmata, Idle Cue, MIDI Out, MadLight Player, MiniMad Controller, OSC Out, Oscillator, Oscillator2D, OscillatorBank, PJLink, Pollution, RandomNoise, Startup Cue, Weather. The system ones (`Resources\SystemModules\`): BPM Global, Media Playback.

Architectural point: in the project file, **generator, laser generator and module are the same entity**. `modules.modules[]` has `moduleName`, `instanceName`, `id`, `activated`, `isSystemModule`, `isGenerator`, `isLaserGenerator` and `attributes` (`Laser Example.mad`). The Laser Mixer, for example, has `attributes` = `{"Input1": "/medias/76", "Input2": "/medias/75", "Mixer/Mode": "Morphing", "Mixer/Mix": 0.0, "Output": ""}` — a module references media by URL and publishes its output as media.

**Laser** (`MadLaser Guide.pdf` p. 3–4): unlimited laser outputs; Ilda protocols via EtherDream or Helios USB, ShowNet, and FB4 via Pangolin Beyond. `MadLaser AVB Support.pdf` p. 2–3 adds AVB / Dante over an audio card (48 and 96 kHz exposed), and names as DAC equivalents "Etherdream, Helios, ShowNET, Moncha". Laser surfaces: Laser Quad, Laser Line, Laser Text, Group (p. 9, 15, 17, 19).

**Calibration tools**: Laser Scanner (Tools menu) photographs with a Sony/Canon/NDI camera what the laser draws and warps the image to the laser's point of view (`Laser Scanner Guide.pdf` p. 2, 5–6); Spatial Scanner and LED Scanner in the same menu (`Introduction…` p. 29).

Menus, from the strings of `Resources\Translations\mm_fr.qm`: Fichier, Edition, Projet, Sorties, Outils, Vue, Compte, Aide — with actions "Raccourcis clavier" (a shortcuts window exists in the app, but no PDF transcribes it), "Scanner Spatial", "Scanner LED", "Enregistreur MadLight", "Exporter/Importer les définitions de Fixtures", "Nouveau Projet avec Mur d'Ecrans", "Créer des Lignes depuis le Contour", "CUER TOUT" (the CUE ALL).

## Scenes and cues

The model, from `Scenes and Cues.pdf` p. 2–10, confirmed by the serialization in `SampleProjects\DMX LED Bar Example.mad` (key `cueBanksV4`).

**Grid.** A project has N Cue Banks (`cueBanksV4.cueBanks[]`), each with `bank_name`, `bank_id`, `column_count` (16 by default), `row_count` (8), and a flattened `cues[]` of 128 cells — index = `row * column_count + column`. The grid grows by itself when something is recorded in the last row or column (p. 3). `cueBanksV4.activeCueBank` holds the active bank and `cueBanksV4.liveMode` the Live mode.

**The first row is reserved for Scenes** (p. 3). Exact difference (p. 2): a Scene stores *everything* (surfaces, fixtures and medias), does not allow removing entries nor storing part of an object, and when fired it **hides every surface/fixture created after it**. A Cue stores what you choose, at whatever granularity you want.

**What a cue is, in data.** Each non-empty cell is: `{name, comment, color, pixmap/pixmapId, is_scene_cue, transition_settings, entries[]}`. And each entry of `entries[]` is `{url, value, ownerUrl, display_path, transition_mode, transition_settings}`. Real examples from `DMX LED Bar Example.mad`:

    {"url": "/fixtures/9/color/red", "value": 1.0,
     "ownerUrl": "/fixtures/9/color",
     "display_path": "/Fixtures/LED BARS/Color/Red",
     "transition_settings": {"type": 0, "duration": 0.0, "params": {}}}

    {"url": "/fixtures/1/visual", "value": "/medias/9",
     "display_path": "/Fixtures/LED BARS/LED BAR-1/Visual"}

This is the central finding: **a cue is a list of (OSC address, value) pairs with per-entry fade**. The same address that `OSC Channels.pdf` documents as the external API is the internal storage key of the cue. The value can be a number or a reference to another object (`/medias/9`), and `display_path` is only the readable label.

**Transition.** `transition_settings` exists at the cue level and, optionally, per parameter: `{type, duration, params}`. The PDF (p. 5) says the same in prose: by default every parameter uses the cue's transition, but each one can be given a local transition ("fade the opacity but not the RGB"), including "No Transition". Double-clicking the `Fade: xxx` label of the cell changes the time (p. 5).

**Firing** (p. 8–9): play button on the cell; column button, which fires that column's scene or, if there is no scene, all the cues of the column from the bottom up; "Next Column" button; Auto Play; Live mode (press the cell, no editing — made for touch screens); Cue Scheduler; Calendar Scheduler; and controls (MIDI/OSC/DMX/keyboard).

**Auto Play**, in data: `auto_play_settings = {active, mode, timing:{source, timeDuration, beatsDuration, switchOnMovieLoopEnd}}`, and each column has `column_settings[i]` with `{name, disabled, hasLocalAutoPlayTimingSettings, autoPlayTiming{...}}` — that is, **global timing with per-column override**. Each row has `row_settings[i] = {name, disabled}`.

**Scheduling** (`Cue Scheduler.pdf` p. 3–5): Schedule tab (year/month/day/hour/minute/second, each field accepting "Each" for recurrence), Automate tab (Use Auto-Play Timings / every N seconds / every N beats of the global BPM / switch at the end of the movie loop), Cue tab (target bank, specific cell or column, or a range from 1 to 16 with Loop, Random and Skip Empty), Manual tab (Go Next Now / Go Previous Now). The module checks the clock **once per second**. With two modules in conflict, the last one in the list wins (p. 5).

**Editing** (p. 6–7): in Edit Mode (`Cmd+Shift+C`) the app paints a red overlay on every "cueable" parameter; red outline = it is in the cue; orange outline = it is, but with a value different from the current one or absent from part of the multi-selection. Clicking a widget adds/removes/updates the parameter in the selected cues; to *change* the value without cueing, hold Shift. A "CUE ALL" button appears on the items of the surfaces list and on the visuals. Backspace removes the parameter from the selected cues — and that does not apply to Scenes. Clicking an already selected surface in the input preview cues/uncues the "Input Geometry"; in the output preview, the "Output Geometry".

**Cue multi-selection** (p. 5) is the bulk editing mechanism: select 10 cues, choose the entry `Surfaces / Quad 1 / Output Geometry` and press "Update from current values".

## States

- **Preview vs output**: the previews can be hidden completely ("Show/hide the UVs view and preview") to gain performance, removing two screen renders (`Introduction…` p. 13).
- **Freeze**: three distinct and independent states in Master Settings (p. 24) and in the file: `engineFrozen` (freezes the engine, Master Speed goes to 0, except if there is Ableton Link/BPM), `videoOutputsFrozen`, `dmxOutputsFrozen`, `laserOutputsFrozen`. Freezing the output does **not** stop the playback. The four masters are separate: `masterLevel`, `masterVideoLevel`, `masterDmxLevel`, `masterLaserLevel`, `masterAudioLevel` — and "if the Master is at zero, everything is at zero".
- **Per surface**: `visible`, `locked` (`Introduction…` p. 14 and keys of the same name in the `.mad`). Global lock of affine transformations with `Alt+B` (`Preferences.pdf` p. 12).
- **Laser armed**: there is an explicit command "Arm your laser! (top right of the previews)" (`Laser Scanner Guide.pdf` p. 5). It is a state at the top of the window, next to the previews, not a menu.
- **Cue in transition**: the progress bar appears on the very cell that generated the transition; if the cue has different transitions per parameter, the progress reflects the longest one; with a "damper" type transition the progress means nothing beyond "there is a transition running" (`Scenes and Cues.pdf` p. 10). If another cue was already transitioning that parameter, the previous one is discarded.
- **Preview groups**: `previewSurfaceGroups` in the `.mad` holds `[{name:"Master", includeMaster:true, surfaces:[]}, {name:"Selection", ..., surfaces:["/laser_surfaces/1"]}]` — the preview filters which surfaces appear, and that is saved state, not transient.
- **Autonomous mode**: the "never ask for confirmation" preference plus "start in fullscreen when opening with a project file" plus "minimize at startup" (`Preferences.pdf` p. 11) — the permanent installation package.

## Shortcuts and control (keyboard, MIDI, OSC)

The PDFs do **not** bring a shortcut table (the app has a "Raccourcis clavier" window in the Help menu, according to `mm_fr.qm`, but it is not on disk as text). The shortcuts actually documented:

| Action | Key | Source |
|---|---|---|
| New mask / new laser path | `Alt+A` | `Masking.pdf` p. 9; `MadLaser Guide.pdf` p. 16 |
| Close the drawn path | `Enter` | `Masking.pdf` p. 9; `Introduction…` p. 15 |
| Add a point to the path | `Alt+Click` | `Masking.pdf` p. 9 |
| Remove a point | `Delete` | `Masking.pdf` p. 9 |
| Enable bezier on a point / turn tangents on-off | right button | `Masking.pdf` p. 6, 9 |
| Rename a list item | double click | `Masking.pdf` p. 9 |
| Copy / paste / duplicate | `Ctrl+C` / `Ctrl+V` / `Ctrl+D` | `Masking.pdf` p. 9; `MadLaser Guide.pdf` p. 17 |
| Duplicate a cue to another cell | `Alt` + drag | `Scenes and Cues.pdf` p. 6 |
| Enter Edit Cues | `Cmd+Shift+C` | `Scenes and Cues.pdf` p. 7 |
| Lock affine transformations | `Alt+B` | `Preferences.pdf` p. 12 |
| Fullscreen / exit fullscreen | `Cmd+U` / `Cmd+T` | `My First Video Mapping.pdf` p. 7 |
| Save | `Ctrl+S` | `My First Video Mapping.pdf` p. 15 |
| Next handle of the quad | `Tab` | `Introduction…` p. 10 |
| Move a handle precisely | arrow keys | `My First Video Mapping.pdf` p. 11 |
| Remove media from the pool | `Delete` | `Introduction…` p. 5 |
| Create a group from the selection | `Shift` + click on the group icon | `Introduction…` p. 16 |

**Mapping grammar** (`Introduction…` p. 26): a **Learn** button puts the app in learning mode and **all mappable functions are highlighted in purple**. Control sources: keyboard, MIDI, OSC, Audio, Playstation controller, MadMapper's own modules (Oscillator etc.) and "others" (Leap Motion, which has to be enabled in the preferences). The Control List filters by category. A control accepts **only one MIDI note, but it accepts a note, a key and an OSC channel simultaneously** (`Knowledge Base.pdf` p. 4).

**OSC addressing** (`OSC Channels.pdf` p. 2–6). Every parameter has a predefined address; they are discoverable through three paths: right button on the widget → "Copy OSC address"; Controls dialog → `+` shows the whole hierarchy; or through OSC Query. The hierarchy is the exposed data model:

    /surfaces/[selected | Group/Name]/opacity | visible | blend_mode | invert_mask
    /surfaces/.../visual/{type,number,name}
    /surfaces/.../output/{x,y,scale,rot,3d_rot_x,3d_rot_y,3d_rot_z}
    /surfaces/.../output/handles/[0-3]/{x,y}
    /surfaces/.../input/{x,y,scale,rot,flip}
    /surfaces/.../color/{red,green,blue,rgba,hue,saturation,value}
    /surfaces/.../lights/0/{distance,latitude,longitude,pos_x,pos_y,pos_z,red,green,blue,color}
    /fixtures/[selected | Group/Name]/{visible,luminosity,response,sliders/[n],color/...,input/...}
    /medias/{select,previous,next}  /medias/per_type_selection/next_[mediaType]
    /medias/[selected | name]/{assign,assign_to_all_surfaces,restart,play_forward,pause,
        play_backward,begin,loop,position,position_sec,position_frame,previous_frame,next_frame,
        play_speed,absolute_speed,loop_start,loop_end,audio_level}
    /cues/active_bank
    /cues/[bank]/{auto_play}
    /cues/[bank]/columns/{start_next,start_previous,select_next,select_previous,
        start_selected,start_by_number,[column]}
    /cues/[bank]/scenes/by_name/[name] | by_cell/col_[n]
    /cues/[bank]/cues/by_name/[name] | by_cell/col_[n]/row_[m]
    /cues/[bank]/cues/{start_next,start_previous,select_next,select_previous,start_selected,
        start_by_number}
    /master/{master_level,master_video_level,master_dmx_level,master_audio_level,
        audio_input_level,video_color/...,freeze_engine,engine_speed,reset_engine_speed,
        freeze_video_output,freeze_dmx_output,output_cursor,output_cursor_size,test_pattern}
    /modules/[name]/{select,active,...}
    /outputs/[name]/{enabled,show_desktop_window,show_test_pattern,publish_to_syphon_spout,
        publish_to_ndi}
    /application/{mad_light_recorder/start_recording, preview/active_preview_group,
        medias/add, medias/remove, view/fullscreen}

A detail that solves a real problem: the `selected` token in place of the name ("the control acts on whatever is selected") and the `by_cell` / `by_name` pair for cues. In Edit Controls the app offers "Map to Cell Position" (default) or "Map To Cue"; with the second one, moving the cue in the grid **does not break the mapping** (`Scenes and Cues.pdf` p. 9).

Declared difference between a predefined address and a mapped control: the predefined one has no "input range", no "output range" and no filter (`OSC Channels.pdf` p. 2). The mapped control has them, plus source/destination ranges and filters (`Modules.pdf` p. 8).

**Network and protocol preferences** (`Preferences.pdf` p. 4–8): MIDI with Input devices and a separate Feedback one, and a global option "MIDI notes carry velocity" (which can also be enabled control by control); OSC with input port 8010 by default, feedback port, feedback IP "Auto" (answers every IP that sent something) or fixed, and discovery by Bonjour; DMX Input separate from DMX Output, with an explicit warning not to use the same device for both; Art-Net with a choice of network interface, Max FPS 44 by default (60 suggested for high-end LED controllers), optional unicast and ArtSync; sACN with interface, Max FPS, **priority** for merging with other sources, and E1.31 synchronization.

**Control surfaces**: the Control Surface module supports APC mini mk2, APC40 mk2, Launchpad MK2, Launchpad Mini Mk3 and Launchpad X; the pads automatically mirror the colors of the scenes and cues, and the bank is navigated with the Up/Down/Left/Right buttons and the module's Offset X/Y sliders — which can be mapped to any other control when there is no physical device (`Control Surface Module Guide.pdf` p. 2–3). The Stream Deck plugin addresses a cue by `(column, row, bank)` and reflects the thumbnail or the cell color on the key; on the Stream Deck+ dials, you paste an OSC address copied from the widget and set the increment per step as a percentage; pressing the dial returns the parameter to its default (`Stream Deck Plugin Guide.pdf` p. 3–4).

## Laser (real parameters)

The names below are the literal `customSettings` keys of the laser output in `SampleProjects\Laser Example.mad` (key `outputs.outputs[0].customSettings`), with the factory values of the example project, cross-checked against the explanation in `MadLaser Guide.pdf`.

Output (`outputs.outputs[0]`): `outputType = "Laser"`, `name`, `id`, `active`, `laserDeviceUrl`, `laserDisplayName = "None"`, `stageSize`, `scale`, `rotation`, `position`, `flip`, `masks`, `maskOpacity`, `publishInternalLoopback`, `loopbackDispatchProjCount`, `loopbackDispatchTolerance`, `publishPonkEnabled`, `publishPonkIp = "127.0.0.1"`.

    Device/PPS            = "Custom"     Device/Custom PPS = 48000
    Device/Buffer Size    = 3000         Device/Delay      = 0
    ILDA/Desired FPS      = 60.0         ILDA/ILDA FPS     = 60.0
    ILDA/Point Count      = 800          ILDA/Scan Area    = 0.0
    ILDA/Mode             = "Preserve Image Quality"
    ILDA/Blank Delay      = 2.0          ILDA/Blank Smth   = 0.70
    ILDA/Blank Curve      = 0.34         ILDA/Blank Color  = MadColor
    ILDA/Blank using Worst Case = True   ILDA/Enable Frame Blending = False
    Color Levels/{Red,Green,Blue} = 1.0
    Min Voltage/{Red,Green,Blue}  = 0.25
    Time Shift/{Red,Green,Blue}   = 0    (offset in ILDA points)
    Response/{Red,Green,Blue}     = curve
    Masks/Render Masks = False  Masks/Level = 0.1  Masks/Color = MadColor
    Test Pattern/Level = 0.25   Test Pattern/Color = MadColor
    Save ILDA Frame = False     Record ILDA Movie = False

Meaning, from `MadLaser Guide.pdf` p. 4–8:

- **PPS**: what the manufacturer promises; it is sent along with the frame and used to compute the number of points. The range is limited on purpose "so as not to hurt the galvo"; above 45 kpps it only makes sense projecting very far away with a small angle (p. 5).
- **Desired FPS** vs **ILDA FPS**: the first is requested, the second is the real one, and the explicit formula holds: `ILDA FPS = PPS / Point Count` (30 kpps / 500 points = 60 FPS). Below 35 FPS the flicker is visible, above 45 the sweep is not perceived (p. 5).
- **Blank Delay**: when jumping from one path to another the beam is turned off and moved; 100% is a value calibrated by the team on several devices, and the export mode allows smaller values (p. 6).
- **Min Voltage**: each diode lights up from a different voltage; you cut below a level so that a dark gray does not turn red (p. 6).
- **Time Shift**: some projectors delay the color response relative to XY, and sometimes each diode relative to the other — hence the per-channel offset **measured in ILDA points**, not in milliseconds (p. 6).
- **Safety Area** (`ILDA/Scan Area`): keeps the beam from reaching the edges of the field. The guide warns against tightening it too much: some scanners go into protection or get damaged, and "listening to the scanner is a good way to notice a problem — the sound should be smooth, not crackling" (p. 6).
- **Masks**: protected regions of the space (people, camera). At 100% opacity the beam does not enter; at 90% it enters with reduced brightness, "useful in countries where sweeping the audience is allowed below a certain level". They can be inverted (then they define the *allowed* area), and they also apply to the mouse cursor when the cursor is being drawn (p. 7).
- **Publish Internal Loopback**: a laser projector with no destination publishes its paths as new media in Live Inputs; `Dispatch Count` splits the paths of that composition across several media, one per real projector. That is how mesh warping of a whole composition is done without remapping each surface (p. 7–8).
- **Store Ilda Frame** and **Record Ilda Movie**: they record into the `ILDA` folder of the workspace. **Movie Mode** chooses between "Record at fixed frame rate" (to play back in MadMapper) and "Record as ILDA stream" (to play on a DAC or on the laser's SD card), because the ILDA format does not carry the playback PPS and the hardware treats the file as a stream of points: a 500-point frame lasts less than a 700-point one (p. 8).

Parameters per **laser surface**, real keys of `surfaces[].customSettings` (`Laser Example.mad`), group `Laser Render`:

    Max Speed = 1.0        Scan Speed = 1.0     Scan Mode = "Auto"
    Samples = 8192         Skip Black = True    Preserve Order = True
    Optimize Angles = True Angle Min = 34.38    Angle Delay = 0.05
    Start Repeat = 0       End Repeat = 6       Soft Close = 25
    In Fade = 1.0          Out Fade = 0.0
    Beam Mode = "Auto"     Beam Level = 10
    Override Loopback Render Settings = False

And the `Active Segment` group (`Start`, `End`, `Strt Smooth`, `End Smooth`), which crops which stretch of the path is drawn.

Semantics (p. 9–11 and `Laser Materials Documentation.pdf` p. 3–5): **Max Speed** distributes the scanning time among the paths in proportion to their length, so that a long line and a small circle come out with the same brightness; maximum value 4, limited because scanning fast heats the galvo. **Optimize Angles** injects points at the vertices, otherwise the physics of the scanner rounds the corners; `Angle Min` in degrees in the UI and in radians in the ISF (`ANGLE_THRESHOLD`). **End Repeat** repeats the last position because the software never knows where the beam really is. **In Fade** avoids the "hot point" at the start of the path (the scanner starts from zero inertia) and **Out Fade** avoids the one at the end. **Point Intensity** (Laser Quad only) controls how much time is spent on an isolated point — a zero-length path —, which is how a strong static beam is made; in Laser Line the equivalent is **Min Points**, the minimum of ILDA points per path. **Skip Black** skips the blanked stretches of the path; turning it off stabilizes the image when the shape is fixed and only the light varies. **Preserve Order** forces the generation order, against the flicker that optimal reordering causes when the paths move.

Video → laser vectorization (`MadLaser Guide.pdf` p. 11–14; real keys in `surfaces[].customSettings` group `Process` of `Laser Example.mad`). Two algorithms:

- **Find Paths** — pixels above `Process/Threshold` become a path; `Use Color` uses the pixel color; `Thickness` states the stroke thickness in the source material for the skeletonization; `Denoizing`; `Max Res` reduces the input media preserving the aspect. In the file there appear the sub-groups the PDF does not detail: `Process/Skeleton/{Mode, Thread Count, Smooth, Max Err Size, Fix Errors}`, `Process/CPU Thinning/{Algo = "zhang_suen_then_guo_hall", Max Iter., Threshold, Blur Size, Thread Cnt, Thrd Margin}`, `Process/GPU Thinning/{Iterations, Threshold, Blur, Blur Size}` and `Process/Recomposing/{Enabled, Angle Tolerance, Angle Weight, Angle Offset, Color Tolerance, Color Weight, Dist Tolerance, Dist Weight, Handle Points, Handle Intersections, Intersection Size}`.
- **Find Contours** — Canny on the GPU: `Process/GPU Canny/{Threshold, Canny Size, Blur Size}`, with a CPU twin (`Process/CPU Canny/{Threshold, Threshold Ratio, Size, Gradient}`).

Output filters: `Path Filtering/{Min Length, Max Length}` as a percentage of the largest dimension of the media, and `Path Limits/{Mode = "Keep Longest", Count = 100}` — "if more than 50 paths show up, keep only the 50 longest ones". `Monitor/Info` is the error field of the vectorization; the engine gives up if it detects more than 2000 paths (p. 14). `Display/Mode = "Output Polylines"` chooses what the preview shows: Source Image, Processed Image, Processed Image + Polylines, or Output Polylines (p. 13).

File formats: import and export of ILDA, import of SVG, stick TTF fonts in the `Stick Fonts` folder of the workspace, with the OneLineFonts bundled — pure skeleton fonts, which scan much faster than the outline of the letters (`MadLaser Guide.pdf` p. 3, 15, 18). There is also `IldaMoviesIndexCache` in `AppData\Roaming\GarageCube\MadMapper\` — the app indexes the .ild files.

**Laser Material** (`Laser Materials Documentation.pdf` p. 1–5): GLSL 150 core shader with the same ISF header as the video materials, but instead of returning a color per pixel it implements

    void laserMaterialFunc(int pointNumber, int pointCount,
                           out vec2 pos, out vec4 color, out int shapeNumber, out vec4 userData)

`pos` in −1..1, `color` with alpha ignored (there is no compositing in a 2D path), and **`shapeNumber`: every time it changes with respect to the previous sample, a new path begins**. Default of 8192 samples, adjustable in `RENDER_SETTINGS.POINT_COUNT` (2 for a straight line, 1000 for a circle). The material can pin the render parameters of the output — `MAX_SPEED`, `SKIP_BLACK`, `PRESERVE_ORDER`, `ANGLE_OPTIMIZATION`, `ANGLE_THRESHOLD`, `ANGLE_MAX_DELAY`, `FIRST_POINT_REPEAT`, `LAST_POINT_REPEAT`, `POLY_FADE_IN`, `MIN_ILDA_POINTS_PER_POLYLINE` — and the documentation itself warns against abusing that, because it takes the surface-level adjustment away from the user (p. 3). The previous frame arrives as `sampler2D mm_LastFrameData`, with a fixed layout: row 0 = `rg` position and `b` shape number, row 1 = color, row 2 = userData (p. 5).

## Material/module parameters (schema)

This is the schema of "a parameter that becomes a widget", and it is the same for video material, laser material, Surface FX and Laser Line FX. Canonical file: `Resources\Materials\All Widgets Template\All Widgets Template.fs`. The material folder contains `<Name>.fs`, optional `<Name>.vs`, `thumbnail.jpg|png`, imported textures and an XML `.info` with `<author>` and `<date>`.

JSON header in a `/*{ … }*/` comment at the top of the `.fs`, with `CREDIT`, `DESCRIPTION`, `TAGS`, `VSN`, `INPUTS`, `GENERATORS`, `IMPORTED`, `RASTERISATION_SETTINGS`, `RENDER_SETTINGS`.

An INPUT is `{"LABEL", "NAME", "TYPE", "MIN", "MAX", "DEFAULT", "VALUES", "FLAGS"}`. Types seen on disk and in the documentation: `float`, `floatRange` (min-max pair in a vec2), `int`, `long` (enum, with `VALUES` and `DEFAULT` by the option text), `bool`, `event` (true for one frame), `color`, `point2D`, `curve` (with `INTERPOLATION: "catmull_rom"` and `DISPLAY: "linear"`), `audio` (waveform) and `audioFFT` (spectrum, with `SIZE`, `ATTACK`, `DECAY`, `RELEASE`).

Two conventions worth stealing:

- **The LABEL defines the UI tree.** `"LABEL": "Noise/Amount"` creates the "Noise" group box with the "Amount" widget inside; the order of the groups is the order of appearance in the header, and it accepts more than one level (`"Scale/Animation/Active"`). There is no separate layout declaration (`Materials Documentation.pdf` p. 10).
- **FLAGS changes the widget without changing the type**: `button` turns a `bool` from a checkbox into a push button; `button,trigger` makes a momentary button; `button_grid` on a `long` becomes a button grid; `spinbox` turns an `int`/`float` slider into a spin box; `no_alpha` removes the alpha from the color picker; `generate_as_define` compiles the value as a `#define` so the shader can use `#ifdef` (`Materials Documentation.pdf` p. 11–12).

**GENERATORS** is the piece that solves, without a timeline, the problem of animating with a variable parameter. They are named filters whose output is a uniform float, and whose `PARAMS` accept a literal value, the name of an INPUT or the name of another GENERATOR (`Materials Documentation.pdf` p. 5–9). Types: `time_base` (integrates speed over time, with `speed`, `reverse`, `speed_curve`, `strob`, `bpm_sync`, `link_speed_to_global_bpm` — changing the speed does not make the animation jump, which is exactly the defect of writing `sin(speed*TIME)`), `animator` (Smooth/In/Out/Linear/Cut/Noise shapes), `damper` (`hardness`, `damping`), `adsr` (`attack`, `decay`, `release`), `linear_filter` (`duration`), `ease` (`type` EaseIn/EaseOut/EaseInOut, `curve` 1..10), `multiplier` (up to 4 inputs), `incrementer` (two event INPUTs, +1 and −1), and `pass_thru` — which takes **any channel of the app by URL** as a uniform, e.g.: `{"TYPE": "pass_thru", "PARAMS": {"input_value": "/custom/BPM/bpmPos"}}`. That is: the same address space of the cues and of OSC feeds the shader.

`IMPORTED` declares textures with `TYPE` `2D`/`CUBE`/`3D`, `PATH` relative to the material folder, `GL_TEXTURE_MIN_FILTER`, `GL_TEXTURE_MAG_FILTER`, `GL_TEXTURE_WRAP` and `DEPTH` (`Materials Documentation.pdf` p. 4). `RASTERISATION_SETTINGS` turns on render-to-texture with `DEFAULT_WIDTH/HEIGHT/PIXEL_FORMAT` and `REQUIRES_LAST_FRAME` for feedback via `mm_LastFrame` (p. 9–10). Declared restrictions: no multi-pass ISF, INPUT names must start with `mat_` (or `fx_` in the FX) so as not to collide when a material and a Surface FX run on the same surface, and there are reserved names the app refuses with an error message (p. 2–3).

In the saved project, these parameters become a flat dictionary with the key being the full LABEL: `"FX/Beams/Group Offset": 1.0`, `"Laser Render/Angle Min": 34.37`, `"Global BPM/BPM Source": "Manual"`. There is no schema in the project file — the schema lives in the `.fs`, the project keeps only name-value pairs, plus `collapsedParameterGroups` (which boxes are folded in the UI).

## File

`.mad` is **Qt QDataStream**, not JSON and not zip. Layout: magic `0B AD BA BE`, `quint32` stream version (12), and then a serialized `QVariantMap` — `quint32` count, and per entry a UTF-16BE `QString` with a size prefix in bytes plus a `QVariant` (`quint32` type id, `quint8` isNull, payload). Types used: bool, int, double, QString, QStringList, QByteArray, QVariantMap, QVariantList, QSize, QPointF, QImage/QPixmap (`qint32` 0/1 marker followed by the raw PNG, with no size), a matrix of 9 doubles for `perspectiveUv`, and four user types identified by name: `MadColor` (mode `"rgb"` + 4 doubles), `FilePath` (a QString), `QVector<float>` (count + doubles) and `IndexFloatMap`.

Root with 44 keys, identical in the 6 examples. Grouped:

- **Content**: `visuals.visuals[]` (the media bin), `surfaces[]`, `surfacesRelations[]` (the group tree, as a list of parent indices), `meshes3D[]`, `outputs{outputs[], masks[], backgrounds[]}`, `stageMasks[]`, `stageBackgrounds[]`, `modules.modules[]`, `mappings[]` (the controls), `linkedOutputs[]`, `linkedS3Ds[]`.
- **Show**: `cueBanksV4{cueBanks[], activeCueBank, liveMode}`, `selectionGroups`, `previewSurfaceGroups[]`, `colorPalette{vsn, colors[]}`, `customColorTable[]`, `notes`.
- **Live state**: `masterLevel`, `masterVideoLevel`, `masterDmxLevel`, `masterLaserLevel`, `masterAudioLevel`, `audioInputLevel`, `engineSpeed`, `engineFrozen`, `videoOutputsFrozen`, `dmxOutputsFrozen`, `laserOutputsFrozen`, `videoMasterColor`, `dmxMasterColor`, `laserMasterColor`, `masterCustomParameters` (which is only the `Global BPM/...` group: `BPM`, `BPM Source`, `Range`, `Beat`, `TAP`, `Resync`, `Ableton Link`, `Peers`, `Midi Input`, `Enable Setting BPM`).
- **Persisted UI**: `previewsData` (23 keys: window geometry, QSplitter state in hex, zoom and scroll of each view, `outputPreviewTransform` as a 3×3 matrix), `resourceEditorData`, `openedFilesInEditor`, `openedFoldersInEditor`, `materialsInEdition`, `editedVisualId`, `mediaRatioForced` and numerator/denominator, `bankIterCount`, `bankGenCount`, `projectPath`.

A surface (`surfaces[]`) carries: `type` (`quad`, `fixture`, `laser_quad`, `laser_lines`, `group`), `surfaceId`, `name`, `visible`, `locked`, `visualId`, `opacity`, `blendMode`, `modulation` (MadColor), `positions` and `uvs` as lists of points, `position`/`positionUv`, `scale`/`scaleUv`, `rotation`/`rotationUv`, `uvFlip`, `isPerspective`, `warpingEnabled`, `aspectRatioMode`, `userAspectRatio`, `geometryPrecision`, `feathering`, the `softEdge{Left,Right,Top,Bottom}{Width,Curve}` family plus `softEdgeGamma` and `softEdgeActive`, `masks[]`, `customSettings` and `collapsedParameterGroups`.

A fixture is a surface of `type = "fixture"` with `startChannel`, `artnetUniverse`, `dmxtype = "ArtNet"`, `responsePower`, `sliders` (IndexFloatMap), `filtering`, `filteringMode`, `filterKernelSize`, `filterAnamorphicKernelSize`, and an embedded `fixture` sub-object: `{group, product, type: "RGB", width, height, pixelMapping, avoidCrossUniversePixels, ignoreAlpha, favorite, isValid}`. The `pixelMapping` is a string of channel offsets separated by spaces (`"1 4 7 10 13 …"`). The user library, `AppData\Roaming\GarageCube\MadMapper\LEDFixtureLib.mfl`, is XML with exactly those fields: `<LEDFixture group product favorite><PixelMapping avoidCrossUniversePixels width height type>offsets</PixelMapping></LEDFixture>`. There are `.mflb` and `.bbkp` next to it — backups of the same library.

User workspace (`Preferences.pdf` p. 3 and `C:\Users\email\Documents\MadMapper\`): folders `Generators`, `LaserGenerators`, `LaserMaterials`, `Materials`, `Modules`, `Stick Fonts`, `Surface2DFX`, `Surface3DFX`, `SurfaceLaserLineFX`, `SurfaceLineFX` — and, when used, `ILDA` and the MadLight sequences. The workspace path is a preference, pointable at Dropbox or at a network share, and exists precisely to switch resource sets per project.

## What Spellcaster should copy

- **Cue = list of (address, value) pairs with per-entry fade.** It is literally `{url, value, transition_settings:{type,duration}}` in `cueBanksV4.cueBanks[].cues[].entries[]`. That fits straight into `PRINCIPIOS.md §1` (the graph is the interface): if every widget is a node with an address, the cue is a diff over the graph and needs no structure of its own. **Benefits: DMX scenes and cues** — and for free it gives a cue of anything, including the laser kpps and module state.
- **A single address, serving as OSC, as a cue key and as a shader input.** The `pass_thru` of the GENERATORS reads `/custom/BPM/bpmPos` as a uniform (`Materials Documentation.pdf` p. 7), the cue stores `/fixtures/9/color/red`, and external OSC uses the same path. **Benefits: orchestrator** — it is the argument against having one namespace for the registry and another for the network.
- **`by_cell` vs `by_name` in the cue mapping** (`Scenes and Cues.pdf` p. 9). Mapping by position in the grid or by the identity of the cue is a choice the operator makes, and the second survives reorganizing the show at 11 p.m. **Benefits: DMX scenes and cues.**
- **Edit Mode with an overlay: red = it is in the cue, orange = it is there with a different value.** Touching the widget cues it; holding Shift touches it without cueing (`Scenes and Cues.pdf` p. 7–8). One mode, two outline states, zero modals — and it solves "what exactly is recorded in this cue" without opening an inspector. **Benefits: DMX scenes and cues, interactive set.**
- **Separate `Desired FPS` from `ILDA FPS` and show both, with `Point Count` next to them.** The relation `ILDA FPS = PPS / Point Count` (`MadLaser Guide.pdf` p. 5) is the only thing the operator needs to see to understand why the frame flickers. **Benefits: ILDA player, NDI→ILDA** — and it is exactly the warning the Aprendiz should give ("the galvo is not keeping up", `TEMAS.md` line 15).
- **Time Shift per color channel measured in ILDA points, not in ms** (`MadLaser Guide.pdf` p. 6). It is the right unit: the hardware delay is in samples, not in wall-clock time. **Benefits: ILDA player.**
- **In Fade / End Repeat / Point Intensity as first-class surface parameters.** They are the physical calibration of the galvo (inertia at the start, position uncertainty at the end, a static point as a beam). **Benefits: ILDA player, NDI→ILDA.**
- **A safety mask with opacity, invertible, and valid for the cursor too** (`MadLaser Guide.pdf` p. 7). A binary mask does not serve the real rule (sweeping the audience below a certain level). **Benefits: ILDA player.**
- **Internal loopback as media.** A laser output with no destination publishes its paths as a Live Input, and `Dispatch Count` splits them across N media (`MadLaser Guide.pdf` p. 7–8). It is chained composition without inventing a separate node graph. **Benefits: NDI→ILDA, orchestrator.**
- **A LABEL with `/` generates the widget tree, and FLAGS swaps the widget without swapping the type** (`Materials Documentation.pdf` p. 10–12). A single place declares name, type, range, default, grouping and widget shape. **Benefits: every Face, and above all the Aprendiz**, which needs a readable schema to explain each parameter.
- **GENERATORS as named filters between control and value** (`time_base`, `damper`, `adsr`, `ease`, `incrementer`). `time_base` exists because `sin(speed*TIME)` jumps when you move the speed — a real bug that Spellcaster will have on the first speed fader it makes. **Benefits: orchestrator, interactive set.**
- **The Cue Scheduler checks the clock at 1 Hz and, in a conflict, the last module in the list wins** (`Cue Scheduler.pdf` p. 5). A simple, declared tie-breaking rule, with no configurable priority. **Benefits: orchestrator.**

## What NOT to copy

- **A Scene as a "cue that hides what came after"** (`Scenes and Cues.pdf` p. 2). It is an exception nailed to the first row of the grid, with rules of its own (you cannot remove an entry, you cannot record part of an object, Backspace does not work). Two semantics in the same grid is a rehearsal trap. If Spellcaster needs a "complete state", let it be a cue with an `exclusive` flag, not a separate class.
- **A 16×8 grid flattened into `cues[128]` with an implicit index.** Moving a cue changes the index, which is precisely why `by_name` exists. Store the cue with an explicit `(col, row)`, or better, with its own id and position as an attribute.
- **A PNG thumbnail embedded byte by byte inside the show file.** In `DMX LED Bar Example.mad` a single cue carries 17,658 bytes of PNG. The `.spell` is JSON and goes to git; the thumbnail goes outside, referenced.
- **UI state inside the project file.** `previewsData` stores window position, saved width, splitter orientation and `QSplitter` states in hex; `projectPath` of the factory example still points to `/Users/matt/Projects/MadMapper/Dev/forge/...`, the developer's Mac. That shuts the door on diff, on merge and on running the same show on the Pi with no screen. Layout in the profile's `config.json`, not in the show.
- **A proprietary binary format with no semantic version.** The only thing versioned is `streamver = 12` (the QDataStream version), and keys like `cueBanksV4` carry the version in the *name*. Migrating that requires the app. `.spell` stays JSON with `"version"` at the top.
- **A module as an encrypted blob.** `Resources\FactoryModules\*\main.ldat` is not readable even by `strings` — neither the factory modules nor the generators (`Laser Text`, `Laser Grid`). Whoever buys the software cannot read nor version what runs in their own show. The Spellcaster module has to be a text file.
- **A fixture with no opacity, overwriting the one below** (`Introduction…` p. 19). A rule inherited from pixel mapping that does not hold for stage light, where HTP and LTP are what is expected. Define the merge policy explicitly and do not leave "the last one wins" as an implementation accident.
- **Two accent colors plus a separate token only for the cue** (`app.css`: `$selection_blue`, `$active_selection_blue`, `$preview_group_green`, `$cue_color`). `PRINCIPIOS.md §2` forbids more than one accent per Theme; here blue means "selected", green means "preview group" and the cue color is chosen by the user per cell — three meanings of color competing on the same screen.
- **An enum as a free string in the saved file.** `"Process/CPU Thinning/Algo": "zhang_suen_then_guo_hall"`, `"ILDA/Mode": "Preserve Image Quality"`, `"Device/PPS": "Custom"`. Renaming the option in the UI silently breaks the saved file. Store the stable identifier and label it in the presentation.
- **The shortcut window as the only documentation of the shortcuts.** The Help menu has "Raccourcis clavier" (`mm_fr.qm`), but none of the 31 PDFs brings the table; the shortcuts appear scattered, in parentheses, in the middle of masking and laser tutorials. `SHORTCUTS.md` already does the opposite, and that is the right way.
