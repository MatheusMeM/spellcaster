# TouchDesigner — interface audit from the real installation

Audit done by file reading only, without opening TouchDesigner. Installed version:
build 2025.30000 (footer of `Laser_Device_CHOP.htm`; snippets in `OPSnippets/Version.txt` = `990.681`).
Where the offline help does not have the page, it is stated.

Offline help root (`$H` from here on):
`C:\Program Files\Derivative\TouchDesigner\Samples\Learn\OfflineHelp\https.docs.derivative.ca\`
It is 2118 `.htm` files — the whole wiki mirrored, not a subset.

## Sources read

Config:
- `C:\Program Files\Derivative\TouchDesigner\Config\PanelShortcuts.txt` (520 B, read in full)
- `C:\Program Files\Derivative\TouchDesigner\Config\TouchShortcuts.txt` (4904 B, read in full)
- `C:\Program Files\Derivative\TouchDesigner\Config\TouchColors` (21105 B, read by grep: families, `parms.*`, `tile.*`, `playbar.*`)
- `C:\Program Files\Derivative\TouchDesigner\Config\MiscColors` (read in full)
- `C:\Program Files\Derivative\TouchDesigner\Config\3DSceneColors` (read in full)
- `C:\Program Files\Derivative\TouchDesigner\Config\opColorPalette.def`, `colorPalette.def` (read in full)

UI Python:
- `C:\Program Files\Derivative\TouchDesigner\bin\Lib\TDJSON.py` (745 lines; lines 1-40, 160-270, 370-400 read)
- there are also `bin\Lib\TDFunctions.py` (1186 lines) and `bin\Lib\TDStoreTools.py` (785 lines) — not opened beyond the `find`

Samples:
- `C:\Program Files\Derivative\TouchDesigner\Samples\Learn\TouchDesignerTips.txt` (read in full, 74 tips)
- `Samples\Learn\OPSnippets\Snippets\{CHOP,TOP,DAT,COMP,POP,SOP,MAT}\` (listed: 112 CHOP, 93 TOP, ...)
- `Samples\Palette\` (listed: Generators, ImageFilters, Mapping, MetaQuest, POPs, TDAbleton, TDBitwig, TDSynchro, TDVR, TDVS, Techniques, ThreadManager, Tools, UI, Vive, WebRTC, `defaultUserPalette.json`, `template.tox`)
- `Samples\Palette\UI\Basic Widgets\` (listed in full)

Offline help (`$H`), pages converted to text and read:
`Laser_CHOP.htm`, `Laser_Device_CHOP.htm`, `Scan_CHOP.htm`, `Lasers.htm`,
`NDI_In_TOP.htm`, `NDI_Out_TOP.htm`, `NDI.htm`, `DMX_Out_CHOP.htm`,
`Network_Editor.htm`, `Pane.htm`, `Timeline.htm`, `Component_Timeline.htm`,
`Parameter_Dialog.htm`, `Parameter_Dialog_Gadgets.htm`, `Parameter_Mode.htm`, `Value_Ladder.htm`,
`Custom_Parameters.htm`, `Page_Class.htm`, `Par_Class.htm`,
`Operator.htm`, `OP_Create_Dialog.htm`, `Flag.htm`, `Cook.htm`,
`Export.htm`, `Binding.htm`, `Perform_Mode.htm`, `Palette.htm`, `Widgets.htm`,
`Replicator_COMP.htm`, `Application_Shortcuts.htm`, `Keyboard_Shortcuts.htm`,
`Errors_Dialog.htm`, `File_Types.htm`, `Toeexpand.htm`, `Base_COMP.htm`, `Text_DAT.htm`,
`Palette-sceneChanger.htm`.

Does not exist / was not found:
- There is no "Keyboard Shortcuts" page with a table: `$H\Keyboard_Shortcuts.htm` has 8 lines and only points to `Application_Shortcuts.htm` and `Panel_Shortcuts.htm`. The real table is in `Application_Shortcuts.htm`, which reproduces the content of `Config\TouchShortcuts.txt`.
- There is no `presets` and no `keyboardIn` in `Samples\Palette\Tools\` (full listing checked). The only keyboard component is `Tools\onScreenKeyboard.tox`. `sceneChanger.tox` does exist.
- There is no ILDA/NDI-In snippet: `OPSnippets\Snippets\CHOP\` has `laserCHOP.tox`, `laserdeviceCHOP.tox`, `etherdreamCHOP.tox`, `dmxoutCHOP.tox`; `TOP\` has only `ndioutTOP.tox`, it does not have `ndiinTOP.tox`.
- `Config\SplashTips\` has no text: only `.tif` (logo, eye/padlock buttons, license types). The tip text lives in `Samples\Learn\TouchDesignerTips.txt`.
- `Config\Help\` has only `command.help` and the `exprhelp` folder (not opened).
- There is no "Externalize" page; externalization is described in the parameters of the COMP Common page (`Base_COMP.htm`) and in the `Sync to File` of the DATs (`Text_DAT.htm`).

## Anatomy of the screen

TouchDesigner is a window with a **menu bar at the top, Timeline at the bottom and Layout in the middle**;
the Layout is made of one or more Panes (`$H\Pane.htm`, definition of "Layout" at the foot of the page).

- **Pane**: work area; 9 types (Network Editor, Panel, Geometry Viewer, TOP Viewer, CHOP Viewer, Animation Editor, Parameters, Textport/DATs, Browser) — `$H\Pane.htm`.
- **Pane Bar**: at the top of every Pane. In the middle of it is the **network path**; to the left and to the right, viewer buttons, bookmarks, history (back/forward), split, fullscreen and link between panes (`$H\Pane.htm`). Clicking the `/` of the path navigates through menus; clicking the empty space lets you type/paste a path. Right button on any name in the path opens the menu of the COMP at that level (`TouchDesignerTips.txt:19`, `:31`).
- **Palette**: opened by the button to the left of the Pane Layout options, under the File menu, or by `Dialogs -> Palette Browser`, or `ui.openPaletteBrowser()` (`$H\Palette.htm`). It closes to gain space (`TouchDesignerTips.txt:13`) — that is, **it is not permanent**.
- **Parameter Dialog**: sits **to the right of the network** ("Parameter dialogs are displayed on the right side of a network"), appears and disappears with the `p` key, and can exist in three places: inside the Network Editor, as a floating window (RMB on the node → Parameters...) or as a Pane type (`$H\Parameter_Dialog.htm`; `TouchDesignerTips.txt:4`, `:36`). Several can stay open at the same time via the Sticky button; the top one is the current OP.
- **OP Create Dialog** (the "Tab menu"): opens with `Tab`, double-click on the network background, `+` button on the Pane Bar next to the path, MMB/RMB on the input/output connectors, or RMB on a wire (`$H\OP_Create_Dialog.htm`). It has search by typing: typing `midi` lights up in white every matching type. Generators (0 inputs) appear in a darker shade of the family color; filters in the lighter shade.
- **Timeline** at the bottom: timecode (frames or beats), `fps` field, `frame` field, and transport Reset / Pause / Reverse Play / Play / Step Back / Step Forward; Range Limit buttons (Loop or Once); Start/End fields (total length), RStart/REnd (working range, drawn as a colored bar above the time index), FPS, BPM, ResetF, T Sig (`$H\Timeline.htm`).
- **Timepath**: the bottom Timeline is not global by decree — it is *scoped* to a Time COMP. Default is root (`/`). The `S` button on a component's mini-timeline scopes that time at the bottom, **and the Timeline changes color** to the color of that Component Time; the `[/]` button goes back to root, whose color is always the dark blue of the interface (`$H\Timeline.htm`, `$H\Component_Timeline.htm`).
- **Component Timeline**: mini-timeline at the bottom of the network editor of any network that has Component Time, with two buttons: `I` (Run Independently) and `S` (Scope) (`$H\Component_Timeline.htm`).

**Perform Mode vs Designer Mode** (`$H\Perform_Mode.htm`):
- Perform Mode renders **a single Window COMP** and nothing else; the network editing window does not exist. It is optimized: starting straight in Perform Mode, "the extra memory the Designer interface requires will not be used".
- `F1` enters, `Esc` exits (`Shift+Esc` if the Window COMP has 'Close on Escape Key' turned off). `ui.performMode = True` in Python.
- Which window is the perform one is set in the **Window Placement** dialog, "Perform Window" column. "Start in Perform Mode" makes the file open already in performance.
- Full-screen exclusive (Windows): only if the window is borderless and covers 100% of the desktop on a single/Mosaic monitor. Without that, the Windows compositor swallows frames and you see stutter even running a stable 60 FPS.
- If the file has the Privacy option on, **you cannot leave** Perform Mode.
- Pausing the timeline in Perform Mode is `Shift+Space`, not `Space` (`$H\Perform_Mode.htm`; `TouchDesignerTips.txt:28`, `:52`).
- `F10`/`F9` with the cursor over any UI element shows its network — including in Perform Mode (`TouchDesignerTips.txt:7`).

## Objects and verbs

**Seven families** (`$H\Operator.htm`): COMP (contain networks), TOP (image, GPU), CHOP (channels: motion, audio, control, protocols), POP (points on the GPU), DAT (text and tables), MAT (materials), SOP (legacy geometry). Hard rule: **"Only operators of the same family (color) can be Wired together."** Reference between different families is done by **Link** (parameter pointing to an OP) or by **Export** (CHOP → parameter of any OP).

Inside each family: **generator** = 0 inputs; **filter** = 1+ inputs.

**Create**: `Tab` → OP Create Dialog → click the name → click the network to place it. With `Ctrl` held you can drop several; with `Shift` held, the operators come out **already wired in series**, and switching family starts a new branch (`$H\OP_Create_Dialog.htm`).

**Connect / edit wiring** (`$H\Network_Editor.htm`, `TouchDesignerTips.txt`):
- Insert a node in the middle of a wire: RMB on the wire or on the output of the source node (`:1`).
- New branch from an output already connected: MMB on the output (`:5`).
- Change the input: click from the output of another node onto any point of the existing wire (`:25`).
- MMB on the wire shows what is passing through; rollover shows source and destination (`:24`).
- **Wire with animated dashes = the source is cooking** (`:29`, `$H\Cook.htm`).
- 3D components and 2D panel components have connectors on top and bottom, for parenting hierarchy and panel grouping — **no data flow** through them (`:22`, `:46`).

**Cook** (`$H\Cook.htm`): to cook = to compute. TD does not cook everything every frame. A node cooks if it has (1) a *request* to cook and (2) a *reason*. The request comes from: a downstream node wants to cook; a node that references it by parameter wants to cook; a viewer is looking; the target of an export wants to cook; `cook()` called. The reason comes from: an input cooked (among others). What triggers the whole chain are the visible viewers, the displayed panels, and the **output** nodes (Touch Out CHOP, OSC Out CHOP, NDI Out TOP, Audio Device Out CHOP). MMB on a node shows the time of the last cook and the cook count. `Dialogs -> Performance Monitor` shows what cooked in a frame; the `probe` component from the Palette watches it live.

**Flags** (`$H\Flag.htm`) — binary states on the left and bottom edge of the node, **they are not parameters, they do not cook and they cannot receive an export**:
- on all: Viewer, Viewer Active, Lock (freezes the data in memory and saves it in the `.toe`/`.tox`), Bypass (input 0 passes straight through; on a COMP it makes everything inside it bypassed), Cooking (on a COMP, it prevents the interior from cooking), Immune (node immune to the clone), Current, Selected, Expose, Python.
- on 3D objects: Render, Display, Pickable.
- on CHOPs: Export.
- on SOPs: Compare, Template.
- In Table View (`Shift+T`) the full set of flags appears as columns (`$H\Flag.htm`; `TouchDesignerTips.txt:21`).

**Viewer Active** = the viewer inside the node becomes interactive; with it off, clicking/dragging on the node moves and selects the node. `a` toggles it on the selected nodes; `Alt+a` puts all of them in Viewer Active while the key is held (`TouchDesignerTips.txt:17`, `:69`). With Viewer Active on, the name field at the bottom of the node still works for dragging/info/menu (`:35`).

**Clone vs Replicator** (`$H\Replicator_COMP.htm`): Clone syncs the interior of a COMP with a master. **Replicator is the for-loop**: it creates and destroys nodes ("replicants") according to the rows of a DAT table or the Number of Replicants parameter. It names them by index (`item1, item2...`) or by a table column. Replicant layout in Off/Horizontal/Vertical/Grid. `Incremental Update` creates N replicants per frame (default 1) so as not to drop frames. If only one table row changes, the other replicants **are not recreated**. Each replicant can go through a callback that adjusts a parameter, an expression or a mode (`c.par.display.mode = ParMode.EXPRESSION`). Example cited: feeding the Replicator directly with the Multi Touch In DAT table to create something on each finger.

## Parameters (type → widget → gesture)

**Header** of the Parameter Dialog (`$H\Parameter_Dialog.htm`): top with OP Type and OP Name; **the header background is the family color** of the operator. Below: Operator Help, Python Class Help, Operator Info, Comment, Clipboard, Python/Tscript toggle, Hide/Show Default Parameters.

**Pages**: every operator has one or more parameter pages (tabs); some specific to the type, others common (Common). When several OPs are selected, the page chosen on the current one becomes the first one open on each of the others — it saves clicks when going through many nodes (`$H\Parameter_Dialog.htm`). Custom parameters appear in a **second row of pages** (`$H\Custom_Parameters.htm`).

**Types** — the canonical list is the set of `append*` of `Page_Class` (`$H\Page_Class.htm`):
`Int, Float, XY, XYZ, XYZW, WH, UV, UVW, RGB, RGBA, Str, StrMenu, Menu, File, FileSave, Folder, Pulse, Momentary, Toggle, Python, Header, Sequence, ParGroup` plus the OP refs: `OP, COMP, Object, PanelCOMP, TOP, CHOP, SOP, POP, MAT, DAT`.
`Par.style` returns exactly that as a string: `'Float'`, `'Int'`, `'Pulse'`, `'XYZ'` and so on (`$H\Par_Class.htm`).
Maximum size of a numeric tuplet is 4 (`$H\Custom_Parameters.htm`).

| Type | Widget (`$H\Parameter_Dialog_Gadgets.htm`) | Gesture |
|---|---|---|
| Float / Int (size 1) | Single Number with Slider (and the variant with Scale) | type in the field; **MMB (or LMB) held over the field or over the label opens the Value Ladder** |
| Float/Int size 2-4 (XY, XYZ, RGB, WH...) | Multiple Numbers on the same line | ladder on the **label** moves the 2/3/4 together; ladder on the field moves only that one |
| RGB / RGBA | Color Picker with RGB tuple | same as above |
| Toggle | Check Box (single and multiple) | click |
| Menu | Drop Down Menu (single and multiple) | click |
| Str | Text Box | type |
| File / FileSave / Folder | File System Path with an open-file button | type or button |
| OP/TOP/CHOP/... ref | Operator Path with "Jump To" button | type the path or button to jump to the node |
| Menu of ordered items | Ordered List | — |
| Pulse | button | fires and returns: `pulse()` "sets the parameter to the value, cooks the operator, and restores the previous value"; for the Pulse type neither value nor time is specified (`$H\Par_Class.htm`) |
| Momentary | button | a type distinct from Pulse and from Toggle; `Par.isMomentary` exists alongside `isPulse` and `isToggle` (`$H\Par_Class.htm`). **The offline help does not describe the semantics of Momentary in prose** — only the existence of the type and of the test. |

**Value Ladder** (`$H\Value_Ladder.htm`, `TouchDesignerTips.txt:16`): hold MMB (or LMB) over the value or the label; it opens a ladder with `.01 .1 1 10 100`; still with the button held, moving **vertically** picks the increment and dragging **right/left** increases/decreases by that increment. Without a middle button, `Alt+RMB` replaces it (`TouchDesignerTips.txt:3`).

**Four parameter modes** (`$H\Parameter_Mode.htm`, `$H\Parameter_Dialog.htm`):
clicking the label (or the `+` that appears on hover) expands the row, showing **the internal name of the parameter** (the name scripts use) and four square buttons:

| Mode | Button color in the doc | Real color in the theme | What it is |
|---|---|---|---|
| Constant | gray | `parms.const.bg 0.425 0.425 0.425` | typed value; default |
| Expression | blue | `parms.expr.bg 0.5 0.7 0.7` (cyan) | Python/Tscript expression |
| Export | green | `parms.override.bg 0.35 0.5 0.25` | driven by CHOP/DAT |
| Bind | purple | `parms.bind.bg.enabled 0.77 0.7 1` | bidirectional binding |

(The doc calls it "blue button for expression"; `TouchColors` stores the real value as a desaturated cyan. Export in `TouchColors` is called `override`, not `export`.)

The point that matters: **the four values are stored at the same time in the parameter**. You can have a constant, an expression, an export and a bind on the same parameter and switch freely. If an expression/export/bind is configured but the mode is not selected, that mode's button shows **a small square in the bottom-left corner**. That is the debugging mechanism: jump to the constant to test a value and come back without losing the logic. Export can only be selected if there is already an export arriving.

**Expression vs Export vs Bind** (`$H\Export.htm`, `$H\Binding.htm`):
- Export = the CHOP (or DAT) pushes to the parameter, one-way. Gesture: Viewer Active on the CHOP, drag the channel onto the node (wait for it to become current and the parameter panel to open), keep dragging onto the parameter, drop, choose "Export CHOP" (`TouchDesignerTips.txt:11`, `:55` — `:55` adds that you can hover over the **page tab** to switch pages mid-drag). It appears in the network as a **dotted gray data link with an arrow**, animated when it cooks. The CHOP's Export Flag turns everything on/off at once.
- Mass export: name the channels as `path/to/op:parameter` and set Export Method = `Channel Name is Path:Parameter`. A single gesture connects dozens of parameters.
- Expression = the parameter pulls. Since ~2017 expressions are compiled and **performance is the same**; the choice is semantic: an expression survives channel reordering (an export does not), allows math (`op('null1')[0] * 2 - 1`) and conditional choice of what to reference; export allows turning off in bulk and mass-export.
- Bind = bidirectional, with a **bind master** that holds the true value and bind references chained or many-to-one. Masters can be a table cell, a Bind CHOP channel, a Panel Value or a Dependency object. A bind reference does not use constant/expression/export. Gesture: **always drag from the master to the reference**.
- `Ctrl+E` over an expression opens it in the text editor (`TouchDesignerTips.txt:2`).
- RMB → Copy Parameter, then RMB → Paste Parameter Reference creates the reference or the bind (`:15`). Dragging from label to label works even on multi-field parameters such as RGB (`:54`).

**Parameter help**: hold `Alt` and hover the mouse over the label to show the wiki help (`$H\Parameter_Dialog.htm`; `TouchDesignerTips.txt:27`). The body of that help is in `Config\TDParameterHelp.json` (5.45 MB) — not parsed here.

**Parameter JSON schema** (`bin\Lib\TDJSON.py`, `parameterToJSONPar`, lines 186-190):
```
('name', 'tupletName', 'label', 'page', 'sequence', 'style',
 'size', 'defaultMode', 'default', 'defaultExpr', 'defaultBindExpr',
 'enable', 'startSection', 'cloneImmune', 'readOnly', 'enableExpr', 'help')
