//! Artboards are clipped group containers in document coordinates.
use crate::{Document, Layer, LayerId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Artboard {
    pub x: i64,
    pub y: i64,
    pub width: u32,
    pub height: u32,
    pub background: Option<[u8; 3]>,
}
impl Artboard {
    pub fn contains(self, x: i64, y: i64) -> bool {
        x >= self.x
            && y >= self.y
            && x < self.x + self.width as i64
            && y < self.y + self.height as i64
    }
    pub fn valid(self) -> bool {
        self.width > 0
            && self.height > 0
            && self.width <= 30_000
            && self.height <= 30_000
            && self.x.abs() <= 30_000
            && self.y.abs() <= 30_000
            && self.x + self.width as i64 <= 30_000
            && self.y + self.height as i64 <= 30_000
    }
}

/// Creates a top-level artboard; intersecting loose layers can be auto-nested.
pub fn create(doc: &mut Document, board: Artboard, auto_nest: bool) -> Option<LayerId> {
    if !board.valid() {
        return None;
    }
    // An artboard document has no locked Background layer: expansion stays transparent.
    for layer in &mut doc.layers {
        layer.is_background = false;
    }
    let number = doc
        .layers
        .iter()
        .filter_map(|l| l.name.strip_prefix("Artboard ")?.parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let id = doc.new_layer_id();
    let mut group = Layer::group(id, format!("Artboard {number}"));
    group.artboard = Some(board);
    if auto_nest {
        let ids: Vec<_> = doc
            .layers
            .iter()
            .filter(|l| {
                l.parent.is_none() && l.artboard.is_none() && !l.lock_nesting && !l.lock_all
            })
            .filter(|l| {
                l.image()
                    .and_then(|i| i.content_bounds())
                    .is_some_and(|(x0, y0, x1, y1)| board.contains((x0 + x1) / 2, (y0 + y1) / 2))
            })
            .map(|l| l.id)
            .collect();
        // Keep an artboard's child block contiguous, including nested groups.
        let mut nested = Vec::new();
        for child in ids {
            if let Some(range) = doc.block(child) {
                let mut block: Vec<_> = doc.layers.drain(range).collect();
                let root = block.last_mut().expect("layer block");
                root.parent = Some(id);
                root.is_background = false;
                nested.extend(block);
            }
        }
        doc.layers.extend(nested);
    }
    doc.layers.push(group);
    doc.select_layer(id);
    doc.mask_target = false;
    ensure_canvas(doc);
    doc.mark_dirty();
    Some(id)
}

pub fn at(doc: &Document, x: i64, y: i64) -> Option<LayerId> {
    doc.layers
        .iter()
        .rev()
        .find(|l| doc.is_shown(l.id) && l.artboard.is_some_and(|b| b.contains(x, y)))
        .map(|l| l.id)
}

/// Resize only the clipping rectangle; moving also translates all descendants.
pub fn set(doc: &mut Document, id: LayerId, board: Artboard, move_contents: bool) -> bool {
    if !board.valid() || doc.position_locked(id) {
        return false;
    }
    let Some(old) = doc.layer(id).and_then(|l| l.artboard) else {
        return false;
    };
    if old == board {
        return false;
    }
    if move_contents {
        let (dx, dy) = (board.x - old.x, board.y - old.y);
        for child in doc.descendants(id) {
            let (w, h) = (doc.width, doc.height);
            if let Some(layer) = doc.layer_mut(child) {
                if let Some(image) = layer.image_mut() {
                    *image = image.with_canvas(w, h, dx, dy, [0; 4]);
                }
                if let Some(mask) = &mut layer.mask {
                    mask.image = mask.image.with_canvas(w, h, dx, dy, [0, 0, 0, 255]);
                }
                if let Some(b) = &mut layer.artboard {
                    b.x += dx;
                    b.y += dy;
                }
            }
        }
    }
    doc.layer_mut(id).expect("artboard").artboard = Some(board);
    ensure_canvas(doc);
    doc.mark_dirty();
    true
}

/// Reparent selected whole-layer blocks by content center on Move release.
/// Nesting locks and membership in ordinary groups prevent extraction.
pub fn auto_nest(doc: &mut Document) -> bool {
    let selected = doc.selected_layers();
    let roots: Vec<_> = selected
        .iter()
        .copied()
        .filter(|id| {
            let mut parent = doc.layer(*id).and_then(|l| l.parent);
            while let Some(id) = parent {
                if selected.contains(&id) {
                    return false;
                }
                parent = doc.layer(id).and_then(|l| l.parent);
            }
            true
        })
        .collect();
    let mut changed = false;
    for id in roots {
        let Some(layer) = doc.layer(id) else {
            continue;
        };
        if layer.artboard.is_some() || layer.is_background || layer.lock_nesting || layer.lock_all {
            continue;
        }
        let parent = layer.parent;
        if parent.is_some_and(|id| {
            doc.layer(id)
                .is_some_and(|l| l.artboard.is_none() || l.lock_nesting || l.lock_all)
        }) {
            continue;
        }
        let mut bounds: Option<(i64, i64, i64, i64)> = None;
        for child in doc.pixel_layers(&[id]) {
            if let Some(b) = doc
                .layer(child)
                .and_then(|l| l.image())
                .and_then(|i| i.content_bounds())
            {
                bounds = Some(match bounds {
                    None => b,
                    Some((x0, y0, x1, y1)) => (x0.min(b.0), y0.min(b.1), x1.max(b.2), y1.max(b.3)),
                });
            }
        }
        let Some((x0, y0, x1, y1)) = bounds else {
            continue;
        };
        let target = at(doc, (x0 + x1) / 2, (y0 + y1) / 2);
        if target == parent
            || target.is_some_and(|id| doc.layer(id).is_some_and(|l| l.lock_nesting || l.lock_all))
        {
            continue;
        }
        let Some(range) = doc.block(id) else {
            continue;
        };
        let mut block: Vec<_> = doc.layers.drain(range).collect();
        block.last_mut().expect("block root").parent = target;
        let index = target
            .and_then(|id| doc.layers.iter().position(|l| l.id == id))
            .unwrap_or(doc.layers.len());
        doc.layers.splice(index..index, block);
        changed = true;
    }
    if changed {
        doc.mark_dirty();
    }
    changed
}

/// Expand the document to contain every artboard, retaining all layer content.
pub fn ensure_canvas(doc: &mut Document) {
    let boards: Vec<_> = doc.layers.iter().filter_map(|l| l.artboard).collect();
    let left = boards.iter().map(|b| b.x).min().unwrap_or(0).min(0);
    let top = boards.iter().map(|b| b.y).min().unwrap_or(0).min(0);
    let right = boards
        .iter()
        .map(|b| b.x + b.width as i64)
        .max()
        .unwrap_or(0)
        .max(doc.width as i64);
    let bottom = boards
        .iter()
        .map(|b| b.y + b.height as i64)
        .max()
        .unwrap_or(0)
        .max(doc.height as i64);
    if left != 0 || top != 0 || right != doc.width as i64 || bottom != doc.height as i64 {
        doc.place_canvas(
            (right - left) as u32,
            (bottom - top) as u32,
            -left,
            -top,
            crate::Color::TRANSPARENT,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, TiledImage};
    #[test]
    fn artboard_auto_nesting_preserves_blocks_and_lock() {
        let mut doc = Document::new_with_background("t", 30, 30, Color::TRANSPARENT);
        doc.layers.clear();
        let id = create(
            &mut doc,
            Artboard {
                x: 10,
                y: 10,
                width: 10,
                height: 10,
                background: None,
            },
            false,
        )
        .unwrap();
        let dot = doc.new_layer_id();
        let mut image = TiledImage::new(30, 30);
        image.set_pixel(12, 12, [255; 4]);
        doc.layers.push(Layer::raster(dot, "dot", image));
        doc.select_layer(dot);
        assert!(auto_nest(&mut doc));
        assert_eq!(doc.layer(dot).unwrap().parent, Some(id));
        assert_eq!(doc.layers.last().unwrap().id, id);
        doc.layer_mut(dot).unwrap().lock_nesting = true;
        doc.layer_mut(dot)
            .unwrap()
            .image_mut()
            .unwrap()
            .set_pixel(12, 12, [0; 4]);
        doc.layer_mut(dot)
            .unwrap()
            .image_mut()
            .unwrap()
            .set_pixel(2, 2, [255; 4]);
        assert!(!auto_nest(&mut doc));
        doc.layer_mut(dot).unwrap().lock_nesting = false;
        assert!(auto_nest(&mut doc));
        assert_eq!(doc.layer(dot).unwrap().parent, None);
    }

    #[test]
    fn empty_artboard_background_and_move_are_real_content() {
        let mut doc = Document::new_with_background("t", 20, 20, Color::TRANSPARENT);
        doc.layers.clear();
        let b = Artboard {
            x: 2,
            y: 3,
            width: 5,
            height: 6,
            background: Some([0; 3]),
        };
        let id = create(&mut doc, b, false).unwrap();
        let pixels = doc.composite_rgba8();
        assert_eq!(
            &pixels[(3 * 20 + 2) * 4..(3 * 20 + 2) * 4 + 4],
            &[0, 0, 0, 255]
        );
        assert_eq!(pixels[3], 0);
        crate::move_tool::Move::begin(&doc, [0; 3])
            .unwrap()
            .apply(&mut doc, 3, 2);
        assert_eq!(
            doc.layer(id).unwrap().artboard,
            Some(Artboard { x: 5, y: 5, ..b })
        );
        assert_eq!(doc.composite_rgba8()[(3 * 20 + 2) * 4 + 3], 0);
    }
    #[test]
    fn negative_artboard_expansion_keeps_loose_pixels_and_coordinates() {
        let mut doc = Document::new_with_background("t", 12, 12, Color::WHITE);
        let original = doc.layers[0].id;
        let b = Artboard {
            x: -5,
            y: -3,
            width: 4,
            height: 2,
            background: None,
        };
        let id = create(&mut doc, b, false).unwrap();
        assert_eq!((doc.width, doc.height), (17, 15));
        assert_eq!(
            doc.layer(id).unwrap().artboard,
            Some(Artboard { x: 0, y: 0, ..b })
        );
        assert_eq!(
            doc.layer(original).unwrap().image().unwrap().pixel(5, 3),
            [255; 4]
        );
    }

    #[test]
    fn artboard_clips_children_and_keeps_pixels_when_resized() {
        let mut doc = Document::new_with_background("t", 20, 20, Color::TRANSPARENT);
        doc.layers.clear();
        let child = doc.new_layer_id();
        doc.layers.push(Layer::raster(
            child,
            "red",
            TiledImage::filled(20, 20, [255, 0, 0, 255]),
        ));
        let b = Artboard {
            x: 5,
            y: 5,
            width: 10,
            height: 10,
            background: Some([255; 3]),
        };
        let id = create(&mut doc, b, true).unwrap();
        assert_eq!(doc.layer(child).unwrap().parent, Some(id));
        assert_eq!(doc.composite_rgba8()[0], 0);
        assert_eq!(doc.layer_at(1, 1), None);
        assert_eq!(doc.layer_at(7, 7), Some(child));
        assert!(set(
            &mut doc,
            id,
            Artboard {
                width: 15,
                height: 15,
                ..b
            },
            false
        ));
        assert_eq!(
            doc.layer(child).unwrap().image().unwrap().pixel(19, 19),
            [255, 0, 0, 255]
        );
        assert_eq!(doc.layer_at(19, 19), Some(child));
    }
    #[test]
    fn artboard_moves_children_and_expands_canvas_with_undo_snapshot() {
        let mut doc = Document::new_with_background("t", 12, 12, Color::TRANSPARENT);
        let child = doc.layers[0].id;
        doc.layers[0]
            .image_mut()
            .unwrap()
            .set_pixel(4, 4, [20, 30, 40, 255]);
        let b = Artboard {
            x: 2,
            y: 2,
            width: 6,
            height: 6,
            background: None,
        };
        let id = create(&mut doc, b, true).unwrap();
        let before = doc.snapshot();
        assert!(set(&mut doc, id, Artboard { x: 10, y: 8, ..b }, true));
        assert_eq!((doc.width, doc.height), (16, 14));
        assert_eq!(
            doc.layer(child).unwrap().image().unwrap().pixel(12, 10),
            [20, 30, 40, 255]
        );
        doc.restore(&before);
        assert_eq!((doc.width, doc.height), (12, 12));
        assert_eq!(doc.layer(id).unwrap().artboard, Some(b));
    }
    #[test]
    fn artboard_auto_nesting_respects_lock_and_rejects_invalid_size() {
        let mut doc = Document::new_with_background("t", 12, 12, Color::WHITE);
        doc.layers[0].lock_nesting = true;
        let b = Artboard {
            x: 0,
            y: 0,
            width: 12,
            height: 12,
            background: None,
        };
        let id = create(&mut doc, b, true).unwrap();
        assert_eq!(doc.layers[0].parent, None);
        assert_eq!(at(&doc, 3, 3), Some(id));
        assert!(create(&mut doc, Artboard { width: 0, ..b }, true).is_none());
        doc.layer_mut(id).unwrap().lock_position = true;
        assert!(!set(&mut doc, id, Artboard { x: 3, ..b }, true));
    }
}
