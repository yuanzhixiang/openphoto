# selection_brush.rs: Painting a selection

## Responsibilities

The pixel side of Photoshop 2026's Selection Brush tool: a brush stroke that adds to or takes from the selection it started from. The tool's input and display live in `crates/op-ui/src/selection_brush.md`.

## `SelectionStroke`

- `new(width, height, current, tip, opacity, subtract)`: starts a stroke over the document's current selection (`None`, or one of another size, counts as nothing selected), with a `paint::BrushTip`, the bar's Opacity (0–1) and the mode.
- `add_point(x, y)`: like a paint stroke, the first point places one dab and later points place dabs every quarter of the diameter along the way (Photoshop's default spacing).
- Each dab records, per pixel, the largest tip coverage (`BrushTip::alpha`, the same falloff and one-pixel anti-aliased edge as painting) this stroke has reached, so overlapping dabs don't build up beyond the opacity, as with a brush at full flow.
- `selection()`: per pixel, with c = coverage × opacity and b the starting value: Add gives b + (255 − b) × c, Subtract gives b × (1 − c), rounded. `None` when nothing is left selected.

## Test coverage

- `painting_adds_and_subtracts`: a stroke selects along its path and its round ends; a subtracting stroke across it splits it; a stroke larger than everything leaves no selection.
- `opacity_selects_partly_and_strokes_dont_build_up`: at 50% opacity, going over the same spot many times leaves it at 128.
