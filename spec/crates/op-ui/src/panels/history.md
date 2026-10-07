# panels/history.rs: History flyout panel

## Component responsibilities

Displays and operates on the current document's history, corresponding to Photoshop's History panel. It flies out from the History button in the icon column (in Photoshop's default workspace, History is collapsed into the icon column); see `../lib.md` for positioning and collapse rules.

## Size and visuals

All sizes are points (pt) measured 1:1 in Photoshop 2026:

- Panel width 235, default height 155, minimum height 100, with a 1-pixel dark outer border `#333333` and a drop shadow.
- **Tab bar**: height 28, background `#424242`. Two tabs, "History" and "Comments", each with 9.5 padding on left and right, font size 12 semibold. Active tab background `#535353`, text `#eaeaea`; inactive tab text `#a8a8a8`, with a divider on its right. On the right, in order: the collapse button ">>" (33 from the right edge), a vertical divider (21 from the right edge), and the panel menu (9 from the right edge, appearance only).
- **Snapshot row**: height 42. At far left is the history brush source column (centered 11 from the left): the brush icon when the snapshot is the source, otherwise an empty 11 pt box; the thumbnail is 30 square (24 from the left) and shows the document as it was when just opened or created, with a checkerboard where transparent; the document name is 58 from the left and truncated with an ellipsis when too long. Below the row is a 2 pt dark divider.
- **State row**: height 22, with a `#474747` divider at the bottom of the row. From left to right: the history brush source box (6 from the left, 11 square; the brush icon instead when the state is the source), the operation icon (center 39 from the left, 14 pt), and the operation name (57 from the left, font size 12).
  - The current state row has background `#6b6b6b`.
  - States after the current state (undone) have icon and text `#868686`.
  - A hovered row has a lighter background.
- **Scrollbar**: appears when the content overflows; a 12-wide track `#4a4a4a` holding a 7-wide rounded thumb `#6e6e6e`. When a new state is added, the list scrolls to the bottom automatically.
- **Bottom button bar**: height 23; from right to left: Delete (36 from the right), Snapshot (65 from the right), New Document from Current State (98 from the right), icons 13 pt.
- **Height resize handle**: a 5 pt tall bar at the very bottom, background `#454545`, with a row of short vertical lines in the middle.

## Interaction

- Clicking the "History" or "Comments" tab switches content. The Comments page currently shows only placeholder text.
- Clicking ">>": collapses the panel.
- Clicking a row's source column (the left 22 pt of the snapshot row, the left 23 pt of a state row) makes that row the History Brush's source (`DocState::history_source`: the state's id, `None` for the snapshot) without jumping to it; the brush icon moves there. When the source state is discarded (deleted, or dropped past the 50-state limit, or undone states replaced by a new edit), the snapshot is the source again.
- Clicking the snapshot row: returns to the first state (the document as it was when opened or created).
- Clicking a state row: jumps to that state; later states become undone (dimmed) and can be clicked back to until a new operation occurs.
- Delete button: deletes the current state and all states after it, and returns to the previous state. Unavailable when the current state is the first state.
- Snapshot and New Document from Current State buttons: grayed out, with a tooltip on hover.
- Dragging the bottom handle: changes the panel height; the cursor is an up-down arrow. The height is kept for the rest of the current run.
- The panel width is fixed.

## Relationship to other parts

- State data comes from `DocState::history` (`op_core::History`), which keeps at most 50 entries; when exceeded, the earliest state other than the first is discarded.
- Undo / Redo / Toggle Last State in the Edit menu operate on the same history shown here, and the panel reflects them immediately.

## Known limitations

- All states use the same document icon; Photoshop shows different icons by operation type (e.g. "T" for type layers).
- The source marker in a state row reuses the snapshot row's brush icon; its exact look in Photoshop has not been measured.
- Snapshots cannot be created, and documents cannot be created from a state.
- The panel menu has no content; the Comments page has no content.
