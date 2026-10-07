# dialogs/exposure.rs: Exposure dialog

## Component responsibilities

The dialog for Image › Adjustments › Exposure..., rebuilt after Photoshop 2026's UXP dialog (401 × 257 pt). For the algorithm see `op-core`'s `adjust.md`.

## Data and layout

- Exposure (−20–20), Offset (−0.5–0.5), Gamma Correction (0.01–9.99), defaulting to 0, 0, 1; values are displayed with trailing zeros removed, and Gamma is displayed as "+1" (as Photoshop does).
- "Preset" (20, 61), dropdown (56, 48.5)–(231, 73.5) (Default, Custom, then Photoshop's Minus 1.0, Minus 2.0, Plus 1.0, Plus 2.0 from `adjust_presets::EXPOSURE`; positive values are shown with "+"), preset menu icon (248, 61).
- Three rows: labels at y 97 / 152 / 207, input fields at x 201–260, y 84 / 139 / 194, tracks 36 pt below the fields. The Exposure and Offset sliders are centered on 0 (step 0.01, 0.0001); Gamma is linear from 0.01 to 9.99 (step 0.01).
- Three eyedroppers (283.5 / 319.5 / 356, 133): the selected one (the white point when opened) is drawn on a `#d8d8d8` rounded background; the eyedroppers themselves cannot sample from the document.
- "Preview (Opt+P)" (272, 162); OK and Cancel on the right. When opened, the Exposure field gets focus with its contents selected.

## Test coverage

- `defaults_and_numbers`; `ui_tests::small_uxp_adjustment_dialogs_apply`; screenshot `exposure_dialog.png`.
