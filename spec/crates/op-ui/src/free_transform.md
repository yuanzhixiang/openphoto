# free_transform.rs: Free Transform canvas interaction

## Component responsibilities

The canvas interaction while Edit › Free Transform (⌘T) is in progress: shows the transform box, drags to move/scale/rotate, and commits or cancels. For the pixel transform see `op-core`'s `transform.md`.

## State

The session is stored in `DocState::free_transform` (`FreeTransform`): the document snapshot at the start `before`, the original bounds `bounds`, translation `offset`, scale `scale`, angle `angle`, skew `skew` (horizontal and vertical, in radians), reference point `reference` and whether it is shown `show_reference`, relative positioning `relative`, width/height link `linked`, interpolation `interpolation`, free corners `quad` (present only after skew, distort or perspective: top-left, top-right, bottom-right, bottom-left, in document pixels), mode `mode` (`TransformMode`: Free, Skew, Distort, Perspective), the drag in progress `drag`, and the transform currently shown in the document `applied` (`Projective`). The box mapping `mapping()`: with free corners, the projective transform from the bounds to the corners (`Projective::rect_to_quad`); otherwise "translate to the bounds center + offset · rotate by angle · skew by skew · scale by scale · translate back from the bounds center". The preview uses the chosen interpolation (`transform_with`).

## Flow

- `start(state)`: checks and obtains the bounds with `transform::bounds`, saves the snapshot and starts the session. `start_in(state, mode)`: Edit › Transform › Scale, Rotate (both Free), Skew, Distort, Perspective; when already transforming it only switches the mode. On failure the caller shows an alert (e.g. "Could not complete the Free Transform command because the layer is locked." on the background layer with no selection).
- While in progress, `preview` runs every frame: when the box mapping differs from `applied`, it first restores the snapshot and then applies the new mapping to the document, so the document shows the result live. The preview records no history.
- Commit (`commit`): Enter, double-clicking inside the box, or the ✓ button in the options bar. When the mapping is not the identity, it records "Free Transform" (Photoshop 2026 records this name whichever mode it started from, as measured) and stores the mapping as `AppState::last_transform` (for Transform › Again); the identity is treated as a cancel.
- Cancel (`cancel`): Esc or the ⦸ button in the options bar restores the snapshot.
- When an input field has keyboard focus, Enter and Esc do not act on the transform.
- During the session `AppState::transforming()` is true, and so is `modal_open()`: menu commands and single-key tool shortcuts have no effect, as in Photoshop.

## Warp

