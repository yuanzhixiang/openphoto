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

The dialog is 346 × 385 pt, centered, and titled "Auto Color Correction Options", measured from 2x captures of Photoshop 2026's dialog (pt from the dialog's top-left corner, title bar included).

- **Group boxes:** 1 pt `#424242` lines; the title at x 30.5 on the top edge, which breaks 6 pt before it and 5 pt after it.
- **"Algorithms" group** (10.75, 46.75)–(259.75, 206.75):
  - four radio buttons centered at x 27.25 and y 71.25, 98.25, 125.25, 152.25 (27 pt apart), labels 16.25 pt after the center:
    - Enhance Monochromatic Contrast
    - Enhance Per Channel Contrast
    - Find Dark & Light Colors
    - Enhance Brightness and Contrast
  - a 1 pt `#3e3e3e` rule at y 169.75 from x 20 to 251;
  - the "Snap Neutral Midtones" checkbox, its 12.5 pt box at (20, 182), the label 10 pt after.
- **"Target Colors & Clipping" group** (10.75, 231.75)–(259.75, 339.75):
  - Rows centered on y 257.25, 288.25 and 319.25 for Shadows:, Midtones: and Highlights:, the labels (`#d6d6d6`) right-aligned at x 81.
  - Each row has a color swatch x 85.5–128, 22.5 pt tall, with a 1 pt `#363636` border.
  - The Shadows and Highlights rows also have "Clip:" right-aligned at x 178, a field x 182–236, 18.5 pt tall, taking 0–9.99 with two decimals and arrow steps of 0.01, and "%" at x 239.5.
- **"Save as defaults"** checkbox, its 12.5 pt box at (10, 359), the label 11 pt after.
- **Buttons:** OK (276, 45)–(336.5, 71) is the default; Cancel (276, 80)–(336.5, 106).
- **Enabled states:** with Enhance Brightness and Contrast chosen, Snap Neutral Midtones and the whole Target Colors & Clipping group are off, as in Photoshop: the labels dim, the swatches are empty frames that don't open the Color Picker, and the Clip fields are `#4e4e4e` boxes with dimmed text.

Compared side by side with Photoshop's capture (`auto_color_options_dark_light.png`), boxes, radios, swatches and fields land within about a point; the text differs by the stand-in font.

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

- Choices don't update the document preview live, as they do in Photoshop. They take effect on OK.
- Saved defaults last only for the session; Photoshop keeps them across launches.

## Test Coverage

- `ui_tests::auto_color_correction_options_from_levels` checks that:
  - Options... opens the dialog with Enhance Brightness and Contrast chosen;
  - Find Dark & Light Colors is cropped to `auto_color_options_dark_light.png` for comparing with Photoshop's capture;
  - choosing Per Channel with Save as defaults and clicking OK stores the options in the Levels dialog and as the defaults;
  - Curves then opens with them.
- Screenshot `auto_color_options.png`.
