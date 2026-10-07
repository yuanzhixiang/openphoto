# dialogs/vibrance.rs: Vibrance and Posterize dialogs

## Component responsibilities

The dialogs for Image › Adjustments › Vibrance... and Posterize..., rebuilt after the Photoshop 2026 UXP dialogs (401 × 166 and 401 × 164 pt).

## Vibrance

- Vibrance, Saturation (−100–100, default 0). Labels at (20, 60.5) / (20, 115.5), input boxes at x 215–260, y 48 / 103, tracks at y 84 / 139 (x 20–260, `#737373`, 0 centered). On open, the Vibrance box takes focus with its contents selected.
- "Preview (Opt+P)" at (272, 127); OK and Cancel on the right.

## Posterize

- Levels (2–255, default 4): label at (20, 60.5), input box (220, 48)–(260, 72), track at y 84, mapping linearly from 2 at the left end to 255 at the right end (`uxp::linear_slider`). On open, the input box takes focus with its contents selected.
- "Preview (Opt+P)" at (272, 127).

## Test coverage

- `defaults_and_ranges`; `ui_tests::small_uxp_adjustment_dialogs_apply`; screenshots `vibrance_dialog.png`, `posterize_dialog.png`.
