# scripts/package-macos.sh: the macOS release package

## Responsibilities

Builds the macOS download attached to GitHub releases: `dist/OpenPhoto-<version>-macos-universal.zip`, holding `OpenPhoto.app`. The version comes from `[workspace.package] version` in the root `Cargo.toml`. `dist/` is ignored by git.

## The app bundle

- **Universal binaries**: `openphoto` (the application) and `op-mcp` (the MCP server) are each built for `aarch64-apple-darwin` and `x86_64-apple-darwin` with `MACOSX_DEPLOYMENT_TARGET=11.0`, joined with `lipo` and stripped of local symbols (`strip -x`). The script adds both Rust targets with `rustup` if they are missing.
- **Layout**: `Contents/MacOS/openphoto` (the bundle's executable), `Contents/MacOS/op-mcp`, `Contents/Info.plist` from `packaging/macos/Info.plist` with `@VERSION@` replaced, and `Contents/Resources/OpenPhoto.icns`.
- **Info.plist**: bundle identifier `com.yuanzhixiang.openphoto`, category Graphics & Design, minimum macOS 11.0, high-resolution capable. It declares no document types: files reach the app through its command line or by dropping them on the window.
- **op-mcp inside the app**: shipping the MCP server in the bundle lets users run it from `/Applications/OpenPhoto.app/Contents/MacOS/op-mcp` once the app is installed, without a Rust toolchain. It is a separate command-line process; launching the app does not start it.

## Signing and notarization

- `op-mcp` is signed first, then the bundle around it, so the bundle's signature seals the signed helper. `codesign --verify --deep --strict` must pass.
- With `SIGN_IDENTITY` set (a Developer ID Application identity), both are signed with the hardened runtime and a secure timestamp, as notarization requires. Without it, both get an ad-hoc signature: enough to run on Apple Silicon, not to pass Gatekeeper on another Mac.
- With `NOTARY_PROFILE` set (a keychain profile created once with `xcrun notarytool store-credentials <name>`), the zip is submitted to Apple's notary service and the script waits for the verdict; the ticket is stapled to the app, `spctl --assess` must accept it, and the zip is rebuilt from the stapled app. The credentials stay in the keychain; the script only names the profile.
- The script ends by printing the zip's SHA-256, which the release notes quote.

## Releases

A release is published from a clean checkout of the commit that bumps the version, so the package holds exactly that commit's code: run the script with both variables, then attach the zip to a GitHub release whose tag points at that commit.
