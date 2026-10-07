# dialogs/adjust/reduce_noise.rs: the Reduce Noise dialog

## Responsibilities

Filter › Noise › Reduce Noise...'s dialog, laid out as Photoshop 2026's (807 × 658 pt, measured from 2x captures of Basic, Advanced › Overall and Advanced › Per Channel). Part of `AdjustDialog` (`adjust.md`), with the legacy parts (`legacy.md`).

## Settings

`Kind::ReduceNoise`'s parameters, in this order:

| # | Setting | Range | Default |
| --- | --- | --- | --- |
| 0 | Mode (`MODE`) | Basic, Advanced | Basic |
| 1 | Strength | 0–10 | 6 |
| 2 | Preserve Details | 0–100 % | 60 |
| 3 | Reduce Color Noise | 0–100 % | 45 |
| 4 | Sharpen Details | 0–100 % | 25 |
| 5 | Remove JPEG Artifact (`JPEG`) | on/off | off |
| 6 | Channel (`CHANNEL`) | Red, Green, Blue | — |
| 7–12 | Red's, green's and blue's Strength and Preserve Details (`CHANNELS`) | — | 0 and 60 |

The effect is `Filter::ReduceNoise` (`op-core`'s `more_filters.md`). The per-channel pairs count only in Advanced.

## Layout

Positions in points from the dialog's top-left corner.

- **Preview** in 8–491 × 36–619, its zoom controls on y 637.5.
- **Top right:** OK and Cancel at x 616. The Preview checkbox's box is at (607, 112.5). The Basic and Advanced radio buttons are centered at (569.5, 155) and (646, 155), with 13 pt labels 16 pt after.
- **Settings group:** 499–796 × 193–617. Its top edge breaks from 512 to 788 for:
  - "Settings:" at (518, 193);
  - the pop-up 574–740.5 × 182.5–202.5;
  - Save (a tray with an arrow down) at about (755.5, 193);
  - Delete (a trash can) at about (776.5, 193).
- **Overall rows:** Strength, Preserve Details, Reduce Color Noise and Sharpen Details, each:
  - label centered at y 255, 311, 367 and 423, right-aligned to 653.5;
  - field 659–703 from 9.5 above the line to 8.5 below, "%" at 708 (not Strength);
  - track 540.5–758.5, 16 pt below the line.

  Remove JPEG Artifact's box is at (530, 472.5).
- **Advanced:** a `#474747` tab bar 500–795 × 213.5–229.5 with Overall (500–549.5) and Per Channel (549.5–622.5). The chosen tab is the dialog's gray with white text; the other has light gray text. Overall shows the rows above.
- **Per Channel:**
  - The chosen channel of the layer as a gray thumbnail (made once on open, at most 240 × 220 pixels, in `lib.rs`), fitted into 120 × 110 pt at no more than a point a pixel, centered at (651.5, 334.5).
  - The channel group 520.5–778.5 × 444–558.5, its top edge broken from 535 to 717 for "Channel:" (539.5, 444) and its pop-up 594.5–713 × 434–453.5.
  - Strength (label to 634 on y 474, field 639–683.5) and Preserve Details (y 519, "%" at 688.5), tracks 540.5–758.5 15.5 pt below. Preserve Details dims while the channel's Strength is 0.

## Settings pop-up

- Default sets everything but Basic/Advanced back to Photoshop's values.
- Save adds the current values as "Settings 1", "Settings 2", … and chooses it; choosing a saved one restores its values.
- Delete removes the chosen saved one (Default can't be deleted).
- Saved settings last for the session (a process-wide list), not across launches.

## Test coverage

- `ui_tests::reduce_noise_dialog_advanced_per_channel`: Advanced, the Per Channel tab, then green's Strength reaches the filter (red's stays 0); Enter applies "Reduce Noise". Screenshots `reduce_noise.png`, `reduce_noise_advanced.png` and `reduce_noise_channel.png`, compared with Photoshop's captures.
