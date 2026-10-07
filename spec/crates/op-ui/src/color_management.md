# color_management.rs: Display color management (macOS)

## Responsibilities

Makes colors in the window be interpreted as sRGB and converted by macOS ColorSync to the display's profile before being shown, matching how Photoshop displays them. Compiled only on macOS (`#[cfg(target_os = "macos")]`).

## Background

On macOS, wgpu outputs frames through a `CAMetalLayer`. When the layer has no color space set, the system treats pixel values directly as the display's native color space (P3 on wide-gamut displays such as Display P3) without any conversion, so the same sRGB values look more saturated. Photoshop color-manages documents from the working space (sRGB) → the display profile, and its UI colors are also converted by the system, so the colors differ when the two are side by side.

With `colorspace = sRGB` set on the `CAMetalLayer`, the system converts sRGB pixels to the display's color space: colors within the sRGB gamut display correctly on any display.

## Public interface

- `use_srgb(window)`: takes the AppKit `NSView` of the winit window, recursively traverses all sublayers starting from its root layer, sets the `colorspace` of every `CAMetalLayer` to `kCGColorSpaceSRGB`, and logs (at `info` level) how many layers were tagged. Does nothing when the window handle is not an AppKit type, the view has no layer, or the system cannot create an sRGB color space.

## Behavior rules

- Called once in `OpenPhotoApp::new`, after the native menu is installed. By then eframe has created the wgpu surface and the Metal layer exists (on a normal launch the log says "tagged 1 Metal layer(s) as sRGB").
- Only changes the color space tag, not pixel values: the values of UI colors, document composites and exported files are all unchanged; only how they appear on wide-gamut displays changes.
- On a display that is itself sRGB, the display looks the same before and after tagging.

## Edge cases

- The traversal runs on the main thread (`OpenPhotoApp::new` is called on the main thread); reading `sublayers` depends on this.
- Windowless UI tests (`new_headless`) do not call this module.

## Known limitations

- Document colors are treated as sRGB; embedded ICC profiles in files are not read, and there is no Color Settings or Assign/Convert to Profile (see the gap list in `README.md`).
- The color space is set only once at startup. If wgpu later recreated the Metal layer (which does not currently happen), the new layer would not be tagged.
