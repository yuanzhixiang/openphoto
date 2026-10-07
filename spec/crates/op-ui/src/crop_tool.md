# crop_tool.rs：裁剪工具

## 组件职责

裁剪工具（C）在画布上的裁剪框：显示、调整、确认与复位，以及工具的选项（`CropOptions`，由选项栏编辑，见 `options_bar.md`）。实际裁剪调用 `op_core::image_ops::crop_extended`，按尺寸裁剪时再调用 `resize`（见 `image_ops.md`）。

## 状态

- 裁剪框保存在 `DocState::crop`（`CropBox`）：`rect` 在「框空间」里——把图像绕自身中心转 −`angle` 后的坐标，框在其中是正的（`angle` 为 0 时就是文档像素坐标）；`angle` 是框在图像上转过的角度（顺时针弧度）；进行中的拖动 `CropDrag`：抓住的控制点、按下时指针的**屏幕**位置、当时的框与角度，以及拖动的种类（`CropDragKind`：框、旋转、拉直）。
- 画布按框空间显示：`rotation` 把（图像中心, `angle`）交给画布着色器，图像转着显示在正立的框下面（Photoshop 非 Classic 模式的样子），其余裁剪代码都在框空间里工作。
- 当前工具是裁剪工具、而文档还没有裁剪框时，框自动覆盖整个画布（与 Photoshop 选中裁剪工具时一致）。切换到其它工具时裁剪框被丢弃（Photoshop 此时会询问是否裁剪；这里直接放弃）。
- `AppState::crop_options`（`CropOptions`），默认值与 Photoshop 2026 一致：预设 W x H x Resolution（三个框为空，即自由裁剪）、分辨率单位 px/in、勾选 Delete Cropped Pixels、Fill 为 Background (default)、叠加 Rule of Thirds 且 Always Show Overlay、不用 Classic Mode、Show Cropped Area 与 Auto Center Preview 打开、Crop Shield 打开、颜色 Match Canvas、不透明度 75%、Auto Adjust Opacity 打开。

## 预设与比例（`CropPreset`、`PRESET_GROUPS`）

比例菜单按 Photoshop 2026 的分组：Ratio、W x H x Resolution | Original Ratio | 1 : 1 (Square)、4 : 5 (8 : 10)、5 : 7、2 : 3 (4 : 6)、16 : 9 | Front Image（置灰，需要另一个打开的文档）、4 x 5 in 300 ppi、8.5 x 11 in 300 ppi、1024 x 768 px 92 ppi、1280 x 800 px 113 ppi、1366 x 768 px 135 ppi | New/Delete Crop Preset...（置灰）。

- `choose`：比例预设把数字填进 W、H（如 16、9）；尺寸预设填「4 in」「5 in」与分辨率 300；Ratio、W x H x Resolution、Original Ratio 清空三个框。
- `aspect`：框要保持的宽高比：Original Ratio 为文档的比例；Ratio 类为 W ÷ H；尺寸类按单位换算后的比例（没有分辨率也能算出比例）。都为空时为自由裁剪。
- `output_size`：尺寸类（W x H x Resolution、尺寸预设）裁剪后要重采样到的像素尺寸。W、H 可写单位 px/in/cm/mm（不写为像素）；英寸等要有分辨率才能换算。
- `swap`（交换按钮）交换 W、H；`clear`（Clear）清空三个框（预设回到 Ratio 或 W x H x Resolution）。
- 选预设、交换或修改框里的数字后，若有比例，框重新设为画布上该比例最大的居中矩形（`fitted`）。

## 交互

