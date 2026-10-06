# image_ops.rs：画布变换（图像大小、旋转、翻转、裁剪、修整）

## 职责

实现 Image 菜单中改变整个画布的操作：Image Size 的重采样、Image Rotation 的固定角度旋转与画布翻转、Crop、Trim。所有操作同时作用于每个图层和选区（包括可以 Reselect 的上一个选区），通过 `Document::transform_canvas` 完成，不记录历史。

## 对外接口与规则

### 图像大小（重采样）

- `Resample`：`Bicubic`（默认，标签「Bicubic (smooth gradients)」）、`Bilinear`、`NearestNeighbor`（「Nearest Neighbor (hard edges)」）。
- `resize(doc, width, height, method)`：把每个图层和选区缩放到新尺寸（宽高必须大于 0）。先横向、再纵向分离重采样：目标像素中心对应源坐标 `(i + 0.5) / 比例 − 0.5`；双三次用 a = −0.5 的三次卷积核（半径 2），双线性用三角核（半径 1），权重归一化；缩小时核按比例加宽，每个源像素都参与（面积正确的缩小）；邻近取最近的源像素。颜色在预乘 alpha 下计算，结果限制在 0–255（双三次会过冲）；alpha 小于 0.5 的像素为全透明。选区的选择程度按同样的方法重采样。

### 旋转与翻转

- `Orientation`：`Rotate180`、`Rotate90Clockwise`、`Rotate90CounterClockwise`、`FlipHorizontal`、`FlipVertical`。`history_name()` 给出 Photoshop 的历史名称：旋转都是「Rotate Canvas」，翻转为「Flip Canvas Horizontal」/「Flip Canvas Vertical」。
- `reorient(doc, orientation)`：逐像素重映射，无插值、无损。90° 旋转交换宽高。顺时针 90° 时原图左上角到右上角；逆时针 90° 时原图左上角到左下角；180° 时左上角到右下角。选区按同样的方式变换。

### 裁剪

- `crop(doc, x0, y0, x1, y1)`：把画布裁成该矩形（右、下边界不含），矩形必须在画布内且非空，否则 panic。图层像素与选区一起平移，选区仍然盖在原来的图像内容上。
- `crop_to_selection(doc)`：Image › Crop。裁到选区外接矩形；没有选区时返回 `false`。选区保留（羽化或非矩形选区的透明部分照常保留在选区中）。

### 修整（Trim）

- `TrimBasis`：`Transparent`（裁掉全透明像素，alpha 为 0）、`TopLeftColor`（裁掉与左上角像素颜色完全相同的像素）、`BottomRightColor`（与右下角像素完全相同）。颜色比较包括 alpha，不允许容差。
- `TrimSides`：上、左、下、右四边是否裁剪，默认全选。
- `trim_bounds(doc, basis, sides)`：在合成后的图像上找出所有「不被裁掉」的像素的外接矩形，未勾选的边保持原位置；所有像素都会被裁掉时返回 `None`。
- `trim(doc, basis, sides)`：按 `trim_bounds` 裁剪；结果为 `None` 或与原画布相同时不改动并返回 `false`。

## 参考线

- `reorient`：180° 时位置变为 `宽 − x` / `高 − y`；顺时针 90° 时垂直参考线 x 变为水平参考线 x、水平参考线 y 变为垂直参考线 `原高 − y`；逆时针 90° 时垂直参考线 x 变为水平参考线 `原宽 − x`、水平参考线 y 变为垂直参考线 y；水平翻转只改垂直参考线（`宽 − x`），垂直翻转只改水平参考线。
- `crop` 按裁剪原点平移参考线；`resize` 按宽高比例缩放参考线。
- 测试 `guides_follow_the_canvas`：旋转、裁剪、缩放后参考线的位置与方向。

## 已知限制

- 没有 Image Rotation › Arbitrary...（任意角度，需要重采样）。
- 图层与画布同样大，裁剪掉的像素直接丢弃（Photoshop 裁剪工具可以选择保留画布外像素）。
- `remapped` 会把整幅图像展开成缓冲区，内存占用为图像尺寸 × 4 字节，与图层稀疏程度无关。

## 测试覆盖

- `rotations_and_flips_move_the_corners`：五种变换后角上像素的位置与尺寸。
- `the_selection_turns_with_the_canvas`：顺时针旋转后选区移到右上角。
- `crop_keeps_the_selected_area`：裁到选区后尺寸、像素与选区位置正确；没有选区时不裁剪。
- `resize_scales_layers_and_selection`：4×2 中左半红色缩小一半后左像素以红为主、右像素接近白，选区跟随；邻近放大保持硬边；双线性放大在边缘混色。
- `trim_removes_borders_of_the_corner_color`：按左上角颜色裁剪四边、按右下角颜色只裁上边；不透明背景上按透明像素裁剪不改动。
