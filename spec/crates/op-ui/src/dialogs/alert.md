# dialogs/alert.rs: macOS-style alert box

## Component responsibilities

Photoshop's error messages and confirmation questions are native macOS alerts (NSAlert). This component draws them as measured from two alerts in Photoshop 2026 ("Could not complete the Copy command because the selected area is empty." and Flatten Image's "Discard hidden layers?"), and is used for all error messages and questions needing confirmation in `AppState::alert`.

## Data

- `Alert { message, icon, cancel, dont_show_again, choices, ok_label }`: the message (a blank line in it shows as a blank line); the icon (`App`, the app icon, or `Caution`, a warning triangle plus a small app icon); whether there is a Cancel; the "Don’t show again" checkbox and whether it is checked (`None` means there is none); the stacked choices; the default button's label (`"OK"` unless renamed, e.g. "Full Screen" for Full Screen Mode's warning).
- `Alert::error(message)`: app icon, OK only. `Alert::caution(message)`: warning icon, Cancel and OK, with the checkbox.
- `Alert::choose(message, choices)`: warning icon, with the buttons as a column of full-width choices (the first is the blue default button), see below.
- `show(ctx, alert)` returns `Option<Answer>`: `Ok { dont_show_again }`, `Cancel` or `Choice(index)`; `None` while not yet answered.

## Layout (measured in Photoshop 2026, pt relative to the alert's top-left)

- Width 260, fill `#b3b3b3`, 0.5 pt `#ebebeb` border, corner radius 16, with a shadow; displayed centered, without dimming what is behind.
- App icon: 51 pt square starting at (26.5, 26.5). OpenPhoto's own icon (a dark green rounded square with a light green "Op"), not Photoshop's. Warning icon: a yellow triangle at (22, 24)–(80, 76) (white border, white exclamation mark), with a 27 pt small app icon at the lower right starting at (55, 55).
- Message: bold system font 13 pt (AppKit's opsz 17 and 13 pt tracking, see `theme.md`), color `#1c1c1c`, starting at x 22.5, wrapping when wider than 216; the capitals of the first line start at y 103, line height 16.
- Checkbox (when present): 13 pt below the message, 16 pt square with corner radius 4, `#9f9f9f` when unchecked, a white check on blue when checked; "Don’t show again" 7 pt to the right of the box, system font 13 pt (with tracking). Clicking the box or the text toggles it.
- Buttons: 13 pt below the last line of the message (16 pt below the checkbox when there is one), 28-high capsules, system font 13 pt (with tracking), the text 1 pt above the capsule's center. With OK only, OK spans (16, …)–(244, …); with Cancel, Cancel (16–126, `#a4a4a4` fill, dark text) and OK (134–244, `#3478f6` fill, white text). Colors darken when pressed.
- Vertical choices (when `choices` is non-empty): each button is 28 pt high, spans (16, …)–(244, …), with 34 pt spacing; the first is white text on blue, the rest on gray; matching Photoshop's "Group and Contents / Group Only / Cancel" when deleting a group.
- The height varies with the number of message lines: 16 pt is left below the buttons.

## Interaction

- Enter: OK (the first choice for vertical choices). Esc: the last choice for vertical choices, Cancel when there is a Cancel, otherwise OK. Clicking buttons works the same way.
- It is modal while shown; menus and shortcuts have no effect.

## Known limitations

- The warning triangle is a sharp-cornered polyline; macOS's is a rounded gradient shape.
- It is not a real NSAlert (so it can run in windowless tests and be drawn by egui like the other dialogs).

## Test coverage

- `constructors` (unit test): defaults of the two constructors.
- `flatten_asks_before_discarding_hidden_layers` (UI test): with hidden layers, Flatten Image asks first; after Esc cancels, both layers remain; after checking "Don’t show again" and clicking OK, the image flattens to one background layer, and later flattening no longer asks.
- `rename_layer_and_alerts` (UI test): the error alert closes with Enter (also covers Rename Layer..., see `commands.md`).
- `screenshot_alerts` (screenshot, `#[ignore]`): both alert types, for comparison with Photoshop screenshots.
