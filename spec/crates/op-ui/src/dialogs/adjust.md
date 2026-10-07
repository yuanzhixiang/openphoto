# dialogs/adjust.rs: Adjustment and filter dialogs

## Component responsibilities

Adjustment and filter dialogs with settings: the adjustments with dialogs under Image › Adjustments, and the Filter menu's Gaussian Blur..., Box Blur..., Surface Blur..., Motion Blur..., Unsharp Mask..., Add Noise..., Dust & Scratches..., Median..., Minimum..., Maximum..., High Pass..., Offset..., Mosaic..., Emboss..., Twirl..., Pinch..., Spherize..., Polar Coordinates..., Custom..., Trace Contour..., Wind.... Levels..., Curves..., Brightness/Contrast..., Color Balance..., Hue/Saturation..., Channel Mixer..., Selective Color..., Vibrance..., Posterize..., Exposure..., Photo Filter..., Black & White..., Threshold..., Gradient Map... have been rebuilt after Photoshop 2026 (that is, every adjustment with a dialog), with their layouts and settings in their own modules (`levels.md`, `curves.md`, `brightness_contrast.md`, `color_balance.md`, `hue_saturation.md`, `channel_mixer.md`, `selective_color.md`, `vibrance.md`, `exposure.md`, `photo_filter.md`, `black_white.md`, `threshold.md`, `gradient_map.md`); this module only draws their window frame and title, forwards OK/Cancel, and handles preview. The dialogs only manage settings and the Preview toggle; preview and apply are done by `lib.rs` (see "Hooking up adjustment and filter dialogs" in `lib.md`). See `adjust.md` and `filter.md` in `op-core` for the pixel algorithms.

A dialog's result is an `Effect`: `Adjustment(Adjustment)` or `Filter(Filter)`. `Effect::name()` is the history name, and `Effect::apply(doc, background)` calls the corresponding `op-core` function.

## Data input

- `AdjustDialog::new(kind, histogram, before)`: `kind` is the dialog kind; `histogram` is the histogram within the active layer's selection at open time (Threshold uses luminance); `before` is the document snapshot at open time.
- Each setting is a "parameter": label, range, default value, number of decimal places, and type — numeric (input box + slider), choice (radio buttons, value is the index of the selected item), checkbox (value 0 or 1). Text in input boxes is formatted by the number of decimal places; Add Noise, Minimum and Maximum strip trailing zeros as Photoshop does (showing "12.5", "1"), the rest keep them (Gaussian Blur shows "1.0"). Defaults and ranges are the same as Photoshop:

| Dialog | Parameters (range, default) |
|---|---|
| Gaussian Blur | Radius (pixels) (0.1–1000.0, 1.0) |
| Box Blur | Radius (pixels) (1–2000, 1) |
| Unsharp Mask | Amount (%) (1–500, 50), Radius (pixels) (0.1–1000.0, 1.0), Threshold (levels) (0–255, 0) |
| Add Noise | Amount (%) (0.1–400, 12.5, at most two decimal places), Distribution (Uniform / Gaussian, Uniform), Monochromatic (unchecked) |
| Median | Radius (pixels) (1–500, 1) |
| Minimum, Maximum | Radius (pixels) (0.2–500.0, 1.0), Preserve (Squareness / Roundness, Squareness) |
| Motion Blur | Angle (°) (−360–360, 0), Distance (pixels) (1–2000, 10) |
| Emboss | Angle (°) (−180–180, 135), Height (pixels) (1–100, 3), Amount (%) (1–500, 100) |
| Twirl | Angle (°) (−999–999, 50) |
| Pinch | Amount (%) (−100–100, 50) |
| Spherize | Amount (%) (−100–100, 100), Mode (Normal / Horizontal only / Vertical only, Normal) |
| Polar Coordinates | Options (Rectangular to Polar / Polar to Rectangular, Rectangular to Polar) |
| High Pass | Radius (pixels) (0.1–1000.0, 10.0) |
| Offset | Horizontal (pixels right), Vertical (pixels down) (−30000–30000, 0), Undefined Areas (Set to Transparent / Repeat Edge Pixels / Wrap Around, Set to Transparent) |
| Mosaic | Cell Size (square) (2–200, 10) |
| Surface Blur | Radius (pixels) (1–100, 5), Threshold (levels) (2–255, 15) |
| Dust & Scratches | Radius (pixels) (1–500, 1), Threshold (levels) (0–255, 0) |
| Trace Contour | Level (0–255, 128), Edge (Lower / Upper, Upper) |
| Wind | Method (Wind / Blast / Stagger, Wind), Direction (From the Right / From the Left, From the Right) |
| Crystallize | Cell Size (3–300, 10) |
| Pointillize | Cell Size (3–300, 5) |
| Diffuse | Mode (Normal / Darken Only / Lighten Only / Anisotropic, Normal) |
| Ripple | Amount (%) (−999–999, 100), Size (Small / Medium / Large, Medium) |
| ZigZag | Amount (−100–100, 30), Ridges (0–20, 4), Style (Around Center / Out From Center / Pond Ripples, Around Center) |
| HSB/HSL | Input mode (RGB / HSB / HSL, RGB), Row order (RGB / HSB / HSL, HSB) |
| Tiles | Number Of Tiles (1–99, 10), Maximum Offset (%) (1–99, 10), Fill Empty Area With (Background Color / Foreground Color / Inverse Image / Unaltered Image, Background Color) |
| Color Halftone | Max. Radius (pixels) (4–127, 8), Channel 1–4 screen angles (−360–360; 108, 162, 90, 45) |
| Mezzotint | Type (Fine Dots, Medium Dots, Grainy Dots, Coarse Dots, Short Lines, Medium Lines, Long Lines, Short Strokes, Medium Strokes, Long Strokes; Fine Dots) |