- 控制点：四角与四边中点，屏幕上 10 pt 以内可抓取。拖动改变对应的边；有比例时角点保持比例（按变化较大的方向，对角固定），边保持比例时另一方向以中线为轴；没有比例时按住 Shift 拖动角点保持开始时的比例；按住 ⌥ 以中心为基准。
- **非 Classic Mode（默认，Auto Center Preview 打开）**：框始终保持在视图正中，图像在框下移动（Photoshop 2026 实测）。拖动控制点时，对边固定在图像上，所以框边相对图像移动的距离是指针移动的两倍；框内拖动是平移图像（框相对图像反向移动）；视图随时把框的中心放回窗口中心（`center_on_box`）。复位、取消、确认后也重新居中。
- **Classic Mode**：框内拖动移动框，控制点按指针移动，视图不动。
- 框外拖动：绕框的中心转动图像（Photoshop 的行为），顺时针拖动使图像顺时针转、框在图像上的角度减小；按住 Shift 以 15° 为步长；转动时框的中心停在图像的同一点上（`set_angle`）。光标为 Alias。拖动结束时框的宽或高小于 1 像素：恢复为拖动前的框。
- Straighten：选项栏的 Straighten 按钮（图标或文字）打开拉直模式（按钮有底色），下一次拖动画一条线，松开后图像转到让这条线水平（更接近竖直时为竖直），框变为当前比例（没有比例时为画布比例）在转过的图像里能放下的最大居中框（`fitted_turned`），然后退出拉直模式；Esc 也退出。
- 框可以超出画布：确认时画布在那里扩大（Fill 为 Background 时背景图层的新区域填背景色，其它图层透明）。
- 确认：Enter、在框内双击，或选项栏的 ✓：框有角度时先用 `rotate_arbitrary` 把图像转 −`angle`（背景色填新角落、画布扩大，框随画布中心平移），再把框四舍五入到整像素后 `crop_extended`（Delete Cropped Pixels 决定画布外的像素是否删除；不删除时背景图层变成「Layer 0」以保留它们）；有 `output_size` 时再用 Automatic 方法重采样到该尺寸，并设置分辨率；记录「Crop」。之后框覆盖新的画布。
- 取消：Esc，或选项栏的 ⦸：框恢复为整个画布。
- 有输入框获得键盘焦点时，Enter 和 Esc 不作用于裁剪框。
- 光标：控制点上为对应方向的双向箭头；框内为移动光标；框外为十字。

## 外观（Photoshop 2026 实测，2x 截图逐像素量取）

- 遮挡层（Crop Shield）：框外的图像与遮挡色混合，在**线性光**下按不透明度混合（Photoshop 就是这样：75% 的画布灰 `#282828` 盖在白色上是 141，盖在 128 灰上是 75，盖在黑色上是 34；在 gamma 空间混合会得到 94）。由画布着色器完成（`shield` 交给 `op-render` 的 `Shield`，见其 `canvas.md`）。Match Canvas 的颜色是画布外的底色 `color::PASTEBOARD`。Show Cropped Area 关闭时框外完全盖住；Crop Shield 关闭时（且显示裁剪区域）不遮挡。
- 框线：紧贴框外一条 1 pt `#323232` 暗线，再外一条 1 pt `#f4f4f4` 亮线。
- 控制柄：`#f4f4f4`，厚 4 pt，画在框外：四角是两臂各长 23 pt 的 L 形，四边中点是 47 pt 长的条，圆角 2.5 pt。
- 叠加（`Overlay`，框内，1 pt `#f4f4f4` 线加一层很淡的黑色阴影，在浅色图像上才看得出）：Rule of Thirds（三等分）、Grid（约 40 pt 的方格）、Diagonal（四角各一条 45° 线）、Triangle（一条对角线和另两个角到它的垂线）、Golden Ratio（0.382 / 0.618 处的线）、Golden Spiral（黄金矩形里的四分之一圆弧螺线）。显示方式：Always 一直显示，Auto 只在拖动时显示，Never 不显示。

## 已知限制

- Classic Mode 下转动时也是图像在转（Photoshop 的 Classic Mode 是框在转）。转动或拉直期间，选区、参考线等其它画布叠加仍按未转动的图像绘制。没有 Fill 的 Generative Expand 与 Content-Aware Fill（置灰）、自定义遮挡颜色、Auto Adjust Opacity 的效果、Front Image 与存储预设、透视裁剪工具。
- Photoshop 裁剪时在 Layers 面板里出现临时的「Crop Preview」图层、标签页标题带「Crop Preview」，这里没有。
- 切换工具时不询问是否裁剪。

## 测试覆盖

- `handles_resize_and_inside_moves`：角点缩放、Shift 保持 2:1、⌥ 以中心缩放、框内拖动移动、1:1 比例下拖右边保持正方形。
- `presets_ratios_and_sizes`：预设填入的数字、16:9 与交换、Original Ratio、4 × 5 in 300 ppi 为 1200 × 1500 像素、像素尺寸不需要分辨率、Clear。
- `ui_tests::crop_tool_crops_to_the_box`：默认模式下右下角拖进 (234, 311) 得到 266 × 189 的框且框保持居中；Classic Mode 下同样的拖动得到 500 × 500；Enter 裁剪并记录「Crop」；换工具丢弃框。
- `ui_tests::crop_shield_presets_and_growing_the_canvas`：1:1 得到居中的 300 × 300；白色图像框外渲染为 141、框内为 255；框超出右边时画布扩大，新区域为背景色。
- `ui_tests::screenshot_crop_tool`（忽略）：与 Photoshop 截图对比用。
- `ui_tests::crop_rotation_and_straighten`：框外按住 Shift 顺时针转四分之一圈，角度为 −90°；Esc 复位；10° 的拉直线使角度为 10°、框缩小并退出拉直模式；确认后画布约为框的大小，四角都不是背景色（框在图像内）。
