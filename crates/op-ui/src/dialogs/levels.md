# dialogs/levels.rs：Levels 对话框

## 组件职责

Image › Adjustments › Levels...（⌘L）的对话框，按 Photoshop 2026 的经典对话框逐像素重做（412 × 371 pt，文字与控件与 Photoshop 截图相差 1–2 像素）。算法见 `op-core` 的 `adjust.md`（`Levels`、`levels_gamma`）。

## 数据与默认值

- 四组色阶：RGB（复合）、Red、Green、Blue，各有输入黑场（0–253）、gamma（0.01–9.99）、输入白场（2–255）、输出黑场、输出白场（0–255），默认 0 / 1.00 / 255 / 0 / 255。每次打开都恢复默认。
- `new(channels)`：红、绿、蓝三个通道的直方图（`adjust::rgb_histograms`），复合直方图为三者之和。
- `adjustment()`：四组都有效且每组黑场比白场至少小 2 时返回 `Adjustment::Levels`。
- Preset：未改动时「Default」，否则「Custom」；菜单的 Default 恢复默认。Photoshop 的其它预设（Darker、Increase Contrast 等）还没有。

## 布局（Photoshop 点）

- 「Preset:」(11, 50)，弹出菜单 (59, 39.5)–(266, 60.5)，齿轮 (283.5, 50)（只画出）。
- Channel 分组框 (11, 90.5)–(295.5, 359.5)，标题处断开；「Channel:」(30.5, 90.5)，弹出菜单 (85, 80)–(206.5, 101)。
- 「Input Levels:」(20.5, 118.5)；直方图 (25, 131)–(281, 230)，`#454545` 底，每级 1 pt 宽的 `#d0d0d0` 竖条，按最大计数归一化；显示当前通道的直方图。
- 输入针：尖端 y 233，0 与 255 分别在 x 25.75 与 279.25；黑（空心）、灰、白三枚。灰针位于黑白针之间 `0.5^gamma` 处。
- 输入框 y 249–268：x 60–105、150–195、240–285。
- 分隔线 y 277，x 20–286，`#3e3e3e`。
- 「Output Levels:」(20.5, 290.5)；黑到白渐变条 (24, 302)–(282, 315)；输出针尖端 y 315；输出框 y 332–351，x 60–105、240–285。
- 右侧按钮 x 313.5–402.5：OK (38.5)、Cancel (73.5)、Auto (115.5)、Options... (157.5)，各 26 高。
- 三个吸管 (327 / 357 / 387, 213)（黑、灰、白，只画出）；「Preview」复选框 (312.5, 243)。

## 交互

- 打开时输入黑场框获得焦点并全选。
- Channel 菜单或 ⌥2–⌥5 切换 RGB、Red、Green、Blue；每个通道保留自己的值。
- 拖动针：在针所在的行按下时选中最近的针；黑场不超过白场 − 2，白场不低于黑场 + 2；灰针按位置反算 gamma（两位小数）。
- Auto：复合通道恢复默认，三个通道各自把最暗与最亮 0.1% 设为黑白场（Photoshop 的「Enhance Per Channel Contrast」；Photoshop 2026 的默认 Auto 算法与 Options... 对话框未实现）。
- Enter / OK 应用，记录「Levels」；Esc / Cancel 取消。

## 已知限制

- Options...（Auto Color Correction Options）、吸管、齿轮菜单（存储/载入预设）没有行为。

## 测试覆盖

- `channels_keep_their_levels`、`auto_stretches_each_channel`。
- `ui_tests::levels_dialog_sets_the_black_point`、`ui_tests::levels_and_curves_edit_one_channel`（⌥3 后只改红色）。截图 `levels_dialog.png`、`levels_red.png`。
