# Camera in the SolidWorks standard — the manual, and what Spellcaster uses

Matheus's request (2026-09-09): *"study the camera movement and navigation commands of solidworks, find the manual describing what alt shift ctrl do when interacting with left click and middle click and the mouse scroll."*

Source: **SOLIDWORKS Design Help 2026** (help.solidworks.com), read on 2026-09-10, one URL per table row; and the **SolidWorks Quick Reference — Keyboard Shortcuts**, an official Dassault document (`SWQRCENG06060`), for the keyboard table: https://files.solidworks.com/supportfiles/Release_Notes/2007/English/quick_reference.pdf

The reason for copying SolidWorks and not Blender: the object on screen is **a device**, not a scene. Whoever operates CAD spends the day rotating a part around the point they clicked, and that is exactly the gesture the SHOW view needs. The program's other two views are not CAD — and that is why they do not get the whole map (§3).

## 1. Mouse — what the manual says

| Gesture | Effect in SolidWorks | Manual page |
|---|---|---|
| Drag with the **middle button** | Rotate View (part and assembly only) | https://help.solidworks.com/2026/english/SolidWorks/Sldworks/r_Middle_Mouse_Button.htm |
| **Middle-click** on a vertex, edge or face, then drag with the middle | rotates **around that point**, not around the screen center | https://help.solidworks.com/2026/english/SolidWorks/Sldworks/r_Middle_Mouse_Button.htm |
| **Ctrl** + drag with the middle | Pan (in an active 2D drawing the Ctrl is not needed) | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_pan_fundamentals.htm |
| **Shift** + drag with the middle | Zoom In/Out | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_zoom_in_out.htm |
| **Alt** + drag with the middle | **Roll View** — rotates the view in the screen plane, around the centroid | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_roll_view.htm |
| **Wheel** forward and backward | Zoom **at the cursor position**. With the cursor outside the graphics area, the zoom is at the model center | https://help.solidworks.com/2026/english/SolidWorks/Sldworks/r_Middle_Mouse_Button.htm |
| **Wheel**, with `View > Modify > Zoom About Screen Center` on | Zoom at the screen center instead of at the cursor | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_zoom_in_out.htm |
| **Wheel**, with `Reverse mouse wheel zoom direction` on | inverts the wheel direction | https://help.solidworks.com/2026/english/SolidWorks/sldworks/HIDD_OPTIONS_VIEW_ROTATION_display.htm |
| Drag with the **left button**, with the Rotate / Pan / Zoom tool active | the same as the middle does directly — the left one only rotates after you pick the tool in the View bar | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_rotate_view.htm |
| **Zoom to Area**: drag a box | frames the box | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_zoom_to_area.htm |
| **Right** button in the graphics area → `Rotate about scene floor` | locks the vertical axis: the model does not tip over the horizon | https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_rotate_view.htm |

Two settings the manual treats as a system option and that here become **calibration**, not a hidden constant: `Mouse speed` ("for finer control and slower rotation, move the slider to the left") and `Arrow keys` (the angular increment of the arrows), both at https://help.solidworks.com/2026/english/SolidWorks/sldworks/HIDD_OPTIONS_VIEW_ROTATION_display.htm

## 2. Keyboard — what the manual says

From the official quick reference (`quick_reference.pdf`, p. 1), verbatim in the middle column:

| Key | Effect | Origin |
|---|---|---|
| Arrows | `Rotate horizontally or vertically` (configurable increment, 15° from the factory) | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_rotate_view.htm |
| `Shift`+arrows | `Rotate 90º` | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_rotate_view.htm |
| `Alt`+arrows | `Rotate about screen center` = **roll** (the current manual: "Hold down Alt and press the left-right arrow keys") | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_roll_view.htm |
| `Ctrl`+arrows | `Pan` | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_pan_fundamentals.htm |
| `Z` / `Shift+Z` | `Zoom in/out` — **`Z` moves away, `Shift+Z` moves closer** | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_zoom_in_out.htm |
| `Ctrl+Shift+Z` | `Previous view` (undoes up to 10 view changes) | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_previous_view.htm |
| `F` | `Zoom to fit` | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_zoom_to_fit.htm |
| `Ctrl+1` … `Ctrl+7` | Front, Back, Left, Right, Top, Bottom, Isometric | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/r_standard_views_toolbar_2.htm |
| `Ctrl+8` | `Normal To` — looks perpendicular to the selected face; again, turns 180° | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/t_viewing_models_normal_to.htm |
| `Space` | opens the Orientation box (standard views and named views) | quick_reference.pdf p.1 · https://help.solidworks.com/2026/english/SolidWorks/sldworks/c_orientation_dialog_box.htm |
| `Ctrl+Space` | View Selector: the view cube; `Alt` selects the back faces | https://help.solidworks.com/2026/english/SolidWorks/sldworks/c_view_selector.htm |

