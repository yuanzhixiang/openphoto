# Cargo workspace (Cargo.toml)

## Responsibilities

The root `Cargo.toml` defines the Cargo workspace: members, shared package metadata, unified dependency versions, and the optimization settings for development builds. Member crates inherit these settings through `xxx.workspace = true` and do not write version numbers in their own `Cargo.toml`.

## Behavior rules

### Workspace members

- `resolver = "3"`.
- `members = ["app", "crates/*"]`: `app` (binary `openphoto`) and all crates under `crates/` — `op-core`, `op-color`, `op-io`, `op-tools`, `op-render`, `op-ui`, and the standalone binary crate `op-mcp` (the MCP server, see `crates/op-mcp/src/main.md`).
- `default-members = ["app"]`: running `cargo build` / `cargo run` directly in the root directory targets only `app`, with other crates compiled as its dependencies; building or testing a single crate on its own requires explicitly using `-p` or `--workspace`.

### Shared package metadata (`[workspace.package]`)

- `version = "0.1.0"`, `edition = "2024"`, `license = "MIT OR Apache-2.0"`.
- `rust-version = "1.95"`: this is the minimum Rust version required by eframe/egui 0.36, and the workspace's MSRV follows it. The toolchain actually used for compilation is pinned by `rust-toolchain.toml` (see `spec/rust-toolchain.md`).

### Shared dependency versions (`[workspace.dependencies]`)

Internal crates are registered as path dependencies: `op-core`, `op-render`, `op-tools`, `op-io`, `op-color`, `op-ui`.

Third-party dependencies:

| Dependency | Version | Notes |
| --- | --- | --- |
| `eframe` | 0.36 | Default features off, only `wgpu`, `default_fonts`, `accesskit` enabled: renders only with wgpu (the glow backend is not compiled), and other default features such as persistence are not enabled |
| `egui` | 0.36 | Same version as eframe |
| `egui-wgpu` | 0.36 | The canvas is embedded in the UI through its paint callback, sharing the wgpu device with egui |
| `egui-phosphor` | 0.14 | Icon font |
| `wgpu` | 30 | Matches the wgpu major version used by egui-wgpu 0.36 |
| `bytemuck` | 1 (`derive`) | `Pod` conversion for GPU uniform structs |
| `glam` | 0.30 | Vector math |
| `image` | 0.25 | Default features off, only the six formats `png`, `jpeg`, `webp`, `tiff`, `bmp`, `gif` enabled |
| `rfd` | 0.17 | Native file dialogs |
| `thiserror` | 2 | Error types |
| `log` / `env_logger` | 0.4 / 0.11 | Logging facade and implementation |
| `serde_json` | 1 | `op-mcp` reads and writes JSON-RPC messages |

The versions of the egui ecosystem (`eframe`, `egui`, `egui-wgpu`, `egui-phosphor`) and `wgpu` must match each other, otherwise two incompatible copies of the types appear; `op-render` uses wgpu through the `egui_wgpu::wgpu` re-export rather than depending on `wgpu` directly.

### Development build optimization (`[profile.dev]`)

- The workspace's own crates: `opt-level = 1`.
- All dependencies (`[profile.dev.package."*"]`): `opt-level = 3`.
- Reason: pixel processing (compositing, mip generation, brushes, etc.) is too slow in a completely unoptimized debug build for normal interaction. Fully optimizing dependencies and lightly optimizing the project's own code keeps debuggability and incremental compile speed while making the development build usable.

## Edge cases

- New directories added under `crates/` automatically become workspace members (wildcard match).
- Because `default-members` contains only `app`, crates not referenced by `app`'s dependency chain are not compiled by the default `cargo build` in the root directory.

## Relationship to other modules

- `rust-toolchain.toml` pins the compiler version actually used, which must be no lower than the `rust-version` here.
- `Cargo.lock` is committed to the repository (`.gitignore` does not ignore it); the locked versions are currently eframe/egui-wgpu 0.36.2 and wgpu 30.0.1.

## Known limitations

- Although `wgpu` and `glam` are declared in `[workspace.dependencies]`, no member crate currently references them.
- No custom `[profile.release]` configuration is defined; release builds use Cargo's defaults.
