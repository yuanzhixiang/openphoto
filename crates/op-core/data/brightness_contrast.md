# data/brightness_contrast.bin：Photoshop 的亮度/对比度曲线

452 × 256 字节：前 301 行为亮度 −150…150（对比度 0）时 Photoshop 2026 对 0–255 每一级的输出，后 151 行为对比度 −50…100（亮度 0）时的输出。`adjust.rs` 的 Brightness/Contrast（非 Legacy）先查亮度行、再查对比度行。

数据由脚本在 Photoshop 2026 中生成：一张 256 × 470 的图每行一条灰阶，每行用选区执行一次 Brightness/Contrast，存为 PNG 后读出（与它一起采的组合与 Legacy 行在 `fixtures/adjust/brightness_contrast.bin`，用于测试）。要更新时按同样方法重采，不要手改。
