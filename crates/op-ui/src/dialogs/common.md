# dialogs/common.rs：对话框公共部件

## 职责

各模态对话框共用的外观部件，保证所有对话框看起来一致。

## 部件

- `frame(ui, rect, title, font)`：画对话框外框：阴影、`#535353` 底色、圆角 10、深色外边框，以及顶部高 28 pt 的浅灰标题栏（`#d0d2d4`，深色居中标题，下方有分隔线）。对应 Photoshop 对话框的 macOS 窗口标题栏。标题比标题栏的正中高 1.5 pt，与 Photoshop 2026 的各对话框一致（Duplicate Layer、New Layer、Lock Layers 的标题墨迹都从 y 8 pt 开始）；标题用调用方给的字体（都是 13 pt 粗体系统字体）按 AppKit 的字距排版（`theme::tracked_galley`）。
- `pill_button(ui, rect, label, font, enabled)`：胶囊形按钮。浅色边框（`#d0d0d0`，1.5），底色与对话框相同，悬停变浅，按下变深；禁用时边框和文字置灰且不响应点击。同一对话框里按钮文字必须唯一（用作交互 id）。

- `text_field(ui, rect, text, id, font, pad, select_all)`：同 `number_field`，但字体和左侧内边距由调用方给出，右侧内边距固定 2 pt（New Layer 对话框用它）。`number_field` 以比例字体、左侧内边距 = 框高 × 0.23 调用它。
- `number_field(ui, rect, text, id, font, select_all)`：对话框里的文字输入框：`#454545` 底、`#777777` 边框、圆角 3；获得焦点时外面一圈 2 像素的蓝色 `#1473e6`。`select_all` 时获得焦点并全选文字（对话框打开时的第一个输入框）。
- `dropdown(ui, rect, id, selected, font, enabled, menu)`：对话框里的下拉框，填满 `rect`：底色与对话框相同、`#6a6a6a` 边框、圆角 3，悬停变浅。

## 使用方

Canvas Size（`canvas_size.md`）、Fill（`fill.md`）和 Color Picker（`color_picker.md`）。

## Photoshop 2026 新式对话框控件

照 New Layer 对话框实测，供 New Layer、Duplicate Layer 等对话框使用（字体见 `theme.md`）：

- `ps_dropdown(ui, rect, id, content, menu)`：圆角 3、1 pt `#7a7a7a` 边、面板色底（悬停变浅）的下拉框，内容由 `content` 画出，距右边 13.5 pt 是 10 × 6 pt 的 V 形箭头；点击弹出 `menu`。
- `ps_checkbox(ui, min, label, checked, enabled)`：12 pt 见方、圆角 2.5 的复选框，未勾选时 1 pt `#a0a0a0` 边，勾选时为浅灰底加深色对勾；文字（`theme::uxp` 12 pt，New Layer 这类 UXP 对话框的字体）在框右 9.5 pt，点击框或文字都会切换；不可用时框和文字为 `#8e8e8e`。`ps_checkbox_with(..., font, label_dy)` 可以换字体并把文字下移（Lock Layers 用面板字体，下移 1 pt）。
- `ps_button(ui, rect, label, default, enabled, bold)`：胶囊形按钮，1 pt 边：默认按钮 `#f1f1f1`、其它 `#727272`、不可用 `#5e5e5e`；`bold` 时标签为 `theme::uxp_bold` 12 pt（New Layer 这类 UXP 对话框），否则为 AppKit 系统字体 12 pt（Duplicate Layer 这类经典对话框）。`ps_button_with(..., font)` 换标签字体。
- `ps_focused_button(ui, rect, label, font)`：拥有键盘焦点的默认按钮，出现在没有输入框的对话框里（Lock Layers）：`#737373` 底、1 pt 白边，外面隔 1 pt 一圈 2 pt 的蓝色 `#2d63cb` 焦点环（Photoshop 2026 实测，鼠标不在按钮上时也是这样）。
- `field_dropdown(ui, rect, id, text, enabled, menu)`：AppKit 风格对话框（Duplicate Layer、Image Size）里像输入框一样的下拉框：`#454545` 底、1 pt `#666666` 边（不可用时 `#4e4e4e` / `#5e5e5e`），值为 12 pt 系统字体、左缩进 8.5，太长时用 `elide` 截断加「...」，距右边 7.5 是 V 形箭头。
