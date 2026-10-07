# app_kit.rs: A few AppKit calls (macOS only)

## Responsibilities

A few AppKit calls that winit and muda do not provide but that are needed to match Photoshop's behavior.

## Functions

- `hide_app()`: hides the application, equivalent to the standard "Hide" menu item. OpenPhoto's own Hide menu item uses Photoshop's ⌃⌘H, so it has to make the call itself.
- `option_held()`: whether Option is down right now (`NSEvent.modifierFlags`), for menu commands that change with it (adjustments opening with their last settings).
- `handling_key_press()`: whether the `NSApp.currentEvent` currently being handled is a key press (`NSEventTypeKeyDown`). Menu event handling uses it to tell "triggered by shortcut" apart from "chosen with the mouse": the Lock Layers... menu item shows ⌘/, but pressing ⌘/ should toggle Lock All instead of opening the dialog (see `commands.md`).

- `pen_pressure()`: what the event being handled (`NSApp.currentEvent`) says about the pen. A tablet event (a left mouse press or drag whose subtype is `NSEventSubtypeTabletPoint`, or a tablet point) gives `Some(Some(pressure))`. A plain mouse press or drag gives `Some(None)`. Any other event gives `None`, which leaves the last value alone. `subtype` and `pressure` are only asked of these events.
- `screen_ppi()`: the main display's pixel density: the native pixel width of its current mode (`CGDisplayModeGetPixelWidth`) over its physical width (`CGDisplayScreenSize`, mm). `None` when the display reports no size. Used by View › Actual Size (see `document_view.md`); it reads the main display even when the window is on another one.

## Constraints

- All of them may only be called on the main thread (menu events and the event loop are both on the main thread).
- They return safely when `NSApplication` cannot be found or there is no current event (`handling_key_press` returns `false`, i.e. the event is treated as a menu selection).
