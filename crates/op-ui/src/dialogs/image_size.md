# dialogs/image_size.rs：Image › Image Size 对话框

## 组件职责

设置图像的像素尺寸（重采样）与分辨率，确定后由 `lib.rs` 执行（见 `lib.md`「Image Size 对话框的接入」，重采样算法见 `op-core` 的 `image_ops.md`）。

## 数据输入

`ImageSizeDialog::new(width, height, resolution)`：当前文档的宽、高、分辨率。默认链接宽高（等比）、Resample 打开、方法 Bicubic。每次打开都恢复默认。

## 布局与视觉

按 Photoshop 对话框右侧一栏的结构排列（单位为 Photoshop 点，尚未逐像素比对；Photoshop 左侧的预览图没有实现）：

- 公共外框与标题栏，标题「Image Size」，440 × 290。
- 「Image Size: 436.5K」：结果的 RGB 内存大小（宽 × 高 × 3 字节，≥ 1 MB 时以 M 为单位，一位小数）；「Dimensions: 367 px × 406 px」：结果的像素尺寸（次要文字颜色）。两者随输入实时更新。
- Width、Height：右对齐标签、80 × 22 输入框、单位「Pixels」；两行左侧是链条图标（链接时为 LINK_SIMPLE，断开时为 LINK_SIMPLE_BREAK）。
- Resolution：输入框与单位「Pixels/Inch」。
- 「Resample:」复选框与方法下拉（三种方法）。
- 底部右侧 Cancel、OK 按钮（与 Photoshop 的 Image Size 一致，按钮在底部）。
- 打开时 Width 输入框获得焦点并全选。

## 交互

- 链条：点击切换是否等比；打开链接时按当前宽度重算高度。链接时修改宽度会按原始比例重算高度（四舍五入，至少 1），修改高度同理重算宽度。
- Resample 关闭：Width、Height 置灰并恢复为原始尺寸，只能改分辨率；方法下拉置灰。
- 有效范围：宽高 1–30000 像素，分辨率 1–10000；无效时 OK 置灰。
- OK 或 Enter：返回 `Outcome::Apply { width, height, resolution, resample }`（Resample 关闭时 `resample` 为 `None`）；Cancel 或 Esc：`Outcome::Cancel`。
- 打开期间是模态的。

## 已知限制

- 只有像素单位，没有百分比、英寸、厘米等；Resample 打开时修改分辨率不会联动改变像素尺寸（Photoshop 会保持打印尺寸）。
- 没有 Fit To 预设、Automatic / Preserve Details / Bicubic Smoother / Bicubic Sharper 等方法、Scale Styles 选项和预览图。

## 测试覆盖

- `the_chain_keeps_proportions`：800×600 改宽 400 得高 300，改高 150 得宽 200；宽为 0 时无效。
