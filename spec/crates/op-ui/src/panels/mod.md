# panels/mod.rs: Right panel column and icon column

## Component responsibility

- Right panel column: three tabbed panel groups stacked vertically, corresponding to the panels on the right of Photoshop's default workspace.
- Icon column: a vertical column of icons to the left of the panel column, corresponding to Photoshop panels collapsed to icons (History, Comments).

## Panel column

### Layout

All sizes below are measured from Photoshop 2026's default Essentials workspace (measured pixel by pixel from 2x screenshots), in pt. The panel column is 322 wide.

- At the top is a 13 pt high collapse bar (the same as the toolbar and icon column, see `header` in `toolbar.md`), with a thin "»" at the right end (appearance only).
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
- Adjustments, Libraries, Channels, Paths: show the placeholder text "{name} — not implemented yet".

## Icon column

- 43 pt wide: a 3 pt divider on each side (`#383838`, `#474747`, `#383838`); the left one is directly against the document's vertical scrollbar and the right one directly against the panel column; the 37 pt in between is the icon area.
- At the top is a 13 pt collapse bar, with a thin "«" at the right end (appearance only).
- Below the collapse bar (16 pt from the top of the column) is the drag handle: 10 short 1 × 4 pt `#454545` vertical lines spaced 2 pt apart, starting 9 pt from the left of the icon area (appearance only).
- History button: icon center 35.5 pt from the top of the column, an icon traced from Photoshop (two hollow small squares stacked on a solid small square, with a curved arrow on the right); clicking opens or closes the History flyout panel (see `history.md`), and while it is open the button is shown in its selected state.
- Comments button: icon center 63.5 pt from the top of the column, a solid speech bubble; appearance only.
- At 81 pt from the top of the column is a 1 pt `#383838` line spanning the icon area.
- Returns the position of the History button, for positioning the flyout panel.

## Known limitations

- Panels cannot be dragged to re-dock, dragged out into floating windows, or collapsed to icons.
- The panel menu (the three lines at the top right) has no content.
