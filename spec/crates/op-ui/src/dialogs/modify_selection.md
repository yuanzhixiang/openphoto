# dialogs/modify_selection.rs：Select › Modify 对话框

## 组件职责

Select › Modify 的 Border...、Smooth...、Expand...、Contract...、Feather...（⇧F6）共用的单值对话框。确定后由 `lib.rs` 修改当前选区并记录「Border」「Smooth」「Expand」「Contract」「Feather」。选区运算见 `op-core` 的 `selection.md`。

## 布局与视觉

按 Photoshop 2026 的五个对话框（UXP 样式，2x 截图）逐点量取，对话框都是 295 × 128 pt，坐标为距对话框左上角的 pt：

- 标题栏：`common::frame`，系统粗体 13 pt，标题分别为「Border Selection」「Smooth Selection」「Expand Selection」「Contract Selection」「Feather Selection」。文字为 12 pt 面板字体（`uxp::label`）。
- 一行：标签（Photoshop 2026 不带冒号：「Width」「Sample Radius」「Expand By」「Contract By」「Feather Radius」）左端 20.5、中心 y 60；输入框紧跟标签，在标签右 9 pt，36 × 24（y 48–72，文字左缩进 11.5 pt，打开时获得焦点并全选，默认 1）；其后 5.5 pt 是「pixels」。
- 五个对话框都有「Apply effect at canvas bounds」复选框 (20, 90)，默认不勾选（与 Photoshop 2026 一致）。
- OK（默认按钮）(205–275，y 48–72)、Cancel (y 84–108)：`common::ps_button`。Enter 确定，Esc 取消。打开期间是模态的。

标签用 Source Sans 3 近似 Adobe Clean，略宽（「Feather Radius」宽 2.5 pt），输入框随之右移；其余元素与 Photoshop 截图相差不超过 1 pt。

## 取值范围

Border 1–200；Smooth、Expand、Contract 1–500；Feather 0.1–1000。超出范围或不是数字时 OK 置灰。Smooth、Expand、Contract 的值四舍五入为整数。

## 已知限制

- 「Apply effect at canvas bounds」只对 Contract 生效；Border、Expand、Smooth 与 Feather 显示这个选项但忽略它，在画布边缘的处理与 Photoshop 可能不同。
- 每次打开都恢复默认值 1，不记忆上次的值。

## 测试覆盖

- `ranges`：Border 超过 200 无效；Feather 接受 0.5。
- `ui_tests.rs` 的 `modify_selection_and_grow`（Expand 5 px）；`screenshot_feather_dialog`（`#[ignore]`）截出 Feather 对话框用于与 Photoshop 比对。
