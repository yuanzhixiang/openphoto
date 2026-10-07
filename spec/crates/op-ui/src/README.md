# op-ui: Overall UI layer specification

`op-ui` is OpenPhoto's UI layer, built on egui 0.36 (eframe + wgpu). It is only responsible for turning user input into operations on `op-core` documents and drawing the state; pixel data, layers and history all live in `op-core`.

## Product goal: a 1:1 replica of Photoshop

The UI follows Photoshop 2026's default "Medium Gray" theme, and layout, sizes, colors, copy, menu structure and shortcuts all match Photoshop. When judging whether some piece of UI is "right", the running Photoshop is the only reference, not personal taste or egui's default style.

Parts that are not replicated:

- Adobe's icons, the Adobe Clean font, the logo and the "Photoshop" name are not used. Icons use Phosphor (MIT); the UI font is Source Sans 3, open-sourced by Adobe under the OFL (bundled with the crate in `assets/fonts/`).
- The application name is displayed as "OpenPhoto".

## Sizing system

The UI has two sources of sizes, both converted into egui logical units:

1. **Reference screenshot pixels**: the sizes of the earlier UI (options bar, toolbar, right-side panels, Canvas Size dialog, etc.) were measured from a screenshot of the Photoshop window (the screenshot is 2000 px wide, corresponding to a 1349 pt Photoshop window). These values are written directly in code and then scaled as a whole to Photoshop's actual size via `theme::UI_SCALE = 0.675` using egui's `zoom_factor`.
2. **Photoshop points (pt)**: later UI is measured 1:1 directly in the running Photoshop (the Photoshop window and OpenPhoto's default window are both 1350×800 pt), converted in code with `theme::pt(x)`, where `pt(x) = x / UI_SCALE`. The History panel uses this system.

All new UI uses system 2: measure in Photoshop, write with `pt()`.

`UI_SCALE` affects only the UI, not the canvas: canvas zoom is always in physical pixels (100% = one image pixel per screen pixel).

## Colors

All colors are defined in `theme::color`, sampled from Photoshop screenshots. The main ones: panel `#535353`, pasteboard outside the canvas `#282828`, tab bar `#424242`, input box `#454545`, selected row `#6b6b6b`, selected tool `#383838`, dark divider `#393939`, body text `#eaeaea`. Colors specific to individual components are defined in the component files (e.g. the History panel's scrollbar track).

## Menus and shortcuts

- Shortcuts match Photoshop 2026's default shortcuts; the data comes from Photoshop's menus read through the accessibility interface. The mapping between commands and shortcuts is centralized in `commands.rs`.
- On macOS the native menu bar is used (`menu.rs`), and ⌘ shortcuts are handled by the menu; other platforms have no native menu, and egui handles the same set of shortcuts.
- Single-key shortcuts (tools, D, X) are handled by `actions::handle_tool_keys` and do not fire while a text input has focus.
- While a modal dialog is open, all menu commands are disabled and single-key shortcuts do not fire, consistent with Photoshop.

## Layout

The window, from outside in: title bar (macOS only), options bar, left toolbar, right panel column, the icon column to the left of the panel column, and the document area in the middle (document tab bar and current document). See `lib.md` for details.

## File index

- `lib.rs`: application entry, overall layout, document tabs, overlays (History flyout panel, Canvas Size dialog, error messages).
- `state.rs`: application state and per-document state.
- `commands.rs`, `menu.rs`, `actions.rs`: commands, menus, file operations and shortcuts.
- `document_view.rs`: canvas and status bar.
- `titlebar.rs`, `options_bar.rs`, `toolbar.rs`: the fixed areas at the top and left of the window.
- `panels/`: right-side panels and the History flyout panel.
- `dialogs/`: modal dialogs.
- `theme.rs`, `icons.rs`, `widgets.rs`: styles, icon mapping, common small widgets.
