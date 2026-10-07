# dialogs/gradient_map.rs: Gradient Map Dialog

## Component Responsibilities

The dialog for Image › Adjustments › Gradient Map..., rebuilt as Photoshop 2026's classic dialog (416 × 230 pt). The algorithm is in `op-core`'s `adjust.md` and `gradient.md`.

## Data and Layout

- The gradient has two colors, foreground to background on open (`AdjustDialog::set_gradient_colors`); Dither and Reverse are unchecked by default; Method defaults to Smooth (Perceptual / Linear / Classic / Smooth).
- Group box "Gradient Used for Grayscale Mapping" (11, 46.5)–(309, 97): gradient box (20, 67.5)–(300, 87.5); the gradient bar (21, 68.5)–(286, 86.5) is drawn according to the current method and Reverse; the ⌄ on the right opens a few two-color gradient presets (Foreground to Background, Black, White, White, Black).
- Group box "Gradient Options" (11, 121)–(309, 219): "Dither" (20, 139) and "Reverse" (20, 166) checkboxes, "Method:" (20.5, 200) and the pop-up menu (72, 189.5)–(157, 210.5).
- Buttons x 325.5–385.5: OK (45), Cancel (80); "Preview" (325, 124.5).

## Known Limitations

- The gradient has only two colors; clicking the gradient bar does not open the gradient editor; Dither has no effect.

## Test Coverage

- `reverse_and_method`; `ui_tests::more_adjustments` (Smooth mapping matches the core); screenshot `gradient_map_dialog.png`.
