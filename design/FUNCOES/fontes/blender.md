# Blender 4.5 — editor organization audit

Direct reading of the files installed in `C:\Program Files\Blender Foundation\Blender 4.5\4.5\scripts\`.
Everything below has `path:line`. What was not read is not here.

Two path roots, abbreviated in the rest of the document:

- `bl_ui/` = `scripts/startup/bl_ui/`
- `keymap/` = `scripts/presets/keyconfig/keymap_data/blender_default.py`

## Sources read

| File | Lines | What was read |
|---|---|---|
| `bl_ui/space_sequencer.py` | 3243 | 131-242 (headers), 388-560 (View), 955-1010, 1100-1197 (Strip), 1461-1500 (mix-ins), 1586-1614 (panel) |
| `bl_ui/space_dopesheet.py` | 1024 | 31-130 (filters), 373-396 (menus), 534-572 (Channel), 591-641 (Key), 813-864 (channel context) |
| `bl_ui/space_graph.py` | 578 | 30-64 (header), 155-168 (menus) |
| `bl_ui/space_nla.py` | 425 | 26-55 (header), 97-111 (menus), 384-399 (track context) |
| `bl_ui/space_node.py` | 1234 | 66-95 (header), 257-300 (menus/Add), 369-425 (Node), 577-599 (Show/Hide) |
| `bl_ui/space_outliner.py` | 559 | 39-68 (header), 99-110 (menus), 219-238 (Visibility), 400-495 (Filter) |
| `bl_ui/space_time.py` | 347 | 1-120 (transport and menus), 232-280 (Playback) |
| `bl_ui/space_topbar.py` | 856 | 20-60 (topbar), 106-126 (menus), 495-520 (Edit), 632-652 (workspace) |
| `bl_ui/space_statusbar.py` | 41 | whole file; `bl_ui/utils.py` (66) whole file |
| `bl_ui/space_info.py`, `space_text.py`, `space_console.py`, `space_spreadsheet.py` | — | `MT_editor_menus` blocks; `INFO_MT_area` in `space_info.py:20-107` |
| `bl_ui/space_properties.py`, `space_userpref.py`, `__init__.py` | — | 10-134; 314-319, 588-628, 1095-1133, 1780-1810; 68-96 |
| `bl_ui/properties_data_bone.py`, `properties_grease_pencil_common.py`, `properties_output.py`, `properties_render.py`, `asset_shelf.py` | — | 268-300 (solo); 675-690; 10-45 (presets); `use_property_decorate` lines |
| `bl_operators/presets.py`, `bl_operators/wm.py` | — | 73-130 (`AddPresetBase`); 32-92 (RNA path lookup) |
| `keymap/` (`blender_default.py`) | 6000+ | 316-360, 410-470, 562-575, 676-900, 1002-1050, 1132-1170, 1831-1990, 2121-2260, 2503-2640, 2668-2700, 2950-3110, 3624-3790 |
| `scripts/presets/interface_theme/Blender_Light.xml` | 1675 | 322-341 (State), 830-868 (Sequencer) |

Note: `Blender_Dark.xml` has 6 lines and is empty (`presets/interface_theme/Blender_Dark.xml:1-6`) — the dark theme is the built-in one in C, not a preset. Only the Light one exposes the values.

## Anatomy of an editor

An editor (`Space`) is a set of regions. The region is chosen by `bl_region_type` in the class:

- `HEADER` — header. It is the default when `Header` only declares `bl_space_type` (`bl_ui/space_sequencer.py:163-164`).
- `TOOL_HEADER` — second bar, only with the active tool's options (`bl_ui/space_sequencer.py:137-139`).
- `UI` — sidebar, N key (`bl_ui/space_sequencer.py:131-134`).
- `TOOLS` — toolbar, T key.
- `CHANNELS` — the channel/track column to the left of the timeline (`keymap/:3262` defines the "Sequencer Channels" keymap).
- `WINDOW` — the main region.
- `HUD` — the floating "Adjust Last Operation" footer (`bl_ui/space_sequencer.py:471`).
- `NAVIGATION_BAR` — the Properties icon column (`bl_ui/space_properties.py:62-64`).
- `PREVIEW` — the preview area when the editor has two views.

Rules that repeat in every editor:

1. **The View menu starts by toggling the regions on and off, in this order**: toolbar, sidebar, tool header, HUD, channels (`bl_ui/space_sequencer.py:467-473`). The user always finds "where did the bar go" in the same place.
2. **The View menu ends with `INFO_MT_area`** (`bl_ui/space_sequencer.py:546`), which is the same everywhere: quadview, Horizontal Split, Vertical Split, separator, screen_full_area, Toggle Fullscreen Area, area_dupli, separator, area_close (`bl_ui/space_info.py:67-89`).
3. **The header always follows the same sequence**: `layout.template_header()` → space mode selector (`st.view_type`) → `MT_editor_menus.draw_collapsible` → `separator_spacer()` → tool settings → `separator_spacer()` → display toggles with popover (`bl_ui/space_sequencer.py:166-215`).
4. **Toggle+popover idiom**: a `row` with the boolean and a `sub` whose `sub.active` is bound to the boolean, containing the `popover` with the details (`bl_ui/space_sequencer.py:206-215`; `bl_ui/space_time.py:21-28`; `bl_ui/space_graph.py:36-40`). The button switches it on; the little arrow next to it opens the adjustment panel. Never two separate buttons.
5. **The sidebar is split into tabs** by `bl_category`. In the sequencer: Strip, View, Tool, Cache, Proxy, Modifiers. In the node editor: Item, Tool, Node, Group, Options.

### Menu order per editor

| Editor | Menus, in the order they are drawn | Source |
|---|---|---|
| Sequencer | View, Select, [Marker], [Add], Strip, [Image] | `bl_ui/space_sequencer.py:225-238` |
| Dope Sheet | View, Select, [Marker], Channel, Key, [Action] | `bl_ui/space_dopesheet.py:381-394` |
| Graph | View, Select, [Marker], Channel, Key | `bl_ui/space_graph.py:162-167` |
| NLA | View, Select, [Marker], Add, Tracks, Strips | `bl_ui/space_nla.py:104-110` |
| Node | View, Select, Add, Node | `bl_ui/space_node.py:263-266` |
| Text | View, Text, [Edit, Select, Format], Templates | `bl_ui/space_text.py:96-104` |
| Console | View, Console | `bl_ui/space_console.py:25-26` |
| Info | View, Info | `bl_ui/space_info.py:26-27` |
| Spreadsheet | View | `bl_ui/space_spreadsheet.py:51` |
| Outliner | none (only in DATA_API mode) | `bl_ui/space_outliner.py:107-108` |
| Topbar | Blender, File, Edit, Render, Window, Help | `bl_ui/space_topbar.py:115-125` |

The law: **View first, Select second, Add third, and last the menu named after the editor's object** (Strip, Node, Key/Channel, Tracks/Strips). Menus in brackets only appear if the state allows it — Marker disappears if `st.show_markers` is false (`bl_ui/space_sequencer.py:232-233`), Action disappears if there is no action (`bl_ui/space_dopesheet.py:393-394`).

### The Timeline as a transport bar case

The Timeline header is a single `row(align=True)` with six buttons and nothing else: rewind, previous keyframe, reverse play, play, next keyframe, fast-forward (`bl_ui/space_time.py:31-49`). While playing, the two play buttons become a single Pause with `row.scale_x = 2` — the button that matters gets physically bigger (`bl_ui/space_time.py:44-47`). Then comes the current frame field and the Start/End pair, which switches to the preview range fields when the toggle is on (`bl_ui/space_time.py:53-70`).

The Timeline menus are not menus: Playback and Keying are popovers, and only then come View and Marker (`bl_ui/space_time.py:88-105`).

## Objects and verbs

Each editor has ONE noun and a menu named after it. Verbs are operators named `domain.verb`.

### Sequencer — the `strip` object

`SEQUENCER_MT_strip` (`bl_ui/space_sequencer.py:1100-1197`), in order: Transform, [Duplicate/Show-Hide/Text in the preview], Retiming, Split and Hold Split, Copy/Paste/Duplicate, Delete, Add Modifier / Copy Modifiers, submenu by type (Effect, Movie, Image, Meta), Color Tag, Lock/Mute, Connect/Disconnect, Inputs.

Sub-blocks that matter:

- **Lock/Mute** (`bl_ui/space_sequencer.py:990-1004`): lock, unlock, separator, mute, unmute, "Mute Unselected Strips", "Unmute Deselected Strips". Lock and mute are different things and sit in separate blocks.
- **Show/Hide** in the preview (`bl_ui/space_sequencer.py:955-964`): "Show Hidden Strips" first, then "Hide Selected" and "Hide Unselected". Revealing comes before hiding.
- **Meta** = group strips into a navigable container: `meta_make`, `meta_separate`, `meta_toggle` (`bl_ui/space_sequencer.py:1177-1183`). Toggle enters and leaves the group with Tab (`keymap/:3055`).
- **Connect/Disconnect** (`bl_ui/space_sequencer.py:1191-1192`): a link between strips that makes selection and movement travel together, and that can be ignored by holding Alt (`keymap/:3005-3009`).

### Dope Sheet / Graph — the `channel` and `keyframe` objects

`DOPESHEET_MT_channel` (`bl_ui/space_dopesheet.py:534-572`): delete, clean, group/ungroup, `channels_setting_toggle` / `_enable` / `_disable` (all as `operator_menu_enum`, that is, mute and protect are *enum values* of one and the same verb, not three buttons), editable_toggle, extrapolation, expand/collapse, move, fcurves_enable, bake, view_selected.

The channel context menu names those enums in human text (`bl_ui/space_dopesheet.py:827-831`):
`Mute Channels`/`Unmute Channels` = `type='MUTE'`; `Protect Channels`/`Unprotect Channels` = `type='PROTECT'`. That is, the channel's state vocabulary is **mute** and **protect** (lock), and there is no solo here.

`DOPESHEET_MT_key` (`bl_ui/space_dopesheet.py:591-629`): Transform, Snap, Mirror, Keyframe Insert, Frame Jump, Copy/Paste/Paste Flipped/Duplicate/Delete, and then **four separate enums**: Keyframe Type, Handle Type, Interpolation Mode, Easing Mode. Then Clean, Bake, Euler Filter.

The keyframe's Transform submenu has four distinct modes of moving in time: Move (`TIME_TRANSLATE`), Extend (`TIME_EXTEND`), Slide (`TIME_SLIDE`), Scale (`TIME_SCALE`) (`bl_ui/space_dopesheet.py:638-641`).

Channel filters appear at two levels: three icon-only toggles straight in the header — `show_only_selected`, `show_hidden`, `show_only_errors` (`bl_ui/space_dopesheet.py:36-45`) — and the complete set in a popover with a text field and a "Filter by Type" grid (`bl_ui/space_dopesheet.py:54-130`).

### Node editor — the `node`, `socket`, `link`, `frame`, `group`, `reroute` objects

`NODE_MT_node` (`bl_ui/space_node.py:369-425`), in order: transform, copy/paste/duplicate/duplicate linked, delete and `delete_reconnect`, "Join in New Frame" / "Remove from Frame", Rename (via `wm.call_panel` of `TOPBAR_PT_name`, `bl_ui/space_node.py:401-403`), link block (`link_make`, "Make and Replace Links", `links_cut`, `links_detach`, `links_mute`), group block (`group_make`, "Insert Into Group", `group_edit`, `group_ungroup`), Show/Hide.

The node's `Show/Hide` (`bl_ui/space_node.py:577-599`): Mute, Node Preview, Node Options, separator, Unconnected Sockets, Collapse, Collapse and Hide Unused. That is, the node has four levels of "hiding": muted (passes straight through), no preview, no options, collapsed.

`node.links_mute` is the verb that **switches a cable off without deleting it** (`bl_ui/space_node.py:410`, shortcut Ctrl+Alt+right-button drag in `keymap/:2196`). It is the closest thing to a route bypass.

### Outliner — the `collection` object and the restriction columns

The toggle columns are configurable and the list changes with the mode (`bl_ui/space_outliner.py:412-429`):

- View Layer mode: `enable`, `select`, `hide`, `viewport`, `render`, `holdout`, `indirect_only` — seven columns, in this order.
- Scenes mode: only `select`, `hide`, `viewport`, `render`.

The collection's Visibility menu (`bl_ui/space_outliner.py:219-238`): **Isolate** first, on its own; then Show / Show All Inside / Hide / Hide All Inside; then "Enable in Viewports" / "Disable in Viewports". Isolate is the Outliner's solo.

### Solo, and what it does to the other toggles

The only solo written in Python is in the bone collections (`bl_ui/properties_data_bone.py:286-295`):

```python
sub_visible = row.row(align=True)
sub_visible.active = (not is_solo_active) and bcoll.is_visible_ancestors
sub_visible.prop(bcoll, "is_visible", text="", icon='HIDE_OFF' if bcoll.is_visible else 'HIDE_ON')
row.prop(bcoll, "is_solo", text="", icon='SOLO_ON' if bcoll.is_solo else 'SOLO_OFF')
```

Three things: (a) the toggle icon changes with the state, it is not a checkbox; (b) while solo is active the visibility toggle goes **dimmed but present** — it does not vanish, it is not removed; (c) solo comes after visibility in the row, and the destructive action (unassign, X icon) is set apart by a spacer (`bl_ui/properties_data_bone.py:298-300`).

## States

### The theme has a block just for state

`USERPREF_PT_theme_interface_state` (`bl_ui/space_userpref.py:1095-1133`) lists, in this order: `error`, `warning`, `info`, `success`, then `inner_anim`, `inner_driven`, `inner_key`, `inner_overridden`, `inner_changed` — each with a `_sel` pair for when the item is selected — and finally `blend`.

Actual values (`presets/interface_theme/Blender_Light.xml:322-340`):

| State | Color | Meaning |
|---|---|---|
| `error` | `#771111` | operation failed |
| `warning` | `#ac8737` | trouble ahead |
| `info` | `#28487d` | neutral notice |
| `success` | `#188625` | finished well |
| `inner_anim` | `#73be4c` green | field is animated, but the current frame has no key |
| `inner_key` | `#f0eb64` yellow | there is a keyframe exactly on this frame |
| `inner_driven` | `#b400ff` purple | value comes from a driver, not from the user |
| `inner_overridden` | `#6bf3cc` cyan | library override |
| `inner_changed` | `#cc7529` orange | different from the default |
| `blend` | `0.5` | blend factor |

