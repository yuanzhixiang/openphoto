# remove.rs

Deterministic offline exemplar filling for the Remove canvas tool. This is a locally designed algorithm, not Adobe's Remove, Firefly, cloud model or a 1:1 reconstruction.

inpaint clones the source, fills a bounded removal region from original unmarked opaque 5x5 patches, choosing the most-known boundary window and lowest mean squared RGBA context cost, with Manhattan distance breaking ties. Candidate centers use a deterministic local grid expanded by 48 pixels. Marked source pixels never become exemplars. Completed patches feed subsequent boundary comparisons. The operation caps the region bounding box at 262144 pixels and work at 32000000 counted comparisons/visits; limits and missing context return errors without partial output.

apply rejects hidden/locked/non-raster/mask targets and dimension mismatches. It intersects the area with the document selection. Current or composited layers supply the source. Only changed marked pixels are written; new-layer output is transparent elsewhere and preserves source layers. Successful writes change the revision once. An unchanged result adds no layer and returns false. Undo/history is owned by the UI.

Tests reconstruct a checkerboard defect deterministically without changing known pixels or the source; full removal without context and locked targets reject atomically. Native synthesis, distraction detection, cloud/device models and exact alpha/blending parity remain unverified or unsupported.
