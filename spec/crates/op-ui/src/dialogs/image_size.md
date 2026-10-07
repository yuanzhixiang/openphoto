# dialogs/image_size.rs: Image › Image Size Dialog

## Component Responsibilities

Sets the image's pixel dimensions (resampling) and resolution; on OK, `lib.rs` carries it out (see "Image Size dialog integration" in `lib.md`; the resampling algorithms are in `op-core`'s `image_ops.md`). The numeric rules live in `Model`, separate from the interface, to make testing easy.

## Data Input

- `ImageSizeDialog::new(width, height, resolution)`: the current document's width, height, and resolution. Every opening restores the defaults: width/height unit Inches, resolution unit Pixels/Inch, Dimensions unit Pixels, width and height linked, Resample on, method Automatic, Fit To Original Size, Scale Styles checked (matching Photoshop 2026's defaults).
- `with_preview(Preview { rgba, width, height })`: the document's composite pixels, for the preview on the left; passed in when the command opens the dialog.

## Layout (measured in Photoshop 2026, 665 × 358 pt, relative to the top-left corner)

Compared item by item against screenshots; text positions and widths differ from Photoshop by no more than 1–2 px:

- Window: the shared frame and title bar (`common::frame`, title 13 pt bold). The top-left corner has the three macOS window buttons, centers (14, 14), (34, 14), (54, 14), diameter 12: since the window is modal, close and minimize are gray `#bababa`; since it is resizable, the zoom button is green `#61c554` (appearance only).
- Preview (10, 38)–(288, 316): 1 pt `#424242` border; inside, the document is shown at 100% (one image pixel per device pixel), initially centered; transparent areas show a white/`#cccccc` checkerboard, and outside the image is white. Dragging pans, and the image cannot be dragged out of view.
- Right column text is the AppKit system font 12 pt (`theme::dialog`); labels and descriptive text are `#d6d6d6`, right-aligned to x 423.5:
  - "Image Size:" center y 50.25, value at x 431.5: the RGB memory size of the result (M with two decimals when ≥ 1 MB, otherwise K with one decimal), with "(was 1.70M)" appended when it differs from the original. To the right, (633.5, 41.5)–(654.5, 58.5) is the gear button, whose menu has only the checkable "Scale Styles" (there are no layer styles yet; only the state is saved).
  - "Dimensions:" center y 79.75; (432, 70.5)–(450, 88) is a small box with a V-shaped arrow, clicked to choose the unit (Percent, Pixels, Inches, Centimeters, Millimeters, Points, Picas); from x 459 is "734 px × 811 px", where "×" is 14 pt and its ink is 8.5 pt from the text on either side.
  - "Fit To:" dropdown (428, 97)–(655, 118): Original Size, Auto Resolution... (grayed out, see Known Limitations), three groups of presets (four screen sizes such as 960 x 640 px 144 ppi; A4, A6, Legal, Letter; 4 x 6, 5 x 7, 8 x 10, 11 x 14 in 300 dpi), Load/Save/Delete Preset... (grayed out), Custom (grayed out, shown only after values are changed).
  - "Width:"/"Height:" center y 136.5 / 165.5: fields (427.5, 127)–(503, 146) and (427.5, 156)–(503, 175), unit dropdowns (511, 126)–(655, 147) and (511, 155)–(655, 176) (Percent, Pixels, Inches, Centimeters, Millimeters, Points, Picas, Columns; width and height share one unit). On open, Width receives focus with all text selected.
  - Chain: button (349, 142)–(365, 160), `#383838` fill, `#636363` border; inside are two interlocking rings and a vertical bar (traced from a 2x screenshot; no vertical bar when unlinked); above and below, a `#828282` bracket line connects to the Width and Height labels.
  - "Resolution:" center y 194.75: field (427.5, 185)–(503, 204), unit dropdown (511, 184)–(655, 205) (Pixels/Inch, Pixels/Centimeter).
  - "Resample:" checkbox at (334.5, 217.5), method dropdown (428, 213)–(655, 234): Automatic | Preserve Details (enlargement), Preserve Details 2.0, Bicubic Smoother (enlargement) | Bicubic Sharper (reduction) | Bicubic (smooth gradients), Nearest Neighbor (hard edges), Bilinear, with ⌥1 … ⌥8 shown on the right of the menu.
