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

- Filter dialogs remember the settings from the last time OK was pressed (Photoshop's behavior): `settings()` returns the text in the input boxes, `lib.rs` stores it by `Kind` into `AppState::filter_settings` when applying, and `commands.rs` puts it back with `restore(values)` the next time the same dialog opens (ignored when the count does not match). Cancel is not remembered. This is valid only within the current run and is not written to preferences. Adjustment dialogs (the adjustment variants of `Custom`) open with default values every time, and `settings()` returns `None`; the Custom filter (`Custom::Kernel`) is likewise remembered, see `custom_filter.md` for details.
- Preview is checked by default.

## Layout and visuals

### Classic filter dialogs

Gaussian Blur, Box Blur, Surface Blur, Motion Blur, Unsharp Mask, Add Noise, Dust & Scratches, Median, Minimum, Maximum, High Pass, Offset, Mosaic, Emboss and Trace Contour are rebuilt point by point after Photoshop 2026's classic dialogs (`classic_ui`); each dialog's size and row positions are in `filter_layout.rs` (see `filter_layout.md`):

- The window frame and title bar are `common::frame` (title AppKit 13 pt bold).
- Top right: OK (the default button) at y 38.5 and Cancel at 73.5, left edge 90.5 from the window's right edge, width 59.5 (80 for Gaussian Blur and High Pass, 60 for Offset), height 26; below them the Preview checkbox.
- The preview pane at top left (present on all except Offset): within the `PANE` area, the center part of the document (with the current preview effect) is shown centered at 100% (one image pixel per screen pixel), without scaling or smoothing; the texture is regenerated by `pane_texture` in `lib.rs` on open and after the preview changes (cropping at most 392 × 392 pixels from the center). Below the preview pane are zoom controls: zoom out (dimmed at 100%), "100%", zoom in, currently display only. Known differences: Photoshop's preview pane shows the effect whether or not Preview is checked, while here it shows the original when unchecked; it also cannot yet zoom or be dragged to view other parts.
- Numeric rows: a label right-aligned to the input box, a 19 pt tall input box (`appkit::field`; on open the first input box takes focus with its contents selected), unit text (the angle's "°" sits right against the input box); below, a 3 pt gray track, with the tip of a white triangle marker 0.5 pt below the track; the marker travels from 2.25 pt left of the track start to 0.75 pt right of the track end, its position converted by the row's `Scale` (most of Photoshop's sliders are non-uniform).
- The angle in Motion Blur and Emboss has an angle dial to the right of the input box: a line inside the circle pointing from the center toward the angle (Motion Blur's line passes through the center to both ends).
- Choices: Preserve in Minimum and Maximum is a dropdown; Add Noise's Distribution, Offset's Undefined Areas, Trace Contour's Edge and Diffuse's Mode are titled group boxes with radio buttons; Add Noise's Monochromatic is a checkbox.

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

## Test coverage

- `defaults_match_photoshop`: the default adjustments for Levels, Exposure and Hue/Saturation.
- `filters_read_choices_and_checkboxes`: Add Noise's distribution and monochromatic, and Offset's undefined areas options map to the corresponding filter parameters.
- `fields_show_values_as_photoshop_does`: the radius in Gaussian Blur and Unsharp Mask shows "1.0", Add Noise shows "12.5", Minimum and Maximum show "1".
- `every_filter_dialog_has_an_effect`: the default values of every filter dialog using the parameter table yield a filter (a missing mapping would gray out OK), and every one has a layout.
- `filter_dialogs_remember_their_last_values` and `more_filters_from_the_menu` in `ui_tests.rs`, plus screenshot tests, cover remembering, opening from the menu and applying, and layout.
