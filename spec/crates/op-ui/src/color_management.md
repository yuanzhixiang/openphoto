# color_management.rs：显示色彩管理（macOS）

## 职责

让窗口里的颜色按 sRGB 解释，经 macOS 的 ColorSync 转换到显示器的配置文件后再显示，与 Photoshop 的显示效果一致。只在 macOS 上编译（`#[cfg(target_os = "macos")]`）。

## 背景

wgpu 在 macOS 上通过 `CAMetalLayer` 输出画面。图层没有设置色彩空间时，系统把像素值直接当作显示器的原生色彩空间（Display P3 等广色域屏上就是 P3），不做任何转换，结果是同样的 sRGB 数值看起来更饱和。Photoshop 对文档按工作空间（sRGB）→ 显示器配置文件做色彩管理，界面颜色也经过系统转换，所以两边并排时颜色不同。

给 `CAMetalLayer` 设置 `colorspace = sRGB` 后，系统负责把 sRGB 像素转换到显示器的色彩空间：sRGB 色域内的颜色在任何显示器上都显示为正确的颜色。

## 对外接口

- `use_srgb(window)`：取 winit 窗口的 AppKit `NSView`，从它的根图层开始递归遍历所有子图层，把每个 `CAMetalLayer` 的 `colorspace` 设为 `kCGColorSpaceSRGB`，并在日志（`info` 级别）中记录标记了几个图层。窗口句柄不是 AppKit 类型、视图没有图层或系统无法创建 sRGB 色彩空间时什么也不做。

## 行为规则

- 在 `OpenPhotoApp::new` 中、原生菜单安装之后调用一次。此时 eframe 已经创建了 wgpu surface，Metal 图层已存在（正常启动时日志为「tagged 1 Metal layer(s) as sRGB」）。
- 只改变色彩空间标记，不改变像素数值：界面颜色、文档合成结果和导出文件的数值都不变；变化的只有它们在广色域显示器上的呈现。
- 在本身就是 sRGB 的显示器上，标记前后显示效果相同。

## 边界情况

- 遍历在主线程进行（`OpenPhotoApp::new` 在主线程调用），读取 `sublayers` 依赖这一点。
- 无窗口 UI 测试（`new_headless`）不调用本模块。

## 已知限制

- 文档颜色按 sRGB 处理，不读取文件内嵌的 ICC 配置文件，也没有 Color Settings、Assign/Convert to Profile（见 `README.md` 差距列表）。
- 色彩空间只在启动时设置一次。如果 wgpu 之后重建了 Metal 图层（目前不会发生），新图层不会被标记。
