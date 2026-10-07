# options_kit.rs：选项栏控件（按 Photoshop 实测定位）

## 组件职责

选项栏里 Photoshop 2026 的各种控件，按「栏坐标」绝对定位：`x` 为距选项栏左边的 pt，`y` 为距选项栏顶上方 0.5 pt 的 pt（与测量时截图的裁剪方式一致）。每个工具的选项栏因此就是一串实测坐标（见 `options_bar.md`「按实测布局的工具」）。

## 控件与实测尺寸

- `sep(x)`：1 pt 分隔线，`#3e3e3e`，y 6.5–29。
- `label(x, text, enabled)`：面板字体文字，`#dddddd`，不可用时 `#878787`，纵向中心 17.25；Source Sans 3 比 Adobe Clean 起笔靠右，所以向左偏 0.75 pt。
- `check(x, label, value, enabled)`：Photoshop 样式复选框（`widgets::checkbox`），方框从 `x` 起，标签在方框右 8 pt。不可用时关闭 egui 对禁用控件的淡化（`disabled_alpha = 1`），由复选框自己画 Photoshop 的禁用色。
- `field` / `value(x0, x1, …)`：输入框，y 9–25.5，`#454545` 底、1 pt `#666666` 边（不可用 `#4d4d4d` / `#5e5e5e`），文字左缩进 4.5 pt；`value` 在输入结束（失去焦点或 Enter）时返回所输入的文字，Enter 不再传给画布。
- `popup` / `choice(x0, x1, …)`：下拉框，y 8–27（`widgets::dropdown_with`）。
- `button(x0, x1, text, enabled)`：按钮，y 5.5–29.5，`#454545` 底、`#666666` 边，文字居中；不可用时 `#4d4d4d` 底、`#5e5e5e` 边、`#878787` 字。`menu_button` 高一点（4.5–30.5）并在右下角画小三角；`chevron_button`（y 5–30）里是 12.5 pt 宽、1 pt 线的下拉箭头。
- `icon(cx, icon, tip, on, enabled)`：图标按钮，按下状态 (`on`) 为 y 4–31 的 `#383838` 框（0.5 pt `#636363` 边，3 pt 圆角）。
- `modes(x, mode)`：选区运算四按钮，每个 26 pt 宽，选中的为按下框；图标按实测绘制：New 是 10 pt 实心方块，Add 两个重叠的实心方块，Subtract 前一个方块减去后一个（后者只画轮廓），Intersect 两个轮廓、重叠处实心。
- `brush_picker(cx, label, size, hardness)`：笔刷选择器：笔尖预览按 Photoshop 的方式画（直径为笔刷大小个设备像素，最大 28 px，软笔刷向外渐隐；中心 y 11），下方 10.5 pt 的大小数字（y 26.25），右侧 21 pt 处的下拉箭头。
- `slider_box(x0, x1, id, current)`：百分比输入框后面共用边框的箭头框，点击弹出 0–100% 滑块，返回拖动后的新值。
- `swatch(x0, x1, fill, chevron_box)`：色块（y 5–30）。混合器画笔的载入颜色是无边框的白块，箭头紧跟其后；图案图章的图案块带 1 pt 边、画默认图案的叶点，箭头在单独的框里。

- `segmented(edges, labels, chosen, enabled)`：分段按钮（y 5–30，`#454545` 底、`#666666` 边，所选段为 `#383838`），返回点击的段。
- `color_box`（无边框颜色块）、`empty_pattern`（不可用的空图案框与箭头框）、`gradient_swatch`（渐变色块与箭头框）。
- `field_off(x0, x1)`：不可用的空输入框，y 8–26（Photoshop 禁用的输入框比可用的高一点）。
- `combo(x0, x1, x2, id, shown, options, enabled)`：输入框（y 8–26）与它右侧共用边框的箭头框（到 `x2`），点箭头框弹出 `options`；返回输入的文字或所选的项。用于字体、字体样式、字号、描边宽度。不可用时两部分都画成禁用色，不响应。
- `framed_swatch(x0, x1, fill)`：形状的 Fill / Stroke 色块：外框 y 7.5–27.5（`#666666` 1 pt 边），颜色内缩 3 pt；`None` 为「无颜色」（白底红斜线）。
- `line_popup(x0, x1, id, options, value)`：显示线型（实线、虚线、点线）的下拉框。
- `framed_color(x0, x1, y, fill)`：带 1 pt `#363636` 深色边的颜色块（文字颜色 y 8.5–26.5，画板背景 y 9–26）。
- `centered_label(cx, text)`：以 `cx` 为中心的文字；`width()` 返回选项栏宽度（pt），用于居中提示。

## 测试覆盖

- `ui_tests.rs` 的 `options_bars_match_photoshops_layout`：已按实测布局的工具，其选项栏的分隔线与各种框的边缘位置（从截图中按颜色识别）与 Photoshop 测得的数值（`PS_BAR_MARKS`）逐一对应，相差不超过 1 pt。
- `screenshot_options_bars`（`#[ignore]`）：把每个工具的选项栏截成 `target/ui-shots/bars/<Tool>.png`，用于与 Photoshop 的截图并排比对。
