# dialogs/curves.rs: Curves dialog

## Component responsibilities

The dialog for Image › Adjustments › Curves... (⌘M), rebuilt after Photoshop 2026's classic dialog (658 × 445 pt). For the algorithm see `op-core`'s `adjust.md` (`Curves`, `curve_table`).

## Data and defaults

- Four curves: RGB, Red, Green, Blue, each a list of points sorted by input, defaulting to the two end points (0, 0) and (255, 255), at most 16 points.
- Display options: Show Amount of is Light (0-255) (default) or Pigment/Ink %; Grid size is quarters (default) or tenths; under Show, Channel Overlays, Histogram, Baseline and Intersection Line are all checked by default; Show Clipping is unchecked.
- `adjustment()`: unchanged channels do not take part (empty point list).
- Preset: "Default" when unchanged, the name of the Photoshop preset the curves match (`adjust_presets::CURVES`: Color Negative, Cross Process, Darker, Increase Contrast, Lighter, Linear Contrast, Medium Contrast, Negative, Strong Contrast), otherwise "Custom". The native menu lists Default, Custom (grayed out), a separator and the presets; picking one sets all four channels' points and clears the selected point.

## Layout (Photoshop points)

- "Preset:" (11, 56.25), popup menu (56, 46)–(345, 67), gear (365.5, 56.5) (drawn only).
- Left group box (11, 87.5)–(368, 434.5), "Channel:" (30.5, 86.75), popup menu (85, 76.5)–(170, 97.5).
- Tools: point tool button (20, 107.5)–(50, 133.5) (selected, `#383838` fill), pencil (64.5, 120) (drawn only).
- Curve graph (89, 108)–(346, 365): `#454545` fill; the current channel's histogram (`#868686`); grid lines `#383838`; diagonal baseline `#808080`; other channels' changed curves drawn 1 pt in the channel color (Channel Overlays); the current curve 1.75 pt (white for RGB, otherwise the channel color); control points are 4 pt squares, the selected one filled. While dragging a point, horizontal and vertical crosshair lines through it are drawn (Intersection Line).
- Output gradient on the left (84, 108)–(87.5, 365), white at top and black at bottom; input gradient below (89, 366)–(346, 369.5), black on the left and white on the right; below it two end point sliders (tips at y 369.5). With Pigment, both gradients, the sliders and the coordinates are reversed.
- "Output:" (20.5, 339.75) and its value (20.5, 355.75); "Input:" (90, 389.75) and its value (89.5, 410.25): shows the selected point, otherwise the position under the pointer; with Pigment, shown as ink percentage (`100 − v/2.55`).
- Targeted adjustment hand at (34, 412) (drawn only), three eyedroppers at (158 / 188 / 218, 411.5) (Set Black, Gray, White Point, as in Levels); "Show Clipping" checkbox (240.5, 404.5).
- Tools at the top left: the point tool (20, 107.5)–(50, 133.5) and the pencil (49.5, 107.5)–(79.5, 133.5), the chosen one pressed (`#383838` box). The pencil turns the curves into tables (`tables`) drawn freehand: each column the pointer crosses takes its height (with straight lines between samples); the curve shows as a line without points. Smooth (enabled only with the pencil) averages the current channel's table over nine levels. Back to the point tool, each channel becomes points every 32 levels along its table (the identity's two points when straight).
- Eyedroppers: Black (White) Point moves each channel's black (white) end point to that channel's value, dropping points beyond it; Gray Point adds a point taking each channel's value to the color's mean; the composite curve returns to its default.
- A selected point's Output and Input become fields, (18, 346.5)–(62, 365.5) and (87.5, 401)–(131.5, 420) (in ink percent with Pigment); typing moves the point (Input stays between its neighbours).
- Show Clipping: while an end-point pin is dragged, the preview shows clipping instead of the result (`clipping`): with the black pin, white where no channel clips to 0 and the clipped channels' colors elsewhere (black where all do); with the white pin, black where nothing clips to 255. Releasing the pin shows the result again.
- Three groups on the right: "Show Amount of:" (394, 46.5)–(547, 122.5), two radios (410, 71.25), (410, 105.25); "Grid size:" (394, 147.5)–(547, 203.5), two 25 pt buttons (416, 181), (442, 181); "Show:" (394, 228)–(547, 351), four checkboxes at x 402.5, y 246 / 272.75 / 299.5 / 326.25.
- Buttons at x 563.5–647.5: OK (45), Cancel (80), Smooth (122, pencil only), Auto (164), Options... (199); "Preview" (563.5, 243.5).

## Interaction

- The Channel menu or ⌥2–⌥5 switches channels and clears the selected point.
- Curve graph: on press, if there is a point within 5 pt it is selected, otherwise a point is added at the pointer (the curve jumps to that point); while dragging, the horizontal coordinate is limited to between the two neighboring points; releasing after dragging more than 20 pt outside the graph deletes the point (end points are not deleted).
- With a point selected: arrow keys move it by 1 (Shift 10), Delete / Backspace deletes it (except end points).
- Dragging the end point sliders below moves the input values of the first and last points (not past the neighboring points).
- Auto: RGB returns to default, and each of the three channels uses its darkest and brightest 0.1% as end points.
- Enter / OK applies and records "Curves"; Esc / Cancel cancels.

## Known limitations

- The targeted adjustment hand, Options... and the gear menu are not implemented. With the pencil the ramps and end-point pins are not drawn.

## Test coverage

- `channels_keep_their_curves`, `pigment_shows_ink_percent`, `auto_moves_each_channels_end_points`, `photoshops_presets`, `pencil_smooth_and_show_clipping` (a drawn step becomes `CurveTables`, Smooth eases it, back to points gives nine points; Show Clipping with the black pin at 50 maps 40 to 0 and 60 to 255).
- `ui_tests::levels_and_curves_eyedroppers_sample_the_image`: Levels' white eyedropper on an orange patch turns it white (and OK keeps it); Curves' gray eyedropper makes it neutral.
- `ui_tests::curves_dialog_adds_points`, `ui_tests::levels_and_curves_edit_one_channel` (after ⌥4 only green changes). Screenshots `curves_dialog.png`, `curves_green.png`.
