//! The Quick Selection and Object Selection tools: regions grown from
//! colors, stopped by edges, without a learned model.

use std::collections::VecDeque;

use crate::selection::Selection;

/// Straight RGBA8 pixels to sample, with their edge strength.
pub struct Sampler {
    width: u32,
    height: u32,
    rgb: Vec<[u8; 3]>,
    /// Gradient magnitude, 0–1.
    edges: Vec<f32>,
}

impl Sampler {
    pub fn new(width: u32, height: u32, rgba: &[u8]) -> Self {
        let map = crate::magnetic::EdgeMap::new(width, height, rgba, 0.0);
        let rgb = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| [p[0], p[1], p[2]])
            .collect();
        let edges = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .map(|(x, y)| map.gradient(x, y))
            .collect();
        Self {
            width,
            height,
            rgb,
            edges,
        }
    }

    fn at(&self, x: u32, y: u32) -> [u8; 3] {
        self.rgb[(y * self.width + x) as usize]
    }
}

fn distance(a: [u8; 3], b: [f32; 3]) -> f32 {
    (0..3)
        .map(|c| (a[c] as f32 - b[c]).abs())
        .fold(0.0, f32::max)
}

/// The Quick Selection tool's region, grown dab by dab.
pub struct QuickSelect {
    sampler: Sampler,
    region: Vec<bool>,
}

impl QuickSelect {
    pub fn new(sampler: Sampler) -> Self {
        let n = (sampler.width * sampler.height) as usize;
        Self {
            sampler,
            region: vec![false; n],
        }
    }

