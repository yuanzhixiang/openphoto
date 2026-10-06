# dialogs/alert.rs：macOS 样式的提示框

## 组件职责

Photoshop 的错误提示和确认询问都是 macOS 原生提示框（NSAlert）。本组件按 Photoshop 2026 中的两种提示框（「Could not complete the Copy command because the selected area is empty.」与 Flatten Image 的「Discard hidden layers?」）实测的样子绘制，用于 `AppState::alert` 的所有错误提示和需要确认的询问。

## 数据

- `Alert { message, icon, cancel, dont_show_again }`：消息；图标（`App` 应用图标，或 `Caution` 警告三角加应用小图标）；是否有 Cancel；「Don’t show again」复选框及是否勾选（`None` 表示没有）。
- `Alert::error(message)`：应用图标、只有 OK。`Alert::caution(message)`：警告图标、Cancel 与 OK、带复选框。
- `Alert::choose(message, choices)`：警告图标，按钮为一列横跨整宽的选项（第一个为蓝色默认按钮），见下文。
- `show(ctx, alert)` 返回 `Option<Answer>`：`Ok { dont_show_again }`、`Cancel` 或 `Choice(序号)`；还没回答时为 `None`。

## 布局（Photoshop 2026 实测，相对提示框左上角的 pt）

- 宽 260，底色 `#b3b3b3`，0.5 pt `#ebebeb` 边，圆角 16，带阴影；居中显示，不压暗背后。
- 应用图标：(26.5, 26.5) 起 51 pt 见方。OpenPhoto 自己的图标（深绿底圆角方块、浅绿「Op」），不是 Photoshop 的。警告图标：(22, 24)–(80, 76) 的黄色三角（白边、白色感叹号），右下 (55, 55) 起是 27 pt 的应用小图标。
- 消息：粗体系统字体 14 pt、颜色 `#1c1c1c`，从 x 22.5 开始，宽度超过 216 时换行；第一行大写字母从 y 103 开始，行距 16。
- 复选框（有时）：消息下方 13 pt，16 pt 见方圆角 4，未勾选 `#9f9f9f`、勾选为蓝底白勾；「Don’t show again」在框右 7 pt，中等字重 14 pt。点击框或文字切换。
- 按钮：在最后一行消息下方 13 pt（有复选框时在复选框下方 16 pt），高 28 的胶囊，系统字体 14 pt。只有 OK 时 OK 横跨 (16, …)–(244, …)；有 Cancel 时 Cancel (16–126，`#a4a4a4` 底、深色字)、OK (134–244，`#3478f6` 底、白字)。按下时颜色变暗。
- 竖排选项（`choices` 非空时）：每个按钮 28 pt 高、横跨 (16, …)–(244, …)，间距 34 pt，第一个蓝底白字，其余灰底；与 Photoshop 删除组时的「Group and Contents / Group Only / Cancel」一致。
- 高度随消息行数变化：按钮下边再留 16 pt。

## 交互

- Enter：OK（竖排选项时为第一个）。Esc：竖排选项时为最后一个，有 Cancel 时为 Cancel，否则为 OK。点击按钮同理。
- 显示期间是模态的，菜单和快捷键不生效。

## 已知限制

- 警告三角是直角折线，macOS 的是圆角渐变图形。
- 不是真正的 NSAlert（为了能在无窗口测试中运行，并与其它对话框一样由 egui 绘制）。

## 测试覆盖

- `constructors`（单元测试）：两种构造的默认值。
- `flatten_asks_before_discarding_hidden_layers`（UI 测试）：有隐藏图层时 Flatten Image 先询问；Esc 取消后两个图层都在；勾选「Don’t show again」后点 OK 拼合为一个背景图层，之后再拼合不再询问。
- `rename_layer_and_alerts`（UI 测试）：报错提示按 Enter 关闭（同时覆盖 Rename Layer...，见 `commands.md`）。
- `screenshot_alerts`（截图，`#[ignore]`）：两种提示框，用于与 Photoshop 的截图比对。
