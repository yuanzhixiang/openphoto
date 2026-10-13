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

Slice CPU regression verifies all three drawing styles, exported dimensions/pixels, Escape cancellation, undo and untouched source pixels.

Guide-grid CPU test covers horizontal/vertical division, duplicate guides and repeated no-op creation.

Slice Select CPU regression covers moving, corner resizing, multi-selection/group movement, Escape, deletion, undo and unchanged image pixels.

Frame CPU regression verifies rectangle clipping, moving/undo, cancelled movement, elliptical clipping, synthetic file-drop replacement, no new document and undo. It writes/removes one temporary PNG; no native input or screenshot is used.

Frame resize/Move CPU regression verifies independent clipping geometry, retained child pixels, grouped Move geometry/clamping and cancellation.

Eyedropper CPU regression verifies alpha-weighted averaging, Alt background selection, transparent/outside no-op, no document/history changes, zero-size fallback and all three sample scopes.

Color Sampler CPU regression covers the ten-marker limit, movement, Alt deletion, Escape and tool-switch cancellation, no history and unchanged pixels.

Ruler CPU regression covers line metrics, Alt protractor/angle, endpoint cancellation, whole-line movement, constrained geometry, no pixel/history edits while measuring, Straighten Layer and undo.

Note CPU regression covers canvas creation, author, staged Unicode multiline text/color commit, marker movement/undo, cancelled dragging and unchanged image pixels.

A Note save regression writes a temporary PSD with pending editor text, reopens it, checks text and saved-state status, then removes the file.

Count CPU regression covers marker creation/movement/Alt deletion, cancelled additions, groups, visibility/style, undo without stale style reapplication, clear/delete/undo and unchanged image pixels.

Count name CPU regression verifies pending dirty state, save committing Unicode group names, PSD reopen, saved status and rename undo.

Frame sibling CPU regression creates multiple independently clipped containers and verifies that selecting existing content does not extract or nest it when another placeholder is drawn.

Patch cancellation regression covers drag state, busy state, release after Escape, unchanged selection/pixels/history without native input.

Clone Stamp canvas regression checks mask source sampling, destination coverage, untouched pixels, one history state and undo.

History Brush regression restores a mask pixel from a selected state on both mask targets, leaves image pixels unchanged and verifies undo.

Gradient regression checks pending busy state, Escape/release preserving pixels/history and a subsequent valid gradient with undo. The CPU harness key helper synthesizes key-down events without host input.

Shape cancellation regression exercises each supported shape tool, releasing after Escape, a subsequent valid shape and undo. It does not establish native vector-layer or rasterization parity.

Rotate View regression checks a quarter-turn drag, Escape while pressed, continued movement/release staying at zero, unchanged pixels and history.

Zoom cancellation regression covers scrubby scale/anchor restoration and box discard, post-Escape movement/release, unchanged document pixels and history.

Toolbar CPU frames construct the Tools panel directly, without a native window. The screen-mode regression clicks its real hit area, checks the warning before Full Screen, simulates confirmation, returns with F and preserves pixels/history.
