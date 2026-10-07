# dialogs/canvas_size.rs: Canvas Size dialog

## Component responsibilities

Image › Canvas Size... (⌥⌘C): changes the canvas size without scaling the image, matching Photoshop's dialog of the same name. The dialog only collects parameters; the actual change is made by `lib.rs` calling `Document::resize_canvas`, which also records history.

## Layout

Measured point by point from Photoshop 2026's Canvas Size dialog (2x screenshots); same family as New Layer (UXP style). The dialog is 458 × 372 pt; coordinates are in pt from the dialog's top-left corner:

- **Title bar**: same as New Layer (`common::frame`, system bold 13 pt "Canvas Size").
- Text is 12 pt panel font (Adobe Clean; Source Sans 3 here), color `#f1f1f1`, `#8e8e8e` when unavailable.
- **Current Size: 13.5K**: bold, left edge 20, center y 57.5; followed 8 pt later by a 1 pt `#737373` line running to x 356. Below it, Width and Height rows (labels right-aligned at 55, center y 81 and 102.5), values starting at 64, e.g. "64 px".
- **New Size: 13.5K**: bold, center y 130.5, with the same line; updates live with input, and shows "—" when the input is invalid.
- Width and Height input fields (64–145, y 145–169 and 174–198; labels right-aligned at 55), unit dropdowns (153–313, same two rows; text indented 9 pt from the left, dropdown arrow 13.5 pt from the right edge).
- "Relative to current dimension" checkbox: 12 pt box at (64, 216), text 9.5 pt to the right of the box.
- "Anchor": left edge 20, center y 255. Anchor grid: 3 × 3, 23 pt per cell, 1 pt `#787878` lines, top-left corner (64, 246). The anchor cell draws a 7 pt diameter dot; arrows are 13.25 pt long, with an arrowhead 7.75 pt long and 7.5 pt wide and a 2 pt shaft, centered on the cell center.
- "Canvas extension color": left edge 20.5, center y 341; dropdown (140–300, y 328–352); swatch (308–356, y 328–352, 3 pt corner radius, 1 pt `#8e8e8e` border, showing the current extension color). When there is no background layer the text dims, the dropdown is drawn as a borderless flat `#5c5c5c` box with `#8e8e8e` text and arrow, and it cannot be clicked.
- **OK** (default button, bold) (368–438, y 48–72) and **Cancel** (368–438, y 84–108): `common::ps_button`.

Compared with Photoshop screenshots, element positions differ by no more than 1 pt; text is slightly wider because of the different font (about 3% on long sentences).

## Size display

Computed the way Photoshop does: width × height × number of color channels (3 for RGB) × bytes per channel, in units of 1024. Below 1M it shows as `263.7K` (one decimal), otherwise as `1.70M` (two decimals), and from 1G as `1.25G`.

## Numeric input

- Units: Pixels, Percent, Inches, Centimeters, Millimeters, Points, Picas. Percent is relative to the original size; physical units convert using the document resolution (1 inch = 2.54 centimeters = 72 points = 6 picas).
- When switching units, the value in the input field is converted to the new unit and the size it represents stays the same. Display precision: integers for pixels, at most 2 decimals for percent, at most 3 decimals otherwise, with trailing zeros removed.
- With "Relative to current dimension" checked, the input fields express an increase or decrease (they become 0 the moment it is checked); unchecking switches back to absolute values with the resulting size unchanged.
- The result is rounded to whole pixels. It must be between 1 and 30000 pixels (Photoshop's limit for regular documents); when the input cannot be parsed or is out of range, New Size shows "—" and OK is grayed out.

## Anchor

- Defaults to the center. Clicking a cell moves the anchor to that cell; the anchor cell shows a dot and adjacent cells show arrows.
- Arrow direction: points outward when the canvas grows (or stays the same) in that direction, and toward the anchor when it shrinks, matching Photoshop; horizontal and vertical are judged separately.
- With the anchor in the middle and an odd size difference: when enlarging the canvas, the extra 1 pixel goes on the right/bottom; when shrinking, the extra 1 pixel cropped is on the left/top (see `crates/op-core/src/document.md`).

## Canvas extension color

- Options: Foreground, Background (default), White, Black, Gray (50% gray, 128), Other.... The swatch shows the color for the current choice.
- Choosing "Other..." or clicking the swatch opens the Color Picker (title "Color Picker", starting at the current extension color, see `color_picker.md`). After confirming in the Color Picker, the extension color switches to Other... and uses the chosen color; canceling keeps the previous choice.
- While the Color Picker is stacked on this dialog, Enter and Esc only act on the Color Picker.
- Only available when the document has a background layer; without one, the dropdown and swatch are grayed out and the text dims. The extension color only fills the new area of the background layer; new areas of other layers are transparent. When shrinking the canvas, the excess is simply cropped.

## Keyboard and focus

- On open, the width input field gets focus with its contents selected, matching Photoshop. The focused input field has a blue outline `#1473e6`.
- Enter: same as OK (when the input is valid). Esc: same as Cancel.
- Menus and shortcuts are disabled while the dialog is open.

## Result

- OK: returns the new size, anchor and extension color. When the size is the same as the current one, nothing happens and no history is created; otherwise the canvas is changed and a "Canvas Size" history entry is recorded, which can be undone.
- Cancel: makes no changes.

## Known limitations

- Does not remember the last settings; every time it opens it resets to defaults (center anchor, Background, Pixels).
- The anchor can only be set by clicking with the mouse, not moved with the keyboard.
- The dialog cannot be dragged.
