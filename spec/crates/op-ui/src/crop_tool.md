# crop_tool.rs: Crop tool

## Component responsibilities

The Crop tool's (C) crop box on the canvas: display, adjustment, commit and reset, plus the tool's options (`CropOptions`, edited by the options bar, see `options_bar.md`). The actual crop calls `op_core::image_ops::crop_extended`, and when cropping to a size it then calls `resize` (see `image_ops.md`).

## State

- The crop box is stored in `DocState::crop` (`CropBox`): `rect` is in "box space", the coordinates obtained by rotating the image about its own center by −`angle`, in which the box is upright (when `angle` is 0 these are document pixel coordinates); `angle` is the angle the box is turned on the image (clockwise, in radians); an in-progress drag `CropDrag` holds the grabbed handle, the pointer's **screen** position at press, the box and angle at that time, and the kind of drag (`CropDragKind`: box, rotate, straighten).
- The canvas displays in box space: `rotation` passes (image center, `angle`) to the canvas shader, and the image is shown rotated beneath the upright box (as in Photoshop's non-Classic mode); the rest of the crop code works in box space.
- When the current tool is the Crop tool and the document has no crop box yet, the box automatically covers the whole canvas (as when the Crop tool is selected in Photoshop). Switching to another tool crops to the box first when it was changed (`modified`), without asking (Photoshop 2026 behaves so; checked: a changed box was cropped and recorded "Crop" when the Move tool was picked), then drops it. While a dialog is open (`AppState::modal_open`) the tool takes no input, so Enter and Escape reach the dialog.
- `AppState::crop_options` (`CropOptions`), with defaults matching Photoshop 2026: preset W x H x Resolution (three empty fields, i.e. free crop), resolution unit px/in, Delete Cropped Pixels checked, Fill set to Background (default), Rule of Thirds overlay with Always Show Overlay, Classic Mode off, Show Cropped Area and Auto Center Preview on, Crop Shield on, color Match Canvas, opacity 75%, Auto Adjust Opacity on; no saved preset chosen (`user`).

## Presets and ratios (`CropPreset`, `PRESET_GROUPS`)

The ratio menu (a native menu, `native_popup.md`) follows Photoshop 2026's grouping: Ratio, W x H x Resolution | Original Ratio | 1 : 1 (Square), 4 : 5 (8 : 10), 5 : 7, 2 : 3 (4 : 6), 16 : 9 | Front Image, 4 x 5 in 300 ppi, 8.5 x 11 in 300 ppi, 1024 x 768 px 92 ppi, 1280 x 800 px 113 ppi, 1366 x 768 px 135 ppi | the saved presets (only when there are any) | New Crop Preset..., Delete Crop Preset... (grayed out without saved presets). The current choice is checked; a saved preset's name is shown in the button (cut to 12 characters and "..." when longer than 15).

- Front Image (`front_image`): fills W and H with the front (active) document's size in pixels and the resolution with its resolution, so that size can be applied when cropping another image.
- Saved presets (`UserCropPreset`, `CropPresets`): New Crop Preset... opens the naming dialog (`dialogs/new_preset.md`, title "New Crop Preset") with a suggested name (`preset_name`: "Unconstrained" for empty fields, "16 : 9" for a ratio, "W x H R ppi" for a size); OK saves the fields (kind, W, H, resolution, unit) at the end of the list, which becomes the chosen preset. Picking a saved preset puts its fields back (`choose_user`) and refits the box; editing the fields, Swap or Clear drops the choice. Delete Crop Preset... opens the Delete sheet (`dialogs/size_presets.md`, title "Delete Crop Preset", centered); after the confirming alert the preset is removed. The list lives in `~/Library/Application Support/OpenPhoto/crop-presets.txt` (one tab-separated line per preset), read at startup (`lib.rs`; tests start with none).

- `choose`: ratio presets fill numbers into W and H (e.g. 16, 9); size presets fill "4 in", "5 in" and resolution 300; Ratio, W x H x Resolution and Original Ratio clear the three fields.
- `aspect`: the aspect ratio the box must keep: Original Ratio is the document's ratio; Ratio types are W ÷ H; size types use the ratio after unit conversion (the ratio can be computed even without a resolution). When all are empty it is a free crop.
- `output_size`: for size types (W x H x Resolution, size presets), the pixel size to resample to after cropping. W and H may carry the units px/in/cm/mm (no unit means pixels); inches and the like need a resolution to convert.
- `swap` (the swap button) swaps W and H; `clear` (Clear) clears the three fields (the preset goes back to Ratio or W x H x Resolution).
- After choosing a preset, swapping or changing a number in the fields, if there is a ratio, the box is reset to the largest centered rectangle of that ratio on the canvas (`fitted`).

## Interaction

- Handles: the four corners and the midpoints of the four edges, grabbable within 10 pt on screen. Dragging changes the corresponding edges; with a ratio, corner handles keep the ratio (following the direction that changed more, with the opposite corner fixed), and edge handles keep the ratio by using the center line as the axis for the other direction; without a ratio, holding Shift while dragging a corner keeps the ratio at the start of the drag; holding ⌥ scales from the center.
- **Non-Classic Mode (default, Auto Center Preview on)**: the box always stays in the center of the view and the image moves beneath it (measured in Photoshop 2026). When dragging a handle, the opposite edge stays fixed on the image, so the box edge moves relative to the image by twice the pointer's movement; dragging inside the box pans the image (the box moves the opposite way relative to the image); the view continually puts the box's center back at the window center (`center_on_box`). It is also recentered after reset, cancel and commit.
- **Classic Mode**: dragging inside the box moves the box, handles follow the pointer, and the view does not move. Dragging outside turns the box itself on the still image (clockwise with the pointer, Shift in 15° steps), about its center, which stays on the same image point; the canvas is not rotated (`rotation` returns none), and the box, its handles, overlay and shield are drawn turned (`tilt`, `box_to_screen`, `screen_to_box`, `draw_turned`). Handle drags are turned into the box's own axes. Committing is the same as for a turned image box (the image is rotated so the box is upright).
- Dragging outside the box (not Classic Mode): rotates the image about the box's center (Photoshop's behavior); dragging clockwise turns the image clockwise and decreases the box's angle on the image; holding Shift steps by 15°; while rotating, the box's center stays at the same point on the image (`set_angle`). The cursor is Alias. If the box's width or height is less than 1 pixel when the drag ends, the box is restored to its pre-drag state.
- Straighten: the options bar's Straighten button (icon or text) turns on straighten mode (the button gets a background). The next drag draws a line; on release the image is rotated so the line becomes horizontal (vertical when it is closer to vertical), the box becomes the largest centered box of the current ratio (the canvas ratio when there is no ratio) that fits in the rotated image (`fitted_turned`), and straighten mode exits; Esc also exits it.
- The box can extend beyond the canvas: on commit the canvas is enlarged there (when Fill is Background, new areas of the background layer are filled with the background color; other layers are transparent).
- Commit: Enter, double-click inside the box, or the options bar's ✓: if the box has an angle, the image is first rotated by −`angle` with `rotate_arbitrary` (new corners filled with the background color, canvas enlarged, box translated with the canvas center); the box is then rounded to whole pixels and `crop_extended` is applied (Delete Cropped Pixels decides whether pixels outside the canvas are deleted; when not deleted, the background layer becomes "Layer 0" to keep them); if there is an `output_size`, the image is then resampled to that size with the Automatic method and the resolution is set; "Crop" is recorded. Afterwards the box covers the new canvas.
- Cancel: Esc, or the options bar's ⦸: the box is restored to the whole canvas.
- While an input field has keyboard focus, Enter and Esc do not act on the crop box.
- Cursors: on a handle, the double-headed arrow for that direction; inside the box, the move cursor; outside the box, a crosshair.

