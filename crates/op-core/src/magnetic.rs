//! The Magnetic Lasso's edge snapping: a cost map that is cheap along
//! strong edges, and the cheapest path between two points through it
//! (the "live wire" the outline follows).

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Per-pixel cost of passing through a pixel: low on edges whose contrast
/// is above the Contrast option, high elsewhere.
pub struct EdgeMap {
    width: u32,
    height: u32,
    /// Gradient magnitude, 0–1.
    gradient: Vec<f32>,
    cost: Vec<f32>,
}

impl EdgeMap {
    /// Builds the map from straight RGBA8 pixels. `contrast` (0–1) is the
    /// weakest edge that still counts: gradients below it cost as much as
    /// flat areas.
    pub fn new(width: u32, height: u32, rgba: &[u8], contrast: f32) -> Self {
        let (w, h) = (width as usize, height as usize);
        let luma: Vec<f32> = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| {
                let a = p[3] as f32 / 255.0;
                (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) / 255.0 * a
            })
            .collect();
        let at = |x: isize, y: isize| {
            let x = x.clamp(0, w as isize - 1) as usize;
            let y = y.clamp(0, h as isize - 1) as usize;
            luma[y * w + x]
        };
        let mut gradient = vec![0f32; w * h];
        for y in 0..h as isize {
            for x in 0..w as isize {
                // Sobel
                let gx = at(x + 1, y - 1) + 2.0 * at(x + 1, y) + at(x + 1, y + 1)
                    - at(x - 1, y - 1)
                    - 2.0 * at(x - 1, y)
                    - at(x - 1, y + 1);
                let gy = at(x - 1, y + 1) + 2.0 * at(x, y + 1) + at(x + 1, y + 1)
                    - at(x - 1, y - 1)
                    - 2.0 * at(x, y - 1)
                    - at(x + 1, y - 1);
                gradient[y as usize * w + x as usize] = ((gx * gx + gy * gy).sqrt() / 4.0).min(1.0);
            }
        }
        let contrast = contrast.clamp(0.0, 0.99);
        let cost = gradient
            .iter()
            .map(|&g| {
                let strength = ((g - contrast) / (1.0 - contrast)).max(0.0);
                1.0 - strength.sqrt() + 0.02
            })
            .collect();
        Self {
            width,
            height,
            gradient,
            cost,
        }
    }

    /// The pixel with the strongest edge within `radius` of `p` (the
    /// pointer snaps to it); `p` itself when nothing there is an edge.
    pub fn snap(&self, (px, py): (u32, u32), radius: u32) -> (u32, u32) {
        let r = radius as i64;
        let mut best = ((px, py), 0.0f32);
        for y in (py as i64 - r).max(0)..=(py as i64 + r).min(self.height as i64 - 1) {
            for x in (px as i64 - r).max(0)..=(px as i64 + r).min(self.width as i64 - 1) {
                let (dx, dy) = (x - px as i64, y - py as i64);
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let g = 1.0 - self.cost[(y as u32 * self.width + x as u32) as usize];
                if g > best.1 + 0.05 {
                    best = ((x as u32, y as u32), g);
                }
            }
        }
        best.0
    }

    /// The cheapest 8-connected path from `from` to `to`, searched within
    /// their bounding box widened by `margin` pixels; both ends included.
    pub fn trace(&self, from: (u32, u32), to: (u32, u32), margin: u32) -> Vec<(u32, u32)> {
        let clampx = |v: i64| v.clamp(0, self.width as i64 - 1) as u32;
        let clampy = |v: i64| v.clamp(0, self.height as i64 - 1) as u32;
        let m = margin as i64;
        let x0 = clampx(from.0.min(to.0) as i64 - m);
        let y0 = clampy(from.1.min(to.1) as i64 - m);
        let x1 = clampx(from.0.max(to.0) as i64 + m);
        let y1 = clampy(from.1.max(to.1) as i64 + m);
        let (bw, bh) = ((x1 - x0 + 1) as usize, (y1 - y0 + 1) as usize);
        let index = |x: u32, y: u32| (y - y0) as usize * bw + (x - x0) as usize;
        let mut dist = vec![f32::INFINITY; bw * bh];
        let mut prev = vec![usize::MAX; bw * bh];
        let mut heap = BinaryHeap::new();
        let start = index(from.0, from.1);
        let goal = index(to.0, to.1);
        dist[start] = 0.0;
        heap.push(Node(0.0, start));
        while let Some(Node(d, i)) = heap.pop() {
            if i == goal {
                break;
            }
            if d > dist[i] {
                continue;
            }
            let (x, y) = ((i % bw) as i64, (i / bw) as i64);
            for (dx, dy) in [
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ] {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= bw as i64 || ny >= bh as i64 {
                    continue;
                }
                let j = ny as usize * bw + nx as usize;
                let (gx, gy) = (x0 + nx as u32, y0 + ny as u32);
                let step = if dx != 0 && dy != 0 { 1.414 } else { 1.0 };
                let nd = d + self.cost[(gy * self.width + gx) as usize] * step;
                if nd < dist[j] {
                    dist[j] = nd;
                    prev[j] = i;
                    heap.push(Node(nd, j));
                }
            }
        }
        let mut path = Vec::new();
        let mut i = goal;
        while i != usize::MAX {
            path.push((x0 + (i % bw) as u32, y0 + (i / bw) as u32));
            if i == start {
                break;
            }
            i = prev[i];
        }
        path.reverse();
        path
    }

    /// The gradient magnitude at (`x`, `y`), 0–1.
    pub fn gradient(&self, x: u32, y: u32) -> f32 {
        self.gradient[(y * self.width + x) as usize]
    }
}

/// A heap entry: the smallest distance first.
struct Node(f32, usize);

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for Node {}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_path_follows_an_edge() {
        // Black left half, white right half: the edge is at x = 20
        let (w, h) = (40u32, 30u32);
        let mut rgba = Vec::new();
        for _y in 0..h {
            for x in 0..w {
                let v = if x < 20 { 0 } else { 255 };
                rgba.extend_from_slice(&[v, v, v, 255]);
            }
        }
        let map = EdgeMap::new(w, h, &rgba, 0.1);
        // From near the edge at the top to near it at the bottom: the path
        // hugs x 19–20 instead of going straight between the ends
        let path = map.trace((17, 2), (17, 27), 10);
        assert_eq!(path.first(), Some(&(17, 2)));
        assert_eq!(path.last(), Some(&(17, 27)));
        let on_edge = path.iter().filter(|p| (19..=20).contains(&p.0)).count();
        assert!(on_edge > path.len() / 2, "{path:?}");
        // The pointer snaps to the edge
        assert!((19..=20).contains(&map.snap((16, 15), 5).0));
        assert!(map.gradient(20, 15) > 0.5);
    }
}
