# move_tool.rs: Move tool

## Responsibilities

The Move tool's changes to pixels: moving all pixels of the current layer, or only the selected pixels.

## Public interface

- `Move::begin(doc, background)`: starts moving the current layer, saving the layer's pixels and the selection at the start. Every `apply` computes from these original pixels, so moving back and forth during a drag accumulates no loss.
- `Move::apply(doc, dx, dy)`: shows the result of moving (`dx`, `dy`) pixels relative to the starting position.
- `MoveError::message()`: Photoshop's message when moving is not possible:
  - No layer: "Could not use the move tool because there is no layer."
  - Locked: "Could not use the move tool because the layer is locked."
  - Hidden: "Could not use the move tool because the target layer is hidden."

## Rules

- No selection: all pixels of the layer are shifted, and the vacated area becomes transparent.
- With a selection: the selected pixels (partially lifted according to selection degree) are moved away, the original position becomes transparent (the background color on the background layer), and they are then placed at the new position on top of the pixels left behind; the selection moves along with them.
- Moving is not possible (`Locked`) in these cases: the layer has "lock position" or "lock pixels", or it is the background layer with no selection. With a selection, the selected pixels of the background layer can be moved, matching Photoshop.

## Known limitations

- Pixels moved off the canvas are kept on the layer (shifted with `with_canvas`, or selected pixels placed with `set_pixel_at`), so the content is still there when moved back, matching Photoshop; the background layer is the exception: its pixels exist only within the canvas, and the part moved off is discarded.
- With a selection, every step traverses the whole layer.

## Multiple selected layers

Without a pixel selection, besides the active layer, the other selected layers and the layers linked to selected layers (`link::with_linked`) all move by the same offset as long as they can move (visible, not the background, position or pixels not locked) (measured in Photoshop 2026: with A selected, pressing → also moves C, which is linked to A); selected layers that cannot move, such as the background, stay in place. With a pixel selection, only the selected pixels of the active layer move (matching Photoshop).

## Layer groups

Without a pixel selection, a selected group moves together with all the pixel layers in it (`Document::pixel_layers`); when the current layer is an empty group and there are no other movable layers, `Locked` is returned. With a pixel selection and a group as the current layer, `MoveError::Group` is returned ("Could not use the move tool because the target layer is a group."). Visibility is judged by `Document::is_shown` (a layer whose group is hidden also counts as hidden).

## Test coverage

- `linked_layers_move_along`: with only one layer selected, the layers linked to it move along; with a pixel selection, linked layers do not move.
- `selected_layers_move_together`: with two regular layers and the background layer selected, both regular layers move and the background does not.

- `pixels_moved_off_the_canvas_come_back`: a dot moved 5 px past the left edge of the canvas is not visible on the canvas but is still on the layer at (−5, 2); moving it back restores it to its original position.
- `selected_pixels_moved_off_the_canvas_are_kept_except_on_the_background`: on a regular layer, selected pixels moved above the canvas are still kept; on the background layer, the part moved off is discarded.

- `moves_whole_layer_from_its_start`: repeated `apply` calls are all computed relative to the starting position.
- `moves_only_selected_pixels`: only pixels inside the selection move, those outside stay put, and the selection moves along.
- `background_needs_a_selection_and_leaves_background_color`: the background layer refuses without a selection; with a selection, the original position is filled with the background color.
