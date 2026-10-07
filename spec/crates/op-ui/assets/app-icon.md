# crates/op-ui/assets/app-icon.png

The app icon at 256 px, exported from `packaging/macos/OpenPhoto.icns` (`icon_128x128@2x.png`) and bundled into the binary with `include_bytes!`, so alerts can show it without reading the app bundle. It must be exported again whenever the icon changes. See `spec/packaging/macos/OpenPhoto.md`.
