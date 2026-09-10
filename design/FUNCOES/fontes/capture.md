# Capture 2024 — interface audit (source for Spellcaster)

Functional audit, not aesthetic. Everything here comes from two sources cited on every claim:
the installed language file and the online manual of version 2024. Nothing from memory about the product.

## Sources read

- `C:\Program Files\Capture 2024\Languages\English.c2l` — 1885 lines, 137 sections, read
  in full. Format `Section <Name>` + `Phrase <Key> <Text>`; cited as `Section/Key`.
  Next to it, `Portuguese.c2l` (same keys, `ISO pt / Version 328`), the source of the vocabulary.
- `C:\Program Files\Capture 2024\Installation\Presentation.zip` — only listed (18 entries), not extracted.
- Online manual 2024, base `https://www.capture.se/Manual/en-UK/2024/`, pages read via
  WebFetch: `introduction.html` (full index), `DesignViews.html` (navigation, selection,
  manipulation, control pane, focus mode, measure mode), `UniversesTab.html`,
  `FixturesTab.html`, `DesignTab.html`, `MediaTab.html`, `SnapshotsTab.html`,
  `ToolsMenu.html`, `NavigateMenu.html`, `WindowMenu.html`, `FileMenu.html`,
  `Appendix.html` (protocols and DMX tables).
- Not installed, not opened: the application. No claim here comes from use.
- There is a pt-BR manual, but only for version 2022 (`/Manual/pt-BR/2022/Introduction.html`) — it was not read.

## Anatomy of the screen

**Menu bar** (`Menus`): File, Edit, View, Navigate, Tools, Window, Arrangements, Help.
Eight menus, fixed order. `Navigate` does not open a window: it takes focus to a pane or tab
(manual NavigateMenu.html — "tool panes and organizational windows").

**Top-level tabs**, each with its own string section (`*Tab`): **Design** (the
project: objects, layers, views, scenes, reports), **Fixtures** (spreadsheet of every
fixture), **Universes** (DMX universes, connectivity, console link), **Media**
(video captures, players, streaming), **Snapshots** (recordings of DMX/video/laser/camera)
and **Library** (installable library of fixtures, truss, materials, gobos).

**Tool panes** of the Design tab, each with its own section and always with the same trio
(list + Add/Delete/Rename + properties): Project, Selected Items, Views, Layers,
Filters, Fixture Groups, Camera Positions, Scenes, Materials, Gobos, Frame Lists, Symbols,
Reports, Plot Styles, Plots, Universes (`*ToolPane`). Sixteen categories in one column —
no search string in any of those sections.

**Simulation views**: exactly three, with fixed names — Alpha, Beta, Gamma
(`SimulatorView/AlphaName`, `BetaName`, `GammaName`). Each view has a `Type`
(`TypeWireframe`, `TypePlot`, `TypeLive`, `TypeCustom`) and, in Custom, orthographic
or perspective projection, wireframe or solid faces, and a "model space" of Screen/Plot/Live. Window
layout: only `Actions/Quad` and `Actions/Wide`. The rest goes to floating windows
(`ReportWindow`, `PlotWindow`, `FrameListWindow`, `ConsolePatchWindow`, `OptionsWindow`).

**Widgets inside the 3D view** (`SimulatorView`): `Widgets`, `HiddenObjects`,
`DimBackground`, `ProjectInformation`, `FixtureInformation`, `SelectionNavigator`,
`ViewNavigator`. The Selection Navigator is a command panel that appears next to the selection
— and `OptionsWindow/NavigatorOnExternalSelection` shows that it also appears when the
selection comes from outside, from the console.

## Objects and verbs

Organized by object. Verbs are Phrases from `SimulatorView` (the scene context menu),
`Actions` (the File/Tools menu) and from the tool panes.

