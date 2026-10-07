# op-tools: Tool Definitions

## Responsibilities

Defines all the tools in the Photoshop 2026 toolbar, their grouping in the toolbar, their display names, and their single-key shortcuts. Tool behavior is implemented in `op-ui`; this crate contains only the static definitions and the rules for selecting tools by key. This crate has no dependencies.

## Public Interface

### `Tool`

70 tools, covering every tool in the Photoshop toolbar and in each group's flyout menu, for example `Move`, `Artboard`, `RectangularMarquee`, `EllipticalMarquee`, `SingleRowMarquee`, `SingleColumnMarquee`, `Lasso`, `PolygonalLasso`… `Hand`, `RotateView`, `Zoom`.

### `TOOLBAR`

The toolbar's "slots" from top to bottom. Each slot is a group of tools sharing one button, and the order within a group matches Photoshop's flyout menu:

| Slot | Tool group |
|---|---|
| 1 | Move, Artboard |
| 2 | Rectangular / Elliptical / Single Row / Single Column Marquee |
| 3 | Selection Brush, Lasso, Polygonal Lasso, Magnetic Lasso |
| 4 | Object Selection, Quick Selection, Magic Wand |
| 5 | Crop, Perspective Crop, Slice, Slice Select |
| 6 | Frame |
| 7 | Eyedropper, Color Sampler, Ruler, Note, Count |
| 8 | Spot Healing Brush, Remove, Healing Brush, Patch, Content-Aware Move, Red Eye |
| 9 | Brush, Pencil, Color Replacement, Mixer Brush |
| 10 | Clone Stamp, Pattern Stamp |
| 11 | History Brush, Art History Brush |
| 12 | Eraser, Background Eraser, Magic Eraser |
| 13 | Gradient, Paint Bucket |
| 14 | Blur, Sharpen, Smudge |
| 15 | Adjustment Brush |
| 16 | Dodge, Burn, Sponge |
| 17 | Pen, Freeform Pen, Curvature Pen, Add / Delete Anchor Point, Convert Point |
| 18 | Horizontal / Vertical Type, Vertical / Horizontal Type Mask |
| 19 | Path Selection, Direct Selection |
| 20 | Rectangle, Ellipse, Triangle, Polygon, Line, Custom Shape |
| 21 | Hand, Rotate View |
| 22 | Zoom |

### `Tool::name()`, `Tool::shortcut()`, `Tool::slot()`

- `name()`: the full English name, the same as Photoshop's, for example `"Elliptical Marquee Tool"`.
- `shortcut()`: the single-key shortcut, matching Photoshop. Tools in the same group share the group's letter (V, M, L, W, C, K, I, J, B, S, Y, E, G, O, P, T, A, U, H, Z); Rotate View uses R on its own. Single Row/Column Marquee, the Blur group, Add/Delete Anchor Point, Convert Point, and Adjustment Brush have no shortcut; the Selection Brush shares the Lasso group's L, as in Photoshop 2026.
- `slot()`: the slot the tool is in.
- `default_in_group(group)`: the tool a slot shows before another is picked: the group's first, except the Lasso in its group (Photoshop 2026 lists the Selection Brush first but shows the Lasso).

### `tool_for_key(key, shift, active, current)`

Which tool is selected when a letter key is pressed, following Photoshop's rules:

- `current(slot)` is the tool currently shown in each slot (Photoshop remembers the most recently used one in each group).
- Without Shift: selects the tool currently shown in the slot that has this letter; if that tool does not use this letter, selects the first tool in the group that does.
- With Shift, and the current tool is in this slot: cycles in group order to the next tool that uses this letter, skipping tools without a shortcut.
- With Shift but the current tool is in another slot: same as without Shift.
- When only one tool in a slot uses the letter (for example R, H), it is selected directly.

## Test Coverage

- `every_tool_appears_once`: each of the 70 tools appears exactly once in the toolbar, and the toolbar has 22 slots.
- `keys_select_the_group_and_shift_cycles`: M, ⇧M cycling (skipping the single row/column marquees), slot memory, pressing ⇧M from another slot, handling of R and H, unused letters.
