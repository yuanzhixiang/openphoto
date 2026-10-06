# dialogs/color_balance.rs：Color Balance 对话框

## 组件职责

Image › Adjustments › Color Balance... 的对话框，按 Photoshop 2026 的 UXP 对话框重做（447 × 295 pt）。

## 数据与默认值

- 阴影、中间调、高光各有三个值（Cyan to Red、Magenta to Green、Yellow to Blue，−100–100，默认 0），分别保存；`tone` 为正在编辑的色调，打开时为 Midtones。
- Preserve Luminosity 默认勾选。
- `adjustment()`：九个值都有效时返回 `Adjustment::ColorBalance`。

## 布局

- 「Tone」标签 (20, 60.5)；三个 24 pt 的色调圆点，圆心 x 68（黑）、153（`#777777`）、240（白），y 60，标签分别从 x 85.5、170.5、258 开始。选中的圆点外圈为 2 pt 白环、其内 2 pt `#454545` 间隙、中心为填充色；未选中的为填充色加 0.5 pt `#a7a7a7` 细边。
- 三行滑块：标签 x 20，输入框 x 249–305.5，y 92 / 140 / 188（24 高）；轨道 y 128 / 176 / 224，x 20–305.5。轨道颜色为 Photoshop 的渐变（青→灰→红、洋红→灰→绿、黄→灰→蓝，按截图取 17 个色标；中间并不是线性插值）。
- 「Preserve Luminosity」复选框 (20, 257)；「Preview (Opt+P)」(318, 127)；右侧 OK、Cancel。

## 交互

- 点击色调圆点或其标签切换色调：滑块与输入框显示该色调的值，第一个输入框获得焦点并全选。
- 打开时 Midtones 的第一个输入框获得焦点并全选。
- 其余同 UXP 对话框（`uxp.md`）。

## 测试覆盖

- `ui_tests::color_balance_tones_keep_their_values`：中间调输入 40、切到阴影输入 −30、取消 Preserve Luminosity，结果等于对应的核心曲线。
- 截图对比：`color_balance_dialog.png`。
