# op-render：crate 入口

## 职责

`op-render` 是 OpenPhoto 的 GPU 渲染 crate。当前它只做一件事：把文档的合成结果画到画布上，包括缩放、平移、透明棋盘格和高倍缩放下的像素网格。

`lib.rs` 本身不含逻辑，只声明私有模块 `canvas` 并对外重导出四个符号：

- `CanvasImage`：要显示的合成图（紧密排列的非预乘 RGBA8）及其纹理槽 key、内容版本号。
- `CanvasView`：视图变换（文档左上角屏幕位置、缩放倍率、是否允许像素网格）。
- `install`：创建渲染管线并注册到 egui-wgpu 的回调资源中，启动时调用一次。
- `paint_callback`：生成一个在指定矩形内绘制画布的 egui 绘制命令。

具体行为见 `spec/crates/op-render/src/canvas.md`。

## 行为规则

- 画布不是独立窗口或独立 surface，而是通过 egui-wgpu 的 paint callback 嵌入 UI：与 egui 共用同一个 wgpu `Device`、`Queue` 和同一个 render pass，在 egui 绘制序列中的对应位置插入一次绘制。
- crate 直接依赖 `egui`、`egui-wgpu`、`bytemuck`，wgpu 类型通过 `egui_wgpu::wgpu` 的重导出使用，不单独依赖 `wgpu` crate，以保证与 egui 使用的 wgpu 版本完全一致。
- 对外只暴露上述四个符号；管线、纹理槽、uniform 布局、mip 生成等都是模块内部细节。

## 边界情况

- 必须先调用 `install` 再提交 `paint_callback` 生成的命令，否则绘制时会因找不到回调资源而 panic（见 canvas spec）。

## 与其它模块的关系

- `op-ui`（`crates/op-ui/src/lib.rs`）在 `OpenPhotoApp::new` 中拿到 eframe 的 `wgpu_render_state` 并调用 `op_render::install`。
- `op-ui` 的 `state.rs` 用文档合成结果构造 `CanvasImage`，`document_view.rs` 构造 `CanvasView` 并通过 `op_render::paint_callback` 把画布加入画家队列。
- `op-render` 不依赖 `op-core`：它不了解图层、混合模式等文档结构，只接收已经合成好的像素。

## 已知限制

- 图层合成、混合模式和滤镜目前都不在本 crate 中，而由上游在 CPU 上完成（`Document::composite_rgba8`），本 crate 只负责显示最终合成图。
- `Cargo.toml` 声明了 `log` 依赖，但当前源码中没有使用它。
