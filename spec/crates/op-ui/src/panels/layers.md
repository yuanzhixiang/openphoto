# panels/layers.rs: Layers Panel

## Component Responsibilities

Displays and edits the current document's layers: selection, show/hide, blending mode, opacity, Fill, locks, new, delete.

## Layout (Top to Bottom)

Positioned point by point from Photoshop 2026 measurements; coordinates are in pt relative to the top-left of the panel's content area. Icons are all vector shapes traced from 2x screenshots (`ps_icons`). At y 31 and y 59 there is a 1 pt `#3e3e3e` divider line each.

1. **Filter row** (center y 18): the "Kind" box (3, 8.5)–(92, 27.5), containing a magnifying glass, "Kind" and a chevron; five filter icons for pixel, adjustment, type, shape and smart object, centered at x 110, 133.5, 157.75, 182, 207; at x 232 is the vertical filter toggle (off). Filtering is off, so the whole row is drawn in Photoshop's disabled colors (`#4d4d4d` box, `#5e5e5e` border, `#878787` text, `#989898` icons). Appearance only; there is no filtering function.
2. **Blending mode and opacity** (center y 46): the blending mode dropdown (3, 36.5)–(134.5, 55.5), grouped like Photoshop's menu with divider lines between groups; "Opacity:" right-aligned to x 177; the percentage box (180, 36.5)–(217.5, 55.5), immediately followed on the right by a small box with a chevron (up to x 232). The value is displayed left-aligned.
3. **Lock and Fill** (center y 73.5): "Lock:" at x 6; five buttons for lock transparent pixels, lock image pixels, lock position, prevent auto-nesting and lock all, centered at x 43.5, 67, 88, 110.5, 129; an active lock has a 20 pt dark background. When "lock all" is on, the other four buttons all show as on and clicking them does nothing. "Fill:" right-aligned to x 177, the percentage box (180, 64.5)–(217.5, 82.5) plus a small arrow box.
4. **Layer list**: starts at y 90 (with a 0.5 pt `#4a4a4a` line above), extending to the footer bar; from top to bottom it runs from the top layer to the bottom layer, scrollable, background `#4d4d4d`. The right 16 pt is the scrollbar track, always `#4a4a4a` (rows and row lines are not drawn into it). When rows overflow, the track holds a 10 pt wide `#696969` capsule-shaped thumb, 3 pt from both sides of the track and 2 pt from both ends; length = available length × visible height ÷ content height (content height excludes the extra 1 pt of the last row, matching Photoshop), minimum 20 pt; dragging the thumb scrolls. egui's built-in scrollbar is hidden and unused (it cannot leave space at the ends of the track), and the edges of the scroll area do not fade out (Photoshop has no fade).
5. **Footer bar**: 25 pt high, with a 1 pt `#3e3e3e` line at the top. Eight buttons, icon centers at 222.5, 196.5, 169.5, 139.75, 113.5, 86.5, 58.25, 30.25 from the panel's right edge, 12.5 from the top of the footer bar: a brush-like icon (new in Photoshop 2026, purpose unconfirmed, appearance only, always grayed out), link layers (enabled when layers can be linked or unlinked; clicking is the same as Layer › Link/Unlink Layers), layer style fx (grayed out for the Background layer or when multiple layers are selected; appearance only), add mask (grayed out when multiple layers are selected), new fill or adjustment layer (appearance only), new group, new layer, delete layer. Enabled icons are `#dddddd`, grayed out `#989898`; enabled buttons have a background on hover.

When the Background layer, or a layer in a Lock all group, is selected, the blending mode, opacity, Fill and lock buttons are drawn in Photoshop's disabled colors (not via egui's fading) and cannot be changed; in the latter case the fx and trash buttons are grayed out too (`Document::in_locked_group`).

## Layer Rows