```
If `isNumber`, it adds `NUMATTRS = ('min','max','normMin','normMax','clampMin','clampMax')` (`TDJSON.py:22`).
If `isMenu`, it adds `('menuSource',)` or `('menuNames','menuLabels')` (`TDJSON.py:213-215`).
On the way back (`addParameterFromJSONDict`), **the required keys are only three**: `{'page', 'style', 'name'}` (`TDJSON.py:379`).
A tuplet becomes **a single dictionary** with `size` = length of the tuplet (`TDJSON.py:255-257`).
Important distinction: `min`/`max` are the hard clamp (with `clampMin`/`clampMax` enabling it or not), `normMin`/`normMax` are **only the slider range** (`$H\Custom_Parameters.htm`).

**Naming convention** (`$H\Custom_Parameters.htm`): a native parameter is all lowercase (`brightness`); a custom parameter starts with a capital and the rest lowercase (`Divisions`) — if the first letter is not a capital, **creation fails with an error**. No underscore. Recommended maximum of 12 characters, preferably 10, "because they become unreadable when you open the parameter with the + icon". Label in Title Case minus prepositions: `Rotate to X Axis`.

## States

**Color by family** — `Config\TouchColors:1-14`, RGB values 0-1:

| Family | RGB | Reading |
|---|---|---|
| CHOP | 0.385 0.55 0.275 | green |
| COMP | 0.19 0.19 0.19 | almost black gray |
| DAT | 0.575 0.36 0.50 | greyish magenta |
| MAT | 0.625 0.58 0.28 | ochre |
| POP | 0.315 0.305 0.75 | blue-violet |
| SOP | 0.29 0.5 0.7 | blue |
| TOP | 0.41 0.36 0.575 | purple |

Each family has `.hilite` (light version for selection) and some have `.editbg`. Note that **all seven are desaturated**, similar luminance; nothing saturates. Generator vs filter is the same hue in two shades (`$H\OP_Create_Dialog.htm`).

`opColorPalette.def` is another thing: these are the **24 colors the user can apply to a node** (`c` key), 18 hues in a circle of identical saturation (~0.8 in the dominant channel) plus 6 greys. `colorPalette.def` are 44 colors for general use in 4 rows (11 greys, 11 dark, 11 pastel, 11 saturated).

**Error and warning** (`Config\MiscColors`, `Config\TouchColors`):
- `ErrorFlag 0.9 0.1 0.1`, `WarningFlag 1 1 0`, `MessageFlag 0.8 0.8 0.8`, `FilteredFlag 0.6 0.6 0.6` — "used for the indicator colors on the info button".
- `tile.error 1 0 0`, `tile.warning 1 1 0` — the whole node.
- `parms.err.bg 1 0 0` with `parms.err.fg 0.8 0.8 0.8`; `parms.disabled.err.bg 0.5 0 0` — error **in the parameter field**, not only in the node.
- `default.error 1 0 0`, `dialog.error 1 0 0`.
- The `Errors Dialog` lists the errors and **clicking takes you to the offending operator** (`$H\Errors_Dialog.htm` — the page has a single sentence). The Error DAT records errors and warnings over time, to hunt intermittent ones (`TouchDesignerTips.txt:66`).
- Info channels of any operator include `warnings` and `errors` as a numeric count, besides `cook_time`, `total_cooks`, `cooked_this_frame` (`$H\Laser_Device_CHOP.htm`, Common Operator Info Channels section). That is: **the error state is readable as data**, not only as paint.

**State on the node** (`Config\TouchColors`, prefix `tile.`):
`tile.current 0 1 0` (pure green — the current node), `tile.picked 0.85 0.85 0` (yellow — selected), `tile.connection.hilite1 1 1 0`, `tile.commented 0.3 0.36 0.6`, `tile.ghost 0.3 0.3 0.3`.
Individually colored flags: `tile.flag.display 0.195 0.49 1` (blue), `tile.flag.render 0.53 0.402 1` (purple), `tile.flag.export 0.525 0.75 0.375` (green), `tile.flag.clone 1 0.3 0.3` (red), `tile.flag.clonechild 0.5 0.15 0.15`, `tile.flag.expose 1 0 0`, `tile.flag.hardlock 1 1 0`, `tile.flag.template 0.871 0.377 0.892`, `tile.flag.pickable 0.885 0.557 0.097`, `tile.flag.bypass.cross 0.8 0 0`, `tile.flagv.cloneimmune 0.9 0.45 0`.
In `MiscColors` there is the on/off pair for each flag, always the same color at two luminances: `DisplayOnColor .3 .5 1` / `DisplayOffColor .2 .3 .6`; `BypassOnColor 1 .5 0` / `BypassOffColor .5 .25 0`; `ExposeOnColor 1 0 0` / `ExposeOffColor .6 0 0`; `CurrentColor 0.25 0.85 0.25`.

**Transport** (`Config\MiscColors`, `TouchColors:473-476`):
`PlayBarOnColor 0.05 0.8 0.05` (green playing) / `PlayBarOffColor 0.1 0.1 0.1` (black stopped) / `PlayBarDisabledColor 0.4 0.4 0.4` / `PlayBarResetColor 1 1 0` (yellow).
There is also an explicit **pending** pair: `PlayBarPending 0.8 0 0` and `PlayBarNoPending 0 0.8 0` — red when there is an unapplied change, green when there is none. `PendingColor 0.75 0.0 0.0` in the parameter block says the same: **a pending change is red**.

**Keyframe state** (`Config\MiscColors`, with the file's own comment):
`IsKeyColor 0.0 0.55 0.25` ("Keyframe!"), `IsSoftKeyColor 0.0 0.25 0.55`, `IsNotKeyColor 0.80 0.80 0.0` ("has a channel but is not on a key"), `LockedColor 0.75 0.55 0.60`. The file header states the rule: "the luminances of most of these colors should be similar, so that none stands out too much and so that the text in the parameter fields stays readable".

**Cooking**: it is not a time bar in the UI — it is the **animated dashed wire** (`$H\Cook.htm`, `TouchDesignerTips.txt:29`) plus the MMB popup on the node (last cook time, count) and the Performance Monitor. The offline help does not describe any cook progress bar.

## Shortcuts

Definitive source: `Config\TouchShortcuts.txt`, read at startup (`$H\Application_Shortcuts.htm`). Three columns: label, key, command. `000` = no key defined. User override: a file with the same format in `app.preferencesFolder\TouchShortcuts.txt`; to disable a shortcut, keep the line and **delete the command**. In Perform Mode you can override by locking the DAT at `/local/shortcuts`.

Global (`TouchShortcuts.txt`, `general.*` block):

| Action | Key |
|---|---|
| Timeline pause/play | `Space` (`general.pause`) |
| Frame step | `←` / `→` (`general.forward` / `general.backward` — the labels are swapped with respect to the movement described in `Application_Shortcuts.htm`, which says right arrow = advance one frame) |
| Turn global cooking on/off | `Ctrl+Space` (`general.cooking`, command `offon`) |
| F1..F12 and Alt+F1..F12 | reserved as named slots (`F1` is Perform Mode) |

Network editor (`TouchShortcuts.txt`, `network.*` block):

| Action | Key |
|---|---|
| Create operator (Tab menu) | `Tab` |
| Quick Null from the current node | `Alt+N` |
| Enter the component | `Enter` or `i` |
| Go up one level | `u` |
| Cancel | `Esc` |
| Show/hide parameters | `p` |
| Frame all / frame selected | `f` / `Shift+F` |
| Home / home selected | `h` / `Shift+H` |
| Overview (network map) | `o` |
| Table mode (Table View) | `Shift+T` |
| Activate viewers of the selected | `a` |
| Toggle Render / Bypass / Display | `r` / `b` / `d` |
| Node color palette | `c` |
| Rename | `n` |
| Connection style | `s` |
| Show data links (exports) | `x` |
| Edit/expose | `e` |
| Load `.tox` | `Shift+X` |
| Comment / annotate / network box | `Shift+C` / `Shift+A` / `Shift+B` |
| Group / open groups | `Shift+G` / `Ctrl+G` |
| Select all | `Ctrl+A`; deselect one node: `Ctrl+click` |
| Copy / paste / paste at mouse / cut | `Ctrl+C` / `Ctrl+V` / `Ctrl+Shift+V` / `Ctrl+X` |
| Delete | `Del` or `Backspace` |
| Find / browser / search | `Ctrl+F` / `Ctrl+B` / `Alt+S` |
| Zoom | `Ctrl+=` / `Ctrl+-`; scroll with MMB; box-zoom with `Ctrl`+MMB |
| Scroll the view | `Ctrl+↑↓←→` |
| Network history | `Alt+←` / `Alt+→` |
| Navigate the list (Table View) | `j` `k` `,` `.` |
| Edit/run DAT | `Ctrl+E` / `Ctrl+R` |
| New project / import / export movie | `Ctrl+P` / `Ctrl+I` / `Ctrl+M` |

Switching Pane type (`network.switchto.*`): `Alt+1` net, `Alt+2` panel, `Alt+3` geoview, `Alt+4` topview, `Alt+5` chopview, `Alt+6` keyframer, `Alt+7` parm, `Alt+8` opbrowser, `Alt+9` textport.
Panes: `Alt+[` split L/R, `Alt+]` split T/B, `` Alt+` `` fullscreen, `Alt+Z` close, `Alt+Shift+C` clone, `Alt++` / `Alt+-` link.
Dialogs: `Alt+P` preferences, `Alt+L` palette, `Alt+O` operator browser, `Alt+T` textport, `Alt+H` help, `Alt+B` bookmarks, `Alt+C` console, `Alt+F` explorer, `Alt+Y` performance, `Alt+D` MIDI mapper, `Alt+K` license, `Alt+W` window placement.
App: `Ctrl+S` save, `Ctrl+Shift+S` save as, `Ctrl+O` open, `Ctrl+Q` quit, `Ctrl+Z` undo, `Ctrl+Y` redo.

