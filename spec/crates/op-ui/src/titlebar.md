# titlebar.rs: Custom-drawn title bar (macOS)

## Component responsibility

On macOS the window hides the system title bar (content extends over the whole window), so this draws a title bar in the same color as the options bar, making it look like Photoshop's window. The traffic-light buttons are still drawn by the system. Other platforms use the system title bar and do not show this component.

## Visuals

- 40 reference pixels high, background `#535353`.
- "OpenPhoto" is shown centered (Photoshop shows "Adobe Photoshop 2026").

## Interaction

- Press and drag: moves the window (`ViewportCommand::StartDrag`).
- Double-click: toggles between maximized and restored.
