# smart_guides.rs: Smart Guides

## Responsibilities

View › Show › Smart Guides while the Move tool drags: magenta lines where the moved pixels' edges or center line up with the canvas's or another layer's, and the move pulled into line. Observed in Photoshop 2026 (the probe document's layer dragged back near its place: lines along both edges and the center lines, 1 pt).

## Behavior

- When a Move drag starts (`snap::begin`) and both Extras and Show › Smart Guides (`ViewOptions::smart_guides`, on by default; also the Properties panel's Guides toggle) are on, `SmartGuides::begin` takes the moved box (`op_core::transform::bounds`: the selection's or the moved layers' pixels) and the boxes it can line up with: the canvas, then every other visible layer's non-transparent pixels (not the background, not the layers being moved).
- Each frame (`align_with`): on each axis, the pull that brings the moved box's left edge, center or right edge (top, middle, bottom) onto another box's edge or center within the snap distance (`snap::tolerance`, 8 screen points). Against View › Snap's own pull (`snap::offset`) the smaller pull wins per axis; with no smart pull the plain one stays. Holding Control turns the smart guides off for the moment, as it does snapping. Smart Guides work whether View › Snap is on or not.
- The lines kept for drawing are every edge or center that matches exactly where the move lands: vertical lines spanning from the higher of the two boxes' tops to the lower bottom, horizontal ones across both boxes. They show only while the drag lasts.
- Drawing (`draw`): 1 pt lines in Photoshop's default Magenta as its screen shows it (`#eb59f7`), mapped from document space (they turn and flip with the view).

## Known limitations

- Smart Guides work for the Move tool only (not for shapes, the marquees, Free Transform or the Path tools), and show no distance labels or equal-spacing marks.
- Guide color settings (Preferences) are not available.

## Test coverage

- `edges_and_centers_line_up` (unit test): a center pulled onto the canvas center, an edge onto another layer's edge, the nearer of two candidates winning, lines spanning both boxes, nothing near meaning no pull and no lines.
- `smart_guides_line_up_a_moved_layer` (UI test): see `ui_tests.md`.
