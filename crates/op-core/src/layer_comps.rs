//! Layer comps: a named record of the layers' visibility, position and
//! appearance, applied back later (Window › Layer Comps).

use crate::document::Document;
use crate::layer::{BlendMode, LayerId};
use crate::tile::TiledImage;

/// What a layer comp records of one layer.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerState {
    pub id: LayerId,
    pub visible: bool,
    /// The top-left of its pixels (None: empty).
    pub position: Option<(i64, i64)>,
    pub opacity: f32,
    pub fill: f32,
    pub blend_mode: BlendMode,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerComp {
    pub name: String,
    pub comment: String,
    /// "Apply To Layers": which of the records it puts back.
    pub visibility: bool,
    pub position: bool,
    pub appearance: bool,
    pub layers: Vec<LayerState>,
}

/// The layers as they are now.
pub fn record(doc: &Document) -> Vec<LayerState> {
    doc.layers
        .iter()
        .map(|l| LayerState {
            id: l.id,
            visible: l.visible,
            position: l
                .image()
                .and_then(|i| i.content_bounds())
                .map(|(x0, y0, _, _)| (x0, y0)),
            opacity: l.opacity,
            fill: l.fill,
            blend_mode: l.blend_mode,
        })
        .collect()
}

/// `image` with its pixels moved by (`dx`, `dy`) (pixels outside the
/// canvas are kept, as layers keep them).
fn shifted(image: &TiledImage, dx: i64, dy: i64) -> TiledImage {
    let mut out = TiledImage::new(image.width(), image.height());
    if let Some((x0, y0, x1, y1)) = image.content_bounds() {
        for y in y0..y1 {
            for x in x0..x1 {
                let p = image.pixel_at(x, y);
                if p[3] > 0 {
                    out.set_pixel_at(x + dx, y + dy, p);
                }
            }
        }
    }
    out
}

/// Puts the comp's records back on the layers that still exist: their
/// visibility, the place of their pixels (not the background's), and
/// their opacity, fill and blend mode, as the comp applies to.
pub fn apply(doc: &mut Document, comp: &LayerComp) {
    for state in &comp.layers {
        let Some(layer) = doc.layer_mut(state.id) else {
            continue;
        };
        if comp.visibility {
            layer.visible = state.visible;
        }
        if comp.appearance {
            layer.opacity = state.opacity;
            layer.fill = state.fill;
            layer.blend_mode = state.blend_mode;
        }
        if comp.position
            && !layer.is_background
            && let Some((x, y)) = state.position
            && let Some(image) = layer.image_mut()
            && let Some((x0, y0, _, _)) = image.content_bounds()
            && (x0, y0) != (x, y)
        {
            *image = shifted(image, x - x0, y - y0);
        }
    }
    doc.mark_dirty();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Layer};

    #[test]
    fn records_and_applies() {
        let mut doc = Document::new_with_background("c", 20, 20, Color::WHITE);
        let mut image = TiledImage::new(20, 20);
        image.set_pixel(2, 3, [255, 0, 0, 255]);
        let layer = Layer::raster(doc.new_layer_id(), "L", image);
        let id = layer.id;
        doc.layers.push(layer);
        let comp = LayerComp {
            name: "A".into(),
            comment: String::new(),
            visibility: true,
            position: true,
            appearance: true,
            layers: record(&doc),
        };
        // Hidden, half opaque, moved
        {
            let l = doc.layer_mut(id).unwrap();
            l.visible = false;
            l.opacity = 0.5;
            let image = l.image_mut().unwrap();
            *image = shifted(image, 5, 5);
        }
        apply(&mut doc, &comp);
        let l = doc.layer(id).unwrap();
        assert!(l.visible && l.opacity == 1.0);
        assert_eq!(l.image().unwrap().pixel(2, 3), [255, 0, 0, 255]);
        assert_eq!(l.image().unwrap().pixel(7, 8)[3], 0);
    }
}
