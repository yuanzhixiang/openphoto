# ui_tests.rs：无窗口 UI 测试

## 职责

用 egui_kittest 在没有窗口的环境下运行整个应用（包括 wgpu 画布），用来模拟输入、检查状态，并把画面渲染成 PNG 与 Photoshop 截图逐像素比对。只在测试中编译。

## 约定

- `harness(files)`：以 Photoshop 默认窗口大小（1350×800 pt）启动应用，不安装 macOS 原生菜单（`OpenPhotoApp::new_headless`）。因为没有原生菜单，⌘ 快捷键由 egui 处理（见 `commands.md`）。
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
- `drag(harness, from, to, modifiers)` 模拟一次带修饰键的拖动；`doc_point` 把文档像素换算成屏幕坐标。
- 截图测试（标记为 `#[ignore]`，只生成图片、不做断言）：用 `cargo test -p op-ui ui_tests -- --ignored` 运行，用于和 Photoshop 对比外观。
