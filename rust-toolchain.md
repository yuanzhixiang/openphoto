# Rust 工具链（rust-toolchain.toml）

## 职责

固定项目使用的 Rust 工具链，使所有开发者和 CI 用同一个编译器版本构建，并保证需要的组件已安装。

## 行为规则

- `channel = "1.99.0"`：精确固定到 1.99.0（截至 2026-09-28 的最新稳定版），而不是浮动的 `stable`。rustup 在仓库目录中执行 `cargo` / `rustc` 时会自动选择并按需安装该版本。
- `components = ["rustfmt", "clippy"]`：随工具链一起安装格式化和 lint 工具。

## 边界情况

- 本地没有 1.99.0 时，首次在仓库内运行 cargo 会触发 rustup 下载该工具链。
- 设置了 `RUSTUP_TOOLCHAIN` 环境变量或使用 `cargo +<toolchain>` 时，会覆盖本文件的选择。

## 与其它模块的关系

- 根 `Cargo.toml` 中的 `rust-version = "1.95"` 是工作区的最低支持版本（由 eframe/egui 0.36 决定）；本文件固定的 1.99.0 高于它，满足要求。
- 版本升级需要手动修改本文件，不会自动跟随新的稳定版。

## 已知限制

- 没有指定 `targets` 或 `profile`，交叉编译目标需要自行添加。
