#!/bin/bash
# Builds OpenPhoto.app as a universal (Apple Silicon + Intel) binary and
# zips it for a GitHub release: dist/OpenPhoto-<version>-macos-universal.zip
# The MCP server ships inside it as Contents/MacOS/op-mcp, so MCP clients
# can run it from the installed app.
#
# SIGN_IDENTITY   a Developer ID Application identity to sign with
#                 (hardened runtime, timestamped); ad hoc without it
# NOTARY_PROFILE  a notarytool keychain profile, created once with
#                 `xcrun notarytool store-credentials <name>`; the app is
#                 then notarized and the ticket stapled
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
DIST=dist
APP="$DIST/OpenPhoto.app"
export MACOSX_DEPLOYMENT_TARGET=11.0

rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null
for target in aarch64-apple-darwin x86_64-apple-darwin; do
  cargo build --release --target "$target" -p openphoto -p op-mcp
done

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
for bin in openphoto op-mcp; do
  lipo -create -output "$APP/Contents/MacOS/$bin" \
    "target/aarch64-apple-darwin/release/$bin" \
    "target/x86_64-apple-darwin/release/$bin"
  strip -x "$APP/Contents/MacOS/$bin"
done
sed "s/@VERSION@/$VERSION/g" packaging/macos/Info.plist > "$APP/Contents/Info.plist"
cp packaging/macos/OpenPhoto.icns "$APP/Contents/Resources/"

# Sign the helper first, then the bundle around it
if [ -n "${SIGN_IDENTITY:-}" ]; then
  sign=(codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY")
else
  # Ad-hoc signature: enough to run on Apple Silicon, not to pass Gatekeeper
  sign=(codesign --force --sign -)
fi
"${sign[@]}" "$APP/Contents/MacOS/op-mcp"
"${sign[@]}" "$APP"
codesign --verify --deep --strict "$APP"

ZIP="$DIST/OpenPhoto-$VERSION-macos-universal.zip"
rm -f "$ZIP"
ditto -c -k --keepParent "$APP" "$ZIP"

if [ -n "${NOTARY_PROFILE:-}" ]; then
  xcrun notarytool submit "$ZIP" --keychain-profile "$NOTARY_PROFILE" --wait
  xcrun stapler staple "$APP"
  spctl --assess --type execute --verbose "$APP"
  rm -f "$ZIP"
  ditto -c -k --keepParent "$APP" "$ZIP"
fi
shasum -a 256 "$ZIP"
