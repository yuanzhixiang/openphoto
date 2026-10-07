# layer_ops.rs: Layer menu operations

## Responsibilities

Implements the operations in the Layer menu and the Layers panel that change layer structure: duplicating layers, Layer Via Copy/Cut, converting the background layer to a regular layer, reordering, renaming, merging, flattening, and deleting hidden layers. All functions only change the document and record no history (`op-ui` records it).

## Public interface and rules

### Duplicate

- The name of a copy (`copy_name`, measured in Photoshop 2026): normally "Name copy"; if that exists, "Name copy 2", "Name copy 3", and so on. When the original name itself ends in " copy" or " copy N", no further "copy" is appended; instead the first free number is found starting at 2 (or N+1): "A copy" → "A copy 2", "E copy 7" → "E copy 8"; "Gcopy" does not count (→ "Gcopy copy").
- `duplicate_selected(doc)`: dragging the selected layers onto the "Create a new layer" button in the Layers panel. With only one selected, same as `duplicate`; with several selected, each one (a group together with its contents) is copied once, and the copies are inserted as a block, in their original top-to-bottom order, directly above the topmost selected layer (in the same parent group as it). Then all copies are selected, and the copy of the active layer becomes the active layer (measured in Photoshop 2026: selecting A and Background copy and dragging them up gives "A copy" on top and "Background copy 2" below it).
- `duplicate(doc)`: Layer › Duplicate Layer. Inserts an identical layer (same pixels, visibility, opacity, Fill, blend mode, locks) directly above the active layer, makes it the active layer, and returns the new ID; returns `None` when there is no active layer. The name is "Name copy"; if that exists, "Name copy 2", "Name copy 3", and so on, matching Photoshop. A copy of the background layer is a regular layer.
- `can_merge_selected(doc)` / `merge_selected(doc)`: Layer › Merge Layers (⌘E with multiple layers selected): the visible ones among the selected layers are merged into one, placed at the position of the topmost one and keeping its name (if the background layer is among them, they are merged into the background layer); the result's opacity, fill and blend mode are reset, and masks are merged into the pixels; hidden selected layers stay untouched. Available when at least two selected layers are visible.
- `delete_selected(doc)`: deletes all selected layers; refuses when it would delete every layer. Afterwards the layer below the lowest deleted layer (or, if there is none, the lowest remaining layer) becomes the active layer.
- `toggle_selected_visibility(doc)`: Layer › Hide Layers / Show Layers: hides all selected layers; when all are already hidden, shows them all instead. Returns whether they are now visible.
- `can_reverse(doc)` / `reverse_selected(doc)`: Layer › Arrange › Reverse: two or more selected layers in the same group (a group together with its contents) swap positions so their order is reversed; unavailable when the background layer is included or the layers are not in the same group.
- `can_merge_group(doc)` / `merge_group(doc)`: ⌘E when the current layer is a non-empty group (Merge Group): the layers in the group are merged into one pixel layer, which takes the group's position and keeps the group's id, name, visibility, opacity, fill, mask, color label and blend mode (Pass Through becomes Normal); the image looks the same before and after the merge.
- Layer groups:
  - `can_group(doc)` / `group_selected(doc)`: Layer › Group Layers (⌘G): the selected layers (together with their contents; layers already inside another selected group are not counted twice) are put into a new "Group N", which is placed at the original position of the topmost selected layer and in its group; unavailable when the background layer is selected. The new group is selected.
  - `ungroup(doc)`: Layer › Ungroup Layers (⇧⌘G): the current group disappears, its direct children move into the group it was in, and all of them are selected.
  - `new_group(doc)`: an empty "Group N", inserted by the same rules as `Document::insert_above_active`, and selected.
  - `can_move_blocks(doc, ids, gap)` / `move_blocks(doc, ids, gap)`: dragging several selected rows: these layers (each together with its contents; a layer nested inside another one of them is not counted twice) move together to `gap` in their original top-to-bottom order, with the same parent-group rules as `move_block`. Moving an already adjacent set to its own edge counts as no change. `move_block` is the single-layer case.
  - `can_move_block(doc, id, gap)` / `move_block(doc, id, gap)`: moves a layer together with its contents to `gap` (a position in `layers` before the move, between `gap - 1` and `gap`). The new parent group: when the layer directly above `gap` is a group and the layer directly below is inside it, that group (placed at the top of the group); otherwise the same group as the layer directly above; at the very top, no group. The background layer cannot move, nothing can be placed below the background, and a group cannot be put inside itself. Drag-reordering in the Layers panel uses this.
  - `delete_selected` deletes a group together with its contents (Photoshop "Group and Contents"). `delete_selected_keep_contents`: "Group Only": the selected groups disappear and their layers stay at the group's original position (moving into the group's parent group); other selected layers are deleted as usual. `deleting_groups_with_contents(doc)`: the name of the first non-empty group among the selected layers; the UI uses it to ask first.
  - Duplicated layers keep their link number, i.e. layers linked to the original are also linked to the copy (measured in Photoshop 2026); links are cleared when duplicating into another document.
  - Duplicating (Duplicate Layer, ⌘J, duplicating into another document) a group copies all the layers in the group with it (`cloned_block`: new ids, with parent-child relations inside the block remapped accordingly), placed directly above the original group (in another document, at `Document::insertion_point`).
