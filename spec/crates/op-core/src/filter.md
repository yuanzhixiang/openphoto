# filter.rs: Filters

## Responsibilities

Implements the filters in the Filter menu, acting on the active layer and limited to the selection. Records no history (`op-ui` records it under the filter's name).

## Public interface

- `Filter` (parameters in braces):
  - `GaussianBlur { radius }`: as measured in Photoshop 2026: for radius (Photoshop uses steps of 0.1) ≤ 2, Photoshop's own 8-bit integer kernels (`SMALL_KERNELS`, center and one-side weights summing to 256; the kernels for small radii are noticeably wider than a Gaussian with σ = r); for 2.1–2.9, measured floating-point kernels (`MID_KERNELS`; 2.2–2.4 and 2.5–2.6 share one each); for ≥ 3, five extended boxes each with variance r²/5 (the extended box of Gwosdek et al., integer radius l plus fractional end weights α) convolved into one kernel, applied as one separable row/column convolution with edge repeat. Within 1 level of Photoshop (probe images at radii 0.3, 1, 2.5 and 10).
  - `BoxBlur { radius }`: the average over a (2r + 1)² box, separable by rows and columns.
  - `Average`: fills the selection with the average color (weighted by selection degree) of all pixels in the selection (the whole layer when there is no selection).
  - `UnsharpMask { amount, radius, threshold }`: applies the Gaussian blur above with `radius`; the sharpening amount is `Δ = (original − blurred) × amount%`; the threshold is not a switch but is subtracted from the sharpening amount: result = `original + sign(Δ) × max(0, |Δ| − threshold)` (measured in Photoshop, within 2 levels). Only processes color channels.
  - `AddNoise { amount, gaussian, monochromatic }`: adds `n × amount% × 127.5` to each channel, where `n` is uniformly distributed in −1–1, or approximately normally distributed with unit variance (the sum of 4 uniform random numbers). When monochromatic, the three channels use the same random number. Random numbers are determined by pixel coordinates and channel, so the same parameters always give the same result, and the preview matches the final result.
  - `Median { radius }`: each channel takes the median within a (2r + 1)² box (matches Photoshop).
  - `Minimum { radius, round }`, `Maximum { radius, round }`: with Preserve set to Squareness, takes the minimum/maximum within a box whose radius is rounded (matches Photoshop; 2.5 means 7 × 7); with Roundness, uses a circular window where edge pixels take part with coverage `clamp(1 − (distance − radius), 0, 1)` (the result is the min/max of `original + (neighbor − original) × coverage`); this is an approximation, and individual pixels can differ significantly from Photoshop.
  - `HighPass { radius }`: `original − Gaussian-blurred + 128`.
  - `Offset { dx, dy, fill }`: moves the whole layer right by `dx` and down by `dy`. The vacated area follows `OffsetFill`: `Background` (transparent for regular layers, the background color for the background layer), `RepeatEdges` (repeats edge pixels), `Wrap` (wraps around from the other side).
  - `Mosaic { cell }`: divides the layer from the top-left corner into `cell` × `cell` cells and fills each cell with its average color (cells on the right and bottom edges may be incomplete).
  - `Solarize`: inverts (`255 − v`) each color channel value greater than 127.
  - `Crystallize { cell }`: one seed point at a repeatable random spot in each `cell` × `cell` block; each pixel joins the nearest seed (searching the 3 × 3 blocks around it) and every cell is filled with its pixels' average (premultiplied) color.
  - `Pointillize { cell }`: the background color (the `paper` passed to `filtered`, Photoshop's background color, on any layer), with dots `cell` pixels across at a random spot in each block of a grid 0.75 × `cell` apart, in the color under the dot's center varied by up to ±24 per channel; later dots cover earlier ones; opaque.
  - `Diffuse { mode }` (`DiffuseMode`): each pixel takes a random pixel of its 3 × 3 neighborhood (Normal), only a darker or lighter one by luminance (Darken Only / Lighten Only), or the one closest in color among three random picks (Anisotropic).
  - `Ripple { amount, size }`: a distortion (`distort_source`, so the dialog preview uses it too): a pixel takes its color from (x + a·sin(2πy/λ), y + a·sin(2πx/λ + 1)), λ 6 / 12 / 24 pixels for Small / Medium / Large and a = amount/100 · λ/4.
  - `Mezzotint { kind }` (`MezzotintType`): each color channel becomes 255 or 0 at random, the channel's value being the chance; Fine Dots draw a chance per pixel, Medium and Coarse Dots per 2 and 3 pixel blocks, Grainy Dots average a per-pixel and a 2-pixel chance; Short / Medium / Long Lines share a chance along horizontal runs of 4 / 8 / 16 pixels, Strokes along diagonal runs; alpha is kept.
  - Their randomness is repeatable (`noise` by position, like Wind's); Photoshop's own random patterns are not reproduced.
  - `Clouds { foreground, background, seed }`: fills the layer opaquely with `clouds` mixed from the foreground (0) to the background color (1). `DifferenceClouds`: the same color per pixel, in Difference mode over the layer (`|layer − cloud|` per channel, alpha kept).
  - `clouds(w, h, seed)`: fractal value noise: octaves from cells a quarter of the image's longer side (at least 32 pixels, rounded to a power of two) down to 2 pixels, each half as strong as the one before, smoothstep-interpolated between hashed lattice values, then stretched to 0–1 over the image. Deterministic for a seed, not repeating across the image; Photoshop's own noise is not reproduced, only its look.
  - `Blur`: 3 × 3 kernel [[0,1,0],[1,4,1],[0,1,0]] / 8; `BlurMore`: [[1,2,1],[2,2,2],[1,2,1]] / 14; `Sharpen`: center 2, up/down/left/right −¼; `SharpenMore`: center 3, all eight neighbors −¼. All measured with impulses and identical to Photoshop level for level.
  - `FindEdges`: each channel is `255 − √(gx² + gy²)`, where gx and gy are unnormalized Sobel kernels (within 1 level of Photoshop).
  - `MotionBlur { angle, distance }`: takes `distance + 1` points at 1-pixel intervals along the angle direction (starting at `−ceil(distance/2)`), each spread bilinearly, and averages them as the convolution kernel (the impulse response is these points, read in the opposite direction). Identical to Photoshop level for level when horizontal; diagonal is an approximation (about 12 levels of difference in the interior; at image edges Photoshop's border handling differs, so the difference is larger).
  - `Emboss { angle, height, amount }` (height 1–100 pixels): each channel is `128 + (I(p + s) − I(p − s)) × amount%`, with `s = (height/2)·(cos a, −sin a)`, sampled bilinearly. Matches Photoshop when horizontal or vertical; when diagonal, Photoshop's sampling is more concentrated and there is error (on probe images, 0.2–0.4 levels on average, at most 31 levels).
  - `Fragment`: averages four copies offset by 4 pixels in each of the four diagonal directions (within 1 level of Photoshop).
  - `Custom { kernel, scale, offset }`: a 5 × 5 convolution kernel (row by row from top to bottom, −999–999); the weighted sum is divided by `scale` (treated as 1 when 0), `offset` is added, and the result is rounded and clamped to 0–255. Only processes color channels (within 1 level of Photoshop).
  - `SurfaceBlur { radius, threshold }` (radius 1–100, threshold 2–255): each channel takes a weighted average within a (2r + 1)² box with neighbor weight `max(0, 1 − |neighbor − original| / (2.5 × threshold))`, so neighbors differing from the original by more than 2.5 times the threshold do not take part, and edges are preserved (within 1 level of Photoshop).
  - `DustAndScratches { radius, threshold }` (radius 1–500, threshold 0–255): first computes the median of each channel within a (2r + 1)² box; pixels where any channel differs from the median by more than the threshold are replaced entirely by the median, and the rest stay as they are (within 1 level of Photoshop).
  - `Despeckle`: each channel mixes between the original and Blur More (the 3 × 3 kernel / 14 above), with Blur More's share being `1 − e`, `e = clamp((|g| − 64) / 192, 0, 1)`, where `|g|` is the length of the channel's unnormalized Sobel gradient: flat areas are fully blurred, and edges with gradient ≥ 256 stay as they are (measured with isolated points, step edges and ramps; within 1 level of Photoshop).
  - `SharpenEdges`: with the same `e`, mixes by `e` between the original and Sharpen (clamped to 0–255 first): sharpens only edges, complementary to Despeckle (within 1 level of Photoshop).
  - `TraceContour { level, upper }`: each channel independently; results are only 0 and 255: with Upper, pixels whose value is ≤ level and that have an up/down/left/right neighbor > level are 0; with Lower, pixels whose value is ≥ level and that have a neighbor < level are 0; all others are 255. Matches Photoshop (on the Upper probe image only one channel value differs: Photoshop also traces an isolated 255 point whose surroundings exactly equal the level).
  - `Wind { method, from_left }`: Photoshop's Wind is random (two runs with the same parameters give different results), so it cannot be compared pixel by pixel; it is reproduced here from the measured statistical behavior, with random numbers determined by pixel coordinates, so the same parameters always give the same result (preview and apply match). Scans each row along the wind direction (rightward with `from_left`); where it darkens by more than a certain amount (weighted luminance 3R+6G+B difference > 40), a streak starts with probability one half, 8–31 pixels long:
    - `WindMethod::Wind`: the first pixel of the streak is the average of the pixels before and after; after that, each pixel decays toward the pixel it passes over by a random proportion (0.08–0.28);
    - `Blast`: extends the color of the previous pixel unchanged;
    - `Stagger`: carries the pixel further along the wind direction (24–93 pixels) before setting it down, and the pixel after it fills the original spot.
    - Wind and Blast only make pixels lighter (taking the larger value per channel); flat areas are unchanged.
  - The Distort filters all use "inverse mapping + bilinear sampling": each pixel takes the color at the position it maps from (`distort_source`); the mappings were measured in Photoshop 2026 with coordinate maps (R and G encode x and y). Twirl, Pinch and Spherize only act inside the ellipse touching the four sides of the image, with distance `t` measured so the ellipse radius is 1:
    - `Twirl { angle }`: rotation `angle × (1 − t)²` (largest at the center, 0 at the edge).
    - `Pinch { amount }`: sample distance `t + amount% × h(t)`, where `h` is a measured 21-point table (`PINCH_SHIFT`, proportional to the amount; positive values pull inward).
    - `Spherize { amount, mode }`: for positive values, sample distance `t + a × ((2/π)·asin t − t)`; for negative, `t + |a| × (sin(πt/2) − t)`; Horizontal only / Vertical only act along one direction only.
    - `PolarCoordinates { to_polar }`: Rectangular to Polar maps the angle around the center (counterclockwise from straight up) to x and the distance to the center to y; Polar to Rectangular is the reverse (the angle takes `(x + 1)/w` of a full turn).
    - Average difference from Photoshop: Twirl 0.5 levels, Pinch 0.2 levels, Spherize 1.2–1.7 levels, Polar 0.03–0.7 levels; isolated single-pixel bright points can differ more because of subpixel coordinate differences.
- `Filter::name()`: menu and history name ("Gaussian Blur", "Box Blur", "Average", "Unsharp Mask", "Add Noise", "Median", "Minimum", "Maximum", "High Pass", "Offset", "Mosaic", "Solarize", "Blur", "Blur More", "Sharpen", "Sharpen More", "Find Edges", "Motion Blur", "Emboss", "Twirl", "Pinch", "Spherize", "Polar Coordinates", "Fragment", "Custom", "Surface Blur", "Dust & Scratches", "Despeckle", "Sharpen Edges", "Trace Contour", "Wind", "Crystallize", "Ripple", "Mezzotint", "Pointillize", "Diffuse", "Clouds", "Difference Clouds").
- `distortion_source(filter, x, y, w, h)`: the source position from which a distort filter takes the color for pixel (x, y) in a `w` × `h` image (other filters return the original position), used by dialogs to draw the preview diagram.
- `apply(doc, filter, background)`: first performs the same checks as adjustments (`adjust::check`: returns `FillError` when there is no layer, the layer is hidden, or pixels are locked), then applies the filter. `background` is the background color Offset uses on the background layer.

## Behavior rules

- Filters read the whole layer for computation (blurring at selection edges uses pixels outside the selection) and only write selected pixels; partially selected pixels mix between the original and new values by selection degree (including alpha).
- Blur and rank filters compute in premultiplied alpha, so the color of transparent pixels does not bleed into neighboring pixels; the result is converted back to straight alpha.
- Outside the layer bounds, the nearest edge pixel is extended (clamp).
- The background layer and layers with locked transparent pixels keep their original alpha.

## Known limitations

- Everything is computed single-threaded on the CPU, which is slow for large images and large radii.
- Wind's random distribution (streak start probability, length, decay) is estimated from probe images and only statistically similar to Photoshop; Stagger's behavior was observed the least.
- Add Noise's strength and the Pixelate filters other than Mosaic have not yet been checked against Photoshop.

## Test coverage

- `blurs_spread_the_dark_pixel`: a black dot in the middle of a 5×1 white strip; Box Blur radius 1 gives `[255, 170, 170, 170, 255]`; Gaussian Blur is symmetric and gets lighter from the center outward; Average gives the average value 204.
- `rank_filters`: Median removes an isolated black dot, Minimum enlarges a black dot, Maximum removes a black dot.
- `mezzotint_leaves_only_full_or_empty_channels` (and brighter columns get more full red), `ripple_shifts_by_its_amount`.
- `crystallize_fills_cells_with_one_color`, `pointillize_paints_dots_on_the_background`, `diffuse_moves_pixels_between_neighbors` (Darken Only never lightens, Lighten Only never darkens).
- `clouds_span_the_colors_without_repeating`: the pattern spans 0–1, doesn't repeat 256 pixels across, is the same for a seed and different for another, and changes softly between neighbors.
- `sharpen_high_pass_and_solarize`: Unsharp Mask keeps the black and white of an edge; High Pass is below 128 at the center and above 128 at the edge; Solarize turns white black.
- `offset_wraps_or_fills_with_the_background`: wrapping around and filling with the background color.
- `mosaic_and_noise`: Mosaic averages a cell; repeated Add Noise gives the same result, and monochromatic noise stays gray.
- `transparent_pixels_do_not_bleed_color`: a red dot on a transparent layer stays red after blurring, with alpha 85.
- `photoshop::filters_match_photoshop`: 36 filter cases compared against Photoshop output (including Fragment, two Custom cases, two Surface Blur cases, two Dust & Scratches cases, Despeckle, Sharpen Edges, Trace Contour Lower) (`fixtures/filter`), each with a maximum error bound.
- `distortions_match_photoshop`: Twirl, Pinch, Spherize (including negative values and Vertical only), and Polar Coordinates in both directions compared against Photoshop, asserting on the average difference.
- `trace_contour_upper_matches_photoshop`: Trace Contour Level 128 Upper compared against Photoshop, with at most one channel value different.
- `wind_streaks_downwind`: Wind and Blast in both directions: results are repeatable, bright points are kept, streaks appear only on the downwind side and only lighten, and some of the 32 rows start a streak; on a flat image all three methods change nothing.
