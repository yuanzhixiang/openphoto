# psd.rs：Photoshop 文档读写

## 职责

读写 Photoshop 文档（.psd），按 Adobe《Photoshop File Formats Specification》实现，支持 8 位 RGB 的像素图层。没有外部依赖。

## 写入（`write(doc)`）

生成整个文件的字节：

1. 文件头：`8BPS`、版本 1、通道数（合成图完全不透明时 3，否则 4）、高、宽、位深 8、颜色模式 3（RGB）。
2. 颜色模式数据：空。
3. 图像资源：只有 ResolutionInfo（ID 1005），水平与垂直分辨率都取 `doc.resolution`（16.16 定点），单位像素/英寸。
4. 图层与蒙版信息：
   - 文档只有一个背景图层时不写任何图层，只写合成图——与 Photoshop 保存单一背景文档的方式相同。
   - 否则写全部图层（自底向上）。合成图有透明时图层数写为负数（表示合成图的 alpha 通道就是合并后的透明度）。
   - 图层组：从下往上，每个组最下面的图层（空组则为组本身）之前写一条名为「</Layer group>」的分隔记录（`lsct` 类型 3），组本身写成没有像素的记录，`lsct` 为 1（展开）或 2（折叠）并带「8BIM」与组的混合模式键（穿透为 `pass`）；嵌套时外层组的分隔记录在前。用 Photoshop 2026 打开验证过嵌套结构、组的不透明度与混合模式；Photoshop 保存的嵌套组文件也能正确读回。
   - 每个图层记录：图层范围为非透明像素（包括画布外的）的外接矩形，可以为负或超出画布（全透明图层范围为空）；通道（−1 = alpha，0/1/2 = R/G/B；背景图层与 Photoshop 一样没有 alpha 通道，只写 0/1/2）；混合模式键（`norm`、`mul `、`scrn` 等 27 种，与 `BlendMode` 一一对应）；不透明度（0–255）；标志位 bit 0「透明像素受保护」（背景图层或锁定透明像素时设置）、bit 1「隐藏」、bit 3（表示 bit 4 有效，Photoshop 5 以后总是设置）；Pascal 格式图层名（非 ASCII 字符写成「?」，补齐到 4 的倍数）；`luni` 附加信息（UTF-16 的完整名称）；`iOpa` 附加信息（填充不透明度）；有颜色标签时写 `lclr`（2 字节编号加 6 字节 0）；非背景图层有锁定时写 `lspf`（4 字节：bit 0 透明、bit 1 像素、bit 2 位置、bit 3 防止自动嵌套、bit 31 全部锁定，用 Photoshop 2026 保存的文件核对过）；背景图层还有 `lnsr` = `bgnd`（名称来源）。这些都与 Photoshop 2026 保存的文件一致（用 Photoshop 另存的 PSD 逐字段核对过）。
   - 有图层蒙版时多写一个通道 −2（覆盖整个画布的蒙版值），并在附加数据的「图层蒙版数据」中写 20 字节：蒙版矩形（整个画布）、矩形外的默认值 255、标志（bit 1 表示蒙版停用）和 2 字节填充。
   - 通道数据逐图层、逐通道写在记录之后，全部使用 PackBits（RLE）压缩：先是每行压缩后的字节数（u16），再是各行数据。
5. 合成图数据：`Document::composite_rgba8` 的结果，RLE 压缩，按 R、G、B（、A）平面顺序。

PackBits 编码：3 个以上相同字节编码为重复段（最长 128），其余为字面段（最长 128）。

## 读取（`read(data, title)`）

