# dialogs/hue_saturation.rs：Hue/Saturation 对话框

## 组件职责

Image › Adjustments › Hue/Saturation...（⌘U）的对话框，按 Photoshop 2026 的新版 UXP 对话框重做（437 × 413 pt）。算法见 `op-core` 的 `adjust.md`。

## 数据与默认值

- Master 与六个颜色范围（Reds、Yellows、Greens、Cyans、Blues、Magentas）各有 Hue（−180–180）、Saturation、Lightness（−100–100）三个值，默认 0；每个范围的边界默认为 Photoshop 的 `HUE_RANGES`。
- Colorize 有自己的三个值：Hue（0–360，打开时为前景色的色相）、Saturation（0–100，25）、Lightness（−100–100，0）。勾选 Colorize 时用这三个值，取消时恢复原来的 Master 与范围设置。
- `adjustment()`：当前所有值都有效时返回 `Adjustment::HueSaturation`。
- Preset：没有任何改动时显示「Default」，否则「Custom」；下拉菜单的 Default 把所有值与范围恢复默认（Colorize 的值保留）。Photoshop 的其它预设（Cyanotype、Sepia 等）还没有。

## 布局

- 「Preset」标签 (20, 61)，下拉框 (56, 48.5)–(266, 73.5)，其右的预设菜单图标 (284, 61)（只画出）。
- (32.5, 95) 为目标调整手形图标（只画出，没有行为）。
- 七个 24 pt 圆点，圆心 y 96，x 从 68 起每 36 pt 一个：Master 为八等分色轮，其余为 Photoshop 的范围颜色；选中的画法同 Color Balance。Colorize 时只剩一个选中的圆点，填充 Colorize 的颜色（该色相与饱和度、亮度 50%）。
- 三行滑块：标签 x 20；输入框 x 251–295.5、y 120 / 175 / 230；轨道 y 156 / 211 / 266，x 20–296。轨道颜色：Hue 为色相环，Saturation 为灰→彩，Lightness 为黑→灰→白（均取自 Photoshop 截图）；Colorize 时 Hue 轨道从左端 0 起，Saturation 为 `#767676` 到该色相的纯色。
- 选中一个范围时，Hue 输入框左侧显示范围中心的色相（例如 Reds 为「0°」）。
- 「Colorize」复选框 (20, 292)；三个吸管 (123.5 / 160 / 196, 298) 与「Invert」按钮 (233, 286)–(296, 310)，都画成不可用。
- 「Before - After」标签 (20, 331)；色带 (20, 343)–(296, 356)：上半为原色相（青在左端、红在中间），下半为当前设置作用后的颜色；其下 y 364–366 为 `#a0a0a0` 横线。
- Master 或 Colorize 时，横线下方左右显示不可用色（`#8e8e8e`）的「0° / 0°」。选中范围时横线上画范围条：两段渐变区为 8 pt 高 `#7a7a7a`，完全作用区为 14 pt 高 `#a0a0a0`，四个白色把手（外侧 2 pt 宽、内侧 3 pt 宽，14 pt 高）；下方左右为「起点°/满强度起点°」与「满强度终点°/终点°」，放不下时以「...」截断。
- 右侧 OK（没有输入框持有焦点时画成持有焦点的样子）、Cancel；「Preview (Opt+P)」(308, 127)。

## 交互

- 点击圆点切换 Master 或范围，滑块与输入框显示该项的值。
- 范围条：在把手 4 pt 内按下拖动该边界（不越过相邻边界）；在其它位置拖动整体平移范围。
- Colorize 勾选或取消时回到 Master。
- 其余同 UXP 对话框（`uxp.md`）。

## 已知限制

- 吸管（在文档上取色选范围、加减范围）、Invert、目标调整手形与预设菜单都没有行为；Photoshop 在选中范围时吸管与 Invert 可用。
- 预设列表只有 Default / Custom。

## 测试覆盖

- `values_ranges_and_preset`：范围各自的值、无效值禁用 OK、Colorize 的值与前景色相、Preset 的 Default/Custom。
- `the_strip_puts_red_in_the_middle`：色带上 0° 在中点、180° 在左端。
- `ui_tests::hue_saturation_ranges_and_colorize`：选 Reds 输入色相 60 应用到红色文档；再用 Colorize 应用，结果等于核心算法。截图 `hue_saturation_reds.png`、`hue_saturation_colorize.png`。
