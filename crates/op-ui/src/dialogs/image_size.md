# dialogs/image_size.rs：Image › Image Size 对话框

## 组件职责

设置图像的像素尺寸（重采样）与分辨率，确定后由 `lib.rs` 执行（见 `lib.md`「Image Size 对话框的接入」，重采样算法见 `op-core` 的 `image_ops.md`）。数值规则在 `Model` 里，与界面分开，便于测试。

## 数据输入

- `ImageSizeDialog::new(width, height, resolution)`：当前文档的宽、高、分辨率。每次打开都恢复默认：宽高单位 Inches、分辨率单位 Pixels/Inch、Dimensions 单位 Pixels、链接宽高、Resample 打开、方法 Automatic、Fit To 为 Original Size、Scale Styles 勾选（与 Photoshop 2026 默认一致）。
- `with_preview(Preview { rgba, width, height })`：文档的合成像素，供左侧预览；命令打开对话框时传入。

## 布局（Photoshop 2026 实测，665 × 358 pt，相对左上角）

截图逐项比对，文字的位置与宽度与 Photoshop 相差不超过 1–2 px：

- 窗口：公共外框与标题栏（`common::frame`，标题 13 pt 粗体）。左上角是 macOS 窗口的三个按钮，圆心 (14, 14)、(34, 14)、(54, 14)，直径 12：模态窗口所以关闭、最小化为灰色 `#bababa`，可缩放所以缩放按钮为绿色 `#61c554`（只有外观）。
- 预览 (10, 38)–(288, 316)：1 pt `#424242` 边框，内部按 100% 显示文档（一个图像像素对应一个设备像素），初始居中；透明区域显示白/`#cccccc` 棋盘格，图像外为白色。拖动可以平移，不会把图像拖出视野。
- 右栏文字为 AppKit 系统字体 12 pt（`theme::dialog`），标签与说明文字 `#d6d6d6`，右对齐到 x 423.5：
  - 「Image Size:」中心 y 50.25，值在 x 431.5：结果的 RGB 内存大小（≥ 1 MB 为两位小数的 M，否则一位小数的 K），与原来不同时追加「(was 1.70M)」。右边 (633.5, 41.5)–(654.5, 58.5) 是齿轮按钮，菜单只有可勾选的「Scale Styles」（还没有图层样式，只保存状态）。
  - 「Dimensions:」中心 y 79.75；(432, 70.5)–(450, 88) 是带 V 形箭头的小框，点击选单位（Percent、Pixels、Inches、Centimeters、Millimeters、Points、Picas）；x 459 起是「734 px × 811 px」，「×」为 14 pt，墨迹与两侧文字各隔 8.5 pt。
  - 「Fit To:」下拉 (428, 97)–(655, 118)：Original Size、Auto Resolution...（置灰，见已知限制）、三组预设（960 x 640 px 144 ppi 等四个屏幕尺寸；A4、A6、Legal、Letter；4 x 6、5 x 7、8 x 10、11 x 14 in 300 dpi）、Load/Save/Delete Preset...（置灰）、Custom（置灰，只在修改数值后显示）。
  - 「Width:」/「Height:」中心 y 136.5 / 165.5：输入框 (427.5, 127)–(503, 146) 与 (427.5, 156)–(503, 175)，单位下拉 (511, 126)–(655, 147) 与 (511, 155)–(655, 176)（Percent、Pixels、Inches、Centimeters、Millimeters、Points、Picas、Columns，宽高共用一个单位）。打开时 Width 获得焦点并全选。
  - 链条：按钮 (349, 142)–(365, 160)，`#383838` 底、`#636363` 边；里面是两个相扣的环与一根竖杆（照 2x 截图描出，不链接时没有竖杆）；上下各有一条 `#828282` 括号线连到 Width、Height 的标签。
  - 「Resolution:」中心 y 194.75：输入框 (427.5, 185)–(503, 204)，单位下拉 (511, 184)–(655, 205)（Pixels/Inch、Pixels/Centimeter）。
  - 「Resample:」复选框在 (334.5, 217.5)，方法下拉 (428, 213)–(655, 234)：Automatic | Preserve Details (enlargement)、Preserve Details 2.0、Bicubic Smoother (enlargement) | Bicubic Sharper (reduction) | Bicubic (smooth gradients)、Nearest Neighbor (hard edges)、Bilinear，菜单右侧显示 ⌥1 … ⌥8。
