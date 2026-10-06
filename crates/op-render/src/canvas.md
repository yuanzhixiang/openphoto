# 画布渲染（canvas.rs + canvas.wgsl）

> 本文档同时覆盖 `crates/op-render/src/canvas.rs`（Rust 侧：资源管理、纹理上传、mip 生成、uniform 写入）和 `crates/op-render/src/canvas.wgsl`（着色器：棋盘格、图像采样、像素网格）。两个源码文件去掉扩展名后同名，因此共用这一份 spec。

## 职责

在 egui 界面中的一个矩形区域内，按给定的缩放和平移把文档合成图画出来：

- 透明区域下方显示棋盘格；
- 缩放 ≥ 100% 时按最近邻显示，像素边缘锐利；缩放 < 100% 时使用 mip 链做平滑缩小；
- 缩放 ≥ 600% 时叠加像素网格；
- 文档范围之外不绘制任何内容，露出 egui 在下层绘制的粘贴板背景。

## 行为规则

### 嵌入方式

- `install(render_state)` 在启动时调用一次：编译 `canvas.wgsl`，创建 bind group 布局（binding 0 为 uniform buffer，1 为可过滤的 2D 浮点纹理，2 为过滤采样器，均仅片元阶段可见）、渲染管线和采样器，并把 `CanvasResources` 插入 egui-wgpu renderer 的 `callback_resources`。
- 管线的颜色目标直接使用 egui 的 `render_state.target_format`，不设置混合状态（片元结果直接覆盖目标），无深度/模板，不做多重采样。
- `paint_callback(rect, image, view)` 返回 `egui_wgpu::Callback::new_paint_callback` 生成的 `egui::PaintCallback`。调用方把它加入 painter（`op-ui` 会再加上裁剪矩形）。画布因此与 egui 共享 `Device`、`Queue` 和 render pass，按 egui 的绘制顺序插入。
- 每帧的 `prepare` 阶段：按需创建或重建纹理槽，并写入本帧的 uniform；不提交额外的命令缓冲。
- `paint` 阶段：绑定管线和该文档槽的 bind group，用 3 个顶点画一个覆盖整个视口（即 callback 矩形）的大三角形，无顶点缓冲。

### 视图变换

- `CanvasView::zoom` 的单位是**物理像素 / 文档像素**。与 Photoshop 一致，100% 表示一个图像像素对应一个屏幕物理像素；在 2× Retina 屏上 100% 的文档在逻辑点上只占一半大小。
- `CanvasView::origin` 是文档左上角在屏幕上的位置，单位是 egui 逻辑点。`prepare` 中乘以 `ScreenDescriptor::pixels_per_point` 转换为物理像素后写入 uniform。
- 着色器使用 `@builtin(position)` 的帧缓冲坐标（物理像素），与 egui 为 callback 设置的 viewport 无关。片元的文档坐标为 `(frag.xy - origin_px) / zoom`。
- 原点是否对齐到物理像素由调用方负责；`op-ui` 在传入前已把 origin 吸附到物理像素网格，以保证 100% 时画面锐利。

### Uniform 布局

两个 `vec4<f32>`（Rust 侧 `Uniforms`，`#[repr(C)]` + `Pod`）：

- `origin_zoom`：xy 为文档左上角物理像素坐标，z 为 zoom，w 为棋盘格格子边长（物理像素）。
- `doc`：xy 为文档宽高（文档像素），z 为最大 mip 级别（`mip_count - 1`），w 为像素网格开关（0 或 1）。

### 纹理槽

- `CanvasResources::slots` 是 `HashMap<u64, Slot>`，以 `CanvasImage::key` 为键，每个文档一个槽（`op-ui` 传入的 key 是文档 id）。
- 每个槽持有：当前上传的 `revision`、mip 级数、该槽专属的 uniform buffer 和 bind group（纹理视图 + 共享采样器 + uniform）。
- `prepare` 时，若槽不存在，或槽内 `revision` 与 `CanvasImage::revision` 不同，则调用 `create_slot` 整体重建：新建纹理、上传所有 mip 级、新建 uniform buffer 和 bind group，并替换掉旧槽（旧槽的 GPU 资源随之释放）。版本号相同时不做任何上传，只写 uniform。
- 是否需要重新上传只看 `revision` 是否相等，不比较像素内容，也不比较尺寸；revision 的含义由调用方保证。

