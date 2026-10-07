# image_ops.rs: canvas transforms (image size, rotate, flip, crop, trim)

## Responsibilities

Implements the Image menu operations that change the whole canvas: Image Size resampling, Image Rotation's fixed-angle rotations and canvas flips, Crop, and Trim. Every operation acts on each layer and the selection at once (including the previous selection available to Reselect), is carried out through `Document::transform_canvas`, and records no history.

## Public interface and rules

### Image size (resampling)

- `Resample`: the eight methods in the Photoshop 2026 menu, in the same order and with the same labels (`ALL`, `label`, `separator_after`):
  - `Automatic` (default): uses Bicubic Sharper when reducing and Bicubic Smoother when enlarging (`resolve`).
  - `PreserveDetails`, `PreserveDetails2`: Lanczos 3 (radius 3). Photoshop's Preserve Details (2.0 uses machine learning) includes noise reduction and other processing; here it is only approximated as Lanczos.
  - `BicubicSmoother`: Mitchell–Netravali (B = C = 1/3), softer.
  - `BicubicSharper`: cubic convolution with a = −0.75, sharper.
  - `Bicubic`: cubic convolution with a = −0.5.
  - `NearestNeighbor`, `Bilinear`.
  Test `resample_methods`: every method keeps a solid color unchanged; Automatic's choice; after enlarging the same step, the overshoot is Sharper > Bicubic > Smoother, and bilinear does not overshoot.
- `resize(doc, width, height, method)`: scales each layer and the selection to the new size (width and height must be greater than 0). Resamples separably, horizontally first, then vertically: the center of a target pixel corresponds to the source coordinate `(i + 0.5) / scale − 0.5`; bicubic uses a cubic convolution kernel with a = −0.5 (radius 2), bilinear uses a triangle kernel (radius 1), and the weights are normalized; when reducing, the kernel is widened in proportion so that every source pixel contributes (area-correct reduction); nearest neighbor takes the closest source pixel. Colors are computed in premultiplied alpha and the results are clamped to 0–255 (bicubic overshoots); pixels with alpha below 0.5 are fully transparent. The selection's degree of selection is resampled the same way. When a layer has pixels outside the canvas (`resize_with_outside`), the canvas together with the surrounding pixels is scaled by the same ratio and placed back at the proportionally scaled position (measured in Photoshop 2026, see the tests below).

### Rotate and flip

- `Orientation`: `Rotate180`, `Rotate90Clockwise`, `Rotate90CounterClockwise`, `FlipHorizontal`, `FlipVertical`. `history_name()` gives Photoshop's history name: rotations are all "Rotate Canvas", flips are "Flip Canvas Horizontal" / "Flip Canvas Vertical".
- `reorient(doc, orientation)`: remaps pixel by pixel, with no interpolation, losslessly. 90° rotations swap width and height. At 90° clockwise, the original top-left corner goes to the top right; at 90° counterclockwise, the original top-left corner goes to the bottom left; at 180°, the top-left corner goes to the bottom right. The selection is transformed the same way. When a layer has pixels outside the canvas, those pixels are rotated or flipped along with it by the same mapping (relative to the canvas; integer coordinates can be negative) rather than discarded.

### Crop

- `crop(doc, x0, y0, x1, y1)`: crops the canvas to that rectangle (right and bottom edges exclusive); the rectangle must be inside the canvas and non-empty, otherwise it panics. Layer pixels and the selection are translated together, so the selection still covers the original image content.
- `crop_to_selection(doc)`: Image › Crop. Crops to the selection's bounding rectangle; returns `false` when there is no selection. The selection is kept (the transparent parts of a feathered or non-rectangular selection stay in the selection as usual).

### Trim

- `TrimBasis`: `Transparent` (trims away fully transparent pixels, alpha 0), `TopLeftColor` (trims away pixels whose color is exactly the same as the top-left pixel), `BottomRightColor` (exactly the same as the bottom-right pixel). Color comparison includes alpha, with no tolerance allowed.
- `TrimSides`: whether each of the top, left, bottom, and right sides is trimmed; all selected by default.
- `trim_bounds(doc, basis, sides)`: finds, on the composited image, the bounding rectangle of all pixels that are "not trimmed away"; unchecked sides keep their original position; returns `None` when all pixels would be trimmed away.
- `trim(doc, basis, sides)`: crops by `trim_bounds`; when the result is `None` or the same as the original canvas, makes no change and returns `false`.

