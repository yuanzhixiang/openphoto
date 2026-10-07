# dialogs/adjust/smart_sharpen.rs: the Smart Sharpen dialog

## Responsibilities

Filter › Sharpen › Smart Sharpen...'s dialog, laid out as Photoshop 2026's: 685 pt wide; 291 tall with Shadows / Highlights folded and 502 open (`smart_sharpen_size`). Measured from 2x captures of both states. Part of `AdjustDialog` (`adjust.md`), with the legacy parts (`legacy.md`).

## Settings

`Kind::SmartSharpen`'s parameters, in this order:

| # | Setting | Range | Default |
| --- | --- | --- | --- |
| 0 | Amount | 1–500 % | 200 |
| 1 | Radius | 0.1–64 px | 1.0 |
| 2 | Reduce Noise | 0–100 % | 10 |
| 3 | Remove | Gaussian Blur, Lens Blur, Motion Blur | Lens Blur |
| 4 | Angle | −180–180° | 0 |
| 5–7 | Shadows' Fade Amount, Tonal Width, Radius | 0–100 %, 0–100 %, 1–100 px | 0, 50, 1 |
| 8–10 | Highlights' three | as Shadows' | as Shadows' |
| 11 | Use Legacy | on/off | off |
| 12 | More Accurate | on/off | off |

The effect is `Filter::SmartSharpen` (`op-core`'s `more_filters.md`).

## Layout

Positions in points from the dialog's top-left corner.

- **Preview:** 8–271 × 36–235.5 folded (zoom controls on y 257.5), 8–271 × 36–457 open (on y 471.5) (`legacy_area`). Folding or opening rebuilds it.
- **Top right:** OK and Cancel at x 613. The Preview checkbox's box is at (378.5, 45.5), its label 9.5 pt after.
- **Gear:** a gear with a menu triangle at about (585, 51.5). Its menu offers:
  - Use Legacy (✓ when on);
  - Save Preset... (keeps the settings as "Preset N" for the session and chooses it);
  - Delete Preset (the chosen saved one).
- **Preset:** "Preset:" right-aligned at 372.5 on y 83.5, and its pop-up 379–579 × 73.5–93.5. Default sets everything but Use Legacy back to Photoshop's values; a saved preset restores its values.
- **Rows:** each has its label right-aligned at 372.5, the field 539–583 (17 pt tall around the line), the unit at 585.5, and a track 386.5–524.5 starting 7 pt above the line with its pin hanging under it.
  - Amount (y 116.5, "%").
  - Radius (143.5, "px"). Its slider spreads values as Photoshop's does, measured by dragging the pin (`RADIUS_SCALE`): 1 at 16 % of the way, 2.1 at 25 %, 8.1 at 49 %, 21.7 at 74 %, 64 at the end.
  - Reduce Noise (170.5, "%"). In Legacy, a More Accurate checkbox at (386.5, 164.5) replaces it.
- **Remove:** "Remove:" right-aligned at 372.5 on y 206.5; the pop-up 379–480.5 × 197–216; the Angle field 489–533 × 197.5–215.5; "°" at 541; a 17.5 pt angle dial centered at (575, 206.5). The field and dial work only for Motion Blur and are dimmed otherwise.
- **Shadows / Highlights:**
  - Folded: a right-pointing chevron at (294, 257.5) with "Shadows / Highlights" in bold 13 pt at 304.5; a click opens it.
  - Open:
    - a line at y 243 (285.5–598.5);
    - "Shadows" under a down-pointing chevron at y 253, which folds it;
    - Fade Amount, Tonal Width and Radius rows at y 285, 312 and 339 ("%", "%", "px");
    - a line at y 356.5 (295–598.5);
    - "Highlights" in bold at (295, 374);
    - its three rows at 399, 426 and 453.

## Known limitations

- Presets live only for the session; Photoshop's Load Preset... (a file) isn't offered.

## Test coverage

- `ui_tests::smart_sharpen_dialog_opens_shadows_highlights`: clicking the folded header opens Shadows / Highlights. Shadows' Fade Amount and Use Legacy reach the filter. Enter applies "Smart Sharpen". Screenshots `smart_sharpen.png` and `smart_sharpen_open.png`, compared with Photoshop's captures: positions within about a point, and the Radius pin within half a point.