The detail that matters most: `blend` exists. **The state color is neither a border nor an icon — it is a tint mixed into the background of the field itself.** The widget stays the widget; it only changes color. And `_sel` exists because the color has to stay legible when the item is selected.

### The per-property decorator

Every property row can carry, on the right, an animation widget. It is switched on with `layout.use_property_decorate`, and panels of non-animatable data switch it off with an explicit comment: `layout.use_property_decorate = False  # No animation.` (`bl_ui/properties_render.py:64`; `bl_ui/asset_shelf.py:20`; `bl_ui/space_time.py:240`). That is: the affordance for "this can be animated" is structural, it shows up on every animatable property, and it is the exception that has to be declared.

Its companion is `layout.use_property_split = True` (`bl_ui/space_sequencer.py:1604`): label on the left, value on the right, decorator column at the far right. Fixed grid, not fluid.

### Dimmed, disabled and alerted

Three distinct levels, all used in the same file:

- `layout.active = not strip.mute` (`bl_ui/space_sequencer.py:1605`) — the whole panel of a muted strip goes **dimmed but clickable**. Still editable; it is just not on air.
- `row.enabled = has_material_slots` (`bl_ui/space_node.py:82`) — gray and **not clickable**.
- `row.alert = True` (`bl_ui/space_text.py:28-31`) — the row turns red. Used when the text file was changed outside Blender and the button becomes "resolve_conflict". Also on invalid preference fields (`bl_ui/space_userpref.py:2266`, `:2294`).

