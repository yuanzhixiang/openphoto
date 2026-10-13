# note_tool.rs
Click creates/selects a Note; dragging an existing marker moves it, release records changed positions, and Escape/tool switch cancels movement. Notes window stages author, multiline text and RGB color until Done/close or selecting another note; Escape discards staged edits. Delete removes the note with undo. Switching tools commits staged text. Canvas markers show with Note selected. Exact Photoshop Notes-panel styling, native keyboard/menu details remain unverified.

The toolbar color swatch edits a persisted session default, or the staged current note color; new notes use that default. Notes-panel toggle opens the selected/latest note or commits and closes the editor.

File save commits staged note edits before encoding.
