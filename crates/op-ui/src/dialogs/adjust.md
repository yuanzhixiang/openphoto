# dialogs/adjust.rs：调整与滤镜对话框

## 组件职责

带设置的调整与滤镜对话框：Image › Adjustments 的 Threshold...、Posterize...、Levels...、Hue/Saturation...、Exposure...，以及 Filter 菜单的 Gaussian Blur...、Box Blur...、Unsharp Mask...、Add Noise...、Median...、Minimum...、Maximum...、High Pass...、Offset...、Mosaic...。对话框只管理设置与 Preview 开关，预览与应用由 `lib.rs` 完成（见 `lib.md`「调整与滤镜对话框的接入」）。像素算法见 `op-core` 的 `adjust.md` 与 `filter.md`。

对话框的结果是 `Effect`：`Adjustment(Adjustment)` 或 `Filter(Filter)`。`Effect::name()` 是历史名称，`Effect::apply(doc, background)` 调用对应的 `op-core` 函数。

## 数据输入

- `AdjustDialog::new(kind, histogram, before)`；`colors` 字段为 Gradient Map 的两种颜色（打开时设为前景色与背景色）。
- `AdjustDialog::new(kind, histogram, before)`：`kind` 为对话框种类；`histogram` 是打开时活动图层选区内的直方图（Threshold 用亮度，Levels 用 R/G/B 合并）；`before` 是打开时的文档快照。
- 每个设置是一个「参数」：标签、范围、默认值、小数位数，以及类型——数值（输入框 + 滑块）、选项（单选按钮，值为选中项的序号）、复选（值为 0 或 1）。输入框中的文字按小数位数格式化。调整的默认值与范围同 Photoshop；滤镜的默认值尚未与 Photoshop 核对：

| 对话框 | 参数（范围，默认值） |
|---|---|
| Threshold | Threshold Level（1–255，128） |
| Curves | 没有数值参数，由曲线图编辑（见下） |
| Posterize | Levels（2–255，4） |
| Levels | 输入黑场（0–253，0）、gamma（0.01–9.99，1.00）、输入白场（2–255，255）、输出黑场（0–255，0）、输出白场（0–255，255） |
| Hue/Saturation | Hue（−180–180，0）、Saturation（−100–100，0）、Lightness（−100–100，0） |
| Exposure | Exposure（−20.00–20.00，0.00）、Offset（−0.5000–0.5000，0.0000）、Gamma Correction（0.01–9.99，1.00） |
| Brightness/Contrast | Brightness（−150–150，0）、Contrast（−50–100，0） |
| Color Balance | Cyan — Red、Magenta — Green、Yellow — Blue（−100–100，0，只作用于中间调）、Preserve Luminosity（勾选） |
| Black and White | Reds、Yellows、Greens、Cyans、Blues、Magentas（%，−200–300，默认 40、60、40、60、20、80） |
| Vibrance | Vibrance、Saturation（−100–100，0） |
| Photo Filter | Filter（Photoshop 的 20 种预设：Warming Filter (85) 等，默认 Warming Filter (85)）、Density（1–100%，25）、Preserve Luminosity（勾选） |
| Gradient Map | Reverse（不勾选；渐变为打开时的前景色到背景色） |
| Gaussian Blur | Radius (pixels)（0.1–1000.0，1.0） |
| Box Blur | Radius (pixels)（1–2000，1） |
| Unsharp Mask | Amount (%)（1–500，50）、Radius (pixels)（0.1–1000.0，1.0）、Threshold (levels)（0–255，0） |
| Add Noise | Amount (%)（0.10–400.00，12.50）、Distribution（Uniform / Gaussian，Uniform）、Monochromatic（不勾选） |
| Median、Minimum、Maximum | Radius (pixels)（1–500，1） |
| High Pass | Radius (pixels)（0.1–1000.0，10.0） |
| Offset | Horizontal (pixels right)、Vertical (pixels down)（−30000–30000，0）、Undefined Areas（Set to Transparent / Repeat Edge Pixels / Wrap Around，Set to Transparent） |
| Mosaic | Cell Size (square)（2–200，10） |

- 每次打开都恢复默认值；Preview 默认勾选。

## 布局与视觉

按 Photoshop 对话框的结构排列（单位为 Photoshop 点，尚未逐像素比对），右侧固定是 OK、Cancel 按钮和其下的 Preview 复选框：

- Threshold（400 × 232）：「Threshold Level:」与输入框；下方 258 × 100 的直方图（深灰底，每个值一条竖线，高度按最大计数归一化）；直方图下方一个三角标记。
- Posterize（330 × 132）：「Levels:」与输入框。
- Curves（420 × 380）：「Preset: Default」「Channel: RGB」（固定文字）；240 × 240 的曲线图：亮度直方图作底、四等分网格、对角基线、曲线（1.5 pt 浅色线）与控制点（6 pt 方块，选中的实心）；图下方显示选中点的「Output:」「Input:」。默认两个端点 (0, 0)、(255, 255)。
  - 单击或开始拖动：8 pt 内有控制点就选中它，否则在指针处加一个新点并选中；拖动时点跟随指针，横坐标限制在相邻两点之间；拖出图表 20 pt 以外松开时删除该点（端点不会被删除）。
- Levels（400 × 330）：「Channel: RGB」（固定文字）；「Input Levels:」直方图，下方三个三角标记（黑色 = 输入黑场、灰色 = gamma、白色 = 输入白场）及对应的三个输入框（左、中、右）；「Output Levels:」黑到白的渐变条，下方两个三角标记（黑、白）及两个输入框。
- 其余对话框（Hue/Saturation、Exposure 与全部滤镜）宽 400，按参数依次排列：数值参数一行（标签在左、输入框在右，下方一条细轨道和三角标记，占 52）；选项参数不超过 4 项时为标签加其下每项一行的单选按钮（每行 24），超过 4 项时为标签右侧 200 宽的下拉框（占 36）；复选参数为一个复选框（占 28）。高度为 36 + 各行高度 + 20，至少 150。
- 打开时第一个输入框获得焦点并全选。

## 交互

- 输入框：任一参数超出范围或不是数字时，OK 置灰，也不预览。Levels 还要求输入黑场至少比白场小 2。
- 三角标记：在轨道上按下时选中最近的标记，拖动或单击把它移到指针位置。普通参数按轨道比例取值（按小数位数取整）；Levels 的黑白场按 0–255 取值；Levels 的 gamma 标记位于黑白场之间 `0.5^gamma` 的位置，拖动时按位置反算 gamma。
- Preview：勾选时文档实时显示结果，取消勾选时恢复原样。
- OK 或 Enter（所有值有效时）：返回 `Outcome::Apply(adjustment)`；Cancel 或 Esc：返回 `Outcome::Cancel`。
- 打开期间是模态的。

## 测试覆盖

- `defaults_match_photoshop`：Levels、Hue/Saturation 的默认调整，Exposure 输入框的默认文字。
- `filters_read_choices_and_checkboxes`：Add Noise 的分布与单色、Offset 的空白区域选项映射到对应的滤镜参数。
- `invalid_values_disable_the_dialog`：Levels 黑白场过近、Hue 超出范围时没有可应用的调整。
