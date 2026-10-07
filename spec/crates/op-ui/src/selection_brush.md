# selection_brush.rs: The Selection Brush tool

## Responsibilities

Photoshop 2026's Selection Brush: painting the selection with a brush, and showing it as a colored overlay while the tool is current. Observed in Photoshop 2026 on the probe document (stroke, history, alert, overlay color and opacity, gear menu).

## Where it is

- First in the Lasso tool group (Selection Brush, Lasso, Polygonal Lasso, Magnetic Lasso), though the slot shows the Lasso until another is picked; shortcut L, Shift+L cycling through the group (see `crates/op-tools/src/lib.md`).
- Options bar (`options_tools.md`): Add / Subtract (`selbrush.mode`), Opacity (the tool's `PaintOptions::opacity`), the brush picker (its `PaintOptions` size, 200 px by default, hard tip), and the gear.
- Its brush options are `AppState::selection_brush`, so `[` / `]`, the number keys and the brush-outline cursor work as for painting tools.

## Painting (`input`)

- Pressing on the canvas starts a `op_core::selection_brush::SelectionStroke` over the current selection (in Subtract mode, or with ⌥ held at the press, it subtracts; ⌥ in Subtract mode adds), and the selection updates live as the pointer moves. Releasing records one "Selection Brush" history state.
- With no layer selected (Layer › Deselect Layers), the press shows Photoshop's alert "Can't use the tool because no layers are selected." (`NO_LAYER`) with Learn More and OK, and nothing is painted. Learn More just closes it.

## Overlay (`draw_overlay`)

- While the Selection Brush is the current tool, the selection is shown as an overlay instead of marching ants (also with Extras off): the overlay color at 50% over fully selected pixels, in proportion for partly selected ones. It is a document-sized texture (nearest-neighbor) rebuilt when the selection or the color changes, drawn through the view's mapping so it follows rotation and flip. Other tools show the same selection with marching ants.
- The gear opens "Overlay Option" with a Color dropdown (`selbrush.overlay`): Blue, Green, Chartreuse, Yellow, Orange, Red, Magenta (default), Purple, Indigo, Fuchsia, Seafoam, White, Black, each with its swatch. Magenta's overlay color (229, 97, 171) was measured on Photoshop's canvas (50% over three known colors); the others use their menu swatch colors.

## Known limitations

- Photoshop's Selection Bar (the contextual task bar shown after painting) is not shown.
- The gear popup is an egui popup, not Photoshop's panel look.
- The other overlay colors are the menu swatches, not measured on the canvas.

## Test coverage

- `selection_brush_paints_the_selection` (UI test): see `ui_tests.md`.
