# dialogs/fill.rs：Fill 对话框

## 组件职责

Edit › Fill...（⇧F5）。选择填充内容和混合方式，确定后由 `lib.rs` 调用 `op_core::fill::fill` 填充当前图层的选区（没有选区时整个图层），并记录一条「Fill」历史。

## 布局

参照 Photoshop 的 Fill 对话框排列，尺寸单位为 pt：

- 对话框 420×222，标题「Fill」（公共外框见 `common.md`）。
- Contents：标签右对齐到 x 84，下拉框 x 92、宽 200、高 22，行中心 y 56。
- 「Blending」半粗体小标题（y 96），右侧一条延伸到 x 296 的分隔线。
- Mode：下拉框列出全部 27 种混合模式，按 Photoshop 菜单分组（y 126）。
- Opacity：宽 52 的输入框加「%」（y 156）。打开时这个输入框获得焦点并全选。
- Preserve Transparency 复选框，与输入框左对齐（y 186）。
- 右侧 OK（y 44）和 Cancel（y 76）胶囊按钮，x 316，88×24。

## Contents 选项

Foreground Color（默认）、Background Color、Color...；Content-Aware、Pattern、History（置灰）；Black、50% Gray（128）、White。

选择「Color...」打开标题为「Color Picker (Fill Color)」的 Color Picker；在那里确定后，Contents 变为 Color... 并使用所选颜色，取消则保持原来的选择。

## 键盘

- Enter：确定（不透明度有效时）。Esc：取消。
- Color Picker 打开期间，Enter/Esc 只作用于 Color Picker。

## 校验

不透明度必须是 0–100 的数字，否则 OK 置灰。

## 已知限制

- 布局是按 Photoshop Fill 对话框的结构排列的，还没有与 Photoshop 的 1:1 截图逐像素比对。
- 没有 Content-Aware 的 Color Adaptation 选项，也没有 Pattern 的图案选择和脚本选项。
- 不记住上次的设置。
