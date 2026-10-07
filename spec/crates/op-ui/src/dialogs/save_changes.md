# dialogs/save_changes.rs: "Save changes?" confirmation

## Component responsibility

When closing a document with unsaved changes (or quitting, or closing the window), asks whether to save first. The answer is handled by `actions::answer_save_prompt` (see "Close" in `actions.md`).

## Layout and visuals

As in Photoshop 2026, it is a macOS alert, drawn directly with `Alert::choose` from `alert.rs` (sizes and colors in `alert.md`):

- An icon of a warning triangle plus the app badge.
- Body text (13 pt bold system font): "Save changes to the OpenPhoto document “document name” before closing?" (Photoshop says "Adobe Photoshop document").
- Three full-width buttons from top to bottom: Save (default button, blue), Don’t Save (curly apostrophe, matching Photoshop), Cancel.

A side-by-side comparison with a Photoshop screenshot matches (in the Photoshop screenshot, the default button is gray because the window is inactive).

## Interaction

- `show(ctx, title)` draws every frame and returns a `SaveChoice` once answered: `Save`, `DontSave`, `Cancel`.
- Click a button; Enter = Save; Esc = Cancel; ⌘D = Don’t Save (the macOS convention).
- It is modal while open (`AppState::modal_open()` is true); menu commands and shortcuts do not work, and keys such as ⌘D are left to it.

## Known limitations

- The text says "OpenPhoto document" rather than Photoshop's "Adobe Photoshop document".
