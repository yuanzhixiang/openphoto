# dialogs/adjust.rs：调整对话框

## 组件职责

Image › Adjustments 中带数值设置的调整对话框：Threshold...、Posterize...、Levels...、Hue/Saturation...、Exposure...。对话框只管理设置与 Preview 开关，预览与应用由 `lib.rs` 完成（见 `lib.md`「调整对话框的接入」）。像素算法见 `op-core` 的 `adjust.md`。

## 数据输入

- `AdjustDialog::new(kind, histogram, before)`：`kind` 为对话框种类；`histogram` 是打开时活动图层选区内的直方图（Threshold 用亮度，Levels 用 R/G/B 合并）；`before` 是打开时的文档快照。
- 每个设置是一个「参数」：标签、范围、默认值、小数位数。输入框中的文字按小数位数格式化。默认值与范围同 Photoshop：

| 对话框 | 参数（范围，默认值） |
|---|---|
| Threshold | Threshold Level（1–255，128） |
| Posterize | Levels（2–255，4） |
| Levels | 输入黑场（0–253，0）、gamma（0.01–9.99，1.00）、输入白场（2–255，255）、输出黑场（0–255，0）、输出白场（0–255，255） |
| Hue/Saturation | Hue（−180–180，0）、Saturation（−100–100，0）、Lightness（−100–100，0） |
| Exposure | Exposure（−20.00–20.00，0.00）、Offset（−0.5000–0.5000，0.0000）、Gamma Correction（0.01–9.99，1.00） |

- 每次打开都恢复默认值；Preview 默认勾选。

## 布局与视觉

按 Photoshop 对话框的结构排列（单位为 Photoshop 点，尚未逐像素比对），右侧固定是 OK、Cancel 按钮和其下的 Preview 复选框：

- Threshold（400 × 232）：「Threshold Level:」与输入框；下方 258 × 100 的直方图（深灰底，每个值一条竖线，高度按最大计数归一化）；直方图下方一个三角标记。
- Posterize（330 × 132）：「Levels:」与输入框。
- Levels（400 × 330）：「Channel: RGB」（固定文字）；「Input Levels:」直方图，下方三个三角标记（黑色 = 输入黑场、灰色 = gamma、白色 = 输入白场）及对应的三个输入框（左、中、右）；「Output Levels:」黑到白的渐变条，下方两个三角标记（黑、白）及两个输入框。
- Hue/Saturation、Exposure（400 × 220）：每个参数一行，标签在左、输入框在右，下方一条细轨道和三角标记，行距 52。
- 打开时第一个输入框获得焦点并全选。

## 交互

- 输入框：任一参数超出范围或不是数字时，OK 置灰，也不预览。Levels 还要求输入黑场至少比白场小 2。
- 三角标记：在轨道上按下时选中最近的标记，拖动或单击把它移到指针位置。普通参数按轨道比例取值（按小数位数取整）；Levels 的黑白场按 0–255 取值；Levels 的 gamma 标记位于黑白场之间 `0.5^gamma` 的位置，拖动时按位置反算 gamma。
- Preview：勾选时文档实时显示结果，取消勾选时恢复原样。
- OK 或 Enter（所有值有效时）：返回 `Outcome::Apply(adjustment)`；Cancel 或 Esc：返回 `Outcome::Cancel`。
- 打开期间是模态的。

## 测试覆盖

- `defaults_match_photoshop`：Levels、Hue/Saturation 的默认调整，Exposure 输入框的默认文字。
- `invalid_values_disable_the_dialog`：Levels 黑白场过近、Hue 超出范围时没有可应用的调整。
