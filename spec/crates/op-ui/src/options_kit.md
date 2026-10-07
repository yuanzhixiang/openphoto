# options_kit.rs: Options bar controls (positioned from Photoshop measurements)

## Component responsibilities

The various Photoshop 2026 controls in the options bar, absolutely positioned in "bar coordinates": `x` is pt from the left edge of the options bar, `y` is pt from 0.5 pt above the top of the options bar (matching how the screenshots were cropped when measuring). Each tool's options bar is therefore a sequence of measured coordinates (see "Tools with measured layouts" in `options_bar.md`).

## Controls and measured sizes

- `sep(x)`: a 1 pt separator, `#3e3e3e`, y 6.5–29.
- `label(x, text, enabled)`: panel-font text, `#dddddd`, `#878787` when disabled, vertical center 17.25; Source Sans 3 starts its strokes further right than Adobe Clean, so it is shifted 0.75 pt to the left.
- `check(x, label, value, enabled)`: a Photoshop-style checkbox (`widgets::checkbox`); the box starts at `x` and the label is 8 pt to the right of the box. When disabled, egui's fading of disabled controls is turned off (`disabled_alpha = 1`) and the checkbox draws Photoshop's disabled colors itself.
- `field` / `value(x0, x1, …)`: an input field, y 9–25.5, `#454545` background, 1 pt `#666666` border (disabled `#4d4d4d` / `#5e5e5e`), text indented 4.5 pt from the left; `value` returns the entered text when input ends (focus lost or Enter), and Enter is no longer passed on to the canvas.
- `popup` / `choice(x0, x1, …)`: a dropdown, y 8–27 (`widgets::dropdown_with`).
- `button(x0, x1, text, enabled)`: a button, y 5.5–29.5, `#454545` background, `#666666` border, text centered; when disabled, `#4d4d4d` background, `#5e5e5e` border, `#878787` text. `menu_button` is a bit taller (4.5–30.5) and draws a small triangle in the bottom-right corner; `chevron_button` (y 5–30) contains a 12.5 pt wide, 1 pt stroke dropdown arrow.
- `icon(cx, icon, tip, on, enabled)`: an icon button; the pressed state (`on`) is a `#383838` box at y 4–31 (0.5 pt `#636363` border, 3 pt corner radius).
- `modes(x, mode)`: the four selection-operation buttons, each 26 pt wide, with the selected one shown as a pressed box; the icons are drawn as measured: New is a 10 pt solid square, Add is two overlapping solid squares, Subtract is the first square minus the second (the latter drawn only as an outline), Intersect is two outlines with the overlap solid.
- `brush_picker(cx, label, size, hardness)`: the brush picker: the brush tip preview is drawn the way Photoshop draws it (diameter of brush-size device pixels, at most 28 px, soft brushes fading outward; center y 11), a 10.5 pt size number below it (y 26.25), and a dropdown arrow 21 pt to the right.
- `slider_box(x0, x1, id, current)`: the arrow box sharing a border with a preceding percentage input field; clicking pops up a 0–100% slider and returns the new value after dragging.
- `swatch(x0, x1, fill, chevron_box)`: a color swatch (y 5–30). The Mixer Brush's load color is a borderless white block with the arrow right after it; the Pattern Stamp's pattern block has a 1 pt border and draws the leaf dots of the default pattern, with the arrow in a separate box.

- `segmented(edges, labels, chosen, enabled)`: a segmented button (y 5–30, `#454545` background, `#666666` border, the selected segment `#383838`); returns the clicked segment.
- `color_box` (borderless color block), `empty_pattern` (disabled empty pattern box and arrow box), `gradient_swatch` (the gradient over a checkerboard and its arrow box; returns the swatch's and the arrow box's click responses).
- `field_off(x0, x1)`: a disabled empty input field, y 8–26 (Photoshop's disabled input fields are a bit taller than enabled ones).
- `combo(x0, x1, x2, id, shown, options, enabled)`: an input field (y 8–26) and an arrow box to its right sharing its border (up to `x2`); clicking the arrow box pops up `options`; returns the entered text or the selected item. Used for font, font style, font size and stroke width. When disabled, both parts are drawn in disabled colors and do not respond.
- `framed_swatch(x0, x1, fill)`: a shape's Fill / Stroke swatch: outer frame y 7.5–27.5 (`#666666` 1 pt border), color inset 3 pt; `None` is "no color" (white background with a red diagonal line).
- `line_popup(x0, x1, id, options, value)`: a dropdown that shows line types (solid, dashed, dotted).
- `framed_color(x0, x1, y, fill)`: a color block with a 1 pt dark `#363636` border (text color y 8.5–26.5, artboard background y 9–26).
- `centered_label(cx, text)`: text centered on `cx`; `width()` returns the options bar width (pt), used for centering hints.

## Test coverage

- `options_bars_match_photoshops_layout` in `ui_tests.rs`: for tools with measured layouts, the positions of the options bar's separators and the edges of its various boxes (identified by color in a screenshot) correspond one by one to the values measured in Photoshop (`PS_BAR_MARKS`), within 1 pt.
- `screenshot_options_bars` (`#[ignore]`): captures each tool's options bar as `target/ui-shots/bars/<Tool>.png` for side-by-side comparison with Photoshop's screenshots.
