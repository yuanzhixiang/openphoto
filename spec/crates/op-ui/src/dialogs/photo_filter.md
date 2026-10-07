# dialogs/photo_filter.rs: Photo Filter dialog

## Component responsibilities

The dialog for Image › Adjustments › Photo Filter..., rebuilt after Photoshop 2026's UXP dialog (401 × 217 pt).

## Data and layout

- Filter radio (26, 60.5) and dropdown (78.5, 48.5)–(260, 72.5): Photoshop's 20 presets (`FILTERS`, default Warming Filter (85)); choosing a preset returns to Filter mode.
- Color radio (26, 94.5) and swatch (79.5, 82.5)–(127, 105.5): shows the current color; clicking it opens Photoshop's Color Picker ("Color Picker (Photo Filter Color)", `PickerTarget::PhotoFilter`, via `AdjustDialog::take_color_request`); its OK sets the color and switches to Color mode (`set_filter_color`). Meanwhile the filter dialog is drawn but takes no keys (`AdjustDialog::blocked`).
- "Density" (20, 131.5), input field (213, 118.5)–(260, 142.5) showing "25%"; the slider at y 154 is drawn linearly over 0–100% (values below 1% are not accepted).
- "Preserve luminosity" checkbox (20, 179.5), checked by default; "Preview (Opt+P)" (272, 127); OK and Cancel on the right. No input field has focus on open.

## Known limitations

- When Photoshop opens it, the Filter radio has the keyboard focus ring.

## Test coverage

- `filter_or_color`; `ui_tests::small_uxp_adjustment_dialogs_apply`; `ui_tests::photo_filter_picks_its_color_in_the_color_picker` (the swatch opens the Color Picker, a typed 00ff00 and OK give Color with green, the dialog still open); screenshot `photo_filter_dialog.png`.
