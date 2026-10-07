# perspective_crop.rs: Perspective Crop tool

## Component responsibilities

The Perspective Crop tool (in the Crop tool's group, C / ⇧C): a four-cornered box drawn on the image whose corners move independently; cropping maps what is inside it onto a rectangular canvas, straightening a photographed plane (Photoshop 2026's tool). The pixel work is `op_core::image_ops::perspective_crop` (`image_ops.md`).

## State

`DocState::perspective_crop` (`PerspectiveBox`): `quad` (top-left, top-right, bottom-right, bottom-left as drawn, document pixels) once a box exists; `clicks`, the corners placed by clicking before that; `drag`, the drag in progress (`New`, `Corner(i)`, `Side(i)` for the side from corner i to i + 1, or `Move`, with the start point and the box then).

The options bar (`options_tools.rs`, `pcrop.*` settings, read by `PerspectiveOptions::from_app`): W and H (a number with an optional px/in/cm/mm unit; no unit means pixels), Resolution with Pixels/in or Pixels/cm, Front Image (the active document's size and resolution), Clear, and Show Grid (on by default).

## Interaction

- Dragging on the image with no box draws a rectangular box; four clicks place its corners in order.
- With a box: a corner handle moves that corner alone; a side's middle handle moves both corners of that side; dragging inside moves the box; dragging outside draws a new box. Handles are grabbable within 10 pt. A drag that leaves the box under 1 pixel wide or high is undone (a new box is dropped).
- Enter, a double click inside the box, the options bar's ✓ (at the Crop tool's place, 1138.5) or picking another tool crops; Escape or ⦸ (1103.75) removes the box. While a dialog is open the tool takes no input.
- Cropping (`commit`): the output size is W × H when both are typed (printed units need a resolution), otherwise the box's mean side lengths (`image_ops::perspective_size`); a typed resolution becomes the document's. Records "Perspective Crop".
- Cursors: Move inside the box, otherwise the crosshair.

## Appearance

- The image and anything around it up to the box is shielded with the pasteboard color at 75% (drawn by egui in gamma space, as a ring of quads between the image's bounds and the box).
- Show Grid: lines across the box through its projective map, about 40 pt apart on screen (at least two cells each way), white at 43%.
- The box: a 1 pt dark `#323232` line with a ½ pt light `#f4f4f4` line on it; 7 pt square handles (light, dark border) at the corners and side middles. Before the box exists, the clicked corners are joined by a light line with handles.

## Known limitations

- The look (shield blending, line and handle sizes, grid spacing) and the output size Photoshop picks without a typed size are approximations; they have not been measured against Photoshop yet.
- Photoshop warns when the box's sides don't describe a plausible perspective; this tool crops any convex box.

## Test coverage

- `typed_sizes_and_the_inside_test`: "4 in" × "300" at 100 ppi is 400 × 300; inches without a resolution give no size; points inside and outside a skewed box.
- `ui_tests::perspective_crop_straightens_a_box`: a drawn box from (100, 100) to (500, 400) and its top-right corner moved to (450, 150) crop to the box's mean side lengths and record "Perspective Crop"; four clicks place a box and typed 120 × "80 px" crops to 120 × 80; Escape removes a box.
