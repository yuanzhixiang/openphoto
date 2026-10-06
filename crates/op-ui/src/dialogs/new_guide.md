# dialogs/new_guide.rs：View › Guides › New Guide... 对话框

## 组件职责

按数值新建一条参考线。确定后由 `lib.rs` 加入当前文档并记录「New Guide」。

## 布局与交互

按 Photoshop 对话框的结构排列（单位为 Photoshop 点，尚未逐像素比对）：

- 公共外框，标题「New Guide」，330 × 170。
- 「Orientation」分组标题，下面 Horizontal / Vertical 两个单选项，默认 Horizontal。
- 「Position:」输入框（默认 0，打开时获得焦点并全选），右侧单位「px」；可以带「px」后缀输入。范围 −30000–30000，超出或不是数字时 OK 置灰。
- 右侧 OK、Cancel；Enter 确定，Esc 取消。打开期间是模态的。

## 已知限制

- 没有 Photoshop 的颜色选项，只能用像素单位。

## 测试覆盖

- `position_accepts_a_px_suffix`：「120 px」解析为 120 的垂直参考线；非数字无效。
