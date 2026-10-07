# document.rs

## Responsibilities

Defines `Document`: an open image document, including its size, resolution, color mode, bit depth, layer list and active layer. Responsible for creating documents new or from pixels, producing and restoring undoable snapshots, resizing the canvas (Image > Canvas Size), and compositing all visible layers into a single RGBA8 image on the CPU.

## Public interface

### `DocId`

The document ID, a `u64` wrapper. It shares a single process-wide atomic counter with `LayerId` (starting at 1, incremented with `Relaxed`), so no document ID and layer ID within the process ever coincide.

### `Anchor`

A position in the 3×3 anchor grid of the Canvas Size dialog, which decides where the original image is pinned on the new canvas when the canvas size changes.

- `x`: 0 = left, 1 = center, 2 = right; `y`: 0 = top, 1 = center, 2 = bottom.
- `Anchor::CENTER` is (1, 1).
- Per-axis offset rule (the coordinate of the original image's top-left corner on the new canvas, `diff = new - old`):
  - 0: offset 0;
  - 1: `diff.div_euclid(2)`, i.e. rounded down;
  - any other value (including 2 and illegal values greater than 2): offset `diff`.
- How odd size differences are split when centered: when enlarging, the extra 1 pixel is added on the right/bottom (e.g. width +3 adds 1 on the left and 2 on the right); when shrinking, because rounding is toward negative infinity, the extra 1 pixel cropped comes from the left/top (e.g. width −3 crops 2 on the left and 1 on the right).

### `Guide`

A guide: `vertical` (vertical or horizontal), `position` (in document pixels; may be outside the canvas) and `color` (its own color from New Guide, or None for the guides' default color). Transforms of the canvas keep each guide's color.

### `Snapshot`

The undoable part of the document: `width`, `height`, `resolution`, `layers`, `active_layer`, `selection`, `last_selection`, `guides`, `quick_mask`. The fields are private; it can only be created by `Document::snapshot()` and used by `Document::restore()`. Cloning is cheap because the tiles in the layers are shared through `Arc`.

`title`, `id`, `color_mode`, `bit_depth` and `revision` are not in the snapshot, and undo/redo does not change them.

### `Document`

Public fields: `id`, `title`, `width`, `height`, `resolution` (ppi), `color_mode`, `bit_depth`, `layers` (bottom to top), `active_layer`, `guides` (guides, undoable like edits, as in Photoshop). Private field `revision`.

- `new_with_background(title, width, height, background)`: File > New. Creates a single background layer (named `"Background"`, `is_background = true`), fills it with the `background` color, and makes it the active layer. Resolution 72 ppi, RGB, 8-bit.
- `from_rgba8(title, width, height, pixels)`: opens a flat bitmap from a tightly packed RGBA8 buffer, producing a single layer that becomes the active layer, following the same rules as Photoshop when opening an image: when every pixel's alpha is 255, it is a background layer named `"Background"` (`is_background = true`); as soon as one pixel's alpha is less than 255, it is a normal layer named `"Layer 0"`, and the document therefore has no background layer. The other metadata is the same as above (72 ppi, RGB, 8-bit).
- `new_layer_id()`: allocates a new `LayerId` from the global counter; it does not modify the document or add a layer to the list.
- `next_layer_name()`: the name of a new layer, "Layer N", where N is the largest number among existing "Layer number" names plus 1, or 1 if there are none, as in Photoshop.
- `insert_above_active(layer)`: inserts the layer directly above the active layer, into the group the active layer is in; when the active layer is an expanded group, inserts it at the top inside the group (as in Photoshop); with no active layer, places it at the top. Makes it the active layer and calls `mark_dirty()`.
- `insertion_point()`: the insertion position and containing group for a new layer (used by `insert_above_active` and by copying to another document).
- `next_group_name()`: the name of a new group, "Group N", by the same rule as `next_layer_name`. `block(id)`: the contiguous range in `layers` occupied by a layer (together with its contents when it is a group). `set_selected_layers(ids)`: selects exactly these layers, the last one being the active layer.
- `revision()` / `mark_dirty()`: read / increment the revision number. The revision is incremented on every change to pixels or layer properties, and the rendering layer and the layer thumbnail cache use it to decide whether to refresh.
- `snapshot()` / `restore(snapshot)`: produce / restore a snapshot. `restore` overwrites the width and height, resolution, layers and active layer, and calls `mark_dirty()`.
- `resize_canvas(width, height, anchor, fill)`: Image > Canvas Size.
- `transform_canvas(width, height, image, selection)`: replaces the canvas with one of `width`×`height`: each layer's image, layer mask and the quick mask go through the `image` function, the current selection and the reselectable selection go through the `selection` function, and then the size and selection revision are updated and `mark_dirty()` is called. Crop, Trim, Image Rotation and canvas flips are all implemented through it (see `image_ops.md`). It records no history.
- `map_guides(f)`: transforms every guide with `f` (called when cropping, extending the canvas, scaling, rotating, flipping). `resize_canvas` offsets the guides by the anchor itself.
- `mask_target`: whether the editing target is the current layer's mask (the mask thumbnail was clicked in the Layers panel) or its pixels. Not in the snapshot; undo does not change it.
- `quick_mask`: the grayscale image in Quick Mask mode (Q) (white = selected, black = unselected); it is in the snapshot, so painting on the quick mask can be undone.
- `enter_quick_mask()`: converts the current selection into a grayscale image (entirely white when there is no selection) and deselects (the selection does not exist until exiting).
- `exit_quick_mask()`: converts the grayscale image back into a selection; all white or all black results in no selection.
- `editing_mask()`: `mask_target` is true and the current layer actually has a mask.
- `edit_target()`: the `EditTarget` of pixel edits (painting, fill, gradient), with priority quick mask > layer mask > layer pixels: the image to write to (layer pixels or mask), whether it is a mask (colors must be converted to grayscale), and whether alpha must be preserved (background layer, locked transparent pixels, or a mask).
- `has_background()`: whether a background layer exists. It decides whether Canvas Size's "Canvas extension color" is meaningful: without a background layer, all extended areas are transparent.
- `layer(id)` / `layer_mut(id)`: find a layer by ID with a linear search.
- `content_bounds()`: the bounding box of the union of the canvas and all layer pixels (including those outside the canvas); used by Reveal All.
- `place_canvas(width, height, dx, dy, fill)`: switches to a canvas of the new size with the old canvas placed at (`dx`, `dy`); shared by Canvas Size and Reveal All; see "Canvas resizing" below for the rules.
- Layer multi-selection (⌘/⇧-click in the Layers panel): the private field `selected_layers` is valid only when it contains the active layer, so any code that changes `active_layer` directly naturally falls back to single selection without leaving an inconsistent selection state; it is not in the snapshot and, as in Photoshop, records no history.
  - `selected_layers()`: the selected layers, bottom to top; when the multi-selection does not contain the active layer it is `[active layer]`, and with no active layer it is empty. `is_layer_selected(id)`.
  - `select_layer(id)`: click, selecting only it. `toggle_layer_selection(id)`: ⌘-click, adds it (and makes it the active layer) or removes it (when removing the active layer, the last selected one becomes the active layer; when only one remains it is not removed). `select_layer_range(id)`: ⇧-click, selects all layers between the active layer and `id` (in layer order), and `id` becomes the active layer.
  - `select_all_layers()`: Select › All Layers, selects all layers except the background (returns false when there are none). `deselect_layers()`: Select › Deselect Layers, selects none.
- `layer_at(x, y)`: used by the Move tool's Auto-Select. Searches from top to bottom for the first layer showing a pixel at (x, y): the layer is visible, its opacity and fill are not 0, the pixel's alpha is greater than 0, and it is not hidden by an enabled layer mask with value 0. `None` when the coordinate is outside the canvas or there is no such layer.
- `composite_rgba8()`: composites into a tightly packed straight RGBA8 buffer of length `width * height * 4`.
- `sample_source(scope)`: a tool's "Sample:" source (`SampleScope`): `Current` is the current layer's pixels, `CurrentAndBelow` is the composite of the layer list up to the current layer, `All` is the full composite; `None` when there is no current layer.
- `composite_layers_rgba8(layers)`: composites only the given layer list by the same rules (bottom to top, same size as the document), used for merging layers; `composite_rgba8` is just this called on all of the document's layers.

## Behavior rules

### Canvas resizing

- The x and y offsets are computed separately by the `Anchor` rules, and then `place_canvas` is called: `TiledImage::with_canvas` is called on each layer. Layer masks move the same way, with new areas filled white (revealed).
- The extended area of the background layer is filled with the RGB of `fill`, with alpha forced to 255 (whatever the alpha of `fill` itself), keeping the background layer opaque.
- The extended areas of non-background layers are filled fully transparent.
- When the canvas shrinks, pixels of non-background layers beyond the new canvas are kept outside the canvas (reappearing when the canvas is enlarged again or on Reveal All); the background layer is cropped to the new canvas (`clipped`), as in Photoshop.
- After all layers have been rewritten, the document width and height are updated and `mark_dirty()` is called.
- This method records no history; the caller calls `History::record` afterward.

### Layer groups

- `descendants(id)`: the layers in a group (at any depth). `pixel_layers(ids)`: replaces the groups among them with the pixel layers inside those groups, in layer order, deduplicated.
- `is_shown(id)`: the layer itself and every group containing it are visible. Move, Free Transform and Auto-Select (`layer_at`) all judge visibility by it.
- Compositing is split into row bands aligned to tile rows, distributed across CPU threads (`std::thread::available_parallelism`), each done separately and then joined; the result is the same as single-threaded: dragging a layer in a 3000 × 1080 document re-composites every frame, taking about 37 ms single-threaded and about 5 ms in parallel. Normal-mode layers take a straight source-over fast path (bypassing the generic blend function, with the same result as `blend::composite`'s Normal).
- Compositing (`composite_layers_rgba8`) recurses through groups: layers at the same level are processed in order; when a group is Pass Through with opacity (including fill) 100% and no mask, its layers are composited directly onto what is below; otherwise the group's layers are first composited into a transparent buffer, which is then blended on as a whole with the group's blend mode (Pass Through treated as Normal), opacity and mask. Hidden groups do not take part. When only some layers are given (merging), layers whose parent group is not among them are treated as top-level.
- `edit_target` returns `None` for groups (groups have no pixels to edit). Canvas transforms such as Canvas Size skip group pixels (groups have none) but do process group masks.

### Compositing

- Iterates bottom to top over layers whose `visible` is true; layers whose `opacity * fill` is less than or equal to 0 are skipped entirely.
- Iterates over the layer's allocated tiles; unallocated tiles are treated as transparent and skipped.
- Each pixel is composited with the layer's blend mode (`blend::composite`; see `blend.md` for the algorithm): the source alpha is pixel alpha × `opacity` × `fill` × (enabled mask value / 255), and the result is straight alpha. Pixels with source alpha 0 are skipped. Masks use the same tile grid as the layer and are read tile by tile; when the mask is missing a tile, that tile is treated as fully hidden.
- Blending happens in gamma-encoded (sRGB) space, matching Photoshop's default settings (see `README.md` for the reasoning).
- Accumulation uses an `f32` buffer; finally each component is clamped to 0..=1 and quantized to `u8` by truncating `×255 + 0.5`.
- Pixels not covered by any layer are `[0, 0, 0, 0]` in the result.
- Example: a 50%-opacity black layer over a white background gives `[128, 128, 128, 255]`.

## Edge cases

- Width or height 0: creating and resizing to a 0 size is allowed; no tiles are allocated and `composite_rgba8` returns an empty buffer. This module does not validate lower or upper size limits; those limits are the caller's responsibility (e.g. the Canvas Size dialog).
- The buffer length for `from_rgba8` must equal `width * height * 4`, or it panics (the check is in `TiledImage::from_rgba8`).
- Calling `new_with_background` with a color whose alpha is 0 gives a fully transparent background layer with no tiles allocated.
- With no layers, or all layers invisible, the composite is fully transparent.
- `composite_rgba8` assumes each layer's image has the same size as the document. If a layer is larger than the document (which can only happen by writing the public fields directly), computing the remaining width and height underflows `usize`.
- `layer` / `layer_mut` return `None` when the ID is not found; the caller guarantees that the layer `active_layer` points to exists.

## Relationship to other modules

- Depends on `tile.rs` (pixel storage, `with_canvas`), `layer.rs` (layers), `color.rs` (color quantization), `pixel.rs` (metadata).
- `history.rs` implements undo through `snapshot()` / `restore()`.
- `op-io` opens files with `from_rgba8` and exports with `composite_rgba8`.
- `op-ui` creates documents, calls Canvas Size, calls `mark_dirty()` after modifying layers, and hands the result of `composite_rgba8` to `op-render` for display.

## Known limitations

- Compositing is done entirely on the CPU and recomputes the whole image every time, with no incremental dirty-region compositing.
- Compositing does not distinguish `opacity` from `fill`; the two are multiplied to form the layer alpha.
- Canvas Size supports only pixel size changes and does no resampling (that belongs to Image Size).

## Selection

- The document stores the current selection `selection` (`None` means no selection, in which case edits apply to the whole document) and the last deselected selection `last_selection` (for Select › Reselect). Both are in the snapshot, so selection changes can be undone, as in Photoshop.
- `set_selection(s)`: replaces the selection; an empty selection is treated as no selection. When deselecting (setting `None`), the previous selection is recorded as `last_selection`.
- `reselect()`: restores the last deselected selection; `can_reselect()` is true when there is no selection and there is one to restore.
- `selection_revision()`: incremented on every selection change (including restoring by undo and canvas size changes); the UI uses it to cache the marching ants outline. Selection changes do not change pixels and do not trigger re-compositing.
- `resize_canvas` also translates the selection and `last_selection`.

## Test coverage

- `selection_deselect_reselect_and_undo`: after deselecting, the selection can be reselected; restoring a snapshot restores the selection in it; an empty selection is treated as no selection.
- `opaque_bitmap_opens_as_background`: fully opaque pixels open as a background layer.
- `transparent_bitmap_opens_as_regular_layer`: with semi-transparent pixels it opens as a normal layer "Layer 0", the document has no background layer, and that layer is the active layer.
- `resize_canvas_centered`: a 2×2 white background extended centered to 4×5 with black as the extension color; verifies the new size, black all around, the original image at (1,1)–(2,2), and that with an odd height difference the extra row is at the bottom (row 3 is black).
- `resize_canvas_keeps_layers_transparent`: when extending to 3×3 with the top-left anchor, the extended area of a non-background layer stays transparent and the original pixels keep their positions.
- `groups_composite_their_layers`: a black layer in a group shows directly in pass-through mode; after hiding the group its layers are not shown, `is_shown` is false and Auto-Select falls through to the background; with group opacity 50% the whole blends to gray; `descendants` and `pixel_layers` are correct.
- `layer_multi_selection`: click, ⇧ range, ⌘-removing the active layer makes the last one the active layer, ⌘-adding the background, setting the active layer directly returns to single selection, All Layers excludes the background, Deselect Layers.
- `canvas_size_keeps_hidden_pixels_except_on_the_background`: after shrinking the canvas, a normal layer's content bounds extend beyond the canvas while the background layer has no pixels outside the canvas; after enlarging again, the hidden pixels reappear.
- `layer_at_finds_the_topmost_visible_pixel`: where the upper layer has pixels it picks the upper layer, at transparent spots it falls through to the background, after hiding the upper layer it falls through to the background, and outside the canvas it is `None`.
- `composite_half_opacity_over_white`: a 50%-opacity black layer over a white background gives `[128, 128, 128, 255]`, verifying gamma-space blending and the quantization rule.

## Effective locks

A layer group's locks apply to all layers in the group (measured in Photoshop 2026: after Lock all on a group, layers in the group also report `allLocked` etc. as true). Edit operations always decide using the methods on `Document` that take ancestor groups into account, rather than looking only at the layer itself:

- `transparency_locked(id)`, `pixels_locked(id)`, `position_locked(id)`: the layer itself or any group containing it has the corresponding lock (the respective `Layer::*_locked`, including Lock all).
- `in_locked_group(id)`: some group containing it has Lock all turned on. Such a layer cannot be deleted, cannot have its blend mode or opacity changed, and cannot have layer styles added (in the Layers panel, the lock row, blend mode, opacity, fx and trash can are all grayed out).
