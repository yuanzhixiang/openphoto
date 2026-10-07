# dialogs/save_changes.rs: "Save changes?" confirmation

## Component responsibility

When closing a document with unsaved changes (or quitting, or closing the window), asks whether to save first. The answer is handled by `actions::answer_save_prompt` (see "Close" in `actions.md`).

## Layout and visuals

As in Photoshop 2026, it is a macOS alert, drawn directly with `Alert::choose` from `alert.rs` (sizes and colors in `alert.md`):

- An icon of a warning triangle plus the app badge.
- Body text (13 pt bold system font): "Save changes to the OpenPhoto document “document name” before closing?" (Photoshop says "Adobe Photoshop document").
- Three full-width buttons from top to bottom: Save (default button, blue), Don’t Save (curly apostrophe, matching Photoshop), Cancel.

Compared pixel by pixel with a 2x capture of Photoshop 2026's prompt (`ui_tests::screenshot_save_prompt`, `#[ignore]`):

- The alert is 260 × 276 pt in both, and the message lines are at the same heights.
- "Don’t Save" spans the same 98–162 pt at y 207.5–217.5.
- The gray buttons' labels are black in both.
- Remaining differences: the backdrop gray (Photoshop's alert material takes on what is behind it: `#acacac` over Photoshop's dark window there, `#b3b3b3` in the alerts `alert.md` was measured on), the app badge (OpenPhoto's own), and the default button, which is gray in the capture only because Photoshop's window was inactive.

## Interaction

- `show(ctx, title)` draws every frame and returns a `SaveChoice` once answered: `Save`, `DontSave`, `Cancel`.
- Click a button; Enter = Save; Esc = Cancel; ⌘D = Don’t Save (the macOS convention).
- It is modal while open (`AppState::modal_open()` is true); menu commands and shortcuts do not work, and keys such as ⌘D are left to it.

## Known limitations

- The text says "OpenPhoto document" rather than Photoshop's "Adobe Photoshop document".
