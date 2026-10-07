# transform.rs: Free Transform and Transform

## Responsibility

The pixel part of Edit › Free Transform and Edit › Transform: moves, scales, rotates, and flips the target layer's pixels with an affine transform (with a selection, only the selected pixels of the active layer are transformed), using bilinear resampling. Records no history.

## Public interface

### `Affine`

A 2D affine mapping `x' = a·x + b·y + c`, `y' = d·x + e·y + f` (document pixel coordinates, y pointing down).

- `IDENTITY`, `translate(x, y)`, `scale(sx, sy)`, `rotate(angle)` (radians, clockwise on screen is positive).
- `after(first)`: the composition of `first` followed by `self`.
- `apply(p)`, `inverse()` (`None` when the determinant is close to 0).
- `around(center, sx, sy, angle, offset)`: scales around `center` first, then rotates, then finally translates by `offset`; this is what the Free Transform box describes.

### `TransformError`

`NoLayer`, `Hidden`, `Locked`, `Empty`, with alert text matching Photoshop: "Could not complete the {command} command because there is no layer. / the target layer is hidden. / the layer is locked. / the selected area is empty.".

### `bounds(doc)`

The extent the transform acts on (x0, y0, x1, y1):

- Active layer hidden → `Hidden`; pixels or position locked, or a background layer without a selection → `Locked` (Photoshop cannot transform the background layer directly).
- No selection: the bounding rectangle of all of the layer's non-transparent pixels (including those outside the canvas), so the Free Transform box encloses parts moved off the canvas, matching Photoshop. With a selection: the bounding rectangle of the selection (the selection must contain non-transparent pixels inside the canvas).
- No transformable pixels → `Empty`.

### `transform(doc, m, background)`

1. First checks with `bounds`; returns `Empty` when `m` is not invertible. The working region is the union of the canvas, the extent of all the layer's pixels, and the extent of the transformed box (it can extend outside the canvas); all of the following steps take place in this region (read with `region_rgba8`, written back with `from_region`).
2. "Moving pixels": the whole layer when there is no selection; the selected pixels when there is a selection (alpha multiplied by the selection degree).
3. "Remaining pixels": nothing remains when there is no selection; with a selection, the layer minus the moving pixels: for a normal layer, alpha multiplied by (1 − selection degree); for a background layer, instead blended with `background` (the background color) by the selection degree, staying opaque.
4. For each destination pixel, the source position is found with the inverse of `m` (pixel center to pixel center), the moving pixels are sampled bilinearly in premultiplied alpha (outside the image is treated as transparent), and composited with Normal onto the remaining pixels.
5. With a selection, the selection itself is transformed the same way (the selection degree sampled bilinearly) and becomes the new selection.
6. Pixels transformed outside the canvas are kept on the layer; the result for a background layer is clipped to the canvas.

### `FixedTransform`

The fixed transforms of Edit › Transform: `Rotate180`, `Rotate90Clockwise`, `Rotate90CounterClockwise`, `FlipHorizontal`, `FlipVertical`. `affine(bounds)` gives the mapping around the center of the extent; `name()` is the menu and history name ("Rotate 180°", "Rotate 90° Clockwise", "Rotate 90° Counter Clockwise", "Flip Horizontal", "Flip Vertical").

## Known limitations

- Rotating 90° when the extent has an odd width or height produces a half-pixel offset (edges slightly blurred after resampling).

## Layer groups

When the current layer is a group (and there is no selection), all pixel layers in the group are targets; with a selection, `Empty` is returned.

## Multiple layers

When there is no selection, the targets (`targets`) are: the active layer, plus the other selected layers and the layers linked to the selected layers (`link::with_linked`), with groups expanded to their pixel layers; apart from the active layer, hidden, background, and pixel- or position-locked layers are skipped (they do not move). `bounds` is the union of the pixel extents of all targets (including outside the canvas), and `transform` applies the same transform to each target. So multiple selected or linked layers are free-transformed together with one transform box, and flipped and rotated together, matching Photoshop. With a selection, only the active layer is transformed.

