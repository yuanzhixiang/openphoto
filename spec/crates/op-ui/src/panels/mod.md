# panels/mod.rs: Right panel column and icon column

## Component responsibility

- Right panel column: three tabbed panel groups stacked vertically, corresponding to the panels on the right of Photoshop's default workspace.
- Icon column: a vertical column of icons to the left of the panel column, corresponding to Photoshop panels collapsed to icons (History, Comments).

## Panel column

### Layout

All sizes below are measured from Photoshop 2026's default Essentials workspace (measured pixel by pixel from 2x screenshots), in pt. The panel column is 322 wide.

- At the top is a 13 pt high collapse bar (the same as the toolbar and icon column, see `header` in `toolbar.md`), with a thin "»" at the right end. Clicking it collapses the column to icons (`AppState::panels_collapsed`, below).
- The three panel groups follow directly after the collapse bar, with a 3 pt divider between groups (three 1 pt lines: `#383838`, `#474747`, `#383838`):
  1. Color, Swatches, Gradients, Patterns, default height 147 (including the tab bar).
  2. Properties, Adjustments, Libraries, taking the remaining space (the flexible group).
  3. Layers, Channels, Paths, default height 286.
  In a 1350 × 800 window, the three groups' tab bars start at y 75, 225, and 514 respectively, the same as Photoshop.
- At the top of each group is a tab bar: 27 pt high, background `#424242`, with a 1 pt `#383838` line below it (28 pt total); at the right end is the panel menu icon: four 10 × 1 pt `#a8a8a8` horizontal lines spaced 2 pt apart, 5.5 pt from the right edge and 10 pt from the top of the tab bar (appearance only). Below is the panel content, background `#535353`.

### Tabs

- Tab width is the text width plus 19 (9 left of the text, about 10 to the right), rounded to a whole point; to the right of each tab is a 1 pt `#383838` vertical line; the text is semibold.
- The active tab has background `#535353` and covers the line below the tab bar so it joins the content; text `#f0f0f0`. Inactive tabs are the same color as the tab bar, with text `#b0b0b0`, which becomes `#f0f0f0` on hover.
- Clicking switches the panel shown in that group.

### Resizing

Dragging the divider between groups changes the heights (the cursor becomes an up-down arrow). When dragging a divider next to the flexible group, it is the fixed-height group on the other side that changes, and the flexible group absorbs the difference automatically. Each group's minimum height is the tab bar plus 40.

### Panels

- Color, Swatches: see `color_panel.md`.
- Properties: see `properties.md`.
- Layers: see `layers.md`.
- Gradients, Patterns: see `presets.md`.
- Channels: see `channels.md`.
- Adjustments, Libraries, Paths: show the placeholder text "{name} — not implemented yet".

## Icon column

- 43 pt wide: a 3 pt divider on each side (`#383838`, `#474747`, `#383838`); the left one is directly against the document's vertical scrollbar and the right one directly against the panel column; the 37 pt in between is the icon area.
- At the top is a 13 pt collapse bar, with a thin "«" at the right end; clicking it opens a collapsed panel column again.
- Below the collapse bar (16 pt from the top of the column) is the drag handle: 10 short 1 × 4 pt `#454545` vertical lines spaced 2 pt apart, starting 9 pt from the left of the icon area (appearance only).
- History button: icon center 35.5 pt from the top of the column, an icon traced from Photoshop (two hollow small squares stacked on a solid small square, with a curved arrow on the right); clicking opens or closes the History flyout panel (see `history.md`), and while it is open the button is shown in its selected state.
- Comments button: icon center 63.5 pt from the top of the column, a solid speech bubble; appearance only.
- At 81 pt from the top of the column is a 1 pt `#383838` line spanning the icon area.
- Returns the position of the History button, for positioning the flyout panel.

## Collapsing to icons

- While the column is collapsed (`AppState::panels_collapsed`), `lib.rs` doesn't show it. The icon column (`Panels::icon_strip`) then lists a 24 pt button for every panel of every group, 26 pt apart and 6 pt more between groups, starting 90 pt down the column. Each button shows the panel's first two letters, where Photoshop draws its icon, with the panel's name as its tooltip.
- Clicking a button opens that panel beside the icons (`Panels::flyout`, `show_flyout`): a 322 × 320 pt frame left of the icon column, 16 pt down, with a tab bar naming the panel above its contents. Clicking the button again, or anywhere outside the frame and the icons, closes it.

## Floating and docking

- **Tearing off:** dragging a tab and letting go more than 30 pt left of the column takes the panel out of its group. It then floats (`Panels::floating`) with its top-left where it was let go (less 45 × 13 pt), and the group shows its nearest tab. A group left without tabs is drawn empty.
- **A floating panel** (`show_floating`) is a 322 × 300 pt frame:
  - a tab bar with its name and ×, above its contents;
  - dragging the tab bar moves it;
  - letting go over a group's tab bar (`bars`, gathered each frame) docks it there as the group's active tab;
  - × docks it into the first group.

## Workspace

Window › Workspace lists Photoshop's items:

- **Essentials (Default):** checked, as the only workspace.
- **3D, Graphic and Web, Motion, Painting, Photography:** grayed out.
- **Reset Essentials:** sets `AppState::reset_workspace`, and `lib.rs` then puts back the default column (`Panels::default`). The panels open again, nothing is collapsed, the floating Window panels close, and hidden panels and tools show. Choosing Essentials (Default) does the same.
- **New Workspace..., Delete Workspace..., Keyboard Shortcuts & Menus..., Lock Workspace:** grayed out.

## Known limitations

- Panels dock only as tabs of an existing group (no new groups or columns, no docking beside the toolbar). The collapsed buttons show initials, not Photoshop's icons, and the floating frames have a fixed size. The layout is not saved across launches.
- The panel menu (the three lines at the top right) has no content.