Keyframer (`keyframer.*`): `h`/`Shift+H` home, `Shift+F`/`Shift+V` home horizontal/vertical, `n` long names, `e` handle scale, `t` tie the selected, `Del` delete key, `Ctrl+C`/`Ctrl+V` copy/paste key, `Ctrl+←`/`Ctrl+→` previous/next key, **`Alt+LMB` adds a key on the nearest**, **`Alt+MMB` adds on the selected**, `Ctrl+J` key on the selected at the playhead, `Ctrl+K` key on all at the playhead, `Ctrl+=`/`Ctrl+-` zoom.

CHOP viewer (`chopviewer.*`): `h` home, `Shift+H`/`Shift+V` adapt horizontal/vertical, `t` time bar, `c` time scroll, `l` labels, `x` extend, `d` dots, `n` handles, `g` grid, `u` units, `e` edit menu, `s` scope menu, `p` precise.

`Config\PanelShortcuts.txt` is a **different and much smaller** table (32 lines): it is the set that applies inside a Panel/in Perform Mode. It only has `space`, `left`, `right` mapped to *nothing* (empty command = disabled) and **`shift.space` → `space`, `shift.left` → `left`, `shift.right` → `right`**, plus `ctrl.s` → `toewrite -s` and the F1..F12/Alt+F1..F12 slots. That is: **inside a panel, the transport requires Shift** — on purpose, so the operator does not pause the show while typing. `Application_Shortcuts.htm` confirms it: "(The keys are different in Panel Shortcuts.)" and Preferences has "Enable Playbar Shortcuts" to turn that off for good (`TouchDesignerTips.txt:58`).