## Guides

- `reorient`: at 180° the position becomes `width − x` / `height − y`; at 90° clockwise, a vertical guide at x becomes a horizontal guide at x, and a horizontal guide at y becomes a vertical guide at `original height − y`; at 90° counterclockwise, a vertical guide at x becomes a horizontal guide at `original width − x`, and a horizontal guide at y becomes a vertical guide at y; a horizontal flip changes only vertical guides (`width − x`), and a vertical flip changes only horizontal guides.
- `crop` translates guides by the crop origin; `resize` scales guides by the width and height ratios.
- Test `guides_follow_the_canvas`: the position and orientation of guides after rotation, cropping, and scaling.

## Reveal All

`crop_extended(doc, (x0, y0, x1, y1), delete_cropped, background)`: the Crop tool's crop. The range can extend past the canvas: the canvas is enlarged there, the new area of the background layer is filled with `background`, and other layers are transparent (`place_canvas`). With `delete_cropped` (Delete Cropped Pixels), pixels outside the new canvas are deleted; otherwise they are kept on the layers, and the background layer becomes the regular layer "Layer 0" (unlocked) to keep its pixels outside the canvas. An empty range returns `false`. Test `crop_tool_crops_past_the_canvas_and_can_keep_pixels`.

`rotate_arbitrary(doc, degrees, background)`: Image › Image Rotation › Arbitrary... (Rotate Canvas). The whole document rotates around its center by `degrees` (positive is clockwise). The canvas is enlarged to the bounding rectangle of the rotated image, rounded up: 200 × 100 rotated 30° becomes 224 × 187, and rotated a further −45° becomes 291 × 291 (measured in Photoshop 2026). Resamples pixel by pixel bilinearly (premultiplied alpha); the new corners of the background layer are filled with `background` (the background color) and clipped to the canvas, the new corners of other layers are transparent, and pixels rotated out of the canvas are kept on the layers; the new corners of masks are revealed (255); the selection rotates along with it. Integer multiples of 360° do nothing and return `false`. Test `rotate_arbitrary_like_photoshop`: at 90° width and height swap and a point to the right of center moves below center; the sizes at 30° and −45°; the background corners are the background color and the center is still the original color.

`reveal_all(doc, background)`: Image › Reveal All. Uses `Document::content_bounds` to find the union of the canvas and all layer pixels (including those outside the canvas), enlarges the canvas to that range (`place_canvas`, with the old canvas placed at the corresponding offset), and fills the extended area of the background layer with `background` (the background color). When no pixel is outside the canvas, it does nothing and returns `false`. Test `reveal_all_grows_the_canvas_to_the_hidden_pixels` covers: pixels off the left and off the bottom right both appear on the new canvas, the background is extended with the background color, and a second call returns `false`.

## Known limitations

- Cropping always deletes pixels outside the new canvas (`clipped`, equivalent to Photoshop with "Delete Cropped Pixels" on); Photoshop's Crop tool can turn it off and keep pixels outside the canvas.
- `remapped` expands the whole image into a buffer, using image size × 4 bytes of memory regardless of how sparse the layers are.

## Test coverage

- `rotations_and_flips_move_the_corners`: positions of corner pixels and sizes after the five transforms.
- `the_selection_turns_with_the_canvas`: after a clockwise rotation the selection moves to the top-right corner.
- `crop_keeps_the_selected_area`: after cropping to the selection, size, pixels, and selection position are correct; no crop with no selection.
- `resize_scales_layers_and_selection`: a 4×2 image with a red left half reduced by half gives a mostly red left pixel and a near-white right pixel, with the selection following; nearest-neighbor enlargement keeps hard edges; bilinear enlargement blends colors at the edge.
- `trim_removes_borders_of_the_corner_color`: trimming by the top-left color trims all four sides, trimming by the bottom-right color trims only the top; trimming by transparent pixels on an opaque background changes nothing.
- `pixels_outside_the_canvas_follow_merges_rotation_and_image_size`: in a 100×100 document, a square half off the left side is merged with another block, then rotated clockwise, reduced by half, and flipped horizontally; at every step the layer range (including outside the canvas) matches Photoshop 2026 measurements: (−20, 10, 60, 60) → (40, −20, 90, 60) → (20, −10, 45, 30) → (5, −10, 30, 30).
