# panels/histogram.rs：Histogram 面板

## 组件职责

紧凑视图：合成图像的亮度直方图（`DocState::composite_histogram`，按修订号缓存，只统计不透明度大于 0 的像素），深灰底上的浅色竖线，高度按最大计数归一化。

## 已知限制

- 没有扩展视图、通道选择（RGB / 红 / 绿 / 蓝 / 颜色）、统计数字与缓存警告。
