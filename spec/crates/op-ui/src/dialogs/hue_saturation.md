# dialogs/hue_saturation.rs: Hue/Saturation dialog

## Component responsibilities

The dialog for Image › Adjustments › Hue/Saturation... (⌘U), rebuilt after Photoshop 2026's new UXP dialog (437 × 413 pt). See `adjust.md` in `op-core` for the algorithm.

## Data and defaults

- Master and the six color ranges (Reds, Yellows, Greens, Cyans, Blues, Magentas) each have three values, Hue (−180–180), Saturation and Lightness (−100–100), defaulting to 0; each range's boundaries default to Photoshop's `HUE_RANGES`.
- Colorize has its own three values: Hue (0–360, the foreground color's hue on open), Saturation (0–100, 25), Lightness (−100–100, 0). When Colorize is checked these three values are used; when unchecked, the previous Master and range settings are restored.
- `adjustment()`: returns `Adjustment::HueSaturation` when all current values are valid.
- Preset: shows "Default" when nothing has been changed, otherwise "Custom"; Default in the dropdown restores all values and ranges to their defaults (the Colorize values are kept). The menu also lists Photoshop's presets (`adjust_presets::HUE_SATURATION`: Cyanotype, Increase Saturation, Increase Saturation More, Old Style, Red Boost, Sepia, Strong Saturation, Yellow Boost): Cyanotype and Sepia turn on Colorize with their values, the others set Master's values (everything else at its default); the Preset shows a preset's name while the values match it.

## Layout

- "Preset" label at (20, 61), dropdown (56, 48.5)–(266, 73.5), and the preset menu icon to its right at (284, 61) (drawn only).
- At (32.5, 95) is the targeted adjustment hand icon (drawn only, no behavior).
- Seven 24 pt dots, centers at y 96, x starting at 68 and every 36 pt: Master is a color wheel divided into eight segments, the rest are Photoshop's range colors; the selected one is drawn as in Color Balance. With Colorize, only one selected dot remains, filled with the Colorize color (that hue and saturation, lightness 50%).
- Three slider rows: labels at x 20; input boxes at x 251–295.5, y 120 / 175 / 230; tracks at y 156 / 211 / 266, x 20–296. Track colors: Hue is a hue ring, Saturation goes gray → color, Lightness goes black → gray → white (all taken from Photoshop screenshots); with Colorize the Hue track starts at 0 on the left end, and Saturation goes from `#767676` to the pure color of that hue.
- When a range is selected, the hue at the range center is shown to the left of the Hue input box (e.g. "0°" for Reds).
- "Colorize" checkbox at (20, 292); three eyedroppers at (123.5 / 160 / 196, 298) and the "Invert" button (233, 286)–(296, 310), all drawn disabled.
- "Before - After" label at (20, 331); color bar (20, 343)–(296, 356): the upper half shows the original hues (cyan at the left end, red in the middle), the lower half the colors after applying the current settings; below it at y 364–366 is a `#a0a0a0` horizontal line.
- For Master or Colorize, "0° / 0°" is shown below the line on the left and right in the disabled color (`#8e8e8e`). When a range is selected, a range bar is drawn on the line: the two falloff zones are 8 pt tall `#7a7a7a`, the full-effect zone is 14 pt tall `#a0a0a0`, with four white handles (outer ones 2 pt wide, inner ones 3 pt wide, 14 pt tall); below it on the left and right are "start°/full-strength start°" and "full-strength end°/end°", truncated with "..." when they do not fit.
- On the right, OK (drawn as focused when no input box holds focus) and Cancel; "Preview (Opt+P)" at (308, 127).

## Interaction

- Clicking a dot switches between Master and the ranges; the sliders and input boxes show that item's values.
- Range bar: pressing within 4 pt of a handle drags that boundary (without crossing adjacent boundaries); dragging anywhere else moves the whole range.
- Checking or unchecking Colorize returns to Master.
- Everything else is the same as for UXP dialogs (`uxp.md`).

## Known limitations

- The eyedroppers (sampling color on the document to pick a range, adding to/subtracting from the range), Invert, the targeted adjustment hand and the preset menu have no behavior; in Photoshop the eyedroppers and Invert are available when a range is selected.
- The preset list has only Default / Custom.

## Test coverage

- `values_ranges_and_preset`: each range's own values, invalid values disabling OK, the Colorize values and the foreground hue, Preset Default/Custom.
- `the_strip_puts_red_in_the_middle`: on the color bar, 0° is at the midpoint and 180° at the left end.
- `ui_tests::hue_saturation_ranges_and_colorize`: selecting Reds and entering hue 60 applies to a red document; applying again with Colorize, the result equals the core algorithm. Screenshots `hue_saturation_reds.png`, `hue_saturation_colorize.png`.
