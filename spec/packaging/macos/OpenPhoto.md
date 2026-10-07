# packaging/macos/OpenPhoto.icns: the app icon and logo

## Responsibilities

`OpenPhoto.icns` is the icon of OpenPhoto.app. `scripts/package-macos.sh` copies it to `Contents/Resources/OpenPhoto.icns`.

This one image is OpenPhoto's logo everywhere, so the brand looks the same wherever people meet it:

- the app icon in Finder, the Dock and Launchpad;
- the icon in the app's alerts, from `crates/op-ui/assets/app-icon.png` (see `spec/crates/op-ui/src/dialogs/alert.md`);
- the website openphoto.si: header and footer logo, favicon, Apple touch icon and the social preview image (in the sites repository, `apps/openphoto.si/public`).

The copies are exported from the `.icns`, never redrawn. When the icon changes, every copy is exported again.

## The image

On macOS's rounded-square icon shape: a dusk sky fading from deep purple at the top (`#2a1e44`) to pink near the horizon (`#af5370`), a pale yellow sun setting behind layered violet mountains, and a dark purple strip of ground. Like every macOS icon it sits on a 1024 × 1024 canvas with a transparent margin of 100 on each side, and the shape has a soft shadow.

It is OpenPhoto's own design. It does not reuse Adobe's icon, colors or letterforms.

## Exporting copies

```bash
iconutil -c iconset packaging/macos/OpenPhoto.icns -o /tmp/OpenPhoto.iconset
```

The iconset holds every size from 16 px to 1024 px. `crates/op-ui/assets/app-icon.png` is `icon_128x128@2x.png` (256 px).

## Previous approaches

- Alerts used to draw a dark green rounded square with a light green "Op", and the website had its own canvas-and-guides mark. Both were replaced by the app icon, so that there is only one logo.
- A crop-symbol mark (white crop brackets around a cyan-to-blue picture) was briefly tried as a replacement for this icon. The sunset icon was kept instead.
