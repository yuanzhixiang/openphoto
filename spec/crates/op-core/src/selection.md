# selection.rs: Selection

## Responsibilities

Pixel selection. As in Photoshop, a selection is an 8-bit mask the same size as the document: 255 is fully selected, 0 is unselected, and values in between are partially selected (anti-aliased or feathered edges). The document's selection is stored in `Document` (see `document.md`).

## Public interface

- `SelectionOp`: how a new shape combines with the existing selection — `Replace` (new), `Add` (add), `Subtract` (subtract), `Intersect` (intersect).
- `Rect`: a rectangle in document pixel coordinates, normalized on construction to top-left to bottom-right; the bottom-right boundary is exclusive, and it may extend beyond the document. It is empty when its width or height is less than 1 pixel.
- `Selection::all(w, h)`: select all.
- `Selection::rect(w, h, rect)`: rectangular selection; the boundaries are rounded to whole pixels and the part outside the document is clipped.
- `expand(radius)`: Select › Modify › Expand. Uses an exact Euclidean distance transform (Felzenszwalb–Huttenlocher) to find each pixel's distance d to the nearest selected pixel (selection degree ≥ 128); coverage is `radius + 1 − d` (clamped to 0–1): pixels within radius are fully selected, a ring outside them is partially selected, and corners are rounded.
- `contract(radius, at_bounds)`: Select › Modify › Contract. With d the distance to the nearest unselected pixel, coverage is `d − radius`. When `at_bounds` is true, a ring outside the canvas is treated as unselected (the selection contracts from the canvas edges); when false, the canvas edges do not make the selection contract, consistent with Photoshop when "Apply effect at canvas bounds" is unchecked.
- `border(width)`: Select › Modify › Border. A band of width `width` centered on the selection edge: selected pixels use the distance to unselected, unselected pixels use the distance to selected, and coverage is `width / 2 + 1 − d`.
- `smooth(radius)`: Select › Modify › Smooth. First binarizes (≥ 128), then takes a (2r + 1)² box average; pixels whose average is at least half are selected, the rest unselected — rounding sharp corners and removing isolated dots.
- `Selection::polygon(w, h, points, anti_alias)`: the interior of a closed polygon (Lasso tool), using the even-odd rule when self-intersecting. With anti-aliasing on, each pixel takes 4 sample rows and accumulates coverage from the exact horizontal overlap of the interior spans on each row with the pixel; with it off, a pixel is selected only if its center is inside. With fewer than 3 points the selection is empty.
- `Selection::ellipse(w, h, rect, anti_alias)`: the ellipse inscribed in `rect`. With anti-aliasing on, edge pixels are partially selected by 4×4 supersampled coverage; with it off, a pixel is selected only if its center is inside the ellipse.
- `Selection::from_mask(w, h, mask, anti_alias)`: created from a 0/255 mask (the Paint Bucket's fill region). With anti-aliasing on, pixels immediately outside the selected region are selected at 50%.
- `get(x, y)`: a pixel's selection degree (0 outside the document).
- `is_empty()`: no pixel is selected.
- `bounds()`: the bounding rectangle of selected pixels (value > 0).
- `inverse()`: invert the selection (each value becomes 255 − v).
- `combine(current, shape, op)`: merges a new shape into the existing selection according to `op`. Add takes the larger value, Intersect the smaller, and Subtract is `a × (255 − b) / 255`. With no existing selection: New and Add yield the shape itself, Subtract and Intersect yield an empty selection.
- `feather(radius)`: feather. Approximates a Gaussian blur with sigma = radius / 2 using three box blurs, extending beyond the edges with the nearest value.
- `with_canvas(w, h, dx, dy)`: the selection after a canvas size change; the original selection is placed at (`dx`, `dy`), and the added area is unselected.
- `remapped(w, h, source)`: the same remapping as `TiledImage::remapped`, used to transform the selection when rotating and flipping the canvas.
- `outline()`: the marching ants outline. Uses 128 as the threshold between selected and unselected, returns all unit pixel edges between the two, merged into horizontal and vertical segments, each as `[x0, y0, x1, y1]` (document pixels).

## Edge cases

- The selection size always equals the document size; `combine` assumes both have the same size.
- With a feather radius less than or equal to 0, the selection is returned unchanged.
- Pixels with a selection degree below 128 are not inside the marching ants (consistent with Photoshop: very faint edges after feathering show no marching ants).

## Known limitations

- The selection is stored densely, 1 byte per pixel, proportional to the document size.
- There are none of Photoshop's Select › Modify (Expand, Contract, Smooth, Border), Grow, Similar, Color Range or similar operations.

## Test coverage

- `rect_and_bounds`: rounding of rectangle boundaries and clipping when extending beyond the document.
- `combine_ops`: the four combination modes, and Subtract with no existing selection.
- `inverse_and_all`: inverse and select all.
- `modify_expand_contract_border_smooth`: the extent and rounded corners of Expand 2; Contract 1; a select-all canvas does not contract at the edges with at_bounds off and does contract with it on; Border 2 straddles both sides of the edge; Smooth removes isolated pixels.
- `polygons_fill_their_inside`: the total anti-aliased coverage of a triangle with legs of 4 is about 8 pixels; without anti-aliasing there are only 0 and 255; fewer than 3 points is empty.
- `ellipse_anti_aliasing`: without anti-aliasing there are only 0 and 255; with anti-aliasing there are intermediate values.
- `feather_softens_edges`: after feathering, the interior is still nearly fully selected, edges have intermediate values, and far away it is close to 0.
- `outline_of_rect`: the outline of a rectangular selection is exactly four edges.
- `with_canvas_shifts`: after canvas expansion, the selection moves with the offset.

## Moving a selection

- `translated(dx, dy)`: the same selection moved by whole pixels; what moves past the canvas edge is lost (used by the Patch tool and Content-Aware Move, whose selection follows the moved pixels).
