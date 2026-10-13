# artboard_tool.rs

Artboard options read the shared settings map: dimensions, white/black/transparent background, and auto-nesting. Size presets set actual dimensions; portrait/landscape swap dimensions; Add creates an adjacent board. Selecting a board synchronizes dimension/background fields. Dimension/background edits update the selected board and record one Artboard Options entry. Alignment offers left/top edges for selected boards; gear toggles auto-nesting.

Canvas press chooses a resize edge/corner, a containing board to move, or empty space to create. Each update restores the initial snapshot/view before applying an absolute change. Eight resize handles change only the clipping rectangle; movement carries descendants. Shift creates square boards. Release records one action; Escape restores the entire original gesture. Borders, names and eight handles are painted using the view transform. Input is egui only and testable without an OS window.

Remaining parity work: exact Photoshop preset catalog, spacing, snapping, additional gear controls, name styling and native descriptor opening need live references; the implementation does not claim those as verified.

Four plus controls create adjacent boards with a 20-pixel gap and the selected board's dimensions/background. Option-drag copies the complete artboard block; zero-offset/cancel removes provisional copies. Shift constrains board moves to axes/45 degrees. These are offline-defined behaviors, with exact native modifiers/spacing still awaiting reference verification.

Canvas expansion at negative coordinates compensates the view origin, so the dragged board stays beneath its document-space pointer. Auto-nesting is applied to whole-layer Move release and cross-document copies; pixel selections and mask-only editing do not reparent layers.

Custom imported backgrounds synchronize as Other plus an RGB setting, so changing dimensions or orientation retains their color. The background swatch opens an egui RGB picker with OK/Cancel; the color is staged until OK, then one options history state is recorded. Exact Photoshop color-dialog styling remains unverified.
