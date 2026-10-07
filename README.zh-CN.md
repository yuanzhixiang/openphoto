<div align="center">

# OpenPhoto

**用 Rust 编写、与 Photoshop 用法一致的开源图像编辑器。**

布局、菜单、快捷键与对话框逐点对齐 Photoshop 2026，<br>
图像算法用 Photoshop 自己的输出逐级校准。

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#许可证)
[![Rust](https://img.shields.io/badge/rust-1.99-orange.svg?logo=rust)](rust-toolchain.toml)
[![Platform](https://img.shields.io/badge/platform-macOS-lightgrey.svg?logo=apple)](#快速开始)
[![Status](https://img.shields.io/badge/status-early%20development-yellow.svg)](#路线图)

[English](README.md) · 简体中文

<img src="docs/images/hero.png" alt="OpenPhoto 编辑一个分层的 Photoshop 文档" width="100%">

</div>

## 为什么做 OpenPhoto

无数人已经把 Photoshop 用成了肌肉记忆：每个面板在哪、每个快捷键做什么、每个对话框怎么响应。OpenPhoto 的目标是做一个免费的编辑器，让这些习惯原封不动地延续下来。

- **像素级一致的界面**：窗口、面板、选项栏和对话框都在运行中的 Photoshop 2026 上按 1:1 量取并逐一并排比对，细到输入框尺寸、滑块刻度和字距。
- **肌肉记忆直接可用**：菜单、菜单项顺序与默认快捷键都和 Photoshop 一致。Photoshop 记住的设置这里也记住，Photoshop 不记的这里也不记。
- **校准过的算法**：调整与滤镜用探测图拟合 Photoshop 的实际输出，逐个卷积核、逐张查找表还原。从 Photoshop 采集的对照数据随仓库提交，每次改动都会测试。
- **原生且快速**：纯 Rust，画布由 GPU（wgpu）合成，像素按块存储并写时复制，没有运行时依赖。

<div align="center">
<img src="docs/images/curves-comparison.png" alt="Photoshop 2026 与 OpenPhoto 的 Curves 对话框并排" width="100%">
<br><sub>Curves 对话框：左为 Photoshop 2026，右为 OpenPhoto。</sub>
</div>

## 功能

> OpenPhoto 处于早期开发阶段。下面列出的是现在已经可用的功能，尚缺的部分见[路线图](#路线图)。

**界面**
- 每个工具的选项栏、工具栏图标、Properties 面板与各对话框都按在 Photoshop 2026 上量取的位置排布，大多误差不超过 1 pt
- Photoshop 2026 的 New Document 对话框，含预设分类、最近使用与已存储的预设

**文档与文件**
- 打开与保存分层的 **PSD**（图层、图层组、混合模式、不透明度与填充、蒙版、锁定、链接、颜色标签），已与 Photoshop 保存的文件逐字段核对
- 打开 PNG、JPEG、WebP、TIFF、BMP、GIF，导出 PNG、JPEG
- 多文档标签页、无限撤销与 History 面板、拖放打开

**图层**
- Photoshop 全部 27 种混合模式、不透明度与填充不透明度、支持穿透模式的图层组
- 图层蒙版、五种锁定、链接、颜色标签、合并与拼合
- 多选、拖动排序、对齐与分布

**选择与变换**
- 矩形、椭圆、单行、单列选框，套索、多边形套索、魔棒
- Select › Modify（边界、平滑、扩展、收缩、羽化）、变换选区
- 自由变换（含斜切、扭曲、透视与变形），Image Size、Canvas Size、旋转画布，带 Photoshop 叠加线与拉直功能的裁剪

**绘画与修饰**
- 画笔、铅笔、橡皮擦、渐变、油漆桶、仿制图章、历史记录画笔
- 模糊、锐化、减淡、加深、海绵；形状与文字；吸管与 Photoshop 式拾色器

**调整**：全部对话框按 Photoshop 2026 重做

Levels · Curves · Brightness/Contrast · Exposure · Vibrance · Hue/Saturation · Color Balance · Black & White · Photo Filter · Channel Mixer · Selective Color · Gradient Map · Threshold · Posterize · Invert · Desaturate · Equalize · Auto Tone / Contrast / Color

**滤镜**：与 Photoshop 的输出对照，大多相差不超过 1 级

Gaussian Blur · Box Blur · Surface Blur · Motion Blur · Blur / Blur More · Unsharp Mask · Sharpen / More / Edges · Add Noise · Despeckle · Dust & Scratches · Median · Minimum · Maximum · High Pass · Offset · Custom · Mosaic · Fragment · Twirl · Pinch · Spherize · Polar Coordinates · Emboss · Find Edges · Solarize · Trace Contour · Wind

<table>
  <tr>
    <td width="50%"><img src="docs/images/huesat.png" alt="Hue/Saturation 对话框"></td>
    <td width="50%"><img src="docs/images/gauss.png" alt="带实时预览的 Gaussian Blur 对话框"></td>
  </tr>
  <tr>
    <td align="center"><sub>Hue/Saturation</sub></td>
    <td align="center"><sub>Gaussian Blur 与实时预览</sub></td>
  </tr>
  <tr>
    <td colspan="2"><img src="docs/images/new-document.png" alt="New Document 对话框"></td>
  </tr>
  <tr>
    <td colspan="2" align="center"><sub>New Document</sub></td>
  </tr>
</table>

## 快速开始

**下载**：在 [Releases](https://github.com/yuanzhixiang/openphoto/releases/latest) 下载已签名并经苹果公证的 macOS 版本（macOS 11 及以上，支持 Apple Silicon 与 Intel）。

从源码构建（OpenPhoto 在 macOS 上开发和测试，并使用 macOS 原生菜单栏）：

```bash
git clone https://github.com/yuanzhixiang/openphoto.git
cd openphoto
cargo run --release -- path/to/image.psd
```

Rust 工具链版本固定在 `rust-toolchain.toml` 中，rustup 会自动安装。不带文件参数时打开一个空白文档；也可以把文件拖到窗口上打开。

### 快捷键

沿用 Photoshop 的默认快捷键。最常用的几个：

| 操作 | 快捷键 |
|---|---|
| 新建 · 打开 · 存储 · 存储为 | ⌘N · ⌘O · ⌘S · ⇧⌘S |
| 撤销 · 重做 · 切换最终状态 | ⌘Z · ⇧⌘Z · ⌥⌘Z |
| 自由变换 | ⌘T |
| 色阶 · 曲线 · 色相/饱和度 | ⌘L · ⌘M · ⌘U |
| 新建图层 · 编组 · 向下合并 | ⇧⌘N · ⌘G · ⌘E |
| 放大 · 缩小 · 适合屏幕 · 100% | ⌘= · ⌘- · ⌘0 · ⌘1 |
| 工具 | V M L W C I B S Y E G O T U H Z |
| 默认颜色 · 交换颜色 | D · X |

## 架构

OpenPhoto 是一个 Cargo workspace。文档模型与界面完全分离，所有编辑操作都可以脱离窗口测试。

| Crate | 职责 |
|---|---|
| [`op-core`](crates/op-core) | 文档模型：图层树、256×256 分块写时复制的像素、混合模式、选区、历史记录、调整、滤镜、变换、绘画 |
| [`op-render`](crates/op-render) | wgpu 画布：缩放、mipmap、透明棋盘格与像素网格，通过 paint callback 嵌入 egui |
| [`op-io`](crates/op-io) | 文件读写：无依赖的 PSD 读写器，位图格式交给 `image` |
| [`op-color`](crates/op-color) | 颜色模型与转换 |
| [`op-tools`](crates/op-tools) | 工具定义与快捷键 |
| [`op-ui`](crates/op-ui) | egui 界面：菜单、选项栏、工具栏、面板、对话框与工具交互 |
| [`app`](app) | 可执行程序 |

## 测试

```bash
cargo test --workspace
```

约 350 个测试同时守护目标的两个方面：

- **与 Photoshop 一致**：[`crates/op-core/fixtures`](crates/op-core/fixtures) 中保存了 Photoshop 2026 对探测图执行各调整与滤镜后的输出。结果偏离超过测量时达到的误差（通常为 1 级）时测试失败。
- **界面行为**：UI 测试通过 `egui_kittest` 无窗口地驱动真实程序：从菜单打开对话框、往输入框里输入、按 Enter，再检查文档内容与历史记录。

## 路线图

尚未完成的主要方向，大致按优先级排列：

- 其余面板与对话框的像素级校准
- 更多工具：画笔预设与压感、修复画笔、快速选择、磁性套索
- 其余调整与滤镜：Shadows/Highlights、Replace Color、Smart Sharpen、Render、Distort
- 可再编辑的文字与矢量形状、图层样式、调整图层与填充图层
- Channels 与 Paths 面板、对齐吸附、智能参考线
- 8 位 RGB 以外的颜色模式、ICC 色彩管理
- 智能对象、动作与自动化

## 参与贡献

欢迎提交 issue 与 pull request。任何改动只遵循一条原则：**以运行中的 Photoshop 2026 为准**。OpenPhoto 的外观或行为与它不同，就是 bug。提交 pull request 前请运行 `cargo fmt`、`cargo clippy --workspace --all-targets` 与 `cargo test --workspace`，并为改动的行为补上测试。

## 许可证

可任选以下之一：

- Apache License, Version 2.0（[LICENSE-APACHE](LICENSE-APACHE)）
- MIT 许可证（[LICENSE-MIT](LICENSE-MIT)）

第三方资源：界面字体为 [Source Sans 3](https://github.com/adobe-fonts/source-sans)（SIL Open Font License，见 [`OFL.txt`](crates/op-ui/assets/fonts/OFL.txt)），图标来自 [Phosphor Icons](https://phosphoricons.com)（MIT）。

<sub>OpenPhoto 是独立项目，与 Adobe Inc. 无关联，也未获其认可或赞助。Adobe 与 Photoshop 是 Adobe Inc. 的商标，此处仅用于说明兼容性。</sub>
