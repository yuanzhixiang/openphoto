# dialogs/threshold.rs：Threshold 对话框

## 组件职责

Image › Adjustments › Threshold... 的对话框，按 Photoshop 2026 的经典对话框重做（381 × 191 pt）。

## 数据与布局

- Threshold Level（1–255，默认 128）。「Threshold Level:」右对齐到 x 171.5、y 47.25；输入框 (177, 38)–(213.5, 57)（`appkit::field`），打开时获得焦点并全选。
- 亮度直方图 (15, 66)–(270, 166)，`#454545` 底、`#d0d0d0` 竖条，按最大计数归一化（打开时活动图层选区内的亮度直方图）。
- 下方一枚白色针（尖端 y 167.5），0 与 255 在 x 14.25 与 268.75；在针所在的行按下或拖动设置色阶。
- 按钮 x 290–370.5：OK (38.5)、Cancel (73.5)；「Preview」复选框 (290, 118)。

## 测试覆盖

- `level_range`；`ui_tests::adjustments_with_dialogs`；截图 `threshold_dialog.png`。
