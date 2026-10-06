# actions.rs：文件操作与单键快捷键

## 职责

文档的打开、新建、导出、关闭，以及单键快捷键（工具切换、默认颜色、交换颜色）。带修饰键的快捷键属于命令，见 `commands.md`。

## 打开

- `open_dialog`：系统文件对话框，可多选，过滤为 `op_io::OPEN_EXTENSIONS` 列出的格式（PNG、JPEG、WebP、TIFF、BMP、GIF）。
- `open_paths`：逐个打开，成功的作为新文档加入并成为当前文档，第一条历史为「Open」。失败时写日志，并通过 `alert` 弹出「Could not open “路径”: 原因」。
- 打开普通位图时整张图是一个图层，与 Photoshop 一致：完全不透明的图片是锁定的「Background」背景图层，带透明的图片是普通的「Layer 0」图层。

## 新建

`new_document`：1920×1080、白色背景、72 ppi、RGB/8，标题依次为「Untitled-1」「Untitled-2」……，第一条历史为「New」。目前没有 Photoshop 的「New Document」对话框，参数固定。

## 导出

`export_dialog`（File › Export › Export As...）：系统保存对话框，默认文件名为去掉扩展名的文档标题加 `.png`，可选 PNG 或 JPEG。导出的是所有可见图层的合成结果；PNG 保留透明，JPEG 的透明区域铺成白色（细节见 `crates/op-io/src/lib.md`）。失败时通过 `alert` 提示「Could not export: 原因」，不会留下不完整的文件。

## 关闭

- Close：关闭当前文档。
- Close All：关闭全部文档。
- Close Others：关闭当前文档以外的全部文档。
- 关闭时同时移除标签和文档状态（`AppState::close_document`）。目前没有「是否保存更改」的确认（也还没有保存功能）。

## 单键快捷键（`handle_tool_keys`）

- 模态对话框打开时、或文本输入框获得焦点时不处理。
- 只响应不带 ⌘/Ctrl/Alt 的按下事件，忽略按键重复。
- D：恢复默认颜色（前景黑、背景白）；X：交换前景色与背景色。
- 其它字母：按 `op_tools::Tool::from_shortcut` 切换工具，对应关系与 Photoshop 一致（V M L W C K I J B S Y E G O P T A U H Z）。

## 已知限制

- Photoshop 里按住 Shift 加字母键可以在同组工具间循环，这里没有。
- 不处理 Q（快速蒙版）、F（屏幕模式）、R（旋转视图）等尚未实现功能的快捷键。
