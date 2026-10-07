# dialogs/adjust_presets.rs: Photoshop's adjustment presets

## Responsibility

The presets Photoshop 2026 ships for its adjustment dialogs, as the numbers its preset files hold (read from `Presets/Levels/*.alv`, `Curves/*.acv`, `Hue and Saturation/*.ahu`, `Black and White/*.blw`, `Channel Mixer/*.cha`, `Exposure/*.eap` in Photoshop's application folder and written out as constants, sorted by name as Photoshop's menus list them). Only numbers are kept; no Adobe file is bundled.

## Tables

- `LEVELS`: per channel (RGB, Red, Green, Blue) input black, gamma × 100, input white, output black, output white (the `.alv` records after the version word).
- `CURVES`: per channel the (input, output) points (`.acv` stores output first).
- `HUE_SATURATION`: Master hue, saturation, lightness, and the Colorize values for the colorizing ones (Cyanotype, Sepia; the hue stored as −145 is 215°). Every preset leaves the six ranges at their defaults.
- `BLACK_WHITE`: the six weights (Reds, Yellows, Greens, Cyans, Blues, Magentas) from the `.blw` descriptor; no preset uses a tint.
- `CHANNEL_MIXER`: all six are monochrome; the gray output's Red, Green, Blue.
- `EXPOSURE`: exposure, offset, gamma (three floats after the version word).

## Used by

`levels.md`, `curves.md`, `hue_saturation.md`, `black_white.md`, `channel_mixer.md`, `exposure.md` (their Preset menus and the preset name shown).

## Tests

Each dialog's `photoshops_presets` test picks a preset, checks the values and that the Preset shows its name, and that a change makes it Custom.
