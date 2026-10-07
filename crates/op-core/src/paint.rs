//! Painting strokes for the Brush, Pencil and Eraser, and the brush-based
//! retouching tools: Dodge, Burn, Sponge, Blur, Sharpen, Clone Stamp and
//! History Brush.
//!
//! Like Photoshop, Flow builds up within a stroke while Opacity caps it: each
//! stroke keeps a per-pixel coverage (0..1) accumulated from its dabs, and the
//! layer is recomputed from its pre-stroke pixels and `coverage × opacity`.
//! Going over the same spot again within one stroke therefore never exceeds
//! the opacity.

use std::collections::HashMap;

use crate::document::Document;
use crate::layer::LayerId;
use crate::selection::Selection;
use crate::tile::{TILE_SIZE, TiledImage};

/// Shape of the brush tip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrushTip {
    /// Diameter in pixels.
    pub diameter: f32,
    /// 0 = soft (fades from the center), 1 = hard edge.
    pub hardness: f32,
    /// Pencil: no anti-aliasing, a pixel is fully in or out.
    pub aliased: bool,
    /// A square tip (the Eraser's Block mode) instead of a round one.
    pub square: bool,
}

impl BrushTip {
    /// Coverage (0..1) at (`dx`, `dy`) from the dab's center.
    fn coverage(&self, dx: f32, dy: f32) -> f32 {
        if self.square {
            let r = self.diameter / 2.0;
            return if dx.abs() <= r && dy.abs() <= r {
                1.0
            } else {
                0.0
            };
        }
        self.alpha((dx * dx + dy * dy).sqrt())
    }

    /// Coverage (0..1) at distance `d` from the dab's center.
    fn alpha(&self, d: f32) -> f32 {
        let r = self.diameter / 2.0;
        if self.aliased {
            return if d <= r.max(0.5) { 1.0 } else { 0.0 };
        }
        // A one-pixel anti-aliased edge even for hard tips
        let inner = (r * self.hardness).min(r - 0.5).max(0.0);
        if d <= inner {
            1.0
        } else if d >= r + 0.5 {
            0.0
        } else {
            let t = (d - inner) / (r + 0.5 - inner);
            // Smooth falloff between the hard core and the edge
            let s = 1.0 - t;
            s * s * (3.0 - 2.0 * s)
        }
    }
}

/// The tones the Dodge and Burn tools work on ("Range").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToneRange {
    Shadows,
    #[default]
    Midtones,
    Highlights,
}

impl ToneRange {
    pub const ALL: [Self; 3] = [Self::Shadows, Self::Midtones, Self::Highlights];

    pub fn label(self) -> &'static str {
        match self {
            Self::Shadows => "Shadows",
            Self::Midtones => "Midtones",
            Self::Highlights => "Highlights",
        }
    }

    /// How strongly a value `v` (0–1) is affected.
    fn weight(self, v: f32) -> f32 {
        match self {
            Self::Shadows => (1.0 - v) * (1.0 - v),
            Self::Midtones => 4.0 * v * (1.0 - v),
            Self::Highlights => v * v,
        }
    }
}

/// How the Brush and Pencil lay their color on the layer (their "Mode"):
/// a blend mode with the layer's pixels as backdrop, Behind (only where
/// the layer is transparent, as if painted under it) or Clear (erasing).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PaintMode {
    #[default]
    Normal,
    Blend(crate::layer::BlendMode),
    Behind,
    Clear,
}

/// What a stroke does to the pixels it covers.
#[derive(Clone, Debug)]
pub enum StrokeKind {
    /// Paint a color (Brush, Pencil).
    Paint([u8; 3]),
    /// Remove pixels (Eraser). On the background layer, or with transparent
    /// pixels locked, the eraser paints `background` instead, as in Photoshop.
    Erase {
        background: [u8; 3],
    },
    /// Lighten (Dodge tool).
    Dodge(ToneRange),
    /// Darken (Burn tool).
    Burn(ToneRange),
    /// Sponge tool: saturate, or desaturate.
    Sponge {
        saturate: bool,
    },
    Blur,
    Sharpen,
    /// Paint pixels of `image` moved by (`dx`, `dy`): the Clone Stamp
    /// (another spot of the layer) and the History Brush (the layer in an
    /// earlier state, not moved).
    Source {
        image: TiledImage,
        dx: i64,
        dy: i64,
    },
}

