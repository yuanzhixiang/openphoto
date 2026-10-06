# state.rs：应用状态与文档状态

## 职责

保存界面层的全部可变状态。`AppState` 是整个应用的状态，`DocState` 是单个打开文档的状态（文档本身、历史记录、视图、缓存）。

## AppState

- `docs` / `active_doc`：打开的文档与当前文档。当前文档由 `lib.rs` 根据 dock 焦点每帧更新。
- `tool`：当前工具，默认是矩形选框（与 Photoshop 新装后的默认一致）。
- `foreground` / `background`：前景色默认 `#14a5dc`，背景色白色。
- `marquee`：选框工具选项（选区运算模式、羽化、消除锯齿、样式），目前只用于选项栏显示与编辑，没有接到选区功能上。
- `editing_background` 与 `picker_hsb`：Color 面板正在编辑前景还是背景，以及缓存的 HSB。缓存 HSB 是为了在灰色（饱和度为 0）时色相不跳回 0。
- `history_open`、`history_panel`：History 弹出面板是否打开，以及它的标签与高度。
- `canvas_size_dialog`：Canvas Size 对话框，打开期间为 `Some`。
- `color_picker`：Color Picker 会话，打开期间为 `Some`，包含对话框和 `PickerTarget`（确定后写入前景色、背景色还是 Canvas Size 的扩展颜色）。`open_color_picker(target)` 以 Photoshop 的标题打开前景或背景色拾色器。
- `swatches`：Swatches 面板的色板列表，初始为固定的 36 个颜色，Color Picker 的「Add to Swatches」会往末尾追加。只在本次运行中保留。
- `alert`：待显示的错误信息。
- `modal_open()`：Canvas Size、Color Picker 或错误提示打开时为真，此时命令与单键快捷键都不执行。

## DocState

- `doc`：`op_core::Document`。
- `history`：该文档的 `op_core::History`。第一条状态名为「Open」（打开文件）或「New」（新建）。
- `view`：视图状态，见下文。
- 合成缓存：`canvas_image()` 返回当前合成结果，只在文档 `revision` 变化时重新合成。
- 图层缩略图：`layer_thumbnail()` 按图层缓存，文档 `revision` 变化后重新生成；最近邻缩小。
- 快照缩略图：创建 `DocState` 时生成一次文档初始状态的缩略图（最长边 96 px，最近邻），供 History 面板顶部的快照行使用，之后不再更新（快照代表打开时的文档）。

### 历史记录的接入规则

- `record(name)`：立即记录一条历史。
- 连续编辑（拖动不透明度、Fill 等）：编辑过程中 `mark_pending()`，编辑结束时 `commit_pending(name)`，只有确实发生过改动时才记录一条。这样一次拖动只产生一条历史。
- `undo` / `redo` / `toggle_last_state` / `jump_to_state` / `delete_states_from` 都会清除未提交的连续编辑标记。

### View

- `zoom`：每个文档像素对应的物理像素数，1.0 = 100%。
- `offset`：文档中心相对视口中心的偏移（逻辑点）。用中心而不是左上角做基准，窗口尺寸变化时文档保持居中，与 Photoshop 一致。
- `initialized` / `viewport`：首次显示时等视口尺寸稳定后再决定初始缩放（见 `document_view.md`）；`viewport` 供快捷键缩放使用。

## 已知限制

- 撤销会把图层显示/隐藏和当前选中图层一起恢复成快照里的样子；Photoshop 默认不记录这两项，撤销时也不会改变它们。
