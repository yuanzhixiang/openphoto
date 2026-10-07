# paint.rs：绘画笔画

## 职责

画笔、铅笔、橡皮擦的笔画计算：沿路径放置笔印，把颜色画到当前图层上，或擦除像素。

## 不透明度与流量

与 Photoshop 一致：一笔之内「流量」不断累加，但不会超过「不透明度」。

- 每一笔保存图层开始前的像素（`base`，因为 tile 共享，复制代价低），以及这一笔在每个像素上的覆盖度（0–1，按 tile 分块存储）。
- 每个笔印把覆盖度增加 `(1 − 覆盖度) × 笔印透明度 × 流量`。
- 像素结果 = 从 `base` 出发，按 `覆盖度 × 不透明度 × 选区` 改变。所以在同一笔内来回涂抹，结果最多达到不透明度。

## 对外接口

- `BrushTip`：直径（像素）、硬度（0 柔边 – 1 硬边）、`aliased`（铅笔：没有抗锯齿）、`square`（橡皮擦的 Block：边长为直径的方块，内部完全覆盖、外部为 0）。笔印透明度：从中心到 `半径 × 硬度` 为 1，之后平滑衰减到半径外 0.5 像素处为 0；硬度为 1 时也保留 1 像素的抗锯齿边缘。铅笔的像素中心在半径内为 1，否则为 0（直径至少按 1 像素计）。
- `StrokeKind`：
  - `Paint(rgb)`：画颜色（画笔、铅笔）。`Erase { background }`：橡皮擦。
  - `Dodge(range)`、`Burn(range)`：减淡、加深。`ToneRange` 为 Shadows / Midtones（默认）/ Highlights，`label()` 为菜单文字。
  - `Sponge { saturate }`：海绵（加色或去色）。
  - `Blur`、`Sharpen`：模糊、锐化。
  - `Source { image, dx, dy }`：画另一张图像中的像素，目标 (x, y) 取 `image` 的 (x − dx, y − dy)——仿制图章（同一图层的另一处）和历史记录画笔（图层的早先状态，偏移为 0）都用它。
  `StrokeKind` 可以克隆，但不再是 `Copy`（`Source` 带着图像）。
- `Stroke::begin(doc, tip, kind, opacity, flow)`：在当前图层上开始一笔，同时记下当前选区。
- `Stroke::add_point(doc, x, y)`：把笔画延伸到 (`x`, `y`)。第一点放一个笔印；之后沿直线每隔直径的 25%（Photoshop 默认间距，至少 1 像素）放一个，跨调用保持间距连续。
- `last_point()`：笔画结束的位置，界面用于 Shift+单击画直线。
- `StrokeError`：不能开始的原因，`message(tool)` 给出 Photoshop 的提示文字：
  - 没有图层：「Could not use the {工具} because there is no layer to paint on.」
  - `lock_pixels`：「Could not use the {工具} because the layer is locked.」
  - 图层隐藏：「Could not use the {工具} because the target layer is hidden.」

## 绘画模式（`PaintMode`）

画笔、铅笔的 Mode，用 `Stroke::with_mode` 设置（默认 Normal）：

- `Blend(mode)`：以图层原有像素为底色，用 `blend::composite` 的混合公式把颜色以「覆盖率 × 不透明度 × 选区」的强度合成上去（Dissolve 按像素随机，与图层混合模式相同）。例：Multiply 的红色画在 `#808080` 上得到 `#800000`。
- `Behind`：颜色画在图层「下面」：原有像素盖在颜色之上，所以只有透明处被涂上。
- `Clear`：按强度降低透明度（像橡皮擦）。
- 背景图层、锁定透明像素或蒙版上，混合结果保留原透明度；Behind 与 Clear 在这些地方不改变像素。

## 像素规则

