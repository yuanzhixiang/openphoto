# clipboard.rs: Pixel rules for cut, copy, and paste

## Responsibility

Implements the pixel part of Edit › Cut, Copy, Copy Merged, Paste, and Paste Special › Paste in Place: which pixels are copied, what is left behind after a cut, and where content is pasted. The clipboard itself (holding the content, exchanging with the system clipboard) is in `clipboard.rs` in `op-ui`.

## Public interface

### `Clip`

Pixels on the clipboard: `width`, `height`, `pixels` (straight-alpha RGBA8, row by row), `origin` (the top-left position in the source document; `None` for images from other applications). `Clip::from_rgba8(w, h, pixels)` constructs an image without a position, and panics when the pixel length is wrong.

### `ClipError`

- `NoLayer`, `Hidden`, `Locked`: alert text is the same as for Fill (see `fill.md`), e.g. "Could not complete the Cut command because the target layer is hidden.".
- `Empty`: only transparent pixels within the copy extent; alert "Could not complete the {command} command because the selected area is empty.", matching Photoshop.
- `message(command)` produces the alert, where `command` is "Cut", "Copy", or "Copy Merged".

### Functions

- `copy(doc)`: copies the active layer.
- `copy_merged(doc)`: copies the composite of all visible layers (`composite_rgba8`).
- `cut(doc, background)`: copies first, then clears according to the rules of Edit › Clear (`fill::clear`): becomes transparent on a normal layer, and is filled with the background color on a background layer or a layer with locked transparent pixels.
- `placement(clip, width, height, visible, in_place)`: computes the paste position (top-left corner).
- `paste(doc, clip, at)`: places the content on a new layer and returns the new layer ID.

## Behavior rules

### Copy extent

- With a selection, the selection's bounding rectangle; without a selection, the whole canvas.
- Each pixel's alpha is multiplied by the selection degree at that point (rounded), so feathered and anti-aliased selection edges come along semi-transparent. Pixels inside the bounding rectangle but outside the selection have alpha 0.
- When all pixels in the extent have alpha 0, `Empty` is returned and the clipboard is not changed. An empty selection (no bounding rectangle) also returns `Empty`.
- `origin` is the top-left corner of the bounding rectangle.

### Availability

- Copy and Copy Merged do not check whether the layer is hidden or locked; they only read, never write.
- Cut requires the active layer to be visible (otherwise `Hidden`) and its pixels not locked (otherwise `Locked`); the checks happen before copying, and on failure the document is unchanged.

### Paste position

- Paste in Place: always uses `origin` (without `origin`, the centering rule below applies).
- Paste: uses `origin` when `origin` exists, the whole content falls inside the canvas, and it intersects the visible area; so copying and pasting within the same document stacks the copy exactly over the original, matching Photoshop. Otherwise the content is centered in "the intersection of the visible area and the canvas"; when the intersection is empty (the canvas is scrolled completely out of the window), it is centered on the canvas. Centered coordinates are rounded to whole pixels.
- `visible` is supplied by the caller (`[x0, y0, x1, y1]` in document pixel coordinates).

### Paste

- Creates a transparent layer the same size as the canvas, named by `Document::next_layer_name` ("Layer N"), inserted directly above the active layer and made the active layer (`insert_above_active`).
- Only pixels with alpha greater than 0 are written; parts that fall outside the canvas are kept on the new layer (`set_pixel_at`), matching Photoshop, and Image › Reveal All can bring them into view.
- After pasting, the selection is deselected (the original selection is recorded as the selection that Reselect can restore), matching Photoshop.
- The caller is responsible for history (`op-ui` records "Paste" and "Cut").

## Edge cases

- Without an active layer, Copy and Cut return `NoLayer`; Copy Merged does not need an active layer.
- Pasted content can be larger than the canvas; the excess is clipped.

## Known limitations

- No Paste Into / Paste Outside (requires layer masks).
- Pasting text (Photoshop creates a type layer) and vector paths is not supported.

## Layer groups

When the current layer is a group, copy and cut return `ClipError::Group`.

## Test coverage

- `copy_takes_the_selection_bounds`: copies the pixels and position within the selection's bounding rectangle; without a selection, copies the whole layer.
- `copying_transparent_pixels_fails`: copying on a transparent layer returns `Empty` and its alert text; Copy Merged picks up the pixels of the layer below.
- `cut_clears_and_paste_stacks_on_the_original`: cutting on a background layer leaves the background color behind; pasting back to the original position creates "Layer 1", which becomes the active layer, and deselects.
- `paste_centers_when_the_origin_is_out_of_view`: centers when the original position extends beyond the canvas; Paste in Place keeps the original position; an external image is centered in the visible area.
- `paste_keeps_pixels_outside_the_canvas`: the part inside the canvas displays normally, pixels beyond the right edge are kept on the layer, and the layer's content extent reaches outside the canvas.
