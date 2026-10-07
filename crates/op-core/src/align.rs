//! Layer > Align and Layer > Distribute (and the Move tool's align
//! buttons): moving the selected layers so their contents line up or are
//! evenly spread, as in Photoshop.

use crate::document::Document;
use crate::layer::LayerId;

/// Layer > Align.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Align {
    Top,
    VerticalCenter,
    Bottom,
    Left,
    HorizontalCenter,
    Right,
}

impl Align {
    /// Menu order (Layer > Align), with a separator before Left.
    pub const ALL: [Self; 6] = [
        Self::Top,
        Self::VerticalCenter,
        Self::Bottom,
        Self::Left,
        Self::HorizontalCenter,
        Self::Right,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Top => "Top Edges",
            Self::VerticalCenter => "Vertical Centers",
            Self::Bottom => "Bottom Edges",
            Self::Left => "Left Edges",
            Self::HorizontalCenter => "Horizontal Centers",
            Self::Right => "Right Edges",
        }
    }

    /// The history name, as Photoshop records it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Top => "Align Top Edges",
            Self::VerticalCenter => "Align Vertical Centers",
            Self::Bottom => "Align Bottom Edges",
            Self::Left => "Align Left Edges",
            Self::HorizontalCenter => "Align Horizontal Centers",
            Self::Right => "Align Right Edges",
        }
    }
}

/// Layer > Distribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Distribute {
    Top,
    VerticalCenter,
    Bottom,
    Left,
    HorizontalCenter,
    Right,
    /// Equal gaps between the layers, left to right.
    Horizontally,
    /// Equal gaps between the layers, top to bottom.
    Vertically,
}

impl Distribute {
    /// Menu order (Layer > Distribute), with separators before Left and
    /// Horizontally.
    pub const ALL: [Self; 8] = [
        Self::Top,
        Self::VerticalCenter,
        Self::Bottom,
        Self::Left,
        Self::HorizontalCenter,
        Self::Right,
        Self::Horizontally,
        Self::Vertically,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Top => "Top Edges",
            Self::VerticalCenter => "Vertical Centers",
            Self::Bottom => "Bottom Edges",
            Self::Left => "Left Edges",
            Self::HorizontalCenter => "Horizontal Centers",
            Self::Right => "Right Edges",
            Self::Horizontally => "Horizontally",
            Self::Vertically => "Vertically",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Top => "Distribute Top Edges",
            Self::VerticalCenter => "Distribute Vertical Centers",
            Self::Bottom => "Distribute Bottom Edges",
            Self::Left => "Distribute Left Edges",
            Self::HorizontalCenter => "Distribute Horizontal Centers",
            Self::Right => "Distribute Right Edges",
            Self::Horizontally => "Distribute Horizontally",
            Self::Vertically => "Distribute Vertically",
        }
    }

    fn horizontal(self) -> bool {
        matches!(
            self,
            Self::Left | Self::HorizontalCenter | Self::Right | Self::Horizontally
        )
    }
}

type Bounds = (i64, i64, i64, i64);

/// The selected layers, and the layers linked to them, that can move and
/// have pixels, with their bounds. A group counts as one, with the box
/// around its layers' pixels.
fn movable(doc: &Document) -> Vec<(LayerId, Bounds)> {
    crate::link::with_linked(doc)
        .into_iter()
        .filter_map(|id| doc.layer(id))
        .filter(|l| !l.is_background && !doc.position_locked(l.id) && !doc.pixels_locked(l.id))
        .filter_map(|l| {
            doc.pixel_layers(&[l.id])
                .into_iter()
                .filter_map(|p| doc.layer(p)?.image()?.content_bounds())
                .reduce(|a, b| (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)))
                .map(|b| (l.id, b))
        })
        .collect()
}

fn shift(doc: &mut Document, id: LayerId, dx: i64, dy: i64) {
    if dx == 0 && dy == 0 {
        return;
    }
    let (w, h) = (doc.width, doc.height);
    // A group moves with all its layers
    for p in doc.pixel_layers(&[id]) {
        if let Some(image) = doc.layer_mut(p).and_then(|l| l.image_mut()) {
            *image = image.with_canvas(w, h, dx, dy, [0; 4]);
        }
    }
}

