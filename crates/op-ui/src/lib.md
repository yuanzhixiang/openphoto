# lib.rs：应用入口与整体布局

## 职责

定义 `OpenPhotoApp`（实现 `eframe::App`），负责：

- 启动时安装字体与样式、注册画布渲染器、在 macOS 上安装原生菜单栏并把窗口标记为 sRGB 色彩空间，然后打开命令行传入的文件。
- 每帧按固定顺序处理输入、排布各区域、显示浮层。
- 排布文档标签栏（`doc_tabs.rs`）和当前文档的视图（`document_view.rs`）。

## 启动

1. `theme::install_fonts`、`theme::apply_style`。
2. `op_render::install` 注册画布管线。依赖 wgpu 渲染后端，缺失时直接 panic（应用只支持 wgpu）。
3. macOS 上 `menu::NativeMenu::install`。`new_headless` 跳过这一步，供无窗口 UI 测试使用（见 `ui_tests.md`）。
4. macOS 上 `color_management::use_srgb` 把窗口的 Metal 图层标记为 sRGB（见 `color_management.md`）。`new_headless` 没有窗口，同样跳过。
5. 连接系统剪贴板（`clipboard::Clipboard::new(true)`）。`new_headless` 跳过这一步，测试不会改动用户的剪贴板。
6. 有命令行文件时逐个打开；没有时新建一个 1920×1080 白色背景的「Untitled-1」文档。

## 每帧流程

1. 拖放进窗口的文件逐个打开。
2. 执行命令：先记下是否有输入框获得焦点（`AppState::typing`）。macOS 上有原生菜单时，取菜单事件和 egui 补充捕获的 ⌘=；没有原生菜单时（其它平台、无窗口测试），由 egui 处理全部快捷键（见 `commands.md`）。
3. 处理单键快捷键（工具、D、X）。
4. 按顺序排布区域（egui 面板先加的在外侧），尺寸按 Photoshop 1:1 量取（pt）：
   - 标题栏（仅 macOS，高 29，含底部 1 pt 分隔线）
   - 选项栏（高 33）
   - 左侧工具栏（宽 42，含右侧 3 pt 深色边）
   - 右侧面板列（宽 321）
   - 面板列左边的图标列（宽 44，含左侧 3 pt 深色边和右侧 5 pt 分隔带）
   - 中间：有文档时，上面是文档标签栏，下面是当前文档的视图；没有文档时只有粘贴板底色。
   这些面板都不使用 egui 自带的分隔线，边框由各区域按 Photoshop 自己绘制。
5. 确定当前文档：当前文档已不存在时，改为最后一个标签（见 `doc_tabs.rs` 的 `ensure_active`）。
6. 浮层：History 弹出面板（`history_open` 时）、Canvas Size 对话框、Fill 对话框、Color Picker（画在前两者之上）、macOS 菜单状态同步、错误提示对话框。

## 输入注入

`raw_input_hook` 在 egui 处理每帧输入之前，把 `AppState::forward_events` 里的事件追加到原始输入中并清空它。菜单的 Cut/Copy/Paste 转交给输入框时用到（见 `commands.md`「剪贴板」）。

## 文档标签

文档按 `AppState::doc_order` 的顺序显示在标签栏上，标签的样式与交互见 `doc_tabs.md`。

## History 弹出面板的定位

照 Photoshop 收起面板的弹出方式：面板右边紧贴图标列左边缘，顶部比 History 按钮高 13 pt，层级在最前，带阴影。

- 点击面板和 History 按钮以外的任何位置，面板收起（对应 Photoshop 默认开启的「Auto-Collapse Iconic Panels」）。
- 面板里的「>>」按钮也会收起面板。

## Canvas Size 对话框的接入

对话框返回「确定」时，若新尺寸与当前尺寸不同，则对当前文档执行 `resize_canvas` 并记录一条「Canvas Size」历史；尺寸相同则不做任何事（不产生历史记录）。

## Fill 对话框的接入

- Fill 对话框请求拾色时（Contents 选 Color...），以「Color Picker (Fill Color)」为标题打开 Color Picker，确定后写回对话框。
- 确定时对当前文档执行填充并记录「Fill」，失败时弹出提示。

## Color Picker 的接入

- Canvas Size 请求拾色时（选择「Other...」或点击色块），以「Color Picker」为标题、当前扩展颜色为初始色打开 Color Picker。
- Color Picker 打开期间，Canvas Size 不响应 Enter/Esc。
- 每帧把 Color Picker 的「Add to Swatches」结果追加到 `swatches`。
- 确定时按目标写回：前景色、背景色，或 Canvas Size 的扩展颜色。

## 错误提示

`AppState::alert` 有内容时显示模态对话框，只有一个「OK」按钮。打开文件失败、导出失败时使用。
