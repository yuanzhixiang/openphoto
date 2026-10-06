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
  - 选区填红后 ⌘C、粘贴（以 egui-winit 实际送出的 `Event::Paste` 模拟 ⌘V）：得到「Layer 1」，叠在原位置，选区被取消，记录「Paste」；在新图层上 ⌘X 剪掉一半，记录「Cut」；复制只有透明像素的区域时弹出「selected area is empty」提示。
  - ⌘J 在背景图层上得到「Layer 1」、在普通图层上得到「Layer 1 copy」；⌘[ 后移一层并记录「Layer Order」；⌘E 向下合并；⇧⌘E 合并所有可见图层。
  - Layers 面板中拖动最上面一行到中间，顺序改变并记录「Layer Order」；拖到背景图层下面无效；双击名称输入新名字后 Enter，记录「Rename Layer」；点击背景图层的锁图标，背景变为「Layer 0」。
  - 顺时针旋转 90° 后宽高互换并记录「Rotate Canvas」；没有选区时 Crop 不可用，有选区时裁剪到选区；白底上画红色块后用 Trim（默认按左上角颜色）裁到红色块。
  - ⌘I 反相（#141414 → #ebebeb）并记录「Invert」；打开 Threshold 后文档立即预览为白色，Esc 取消后恢复，历史不变；Posterize 按 Enter 用默认 4 级应用并记录「Posterize」。
  - ⌘L 打开 Levels，在已聚焦的输入黑场输入 20 后文档预览为黑色，Enter 后记录「Levels」。
  - 白色方块上应用 Gaussian Blur（在聚焦的半径输入框输入 4，Enter），边缘变为中间值；之前 Last Filter 不可用，之后 ⌃⌘F 再应用一次并新增一条「Gaussian Blur」历史。
- `run_command(harness, command)` 直接执行一个命令（用于没有快捷键的菜单项）；`double_click(harness, pos)` 模拟一次双击；`drag(harness, from, to, modifiers)` 模拟一次带修饰键的拖动；`doc_point` 把文档像素换算成屏幕坐标；`layer_pixel(harness, layer, x, y)` 读取某个图层（自底向上的序号）的像素。
- 截图测试（标记为 `#[ignore]`，只生成图片、不做断言）：用 `cargo test -p op-ui ui_tests -- --ignored` 运行，用于和 Photoshop 对比外观。
