# dialogs/channel_mixer.rs: Channel Mixer dialog

## Component responsibilities

The dialog for Image › Adjustments › Channel Mixer..., rebuilt after Photoshop 2026's UXP dialog (401 × 426 pt). For the algorithm see `op-core`'s `adjust.md` (`ChannelMixer`).

## Data and defaults

- Each of the three output channels red, green and blue has Red, Green, Blue (−200–200%) and Constant (−200–200%), defaulting to identity (itself 100, the others 0).
- Monochrome: when checked, a single grayscale mix is edited instead, with Photoshop's initial values 40 / 40 / 20 / 0.
- `adjustment()`: returns `Adjustment::ChannelMixer` when all values are valid; Preset is "Default" while unchanged, a Photoshop preset's name while the values match it, and "Custom" otherwise. The menu lists Default, Custom and Photoshop's six monochrome presets (`adjust_presets::CHANNEL_MIXER`, Black & White Infrared … with Yellow Filter); picking one turns on Monochrome with the gray's Red, Green, Blue (constant 0).

## Layout (Photoshop points)

- "Preset" (20, 61), dropdown (56, 48.5)–(231, 73.5), preset menu icon (248, 61) (drawn only).
- "Output channel" (20, 97), three dots centered at (121 / 150 / 178.75, 96), drawn the same way as in Color Balance; in Monochrome only one selected gray dot remains.
- "Monochrome" checkbox (20, 127); "Preview (Opt+P)" (272, 127).
- Four slider rows: Red, Green, Blue, Constant, input fields at x 203–260, y 156 / 211 / 266 / 363, tracks 36 pt below the fields, x 20–260. Track colors are taken from Photoshop: black → channel color → white (Constant is black → gray → white).
- "Total" (20, 330) with the sum of the three items (right-aligned to 238.5) and "%" (250); below it at y 350 a `#737373` separator line.
- OK and Cancel on the right.

## Interaction

- Clicking a dot switches the output channel; when opened, the Red input field gets focus with its contents selected. Everything else is the same as other UXP dialogs (`uxp.md`).

## Known limitations

- Photoshop's presets (Black & White with Red Filter, etc.) and the warning icon shown when Total exceeds 100% are not present.

## Test coverage

- `rows_and_monochrome`; `ui_tests::channel_mixer_and_selective_color_apply`; screenshot `channel_mixer_dialog.png`.
