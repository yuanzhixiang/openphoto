# dialogs/save_changes.rs：「Save changes?」确认框

## 组件职责

关闭有未保存修改的文档（或退出、关闭窗口）时，询问是否先保存。回答由 `actions::answer_save_prompt` 处理（见 `actions.md`「关闭」）。

## 布局与视觉

与 Photoshop 2026 一样是 macOS 的提示框，直接用 `alert.rs` 的 `Alert::choose` 绘制（尺寸与配色见 `alert.md`）：

- 警告三角加应用徽标的图标。
- 正文（13 pt 系统粗体）：「Save changes to the OpenPhoto document “文档名” before closing?」（Photoshop 是「Adobe Photoshop document」）。
- 三个整宽按钮自上而下：Save（默认按钮，蓝色）、Don’t Save（弯撇号，与 Photoshop 一致）、Cancel。

与 Photoshop 截图并排比对一致（Photoshop 的截图是窗口未激活时的灰色默认按钮）。

## 交互

- `show(ctx, title)` 每帧绘制，给出回答时返回 `SaveChoice`：`Save`、`DontSave`、`Cancel`。
- 点击按钮；Enter = Save；Esc = Cancel；⌘D = Don’t Save（macOS 的惯例）。
- 打开期间是模态的（`AppState::modal_open()` 为真），菜单命令与快捷键不生效，⌘D 等按键留给它。

## 已知限制

- 文字是「OpenPhoto document」而不是 Photoshop 的「Adobe Photoshop document」。
