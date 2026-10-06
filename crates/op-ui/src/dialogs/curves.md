# dialogs/curves.rs：Curves 对话框

## 组件职责

Image › Adjustments › Curves...（⌘M）的对话框，按 Photoshop 2026 的经典对话框重做（658 × 445 pt）。算法见 `op-core` 的 `adjust.md`（`Curves`、`curve_table`）。

## 数据与默认值

- 四条曲线：RGB、Red、Green、Blue，各为按输入排序的点列，默认两个端点 (0, 0)、(255, 255)，最多 16 个点。
- 显示选项：Show Amount of 为 Light (0-255)（默认）或 Pigment/Ink %；Grid size 为四等分（默认）或十等分；Show 下 Channel Overlays、Histogram、Baseline、Intersection Line 默认都勾选；Show Clipping 不勾选。
- `adjustment()`：未改动的通道不参与（空点列）。
- Preset：未改动时「Default」，否则「Custom」；菜单的 Default 恢复默认。

## 布局（Photoshop 点）

- 「Preset:」(11, 56.25)，弹出菜单 (56, 46)–(345, 67)，齿轮 (365.5, 56.5)（只画出）。
- 左侧分组框 (11, 87.5)–(368, 434.5)，「Channel:」(30.5, 86.75)，弹出菜单 (85, 76.5)–(170, 97.5)。
- 工具：点工具按钮 (20, 107.5)–(50, 133.5)（选中，`#383838` 底），铅笔 (64.5, 120)（只画出）。
- 曲线图 (89, 108)–(346, 365)：`#454545` 底；当前通道的直方图（`#868686`）；网格线 `#383838`；对角基线 `#808080`；其它通道改动过的曲线按通道颜色画 1 pt（Channel Overlays）；当前曲线 1.75 pt（RGB 白色，其余为通道颜色）；控制点为 4 pt 方块，选中的实心。拖动点时画过该点的横竖交叉线（Intersection Line）。
- 左侧输出渐变 (84, 108)–(87.5, 365)，上白下黑；下方输入渐变 (89, 366)–(346, 369.5)，左黑右白；其下两枚端点针（尖端 y 369.5）。Pigment 时两条渐变、针与坐标都反过来。
- 「Output:」(20.5, 339.75) 与其值 (20.5, 355.75)；「Input:」(90, 389.75) 与其值 (89.5, 410.25)：显示选中点，否则显示指针所在处；Pigment 时以油墨百分比显示（`100 − v/2.55`）。
- (34, 412) 目标调整手形、(158 / 188 / 218, 411.5) 三个吸管（只画出）；「Show Clipping」复选框 (240.5, 404.5)。
- 右侧三个分组：「Show Amount of:」(394, 46.5)–(547, 122.5)，两个单选 (410, 71.25)、(410, 105.25)；「Grid size:」(394, 147.5)–(547, 203.5)，两个 25 pt 按钮 (416, 181)、(442, 181)；「Show:」(394, 228)–(547, 351)，四个复选框 x 402.5、y 246 / 272.75 / 299.5 / 326.25。
- 按钮 x 563.5–647.5：OK (45)、Cancel (80)、Smooth (122，不可用)、Auto (164)、Options... (199)；「Preview」(563.5, 243.5)。

## 交互

- Channel 菜单或 ⌥2–⌥5 切换通道，选中点清空。
- 曲线图：按下时 5 pt 内有点就选中它，否则在指针处加一个点（曲线跳到这个点）；拖动时横坐标限制在相邻两点之间；拖到图外 20 pt 以外松开时删除（端点不删）。
- 选中点时：方向键移动 1（Shift 10），Delete / Backspace 删除（端点除外）。
- 拖动下方端点针移动首、末点的输入值（不越过相邻点）。
- Auto：RGB 恢复默认，三个通道各自以最暗、最亮 0.1% 为端点。
- Enter / OK 应用，记录「Curves」；Esc / Cancel 取消。

## 已知限制

- 铅笔模式与 Smooth、吸管、目标调整、Show Clipping 的效果、Options...、齿轮菜单、Input/Output 的数值输入没有实现。

## 测试覆盖

- `channels_keep_their_curves`、`pigment_shows_ink_percent`、`auto_moves_each_channels_end_points`。
- `ui_tests::curves_dialog_adds_points`、`ui_tests::levels_and_curves_edit_one_channel`（⌥4 后只改绿色）。截图 `curves_dialog.png`、`curves_green.png`。
