# dialogs/duplicate_layer.rs：Duplicate Layer 对话框

## 组件职责

Layer › Duplicate Layer... 弹出的对话框：给活动图层的副本命名，并选择放到哪个文档（当前文档、另一个打开的文档或新文档），对应 Photoshop 2026 的 Duplicate Layer 对话框。

## 布局（Photoshop 2026 实测，相对对话框左上角的 pt）

对话框 447 × 208，居中显示；标题「Duplicate Layer」为 13 pt 粗体（见 `theme.md`）。文字为 AppKit 的系统字体 12 pt（`theme::dialog`，opsz 17），标签 `#d7d7d7`、值 `#f1f1f1`、不可用 `#888888`；标签都右对齐到 x 81.5（各段文字的位置与宽度与 Photoshop 相差不超过 1 px）。

- 「Duplicate:」中心 y 45.5，右边 x 90.5 是活动图层的名称。
- 「As:」中心 y 70.5；输入框 (86, 61)–(361, 80)，打开时获得焦点并全选，默认值为「原名 copy」（`layer_ops::duplicate_name`）。
- 「Destination」分组框 (10.5, 96.5)–(361, 197.5)：1 pt `#424242` 框线，标题在 x 31 处断开框线。
  - 「Document:」中心 y 121.5；下拉 (86, 111)–(351.5, 132)，`#454545` 底、1 pt `#666666` 边，值在左 8.5，距右边 7.5 是 V 形箭头；值太长时截断并加「...」（Photoshop 截断处比这里多一两个字符）。菜单列出当前文档、其余打开的文档和「New」。
  - 「Artboard:」与下拉 (86, 140)–(351.5, 161)：始终不可用，显示「Canvas」（本应用没有画板）。
  - 「Name:」与输入框 (86, 169)–(351.5, 188)：只有选择「New」时可用，默认值为下一个「Untitled-N」；否则为不可用的空框。
- 按钮：OK (377.5, 39)–(437, 64) 为默认按钮（亮边），Cancel (377.5, 74)–(437, 99)；标签为 AppKit 系统字体 12 pt（与 New Layer 对话框的粗体 Adobe Clean 不同，这是 Photoshop 两类对话框的差别）。

## 交互

- Enter 或 OK：按选择复制（`actions::duplicate_layer`）：
  - 当前文档：`layer_ops::duplicate_named`，记录「Duplicate Layer」。
  - 另一个打开的文档：`layer_ops::duplicate_into`，副本在目标文档的活动图层上方、像素位置不变，目标文档记录「Duplicate Layer」；当前文档保持不变，仍是活动文档。
  - New：`layer_ops::duplicate_to_new` 生成新文档并打开为活动文档，初始历史状态为「Duplicate Layer」，Untitled 计数加一。
  - 「As」为空时用原名。
- Esc 或 Cancel：关闭，不做任何事。

## 已知限制

- 没有画板，Artboard 总是不可用。
- 「As」输入框获得焦点时是 2 pt 的蓝色外圈，Photoshop 这里是 1 pt 的蓝色边框。

## 测试覆盖

- `destinations`（单元测试）：默认目标是当前文档，选「New」后目标为新文档标题。
- `duplicate_layer_and_layer_from_background_dialogs`（UI 测试）：输入「Copy A」回车复制到当前文档；再打开对话框，从 Document 菜单选「New」点 OK，新文档打开、只有「Copy A copy」；回到第一个文档用 Layer from Background... 输入「Base」确认，背景变为普通图层「Base」并记录「Layer From Background」。
- `screenshot_duplicate_layer_dialog`（截图，`#[ignore]`）：用于与 Photoshop 的截图比对。
