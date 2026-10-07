//! The Selection Brush tool: painting a selection with a brush, adding to
//! it or taking from it.

use crate::paint::BrushTip;
use crate::selection::Selection;

/// One stroke of the Selection Brush over the selection it started from.
pub struct SelectionStroke {
    width: u32,
    height: u32,
    tip: BrushTip,
    /// How strongly the stroke selects (the bar's Opacity), 0–1.
    opacity: f32,
    subtract: bool,
    /// The selection when the stroke began (0–255).
    base: Vec<u8>,
    /// The most each pixel has been covered by this stroke's dabs, so
    /// overlapping dabs don't build up past the opacity (like a brush).
    covered: Vec<f32>,
    last: Option<(f32, f32)>,
    since_dab: f32,
}

impl SelectionStroke {
    /// Starts a stroke on a `width` × `height` document whose selection is
    /// `current` (none: nothing selected).
    pub fn new(
        width: u32,
        height: u32,
        current: Option<&Selection>,
        tip: BrushTip,
        opacity: f32,
        subtract: bool,
    ) -> Self {
        let n = (width * height) as usize;
        let base = match current {
            Some(s) if s.width() == width && s.height() == height => (0..height)
                .flat_map(|y| (0..width).map(move |x| (x, y)))
                .map(|(x, y)| s.get(x, y))
                .collect(),
            _ => vec![0; n],
        };
        Self {
            width,
            height,
            tip,
            opacity: opacity.clamp(0.0, 1.0),
            subtract,
            base,
            covered: vec![0.0; n],
            last: None,
            since_dab: 0.0,
        }
    }

    /// Continues the stroke to (`x`, `y`), a dab every quarter of the
    /// brush's diameter (the first point places one dab).
    pub fn add_point(&mut self, x: f32, y: f32) {
        let Some((lx, ly)) = self.last else {
            self.dab(x, y);
            self.last = Some((x, y));
            return;
        };
        let (dx, dy) = (x - lx, y - ly);
        let dist = (dx * dx + dy * dy).sqrt();
        let spacing = (self.tip.diameter * 0.25).max(1.0);
        let mut t = spacing - self.since_dab;
        while t <= dist {
            let f = t / dist;
            self.dab(lx + dx * f, ly + dy * f);
            t += spacing;
        }
        self.since_dab = dist - (t - spacing);
        self.last = Some((x, y));
    }

    fn dab(&mut self, cx: f32, cy: f32) {
        let r = self.tip.diameter / 2.0 + 1.0;
        let x0 = (cx - r).floor().max(0.0) as u32;
        let y0 = (cy - r).floor().max(0.0) as u32;
        let x1 = ((cx + r).ceil() as i64).clamp(0, self.width as i64) as u32;
        let y1 = ((cy + r).ceil() as i64).clamp(0, self.height as i64) as u32;
        for y in y0..y1 {
            for x in x0..x1 {
                let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                let a = self.tip.alpha((dx * dx + dy * dy).sqrt());
                let i = (y * self.width + x) as usize;
                if a > self.covered[i] {
                    self.covered[i] = a;
                }
            }
        }
    }

    /// The selection so far: the starting one with this stroke's coverage
    /// (times the opacity) added to it or taken from it. `None` when
    /// nothing is left selected.
    pub fn selection(&self) -> Option<Selection> {
        let mask: Vec<u8> = self
            .base
            .iter()
            .zip(&self.covered)
            .map(|(&b, &c)| {
                let c = c * self.opacity;
                let b = b as f32;
                let v = if self.subtract {
                    b * (1.0 - c)
                } else {
                    b + (255.0 - b) * c
                };
                v.round().clamp(0.0, 255.0) as u8
            })
            .collect();
        let s = Selection::from_mask(self.width, self.height, mask, false);
        (!s.is_empty()).then_some(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tip(diameter: f32) -> BrushTip {
        BrushTip {
            diameter,
            hardness: 1.0,
            aliased: false,
            square: false,
            angle: 0.0,
            roundness: 1.0,
            spacing: 0.25,
        }
    }

    #[test]
    fn painting_adds_and_subtracts() {
        let mut s = SelectionStroke::new(40, 20, None, tip(6.0), 1.0, false);
        s.add_point(5.0, 10.0);
        s.add_point(30.0, 10.0);
        let sel = s.selection().unwrap();
        assert_eq!(sel.get(5, 10), 255);
        assert_eq!(sel.get(20, 10), 255);
        assert_eq!(sel.get(20, 2), 0);
        let (x0, _, x1, _) = sel.bounds().unwrap();
        assert!(x0 <= 3 && x1 >= 32, "{x0} {x1}");
        // Subtracting through the middle splits it
        let mut t = SelectionStroke::new(40, 20, Some(&sel), tip(4.0), 1.0, true);
        t.add_point(18.0, 0.0);
        t.add_point(18.0, 19.0);
        let cut = t.selection().unwrap();
        assert_eq!(cut.get(18, 10), 0);
        assert_eq!(cut.get(5, 10), 255);
        // Taking everything away leaves no selection
        let mut u = SelectionStroke::new(40, 20, Some(&cut), tip(60.0), 1.0, true);
        u.add_point(20.0, 10.0);
        assert!(u.selection().is_none());
    }

    #[test]
    fn opacity_selects_partly_and_strokes_dont_build_up() {
        let mut s = SelectionStroke::new(20, 20, None, tip(8.0), 0.5, false);
        for _ in 0..5 {
            s.add_point(10.0, 10.0);
            s.add_point(10.5, 10.0);
        }
        let sel = s.selection().unwrap();
        assert_eq!(sel.get(10, 10), 128);
    }
}
