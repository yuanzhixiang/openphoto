# fixtures/adjust：Photoshop 2026 的调整结果

`adjust.rs` 的 `photoshop` 测试用这些文件对照 Photoshop 的输出：

- `probe.rgb`：64 × 72 的 RGB 探测图（前 64 行为 16 级 RGB 立方体，接着 4 行 0–255 灰阶，最后 4 行随机颜色）。
- `hs_*.rgb`、`hr_*.rgb`：Photoshop 对探测图执行 Hue/Saturation（Master、Colorize、颜色范围）后的结果，设置写在测试里。
- `levels.bin`、`brightness_contrast.bin`：每 256 字节一行，Photoshop 对 0–255 灰阶执行各组设置后的输出（设置在测试的 `LEVELS_ROWS`、`BRIGHTNESS_CONTRAST_ROWS` 中）。
- `color_balance.rgb`：每行 256 个 RGB 像素，80 组随机 Color Balance 设置（`COLOR_BALANCE_ROWS`）作用于灰阶的结果。

都由脚本驱动 Photoshop 生成（每行用选区套用一组参数，存为 PNG 后读出），不要手改。
