# dialogs/uxp.rs: Controls for UXP Adjustment Dialogs

## Component Responsibilities

Controls shared by Photoshop 2026's UXP adjustment dialogs (Brightness/Contrast, Color Balance, Hue/Saturation), with sizes measured from Photoshop's 2x screenshots (in Photoshop points).

## Controls

- `label(ui, left_center, text)`: a 12 pt Adobe Clean (approximated here with Source Sans 3) `#f0f0f0` label; `left_center` is the midline of the capital letters, and drawing shifts it down 0.75 pt (Source Sans sits higher than Adobe Clean).
- `number_box(ui, rect, text, id, range, focus)`: a 24 pt high number box (`common::text_field`, 11 pt left margin for the digits, a blue outer ring when focused). When focused, ↑/↓ add or subtract 1, or 10 with Shift held, limited to `range`. When `focus` is true, it takes focus and selects all.
- `slider(ui, id, x0, x1, y, value, (min, max), colors)`: a Spectrum slider. A 2 pt thick track, colored by `colors(t)` (t is the position from 0–1), broken for 4.5 pt on each side of the slider ring; the thumb is a 13 pt hollow ring (1 pt, `#d0d0d0`, `#f0f0f0` while dragging).
  - Values: the travel of the thumb's center is the track inset by 6.5 pt (the ring radius) at each end. When `min` is less than 0, 0 is at the midpoint of the travel, and positive values (by `max`) and negative values (by `min`) each take half (so Contrast's −50–100 is also centered on 0, matching Photoshop); when `min` is not less than 0 (Colorize's hue and saturation), values run linearly from the left end.
  - Pressing or dragging on the track makes the thumb jump to the pointer, returning the new rounded value.
- `slider_stepped`: a slider centered on 0 that takes values by step (Exposure's 0.01, 0.0001). `linear_slider`: values run linearly from `min` to `max` (Posterize, Black & White, Photo Filter, Gamma). `draw_slider`: a slider drawn without interaction (the Tint row when not enabled).
- `three(a, mid, b)`: a three-color gradient.
- `stops(colors)`: a gradient of evenly spaced color stops, blending between adjacent stops in sRGB; each dialog's track colors are color stops picked from Photoshop screenshots.
- `buttons(ui, frame, third, ok_enabled, ok_focused)`: the right-hand button column, 109 × 24 pt, 20 pt from the right edge, one every 36 pt starting at 48 pt: OK (the default button, with a white border), Cancel, and an optional third (`(label, enabled)`, e.g. Auto). With `ok_focused`, OK is drawn as holding keyboard focus (`common::ps_focused_button`). Enter is the same as OK (only when `ok_enabled`). Button text is 12 pt bold.
- `preview(ui, min, preview)`: the "Preview (Opt+P)" checkbox; ⌥P toggles it as well.
- `radio(ui, center, label, chosen)`: a Spectrum radio button; selected is a 12 pt light gray disc with a 3.5 pt hole in the middle, unselected is a `#8e8e8e` ring; the label is 15.5 pt to the right of the center. Returns whether it was clicked.
- `checkbox(ui, min, label, checked)`: a 12 pt checkbox with the label 9.5 pt to the right, aligned to Photoshop's baseline (shifted down 1 pt).

## Known Limitations

- Sliders do not support keyboard operation (Photoshop's sliders can use the arrow keys).
