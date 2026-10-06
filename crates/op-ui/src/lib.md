# lib.rs：应用入口与整体布局

## 职责

定义 `OpenPhotoApp`（实现 `eframe::App`），负责：

- 启动时安装字体与样式、注册画布渲染器、在 macOS 上安装原生菜单栏，并打开命令行传入的文件。
- 每帧按固定顺序处理输入、排布各区域、显示浮层。
- 用 egui_dock 管理文档标签。

## 启动

1. `theme::install_fonts`、`theme::apply_style`。
2. `op_render::install` 注册画布管线。依赖 wgpu 渲染后端，缺失时直接 panic（应用只支持 wgpu）。
3. macOS 上 `menu::NativeMenu::install`。`new_headless` 跳过这一步，供无窗口 UI 测试使用（见 `ui_tests.md`）。
4. 有命令行文件时逐个打开；没有时新建一个 1920×1080 白色背景的「Untitled-1」文档。

## 每帧流程

1. 拖放进窗口的文件逐个打开。
2. 执行命令：macOS 上有原生菜单时，取菜单事件和 egui 补充捕获的 ⌘=；没有原生菜单时（其它平台、无窗口测试），由 egui 处理全部快捷键（见 `commands.md`）。
3. 处理单键快捷键（工具、D、X）。
4. 按顺序排布区域（egui 面板先加的在外侧）：
   - 标题栏（仅 macOS，高 40）
   - 选项栏（高 52）
   - 左侧工具栏（宽 58）
   - 右侧面板列（宽 478）
   - 面板列左边的图标列（宽 58）
   - 中间：有文档时显示 egui_dock 文档区，没有文档时只有粘贴板底色。
   以上尺寸是参考截图像素，再经 `UI_SCALE` 缩放。
5. 确定当前文档：取 dock 中获得焦点的标签；没有焦点时，若当前文档已不存在则改为第一个标签。
6. 浮层：History 弹出面板（`history_open` 时）、Canvas Size 对话框、Color Picker（画在 Canvas Size 之上）、macOS 菜单状态同步、错误提示对话框。

## 文档标签

- 标签文字格式与 Photoshop 一致：`{文件名} @ {缩放} ({模式简写}/{位深})`，例如 `1.webp @ 100% (RGB/8)`，半粗体。
- 标签样式：标签栏 `#424242`，激活标签 `#535353`，未激活标签与标签栏同色，悬停 `#4c4c4c`；标签内容区底色为粘贴板色，无内边距、无滚动条。
- 关闭标签时从状态里移除该文档；标签不允许拖出成独立窗口。
- 不显示「新增标签」「全部关闭」「折叠」按钮。

## History 弹出面板的定位

照 Photoshop 收起面板的弹出方式：面板右边紧贴图标列左边缘，顶部比 History 按钮高 13 pt，层级在最前，带阴影。

- 点击面板和 History 按钮以外的任何位置，面板收起（对应 Photoshop 默认开启的「Auto-Collapse Iconic Panels」）。
- 面板里的「>>」按钮也会收起面板。

## Canvas Size 对话框的接入

对话框返回「确定」时，若新尺寸与当前尺寸不同，则对当前文档执行 `resize_canvas` 并记录一条「Canvas Size」历史；尺寸相同则不做任何事（不产生历史记录）。

## Color Picker 的接入

- Canvas Size 请求拾色时（选择「Other...」或点击色块），以「Color Picker」为标题、当前扩展颜色为初始色打开 Color Picker。
- Color Picker 打开期间，Canvas Size 不响应 Enter/Esc。
- 每帧把 Color Picker 的「Add to Swatches」结果追加到 `swatches`。
- 确定时按目标写回：前景色、背景色，或 Canvas Size 的扩展颜色。

## 错误提示

`AppState::alert` 有内容时显示模态对话框，只有一个「OK」按钮。打开文件失败、导出失败时使用。
