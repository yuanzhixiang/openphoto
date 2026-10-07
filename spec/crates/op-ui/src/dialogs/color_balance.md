# dialogs/color_balance.rs: Color Balance Dialog

## Component Responsibilities

The dialog for Image › Adjustments › Color Balance..., rebuilt after Photoshop 2026's UXP dialog (447 × 295 pt).

## Data and Defaults

- Shadows, midtones and highlights each have three values (Cyan to Red, Magenta to Green, Yellow to Blue, −100–100, default 0), stored separately; `tone` is the tone being edited, Midtones when opened.
- Preserve Luminosity is checked by default.
- `adjustment()`: returns `Adjustment::ColorBalance` when all nine values are valid.

## Layout

- "Tone" label (20, 60.5); three 24 pt tone dots, centers at x 68 (black), 153 (`#777777`), 240 (white), y 60, with labels starting at x 85.5, 170.5, 258 respectively. The selected dot has a 2 pt white outer ring, a 2 pt `#454545` gap inside it, and the fill color at the center; unselected dots are the fill color with a thin 0.5 pt `#a7a7a7` edge.
- Three rows of sliders: labels at x 20, input boxes at x 249–305.5, y 92 / 140 / 188 (24 high); tracks at y 128 / 176 / 224, x 20–305.5. The track colors are Photoshop's gradients (cyan→gray→red, magenta→gray→green, yellow→gray→blue, 17 color stops each taken from screenshots; the middle is not a linear interpolation).
- "Preserve Luminosity" checkbox (20, 257); "Preview (Opt+P)" (318, 127); OK and Cancel on the right.

## Interactions

- Clicking a tone dot or its label switches the tone: the sliders and input boxes show that tone's values, and the first input box takes focus with its contents selected.
- When opened, Midtones' first input box takes focus with its contents selected.
- Otherwise the same as the UXP dialogs (`uxp.md`).

## Test Coverage

- `ui_tests::color_balance_tones_keep_their_values`: entering 40 for midtones, switching to shadows and entering −30, and unchecking Preserve Luminosity gives a result equal to the corresponding core curves.
- Screenshot comparison: `color_balance_dialog.png`.
