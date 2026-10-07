# dialogs/plain_filter.rs: Filter dialogs without a preview

## Responsibilities

The layouts of Photoshop 2026's small filter dialogs that have no preview: Stylize › Tiles..., Pixelate › Color Halftone..., Other › HSB/HSL ("HSB/HSL Parameters"), Blur › Radial Blur..., Distort › Shear..., Distort › Displace... and Stylize › Extrude.... They are drawn by `AdjustDialog::plain_ui` (`adjust.md`): the title bar, the items in order, and OK and Cancel at `buttons_x` with the layout's `buttons` (width, OK's and Cancel's tops, height, label size; the plug-in style ones are 89 × 26 pt at y 41 and 77 with 13 pt labels). Labels, units and radio labels are in the layout's `text_size` (11 pt in all), field values 12 pt. Measured on Photoshop 2026: Tiles' and Color Halftone's labels and fields within half a point, HSB/HSL's within about a point (Source Sans 3 runs a little wider than Photoshop's font there).

## Items (`Item`)

- `Text`: a label at (left x, center y).
- `Field`: a label (left x) and an optional unit (left x), both on the field's center line, and the number field (`appkit::field`); the first field (whatever setting it is) is focused with its value selected when the dialog opens.
- `Radios`: one button per choice at the given centers, labels `gap` pt after the center (`appkit::radio_with`).
- `Track`: a slider for the field before it: a 3 pt `#757575` track from x0 to x1 at `top`, the white pin standing on it (tip 3 pt above the track), its middle at `pins.0` for the setting's minimum and `pins.1` for its maximum, evenly between (`AdjustDialog::plain_track`). A click or drag on it sets the value.
- `Group`: a group box (`appkit::group`) with its title on the top edge from `title_x`.
- `Check`: a checkbox (`appkit::checkbox_with`) with its box's top-left corner, size and the label's gap after it; sets the next setting (on or off).
- `CenterBox`: Radial Blur's Blur Center (`AdjustDialog::center_box`). A white box showing the blur's pattern around the center: points 8 pt apart along grid lines a quarter of the box apart through the center, each drawn as a 0.5 pt black arc of the Amount's angle round the center (Spin) or a stroke away from it, Amount% of its distance long (Zoom), as Photoshop draws them. A click or drag in it puts the center there (`Extra::radial_center`, 0–1 across and down; the middle at first), which the filter uses.
- `CurveGrid`: Shear's curve grid (`AdjustDialog::curve_grid`). A white box with a 0.5 pt black border, dotted lines at its quarters (one physical pixel on, one off, drawn as a mesh so they stay crisp), and the curve: a 0.5 pt black natural spline through `Extra::shear_points` (height 0–1 down the box, offset −0.5–0.5 of its width from the middle; the top and bottom ends first, a straight line down the middle at first), with each point a 4 pt black square (the ends' squares just inside the box). Dragging a point moves it (the ends only sideways); pressing on the curve within 4 pt elsewhere adds a point there and drags it, up to eight; an inner point dragged more than 8 pt off the box is removed.
- `Pane`: a `#4d4d4d` box with a 1 pt `#3e3e3e` border showing the document as filtered at 100% (`pane_ui`, 600 × 300 pixels for Shear).
- `Text`, `Group`, `CenterBox`, `CurveGrid` and `Pane` set nothing; the other items set the next of the filter's settings (`adjust.md`'s parameter table).

## Layouts

