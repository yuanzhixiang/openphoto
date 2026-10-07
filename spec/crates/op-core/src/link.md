# link.rs: Linked layers

## Responsibilities

The rules for Layer › Link Layers / Unlink Layers / Select Linked Layers. Layers linked together move along with the Move tool (`move_tool.md`), show a link icon in the Layers panel (`panels/layers.md` in `op-ui`), and are stored in PSD image resource 1026 (`psd.md` in `op-io`).

## Data

`Layer::link: Option<u32>` is the link number; layers with the same number are linked together. A number that no other layer shares is equivalent to no link, so leftover numbers need no cleanup after deleting layers or unlinking.

## Rules (tested in Photoshop 2026)

- `linked_with(doc, id)`: the other layers linked with `id`. `is_linked`: whether there are any.
- `with_linked(doc)`: the selected layers, plus the layers linked with any selected layer (bottom to top). The panel shows the link icon on the linked layers among them; these are what the Move tool moves.
- `can_unlink(doc)`: every selected layer is linked (to anything, not necessarily the same group). In that case the menu item and panel button act as "Unlink Layers".
- `can_link(doc)`: two or more layers are selected and `can_unlink` does not hold.
- `link_selected(doc)`: the selected layers and the layers already linked with them are merged into one group (using a new number). For example, with A and C linked, selecting B and A and linking makes A, B and C all linked.
- `unlink_selected(doc)`: only removes the selected layers from their respective link groups; the remaining layers in a group stay linked to each other (for example, with A, B, C linked, selecting only C and unlinking leaves A and B still linked).
- `toggle(doc)`: unlinks when unlinking is possible, otherwise links; returns the history name "Unlink Layers" or "Link Layers", or `None` when neither is possible.
- `can_select_linked` / `select_linked`: Select Linked Layers is available only when it would select more layers (it is grayed out when the selection already includes all linked layers); the active layer is unchanged after it runs. No history is recorded.
- The background layer can also take part in links.
- When a layer is duplicated, the copy keeps the number (linked with the original layer's link group); it is cleared when duplicating into another document (`layer_ops.md`).

## Tests

- `matches_photoshop`: checks step by step in the order walked through in Photoshop 2026 (a single layer cannot be linked; link A, C; B selected alone shows no link; linking B, A brings in C; unlink C alone; link C with the background; with A linked and D unlinked it is Link; A and C in two different groups give Unlink; availability and results of Select Linked Layers).
- `deleting_a_linked_layer_leaves_no_stale_link`: after deleting one of two linked layers, the other no longer counts as linked.

## Known limitations

- Free Transform and Align/Distribute do not yet carry linked layers along; ⇧-clicking the link icon to temporarily disable a link is not implemented.
