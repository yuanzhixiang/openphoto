# pixel.rs

## Responsibilities

Defines document-level pixel format descriptions: the color mode `ColorMode` (corresponding to Photoshop's Image > Mode) and the per-channel bit depth `BitDepth`. They are document metadata, for display and selection in the UI; they currently do not affect the actual pixel storage format.

## Public interface

### `ColorMode`

- Values: `Bitmap`, `Grayscale`, `Indexed`, `Rgb` (default), `Cmyk`, `Lab`, `Multichannel`.
- `ALL`: lists all 7 values in the order above, which matches the order of Photoshop's Image > Mode menu, for iterating in menus/dropdowns.
- `label()`: the full English name, e.g. `"RGB Color"`, `"Indexed Color"`, `"CMYK Color"`.
- `short()`: the short name on the document tab, e.g. `"RGB"`, `"Gray"`, `"Index"`, combined with the bit depth into displays like `RGB/8`.

### `BitDepth`

- Values: `U8` (default), `U16`, `F32`.
- `ALL`: listed in the order 8, 16, 32.
- `bits()`: returns 8, 16, 32.
- `label()`: `"8 Bits/Channel"`, `"16 Bits/Channel"`, `"32 Bits/Channel"`.

## Behavior rules

- New and opened documents are always `ColorMode::Rgb` + `BitDepth::U8` (see `document.rs`).
- `Document`'s `color_mode` and `bit_depth` are public fields and can be rewritten, but rewriting them only changes metadata and does not convert any pixels.
- These two fields are not part of `Snapshot`, so they do not take part in undo/redo.

## Relationship to other modules

- `Document` holds these two fields.
- The Properties panel in `op-ui` iterates `ColorMode::ALL` and `BitDepth::ALL` to show options; the document tab uses `short()` and `bits()`.

## Known limitations

- Only the RGB color mode is implemented; the other 6 color modes are just enum values with no corresponding pixel representation or conversion.
- Pixel storage is 8-bit only; `U16` and `F32` have no corresponding storage implementation.

## Test coverage

This file has no unit tests.
