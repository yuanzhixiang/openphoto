# dialogs/levels.rs: Levels dialog

## Component responsibility

The dialog for Image › Adjustments › Levels... (⌘L), rebuilt pixel by pixel after Photoshop 2026's classic dialog (412 × 371 pt; text and controls are within 1–2 pixels of Photoshop screenshots). The algorithm is in `adjust.md` in `op-core` (`Levels`, `levels_gamma`).

## Data and defaults

- Four sets of levels: RGB (composite), Red, Green, Blue, each with input black point (0–253), gamma (0.01–9.99), input white point (2–255), output black point, and output white point (0–255), defaulting to 0 / 1.00 / 255 / 0 / 255. Defaults are restored every time the dialog opens.
- `new(channels)`: histograms of the red, green, and blue channels (`adjust::rgb_histograms`); the composite histogram is the sum of the three.
- `adjustment()`: returns `Adjustment::Levels` when all four sets are valid and in each set the black point is at least 2 less than the white point.
- Preset: "Default" when unchanged, the name of the Photoshop preset the values match (`adjust_presets::LEVELS`: Darker, Increase Contrast 1–3, Lighten Shadows, Lighter, Midtones Brighter, Midtones Darker), otherwise "Custom". The native menu lists Default, Custom (grayed out), a separator and the presets, the current one checked; picking one sets all four channels' values.

## Layout (Photoshop points)

- "Preset:" (11, 50), pop-up menu (59, 39.5)–(266, 60.5), gear (283.5, 50) (drawn only).
- Channel group box (11, 90.5)–(295.5, 359.5), broken at the title; "Channel:" (30.5, 90.5), pop-up menu (85, 80)–(206.5, 101).
- "Input Levels:" (20.5, 118.5); histogram (25, 131)–(281, 230), `#454545` background, a 1 pt wide `#d0d0d0` vertical bar per level, normalized to the maximum count; shows the current channel's histogram.
- Input sliders: tips at y 233, with 0 and 255 at x 25.75 and 279.25 respectively; three sliders: black (hollow), gray, white. The gray slider sits between the black and white sliders at `0.5^gamma`.
- Input fields y 249–268: x 60–105, 150–195, 240–285.
- Separator at y 277, x 20–286, `#3e3e3e`.
- "Output Levels:" (20.5, 290.5); black-to-white gradient bar (24, 302)–(282, 315); output slider tips at y 315; output fields y 332–351, x 60–105, 240–285.
- Right-side buttons x 313.5–402.5: OK (38.5), Cancel (73.5), Auto (115.5), Options... (157.5), each 26 high.
- Three eyedroppers (327 / 357 / 387, 213): Set Black, Gray and White Point (`appkit::eyedropper`, the chosen one pressed; clicking it again turns it off); "Preview" checkbox (312.5, 243).
- With an eyedropper chosen, a click on the image outside the dialog samples the merged image as it was before adjusting (`sample`): Black (White) Point sets each channel's input black (white) to that channel's value (kept off the other end), Gray Point sets each channel's gamma so the color comes out at its channels' mean (neutral); the composite channel returns to its defaults.

## Interaction

- On open, the input black point field gets focus with everything selected.
- The Channel menu or ⌥2–⌥5 switches between RGB, Red, Green, Blue; each channel keeps its own values.
- Dragging sliders: pressing in a slider's row selects the nearest slider; the black point does not exceed the white point − 2, and the white point does not go below the black point + 2; the gray slider's position is converted back to gamma (two decimals).
- Auto: computed by `op_core::auto` from the layer's pixels (`AdjustDialog::extra.auto_samples`, gathered on open) with the Auto Color Correction Options in effect (`extra.auto_options`: the saved defaults, else Photoshop's Enhance Brightness and Contrast). All values return to defaults first; when the three channels come out identical (Monochromatic, Brightness and Contrast without snapping, with neutral targets) the values go on the composite RGB channel, otherwise on Red, Green and Blue (input black, gamma with two decimals, input white, output black and white from the target colors).
- Options... opens Auto Color Correction Options (`auto_options.md`); its OK replaces `extra.auto_options` and runs Auto with them at once.
- Enter / OK applies, recording "Levels"; Esc / Cancel cancels.

## Known limitations

- The gear menu (save/load presets) has no behavior.
- Changing Auto Color Correction Options does not preview live in the document while that dialog is open (Photoshop updates the preview as you choose); it takes effect on its OK.

## Test coverage

- `channels_keep_their_levels`, `auto_fills_the_composite_or_each_channel`; `ui_tests::auto_color_correction_options_from_levels`, `photoshops_presets`; `ui_tests::levels_and_curves_eyedroppers_sample_the_image` (see `curves.md`).
- `ui_tests::levels_dialog_sets_the_black_point`, `ui_tests::levels_and_curves_edit_one_channel` (after ⌥3 only red changes). Screenshots `levels_dialog.png`, `levels_red.png`.
