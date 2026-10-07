# tool_icons.rs: Toolbar Tool Icons

## Component Responsibilities

Vector icons drawn in-house in the style of Photoshop 2026's toolbar icons (not Adobe's icon assets): 1 pt (2 device pixel) lines and solid silhouettes, with position, size and shape measured one by one from 2x screenshots of Photoshop's toolbar. Every tool button in the toolbar (`toolbar.rs`) and the right-click tool flyout use it. The Remove Tool has no script ID in Photoshop that switches to it; its icon (the healing brush's bandage plus two four-pointed stars at the top left, knocked out around the stars) was measured from the right-click flyout of Photoshop's healing tool group (at 0.93×), and its `nudge` is (2, 2).

## Coordinates and Position

- Each icon is drawn in a 56 × 52 device pixel (2x) cell, `x` to the right and `y` downward; coordinates come directly from measurements of Photoshop's icons.
- `paint(painter, center, tool, color, bg)`: the cell's left edge is 29 px left of the button center and its top edge 25 px above the button center (rounded down); for rows where the button center falls exactly on a whole pixel (the toolbar's 25.9 pt pitch repeats every 5 rows, i.e. the 1st, 6th, 11th, 16th and 21st buttons), it moves down another 1 px. This rounding rule was derived by comparing all 22 of Photoshop's buttons one by one.
- `nudge(tool)`: for a few icons whose measured coordinates were off by 1–2 px, the amount to shift the whole icon when drawing.
- Icon centers are computed by the toolbar from Photoshop's row pitch (the first button's center 43 pt below the top of the toolbar, then every 25.9 pt), not taken from the button rectangle (avoiding drift from rounding button heights).
- `paint_scaled(…, scale)`: draws scaled down, used for the flyout (in Photoshop's list the Move tool icon is 14 pt wide, 15 pt in the toolbar, a ratio of 0.93).

## Visuals

- Icon color `#dddddd` (`COLOR`, Photoshop measurement); the interiors of the shape tools (rectangle, ellipse, triangle, polygon, custom shape) and the pencil are `#6f6f6f`, the patch tool's interior is 144 gray, the selection brush's base is `#757575`, the gradient icon's interior is a horizontal gradient from 47 to 202 gray, and the path selection tool's arrow interior is black.
- Parts that need to be "knocked out" (the bandage's four holes, the sponge's holes, the red eye's highlight, the gaps between the fingers of the smudge and burn tools) are drawn in the button background color `bg`: `TOOL_ACTIVE` when selected, `HOVER` when hovered, otherwise the panel background; in the flyout, the hovered row uses the accent color.
- Concave solid shapes are filled in one go with a triangle mesh (`fan`), avoiding seams left by anti-aliasing each triangle separately, and a thin line is then stroked along the outer edge for anti-aliasing.

## Known Differences

- The shapes are approximations redrawn from screenshots: line-style icons (lasso, pencil, eyedropper, custom shape, etc.) overlap Photoshop's bright pixels by about 0.5–0.7, and block-style icons (move, marquee, crop, type, patch, rectangle, etc.) by 0.9–1.0. The magnetic lasso's magnet, the shade of the selection brush's base, and the slice tool's blade differ most from Photoshop.

## Test Coverage

- `tool_icons_match_photoshops_extents` in `ui_tests.rs`: renders the icons of all 69 tools; each one's bright-pixel bounding box differs by at most 2 px from the bounding box measured in Photoshop (`PS_ICON_EXTENTS`, which stores only numbers).
- `compare_tool_icons_with_photoshop` (`#[ignore]`; requires the environment variable `PS_ICONS` to point at a directory of icons captured from Photoshop's toolbar; `ICONS` can be used to compare only some tools): prints each icon's bright-pixel overlap with Photoshop and the best offset, and outputs a top/bottom comparison image `target/ui-shots/icon_compare.png`. Photoshop screenshots are used only locally and are not committed to the repository.
