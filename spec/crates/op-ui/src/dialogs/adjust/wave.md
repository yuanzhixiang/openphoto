# dialogs/adjust/wave.rs: the Wave dialog

## Responsibilities

Filter › Distort › Wave...'s dialog, laid out as Photoshop 2026's (553 × 396 pt, measured from a 2x capture). Part of `AdjustDialog` (`adjust.md`). Its settings are `Kind::Wave`'s parameters:

| # | Setting | Range | Default |
| --- | --- | --- | --- |
| 0 | Number of Generators | 1–999 | 5 |
| 1–2 | Wavelength Min. and Max. | — | 10 and 120 |
| 3–4 | Amplitude Min. and Max. | — | 5 and 35 |
| 5–6 | Scale Horiz. and Vert. | 1–100 % | — |
| 7 | Type | Sine, Triangle, Square | — |
| 8 | Undefined Areas | Wrap Around, Repeat Edge Pixels | Repeat Edge Pixels |

The random pattern is `Extra::seed`: a new one when the dialog opens and each time Randomize is pressed.

## Layout

Positions in points from the dialog's top-left corner.

- **Left column:**
  - "Number of Generators:" (15.5, 44) and its field 246–273.5 × 35.5–53.5, which is focused when the dialog opens.
  - "Wavelength:" (15.5, 106.5), "Amplitude:" (15.5, 205) and "Scale:" (15.5, 308), each with two 34 × 19 fields at x 165 and 231 (Scale's at 165 and 236, each followed by "%" 40.5 pt after its left edge). The fields' tops are at y 96, 195 and 296, with headers ("Min." / "Max.", "Horiz." / "Vert.") centered 12.5 pt above them.
- **Sliders:**
  - Each is a 3 pt `#757575` track from x 10 to 277.5. Pins travel from x 16 (minimum) to 272.5 (maximum), evenly; dragging along a track sets its setting.
  - Number of Generators' track is at y 62.
  - Wavelength's tracks are at y 140 and 155, and Amplitude's at 239 and 254. In each pair the first (Min.) has a pin pointing down onto it, its tip 5 pt below the track's top; the second (Max.) has a pin pointing up, its tip 2.5 pt above the track's top.
  - Scale's tracks (337, 349) both have up-pointing pins.
  - The pins are 9 pt wide and 8 pt long, `#f0f0f0` with a `#212121` edge (`small_pin`).
- **Type:** a group 298.5–378.5 × 46–117 titled "Type:" (316). Its radio buttons are at (312, 64 / 84 / 104), labels 13 pt after.
- **Preview:** a `#4c4c4c` box with a 1 pt `#3e3e3e` border, 294.5–540.5 × 130.5–281.5 (`PREVIEW`), showing the document as filtered at 100% (`pane_ui`).
- **Randomize:** a flat button 371–464 × 291–317 (`#454545`, `#666666` edge, `#5a5a5a` while pressed), 13 pt label.
- **Undefined Areas:** a group 295–540.5 × 333.5–386 titled "Undefined Areas:" (312.5). Its radio buttons are at (308, 352 / 372).
- **OK and Cancel:** 392.5–531, y 39–65 and 75–101, 13 pt labels.

## Behavior

- Wave previews on the document (and so in its preview box) like the other filter dialogs.
- OK or Enter applies (unless a field is being typed in); Cancel or Esc closes.

## Known limitations

- Photoshop's own wave generators are random in their own way; OpenPhoto's pattern (`more_filters.md`) differs pixel by pixel.

## Test coverage

- `ui_tests::wave_dialog_randomizes`: Randomize draws a new seed; dragging Wavelength's Max. slider to its end sets 999; Enter applies "Wave". Screenshot `wave.png`, compared with Photoshop's capture.
