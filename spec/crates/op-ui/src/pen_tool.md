# pen_tool.rs: Editable path-mode Pen

The Pen tool adds knots on primary press and records one history state on release. Clicks create corners; dragging creates symmetric linked controls. Shift constrains handle direction to multiples of 45 degrees; Alt moves the outgoing control independently. Clicking the first anchor within six screen points closes a subpath. Escape restores an active press transaction and ends continuation without discarding earlier committed knots. Undo/redo discard session continuation. Pixel layers and selections remain unchanged.

Selected paths display blue adaptive cubic polylines, anchor squares and control handles, mapped through document view rotation/flip. The display is clipped to the canvas viewport.

Only path creation is wired; the Shape/Pixels modes, path operation menus and rubber-band options remain unsupported. Freeform/Curvature/add/delete/convert and Path/Direct Selection are separate tools. The path model preserves actual cubic controls, but native Photoshop interaction/visual parity remains unverified.

Add Anchor Point picks the nearest active-path cubic within six screen points and splits it exactly with de Casteljau. Existing endpoints and misses do nothing. One successful insertion records Add Anchor Point and preserves pixels and curve geometry. Hit search uses sampled localization and bounded refinement; native picking tolerance is unverified.

Delete Anchor Point removes an active-path anchor within six screen points, preserving neighboring controls and the closed flag. Empty subpaths and empty paths are removed; continuation is cleared. One deletion records history. Misses do nothing and pixel layers remain unchanged.

Convert Point clicks collapse a knot to a corner; anchor drags create symmetric linked handles with optional Shift 45-degree constraint. Dragging an existing control changes only that control and breaks linkage. Transactions commit one history state only on geometry changes; Escape/tool departure restores the original snapshot. Native tool cursor and exact gesture thresholds are unverified.