/// Why a stroke can't start; the messages match Photoshop's alerts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrokeError {
    NoLayer,
    Locked,
    Hidden,
    /// Painting pixels while a group is the active layer.
    Group,
}

impl StrokeError {
    pub fn message(self, tool: &str) -> String {
        match self {
            Self::NoLayer => {
                format!("Could not use the {tool} because there is no layer to paint on.")
            }
            Self::Locked => format!("Could not use the {tool} because the layer is locked."),
            Self::Hidden => format!("Could not use the {tool} because the target layer is hidden."),
            Self::Group => format!("Could not use the {tool} because the target layer is a group."),
        }
    }
}

/// What a stroke paints on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    Pixels,
    LayerMask,
    QuickMask,
}

pub struct Stroke {
    layer: LayerId,
    target: Target,
    /// The layer's pixels before the stroke.
    base: TiledImage,
    coverage: HashMap<(u32, u32), Box<[f32]>>,
    selection: Option<Selection>,
    tip: BrushTip,
    kind: StrokeKind,
    opacity: f32,
    flow: f32,
    /// Paint the background color instead of erasing / keep alpha.
    preserve_alpha: bool,
    mode: PaintMode,
    last: Option<(f32, f32)>,
    /// Distance travelled since the last dab.
    since_dab: f32,
}

impl Stroke {
    /// Starts a stroke on the document's active layer.
    pub fn begin(
        doc: &Document,
        tip: BrushTip,
        kind: StrokeKind,
        opacity: f32,
        flow: f32,
    ) -> Result<Self, StrokeError> {
        let id = doc.active_layer.ok_or(StrokeError::NoLayer)?;
        let layer = doc.layer(id).ok_or(StrokeError::NoLayer)?;
        // Quick Mask can be painted whatever the layer's state
        let target = if doc.quick_mask.is_some() {
            Target::QuickMask
        } else if doc.editing_mask() {
            Target::LayerMask
        } else {
            Target::Pixels
        };
        if target != Target::QuickMask {
            if !layer.visible {
                return Err(StrokeError::Hidden);
            }
            if doc.pixels_locked(id) {
                return Err(StrokeError::Locked);
            }
            if target == Target::Pixels && layer.is_group() {
                return Err(StrokeError::Group);
            }
        }
        // On a mask colors become grays, and erasing paints the background
        // color's gray
        let gray = |kind| match kind {
            StrokeKind::Paint(c) => StrokeKind::Paint(crate::adjust::mask_gray(c)),
            StrokeKind::Erase { background } => StrokeKind::Erase {
                background: crate::adjust::mask_gray(background),
            },
            other => other,
        };
        let (base, kind) = match target {
            Target::QuickMask => (doc.quick_mask.clone().expect("checked"), gray(kind)),
            Target::LayerMask => (
                layer.mask.as_ref().expect("checked").image.clone(),
                gray(kind),
            ),
            Target::Pixels => (layer.image().expect("checked: not a group").clone(), kind),
        };
        let on_mask = target != Target::Pixels;
        Ok(Self {
            layer: id,
            target,
            base,
            coverage: HashMap::new(),
            selection: doc.selection().cloned(),
            tip,
            kind,
            opacity: opacity.clamp(0.0, 1.0),
            flow: flow.clamp(0.0, 1.0),
            preserve_alpha: on_mask || layer.is_background || doc.transparency_locked(id),
            mode: PaintMode::Normal,
            last: None,
            since_dab: 0.0,
        })
    }

    /// Paints in `mode` (the Brush's and Pencil's Mode).
    pub fn with_mode(mut self, mode: PaintMode) -> Self {
        self.mode = mode;
        self
    }

    /// Distance between dabs: 25% of the diameter, Photoshop's default spacing.
    fn spacing(&self) -> f32 {
        (self.tip.diameter * 0.25).max(1.0)
    }

    /// Continues the stroke to (`x`, `y`) in document pixels, placing dabs
    /// along the way. The first point places a single dab.
    pub fn add_point(&mut self, doc: &mut Document, x: f32, y: f32) {
        let Some((lx, ly)) = self.last else {
            self.dab(doc, x, y);
            self.last = Some((x, y));
            return;
        };
        let (dx, dy) = (x - lx, y - ly);
        let dist = (dx * dx + dy * dy).sqrt();
        let spacing = self.spacing();
        let mut t = spacing - self.since_dab;
        while t <= dist {
            let f = t / dist;
            self.dab(doc, lx + dx * f, ly + dy * f);
            t += spacing;
        }
        self.since_dab = dist - (t - spacing);
        self.last = Some((x, y));
    }

