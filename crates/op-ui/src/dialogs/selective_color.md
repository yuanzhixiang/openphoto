# dialogs/selective_color.rs：Selective Color 对话框

## 组件职责

Image › Adjustments › Selective Color... 的对话框，按 Photoshop 2026 的 UXP 对话框重做（473 × 400 pt）。算法见 `op-core` 的 `adjust.md`（`SelectiveColor`）。

## 数据与默认值

- 九个颜色范围（Reds、Yellows、Greens、Cyans、Blues、Magentas、Whites、Neutrals、Blacks）各有 Cyan、Magenta、Yellow、Black（−100–100%），默认 0；Method 默认 Relative。
- 输入框可带「%」，解析时去掉；`adjustment()` 在全部有效时返回 `Adjustment::SelectiveColor`。Preset 规则同其它 UXP 对话框。

## 布局（Photoshop 点）

- 「Preset」(20, 61)，下拉 (56, 48.5)–(302, 73.5)，预设菜单图标 (319, 61)。
- 九个圆点：圆心 x 从 32 起每 36 pt，y 96；填充为 Photoshop 的颜色（白、`#807f7f` 灰、黑表示 Whites、Neutrals、Blacks），悬停提示范围名。
- 四行滑块：Cyan、Magenta、Yellow、Black，输入框 x 277–331.5、y 120 / 175 / 230 / 285，数字后画「%」；轨道在框下 36 pt，x 20–331.5，颜色为红→灰→青、绿→灰→洋红、蓝→灰→黄、白→灰→黑（取自 Photoshop）。
- 「Method」(20, 348.5)，单选「Relative」(26, 368.5)、「Absolute」(98, 368.5)（`uxp::radio`）。
- 「Preview (Opt+P)」(344, 127)；右侧 OK、Cancel。

## 交互

- 点击圆点切换范围，Cyan 框获得焦点并全选；打开时 Reds 的 Cyan 框获得焦点。

## 测试覆盖

- `ranges_keep_their_values`；`ui_tests::channel_mixer_and_selective_color_apply`（Yellows、Absolute）；截图 `selective_color_dialog.png`。
