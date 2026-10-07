# dialogs/trim.rs: Image › Trim dialog

## Component responsibilities

Chooses what Trim is based on and which edges to trim away; on OK, `lib.rs` runs `image_ops::trim` (for the rules see `op-core`'s `image_ops.md`).

## Layout and visuals

Measured point by point from Photoshop 2026's Trim dialog (2x screenshots), the same kind as New Layer and Canvas Size (UXP style). The dialog is 258 × 248 pt; coordinates are pt from the dialog's top-left:

- Title bar: `common::frame`, system bold 13 pt "Trim". Text is the 12 pt panel font (Source Sans 3 approximating Adobe Clean), `#f1f1f1`.
- "Based on" bold heading, left end at 20, centered at y 57. Three radio options (sentence case, matching Photoshop 2026): Transparent pixels, Top left pixel color, Bottom right pixel color, centered at (26, 84 / 108 / 132), 12 pt in diameter, with text 15 pt to the right of the center. Selected is a solid `#d6d6d6` circle with a dark dot; unselected is a 1 pt `#a0a0a0` ring; when not selectable, the ring and text are dimmed.
- "Trim away" bold heading, centered at y 166. Four checkboxes (`common::ps_checkbox`, 12 pt): Top (20, 186), Left (97, 186), Bottom (20, 210), Right (97, 210).
- OK (default button) (168–238, y 48–72), Cancel (y 84–108): `common::ps_button`.

Compared with Photoshop screenshots, each element differs by no more than 1 pt.

## Data input and defaults

- `TrimDialog::new(has_background)`: when the document has a background layer, Transparent pixels is grayed out and Top left pixel color is selected by default; without a background layer, Transparent pixels is selected by default. All four edges are checked by default. Defaults are taken again on each open; the last choice is not remembered.

## Interaction

- OK or Enter: returns `Outcome::Apply { basis, sides }`.
- Cancel or Esc: returns `Outcome::Cancel`.
- Modal while open (`AppState::modal_open()` is true).

## Known limitations

- Photoshop remembers the last choice; here the defaults are restored each time.