### Color in the timeline: identity, not state

The sequencer theme (`presets/interface_theme/Blender_Light.xml:830-868`) gives a color **per strip type** — movie `#4d6890`, image `#8f744b`, scene `#828f50`, audio `#4c8f8f`, effect `#4c456c`, meta `#5b4d91`, text `#824c8f` — and reserves only two colors for state: `active_strip` white and `selected_strip` orange `#ff6a00`. The playhead is a single `frame_current` line `#5680c2`.

Keyframes have their own family of states with color: `keyframe`, `keyframe_breakdown`, `keyframe_movehold`, `keyframe_generated`, each with a `_selected` pair and a common border (`presets/interface_theme/Blender_Light.xml:850-859`).

### The status bar

Four things, in order, and nothing else (`bl_ui/space_statusbar.py:12-30`):

1. `template_input_status()` — what the mouse buttons do RIGHT NOW, in the current context. It is the software's learning line.
2. `template_reports_banner()` — the last error/warning message.
3. `template_running_jobs()` — progress bar for whatever is running.
4. `template_status_info()` — statistics.

The content of item 4 is a user preference, not an obligation: scene stats, scene duration, system memory, video memory, version (`bl_ui/space_userpref.py:314-319`). And if the status bar is hidden, the reports banner and the jobs banner migrate to the topbar (`bl_ui/space_topbar.py:47-50`) — the message never disappears, it only changes place.

