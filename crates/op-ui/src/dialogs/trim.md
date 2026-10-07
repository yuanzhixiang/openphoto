# dialogs/trim.rs：Image › Trim 对话框

## 组件职责

选择 Trim 的依据和要裁掉的边，确定后由 `lib.rs` 执行 `image_ops::trim`（规则见 `op-core` 的 `image_ops.md`）。

## 布局与视觉

按 Photoshop 2026 的 Trim 对话框（2x 截图）逐点量取，与 New Layer、Canvas Size 同一类（UXP 样式）。对话框 258 × 248 pt，坐标为距对话框左上角的 pt：

- 标题栏：`common::frame`，系统粗体 13 pt「Trim」。文字为 12 pt 面板字体（Source Sans 3 近似 Adobe Clean），`#f1f1f1`。
- 「Based on」粗体标题，左端 20，中心 y 57。三个单选项（句首大写，与 Photoshop 2026 一致）：Transparent pixels、Top left pixel color、Bottom right pixel color，圆心 (26, 84 / 108 / 132)，直径 12 pt，文字在圆心右 15 pt。选中为 `#d6d6d6` 实心圆加深色圆点；未选中为 1 pt `#a0a0a0` 圆环；不能选时圆环与文字变暗。
- 「Trim away」粗体标题，中心 y 166。四个复选框（`common::ps_checkbox`，12 pt）：Top (20, 186)、Left (97, 186)、Bottom (20, 210)、Right (97, 210)。
- OK（默认按钮）(168–238，y 48–72)、Cancel (y 84–108)：`common::ps_button`。

与 Photoshop 截图相比各元素相差不超过 1 pt。

## 数据输入与默认值

- `TrimDialog::new(has_background)`：文档有背景图层时，Transparent pixels 置灰，默认选 Top left pixel color；没有背景图层时默认选 Transparent pixels。四个边默认全部勾选。每次打开都重新取默认值，不记忆上次的选择。

## 交互

- OK 或 Enter：返回 `Outcome::Apply { basis, sides }`。
- Cancel 或 Esc：返回 `Outcome::Cancel`。
- 打开期间是模态的（`AppState::modal_open()` 为真）。

## 已知限制

- Photoshop 会记住上次的选择；这里每次恢复默认值。
