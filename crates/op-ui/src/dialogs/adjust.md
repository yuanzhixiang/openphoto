# dialogs/adjust.rs：Threshold 与 Posterize 对话框

## 组件职责

单一设置的调整对话框：Image › Adjustments › Threshold... 与 Posterize...。对话框只管理设置和预览开关，预览与应用由 `lib.rs` 完成（见 `lib.md`「Threshold / Posterize 对话框的接入」）。

## 数据输入

- `AdjustDialog::new(kind, histogram, before)`：`kind` 为 `Threshold` 或 `Posterize`；`histogram` 是打开时活动图层选区内的亮度直方图；`before` 是打开时的文档快照。
- 默认值与范围同 Photoshop：Threshold Level 128（1–255），Posterize Levels 4（2–255）。Preview 默认勾选。每次打开都恢复默认值。

## 布局与视觉

按 Photoshop 对话框的结构排列（单位为 Photoshop 点，尚未逐像素比对）：

- 公共外框与标题栏，标题「Threshold」/「Posterize」。Threshold 对话框 400 × 232，Posterize 330 × 132。
- 左上方标签「Threshold Level:」或「Levels:」，右侧 52 × 22 的数字输入框；打开时输入框获得焦点并全选。
- Threshold：输入框下方是 258 × 100 的直方图框（深灰底，每个亮度值一条竖线，高度按最大计数归一化），直方图下方的白色三角形标出当前阈值。
- 右侧：OK、Cancel 按钮，下面是 Preview 复选框。

## 交互

- 输入框：输入整数，超出范围或不是数字时 OK 置灰，也不预览。
- Threshold 的三角形轨道：单击或拖动把阈值设为指针位置对应的值（1–255）。
- Preview：勾选时文档实时显示结果，取消勾选时恢复原样。
- OK 或 Enter（值有效时）：返回 `Outcome::Apply(adjustment)`；Cancel 或 Esc：返回 `Outcome::Cancel`。
- 打开期间是模态的。

## 已知限制

- Photoshop 会记住上次的值；这里每次恢复默认值。
