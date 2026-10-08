# dialogs/adjust/replace_color.rs: Replace Color

## Responsibility

Image › Adjustments › Replace Color..., drawn as Photoshop 2026's dialog as part of `AdjustDialog` (`adjust.md`). Its settings (`Kind::params`): 0 Fuzziness (0–200, 40), 1–3 Hue (−180–180), Saturation and Lightness (−100–100), 4 Localized Color Clusters. The sampled colors are `Extra::replace` (`op_core::color_match::ReplaceSamples`), the chosen eyedropper `Extra::rc_tool`.

## Layout

Measured from a 2x capture of Photoshop 2026's dialog: 436 × 474 pt, points from the dialog's top-left corner (title bar included).

- **Tools:** the eyedropper, plus and minus eyedroppers centered at x 24.75, 54.75, 84.75 on y 51; the chosen one in a 29.5 × 25.5 pt `#383838` box with 3 pt corners.
- **Localized Color Clusters** checkbox, its 12 pt box at (10, 76.5), the label 11 pt after.
- **Color:** right-aligned at 281 on y 64.25; the swatch (290.5, 44.5)–(330, 84) shows the last color picked, with a 1 pt `#363636` border.
- **Fuzziness:** the label at x 11 on y 108.5, the field 287–330 (17 pt tall, focused when the dialog opens), the classic track 89.5–267.5 with its top at 103.
- **Preview** area (10, 124)–(331, 328): the active layer made small (`Extra::thumb`, at most 642 × 408 pixels, taken when the dialog opens) as the selection (gray by how much each pixel is replaced; Localized clusters grown from the picked points moved onto the thumbnail) or as the image, fitted and centered, never larger than a thumbnail pixel a screen pixel (a small document shows at its pixel size, as in Photoshop). Rebuilt only when the samples, Fuzziness or the choice change.
- **Selection / Image** radios centered at (92.25, 354) and (189.75, 354), labels 17.5 pt after.
- A 1 pt `#3e3e3e` rule at y 381 from 10 to 330.
- **Hue, Saturation, Lightness:** labels at x 11 on y 400.5, 427.5, 454.5; fields 239–282; tracks 92–219.5 with their tops 5.5 pt above each line.
- **Result** swatch (290.5, 396)–(330, 435.5): the color picked with the shift applied; "Result" centered at (310.5, 451.25).
- **Buttons** x 346–426.5, 26 pt tall: OK (38.5, the default, off while the settings don't make an effect), Cancel (73.5), Load... (108.5), Save... (143.5); **Preview** with its 12 pt box at (345.5, 181).

Compared side by side with the capture (`replace_color.png`), every control lands within about a point.

## Interaction

- A click on the image outside the dialog samples (`lib.md`): the eyedropper picks the color alone, the plus eyedropper (or Shift) adds it, the minus eyedropper (or Option) takes it out (`AdjustDialog::sample_at`, with the document point for Localized Color Clusters).
- Load... and Save... read and write Fuzziness and the shift (`preset_files::REPLACE_COLOR`, `.axt`, OpenPhoto's own layout: Photoshop's hasn't been decoded).
- OK or Enter (not while typing in a field) applies; Cancel or Escape cancels.

## Known limitations

- The Color and Result swatches don't open the Color Picker (Photoshop picks the selection color and the replacement there).
- Load... and Save... don't read or write Photoshop's `.axt` files.
