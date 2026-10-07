# blend.rs: Blend Mode Algorithms

## Responsibilities

Implements the pixel math for all 27 of Photoshop's layer blend modes, used by `Document::composite_rgba8`. The formulas follow the W3C Compositing and Blending specification, which matches Photoshop's definitions of these modes. All values are straight-alpha (non-premultiplied), gamma-encoded 0–1 floats, and blending happens in gamma space (matching Photoshop's default).

## Public Interface

- `blend(mode, cb, cs)`: the blend result B(cb, cs) of two opaque colors; `cb` is the lower layer (backdrop), `cs` is the upper layer (source).
- `composite(mode, dst, src, src_alpha, x, y)`: composites one source pixel onto a lower-layer pixel. `src_alpha` already includes pixel alpha, layer opacity, and fill. Following the W3C general formula: where the lower layer is transparent, the source color is shown directly; where both exist, the blend result is used; then source-over is applied. `x`, `y` are the pixel's position in the document, used only by Dissolve.

## Modes

| Mode | Computation |
|---|---|
| Normal | Source color |
| Dissolve | Each pixel is fully shown or fully hidden according to "pseudo-random number < alpha", with no semi-transparency; the random number is determined by the pixel position, so the result at a given position is fixed |
| Darken / Lighten | Per-channel minimum / maximum |
| Multiply / Screen | `b·s` / `b + s − b·s` |
| Color Burn | 1 when `b = 1`, 0 when `s = 0`, otherwise `1 − min(1, (1 − b) / s)` |
| Color Dodge | 0 when `b = 0`, 1 when `s = 1`, otherwise `min(1, b / (1 − s))` |
| Linear Burn / Linear Dodge (Add) | `max(0, b + s − 1)` / `min(1, b + s)` |
| Darker Color / Lighter Color | Compares the sum of the three channels and takes the darker / lighter color as a whole (not per channel) |
| Overlay | Hard Light conditioned on the lower layer, i.e. `HardLight(s, b)` |
| Soft Light | W3C formula (`b − (1 − 2s)·b·(1 − b)` when `s ≤ 0.5`, otherwise using `D(b)`) |
| Hard Light | `Multiply(b, 2s)` when `s ≤ 0.5`, otherwise `Screen(b, 2s − 1)` |
| Vivid Light | `ColorBurn(b, 2s)` when `s ≤ 0.5`, otherwise `ColorDodge(b, 2s − 1)` |
| Linear Light | `b + 2s − 1`, clipped to 0–1 |
| Pin Light | `min(b, 2s)` when `s ≤ 0.5`, otherwise `max(b, 2s − 1)` |
| Hard Mix | 1 when `b + s ≥ 1`, otherwise 0 |
| Difference / Exclusion | `|b − s|` / `b + s − 2bs` |
| Subtract / Divide | `max(0, b − s)` / `min(1, b / s)` (when `s = 0`: 0 if the lower layer is 0, otherwise 1) |
| Hue / Saturation / Color / Luminosity | W3C non-separable modes: luminosity `Lum = 0.3R + 0.59G + 0.11B`; SetLum, SetSat, and ClipColor combine the hue, saturation, and luminosity of the source and lower layer |

## Known Limitations

- Photoshop treats the effect of Fill differently from Opacity for 8 modes—Color Burn, Linear Burn, Color Dodge, Linear Dodge, Linear Light, Vivid Light, Hard Mix, Difference—whereas here both are simply multiplied together as the source alpha.
- Soft Light uses the W3C formula, which differs slightly from Photoshop's results for some values.
- Dissolve's random pattern differs from Photoshop's.

## Test Coverage

- `separable_modes`: specific values for Multiply, Screen, Difference, Darken, Lighten, Linear Dodge, Subtract (for example, 128 multiplied by 128 is 64, and screened is 192); multiplying by white and screening with black do not change the color; Overlay preserves black and white lower layers.
- `non_separable_modes`: Color keeps the lower layer's luminosity and takes the source's hue; Luminosity takes the source's luminosity; Saturation with a gray source gives an achromatic color.
- `composite_over_transparent_shows_source`: when the lower layer is transparent, the source color is shown.
- `dissolve_is_all_or_nothing`: Dissolve pixels have only two outcomes, shown or hidden, and the proportion shown is close to alpha.
