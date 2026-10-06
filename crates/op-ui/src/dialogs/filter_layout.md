# dialogs/filter_layout.rs：经典滤镜对话框的布局

## 组件职责

记录 Photoshop 2026 经典滤镜对话框（Gaussian Blur、Box Blur、Unsharp Mask、Add Noise、Median、Minimum、Maximum、High Pass、Offset、Mosaic、Motion Blur、Emboss、Surface Blur、Dust & Scratches、Trace Contour）的尺寸、各行控件的位置与滑块刻度。只有数据，不画任何东西；绘制与交互在 `adjust.rs` 的 `classic_ui`（见 `adjust.md`「经典滤镜对话框」）。

所有坐标是从对话框左上角（含标题栏）起算的 Photoshop 点，由 Photoshop 2026 的截图逐个测得。

## 数据输入

- `Layout`：`size`（宽、高）、`button_width`（OK / Cancel 宽，多数 59.5，Gaussian Blur 与 High Pass 为 80，Offset 为 60）、`preview_y`（Preview 复选框的上沿）、`pane`（是否有预览框，Offset 没有）、`rows`。
- `rows` 与对话框参数一一对应，顺序相同：
  - `Row::Number`：标签与其右端、输入框矩形、单位与其起点、可选的轨道（起点、终点、上沿）、刻度 `Scale`、可选的角度盘（圆心、半径）。
  - `Row::Popup`：标签与其右端、下拉框矩形。
  - `Row::Radios`：分组框标题、分组框矩形（标题压在上边框上）、各单选按钮中心的 y（x 固定 27）。
  - `Row::Check`：复选框标签与左上角。
- `PANE`（16, 43）–（212, 239）是预览框，`ZOOM_Y` 261.75 是缩放控件的中线。
- 尺寸：Gaussian Blur、High Pass 324 × 335；Box Blur、Median 324 × 342；Minimum、Maximum 324 × 378；Unsharp Mask 324 × 436；Add Noise 324 × 452；Mosaic 324 × 338；Motion Blur 324 × 389；Emboss 324 × 431；Surface Blur 324 × 395；Dust & Scratches 324 × 387；Trace Contour 324 × 425（Edge 分组框与 Add Noise 的 Distribution 同位置）；Offset 323 × 256。
- Box Blur 与 Median 的布局相同，但刻度不同（前者 1–2000，后者 1–500），所以是两个 `Layout`。

## 滑块刻度（`Scale`）

Photoshop 的滑块大多不是均匀的。刻度在 Photoshop 里逐个输入数值、读出三角标记的位置测得，`place(v, min, max)` 给出 0–1 的位置，`value(place, min, max)` 是它的反函数：

- `Linear`：从参数最小值到最大值均匀分布（Add Noise、Mosaic、Emboss 的 Height、Surface Blur、Offset）。
- `RADIUS_1000`：0.1–1000 的半径（Gaussian Blur、High Pass、Unsharp Mask 的 Radius），如 1 在 0.048、10 在 0.402、250 在 0.831。
- `RANGE_2000`：1–2000（Box Blur 的半径、Motion Blur 的距离）。
- `RANGE_500`：1–500（Median、Minimum、Maximum、Dust & Scratches 的半径，Unsharp Mask 与 Emboss 的 Amount）。
- `LEVELS_255`：0–255 的阈值（Unsharp Mask、Dust & Scratches）与 Trace Contour 的 Level（单独扫描 12 个值验证过同一张表）。
- 表之间线性插值，超出表的一端取 0 或 1。

## 边界与限制

- Offset 的滑块刻度没有测量，暂按线性。
- Surface Blur 的两个滑块刻度没有测量，暂按线性。
- 只有 100% 一种缩放，预览框不能拖动（见 `adjust.md`）。

## 测试覆盖

- `scales_match_photoshops_pins`：Gaussian Blur 半径 1、250 的标记位置与 Photoshop 的测量相差不到半点；Add Noise 100% 在 170 点长的轨道上 42.5 处；每张刻度表 `place` 与 `value` 互逆。
- `ui_tests.rs` 的 `screenshot_filter_dialogs` 生成全部经典滤镜对话框的截图，用于与 Photoshop 并排比对。
