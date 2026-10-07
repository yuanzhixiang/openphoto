# Rust toolchain (rust-toolchain.toml)

## Responsibility

Pins the Rust toolchain used by the project, so all developers and CI build with the same compiler version, and ensures the needed components are installed.

## Behavior rules

- `channel = "1.99.0"`: pinned exactly to 1.99.0 (the latest stable release as of 2026-09-28), rather than a floating `stable`. When `cargo` / `rustc` run in the repository directory, rustup automatically selects this version and installs it if needed.
- `components = ["rustfmt", "clippy"]`: the formatter and lint tool are installed along with the toolchain.

## Edge cases

- When 1.99.0 is not installed locally, the first cargo run inside the repository triggers rustup to download that toolchain.
- Setting the `RUSTUP_TOOLCHAIN` environment variable or using `cargo +<toolchain>` overrides this file's choice.

## Relationship to other modules

- `rust-version = "1.95"` in the root `Cargo.toml` is the workspace's minimum supported version (determined by eframe/egui 0.36); the 1.99.0 pinned by this file is higher, so it satisfies the requirement.
- Upgrading the version requires editing this file manually; it does not automatically follow new stable releases.

## Known limitations

- No `targets` or `profile` is specified; cross-compilation targets must be added manually.
