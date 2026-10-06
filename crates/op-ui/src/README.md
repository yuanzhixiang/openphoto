# op-ui：界面层总体规范

`op-ui` 是 OpenPhoto 的界面层，基于 egui 0.36（eframe + wgpu）。它只负责把用户输入转换成对 `op-core` 文档的操作，并把状态画出来；像素数据、图层、历史记录都在 `op-core` 里。

## 产品目标：1:1 复刻 Photoshop

界面以 Photoshop 2026 默认的「中灰」主题为准，布局、尺寸、颜色、文案、菜单结构和快捷键都对齐 Photoshop。判断某处 UI「对不对」时，以运行中的 Photoshop 为唯一参照，不以个人审美或 egui 默认样式为准。

不复刻的部分：

- Adobe 的图标、Adobe Clean 字体、Logo 和「Photoshop」名称不使用。图标使用 Phosphor（MIT），界面字体使用 Adobe 以 OFL 开源的 Source Sans 3（随 crate 打包在 `assets/fonts/`）。
- 应用名称显示为「OpenPhoto」。

## 尺寸体系

界面有两套尺寸来源，都换算成 egui 的逻辑单位：

1. **参考截图像素**：早期界面（选项栏、工具栏、右侧面板、Canvas Size 对话框等）的尺寸按一张 Photoshop 窗口截图量取（该截图宽 2000 px，对应 Photoshop 窗口 1349 pt）。这些数值直接写在代码里，再通过 `theme::UI_SCALE = 0.675` 用 egui 的 `zoom_factor` 整体缩放到 Photoshop 的实际大小。
2. **Photoshop 点（pt）**：之后的界面直接在运行中的 Photoshop 里按 1:1 量取（Photoshop 窗口与 OpenPhoto 默认窗口同为 1350×800 pt），代码里用 `theme::pt(x)` 换算，`pt(x) = x / UI_SCALE`。History 面板使用这一体系。

新做的界面一律使用第 2 种：在 Photoshop 里量，用 `pt()` 写。

`UI_SCALE` 只影响界面，不影响画布：画布缩放始终以物理像素计（100% = 一个图像像素对应一个屏幕像素）。

## 颜色

所有颜色定义在 `theme::color`，取自 Photoshop 截图取色。主要的几种：面板 `#535353`、画布外粘贴板 `#282828`、标签栏 `#424242`、输入框 `#454545`、选中行 `#6b6b6b`、选中工具 `#383838`、深色分隔线 `#393939`、正文 `#eaeaea`。个别组件的专用颜色定义在组件文件内（例如 History 面板的滚动条轨道）。

## 菜单与快捷键

- 快捷键与 Photoshop 2026 的默认快捷键一致，数据来自通过辅助功能接口读取的 Photoshop 菜单。命令与快捷键的对应关系集中在 `commands.rs`。
- macOS 上使用原生菜单栏（`menu.rs`），带 ⌘ 的快捷键由菜单处理；其它平台没有原生菜单，由 egui 处理同一组快捷键。
- 单键快捷键（工具、D、X）由 `actions::handle_tool_keys` 处理，文本输入框获得焦点时不触发。
- 模态对话框打开期间，所有菜单命令禁用，单键快捷键也不触发，与 Photoshop 一致。

## 布局

窗口从外到内：标题栏（仅 macOS）、选项栏、左侧工具栏、右侧面板列、面板列左边的图标列、中间的文档区域（egui_dock 文档标签）。详见 `lib.md`。

## 文件索引

- `lib.rs`：应用入口、整体布局、文档标签、浮层（History 弹出面板、Canvas Size 对话框、错误提示）。
- `state.rs`：应用状态与单文档状态。
- `commands.rs`、`menu.rs`、`actions.rs`：命令、菜单、文件操作与快捷键。
- `document_view.rs`：画布与状态栏。
- `titlebar.rs`、`options_bar.rs`、`toolbar.rs`：窗口上方与左侧的固定区域。
- `panels/`：右侧面板与 History 弹出面板。
- `dialogs/`：模态对话框。
- `theme.rs`、`icons.rs`、`widgets.rs`：样式、图标映射、通用小部件。
