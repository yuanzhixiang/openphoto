# dialogs/save_changes.rs：「Save changes?」确认框

## 组件职责

关闭有未保存修改的文档（或退出、关闭窗口）时，询问是否先保存。回答由 `actions::answer_save_prompt` 处理（见 `actions.md`「关闭」）。

## 布局与视觉

按 Photoshop 确认框的结构排列（单位为 Photoshop 点，尚未逐像素比对）：

- 公共外框与标题栏（`common.md`），标题「OpenPhoto」，420 × 150。
- 正文：「Save changes to the OpenPhoto document “文档名” before closing?」，13 pt，左右各留 24，超长时换行。
- 底部一排按钮：左侧「Don't Save」，右侧「Cancel」「Save」，按钮 96 × 24，距底边 24。

## 交互

- `show(ctx, title)` 每帧绘制，给出回答时返回 `SaveChoice`：`Save`、`DontSave`、`Cancel`。
- 点击按钮；Enter = Save；Esc = Cancel；⌘D = Don't Save（macOS 的惯例）。
- 打开期间是模态的（`AppState::modal_open()` 为真），菜单命令与快捷键不生效，⌘D 等按键留给它。

## 已知限制

- Photoshop 的确认框带应用图标、文字为「Adobe Photoshop document」，并以蓝色突出默认按钮；这里的按钮与其它对话框一致。
