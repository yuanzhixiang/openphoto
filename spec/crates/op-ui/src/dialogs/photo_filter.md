# dialogs/photo_filter.rs：Photo Filter 对话框

## 组件职责

Image › Adjustments › Photo Filter... 的对话框，按 Photoshop 2026 的 UXP 对话框重做（401 × 217 pt）。

## 数据与布局

- Filter 单选 (26, 60.5) 与下拉 (78.5, 48.5)–(260, 72.5)：Photoshop 的 20 种预设（`FILTERS`，默认 Warming Filter (85)）；选择预设时回到 Filter 模式。
- Color 单选 (26, 94.5) 与色块 (79.5, 82.5)–(127, 105.5)：显示当前颜色，点击打开取色弹窗（egui 的取色器，不是 Photoshop 的 Color Picker），取色后切到 Color 模式。
- 「Density」(20, 131.5)，输入框 (213, 118.5)–(260, 142.5) 显示「25%」；滑块 y 154 按 0–100% 线性画（1% 以下不接受）。
- 「Preserve luminosity」复选框 (20, 179.5)，默认勾选；「Preview (Opt+P)」(272, 127)；右侧 OK、Cancel。打开时没有输入框持有焦点。

## 已知限制

- Photoshop 打开时 Filter 单选带键盘焦点环；色块取色不是 Photoshop 的拾色器。

## 测试覆盖

- `filter_or_color`；`ui_tests::small_uxp_adjustment_dialogs_apply`；截图 `photo_filter_dialog.png`。
