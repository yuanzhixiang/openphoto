# dialogs/preset_files.rs: Adjustment preset files

## Responsibilities

The preset gear menus of the Levels, Curves, Hue/Saturation, Channel Mixer, Black & White and Exposure dialogs, the saved presets their Preset pop-ups list, and Photoshop's preset file formats. The formats were decoded from the presets Photoshop 2026 ships (`/Applications/Adobe Photoshop 2026/Presets/<folder>`), so presets saved here open in Photoshop and Photoshop's open here.

## Kinds and where saved presets live

| Kind | Folder | Extension |
| --- | --- | --- |
| `LEVELS` | Levels | `.alv` |
| `CURVES` | Curves | `.acv` |
| `HUE_SATURATION` | Hue and Saturation | `.ahu` |
| `CHANNEL_MIXER` | Channel Mixer | `.cha` |
| `BLACK_WHITE` | Black and White | `.blw` |
| `EXPOSURE` | Exposure | `.eap` |
| `SHADOWS_HIGHLIGHTS` | Shadows Highlights | `.shh` (OpenPhoto's own layout, for Shadows/Highlights' Load... and Save...) |

Saved presets live in `~/Library/Application Support/OpenPhoto/Presets/<folder>` (`Kind::dir`). `Kind::saved` lists the files there with the kind's extension, by name (the file name without the extension, case-insensitive order).

## The gear menu (`gear`)

The dialog passes the gear's rect and its values encoded as a preset file (None while a field is invalid). A click opens a native menu (`native_popup`):

- **Save Preset...**: a save panel starting in the kind's folder (created if needed) with "Untitled.<ext>"; the file is written and becomes the current preset. Nothing happens while a field is invalid.
- **Load Preset...**: an open panel for the kind's extension; the file's bytes go back to the dialog, which decodes them into its fields, and it becomes the current preset.
- **Delete Current Preset**: enabled only while the current saved preset is shown; deletes its file from the folder.

## The Preset pop-up

- `add_saved` appends a separator and the saved presets to a pop-up's entries; `picked_saved` maps a pick to the file's bytes. The UXP dialogs' pop-ups list them the same way and read a pick with `load_saved`.
- The current preset (per kind, for the session: the one saved, loaded or picked last) is shown by name (`shown`) while the dialog's values still encode to its bytes; otherwise the dialog shows Default, a Photoshop preset's name or Custom as before.

## Formats

All numbers are big-endian.

- **Levels (.alv)**: version 2, then 29 records of five 16-bit values (input black, input white, output black, output white, gamma × 100) for composite, red, green, blue and unused channels; then "Lvls", version 3, the total record count 62, and the remaining 33 records. 630 bytes.
- **Curves (.acv)**: version 4, the curve count (5: composite, red, green, blue and an unused one), then per curve its point count and (output, input) pairs. Curves drawn with the pencil are saved as the points the point tool would turn them into (every 32 levels); Photoshop's own arbitrary-map files (.amp) aren't read or written.
- **Hue/Saturation (.ahu)**: version 2, the Colorize byte and a pad byte, Colorize's hue (−180–180; the dialog's 0–360 maps onto it), saturation and lightness, Master's hue, saturation and lightness, then each of the six ranges' four bounds and three values. 100 bytes.
- **Channel Mixer (.cha)**: version 1, Monochrome, then four output channels of four source amounts and a constant (RGB uses the first three of each). With Monochrome the gray's mix is the first channel and the dialog's color rows go back to identity. 44 bytes.
- **Black & White (.blw)**: an action descriptor (version 16, class "null", ten items): the six weights as longs (`Rd  `, `Yllw`, `Grn `, `Cyn `, `Bl  `, `Mgnt`), `useTint` (bool), `tintColor` (an RGBC object of three doubles), `bwPresetKind` (long 3) and an empty `blackAndWhitePresetFileName`. Reading looks the keys up wherever they are. The dialog turns the tint color into the hue and saturation that make it (`black_white::tint_of`, the color's HSB hue and saturation).
- **Exposure (.eap)**: version 1, then exposure, offset and gamma as 32-bit floats. 14 bytes.
- **Shadows/Highlights (.shh)**: "OPSH", version 1, then the ten settings as 32-bit floats. This is OpenPhoto's own layout; Photoshop's `.shh` hasn't been decoded (no sample ships with Photoshop).

## Test coverage

- `formats_round_trip`: each format encodes and decodes back to the same values, at Photoshop's sizes.
- `photoshops_own_presets_read` (when Photoshop is installed): Darker.alv, Darker (RGB).acv, Cyanotype.ahu, the Blue Filter channel mixer and Black & White presets and Minus 1.0.eap read as Photoshop's values, and the Levels, Curves, Hue/Saturation and Exposure files are written back byte for byte.
- `saved_presets_are_listed_by_name`.
- Each dialog's `preset_files_round_trip` (Levels, Curves, Hue/Saturation, Channel Mixer, Exposure) and `black_white::tint_round_trips_through_its_color`.

## Known limitations

- The menus are native pop-ups listing Photoshop's three items; Photoshop's gear menus have no other items for these dialogs.
