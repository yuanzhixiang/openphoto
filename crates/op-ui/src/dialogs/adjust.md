# dialogs/adjust.rs：调整与滤镜对话框

## 组件职责

带设置的调整与滤镜对话框：Image › Adjustments 的 Threshold...、Posterize...、Levels...、Hue/Saturation...、Exposure...，以及 Filter 菜单的 Gaussian Blur...、Box Blur...、Unsharp Mask...、Add Noise...、Median...、Minimum...、Maximum...、High Pass...、Offset...、Mosaic...。Levels...、Curves...、Brightness/Contrast...、Color Balance...、Hue/Saturation... 已按 Photoshop 2026 重做，布局与设置在各自的模块里（`levels.md`、`curves.md`、`brightness_contrast.md`、`color_balance.md`、`hue_saturation.md`），本模块只为它们画窗口框与标题、转交 OK/Cancel 并处理预览。对话框只管理设置与 Preview 开关，预览与应用由 `lib.rs` 完成（见 `lib.md`「调整与滤镜对话框的接入」）。像素算法见 `op-core` 的 `adjust.md` 与 `filter.md`。

对话框的结果是 `Effect`：`Adjustment(Adjustment)` 或 `Filter(Filter)`。`Effect::name()` 是历史名称，`Effect::apply(doc, background)` 调用对应的 `op-core` 函数。

## 数据输入

- `AdjustDialog::new(kind, histogram, before)`；`colors` 字段为 Gradient Map 的两种颜色（打开时设为前景色与背景色）。
- `AdjustDialog::new(kind, histogram, before)`：`kind` 为对话框种类；`histogram` 是打开时活动图层选区内的直方图（Threshold 用亮度）；`before` 是打开时的文档快照。
- 每个设置是一个「参数」：标签、范围、默认值、小数位数，以及类型——数值（输入框 + 滑块）、选项（单选按钮，值为选中项的序号）、复选（值为 0 或 1）。输入框中的文字按小数位数格式化。调整的默认值与范围同 Photoshop；滤镜的默认值尚未与 Photoshop 核对：

| 对话框 | 参数（范围，默认值） |
|---|---|
| Threshold | Threshold Level（1–255，128） |
| Posterize | Levels（2–255，4） |
| Exposure | Exposure（−20.00–20.00，0.00）、Offset（−0.5000–0.5000，0.0000）、Gamma Correction（0.01–9.99，1.00） |
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
- 其余对话框（Exposure、Black and White、Vibrance、Photo Filter、Gradient Map 与全部滤镜）宽 400，按参数依次排列：数值参数一行（标签在左、输入框在右，下方一条细轨道和三角标记，占 52）；选项参数不超过 4 项时为标签加其下每项一行的单选按钮（每行 24），超过 4 项时为标签右侧 200 宽的下拉框（占 36）；复选参数为一个复选框（占 28）。高度为 36 + 各行高度 + 20，至少 150。
- 打开时第一个输入框获得焦点并全选。

## 重做的对话框（`Custom`）

- `AdjustDialog` 对这五种对话框持有各自的状态（`Custom::Levels`、`Curves`、`BrightnessContrast`、`ColorBalance`、`HueSaturation`），`effect()` 取自它们，`show` 用它们的尺寸并调用它们的 `ui`；窗口框与标题栏仍是 `common::frame`（标题用 AppKit 13 pt 粗体）。
- 它们返回 `uxp::Button`：OK → `Outcome::Apply`，Cancel → `Outcome::Cancel`，Brightness/Contrast 的 Auto → 按直方图算出 Auto 的取值（见 `brightness_contrast.md`）。
- `set_channel_histograms(h)`：打开 Levels、Curves 时由 `commands.rs` 传入红绿蓝三个通道的直方图。
- `set_colorize_hue(hue)`：打开 Hue/Saturation 时由 `commands.rs` 传入前景色的色相，作为 Colorize 的初始色相（Photoshop 的行为）。

## 交互

- 输入框：任一参数超出范围或不是数字时，OK 置灰，也不预览。
- 三角标记：在轨道上按下或拖动时把它移到指针位置，按轨道比例取值（按小数位数取整）。
- Preview：勾选时文档实时显示结果，取消勾选时恢复原样。
- OK 或 Enter（所有值有效时）：返回 `Outcome::Apply(adjustment)`；Cancel 或 Esc：返回 `Outcome::Cancel`。
- 打开期间是模态的。

## 测试覆盖

- `defaults_match_photoshop`：Levels、Hue/Saturation 的默认调整，Exposure 输入框的默认文字。
- `filters_read_choices_and_checkboxes`：Add Noise 的分布与单色、Offset 的空白区域选项映射到对应的滤镜参数。
