# dialogs/trim.rs：Image › Trim 对话框

## 组件职责

选择 Trim 的依据和要裁掉的边，确定后由 `lib.rs` 执行 `image_ops::trim`（规则见 `op-core` 的 `image_ops.md`）。

## 布局与视觉

按 Photoshop Trim 对话框的结构排列（尺寸单位为 Photoshop 点，尚未与 Photoshop 逐像素比对）：

- 外框与标题栏使用公共部件（`common.md`），标题「Trim」，对话框 380 × 236。
- 「Based On」分组标题（半粗体，右侧一条横线），下面三个单选项：Transparent Pixels、Top Left Pixel Color、Bottom Right Pixel Color，行距 24。
- 「Trim Away」分组标题，下面四个复选框，两列两行：Top、Left / Bottom、Right。
- 右侧上方 OK、Cancel 两个圆角按钮。

## 数据输入与默认值

- `TrimDialog::new(has_background)`：文档有背景图层时，Transparent Pixels 置灰，默认选 Top Left Pixel Color；没有背景图层时默认选 Transparent Pixels。四个边默认全部勾选。每次打开都重新取默认值，不记忆上次的选择。

## 交互

- OK 或 Enter：返回 `Outcome::Apply { basis, sides }`。
- Cancel 或 Esc：返回 `Outcome::Cancel`。
- 打开期间是模态的（`AppState::modal_open()` 为真）。

## 已知限制

- Photoshop 会记住上次的选择；这里每次恢复默认值。
