# path.rs: Editable Bezier geometry

Paths contain named collections of open or closed subpaths in document pixels. Knots retain incoming, anchor and outgoing points and the linked-control flag. They do not rasterize or modify layers. Translation moves all three coordinates together. Exact de Casteljau insertion divides a cubic without changing its geometry; invalid segment indexes or parameters return None. Adaptive subdivision supplies a bounded-depth display polyline; this display tolerance is an implementation choice, not native Photoshop rasterization evidence.

The geometry regression compares 101 points before/after a split and rejects invalid insertions. Native editing constraints, vector masks and shape layers are separate capabilities.

interpolating_subpath interpolates sampled anchors with symmetric centered tangents, preserving explicit corners and optional closure. It is an application spline and does not claim native Photoshop Freeform/Curvature fitting. The regression checks interpolation endpoints, tangent continuity and corner controls.

Smooth/corner intent survives a single-knot subpath even when both controls coincide with the anchor.
