# widgets.rs: Common Widgets

## Responsibilities

Small widgets shared by the panels, giving them a consistent Photoshop-style appearance.

## Widgets

- `icon_button`: a square icon button. Background `#383838` when selected or pressed, `#5e5e5e` when hovered, otherwise transparent; corner radius 4. When disabled, the icon turns the disabled color.
- `icon_button_sized`: same as above, with separately specified button size and icon font size.
- `icon_button_font`: same as above, with a specified icon font (the toolbar uses it to draw Phosphor Bold icons).
- `checkbox`: a Photoshop-style checkbox. 10 pt square, corner radius 2.5 pt; when checked, a solid `#d4d4d4` square with a `#323232`, 1.7 pt thick checkmark (a polyline traced from Photoshop 2x screenshots), and the square brightens to `#e6e6e6` on hover; when unchecked, only a 1 pt border of the same color is drawn; when disabled, per Photoshop measurements it is drawn as a `#4d4d4d` square with a 1 pt `#5e5e5e` border (the square is `#5e5e5e` when checked), with text `#878787`. The text is 8 pt to the right of the square, color `#f0f0f0`. Clicking either the square or the text toggles it.
- `dropdown`: a Photoshop options bar dropdown. 18.5 pt high, `#454545` background, 1 pt `#666666` border (`#808080` on hover), corner radius 2.5 pt; the value is 7 pt from the left, and a traced chevron is 7.75 pt from the right. Clicking pops up a menu filled by `menu`.
- `dropdown_with`: same as above, but whether it is enabled is decided by a parameter; when disabled it is drawn in Photoshop's disabled colors (`#4d4d4d` background, `#5e5e5e` border, `#878787` text), without egui's fading of disabled controls, and does not pop up the menu. `dropdown` calls it with whether the current Ui is enabled.
- `ps_icon_button`: a button that draws an icon traced in `ps_icons`, with a rounded background on hover or press; when disabled, the icon uses `#989898`.
- `hseparator`: a full-width horizontal divider line.
- `checkerboard`: a transparency checkerboard, alternating white and `#cccccc`, used behind thumbnails.
- `text_box(ui, rect, text, id, enabled)`: an editable input box in the options bar, placed in the given rectangle: `#454545` background, 1 pt `#666666` border, corner radius 2 pt, text in the panel font with a 6 pt left indent; when disabled, `#4d4d4d` / `#5e5e5e` / `#878787`. The Crop tool's W, H and resolution boxes use it.
