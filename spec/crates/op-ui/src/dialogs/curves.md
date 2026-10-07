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
- Targeted adjustment hand at (34, 412), three eyedroppers at (158 / 188 / 218, 411.5) (drawn only); "Show Clipping" checkbox (240.5, 404.5).
- Three groups on the right: "Show Amount of:" (394, 46.5)–(547, 122.5), two radios (410, 71.25), (410, 105.25); "Grid size:" (394, 147.5)–(547, 203.5), two 25 pt buttons (416, 181), (442, 181); "Show:" (394, 228)–(547, 351), four checkboxes at x 402.5, y 246 / 272.75 / 299.5 / 326.25.
- Buttons at x 563.5–647.5: OK (45), Cancel (80), Smooth (122, disabled), Auto (164), Options... (199); "Preview" (563.5, 243.5).

## Interaction

- The Channel menu or ⌥2–⌥5 switches channels and clears the selected point.
- Curve graph: on press, if there is a point within 5 pt it is selected, otherwise a point is added at the pointer (the curve jumps to that point); while dragging, the horizontal coordinate is limited to between the two neighboring points; releasing after dragging more than 20 pt outside the graph deletes the point (end points are not deleted).
- With a point selected: arrow keys move it by 1 (Shift 10), Delete / Backspace deletes it (except end points).
- Dragging the end point sliders below moves the input values of the first and last points (not past the neighboring points).
- Auto: RGB returns to default, and each of the three channels uses its darkest and brightest 0.1% as end points.
- Enter / OK applies and records "Curves"; Esc / Cancel cancels.

## Known limitations

- Pencil mode and Smooth, the eyedroppers, targeted adjustment, the effect of Show Clipping, Options..., the gear menu and numeric input for Input/Output are not implemented.

## Test coverage

- `channels_keep_their_curves`, `pigment_shows_ink_percent`, `auto_moves_each_channels_end_points`.
- `ui_tests::curves_dialog_adds_points`, `ui_tests::levels_and_curves_edit_one_channel` (after ⌥4 only green changes). Screenshots `curves_dialog.png`, `curves_green.png`.
