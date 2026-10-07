# native_popup.rs: native menus for dialog dropdowns

## Responsibility

Photoshop's AppKit-drawn dialogs open native macOS menus from their pop-up buttons (light and translucent in light mode, the current item checked and laid over the button's value). This module lets a dialog dropdown ask for such a menu and get the pick back, without the dialog knowing about AppKit.

## Interface

- `Entry`: `Item { label, checked, enabled, shortcut }` or `Separator`. `Entry::item(label, checked)`, then `.enabled(on)` and `.shortcut("⌥1")`.
- `dropdown(ui, response, id, entries) -> Option<usize>`: call with the dropdown button's response every frame. When native menus are available and the button is clicked, it stores a `Request { id, entries, button }` in the egui context; when the pick for `id` arrives, it returns its index (separators count) once. Without native menus (other platforms, UI tests) it opens the same entries as an egui menu (`Button::selectable`, disabled items grayed, shortcuts at the right) and returns a click directly.
- `set_available(ctx, on)` / `available(ctx)`: set by `lib.rs` once the window's NSView is known.
- `take_request(ctx)`: the menu asked for this frame. `deliver(ctx, id, index)`: hands a pick back (a pick for another `id` is left for its own dropdown).
- `Request::positioning()`: the checked entry (else the first), the one macOS places over the button.

## How the app serves a request (`lib.rs`, `menu.rs`)

After the frame, `lib.rs` takes the request and calls `NativeMenu::popup_menu(request, scale)`, `scale` turning egui points into the view's points. It builds a muda menu of check items (ids `popup:<index>`, key equivalents from shortcuts like "⌥1"), sets the NSMenu's minimum width to the button's, its appearance to the application's (the system's light or dark look, not the window's forced dark one, matching Photoshop), and opens it with `popUpMenuPositioningItem:atLocation:inView:` with the checked item at the button's left −13.5 pt and top −4.5 pt (Photoshop 2026's menus open there). The call returns when the menu closes; the menu event handler sends `popup:<index>` picks through their own channel, and `lib.rs` delivers the pick to the requesting dropdown, which returns it on the next frame.

## Used by

`appkit::popup` (Levels, Curves, Fill, Fade, Gradient Map, the filter dialogs' choices, Auto Resolution, Delete Preset) and `common::field_popup` (Image Size), plus Image Size's Dimensions box and gear menu.

## Known Limitations

- macOS lays rows out at its own height (24 pt); some of Photoshop's menus (Image Size's Fit To) are denser.

## Test Coverage

- `a_native_pick_comes_back_to_its_dropdown`: a request carries the entries and positions on the checked one; a pick for another id is not returned; the dropdown's pick is returned once.
- UI tests drive the egui fallback through the same entries (`image_size_presets`, `image_size_auto_resolution`, and the dialog tests that pick from these menus).
