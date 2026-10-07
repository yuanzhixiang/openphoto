# toolbar.rs: Left Toolbar

## Component Responsibilities

The vertical toolbar on the left side of the window, corresponding to Photoshop's Tools panel (single-column layout).

## Layout (Top to Bottom)

The toolbar is 42 pt wide; the right 3 pt is the divider against the canvas, and tool buttons are centered in the remaining width.


1. A 13 pt collapse bar at the top: a 1 pt `#383838` line above and below, `#424242` in between; on the left is a bold pixel-aligned "»" (`#c8c8c8`, traced from Photoshop), appearance only. The collapse bar's `header(ui, Collapse)` is shared by the toolbar, the icon column (thin "«") and the panel column (thin "»"). Below it is the drag handle: 10 short 1 × 4 pt `#454545` vertical lines, 2 pt apart, starting at x 10, their tops 16 pt from the top of the toolbar (appearance only).
2. Tool buttons; each button is a tool group (grouping and order in `crates/op-tools/src/lib.md`) and shows the most recently used tool in that group (initially the first tool in the group). Cells are evenly spaced with no gaps between groups: a pitch of 25.9 pt, with the first cell's center 43 pt below the top of the toolbar, matching Photoshop 2026 measurements.
3. The widgets below the tools are absolutely positioned at Photoshop 2026's positions (traced from 2x screenshots; coordinates below are in pt relative to the toolbar's top-left):
   - Edit Toolbar ("•••"): three dots of diameter 4, spaced 6 apart, centered at (19.25, 613.75), with a small gray triangle (`#bcbcbc`) at the bottom right; appearance only.
   - Default colors icon: two 9 pt small squares, black at the top left and white with a black border at the bottom right, surrounded by a light gray `#c3c3c3` edge, at (3, 632)–(15, 644).
   - Swap colors icon: a quarter-circle arc with an arrow at each end (pointing left and down), at (21.5, 631.5)–(34, 643).
   - Foreground color swatch (4, 648)–(24, 668), background color swatch (14, 658)–(34, 678), foreground on top.
   - Quick Mask button: a ring of 12 small square dots inside a rounded rectangle (9, 689)–(27, 702); shown pressed while in Quick Mask.
   - Change Screen Mode button: two overlapping window shapes (the top-right one in front), with a small gray triangle at the bottom right; appearance only.
   Icon color `#dddddd`; the buttons' hover hit areas are the same size as the tool buttons'.
4. The 3 pt divider on the right (`#383838`, `#474747`, `#383838`), against the canvas.

The Quick Mask button ("Edit in Quick Mask Mode (Q)") enters/exits Quick Mask for the current document when clicked (`toggle_quick_mask`, recording "Quick Mask").

## Tool Buttons

- 30.5 pt wide and the pitch minus 1 pixel high, horizontally centered in the toolbar. The current tool has a `#383838` dark background with a 1 pixel `#606060` outer stroke (corner radius 4); lighter on hover.
- When a group has multiple tools, a small triangle is drawn at the button's bottom right.
- The hover tooltip is "Tool name (shortcut)", e.g. "Move Tool (V)".
- Left click: selects the tool shown on the button.
- Right click, or holding the button for 0.4 seconds (`HOLD_SECONDS`, Photoshop's press-and-hold flyout): a list of the group's tools pops up to the right of the button (when opened by holding, the list stays open after releasing the mouse, and that release does not count as a click or switch tools); each row is an icon, a name and a shortcut; the currently shown tool has a small square marker in front of it; the hovered row is highlighted in the accent color. Clicking a row selects that tool, makes this cell show it from then on, and closes the list.
- The list's size and position follow Photoshop 2026 measurements (`flyout_metrics`, in pt): a 1 pt `#3e3e3e` border, the background the same as the panel, with a downward shadow; each row is 19 high; the current tool's 4 pt square is at x 7; icon center at x 28.5 (`tool_icons` draws at 0.93×; the Move tool icon is about 14 pt wide); names start at x 41, panel font 11.5 pt ("Move Tool" is 47.5 wide in Photoshop's Adobe Clean, 48.7 in Source Sans 3); the shortcut's right end is 7 from the border's right edge, at least 11 from the longest name. The width is computed from the content; the Move group is 130 × 40. The border's top-left corner is 1.5 further right of the button's right edge, flush with the button's top edge; that is, the Move tool's list starts at window (36, 92).
- Icons use Phosphor Bold glyphs (`theme::tool_icon`, 17.5 pt), closer than Regular to Photoshop's heavier tool icons; the mapping is in `icons.md`.

## Foreground/Background Colors

- The foreground color is at the top left and the background color at the bottom right, stacked offset from each other; both are 20 pt square with a 1 pt `#363636` dark frame, and the background color also has a 1 pt white frame inside the dark frame (`swatch`, matching Photoshop).
- The small icon at the top left: restores default colors (foreground black, background white), the same as pressing D.
- The small icon at the top right: swaps the foreground and background colors, the same as pressing X.
- Clicking the foreground or background swatch: opens the corresponding Color Picker (see `dialogs/color_picker.md`), and also switches the Color panel to editing that color.

## Known Limitations

- The toolbar cannot collapse into two columns and cannot be dragged.

## Tool Icons

The icons in the buttons are drawn in Photoshop's style by `tool_icons.rs` (see `tool_icons.md`), with icon centers computed from Photoshop's row pitch; tools for which `paint` returns false (currently none) fall back to Phosphor font icons.
