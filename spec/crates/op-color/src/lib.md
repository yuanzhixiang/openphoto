# op-color: Color Model Conversion

## Responsibilities

Converts between `op_core::Color` (sRGB-encoded RGBA floating-point color) and the other representations used by the color picker: HSB, Lab, CMYK, 6-digit hexadecimal, and web-safe colors. All conversions are pure math, with no ICC color management.

## Public Interface

### `Hsb`

HSB as in Photoshop's color picker: `h` is the hue, in 0..360 degrees; `s` (saturation) and `b` (brightness) are in 0..=1. The fields are public.

- `Hsb::from_color(c)`: standard RGB→HSV conversion. `b` is the maximum of the three channels; `s = (max − min) / max`, with `s = 0` when `max` is 0; for achromatic colors (`max == min`), `h = 0`. Ignores `c.a`.
- `Hsb::to_color()`: standard HSV→RGB conversion; output alpha is always 1. The hue is first taken as the Euclidean remainder modulo 360, so negative values and values greater than or equal to 360 are folded back into 0..360.

### `Lab`

CIELAB with a D50 reference white, matching the values shown by Photoshop's color picker (for example, #00afdc is L66, a−27, b−34, identical after rounding). `l` is 0–100; `a` and `b` are roughly −128–127.

- `Lab::from_color(c)`: sRGB → linear RGB → XYZ (D65) → Bradford chromatic adaptation to D50 → Lab. The two matrix steps are combined into one constant matrix. Ignores alpha.
- `Lab::to_color()`: the reverse conversion. Results outside the sRGB gamut are clipped to 0–1 in linear space, so the returned color is always within sRGB. Alpha is always 1.

### `Cmyk`

Four components C, M, Y, K in 0–1.

- `Cmyk::from_color(c)`: device-independent formula: K = 1 − max(R,G,B), C = (1 − R − K) / (1 − K), likewise for M and Y; pure black is C=M=Y=0, K=1.
- `Cmyk::to_color()`: R = (1 − C)(1 − K), likewise for G and B.
- This is not Photoshop's CMYK: Photoshop converts through the ICC profile of the CMYK working space (default U.S. Web Coated SWOP), and the values differ (#00afdc is C72 M10 Y6 K0 in Photoshop, C100 M20 Y0 K14 here).

### Web-Safe Colors

- `web_safe(c)`: each channel is first quantized to 8 bits, then snapped to the nearest multiple of 0x33 (0, 51, 102, 153, 204, 255); alpha is kept.
- `is_web_safe(c)`: all three channels are multiples of 51 after 8-bit quantization.

### Hexadecimal

- `to_hex(c)`: returns a 6-digit lowercase hexadecimal string without `#`, for example `"1fa3e0"`. Quantized with rounding via `Color::to_rgba8` first; alpha is discarded.
- `from_hex(s)`: parses 6-digit hexadecimal. Leading and trailing whitespace and a leading `#` (optional) are stripped first; the remainder must be exactly 6 bytes, otherwise `None` is returned. Either case is accepted. The resulting alpha is always 1.

## Edge Cases

- `from_color` always gives `h = 0` when saturation is 0, so a gray loses its original hue after an HSB round trip; if the color picker needs to keep the hue on gray, it must store the `Hsb` value itself (`op-ui` stores `picker_hsb` separately for this).
- `to_color` does not clamp `s` and `b`; inputs outside 0..=1 produce out-of-range components, which are only clipped at `to_rgba8`.
- `from_hex` accepts only the 6-digit form, not the 3-digit shorthand (such as `fff`) or the 8-digit form with alpha.
- `from_hex` parses with `u32::from_str_radix`, so a 6-byte string starting with `+` (for example `"+12345"`) is also accepted.
- Multiple leading `#` characters are all stripped.

## Relationship to Other Modules

- Depends on `op-core` (`Color`).
- `op-ui`'s Color panel uses `Hsb`; the Color Picker uses all the representations here (see `crates/op-ui/src/dialogs/color_picker.md`).

## Known Limitations

- No ICC color management; all colors are processed directly as sRGB-encoded values, so CMYK differs from Photoshop.
- No conversions for grayscale or other color modes.

## Test Coverage

- `hsb_round_trip`: pure red, an arbitrary intermediate color, 50% gray, and black match the original colors after an HSB round trip and 8-bit quantization.
- `hex`: `"#1fa3e0"` is parsed and output again as `"1fa3e0"`.
- `lab_matches_photoshop`: the Lab of #00afdc rounds to (66, −27, −34), the same as Photoshop; white is (100, 0, 0).
- `lab_round_trip`: several colors keep their hex value after a Lab round trip.
- `cmyk_round_trip`: unchanged after a CMYK round trip; black's K is 1.
- `web_safe_snapping`: #00afdc is not a web-safe color and snaps to #0099cc.
