# panels/properties.rs: Properties panel

## Component responsibilities

Shows the current document's properties, corresponding to the "Document" content Photoshop's Properties panel shows when no particular object is selected: four sections, Canvas, Rulers & Grids, Guides and Quick Actions.

Positioned point by point from measurements of Photoshop 2026 (coordinates are pt relative to the top-left of the panel content area), with icons traced as vector shapes from 2x screenshots. Label text is `#d6d6d6`, values and section titles are `#f0f0f0`, and text is aligned by the vertical center of its capital letters to the y values below.

1. Top: document icon box (10, 5)–(34, 29), `#383838` fill, 1 pt `#636363` border, containing a page icon with a folded corner (1 pt lines, `#d7d7d7`); "Document" at x 37.5, y 17.75. At y 33 is a 1 pt `#3e3e3e` separator.
2. Four collapsible sections, top to bottom: each section starts with a 1 pt `#3e3e3e` separator (the first at y 33), with the title 16.25 below the line: a V-shaped arrow centered at (11, line + 16.5) (a right-pointing > when collapsed) and a semibold title at x 21. Clicking around the title toggles expanded/collapsed; the state is kept in egui temporary memory, and all are expanded by default. Expanded heights (measured): Canvas 225.25, Rulers & Grids 68, Guides 68, Quick Actions 91; collapsed, only the title remains, 32 high. The y values below are positions when all sections are expanded; collapsing a section above moves everything up.
3. **Canvas** (title y 49.25):
   - Width/height link icon: centered at (23.25, 84.25), two vertical chain links with a short bar between them.
   - W, H: labels right-aligned to x 59.5; input fields (64, 64)–(118, 81) and (64, 88)–(118, 105), 1 pt `#666666` border, `#454545` fill, value (e.g. `734 px`) inset 4 from the left. Read-only appearance.
   - X, Y: labels right-aligned to x 138; input fields (142.5, …)–(196.5, …), disabled style (`#4d4d4d` fill, `#5e5e5e` border, `#6a6a6a` text), value `0 px`.
   - Orientation: portrait icon centered at (78, 128), landscape at (104, 128), both a portrait figure in a frame; the one matching the document's orientation (portrait when height ≥ width) has a 26 pt square selection box (styled like the document icon box). Clicking does nothing.
   - "Resolution: N pixels/inch" at x 64.5, y 156.75.
   - Mode: label right-aligned to x 59, y 183.25; color mode dropdown (65, 174)–(197, 193), listing all of Photoshop's modes, only RGB Color selectable. Bit depth dropdown (65, 200)–(197, 219): 8/16/32 Bits/Channel, only 8 Bits/Channel selectable.
   - Fill: label right-aligned to x 59, y 238.5; swatch (64.5, 229.25)–(82.5, 248.25), 1 pt `#363636` border, empty (transparent) when there is no background layer and white when there is one; dropdown (86.5, 229.25)–(196.5, 248.25) shows "Transparent" or "White", and the menu lists Photoshop's options (White, Black, Background Color, Transparent, Custom...), with only the current item selectable.
4. **Rulers & Grids** (separator 258.25, title 274.25): three 26 pt square icon buttons centered at x 23, 56, 89, y 302.75: rulers (an L shape with ticks) toggles `ViewOptions::rulers`; grid (3 × 3) toggles `ViewOptions::grid`; pixel grid (checkerboard) does nothing. Options that are on are drawn as pressed boxes (`#383838` fill, `#636363` border). Unit dropdown (119.5, 293.25)–(196.5, 312.25) shows Pixels, with the other units grayed out.
5. **Guides** (separator 326.25, title 342.25): icon buttons centered at x 23, 56, 89, y 371.25: show guides (four guides; dimmed `#989898` and not clickable when the document has no guides), lock guides (adds a lock, toggles `ViewOptions::lock_guides`), smart guides (guides, an arrow and a lightning bolt, toggles `ViewOptions::smart_guides`, pressed by default). The line style dropdown (119.5, 361.25)–(196.5, 380.25) shows a solid line, and only Lines is selectable in its menu.
6. **Quick Actions** (separator 394.25, title 410.5): four 90 × 24 buttons (`#454545` fill, 1 pt `#666666` border, centered text): Image Size (10.5, 426.75), Crop (106.5, 426.75), Trim (10.5, 456.75), Rotate (106.5, 456.75). As in Photoshop 2026: Image Size opens the Image Size dialog, Crop switches to the Crop tool, Trim opens the Trim dialog, Rotate opens Rotate Canvas (Image Rotation › Arbitrary...).
7. The content area's total height is 485.25 pt (all expanded); it scrolls when it exceeds the panel height.

When no document is open, the panel shows "No Properties" in the center.

## Known limitations

- Width/height and resolution cannot be edited here (in Photoshop they can be changed directly). Use Image › Canvas Size to change the canvas size.
- Color mode and bit depth cannot be converted.
- Photoshop's dropdown borders are thinner than this app's (about 0.5 pt).
- Ruler units, guide line style, the pixel grid and the other Fill options cannot be switched yet; the smart guides toggle is View › Show › Smart Guides (see `smart_guides.md`).

## Test coverage

- `properties_sections_toggle_views_and_run_quick_actions` in `ui_tests.rs`: after collapsing Canvas, clicking the rulers and grid buttons turns on rulers and grid; after collapsing Rulers & Grids, clicking lock guides; after collapsing Guides, clicking Crop switches to the Crop tool and clicking Trim opens the Trim dialog. `screenshot_properties_sections` (`#[ignore]`) captures the top, middle and bottom of the panel for side-by-side comparison with Photoshop.
