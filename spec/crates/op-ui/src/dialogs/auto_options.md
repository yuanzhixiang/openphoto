# dialogs/auto_options.rs: Auto Color Correction Options

## Component Responsibilities

This is the dialog that the Options... button in Levels and in Curves opens. It sets:

- which algorithm their Auto button uses;
- Snap Neutral Midtones;
- the target colors and the clipping;
- whether these settings become the defaults.

The computing itself is done by `op_core::auto` (see `auto.md`). The dialog only edits an `op_core::auto::Options` and returns it.

## Data and Layout

The dialog opens with the Levels or Curves dialog's current options (`AdjustDialog::extra.auto_options`). "Save as defaults" starts unchecked.

The dialog is 400 × 350 pt, centered, and titled "Auto Color Correction Options". The arrangement follows Photoshop's, but it has **not been measured against Photoshop 2026**, so the positions below are provisional (pt from the dialog's top-left corner).

- **"Algorithms" group** (18, 45)–(300, 180):
  - four radio buttons centered at x 36 and y 66, 88, 110, 132:
    - Enhance Monochromatic Contrast
    - Enhance Per Channel Contrast
    - Find Dark & Light Colors
    - Enhance Brightness and Contrast
  - the "Snap Neutral Midtones" checkbox at (29.5, 150).
- **"Target Colors & Clipping" group** (18, 200)–(300, 302):
  - Rows at y 222, 252 and 282 for Shadows:, Midtones: and Highlights:.
  - Each row has its label right-aligned at x 100 and a color swatch x 106–152.
  - The Shadows and Highlights rows also have "Clip:" right-aligned at x 200, a field x 206–256 taking 0–9.99 with two decimals and arrow steps of 0.01, and "%" after it.
- **"Save as defaults"** checkbox at (18, 318).
- **Buttons:** OK (312, 38.5)–(382, 64.5) is the default; Cancel (312, 73.5)–(382, 99.5).

## Interactions

- Clicking a radio button picks that algorithm. Clicking the checkbox toggles it.
- Typing in a Clip field applies the value as you type.
- Clicking a swatch opens the Color Picker for that target (`PickerTarget::AutoTarget`), titled "Select target shadow color:", "Select target midtone color:" or "Select target highlight color:". Its OK sets the target. While the picker is open, the dialog ignores Escape.
- OK (or Enter while no field is being typed in) returns the options and the Save as defaults choice. The app then:
  - keeps the options as `AppState::auto_saved` when Save as defaults is checked;
  - hands them to the Levels or Curves dialog, which runs Auto with them right away (`AdjustDialog::set_auto_options`).
- Cancel or Escape changes nothing.
- While the dialog is open, the Levels or Curves dialog underneath is blocked.

## Known Limitations

- The layout is provisional (not measured).
- Choices don't update the document preview live, as they do in Photoshop. They take effect on OK.
- Saved defaults last only for the session; Photoshop keeps them across launches.

## Test Coverage

- `ui_tests::auto_color_correction_options_from_levels` checks that:
  - Options... opens the dialog with Enhance Brightness and Contrast chosen;
  - choosing Per Channel with Save as defaults and clicking OK stores the options in the Levels dialog and as the defaults;
  - Curves then opens with them.
- Screenshot `auto_color_options.png`.
