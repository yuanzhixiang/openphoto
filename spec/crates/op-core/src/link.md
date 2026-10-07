# link.rs: Linked layers

## Responsibilities

The rules for Layer › Link Layers / Unlink Layers / Select Linked Layers. Layers linked together move along with the Move tool (`move_tool.md`), show a link icon in the Layers panel (`panels/layers.md` in `op-ui`), and are stored in PSD image resource 1026 (`psd.md` in `op-io`).

## Data

`Layer::link: Option<u32>` is the link number; layers with the same number are linked together. A number that no other layer shares is equivalent to no link, so leftover numbers need no cleanup after deleting layers or unlinking.

## Rules (tested in Photoshop 2026)

- `linked_with(doc, id)`: the other layers linked with `id` that take part in moves: empty when `id`'s link is disabled, and partners whose link is disabled are left out. `is_linked`: whether there are any.
- `link_set(doc, id)`: every layer in `id`'s link group, including `id` and disabled partners (empty when nothing else shares the number). The Layers panel uses it to decide which rows show the link icon.
- `with_linked(doc)`: the selected layers, plus the layers linked with any selected layer (bottom to top). These are what the Move tool, Free Transform and Align/Distribute move.
- `toggle_disabled(doc, id)`: ⇧-clicking a linked layer's link icon in the Layers panel flips `Layer::link_disabled`. A disabled layer keeps its link number (it is still shown as linked, with a red ×) but moves on its own, and its partners move without it. Like Photoshop, no history is recorded; the flag is not saved to PSD.
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
- `a_disabled_link_moves_on_its_own`: with a disabled link, neither side is carried along by the other, `link_set` still lists both, and toggling again restores the link.

## Known limitations

- The disabled state is not written to PSD (Photoshop stores it in its layer records; reopened files come back enabled).
