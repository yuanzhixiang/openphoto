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

- 普通测试（`cargo test` 会运行）：验证交互与状态，例如打开 Color Picker 后模态状态生效、Esc 取消且不改动颜色。
- 截图测试（标记为 `#[ignore]`，只生成图片、不做断言）：用 `cargo test -p op-ui ui_tests -- --ignored` 运行，用于和 Photoshop 对比外观。