Mouse in the Network Editor (`$H\Network_Editor.htm`): LMB selects and drags the node; LMB on empty space **pans without changing the zoom**; `Shift`+LMB drag or RMB does box-select; `Ctrl`+click adds to the selection; MMB dragging controls the zoom; scroll = zoom; `Ctrl`+MMB left to right = box zoom in, right to left = zoom out; `f` returns to zoom 1. Zoom into a COMP until you enter it, zoom out until you leave it. MMB on the node opens the info popup.

## Laser and NDI (real parameters)

### Five laser paths (`$H\Lasers.htm`)
EtherDream (Ethernet → ILDA), Helios (serial/USB, and IDN over Ethernet), ShowNET (Ethernet, embedded DAC), LaserAnimation Sollinger AVB (via Audio Device Out CHOP on a low-latency AVB device; the AVB2ILDA gives 24 bits in X/Y and color, electronic zone masking and per-channel color delay), and Pangolin Beyond (Pangolin CHOP; Beyond is the one that manages the safety mask and shutdown). In all of them except Pangolin: `CHOP/POP/SOP → Laser CHOP → Laser Device CHOP`.

### Laser CHOP (`$H\Laser_CHOP.htm`)
Replaces the Scan CHOP, which is marked DEPRECATED (`$H\Scan_CHOP.htm`). Typical declared rate: **10,000 to 96,000 samples/s**.

