# crates/op-mcp/src/main.rs：OpenPhoto 的 MCP 服务器

## 职责

让 AI Agent 通过 MCP（Model Context Protocol）编辑图像。`op-mcp` 是一个独立的可执行程序（二进制名 `op-mcp`），通过标准输入输出说 JSON-RPC 2.0，每行一条消息。它不带界面，也不连接正在运行的 OpenPhoto 窗口：文档只存在于这个进程的内存里，进程退出即消失（需要先 `save_image` / `export_composite`）。

所有编辑都调用应用本身使用的 `op-core` / `op-io` 函数（调整、滤镜、图层、选区、画布、填充、历史记录），不重新实现，所以结果与在 OpenPhoto 里操作一致。

启动：`cargo run -p op-mcp`。客户端配置示例见仓库 `README.md` 的「MCP server」一节。

## 对外接口：只有两个 MCP 工具

不把一百多个操作逐个注册成 MCP 工具（那样会占满客户端的上下文），而是只暴露两个：

- `search_tools`：参数 `query`（必填，关键词）与 `limit`（1–50，默认 10）。在内部工具目录里按关键词打分，返回匹配的工具名、说明和完整的参数 JSON Schema（`inputSchema`），以及 `count` 和 `total_tools`。`query` 为空、`*`、`all`、`list`、`list all` 时按目录顺序列出前 `limit` 个。
- `call_tool`：参数 `name`（必填，内部工具名）与 `arguments`（对象，可省略）。执行该内部工具。

用法是先搜后调：`search_tools {"query": "gaussian blur"}` 得到 `filter_gaussian_blur` 及其参数说明，再 `call_tool {"name": "filter_gaussian_blur", "arguments": {"radius": 2}}`。

### 搜索打分

查询与每个工具的「名称 + 关键词 + 说明」比较，按分数从高到低排序，同分按名称排序，只返回分数大于 0 的工具：

- 去掉非字母数字后，工具名与查询完全相同 +200，工具名包含查询 +100；说明包含整个查询 +20。
- 查询按非字母数字切成词，长度 < 3 的词忽略。每个词取它在工具词表里的最好匹配：完全相同 +15，工具词包含查询词 +7，查询词包含长度 ≥ 5 的工具词 +5（如「blurring」匹配「blur」；限制长度是为了不让 per、ion 之类的短词误配）；查询词正好是该工具的关键词时再 +10。

## 协议处理

- `initialize`：返回客户端请求的 `protocolVersion`（没有时 `2024-11-05`）、`capabilities.tools`（`listChanged: false`）、`serverInfo`（`openphoto-mcp` 与包版本）和一段说明用法的 `instructions`。
- `ping`：返回空对象。`tools/list`：返回上面两个工具的定义。
- `tools/call`：`search_tools` / `call_tool` 之外的名称返回 JSON-RPC 错误 -32601。内部工具执行失败时不是 JSON-RPC 错误，而是正常结果里 `isError: true`，`content` 是 `{"error": "原因"}`；成功时 `content` 是格式化的 JSON 文本。`call_tool` 缺少 `name` 时同样以 `isError` 返回。
- 以 `notifications/` 开头的方法和没有 `id` 的消息不回复。其它未知方法有 `id` 时返回 -32601。
- 无法解析的一行返回 `id: null` 的 -32700 错误；空行跳过。读 stdin 出错或写 stdout 出错时退出。

## 文档与状态

- 进程里可同时打开多个文档，各自带一份 `History`。新建或打开后成为「当前文档」；每个内部工具都接受可选的 `doc`（文档 id，即 `Document` 的 id 数字的字符串），省略时作用于当前文档，没有文档时报错。
- `close_document` 关闭后，若关的是当前文档，当前文档换成剩下文档中的任意一个（`HashMap` 顺序）。
- 图层用 `layer`（图层 id 数字）指定，省略时为活动图层。

## 内部工具目录（107 个）

