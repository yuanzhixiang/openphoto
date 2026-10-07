# dialogs/black_white.rs: Black and White Dialog

## Component Responsibilities

The dialog for Image › Adjustments › Black & White... (⌥⇧⌘B), rebuilt after Photoshop 2026's UXP dialog (401 × 568 pt).

## Data and Layout

- Six weights Reds, Yellows, Greens, Cyans, Blues, Magentas (−200–300%, Photoshop defaults 40 / 60 / 40 / 60 / 20 / 80). Labels at x 20, input boxes at x 209–260, y starting at 84 every 55 pt, with "%" drawn after the number; the track is 36 pt below the box, linear from −200 to 300 (not centered on 0), colored black → that color (at the middle of the range) → white. When opened, the Reds box takes focus with its contents selected.
- "Preset:" (20, 61), dropdown (59, 48.5)–(231, 73.5), preset menu icon (248, 61). The menu lists Default, Custom and Photoshop's 12 presets (`adjust_presets::BLACK_WHITE`, Blue Filter … Yellow Filter, none with a tint); picking one sets the six weights (Tint off), and the Preset shows its name while the weights match.
- Tint: checkbox (20, 420.5) and swatch (72.5, 414)–(120, 437.5); Hue (0–360°, default 42) and Saturation (0–100%, default 20), input boxes at y 450 / 505, tracks at y 486 / 541. When Tint is unchecked, these two rows are dimmed and disabled. The Tint color is HSB (hue, saturation, brightness 88.2%); `tint_color(42, 20)` = (225, 211, 180).
- OK, Cancel and Auto on the right. Auto sets the six weights from the layer's pixels (`op_core::auto::black_white` on `AdjustDialog::extra.auto_samples`, gathered on open; the tint is kept); "Preview (Opt+P)" (272, 163).

## Test Coverage

- `weights_and_tint`; `ui_tests::black_and_white_auto_picks_weights_from_the_image` (red and blue halves end up further apart in gray); `ui_tests::small_uxp_adjustment_dialogs_apply`; screenshot `black_white_dialog.png`.
