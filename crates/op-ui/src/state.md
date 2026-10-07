# state.rs：应用状态与文档状态

## 职责

保存界面层的全部可变状态。`AppState` 是整个应用的状态，`DocState` 是单个打开文档的状态（文档本身、历史记录、视图、缓存）。

## AppState

- `docs` / `doc_order` / `active_doc`：打开的文档、标签顺序与当前文档。`add_document` 把文档加到最后并设为当前文档；`close_document` 关闭文档，关闭的是当前文档时，它左边的文档（没有则第一个）成为当前文档。
- `tool`：当前工具，默认是矩形选框（与 Photoshop 新装后的默认一致）。
- `tool_slots`：工具栏每一格当前显示的工具（该组最近用过的那个），初始为各组第一个。`select_tool(t)` 同时设置当前工具和它所在格显示的工具。
- `foreground` / `background`：前景色默认 `#14a5dc`，背景色白色。
- `view`：视图开关（`ViewOptions`，见 `rulers.md`）。其中 `smart_guides`（智能参考线，默认开）只由 Properties 面板的 Guides 分区切换，还没有效果。
- `modify_dialog`：Select › Modify 对话框，打开期间为 `Some`。
- `new_document_dialog`：New 对话框，打开期间为 `Some`。
- `new_guide_dialog`：New Guide 对话框，打开期间为 `Some`。
- `marquee`：选框工具选项：组合方式、羽化（像素）、消除锯齿（默认开启，与 Photoshop 一致）、样式。前三项在创建选区时生效，样式中的 Fixed Ratio / Fixed Size 目前没有效果。
- `dodge`、`burn`、`sponge`、`blur`、`sharpen`、`clone_stamp`、`history_brush`：修饰工具各自的 `PaintOptions`（默认 30 px、硬度 0%；减淡/加深的不透明度即 Exposure 50%，模糊/锐化的不透明度即 Strength 50%，海绵的流量 50%）。`retouch`（`RetouchOptions`）：减淡与加深各自的 Range（默认 Midtones）、海绵是否 Saturate（默认否）、仿制图章 Aligned（默认是）。`paint_options(tool)` 也返回这些工具的设置。
- `brush` / `pencil` / `eraser`：各绘画工具的 `PaintOptions`（大小 px、硬度、不透明度、流量），与 Photoshop 一样每个工具单独保存。默认值：画笔与橡皮擦 30 px、硬度 0%、不透明度和流量 100%；铅笔 1 px。`paint_options(tool)` 取当前绘画工具的设置。`size_step(size)` 是 `[`、`]` 的步长（小于 10 为 1，10–50 为 5，50–100 为 10，100–200 为 25，200–300 为 50，再往上为 100），大小范围 1–5000。
- `eyedropper`：吸管选项（取样大小 `size`，1 表示 Point Sample；`all_layers`）。`EyedropperOptions::SIZES` 是 Photoshop 的七个取样大小。
- `type_options`：文字工具选项（见 `type_tool.md`）。
- `typing_text()`：当前文档有正在输入的文字时为真；此时 `modal_open()` 也为真。
- `shape`：形状工具选项（`ShapeOptions`：多边形边数 5、直线粗细 1 px，Photoshop 的默认值）。
- `move_options`：移动工具选项（`MoveOptions`：`auto_select`、`show_transform_controls`，都默认关闭，与 Photoshop 一致）。
- `gradient`：渐变工具选项（`op_core::gradient::GradientOptions`：类型、混合模式、不透明度、反向）。
- `wand`：魔棒选项：组合方式与区域规则（`op_core::fill::BucketOptions` 的容差、消除锯齿、连续、所有图层，默认值同油漆桶）。
- `editing_background` 与 `picker_hsb`：Color 面板正在编辑前景还是背景，以及缓存的 HSB。缓存 HSB 是为了在灰色（饱和度为 0）时色相不跳回 0。
- `floating`：浮动面板（Info、Navigator、Histogram）是否打开（见 `panels/floating.md`）。
- `history_open`、`history_panel`：History 弹出面板是否打开，以及它的标签与高度。
- `fill_dialog`：Fill 对话框，打开期间为 `Some`。
- `bucket`：油漆桶选项（默认值见 `crates/op-core/src/fill.md`）。
- `canvas_size_dialog`：Canvas Size 对话框，打开期间为 `Some`。
- `color_picker`：Color Picker 会话，打开期间为 `Some`，包含对话框和 `PickerTarget`（确定后写入前景色、背景色、Canvas Size 的扩展颜色，还是 Fill 对话框的 Color...）。`open_color_picker(target)` 以 Photoshop 的标题打开前景或背景色拾色器。
- `swatches`：Swatches 面板的色板列表，初始为固定的 36 个颜色，Color Picker 的「Add to Swatches」会往末尾追加。只在本次运行中保留。
- `alert`：待显示的错误信息。
- `delete_group_prompt`：删除含图层的组时的询问框；`delete_layers()` 删除选中图层，遇到非空组时先弹出它（「Delete the group “名字” and its contents or delete only the group?」，Group and Contents / Group Only / Cancel）。命令 DeleteLayer 和 Layers 面板的删除按钮都走它。
- `flatten_prompt`：Flatten Image 的「Discard hidden layers?」询问框（显示期间算作模态对话框）；`skip_flatten_prompt`：其中勾选了「Don’t show again」，本次运行不再询问。
- 关闭与退出：`close_queue` 是等待关闭的文档；`save_prompt` 是正在询问「Save changes?」的文档；`quit_after_close` 表示队列处理完后要退出；`quit_approved` 表示可以关闭窗口了（见 `actions.md`「关闭」）。
- `clipboard`：Cut/Copy/Paste 用的剪贴板（见 `clipboard.md`）。默认不连接系统剪贴板（无窗口测试不应改动用户的剪贴板），`OpenPhotoApp::new` 启动时换成连接系统剪贴板的版本。
- `typing`：上一帧是否有输入框获得键盘焦点，每帧执行命令前更新。决定菜单的 Cut/Copy/Paste 作用于输入框还是文档。
- `forward_events`：要注入 egui 下一帧输入的事件（菜单的 Cut/Copy/Paste 转交给输入框时使用，见 `commands.md`）。
- `image_size_dialog`：Image Size 对话框，打开期间为 `Some`。
- `trim_dialog`：Trim 对话框，打开期间为 `Some`。
- `adjust_dialog`：调整或滤镜对话框（见 `dialogs/adjust.md`），打开期间为 `Some`。
- `last_transform`：上次应用的变换映射，供 Edit › Transform › Again 使用；只在本次运行中保留，所有文档共用。
- `transforming()`：当前文档正在自由变换时为真；此时 `modal_open()` 也为真。
- `last_filter`：上次成功应用的滤镜及其设置，供 Filter › Last Filter 使用；只在本次运行中保留，所有文档共用。
- `filter_settings`：每种滤镜对话框上次按 OK 时输入框里的设置（按 `AdjustKind` 存），下次打开同一对话框时放回；只在本次运行中保留，所有文档共用。
- `modal_open()`：Canvas Size、Image Size、New、New Guide、Modify、Fill、Trim、调整对话框、Color Picker、「Save changes?」确认或错误提示打开时为真，此时命令与单键快捷键都不执行。

