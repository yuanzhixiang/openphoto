# panels/color_panel.rs: Color and Swatches Panels

## Color Panel

### Component Responsibilities

Views and edits the foreground or background color, corresponding to the default appearance of Photoshop's Color panel (swatches on the left, saturation/brightness area in the middle, hue bar on the right).

### Layout

Per Photoshop 2026 measurements; coordinates are relative to the top-left of the panel's content area, in pt:

- Top left: the foreground swatch at (9, 10) and the background swatch at (19, 20), both 20 square. The foreground color is a 1 pt dark frame (`#363636`) plus the color; the background color is a 1 pt dark frame, a 1 pt white frame, then the color. The one being edited is drawn on top with an extra 1 pt light gray frame (`#8c8c8c`) outside it, and its dark frame becomes `#252525`.
- Middle: the saturation/brightness area, top-left at (45, 7), default size 224 × 94; horizontal is saturation (0 on the left, 1 on the right), vertical is brightness (1 at the top, 0 at the bottom), in the current hue.
- Right: a vertical hue bar, 19 wide, 17 from the right edge of the content area, the same height as the saturation/brightness area, with 17 between them; 360° at the top, 0° at the bottom (red–magenta–blue–cyan–green–yellow–red, matching Photoshop).
- When the group height changes, the bottom edges of the area and the hue bar always stay 18 from the bottom of the content area (minimum 40).

### Interactions

- Clicking the foreground or background swatch: switches the color being edited; clicking the one already being edited opens the corresponding Color Picker, matching Photoshop.
- Pressing or dragging in the saturation/brightness area: sets saturation and brightness.
- Pressing or dragging on the hue bar (hit area widened by about 7 pt on each side): sets the hue.
- Changes are written immediately to the foreground or background color in `AppState`.

### Indicators

- In the saturation/brightness area is a hollow ring of radius 5 and line width 1, black in light, low-saturation areas (brightness > 0.6 and saturation < 0.5) and white elsewhere, so it stays visible; the ring is clipped to the area (when the color is at a corner only a quarter circle is visible, matching Photoshop).
- To the left of the hue bar is a white teardrop-shaped pointer with a 1 pt black edge: an arc of radius 4 on the left, with its tip touching the hue bar on the right.

### Edge States

When the color is changed elsewhere (eyedropper, swatches, swapping colors), the panel recomputes HSB from the color. To keep the hue from jumping back to 0 for grays, it recomputes only when the color differs from the color corresponding to the cached HSB.

### Rendering

The saturation/brightness area and the hue bar are drawn with vertex colors on subdivided meshes (32×32 and 36 segments) to keep the interpolation accurate.

## Swatches Panel

- Shows `AppState::swatches`: initially 36 swatches (6 grays plus a number of colors); the Color Picker's "Add to Swatches" appends to the end. 24 square, spaced 4 apart, wrapping automatically to the panel width.
- A hovered swatch has a white outline; clicking sets that color as the color currently being edited (foreground or background).

## Known Limitations

- The Color panel has only this one display mode; it lacks the RGB sliders, color wheel and other modes from Photoshop's panel menu.
- Swatches can only be appended through the Color Picker; they cannot be deleted, renamed, grouped or imported, and are not saved for the next launch.