1. 校验 `8BPS` 签名和版本 1（版本 2 的 PSB 不支持）；只接受 8 位 RGB，否则返回 `IoError::Psd`。
2. 跳过颜色模式数据；从图像资源中读取 ResolutionInfo 的水平分辨率作为文档分辨率（没有时为 72）。
3. 读图层：名称优先用 `luni`，其次是 Pascal 名；`iOpa` 为填充不透明度；`lclr` 为颜色标签；带 `lsct`/`lsdk` 的记录是图层组：类型 3 是组底部的分隔记录（读到时记下位置），类型 1/2 是组本身（展开/折叠），读到时生成组图层，从对应分隔记录以来还没有父组的图层都归入它，嵌套组用栈还原。通道数据支持不压缩（0）和 RLE（1），ZIP（2、3）返回错误；带图层蒙版数据时读取通道 −2，按蒙版矩形和默认值还原成整幅画布的蒙版，并读取启用状态；−3（真实用户蒙版）及其它通道被跳过。图层像素按记录中的范围放到图层上，超出画布的部分保留在画布外（与 Photoshop 一致）；背景图层被裁到画布。
4. 最底层的记录如果「透明像素受保护」，并且没有 alpha 通道（Photoshop 的写法）或名为「Background」，就是背景图层；其它记录的锁定取自 `lspf`；没有 `lspf` 时，这个标志位映射为锁定透明像素。有 `lspf` 时不看这个标志位：Photoshop 锁定像素时也会设置它，只看它会误读出透明锁定。可见性、不透明度、填充不透明度、混合模式（未知的键按 Normal）照记录设置。最上面的图层成为活动图层。
5. 没有图层时读取合成图（不压缩或 RLE），按 `Document::from_rgba8` 的规则打开（完全不透明则为背景图层）。

## 已知限制

- 不支持图层组（打开时展开、保存时没有组）、矢量蒙版、调整图层、文字图层、智能对象、图层样式、剪贴蒙版、通道与路径，以及 16/32 位、灰度、CMYK 等模式。
- 不读写 ICC 配置文件与其它图像资源（参考线、切片、缩略图等）。
- 保存时不写入缩略图，Finder 中的 PSD 预览可能依赖系统自行渲染合成图。
- 已验证：Photoshop 2026 打开 OpenPhoto 写出的 PSD，图层名、不透明度、混合模式、可见性与背景图层都正确；OpenPhoto 读取 Photoshop 保存的 PSD，背景图层与普通图层都正确。图层蒙版尚未用 Photoshop 验证。

## 测试覆盖

- `groups_round_trip`：嵌套组 Outer { Inner { a }, b } 与顶层图层写出再读回，顺序、父子关系、折叠状态、穿透模式和组的不透明度都不变。`write_group_sample`（`#[ignore]`）写出供 Photoshop 打开核对的样例。

- `color_labels_round_trip`：带 Violet 标签的图层写出再读回标签不变，背景图层没有标签。用 Photoshop 2026 依次设了 Red…Gray 七种标签并保存的文件，本模块读出的标签与之一一对应。

- `pixels_outside_the_canvas_round_trip`：图层在画布左边外、右下外和画布内各有像素，写出再读回后内容范围和三个像素都不变。写出的文件用 Photoshop 2026 打开验证过：图层范围为 (20, −5)–(90, 30)，与写入的一致。

- `packbits_round_trip`：长重复段、256 个不同字节和短重复混合的一行，压缩后变短，解压后不变。
- `layers_round_trip`：背景加一个带 Unicode 名称、50% 不透明度、25% 填充、Multiply、隐藏的图层，写入后读回所有属性与像素（包括半透明像素和空白处）一致，分辨率 300 保留，最上层为活动图层。
- `masks_round_trip`：停用的蒙版写入后读回，值和启用状态一致。
- `a_lone_background_is_stored_as_the_merged_image`：只有背景时写为合成图，读回仍是背景图层，像素一致。
- `rejects_other_files`：非 PSD 数据返回错误。
- `photoshop_check::write_sample`（`#[ignore]`）：写出 `target/psd-check/ours.psd`，供用 Photoshop 或其它软件打开检查。
- `photoshop_check::read_photoshop_file`（`#[ignore]`）：读取环境变量 `OPENPHOTO_PSD` 指定的、由 Photoshop 保存的 PSD，最底层应为背景图层。

### 锁定的测试

- `locks_round_trip`：五个锁定标志写出再读回不变。
- `reads_photoshop_locks`：读 `fixtures/photoshop_locks.psd`（Photoshop 2026 保存，每个图层一种锁定：tp、px、pos、nest、all），每个图层只读出对应的那一种锁定。
