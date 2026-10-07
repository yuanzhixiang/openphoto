# dialogs/new_guide.rs: View › Guides › New Guide... dialog

## Component responsibilities

Creates a new guide from numeric values. On confirmation, `lib.rs` adds it to the current document and records "New Guide".

## Layout

Measured point by point from Photoshop 2026's "New guide" dialog (UXP style, 2x screenshot); the dialog is 390 × 188 pt, and coordinates are in pt from the dialog's top-left corner:

- Title bar: `common::frame`, system bold 13 pt "New guide" (lowercase g, matching Photoshop 2026), or "Edit Guide" when it edits a selected guide (`NewGuideDialog::editing(guide, index)`, filled in with the guide's orientation and position; `editing` tells `lib.rs` to change that guide instead of adding one). Text is in the 12 pt panel font (`uxp::font`).
- "Orientation" in bold, left edge 20, center y 57; two radio options side by side: Horizontal (circle center 26, 84) and Vertical (circle center 108, 84), `uxp::radio`.
- "Position" right-aligned at 59.5, center y 120.5; input field (68–161, y 108–132), default "0 px", text indented 11.5 pt from the left.
- "Color" right-aligned at 59.5, center y 156.5; dropdown (68–228, y 144–168): a 16 pt color square (`#a9a9a9` border) plus the color name; the options are Photoshop's guide colors Cyan (default), Light Blue, Light Red, Green, Medium Blue, Yellow, Magenta, Gray, Black. On the right (240–288) is the white swatch for a custom color.
- OK (300–370, y 48–72) is the default button holding keyboard focus (`common::ps_focused_button`: `#737373` background, white border, blue focus ring), Cancel (y 84–108). When opened, the input field does not have focus, matching Photoshop.

Compared with the Photoshop screenshot, each element differs by no more than 1.5 pt.

## Interaction

- Clicking a radio option switches the orientation; the default is Horizontal.
- Position can be typed with a "px" suffix. The range is −30000–30000; when out of range or not a number, OK / Enter does nothing.
- Enter confirms, Esc cancels. It is modal while open.

## Known limitations

- The Color choice has no effect yet: all guides are drawn in the same color; the custom color swatch cannot be clicked.
- Only pixel units are available.
- Photoshop remembers the last orientation (Vertical in the screenshot); here it resets to Horizontal every time.

## Test coverage

- `position_accepts_a_px_suffix`: "120 px" parses to a vertical guide at 120; non-numbers are invalid.
- `new_guide_dialog_adds_a_guide` in `ui_tests.rs`: selecting Vertical, typing "100 px" into Position, and pressing Return adds a vertical guide at position 100 to the document. `screenshot_new_guide_dialog` (`#[ignore]`) captures the dialog for comparison with Photoshop.