- 画颜色（普通图层）：颜色以 `数量` 为 alpha 做 source-over，叠到原像素上。
- 背景图层或「锁定透明像素」：只把颜色混向目标色，alpha 保持不变。
- 橡皮擦（普通图层）：alpha 乘以 `1 − 数量`。
- 橡皮擦（背景图层或锁定透明像素）：像画颜色一样，把颜色混向背景色，alpha 不变 —— 与 Photoshop 在背景图层上擦出背景色一致。
- 选区：按选中程度（0–255 → 0–1）缩放数量，羽化的选区按比例生效。
- 每个笔印之后调用 `mark_dirty`。

### 在蒙版上绘画

笔画的目标按快速蒙版 > 图层蒙版 > 像素的优先级决定。快速蒙版模式下，图层隐藏或锁定也可以绘画（只改快速蒙版）。编辑目标是蒙版（快速蒙版或 `Document::editing_mask`）时，笔画的基准图像是蒙版：画笔/铅笔的颜色换成灰度（`adjust::mask_gray`，即亮度），橡皮擦画背景色的灰度，修饰工具直接作用于灰度图像；蒙版不透明，alpha 保持不变。

### 修饰工具

修饰工具先算出笔画前像素在「满强度」下的目标值，再按 `数量`（覆盖率 × 不透明度 × 选择程度；界面把 Exposure / Strength 作为不透明度传入，海绵的 Flow 作为流量）在预乘 alpha 下从原像素混向目标；背景图层或锁定透明像素时只混颜色、保持 alpha。目标值（v 为 0–1 的通道值）：

- 减淡：`v + w(v) × (1 − v)`；加深：`v − w(v) × v`。权重 w：Shadows `(1 − v)²`、Midtones `4v(1 − v)`、Highlights `v²`。逐通道计算；这是对 Photoshop 算法的近似。
- 海绵：在 HSL 中把饱和度变为 0（去色）或加倍（加色，最大 1）。
- 模糊：笔画前像素的 3×3 预乘平均；锐化：`v + (v − 3×3 平均)`。因为基于笔画前的像素，同一笔内来回涂抹不会继续累积模糊/锐化。
- 来源：`image` 中对应位置的像素（含 alpha）；超出 `image` 范围时保持原像素。

## 已知限制

- 只有圆形笔尖，没有笔刷预设、角度、圆度、间距设置，也没有压感、平滑（Smoothing）和喷枪。
- 没有涂抹工具（Smudge）；修饰工具的 Protect Tones、Vibrance、Sample All Layers、Protect Detail 选项没有实现。
- 画笔模式只有 Normal；橡皮擦只有 Brush 模式（没有 Pencil、Block 模式）。
- 每个笔印都会触发整个文档重新合成和上传，大文档上绘画较慢。

## 图层组

当前图层是组且目标是像素时，开始笔画返回 `StrokeError::Group`（「Could not use the {工具} because the target layer is a group.」）；组的蒙版仍可以绘画。

## 测试覆盖

- `paint_modes_blend_with_the_layer`：Multiply、Behind、Clear；`a_square_tip_covers_a_block`：方形笔尖的角也被覆盖。
- `hard_brush_paints_full_color_in_its_core`、`pencil_is_aliased`：硬边画笔中心为完整颜色；铅笔只有完全透明和完全不透明。
- `dodge_burn_and_sponge`：中间调减淡变亮、加深变暗；Highlights 几乎不影响暗像素；去色后三通道相等。
- `quick_mask_round_trip`：快速蒙版模式下在隐藏图层上也能画；涂黑处退出后不在选区内；什么都不画时退出后没有选区。
- `blur_sharpen_and_clone`：模糊让黑白边缘出现中间值；仿制把 (5, 5) 的红点画到 (20, 20)，旁边不变。
- `opacity_caps_a_single_stroke`：同一笔内反复涂抹，结果停在 50% 不透明度。
- `flow_builds_up`：低流量时重复经过同一点会加深。
- `selection_masks_paint`：选区外不被画到。
- `eraser_removes_alpha_and_paints_background_on_background_layer`：普通图层上擦成透明，背景图层上擦出背景色。
- `locked_and_hidden_layers_refuse`：锁定像素和隐藏的图层拒绝绘画。
- `spacing_places_dabs_along_a_line`：沿直线的笔印没有间断。