- `selected_and_linked_layers_transform_together`: two layers selected, a third linked to one of them, a fourth linked but position-locked: the transform box is the union of the three, the three move together, and the locked one does not move; with a selection, only the selected pixels of the active layer move.

## Test coverage

- `affine_math`: the mapping and inverse mapping of `around`, rotation direction, non-invertible matrices.
- `bounds_and_locks`: layer content extent; a background layer is locked without a selection, and with a selection the extent is the selection's.
- `move_scale_and_flip_the_layer`: after translation, pixels arrive at the new position and the original spot becomes transparent; when scaled up 2x, the interior is solid and the outer ring is translucent due to interpolation; horizontal flip.
- `selected_pixels_move_and_leave_the_background_color`: moving selected pixels on a background layer fills the original spot with the background color, and the selection follows the move.
- `transforming_takes_pixels_outside_the_canvas_along`: a horizontal bar half outside the left edge of the canvas; the extent includes the part outside the canvas; after translating right, both pixels are inside the canvas; after translating up off the canvas, the pixels are still on the layer, with extent (2, −4)–(4, −3).

## Projective transform (`Projective`)

The 3 × 3 projective transform (double precision) needed for Distort and Perspective: `rect_to_quad(extent, corners)` maps the extent onto an arbitrary quadrilateral (Heckbert's unit-square-to-quadrilateral construction, which degenerates to affine when the corners form a parallelogram), `inverse`, `after`, `from_affine`, `is_affine`. `transform` is generic over the mapping (the `Mapping` trait, implemented by both `Affine` and `Projective`), sampling bilinearly per pixel with the inverse mapping, so affine and projective transforms take the same path.

- `projective_maps_the_box_onto_any_quad`: the four corners of a trapezoid correspond exactly, inverse mapping, a parallelogram is affine, agreement with the affine mapping.
- `distorting_the_layer`: pulling the two top corners of a square inward makes the top row's coverage less than the bottom row's.

## Interpolation (`Interpolation`)

The six options in the Free Transform options bar: Nearest Neighbor, Bilinear, Bicubic (default, Keys a = −0.5), Bicubic Smoother (Mitchell–Netravali), Bicubic Sharper (a = −0.75), Bicubic Automatic (same as Bicubic). `transform_with(doc, m, background, how)` samples with the chosen method (premultiplied color; the overshoot of the cubic kernels is clamped to the valid range); `transform` uses Bicubic; the selection mask is always bilinear. `Affine::skew(h, v)` is horizontal and vertical skew. Test `interpolation_methods`: when scaling up 2x, nearest neighbor keeps hard edges, bilinear has a transition, and every method is solid in the middle.

## Transforming only the selection

`selection_bounds(doc)` is the selection's extent; `transform_selection(doc, m)` moves only the selection by the mapping (resampling the selection degree bilinearly), leaving pixels in place, and returns `false` when there is no selection. `transform` also uses the same `turned_selection` when moving selected pixels. Test `transforming_the_selection_only`.

## Warp

- `WarpMesh`: a bicubic Bézier surface over the box, 4 × 4 control points (row by row from the top left). `flat(extent)` is a flat mesh (control points at the thirds); `at(u, v)` is the point on the surface; `pull((u, v), d)` moves the surface point at (u, v) by exactly `d`: each control point takes a share according to its Bernstein weight at that point (weight ÷ sum of squared weights), which is the effect of dragging inside the mesh in Photoshop.
- `warp(doc, extent, mesh, background color, interpolation)`: cuts the extent into 24 × 24 cells with two triangles each; a destination pixel finds the triangle it falls in (triangles are placed into a 64 × 64 grid index by where they land), uses barycentric coordinates to map back to the source point in the extent, and samples there. Shares `resample_targets` with the affine and projective transforms (moving the target layers or selected pixels, handling pixels outside the canvas and background layers).
- Tests `warp_mesh_and_pull` (a flat mesh is the box; the pulled point follows exactly; corner points move much less), `warping_the_layer` (a flat mesh does not change the image; after pulling the middle, the middle pixels move).
