# pen_tool.rs: Editable path-mode Pen

The Pen tool adds knots on primary press and records one history state on release. Clicks create corners; dragging creates symmetric linked controls. Shift constrains handle direction to multiples of 45 degrees; Alt moves the outgoing control independently. Clicking the first anchor within six screen points closes a subpath. Escape restores an active press transaction and ends continuation without discarding earlier committed knots. Undo/redo discard session continuation. Pixel layers and selections remain unchanged.

Selected paths display blue adaptive cubic polylines, anchor squares and control handles, mapped through document view rotation/flip. The display is clipped to the canvas viewport.

Only path creation is wired; the Shape/Pixels modes, path operation menus and rubber-band options remain unsupported. Freeform/Curvature/add/delete/convert and Path/Direct Selection are separate tools. The path model preserves actual cubic controls, but native Photoshop interaction/visual parity remains unverified.
