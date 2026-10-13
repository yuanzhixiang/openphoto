# pen_tool.rs: Editable path-mode Pen

The Pen tool adds knots on primary press and records one history state on release. Clicks create corners; dragging creates symmetric linked controls. Shift constrains handle direction to multiples of 45 degrees; Alt moves the outgoing control independently. Clicking the first anchor within six screen points closes a subpath. Escape restores an active press transaction and ends continuation without discarding earlier committed knots. Undo/redo discard session continuation. Pixel layers and selections remain unchanged.

Selected paths display blue adaptive cubic polylines, anchor squares and control handles, mapped through document view rotation/flip. The display is clipped to the canvas viewport.

Only path creation is wired; the Shape/Pixels modes, path operation menus and rubber-band options remain unsupported. Freeform, Add/Delete Anchor Point, Convert Point and Path/Direct Selection share this model. Curvature remains separate. The path model preserves actual cubic controls, but native Photoshop interaction/visual parity remains unverified.

Add Anchor Point picks the nearest active-path cubic within six screen points and splits it exactly with de Casteljau. Existing endpoints and misses do nothing. One successful insertion records Add Anchor Point and preserves pixels and curve geometry. Hit search uses sampled localization and bounded refinement; native picking tolerance is unverified.

Delete Anchor Point removes an active-path anchor within six screen points, preserving neighboring controls and the closed flag. Empty subpaths and empty paths are removed; continuation is cleared. One deletion records history. Misses do nothing and pixel layers remain unchanged.

Convert Point clicks collapse a knot to a corner; anchor drags create symmetric linked handles with optional Shift 45-degree constraint. Dragging an existing control changes only that control and breaks linkage. Transactions commit one history state only on geometry changes; Escape/tool departure restores the original snapshot. Native tool cursor and exact gesture thresholds are unverified.

Path Selection picks complete paths through anchors/cubics, supports Shift multi-selection/toggle, moves every subpath/control by a single delta, constrains Shift drags to the dominant axis and Alt-duplicates selected paths. Escape/tool departure restores geometry; release records one changed transaction. Misses deselect. Rectangle selection, alignment/arrangement, path booleans and transform options remain unsupported.

Direct Selection picks anchors across paths and active-path handles. Shift toggles/adds anchor selection; dragging anchors translates their controls without moving neighboring knots. Linked handle rotation preserves the opposite handle length; Alt breaks linkage and changes only the picked control. Shift constrains movement to the dominant axis. Arrow keys nudge selected anchors by one document pixel, or ten with Shift. Changed release/nudge records history; Escape/tool departure restores pointer edits. Rectangle selection, node deletion/split/join and path transforms remain unsupported.

Freeform Pen records document-space pointer samples at least one pixel apart into editable cubic paths. Alt samples become corners; other samples use a centered-tangent interpolating spline. Escape/tool departure restores the pointer snapshot, clicks create no path, valid release records one state. Paths use the standard PSD codec. Magnetic mode is disabled at canvas dispatch and does not silently run ordinary Freeform. Native curve fitting, tolerance defaults, magnetic Freeform and Shape/Pixels modes remain unsupported; this spline is not 1:1 acceptance.
