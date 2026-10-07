# dialogs/plain_filter.rs: Filter dialogs without a preview

## Responsibilities

The layouts of Photoshop 2026's small filter dialogs that have no preview: Stylize › Tiles... and Pixelate › Color Halftone.... They are drawn by `AdjustDialog::plain_ui` (`adjust.md`): the title bar, the items in order, and plug-in style OK and Cancel (89 × 26 pt, 13 pt labels, at y 41 and 77 from `buttons_x`). Measured on Photoshop 2026; labels, units and radio labels are 11 pt (`distort::label_font`), field values 12 pt; every label and field was checked within half a point.

## Items (`Item`)

- `Text`: a label at (left x, center y).
- `Field`: a label (left x) and an optional unit (left x), both on the field's center line, and the number field (`appkit::field`); the first field is focused with its value selected when the dialog opens.
- `Radios`: one button per choice at the given centers, labels `gap` pt after the center (`appkit::radio_with`).
- Every item but `Text` sets the next of the filter's settings (`adjust.md`'s parameter table).

## Layouts

| Dialog | Size | Buttons x | Items |
| --- | --- | --- | --- |
| Tiles (`TILES`) | 302 × 220 | 195.5 | "Number Of Tiles:" (x 17) field 117–146 × 44–65; "Maximum Offset:" field 118–147 × 81–102, "%" at 156; "Fill Empty Area With:" (23, 116.75); radios at x 21, y 134 / 154 / 174 / 194, labels 13 pt after |
| Color Halftone (`COLOR_HALFTONE`) | 326 × 243 | 219.5 | "Max. Radius:" (x 9) field 87–150.5 × 36.5–56.5, "(Pixels)" at 160.5; "Screen Angles (Degrees):" (13, 76); "Channel 1:"–"Channel 4:" (x 17) fields 83.5–146.5 at y 95.5, 132.5, 169.5, 206.5 (20 tall) |

## Behavior

- These dialogs don't preview on the document (`AdjustDialog::preview` starts off for them), as in Photoshop; OK or Enter applies, Esc or Cancel closes.

## Test coverage

- `crystallize_pointillize_and_diffuse_apply` (UI test) also applies both and checks the document is untouched while their dialogs are open; `screenshot_pixelate_and_diffuse_dialogs` (`#[ignore]`) captures them.
