# rulers.rs: Rulers, guides and grid

## Component responsibilities

The rulers (View › Rulers, ⌘R), guides (dragging out, moving, deleting, drawing) and grid (View › Show › Grid, ⌘') in the document window. Guide data is stored in the document (`op_core::Guide`, see `document.md`) and can be undone.

## Rulers

Measured on Photoshop 2026 (probe document at nine zoom levels and in every unit):

- When rulers are on, the canvas area of the document window gives up 19 pt (`RULER`) at the top and left for the rulers, with a square of the same color in the top-left corner; the canvas area shrinks accordingly (zoom, Fit on Screen and so on are based on the remaining area). Background `#474747`, no separator line on the canvas side.
- Unit (`RulerUnit`, `AppState::ruler_units`, Pixels by default): Pixels, Inches, Centimeters, Millimeters, Points, Picas, Percent (of the image's width on the top ruler, of its height on the left one). Right-clicking either ruler opens Photoshop's native menu of the seven units with the current one checked (`AppState::ruler_menu`, shown after the frame by `NativeMenu::popup_ruler_units`; a pick runs `Command::RulerUnits`).
- Ticks (`ticks`): each unit has a ladder of spacings (1, 2, 5 × 10ⁿ; inches also ½ down to ¹⁄₁₆; picas 1, 3, 6, 12, 24, 60, … and points twelve times those). The numbered major step is the smallest on the ladder at least 30 pt apart on screen; the minor step the smallest dividing it that is at least 4.75 pt apart; the medium step the largest between them that divides the major and is divided by the minor (none when there is none). For example, pixels at 100% (Retina): 100 / 50 / 10; at 200%: 50 / 10 / 5; at 800%: 10 / none / 2; inches at 100% on a 72 ppi image: 1 / ½ / ¼; points: 72 / 36 / 12.
- Ticks are one physical pixel wide, `#666666`, standing on the canvas side: major ticks the full thickness, medium 4 pt, minor 2 pt.
- Numbers (`ruler_label`): the distance from the origin in the unit, without a sign (Photoshop writes −100 as 100), whole or with up to three decimals; Source Sans 3 10.5 pt, `#9e9e9e`. On the top ruler they start 1.5 pt after the tick, centered 11.25 pt from the ruler's top, with 1.1 pt tracking (Adobe Clean's digits are wider); on the left ruler they are stacked digit by digit, centered 9.5 pt from the ruler's left, the first 5.75 pt below the tick and each next 9.75 pt lower, as in Photoshop.
- When the pointer is inside the window, each ruler marks its position with a 1 pt `#dcdcdc` line from 1.5 to 13.5 pt across the ruler.

## Ruler origin

- The origin (`DocState::ruler_origin`, document pixels, the image's top-left corner by default) is where the rulers count from; it moves with zoom and scrolling.
- Dragging out of the corner square shows a crosshair (1 point `#dcdcdc` lines across the canvas area) following the pointer, and letting go moves the origin there (snapping to the View › Snap targets, gathered as the drag starts). Double-clicking the corner puts it back at the image's top-left corner. Neither is recorded in the history; the origin is not saved with the document.
- The grid starts at the origin (`grid_lines`), as in Photoshop.

## Guides

- Drag down from the top ruler to create a horizontal guide, and right from the left ruler to create a vertical guide; the cursor over a ruler is the resize cursor for the corresponding direction. While dragging, the guide follows the pointer, snapping to the View › Snap targets (`snap.md`; Control held: no snapping); releasing inside the canvas area adds it to the document and records "New Guide", and releasing outside the area discards it.
- Moving: when the current tool is the Move tool, or while holding ⌘ (in Photoshop ⌘ temporarily switches to the Move tool), the resize cursor appears when the pointer is within 4 pt of a guide, and dragging moves it; releasing inside the canvas area records "Move Guide", and releasing after dragging outside the area (e.g. back onto a ruler) deletes it and records "Delete Guide". Guides cannot be dragged when locked (View › Guides › Lock Guides) or hidden.
- Drawing: a 1 pt cyan (`#4affff`, Photoshop's default Cyan) straight line across the entire canvas area (including the gray area outside the canvas). A guide being dragged is always drawn. The line is mapped from document space (`guide_line`), so it turns and flips with the view (Rotate View, Flip Horizontal); `guide_at` measures the pointer's distance in document space, scaled to screen points.
- Guides move with canvas changes: Canvas Size offsets them by the anchor, cropping by the crop origin, Image Size scales them proportionally, and Image Rotation and canvas flips transform them geometrically (see `image_ops.md` and `document.md` in `op-core`).

## Grid

- One major line per inch (document resolution in pixels), subdivided into 4 (Photoshop's default setting), counted from the rulers' origin, drawn only within the canvas, each line mapped end to end from document space so the grid follows the view's rotation and flip; the grid is not drawn when subdivision lines would be less than 4 pt apart on screen.
- The color is semi-transparent gray, with major lines somewhat brighter than subdivision lines.

## Display toggles

`AppState::view` (`ViewOptions`): `rulers` (off by default), `extras` (on by default), `guides` (on by default), `grid` (off by default), `lock_guides` (off by default), `smart_guides` (on by default), `pixel_grid` (on by default), `selection_edges` (on by default), `layer_edges` (off by default), and the snap switches (`snap` and the Snap To items, all on by default; see `snap.md`). Guides are shown when both Extras and Guides are on (`guides_visible`), and the grid is shown when both Extras and Grid are on (`grid_visible`); when Extras is off, the selection's marching ants, the layer edges and the pixel grid are not shown either (the selection remains in effect). Show › Selection Edges, Layer Edges and Pixel Grid each hide their own item the same way.

## Known limitations

- No smart guides, artboard guides, guide layouts, or guide color and grid settings (Preferences).
- The only ruler unit is pixels; double-clicking a ruler does not change the unit, and the origin cannot be changed by dragging the top-left corner.
- Guides are not saved to PSD files.

## Test coverage

- `ticks_match_photoshops`: the major, medium and minor steps at Photoshop's nine measured zoom levels in pixels, and at 100% in inches, centimeters, millimeters, points, picas and percent, equal Photoshop's.
- `labels_drop_the_sign`, `the_grid_starts_at_the_origin`.
- `ruler_units_and_origin` (UI test): see `ui_tests.md`. `screenshot_rulers` (`#[ignore]`) captures the rulers on the probe document for comparison with Photoshop.
