# dialogs/modify_selection.rs：Select › Modify 对话框

## 组件职责

Select › Modify 的 Border...、Smooth...、Expand...、Contract...、Feather...（⇧F6）共用的单值对话框。确定后由 `lib.rs` 修改当前选区并记录「Border」「Smooth」「Expand」「Contract」「Feather」。选区运算见 `op-core` 的 `selection.md`。

## 布局与视觉

按 Photoshop 对话框的结构排列（单位为 Photoshop 点，尚未逐像素比对）：

- 公共外框，标题分别为「Border Selection」「Smooth Selection」「Expand Selection」「Contract Selection」「Feather Selection」，宽 380。
- 一行：标签（「Width:」「Sample Radius:」「Expand By:」「Contract By:」「Feather Radius:」）、60 × 22 输入框（打开时获得焦点并全选，默认 1）、单位「pixels」。
- Smooth、Contract、Feather 还有「Apply effect at canvas bounds」复选框（默认不勾选）。
- 右侧 OK、Cancel；Enter 确定，Esc 取消。打开期间是模态的。

## 取值范围

Border 1–200；Smooth、Expand、Contract 1–500；Feather 0.1–1000。超出范围或不是数字时 OK 置灰。Smooth、Expand、Contract 的值四舍五入为整数。

## 已知限制

- 「Apply effect at canvas bounds」只对 Contract 生效；Smooth 与 Feather 在画布边缘的处理与 Photoshop 可能不同。
- 每次打开都恢复默认值 1，不记忆上次的值。

## 测试覆盖

- `ranges`：Border 超过 200 无效；Feather 接受 0.5。
