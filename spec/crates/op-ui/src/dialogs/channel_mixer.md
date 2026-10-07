# dialogs/channel_mixer.rs：Channel Mixer 对话框

## 组件职责

Image › Adjustments › Channel Mixer... 的对话框，按 Photoshop 2026 的 UXP 对话框重做（401 × 426 pt）。算法见 `op-core` 的 `adjust.md`（`ChannelMixer`）。

## 数据与默认值

- 红、绿、蓝三个输出通道各有 Red、Green、Blue（−200–200%）与 Constant（−200–200%），默认为恒等（自身 100，其余 0）。
- Monochrome：勾选时改为编辑一组灰度混合，Photoshop 的初值 40 / 40 / 20 / 0。
- `adjustment()`：所有值有效时返回 `Adjustment::ChannelMixer`；Preset 在未改动时为「Default」，否则「Custom」，菜单的 Default 恢复默认。

## 布局（Photoshop 点）

- 「Preset」(20, 61)，下拉 (56, 48.5)–(231, 73.5)，预设菜单图标 (248, 61)（只画出）。
- 「Output channel」(20, 97)，三个圆点圆心 (121 / 150 / 178.75, 96)，画法同 Color Balance；Monochrome 时只剩一个选中的灰色圆点。
- 「Monochrome」复选框 (20, 127)；「Preview (Opt+P)」(272, 127)。
- 四行滑块：Red、Green、Blue、Constant，输入框 x 203–260、y 156 / 211 / 266 / 363，轨道在框下 36 pt，x 20–260。轨道颜色取自 Photoshop：黑→通道色→白（Constant 为黑→灰→白）。
- 「Total」(20, 330) 与三项之和（右对齐到 238.5）和「%」(250)；其下 y 350 一条 `#737373` 分隔线。
- 右侧 OK、Cancel。

## 交互

- 点击圆点切换输出通道；打开时 Red 输入框获得焦点并全选。其余同 UXP 对话框（`uxp.md`）。

## 已知限制

- Photoshop 的预设（Black & White with Red Filter 等）与 Total 超过 100% 时的警告图标没有。

## 测试覆盖

- `rows_and_monochrome`；`ui_tests::channel_mixer_and_selective_color_apply`；截图 `channel_mixer_dialog.png`。