    fn dab(&mut self, doc: &mut Document, cx: f32, cy: f32) {
        let (w, h) = (doc.width, doc.height);
        let r = self.tip.diameter / 2.0 + 1.0;
        let x0 = (cx - r).floor().max(0.0) as u32;
        let y0 = (cy - r).floor().max(0.0) as u32;
        let x1 = ((cx + r).ceil().max(0.0) as u32).min(w);
        let y1 = ((cy + r).ceil().max(0.0) as u32).min(h);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let image = if self.target == Target::QuickMask {
            let Some(q) = &mut doc.quick_mask else {
                return;
            };
            q
        } else {
            let Some(layer) = doc.layer_mut(self.layer) else {
                return;
            };
            match (&mut layer.mask, self.target) {
                (Some(mask), Target::LayerMask) => &mut mask.image,
                _ => match layer.image_mut() {
                    Some(image) => image,
                    None => return,
                },
            }
        };
        for y in y0..y1 {
            for x in x0..x1 {
                let (px, py) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                let a = self.tip.coverage(px, py);
                if a <= 0.0 {
                    continue;
                }
                let key = (x / TILE_SIZE, y / TILE_SIZE);
                let cov = self.coverage.entry(key).or_insert_with(|| {
                    vec![0.0; (TILE_SIZE * TILE_SIZE) as usize].into_boxed_slice()
                });
                let i = ((y % TILE_SIZE) * TILE_SIZE + x % TILE_SIZE) as usize;
                let c = &mut cov[i];
                *c += (1.0 - *c) * a * self.flow;
                let selected = self
                    .selection
                    .as_ref()
                    .map_or(1.0, |s| s.get(x, y) as f32 / 255.0);
                let amount = *c * self.opacity * selected;
                let base = self.base.pixel(x, y);
                let px = match &self.kind {
                    &StrokeKind::Paint(color) if self.mode != PaintMode::Normal => {
                        paint_mode(base, amount, color, self.mode, self.preserve_alpha, (x, y))
                    }
                    StrokeKind::Paint(_) | StrokeKind::Erase { .. } => {
                        apply(base, amount, &self.kind, self.preserve_alpha)
                    }
                    other => {
                        let target = self.target(other, x, y, base);
                        mix(base, target, amount, self.preserve_alpha)
                    }
                };
                image.set_pixel(x, y, px);
            }
        }
        doc.mark_dirty();
    }

    /// What the retouching tools turn the pre-stroke pixel at (`x`, `y`)
    /// into at full strength.
    fn target(&self, kind: &StrokeKind, x: u32, y: u32, base: [u8; 4]) -> [u8; 4] {
        let rgb = [base[0], base[1], base[2]].map(|v| v as f32 / 255.0);
        let out = |c: [f32; 3]| {
            let [r, g, b] = c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
            [r, g, b, base[3]]
        };
        match kind {
            StrokeKind::Dodge(range) => out(rgb.map(|v| v + range.weight(v) * (1.0 - v))),
            StrokeKind::Burn(range) => out(rgb.map(|v| v - range.weight(v) * v)),
            StrokeKind::Sponge { saturate } => {
                let [h, s, l] = crate::adjust::rgb_to_hsl(rgb);
                let s = if *saturate { (s * 2.0).min(1.0) } else { 0.0 };
                out(crate::adjust::hsl_to_rgb([h, s, l]))
            }
            StrokeKind::Blur | StrokeKind::Sharpen => {
                // 3×3 average of the pre-stroke pixels, premultiplied
                let mut sum = [0f32; 4];
                for dy in -1i64..=1 {
                    for dx in -1i64..=1 {
                        let sx = (x as i64 + dx).clamp(0, self.base.width() as i64 - 1) as u32;
                        let sy = (y as i64 + dy).clamp(0, self.base.height() as i64 - 1) as u32;
                        let p = self.base.pixel(sx, sy);
                        let a = p[3] as f32 / 255.0;
                        for c in 0..3 {
                            sum[c] += p[c] as f32 / 255.0 * a;
                        }
                        sum[3] += a;
                    }
                }
                if sum[3] <= 0.0 {
                    return base;
                }
                let blurred = [sum[0] / sum[3], sum[1] / sum[3], sum[2] / sum[3]];
                if matches!(kind, StrokeKind::Blur) {
                    out(blurred)
                } else {
                    out([0, 1, 2].map(|c| rgb[c] + (rgb[c] - blurred[c])))
                }
            }
            StrokeKind::Source { image, dx, dy } => {
                let (sx, sy) = (x as i64 - dx, y as i64 - dy);
                if sx < 0 || sy < 0 || sx >= image.width() as i64 || sy >= image.height() as i64 {
                    return base;
                }
                image.pixel(sx as u32, sy as u32)
            }
            StrokeKind::Paint(_) | StrokeKind::Erase { .. } => base,
        }
    }