/// Whether `align` can run: a pixel selection with a layer to move, or two
/// or more layers to line up.
pub fn can_align(doc: &Document) -> bool {
    can_align_count(doc, movable_count(doc))
}

/// How many selected layers align and distribute would move (each needs a
/// scan for its pixels' bounds, so callers asking every frame cache it).
pub fn movable_count(doc: &Document) -> usize {
    movable(doc).len()
}

/// [`can_align`] for `n` movable layers.
pub fn can_align_count(doc: &Document, n: usize) -> bool {
    if doc.selection().is_some() {
        n >= 1
    } else {
        n >= 2
    }
}

/// Lines the selected layers up: to the pixel selection's bounds when
/// there is one, else to the box around all of them. Returns whether
/// anything could be aligned.
pub fn align(doc: &mut Document, how: Align) -> bool {
    if !can_align(doc) {
        return false;
    }
    let layers = movable(doc);
    let target: Bounds = match doc.selection().and_then(|s| s.bounds()) {
        Some((x0, y0, x1, y1)) => (x0 as i64, y0 as i64, x1 as i64, y1 as i64),
        None => layers.iter().fold(layers[0].1, |a, (_, b)| {
            (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
        }),
    };
    let center = |a: i64, b: i64| (a + b) as f32 / 2.0;
    for (id, (x0, y0, x1, y1)) in layers {
        let (dx, dy) = match how {
            Align::Left => (target.0 - x0, 0),
            Align::Right => (target.2 - x1, 0),
            Align::HorizontalCenter => (
                (center(target.0, target.2) - center(x0, x1)).round() as i64,
                0,
            ),
            Align::Top => (0, target.1 - y0),
            Align::Bottom => (0, target.3 - y1),
            Align::VerticalCenter => (
                0,
                (center(target.1, target.3) - center(y0, y1)).round() as i64,
            ),
        };
        shift(doc, id, dx, dy);
    }
    doc.mark_dirty();
    true
}

/// Whether `distribute` can run: three or more layers to spread.
pub fn can_distribute(doc: &Document) -> bool {
    movable_count(doc) >= 3
}

/// Spreads the selected layers evenly: the chosen edges or centers at equal
/// steps between the two outermost ones, or (Horizontally, Vertically)
/// equal gaps between the layers. The outermost layers stay put.
pub fn distribute(doc: &mut Document, how: Distribute) -> bool {
    if !can_distribute(doc) {
        return false;
    }
    let horizontal = how.horizontal();
    // A layer's extent along the axis
    let span = |b: Bounds| if horizontal { (b.0, b.2) } else { (b.1, b.3) };
    let key = |b: Bounds| -> f32 {
        let (a, z) = span(b);
        match how {
            Distribute::Left | Distribute::Top => a as f32,
            Distribute::Right | Distribute::Bottom => z as f32,
            _ => (a + z) as f32 / 2.0,
        }
    };
    let mut layers = movable(doc);
    layers.sort_by(|a, b| key(a.1).total_cmp(&key(b.1)));
    let n = layers.len();
    let moves: Vec<(LayerId, i64)> =
        if matches!(how, Distribute::Horizontally | Distribute::Vertically) {
            // Equal gaps: the space left between the first and last layers,
            // shared out
            let first = span(layers[0].1);
            let last = span(layers[n - 1].1);
            let sizes: i64 = layers[1..n - 1]
                .iter()
                .map(|(_, b)| {
                    let (a, z) = span(*b);
                    z - a
                })
                .sum();
            let gap = (last.0 - first.1 - sizes) as f32 / (n - 1) as f32;
            let mut at = first.1 as f32;
            layers[1..n - 1]
                .iter()
                .map(|(id, b)| {
                    let (a, z) = span(*b);
                    at += gap;
                    let d = at.round() as i64 - a;
                    at += (z - a) as f32;
                    (*id, d)
                })
                .collect()
        } else {
            let (first, last) = (key(layers[0].1), key(layers[n - 1].1));
            let step = (last - first) / (n - 1) as f32;
            layers[1..n - 1]
                .iter()
                .enumerate()
                .map(|(k, (id, b))| {
                    (
                        *id,
                        (first + step * (k + 1) as f32 - key(*b)).round() as i64,
                    )
                })
                .collect()
        };
    for (id, d) in moves {
        if horizontal {
            shift(doc, id, d, 0);
        } else {
            shift(doc, id, 0, d);
        }
    }
    doc.mark_dirty();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Layer, TiledImage};

    /// Photoshop 2026's result on three boxes (checked in Photoshop):
    /// Align Left, Vertical Centers, Bottom, then Distribute Vertical Centers.
    #[test]
    fn matches_photoshop() {
        let mut doc = Document::new_with_background("t", 200, 200, Color::WHITE);
        let mut ids = Vec::new();
        for (x0, y0, x1, y1) in [(10, 10, 30, 30), (60, 50, 90, 80), (120, 100, 140, 160)] {
            let mut image = TiledImage::new(200, 200);
            for y in y0..y1 {
                for x in x0..x1 {
                    image.set_pixel(x, y, [255, 0, 0, 255]);
                }
            }
            let id = doc.new_layer_id();
            doc.layers.push(Layer::raster(id, "box", image));
            ids.push(id);
        }
        assert!(!can_align(&doc));
        doc.select_all_layers();
        assert!(align(&mut doc, Align::Left));
        assert!(align(&mut doc, Align::VerticalCenter));
        assert!(align(&mut doc, Align::Bottom));
        assert!(distribute(&mut doc, Distribute::VerticalCenter));
        let bounds = |id| {
            let image = doc.layer(id).unwrap().image().unwrap();
            image.content_bounds().unwrap()
        };
        assert_eq!(bounds(ids[0]), (10, 95, 30, 115));
        assert_eq!(bounds(ids[1]), (10, 80, 40, 110));
        assert_eq!(bounds(ids[2]), (10, 55, 30, 115));
    }

    #[test]
    fn equal_gaps_and_selection_alignment() {
        let mut doc = Document::new_with_background("t", 100, 10, Color::WHITE);
        let mut ids = Vec::new();
        for (x0, x1) in [(0, 10), (12, 22), (80, 100)] {
            let mut image = TiledImage::new(100, 10);
            for x in x0..x1 {
                image.set_pixel(x, 0, [0, 0, 0, 255]);
            }
            let id = doc.new_layer_id();
            doc.layers.push(Layer::raster(id, "bar", image));
            ids.push(id);
        }
        doc.select_all_layers();
        assert!(distribute(&mut doc, Distribute::Horizontally));
        let bounds = |doc: &Document, id| {
            let image = doc.layer(id).unwrap().image().unwrap();
            image.content_bounds().unwrap()
        };
        // Gaps of 30 on both sides of the middle bar
        assert_eq!(bounds(&doc, ids[1]).0, 40);
        // With a pixel selection, a single layer aligns to it
        doc.select_layer(ids[0]);
        doc.set_selection(Some(crate::selection::Selection::rect(
            100,
            10,
            crate::selection::Rect::new(50.0, 0.0, 70.0, 10.0),
        )));
        assert!(align(&mut doc, Align::Right));
        assert_eq!(bounds(&doc, ids[0]).2, 70);
    }

    #[test]
    fn linked_layers_align_with_the_selection() {
        let mut doc = Document::new_with_background("t", 100, 10, Color::WHITE);
        let mut ids = Vec::new();
        for (x0, x1) in [(10, 20), (40, 50), (70, 80)] {
            let mut image = TiledImage::new(100, 10);
            for x in x0..x1 {
                image.set_pixel(x, 0, [0, 0, 0, 255]);
            }
            let id = doc.new_layer_id();
            doc.layers.push(Layer::raster(id, "bar", image));
            ids.push(id);
        }
        // The middle bar linked to the right one; only the left and the
        // middle selected
        doc.set_selected_layers(vec![ids[1], ids[2]]);
        crate::link::link_selected(&mut doc);
        doc.set_selected_layers(vec![ids[0], ids[1]]);
        assert!(align(&mut doc, Align::Left));
        let x0 = |doc: &Document, id| {
            doc.layer(id)
                .unwrap()
                .image()
                .unwrap()
                .content_bounds()
                .unwrap()
                .0
        };
        assert_eq!((x0(&doc, ids[1]), x0(&doc, ids[2])), (10, 10));
    }
}
