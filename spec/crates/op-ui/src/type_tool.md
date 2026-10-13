# type_tool.rs: Type and Type Mask canvas interaction

## Component responsibilities

Input, live preview, commit, and cancel for Horizontal/Vertical Type and both Type Mask tools (T) on the canvas. For rasterization, see `op-core`'s `text.md`.

## State

Text in progress is stored in `DocState::text_edit` (`TextEdit`): the start of the first line's baseline `origin` (document pixels), the text typed so far, the document snapshot from before it started `before`, and the text currently shown on the document `shown`. Text options are in `AppState::type_options` (`TypeOptions`: Regular / Semibold, font size, default 12 pt).

## Interaction

- Clicking the canvas: when there is no text in progress, places the insertion point (baseline start) at the click and starts input; when there is text in progress, commits it (matching Photoshop when clicking elsewhere).
- While typing (`AppState::typing_text()` is true, and `modal_open()` is also true), menu commands and single-key tool shortcuts do not take effect, and typed characters go into the text:
  - Characters: appended to the end; ⌫: deletes the last character; Enter: new line.
  - Esc: cancels, and the document is restored to before it started.
  - ⌘Enter, the ✓ in the options bar, clicking elsewhere on the canvas, or switching to another tool: commits. When the text is empty (or only whitespace), nothing is left behind; otherwise the text layer is kept and "Type Tool" is recorded.
  - The ⦸ in the options bar: cancels.
  - When an input field (such as the font size field in the options bar) has focus, typed content does not go into the text on the canvas.
- Preview: when text, font, size or color changes, the snapshot from before it started is restored, and the text layer is then generated with the current options (foreground color, font size converted to pixels: `pt × resolution / 72`), so what is seen on the document is the final result. The preview records no history.
- Cursor: after the last character a blinking (every 0.5 seconds) insertion line is drawn, 3 pt black plus 1 pt white outside, visible on any background. The mouse cursor over the canvas is the text cursor.

## Known limitations

- The insertion point cannot be moved into the middle of the text, text cannot be selected or moved with the arrow keys; committed text layers cannot be edited.
- Only Source Sans 3 Regular and Semibold (the fonts bundled with the application) are available; there is no system font list.

Horizontal Type Mask uses glyph coverage as a selection and creates no layer. Shift adds, Alt subtracts and both intersect, latched when editing starts. Preview restores the original snapshot before recomputing; Escape and empty/no-change commits restore it. A changed commit records Type Mask. Font/size/color changes refresh previews even with unchanged text. The red editing overlay covers unselected pixels proportionally, mapped through rotation/flip; it is display-only. Native glyph rasterization, overlay color/opacity and UI parity remain unmeasured.

Vertical Type shares input/preview/cancel/commit with Horizontal Type, with upright glyph cells and a horizontal caret. Orientation changes invalidate the preview cache. Committed type remains raster; editable type metadata/native glyph parity are still pending.

Vertical Type Mask shares selection operations and cancellation with Horizontal Type Mask and uses upright vertical glyph coverage. Switching text orientation retains the active edit and regenerates its preview without an intermediate history state. Native vertical typography remains unverified.
