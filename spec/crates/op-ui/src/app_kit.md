# app_kit.rs: A few AppKit calls (macOS only)

## Responsibilities

A few AppKit calls that winit and muda do not provide but that are needed to match Photoshop's behavior.

## Functions

- `hide_app()`: hides the application, equivalent to the standard "Hide" menu item. OpenPhoto's own Hide menu item uses Photoshop's ⌃⌘H, so it has to make the call itself.
- `handling_key_press()`: whether the `NSApp.currentEvent` currently being handled is a key press (`NSEventTypeKeyDown`). Menu event handling uses it to tell "triggered by shortcut" apart from "chosen with the mouse": the Lock Layers... menu item shows ⌘/, but pressing ⌘/ should toggle Lock All instead of opening the dialog (see `commands.md`).

## Constraints

- All of them may only be called on the main thread (menu events and the event loop are both on the main thread).
- They return safely when `NSApplication` cannot be found or there is no current event (`handling_key_press` returns `false`, i.e. the event is treated as a menu selection).