CHOP input: channels `x`, `y` mandatory; optional `z`, `r`, `g`, `b`, `id`. **`id` groups points into a shape**: id=0 is the first shape, id=1 the second; without `id`, each point is loose and disconnected. Any other channel is treated as color and receives blanking — which allows projectors with more diodes than RGB. Extra recognized channels: `lascorner`, `lascornerholdadd`, `lascornerholdlookupfactor`.

**Corner points vs guide points** (new in 2025.30000; before that every point was a corner). Boolean attribute `LasCorner` in SOP/POP, channel `lascorner` in CHOP. A corner defines the start/end of a segment and carries repeated hold points as a function of the angle; a guide point only helps the curve and **is emitted once, never repeated**. The repetition is `NumRepeatPoints = HoldAdd + HoldLookupFactor * H`.

Laser page:
`active`; `source` (menu `sop` / `chop` / `pop`); `sop` / `chop` / `pop` (path); **`outputrate`** — Output Sample Rate, samples/s, **default 48000; at 60 fps that gives 800 position+color pairs per frame**; `swap` (swaps the X/Y axes); `xscale`; `yscale`; `rotate`; `camera` (Camera COMP to draw a SOP from the camera view); `updatemethod` (menu `alldrawn` / `everyframe`); `startpulse` (Frame Start Pulse — inserts a sample with all colors at **-1** at the start of the frame); `debugchan`; `cornerattr` (default `LasCorner`); `cornerholdaddattr` (default `LasCornerHoldAdd`); `cornerholdfactorattr` (default `LasCornerHoldLookupFactor`).