### 纹理格式

- 纹理格式为 `Rgba8Unorm`，刻意**不用** sRGB 格式：Photoshop 在 gamma 编码空间中混合和显示，因此采样、mip 平均、与棋盘格的合成都直接在 gamma 编码值上进行，不做线性化。
- 片元输出同样不做颜色空间转换，按原值写入 egui 的目标格式。

### 预乘与 mip 链（CPU）

- 输入 `CanvasImage::pixels` 是紧密排列的**非预乘** RGBA8。`build_mips` 先在 CPU 上预乘：每个颜色通道 `c' = (c * a + 127) / 255`（四舍五入），alpha 不变。
- 然后从原尺寸开始，用 2×2 盒式滤波逐级缩小，直到 1×1：下一级尺寸为 `max(w/2, 1) × max(h/2, 1)`，每个输出像素是源中 `(2x, 2y)`、`(2x+1, 2y)`、`(2x, 2y+1)`、`(2x+1, 2y+1)` 四个像素（坐标越界时钳制到最后一行/列）逐通道求和后 `(sum + 2) / 4`。在预乘数据上平均，因此透明像素不会把颜色“渗”进来。
- 每一级生成后，只有宽和高都不超过 `device.limits().max_texture_dimension_2d` 的级别才会被保留。超出限制的大级别被跳过，纹理的第 0 级就是第一个能放进 GPU 的级别。

### 采样

- 着色器先把文档坐标归一化为 `uv = d / doc_size`，再乘以纹理第 0 级的实际尺寸取纹素。这种按比例映射让跳过大级别后（纹理比文档小）依然覆盖整个文档区域。
- `zoom >= 1.0`：用 `textureLoad` 读取第 0 级的 `floor(uv * dims)` 纹素，即最近邻采样，不经过采样器。
- `zoom < 1.0`：用 `textureSampleLevel` 以显式 LOD 采样，`lod = clamp(log2(dims.x / doc.x / zoom), 0, max_level)`；采样器的放大、缩小、mip 过滤均为线性（三线性），寻址模式为默认的 clamp-to-edge。LOD 只按 x 方向计算。

### 棋盘格

- 格子边长为 8 个逻辑点，换算成物理像素后取整（`round(8 * pixels_per_point)`），不随 zoom 变化。
- 格子以文档左上角为锚点（`floor((frag - origin) / cell)`），随平移一起移动；交替的两种颜色为白色 `1.0` 和灰色 `0.8`。
- 因为纹理是预乘的，合成公式为 `rgb = c.rgb + checker * (1 - c.a)`。输出 alpha 恒为 1。

### 像素网格

- 当 `pixel_grid` 为真且 `zoom >= 6.0`（600%）时绘制。
- 对片元文档坐标取小数部分，计算到最近像素边缘的距离（乘以 zoom 换算成物理像素）；任一方向距离 < 0.5 物理像素时，把颜色以 35% 的比例混向灰色 `0.55`。效果是每条像素边界约 1 个物理像素宽的半透明灰线。

### 文档范围外

- 片元的文档坐标 `d` 满足 `d.x < 0`、`d.y < 0`、`d.x >= 宽` 或 `d.y >= 高` 时直接 `discard`，不写入任何颜色。callback 矩形内文档之外的区域保留 egui 之前画的内容（粘贴板）。

## 边界情况