- `duplicate_name(doc)`: the default value of "As" in the Duplicate Layer dialog (same naming rules as `duplicate`).
- `duplicate_named(doc, name)`: duplicates within the same document with the name `name`.
- `duplicate_into(source, target, name)`: duplicates the active layer of `source` into another document `target`: pixel positions stay the same (`with_canvas` with an offset of 0; pixels outside the `target` canvas are kept outside the canvas), the layer mask is extended (reveal) or cropped to the new canvas, and a copy of the background layer is a regular layer; it is inserted above the active layer of `target` and made the active layer.
- `duplicate_to_new(source, title, name)`: creates a new document with the same size and resolution as `source` and the title `title`, containing only the copy `name` of the active layer.
- `via_copy(doc)`: Layer › New › Layer Via Copy (⌘J).
  - No selection: copies the whole layer. A copy of the background layer is named "Layer N" by `next_layer_name`; a copy of a regular layer is named "Name copy" (same rules as above).
  - With a selection: puts the pixels inside the selection (same rules as Edit › Copy, see `clipboard.md`) onto a new layer "Layer N" at the original position; the selection is kept (unlike Paste). Returns `ClipError::Empty` when the selection contains only transparent pixels.
  - The new layer takes the source layer's opacity, Fill and blend mode (defaults when the source is the background layer).
- `via_cut(doc, background)`: Layer › New › Layer Via Cut (⇧⌘J). Cuts the selection like Edit › Cut (leaving the `background` color on the background layer; returns an error for hidden layers or layers with locked pixels), then puts the pixels onto a new layer "Layer N" at the original position; the selection is kept, and attribute rules are as above. With no selection it acts on the whole layer, but the UI only offers this command when there is a selection.

### Background layer

- `layer_from_background(doc)`: turns the background layer into a regular layer "Layer 0" and removes all locks; afterwards it can have transparent pixels and can be moved and reordered. Returns `false` when there is no background layer.

### Order

- The layer list runs bottom to top; the background layer can only be at the very bottom.
- Locks (the background layer does not take part; when only the background is selected, all three functions do nothing):
  - `selected_locks(doc)`: the initial state of the Lock Layers dialog. Each item counts as on only if it is on for every selected layer; `None` when only the background is selected (or nothing is selected), in which case Layer › Lock Layers... is unavailable.
  - `set_selected_locks(doc, locks)`: sets the five lock flags on every selected layer and returns whether anything changed.
  - `toggle_lock_all(doc)`: ⌘/. When not all selected layers are "lock all", turns on `lock_all` for them (individual locks stay as they are); when all of them are already "lock all", clears all locks (including the individual ones, unlike the panel button). Returns whether they are now locked; `None` when only the background is selected. All of the above was measured in Photoshop 2026.
