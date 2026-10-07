# dialogs/threshold.rs: Threshold dialog

## Component responsibilities

The dialog for Image › Adjustments › Threshold..., redone after Photoshop 2026's classic dialog (381 × 191 pt).

## Data and layout

- Threshold Level (1–255, default 128). "Threshold Level:" right-aligned to x 171.5, y 47.25; input field (177, 38)–(213.5, 57) (`appkit::field`), which takes focus and selects all when opened.
- Luminosity histogram (15, 66)–(270, 166), `#454545` background, `#d0d0d0` vertical bars, normalized by the maximum count (the luminosity histogram within the active layer's selection at the time of opening).
- Below it a single white slider (tip at y 167.5), with 0 and 255 at x 14.25 and 268.75; pressing or dragging in the slider's row sets the level.
- Buttons at x 290–370.5: OK (38.5), Cancel (73.5); "Preview" checkbox (290, 118).

## Test coverage

- `level_range`; `ui_tests::adjustments_with_dialogs`; screenshot `threshold_dialog.png`.
