# dialogs/common.rs: shared dialog parts

## Responsibilities

Appearance parts shared by the modal dialogs, ensuring all dialogs look consistent.

## Parts

- `frame(ui, rect, title, font)`: draws the dialog frame: shadow, `#535353` background, corner radius 10, dark outer border, and a light gray title bar 28 pt high at the top (`#d0d2d4`, dark centered title, with a divider below). It corresponds to the macOS window title bar of Photoshop dialogs. The title sits 1.5 pt above the exact middle of the title bar, matching the dialogs of Photoshop 2026 (the title ink of Duplicate Layer, New Layer, and Lock Layers all starts at y 8 pt); the title uses the font given by the caller (always a 13 pt bold system font) and is laid out with AppKit's tracking (`theme::tracked_galley`).
- `pill_button(ui, rect, label, font, enabled)`: a pill-shaped button. Light border (`#d0d0d0`, 1.5), background the same as the dialog, lighter on hover, darker when pressed; when disabled, the border and text are grayed out and it does not respond to clicks. Button text must be unique within one dialog (it is used as the interaction id).

- `text_field(ui, rect, text, id, font, pad, select_all)`: a text input field in a dialog: `#454545` background, `#777777` border, corner radius 3; when focused, a 2-pixel blue `#1473e6` ring surrounds it. The font and left padding are given by the caller; the right padding is fixed at 2 pt. With `select_all`, it takes focus and selects all text (the first input field when the dialog opens).

## Users

Canvas Size (`canvas_size.md`), Fill (`fill.md`), and Color Picker (`color_picker.md`).

## Photoshop 2026 new-style dialog controls

Measured from the New Layer dialog, used by dialogs such as New Layer and Duplicate Layer (for fonts see `theme.md`):

- `ps_dropdown(ui, rect, id, content, menu)`: a dropdown with corner radius 3, a 1 pt `#7a7a7a` border, and a panel-colored background (lighter on hover); the content is drawn by `content`, and 13.5 pt from the right edge there is a 10 × 6 pt V-shaped arrow; clicking opens `menu`.
- `ps_checkbox(ui, min, label, checked, enabled)`: a checkbox 12 pt square with corner radius 2.5; unchecked it has a 1 pt `#a0a0a0` border, checked it has a light gray background with a dark check mark; the text (`theme::uxp` 12 pt, the font of UXP dialogs like New Layer) is 9.5 pt to the right of the box, and clicking either the box or the text toggles it; when unavailable, the box and text are `#8e8e8e`. `ps_checkbox_with(..., font, label_dy)` can change the font and move the text down (Lock Layers uses the panel font, moved down 1 pt).
- `ps_button(ui, rect, label, default, enabled, bold)`: a pill-shaped button with a 1 pt border: `#f1f1f1` for the default button, `#727272` for others, `#5e5e5e` when unavailable; with `bold` the label is `theme::uxp_bold` 12 pt (UXP dialogs like New Layer), otherwise the AppKit system font 12 pt (classic dialogs like Duplicate Layer). `ps_button_with(..., font)` changes the label font.
- `ps_focused_button(ui, rect, label, font)`: the default button that has keyboard focus, appearing in dialogs without an input field (Lock Layers): `#737373` background, 1 pt white border, and, separated by 1 pt outside it, a 2 pt blue `#2d63cb` focus ring (measured in Photoshop 2026; it looks like this even when the mouse is not over the button).
- `field_popup(ui, rect, id, text, enabled, entries) -> Option<usize>`: `field_dropdown`'s look, opening `entries` as a native menu (`native_popup.md`); Image Size's dropdowns use it.
- `field_dropdown(ui, rect, id, text, enabled, menu)`: a dropdown that looks like an input field in AppKit-style dialogs (Duplicate Layer): `#454545` background, 1 pt `#666666` border (`#4e4e4e` / `#5e5e5e` when unavailable); the value is in the 12 pt system font, indented 8.5 from the left, truncated with `elide` and "..." when too long, and 7.5 from the right edge there is a V-shaped arrow.
