# dialogs/adjust/oil_paint.rs: the Oil Paint dialog

## Responsibilities

Filter › Stylize › Oil Paint...'s dialog, laid out as Photoshop 2026's (324 × 651 pt, measured from a 2x capture). Part of `AdjustDialog` (`adjust.md`), with the legacy parts (`legacy.md`).

## Settings

`Kind::OilPaint`'s parameters, in this order. The effect is `Filter::OilPaint` (`op-core`'s `more_filters.md`).

| # | Setting | Range | Default |
| --- | --- | --- | --- |
| 0 | Stylization | 0.1–10 | 1.5 |
| 1 | Cleanliness | 0–10 | 5.0 |
| 2 | Scale | 0.1–10 | 1.0 |
| 3 | Bristle Detail (`BRISTLE`) | 0–10 | 0.0 |
| 4 | Lighting (`LIGHTING`) | on/off | on |
| 5 | Angle (`ANGLE`) | −180–180° | −60 |
| 6 | Shine (`SHINE`) | 0–10 | 1.0 |

Values show one decimal, except Angle.

## Layout

Positions in points from the dialog's top-left corner.

- **Preview:** 8–220 × 36–248, its zoom controls on y 261.5.
- **Top right:** OK and Cancel at x 232. The Preview checkbox's box is at (233.5, 119.5).
- **Brush:** a group 11–299 × 296–506 titled "Brush" at x 30.5. Rows for Stylization, Cleanliness, Scale and Bristle Detail are on y 319.5, 368.5, 417.5 and 466.5. Each has its label right-aligned at 104, the field 110–169, and a track 31–293, 17.5 pt below the line, with the pin under it. Stylization's field takes the focus when the dialog opens.
- **Lighting:** a group 11–299 × 532.5–640. Its title is the Lighting checkbox, a 12 pt box at (30, 526.5) with its label 10 pt after.
  - "Angle:" on y 565, with the field 110–154, "°" at 160 and an 18 pt dial centered at (191, 565).
  - Shine's row on y 599.5.
  - With Lighting off, these dim and don't change.

## Test coverage

- `ui_tests::oil_paint_dialog_lighting_and_bristles`: the Lighting checkbox turns lighting off, and Bristle Detail reaches the filter. Enter applies "Oil Paint". Screenshots `oil_paint.png` and `oil_paint_unlit.png`; `oil_paint.png` was compared with Photoshop's capture.
