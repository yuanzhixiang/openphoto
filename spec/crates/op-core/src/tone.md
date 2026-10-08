# tone.rs: Shadows/Highlights and HDR Toning

## Responsibility

The pixel work of Image › Adjustments › Shadows/Highlights and HDR Toning. Both look at a pixel's surroundings (a blurred luminosity), so they run as filters (`Filter::ShadowsHighlights`, `Filter::HdrToning` in `filter.md`) on the active layer. Their dialogs are in `op-ui` (`dialogs/adjust.md`).

## Shadows/Highlights (`ShadowsHighlights`, `shadows_highlights`)

- Settings as Photoshop's (Show More Options): Shadows and Highlights each Amount (0–100 %), Tone (0–100 %), Radius (px); Color (−100–100), Midtone (−100–100), Black and White Clip (%). `Default` is Photoshop 2026's: Shadows 35 / 50 / 30, Highlights 0 / 50 / 30, Color 20, Midtone 0, clips 0.01 %.
- The Rec. 601 luminosity is blurred (Gaussian, σ = radius / 2, as three box blurs with repeated edges: `blur`) per side. A pixel's shadow weight is `(1 − blurred / tone)²` and its highlight weight `((blurred − (1 − tone)) / tone)²` (both clamped to 0–1); the luminosity rises by `amount × weight × (1 − L) × 1.25` and falls by `amount × weight × L × 1.25`. Midtone then scales the distance from 0.5, most at the middle tones. Black and White Clip send the result's darkest and brightest clip percent to black and white (the rest is left as it is).
- Each pixel's color is scaled to its new luminosity (`relight`: what rises past white spills toward white), with its saturation changed by `1 + Color × |change| × 2`.

## HDR Toning (`HdrToning`, `hdr_toning`)

- Methods (`HdrMethod`): Exposure and Gamma, Highlight Compression, Equalize Histogram, Local Adaptation (Photoshop's default). Settings: Edge Glow Radius (px) and Strength, Gamma, Exposure, Detail (%), and Shadow, Highlight, Vibrance, Saturation (−100–100). `Default` is Photoshop 2026's: Local Adaptation, 15 px, 0.52, gamma 1, exposure 0, detail 30 %, saturation 20.
- Local Adaptation: the luminosity's blurred base is pulled toward the middle by `1 / (1 + strength)`, the detail (luminosity − base) is added back times `1 + detail`, then exposure (× 2^exposure), gamma (^(1 / gamma)), Shadow and Highlight (cubic lifts of the ends), and Saturation and Vibrance (more for muted colors). Exposure and Gamma: those two only. Highlight Compression: `L × 3 / (1 + 2L)`. Equalize Histogram: the luminosity's cumulative histogram.
- Smooth Edges: across a strong step the blurred base keeps to the pixel's own level (`base = l + d·e^(−d²/0.02)`, d = base − l), so the glow doesn't spill over edges.
- The Toning Curve (`curve`, `curve_len`, `corners`): `curve_table` runs a smooth curve through the points, a separate one between corner points, level outside the end points; in Local Adaptation the toned luminosity goes through it (interpolated between table entries).
- Photoshop flattens the document for HDR Toning; here the active layer is toned.

## Known limitations

These follow Photoshop's controls and pictures; Adobe's exact curves are its own, so results differ.

## Test coverage

- `toning_curve_table_and_corners`: the default curve is straight, an inverting one inverts, a corner bends sharply with straight runs either side, and toning with the inverting curve darkens a light gray.

- `shadows_lift_and_highlights_fall`: on a dark/bright split the defaults lighten the dark half and leave the bright one; Highlights 50 % darkens the bright half and leaves the dark one.
- `hdr_toning_compresses_and_keeps_detail`: the defaults bring the halves toward each other; Exposure and Gamma at +1 doubles a dark gray.
- `blur_keeps_a_flat_field`.
