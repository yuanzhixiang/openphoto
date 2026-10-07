# dialogs/filter_layout.rs: Layout of the classic filter dialogs

## Component responsibilities

Records the sizes, per-row control positions and slider scales of Photoshop 2026's classic filter dialogs (Gaussian Blur, Box Blur, Unsharp Mask, Add Noise, Median, Minimum, Maximum, High Pass, Offset, Mosaic, Motion Blur, Emboss, Surface Blur, Dust & Scratches, Trace Contour). It contains data only and draws nothing; drawing and interaction are in `adjust.rs`'s `classic_ui` (see "Classic filter dialogs" in `adjust.md`).

All coordinates are Photoshop points from the dialog's top-left corner (including the title bar), each measured from Photoshop 2026 screenshots.

## Data input

- `Layout`: `size` (width, height), `button_width` (OK / Cancel width; mostly 59.5, 80 for Gaussian Blur and High Pass, 60 for Offset), `preview_y` (top edge of the Preview checkbox), `pane` (whether there is a preview pane; Offset has none), `rows`.
- `rows` correspond one to one to the dialog's parameters, in the same order:
  - `Row::Number`: label and its right end, input field rectangle, unit and its start, optional track (start, end, top edge), scale `Scale`, optional angle dial (center, radius).
  - `Row::Popup`: label and its right end, dropdown rectangle.
  - `Row::Radios`: group box title, group box rectangle (the title sits on the top border), the y of each radio button's center (x fixed at 27).
  - `Row::Check`: checkbox label and top-left corner.
- `PANE` (16, 43)–(212, 239) is the preview pane, and `ZOOM_Y` 261.75 is the center line of the zoom controls.
- Sizes: Gaussian Blur, High Pass 324 × 335; Box Blur, Median 324 × 342; Minimum, Maximum 324 × 378; Unsharp Mask 324 × 436; Add Noise 324 × 452; Mosaic 324 × 338; Motion Blur 324 × 389; Emboss 324 × 431; Surface Blur 324 × 395; Dust & Scratches 324 × 387; Trace Contour 324 × 425 (the Edge group box is in the same position as Add Noise's Distribution); Offset 323 × 256.
- Box Blur and Median have the same layout but different scales (1–2000 for the former, 1–500 for the latter), so they are two `Layout`s.

## Slider scales (`Scale`)

Most of Photoshop's sliders are not uniform. The scales were measured by entering values one by one in Photoshop and reading off the position of the triangle marker; `place(v, min, max)` gives the 0–1 position, and `value(place, min, max)` is its inverse:

- `Linear`: uniform from the parameter's minimum to its maximum (Add Noise, Mosaic, Emboss's Height, Surface Blur, Offset).
- `RADIUS_1000`: a 0.1–1000 radius (Radius of Gaussian Blur, High Pass, Unsharp Mask), e.g. 1 at 0.048, 10 at 0.402, 250 at 0.831.
- `RANGE_2000`: 1–2000 (Box Blur's radius, Motion Blur's distance).
- `RANGE_500`: 1–500 (radius of Median, Minimum, Maximum, Dust & Scratches; Amount of Unsharp Mask and Emboss).
- `LEVELS_255`: 0–255 thresholds (Unsharp Mask, Dust & Scratches) and Trace Contour's Level (a separate scan of 12 values verified the same table).
- Values between table entries are linearly interpolated; beyond either end of the table, 0 or 1 is used.

## Edge cases and limitations

- Offset's slider scale has not been measured and is treated as linear for now.
- Surface Blur's two slider scales have not been measured and are treated as linear for now.
- There is only a single 100% zoom level, and the preview pane cannot be dragged (see `adjust.md`).

## Test coverage

- `scales_match_photoshops_pins`: the marker positions for Gaussian Blur radius 1 and 250 differ from Photoshop's measurements by less than half a point; Add Noise 100% is at 42.5 on a 170-point track; `place` and `value` are inverses for every scale table.
- `screenshot_filter_dialogs` in `ui_tests.rs` generates screenshots of all classic filter dialogs for side-by-side comparison with Photoshop.
