# ui_tests.rs：无窗口 UI 测试

## 职责

用 egui_kittest 在没有窗口的环境下运行整个应用（包括 wgpu 画布），用来模拟输入、检查状态，并把画面渲染成 PNG 与 Photoshop 截图逐像素比对。只在测试中编译。

## 约定

- `harness(files)`：以 Photoshop 默认窗口大小（1350×800 pt）启动应用，不安装 macOS 原生菜单（`OpenPhotoApp::new_headless`）。因为没有原生菜单，⌘ 快捷键由 egui 处理（见 `commands.md`）。每步推进 1/60 秒（kittest 默认是 1/4 秒），与真实帧间隔一致，双击才能落在 egui 的 0.3 秒双击判定内。
- 像素密度设为每 pt 2 个物理像素，与拍摄 Photoshop 参考截图的 Retina 屏一致，所以 100% 缩放的画布大小也一致。
- `shot(harness, name)`：把当前画面写到 `target/ui-shots/{name}.png`（2700×1600），同时写一份缩小到每 pt 1 像素的 `{name}_1x.png`（1350×800），与 Photoshop 窗口的 1:1 截图坐标完全对应，可以直接裁剪比较。
- `reference_document(harness)`：关闭所有文档，打开一个与 Photoshop 参考截图中相同尺寸和文件名的文档（734×811，未嵌入配置文件），用于对比截图。
- `click(harness, pos)`：在某个位置模拟一次左键单击。`at_pt(x, y)` 把 Photoshop 点数换算成 egui 坐标。

## 测试