`updatemethod` is the parameter that describes **flicker from too many points**: if the drawing does not fit in a frame at the configured rate, "the effect will be visible as the laser image flickering". `alldrawn` (default) only takes new data when it has finished drawing everything; `everyframe` updates always.

`debugchan` generates a channel with the **state of each point**, and that is a ready-made state machine:
`-1` Frame Start Pulse, `0` Color, `1` Corner Hold Point, `2` Start Point Hold Time, `3` Pre Blank On, `4` Post Blank On, `5` Blanking, `6` Pre Blank Off, `7` Post Blank Off.

Scanning page: `stepsize` (distance x,y may change per sample **while emitting color**); `bstepsize` (Blanking Step Size — the same distance **while blanked**); `mincornerhold` and `maxcornerhold` (the point hold is interpolated linearly between the two **according to the opening of the angle**: 180° gives the minimum, 0° gives the maximum; if max < min, max is clamped upward); `cornerholdchop` (CHOP as a custom lookup curve in place of the linear interpolation); `closedoverlap` (Closed Shape Overlap, **in milliseconds**, start/end overlap in a closed shape so the color interpolation closes evenly).

Color page — **all four blanking delays are in ms**:
`redscale`, `greenscale`, `bluescale`; `preblankon` (wait before **turning off** the color), `postblankon` (wait after turning off), `preblankoff` (wait before **turning on**), `postblankoff` (wait after turning on); `starthold` (Start-Point Hold Time, ms, wait on the first point of a new frame); `colordelay` (delay of the color channels on output, ms); `interpcolors`; `brightnesscurvechop`.

The physical reason is written on the page: "since the laser mirrors are moved by motors, the positional data sent is likely ahead of the actual mirror position — the mirror needs to catch up with the data. The color data, however, is on time, and the result can be visible tails at the points where the laser turns the color off." The blanking parameters exist to compensate for that.

Explicit compatibility note: in 2025.30000 **Start Point Hold Time** and **Input Rate** disappeared; point generation now always happens at **192000** and is then resampled to the Output Rate, which changes the effect of `stepsize` and `bstepsize` in old files.

The Laser CHOP **exposes no specific info channels** — only the common CHOP and operator ones (`$H\Laser_CHOP.htm`, Info CHOP Channels section). There is no "points per frame" channel and no "effective kpps" channel.

**There is no safe zone parameter, and no max points parameter, in the Laser CHOP or in the Laser Device CHOP.** Safety is external: the all-caps warning on the page ("LASERS ARE DANGEROUS...", requiring a certified Laser Safety Officer, an emergency button within reach, nobody in the projection area, no reflective surface) and the masking done by the AVB2ILDA or by Pangolin Beyond (`$H\Lasers.htm`). No page reads "kpps" — TD's unit is always samples/second.

### Laser Device CHOP (`$H\Laser_Device_CHOP.htm`)
Input/output channels, with declared range: `x` and `y` **between -1 and 1**; `r`, `g`, `b`, `i` **between 0 and 1**; `user1`..`user4` optional (**`user3` and `user4` are not supported by the EtherDream**).

`active`; `type` (menu `etherdream` / `helios` / `shownet`); `device` (menu that **self-populates** with connected Helios and ShowNET; ShowNET refreshes by itself); `scan` (pulse — scans for Helios; the page warns that **there must be no active Helios connection during the scan, otherwise it is closed by the process**); `netaddress` and `port` (EtherDream); `localaddress` (picks the NIC when there are several); `queuetime` + `queueunits` (menu `samples` / `frames` / `seconds`) — "determines the size of the Helios/EtherDream point buffer queue and the corresponding time to drain it; it is often useful to reduce this value when sending few points"; `xscale`, `yscale`, `redscale`, `greenscale`, `bluescale`, `intensityscale`.

Blanking here is implicit: "it happens when the input RGB channels are all zero, **or** when Red Scale, Green Scale and Blue Scale are all zero" — that is, the three scales at zero work as a shutter.

Discovering the EtherDream IP is a known problem and the solution is the EtherDream DAT (`$H\Laser_Device_CHOP.htm`, `$H\Lasers.htm`).

### Scan CHOP — deprecated, but it is the only documented raster→vector path (`$H\Scan_CHOP.htm`)
It matters because it is literally NDI→ILDA: it converts a **TOP** (image) into x/y control waves.
Scan page: `source` (menu `top` / `sop` / `chop`); `rate` (Sample Rate, samples/s); `swap`; `xscale`; `yscale`; `rotate` (degrees); `randomize` (emits the samples in random order — "creates a diffuse and chaotic image on the oscilloscope"); `color` (turns on the r,g,b channels); `redscale`, `greenscale`, `bluescale`; `blankingcount` (**count of blanked positions** — between geometry primitives in the SOP case, between full sweeps in the TOP case; requires Output Color on).
TOP page: `top`; `width` (columns to resample); `height` (rows); `level` (**number of brightness levels each pixel can have**); `limit` (Auto Reduce — dynamically reduces rows and columns to keep the output frame rate constant); `layered` (emits the pixels **in brightness order**, instead of left-to-right per row); `interleave` (menu `sweep` / `evenodd` / `max` — "controls the order in which the rows are emitted to minimize flicker").
SOP page: `sop`; `vertexorder`; `limitstep` ("breaks long x,y jumps into several smaller incremental jumps"); `stepsize`; `vertexrepeat`; `camera`.
TD's text about the TOP mode: "luminance is controlled by how long each sample is 'drawn' on the scope. Since the output is low bandwidth, different resampling and ordering options are available to minimize (or enhance) the flicker."

