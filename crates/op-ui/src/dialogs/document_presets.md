# dialogs/document_presets.rs：New Document 的预设

## 组件职责

New Document 对话框（`new_document.md`）用到的数据与绘图：长度单位、预设、各类别的预设列表，以及卡片上的图标。

## 单位（`Unit`）

Pixels、Inches、Centimeters、Millimeters、Points、Picas。`pixels(ppi)` 给出一个单位等于多少像素（1 英寸 = 2.54 厘米 = 25.4 毫米 = 72 点 = 6 派卡）；`short` 是卡片上的简写（px、in、cm、mm、pt、pica）；`format` 显示数值：像素取整，其它最多 3 位小数，去掉末尾的 0。

## 预设（`Preset`）

名称、宽、高、单位、分辨率（ppi）与卡片图标的种类（`Kind`）。`size_label` 是卡片上的尺寸行，如「7 x 5 in @ 300 ppi」；`pixels` 换算成像素。

`category(index)` 按 Photoshop 2026 的顺序给出各页的空白文档预设：

- **Photo**（9 个，英寸、300 ppi）：Default Photoshop Size 7 × 5，Landscape 3 × 2、6 × 4、7 × 5、10 × 8，Portrait 2 × 3、4 × 6、5 × 7、8 × 10。
- **Print**（300 ppi）：Letter、Legal、Tabloid（英寸）；A4、A6、A5、A3、B5、B4、B3、C4、C5（毫米）。
- **Art & Illustration**（300 ppi）：1000 / 2000 pixel grid、Poster 18 × 24 in、Postcard 4 × 6 in、1080p、720p。
- **Web**（像素、72 ppi）：Web Most Common 1366 × 768、Web Large、Web Medium、Web Minimum、Web Small、MacBook Pro 13 / 15 (Retina)、iMac 27、Desktop HD Design。
- **Mobile**（像素、72 ppi）：iPhone、iPad、Android、Surface、Apple Watch、Mobile Design、iOS 7 与 Mac 图标尺寸，共 24 个。
- **Film & Video**（像素、72 ppi）：HDTV、HDV、DVCPRO HD、DCI 2K/4K/8K、UHDTV、FUHDTV、NTSC、PAL、Cineon、Film (2K)/(4K)，共 24 个。

## 卡片图标（`paint_icon`）

1 pt `#b9b9b9` 线条，按预设的长宽比画框：最长边按它在本页中的大小（`scales`：像素面积的平方根在本页中的位置，0–1）开方后从 36 pt 到 82 pt，另一边最多 62 pt（照 Photoshop Photo 页的卡片量取）。按种类在框里画：照片（山与太阳）、页面（折角；Custom 在左上角外画十字准星）、点阵网格、画笔、浏览器（顶栏）、手机/平板（圆角与 Home 键）、Surface（右侧触控笔）、手表（表带）、应用图标（圆角点阵）、细长条、视频（播放三角与 2K/4K/8K 角标）。

## 已知限制

- 图标是按形状画的近似图形，不是 Photoshop 的图标资源；Print、Mobile、Film & Video 只列出了截图里能看到的预设（Photoshop 分别有 14、28、25 个）。

## 测试覆盖

- `presets_convert_to_pixels`：Photo 有 9 个、第一个是 2100 × 1500 像素；A4 是 2480 × 3508；Web 的尺寸行；英寸的三位小数。