- 普通测试（`cargo test` 会运行）：验证交互与状态：
  - 点击工具栏前景色块打开 Color Picker，Esc 取消且不改动颜色。
  - 用矩形选框拖动得到对应范围的选区并记录历史，Shift 拖动添加选区，单击取消选区；⇧⌘D 重新选择、⌘A 全选、⌘D 取消选择。
  - ⇧M 切换到椭圆选框，之后按 M 仍选中椭圆选框（格的记忆）。
  - B 选中画笔，`]` 把 30 px 增大到 35 px，数字键改变不透明度；拖动一笔后经过的像素变为前景色，没经过的不变，并记录「Brush Tool」。
  - 图层隐藏时用画笔单击，弹出 Photoshop 的提示。
  - ⌥⌫、⌘⌫ 用前景色、背景色填充选区；普通图层上 ⌫ 清除选区；没有选区时 ⌫ 删除图层。
  - ⇧F5 打开 Fill 对话框，Enter 后填充前景色。
  - 油漆桶单击填充整片连通区域，记录「Paint Bucket」。
  - 移动工具拖动图层上的色块，方向键和 Shift+方向键微移，分别记录「Move」和「Nudge」；没有选区时拖动背景图层弹出锁定提示。
  - 点选项栏的 Auto-Select 复选框后，在上层空图层为当前图层时拖动下层色块，会自动选中并移动下层；打开 Show Transform Controls 后拖动框的角点进入自由变换。
  - 选区填红后 ⌘C、粘贴（以 egui-winit 实际送出的 `Event::Paste` 模拟 ⌘V）：得到「Layer 1」，叠在原位置，选区被取消，记录「Paste」；在新图层上 ⌘X 剪掉一半，记录「Cut」；复制只有透明像素的区域时弹出「selected area is empty」提示。
  - ⌘J 在背景图层上得到「Layer 1」、在普通图层上得到「Layer 1 copy」；⌘[ 后移一层并记录「Layer Order」；⌘E 向下合并；⇧⌘E 合并所有可见图层。
  - Layers 面板中拖动最上面一行到中间，顺序改变并记录「Layer Order」；拖到背景图层下面无效；双击名称输入新名字后 Enter，记录「Rename Layer」；点击背景图层的锁图标，背景变为「Layer 0」。
  - 顺时针旋转 90° 后宽高互换并记录「Rotate Canvas」；没有选区时 Crop 不可用，有选区时裁剪到选区；白底上画红色块后用 Trim（默认按左上角颜色）裁到红色块。
  - ⌘I 反相（#141414 → #ebebeb）并记录「Invert」；打开 Threshold 后文档立即预览为白色，Esc 取消后恢复，历史不变；Posterize 按 Enter 用默认 4 级应用并记录「Posterize」。
  - ⌘L 打开 Levels，在已聚焦的输入黑场输入 20 后文档预览为黑色，Enter 后记录「Levels」。
  - 白色方块上应用 Gaussian Blur（在聚焦的半径输入框输入 4，Enter），边缘变为中间值；之前 Last Filter 不可用，之后 ⌃⌘F 再应用一次并新增一条「Gaussian Blur」历史。
  - 修改后标签标题以「 *」结尾；`save_to` 保存为 PSD 后标题变为新文件名、「*」消失；撤销到保存前算修改，重做回来不算；修改后按 F12 恢复为文件内容并记录「Revert」；关闭有修改的文档时弹出确认，Esc 取消后文档仍在，⌘D（Don't Save）后文档关闭。
  - 吸管单击取红色为前景色，⌥单击取背景色，5×5 平均跨越方块角落时得到中间色；魔棒单击方块得到方块选区（外圈因消除锯齿半选）；多边形套索点三个角后 Enter 生成三角形选区；套索拖出三角形轨迹后生成选区。
  - G 选中渐变工具；黑到白从 x=100 拖到 x=600 后，左端为黑、右端为白、中间约为 128，记录「Gradient」（同时生成 `gradient` 截图）。
  - 背景图层无选区时 ⌘T 弹出锁定提示；新图层上 ⌘T 后在框内拖动，预览立即可见，Enter 后记录「Free Transform」；再次 ⌘T 拖右下角控制点放大约 2 倍后 Esc，图层不变；⇧⌘T 再平移一次并记录「Transform Again」（同时生成 `free_transform` 截图）。
  - C 选中裁剪工具，裁剪框覆盖整个画布；拖右下角控制点到 (500, 500) 后 Enter，画布约为 500×500 并记录「Crop」，框重新覆盖新画布；切换到 M 后裁剪框消失（同时生成 `crop` 截图）。
  - ⌥⌘I 打开 Image Size，在已聚焦的宽度输入 367 后高度按比例变为 406，Enter 后文档为 367×406，记录「Image Size」（同时生成 `image_size` 截图）。
  - ⌘R 显示标尺；从上标尺拖到画布内得到水平参考线并记录「New Guide」；⌘' 显示网格（生成 `rulers` 截图）；移动工具把参考线往下拖并记录「Move Guide」；拖回标尺删除并记录「Delete Guide」，⌘Z 恢复；⌥⌘; 锁定参考线。
  - 没有选区时 Expand 不可用；100..140 的选区扩展 5 后为 95..145，记录「Expand」（生成 `expand_selection` 截图）；⇧F6 打开 Feather；纯色文档上 Similar 选中整幅图像。
  - O 减淡让暗灰背景变亮并记录「Dodge Tool」；S 仿制图章未设取样点时弹出提示，⌥单击红色方块后在别处单击画出红色；Y 历史记录画笔把该处恢复为打开时的颜色。
  - 红色图层上用 Hide Selection 建蒙版，选区处露出背景，记录「Add Layer Mask」（生成 `layer_mask` 截图）；在蒙版上填黑隐藏更多而图层像素仍为红；停用蒙版后全部显示；Apply 后蒙版消失、对应像素变透明。
  - 有选区时按 Q 进入快速蒙版，记录「Quick Mask」，标题含「(Quick Mask/8」，选区外显示为红色；用画笔在选区中涂黑后再按 Q，得到排除该处的选区（生成 `quick_mask` 截图）。
  - U 选中矩形工具，拖动生成「Rectangle 1」并记录「Rectangle Tool」；⇧U 切到椭圆，按住 ⌥ 拖动以按下处为中心生成「Ellipse 1」（生成 `shapes` 截图）。
  - T 选中文字工具，单击后输入「Hello」、Enter、「World」，期间单键快捷键不生效，图层实时显示为「Hello」（生成 `type_tool` 截图）；⌘Enter 后记录「Type Tool」，基线上方有白色像素；另起一段输入后按 Esc，文字被丢弃。
  - ⌘N 打开 New 对话框（生成 `new_document` 截图），Enter 后创建 1920×1080 的「Untitled-2」；透明背景内容创建「Layer 1」普通图层，分辨率按设置。
  - 红色背景上 ⌥⇧⌘B 打开 Black & White 并按默认预设应用得到 102 灰；Gradient Map 用黑到白；打开 Photo Filter（生成 `photo_filter` 截图）。
  - ⌘M 打开 Curves，在曲线图 (128, 192) 处单击加点（生成 `curves` 截图），Enter 后 128 灰变为约 192，记录「Curves」。
  - F8 打开 Info，再打开 Navigator 与 Histogram，三个面板互不重叠（生成 `floating_panels` 截图）；悬停时记录指针的文档坐标；`center_on` 改变视图；再按 F8 关闭 Info。
- `active_canvas_pixel(harness, x, y)` 读取显示用的画布像素（含快速蒙版的红色）；`alt_click(harness, pos)` 模拟一次 ⌥单击。
- `run_command(harness, command)` 直接执行一个命令（用于没有快捷键的菜单项）；`double_click(harness, pos)` 模拟一次双击；`drag(harness, from, to, modifiers)` 模拟一次带修饰键的拖动；`doc_point` 把文档像素换算成屏幕坐标；`layer_pixel(harness, layer, x, y)` 读取某个图层（自底向上的序号）的像素。
- `dragging_a_layer_off_the_canvas_and_reveal_all`：用移动工具把新图层上的红色方块拖到画布左边外 100 px，画布内看不到；拖回来后像素复原；再拖出去后执行 Image › Reveal All，画布向左扩大 90 px，方块出现在新画布左边，记录「Reveal All」。
- `new_layer_dialog_names_colors_and_blends`：见 `dialogs/new_layer.md`。
- `duplicate_layer_and_layer_from_background_dialogs`：见 `dialogs/duplicate_layer.md`。
- `double_clicking_the_background_asks_for_a_name`：双击背景图层名称弹出 Layer from Background 版本的 New Layer 对话框，Esc 后背景图层不变。
- `flatten_asks_before_discarding_hidden_layers`、`rename_layer_and_alerts`：见 `dialogs/alert.md`。
- `layers_multi_selection`：单击 + ⌘ 单击选中两层，⌘E 合并为「Merge Layers」；⇧ 单击选范围，⌘, 隐藏、再按显示；⌥⌘A 选中所有非背景图层，底部删除按钮全部删除；Deselect Layers 后没有选中图层。
- `align_buttons_line_up_selected_layers`：只有一个图层时对齐不可用；⌥⌘A 选中三个带方块的图层后，点选项栏第一个对齐按钮，三者左边对齐并记录「Align Left Edges」；Distribute Vertical Centers 记录对应历史。
- `layer_groups_in_the_layers_panel`：⌥⌘A 后 ⌘G 编组并记录「Group Layers」；点箭头折叠、展开（不记录历史）；把组里的图层拖到组上方移出组并记录「Layer Order」；选中组 ⇧⌘G 取消编组；底部文件夹按钮新建空组「Group 1」。`screenshot_layer_groups`（`#[ignore]`）用于与 Photoshop 的嵌套组截图比对。
- `deleting_a_group_asks_what_to_delete`：删除组时弹出三选一询问（文字与选项和 Photoshop 一致）；点 Group Only 后图层留下并移出组；Enter（Group and Contents）全部删除；⌘J 复制组得到「Group 1 copy」及其中的图层。
- `merge_group_and_reverse`：⌥⌘A 后 Arrange › Reverse 反转三个图层并记录「Reverse」；⌘G 编组后 ⌘E 合并为一个名为「Group 1」的像素图层并记录「Merge Group」。
- `lock_layers_dialog`、`cmd_slash_toggles_lock_all`：见 `dialogs/lock_layers.md`。
- `layers_panel_footer_and_lock_buttons`：点 Layers 底部的「新建图层」按钮（距右边 58.25 pt）得到「Layer 1」；点锁定透明像素和锁定位置两个按钮，两个标记打开、记录「Lock Layer」；点全部锁定后其它按钮点击无效，再关掉全部锁定时原来的两个单项锁定恢复；防止自动嵌套按钮切换自己的标志；背景图层的锁定按钮点击无效；选中 Layer 1 后点删除按钮（距右边 30.25 pt）删除它，记录「Delete Layer」。
- 布局回归测试 `layout_matches_photoshop_2026`（不忽略，随 `cargo test` 运行）：渲染默认工作区（参考文档、移动工具、白色前景），在 `PHOTOSHOP_PIXELS` 表里约 60 个点上比较灰度与 Photoshop 2026 的 2x 截图（容差 6）。这些点都选在实测的 1 pt 边上：折叠条与分隔条的线、面板组间隔、标签栏底线、图标列与工具栏的分隔条、选项栏分隔线、Home 图标、色块边框、Color 面板的区域边缘、Properties 的框与输入框、面板菜单图标等，所以任何一处布局移动 1 pt 都会失败，并列出所有不一致的点。重新校准某处布局后，要从新的 Photoshop 截图更新这张表。
- 截图测试（标记为 `#[ignore]`，只生成图片、不做断言）：用 `cargo test -p op-ui ui_tests -- --ignored` 运行，用于和 Photoshop 对比外观。
- `brightness_contrast_dialog_types_and_applies`、`color_balance_tones_keep_their_values`、`hue_saturation_ranges_and_colorize`：重做的 UXP 调整对话框的输入、切换色调/范围、Colorize 与应用结果（辅助函数 `in_dialog` / `click_dialog` 按居中对话框的 Photoshop 点坐标点击，`color_document` 建单色文档）。`screenshot_adjustment_dialogs` 也截取 Brightness/Contrast、Color Balance、Curves。
- `levels_and_curves_edit_one_channel`：Levels 用 ⌥3 只改红色通道的黑场，Curves 用 ⌥4 只抬高绿色通道。`curves_dialog_adds_points` 按新的曲线图位置点击。
- `channel_mixer_and_selective_color_apply`：Channel Mixer 的红色输出改为 50% 红；Selective Color 选 Yellows、Cyan 100、Absolute，结果等于核心算法。
- `small_uxp_adjustment_dialogs_apply`：依次用 Vibrance、Posterize、Exposure、Black & White 的第一个输入框输入并应用，再用 Photo Filter 的默认值应用，结果都等于核心算法。
- `equalize_asks_about_the_selection`：见 `dialogs/equalize.md`。
- `more_filters_from_the_menu`：Blur、Blur More、Sharpen、Sharpen More、Find Edges、Despeckle、Sharpen Edges 直接执行并成为 Last Filter；Motion Blur、Emboss、Surface Blur、Dust & Scratches、Trace Contour、Wind 的对话框以默认值应用（若对话框的 OK 置灰，Enter 不会应用，测试即失败）。
- `filter_dialogs_remember_their_last_values`：Gaussian Blur 输入 4 后应用，再次打开时从 4 开始；另一种滤镜（Box Blur）仍是自己的默认值。
- `screenshot_filter_dialogs`（`#[ignore]`）：截取 Gaussian Blur、Add Noise、Offset、Unsharp Mask、Motion Blur、Minimum、Emboss、Surface Blur、Dust & Scratches、Mosaic、Box Blur、Custom、Trace Contour 的经典对话框，以及按 Photoshop 截图时的数值（120、−40、70）打开的 Twirl、Pinch、Spherize 与 Polar Coordinates、Wind 插件式对话框，用于与 Photoshop 并排比对。
- `custom_filter_types_a_kernel_and_remembers_it`：Custom 对话框在左上角格子输入 2 后 Enter 应用，记录「Custom」，再次打开时保留该核。
- `distort_filters_from_the_menu`：四个扭曲滤镜的对话框以默认值应用并成为 Last Filter。
- `dragging_on_a_large_document_keeps_up`：在 3000 × 1080 的文档上用移动工具、画笔、橡皮擦拖动 20 帧，平均每帧 UI 更新须少于 40 ms（目前约 10 ms，修复前移动工具约 56 ms），防止合成、图层外框扫描或缩略图再次变慢。
- `layers_panel_footer_and_lock_buttons` 另外检查：Lock all 打开时只有锁按钮有按下底色，其余按钮没有；此时拖动 Opacity、Fill 不起作用，关掉后拖动 Opacity 生效。
- `tool_flyout_matches_photoshop_and_switches_tools`：右键移动工具，弹出列表的外框在 (36, 92)，130 × 40（误差 1 pt）；点第二行切到 Artboard Tool 并关闭列表。`screenshot_tool_flyout`（同时截出修复工具组的弹出列表 `heal_flyout.png`，用于与 Photoshop 比对 Remove Tool 图标）（`#[ignore]`）用于与 Photoshop 截图比对。
- `align_buttons_line_up_selected_layers` 另外检查：⌘/ 锁定选中的图层后，Align 与 Distribute 变为不可用（缓存的可用状态随之更新）。
- `tool_icons_match_photoshops_extents`：69 个工具图标的亮像素外框与 Photoshop 测得的外框相差不超过 2 px（见 `tool_icons.md`）。`compare_tool_icons_with_photoshop`（`#[ignore]`）逐个与本地的 Photoshop 截图比对。
- `options_bars_match_photoshops_layout`：按实测布局的选项栏，分隔线与各种框的边缘与 Photoshop 的数值（`PS_BAR_MARKS`）逐一对应，误差 1 pt。`screenshot_options_bars`（`#[ignore]`）截出每个工具的选项栏用于比对。
- `selection_options_bars_edit_their_settings`：在选框栏的 Feather 输入 5 并回车生效、点第二个运算按钮切到 Add；魔棒 Tolerance 改为 60、Contiguous 取消勾选；磁性套索的 Frequency 改为 80 后保存在 `tool_settings`。
- `brush_options_bars_edit_their_settings`：见 `options_tools.md`。
- `fill_and_sample_options_bars_edit_their_settings`：见 `options_tools.md`。
- `type_shape_and_view_options_bars_edit_their_settings`：见 `options_tools.md`。
- `canvas_size_dialog_resizes_around_the_anchor`：Canvas Size 打开时宽度框已全选，输入 800、点左上锚点、回车：文档变为 800 × 811，图像留在左边、右侧扩展为背景色（白），记录「Canvas Size」；勾选 Relative 后高度输入 10 得到 821；输入后按 Esc 不做修改。`dialog_origin` 从标题栏底色找出对话框左上角；`probe_document` 打开与 Photoshop 测量用的 probe.png 相同的 64 × 72 透明文档。`screenshot_canvas_size_dialog`（`#[ignore]`）把对话框截成 `canvas_size.png`，用于与 Photoshop 并排比对。
- `screenshot_layer_drag_and_rename`（`#[ignore]`）：拖动中途（被拖行的底色与落点横线）和改名输入框的截图，用于与 Photoshop 比对。
- `properties_sections_toggle_views_and_run_quick_actions`、`new_guide_dialog_adds_a_guide`、`type_shape_and_view_options_bars_edit_their_settings`：见各自的 spec。`shot_dialog` 把打开的对话框按 Photoshop 截图的尺寸裁下（`screenshot_fill_dialog`、`screenshot_trim_dialog`、`screenshot_feather_dialog`、`screenshot_new_guide_dialog`、`screenshot_canvas_size_dialog`）。
- `new_document_dialog_presets_recent_and_saved`：New Document 打开后点 Photo 页、选 Landscape, 6 x 4、点存储图标存为预设、Create：文档为 1800 × 1200、300 ppi；再打开时 Recent 第一个就是它，Saved 页列出存下的预设；Close 不新建文档。`screenshot_new_document_dialog`（`#[ignore]`）截出 Recent 与 Photo 页，`new_document_origin` 从标题栏颜色找出对话框位置。
- `brush_modes_and_eraser_block`：画笔 Mode 设为 Multiply 后在 `#141414` 上画红色得到 `#140000`；橡皮擦 Block 在背景图层上擦出屏幕 16 像素见方的白色方块。
- `holding_a_toolbar_button_opens_its_flyout`：短按不弹出；按住 0.5 秒弹出，松开后仍打开且没有切换工具；点第二行切到 Artboard 并关闭。
- `lasso_with_alt_draws_straight_edges`：套索拖一段后按住 ⌥ 松开鼠标没有选区，⌥ 单击加一个角，松开 ⌥ 后闭合成选区，记录「Lasso」。
- `sampling_scopes_and_the_sampling_ring`：顶层为空、下面一层有红色方块时，吸管 Current Layer 取不到颜色，Current & Below 取到红色；按住吸管时取样环上半是红色、下半是原来的蓝色；仿制图章 Sample 为 All Layers 时把红色仿制到空图层上。
