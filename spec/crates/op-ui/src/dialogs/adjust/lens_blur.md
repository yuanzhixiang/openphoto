# dialogs/adjust/lens_blur.rs: the Lens Blur dialog

## Responsibilities

Blur › Lens Blur...'s dialog, laid out as Photoshop 2026's: a window nearly the screen's size with the preview on the left and the settings in a column on the right. It is part of `AdjustDialog` (`adjust.md`): its settings are `Kind::LensBlur`'s parameters, in this order (the constants `MODE` … `MONOCHROMATIC`): Preview mode (Faster, More Accurate), Depth Map Source (None, Transparency, Layer Mask), Blur Focal Distance (0–255), Invert, Iris Shape (Triangle (3) … Octagon (8), Hexagon at first), Radius (0–100, 15), Blade Curvature (0–100), Rotation (0–360), Brightness (0–100), Threshold (0–255, 255), Noise Amount (0–100), Distribution (Uniform, Gaussian) and Monochromatic. The effect is `Filter::LensBlur` (`op-core`'s `more_filters.md`).

## Layout

Measured from a 2x capture of Photoshop 2026 (1508 × 880 pt). Positions are in points from the window's top-left; the column's x positions are from its right edge (R).

- **Window:** 1508 × 880, or the screen less 4 pt across and 40 pt down when that is smaller (`size`). The title is "Lens Blur (zoom)", e.g. "Lens Blur (100%)".
- **Preview:** a `#4d4d4d` box with a 1 pt `#3e3e3e` border from (8, 36) to 252 pt from the right and 9 pt from the bottom. The image area inside is 9–(R−267) × 37–(bottom−24) (`image_rect`). It shows the document as filtered at the preview's zoom (`pane_ui`), and dragging pans it. The zoom bar (− and + boxes, the zoom in a combo, as the Distort dialogs' `distort::zoom_bar`) sits 21 pt above the bottom.
- **OK and Cancel:** (R−187.5)–(R−63.5), at y 42–68 and 78–104.
- **Preview group** (R−238)–(R−13.5) × 123.5–160.5: the Preview checkbox on its top edge (its box at (R−223.5, 116)); the Faster and More Accurate radio buttons at (R−218, 145) and (R−150, 145), labels 13 pt after.
- **Depth Map** (172.5–316):
  - "Source:" right-aligned at R−154.5 by the popup (R−143.5)–(R−26.5) × 184.5–201.5.
  - Set Focal Point: a 25 pt button at (R−227)–(R−202) × 215–240 with a dashed square and a cross, and its label at R−198.
  - Blur Focal Distance: a slider row at y 257.
  - Invert: a checkbox at (R−228.5, 292.5).
  - All of these dim, and do nothing, while Source is None.
- **Iris** (328.5–485): "Shape:" and its popup at y 349.5 (popup 341.5–357.5); slider rows Radius (378), Blade Curvature (416), Rotation (454).
- **Specular Highlights** (497.5–587): Brightness (518), Threshold (556).
- **Noise** (599.5–729):
  - Amount (620).
  - A Distribution group (R−232)–(R−19.5) × 657–704.5 with Uniform and Gaussian at (R−212, 676) and (R−212, 692).
  - Monochromatic: a checkbox at (R−230.5, 710.5).
- **Groups** span (R−238)–(R−13.5), with their titles on the top edge at R−218.
- **Slider row** (`slider_row`): the label 225 pt from the right on the row's line, the field (R−61)–(R−25.5) 19 pt tall around it, and an 18 pt lower track from R−230 to R−21.5 with the pin standing on it, travelling from R−226 (minimum) to R−26 (maximum). Disabled, the label dims and the field shows its value without taking input.

## Interaction

- Set Focal Point toggles (`Extra::lens_pick`, shown pressed); while on, a click in the preview marks that document pixel (`Extra::lens_focal_at`). The app takes the preview off the document, reads the depth there from the source (the layer's alpha for Transparency, its mask for Layer Mask) and puts it in Blur Focal Distance (`set_focal_distance`); the preview comes back on the next frame.
- Changing Source to None turns Set Focal Point off.
- Preview previews on the document (and so in the preview box), like the other filter dialogs. Faster and More Accurate are kept but compute the same.
- OK or Enter applies (unless a field is being typed in); Cancel or Esc closes.

## Known limitations

- Photoshop's own Lens Blur algorithm (its iris rendering, how highlights bloom, its noise) is approximated (`more_filters.md`).
- The Depth Map popup offers Layer Mask even when the layer has none (the depth map is then ignored).
- The column doesn't scroll when the window is too short for it.

## Test coverage

- `ui_tests::lens_blur_dialog_picks_its_focal_point`: Source Transparency, Set Focal Point, then a click in the preview reads the opaque document's depth (255). Blade Curvature and Rotation reach the filter. Enter applies "Lens Blur". Screenshot `lens_blur.png`, compared column by column with Photoshop's capture.