| Dialog | Size | Buttons x | Items |
| --- | --- | --- | --- |
| Tiles (`TILES`) | 302 × 220 | 195.5 | "Number Of Tiles:" (x 17) field 117–146 × 44–65; "Maximum Offset:" field 118–147 × 81–102, "%" at 156; "Fill Empty Area With:" (23, 116.75); radios at x 21, y 134 / 154 / 174 / 194, labels 13 pt after |
| HSB/HSL (`HSB_HSL`) | 270 × 163 | 172 (78 × 24 buttons at y 48 and 80, 11 pt) | "Input mode" (20, 60.5) over radios at x 26, y 83 / 107 / 131; "Row order" (101, 60.5) over radios at x 107; labels 15 pt after the centers |
| Radial Blur (`RADIAL_BLUR`) | 274 × 280 | 148 (108.5 × 26 buttons at y 39 and 75, 13 pt) | "Amount" (x 19.5) field 83–117 × 48.5–67; track 14.5–121.5 at y 76, pins 17.75 (1) to 116.75 (100); group "Blur Method:" 12–117.5 × 113–175.5 with radios at x 26, y 133 / 156; group "Quality:" 12–117.5 × 184–266.5 with radios at y 204 / 226 / 248; labels 13.5 pt after the centers; "Blur Center" (133, 125) over the box 132–262 × 140–270 |
| Shear (`SHEAR`) | 318 × 392 | 215.5 (88.5 × 26 buttons at y 35 and 71, 13 pt) | curve grid 8–138 × 34–164; group "Undefined Areas:" 8–143.5 × 181–222.5 with radios at x 20, y 196 / 212 (Wrap Around first, as Photoshop), labels 13.5 pt after; preview box 8–310 × 234–386 |
| Displace (`DISPLACE`) | 330 × 232 | 223.5 | "Horizontal Scale" and "Vertical Scale" (x 9) fields 143–205 × 37.5–56.5 and 66.5–85.5; group "Displacement Map:" 9–205 × 101–149 (title x 23) with Stretch To Fit and Tile at (20.5, 118 / 138); group "Undefined Areas:" 9–205 × 158–206 (title x 23) with radios at (20.5, 175 / 195); labels 13.5 pt after |
| Extrude (`EXTRUDE`) | 402 × 167 | 295.5 | "Type:" (9, 46) with Blocks and Pyramids at (55, 46) and (125, 46), labels 13 pt after; "Size:" field 43–77 × 65.5–84.5 with "Pixels" at 83; "Depth:" field 53–87 × 96.5–115.5 with Random and Level-based at (108.5, 106) and (190, 106), labels 12.5 pt after; 11 pt checkboxes at (11, 129.5) and (11, 145.5), labels 7 pt after |
| Color Halftone (`COLOR_HALFTONE`) | 326 × 243 | 219.5 | "Max. Radius:" (x 9) field 87–150.5 × 36.5–56.5, "(Pixels)" at 160.5; "Screen Angles (Degrees):" (13, 76); "Channel 1:"–"Channel 4:" (x 17) fields 83.5–146.5 at y 95.5, 132.5, 169.5, 206.5 (20 tall) |

## Behavior

- These dialogs don't preview on the document (`AdjustDialog::preview` starts off for them), as in Photoshop; OK or Enter applies, Esc or Cancel closes.
- Shear is the exception: its preview box shows the document filtered, which OpenPhoto does by previewing on the document as the classic dialogs do (Photoshop leaves the document alone until OK).

## Test coverage

- `ui_tests::shear_curve_grid_bends_the_line`: dragging the top end a quarter of the box right, pressing halfway down the curve and dragging left adds a point there; dragging it off the box removes it; Enter applies "Shear". Screenshots `shear.png` and `shear_bent.png`, compared with Photoshop's capture.

- `ui_tests::screenshot_plain_filter_dialogs` (`#[ignore]`) captures Displace, Extrude, Fibers and Lens Flare for comparing with Photoshop's captures.
- `ui_tests::radial_blur_dialog_sets_its_center`: Radial Blur has no preview, a click in Blur Center's upper left quarter moves the center there, and Enter applies "Radial Blur". Screenshots `radial_blur.png` and `radial_blur_center.png`, compared with Photoshop's capture: positions within about a point (Photoshop's group boxes are a shade lighter and rounded).

- `crystallize_pointillize_and_diffuse_apply` (UI test) also applies both and checks the document is untouched while their dialogs are open; `screenshot_pixelate_and_diffuse_dialogs` (`#[ignore]`) captures them.
