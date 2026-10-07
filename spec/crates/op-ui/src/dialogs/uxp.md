# dialogs/uxp.rs：UXP 调整对话框的控件

## 组件职责

Photoshop 2026 的 UXP 调整对话框（Brightness/Contrast、Color Balance、Hue/Saturation）共用的控件，尺寸量自 Photoshop 的 2x 截图（单位为 Photoshop 点）。

## 控件

- `label(ui, left_center, text)`：12 pt Adobe Clean（这里用 Source Sans 3 近似）`#f0f0f0` 标签；`left_center` 是大写字母的中线，绘制时下移 0.75 pt（Source Sans 比 Adobe Clean 偏高）。
- `number_box(ui, rect, text, id, range, focus)`：24 pt 高的数值框（`common::text_field`，数字左边距 11 pt，有焦点时蓝色外环）。有焦点时 ↑/↓ 加减 1，按住 Shift 加减 10，限制在 `range` 内。`focus` 为真时获得焦点并全选。
- `slider(ui, id, x0, x1, y, value, (min, max), colors)`：Spectrum 滑块。2 pt 粗的轨道，颜色由 `colors(t)` 给出（t 为 0–1 的位置），在滑块圆环两侧各断开 4.5 pt；滑块是 13 pt 的空心圆环（1 pt，`#d0d0d0`，拖动时 `#f0f0f0`）。
  - 取值：滑块圆心的行程是轨道两端各内缩 6.5 pt（圆环半径）。`min` 小于 0 时 0 在行程中点，正值按 `max`、负值按 `min` 各占半段（所以 Contrast 的 −50–100 也以 0 居中，与 Photoshop 一致）；`min` 不小于 0 时（Colorize 的色相与饱和度）从左端起线性取值。
  - 在轨道上按下或拖动时滑块跳到指针处，返回取整后的新值。
- `slider_stepped`：以 0 居中、按步长取值的滑块（Exposure 的 0.01、0.0001）。`linear_slider`：从 `min` 到 `max` 线性取值（Posterize、Black & White、Photo Filter、Gamma）。`draw_slider`：只画不交互的滑块（未启用的 Tint 行）。
- `three(a, mid, b)`：三色渐变。
- `stops(colors)`：等距色标的渐变，相邻色标之间按 sRGB 混合；各对话框的轨道颜色都是从 Photoshop 截图上取的色标。
- `buttons(ui, frame, third, ok_enabled, ok_focused)`：右侧按钮列，109 × 24 pt，距右边 20 pt，从 48 pt 起每 36 pt 一个：OK（默认按钮，白色边框）、Cancel，以及可选的第三个（`(标签, 是否可用)`，如 Auto）。`ok_focused` 时 OK 画成持有键盘焦点的样子（`common::ps_focused_button`）。Enter 等于 OK（仅在 `ok_enabled` 时）。按钮文字为 12 pt 粗体。
- `preview(ui, min, preview)`：「Preview (Opt+P)」复选框；⌥P 同样切换。
- `radio(ui, center, label, chosen)`：Spectrum 单选按钮，选中为 12 pt 浅灰圆盘中间留 3.5 pt 的孔，未选中为 `#8e8e8e` 圆环；标签在圆心右侧 15.5 pt。返回是否被点击。
- `checkbox(ui, min, label, checked)`：12 pt 复选框，标签在右侧 9.5 pt，与 Photoshop 的基线对齐（下移 1 pt）。

## 已知限制

- 滑块不支持键盘操作（Photoshop 的滑块可用方向键）。
