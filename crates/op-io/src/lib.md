# op-io：文件读写

## 职责

在磁盘文件与 `op_core::Document` 之间转换：打开位图文件为文档，以及把文档合成结果导出为位图文件。位图编解码全部委托给 `image` crate（0.25，只启用 png、jpeg、webp、tiff、bmp、gif 六种格式特性）。

## 对外接口

### `IoError`

- `Image(image::ImageError)`：`image` crate 返回的任何错误，包括解码失败、编码失败和底层 I/O 错误。显示文案统一为 `could not read image: …`。
- `Unsupported(String)`：扩展名不在可打开列表中，携带（已转为小写的）扩展名。显示文案为 `unsupported file format: …`。

### `OPEN_EXTENSIONS`

打开对话框提供的扩展名：`png`、`jpg`、`jpeg`、`webp`、`tif`、`tiff`、`bmp`、`gif`。

### `open(path)`

1. 取路径扩展名并转为 ASCII 小写；没有扩展名时视为空字符串。
2. 扩展名不在 `OPEN_EXTENSIONS` 中时返回 `Unsupported`，不读取文件。
3. 用 `image::open` 解码，并统一转换为 8 位 RGBA（`into_rgba8`）。
4. 文档标题为文件名（含扩展名）；拿不到文件名时为 `"Untitled"`。
5. 通过 `Document::from_rgba8` 构建文档：单一 `"Background"` 背景图层，72 ppi，RGB，8 位。

### `export_composite(doc, path)`

把 `Document::composite_rgba8` 的结果作为 RGBA8 图像保存到 `path`，格式由 `image` crate 根据扩展名决定。本函数自身不做扩展名白名单检查。

## 行为规则与边界情况

- 扩展名判断不区分大小写（`.PNG` 可以打开）。
- 实际解码格式由 `image::open` 按扩展名选择，而不是嗅探文件内容；扩展名与真实格式不符的文件会解码失败并返回 `Image` 错误。
- 打开时丢弃源文件的位深、颜色空间/ICC 配置文件和分辨率元数据：16 位图像被降为 8 位，分辨率一律为 72 ppi。
- 带透明通道的文件也作为背景图层打开（Photoshop 对含透明的 PNG 会打开为普通图层，这一点与 Photoshop 不同）。
- GIF 等多帧格式只取第一帧。
- 导出时始终提供 RGBA8 数据。PNG、TIFF、WebP 等支持 RGBA8 的格式可以直接保存；`image` 0.25 的 JPEG 编码器不接受 RGBA8，因此导出为 `.jpg`/`.jpeg` 会返回 `Image` 错误（且错误文案仍以 `could not read image` 开头），并可能在目标路径留下空文件，因为文件在编码前就已创建。
- `export_composite` 内部的 `RgbaImage::from_raw` 依赖合成缓冲区长度等于 `width * height * 4`，此不变量由 `Document` 保证，不满足时 panic。

## 与其它模块的关系

- 依赖 `op-core`（`Document`）、`image`、`thiserror`。
- `op-ui` 的打开流程用 `OPEN_EXTENSIONS` 设置文件对话框过滤器并调用 `open`，成功后以 `"Open"` 作为第一条历史状态；导出流程（对话框提供 PNG 与 JPEG 两种过滤器）调用 `export_composite`，失败时以警告框显示错误文案。

## 已知限制

- 不支持 PSD 的读写。
- 没有保存分层文档的能力，只能导出合成后的扁平图像。
- 导出不支持 JPEG（见上文），也没有质量、压缩等导出选项。
- 不读取、不写入任何元数据（EXIF、ICC、分辨率）。

## 测试覆盖

本 crate 没有单元测试。
