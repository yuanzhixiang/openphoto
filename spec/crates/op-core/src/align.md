# align.rs: Align and Distribute

## Responsibilities

Layer › Align, Layer › Distribute, and the align/distribute buttons in the Move tool options bar: moves the selected layers by their pixel extents so that they are aligned or evenly distributed, following the same rules as Photoshop 2026 (results verified by testing in Photoshop).

## Public interface

- `Align`: `Top`, `VerticalCenter`, `Bottom`, `Left`, `HorizontalCenter`, `Right`; `ALL` is the menu order, `label()` is the menu text ("Top Edges" etc.), `name()` is the history name ("Align Left Edges" etc., matching what Photoshop records).
- `Distribute`: the six above plus `Horizontally`, `Vertically` (equal spacing); likewise has `ALL`, `label()`, `name()` ("Distribute Vertical Centers" etc.).
- `movable_count(doc)`: the number of selected layers that would be moved (requires scanning each layer's pixel bounds; `op-ui` caches it by revision number). `can_align_count(doc, n)` is `can_align` given that count.
- `can_align(doc)` / `align(doc, how)`: with a pixel selection, the selected layers align to the selection's bounding box (one layer is enough); with no selection, at least two layers are required, and they align to their combined bounding box. Center alignment offsets are rounded to whole pixels.
- `can_distribute(doc)` / `distribute(doc, how)`: requires at least three layers. Layers are sorted by the chosen edge or center; the two outermost stay put and the ones in between are spaced evenly by the chosen edge or center; `Horizontally`/`Vertically` instead make the gaps between adjacent layers equal.

Participating layers: those that are selected, not the background, have neither position nor pixels locked, and have opaque pixels. Pixel extents use `TiledImage::content_bounds` (including pixels outside the canvas); moving uses a `with_canvas` translation, and pixels outside the canvas are preserved.

## Layer groups

A selected group counts as a single unit: its extent is the union of all pixel layers in the group, and when moved, all pixel layers in the group move together.

## Test coverage

- `matches_photoshop`: three squares are successively left-aligned, vertically center-aligned, bottom-aligned and distributed by vertical centers; the results match exactly the layer extents after the same operations in Photoshop 2026.
- `equal_gaps_and_selection_alignment`: after equal-spacing distribution, the gaps on both sides of the middle strip are equal; with a pixel selection, a single layer right-aligns to the selection.
