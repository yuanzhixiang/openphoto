# dialogs/lock_layers.rs: Lock Layers dialog

## Responsibilities

The dialog for Layer › Lock Layers...: sets the locks of all selected layers (except the background). It can only be opened from the menu; in Photoshop ⌘/ toggles "Lock All" directly without opening the dialog (see LockLayers / ToggleLockAll in `commands.md`).

## Input and output

- `LockLayersDialog::new(locks)`: `locks` is the result of `layer_ops::selected_locks`; each item is checked only when it is on for all selected layers.
- `show(ctx)` returns `Outcome::Open` (still open), `Cancel` or `Apply(Locks)`. `lib.rs` applies it with `layer_ops::set_selected_locks` and, when something changed, records a "Lock Layers" history state (the name measured in Photoshop 2026, which differs from the panel button's "Lock Layer").

## Layout

Size 269 × 206 pt, measured pixel by pixel from Photoshop 2026 screenshots (2x); coordinates are relative to the dialog's top-left corner:

- The title bar is the same as in other dialogs (`common::frame`); the title "Lock Layers" is in the bold 14 pt system font.
- Five rows with row centers at y 60, 86, 112, 138, 174:
  - Small lock icons on the left (`ps_icons::paint_scaled`, about 0.45–0.5×): Transparency (27.75, 55.75), Pixels (27.75, 82.75), Position (27.75, 107.75), Prevent auto-nest (25, 134.5), All (28, 169.75), color `#f1f1f1`.
  - 12 pt checkboxes with left edge at x 52; labels 9.5 pt to the right of the box: Transparency, Image, Position, Prevent auto-nest, All.
- Labels and buttons use the panel font (Adobe Clean, approximated in this app by Source Sans 3): labels are `theme::body()` (regular 11.5 pt), 1 pt below the box's center; button labels are semibold 12 pt. This was determined by measuring glyph widths; this dialog does not use the system font.
- OK (179, 48)–(249, 72): the default button with keyboard focus (`common::ps_focused_button`: `#737373` background, white border, blue focus ring). Cancel (179, 84)–(249, 108): a normal button.

## Interaction

- When All is checked, the other four items appear checked and grayed out and cannot be changed; unchecking All restores them to their state before All was checked. On confirm, All is saved as a separate flag (`lock_all`) and the other four items are saved according to their own check states.
- Enter is OK, Esc is Cancel.
- While the dialog is open, other commands are unavailable (`AppState::modal_open`).

## Tests

- `ui_tests::lock_layers_dialog`: the menu item is disabled when only the background is selected; opening it checks the existing locks; clicking Image and Prevent auto-nest and then OK applies the locks and records "Lock Layers"; checking All and pressing Enter turns on all locks; Cancel changes nothing.
- `ui_tests::cmd_slash_toggles_lock_all`: ⌘/ on the background shows an alert; on a normal layer it does not open the dialog but turns on all locks directly ("Lock Layer"), and pressing it again clears all locks ("Unlock Layer").
- `ui_tests::screenshot_lock_layers_dialog` (ignored, run manually): a screenshot for comparison with Photoshop.