## Shortcuts

### Modifier grammar

Extracted from the templates, which exist precisely so that no editor invents its own:

| Pattern | Rule | Source |
|---|---|---|
| Select all / none / invert | `A` / `Alt+A` / `Ctrl+I` | `keymap/:410-431` |
| Hide / hide the rest / reveal | `H` / `Shift+H` / `Alt+H` | `keymap/:451-456` |
| Context menu | primary key + `APP` (the keyboard's menu key) | `keymap/:316-321` |
| Toolbar / sidebar / channels | `T` / `N` / (per editor) via `wm.context_toggle` on `show_region_*` | `keymap/:330-357` |
| Move the playhead with the mouse | `Shift+right button` when selection is the left button | `keymap/:562-570` |

The rule behind it: **no modifier = the action; Shift = the same action on the complement or extending it; Alt = the inverse or the "clear"; Ctrl = the strong variant.** `H`/`Shift+H`/`Alt+H` (hide / hide unselected / reveal) and `H`/`Ctrl+H`/`Ctrl+Alt+H` in the sequencer (mute / lock / unlock) are the same scheme (`keymap/:3028-3037`).

### Window and screen

| Action | Key | Source |
|---|---|---|
| Menu search (command palette) | `F3` | `keymap/:758` |
| Rename active item / batch rename | `F2` / `Ctrl+F2` | `keymap/:756-757` |
| User favorites menu | `Q` | `keymap/:716` |
| New / Open / Recent / Save / Save as / Quit | `Ctrl+N` / `Ctrl+O` / `Shift+Ctrl+O` / `Ctrl+S` / `Shift+Ctrl+S` / `Ctrl+Q` | `keymap/:705-713` |
| Change the area's editor type | `Shift+F1`..`Shift+F12` | `keymap/:719-738` |
| Undo / redo | `Ctrl+Z` / `Shift+Ctrl+Z` | `keymap/:822-823` |
| Maximize area | `Ctrl+Space` | `keymap/:836` |
| Maximize without panels (fullscreen) | `Ctrl+Alt+Space` | `keymap/:837` |
| Adjust last operation | `F9` | `keymap/:839` |
| Repeat last operation | `Shift+R` | `keymap/:813` |
| Cycle editor context | `Ctrl+Tab` / `Shift+Ctrl+Tab` | `keymap/:802-805` |
| Cycle workspace | `Ctrl+PageDown` / `Ctrl+PageUp` | `keymap/:806-809` |
| Preferences | `Ctrl+,` | `keymap/:864` |

`Shift+F8` = Video Sequencer, `Shift+F12` = Dope Sheet, `Shift+F6` = Graph Editor, `Shift+F3` = Node Editor (`keymap/:725-737`). Each editor has a fixed number, and the number does not change with the layout.

### Frames and playback (keymap "Frames", valid in every window)

| Action | Key | Source |
|---|---|---|
| Previous / next frame | `←` / `→` (with repeat) | `keymap/:3634-3637` |
| Go to start / end of the range | `Shift+←` / `Shift+→` | `keymap/:3638-3641` |
| Previous / next keyframe | `↓` / `↑` | `keymap/:3642-3645` |
| Play / pause | `Space` (or `Shift+Space` if Space is search/tool) | `keymap/:3658-3665` |
| Reverse play | `Shift+Ctrl+Space` | `keymap/:3670-3671` |
| Cancel playback (returns to the frame it started from) | `Esc` | `keymap/:3690` |
| Play/stop on a media keyboard | `MEDIA_PLAY` / `MEDIA_STOP` | `keymap/:3691-3692` |
| Step frame forward/back on the wheel | `Alt+wheel` | `keymap/:3650-3653` |

Structural detail: **Space is configurable** between TOOL, SEARCH and PLAY, and the rest of the keymap rearranges itself around it (`keymap/:774-788` and `:3656-3665`). The factory default is Space = play, with search on F3.

Keymap "Animation" (`keymap/:3698-3715`): set preview range `P`, clear it `Alt+P`, `Ctrl+Home` sets the start frame, `Ctrl+End` sets the end, `Ctrl+T` toggles frames/seconds.

### Keyframe and driver from any field (keymap "User Interface")

| Action | Key | Source |
|---|---|---|
| Insert keyframe on the field under the mouse | `I` | `keymap/:1024` |
| Delete the field's keyframe | `Alt+I` | `keymap/:1026` |
| Clear all animation on the field | `Shift+Alt+I` | `keymap/:1028` |
| Add driver | `Ctrl+D` | `keymap/:1030` |
| Remove driver | `Ctrl+Alt+D` | `keymap/:1031` |
| Add to / remove from the keying set | `K` / `Alt+K` | `keymap/:1032-1033` |
| Back to the default value | `Backspace` | `keymap/:1034` |
| Copy the field's RNA data path | `Shift+Ctrl+C` | `keymap/:1020` |
| Filter the list under the cursor | `Ctrl+F` | `keymap/:1036-1037` |

This keymap is what makes Blender feel coherent: the keys work over **any** field of **any** editor, because they live in the UI keymap and not in the editor's.

### Sequencer

| Action | Key | Source |
|---|---|---|
| Split (soft) / hard split | `K` / `Shift+K` | `keymap/:3024-3027` |
| Mute selected / mute the rest | `H` / `Shift+H` | `keymap/:3028-3031` |
| Unmute / unmute the rest | `Alt+H` / `Shift+Alt+H` | `keymap/:3032-3035` |
| Lock / unlock | `Ctrl+H` / `Ctrl+Alt+H` | `keymap/:3036-3037` |
| Duplicate | `Shift+D` | `keymap/:3045` |
| Delete | `X` or `Del` | `keymap/:3048-3049` |
| Copy / paste / paste keeping offset | `Ctrl+C` / `Ctrl+V` / `Shift+Ctrl+V` | `keymap/:3050-3053` |
| Make meta / enter and leave it | `Ctrl+G` / `Tab` | `keymap/:3055-3056` |
| Frame all / selection / playhead | `Home` / `NumPad .` / `NumPad 0` | `keymap/:3058-3061` |
| Previous / next strip | `PageUp` / `PageDown` | `keymap/:3062-3069` |
| Remove gap / remove all gaps | `Backspace` / `Shift+Backspace` | `keymap/:3074-3078` |
| Snap to playhead / slip / move in time | `Shift+S` / `S` / `G` | `keymap/:3079`, `:3091`, `:3094` |
| Add menu / View pie / marker at the playhead | `Shift+A` / `` ` `` / `M` | `keymap/:3088`, `:3090`, `:3104` |
| Toolbar / sidebar | `T` / `N` | `keymap/:2959-2963` |
| Toggle Sequencer/Preview | `Ctrl+Tab` | `keymap/:2966-2967` |
| Snapping on/off | `Shift+Tab` | `keymap/:2968-2969` |

### Dope Sheet

| Action | Key | Source |
|---|---|---|
| Insert keyframe | `I` | `keymap/:2605` |
| Duplicate / delete | `Shift+D` / `X` or `Del` | `keymap/:2603-2604` |
| Copy / paste / paste flipped | `Ctrl+C` / `Ctrl+V` / `Shift+Ctrl+V` | `keymap/:2606-2609` |
| Handle type / Interpolation / Easing / Keyframe type | `V` / `T` / `Ctrl+E` / `R` | `keymap/:2595-2599` |
| Extrapolation / snap pie / mirror | `Shift+E` / `Shift+S` / `Ctrl+M` | `keymap/:2597`, `:2590-2593`, `:2594` |
| Select column (keys / current frame / markers) | `K` / `Ctrl+K` / `Shift+K` | `keymap/:2577-2586` |
| Frame all / selection / playhead | `Home` / `NumPad .` / `NumPad 0` | `keymap/:2611-2614` |
| Lock channel editing / filter by name | `Tab` / `Ctrl+F` | `keymap/:2616-2617` |
| Move / extend / scale / slide in time | `G` / `E` / `S` / `Shift+T` | `keymap/:2618-2627` |
| Marker / preview range from the selection | `M` / `Ctrl+Alt+P` | `keymap/:2630`, `:2610` |
| Go to the Graph Editor | `Ctrl+Tab` | `keymap/:2516-2517` |

### Graph Editor

Identical to the Dope Sheet wherever that makes sense (`I`, `V`, `T`, `Ctrl+E`, `Home`, `NumPad .`, `Tab`, `Ctrl+F` — `keymap/:1922-1947`), plus:

| Action | Key | Source |
|---|---|---|
| Insert keyframe by clicking on the curve | `Ctrl+click` | `keymap/:1932` |
| Smooth | `Alt+O` | `keymap/:1925` |
| Add F-modifier | `Shift+Ctrl+M` | `keymap/:1845-1846` |
| Hide curves / reveal | `H` / `Shift+H` / `Alt+H` | `keymap/:1848` |
| Smoothing / blending menu | `Alt+S` / `Alt+D` | `keymap/:1940-1941` |
| Go to the Dope Sheet | `Ctrl+Tab` | `keymap/:1849-1850` |

### Animation channels (keymap "Animation Channels")

| Action | Key | Source |
|---|---|---|
| Toggle / enable / disable a setting (mute, protect…) | `Shift+W` / `Shift+Ctrl+W` / `Alt+W` | `keymap/:3758-3760` |
| Lock/unlock editing | `Tab` | `keymap/:3761` |
| Expand / collapse | `NumPad +` / `NumPad -` | `keymap/:3763-3764` |
| Move one up/down, or to top/bottom | `PageUp`/`PageDown`, with `Shift` | `keymap/:3770-3777` |
| Group / ungroup | `Ctrl+G` / `Ctrl+Alt+G` | `keymap/:3779-3780` |
| Delete channel / rename | `X` or `Del` / double click | `keymap/:3755-3756`, `:3738` |
| Filter by name / frame selected | `Ctrl+F` / `NumPad .` | `keymap/:3744`, `:3785` |

### Node Editor

| Action | Key | Source |
|---|---|---|
| Make link / make and replace | `J` / `Shift+J` | `keymap/:2213-2216` |
| Cut / mute links / reroute (right-button drag) | `Ctrl` / `Ctrl+Alt` / `Shift` | `keymap/:2192-2196` |
| Mute node / collapse / hide unconnected sockets / preview | `M` / `H` / `Ctrl+H` / `Shift+H` | `keymap/:2227-2230` |
| Group / ungroup / separate | `Ctrl+G` / `Ctrl+Alt+G` / `P` | `keymap/:2250-2252` |
| Enter/leave the group | `Tab` / `Ctrl+Tab` | `keymap/:2253-2256` |
| Named frame around the selection / find node by name | `F` / `Ctrl+F` | `keymap/:2226`, `:2249` |
| Delete / delete and reconnect | `X` / `Ctrl+X` | `keymap/:2235-2238` |
| Frame all / selection | `Home` / `NumPad .` | `keymap/:2231-2233` |
| Toolbar / sidebar | `T` / `N` | `keymap/:2129-2133` |

`node.delete_reconnect` (`Ctrl+X`) deletes the node and **reconnects the cable across it**. There is no equivalent in the sequencer.

### Markers (keymap "Markers", valid in every timeline)

A single keymap, shared by sequencer, dope sheet, graph, NLA and timeline: create at the playhead `M` (`keymap/:1141`), duplicate `Shift+D` (`:1145`), move `G` or drag (`:1164`, `:1142`), delete `X`/`Del` (`:1159-1160`), rename `F2` or double click (`:1161-1162`). The same verbs with the same keys as everything else — the marker did not get a vocabulary of its own.

## Command palette

`wm.search_menu` is the palette, bound to `F3` (`keymap/:758`). It shows up in the Edit menu as "Menu Search..." with a magnifier icon, and next to it an "Operator Search..." that **only appears if developer mode is on** (`bl_ui/space_topbar.py:505-507`):

```python
layout.operator("wm.search_menu", text="Menu Search...", icon='VIEWZOOM')
if show_developer:
    layout.operator("wm.search_operator", text="Operator Search...")
```

That is the whole design decision in two lines: **the default search indexes the menus, not the operators.** What the user types is the label he saw in the menu, and the result shows the menu path where that thing lives. Searching by internal operator name is a developer tool.

Practical consequence for whoever writes UI: a command is only findable if it is declared in some `Menu.draw` as `layout.operator("domain.verb", text="Label")`. The `text=` is the search key. Real examples: `layout.operator("sequencer.split", text="Split")` (`bl_ui/space_sequencer.py:1133`), `layout.operator("node.link_make", text="Make and Replace Links")` (`bl_ui/space_node.py:407`).

Two variants:

- `WM_OT_search_single_menu` — searches inside ONE menu only. Used in the Add menu of the node editor and of the 3D View, which are too big to navigate (`bl_ui/space_node.py:282`; `bl_ui/space_view3d.py:2655`).
- `bl_options = {'SEARCH_ON_KEY_PRESS'}` on the menu class (`bl_ui/space_node.py:269-274`) — the menu turns into a search field as soon as the user types a letter, with no need to click "Search...".

Complements that are not the palette but solve the same problem:

- `SCREEN_MT_user_menu` on `Q` (`keymap/:716`): favorites the user assembles himself, button by button.
- `screen.redo_last` on `F9` (`keymap/:839`) and the HUD region: the last operation becomes an editable panel instead of demanding undo-and-redo.
- `ui.copy_data_path_button` on `Shift+Ctrl+C` (`keymap/:1020`): copies the RNA path of the field under the mouse, which is literally the name the script uses. The GUI teaches the API.
- Presets: any panel can become a preset menu by inheriting `PresetPanel`, which only needs `preset_subdir` and `preset_operator` (`bl_ui/utils.py:9-40`; real use in `bl_ui/properties_output.py:15-20`). The preset file is generated from a list of data paths (`bl_operators/presets.py:73-78`).

## Declarative panel (real example)

Copied whole from `bl_ui/space_sequencer.py:1586-1612`:

```python
class SEQUENCER_PT_adjust_crop(SequencerButtonsPanel, Panel):
    bl_label = "Crop"
    bl_options = {'DEFAULT_CLOSED'}
    bl_category = "Strip"

    @classmethod
    def poll(cls, context):
        if not cls.has_sequencer(context):
            return False

        strip = context.active_strip
        if not strip:
            return False

        return strip.type != 'SOUND'

    def draw(self, context):
        strip = context.active_strip
        layout = self.layout
        layout.use_property_split = True
        layout.active = not strip.mute

        col = layout.column(align=True)
        col.prop(strip.crop, "min_x")
        col.prop(strip.crop, "max_x")
        col.prop(strip.crop, "max_y")
        col.prop(strip.crop, "min_y")
```

What each piece does:

- `bl_space_type` and `bl_region_type` come from the `SequencerButtonsPanel` mix-in (`bl_ui/space_sequencer.py:1461-1471`), which also brings a base `poll`. The panel declares only what is its own.
- `bl_label` is the title and, together with `bl_category`, is what the menu search and the sidebar tab use.
- `bl_options = {'DEFAULT_CLOSED'}` — born closed. There is also `{'HIDE_HEADER'}` for a panel with no title (`bl_ui/space_time.py:277`).
- `poll` decides whether the panel exists in this context. Returning `False` makes the panel disappear entirely; there is no empty panel.
- `draw` is called on every redraw and holds no state. All state is in the data (`strip.crop.min_x`), never in the widget.

- Subpanel: same class plus `bl_parent_id = "SEQUENCER_PT_effect"` (`bl_ui/space_sequencer.py:1777-1780`); the nesting reaches three levels (`SEQUENCER_PT_effect` → `_effect_text_style` → `_effect_text_shadow`, `bl_ui/space_sequencer.py:1851-1855`).
- Checkbox in the panel header: a separate `draw_header` with a single `layout.prop(strip, "use_shadow", text="")` (`bl_ui/space_sequencer.py:1862-1865`).

## File

The `.blend` stores the UI — workspaces, screens, areas and the state of the regions — and that is optional on open. The evidence on the Python side is the `use_load_ui` preference, listed under the "Default To" heading together with relative paths and compression (`bl_ui/space_userpref.py:1799-1803`). That is: the file always carries the UI inside it, and the user decides whether, on opening, he wants the file's UI or the one already on screen.

Workspaces are complete datablocks: they have add, duplicate, delete, reorder to front/back operators (`bl_ui/space_topbar.py:632-652`), and appear as tabs via `layout.template_ID_tabs(window, "workspace", ...)` in the topbar (`bl_ui/space_topbar.py:36`). In fullscreen the tab disappears and becomes a "Back to Previous" (`bl_ui/space_topbar.py:37-38`).

The C structure is not in this script tree: `bl_ui/__init__.py:68-96` only registers the UI modules; there is nothing in Python about serialization of `bScreen`/`ScrArea`. What can be asserted from reading these files is what is above.

## What Spellcaster should copy

- **Fixed menu order per panel: View, Select, Add, <the panel's noun>** (`bl_ui/space_sequencer.py:225-238`, `bl_ui/space_node.py:263-266`). The orchestrator gets `View / Select / Add / Module`; the cue timeline gets `View / Select / Add / Cue`; the ILDA player gets `View / Select / Add / Frame`. Whoever learns one panel navigates the others without reading anything. Covers `design/PRINCIPIOS.md §4`.
- **The View menu opens with the region toggles and closes with the area menu** (`bl_ui/space_sequencer.py:467-473`, `bl_ui/space_info.py:67-89`). "Where did the Inspector go" and "I want this window fullscreen" are always in the same place, in every Face. It helps the operator of `PRINCIPIOS.md` at 11pm, and fits `Shift+1..7` and the Ctrl+backtick already defined in `design/SHORTCUTS.md`.
- **One global UI keymap that works over any field of any panel** (`keymap/:1002-1037`): `I` inserts a keyframe on the field under the mouse, `Backspace` goes back to the default, `Shift+Ctrl+C` copies the data path. For Spellcaster: `Backspace` returns the parameter to the scene value, and the copy-data-path equivalent copies **the CLI command for that field** — which is exactly what `PRINCIPIOS.md §4` demands of every screen. It serves DMX scenes/cues, ILDA player and NDI→ILDA with no per-panel code.
- **State as tint in the field background, with a blend factor, not as a border or an icon** (`bl_ui/space_userpref.py:1095-1133`; `Blender_Light.xml:322-340`). Spellcaster has a single accent, and the same mechanism solves it: a `blend` over the gray of the field distinguishes *armed*, *live*, *fading*, *overridden by cue* without inventing a new color. And each state needs a selected/unselected pair, otherwise it disappears when the row is selected. Valid for all four themes in `design/TEMAS.md`.
- **Three distinct levels of "does not act now": `active` (dimmed, editable), `enabled` (gray, locked), `alert` (red)** (`bl_ui/space_sequencer.py:1605`; `bl_ui/space_node.py:82`; `bl_ui/space_text.py:28-31`). A universe that does not answer = `alert`. A fixture with a muted cue = `active` False, still editable. An output not armed in rehearsal mode = `enabled` False. This solves rehearsal mode for DMX scenes/cues without hiding a single control.
- **Solo dims the visibility toggle instead of hiding it** (`bl_ui/properties_data_bone.py:288-295`). In the orchestrator and in the universe patch, solo on one module dims the mute of the others but keeps them all on screen and clickable — the console does not change shape in the middle of the show. Fits `PRINCIPIOS.md §3`.
- **A command palette that indexes menu labels, with search by internal name hidden behind "developer mode"** (`bl_ui/space_topbar.py:505-507`). In Spellcaster the menu label already IS the registry name, so the palette indexes both at once, and `SEARCH_ON_KEY_PRESS` (`bl_ui/space_node.py:273`) turns any long menu — fixture list, orchestrator module list, ILDA frame list — into a search field at the first character.
- **An "adjust last operation" panel triggered by a key, instead of undo and redo** (`keymap/:839`, `HUD` region). After a GO, the operator wants to fix the fade without repeating the cue. Aprendiz is the natural home for that surface: it already knows what just happened.
- **Popovers in the header instead of dialogs**: Playback and Keying in the Timeline are popovers, not menus and not windows (`bl_ui/space_time.py:88-105`); the toggle+arrow pattern shows up in every editor (`bl_ui/space_sequencer.py:206-215`). This delivers the "zero modals for operation" of `design/SHORTCUTS.md` with a single mechanism, reusable in the ILDA player's kpps/size/color and in the Canny parameters of NDI→ILDA.
- **Two container-and-route gestures, stolen whole for the orchestrator**: `Tab` enters and leaves any container — meta-strip in the sequencer, node group in the node editor (`keymap/:3055`, `:2253`) — and works the same for a cue group, a sub-patch and a composite ILDA clip; and `delete_reconnect` (`bl_ui/space_node.py:394`, `keymap/:2237`) deletes a link **reconnecting what was on both sides**, which is exactly pulling a conversion module out of the middle of the route without dismantling the patch.

## What NOT to copy

- **The `Space` key meaning different things depending on a preference** (`keymap/:774-788`). Blender has three mutually exclusive settings for Space (tool, search, play) and the rest of the keymap rearranges itself around it. In a show, the most used key cannot depend on configuration: `Space` is play, full stop, as already set in `design/SHORTCUTS.md`.
- **Two shortcut families for the same action because of legacy** (`keymap/:844-858`, the `params.legacy` branch). Blender carries the whole 2.7x keymap as an alternative. Cost: any change has to be made twice and tested on both branches. Spellcaster has one map and remapping in `config.json`; it must not have a "legacy mode".
- **Numpad keys as part of the basic vocabulary** (`NumPad .` to frame the selection, `NumPad 0` to center on the playhead, `NumPad +/-` to expand channels — `keymap/:3060-3061`, `:3763-3764`). A laptop keyboard on the assembly table has no numpad. Framing and navigating have to live on main keys (`Home`, `=`/`-`, `Shift+Z` already chosen in `SHORTCUTS.md`).
- **`X` as the delete key** (`keymap/:2235`, `:3048`, `:3755`). It sits next to `C` and `V` and has nothing to do with the word "delete". In software where deleting a cue in the middle of the show is irreversible in practice, delete is `Delete`, and nothing else.
- **Seven restriction columns configurable by preference** (`bl_ui/space_outliner.py:412-421`). The user chooses which toggles exist, so two operators on the same version see different interfaces and the documentation does not match the screen. It contradicts `PRINCIPIOS.md §3` directly: the operator needs to find the button with his eyes closed, and he only finds it if the button was always there.
- **Spreading the same concept across `mute`, `hide`, `disable`, `enable`, `protect`, `lock` and `restrict`** (`bl_ui/space_sequencer.py:990-1004` uses lock/mute; `bl_ui/space_dopesheet.py:827-831` uses mute/protect; `bl_ui/space_outliner.py:229-238` uses show/hide/enable/disable; `bl_ui/space_node.py:586-599` uses mute/hide/collapse). Four editors, four vocabularies for "does not act now". Spellcaster has a registry: **one verb per concept, everywhere** — `mute`, `lock`, `solo`, and nothing else.
- **Editor panels defined outside the declarative layer** — the "Active F-Curve" and "Active Keyframe" tabs of the Graph Editor do not exist in Python (searching for those labels in `scripts/**/*.py` returns nothing; `bl_ui/space_graph.py` only declares header popovers). Result: an add-on cannot reorder or extend those tabs. In Spellcaster, if the Graph is the interface (`PRINCIPIOS.md §1`), then **every** panel has to be declared in the same place, with no privileged exception.
- **A Strip menu with 25 items and no search** (`bl_ui/space_sequencer.py:1100-1197`). It is what `PRINCIPIOS.md §4` explicitly forbids. Blender itself recognizes the problem and solved it only in the Add menu, with `SEARCH_ON_KEY_PRESS` (`bl_ui/space_node.py:273`) — the solution exists, but was not applied where it hurts most.
- **Icon without a label on an action with consequences**: the sequencer header has eight controls in a row that are `icon_only=True` or `text=""` (`bl_ui/space_sequencer.py:173-215`), including `overlap_mode`, which changes what happens when a strip is dropped on top of another. Text labels on everything that changes behavior, per `PRINCIPIOS.md §4`.