    /// The layer the stroke paints on.
    pub fn layer(&self) -> LayerId {
        self.layer
    }

    /// Where the stroke ended (for Shift-click straight lines).
    pub fn last_point(&self) -> Option<(f32, f32)> {
        self.last
    }
}

/// `base` moved toward `target` by `amount`, in premultiplied alpha; with
/// `preserve_alpha` only the color changes.
fn mix(base: [u8; 4], target: [u8; 4], amount: f32, preserve_alpha: bool) -> [u8; 4] {
    let (ba, ta) = (base[3] as f32 / 255.0, target[3] as f32 / 255.0);
    let oa = if preserve_alpha {
        ba
    } else {
        ba + (ta - ba) * amount
    };
    if oa <= 0.0 {
        return [0; 4];
    }
    let mut out = [0u8; 4];
    for c in 0..3 {
        let v = if preserve_alpha {
            base[c] as f32 + (target[c] as f32 - base[c] as f32) * amount
        } else {
            let (b, t) = (base[c] as f32 * ba, target[c] as f32 * ta);
            (b + (t - b) * amount) / oa
        };
        out[c] = v.round().clamp(0.0, 255.0) as u8;
    }
    out[3] = (oa * 255.0).round() as u8;
    out
}

/// One pixel of a Brush or Pencil stroke in a mode other than Normal.
fn paint_mode(
    base: [u8; 4],
    amount: f32,
    color: [u8; 3],
    mode: PaintMode,
    preserve_alpha: bool,
    (x, y): (u32, u32),
) -> [u8; 4] {
    let f = |v: u8| v as f32 / 255.0;
    let dst = [f(base[0]), f(base[1]), f(base[2]), f(base[3])];
    let src = [f(color[0]), f(color[1]), f(color[2])];
    let out = match mode {
        PaintMode::Normal => {
            crate::blend::composite(crate::BlendMode::Normal, dst, src, amount, x, y)
        }
        PaintMode::Blend(m) => crate::blend::composite(m, dst, src, amount, x, y),
        // The layer over the color: only its transparent parts take paint
        PaintMode::Behind => {
            let under = [src[0], src[1], src[2], amount];
            crate::blend::composite(
                crate::BlendMode::Normal,
                under,
                [dst[0], dst[1], dst[2]],
                dst[3],
                x,
                y,
            )
        }
        PaintMode::Clear => [dst[0], dst[1], dst[2], dst[3] * (1.0 - amount)],
    };
    let to = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    if preserve_alpha {
        // The background and locked transparency keep their alpha (Clear
        // and Behind change nothing there)
        if matches!(mode, PaintMode::Clear | PaintMode::Behind) {
            return base;
        }
        return [to(out[0]), to(out[1]), to(out[2]), base[3]];
    }
    if out[3] <= 0.0 {
        return [0; 4];
    }
    [to(out[0]), to(out[1]), to(out[2]), to(out[3])]
}

