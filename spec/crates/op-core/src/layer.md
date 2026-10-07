# layer.rs

## Responsibilities

Defines layers and their attributes: layer ID, blend mode, layer content type, and Photoshop layer attributes such as visibility, opacity and locks.

## Public interface

### `LayerId`

An opaque `u64` wrapper, comparable and hashable. IDs are allocated by `Document::new_layer_id()` from a process-wide counter (shared with `DocId`); this module does not allocate them.

### `BlendMode`

`PassThrough` ("Pass Through") is used only for groups: the layers in the group blend directly with the content below the group, as if there were no group; when the group is composited as a whole, it is equivalent to Normal.


- Covers all 27 Photoshop layer blend modes, with the enum order matching Photoshop's blend mode menu; the default is `Normal`.
- `GROUPS`: split into 6 groups as in Photoshop's menu, with a separator line between groups:
  1. `Normal`, `Dissolve`
  2. `Darken`, `Multiply`, `ColorBurn`, `LinearBurn`, `DarkerColor`
  3. `Lighten`, `Screen`, `ColorDodge`, `LinearDodge`, `LighterColor`
  4. `Overlay`, `SoftLight`, `HardLight`, `VividLight`, `LinearLight`, `PinLight`, `HardMix`
  5. `Difference`, `Exclusion`, `Subtract`, `Divide`
  6. `Hue`, `Saturation`, `Color`, `Luminosity`
- `label()`: the English name shown in the menu, worded as in Photoshop; for example `LinearDodge` is shown as `"Linear Dodge (Add)"`.

### `LayerKind`

- `Raster(TiledImage)`: a pixel layer.
- `Group { collapsed }`: a layer group (folder). The group itself has no pixels; its layers sit directly below it in `Document::layers`, each with `parent` pointing to the group's id (the same as PSD storage: the group is recorded above, its layers below). `collapsed` means collapsed in the Layers panel.
- `Layer::group(id, name)` creates a new group with blend mode `PassThrough`. `is_group()`; `image()` / `image_mut()` return the image of a pixel layer and `None` for a group. All code that processes pixels gets the image through them and must handle the group case explicitly.

(Original `LayerKind` description)

Layer content type. Currently only `Raster(TiledImage)`: a raster layer whose image size matches the document.

### `Layer`

Public fields:

- `id`, `name`
- `visible`: whether it takes part in compositing.
- `opacity`: layer opacity; semantically applies to the whole layer (including layer styles).
- `fill`: fill opacity; semantically applies only to the pixels themselves, not to layer styles.
- `blend_mode`
- `is_background`: the "Background" layer flag. By Photoshop semantics, the background layer is locked, opaque and always at the bottom.
- `lock_transparency`, `lock_pixels`, `lock_position`, `lock_nesting` (prevents auto-nesting into and out of artboards and frames): the four individual locks.
- `link`: link number (`Option<u32>`). Layers with the same number are linked together; a number no other layer shares is the same as not linked, so nothing needs tidying after deleting or unlinking (see `link.md`).
- `link_disabled`: the link is temporarily disabled (⇧-click on the link icon); the layer keeps its number but is not carried along with its partners. Not saved.
- `lock_all`: lock all. It is a separate flag rather than turning on all four individual locks, so turning it off restores the individual locks as they were (measured in Photoshop 2026: lock position, then click "Lock all" twice, and the position lock is still there). In PSD it is also a separate bit (see `psd.md` in `op-io`).
- `kind`: layer content.
- `mask`: layer mask (`Option<LayerMask>`); none for a new layer.
- `color`: color label (`LayerColor`); `None` for a new layer.
- `parent`: the group it is in; `None` when not in a group.

### `LayerColor`

The layer's color label: `None`, `Red`, `Orange`, `Yellow`, `Green`, `Blue`, `Violet`, `Gray`, in the same order as Photoshop's Color menu. `label()` is the English name; `psd_index()` / `from_psd_index()` are the numbers in the PSD `lclr` block (0 = None … 7 = Gray, checked against files saved by Photoshop 2026).

### `neutral_color(mode)`

The neutral color used by "Fill with ‹mode›-neutral color" in the New Layer dialog: it does not change the color of the image below in that blend mode. Overlay, Soft Light, Hard Light, Vivid Light, Linear Light and Pin Light use 50% gray (128, 128, 128); Multiply, Color Burn, Linear Burn, Darken and Divide use white; Screen, Color Dodge, Linear Dodge, Lighten, Difference, Exclusion and Subtract use black; the other modes (Normal, Dissolve, Darker/Lighter Color, Hard Mix, Hue, Saturation, Color, Luminosity) have no neutral color and return `None`.

### `LayerMask`

Layer mask: white reveals the layer, black hides it, gray partially reveals it. Stored as an opaque grayscale `TiledImage` (every color channel holds the mask value and alpha is always 255), so a pure white or pure black mask shares a single tile across the whole image.

- `enabled`: false when turned off with Layer › Layer Mask › Disable; the mask is kept but ignored in compositing.
- `filled(w, h, value)`: a mask with a single value (255 reveals all, 0 hides all).
- `from_values(w, h, f)`: built from a per-pixel value (for example the selection degree).
- `value(x, y)`: the mask value at a pixel (0 outside the image).

`Layer::raster(id, name, image)` constructs a regular raster layer: visible, `opacity` and `fill` both 1.0, Normal blend, not the background, no locks.

`is_locked()`: returns `true` when the layer is the background layer or its pixels or position are locked. The transparency lock does not count; in Photoshop it is a partial lock.

Effective locks (editing operations always use these methods instead of reading the fields directly, otherwise "lock all" would be missed):

- `transparency_locked()` = `lock_transparency || lock_all`
- `pixels_locked()` = `lock_pixels || lock_all`
- `position_locked()` = `lock_position || lock_all`

`locks()` / `set_locks(Locks)` read and write the five flags at once. `Locks { transparency, pixels, position, nesting, all }` is the value type used by the Lock Layers dialog and `layer_ops`.

## Behavior rules

- `opacity` and `fill` are floats in 0..=1; this module does not validate the range.
- The background layer's "locked, opaque, at the bottom" is a semantic convention that this module does not enforce: the fields are all public and maintained by callers (the document constructors and the `op-ui` Layers panel). The `op-ui` Layers panel disables editing of blend mode, opacity and lock toggles for the background layer.
- In compositing, `opacity * fill` is the layer's overall alpha factor (see `document.md`); since there are no layer styles yet, the two have the same effect.
- `blend_mode` takes effect in compositing; for the algorithm see `blend.md`.

## Relation to other modules

- `document.rs` holds `Vec<Layer>` and sets up the background layer at construction.
- The `op-ui` Layers panel builds the blend mode menu with `BlendMode::GROUPS` and `label()`, uses `is_locked()` to decide the lock icon state, and modifies `Layer` fields directly and then calls `Document::mark_dirty()`.

## Known limitations

- Raster layers and layer groups only; no adjustment layers, type layers, shape layers or smart objects.
- Layer masks are supported, but there are no clipping masks or layer styles (so `fill` and `opacity` have the same effect).
- Prevent auto-nesting (`lock_nesting`) only stores state: there are no artboards or frames yet, so it does not affect any operation.

## Test coverage

This file has no unit tests.
