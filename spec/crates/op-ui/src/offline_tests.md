# offline_tests.rs

CPU-only canvas integration harness. An egui Context and AppState receive synthetic pointer/key events in the root Ui supplied by Context::run_ui. Document paint callbacks are built but are never prepared, submitted or rendered; there is no GPU device, native window, clipboard access or desktop input. Logical view coordinates come from the same document_view mapping used by the application. Single-key actions are dispatched before canvas input.

The single-row regression tests live dragging, combination, feather, Escape, undo and unchanged document pixels. It is called by ui_tests to retain the existing test entry name.

FullOutput texture deltas are intentionally cleared instead of uploaded, satisfying egui's output lifecycle contract without a renderer.
