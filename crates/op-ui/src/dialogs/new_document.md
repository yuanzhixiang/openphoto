# dialogs/new_document.rs：File › New 对话框

## 组件职责

新建文档时设置名称、尺寸、分辨率与背景内容。确定后由 `actions::create_document` 创建（见 `actions.md`「新建」）。

## 布局与视觉

按 Photoshop 旧版 New 对话框（偏好设置里的「Use Legacy New Document Interface」）的结构排列，单位为 Photoshop 点，尚未逐像素比对：

- 公共外框，标题「New」，470 × 300。
- 右对齐的标签列与输入列：Name（180 宽，打开时获得焦点并全选）；Document Type: Custom（只读）；Width、Height（Pixels）；Resolution（Pixels/Inch）；Color Mode: RGB Color 8 bit（只读）；Background Contents 下拉（White、Black、Background Color、Transparent，默认 White）。
- 右侧 OK、Cancel；Enter 确定，Esc 取消。打开期间是模态的。

## 数据输入

`NewDocumentDialog::new(name, size)`：名称与默认尺寸（剪贴板图像的尺寸，没有时 1920 × 1080），分辨率默认 72。

## 校验

宽高 1–30000 像素、分辨率 1–10000，否则 OK 置灰。名称为空时用「Untitled」。

## 已知限制

- 没有 Photoshop 新版「New Document」界面（预设、最近使用、模板、画板选项），没有单位选择、颜色模式与位深选择、颜色配置文件与像素长宽比。

## 测试覆盖

- `defaults_and_validation`：默认 1920 × 1080、按剪贴板尺寸；宽为 0 无效。
