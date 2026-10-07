# color_match.rs: Replace Color, Match Color, Color Lookup

## Responsibility

The color math of Image › Adjustments › Replace Color, Match Color and Color Lookup, used by `Adjustment::ReplaceColor`, `MatchColor` and `ColorLookup` (`adjust.md`). Their dialogs are not built yet (the menu items are placeholders).

## Lab

`to_lab` / `from_lab`: sRGB 0–255 to CIE Lab with a D50 white (Photoshop's connection space, the same sRGB→XYZ matrix as `adjust.rs`), and back (clamped). They round-trip exactly on 8-bit colors.

## Match Color

- `lab_stats(pixels)`: mean and standard deviation of L, a, b (`[mean L, a, b, sd L, a, b]`; a neutral default for no pixels).
- `match_color(rgb, target, source, luminance, intensity, fade)`: Reinhard's transfer — each Lab channel moved from the target's mean and deviation to the source's — then Luminance (%) scales L, Color Intensity (%) scales a and b, and Fade (%) mixes the original back.

## Replace Color

`replace_weight(rgb, sample, fuzziness)`: 1 within half the Fuzziness (Lab distance), falling linearly to 0 at the full Fuzziness. The adjustment mixes the Hue/Saturation master shift of the pixel in by that weight.

## Color Lookup

- `Lut { size, data }`: a 3D table, red varying fastest, values 0–1; `apply` interpolates trilinearly.
- `parse_cube`: `.cube` files (`LUT_3D_SIZE`, optional `DOMAIN_MIN` / `DOMAIN_MAX`, comments and titles skipped); refuses a wrong count. `parse_3dl`: `.3dl` files (a line of input levels, then rows with blue varying fastest; 10-, 12- or 16-bit by their largest value).
- Tables are kept in a registry (`register` returns a number, the same one for an identical table; `lut(id)`), so `Adjustment::ColorLookup(id)` stays a small copyable value.

## Test coverage

`lab_round_trips`, `match_color_moves_toward_the_source` (a bluish image matched to reddish stats turns warmer; Fade 100 keeps it), `replace_weight_falls_off_with_fuzziness`, `cube_and_3dl_luts` (identity and inverting cubes, a 10-bit 3dl identity, a short cube refused, registering twice gives one number).
