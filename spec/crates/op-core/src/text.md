# text.rs：文字栅格化

## 职责

把文字工具输入的文字栅格化为像素：用 `skrifa` 从字体文件取字形轮廓，曲线展平成线段，按非零环绕规则消除锯齿地填充，得到覆盖值；再把覆盖值作为 alpha 生成文字图层。不记录历史。

## 对外接口

- `coverage(font_data, text, size, origin, width, height)`：文字在 `width` × `height` 画布上的覆盖值（每像素 0–255）。`size` 为像素字号（em 大小），`origin` 是第一行基线的起点；`\n` 换行，行距为字体的 ascent − descent + line gap（没有时为 1.2 倍字号），字体中没有的字符跳过。字体数据无法解析时返回 `FontError`。
- `caret(font_data, text, size, origin)`：最后一个字符之后的插入点位置（最后一行的基线上）与行高，供界面画光标。
- `add_text_layer(doc, font_data, text, size, origin, color)`：生成新图层插在当前图层之上并选中：颜色为 `color`，alpha 为覆盖值；图层名为文字的第一行（最多 30 个字符），与 Photoshop 文字图层以内容命名一致。文字没有可见像素（例如只有空格）时不加图层，返回 `Ok(None)`。

## 栅格化规则

- 字形按各自的前进宽度（advance width）依次排列，不做字距调整（kerning）与复杂文字整形。
- 二次曲线分 8 段、三次曲线分 12 段展平；字体坐标 y 向上，画布 y 向下。
- 每个像素取 4 条采样行，在每条行上按非零环绕规则求出内部区间，再按区间与像素的精确水平重叠累计覆盖率。

## 已知限制

- 文字图层是栅格图层，不能再编辑文字（Photoshop 的文字图层可以）。
- 没有字距、字符间距、行距设置，没有段落文字（文本框）、直排、文字变形，也不支持需要整形的文字（阿拉伯文、印度文字等）和字体中缺失字符的回退。

## 测试覆盖

- `letters_cover_pixels_above_the_baseline`：「I」的竖画在基线与大写字母高度之间实心，基线下、字顶上为空。
- `holes_stay_open_and_lines_stack`：「O」中间是空的；第二行比第一行低一个行高。
- `text_layers_are_named_after_the_text`：两行文字的图层名为第一行；只有空格时不加图层；无效字体返回错误。
