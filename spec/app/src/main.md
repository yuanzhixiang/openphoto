# Application Entry Point (app/src/main.rs)

## Responsibilities

The `main` function of the `openphoto` binary: initializes logging, collects file paths passed on the command line, configures the native window, then starts `op_ui::OpenPhotoApp` with eframe. It contains no other business logic.

## Behavior Rules

### Logging

- Uses `env_logger` with a default filter level of `warn`; when the `RUST_LOG` environment variable is set, the environment variable takes precedence.

### Command-Line Arguments

- **All** arguments after the program name are treated as file paths to open (`args_os().skip(1)`), converted as-is to `PathBuf`, preserving non-UTF-8 paths; no options or flags are parsed.
- The path list is passed to `OpenPhotoApp::new(cc, files)`:
  - When the list is empty, a default document (`Untitled-N`, 1920×1080, white background) is created after startup.
  - When the list is non-empty, the files are opened one by one in order; if a file fails to open, an `error` log entry is recorded and an alert is shown, without affecting the other files and without falling back to creating a blank document.

### Window

- The window title and the eframe app name are both `OpenPhoto`.
- Initial content area size 1350×800 logical points, minimum 960×600 logical points, centered on screen at startup (`centered: true`).
- File drag and drop is enabled (`with_drag_and_drop(true)`).
- The renderer is fixed to `eframe::Renderer::Wgpu`; the canvas depends on egui-wgpu's paint callback, and `OpenPhotoApp::new` panics when it cannot get the wgpu render state.
- On macOS only: the full-size content view is enabled (`with_fullsize_content_view(true)`), and the system title bar and title text are hidden. Content extends to the top of the window, and the app draws the title bar area itself, in the same color as the options bar.

### Exit

- `main` returns `eframe::Result`; when eframe fails to start (for example, a wgpu device or window cannot be created), it exits with an error.

## Edge Cases

- A passed path that does not exist or has an unsupported format: handled by `op-ui`'s `open_paths`, which logs it and shows an alert; the window opens as usual, possibly without any document.
- Arguments like `--foo` are also treated as file paths and an attempt is made to open them.
- Non-macOS platforms use the system's default title bar.

## Relationship to Other Modules

- Depends on `op-ui` (`OpenPhotoApp`), `eframe`, `env_logger`; all interface, document, and rendering logic lives in `op-ui` and its downstream crates.
- `OpenPhotoApp::new` installs fonts and styles, calls `op_render::install`, installs the native macOS menu, and creates or opens documents according to `files`.
- eframe's `default-features` are turned off, and only `wgpu`, `default_fonts`, `accesskit` are enabled (see `spec/Cargo.md`), so eframe persistence is not enabled and the window size and position are not saved between launches.

## Known Limitations

- No command-line options (such as version or help) are supported; all arguments are treated as file paths.
- The window size and position revert to the defaults on every launch.