## 3. The map **this** program uses

Rule 2 of this folder's `README.md` holds here: every gesture is a textual address (`cam.rotL`, `cam.fit`, `cam.front`), registered in `Bind.def`, remappable by key and by MIDI, and listed in `Bind.manifest()`. What changes per view **is not the keyboard, it is the camera law** — `CAM.mode("free" | "rear" | "inside")`, in `cam.js`.

Three views, three laws. The device is the model; the view says what you can do with it.

| Gesture | `show` = **free** (the viewer) | `rear` = **fixed** (the rear panel is the menu) | `inside` = **restricted** (lid open) |
|---|---|---|---|
| Drag middle | rotates around the clicked point | — | rotates within the limits |
| Ctrl+middle | pan | — | — |
| Shift+middle | zoom | — | — |
| Alt+middle | **roll** | — | — |
| Drag left in empty space | rotates (laptop with no middle button) | — | rotates within the limits |
| Drag left **on a part** | belongs to the part, never to the camera (the knob: §4 of `FUNCOES/README` rule 3 — a value is a type, and the value gesture is TouchDesigner's) | same | same |
| Wheel in empty space | zoom at the cursor, small step | — | — |
| Wheel over the encoder | turns the encoder | turns the encoder | — |
| Arrows / `Shift`+arrows / `Ctrl`+arrows / `Alt`+arrows | 15° · 90° · pan · roll | — | — |
| `F`, `Ctrl+1..7`, `Z`, `Shift+Z` | like SolidWorks | — | — |
| `1` `2` `3` | view switch (it is the program's, not SolidWorks's) | same | same |

**`show` — free.** The whole of SolidWorks, plus three things SolidWorks does not need to have and a laser viewer does:

- **Critical damping** instead of a fixed-factor `lerp`. `lerp(k = dt·5)` depends on the frame rate and never arrives: the camera "swims" behind the mouse and keeps moving after the button is released. A spring with `ζ = 1` (`x += v·dt ; v += (−2ω·v − ω²·(x−alvo))·dt`, `ω = 18`) reaches the target without overshooting and stops.
- **Sensitivity proportional to distance**: rotating and panning with the camera 20 cm from the device cannot move the same amount as at 3 m. It is the manual's `Mouse speed`, only automatic.
- **Limits**: `d ≥ R + folga` (R = radius of the device's box), `d ≤` half the room, `|pitch| ≤ 85°`, target inside the room. The camera never enters the device nor goes through the wall.

**`rear` — fixed.** The rear panel **is the menu**; a loose camera over a menu is the same thing as a menu that moves when you hover it. The pose is computed from the rear panel's normal, framing the 400 × 180 mm with margin, and **recomputed when the window changes size** (the framing depends on the aspect). No drag, no middle, no wheel-zoom, no arrows. The only movement is a ±2° breathing that follows the mouse, and it **does not change the distance**: it is parallax, not navigation.

**`inside` — restricted.** Orbit around the center of the optical bench, `yaw ∈ [−60°, +60°]` relative to the **view's entry pose**, `pitch ∈ [20°, 80°]`, **fixed distance**. The yaw reference is the entry pose and not the open lid's normal because the open lid points **up**: ±60° around a vertical vector limits no yaw at all. The entry pose is already the one looking at the bench through the aperture, and it is from that one that the ±60° make sense. It is the same idea as the manual's `Rotate about scene floor` (lock an axis so the model does not tip), taken to the limit: here what is locked is the whole box, so the open lid never enters the frame from behind the camera.

## 4. What stays out, and why

| From SolidWorks | Why not |
|---|---|
| `Zoom to Area` (selection box) | dragging with the left is already rotate-in-empty-space; giving two functions to the same button asks for a modal tool, and a modal tool on top of a device is exactly what this program is not |
| `Previous view` (`Ctrl+Shift+Z`) with a stack of 10 | the program's views are three, named, with a key: `1`, `2`, `3`. A stack of anonymous views on top of that is hidden state |
| `Space` = Orientation box, `Ctrl+Space` = View Selector | `Space` is already play (rule 11 of `FUNCOES/README`: "Space is play, full stop"). The standard views stay on `Ctrl+1..7` |
| `Ctrl+8` Normal To | it needs a selected face, and here clicking a part opens its screen, it does not select it |
| `Zoom About Screen Center` as an option | one option fewer: the wheel always goes to the cursor (the ponytail rule — no config for a value that does not change) |
| `Rotate about scene floor` as a toggle | the limited `pitch` already prevents tipping, in all three views, with no toggle |
| Trimetric / Dimetric | `Ctrl+7` isometric is enough |
| `Reverse mouse wheel zoom direction` | **it stays**, and stays on by default (`cam.reverse`, persisted in `localStorage`), because it is the only option in the manual that exists precisely because half the people want the opposite of the other half |