## Appearance (measured in Photoshop 2026, pixel by pixel from 2x screenshots)

- Shield layer (Crop Shield): the image outside the box is mixed with the shield color by opacity in **linear light** (this is what Photoshop does: 75% canvas gray `#282828` over white gives 141, over 128 gray gives 75, over black gives 34; mixing in gamma space would give 94). This is done by the canvas shader (`shield` is passed to `op-render`'s `Shield`, see its `canvas.md`). The Match Canvas color is the background outside the canvas, `color::PASTEBOARD`. When Show Cropped Area is off, the outside of the box is fully covered; when Crop Shield is off (and the cropped area is shown), there is no shield. The gear panel's Color is a native menu (Match Canvas, Custom, as in Photoshop) with a swatch; Custom or a click on the swatch opens the Color Picker (`PickerTarget::CropShield`), whose OK sets the color. Auto Adjust Opacity: while the box is being dragged the shield is drawn at half its opacity (`AUTO_OPACITY`). In Classic Mode the shield is a turned box on the image (`op_render::Shield::angle`).
- Box lines: a 1 pt `#323232` dark line hugging the outside of the box, and outside that a 1 pt `#f4f4f4` light line.
- Handles: `#f4f4f4`, 4 pt thick, drawn outside the box: the four corners are L shapes with arms 23 pt long, the edge midpoints are bars 47 pt long, with 2.5 pt rounded corners.
- Overlay (`Overlay`, inside the box, 1 pt `#f4f4f4` lines plus a very faint black shadow, visible only on light images): Rule of Thirds (thirds), Grid (squares of about 40 pt), Diagonal (a 45° line from each of the four corners), Triangle (one diagonal and perpendiculars to it from the other two corners), Golden Ratio (lines at 0.382 / 0.618), Golden Spiral (a quarter-circle-arc spiral in golden rectangles). Display: Always shows all the time, Auto shows only while dragging, Never does not show.

## Crop Preview

While the box is being dragged or differs from the whole canvas (`previewing`), Photoshop 2026 shows a temporary layer: the tab title reads "name @ zoom (Crop Preview, RGB/8...)" (`doc_tabs.md`) and the Layers panel lists only a selected "Crop Preview" row with the merged image as its thumbnail (`panels/layers.md`). Committing or cancelling brings the layers back.

## Snapping

While the box isn't turned, View › Snap pulls a moved box's edges or middle, and the edges a handle moves, to the snap targets (`snap_box`, `snap.md`; Control held: no snapping).

## Known limitations

- During rotation or straightening, other canvas overlays such as the selection and guides are still drawn for the unrotated image.
- Fill's Generative Expand (Adobe cloud, P3 #27) and Content-Aware Fill (P3 #26) are listed grayed out.
- Auto Adjust Opacity's half strength while dragging is an estimate; Photoshop's exact amount has not been measured.
- While cropping, Photoshop's blend mode, opacity and lock rows apply to the Crop Preview layer (it has position lock on); here they still show the active layer.
- The overlay menu's "Cycle Orientation" is grayed out.

## Test coverage

- `handles_resize_and_inside_moves`: corner scaling, Shift keeping 2:1, ⌥ scaling from the center, dragging inside moves, dragging the right edge under a 1:1 ratio keeps a square.
- `presets_ratios_and_sizes`: numbers filled in by presets, 16:9 and swap, Original Ratio, 4 × 5 in 300 ppi is 1200 × 1500 pixels, pixel sizes need no resolution, Clear.
- `ui_tests::crop_tool_crops_to_the_box`: in the default mode, dragging the bottom-right corner in to (234, 311) gives a 266 × 189 box and the box stays centered; the same drag in Classic Mode gives 500 × 500; Enter crops and records "Crop"; changing tools discards the box.
- `ui_tests::crop_shield_presets_and_growing_the_canvas`: 1:1 gives a centered 300 × 300; a white image renders as 141 outside the box and 255 inside; when the box extends past the right edge the canvas is enlarged and the new area is the background color.
- `ui_tests::screenshot_crop_tool` (ignored): for comparison against Photoshop screenshots.
- `ui_tests::crop_tool_crops_to_the_box` (continued): the title shows "(Crop Preview, RGB/8" while the box is changed and not after the crop; picking the Marquee with an unchanged box only drops it, with a changed box it crops first (one more history state).
- `ui_tests::classic_mode_turns_the_box`: in Classic Mode a quarter turn with Shift gives 90°, the image is not turned, the box's center stays on the same image point, and committing a 300 × 200 box turned 90° gives a 200 × 300 image.
- `ui_tests::crop_presets_front_image_and_shield_color`: Front Image fills 734 px, 811 px, 72; with 16 : 9, New Crop Preset... suggests "16 : 9" and saves "Wide"; picking "Wide" after 1 : 1 restores 16 : 9 and refits the box; Delete Crop Preset... with Enter twice removes it; a custom shield color reaches the shield, and a drag halves its opacity.
- `ui_tests::crop_rotation_and_straighten`: holding Shift and turning a quarter turn clockwise outside the box gives an angle of −90°; Esc resets; a 10° straighten line makes the angle 10°, shrinks the box and exits straighten mode; after commit the canvas is about the size of the box and none of the four corners is the background color (the box is inside the image).
