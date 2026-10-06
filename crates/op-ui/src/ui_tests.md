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
- 截图测试（标记为 `#[ignore]`，只生成图片、不做断言）：用 `cargo test -p op-ui ui_tests -- --ignored` 运行，用于和 Photoshop 对比外观。
