#!/bin/bash
# Builds OpenPhoto.app as a universal (Apple Silicon + Intel) binary and
# zips it for a GitHub release: dist/OpenPhoto-<version>-macos-universal.zip
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
DIST=dist
APP="$DIST/OpenPhoto.app"
export MACOSX_DEPLOYMENT_TARGET=11.0

rustup target add aarch64-apple-darwin x86_64-apple-darwin >/dev/null
for target in aarch64-apple-darwin x86_64-apple-darwin; do
  cargo build --release --target "$target"
done

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create -output "$APP/Contents/MacOS/openphoto" \
  target/aarch64-apple-darwin/release/openphoto \
  target/x86_64-apple-darwin/release/openphoto
strip -x "$APP/Contents/MacOS/openphoto"
sed "s/@VERSION@/$VERSION/g" packaging/macos/Info.plist > "$APP/Contents/Info.plist"
cp packaging/macos/OpenPhoto.icns "$APP/Contents/Resources/"

# Ad-hoc signature: required to run on Apple Silicon; not notarized
codesign --force --deep --sign - "$APP"

ZIP="$DIST/OpenPhoto-$VERSION-macos-universal.zip"
rm -f "$ZIP"
ditto -c -k --keepParent "$APP" "$ZIP"
shasum -a 256 "$ZIP"