### NDI In TOP (`$H\NDI_In_TOP.htm`)
`active`; `name` (**Source Name**, menu of discovered streams); `extraips` ("by default NDI searches by mDNS, which is usually limited to local networks" — list of IPs separated by spaces for sources outside multicast range); `bandwidth` (menu **`high` / `low`**, two values only); `hwdecode` (only works for NDI|HX, which is H264 — "the NDI Out TOP and other software NDI solutions can only send native NDI, not NDI|HX"); `inputpixelformat` (menu `native` / `fixed8`); `inputcolorspace` (long menu, `automatic` by default); `inputreferencewhite`; `grouptable` (DAT with group names to filter the listed sources); `audiobuflen` (seconds — "the audio output is delayed by this value").
**There is no FPS parameter on the NDI In.** The frame rate arrives as a reading, via Info CHOP: `connected`, `receive_fps`, `num_source`, `queue_size`, `received_frames`, `missed_frames`. These six are the health panel of the link.

### NDI Out TOP (`$H\NDI_Out_TOP.htm`)
`active`; `name` (Source Name); `failovername` (format `MACHINENAME (SourceName)` — where the receivers migrate to if this source goes down); **`fps`** — and the important note: "NDI uses the FPS partly as a guide to control how the frames are compressed. The higher the FPS, the more compressed the frames. Sending at 1 FPS results in higher image quality than sending at 30 FPS"; `lowperformancebehavior` (menu `stallmainthread` / `skipframes` — either the main thread stalls and yields resources to the sender, or it goes on and the sender starves and drops FPS: **both paths lose, and the UI forces the choice**); `outputpixelformat` (menu `fixed8` / `fixed16`); `includealpha`; `grouptable`; `audiochop`; `metadata` (DAT as a table or XML); `outputcolorspace`.

### NDI, the protocol (`$H\NDI.htm`)
Video in YUV 4:2:2, compressed with SpeedHQ (CPU), 8 or 16 bits, optional alpha. No FPS or resolution limit beyond the hardware. Discovery by mDNS. Multicast is configured **outside TouchDesigner**, by the NDI Access Manager. Metadata is XML; a DAT table is automatically converted to `<TouchDesignerFormat>`. Network recommendation: Jumbo Frames at 9014 bytes solved dropped frames at high resolutions on gigabit NIC/switch.

### DMX Out CHOP — for the scenes/cues (`$H\DMX_Out_CHOP.htm`)
`interface` (menu `serial` / `enttecusbpro` / `enttecusbpromk2` / `artnet` / `sacn` / `kinet`); `format` (menu **`packetpersample`** = each channel is a DMX address, 512 channels per universe; **`packetperchan`** = each sample of a channel is a DMX address, so 1 channel with 512 samples = 1 universe, and universes become channels); `rate` with a built-in warning: "DMX512 devices have a maximum refresh rate of 44 Hz. It is recommended that Rate <= 44".
Network page: `net` (0-127), `subnet` (0-15), `universe` (0-15) for Art-Net; `multicast` for sACN ("builds the IP automatically from Net, Subnet and Universe"); `netaddress` (default `255.255.255.255` = broadcast); `localaddress` (picks the NIC); `localport` (-1 = the OS chooses); `customport` + `netport` (default 6454); `sendartsync` + `artsynctimeout` (ms — waits for all the ArtDmx to go out before the ArtSync; if the timeout expires **the ArtSync is not sent** and a new frame begins); `cid` (unique sender ID), `source` (name assigned by the user, informational), `priority` (priority when there are multiple sources) — the three sACN ones.
There is also a **Routing Table in a DAT**, where each row is a channel and specifies net, subnet and universe.

## File

Three native types (`$H\File_Types.htm`): `.toe` = the whole project (TOuch Environment file); `.tox` = a saved component, "for reuse and portability of component libraries"; `.tog` = geometry in native format.

`.toe` is binary, but **there is an official converter to text**: `toeexpand` expands the `.toe` into a collection of readable ASCII files, and `toecollapse` reverses it; both in `C:\Program Files\Derivative\TouchDesigner\bin` (`$H\Toeexpand.htm`).

What the Lock flag keeps: "the data the node emits is frozen in memory **and saved in the `.toe` `.tox`**" (`$H\Flag.htm`). That is, the file carries cooked data, not just the description of the network.

**COMP externalization** — Common page parameters of any COMP (`$H\Base_COMP.htm`):
`externaltox` (path of the `.tox` on disk that supplies the COMP content when the `.toe` opens — "allows components to contain networks that can be updated independently"; if the `.tox` is not found, it loads what was in the `.toe`); `enableexternaltox` (on by default; turning it off loads from the `.toe` and leaves reloading manual); `enableexternaltoxpulse` (reload now); `reloadcustom` and `reloadbuiltin` (whether the top-level parameters go back to the `.tox` value on reload — **the parameters of the nodes inside always go back**); `savebackup` (keeps a copy inside the `.toe` in case the `.tox` disappears or the project runs on another machine); `subcompname` (goes into the `.tox` and promotes an internal COMP to top level); `relpath` (menu: paths relative to the `.toe`, to the `.tox`, or inherited from the parent).

**DAT externalization** — `Text_DAT.htm`: `file` (path); `syncfile` (**Sync to File**: "the file is monitored, so any change made to the file updates the DAT, and any change made to the DAT is written to the file immediately"; if the file does not exist, it is created; if it is removed, the DAT keeps the content); `loadonstart` + `loadonstartpulse`; `write` (Write on Toe Save) + `writepulse`; `language`; `extension` (extension exposed to external editors).

**Parameter JSON schema**: see the Parameters section above. `TDJSON.py` also serializes **Sequence blocks** separately, with `blockParNames`, `numBlocks` and a `blocks` dictionary indexed by the string of the index, each block keeping `val`, `expr`, `bindExpr`, `mode`, `readOnly`, `enable` per base parameter (`TDJSON.py:224-248`). `LISTATTRS` (`TDJSON.py:23-27`) is the set of attributes that become a list when the parameter is a tuplet.

## What Spellcaster should copy

