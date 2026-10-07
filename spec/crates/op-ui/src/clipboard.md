# clipboard.rs: Application Clipboard

## Responsibilities

Holds the pixels produced by Cut/Copy/Copy Merged and exchanges images with the system clipboard: pixels copied in OpenPhoto can be pasted into other applications, and images copied in other applications (for example screenshots) can be pasted in. The pixel rules are in `op-core`'s `clipboard.md`.

## Public Interface

- `Clipboard::new(use_system)`: when `use_system` is true, connects to the system clipboard (arboard); if connecting fails, it behaves as not connected. `AppState` is not connected by default, and `OpenPhotoApp::new` replaces it with a connected version at startup; windowless tests stay unconnected and do not alter the user's clipboard.
- `set(clip)`: saves to the internal clipboard, also writes the image to the system clipboard, and records whether the write succeeded.
- `get()`: the content Paste will paste.
- `text()`: the text in the system clipboard, used when the menu's Paste is handed to a text field.

## Behavior Rules

How `get()` decides:

- Not connected to the system clipboard: returns the internal clipboard.
- The system clipboard has an image: if it is the one written by the last copy here (same size, and the average difference across all channels is no more than 3), returns the internal clipboard—it carries the source position, so pasting can put it back in place; otherwise it is treated as another application's image, and a `Clip` without a position is returned (centered on paste).
- The system clipboard has no image: if the last copy was written to the system clipboard, something else was copied elsewhere afterwards, so `None` is returned (nothing is pasted); if the write failed at the time, returns the internal clipboard.

A small difference is allowed in the comparison because after a round trip through the system's image format, semi-transparent pixels (premultiplied alpha) and color values may change slightly.

## Known Limitations

- Only the pixel image is written to the system clipboard, without a position; copying and pasting between two OpenPhoto processes centers the paste instead of keeping the original position.
- Text in the system clipboard is not pasted as a type layer.

## Test Coverage

- `internal_clipboard_keeps_the_position`: when not connected to the system clipboard, the retrieved content is exactly what was stored (including the position); `None` when nothing has been copied.
- `round_tripped_images_are_recognized`: slight differences are still recognized as the same image; a different size or too large a difference is not.
