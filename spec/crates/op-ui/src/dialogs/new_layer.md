# dialogs/new_layer.rs: New Layer dialog

## Component responsibility

The dialog opened by Layer › New › Layer... (⇧⌘N); it sets the new layer's name, color label, blend mode, opacity, and whether to fill with a neutral color, corresponding to Photoshop 2026's New Layer dialog. ⌥⇧⌘N skips the dialog and creates the layer directly (see `commands.md`); ⌥-clicking the "New Layer" button at the bottom of the Layers panel also opens it.

## Layout (measured from Photoshop 2026, in pt relative to the dialog's top-left corner)

The dialog is 552 × 186, centered; the title bar and border are described by `frame` in `common.md`; the title "New Layer" is the window title (13 pt bold system font). This is a UXP dialog whose text is Adobe Clean, so `theme::uxp` (Source Sans 3 Regular 12 pt) is used, color `#f1f1f1`; the width of short text differs from Photoshop by 1–3 px, and long sentences are about 2.5% wider (the inherent difference between the two fonts).

- "Name" right-aligned to x 49, center y 61; name field (59, 49)–(243, 71), with 10 left of the text; it gets focus with everything selected on open; the default value is the next "Layer N" (`Document::next_layer_name`).
- "Color" right-aligned to x 282 ("Opacity" to 281.5, "Mode" to 49.5); color dropdown (290, 48)–(450, 72): 1 pt `#7a7a7a` border, corner radius 3; on the left a 14 pt color square (a hollow square with a cross for None), the color name from x 321, and a 10 × 6 pt V-shaped arrow 13.5 from the right edge. The menu lists all eight `LayerColor` entries.
- "Use previous layer to create clipping mask" checkbox (58, 83), 12 pt square, text 9.5 to the right of the box. This app does not support clipping masks yet, so it is always grayed out and unchecked (it is enabled in Photoshop).
- "Mode" right-aligned to x 49, center y 125.5; mode dropdown (58, 113)–(218, 137), value 9 from the left; the menu lists all blend modes in Photoshop's groups, with separators between groups.
- "Opacity" right-aligned to x 280.5; value box (290, 113)–(340, 137) (11.5 left of the text; can be typed into directly, with or without %); immediately to its right a small box with a V-shaped arrow (339, 113)–(358, 137), which on click pops up a 0–100 slider.
- "Fill with ‹mode›-neutral color" checkbox (58, 148) and swatch (192, 142)–(216, 166): enabled only when the current mode has a neutral color (`op_core::neutral_color`); the text then becomes e.g. "Fill with Multiply-neutral color", and the swatch shows the neutral color and follows 12 pt after the text; otherwise the text is "Fill with neutral color" and grayed out, and the swatch is gray.
- Buttons: OK (462, 48)–(532, 72), the default button, with a 1 pt bright `#f1f1f1` border; Cancel (462, 84)–(532, 108), with a 1 pt `#727272` border. Both are pill-shaped (`common::ps_button`), with labels in `theme::uxp_bold` (Source Sans 3 Semibold 12 pt); the background changes on hover or press.

## Layer from Background variant

Layer › New › Layer from Background... and double-clicking the background layer in the Layers panel open a variant of the same dialog (`NewLayerDialog::from_background`): the title is still "New Layer", height 157 (no neutral color row), the name defaults to "Layer 0", and "Use previous layer..." is grayed out. On confirm, the background layer becomes a normal layer using the dialog's name, color label, blend mode, and opacity (`panels::layer_from_background_with`), recording "Layer From Background".

## New Group and New Group from Layers variants

Layer › New › Group... and Group from Layers... open variants with the same layout (`NewLayerDialog::group`, `Kind::Group` / `Kind::GroupFromLayers`): titled "New Group" and "New Group from Layers" respectively, height 128, without the clipping mask and neutral color rows; the Mode row moves up 29 pt (center y 96.5); the default name is the next "Group N"; the default mode is Pass Through; and the mode menu has an extra "Pass Through" item at the top (matching Photoshop 2026 screenshots). On confirm, an empty group is created or the selected layers are grouped, using the dialog's name, color label, mode, and opacity (`panels::new_group_from`), recording "New Group" and "Group Layers" respectively.

## Interaction

- Enter or OK: creates a layer with the current values (`panels::new_layer_from`): inserted above the current layer and selected, with the name (when empty, "Layer"), color label, blend mode, and opacity (clamped to 0–100%); when neutral color fill is checked and the mode has a neutral color, the whole canvas is filled with that color. Records one "New Layer" history entry.
- Esc or Cancel: closes without doing anything.
- OK is disabled when the opacity is not a number.
- Menus and shortcuts do not work while the dialog is open (`AppState::modal_open`).

## Known limitations

- "Use previous layer to create clipping mask" is unavailable (clipping masks are not implemented yet).
- In Photoshop, the opacity arrow pops up a Photoshop-style slider; here it is an egui slider.

## Test coverage

- `defaults_and_values` (unit test): default values; opacity parsing, clamping, and invalid input; neutral color fill only takes effect in modes that have a neutral color.
- `new_layer_dialog_names_colors_and_blends` (UI test): ⇧⌘N opens the dialog; typing "Shade", choosing Violet from the color menu and Multiply from the mode menu, checking neutral color fill, and clicking OK produces a layer named Shade with a violet label, Multiply, filled entirely white, and records "New Layer"; Esc cancels without creating a layer; ⌥⇧⌘N produces "Layer 1" directly without the dialog.
- `screenshot_new_layer_dialog` (screenshot, `#[ignore]`): used for pixel-by-pixel comparison with Photoshop screenshots.
