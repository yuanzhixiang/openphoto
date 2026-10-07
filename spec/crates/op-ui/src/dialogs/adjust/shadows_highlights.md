# dialogs/adjust/shadows_highlights.rs: Shadows/Highlights (short layout)

## Responsibility

Image › Adjustments › Shadows/Highlights... opens in Photoshop 2026's short layout: a UXP dialog with only the two Amounts. This module draws it as part of `AdjustDialog` (`adjust.md`) whenever Show More Options (setting 10) is off; with it on, the dialog switches to the layout with every setting (`filter_layout::SHADOWS_HIGHLIGHTS`, `filter_layout.md`).

## Layout

Measured from a 2x capture of Photoshop 2026's dialog. 449 × 236 pt; positions in points from the dialog's top-left corner (title bar included). The controls come from the UXP kit (`uxp.md`).

- Title bar: `common::frame`, "Shadows/Highlights".
- **Shadows** (setting 0) and **Highlights** (setting 3) rows:
  - labels right-aligned at 68.5, their capitals centered on y 60.25 and 96.25;
  - Spectrum sliders (`uxp::linear_slider`, 0–100 evenly, step 1) on the track 77.25–236.5 at y 59.75 and 95.75: the thumb's ring (13 pt) runs 83.75–230, and the track stops 4.5 pt short of it;
  - number boxes 245–308 × 48–72 and 84–108. The fields hold the percent sign as Photoshop's do ("35%", added on open and by the slider); a typed number without one shows a "%" after it. Shadows' field opens focused with its text selected.
- **Show more options** checkbox with its box at (20, 126); checking it switches to the full layout.
- **Buttons** down the right, 109 pt wide, 20 pt from the right edge: OK (48, the default), Cancel (84), Load... (120), Save... (156).
- **Preview (Opt+P)** with its box at (320, 198).

Compared side by side with the capture (`ui_tests::shadows_highlights_more_options_and_replace_color_preview`'s `shadows_highlights.png`), boxes, tracks, thumbs and buttons land on Photoshop's; the labels are a little wider because Source Sans 3 stands in for Adobe Clean.

## Load... and Save...

Save... writes the ten settings to a file (`preset_files::SHADOWS_HIGHLIGHTS`, `.shh`, a save panel starting in the presets folder); Load... reads one back into the fields. The file is OpenPhoto's own layout (`preset_files.md`): Photoshop's own `.shh` hasn't been decoded yet.

## Known limitations

- The full layout (Show More Options) is the classic layout kit's, not yet measured against Photoshop 2026's.
- Load... and Save... don't read or write Photoshop's `.shh` files.