- **The four parameter modes stored at the same time, with a "there is something here" indicator** (`$H\Parameter_Mode.htm`). A scene value stores the constant, the expression and the binding simultaneously; switching does not destroy the other, and the button of the non-active mode shows the small square in the corner when it has content. That is exactly what a lighting console lacks: being able to take a fixture out of timeline control to test a fixed value at 11 p.m. and give it back afterwards without reprogramming. Benefits **DMX scenes/cues** and the **orchestrator**.
- **The `min`/`max` (hard clamp) versus `normMin`/`normMax` (slider range only) distinction** (`TDJSON.py:22`, `$H\Custom_Parameters.htm`). A dimmer has a fixed 0-255 range but the useful slider can be 0-180 in rehearsal. Two properties, not one. Benefits **DMX scenes/cues** and the **ILDA player** (kpps has the galvo's physical limit and the show's working range).
- **Value Ladder instead of a thin slider** (`$H\Value_Ladder.htm`, `TouchDesignerTips.txt:16`): hold and pick the order of magnitude vertically, then drag horizontally. It solves the real console problem — the same field needs 0.01 precision and 100 of travel without switching widget and without a 3 px target. Benefits **ILDA player** (kpps, scale, rotation) and **NDI→ILDA**.
- **Ladder on the label moves the group, ladder on the field moves the component** (`$H\Parameter_Dialog.htm`). XY, XYZ, RGB. One gesture, two ranges. Benefits **ILDA player** (X/Y scale together) and **interactive set** (position of a fixture in the scale model).
- **The debug channel with per-point state** (`$H\Laser_CHOP.htm`, `debugchan`, values -1 to 7). Spellcaster should emit the same thing: one state per ILDA output sample (frame-start, color, corner hold, pre/post blank on, blanking, pre/post blank off). It is what turns "the trace has a tail" into a diagnosis. Benefits **ILDA player**, **NDI→ILDA** and is the raw material of the **Aprendiz** warning.
- **The six NDI In info channels as the health panel of the link** (`$H\NDI_In_TOP.htm`): `connected`, `receive_fps`, `num_source`, `queue_size`, `received_frames`, `missed_frames`. `missed_frames` rising is the **Aprendiz** sentence; `receive_fps` measured against the configured kpps is the calculation of whether the galvo keeps up. Benefits **NDI→ILDA** and the **Aprendiz**.
- **Error and warning as a number read from the node itself, not only as paint** (`warnings`, `errors` in the Common Operator Info Channels, `$H\Laser_Device_CHOP.htm`). If every Graph node exposes error and warning counts as data, the state panel, the log, the CLI and the **Aprendiz** read the same source, and `spell` over SSH shows the same as the GUI. Benefits **orchestrator** and **Aprendiz**.
- **The split between application shortcut and panel shortcut, with the transport requiring Shift inside the panel** (`Config\PanelShortcuts.txt`, `$H\Application_Shortcuts.htm`). In Perform Mode, `Space` alone is disabled and only `Shift+Space` pauses. That protects the show from an accidental space — and the current `SHORTCUTS.md` does not have that layer: `Space` is play/pause everywhere. Worth copying the pair of tables: one for the editor, another for the `performance` Face. Benefits **DMX scenes/cues** and all live functions.
- **Disabling a shortcut by deleting the command, not by removing the line** (`$H\Application_Shortcuts.htm`). The key map stays complete and auditable even with keys turned off; the user override is a file with the same format as the default. It fits straight into the `"keys": {...}` of `config.json`. Benefits all.
- **Externalization with an embedded backup** (`$H\Base_COMP.htm`, `savebackup`). A `.spell` that references external modules needs to carry a safety copy of each one, otherwise the show dies on a USB stick that does not have the folder. And explicit `relpath`: relative to the show or to the module, a declared choice. Benefits **orchestrator** and Windows→Pi portability.
- **The transport state ladder with a red "pending"** (`Config\MiscColors`: `PlayBarOnColor` green, `PlayBarOffColor` black, `PlayBarResetColor` yellow, `PlayBarPending` red / `PlayBarNoPending` green). Four states, not two: stopped, playing, reset, and **unapplied change**. `PRINCIPIOS.md §2` already asks for accent = state; "pending" is the state missing from the list (armed, live, GO, rehearsal, error). Benefits **DMX scenes/cues**.

## What NOT to copy

- **The Palette as a panel that opens and closes to "gain space"** (`$H\Palette.htm`, `TouchDesignerTips.txt:13`). If the module library has to disappear for the work to fit on the screen, the layout is wrong. It contradicts `PRINCIPIOS.md §3` (nothing that moves on its own, the operator finds it with eyes closed).
- **The Parameter Dialog in three different places at the same time** (inside the network, floating, and as a Pane — `$H\Parameter_Dialog.htm`), with a Sticky button allowing several open and "the top one is the current one". Three places for the same information is three places to look at 11 p.m. One Inspector, fixed position (`SHORTCUTS.md`: Inspector on the right, à la Resolve).
- **Desaturated family color in seven hues** (`Config\TouchColors:1-14`). Seven permanent colors on the screen, all on all the time, kills the single-accent rule: when everything is colored, nothing is state. `PRINCIPIOS.md §2` forbids that. The information "which family is this node" should be shape or label, not color.
- **Parameter name limited to 10-12 characters because the UI does not fit it** (`$H\Custom_Parameters.htm`: "keep it below 12, preferably 10, because they become unreadable when you open the parameter with the + icon"). It is a layout restriction turning into a vocabulary restriction. It contradicts `PRINCIPIOS.md §4` — the name in the GUI has to be the registry name, in full.
- **A custom name forced to start with a capital, under penalty of an error on creation** (`$H\Custom_Parameters.htm`). A capitalization convention carrying semantic information (native vs custom) is the thing that breaks when the user types fast. If the distinction matters, it is a field, not a capital letter.
- **Two shortcut tables with the same label meaning different keys** — `general.pause` exists in `TouchShortcuts.txt` and in `PanelShortcuts.txt` with a different mapping. Copying the *concept* (separate contexts) yes; copying the *name collision* no. Name the contexts in the label itself.
- **Shortcut labels that lie about what they do**: `TouchShortcuts.txt` carries `general.forward left` and `general.backward right`, while `$H\Application_Shortcuts.htm` says the right arrow advances one frame. One of the two has been wrong forever and nobody fixed it. If the key map is documentation, it needs a test that compares it against the behavior.
- **A global Timeline that changes color according to the scoped Component Time** (`$H\Timeline.htm`, `$H\Component_Timeline.htm`). The idea of scoping a subcomponent's time in the main transport is good; using **color** to say which one is scoped is not, because color is reserved for show state. Scope is text: the Timepath is already there.
- **Laser safety entirely outside the software.** Neither the Laser CHOP nor the Laser Device CHOP has a safe zone, a point limit or a watchdog; the only barrier is the all-caps warning on the page and the masking done by the AVB2ILDA hardware or by Pangolin Beyond (`$H\Laser_CHOP.htm`, `$H\Lasers.htm`). Spellcaster must not inherit that omission: shutter closed by default, a blackout that really cuts, and a zone limit in the engine — not only in a third party's DAC.
- **Implicit blanking from "all scales at zero"** (`$H\Laser_Device_CHOP.htm`). A shutter closed by arithmetic coincidence of three color parameters is a state that appears nowhere in the UI. The shutter is a named state, with its own indicator.
- **A silent contract change between versions**: in 2025.30000 the Laser CHOP began always generating at 192000 and resampling, and `Start Point Hold Time` and `Input Rate` disappeared — old files open and produce different output, with a single warning on the wiki page (`$H\Laser_CHOP.htm`). A `.spell` file that opens and plays differently is worse than one that refuses to open. Version the format and fail loudly.
