# auto.rs: automatic tonal and color correction

## Responsibilities

This module implements Photoshop's Auto Color Correction Options:

- the four Auto algorithms;
- Snap Neutral Midtones;
- the shadow, midtone and highlight target colors;
- the shadow and highlight clipping percentages.

It turns these into per-channel Levels values. Those values drive:

- Levels' Auto (`dialogs/levels.md` in `op-ui`), which shows them as the values themselves;
- Curves' Auto (`dialogs/curves.md`), which shows them as curve points;
- Image › Auto Tone, Auto Contrast and Auto Color (`Adjustment::AutoTone` / `AutoContrast` / `AutoColor` in `adjust.md`), which apply them as lookup tables.

## Public interface

### Types

- **`Algorithm`**, in the dialog's order (`ALL`):
  - `Monochromatic`: Enhance Monochromatic Contrast.
  - `PerChannel`: Enhance Per Channel Contrast.
  - `DarkLight`: Find Dark & Light Colors.
  - `BrightnessContrast`: Enhance Brightness and Contrast. This is the default, as it is for Levels and Curves in Photoshop 2026.
  - `label()` gives the radio button's text.
- **`Targets`** holds `shadows`, `midtones`, `highlights` (RGB) and `shadow_clip`, `highlight_clip` (percent). It defaults to black, 128 gray, white and 0.10% each.
- **`Options`** holds `{ algorithm, snap_neutral, targets }`. Its default is Enhance Brightness and Contrast with no snapping.
  - Image › Auto Tone uses `Options::auto_tone(targets)`: Per Channel.
  - Image › Auto Contrast uses `auto_contrast`: Monochromatic.
  - Image › Auto Color uses `auto_color`: Dark & Light with Snap Neutral Midtones.
- **`Channel`** is one channel's `{ black, gamma, white, out_black, out_white }`.
  - `identity()` gives the values that change nothing.
  - `map(v)` computes `out_black + ((v − black) / (white − black))^(1/gamma) × (out_white − out_black)`, clamping the stretch to 0–1.
  - `table()` gives 256 rounded entries.

### Functions

- **`samples(doc)`** returns the active layer's RGB pixels:
  - only those inside the selection whose alpha is nonzero;
  - every n-th in each direction, so that about 250,000 at most are kept.
- **`compute(pixels, options) -> [Channel; 3]`** returns red, green and blue. Each algorithm works like this:
  - **Per Channel:** each channel's histogram, minus `shadow_clip`% at the dark end and `highlight_clip`% at the light end, sets that channel's black and white.
  - **Monochromatic:** the same, with the three channels counted together into one range, so all channels get the same black and white.
  - **Dark & Light:**
    - Pixels are ordered by luminosity (0.299 R + 0.587 G + 0.114 B).
    - The darkest `shadow_clip`% (at least one pixel) are averaged per channel, and that average color becomes the black points.
    - The lightest `highlight_clip`% are averaged the same way, and that average becomes the white points.
  - **Brightness and Contrast:**
    - It starts with the Monochromatic stretch.
    - Then one gamma for all channels moves the stretched mean luminosity halfway to the middle gray.
    - That gamma is limited to 0.8–1.25.
  - **In every case:**
    - A channel whose white would come within 2 of its black is left unchanged.
    - The output black and white are the shadow and highlight targets' values for that channel.
  - **Snap Neutral Midtones:**
    - It looks at the pixels whose stretched color is nearly neutral (channels within 40 levels of each other) and whose stretched luminosity is between 40 and 215.
    - Their average original color is given each channel's gamma so that it maps to the midtone target.
    - Gamma is limited to 0.1–9.99.
    - If fewer than 0.1% of the pixels qualify, nothing changes.
- **`tables(doc, options)`** returns the lookup tables for `samples(doc)`.

## Known limitations

- Photoshop doesn't document how Enhance Brightness and Contrast works, and it behaves like Brightness/Contrast's Auto. The halfway-to-gray gamma is an approximation. Making it match exactly is part of P3 #26, together with Brightness/Contrast's Auto.
- Snap Neutral Midtones' thresholds and Find Dark & Light Colors' averaging haven't been compared with Photoshop level by level.
- Large images are sampled, not counted in full.

## Test coverage

- `per_channel_and_monochromatic`: per-channel ranges, and one shared range for Monochromatic.
- `dark_and_light_colors_and_targets`: the darkest and lightest pixels set each channel, and the shadow target becomes the output black.
- `snapping_neutral_midtones`: a red-cast gray ramp comes back neutral in the midtones.
- `brightness_and_contrast_lifts_a_dark_image`: a shadow-heavy image gets a gamma above 1, the same for all channels.