- **文件与文档**：`new_document`（宽高默认 800 × 600，背景色 `fill` 默认白）、`open_image`、`save_image`（`.psd` 保留图层，其它扩展名存合成图）、`export_composite`、`list_documents`、`get_document_info`、`close_document`、`set_active_document`、`get_pixel`（合成图或单个图层的一个像素）、`get_histogram`（活动图层在选区内的 RGB 直方图）。
- **调整**（`adjust_*`，记录与应用相同的历史名称）：Invert、Desaturate、Threshold、Posterize、Equalize（含 Entire Image）、Levels（复合与 R/G/B 各通道）、Hue/Saturation（主控与六个色彩范围、Colorize）、Exposure、Brightness/Contrast（含 Legacy）、Color Balance、Black & White、Vibrance、Photo Filter、Gradient Map、Auto Tone / Contrast / Color、Curves（每通道最多 16 点）、Channel Mixer、Selective Color。各参数的取值范围与对应对话框一致，超出时报错。
- **滤镜**（`filter_*`）：Gaussian Blur、Box Blur、Average、Unsharp Mask、Add Noise、Median、Minimum、Maximum、Blur、Blur More、Sharpen、Sharpen More、Sharpen Edges、Despeckle、Find Edges、Motion Blur、Emboss、Fragment、Custom（5 × 5 卷积核）、Surface Blur、Dust & Scratches、Trace Contour、Wind、Twirl、Pinch、Spherize、Polar Coordinates、High Pass、Offset、Mosaic、Solarize。可选 `background`（[r, g, b]）是需要背景色的滤镜所用的背景色，默认白。
- **图层**：`list_layers`、`set_active_layer`、`new_raster_layer`（透明或纯色）、`new_group`、`duplicate_layer`、`delete_selected_layers`、`rename_layer`、`set_layer_visibility`、`set_layer_opacity`、`set_layer_fill`、`set_layer_blend_mode`、`arrange_layer`、`merge_down`、`merge_visible`、`flatten_image`、`group_selected_layers`、`ungroup_layers`、`add_layer_mask`、`convert_background_to_layer`、`align_layers`、`distribute_layers`、`layer_via_copy`、`layer_via_cut`。
- **选区**：`select_all`、`clear_selection`、`reselect_selection`、`select_rect`、`select_ellipse`（可关消除锯齿）、`feather_selection`（半径 0.1–1000）。
- **画布**：`rotate_flip_canvas`、`rotate_canvas_arbitrary`、`resize_canvas`（锚点 `anchor_x` / `anchor_y` 0–2，默认居中）、`crop_image`、`crop_to_selection`、`resize_image`（重采样方法默认 Automatic）、`trim_image`、`reveal_all`。
- **编辑**：`fill_solid`（颜色必填、混合模式、不透明度、保留透明区域）、`clear_layer`、`paint_bucket`（点击位置与颜色必填，容差、消除锯齿、连续、所有图层）、`gradient_fill`（两点、两色、`kind` 线性/径向/角度/对称/菱形、不透明度、反向）、`move_active_layer`（移动工具：移动图层或选中的像素）。
- **历史记录**：`history_undo`、`history_redo`、`list_history`。

每个工具返回 JSON：编辑类返回文档 id、宽高、活动图层和 `revision`，以及操作特有的字段；查询类返回所查的内容。

## 校验与限制

- 参数类型不对（如把字符串传给数值）或缺少必填参数时报错，不执行。各工具 schema 的 `required` 与处理函数实际要求的参数一致（例如 `fill_solid`、`paint_bucket` 的 `color`）。
- 宽高上限与应用一致：`new_document`、`resize_image`、`resize_canvas` 的宽高必须是 1–30000 像素，避免一次调用申请几十 GB 内存而使进程崩溃。
- 滤镜参数按应用里滤镜对话框的范围检查（例如 Gaussian Blur、High Pass、Unsharp Mask 半径 0.1–1000，Box Blur 1–2000，Median 1–500，Minimum / Maximum 0.2–500，Motion Blur 角度 −360–360、距离 1–2000，Unsharp Mask 数量 1–500，Add Noise 数量 0.1–400，Offset 位移 ±30000，Mosaic 单元 2–200）。服务器在一个线程里依次处理请求，过大的半径会让它长时间不响应，所以超出范围直接报错。
- `gradient_fill` 没有插值方法（op-core 的渐变工具没有这个选项，只有渐变映射有），传 `method` 会报错并提示改用 `adjust_gradient_map`。

## 历史记录

- 新建或打开文档时以「New」或「Open」开始历史。
- 每个改动文档的成功操作记录一步，名称与应用相同（如「Gaussian Blur」「Merge Down」「Canvas Size」）。失败的操作不记录；`arrange_layer`、`trim_image`、`reveal_all` 没有改动时也不记录。
- 选区变化也是历史步骤，与应用和 Photoshop 一致：「Select All」「Deselect」（没有选区时取消选择不记录）、「Reselect」「Rectangular Marquee」「Elliptical Marquee」「Feather」。所以在框选之后撤销只撤掉选区，不会连带撤掉之前的编辑。
- 历史快照是整个文档（含选区），`history_undo` / `history_redo` 在快照间切换。

## 已知限制

- 文档只在进程内存里，与正在运行的 OpenPhoto 窗口互不相通；进程退出前未保存的修改会丢失。
- `set_layer_visibility` 会记录「Show Layer」/「Hide Layer」历史，而应用里切换眼睛不记录历史。
- `get_pixel` 每次都重新合成整张图，对大图较慢。
- `initialize` 原样返回客户端请求的协议版本，不做协商；不支持 JSON-RPC 批量请求（数组形式的消息会被忽略、不回复）。
- 没有画笔、文字、形状、蒙版绘制等需要拖动或输入的工具，也没有资源（resources）与提示（prompts）。

## 测试覆盖

`cargo test -p op-mcp`：

- `search_finds_blur_and_returns_definitions`、`search_empty_query_lists_tools`、`search_unknown_query_returns_empty`：搜索打分与返回的定义。
- `new_invert_and_pixel_roundtrip`、`gaussian_blur_and_undo_redo`、`canvas_and_layer_ops_roundtrip`：编辑、撤销重做、画布与图层操作、导出。
- `mcp_initialize_list_call_roundtrip`：协议层的 initialize、tools/list、search_tools 与 call_tool（未知工具以 `isError` 返回）。
- `selections_are_history_states`：模糊后框选再撤销，只撤掉选区，模糊保留；没有选区时取消选择不新增历史。
- `sizes_and_filter_parameters_are_limited`：超大尺寸与超范围的滤镜参数报错且不记录历史，范围内的仍然生效。
- `schemas_list_what_handlers_require`：`fill_solid`、`paint_bucket` 的 schema 把 `color` 列为必填；`gradient_fill` 传 `method` 报错。
