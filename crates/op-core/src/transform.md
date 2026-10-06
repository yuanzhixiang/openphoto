# transform.rs：自由变换与变换

## 职责

Edit › Free Transform 与 Edit › Transform 的像素部分：用仿射变换移动、缩放、旋转、翻转目标图层的像素（有选区时只变换活动图层选中的像素），双线性重采样。不记录历史。

## 对外接口

### `Affine`

二维仿射映射 `x' = a·x + b·y + c`，`y' = d·x + e·y + f`（文档像素坐标，y 向下）。

- `IDENTITY`、`translate(x, y)`、`scale(sx, sy)`、`rotate(angle)`（弧度，屏幕上顺时针为正）。
- `after(first)`：先 `first` 再 `self` 的复合。
- `apply(p)`、`inverse()`（行列式接近 0 时为 `None`）。
- `around(center, sx, sy, angle, offset)`：绕 `center` 先缩放、再旋转，最后平移 `offset`——自由变换框描述的就是它。

### `TransformError`

`NoLayer`、`Hidden`、`Locked`、`Empty`，提示文字与 Photoshop 一致：「Could not complete the {命令} command because there is no layer. / the target layer is hidden. / the layer is locked. / the selected area is empty.」。

### `bounds(doc)`

变换作用的范围（x0, y0, x1, y1）：

- 活动图层隐藏 → `Hidden`；像素或位置锁定，或者是背景图层而没有选区 → `Locked`（Photoshop 不能直接变换背景图层）。
- 没有选区：图层全部非透明像素（包括画布外的）的外接矩形，所以自由变换的框会包住移出画布的部分，与 Photoshop 一致。有选区：选区的外接矩形（选区内要有画布内的非透明像素）。
- 没有可变换的像素 → `Empty`。

### `transform(doc, m, background)`

1. 先用 `bounds` 检查；`m` 不可逆时返回 `Empty`。工作区域是画布、图层全部像素的范围和变换后框的范围三者的并集（可以伸到画布外），下面各步都在这个区域里进行（`region_rgba8` 读出、`from_region` 写回）。
2. 「移动的像素」：没有选区时为整个图层；有选区时为选中的像素（alpha 乘以选择程度）。
3. 「留下的像素」：没有选区时什么都不留；有选区时为图层减去移动的像素——普通图层 alpha 乘以 (1 − 选择程度)；背景图层改为按选择程度混合 `background`（背景色），保持不透明。
4. 对每个目标像素，以 `m` 的逆映射找到源位置（像素中心对像素中心），在预乘 alpha 下双线性采样移动的像素（图像外视为透明），用 Normal 合成到留下的像素上。
5. 有选区时选区本身也按同样的方式变换（选择程度双线性采样），成为新的选区。
6. 变换到画布外的像素保留在图层上；背景图层的结果被裁到画布。

### `FixedTransform`

Edit › Transform 的固定变换：`Rotate180`、`Rotate90Clockwise`、`Rotate90CounterClockwise`、`FlipHorizontal`、`FlipVertical`。`affine(bounds)` 给出绕范围中心的映射；`name()` 是菜单与历史名称（「Rotate 180°」「Rotate 90° Clockwise」「Rotate 90° Counter Clockwise」「Flip Horizontal」「Flip Vertical」）。

## 已知限制

- 只有双线性插值，没有 Photoshop 的 Bicubic 系列和 Nearest Neighbor 选项。
- 没有斜切、扭曲、透视、变形（Skew、Distort、Perspective、Warp）。
- 范围为奇数宽高时旋转 90° 会产生半像素偏移（重采样后边缘略微模糊）。

## 图层组

当前图层是组时（没有选区），组里的所有像素图层都是目标；有选区时返回 `Empty`。

## 多个图层

没有选区时，目标（`targets`）是：活动图层，加上其它选中的图层和与选中图层链接的图层（`link::with_linked`），组展开为其中的像素图层；除活动图层外，隐藏、背景、像素或位置锁定的图层被跳过（它们不动）。`bounds` 为所有目标的像素范围（含画布外）的并集，`transform` 对每个目标施加同一个变换。所以多选或链接的图层用一个变换框一起自由变换、一起翻转旋转，与 Photoshop 一致。有选区时只变换活动图层。

- `selected_and_linked_layers_transform_together`：选中两层、第三层与其中一层链接、第四层链接但锁定位置：变换框是三者的并集，三者一起移动，锁定的不动；有选区时只有活动图层的选中像素移动。

## 测试覆盖

- `affine_math`：`around` 的映射与逆映射、旋转方向、不可逆矩阵。
- `bounds_and_locks`：图层内容范围；背景图层无选区时锁定，有选区时为选区范围。
- `move_scale_and_flip_the_layer`：平移后像素到达新位置、原处变透明；放大 2 倍内部为实色、外圈因插值半透明；水平翻转。
- `selected_pixels_move_and_leave_the_background_color`：背景图层上移动选中像素，原处填背景色，选区跟随移动。
- `transforming_takes_pixels_outside_the_canvas_along`：一半在画布左边外的横条，范围包括画布外部分；向右平移后两个像素都在画布内；再向上平移到画布外，像素仍在图层上，范围为 (2, −4)–(4, −3)。

## 投影变换（`Projective`）

扭曲、透视需要的 3 × 3 投影变换（双精度）：`rect_to_quad(范围, 四角)` 把范围映射到任意四边形（Heckbert 的单位正方形到四边形构造，四角为平行四边形时退化为仿射），`inverse`、`after`、`from_affine`、`is_affine`。`transform` 对映射是泛型的（`Mapping` trait，`Affine` 与 `Projective` 都实现），逐像素用逆映射双线性采样，所以仿射与投影的变换走同一条路径。

- `projective_maps_the_box_onto_any_quad`：梯形的四角精确对应、逆映射、平行四边形是仿射、与仿射映射一致。
- `distorting_the_layer`：把方块的上边两角向内收，上面一行的覆盖少于下面一行。

## 插值（`Interpolation`）

自由变换选项栏的六种：Nearest Neighbor、Bilinear、Bicubic（默认，Keys a = −0.5）、Bicubic Smoother（Mitchell–Netravali）、Bicubic Sharper（a = −0.75）、Bicubic Automatic（按 Bicubic）。`transform_with(doc, m, background, how)` 按所选方法采样（预乘颜色，三次核的过冲被限制在有效范围内）；`transform` 用 Bicubic；选区的遮罩总是双线性。`Affine::skew(h, v)` 为水平、竖直斜切。测试 `interpolation_methods`：放大 2 倍时邻近保持硬边、双线性有过渡、各方法中间都是实心。

## 只变换选区

`selection_bounds(doc)` 为选区的范围；`transform_selection(doc, m)` 只按映射移动选区（双线性重采样选择程度），像素不动，没有选区时返回 `false`。`transform` 移动选中像素时也用同一个 `turned_selection`。测试 `transforming_the_selection_only`。