- Edit › Transform › Warp, Warp in the context menu, and the warp toggle button in the options bar enter Warp mode (`set_mode`): the mesh starts as a flat grid and then goes through the box's current mapping, so earlier scaling and rotation are kept.
- Dragging: pressing on a shown control point (within 8 pt; the anchors and the handles along the patches' edges) moves that control point; pressing on the surface pulls that point (`WarpMesh::pull`, within its patch) and the surface bends with it; pressing outside the surface has no effect.
- Split (the options bar's Split Crosswise, Vertically, Horizontally buttons, `FreeTransform::warp_split`): the chosen button is lit until the next click on the mesh, which adds a column and/or a row of patches at that surface point (`split_u` / `split_v`, the surface unchanged); clicking the button again turns it off. Not available with a preset style.
- Grid (options bar, native menu): Default (one patch, drawn with thirds), 3 x 3, 4 x 4, 5 x 5 patches; picking one regrids the current surface (`regrid`). A split mesh shows "Custom".
- Warp (options bar, native menu): Custom, then the 15 preset styles. A style replaces the mesh with that shape at 50% Bend (keeping the box's earlier transform in `then`); only its Bend handle is shown (a white square on the middle of the top edge, the left edge in the vertical orientation), and dragging it up (left) bends more, by two times the box's height (width) per 100%. Bend, H and V can be typed in the options bar (percent, −100..100); the orientation button turns the shape. Custom turns the shape into editable points (`freeze_style`).
- Appearance (Photoshop 2026): a blue `#5b8be6` surface border (1.5 pt) and curves at the thirds (0.75 pt) for one patch, at the patches' edges for a grid; the shown control points are blue dots, larger at the anchors.
- The preview uses `transform::warp`; commit records "Warp" (measured in Photoshop 2026), and a surface that has not moved from the box (checked on a 9 × 9 grid of points) counts as a cancel; it does not take part in Transform Again.
- The options bar becomes the warp bar (see `options_bar.md`).

## Transform Selection

Select › Transform Selection (`start_selection`, available when there is a selection): the same transform box surrounds the selection, but only the selection outline moves (`transform::transform_selection`), not the pixels; commit records "Transform Selection" and does not update Transform Again. Test `ui_tests::transform_selection_moves_only_the_outline`.

## Interaction (matching Photoshop)

- On press, the part grabbed is determined (in screen coordinates): within 8 pt of one of the 8 control points (the four corners and four edge midpoints) is scaling; inside the box is moving; outside the box is rotating.
- Move: the box follows the pointer.
- Scale:
  - By default the opposite control point is the fixed point; holding ⌥ makes the reference point the fixed point (the center by default), the handle's distance to it scaling along the box's axes.
  - Corner control points scale proportionally by default (the pointer displacement is projected onto the box's diagonal); holding Shift scales freely.
  - Edge control points change the size in one direction only; the center in the other direction stays put.
  - The absolute scale factor is at least 1 / the longer side of the original bounds (the box never shrinks to 0); crossing the fixed point flips it.
  - After the box is rotated, scaling is computed along the box's own axes.
- Rotate: about the reference point (the box's center unless another was picked or the point was dragged), the angle following the pointer's angle relative to it; holding Shift snaps to multiples of 15°.
- The reference point (when shown with the options bar's checkbox): dragging it (within 8 pt, before the box has free corners) moves it anywhere, a point of the box that then turns and scales with it (`reference_custom`); picking one of the nine in the options bar puts it back on the box. Its cursor is the crosshair.
- Skew, distort, perspective (Photoshop's modifier keys; in Skew, Distort and Perspective modes the same happens without modifier keys):
  - ⌘-drag a corner: distort, moving only that corner; ⌘-drag an edge: both corners of that edge move together.
  - ⌘⇧-drag an edge: skew, the edge slides along its own direction; in Skew mode dragging a corner moves it only along whichever of horizontal or vertical is larger.
  - ⌘⌥⇧-drag a corner: perspective, the corner moves along the larger direction and the adjacent corner on the same edge moves the same distance in the opposite direction.
  - Once there are free corners: an ordinary drag on a control point continues to be treated as distort; dragging inside the box translates the corners; dragging outside the box rotates the corners about their midpoint (Shift 15°).
- Context menu (`context_menu`): Free Transform, Scale, Rotate, Skew, Distort, Perspective (switch mode), grayed-out Warp, Content-Aware Scale, Puppet Warp, Rotate 180°, Rotate 90° Clockwise, Rotate 90° Counter Clockwise, Flip Horizontal, Flip Vertical (`turn_box`: rotates or flips the box itself; the parametric box changes its angle or scale sign, free corners rotate or flip about their midpoint).
- Cursor: the move cursor inside the box; the rotate (`Alias`) cursor outside the box; on control points, a double-headed arrow adjusted for the box's angle.

## Appearance

- Box: a thin 1 pt blue (`#2c8be8`) line.
- Control points: 7 pt white squares with a dark gray stroke.
- Center reference point: a circle of radius 4 pt with crosshairs.
- The reference point (a circle of radius 4 pt with crosshairs) is shown only when the reference point checkbox in the options bar is on, at the transformed position of the chosen reference point (center by default); Photoshop 2026 does not show it by default.
- Options bar (see `options_bar.md`): reference point, X, Y, W, H, angle, skew, interpolation, cancel and commit.

## Move tool transform controls

`controls_handle_at` and `draw_controls` are used by the Move tool's Show Transform Controls (see `document_view.md`): the box and control points are drawn with `draw_box`, shared with Free Transform, without the center reference point; `controls_handle_at` determines whether the pointer grabs a control point (the same 8 pt grab radius as Free Transform).

## Known limitations

- Warp's Grid menu has no "Custom..." size, and the transform box's and handles' look has not been compared pixel by pixel with Photoshop 2026 yet.
- After switching from Warp back to Free Transform, Free Transform drags no longer stack on top of the warp. There is no Content-Aware Scale or Puppet Warp (P3 #26).
- The preset styles' shapes approximate Photoshop's (see `op-core`'s `transform.md`).

## Test coverage

- `corner_scales_proportionally_from_the_opposite_corner`: dragging a corner to scale up 2× proportionally keeps the top-left corner fixed; with Shift only the width grows.
- `side_handles_move_and_rotate`: edge control points scale in one direction only with the opposite edge fixed; the move offset; Shift rotation snaps to 90°.
- `hit_testing_the_quad`: whether a point is inside the box.
- `distort_skew_and_perspective`: ⌘-dragging the bottom-right corner moves only it, and ordinary drags afterwards continue to distort; in perspective mode the top-right corner pulls out and the top-left pulls in; ⌘⇧-dragging the right edge slides it along itself; the mapping follows the corners.
- `turning_and_flipping_the_box`: the context menu's rotate and flip change the parametric box's angle and scale; free corners rotate 180° about their midpoint.
- `ui_tests::transform_distort_from_the_menu`: after Edit › Transform › Distort, dragging the bottom-right corner and committing records "Free Transform"; the far corner turns red while the top-left stays put, and Transform Again is available.
- `options_bar_numbers_pivot_on_the_reference_point`: with the reference point at the top-left, W 50% keeps the top-left corner fixed; rotating 90° about the center keeps the center fixed; a 45° horizontal skew moves the top-left corner left.
- `ui_tests::transform_bar_takes_typed_numbers`: typing 50 in the W field and pressing Return makes both W and H 50% while still transforming; after entering 90 for the angle the center stays put.
- `rotating_and_alt_scaling_pivot_on_the_reference_point`: with the reference point at the top left, a quarter turn and an ⌥ corner drag to twice the size keep that corner; the reference point dragged to the bottom-right corner becomes the custom point and a turn keeps it in place.
- `ui_tests::warp_splits_grids_and_styles`: Split Crosswise and a click make 2 × 2 patches at about (0.5, 0.25) without moving the surface; Grid 4 x 4; Warp › Arc raises the top's middle, Bend 100 raises it more, dragging the Bend handle down lowers the bend; committing records "Warp".
- `ui_tests::warp_pulls_the_surface`: after Edit › Transform › Warp, dragging the bottom-right control point out 30 pixels and pulling the middle of the surface up, committing records "Warp", and the stretched corner now has pixels.