- 说明「Create a new, larger document with more detail」在 (328.5, 276.5)，下面是带下划线的蓝色 `#5e9eee` 链接「Open in Generative Upscale...」。
- 按钮：Cancel (329, 311.5)–(486.5, 337.5)，OK (496.5, 311.5)–(654, 337.5) 为默认按钮。

## 交互（`Model`，与 Photoshop 一致）

- 输入框按当前单位显示：像素为整数，其它最多三位小数并去掉末尾的 0（734 px 在 72 ppi 下是「10.194」英寸）。编辑某个框时，其它框随之刷新，正在编辑的框保留输入的文字。
- Resample 打开：改宽或高就改像素；链接时另一边按原始比例跟着变。改分辨率时，单位是打印尺寸（英寸等）则保持打印尺寸、像素随之变化，单位是像素或百分比则像素不变。
- Resample 关闭：像素恢复为文档的尺寸且不能改；链条强制链接、不能点；改打印尺寸会改分辨率；单位为像素或百分比时输入框置灰；方法下拉置灰，⌥ 数字键无效。
- Fit To 预设：把图像等比放进预设的框里（框按图像方向转过来，竖图用竖框），设为预设的分辨率和单位，并打开 Resample 与链条；Original Size 恢复原始像素与分辨率。之后再改任何数值，Fit To 显示 Custom。
- ⌥1 … ⌥8：选择对应的重采样方法。
- 有效范围：宽高 1–30000 像素，分辨率 1–10000；无效时 OK 置灰。
- OK 或 Enter：返回 `Outcome::Apply { width, height, resolution, resample }`（像素四舍五入；Resample 关闭时 `resample` 为 `None`）；Cancel 或 Esc：`Outcome::Cancel`。打开期间是模态的。

## 已知限制

- 下拉菜单用的是 egui 的弹出菜单，Photoshop 这里是 macOS 原生菜单（浅色半透明、勾号在左），外观不同。
- Auto Resolution... 与 Load/Save Preset... 还没有实现（置灰）；Generative Upscale 依赖 Adobe 云服务，链接没有动作。
- 输入框获得焦点时是 2 pt 的蓝色外圈，Photoshop 是 1 pt 的蓝色边框；对话框不能拖动改变大小。

## 测试覆盖

- `opens_like_photoshop`：734 × 811 / 72 ppi 打开时显示 10.194 × 11.264 英寸、1.70M、734 px × 811 px，方法 Automatic。
- `the_chain_and_units`：像素与百分比、厘米的换算，链条的等比，「436.5K (was 1.70M)」。
- `resolution_keeps_the_printed_size_or_the_pixels`：英寸单位下 144 ppi 使像素加倍；像素单位下像素不变；Resample 关闭时改英寸会改分辨率、像素不能输入；Pixels/Centimeter 的换算。
- `fit_to_presets`：1024 × 768 预设对竖图转为 768 × 1024 的框，得到 768 × 849；4 × 6 in 300 dpi 得到 1200 × 1326；修改后为 Custom，Original Size 复原。
- `ui_tests::image_size_resamples_proportionally`：输入 5.097 英寸后确认，文档变为 367 × 405，记录「Image Size」。
- `ui_tests::image_size_dialog_controls`：⌥5 选 Bicubic Sharper；点链条断开；关闭 Resample 后像素复原、链条恢复、⌥ 数字键无效；Cancel 关闭。
- `ui_tests::screenshot_image_size_dialog`（忽略，手动运行）：与 Photoshop 截图对比用。