**Fixture** — `Fixture`, `LightingFixture`, `MotionFixture`, `EffectFixture`.
Verbs: `PatchAction`, `UnpatchAction`, `ChannelAction`, `CircuitAction`, `UnitAction`,
`FocusAction`, `RemoveFiltersAction`, `RemoveGobosAction`, `ReplaceAction`,
`DuplicateAction`, `Common/Assign` (dragging a gobo/gel onto the selection) and `Sequential` —
batch numbering of Unit, Circuit, Patch and Channel (EditMenu.html#sequential).

**Universe** — `DMXUniverse`, `UniversesToolPane`, `AddUniversesDialog`. Verbs:
`Common/Add`, `AddMultiple`, `ResetExternalUniverse`, `ResetLevels`,
`ConnectivityOptions..`, `ConfigureMANet2..`, `ConnectivityStatus..`.

**Layer** — `Layer`, `LayersToolPane`. Verbs: `SelectUsedBySelectedObjects`,
`SelectUnused`, `SelectUsingObjects`, `DeselectUsingObjects`, `AddToAllFilters`,
`RemoveFromAllFilters`. The layer is not only visibility: it has `Locked`, `Unselectable`,
`IncludeInReports`, `FixtureInformation` and `FixtureSimulation` — you can switch off the
**simulation** of a whole layer without deleting it.

**Filter** — `FiltersToolPane`: a container of **layers and universes** (`LayersAndUniverses`),
with `Include`, `IncludeByDefault`, `ApplyToAllViews`, `ClearAllViewFilters`; triggerable by
DMX up to 255 (DesignTab.html).

**View / Camera** — `SwingToTop`, `SwingToFront`, `SwingToSection`, `SwingToSelection`,
`FocusSelection`, `FocusAll`, `StoreCamera`, `Position {0}`, `Fullscreen..`, `SaveImage..`,
`RenderImage..`. Positions live in catalogs (`CameraPositionCatalog`), triggerable by DMX.

**Scene** — `Goto`, `RecallSelectedObjects`, `StoreSelectedObjects`. A scene in Capture stores
**object position and visibility** (`Object/IncludeInScenes`), not channel values
(DesignTab.html): it is a set change between parts of the show.

**Snapshot** — contents `CameraContents`, `DMXContents`, `VideoContents`, `LaserContents`;
verbs `RecordStill`, `RecordMovie..`, `Play`, `Stop`, `RecallDMX`, `RenderMovie..`.

**Fixture group** — numbered, created from the selection, with Update and Duplicate; they do not
keep selection order (DesignTab.html#FixtureGroups).

**Documentation** — `Plot`, `PlotStyle`, `PlotSymbol`, `Report`, `ReportItem`,
`ExportFocusSheets`, `ExportDocumentation`. Ready-made reports (`ReportsToolPane`):
Equipment, Rigging point, Cable, Fixture, Fixture groups, Fixture locations, Frame lists.

**Selection** — `SelectAllActionGroup`, ten criteria: By Layer, By Location, By Model,
By Drawing Block Name, Motion Controlled, Connected Truss, Fixtures on Truss,
By Fixture Type, By Fixture Group, By Cable Type.

## Patch and universes

**The four numbers of a fixture are different things**, and the PT translation confirms it:

| Field | `.c2l` | Official PT | What it is |
|---|---|---|---|
| Patch | `Fixture/Patch`, `PatchUniverse`, `PatchChannel` | Patch / Universo do patch / Canal do patch | DMX address: universe + start channel |
| Channel | `Fixture/Channel` | **Canal (ID)** | the fixture's number on the console (identity), not an address |
| Unit | `Object/Unit` | **Etiqueta** | physical number of the unit on the bar/stage |
| Circuit | `Fixture/Circuit` | Circuito | electrical circuit |

Still in `Fixture`: `Purpose`, `Groups`, `Mode`, `ConsoleIdentifier`, `ExternalIdentifier`,
and `ControllerMode {0} mode` / `ControllerPatch {0} patch` — the same fixture carries patch
and mode **per controller**. `Unpatched` is a named state, not an empty one.

Geometry comes from `Object` (`Layer`, `Location`, `PositionX/Y/Z`, `RotationX/Y/Z`,
`Identifier`, `Note`, `Hidden`, `CastsShadows`, `IncludeInScenes`, `MotionFixture`) and the
mechanical behavior from `LightingFixture`: `InvertPan`/`InvertTilt`/`InvertZoom`/
`InvertIris`/`InvertColorMix`, `LimitPanStart`/`LimitPanEnd`/`LimitTiltStart`/
`LimitTiltEnd`, `IntensityScale`, `SimulateFocus`, `Optics`, `Photometry`, `ThrowsLight`,
`InternalAccessory`, `ExternalAccessory`, `Filters`, `Gobos`. Inverting pan and limiting travel
are properties of the **instance**, not of the profile — that is how you correct a fixture
hung upside down.

**Patch dialog** (`Patch`): `StartAddress`, `FixtureOverlap` "Fixtures per channel:",
`ChannelOffset` "Channel offset of fixtures:", `ChannelsRequired` (calculated and shown
before confirming) and `ContinueUntilComplete`. Overflow is a question, not an error:
`OverflowWithContinue` — "All channels in universe '{0}' have been used. Do you wish to
continue patching in universe '{1}'? Stopping will leave one or more fixtures unpatched."

**Universe** (`DMXUniverse`): `Universe` (name), `PatchBase` (the universe's numeric position,
independent of the name), `PatchStyle` `Indexed`/`Contiguous` (contiguous = a single 1–2048 range
in the theater, UniversesTab.html), `ExternalUniverse` with `(auto)`, `(no)` and `Searching..`, and
`BlindLevelsMode` (what to do with blind DMX from sACN and CITP — some consoles send the
programmer or the cue preview through it). Batch creation:
`AddUniversesDialog/NumberOfUniverses` + `FirstUniverseName`. The universe view
(`DMXUniverseView`) has two selectors: `Mode:` Channels or Fixtures and `Levels:` `%` or `DMX`.

**Data input** (Appendix.html#Protocols): Art-Net, CITP, Compulite VC, EntTec DMX USB
Pro Mk1/Mk2, ETC Eos over OSC ("bidirectional channel selection and pan/tilt feedback is
supported"), MA-Net2 (requires MA v2.9+), MA-Net3 (plugin on the grandMA3), Streaming ACN. Others:
Blacktrax RTTrP, CITP/CAEX (laser), CITP/MSEX (video), Kinesys K2, LaserAnimation, NDI,
OSC, Pangolin Beyond, PosiStageNet. OSC: "1.0 and 1.1 over TCP and UDP, default port 4004",
with `/ping`, `/getCatalogs`, camera positions and control of ambient lighting, exposure,
bloom and white balance. Network in `Communication`: `Interface`, `IPAddress`,
`ServerPortNumber`, `MulticastIPAddress`/`MulticastPortNumber`, `EnableDiscovery`,
`FirstUniverse`/`LastUniverse`, `CITPMulticastAddress` Legacy/Standard, `CITPVideoFormat`,
`PangolinNumberOfProjectors`.

**Console link** (`UniverseTab`): `ProjectConsoleLink` with `(Automatic)` and `(Disabled)`;
only one console at a time. `ProjectConsoleLinkViewPatch` "View Fixture Patch.." shows what
is patched on the console and **missing** in the project; `ConsolePatchWindow` lists
Fixture / Library fixture / Position with `Identify..` and `ImportAtPosition`.
Three capabilities beyond DMX (UniversesTab.html):
- **DMX talkback** (formerly "autofocus"): clicking in the 3D sends pan/tilt back to the console — CITP/SDMX, EOS over OSC, Hog 4.
- **Fixture selection**: selection synchronized in both directions — CITP/FSEL, CITP/CAEX, EOS/OSC, Hog 4 (receive only).
- **Fixture patch**: bidirectional patch exchange — CITP/FPTC, CITP/CAEX.

**Media player as a patched fixture** (`MediaPlayer`, MediaTab.html): `OutputResolution`,
`ILDAFrameRate`, and mode `Mode_Full` "Full (256 Playlist Entries)" / `Mode_Legacy`
"Legacy (8 Playlist Entries)". Per the Appendix, the player takes 2 DMX channels (control
play/pause/stop/replay + media selection); cameras take 12 channels in Standard mode
(catalog, position, time, damping, curvature, ambient light, exposure); smoke
boxes, 4; DMX movers and rotators, 8 or 16 bits.

## 3D scene and interaction

**Navigation** (DesignViews.html#navigation): middle button **or** Alt+left click in
any part of the view; Shift swaps between orbit and pan; Ctrl rotates without moving the
camera. Zoom on the buttons below the cube: Shift+zoom moves the focal point along, Ctrl+zoom changes
the field of view instead of the position. Modes in `Navigator`: `OrbitNavigation`,
`FreeFlightNavigation`, `LookAroundNavigation`, plus `NoSnapping`/`Snapping`, `Orthogonal`,
`Quality`, and a six-faced orientation cube. Preferences in `OptionsWindow`:
`ZoomToCursor`, `InvertZoom`, `SlidingEdges`, `3DMouseNavigation` Camera/Object,
`RotationSnapAngle`, `LiveUpdateTransformations`.

**Selection** (DesignViews.html#selection): a click selects; Shift+click adds;
Ctrl+click toggles item by item. A rectangle dragged left to right takes only what is
entirely inside; right to left also takes what it touches. Clicking a grouped object
selects the whole group; double-click goes down one level — including in implicit
groups, such as a fixture with accessories.

**Manipulation**: dragging inside the red outline moves; Shift locks to orthogonal; the
objects snap to the outline of others and Ctrl switches snapping off; the corners scale
(redistribute). A red triangle is the rotation handle: the inner region rotates the set as a
block, the outer region rotates each object on its own axis; Shift locks to 5°.

**Control pane** (DesignViews.html#ControlPane) — exists only in Live mode. Each type of
selected fixture becomes a column. Quick buttons: Light (turns the selection on/off), Home
(back to the default, with optional pan/tilt), sliders for ambient light and for background
transparency. A slider with Shift gives fine adjustment; zoom, DMX mover and rotator accept a typed
number; pan/tilt fanning scales with Ctrl and becomes an offset with Alt. Parameters (`Control`):
Dimmer, Shutter, Intensity, Pan/Tilt, Framing Shutters, Iris, Gobo Rotation, Filament
Angle, Zoom, Focus, Frost, Colour, CTO, CTB, Keystone, Shift, Scale, Media Segment, Motion,
Rotation, Pump, Valve, Home, Blinder.

**Focus mode** — this is the real "interactive set": in focus mode, clicking an
object does **not** select it, it points the selected fixtures at that spot
(DesignViews.html#FocusMode). You leave through the selection navigator button. The focus plane
(`FocusPlane`, `AllFocusPlanes`) cycles between invisible, grid (to align), solid
(to remove distraction) and heatmap (`HeatmapMin`/`HeatmapMax`, to judge the lighting level).

**Measure mode**: a click starts, a click ends; Shift+click adds points; the cursor
shows X, Y and Z. Escape clears the measurement; Escape with no measurement leaves the mode.

**Atmosphere and realism**: `Smoke` (`Density`, `Variation`, `EdgeSoftness`, `AutoSize`),
`AllSmoke/Speed` (the speed of all the smoke at once), `HDRI`, `ReflectionPlane`;
on the camera, `WhiteBalance`, `HueClamp`, `AmbientLighting`, `FillLighting`, `BloomEffect`,
`AutomaticExposure` and `LaserFlickerEffect` — laser flicker is a camera parameter,
not decoration.

**Scene cost** is visible: `VisualisationSettingsDialog` chooses between `Framerate` and
`Detail`, with `AdaptiveQuality` "(recommended)", `ResolutionLimit` and
`ShowPerformanceInformation`.

## States and messages

**Named states**, each with its own string (they are not colors without a legend):
`Fixture/Unpatched`; `DMXUniverse/ExternalUniverse` with `(auto)` / `(no)` / `Searching..`;
`Layer/Current` (the layer where new objects are born); `Property/MixedValues` for a divergent
multiple selection; `Property/WaitingFor` "(Waiting for '{0}'..)"; `MediaTab/Requesting` and
`Receiving` — the media says what it asked for and what it is receiving; and `MainWindow/Locked`,
`Expired`, `Demo`, `VideoCardIssues` in the window title.

**Connectivity** (`ConnectivityStatusDialog`): `InitFailed` "Initialisation failed.",
`PotentiallyBlocked` "Potentially blocked by firewall.", `NetworkInitFailed`, and
`OpenLogFolder` "Open Log Folder..". The manual says that green means **activity**, and that
activity does not guarantee that it works — the distinction is written down, not implied.

**Project vs console divergence** (`UniverseTab`), the two most useful messages in the file:
- `ProjectConsoleLinkUniverseMismatch`: "One or more project fixtures are patched to universes not in control by the console..."
- `ProjectConsoleLinkFixtureMismatch`: "A difference in patch or type of fixture between console and project fixtures has been detected. The link between these fixtures will be broken."

**Patch and export warnings**: `Hog4DuplicateChannels` ("Some fixtures use the same
channel numbers and may not import correctly in the Hog 4." — duplicate address detected
at export), `Hog4MissingChannels`, `ExportFocusSheets/MissingUnits` ("One or more
fixtures did not have a unit set and were not exported!"),
`MainWindow/FixtureIdInconsistenciesFoundAndFixed` and `UnresolvedFixturesGroupsMayHaveChanged`.

**Scene limits** (`Navigator`), ceiling warnings: `TooManySmokeObjects`, `TooManyHDRIs`,
`TooManyReflectionPlanes`, `TooManyLiveFilters`.

**File** (`MainWindow`): `FileSizeError`, `ChecksumError` ("The file data is incorrect,
it is not as originally written."), `FormatTooNewError`,
`FormatUnsupportedWithoutLibraryError` and `FileErrorUseBackup` — which offers the automatic
backup instead of just failing. `SaveProjectLimitations`: "The project contains features
that cannot be saved and will be left out!" At startup (`Application`) there are seven separate,
named failures: configuration, licensing, network, video framework, external
connectivity, resources and real-time processing — failing early, each with a name of its own.

## Shortcuts

The `.c2l` **has no shortcuts section** and no Phrase contains "Ctrl", "Shift" or "F1":
a shortcut is not a translatable string. What exists is their editor, `OptionsWindow/KeyBindings`,
with `Command` and `Binding` columns and `Clear` and `Reset` buttons. The allowed grammar
(ToolsMenu.html#OptionsKeyBindingsTab) on Windows is `Ctrl` + A-Z, 0-9, comma, period,
hyphen or plus, with Shift and/or Alt optional (on macOS the same with Cmd, and Ctrl becomes an
extra modifier); a conflict between custom bindings warns, a conflict with the OS does not.
That is: **no bare key** (letter, Space, J/K/L, arrows) can be a command shortcut —
and the manual does not publish the list of default bindings on any of the pages read.

**Modifiers in the 3D view** (DesignViews.html) — these, yes, documented:

| Action | Input |
|---|---|
| Orbit / pan | middle button, or Alt + drag |
| Swap orbit ↔ pan | Shift (held) |
| Rotate without moving the camera | Ctrl / Cmd |
| Zoom moving the focal point | Shift + zoom |
| Zoom changing the field of view | Ctrl / Cmd + zoom |
| Add to the selection / toggle item | Shift + click / Ctrl + click |
| Box: only what is entirely inside | drag left to right |
| Box: also what it touches | drag right to left |
| Go down one level in the group | double-click |
| Move orthogonally only, 5° rotation, fine slider adjustment | Shift |
| Switch snapping off while moving | Ctrl / Cmd |
| Pan/tilt fan (scale) | Ctrl / Cmd + drag |
| Pan/tilt fan (offset) | Alt + drag |
| Extra point in the measurement / clear and leave | Shift + click / Escape |

In the Fixtures tab the table "can be navigated and edited as a spreadsheet using the arrow and
Enter/Return keys" (FixturesTab.html), with a search box in the top right corner and sorting by
clicking the column header (an arrow indicates the direction).

## EN → PT vocabulary

From the official translation (`Portuguese.c2l`). The third column is the decision for Spellcaster.

| EN | PT (Capture) | Spellcaster |
|---|---|---|
| Fixture | Aparelho | **Aparelho** (not "fixture", not "luminária") |
| Patch (noun and verb) | Patch / "Fazer patch" | **Patch** — the term belongs to the trade |
| Unpatch / Unpatched | Retirar do patch / Sem patch | same |
| Universe / Patch universe / Patch channel | Universo / Universo do patch / Canal do patch | Universo / Canal |
| Patch base | Base do patch | Base do patch |
| Patch style: Indexed / Contiguous | Indexado / Contínuo | Indexado / Contínuo |
| Start address | Endereço Inicial | Endereço inicial |
| Channels required | Canais requeridos | Canais necessários |
| Channel (the fixture's ID) | **Canal (ID)** | Canal (ID) — the disambiguation is good, copy it |
| Ch / Chs (DMX channel) | Canal / Canais | Canal DMX |
| Unit | **Etiqueta** | **Unidade** — "Etiqueta" loses the sense of numbering |
| Dimmer / Intensity | Dimmer / Intensidade | same |
| Shutter | Cortina (Shutter) | Shutter |
| Blinder | Máscara (Blinder) | Blinder |
| Framing shutters | **Facas** | Facas |
| Zoom | **Zum** | **Zoom** — nobody uses "Zum" |
| Focus | Foco / Afinação (`PlotFocus`) | Foco (the parameter), **Afinação** (the act) |
| Frost | Difuso (Frost) | Frost |
| Gobo / Gobo rotation | Gobo / Rotação do Gobo | Gobo / Rotação do gobo |
| CTO / CTB | CTO (correção para Âmbar) / CTB (para Azul) | CTO / CTB |
| Filter (of layers) | **Filtragem por camadas** | **Vista filtrada** — "Filtro" is kept for gel only |
| Filter (gel) | Filtro | Gel |
| Fixture group | Agrupamento / Grupo de Aparelhos | Grupo |
| Wireframe / Plot / Live | Aramada / Planta / Ao vivo | Aramada / Planta / **Ao vivo** |
| Camera position / Store camera | Posição de Câmeras / Gravar visão desta Câmera | Posição de câmera / Guardar câmera |
| Scene | Cena | Cena (but see "what NOT to copy", item 5) |
| Snapshot | **Instantâneo** | Instantâneo |
| Truss / Rigging point | Estrutura / Ponto de ancoragem | Treliça / Ponto de ancoragem |
| Smoke | **Caixa de fumaça** | Fumaça |
| Media player / ILDA frame rate | Reprodutor de Mídia / Taxa de Quadros ILDA | Player / Taxa de quadros ILDA |
| Frame list / Frame | **Lista de Caixilho / Caixilho** | **Roda de cor / Posição** |
| Material | **Textura** | **Material** |
| Connectivity status / Console link | Status da Conexão / Link com console | Estado da conexão / Vínculo com console |
| Levels / Blind levels | Intensidades / Níveis para o blind | Níveis / Níveis em blind |
| Plot / Report | Planta (Plotagem) / Relatório | Planta / Relatório |

## File

- Types declared in `FileTypes`: `CaptureProjectFiles` "Capture Project Files",
  `CaptureVersionProjectFiles` "Capture {0} Project Files", `CapturePresentationFiles`
  "Capture Presentation Files". **The extensions appear neither in the `.c2l` nor in the manual
  pages read** — I do not claim `.c2p` without a source.
- Model import: `.3ds`, `.dxf`, `.dwg`, `.gltf`/`.glb`, `.c4d`, `.mvr`, `.pdf`,
  `.skp`, `.obj` (FileMenu.html). Model export: DWG/DXF, glTF, MVR.
- Fixture data: CSV, TSV, Lightwright, grandMA2 XML, Hog 4 XML. The import
  (`ImportDataDialog`) maps file columns to `PositionX/Y/Z`, `RotationX/Y/Z`,
  `DefaultUnit`, `FocusPan`, `FocusTilt`, `ModeChannels`, identifying fixtures by a chosen
  property (`IdentifyFixtureBy`) or by drawing block name
  (`DrawingNameMatching`: Exact / Contains), and returns a report with `FixturesUpdated`,
  `FixturesAdded`, `LinesSkipped`, `MatchingGroupedImportedObjects`.
- Plot symbols import from SVG (`PlotSymbolsToolPane/ImportSymbol`).
- **`Installation\Presentation.zip`** (170,267,492 bytes, 18 entries, only listed): contains
  `Presenter.exe` (115 MB) and `Presenter.app/` for macOS (with `libndi.dylib` and
  `Resources2.blob`). No project or demo inside. It is the **payload of "Export
  Presentation"**: per FileMenu.html, the exported presentation is "a ZIP archive file
  containing" a Windows executable, a macOS app, a **non-modifiable** project and the
  connectivity settings; whoever receives it extracts it, runs it, and the visualiser opens the project
  in the Alpha view, with a snapshots panel in the Window menu. The standalone player is the same
  binary for everyone and the show is the locked project next to it — exactly the architecture
  of Spellcaster's standalone player.

## What Spellcaster should copy

- **Focus mode as a mode, not as a special click** (DesignViews.html#FocusMode): there is a
  mode in which clicking on the scale model points the already selected fixtures at the clicked spot,
  instead of selecting what was clicked. This solves the central ambiguity of the **interactive
  set** (function 4): the same click cannot mean "I want this fixture" and
  "point over here". Two modes, one shortcut, no modal.
- **Talkback: the scale model is input, not only output** (UniversesTab.html#dmx-talkback): clicking
  in the 3D returns pan/tilt to the console over CITP/SDMX, OSC or Hog. For the **orchestrator** (function 3)
  this makes the scene node bidirectional — the scale model is a source of events in the Graph,
  just like an OSC module.
- **Three named universe states** — `(auto)`, `(no)`, `Searching..`
  (`DMXUniverse/ExternalUniverse`) — plus `ResetLevels` and `ResetExternalUniverse`
  (`UniversesToolPane`). For **DMX scenes and cues** (function 4) and for the Outputs panel:
  "no signal" and "searching" are different things, and the operator needs to see which of the two it is.
- **The patch dialog shows `Channels required` before confirming** and treats universe
  overflow as a question with "Continue until complete" (`Patch/ChannelsRequired`,
  `OverflowWithContinue`). `spell patch` must say how many channels it will consume and where it
  overflows to before writing — **DMX scenes and cues**.
- **Four separate numbers per fixture, and batch sequential numbering for all four**:
  patch, channel (ID), unit and circuit (`Fixture/Patch`, `Channel`, `Object/Unit`,
  `Fixture/Circuit`), with Sequential Unit / Circuit / Patch / Channel
  (`SimulatorView/Sequential`). Whoever rigs uses all four, and patching 40 PARs in one operation
  is what separates a tool from a toy — **DMX scenes and cues**.
- **Simulation switchable off per layer** (`Layer/FixtureSimulation`, `Locked`, `Unselectable`):
  hiding is different from not simulating and from not being clickable. In the **interactive set** this is
  the performance button at show time, without deleting anything from the scale model.
- **A snapshot that records DMX, video, laser and camera together, and on playback overrides the
  external input** (SnapshotsTab.html: "DMX or media from external sources has no effect on the
  visualisation"). It is Spellcaster's rehearsal mode ready-made: a single recorder for the four
  streams, and "Recall DMX" to freeze a state when there is no signal — it serves the **ILDA
  player** (function 1), the **NDI→ILDA** (function 2) and cue rehearsal.
- **The media player is a patched fixture**: `ILDAFrameRate`, a 256-entry playlist
  (`MediaPlayer/Mode_Full`) and 2 DMX channels (control + selection). For the **ILDA player**: an
  `.ild` on stage must answer to a DMX channel like any other fixture, and the frame rate
  is a property of the player, not of the file.
- **A Connectivity Status that separates "there is traffic" from "it works"** and names the likely cause
  (`PotentiallyBlocked` "Potentially blocked by firewall.", `OpenLogFolder`): it is the speech
  pattern of **Aprendiz** (function 5) — the message says what to do and the log is one click away.
  Together with the **fixture table navigable as a spreadsheet** (arrows + Enter, sortable by
  header, with search — FixturesTab.html), it gives Aprendiz/menu and the interactive set the
  text surface that `PRINCIPIOS.md §4` demands.

## What NOT to copy

- **Three fixed views called Alpha, Beta and Gamma** (`SimulatorView/AlphaName`..`GammaName`).
  An opaque name that says nothing about the function, and a count locked at three. It breaks
  `PRINCIPIOS.md §4` (the name in the GUI teaches the vocabulary). In Spellcaster, a view is named after
  its use: `stage`, `house`, `bars`.
- **Layout in only two arrangements, Quad and Wide** (`Actions/Quad`, `Wide`), with the rest in
  floating windows (`ReportWindow`, `PlotWindow`, `FrameListWindow`, `ConsolePatchWindow`).
  A floating window at 11pm in a dark room is a window lost behind another. `PRINCIPIOS.md §3`
  asks for a fixed grid; `SHORTCUTS.md` already solves it with panels focusable by `Shift+1..7`.
- **Shortcuts limited to `Ctrl`+letter/digit** (ToolsMenu.html#OptionsKeyBindingsTab). That
  grammar makes the `SHORTCUTS.md` map impossible: Space, J/K/L, I/O, `M`, `S`, `R`,
  arrows. Do not inherit the restriction — inherit only the idea of the bindings editor with Command/Binding
  and a Reset button.
- **Sixteen tool panes in a column, with no search** (from `ProjectToolPane` to `UniversesToolPane`).
  `PRINCIPIOS.md §4` forbids a menu with more than 8 items and no search. The command palette solves
  that; the scrollable list does not.
- **"Scene" with Capture's meaning**. There, a scene stores the position and visibility of set
  objects (`Object/IncludeInScenes`) and the manual says there is no mechanism for storing and
  recalling values. In Spellcaster a scene is **a value per fixture**. Using the same word for
  both things would break the registry contract; Capture's equivalent here is
  "set position", kept separate.
- **"Blind levels" as a per-universe option** (`DMXUniverse/BlindLevelsMode`). It is a patch-up
  for consoles that send the programmer along with the stage. In Spellcaster, preview is an engine
  mode (rehearsal vs live, `Ctrl+Shift+R` / `Ctrl+Shift+Enter`), not a little box hidden in the
  properties of every universe.
- **Ceiling warnings only once the ceiling is already blown**: `TooManySmokeObjects`, `TooManyHDRIs`,
  `TooManyReflectionPlanes`, `TooManyLiveFilters` (`Navigator`). They arrive after the scene has already
  choked. Aprendiz must warn when the cost goes up, not when the limit blows — and the
  limit must be visible beforehand, the way the patch's `ChannelsRequired` is.
- **`DisableAdaptiveQualityWarning`** — "Are you really sure? ... please contact support so
  we can improve it.": a dialog that apologizes for a bug instead of fixing it. And **the whole
  licensing apparatus** (`Licensing`, `Unlock`, `Lock`, `GetKeyFileDialog`, `UnlockName`,
  `Upgrade`: 65 strings; one sixth of the language file is DRM). Out of scope.
- **Bad terms in the PT translation**: "Zum" (Zoom), "Caixilho" for a color-wheel frame,
  "Textura" for Material, "Etiqueta" for Unit, "Filtragem por camadas" and "Filtro" for two
  different things on the same screen. See the Spellcaster column of the vocabulary: Capture's
  translation is a good source of trade vocabulary and a bad source of consistent naming.