- The description "Create a new, larger document with more detail" is at (328.5, 276.5), followed by the underlined blue `#5e9eee` link "Open in Generative Upscale...".
- Buttons: Cancel (329, 311.5)–(486.5, 337.5); OK (496.5, 311.5)–(654, 337.5) is the default button.

## Interaction (`Model`, matching Photoshop)

- Fields display in the current unit: integers for pixels, otherwise at most three decimals with trailing zeros removed (734 px at 72 ppi is "10.194" inches). While one field is being edited, the other fields refresh along with it, and the field being edited keeps the typed text.
- Resample on: changing width or height changes the pixels; when linked, the other side follows at the original ratio. When changing resolution, if the unit is a print size (inches, etc.) the print size is kept and the pixels change; if the unit is pixels or percent, the pixels do not change.
- Resample off: pixels revert to the document's dimensions and cannot be changed; the chain is forced linked and cannot be clicked; changing the print size changes the resolution; fields are grayed out when the unit is pixels or percent; the method dropdown is grayed out and ⌥ number keys have no effect.
- Fit To presets: fit the image proportionally into the preset's box (the box is turned to the image's orientation, so a portrait image uses a portrait box), set the preset's resolution and unit, and turn on Resample and the chain; Original Size restores the original pixels and resolution. After any further value change, Fit To shows Custom.
- ⌥1 … ⌥8: choose the corresponding resampling method.
- Valid range: width and height 1–30000 pixels, resolution 1–10000; OK is grayed out when invalid.
- OK or Enter: returns `Outcome::Apply { width, height, resolution, resample }` (pixels rounded; `resample` is `None` when Resample is off); Cancel or Esc: `Outcome::Cancel`. Modal while open.

## Known Limitations

- The dropdown menus are egui popup menus, whereas Photoshop uses native macOS menus here (light and translucent, checkmark on the left), so the appearance differs.
- Auto Resolution... and Load/Save Preset... are not implemented yet (grayed out); Generative Upscale depends on Adobe cloud services, and the link does nothing.
- A focused field has a 2 pt blue outer ring, whereas Photoshop has a 1 pt blue border; the dialog cannot be resized by dragging.

## Test Coverage

- `opens_like_photoshop`: opening at 734 × 811 / 72 ppi shows 10.194 × 11.264 inches, 1.70M, 734 px × 811 px, method Automatic.
- `the_chain_and_units`: conversions between pixels and percent and centimeters, the chain's proportional linking, "436.5K (was 1.70M)".
- `resolution_keeps_the_printed_size_or_the_pixels`: in inches, 144 ppi doubles the pixels; in pixels, the pixels do not change; with Resample off, changing inches changes the resolution and pixels cannot be entered; Pixels/Centimeter conversion.
- `fit_to_presets`: the 1024 × 768 preset turns into a 768 × 1024 box for a portrait image, giving 768 × 849; 4 × 6 in 300 dpi gives 1200 × 1326; after a change it is Custom, and Original Size restores it.
- `ui_tests::image_size_resamples_proportionally`: entering 5.097 inches and confirming makes the document 367 × 405 and records "Image Size".
- `ui_tests::image_size_dialog_controls`: ⌥5 chooses Bicubic Sharper; clicking the chain unlinks it; turning off Resample restores the pixels and the chain, and ⌥ number keys have no effect; Cancel closes.
- `ui_tests::screenshot_image_size_dialog` (ignored, run manually): for comparison with Photoshop screenshots.
