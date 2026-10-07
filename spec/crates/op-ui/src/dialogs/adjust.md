# dialogs/adjust.rs：调整与滤镜对话框

## 组件职责

带设置的调整与滤镜对话框：Image › Adjustments 下带对话框的调整，以及 Filter 菜单的 Gaussian Blur...、Box Blur...、Surface Blur...、Motion Blur...、Unsharp Mask...、Add Noise...、Dust & Scratches...、Median...、Minimum...、Maximum...、High Pass...、Offset...、Mosaic...、Emboss...、Twirl...、Pinch...、Spherize...、Polar Coordinates...、Custom...、Trace Contour...、Wind...。Levels...、Curves...、Brightness/Contrast...、Color Balance...、Hue/Saturation...、Channel Mixer...、Selective Color...、Vibrance...、Posterize...、Exposure...、Photo Filter...、Black & White...、Threshold...、Gradient Map... 已按 Photoshop 2026 重做（即全部带对话框的调整），布局与设置在各自的模块里（`levels.md`、`curves.md`、`brightness_contrast.md`、`color_balance.md`、`hue_saturation.md`、`channel_mixer.md`、`selective_color.md`、`vibrance.md`、`exposure.md`、`photo_filter.md`、`black_white.md`、`threshold.md`、`gradient_map.md`），本模块只为它们画窗口框与标题、转交 OK/Cancel 并处理预览。对话框只管理设置与 Preview 开关，预览与应用由 `lib.rs` 完成（见 `lib.md`「调整与滤镜对话框的接入」）。像素算法见 `op-core` 的 `adjust.md` 与 `filter.md`。

对话框的结果是 `Effect`：`Adjustment(Adjustment)` 或 `Filter(Filter)`。`Effect::name()` 是历史名称，`Effect::apply(doc, background)` 调用对应的 `op-core` 函数。

## 数据输入

- `AdjustDialog::new(kind, histogram, before)`：`kind` 为对话框种类；`histogram` 是打开时活动图层选区内的直方图（Threshold 用亮度）；`before` 是打开时的文档快照。
- 每个设置是一个「参数」：标签、范围、默认值、小数位数，以及类型——数值（输入框 + 滑块）、选项（单选按钮，值为选中项的序号）、复选（值为 0 或 1）。输入框中的文字按小数位数格式化；Add Noise、Minimum、Maximum 与 Photoshop 一样去掉末尾的 0（显示「12.5」「1」），其余保留（Gaussian Blur 显示「1.0」）。默认值与范围同 Photoshop：

| 对话框 | 参数（范围，默认值） |
|---|---|
| Gaussian Blur | Radius (pixels)（0.1–1000.0，1.0） |
| Box Blur | Radius (pixels)（1–2000，1） |
| Unsharp Mask | Amount (%)（1–500，50）、Radius (pixels)（0.1–1000.0，1.0）、Threshold (levels)（0–255，0） |
| Add Noise | Amount (%)（0.1–400，12.5，最多两位小数）、Distribution（Uniform / Gaussian，Uniform）、Monochromatic（不勾选） |
| Median | Radius (pixels)（1–500，1） |
| Minimum、Maximum | Radius (pixels)（0.2–500.0，1.0）、Preserve（Squareness / Roundness，Squareness） |
| Motion Blur | Angle (°)（−360–360，0）、Distance (pixels)（1–2000，10） |
| Emboss | Angle (°)（−180–180，135）、Height (pixels)（1–100，3）、Amount (%)（1–500，100） |
| Twirl | Angle (°)（−999–999，50） |
| Pinch | Amount (%)（−100–100，50） |
| Spherize | Amount (%)（−100–100，100）、Mode（Normal / Horizontal only / Vertical only，Normal） |
| Polar Coordinates | Options（Rectangular to Polar / Polar to Rectangular，Rectangular to Polar） |
| High Pass | Radius (pixels)（0.1–1000.0，10.0） |
| Offset | Horizontal (pixels right)、Vertical (pixels down)（−30000–30000，0）、Undefined Areas（Set to Transparent / Repeat Edge Pixels / Wrap Around，Set to Transparent） |
| Mosaic | Cell Size (square)（2–200，10） |
| Surface Blur | Radius (pixels)（1–100，5）、Threshold (levels)（2–255，15） |
| Dust & Scratches | Radius (pixels)（1–500，1）、Threshold (levels)（0–255，0） |
| Trace Contour | Level（0–255，128）、Edge（Lower / Upper，Upper） |
| Wind | Method（Wind / Blast / Stagger，Wind）、Direction（From the Right / From the Left，From the Right） |

- 滤镜对话框记住上次按 OK 时的设置（Photoshop 的行为）：`settings()` 返回输入框里的文字，`lib.rs` 在应用时按 `Kind` 存进 `AppState::filter_settings`，`commands.rs` 下次打开同一对话框时用 `restore(values)` 放回（个数不符时忽略）。Cancel 不记住。只在本次运行内有效，不写入偏好。调整对话框（`Custom` 的各调整变体）每次打开都是默认值，`settings()` 返回 `None`；Custom 滤镜（`Custom::Kernel`）同样记住，内容见 `custom_filter.md`。
- Preview 默认勾选。

## 布局与视觉

### 经典滤镜对话框

Gaussian Blur、Box Blur、Surface Blur、Motion Blur、Unsharp Mask、Add Noise、Dust & Scratches、Median、Minimum、Maximum、High Pass、Offset、Mosaic、Emboss、Trace Contour 按 Photoshop 2026 的经典对话框逐点重做（`classic_ui`），每个对话框的尺寸与各行位置在 `filter_layout.rs`（见 `filter_layout.md`）：