- 未调用 `install` 就执行 paint callback：`prepare` / `paint` 中的 `expect("canvas renderer installed")` 会 panic。
- 宽或高为 0 的图像：`build_mips` 返回空列表，`create_slot` 退回到 1×1 全透明纹理；着色器中文档尺寸为 0，所有片元都会被 discard，画布区域不绘制任何内容。
- 图像超过 GPU 最大纹理尺寸：显示的是第一个能放下的缩小级别；在 `zoom >= 1` 时它被按比例放大并以最近邻显示，因此看起来比原图粗糙，实际分辨率低于文档分辨率。
- `paint` 时找不到对应槽（正常流程下不会发生，因为 `prepare` 总会先建槽）：直接返回，不绘制。
- 奇数尺寸缩小：`w/2` 向下取整，所以奇数宽（高）的级别在缩小时最后一列（行）不参与下一级的平均；钳制只在源尺寸为 1 时起作用。
- 同一个 key 在一帧中被提交多次（例如同一文档有两个视图）时，它们共用一个 uniform buffer，`prepare` 中后写入的 uniform 会覆盖先写入的，所有绘制都使用最后一次的视图参数。
- 像素网格在文档的外边缘内侧也会出现半条线（边缘处到像素边界的距离同样 < 0.5 物理像素）。
- `pixels` 长度必须恰好是 `width * height * 4`。多出的不足 4 字节的尾部会被预乘步骤忽略；长度不足则后续纹理写入的数据量不匹配。

## 与其它模块的关系

- `crates/op-render/src/lib.rs` 重导出 `CanvasImage`、`CanvasView`、`install`、`paint_callback`。
- `op-ui/src/lib.rs`：`OpenPhotoApp::new` 中调用 `install`，要求 eframe 使用 wgpu 渲染器。
- `op-ui/src/state.rs`：`DocState::canvas_image` 以 `doc.id.0` 为 key、`doc.revision()` 为 revision，在 CPU 上调用 `Document::composite_rgba8()` 生成像素，并按 revision 缓存 `Arc<CanvasImage>`。
- `op-ui/src/document_view.rs`：计算吸附到物理像素的 origin，构造 `CanvasView`（`pixel_grid` 恒为 `true`，zoom 限制在 0.01–128），通过 `paint_callback` 加入带裁剪矩形的 painter。

## 已知限制

- 合成在上游 CPU 上完成；本模块只显示最终合成图，不在 GPU 上做图层合成、混合模式或滤镜。
- 任何内容变化（revision 改变）都会触发整张图的重新预乘、重新生成完整 mip 链并重新上传全部级别，没有脏区域的局部更新。
- 纹理槽在文档关闭后不会被释放：`slots` 只有插入和按 key 替换，没有删除路径，已关闭文档的纹理会一直占用显存直到程序退出。
- 超过 GPU 纹理尺寸限制的级别仍然会在 CPU 上完整生成后才被丢弃，大图会产生对应的 CPU 时间和内存开销；同时显示分辨率被降低，没有分块（tiling）显示。
- mip 选择只按 x 方向的缩放比计算，不做各向异性过滤。
- 棋盘格颜色、格子大小和像素网格颜色/阈值是写死在代码里的常量，不可配置。

## 裁剪遮挡（`Shield`）

`CanvasView::shield` 为 `Some` 时，片元着色器把文档坐标转到遮挡框自己的坐标系（中心、半宽高、顺时针角度，为旋转的裁剪框预留），框外的颜色与遮挡色按不透明度在**线性光**中混合后再转回 sRGB：先把两者从 sRGB 转成线性值，`mix`，再转回。这与 Photoshop 2026 的裁剪遮挡一致（75% 的 `#282828` 盖在白色上为 141）。uniform 增加三个 vec4：`shield_box`（中心 xy、半宽高 zw）、`shield_color`（rgb 与角度）、`shield_params`（不透明度、是否启用）。

## 图像转动（`rotation`）

`CanvasView::rotation` 为 `Some((pivot, angle))` 时，屏幕点先按视图换算成「框空间」点 b，再绕 `pivot` 顺时针转 `angle` 得到要采样的图像点 d（`d = pivot + R(angle)(b − pivot)`），所以图像看上去转了 −angle。遮挡框在框空间里判断。裁剪工具转动图像时使用。
