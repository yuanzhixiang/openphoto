# smart_guides.rs: Smart Guides

## Responsibilities

View › Show › Smart Guides while the Move tool (or Free Transform) drags, and while the marquees and shape tools place points: magenta lines where the moved pixels' edges or center line up with the canvas's or another layer's, and the move pulled into line. Observed in Photoshop 2026 (the probe document's layer dragged back near its place: lines along both edges and the center lines, 1 pt).

## Behavior

- When a Move drag starts (`snap::begin`) and both Extras and Show › Smart Guides (`ViewOptions::smart_guides`, on by default; also the Properties panel's Guides toggle) are on, `SmartGuides::begin` takes the moved box (`op_core::transform::bounds`: the selection's or the moved layers' pixels) and the boxes it can line up with: the canvas, then every other visible layer's non-transparent pixels (not the background, not the layers being moved).
- Each frame (`align_with`): on each axis, the pull that brings the moved box's left edge, center or right edge (top, middle, bottom) onto another box's edge or center within the snap distance (`snap::tolerance`, 8 screen points). Against View › Snap's own pull (`snap::offset`) the smaller pull wins per axis; with no smart pull the plain one stays. Holding Control turns the smart guides off for the moment, as it does snapping. Smart Guides work whether View › Snap is on or not.
- The lines kept for drawing are every edge or center that matches exactly where the move lands: vertical lines spanning from the higher of the two boxes' tops to the lower bottom, horizontal ones across both boxes. They show only while the drag lasts.
- **Points:** a drag that places points (the marquees, the shape tools, the rulers' origin) starts `SmartGuides::for_points`. Its box is a single point, lined up with the canvas's and every visible layer's edges and centers (the layers being drawn on included). `snap::point` pulls the point and draws the lines through it, the smaller of the smart and the plain pull winning per axis. The lines go when the button is let go.
- **Distance labels:** while a box (not a point) is moved, `labels` holds the gap to the nearest other layer on each side whose extent overlaps the box across (left, right, above, below; not the canvas). Each gap is drawn as a magenta line between the two boxes, at the middle of their overlap, with its length ("130 px") in white on a magenta tag at its middle.
- **Equal spacing:** a moved box between two other boxes that overlap it across, on the same axis, is also pulled to the place leaving equal gaps on both sides, within the snap distance (`equal_spacing`). This pull wins if no edge pull applies or if it is nearer. The two equal gaps then show as labels.
- **⌘-hover distances** (`measure`, `DocState::measure`): with the Move tool, ⌘ held, nothing being dragged, Extras and Smart Guides on, each frame measures from the active layer's pixels (or the selection) to the topmost other visible layer whose pixel is under the pointer, as gap labels. Over no layer, it gives the four distances to the canvas's edges, from the middle of each side. These labels are drawn like the moving labels.
- Drawing (`draw`): 1 pt lines in Photoshop's default Magenta as its screen shows it (`#eb59f7`), mapped from document space (they turn and flip with the view).

## Known limitations

- Smart Guides don't work for the Path tools. Equal spacing is only between two neighbors (not chains of equal gaps). ⌘-hover shows gaps only when the boxes don't overlap on that axis (Photoshop also measures inside overlapping boxes).
- Guide color settings (Preferences) are not available.

## Test coverage

- `edges_and_centers_line_up` (unit test): a center pulled onto the canvas center, an edge onto another layer's edge, the nearer of two candidates winning, lines spanning both boxes, nothing near meaning no pull and no lines.
- `equal_spacing_between_two_boxes` (unit test): a box 2 px short of the middle between two others lands there, with two 20 px labels.
- `gaps_and_points` (unit test): a moved box 20 px left of another layer gets a 20 px label; a point lines up with an edge and has no labels.
- `smart_guides_line_up_a_moved_layer` (UI test): see `ui_tests.md`.
- `command_hover_measures_to_another_layer` (UI test): ⌘ over another square shows the 150 px gap, over nothing the four distances to the canvas, and nothing without ⌘. Screenshot `command_measure.png`.
- `smart_guides_for_marquees_and_distance_labels` (UI test): with Snap off, a marquee ended 3 px off a layer's center line ends on it; moving a square 20 px shows the 130 px gap to a second square. Screenshot `smart_guides_labels.png`.