## DocState

- `doc`：`op_core::Document`。
- `path`：文档打开自或最近保存到的文件；新建且未保存的文档为 `None`。
- 未保存修改：`saved_state` 记录与磁盘文件（新建文档则为新建时）一致的历史状态 id。`is_dirty()` 在当前历史状态的 id 与它不同时为真（所以撤销回保存时的状态视为没有修改，与 Photoshop 一致）；`mark_saved()` 把当前状态记为已保存。
- `untagged`：文档没有嵌入色彩配置文件（标签标题里显示「#」）。打开的文件为真，新建的文档为假。
- `history`：该文档的 `op_core::History`。第一条状态名为「Open」（打开文件）或「New」（新建）。
- `view`：视图状态，见下文。
- 合成缓存：`canvas_image()` 返回当前合成结果，只在文档 `revision` 变化时重新合成。
- 图层缩略图：`layer_thumbnail()` 按图层缓存，文档 `revision` 变化后重新生成；最近邻缩小。
- 移动：`move_drag` 是进行中的移动和拖动起点（文档像素）。
- `transform_controls_bounds()`：移动工具 Show Transform Controls 的框（文档像素），即 `op_core::transform::bounds` 的结果，不能变换时为 `None`；按（修订号、当前图层、选区修订号）缓存。
- 绘画：`stroke` 是进行中的笔画和它的工具；`last_paint_point` 是上一笔结束的位置，用于 Shift+单击画直线。
- `text_edit`：正在输入的文字（见 `type_tool.md`）。
- `shape_drag`：形状工具拖动中的起点与当前点。
- `gradient_drag`：渐变工具拖动中的起点与当前点（文档像素）。
- `clone_source`、`clone_offset`、`picking_clone_source`：仿制图章的取样点、对齐偏移，以及「这次按压是在设定取样点」的标记（见 `document_view.md`）。
- `pointer`：指针在文档上的位置（文档像素），指针不在画布上时为 `None`；Info 面板使用。
- `movable_layers()`：Align / Distribute 会移动的选中图层数，按文档修订号、选区修订号和选中图层缓存。对齐按钮和菜单项的可用状态每帧要问十几次，每次都要扫描图层像素的外框，不缓存时拖动大文档会明显卡顿。
- 直方图与合成缩略图：`composite_histogram()`、`composite_texture(ctx, max_px)`，都按文档修订号缓存，供 Histogram 与 Navigator 面板使用。两者都取画布已有的合成图（`canvas_image()`），不再另外合成一遍；Quick Mask 打开时（画布带红色遮罩）Navigator 缩略图仍单独合成。
- `guide_drag`：正在拖动的参考线（从标尺拖出时 `index` 为 `None`，否则为被移动参考线的序号）。
- `crop`：裁剪工具的裁剪框（见 `crop_tool.md`）。
- `free_transform`：自由变换会话（见 `free_transform.md`）。
- `lasso`：正在绘制的套索轨迹（文档像素坐标的点、组合方式、是否为多边形套索）。
- 画布图像（`canvas_image`）：快速蒙版模式下在合成结果上叠加红色，未选中处 50%（Photoshop 默认的「被蒙版区域」显示），按灰度值线性变化；只影响显示。
- `sample_average(x, y, size, scope)`：吸管取样（`SampleScope`：当前图层、当前及下方、全部），见 `document_view.md`。
- `renaming`：Layers 面板中正在改名的图层和输入中的文字（见 `panels/layers.md`）。
- 选框拖动：`marquee_drag` 保存拖动中的起点、当前点（文档像素）、组合方式，以及 Shift/⌥ 是否已用于选择组合方式。
- 蚂蚁线轮廓：`selection_outline()` 按选区版本号缓存轮廓线段。
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
- `new_document_recent`、`new_document_saved`、`new_document_welcome_closed`：New Document 对话框的 Recent 列表（新的在前、不重复、最多 20 个）、用存储图标存下的预设、Recent 页的欢迎框是否关过；只在本次运行内保留（见 `dialogs/new_document.md`）。
