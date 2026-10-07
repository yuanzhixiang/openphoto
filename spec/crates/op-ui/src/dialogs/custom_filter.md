# dialogs/custom_filter.rs: Custom filter dialog

## Component responsibilities

The dialog for Filter › Other › Custom..., rebuilt point by point after Photoshop 2026's classic dialog: edits a 5 × 5 convolution kernel, Scale and Offset, and loads/saves Photoshop .acf files. For the convolution algorithm see `op-core`'s `filter.md` (`Filter::Custom`). The window frame, preview pane and zoom controls are drawn by the host `adjust.rs` (`Custom::Kernel`); this module draws only the grid, the fields and the buttons.

## Data input

- `Dialog { cells: [String; 25], scale, offset }`: the text in the fields, row by row from top to bottom.
- `Default`: the kernel Photoshop shows the first time it opens: a sharpening cross (center 5, top/bottom/left/right −1, the rest empty), Scale 1, Offset empty.
- `filter()`: yields `Filter::Custom` when all fields are valid, otherwise `None` (OK and Save... are grayed out, and there is no preview).
  - Kernel: integers −999–999; empty fields count as 0.
  - Scale: integer 1–9999; may not be empty.
  - Offset: integer −9999–9999; empty counts as 0.
  - Decimals and non-numbers are all invalid.
- `settings()` / `restore(values)`: 27 strings (25 cells, Scale, Offset) that let the host remember the settings from the last OK (as with the other filter dialogs); ignored when the count does not match.

## Layout and visuals

Size 634 × 282 pt; coordinates are Photoshop points from the dialog's top-left corner:

- The preview pane and zoom controls on the left are the same as in the other classic filter dialogs (`PANE` and `ZOOM_Y` in `filter_layout.md`).
- Grid: 25 input fields of 44 × 19, column left edges at 277, 329, 380.5, 432.5, 484.5, row top edges at 78, 105, 132, 159, 186.
- Below the grid at y 213–232: "Scale:" with its right edge at 325.5, input field 330.5–374.5; "Offset:" with its right edge at 429.5, input field 434.5–478.5.
- Buttons on the right at x 543.5–623.5, 26 tall: OK (default button) y 38.5, Cancel 73.5, Load... 143.5, Save... 178.5; Preview checkbox top-left corner (543, 216.5).

## Interaction

- When opened, the top-left cell gets focus with its contents selected; Tab moves focus in creation order (row by row, then Scale, Offset); the input fields support ↑↓ (±10 with ⇧).
- OK or Enter (when valid) applies; Cancel or Esc cancels.
- Load...: a system open dialog selects an .acf file and the first 54 bytes are read: 27 big-endian 16-bit integers (25 cells, Scale, Offset). Cells and Offset that are 0 are displayed as empty; Scale is at least 1. Files shorter than 54 bytes are ignored.
- Save...: a system save dialog (default name Untitled.acf) writes the same format. Write failures are silently ignored.

## Known limitations

- There is no message when Load/Save fails (Photoshop shows an error).

## Test coverage

- `opens_with_photoshops_sharpening_cross`: the default kernel and Scale/Offset.
- `fields_must_be_integers_in_range`: out-of-range values, decimals, and an empty or 0 Scale are invalid; boundary values are valid.
- `acf_files_round_trip`: writes 54 bytes, big-endian; reading back yields the same filter, 0 reads back as empty, and files that are too short are rejected.
- `settings_round_trip`: restoring the remembered settings yields the same filter.
- `custom_filter_types_a_kernel_and_remembers_it` in `ui_tests.rs`: opens from the menu, types 2 in the top-left cell, applies with Enter and records "Custom", and the kernel is kept when reopening; `screenshot_filter_dialogs` captures this dialog for comparison with Photoshop.