- Filter dialogs remember the settings from the last time OK was pressed (Photoshop's behavior): `settings()` returns the text in the input boxes, `lib.rs` stores it by `Kind` into `AppState::filter_settings` when applying, and `commands.rs` puts it back with `restore(values)` the next time the same dialog opens (ignored when the count does not match). Cancel is not remembered. This is valid only within the current run and is not written to preferences. Adjustment dialogs (the adjustment variants of `Custom`) open with default values, and `settings()` returns `None`; but when the menu command is chosen with Option held they open with the settings of their last OK, as Photoshop's "last settings" (`remembered()` clones the dialog at OK into `AppState::last_adjustments` by kind, `recall` puts it back when the same kind opens; Levels and Curves then take the current histograms with `set_histograms`, keeping their values); the Custom filter (`Custom::Kernel`) is likewise remembered, see `custom_filter.md` for details.
- Preview is checked by default.

## Layout and visuals

### Classic filter dialogs

Gaussian Blur, Box Blur, Surface Blur, Motion Blur, Unsharp Mask, Add Noise, Dust & Scratches, Median, Minimum, Maximum, High Pass, Offset, Mosaic, Emboss and Trace Contour are rebuilt point by point after Photoshop 2026's classic dialogs (`classic_ui`); each dialog's size and row positions are in `filter_layout.rs` (see `filter_layout.md`):

- The window frame and title bar are `common::frame` (title AppKit 13 pt bold).
- Top right: OK (the default button) at y 38.5 and Cancel at 73.5, left edge 90.5 from the window's right edge, width 59.5 (80 for Gaussian Blur and High Pass, 60 for Offset), height 26; below them the Preview checkbox.
- The preview pane at top left (present on all except Offset): within the `PANE` area, part of the document (with the current preview effect) is shown centered, two preview pixels a point (one image pixel per screen pixel at 100%), without smoothing.
  - The texture is regenerated by `pane_texture` in `lib.rs` when the dialog opens, after the preview changes, and after the zoom or the view moves. It is at most `pane_px()` pixels: 392 × 392 here, 512 × 512 in the plug-in style dialogs.
  - Zoom and position live in `pane_zoom` (1 is 100%) and `pane_center` (the document point in the middle; the document's middle at first, kept so that the view stays on the document).
  - Below the preview pane are the zoom controls: zoom out, the zoom (for example "100%", "66.67%", "6.25%"), and zoom in. They step through Photoshop's zooms (6.25, 8.33, 12.5, 16.67, 25, 33.33, 50, 66.67, 100, 200–800, 1200, 1600%), and each is dimmed at its end.
  - Dragging in the pane moves the picture with the pointer (a grab cursor).
  - Clicking on the document outside the dialog (when no eyedropper is sampling) centers the preview on that point.
  - Known differences:
    - Photoshop's preview pane shows the effect whether or not Preview is checked, while here it shows the original when unchecked.
    - Pressing in Photoshop's pane shows the image before the filter, which this one doesn't.
    - The effect is computed on the whole document, not just the visible part.
- Numeric rows: a label right-aligned to the input box, a 19 pt tall input box (`appkit::field`; on open the first input box takes focus with its contents selected), unit text (the angle's "°" sits right against the input box); below, a 3 pt gray track, with the tip of a white triangle marker 0.5 pt below the track; the marker travels from 2.25 pt left of the track start to 0.75 pt right of the track end, its position converted by the row's `Scale` (most of Photoshop's sliders are non-uniform).
- The angle in Motion Blur and Emboss has an angle dial to the right of the input box: a line inside the circle pointing from the center toward the angle (Motion Blur's line passes through the center to both ends).
- Choices: Preserve in Minimum and Maximum is a dropdown; Add Noise's Distribution, Offset's Undefined Areas, Trace Contour's Edge and Diffuse's Mode are titled group boxes with radio buttons; Add Noise's Monochromatic is a checkbox. On the Background layer (`Extra::on_background`, set on open) Offset's first choice reads "Set to Background" instead of "Set to Transparent", as in Photoshop (the filter fills with the background color there either way).

### Plugin-style distort dialogs

Twirl, Pinch, Spherize, Polar Coordinates, Wind, Crystallize, Pointillize, Ripple, ZigZag and Mezzotint are rebuilt after Photoshop 2026's plugin-style dialogs (`distort_ui`), with layout and drawing in `distort.rs` (see `distort.md`): a large preview pane (with scroll troughs and a zoom bar at bottom left), OK / Cancel at top right (89 × 26, 13 pt text), settings below the preview pane (input box + pentagon slider, or the radio groups of Polar Coordinates and Wind), and a distortion diagram at bottom right. The settings still live in this module's parameter table, so remembering, validation and preview are the same as for classic dialogs.

Tiles, Color Halftone and HSB/HSL have no preview: their layouts are in `plain_filter.rs` (`plain_ui`, see `plain_filter.md`), and they don't preview on the document. Tiles' Foreground Color fill takes the app's foreground color when applied (`lib.rs` fills it in).

Every filter dialog has either a classic, a plugin-style or a plain layout (`Kind::size` panics when there is no layout, and `every_filter_dialog_has_an_effect` exposes omissions); the old generic layout has been removed.

## Rebuilt dialogs (`Custom`)

- For these dialogs `AdjustDialog` holds their own state (the variants of `Custom`); `effect()` is taken from them, and `show` uses their size and calls their `ui`; the window frame and title bar are still `common::frame` (title in AppKit 13 pt bold).
- They return `uxp::Button`: OK → `Outcome::Apply`, Cancel → `Outcome::Cancel`, Brightness/Contrast's Auto → computes the Auto values from the histogram (see `brightness_contrast.md`).
- Custom filter (`Custom::Kernel`, see `custom_filter.md`): the host first draws the preview pane and zoom controls (`wants_pane()` is true for it), then the grid; its Load... and Save... requests are carried out by the host with the system file dialog (`load_kernel` / `save_kernel`).
- `set_channel_histograms(h)`: when Levels or Curves opens, `commands.rs` passes in the histograms of the red, green and blue channels.
- `set_colorize_hue(hue)`: when Hue/Saturation opens, `commands.rs` passes in the foreground color's hue as the initial Colorize hue (Photoshop's behavior).

## Interaction

- Input boxes: when any parameter is out of range or not a number, OK is grayed out and there is no preview. Every filter dialog must map to a `Filter` in `effect()`, otherwise OK is always grayed out.
- Triangle marker: pressing or dragging on the track moves it to the pointer position, converting the value by the row's scale (rounded to the number of decimal places). The pentagon sliders in plugin-style dialogs take integers linearly.
- Angle dial: pressing or dragging inside the dial takes the angle of the pointer's direction (whole degrees); Motion Blur's angle wraps into −90–90.
- Preview: when checked, the document shows the result live; when unchecked, it reverts to the original.
- OK or Enter (when all values are valid): returns `Outcome::Apply(adjustment)`; Cancel or Esc: returns `Outcome::Cancel`.
- Modal while open.

## Sampling the image and the Color Picker

- `sampling()`: an eyedropper is chosen in Levels, Curves or Hue/Saturation; `targeting()`: Hue/Saturation's hand is on. `lib.rs` then takes a click on the image outside the dialog (`rect`, where it was last drawn): it removes the preview, reads the merged image's pixel there and passes it to `sample(rgb)` or `target_press(rgb, command)`; a hand drag follows with `target_drag(dx)` until release (`target_from`).
- Photo Filter's swatch asks for the Color Picker with `take_color_request()`; `set_filter_color` takes its OK. While the Color Picker is open, `blocked` makes the dialog ignore keys and buttons.

## The dialogs added after the measured ones

Radial Blur, Smart Blur, Shape Blur, Lens Blur, Reduce Noise, Smart Sharpen, Fibers, Lens Flare, Extrude, Oil Paint, Wave, Shear and Displace (filters), and Shadows/Highlights, HDR Toning, Replace Color, Match Color and Color Lookup (adjustments) use the classic layout kit (`filter_layout.md`) with fields in the order of their settings (`Kind::params`), and build their effects from them (`op-core`'s `more_filters.md`, `tone.md`, `color_match.md`):

- What they need besides their fields is in `Extra`: a seed drawn at opening (Fibers, Wave, Extrude: a new pattern each time), the foreground and background colors (`set_colors`: Fibers; Replace Color starts from the foreground), Match Color's Lab statistics of the active layer and of the other open documents (`set_match_sources`, sampled at most every 20 000th pixel), Color Lookup's cube files (`lut_files`: Photoshop's `Presets/3DLUTs` when installed and `~/Library/Application Support/OpenPhoto/3DLUTs`, `.cube` and `.3dl`), and Displace's map.
- Popups whose choices the dialog fills in (`Extra::labels`): Match Color's Source (None, then the other documents' titles) and Color Lookup's 3DLUT File ("Load 3D LUT...", then the files). Picking a file loads it into the lookup registry (`load_lut`); "Load 3D LUT..." opens the open panel and adds the file. Color Lookup's OK stays off until a cube is chosen.
- Replace Color always samples: a click on the image outside the dialog sets its color (`sample`). Its preview box shows the active layer made small (`thumbnail`, at most 370 × 280 pixels, taken on open into `Extra::thumb`) either as the selection, gray by `color_match::replace_weight` for the sampled color and Fuzziness (white is fully replaced), or as the image, by the Selection and Image radio buttons (`Extra::show_image`, Selection at first); the picture is rebuilt only when the color, Fuzziness or the choice changes.
- Shadows/Highlights opens short, with only the two Amounts; its "Show More Options" checkbox (an eleventh setting that the effect ignores, remembered with the others) switches to the layout with every setting, resizing the dialog. Match Color with Source None matches the layer to itself, so only Luminance, Color Intensity, Fade and Neutralize (which takes the mean color cast out) change it.
- Displace has no preview; after OK the app asks for the map (`lib.rs`), as Photoshop does, and Cancel there cancels the filter.
- Radial Blur's and Lens Flare's centers are the image's middle; Smart Blur's Quality, Lens Blur's iris (shape only), Reduce Noise's Remove JPEG Artifact, Smart Sharpen's Shadows/Highlights fades, Oil Paint's Bristle Detail and Extrude's Mask Incomplete Blocks are shown but have no effect; Shear is three offsets (top, middle, bottom, in percent of half the width) instead of Photoshop's curve grid.

These dialogs' layouts (sizes and positions, including Shadows/Highlights' two layouts and Replace Color's preview box) have not been measured against Photoshop 2026 yet. Replace Color's selection preview has no eyedropper add/subtract tools or Localized Color Clusters, and Shadows/Highlights has no Load/Save or Save Defaults buttons.

## Test coverage

- `ui_tests::offset_on_the_background_says_set_to_background`: the Background layer marks the dialog, a new layer doesn't; screenshot `offset_background.png`.
- `ui_tests::filter_preview_zooms_and_pans`: Gaussian Blur's zoom in gives 200%, two zoom outs 66.67%; at 200% a 20 pt drag moves the center 20 pixels; a click on the document centers the preview there.
- `ui_tests::shadows_highlights_more_options_and_replace_color_preview`: Shadows/Highlights is 170 pt tall until Show More Options makes it 560; Replace Color has a thumbnail and a selection picture, and its Image radio switches the box. Screenshots `shadows_highlights_more.png`, `replace_color_preview.png`.
- `ui_tests::more_filters_and_adjustments_apply`: each new filter and adjustment opens, Enter applies its defaults (Replace Color after sampling and a hue shift), records its name and becomes the Last Filter where it is one; Shear with a top offset applies; Color Lookup's OK is off without a cube; Match Color with Neutralize applies.

- `defaults_match_photoshop`: the default adjustments for Levels, Exposure and Hue/Saturation.
- `filters_read_choices_and_checkboxes`: Add Noise's distribution and monochromatic, and Offset's undefined areas options map to the corresponding filter parameters.
- `fields_show_values_as_photoshop_does`: the radius in Gaussian Blur and Unsharp Mask shows "1.0", Add Noise shows "12.5", Minimum and Maximum show "1".
- `every_filter_dialog_has_an_effect`: the default values of every filter dialog using the parameter table yield a filter (a missing mapping would gray out OK), and every one has a layout.
- `filter_dialogs_remember_their_last_values` and `more_filters_from_the_menu` in `ui_tests.rs`, plus screenshot tests, cover remembering, opening from the menu and applying, and layout.
