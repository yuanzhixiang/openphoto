//! Editable frame containers, raster contents and a clipping mask.
use crate::{Document, Layer, LayerId, TiledImage, layer::LayerMask};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    pub bounds: (u32, u32, u32, u32),
    pub elliptical: bool,
}
impl Frame {
    pub fn valid(self, w: u32, h: u32) -> bool {
        let (x0, y0, x1, y1) = self.bounds;
        x0 < x1 && y0 < y1 && x1 <= w && y1 <= h
    }
    pub fn contains(self, x: f32, y: f32) -> bool {
        let (x0, y0, x1, y1) = self.bounds;
        if x < x0 as f32 || y < y0 as f32 || x >= x1 as f32 || y >= y1 as f32 {
            return false;
        }
        if !self.elliptical {
            return true;
        }
        let nx = (x - (x0 as f32 + x1 as f32) / 2.0) / ((x1 - x0) as f32 / 2.0);
        let ny = (y - (y0 as f32 + y1 as f32) / 2.0) / ((y1 - y0) as f32 / 2.0);
        nx * nx + ny * ny <= 1.0
    }
    fn mask(self, w: u32, h: u32) -> LayerMask {
        LayerMask::from_values(w, h, |x, y| {
            let (x0, y0, x1, y1) = self.bounds;
            if x < x0 || y < y0 || x >= x1 || y >= y1 {
                return 0;
            }
            if !self.elliptical {
                return 255;
            }
            let mut n = 0;
            for sy in 0..4 {
                for sx in 0..4 {
                    n += u32::from(self.contains(
                        x as f32 + (sx as f32 + 0.5) / 4.0,
                        y as f32 + (sy as f32 + 0.5) / 4.0,
                    ));
                }
            }
            ((n * 255 + 8) / 16) as u8
        })
    }
}
pub fn at(doc: &Document, x: f32, y: f32) -> Option<LayerId> {
    doc.layers
        .iter()
        .rev()
        .find(|l| doc.is_shown(l.id) && l.frame.is_some_and(|f| f.contains(x, y)))
        .map(|l| l.id)
}
pub fn create(doc: &mut Document, frame: Frame) -> Option<LayerId> {
    if !frame.valid(doc.width, doc.height) {
        return None;
    }
    let id = doc.new_layer_id();
    let n = doc.layers.iter().filter(|l| l.frame.is_some()).count() + 1;
    let mut group = Layer::group(id, format!("Frame {n}"));
    group.frame = Some(frame);
    group.mask = Some(frame.mask(doc.width, doc.height));
    let mut container = None;
    let mut target = doc.active_layer;
    for _ in 0..doc.layers.len() {
        let Some(layer) = target.and_then(|id| doc.layer(id)) else {
            break;
        };
        if layer.frame.is_some() {
            container = Some(layer.id);
            break;
        }
        target = layer.parent;
    }
    let wrap = doc
        .active_layer
        .filter(|_| container.is_none())
        .and_then(|id| {
            doc.layers.iter().position(|l| {
                l.id == id
                    && l.image().is_some()
                    && !l.lock_all
                    && !l.lock_nesting
                    && !l.lock_pixels
            })
        });
    if let Some(i) = wrap {
        group.parent = doc.layers[i].parent;
        doc.layers[i].parent = Some(id);
        doc.layers[i].is_background = false;
        doc.layers.insert(i + 1, group);
    } else if let Some(i) = container.and_then(|id| doc.layers.iter().position(|l| l.id == id)) {
        group.parent = doc.layers[i].parent;
        doc.layers.insert(i + 1, group);
    } else {
        doc.insert_above_active(group);
    }
    doc.select_layer(id);
    doc.mask_target = false;
    doc.mark_dirty();
    Some(id)
}
pub fn set(doc: &mut Document, id: LayerId, frame: Frame, move_contents: bool) -> bool {
    if !frame.valid(doc.width, doc.height) || doc.position_locked(id) {
        return false;
    }
    let Some(old) = doc.layer(id).and_then(|l| l.frame) else {
        return false;
    };
    if old == frame {
        return false;
    }
    if move_contents {
        let dx = frame.bounds.0 as i64 - old.bounds.0 as i64;
        let dy = frame.bounds.1 as i64 - old.bounds.1 as i64;
        for child in doc.descendants(id) {
            let (w, h) = (doc.width, doc.height);
            if let Some(l) = doc.layer_mut(child) {
                if let Some(image) = l.image_mut() {
                    *image = image.with_canvas(w, h, dx, dy, [0; 4]);
                }
                if let Some(mask) = &mut l.mask {
                    *mask = LayerMask {
                        image: mask.image.with_canvas(w, h, dx, dy, [0, 0, 0, 255]),
                        ..mask.clone()
                    };
                }
            }
        }
    }
    let mask = frame.mask(doc.width, doc.height);
    let l = doc.layer_mut(id).unwrap();
    l.frame = Some(frame);
    l.mask = Some(mask);
    doc.mark_dirty();
    true
}
/// Replaces raster contents with a centered cover fit; original source is not retained as a Smart Object.
pub fn place(doc: &mut Document, id: LayerId, source: &TiledImage, name: &str) -> bool {
    let Some(f) = doc.layer(id).and_then(|l| l.frame) else {
        return false;
    };
    if source.width() == 0 || source.height() == 0 || doc.pixels_locked(id) {
        return false;
    }
    let (x0, y0, x1, y1) = f.bounds;
    let scale =
        ((x1 - x0) as f32 / source.width() as f32).max((y1 - y0) as f32 / source.height() as f32);
    let (w, h) = (
        (source.width() as f32 * scale).round().max(1.0) as u32,
        (source.height() as f32 * scale).round().max(1.0) as u32,
    );
    let ox = x0 as i64 + ((x1 - x0) as i64 - w as i64) / 2;
    let oy = y0 as i64 + ((y1 - y0) as i64 - h as i64) / 2;
    let mut image = TiledImage::new(doc.width, doc.height);
    for y in 0..h {
        for x in 0..w {
            let sx = ((x as f32 + 0.5) / scale).floor() as u32;
            let sy = ((y as f32 + 0.5) / scale).floor() as u32;
            image.set_pixel_at(
                ox + x as i64,
                oy + y as i64,
                source.pixel(sx.min(source.width() - 1), sy.min(source.height() - 1)),
            );
        }
    }
    let ids = doc.descendants(id);
    doc.layers.retain(|l| !ids.contains(&l.id));
    let i = doc.layers.iter().position(|l| l.id == id).unwrap();
    let mut child = Layer::raster(doc.new_layer_id(), name, image);
    child.parent = Some(id);
    doc.layers.insert(i, child);
    doc.mark_dirty();
    true
}
