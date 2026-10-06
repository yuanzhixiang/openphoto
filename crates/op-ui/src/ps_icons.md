# ps_icons.rs：照 Photoshop 描出的矢量图标

## 职责

选项栏里的图标按 Photoshop 2026 的 2x 截图逐像素描成矢量图形（矩形、凸多边形、折线、圆环），用 egui 的绘图指令画出，所以形状、粗细和位置都与 Photoshop 一致，而不是用图标字体近似。不使用 Adobe 的图标资源文件。

## 对外接口

- `Icon`：Home、Move（四向箭头）、Caret（V 形下拉箭头）、Share、Bell、Search、Lightbulb、Workspace，对齐与分布的八个图标（AlignLeft、AlignHorizontalCenter、AlignRight、DistributeVertically、AlignTop、AlignVerticalCenter、AlignBottom、DistributeHorizontally），More（三个圆点），Gear（齿轮，右下带白色小三角）。
- `paint(painter, center, icon, color, background)`：以 `center` 为中心画图标。`background` 是图标后面的底色，用于齿轮的中心孔。

## 坐标约定

每个图标的坐标都是 Photoshop 2x 设备像素（即半个 pt），相对于图标中心。图标中心取描图时墨迹范围的中心，选项栏按 Photoshop 中测得的中心位置放置（见 `options_bar.md`）。线宽同样以 2x 像素给出。

## 视觉状态

颜色由调用方给出：正常 `#dddddd`，铃铛 `#b9b9b9`，禁用 `#989898`；齿轮的小三角总是白色。

## 已知限制

- 只描了移动工具选项栏和右侧按钮用到的图标；其它工具的选项栏和工具栏仍用 Phosphor 图标。
- 头像是占位的纯色圆，不是 Photoshop 的账户头像。