- The size of the thumbnail frame (including its border) follows Photoshop 2026's algorithm (`thumb_size`, checked against measurements at 39 document sizes): s = 30 ÷ long side; each side takes ⌈side × s⌉ (computed in double precision) plus 2.5 pt. A square document gives 32.5 × 32.5, a 1:2 document 17.5 × 32.5; floating-point error in `(30 / long side) × long side` rounds a few long sides (such as 811, 812, 1622) up to 31, so the 734 × 811 reference document gives 30.5 × 33.5. The thumbnail is at x 34, 4 pt below the row top in the first row (3.5 pt in other rows, see below); transparent areas show a 2 pt white/`#cccccc` checkerboard, and the image has a 1 pt `#2e2e2e` inner border.
- A layer row's height is the thumbnail frame height plus 7.5 pt, with a 1 pt `#454545` line below it (the row line stops before the scrollbar track); the first row is 0.5 pt taller (added at the top), the last row's line is moved down 1 pt, with the list background in between. A square document has a row pitch of 41 pt (first row 41.5), the reference document 42 pt. Content within a row is laid out using the "first row" height, and other rows are shifted up 0.5 pt as a whole, so the thumbnail, name and eye sit in each row at the same position as in Photoshop (checked against screenshots with 4 aspect ratios and 3 rows). Group rows work the same way: 24 pt high, first row 0.5 pt taller.
- At the far left is the 29.5 pt wide eye column, its background the color of the layer's color label (Photoshop 2026 measurements: Red `#a34943`, Orange `#9c6524`, Yellow `#a58a2e`, Green `#668145`, Blue `#586e96`, Violet `#6d569b`, Gray `#6a6a6a`; `layer_color`), or the panel color `#535353` with no label; a 1 pt `#454545` vertical line on its right; when the layer is visible, the eye icon is drawn at x 15, 0.25 pt below half the row height; when not visible, it is blank.
- The rest of the row (up to the scrollbar position): `#6b6b6b` when selected, lighter on hover, otherwise the panel color.
- With a layer mask, the mask thumbnail (grayscale) is shown 10 pt to the right of the layer thumbnail, with a link icon in between; when the mask is disabled, a red cross is drawn over the thumbnail.
- When exactly one regular layer (not the Background layer) is selected, the thumbnail of the current editing target (pixels or mask) has a 1.5 pt white frame 0.5 pt outside it; when multiple layers are selected, none is drawn (Photoshop 2026 measurement).
- The layer name is 8 pt to the right of the last thumbnail, vertically centered, color `#f0f0f0`.
- Lock icon at the right end of the row (21.5 from the scrollbar position, vertically centered; Photoshop 2026 measurements, checked pixel by pixel against screenshots): a Lock all layer shows a solid lock (with keyhole, `LayerLockFull`); a partially locked layer and the Background layer show a hollow lock (a dot inside the frame, `LayerLock`), both `#dddddd`; a layer that has no locks of its own but sits in a group with locks shows a dimmer `#a6a6a6` lock (solid when the group is Lock all, otherwise hollow). When a lock icon is shown, the link icon is not shown.
- Link icon: layers linked to any selected layer (including the selected layer itself) show a link icon in the same position (the same as the footer bar's link icon); when only unlinked layers are selected, none is shown (the union of `link::link_set` of the selected layers, so disabled partners still show it; Photoshop 2026 measurement). ⇧-click on a linked row's link icon toggles the link's temporary disable (`link::toggle_disabled`, no history); a disabled link has a red `#e32b2b` × (1.5 pt strokes, ±4 pt) drawn over the icon.

## While cropping

While the Crop tool's box is being changed (`crop_tool::previewing`), the list shows only Photoshop 2026's temporary layer: one selected row named "Crop Preview" with the eye and the merged image as its thumbnail (laid out as a layer row), until the crop is committed or cancelled (`crop_preview_list`).

## Layer Groups

Per Photoshop 2026 measurements:
- The list shows only layers not hidden by a collapsed group (`visible_rows`); each level deeper shifts the row's thumbnail, name, etc. 16 pt right as a whole (the eye column does not move).
- Group rows are 24 pt high (plus a 1 pt divider), from the left: at x 38 the expand (V) / collapse (>) arrow (`#e0e0e0`), at x 46–60 the folder icon, the name starting at x 71; all shift right with indentation. Clicking the arrow collapses or expands (no history recorded); other interactions (selection, multi-select, rename, eye) are the same as for layer rows.
- Dragging a row onto the footer bar buttons below the list (Photoshop 2026 measurement, `drop_on_footer`): while dragging, the button under the pointer has the hover background; on release, if the dragged layer is not selected, it is first selected alone, then:
  - "New layer": duplicates the selected layers (`layer_ops::duplicate_selected`), recording "Duplicate Layer";
  - "New group": groups the selected layers, recording "Create Group from Layers";
  - "Add mask": when exactly one layer is selected, adds a mask to it (from the selection if there is one), recording "Add Layer Mask";
  - Trash: without asking, deletes the selected layers directly, groups together with their contents (unlike clicking the trash, which asks), recording "Delete Layer", or "Delete Group" when a group is dragged;
  - Other buttons: no response.
- A group with a layer mask (Photoshop 2026 measurement): the row grows to a layer row's height; the arrow and folder stay at their columns, centered on the row; the link icon is at x 71 and the mask thumbnail box at x 77 (4 pt from the row top, the layer thumbnail size), with the name 8 pt right of it. Clicking the mask thumbnail targets the group's mask (white frame around it), so painting and fills edit the mask; clicking the rest of the row targets the group.
- ⌥-click on a group's arrow: expands or collapses this group together with all groups inside it (Photoshop's behavior).
- When the active layer changes (or layers are added or removed), the list automatically scrolls to its row (`scroll_to_rect`), e.g. after switching layers with ⌥[ / ⌥] or creating a new layer.
- Dragging a selected row moves all selected layers together (`layer_ops::move_blocks`); dragging an unselected row moves only that row.
- Drag reordering finds the nearest between-row position by actual row positions; the drop rules are in `layer_ops::move_block` (a group moves together with its contents; dropping above a group's first row enters the group).
- When the current layer is a group, the blending mode dropdown gains "Pass Through" at the top.
- The footer's "New group" button directly creates "Group N" (recording "New Group"); ⌥-click opens the New Group dialog.

## Interactions

- Clicking the eye column: toggles visibility. Matching Photoshop's default settings, no history is recorded.
- Clicking elsewhere in a row: selects only that layer; ⌘-click adds it to or removes it from the selection (`Document::toggle_layer_selection`); ⇧-click selects all layers from the current layer to it (⇧-click on a mask thumbnail still disables/enables the mask). All selected rows are highlighted; the thumbnail's white frame is drawn only on the active layer. Clicking the mask thumbnail makes the mask the editing target; clicking elsewhere makes the pixels the target (`Document::mask_target`). No history is recorded.
- ⇧-click on the mask thumbnail: disables/enables the mask, recording "Disable Layer Mask" / "Enable Layer Mask".
- Blending mode: switching immediately records one "Blending Change" history entry.
- Opacity, Fill: nothing is recorded while dragging or typing; one "Opacity Change" or "Fill Opacity Change" is recorded at the end, so a single drag does not produce multiple history entries.
- The five lock buttons: each toggles its corresponding flag, recording a "Lock Layer" history entry (Photoshop 2026 measurement; both on and off use this name); clicking on the Background layer does nothing. "Lock all" is an independent flag: turning it off restores the previous individual locks as they were.
- When "Lock all" is on (Photoshop 2026 measurement): only the lock button shows as pressed; the other four buttons show as unpressed and dimmed and cannot be clicked (their flags are kept underneath); blending mode, Opacity, Fill and the "Fill:" text are likewise dimmed and cannot be changed.
- "Add a mask" button: enabled when the current layer can take a mask; with a selection, it builds the mask from the selection (Reveal Selection), otherwise all white (Reveal All), recording "Add Layer Mask", and the editing target switches to the mask.
- New layer (footer button): inserts a transparent layer above the current layer (`Document::insert_above_active`), named "Layer N" by `Document::next_layer_name` (matching Photoshop: after opening an image with transparency you get "Layer 0", and the new one is "Layer 1"), selects it, and records "New Layer". ⌥⇧⌘N also works. ⌥-clicking this button, pressing ⇧⌘N, or Layer › New › Layer... first opens the New Layer dialog (`dialogs/new_layer.md`), matching Photoshop.
- Delete layer: deletes all selected layers (`layer_ops::delete_selected`), selects the layer below them (or the bottom layer if none), and records "Delete Layer". The button is disabled when only one layer remains.
- Layer › Hide Layers (⌘,) also toggles the current layer's visibility, likewise without recording history.
- Dragging a layer row (Photoshop 2026 measurement): while dragging, the dragged row (except the eye column) becomes `#516291`; at the between-row position nearest the pointer, two `#60a4f8` horizontal lines indicate where it will go on release, one 0.5–1.5 pt below the row boundary and one 2–3.5 pt below, spanning the rows but not drawn into the scrollbar track on the right; on release the layer moves (`layer_ops::move_block`), recording "Layer Order". When the target position is not allowed (the Background layer itself cannot move, layers cannot be placed below the Background layer, or the position does not change), no lines are drawn and nothing happens on release.
- Double-clicking a layer name (to the right of the thumbnail): opens an input box at the name with the name fully selected. The input box follows Photoshop 2026: a borderless `#454545` box from 2 pt before the name to 5 pt after it (widening with input, not past the scrollbar track), 14.5 pt high, centered on the row center; selected text has a `#4374b3` background. Enter or clicking elsewhere confirms, Esc cancels; if the name changed and is not empty, the layer is renamed (leading and trailing whitespace removed), recording "Rename Layer". While typing, single-key shortcuts do not take effect.
- Double-clicking the Background layer's name: as in Photoshop, opens the Layer from Background version of the New Layer dialog (`dialogs/new_layer.md`); on confirm it is converted to a regular layer.
- Clicking the lock icon on the right of the Background layer: without a dialog, directly converts it to the regular layer "Layer 0" (`layer_ops::layer_from_background`), recording "Layer From Background", matching Photoshop.

## Background Layer

The Background layer's blending mode, opacity, lock and Fill controls are all grayed out, matching Photoshop.

## Known Limitations

- Prevent auto-nesting only stores state (there are no artboards or frames yet).

## Tests

- `layers_rows_and_scrollbar_match_photoshop`: a 200 × 200 document with three layers A/B/C, A and C linked and selected; after rendering, compared point by point against Photoshop 2026 screenshots at 2x device pixels: the y of row highlights and row lines, the start of the thumbnail border, and the extent of the scrollbar track and thumb.
- `thumbnails_are_sized_like_photoshop`: the thumbnail frames for 12 document sizes match Photoshop measurements (including the rounding up for 811, 812, 1622).
- `link_layers_from_the_panel_and_the_menu`: see `op-core`'s `link.md`; also checks the red × after disabling a link and that the disabled layer is no longer moved with its partners.
- `a_group_gets_a_mask_and_shows_it`: Reveal All on a group targets its mask; brushing black on it hides the group's content there while the rest stays white.
- `dragging_layers_onto_the_footer_buttons`: uses real drag events to drag rows onto the new layer, trash, new group, trash (group, no prompt) and mask buttons, checking layers and history names.
- `lock_icons_on_the_rows_match_photoshop`: the solid lock, hollow lock and dim lock of a layer in a group match Photoshop screenshots at 2x device pixels; a layer in the group cannot be deleted.
- `alt_click_opens_nested_groups_and_the_list_follows_the_active_layer`: ⌥-click expands/collapses nested groups; with enough layers to need scrolling, selecting the background scrolls its row into view (verified: the test fails when auto-scrolling is turned off).
- `dragging_several_selected_rows`: two non-adjacent selected rows dragged together above the background.
