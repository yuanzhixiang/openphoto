# panels/floating.rs: floating panels

## Component responsibilities

Panels opened from the Window menu that are not docked in the right-hand panel column: Info, Navigator, Histogram. Each panel has its own frame and can be dragged.

## State

`AppState::floating` (`FloatingPanels`): whether each of the three panels is open. `toggle` toggles it. Positions are remembered by egui's Area (kept after dragging).

## Layout and visuals

- Frame: panel background color, 1 pt stroke in the divider color, shadow.
- Header 26 pt: tab bar background color, a 90 pt wide tab on the left (panel background color, panel name in semibold), and a close button on the right (×, brighter on hover) that closes the panel when clicked.
- The content area has 10 pt of margin on all sides. Sizes (excluding the header): Navigator 250 × 230, Histogram 250 × 150, Info 250 × 200, taller by `info::SAMPLER_ROW` for every row of two color samplers.
- Default position: to the left of the icon column, arranged in a column from top to bottom in the order Navigator, Histogram, Info, with 8 pt gaps; each panel's default position is fixed (regardless of whether other panels are open), so a panel opened later does not cover one opened earlier.

## Known limitations

- Panels cannot be docked into the panel column, collapsed to icons, or grouped with other panels into tab groups; there is no panel menu.
- The open state is not remembered after the application closes.
