# rulers.rs: Rulers, guides and grid

## Component responsibilities

The rulers (View › Rulers, ⌘R), guides (dragging out, moving, deleting, drawing) and grid (View › Show › Grid, ⌘') in the document window. Guide data is stored in the document (`op_core::Guide`, see `document.md`) and can be undone.

## Rulers

- When rulers are on, the canvas area of the document window gives up 16 pt (`RULER`) at the top and left for the rulers, with a square of the same color in the top-left corner; the canvas area shrinks accordingly (zoom, Fit on Screen and so on are based on the remaining area).
- The unit is pixels, with the origin at the top-left corner of the canvas, moving with zoom and scrolling.
- Ticks: the spacing of numbered major ticks is the smallest of 1, 2, 5 × 10ⁿ pixels that is at least 60 pt on screen; each major interval is divided into 10 minor divisions, the 5th medium length (40% of the ruler thickness), the rest 20%, and major ticks span the full thickness.
- Numbers: 9 pt, color `#a8a8a8`; on the top ruler they are written to the right of the tick, and on the left ruler they are stacked vertically digit by digit, as in Photoshop.
- Background `#3c3c3c`, with a separator line on the side next to the canvas.
- When the pointer is inside the window, a thin line on each ruler marks the pointer position.

## Guides

- Drag down from the top ruler to create a horizontal guide, and right from the left ruler to create a vertical guide; the cursor over a ruler is the resize cursor for the corresponding direction. While dragging, the guide follows the pointer; releasing inside the canvas area adds it to the document and records "New Guide", and releasing outside the area discards it.
- Moving: when the current tool is the Move tool, or while holding ⌘ (in Photoshop ⌘ temporarily switches to the Move tool), the resize cursor appears when the pointer is within 4 pt of a guide, and dragging moves it; releasing inside the canvas area records "Move Guide", and releasing after dragging outside the area (e.g. back onto a ruler) deletes it and records "Delete Guide". Guides cannot be dragged when locked (View › Guides › Lock Guides) or hidden.
- Drawing: a 1 pt cyan (`#4affff`, Photoshop's default Cyan) straight line across the entire canvas area (including the gray area outside the canvas). A guide being dragged is always drawn.
- Guides move with canvas changes: Canvas Size offsets them by the anchor, cropping by the crop origin, Image Size scales them proportionally, and Image Rotation and canvas flips transform them geometrically (see `image_ops.md` and `document.md` in `op-core`).

## Grid

- One major line per inch (document resolution in pixels), subdivided into 4 (Photoshop's default setting), drawn only within the canvas; the grid is not drawn when subdivision lines would be less than 4 pt apart on screen.
- The color is semi-transparent gray, with major lines somewhat brighter than subdivision lines.

## Display toggles

`AppState::view` (`ViewOptions`): `rulers` (off by default), `extras` (on by default), `guides` (on by default), `grid` (off by default), `lock_guides` (off by default), `smart_guides` (on by default), `pixel_grid` (on by default), `selection_edges` (on by default), `layer_edges` (off by default). Guides are shown when both Extras and Guides are on (`guides_visible`), and the grid is shown when both Extras and Grid are on (`grid_visible`); when Extras is off, the selection's marching ants, the layer edges and the pixel grid are not shown either (the selection remains in effect). Show › Selection Edges, Layer Edges and Pixel Grid each hide their own item the same way.

## Known limitations

- No snapping (Snap, Snap To), no smart guides, artboard guides, guide layouts, or guide color and grid settings (Preferences).
- The only ruler unit is pixels; double-clicking a ruler does not change the unit, and the origin cannot be changed by dragging the top-left corner.
- Guides are not saved to PSD files.

## Test coverage

- `ruler_steps_are_round_numbers`: at different zoom levels the major tick spacing is 100, 200, 20 and 2 pixels.
