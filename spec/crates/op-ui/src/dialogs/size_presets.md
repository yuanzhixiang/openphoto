# dialogs/size_presets.rs: Image Size presets and the Delete Preset sheet

## Component Responsibilities

The Image Size dialog's own Fit To presets (`image_size.md`): the `.imz` file format, the presets folder, and the Delete Preset sheet.

## The `.imz` file

The layout of Photoshop 2026's files (decoded from files it saved), 46 bytes, big-endian, so presets can be exchanged with Photoshop:

| Offset | Size | Contents |
| --- | --- | --- |
| 0x00 | u32 | version, 2 |
| 0x04, 0x06 | u16 × 2 | width and height unit: 0 pixels, 1 inches, 2 centimeters, 3 millimeters, 4 points, 5 picas, 6 percent, 7 columns |
| 0x08, 0x0c | u32 × 2 | the pixels when saved |
| 0x10 | u16 | 0 |
| 0x12, 0x1a | f64 × 2 | width and height: in the unit for pixels and percent, in inches for printed units |
| 0x22 | f64 | resolution, pixels per inch |
| 0x2a | u16 | resolution unit: 1 per inch, 2 per centimeter |
| 0x2c | u8 | chain (constrain proportions) |
| 0x2d | u8 | Resample |

`decode` refuses short files, another version, an unknown unit or a non-positive resolution. The resampling method is not stored (Photoshop doesn't either).

## The folder

`folder()`: `~/Library/Application Support/OpenPhoto/Presets/Image Size` (Photoshop uses its own `Presets/Image Size`). `list(dir)`: the `.imz` files that decode, oldest first (by creation time, then path), each named after its file stem, as Photoshop adds new presets to the end of the menu group. `read` / `write` read and write one file (`write` creates the folder).

## Delete Preset sheet (measured in Photoshop 2026, 356 × 110 pt)

- Shown 11 pt right and 28 pt down from the Image Size window's corner, with the shared frame and the title "Delete Preset".
- "Preset:" right-aligned to x 49.5 at y 48.5; the pop-up (55, 38)–(264, 59) lists the presets (a native menu), the first one chosen.
- Delete (279.5, 38.5)–(345.5, 64.5), the default button; Cancel (279.5, 73.5)–(345.5, 99.5).
- Delete or Enter asks "Do you really want to delete the preset '<name>'?" in an alert with No and Yes (Yes is the default, as in Photoshop). Yes returns `DeleteOutcome::Delete(index)`; No goes back to the sheet; Cancel or Escape closes it.

## Test Coverage

- `reads_photoshops_files_and_writes_them_back`: two files Photoshop saved (inches at Pixels/Inch with the chain; pixels at Pixels/Centimeter without it) decode to the expected settings and encode back byte for byte; a short file is refused.
- `the_folder_lists_presets_oldest_first`: two presets come back in the order they were written, other files are ignored.
- `ui_tests::image_size_presets`: see `image_size.md`.
