# dialogs/selective_color.rs: Selective Color dialog

## Component responsibilities

The dialog for Image › Adjustments › Selective Color..., rebuilt after Photoshop 2026's UXP dialog (473 × 400 pt). For the algorithm see `adjust.md` in `op-core` (`SelectiveColor`).

## Data and defaults

- Each of the nine color ranges (Reds, Yellows, Greens, Cyans, Blues, Magentas, Whites, Neutrals, Blacks) has Cyan, Magenta, Yellow and Black (−100–100%), default 0; Method defaults to Relative.
- Input fields may include "%", which is stripped when parsing; `adjustment()` returns `Adjustment::SelectiveColor` when everything is valid. Preset rules are the same as in the other UXP dialogs.

## Layout (Photoshop points)

- "Preset" (20, 61), dropdown (56, 48.5)–(302, 73.5), preset menu icon (319, 61).
- Nine dots: centers at x starting from 32 every 36 pt, y 96; filled with Photoshop's colors (white, `#807f7f` gray and black represent Whites, Neutrals and Blacks), with the range name as a hover tooltip.
- Four slider rows: Cyan, Magenta, Yellow, Black, input fields x 277–331.5, y 120 / 175 / 230 / 285, with "%" drawn after the number; tracks are 36 pt below the field, x 20–331.5, colored red→gray→cyan, green→gray→magenta, blue→gray→yellow, white→gray→black (taken from Photoshop).
- "Method" (20, 348.5), radio buttons "Relative" (26, 368.5) and "Absolute" (98, 368.5) (`uxp::radio`).
- "Preview (Opt+P)" (344, 127); OK and Cancel on the right.

## Interaction

- Clicking a dot switches the range, and the Cyan field gets focus with its contents selected; on open, the Cyan field of Reds has focus.

## Test coverage

- `ranges_keep_their_values`; `ui_tests::channel_mixer_and_selective_color_apply` (Yellows, Absolute); screenshot `selective_color_dialog.png`.
