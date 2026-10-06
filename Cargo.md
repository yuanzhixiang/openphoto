# Cargo 工作区（Cargo.toml）

## 职责

根 `Cargo.toml` 定义 Cargo 工作区：成员、共享的包元数据、统一的依赖版本，以及开发构建的优化配置。各成员 crate 通过 `xxx.workspace = true` 继承这些设置，不在自己的 `Cargo.toml` 中写版本号。

## 行为规则

### 工作区成员

- `resolver = "3"`。
- `members = ["app", "crates/*"]`：`app`（二进制 `openphoto`）以及 `crates/` 下的全部 crate——`op-core`、`op-color`、`op-io`、`op-tools`、`op-render`、`op-ui`。
- `default-members = ["app"]`：在根目录直接 `cargo build` / `cargo run` 只针对 `app`，其它 crate 作为它的依赖被编译；要单独构建或测试某个 crate 需显式用 `-p` 或 `--workspace`。

### 共享包元数据（`[workspace.package]`）

- `version = "0.1.0"`，`edition = "2024"`，`license = "MIT OR Apache-2.0"`。
- `rust-version = "1.95"`：这是 eframe/egui 0.36 要求的最低 Rust 版本，工作区的 MSRV 跟随它。实际编译使用的工具链由 `rust-toolchain.toml` 固定（见 `spec/rust-toolchain.md`）。

### 共享依赖版本（`[workspace.dependencies]`）

内部 crate 以 path 依赖登记：`op-core`、`op-render`、`op-tools`、`op-io`、`op-color`、`op-ui`。

第三方依赖：

| 依赖 | 版本 | 说明 |
| --- | --- | --- |
| `eframe` | 0.36 | 关闭默认特性，只启用 `wgpu`、`default_fonts`、`accesskit`：只用 wgpu 渲染（不编译 glow 后端），不启用持久化等其它默认特性 |
| `egui` | 0.36 | 与 eframe 同版本 |
| `egui-wgpu` | 0.36 | 画布通过它的 paint callback 嵌入 UI，与 egui 共享 wgpu 设备 |
| `egui_dock` | 0.21 | 文档标签页停靠，需与 egui 0.36 匹配 |
| `egui-phosphor` | 0.14 | 图标字体 |
| `wgpu` | 30 | 与 egui-wgpu 0.36 使用的 wgpu 主版本一致 |
| `bytemuck` | 1（`derive`） | GPU uniform 结构体的 `Pod` 转换 |
| `glam` | 0.30 | 向量数学 |
| `image` | 0.25 | 关闭默认特性，只启用 `png`、`jpeg`、`webp`、`tiff`、`bmp`、`gif` 六种格式 |
| `rfd` | 0.17 | 原生文件对话框 |
| `thiserror` | 2 | 错误类型 |
| `log` / `env_logger` | 0.4 / 0.11 | 日志门面与实现 |

egui 生态（`eframe`、`egui`、`egui-wgpu`、`egui_dock`、`egui-phosphor`）与 `wgpu` 的版本必须相互匹配，否则会出现两份不兼容的类型；`op-render` 通过 `egui_wgpu::wgpu` 重导出使用 wgpu，而不是直接依赖 `wgpu`。

### 开发构建优化（`[profile.dev]`）

- 工作区自身的 crate：`opt-level = 1`。
- 所有依赖（`[profile.dev.package."*"]`）：`opt-level = 3`。
- 原因：像素处理（合成、mip 生成、画笔等）在完全不优化的 debug 构建中太慢，无法正常交互。依赖完全优化、自身代码轻度优化，在保留可调试性和增量编译速度的同时让开发构建可用。

## 边界情况

- 新增 `crates/` 下的目录会自动成为工作区成员（通配符匹配）。
- 由于 `default-members` 只有 `app`，没有被 `app` 依赖链引用的 crate 在根目录的默认 `cargo build` 中不会被编译。

## 与其它模块的关系

- `rust-toolchain.toml` 固定实际使用的编译器版本，必须不低于此处的 `rust-version`。
- `Cargo.lock` 被提交到仓库（`.gitignore` 未忽略它），锁定的版本当前为 eframe/egui-wgpu 0.36.2、egui_dock 0.21.1、wgpu 30.0.1。

## 已知限制

- `wgpu` 和 `glam` 虽在 `[workspace.dependencies]` 中声明，但当前没有任何成员 crate 引用它们。
- 没有定义 `[profile.release]` 的自定义配置，release 构建使用 Cargo 默认值。
