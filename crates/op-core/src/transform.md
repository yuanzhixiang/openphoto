# transform.rs：自由变换与变换

## 职责

Edit › Free Transform 与 Edit › Transform 的像素部分：用仿射变换移动、缩放、旋转、翻转活动图层的像素（有选区时只变换选中的像素），双线性重采样。不记录历史。

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

当前图层是组时（没有选区），`bounds` 为组里所有像素图层的像素范围的并集，`transform` 对其中每个像素图层施加同一个变换；有选区时返回 `Empty`。

## 测试覆盖

- `affine_math`：`around` 的映射与逆映射、旋转方向、不可逆矩阵。
- `bounds_and_locks`：图层内容范围；背景图层无选区时锁定，有选区时为选区范围。
- `move_scale_and_flip_the_layer`：平移后像素到达新位置、原处变透明；放大 2 倍内部为实色、外圈因插值半透明；水平翻转。
- `selected_pixels_move_and_leave_the_background_color`：背景图层上移动选中像素，原处填背景色，选区跟随移动。
- `transforming_takes_pixels_outside_the_canvas_along`：一半在画布左边外的横条，范围包括画布外部分；向右平移后两个像素都在画布内；再向上平移到画布外，像素仍在图层上，范围为 (2, −4)–(4, −3)。
