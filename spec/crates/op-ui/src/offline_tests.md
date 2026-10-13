# offline_tests.rs

CPU-only canvas integration harness. An egui Context and AppState receive synthetic pointer/key events in the root Ui supplied by Context::run_ui. Document paint callbacks are built but are never prepared, submitted or rendered; there is no GPU device, native window, clipboard access or desktop input. Logical view coordinates come from the same document_view mapping used by the application. Single-key actions are dispatched before canvas input.

The single-row regression tests live dragging, combination, feather, Escape, undo and unchanged document pixels. It is called by ui_tests to retain the existing test entry name.

FullOutput texture deltas are intentionally cleared instead of uploaded, satisfying egui's output lifecycle contract without a renderer.

`single_column_drag_feather_cancel_nudge_and_undo_cpu` exercises full-height drag, combination, keyboard movement, feather, cancellation and undo without rendering.

Selection Brush CPU regression covers live selection, cancellation, zero opacity, Alt subtraction, unchanged pixels and undo.

The Lasso regression covers degenerate paths, polygon-shaped freehand closure, pointer-down mode latching, cancellation, addition, untouched pixels and undo.

Polygonal Lasso CPU tests check constrained vertices, Backspace removing one corner, Enter closing, Escape preserving selection/history and undo with unchanged pixels.

Magnetic Lasso CPU tests verify auto/manual anchors, anchor deletion, closure, initial mode latching, cancellation, undo and untouched pixels.

Object Selection CPU regression tests freehand ROI selection on a contrasting object, excluded background, cancellation, undo and pixel preservation.

Quick Selection CPU regression checks live segmentation, cancel, New-to-Add transition, no-op suppression, Alt subtraction, undo, unchanged pixels and invalid size fallback.

Magic Wand CPU regression checks average versus point sampling, pointer-down Add, no-op history, undo and pixel preservation.

Crop CPU regression verifies resolution-only commit/history/undo, Escape preserving pixels/dimensions and invalid infinite inputs.

Perspective Crop CPU regression covers crossing-box rejection, valid drawn-box commit, typed dimensions/resolution, undo and non-finite physical-unit input.
