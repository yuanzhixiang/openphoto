# dialogs/exposure.rs：Exposure 对话框

## 组件职责

Image › Adjustments › Exposure... 的对话框，按 Photoshop 2026 的 UXP 对话框重做（401 × 257 pt）。算法见 `op-core` 的 `adjust.md`。

## 数据与布局

- Exposure（−20–20）、Offset（−0.5–0.5）、Gamma Correction（0.01–9.99），默认 0、0、1；数值去掉末尾的 0 显示，Gamma 显示为「+1」（Photoshop 如此）。
- 「Preset」(20, 61)，下拉 (56, 48.5)–(231, 73.5)（Default/Custom），预设菜单图标 (248, 61)。
- 三行：标签 y 97 / 152 / 207，输入框 x 201–260、y 84 / 139 / 194，轨道在框下 36 pt。Exposure、Offset 的滑块以 0 居中（步长 0.01、0.0001），Gamma 从 0.01 到 9.99 线性（步长 0.01）。
- 三个吸管 (283.5 / 319.5 / 356, 133)：选中的一个（打开时为白点）画在 `#d8d8d8` 圆角底上；吸管本身不能在文档上取样。
- 「Preview (Opt+P)」(272, 162)；右侧 OK、Cancel。打开时 Exposure 框获得焦点并全选。

## 测试覆盖

- `defaults_and_numbers`；`ui_tests::small_uxp_adjustment_dialogs_apply`；截图 `exposure_dialog.png`。
