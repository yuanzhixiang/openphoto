# dialogs/appkit.rs: Controls for the classic adjustment dialogs

## Component responsibilities

Controls shared by the classic adjustment dialogs that Photoshop 2026 draws with AppKit (Levels, Curves); sizes are measured from Photoshop's 2x screenshots (in Photoshop points).

## Controls

- `font()` / `text` / `label`: 12 pt system font (`theme::dialog`, laid out with macOS letter spacing), `#f0f0f0`; disabled text `#8e8e8e`. A `label`'s position is the left-middle point of its capital letters.
- `field(ui, rect, value, id, range, step, decimals, focus)`: a numeric field with a `#454545` background and a 1 pt `#666666` border, the number inset 4 pt from the left; with focus, ↑/↓ step by `step` (Shift multiplies by 10), clamped to `range` and formatted with `decimals` decimal places. With `focus`, it gets focus with its contents selected.
- `popup(ui, rect, id, value, menu)`: a pop-up menu button with a `#454545` background, `#666666` border and 2 pt corner radius, text inset 8.5 pt from the left, and a `Caret` 7.75 pt from the right; slightly brighter on hover. Clicking opens `menu`.
- `button(ui, rect, label, default, enabled)`: a 26 pt tall pill button; the default button's border is `#e1e1e1`, the others' `#7d7d7d`, and a disabled button's is `#606060` with dimmed text; the text sits 1 pt above the geometric center (matching Photoshop). `button_with(ui, rect, label, (default, enabled), size, lift)` lets the caller set the text size and upward offset: the buttons of plug-in-style distort dialogs use 13 pt, centered (`distort.md`).
- `checkbox(ui, min, label, checked)`: a 13 pt checkbox; when checked, a `#d4d4d4` background with a dark gray (`#323232`) check mark; when unchecked, a `#454545` background with a `#a0a0a0` border; the label is 10.5 pt to the right.
- `radio(ui, center, label, chosen)`: a radio button; selected is a 13 pt light gray disc with a 5 pt dark dot, unselected is a `#474747` disc with a `#848484` ring; the label is 16.5 pt to the right of the center. Returns whether it was clicked.
- `group(painter, rect, gap)`: a group box with a 1 pt `#424242` frame whose top edge is broken between the two x values of `gap` to make room for the title.
- `pin(painter, tip, kind)`: the slider "pin" below a histogram or gradient bar: a 12 × 10.5 pt house shape (pointed top, rounded bottom corners) with a 1 pt near-black outline; `Black` is hollow, `Gray` is filled with `#a0a0a0`, `White` with `#e6e6e6`.

- `radio_with(ui, center, label, chosen, (gap, font))`: `radio` with the label's gap after the center and its font chosen (the plain filter dialogs). A radio button's id includes its center, so buttons with the same label in different groups (HSB/HSL's two RGB / HSB / HSL columns) stay apart.