    /// One dab of `radius` at (`cx`, `cy`): from the pixels under the
    /// brush, the region grows through neighbors whose color is close to
    /// theirs (within 24 levels plus 1.5 × their spread, at most 80), not
    /// across strong edges (gradient 0.35 and up), at most eight radii
    /// from the center.
    pub fn dab(&mut self, (cx, cy): (f32, f32), radius: f32) {
        let s = &self.sampler;
        let (w, h) = (s.width as i64, s.height as i64);
        let r = radius.max(1.0);
        let mut seeds = Vec::new();
        for y in (cy - r).floor() as i64..=(cy + r).ceil() as i64 {
            for x in (cx - r).floor() as i64..=(cx + r).ceil() as i64 {
                if x < 0 || y < 0 || x >= w || y >= h {
                    continue;
                }
                let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                if dx * dx + dy * dy <= r * r {
                    seeds.push((x as u32, y as u32));
                }
            }
        }
        if seeds.is_empty() {
            return;
        }
        let n = seeds.len() as f32;
        let mut mean = [0f32; 3];
        for &(x, y) in &seeds {
            let p = s.at(x, y);
            for c in 0..3 {
                mean[c] += p[c] as f32 / n;
            }
        }
        let spread = seeds
            .iter()
            .map(|&(x, y)| distance(s.at(x, y), mean))
            .sum::<f32>()
            / n;
        let threshold = (24.0 + 1.5 * spread).min(80.0);
        let reach2 = (8.0 * r) * (8.0 * r);
        let mut queue = VecDeque::new();
        let mut seen = vec![false; self.region.len()];
        for &(x, y) in &seeds {
            let i = (y * s.width + x) as usize;
            seen[i] = true;
            self.region[i] = true;
            queue.push_back((x, y));
        }
        while let Some((x, y)) = queue.pop_front() {
            for (nx, ny) in [
                (x as i64 - 1, y as i64),
                (x as i64 + 1, y as i64),
                (x as i64, y as i64 - 1),
                (x as i64, y as i64 + 1),
            ] {
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }
                let (nx, ny) = (nx as u32, ny as u32);
                let i = (ny * s.width + nx) as usize;
                if seen[i] {
                    continue;
                }
                seen[i] = true;
                let (dx, dy) = (nx as f32 + 0.5 - cx, ny as f32 + 0.5 - cy);
                if dx * dx + dy * dy > reach2 {
                    continue;
                }
                if s.edges[i] >= 0.35 || distance(s.at(nx, ny), mean) > threshold {
                    continue;
                }
                self.region[i] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    /// The region so far as a selection.
    pub fn selection(&self) -> Selection {
        let mask = self
            .region
            .iter()
            .map(|&b| if b { 255 } else { 0 })
            .collect();
        Selection::from_mask(self.sampler.width, self.sampler.height, mask, false)
    }
}

/// The Object Selection tool (Rectangle mode): within `rect` (x0, y0, x1,
/// y1), the pixels that stand out from the rectangle's edge (farther than
/// 40 levels from every color sampled along it) form the object; its
/// largest connected part is kept, with every part at least a tenth of its
/// size, and holes not reaching the edge are filled. `None` when nothing
/// stands out.
pub fn object_in_rect(
    sampler: &Sampler,
    (x0, y0, x1, y1): (u32, u32, u32, u32),
) -> Option<Selection> {
    let (w, h) = (sampler.width, sampler.height);
    let (x1, y1) = (x1.min(w), y1.min(h));
    if x1 <= x0 + 2 || y1 <= y0 + 2 {
        return None;
    }
    // Background colors: every few pixels along the rectangle's edge
    let mut border = Vec::new();
    let step = (((x1 - x0) + (y1 - y0)) / 100).max(1);
    for x in (x0..x1).step_by(step as usize) {
        border.push(sampler.at(x, y0));
        border.push(sampler.at(x, y1 - 1));
    }
    for y in (y0..y1).step_by(step as usize) {
        border.push(sampler.at(x0, y));
        border.push(sampler.at(x1 - 1, y));
    }
    let as_f = |c: [u8; 3]| c.map(|v| v as f32);
    let border: Vec<[f32; 3]> = border.into_iter().map(as_f).collect();
    let (bw, bh) = ((x1 - x0) as usize, (y1 - y0) as usize);
    let fg: Vec<bool> = (0..bw * bh)
        .map(|k| {
            let p = sampler.at(x0 + (k % bw) as u32, y0 + (k / bw) as u32);
            border.iter().all(|&b| distance(p, b) > 40.0)
        })
        .collect();
    // Connected parts of the foreground
    let mut label = vec![0u32; bw * bh];
    let mut sizes = vec![0usize];
    for start in 0..bw * bh {
        if !fg[start] || label[start] != 0 {
            continue;
        }
        let id = sizes.len() as u32;
        let mut size = 0;
        let mut queue = VecDeque::from([start]);
        label[start] = id;
        while let Some(i) = queue.pop_front() {
            size += 1;
            let (x, y) = (i % bw, i / bw);
            for (ok, j) in [
                (x > 0, i.wrapping_sub(1)),
                (x + 1 < bw, i + 1),
                (y > 0, i.wrapping_sub(bw)),
                (y + 1 < bh, i + bw),
            ] {
                if ok && fg[j] && label[j] == 0 {
                    label[j] = id;
                    queue.push_back(j);
                }
            }
        }
        sizes.push(size);
    }
    let largest = *sizes.iter().max()?;
    if largest == 0 {
        return None;
    }
    let keep: Vec<bool> = sizes.iter().map(|&s| s > 0 && s * 10 >= largest).collect();
    let object: Vec<bool> = label.iter().map(|&l| l != 0 && keep[l as usize]).collect();
    // Fill holes: background not connected to the rectangle's edge
    let mut outside = vec![false; bw * bh];
    let mut queue = VecDeque::new();
    for k in 0..bw * bh {
        let (x, y) = (k % bw, k / bw);
        if (x == 0 || y == 0 || x + 1 == bw || y + 1 == bh) && !object[k] {
            outside[k] = true;
            queue.push_back(k);
        }
    }
    while let Some(i) = queue.pop_front() {
        let (x, y) = (i % bw, i / bw);
        for (ok, j) in [
            (x > 0, i.wrapping_sub(1)),
            (x + 1 < bw, i + 1),
            (y > 0, i.wrapping_sub(bw)),
            (y + 1 < bh, i + bw),
        ] {
            if ok && !object[j] && !outside[j] {
                outside[j] = true;
                queue.push_back(j);
            }
        }
    }
    let mut mask = vec![0u8; (w * h) as usize];
    for (k, &out) in outside.iter().enumerate() {
        if !out {
            let (x, y) = (x0 + (k % bw) as u32, y0 + (k / bw) as u32);
            mask[(y * w + x) as usize] = 255;
        }
    }
    Some(Selection::from_mask(w, h, mask, false))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A gray image with a red disc (radius 10 at 30, 30) holding a white
    /// dot.
    fn image() -> (u32, u32, Vec<u8>) {
        let (w, h) = (60u32, 60u32);
        let mut rgba = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let (dx, dy) = (x as f32 - 30.0, y as f32 - 30.0);
                let d = (dx * dx + dy * dy).sqrt();
                let c = if d < 2.0 {
                    [255, 255, 255]
                } else if d < 10.0 {
                    [200, 30, 30]
                } else {
                    [100, 100, 100]
                };
                rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
            }
        }
        (w, h, rgba)
    }

    #[test]
    fn quick_selection_stays_within_edges() {
        let (w, h, rgba) = image();
        let mut q = QuickSelect::new(Sampler::new(w, h, &rgba));
        q.dab((34.0, 30.0), 3.0);
        let s = q.selection();
        // Grows over the red, stops at the disc's edge
        assert!(s.get(26, 30) > 0);
        assert_eq!(s.get(45, 30), 0);
    }

    #[test]
    fn object_selection_finds_the_disc() {
        let (w, h, rgba) = image();
        let s = object_in_rect(&Sampler::new(w, h, &rgba), (10, 10, 50, 50)).unwrap();
        let (x0, y0, x1, y1) = s.bounds().unwrap();
        assert!(
            x0.abs_diff(21) <= 1 && y0.abs_diff(21) <= 1,
            "{:?}",
            (x0, y0, x1, y1)
        );
        assert!(
            x1.abs_diff(40) <= 1 && y1.abs_diff(40) <= 1,
            "{:?}",
            (x0, y0, x1, y1)
        );
        // The white dot inside is filled in
        assert_eq!(s.get(30, 30), 255);
        // A plain area holds nothing
        assert!(object_in_rect(&Sampler::new(w, h, &rgba), (0, 0, 12, 12)).is_none());
    }
}
