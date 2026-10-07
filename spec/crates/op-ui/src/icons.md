# icons.rs: Icon mapping

## Responsibilities

Re-exports all Phosphor (Regular) icon constants and provides `tool()`: the mapping from tools to icons. Adobe's icon assets are not used. The toolbar and tool flyouts now use icons drawn by `tool_icons.rs`; `tool()` serves only as a fallback for tools without a drawn icon (Remove Tool).

## Tool icons

`tool()` assigns a Phosphor icon to each of the 70 tools in `op-tools`, choosing the one closest in shape to the Photoshop icon, for example `SELECTION` for Rectangular Marquee, `CIRCLE_DASHED` for Elliptical Marquee, `MAGIC_WAND` for Magic Wand, `PENCIL` for Pencil, `GRADIENT` for Gradient, `ARROW_CLOCKWISE` for Rotate View, `PAINT_BRUSH_BROAD` for Selection Brush and `PAINT_BRUSH_HOUSEHOLD` for Adjustment Brush. Some tools in the same group share an icon (for example all three erasers use `ERASER`).

The icons look different from Photoshop's: Photoshop's tool icons are larger and mostly in a filled style.
