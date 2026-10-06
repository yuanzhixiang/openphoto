# fixtures/filter：Photoshop 2026 的滤镜结果

`filter.rs` 的 `photoshop::filters_match_photoshop` 用这些文件对照 Photoshop：

- `probe.rgb`：64 × 64 的 RGB 探测图（红绿渐变底、白色方块、一条黑线、散布的彩色点）。
- `f_*.rgb`：Photoshop 2026 对探测图执行各滤镜后的结果（Gaussian Blur 0.3/1/2.5/10、Box Blur、Unsharp Mask、Median、Minimum/Maximum、High Pass、Mosaic、Solarize、Blur、Blur More、Sharpen、Sharpen More、Find Edges、Motion Blur、Emboss、Fragment、Custom（`f_cust` 为 3 × 3 锐化核除以 4 加 10，`f_cust2` 为 5 × 5 全 1 除以 25）、Surface Blur（`f_surf` 半径 5 阈值 15，`f_surf2` 半径 2 阈值 40）、Dust & Scratches（`f_dust` 半径 2 阈值 0，`f_dust2` 半径 3 阈值 20）等），参数写在测试里。

由脚本驱动 Photoshop 生成，不要手改。另有未入库的测量：Gaussian Blur 的核由一维脉冲（一条竖线）在 16 位灰度下测得，Motion Blur、Emboss、Minimum 的核由单像素脉冲测得。