- `arrange_target(doc, arrange)` / `arrange(doc, arrange)`: the four Layer › Arrange items only move within the group the current layer is in (among sibling layers), and a group moves together with its contents (via `move_block`), so Bring to Front goes to the top of the group, not to the top of the whole document; the background layer does not move, and no layer passes it.
- `arrange(doc, arrange)`: performs the move above and returns whether anything moved.

### Rename

- `rename(doc, id, name)`: renames after trimming leading and trailing whitespace. Returns `false` when the name is empty or the same as the old one.

### Merge

Merging (Merge Down, Merge Layers, Merge Group, Merge Visible) includes pixels outside the canvas and keeps them in the result (measured in Photoshop 2026): compositing happens on a temporary canvas enlarged to cover all pixels, and the result is put back in place. When the result lands on the background layer, only the part inside the canvas is kept.

- `can_merge_down(doc)`: the active layer is not the bottom layer, and both it and the layer below are visible.
- `merge_down(doc)`: Layer › Merge Down (⌘E). Composites the active layer onto the pixels of the layer below using its blend mode, opacity and Fill (the lower layer's pixels take part as Normal, 100%); the result keeps the lower layer's name and attributes, the active layer is deleted, and the lower layer becomes the active layer.
- `can_merge_visible(doc)`: more than one visible layer.
- `merge_visible(doc)`: Layer › Merge Visible (⇧⌘E). Composites all visible layers into one layer: into the background layer when a visible background layer exists, otherwise into the active layer (the topmost visible layer when the active layer is hidden). The result's blend mode is reset to Normal and its opacity and Fill to 100% (these are already reflected in the composite), and it becomes the active layer; hidden layers stay unchanged.
- `flatten(doc)`: Layer › Flatten Image. Composites the visible layers over white, giving a single opaque "Background" background layer; hidden layers are discarded.
- `delete_hidden(doc)`: Layer › Delete › Hidden Layers. Deletes all hidden layers; returns `false` and changes nothing when there are no hidden layers or all layers are hidden. When the active layer is deleted, the topmost layer becomes the active layer.

All merges are computed via `Document::composite_layers_rgba8`, with the same rules as document compositing (see `document.md`).

### Layer masks

- `NewMask`: `RevealAll`, `HideAll`, `RevealSelection`, `HideSelection` (the first four items of Layer › Layer Mask).
- `can_add_mask(doc)`: the current layer exists, is not the background layer, and has no mask yet.
- `add_mask(doc, kind)`: adds a mask to the current layer: all white, all black, the selection (the selection degree is the mask value) or the inverted selection; the selection kinds require a selection (the selection is kept). Afterwards the editing target switches to the mask (`mask_target = true`), matching Photoshop.
- `delete_mask(doc)`: deletes the mask without applying it; the editing target returns to the pixels.
- `apply_mask(doc)`: an enabled mask is multiplied into the layer's alpha (rounded), then the mask is deleted; a disabled mask is simply deleted. The editing target returns to the pixels.
- `toggle_mask(doc)`: disables/enables and returns the new state; `None` when there is no mask.
- Merging: the results of Merge Down and Merge Visible already include the effect of each layer's mask, and the resulting layer has no mask; duplicating a layer duplicates its mask too.

## Known limitations

- When the lower layer in a merge has a non-Normal blend mode or opacity, Photoshop's result may differ from the approximation here (composite as Normal 100% first, then keep the lower layer's attributes).
- No layer groups or multi-selection; the only Merge Layers variant is Merge Down.
- Layers are the same size as the canvas, and so are merge results.

