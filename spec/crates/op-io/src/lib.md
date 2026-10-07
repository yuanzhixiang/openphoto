# op-io：文件读写

## 职责

在磁盘文件与 `op_core::Document` 之间转换：打开 Photoshop 文档（PSD，见 `psd.md`）和位图文件为文档，把文档保存为 PSD，以及把文档合成结果导出为位图文件。位图编解码全部委托给 `image` crate（0.25，只启用 png、jpeg、webp、tiff、bmp、gif 六种格式特性）。

## 对外接口

### `IoError`

| 变体 | 含义 | 显示文案 |
|---|---|---|
| `Read(ImageError)` | 打开时解码失败（包括读文件出错） | `could not read image: …` |
| `Write(ImageError)` | 导出时编码失败 | `could not write image: …` |
| `File(io::Error)` | 导出时写文件失败 | `could not save file: …` |
| `Unsupported(String)` | 扩展名不支持，携带转为小写的扩展名 | `unsupported file format: …` |
| `Psd(String)` | PSD 文件格式不对或使用了不支持的特性 | `could not read Photoshop document: …` |

`File(io::Error)` 也用于打开 PSD 时读文件出错。

### `OPEN_EXTENSIONS`

打开对话框提供的扩展名：`psd`、`png`、`jpg`、`jpeg`、`webp`、`tif`、`tiff`、`bmp`、`gif`。

### `SAVE_FORMATS`

Save As 对话框提供的格式（名称与扩展名），顺序即对话框中的顺序，第一个是默认格式：Photoshop（`psd`）、PNG（`png`）、JPEG（`jpg`、`jpeg`）。

### `open(path)`

1. 取路径扩展名并转为 ASCII 小写；没有扩展名时视为空字符串。
2. 扩展名不在 `OPEN_EXTENSIONS` 中时返回 `Unsupported`，不读取文件。
3. `psd` 交给 `psd::read`（保留图层）；其余用 `image::open` 解码，并统一转换为 8 位 RGBA（`into_rgba8`）。
4. 文档标题为文件名（含扩展名）；拿不到文件名时为 `"Untitled"`。
5. 位图通过 `Document::from_rgba8` 构建文档，72 ppi，RGB，8 位。图层规则与 Photoshop 一致：图像完全不透明时是锁定的「Background」背景图层；只要有任何一个像素不是完全不透明，就是名为「Layer 0」的普通图层（见 `crates/op-core/src/document.md`）。

### `save(doc, path)`

File › Save / Save As 写文件：扩展名为 `psd`（不区分大小写）时写入 `psd::write` 的结果（保留图层）；其它扩展名交给 `export_composite`（合成后的扁平图像）。PSD 同样先在内存中生成，再一次写入。

### `export_composite(doc, path)`

1. 按扩展名确定格式（`ImageFormat::from_path`）。无法识别时返回 `Unsupported`，不创建文件。
2. 取 `Document::composite_rgba8` 的合成结果。
3. JPEG 没有透明通道：先把合成结果按 alpha 铺到白色底上（与合成相同的 gamma 编码空间），转成 RGB 再编码。白色与 Photoshop Export As 导出 JPG 时的默认底色一致。其它格式（PNG、WebP、TIFF、BMP、GIF）直接以 RGBA 编码，保留透明。
4. 先在内存里完成编码，成功后才写入文件。编码失败时不会创建或改动目标文件。

## 行为规则与边界情况

- 扩展名判断不区分大小写（`.PNG` 可以打开）。
- 实际解码格式由 `image::open` 按扩展名选择，而不是嗅探文件内容；扩展名与真实格式不符的文件会解码失败并返回 `Read` 错误。
- 打开位图时丢弃源文件的位深、颜色空间/ICC 配置文件和分辨率元数据：16 位图像被降为 8 位，分辨率一律为 72 ppi。PSD 会读取分辨率。
- 带 alpha 通道但所有像素都完全不透明的图片（例如不透明的 RGBA PNG）按不透明处理，打开为背景图层。
- GIF 等多帧格式只取第一帧。
- `export_composite` 内部的 `RgbaImage::from_raw` 依赖合成缓冲区长度等于 `width * height * 4`，此不变量由 `Document` 保证，不满足时 panic。

## 与其它模块的关系

- 依赖 `op-core`（`Document`）、`image`、`thiserror`。
- `op-ui` 的打开流程用 `OPEN_EXTENSIONS` 设置文件对话框过滤器并调用 `open`，成功后以 `"Open"` 作为第一条历史状态；保存流程用 `SAVE_FORMATS` 设置过滤器并调用 `save`；导出流程（对话框提供 PNG 与 JPEG 两种过滤器）调用 `export_composite`，失败时以警告框显示错误文案。

## 已知限制

- PSD 的限制见 `psd.md`；TIFF 不保存图层。
- 没有质量、压缩、底色等导出选项；JPEG 使用 `image` crate 的默认质量，底色固定为白色。
- 不读取、不写入任何元数据（EXIF、ICC、分辨率）。

## 测试覆盖

- `jpeg_export_flattens_onto_white`：半透明文档导出 JPEG 后能读回，不透明像素保持原色，透明像素变为白色（JPEG 有损，按容差比较）。
- `png_export_keeps_transparency`：导出 PNG 保留透明。
- `unknown_extension_writes_nothing`：未知扩展名返回 `Unsupported`，且不创建文件。
- `transparent_png_opens_as_regular_layer`：含半透明像素的 PNG 打开后是「Layer 0」普通图层。
- `flatten_blends_alpha`：50% 透明的黑色铺到白底上得到 `(127, 127, 127)`。
