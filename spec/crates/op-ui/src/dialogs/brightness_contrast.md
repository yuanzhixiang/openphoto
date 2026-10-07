# dialogs/brightness_contrast.rs: Brightness/Contrast dialog

## Component responsibility

The dialog for Image › Adjustments › Brightness/Contrast..., rebuilt pixel by pixel after Photoshop 2026's UXP dialog (401 × 212 pt, within 1 pixel of Photoshop screenshots). It only manages the settings; preview and apply are handled by the host in `adjust.rs` and by `lib.rs`.

## Data and defaults

- `brightness` (−150–150) and `contrast` (−50–100) are field text, defaulting to 0; `legacy` (Use Legacy) is unchecked by default. Defaults are restored every time the dialog opens.
- `adjustment()`: returns `Adjustment::BrightnessContrast` when both values are valid, otherwise `None` (OK grayed out, no preview). Decimals are rounded.

## Layout (Photoshop points, from the dialog's top-left corner)

- "Brightness" label at x 20, field (203, 48)–(260, 72), slider track at y 84, x 20–260.
- "Contrast" label, field (203, 103)–(260, 127), slider at y 139. The slider's 0 is at the midpoint of the track (Contrast's negative half only goes to −50). The track is `#737373`.
- "Use Legacy" checkbox (20, 164); "Preview (Opt+P)" checkbox (272, 174).
- Right-side button column: OK, Cancel, Auto (`uxp::buttons`).
- On open, the Brightness field gets focus with everything selected.

## Interaction

- The fields accept typing, with ↑/↓ stepping by 1 (Shift 10); dragging or clicking a slider changes the value.
- Auto: picks brightness and contrast from the layer histogram. Photoshop's Auto rule could not be reproduced (for example, for an even 0–255 gray ramp it gives 14 / −19, and for 128–255 it gives 0 / 0), so here the pair on Photoshop's brightness/contrast curves that comes closest to the Auto Contrast stretch (clipping 0.1% at each end) is chosen (histogram-weighted squared error, step 2).
- Enter / OK applies, recording the history "Brightness/Contrast"; Esc / Cancel cancels; ⌥P toggles preview.

## Test coverage

- `ui_tests::brightness_contrast_dialog_types_and_applies`: after typing 50 and checking Use Legacy, 100 gray becomes 150; when unchecked, the result equals Photoshop's brightness curve.
- Screenshot comparison: `ui_tests::screenshot_adjustment_dialogs` (`brightness_contrast_dialog.png`).