## Test coverage

- `arranging_stays_within_the_group_and_reverse`: Bring to Front inside a group only goes to the top of the group; moving a group backward carries its contents past the layer below; a top-level layer moves forward; Reverse reverses the order of a group and two layers while the group's contents stay unchanged.
- `merging_a_group`: after merging a group only one pixel layer with the same id and name remains, and the image is identical to before the merge.
- `deleting_only_the_group_keeps_its_layers`, `duplicating_a_group_copies_its_layers`: deleting only the group keeps its layers and moves them out of the group; duplicating a group gives "Group 1 copy" and copies of its layers, and duplicating into another document also brings the group's layers.
- `grouping_ungrouping_and_moving_blocks`: ⌘G puts two non-adjacent layers into a new group (position, parent-child relations, selection); when the group is expanded a new layer is inserted at the top of the group; a layer outside the group moves to the top inside the group; a group cannot move into itself or below the background; after ungrouping the structure is restored and the former children are selected; deleting a group deletes its contents.
- `selected_layers_merge_delete_and_hide`: two layers merge into the upper one with the correct name and position and the opaque pixels stacked, unselected layers stay put; hiding/showing selected layers; after deleting selected layers the active layer is correct, and the last layer cannot be deleted.
- `duplicate_layer_dialog_targets`: default name "Background copy"; duplicating with a new name within the same document; duplicating into a smaller document keeps the position, keeps the overflow outside the canvas, and becomes the active layer; duplicating into a new document gives the correct size, title and single layer.
- `duplicates_are_named_like_photoshop`: "copy", "copy 2" naming; ⌘J on the background layer gives a regular layer like "Layer 2".
- `via_copy_and_cut_move_the_selection_in_place`: the selection is copied to "Layer N", taking the opacity and keeping the selection; after cutting, the corresponding pixels of the original layer become transparent.
- `arrange_keeps_the_background_at_the_bottom`: Send to Back stops above the background, and the background cannot move up.
- `dragging_and_renaming` (dragging uses `move_block`): allowed and refused cases of `move_layer`; renaming trims whitespace and rejects empty and unchanged names.
- `masks_hide_reveal_and_apply`: the background layer cannot have a mask; Hide All hides the layer, which shows again when the mask is disabled; filling white on the mask reveals it again with the layer pixels unchanged; applying a 50% gray mask gives alpha 128; creating a Hide Selection mask from a selection; deleting a mask.
- `merge_down_keeps_the_lower_layer`: 50% red merged onto a white background gives `[255, 128, 128, 255]`, and the result is still the background layer.
- `merge_visible_leaves_hidden_layers`: hidden layers are kept, and visible layers are merged into the background layer.
- `flatten_fills_transparency_with_white`: hidden layers are discarded, and transparent areas become white.
- `delete_hidden_keeps_visible_layers`: deletes hidden layers and fixes up the active layer.
- `lock_layers_sets_every_selected_layer_but_the_background`: with only the background selected there are no lockable layers; the dialog's checkboxes are the intersection of all selected layers; the setting applies to the selected non-background layers; setting it again makes no change.
- `lock_all_is_a_flag_of_its_own`: ⌘/ turns on "lock all" without changing the individual locks, protecting pixels and transparency; pressing it again clears all locks.
- `copies_of_copies_count_on`: copies of copies are named by Photoshop's numbering rules.
- `duplicating_several_layers_puts_the_copies_on_top`: position, names and selection state of a multi-selection duplicate match Photoshop; with a single selection the copy is directly above the original layer.
- `moving_several_blocks_together`: two non-adjacent layers move together to the top and to directly above the background; moving in place, moving below the background and moving the background are not allowed.
- `a_locked_group_locks_its_layers`: when a group locks position, layers inside it cannot move but can be painted; when a group is Lock all, layers inside it cannot be painted, moved or deleted (`delete_selected` refuses to delete layers inside a locked group).
