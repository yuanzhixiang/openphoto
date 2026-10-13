# remove_tool.rs

Remove has a session-local marked area, independent of the document selection. Primary strokes add; Subtract or latched Alt removes from the pending area. Brush size is finite 1–5000 px and normalized pen pressure affects size when enabled. Pink coverage runs map through document rotation/flip into the canvas. Pixels and history stay unchanged while marks are pending.

Remove after each stroke commits on release; otherwise Enter or Apply commits the accumulated area. Reset discards all marks. Escape during a stroke restores its preceding pending area; Escape without a stroke clears the session. Successful changed output records one Remove Tool state; unchanged output clears marks without history. Errors retain marks for correction/retry and leave pixels unchanged. Target/selection/source/new-layer behavior comes from op_core::remove.

Auto and Generative AI off use the bounded deterministic offline exemplar backend. Generative AI on rejects without calling a substitute or external service. This backend is an own algorithm, not Photoshop 1:1 synthesis. Cloud/device model selection, distraction detection, native overlay/color, processing progress and exact pressure curves remain unsupported or unmeasured. Pending marks are not PSD data; applied raster output uses normal save/undo support.

Switching away from Remove discards pending marks without editing pixels or history. Marking rejects documents above 16777216 pixels before allocating its dense brush buffers.

Pending Remove strokes have local undo/redo snapshots; document history/pixels are unaffected. Mark history is bounded to 32 strokes and 16777216 snapshot pixels. Applied output uses ordinary document undo/redo.

Remove draws a pressure-scaled circular cursor outline and hides the arrow. Unimplemented distraction detection/additional-options controls are disabled rather than presenting working actions. Cursor/overlay native styling remains unmeasured.