- 窗口框与标题栏是 `common::frame`（标题 AppKit 13 pt 粗体）。
- 右上角：OK（默认按钮）在 y 38.5、Cancel 在 73.5，左边距窗口右缘 90.5，宽 59.5（Gaussian Blur、High Pass 为 80，Offset 为 60），高 26；其下 Preview 复选框。
- 左上角预览框（除 Offset 外都有）：`PANE` 区域里居中显示文档（带当前预览效果）的中心部分，100%（一个图像像素对应一个屏幕像素），不缩放、不平滑；纹理由 `lib.rs` 的 `pane_texture` 在打开时与预览变化后重新生成（截取中心不超过 392 × 392 像素）。预览框下方是缩放控件：缩小（100% 时变暗）、「100%」、放大，目前只是显示。已知差异：Photoshop 的预览框不论 Preview 是否勾选都显示效果，这里取消勾选时显示原图；也还不能缩放、拖动查看其他部分。
- 数值行：右对齐到输入框的标签、19 pt 高的输入框（`appkit::field`，打开时第一个输入框获得焦点并全选）、单位文字（角度的「°」紧贴输入框）；下方 3 pt 灰色轨道，白色三角标记的尖端在轨道下 0.5 pt，标记从轨道起点左 2.25 pt 走到终点右 0.75 pt，位置按该行的 `Scale` 换算（Photoshop 的滑块大多不均匀）。
- Motion Blur、Emboss 的角度在输入框右侧带角度盘：圆内一条从中心指向角度的线（Motion Blur 的线穿过中心两端）。
- 选项：Minimum、Maximum 的 Preserve 为下拉框；Add Noise 的 Distribution、Offset 的 Undefined Areas 与 Trace Contour 的 Edge 为带标题的分组框加单选按钮；Add Noise 的 Monochromatic 为复选框。

### 插件式扭曲对话框

Twirl、Pinch、Spherize、Polar Coordinates、Wind 按 Photoshop 2026 的插件式对话框重做（`distort_ui`），布局与绘制在 `distort.rs`（见 `distort.md`）：大预览框（带滚动槽与左下缩放条）、右上 OK / Cancel（89 × 26，13 pt 文字），预览框下方是设置（输入框 + 五边形滑块，或 Polar Coordinates、Wind 的单选分组），右下是扭曲示意图。设置仍存在本模块的参数表里，所以记忆、校验、预览与经典对话框相同。

每个滤镜对话框都有经典或插件式布局之一（`Kind::size` 在没有布局时 panic，`every_filter_dialog_has_an_effect` 会暴露遗漏）；旧的通用布局已删除。

## 重做的对话框（`Custom`）

- `AdjustDialog` 对这些对话框持有各自的状态（`Custom` 的各变体），`effect()` 取自它们，`show` 用它们的尺寸并调用它们的 `ui`；窗口框与标题栏仍是 `common::frame`（标题用 AppKit 13 pt 粗体）。
- 它们返回 `uxp::Button`：OK → `Outcome::Apply`，Cancel → `Outcome::Cancel`，Brightness/Contrast 的 Auto → 按直方图算出 Auto 的取值（见 `brightness_contrast.md`）。
- Custom 滤镜（`Custom::Kernel`，见 `custom_filter.md`）：宿主先画预览框与缩放控件（`wants_pane()` 对它为真），再画网格；它的 Load...、Save... 请求由宿主用系统文件对话框完成（`load_kernel` / `save_kernel`）。
- `set_channel_histograms(h)`：打开 Levels、Curves 时由 `commands.rs` 传入红绿蓝三个通道的直方图。
- `set_colorize_hue(hue)`：打开 Hue/Saturation 时由 `commands.rs` 传入前景色的色相，作为 Colorize 的初始色相（Photoshop 的行为）。

## 交互

- 输入框：任一参数超出范围或不是数字时，OK 置灰，也不预览。每个滤镜对话框都必须在 `effect()` 里映射到 `Filter`，否则 OK 永远置灰。
- 三角标记：在轨道上按下或拖动时把它移到指针位置，按该行的比例换算取值（按小数位数取整）。插件式对话框的五边形滑块线性取整数。
- 角度盘：在盘内按下或拖动时取指针方向的角度（整度）；Motion Blur 的角度折回 −90–90。
- Preview：勾选时文档实时显示结果，取消勾选时恢复原样。
- OK 或 Enter（所有值有效时）：返回 `Outcome::Apply(adjustment)`；Cancel 或 Esc：返回 `Outcome::Cancel`。
- 打开期间是模态的。

## 测试覆盖

- `defaults_match_photoshop`：Levels、Exposure、Hue/Saturation 的默认调整。
- `filters_read_choices_and_checkboxes`：Add Noise 的分布与单色、Offset 的空白区域选项映射到对应的滤镜参数。
- `fields_show_values_as_photoshop_does`：Gaussian Blur、Unsharp Mask 的半径显示「1.0」，Add Noise 显示「12.5」，Minimum、Maximum 显示「1」。
- `every_filter_dialog_has_an_effect`：每种用参数表的滤镜对话框的默认值都能得到滤镜（漏掉映射会让 OK 置灰），并且都有布局。
- `ui_tests.rs` 的 `filter_dialogs_remember_their_last_values`、`more_filters_from_the_menu` 与截图测试覆盖记忆、从菜单打开并应用、以及布局。
