# dialogs/adjust/hdr_toning.rs: HDR Toning

## Responsibility

Image › Adjustments › HDR Toning..., drawn as Photoshop 2026's dialog as part of `AdjustDialog` (`adjust.md`). Its settings (`Kind::params`): 0 Method, 1 Radius, 2 Strength, 3 Gamma, 4 Exposure, 5 Detail, 6 Shadow, 7 Highlight, 8 Vibrance, 9 Saturation, 10 Smooth Edges. The Toning Curve, the point picked on it and which sections are open are `Extra::hdr` (`HdrExtra`).

## Layout

Measured from 2x captures of Photoshop 2026's dialog: 441 pt wide; 479 pt tall with the curve folded, 802 with it open. Points from the dialog's top-left corner.

- **Preset:** right-aligned at 44.5 on y 49.5; the pop-up 49–306 × 40.5–59; the gear at (320, 50).
- **Method:** right-aligned at 71 on y 89; the pop-up 75–208 × 80–98.5, breaking the top line of the outer group (10.75, 90.25)–(333.75, bottom).
- **Sections** (Local Adaptation): Edge Glow, Tone and Detail, Advanced, Toning Curve and Histogram. Each title line has a triangle at x 29.75 (down when open, right when folded) and the title at 40.5; a click on either folds or opens it. Open, a box (20.75, title)–(323.75, title + its height: 87.25, 90.25, 115.5, 334.75) whose top breaks around the title; folded, only the title line. Titles sit at 115.5, 215.5, 318.25 and 446 when the sections above are open (25 pt apart for a folded one); the outer group ends 10 pt under an open last box or 22 pt under a folded last title, and the dialog 11 pt under the group.
- **Rows:** the label right-aligned at 105 on the row's line; the field 236–279.5 (17 pt tall; Radius's focused on open); "px" or "%" at 281; the classic track 117–224.5 with its top 4.5 pt above the line. Gamma's track centers 1 and Detail's 0 (`Scale::Table`). Edge Glow: Radius and Strength 23.5 and 48.5 under the title, Smooth Edges (its 10.5 pt box at (123.5, title + 67.5)); Tone and Detail: Gamma, Exposure, Detail 23, 48, 73 under; Advanced: Shadow, Highlight, Vibrance, Saturation 23.25 pt under and 25 apart.
- **Toning Curve and Histogram:** the graph (49, title + 25)–(305, title + 280.5) on `#454545` with the luminosity histogram and a quarter grid, the white curve and its points; ramps down the left (white to black, x 40–49) and along the bottom (black to white, 9 pt); under them on title + 306.5: "Input:" at 40 with its field 85.5–118 and "%", "Output:" at 138 with its field 183.5–216 and "%", the Corner checkbox at 232 and the reset button at 296.
- **Exposure and Gamma** method: only Gamma and Exposure rows (y 121 and 146), the group ending at 164; **Highlight Compression** and **Equalize Histogram**: no settings, the group ending at 110. These two layouts are not measured.
- **OK** (351, 38.5)–(430, 64), the default; **Cancel** (351, 73.5)–(430, 99); **Preview** with its box at (350, 118.5).

Compared side by side with the captures (`ui_tests::hdr_toning_sections_and_curve`: `hdr_toning.png`, `hdr_toning_curve.png`), the sections, rows, fields and the curve area land within about a point; Photoshop shows a sign on positive Detail and Saturation ("+30"), which the fields don't.

## Toning Curve

- A press near a point picks it (and drags it, between its neighbors); elsewhere it adds a point there (up to 16). A middle point dragged well out of the graph goes. The picked point's Input and Output show in percent and can be typed; Corner makes it a corner (the curve bends sharply there). The reset button straightens the curve.
- The curve goes into the effect (`op_core::tone::HdrToning::curve`, `curve_table`), applied to the toned luminosity in Local Adaptation.

## Presets

The Preset pop-up lists Default, Custom (only shown), Photoshop's HDR Toning presets when Photoshop is installed, and the saved ones; the gear saves, loads and deletes `.hdt` files (`preset_files.md`: Photoshop's layout as decoded from its presets). Picking one sets the settings and the curve; Default also straightens the curve.

## Known limitations

- Photoshop flattens a layered document before HDR Toning (asking first); here it tones the active layer.
- The Method pop-up lists the methods in OpenPhoto's order; Photoshop's own order hasn't been read.
