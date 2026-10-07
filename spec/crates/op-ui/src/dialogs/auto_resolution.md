# dialogs/auto_resolution.rs: Image Size › Fit To › Auto Resolution...

## Component Responsibilities

Photoshop's Auto Resolution sheet: picks a print resolution from a halftone screen frequency and a quality. It is opened by the Image Size dialog (`image_size.md`), shown over it, and its OK hands back a resolution in pixels per inch.

## Data Input

- `AutoResolutionDialog::new()`: starts from the last settings confirmed with OK in this session (`last()`), or Photoshop's defaults: 133, Lines/Inch, Good.
- `show(ctx, corner)`: `corner` is the Image Size window's top-left corner; the sheet's corner is 15 pt right and down from it (Photoshop 2026).

## Layout (measured in Photoshop 2026, 348 × 135 pt, relative to its top-left corner)

- The shared frame and light title bar (`common::frame`), title "Auto Resolution" in 13 pt bold.
- "Screen:" right-aligned to x 61.5 at center y 48.5 (AppKit 12 pt text); the number field (66.5, 39)–(133.5, 58), focused with its text selected on opening; the unit pop-up (141.5, 38)–(262.5, 59): Lines/Inch, Lines/Centimeter (a native menu, `native_popup.md`).
- The "Quality" group box (10.5, 75.5)–(262.5, 124.5), its title at x 30.5 breaking the top edge 5 pt either side; radio buttons Draft, Good, Best centered at x 27.5, 108.25, 190.75, y 107, labels 15.5 pt after the centers.
- OK (278, 38.5)–(338.5, 64.5), the default button; Cancel (278, 73.5)–(338.5, 99.5).

## Interaction

- Resolution = screen frequency in lines per inch × 1 (Draft), 1.5 (Good) or 2 (Best), rounded to whole pixels per inch with halves up (Photoshop 2026: 133 at Good is 200, at Best 266; 50 at Draft is 50 — there is no lower limit).
- Changing the unit converts the number to three decimals: 133 lines/inch is "52.362" lines/centimeter, which converts back to "132.999"; the result is the same either way.
- OK or Enter (with a positive number): remembers the settings and returns `Outcome::Apply(ppi)`. Cancel or Escape: `Outcome::Cancel`. Both keys are consumed so the Image Size dialog behind does not react.

## Known Limitations

- Photoshop keeps the settings across launches; here they last for the session.

## Test Coverage

- `resolutions_match_photoshop`, `units_convert_like_photoshop` (unit tests): the values above.
- `ui_tests::image_size_auto_resolution`: see `image_size.md`.
