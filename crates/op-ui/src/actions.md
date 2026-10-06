# actions.rs：文件操作与单键快捷键

## 职责

文档的打开、新建、保存、恢复、导出、关闭（含未保存修改的确认）与退出，剪贴板命令的执行，以及单键快捷键（工具切换、默认颜色、交换颜色）。带修饰键的快捷键属于命令，见 `commands.md`。

## 打开

- 打开成功后记下文档的文件路径（`DocState::path`）。

- `open_dialog`：系统文件对话框，可多选，过滤为 `op_io::OPEN_EXTENSIONS` 列出的格式（PNG、JPEG、WebP、TIFF、BMP、GIF）。
- `open_paths`：逐个打开，成功的作为新文档加入并成为当前文档，第一条历史为「Open」。失败时写日志，并通过 `alert` 弹出「Could not open “路径”: 原因」。
- 打开普通位图时整张图是一个图层，与 Photoshop 一致：完全不透明的图片是锁定的「Background」背景图层，带透明的图片是普通的「Layer 0」图层。

## 新建

`new_document`：1920×1080、白色背景、72 ppi、RGB/8，标题依次为「Untitled-1」「Untitled-2」……，第一条历史为「New」。目前没有 Photoshop 的「New Document」对话框，参数固定。

## 导出

`export_dialog`（File › Export › Export As...）：系统保存对话框，默认文件名为去掉扩展名的文档标题加 `.png`，可选 PNG 或 JPEG。导出的是所有可见图层的合成结果；PNG 保留透明，JPEG 的透明区域铺成白色（细节见 `crates/op-io/src/lib.md`）。失败时通过 `alert` 提示「Could not export: 原因」，不会留下不完整的文件。

## 关闭

- Close：关闭当前文档；Close All：关闭全部文档；Close Others：关闭当前文档以外的全部文档；标签上的「×」关闭该文档。
- 所有关闭都经过 `request_close(ids)`：把要关闭的文档放进 `close_queue`，`continue_closing` 依次处理：没有未保存修改的直接关闭（`AppState::close_document`，同时移除标签和文档状态）；有未保存修改的切换为当前文档，弹出「Save changes?」确认（`save_prompt`，见 `dialogs/save_changes.md`）并暂停。
- `answer_save_prompt`：Save → 执行 Save（可能弹出 Save As 对话框），成功后关闭并继续处理队列；Don't Save → 直接关闭并继续；Cancel，或 Save As 对话框被取消、保存失败 → 清空队列，停止关闭（也停止退出）。
- `quit`：Quit OpenPhoto 与关闭窗口：把全部文档放进队列并标记 `quit_after_close`；队列处理完时设置 `quit_approved`，`lib.rs` 随后关闭窗口、退出应用。

## 保存

- `save(id)`：File › Save。文档有文件、且文件能容纳它（PSD 总是可以；PNG/JPEG 等扁平格式只在只有一个图层时可以）时，直接写回该文件并标记为已保存；否则转到 Save As。返回是否保存成功。
- `save_as(id, copy)`：File › Save As...（`copy` 为假）与 Save a Copy...（为真）。系统保存对话框，默认文件名为标题去掉扩展名加 `.psd`，默认目录为文档所在目录，格式过滤器为 `op_io::SAVE_FORMATS`。取消时返回假。
- `save_to(id, path, copy)`：写入 `path`（失败时弹出「Could not save “路径”: 原因」）。之后：如果是副本，或所选格式无法容纳文档的图层（例如多图层文档存为 PNG，相当于 Photoshop 的「存储副本」），文档的文件、标题和已保存状态都不变；否则文档改用新文件，标题改为新文件名，并标记为已保存。
- `revert()`：File › Revert（F12）。重新读取文档的文件，用它的内容替换文档（尺寸、图层、选区等，通过快照恢复），记录一条「Revert」历史并标记为已保存。读取失败时弹出提示。

## 剪贴板（`clipboard`）

执行 Cut、Copy、Copy Merged、Paste、Paste in Place；输入框获得焦点时转交给输入框。完整规则见 `commands.md`「剪贴板」。

## 单键快捷键（`handle_tool_keys`）

- Q：进入/退出快速蒙版（与工具栏按钮相同）。

- 模态对话框打开时、或文本输入框获得焦点时不处理。
- 只响应不带 ⌘/Ctrl/Alt 的按下事件（可以带 Shift），忽略按键重复。
- 当前工具是画笔、铅笔或橡皮擦时，先处理绘画按键（与 Photoshop 一致）：
  - `[`、`]`：按 `PaintOptions::size_step` 的步长减小、增大笔刷。
  - Shift+`[`、Shift+`]`：硬度减少、增加 25%。
  - 数字键 1–9、0：不透明度设为 10%–90%、100%；Shift+数字键设置流量（铅笔没有流量）。
- 当前工具是移动工具时，方向键把当前图层（或选中的像素）移动 1 像素，Shift+方向键移动 10 像素，每次记录一条「Nudge」，与 Photoshop 一致。不能移动时弹出提示。
- D：恢复默认颜色（前景黑、背景白）；X：交换前景色与背景色。
- 其它字母：按 `op_tools::tool_for_key` 选择工具（规则见 `crates/op-tools/src/lib.md`），并让工具栏那一格显示所选工具。Shift+字母在同组工具间循环，与 Photoshop 一致。

## 已知限制

- 不处理 Q（快速蒙版）、F（屏幕模式）、R（旋转视图）等尚未实现功能的快捷键。
