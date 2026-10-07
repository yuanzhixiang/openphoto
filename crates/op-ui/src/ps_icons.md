# ps_icons.rs：照 Photoshop 描出的矢量图标

## 职责

选项栏里的图标按 Photoshop 2026 的 2x 截图逐像素描成矢量图形（矩形、凸多边形、折线、圆环），用 egui 的绘图指令画出，所以形状、粗细和位置都与 Photoshop 一致，而不是用图标字体近似。不使用 Adobe 的图标资源文件。

## 对外接口

- `paint_scaled(painter, center, icon, color, background, scale)`：按比例缩放后绘制（坐标与线宽都乘 `scale`）；`paint` 即 `scale = 1`。Lock Layers 对话框用约 0.45–0.5 倍的锁定图标。
- `Icon`：Home、Move（四向箭头）、Caret（V 形下拉箭头）、Share、Bell、Search、Lightbulb、Workspace，对齐与分布的八个图标（AlignLeft、AlignHorizontalCenter、AlignRight、DistributeVertically、AlignTop、AlignVerticalCenter、AlignBottom、DistributeHorizontally），More（三个圆点），Gear（齿轮，右下带白色小三角），CollapseToolbar（工具栏折叠条的粗「»」）、CollapseRight / CollapseLeft（面板列与图标列折叠条的细线「»」「«」），History、Comments（图标列的两个按钮），以及 Layers 面板的过滤按钮（FilterPixel 等五个）、锁定按钮（LockTransparent、LockPixels、LockPosition、LockArtboards、LockAll）、眼睛 Eye、空心锁 LayerLock（背景图层与部分锁定的图层）、实心锁 LayerLockFull（Lock all 的图层） 和底部栏的八个按钮图标。
- `paint(painter, center, icon, color, background)`：以 `center` 为中心画图标。`background` 是图标后面的底色，用于齿轮的中心孔。

## 坐标约定

每个图标的坐标都是 Photoshop 2x 设备像素（即半个 pt），相对于图标中心。图标中心取描图时墨迹范围的中心，选项栏按 Photoshop 中测得的中心位置放置（见 `options_bar.md`）。线宽同样以 2x 像素给出。

## 视觉状态

颜色由调用方给出：正常 `#dddddd`，铃铛 `#b9b9b9`，禁用 `#989898`；齿轮的小三角总是白色。

## 已知限制

- 工具栏的工具图标在 `tool_icons.rs` 里画；工具栏底部的图形直接画在 `toolbar.rs` 里。
- 头像是占位的纯色圆，不是 Photoshop 的账户头像。
- 裁剪工具选项栏的图标（照 Photoshop 2026 的 2x 截图描出）：Straighten（水平仪：上方的点、两侧方块与带气泡的杯形）、CropOverlay（3 × 3 网格加裁切标记和菜单小三角）、Info（圆圈里的 i）、CropReset（转动的箭头加底线）、CropCancel（⦸）、CropCommit（✓）、Swap（上下两个相反的实心箭头）。
- Hue/Saturation 对话框的图标（描自 Photoshop 2026 的 2x 截图）：`TargetedHand`（目标调整手形，两侧小三角）、`Eyedropper`、`EyedropperPlus`、`EyedropperMinus`（空心管身、斜向的箍和顶端圆球，后两者右下角加「+」「−」）、`PresetMenu`（三条横线与右下角的小三角）。
- Levels / Curves 的图标：`EyedropperBlack` / `EyedropperGray` / `EyedropperWhite`（管身下半为该颜色的吸管）、`CurvePoints`（穿过四个点的波浪线）、`Pencil`、`TargetedHandVertical`（带上下箭头的手形）、`GridQuarters`、`GridTenths`。

## 选项栏图标（按 Photoshop 2026 截图近似重画）

`PenPressure`（压感）、`Refresh`、`ObjectFinder`、`Feedback`（对象选择）、`QuickNew` / `QuickAdd` / `QuickSubtract`（快速选择的模式）、`Angle`（笔刷角度）、`BrushPanel` / `CloneSourcePanel`（面板开关：实心方块挖出画笔或图章）、`OpacityPressure`、`Airbrush`、`Symmetry`（带菜单小三角的蝴蝶）、`IgnoreAdjustments`、`SampleContinuous` / `SampleOnce` / `SampleSwatch`（取样方式）、`MixerLoad` / `MixerClean`、五个渐变类型（14 pt 方框里的灰度渐变：线性、径向、角度、对称、菱形）、`NotesPanel`。

钢笔、文字、形状、视图等工具：`PathOperations`（两个方块，后一个挖空）与 `PathCombine`（实心方块，形状与路径选择工具没有可组合的路径时）、`PathAlignment`、`PathArrangement`、`GearMenu`（齿轮），四者右下角都有菜单小三角（`menu_mark`）；`TextOrientation`、`FontSize`、`TextLeft` / `TextCenter` / `TextRight`（横排对齐的长短横线）与 `TextTop` / `TextMiddle` / `TextBottom`（直排对齐的竖线）、`WarpText`、`Text3d`、`CharacterPanels`；`Link`（链环）、`CornerRadius`（四分之一圆弧）、`PolygonSides`（六边形里的 #）；`ZoomIn` / `ZoomOut`；`ArrangeFront` / `ArrangeForward` / `ArrangeBackward` / `ArrangeBack`（四层菱形，实心的那层表示目标位置，左侧上下箭头）；`ArtboardPortrait` / `ArtboardLandscape`（折角页面与角上的刻线）、`AddArtboard`（折角页面里的 +）；画框的 `FrameRect` / `FrameEllipse` / `FrameTriangle` / `FrameHexagon` / `FrameCustom`（灰色填充 `#6f6f6f`、浅色轮廓与叉）；`StrokeCenter`（带四角控制点的方框）。
