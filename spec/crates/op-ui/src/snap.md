# snap.rs: View › Snap

## Responsibilities

Pulls the points and boxes of a drag to guides, grid lines, layer edges and the document's bounds when they come close, as Photoshop's View › Snap does.

## Settings

`ViewOptions` (see `rulers.md`): `snap` (View › Snap, ⇧⌘;) and the View › Snap To items `snap_guides`, `snap_grid`, `snap_layers`, `snap_slices`, `snap_bounds`. All are on by default, matching Photoshop 2026 (read from its menus). Snap To › All / None turn the five items on or off together; the Snap To items stay available when Snap is off, and then have no effect.

## Targets (`Targets`)

Gathered once when a drag starts (`begin`, called by `document_view.rs`) and kept in `DocState::snap`, since finding layer bounds every frame is not cheap:

- Guides (Snap To › Guides): only while guides are shown (Extras and Show › Guides on). Vertical guides give x lines, horizontal ones y lines.
- Grid (Snap To › Grid): only while the grid is shown; every major line and subdivision (a quarter inch apart from the rulers' origin, the same lines `rulers::draw_grid` draws).
- Layers (Snap To › Layers): the four edges of the non-transparent pixels of every visible layer except the background and the layers being moved.
- Document Bounds: x = 0 and the width, y = 0 and the height.
- Slices: a switch only; there are no slices yet.

`Targets::new` returns `None` when Snap is off.

## Snapping

- Distance: 8 screen points (`DISTANCE`), converted to document pixels at the current zoom (`tolerance`). Each axis snaps on its own to the nearest target within it.
- `point`: a document point pulled to the targets. Used by the Rectangular and Elliptical Marquee (the start point and the dragged point) and by the shape tools (Rectangle, Ellipse, Triangle, Polygon, Line, Custom Shape). The marquee then rounds to whole pixels as usual.
- `offset`: the Move tool's drag offset adjusted so the left edge, right edge or center (top, bottom, middle) of the box being moved lands on a target; of the candidates within the distance, the one changing the offset least wins. The box is `op_core::transform::bounds` at the start (the selection's bounds, or the moved layers' pixels). The moved layers (the selected layers and those linked to them) are left out of the targets.
- Free Transform: a drag while transforming gathers targets as the Move tool does, leaving out the layers being transformed. While the box is neither turned nor reshaped:
  - Moving it pulls the box's edges or middle onto targets (`offset`, with the box as it is at the drag's start as `moving`).
  - Moving it also lets the smart guides line the box up (`SmartGuides::align_with`, as for the Move tool).
  - A side or corner handle pulls the pointer itself to the targets only (`point_plain`: the drag's smart guides belong to the box).
- Crop tool: a drag gathers targets. While the box isn't turned (box space is then the document's), `crop_tool::snap_box` does the snapping:
  - Moving the box pulls its edges or middle onto targets.
  - A handle pulls each edge it moved to the nearest line within the distance.
- Guides: dragging a guide out of a ruler or moving one gathers targets, and the guide's position is pulled to the nearest line. A moved guide doesn't snap to itself.
- Holding Control while dragging turns snapping off for that moment, as in Photoshop.
- `begin` also starts the smart guides (`smart_guides.md`): for a Move drag (or a transform) the moved box's, otherwise a point's (`for_points`). `point` takes `&mut DocState` and combines both pulls, the smaller winning per axis.

## Known limitations

- The Object Selection rectangle and the lassos do not snap; Free Transform's turned or reshaped boxes, Warp and a turned crop box do not either.
- Shift-dragging a guide does not snap it to the rulers' ticks.
- Photoshop's exact snap distance has not been measured; 8 points is an approximation.

## Test coverage

- `snap::tests::points_and_boxes_snap_within_the_distance`: points snap per axis within the distance and not beyond; a box's edge and its center pull the offset onto a target.
- `ui_tests::snapping_to_guides_layers_and_bounds`: see `ui_tests.md`.
- `ui_tests::guides_transforms_and_crops_snap`: a guide moved to 203 lands on a layer edge at 200; a transformed layer moved 47 px lands its edge on a guide (offset 50); a Classic crop box's right side dragged to 351 lands on the guide at 350.
