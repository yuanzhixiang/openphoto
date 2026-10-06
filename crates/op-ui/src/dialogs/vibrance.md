# dialogs/vibrance.rs：Vibrance 与 Posterize 对话框

## 组件职责

Image › Adjustments › Vibrance... 与 Posterize... 的对话框，按 Photoshop 2026 的 UXP 对话框重做（401 × 166 与 401 × 164 pt）。

## Vibrance

- Vibrance、Saturation（−100–100，默认 0）。标签 (20, 60.5) / (20, 115.5)，输入框 x 215–260、y 48 / 103，轨道 y 84 / 139（x 20–260，`#737373`，0 居中）。打开时 Vibrance 框获得焦点并全选。
- 「Preview (Opt+P)」(272, 127)；右侧 OK、Cancel。

## Posterize

- Levels（2–255，默认 4）：标签 (20, 60.5)，输入框 (220, 48)–(260, 72)，轨道 y 84，从左端 2 到右端 255 线性取值（`uxp::linear_slider`）。打开时输入框获得焦点并全选。
- 「Preview (Opt+P)」(272, 127)。

## 测试覆盖

- `defaults_and_ranges`；`ui_tests::small_uxp_adjustment_dialogs_apply`；截图 `vibrance_dialog.png`、`posterize_dialog.png`。
