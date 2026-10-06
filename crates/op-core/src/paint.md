# paint.rs：绘画笔画

## 职责

画笔、铅笔、橡皮擦的笔画计算：沿路径放置笔印，把颜色画到当前图层上，或擦除像素。

## 不透明度与流量

与 Photoshop 一致：一笔之内「流量」不断累加，但不会超过「不透明度」。

- 每一笔保存图层开始前的像素（`base`，因为 tile 共享，复制代价低），以及这一笔在每个像素上的覆盖度（0–1，按 tile 分块存储）。
- 每个笔印把覆盖度增加 `(1 − 覆盖度) × 笔印透明度 × 流量`。
- 像素结果 = 从 `base` 出发，按 `覆盖度 × 不透明度 × 选区` 改变。所以在同一笔内来回涂抹，结果最多达到不透明度。

## 对外接口

- `BrushTip`：直径（像素）、硬度（0 柔边 – 1 硬边）、`aliased`（铅笔：没有抗锯齿）。笔印透明度：从中心到 `半径 × 硬度` 为 1，之后平滑衰减到半径外 0.5 像素处为 0；硬度为 1 时也保留 1 像素的抗锯齿边缘。铅笔的像素中心在半径内为 1，否则为 0（直径至少按 1 像素计）。
- `StrokeKind::Paint(rgb)`：画颜色（画笔、铅笔）。`StrokeKind::Erase { background }`：橡皮擦。
- `Stroke::begin(doc, tip, kind, opacity, flow)`：在当前图层上开始一笔，同时记下当前选区。
- `Stroke::add_point(doc, x, y)`：把笔画延伸到 (`x`, `y`)。第一点放一个笔印；之后沿直线每隔直径的 25%（Photoshop 默认间距，至少 1 像素）放一个，跨调用保持间距连续。
- `last_point()`：笔画结束的位置，界面用于 Shift+单击画直线。
- `StrokeError`：不能开始的原因，`message(tool)` 给出 Photoshop 的提示文字：
  - 没有图层：「Could not use the {工具} because there is no layer to paint on.」
  - `lock_pixels`：「Could not use the {工具} because the layer is locked.」
  - 图层隐藏：「Could not use the {工具} because the target layer is hidden.」

## 像素规则

- 画颜色（普通图层）：颜色以 `数量` 为 alpha 做 source-over，叠到原像素上。
- 背景图层或「锁定透明像素」：只把颜色混向目标色，alpha 保持不变。
- 橡皮擦（普通图层）：alpha 乘以 `1 − 数量`。
- 橡皮擦（背景图层或锁定透明像素）：像画颜色一样，把颜色混向背景色，alpha 不变 —— 与 Photoshop 在背景图层上擦出背景色一致。
- 选区：按选中程度（0–255 → 0–1）缩放数量，羽化的选区按比例生效。
- 每个笔印之后调用 `mark_dirty`。

## 已知限制

- 只有圆形笔尖，没有笔刷预设、角度、圆度、间距设置，也没有压感、平滑（Smoothing）和喷枪。
- 画笔模式只有 Normal；橡皮擦只有 Brush 模式（没有 Pencil、Block 模式）。
- 每个笔印都会触发整个文档重新合成和上传，大文档上绘画较慢。

## 测试覆盖

- `hard_brush_paints_full_color_in_its_core`、`pencil_is_aliased`：硬边画笔中心为完整颜色；铅笔只有完全透明和完全不透明。
- `opacity_caps_a_single_stroke`：同一笔内反复涂抹，结果停在 50% 不透明度。
- `flow_builds_up`：低流量时重复经过同一点会加深。
- `selection_masks_paint`：选区外不被画到。
- `eraser_removes_alpha_and_paints_background_on_background_layer`：普通图层上擦成透明，背景图层上擦出背景色。
- `locked_and_hidden_layers_refuse`：锁定像素和隐藏的图层拒绝绘画。
- `spacing_places_dabs_along_a_line`：沿直线的笔印没有间断。