/// One pixel of the stroke: the pre-stroke pixel `base` changed by `amount`
/// (coverage × opacity × selection).
fn apply(base: [u8; 4], amount: f32, kind: &StrokeKind, preserve_alpha: bool) -> [u8; 4] {
    let mix = |a: u8, b: u8, t: f32| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    match kind {
        &StrokeKind::Erase { background } if preserve_alpha => {
            let [r, g, b, a] = base;
            [
                mix(r, background[0], amount),
                mix(g, background[1], amount),
                mix(b, background[2], amount),
                a,
            ]
        }
        StrokeKind::Erase { .. } => {
            let [r, g, b, a] = base;
            [r, g, b, (a as f32 * (1.0 - amount)).round() as u8]
        }
        &StrokeKind::Paint(color) if preserve_alpha => {
            let [r, g, b, a] = base;
            [
                mix(r, color[0], amount),
                mix(g, color[1], amount),
                mix(b, color[2], amount),
                a,
            ]
        }
        &StrokeKind::Paint(color) => {
            // Source-over of `color` at alpha `amount` onto the straight-alpha base
            let ba = base[3] as f32 / 255.0;
            let oa = amount + ba * (1.0 - amount);
            if oa <= 0.0 {
                return [0; 4];
            }
            let ch = |s: u8, d: u8| {
                ((s as f32 * amount + d as f32 * ba * (1.0 - amount)) / oa).round() as u8
            };
            [
                ch(color[0], base[0]),
                ch(color[1], base[1]),
                ch(color[2], base[2]),
                (oa * 255.0).round() as u8,
            ]
        }
        // The retouching tools go through `mix`
        _ => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Layer};

    fn doc_with_layer() -> (Document, LayerId) {
        let mut doc = Document::new_with_background("t", 40, 40, Color::WHITE);
        let id = doc.new_layer_id();
        doc.layers
            .push(Layer::raster(id, "Layer 1", TiledImage::new(40, 40)));
        doc.active_layer = Some(id);
        (doc, id)
    }

    fn pixel(doc: &Document, id: LayerId, x: u32, y: u32) -> [u8; 4] {
        let img = doc.layer(id).unwrap().image().unwrap();
        img.pixel(x, y)
    }

    const HARD: BrushTip = BrushTip {
        diameter: 10.0,
        hardness: 1.0,
        aliased: false,
        square: false,
    };

    #[test]
    fn hard_brush_paints_full_color_in_its_core() {
        let (mut doc, id) = doc_with_layer();
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Paint([255, 0, 0]), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 20.0, 20.0);
        assert_eq!(pixel(&doc, id, 20, 20), [255, 0, 0, 255]);
        assert_eq!(pixel(&doc, id, 30, 20)[3], 0);
    }

    #[test]
    fn opacity_caps_a_single_stroke() {
        let (mut doc, id) = doc_with_layer();
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Paint([0, 0, 0]), 0.5, 1.0).unwrap();
        // Scrubbing back and forth within one stroke
        for i in 0..20 {
            let x = if i % 2 == 0 { 15.0 } else { 25.0 };
            s.add_point(&mut doc, x, 20.0);
        }
        let a = pixel(&doc, id, 20, 20)[3];
        assert!((126..=129).contains(&a), "{a}");
    }

    #[test]
    fn flow_builds_up() {
        let (mut doc, id) = doc_with_layer();
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Paint([0, 0, 0]), 1.0, 0.2).unwrap();
        s.add_point(&mut doc, 20.0, 20.0);
        let first = pixel(&doc, id, 20, 20)[3];
        s.add_point(&mut doc, 20.5, 20.0);
        s.add_point(&mut doc, 23.0, 20.0);
        s.add_point(&mut doc, 20.0, 20.0);
        assert!(pixel(&doc, id, 20, 20)[3] > first);
    }

    #[test]
    fn selection_masks_paint() {
        let (mut doc, id) = doc_with_layer();
        doc.set_selection(Some(Selection::rect(
            40,
            40,
            crate::selection::Rect::new(0.0, 0.0, 20.0, 40.0),
        )));
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Paint([0, 0, 255]), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 20.0, 20.0);
        assert_eq!(pixel(&doc, id, 18, 20)[3], 255);
        assert_eq!(pixel(&doc, id, 21, 20)[3], 0);
    }

    #[test]
    fn eraser_removes_alpha_and_paints_background_on_background_layer() {
        let (mut doc, id) = doc_with_layer();
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Paint([0, 0, 0]), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 20.0, 20.0);
        let mut e = Stroke::begin(
            &doc,
            HARD,
            StrokeKind::Erase {
                background: [9, 9, 9],
            },
            1.0,
            1.0,
        )
        .unwrap();
        e.add_point(&mut doc, 20.0, 20.0);
        assert_eq!(pixel(&doc, id, 20, 20)[3], 0);

        // On the background layer the eraser paints the background color
        let bg = doc.layers[0].id;
        doc.active_layer = Some(bg);
        let mut e = Stroke::begin(
            &doc,
            HARD,
            StrokeKind::Erase {
                background: [9, 9, 9],
            },
            1.0,
            1.0,
        )
        .unwrap();
        e.add_point(&mut doc, 5.0, 5.0);
        assert_eq!(pixel(&doc, bg, 5, 5), [9, 9, 9, 255]);
    }

    #[test]
    fn pencil_is_aliased() {
        let (mut doc, id) = doc_with_layer();
        let tip = BrushTip {
            diameter: 7.0,
            hardness: 1.0,
            aliased: true,
            square: false,
        };
        let mut s = Stroke::begin(&doc, tip, StrokeKind::Paint([0, 0, 0]), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 20.0, 20.0);
        let img = doc.layer(id).unwrap().image().unwrap();
        for y in 10..30 {
            for x in 10..30 {
                let a = img.pixel(x, y)[3];
                assert!(a == 0 || a == 255);
            }
        }
    }

    #[test]
    fn locked_and_hidden_layers_refuse() {
        let (mut doc, id) = doc_with_layer();
        doc.layer_mut(id).unwrap().lock_pixels = true;
        assert_eq!(
            Stroke::begin(&doc, HARD, StrokeKind::Paint([0; 3]), 1.0, 1.0).err(),
            Some(StrokeError::Locked)
        );
        doc.layer_mut(id).unwrap().lock_pixels = false;
        doc.layer_mut(id).unwrap().visible = false;
        assert_eq!(
            Stroke::begin(&doc, HARD, StrokeKind::Paint([0; 3]), 1.0, 1.0).err(),
            Some(StrokeError::Hidden)
        );
    }

    #[test]
    fn spacing_places_dabs_along_a_line() {
        let (mut doc, id) = doc_with_layer();
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Paint([0, 0, 0]), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 5.0, 20.0);
        s.add_point(&mut doc, 35.0, 20.0);
        for x in 5..35 {
            assert_eq!(pixel(&doc, id, x, 20)[3], 255, "x={x}");
        }
    }

    /// 40×40 opaque layer filled with `rgb`.
    fn doc_filled(rgb: [u8; 4]) -> (Document, LayerId) {
        let mut doc = Document::new_with_background("t", 40, 40, Color::WHITE);
        let id = doc.new_layer_id();
        doc.layers.push(Layer::raster(
            id,
            "Layer 1",
            TiledImage::filled(40, 40, rgb),
        ));
        doc.active_layer = Some(id);
        (doc, id)
    }

    fn one_dab(doc: &mut Document, kind: StrokeKind, strength: f32) {
        let mut s = Stroke::begin(doc, HARD, kind, strength, 1.0).unwrap();
        s.add_point(doc, 20.0, 20.0);
    }

    #[test]
    fn quick_mask_round_trip() {
        let mut doc = Document::new_with_background("t", 40, 40, Color::WHITE);
        doc.layers[0].visible = false;
        doc.enter_quick_mask();
        assert!(doc.selection().is_none());
        // Painting black masks a spot; even a hidden layer doesn't stop it
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Paint([0, 0, 0]), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 20.0, 20.0);
        doc.exit_quick_mask();
        let sel = doc.selection().unwrap();
        assert_eq!(sel.get(20, 20), 0);
        assert_eq!(sel.get(2, 2), 255);
        assert!(doc.quick_mask.is_none());
        // Nothing painted: no selection afterwards
        doc.set_selection(None);
        doc.enter_quick_mask();
        doc.exit_quick_mask();
        assert!(doc.selection().is_none());
    }

    #[test]
    fn dodge_burn_and_sponge() {
        let (mut doc, id) = doc_filled([128, 128, 128, 255]);
        one_dab(&mut doc, StrokeKind::Dodge(ToneRange::Midtones), 0.5);
        let lighter = pixel(&doc, id, 20, 20);
        assert!(lighter[0] > 150 && lighter[0] < 255, "{lighter:?}");
        let (mut doc, id) = doc_filled([128, 128, 128, 255]);
        one_dab(&mut doc, StrokeKind::Burn(ToneRange::Midtones), 0.5);
        assert!(pixel(&doc, id, 20, 20)[0] < 100);
        // Highlights barely touch dark pixels
        let (mut doc, id) = doc_filled([20, 20, 20, 255]);
        one_dab(&mut doc, StrokeKind::Dodge(ToneRange::Highlights), 1.0);
        assert!(pixel(&doc, id, 20, 20)[0] < 25);
        let (mut doc, id) = doc_filled([200, 50, 50, 255]);
        one_dab(&mut doc, StrokeKind::Sponge { saturate: false }, 1.0);
        let gray = pixel(&doc, id, 20, 20);
        assert_eq!((gray[0], gray[1]), (gray[1], gray[2]));
    }

    #[test]
    fn blur_sharpen_and_clone() {
        // A vertical edge at x = 20: black left, white right
        let (mut doc, id) = doc_filled([255, 255, 255, 255]);
        {
            let img = doc.layer_mut(id).unwrap().image_mut().unwrap();
            for y in 0..40 {
                for x in 0..20 {
                    img.set_pixel(x, y, [0, 0, 0, 255]);
                }
            }
        }
        one_dab(&mut doc, StrokeKind::Blur, 1.0);
        let edge = pixel(&doc, id, 19, 20)[0];
        assert!(edge > 0 && edge < 128, "{edge}");

        let (mut doc, id) = doc_filled([100, 100, 100, 255]);
        {
            let img = doc.layer_mut(id).unwrap().image_mut().unwrap();
            img.set_pixel(5, 5, [255, 0, 0, 255]);
        }
        // Clone (5, 5) to (20, 20)
        let img = doc.layer(id).unwrap().image().unwrap();
        let source = StrokeKind::Source {
            image: img.clone(),
            dx: 15,
            dy: 15,
        };
        let mut s = Stroke::begin(&doc, HARD, source, 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 20.5, 20.5);
        assert_eq!(pixel(&doc, id, 20, 20), [255, 0, 0, 255]);
        assert_eq!(pixel(&doc, id, 21, 20), [100, 100, 100, 255]);
    }

    #[test]
    fn paint_modes_blend_with_the_layer() {
        use crate::BlendMode;
        let (mut doc, id) = doc_with_layer();
        // A gray layer, then Multiply with half red
        let image = doc.layer_mut(id).unwrap().image_mut().unwrap();
        for y in 0..40 {
            for x in 0..40 {
                image.set_pixel(x, y, [128, 128, 128, 255]);
            }
        }
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Paint([255, 0, 0]), 1.0, 1.0)
            .unwrap()
            .with_mode(PaintMode::Blend(BlendMode::Multiply));
        s.add_point(&mut doc, 20.0, 20.0);
        assert_eq!(pixel(&doc, id, 20, 20), [128, 0, 0, 255]);
        // Behind paints only transparent pixels; Clear erases
        let (mut doc, id) = doc_with_layer();
        doc.layer_mut(id)
            .unwrap()
            .image_mut()
            .unwrap()
            .set_pixel(20, 20, [0, 0, 255, 255]);
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Paint([255, 0, 0]), 1.0, 1.0)
            .unwrap()
            .with_mode(PaintMode::Behind);
        s.add_point(&mut doc, 20.0, 20.0);
        assert_eq!(pixel(&doc, id, 20, 20), [0, 0, 255, 255]);
        assert_eq!(pixel(&doc, id, 21, 20), [255, 0, 0, 255]);
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Paint([0, 255, 0]), 1.0, 1.0)
            .unwrap()
            .with_mode(PaintMode::Clear);
        s.add_point(&mut doc, 20.0, 20.0);
        assert_eq!(pixel(&doc, id, 20, 20)[3], 0);
    }

    #[test]
    fn a_square_tip_covers_a_block() {
        let (mut doc, id) = doc_with_layer();
        let tip = BrushTip {
            square: true,
            ..HARD
        };
        let mut s = Stroke::begin(&doc, tip, StrokeKind::Paint([0, 0, 0]), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 20.0, 20.0);
        // The corner of the 10 px square is painted, unlike a round tip's
        assert_eq!(pixel(&doc, id, 16, 16)[3], 255);
        assert_eq!(pixel(&doc, id, 26, 20)[3], 0);
    }
}
