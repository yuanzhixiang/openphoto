# adjust.rs: Image Adjustments

## Responsibilities

Implements the per-pixel adjustments in Image › Adjustments. They act on the active layer, limited to the selection. No history is recorded here (`op-ui` records it under the adjustment's name).

## Public Interface

- `Adjustment`:
  - `Invert`: each color channel becomes `255 - v`.
  - `Desaturate`: all three channels are set to the pixel's HSL lightness `(max + min) / 2` (rounded up), matching Photoshop's Desaturate (not a weighted luminosity).
  - `EqualizeEntireImage`: builds the same table from the histogram inside the selection, but applies it to the whole layer (ignoring the selection).
  - `Threshold(level)`: pixels whose luminosity (`luminosity`) is greater than or equal to `level` become white; all others become black. Photoshop's range is 1–255, default 128.
  - `Posterize(levels)`: each channel is quantized to `levels` levels: `round(round(v × (L−1) / 255) × 255 / (L−1))`. Range 2–255, default 4.
  - `Equalize`: histogram equalization. The R, G and B values of pixels with nonzero alpha inside the selection on the active layer are counted together into one histogram (`channel_histogram`); each value maps to its position in the cumulative histogram × 255 (rounded), and the same table is applied to all three channels.
  - `Levels([Levels; 4])`: one set each for the RGB composite channel and the red, green and blue channels (`Levels { input_black, input_white, gamma, output_black, output_white }`). Each channel's own levels are applied first, then the composite's (Photoshop's order). `t = clamp((v − input black) / (input white − input black), 0, 1)`, passed through `levels_gamma(t, gamma)`, then mapped to the output black and white points, rounding halves up. When the input white point is not greater than the black point, it is treated as black point + 1. `Levels::composite()` sets only the composite channel.
  - `levels_gamma(t, gamma)`: Photoshop's gamma curve (measured from Levels in Photoshop 2026): `t^(1/gamma)`, but when gamma is greater than 1 the slope at the black end is limited to `2^gamma`: it starts from 0 with the quadratic `a·t + b·t²` (`a = 2^gamma`) and joins the power curve at the point where their slopes are equal. From gamma 8 up, the power curve itself is no steeper than that, so this segment does not exist. Differs from Photoshop by at most 2 levels (within 6 levels at the darkest few levels for gamma 5–7).
  - `HueSaturation(HueSaturation)`: three Master values, six color ranges (`HueRange { bounds, hue, saturation, lightness }`, default bounds `HUE_RANGES`: Reds 315/345/15/45, the rest one group every 60°), and Colorize. Per pixel (fitted to Photoshop 2026, see tests):
    - Each range contributes with a weight based on the pixel's original hue: linear ramp in and out between the bounds, half a degree later than the bounds (`HueRange::weight`).
    - The hue rotation is Master's plus each range's weighted amount, keeping the maximum and minimum channels unchanged;
    - Master lightness: positive values blend toward white `v + (255 − v)·k`, negative values toward black `v·(1 + k)`;
    - Each range's weighted lightness uses a different algorithm: positive values move each channel toward the maximum channel, negative values toward the minimum channel;
    - Then each range's weighted saturation first, then Master's saturation: for positive values, each channel moves away from the HSL lightness `L = (max+min)/2` by a factor of `1/a − 1`, where `a` is `1 − amount`; when the amount plus the color's HSL saturation reaches 1, `a` is that saturation (i.e., pushed to full saturation). For negative values, channels shrink toward L by a factor of `(1 + amount)`.
    - Colorize: takes the pixel's HSL lightness, adjusts it by Master lightness, then forms a color with Master's hue (0–360) and saturation (0–100).
    - Results differ from Photoshop by: at most 4 levels for Master, 3 levels for Colorize, 4 levels for color ranges (at the edges of the ramps).
    - `HueSaturation::master(h, s, l)` builds a setting with only Master; `HueSaturation::apply(px)` applies to one pixel (the dialog's Before - After color bars also use it).
  - `Exposure { exposure, offset, gamma }`: computed in gamma-2.2 linear light (not the sRGB curve, matching Photoshop): `v^2.2` multiplied by `2^exposure`, plus `offset`, negative values clipped to 0, then `^(1/gamma)`, and finally `^(1/2.2)`. Differs from Photoshop by at most 2 levels.
  - `BrightnessContrast { brightness, contrast, legacy }` (−150–150, −50–100): the default algorithm uses Photoshop 2026's curves directly: `data/brightness_contrast.bin` stores the 256-level table Photoshop outputs for each brightness value (301 entries) and each contrast value (151 entries), composed as "brightness first, then contrast" (differs from Photoshop's combined result by at most 1 level). No closed form was found for these curves (the initial slope of brightness is `2^(b/110)`, and positive and negative brightness are inverses of each other). `legacy` (Use Legacy): brightness shifts `v + b`; positive contrast stretches around 128 by a factor of `100/(100 − c)` (at 100 it is a threshold at 128), negative contrast compresses by a factor of `(100 + c)/100`, with the offset rounded (halves away from the center); with negative contrast, contrast comes before brightness; with positive contrast, brightness comes before contrast (matching Photoshop's results).
  - `ColorBalance { shadows, midtones, highlights, preserve_luminosity }`: one curve per channel (Photoshop's Color Balance is a per-channel lookup table, including Preserve Luminosity), shaped like Levels: when shadows are negative, input black point = −value; when highlights are positive, input white point = 255 − value; gamma is `2^(G/100)` (via `levels_gamma`). Without preserving luminosity, `G = midtones + (shadows + highlights)/2`; with preserving luminosity, across the three axes the shadows first subtract their maximum, the highlights subtract their minimum, the midtones subtract (max + min)/2, and only the midtones affect gamma. Differs from Photoshop 2026 by at most 3 levels over 80 random settings.
  - `BlackWhite { weights, tint }`: six weights (percentages) for red, yellow, green, cyan, blue and magenta. Gray = minimum channel + (middle channel − minimum) × secondary color weight + (maximum − middle) × primary color weight; the primary color is the largest channel (red/green/blue), the secondary color is the color formed by the two largest channels (yellow/cyan/magenta). Identical to Photoshop at every level (default preset 40, 60, 40, 60, 20, 80). When `tint` is a Tint color, it is set onto this gray with the "Color" blend (`set_lum`: shift by the luminosity 0.3 R + 0.59 G + 0.11 B, then pull back into 0–1 with ClipColor), differing from Photoshop by at most 2 levels.
  - `ReplaceColor { color, fuzziness, shift }`, `MatchColor { target, source, luminance, intensity, fade }`, `ColorLookup(id)`: see `color_match.md` (named "Replace Color", "Match Color", "Color Lookup").
  - `CurveTables([[u8; 256]; 4])`: Curves drawn with the pencil (or shown as clipping): tables for the composite and the red, green, blue channels, each channel's own first; named "Curves".
  - `Vibrance { vibrance, saturation }`: Vibrance is an approximation: in HSL, saturation is multiplied by `1 + vibrance × (1 − s)` (stronger effect on low-saturation colors); a positive vibrance is reduced by up to 70% around skin-tone hues (a Gaussian of 20° about 25°), as Photoshop protects skin tones. Saturation matches Photoshop (within 2 levels): in sRGB linear light, each channel is scaled by `1 + saturation/100` around the gray `0.2878 R + 0.7122 G` (blue has weight 0, matching Photoshop measurements).
  - `PhotoFilter { color, density, preserve_luminosity }`: in D50 XYZ (linear sRGB via `SRGB_TO_XYZ_D50`), X, Y and Z are each multiplied by `1 − density + density × the filter color's corresponding component / white's component`, then converted back to sRGB. This is exactly what Photoshop does (the eigenvectors of the fitted transform matrix are these primaries), differing from Photoshop by at most 1 level. With Preserve Luminosity, `set_lum` then sets the result's luminosity back to the original pixel's luminosity (within 6 levels).
  - `GradientTable { table, dither }`: Gradient Map with any gradient: the exact luminosity is interpolated between two of the 256 table colors; Dither adds up to ±half a level of repeatable per-pixel noise (`dither_noise`). Named "Gradient Map".
  - `GradientMap { from, to, method }`: the pixel's luminosity (0.299 R + 0.587 G + 0.114 B, matching Photoshop) determines its position on the gradient; the color is interpolated by `gradient::blend_colors` according to the method (see `gradient.md`).
  - `AutoTone`, `AutoColor`: each channel separately drops the darkest and brightest 0.1%, then stretches to 0–255 (Photoshop's Auto Color also neutralizes midtones; this one does not). `AutoContrast`: the three channels are counted together and stretched by the same range, keeping color relationships unchanged. The histogram uses pixels with nonzero alpha inside the selection on the current layer.
  - `Curves { points, counts }`: up to 16 (input, output) points each for the RGB composite, red, green and blue channels (`Adjustment::curves(points)` sets only the composite channel; `curves_per_channel([..; 4])` sets all four channels; an empty list means that channel is unchanged). Each channel's own curve is applied first, then the composite curve. `curve_table(points)` builds the lookup table: sort by input, remove duplicate inputs, then fit a natural cubic spline (second derivative 0 at both ends); before the first point and after the last point it stays flat; results are clamped to 0–255 and rounded. Identical at every level to 12 curves from Photoshop 2026. With no points it is the identity; with one point it is a constant.
  - `ChannelMixer { rows, monochrome }`: each output channel (with Monochrome, the same gray for all three channels) = (R × red% + G × green% + B × blue%) / 100 + constant% × 2.55, rounding halves up and clamped to 0–255. Differs from Photoshop 2026 by at most 1 level.
  - `SelectiveColor { colors, absolute }`: nine ranges (Reds, Yellows, Greens, Cyans, Blues, Magentas, Whites, Neutrals, Blacks), each with C, M, Y, K (−100–100%). Each range has a weight for a pixel: for Reds/Greens/Blues, the difference between the maximum and middle values when that channel alone is the maximum; for Cyans/Magentas/Yellows, the difference between the middle and minimum values when red/green/blue alone is the minimum; Whites is `2·(min − 50%)`, Blacks is `2·(50% − max)` (not less than 0); Neutrals is `1 − (|max − 50%| + |min − 50%|)`. Each channel's ink amount `ink = 1 − v` (C for red, M for green, Y for blue) changes per range by `weight × clamp(d, −ink, 1 − ink)`, where `d = amount + K × (1 + amount)`; with Relative, d is further multiplied by ink. The changes from all ranges are summed (not applied in sequence). Differs from Photoshop 2026 by at most 1 level over a dozen or so settings (including multiple ranges and Absolute).
  - Levels, Exposure, Brightness/Contrast, Color Balance, Curves, Equalize and the Auto family first compute 256-entry lookup tables for the three channels (`Adjustment::tables()`), then look up each pixel.
- `Adjustment::name()`: the menu and history name ("Invert", "Desaturate", "Threshold", "Posterize", "Equalize", "Levels", "Hue/Saturation", "Exposure", "Brightness/Contrast", "Color Balance", "Black & White", "Vibrance", "Photo Filter", "Gradient Map", "Auto Tone", "Auto Contrast", "Auto Color", "Curves", "Channel Mixer", "Selective Color").
- `rgb_histograms(doc)`: the red, green and blue histograms of pixels with nonzero alpha inside the selection on the active layer (Levels and Curves dialogs).
- `channel_histogram(doc)`: the histogram of the R, G and B values counted together for pixels with nonzero alpha inside the selection on the active layer (used by Equalize and the Levels dialog).
- `hsl_color(h, s, l)`, `hue_of(rgb)`: conversion between HSL and RGB (the Colorize color, the foreground color's hue).
- `mask_gray(rgb)`: the gray a color paints on a layer mask (luminosity, the same in all three channels).
- `luminosity(px)`: luminosity `(299 R + 587 G + 114 B) / 1000`, rounded to 0–255 (Rec. 601 weights).
- `luminosity_histogram(doc)`: the luminosity histogram of pixels with nonzero alpha inside the selection on the active layer (shown by the Threshold dialog).
- `check(doc)`: the check before an adjustment; on failure it returns a `FillError` (no layer, layer hidden, pixels locked), with message text in the same format as Fill, e.g. "Could not complete the Invert command because the target layer is hidden.".
- `apply(doc, adjustment)`: runs `check` first, then applies per pixel.

## Behavior Rules

- Only color channels change; alpha stays the same. Fully transparent pixels are skipped.
- With a selection, only selected pixels are processed; partially selected pixels (feathered, anti-aliased edges) are linearly blended between the original color and the new color by their degree of selection.
- The Background layer can also be adjusted (the Background layer only cannot be moved or have transparency; its pixels can be modified).

## Known Limitations

- The Vibrance slider itself, including its skin-tone protection, is an approximation of Photoshop's.

## Data Sources

The Brightness/Contrast tables and the reference data under `fixtures/adjust/` were all generated by scripts in Photoshop 2026: an image with one gray ramp per row, each row given different parameters via a selection, saved as PNG and read back. `probe.rgb` is a 64 × 72 probe image (a 16-level RGB cube, gray ramps, random colors); the other `.rgb` files are Photoshop's output for it.

## Test Coverage

- `invert_desaturate_threshold_posterize`: each adjustment's result for pixels such as (200, 100, 0), including the Threshold boundary exactly on either side of luminosity 119.
- `equalize_stretches_the_range`: with only the values 100 and 200, they map to 128 and 255 respectively.
- `levels_hue_saturation_and_exposure`: Levels black/white point stretching and gamma 2 (128 → 181); red with hue +120° becomes green; saturation −100 plus lightness +50 gives 191 gray; exposure +1 stop turns 128 into 176.
- `color_adjustments`: with the Black & White default preset, pure red is 102 and pure yellow is 153; a black-to-red gradient map; Vibrance raises low-saturation colors and a muted orange gains under 60% of what an equally muted blue does; a 50% blue filter turns 200 gray into (100, 100, 200).
- `curves_pass_through_their_points`: a two-point curve is the identity; an S curve passes through each point and is monotonic; flat beyond the endpoints; a (64→128) curve turns 64 gray into 128.
- `auto_tone_stretches_each_channel`: with two pixels, each channel stretches to 0 and 255.
- `only_the_selection_changes`: pixels outside the selection do not change.
- `hidden_layers_are_refused`: a hidden layer returns an error with Photoshop's message text.
- `photoshop::*`: comparison against Photoshop 2026's output (`fixtures/adjust/`):
  - `hue_saturation_master_matches_photoshop`, `hue_saturation_colorize_and_ranges_match_photoshop`: seven Master settings, Colorize, and five color range settings, bounding the maximum error and the number of channels off by more than 1 level.
  - `levels_match_photoshop`: 28 Levels settings (various gamma, black/white points, output ranges).
  - `channel_levels_apply_before_the_composite`, `curves_per_channel`: individual channels come before the composite channel.
  - `brightness_contrast_matches_photoshop`: seven combinations and eleven Use Legacy settings.
  - `color_balance_matches_photoshop`: 80 random three-tone settings (including Preserve Luminosity), with error at most 3 per level.
- `channel_histograms`: the three channel histograms and the combined histogram.
- `photoshop::channel_mixer_matches_photoshop`, `photoshop::selective_color_matches_photoshop`: three channel mixer settings (including monochrome) and seven selective color settings compared against Photoshop.
- `photoshop::black_white_photo_filter_and_exposure_match_photoshop`: Black & White (including two Tint settings), four Photo Filter settings, four Exposure settings, and three Vibrance Saturation settings compared against Photoshop.
- `photoshop::gradient_map_matches_photoshop`: a red→blue gradient under four methods compared against Photoshop.
- `equalize_the_entire_image_from_the_selection`: with EqualizeEntireImage, pixels outside the selection change by the selection's table; with Equalize, they do not change.
