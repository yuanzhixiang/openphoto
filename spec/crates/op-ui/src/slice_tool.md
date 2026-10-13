# slice_tool.rs
Slice input creates non-destructive regions in Normal, Fixed Aspect Ratio or Fixed Size styles. A valid image press starts an absolute preview, release clips it to the canvas and records one New Slice state. Escape and switching away cancel preview. Slice/Select tools show numbered blue outlines and selected borders. Auto slices and options dialogs are not provided. Values must be finite and positive.

Slice Select hit-tests user slices in front-to-back order. Click selects, Shift-click toggles multi-selection, selected edge/corner handles resize one region, and interior dragging moves selected regions together while clamping their union to the canvas. Previews derive from original bounds; Escape/tool switching restores the initial snapshot, release records only changed bounds, Delete/Backspace deletes selected regions in one undoable state. Selected regions display eight handles. Arrange/align/divide/promote and auto-slice controls remain unavailable.

Reading slice drawing options does not initialize stored width/height values: Normal retains disabled empty fields until a fixed style is selected.
