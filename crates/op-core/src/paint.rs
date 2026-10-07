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
    /// Brush Tip Shape: the tip's angle (degrees, counterclockwise) and
    /// roundness (its height over its width, 0.01–1), and the spacing
    /// between dabs as a fraction of the diameter.
    pub angle: f32,
    pub roundness: f32,
    pub spacing: f32,
}

/// The Clone Source panel's transform of a `Source` stroke: the source
/// point, and how the source is scaled (W, H) and turned (degrees,
/// counterclockwise) about it on its way to the target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceTransform {
    pub origin: (f32, f32),
    pub scale: (f32, f32),
    pub angle: f32,
}

/// The Art History Brush's Style.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ArtStyle {
    #[default]
    TightShort,
    TightMedium,
    TightLong,
    LooseMedium,
    LooseLong,
    Dab,
    TightCurl,
    TightCurlLong,
    LooseCurl,
    LooseCurlLong,
}

impl ArtStyle {
    pub const ALL: [ArtStyle; 10] = [
        ArtStyle::TightShort,
        ArtStyle::TightMedium,
        ArtStyle::TightLong,
        ArtStyle::LooseMedium,
        ArtStyle::LooseLong,
        ArtStyle::Dab,
        ArtStyle::TightCurl,
        ArtStyle::TightCurlLong,
        ArtStyle::LooseCurl,
        ArtStyle::LooseCurlLong,
    ];

    /// A stroke's length in brush diameters, how far its direction strays
    /// (0 tight – 1 loose), and whether it curls.
    fn shape(self) -> (f32, f32, bool) {
        match self {
            ArtStyle::TightShort => (1.5, 0.1, false),
            ArtStyle::TightMedium => (3.0, 0.1, false),
            ArtStyle::TightLong => (6.0, 0.1, false),
            ArtStyle::LooseMedium => (3.0, 0.6, false),
            ArtStyle::LooseLong => (6.0, 0.6, false),
            ArtStyle::Dab => (0.0, 0.0, false),
            ArtStyle::TightCurl => (3.0, 0.1, true),
            ArtStyle::TightCurlLong => (6.0, 0.1, true),
            ArtStyle::LooseCurl => (3.0, 0.6, true),
            ArtStyle::LooseCurlLong => (6.0, 0.6, true),
        }
    }
}

/// The retouching tools' options: Dodge's and Burn's Protect Tones, the
/// Sponge's Vibrance, the Sharpen tool's Protect Detail.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Retouch {
    pub protect_tones: bool,
    pub vibrance: bool,
    pub protect_detail: bool,
}

/// Which of a stroke's settings follow the pen's pressure (Shape
/// Dynamics' Size Jitter control and Transfer's Opacity and Flow Jitter
/// controls set to Pen Pressure).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pressure {
    pub size: bool,
    pub opacity: bool,
    pub flow: bool,
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
        // An elliptical tip: turn the offset into the tip's frame and
        // stretch its short axis
        let (dx, dy) = if self.angle != 0.0 || self.roundness < 1.0 {
            let (s, c) = self.angle.to_radians().sin_cos();
            // (screen y points down; the angle turns counterclockwise)
            let u = dx * c - dy * s;
            let v = dx * s + dy * c;
            (u, v / self.roundness.clamp(0.01, 1.0))
        } else {
            (dx, dy)
        };
        self.alpha((dx * dx + dy * dy).sqrt())
    }

    /// Coverage (0..1) at distance `d` from the dab's center.
    pub(crate) fn alpha(&self, d: f32) -> f32 {
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

/// Where the Background Eraser and Color Replacement tools take the color
/// they act on ("Sampling").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sampling {
    /// Under the brush's center at every dab.
    Continuous,
    /// Under the brush's center where the stroke starts.
    Once,
    /// A given color (the background swatch).
    Swatch([u8; 3]),
}

/// Which pixels the Background Eraser and Color Replacement change: those
/// within `tolerance` (0–1, of the largest channel difference) of the
/// sampled color, either all of them under the brush or only those
/// connected to its center (`contiguous`); `protect` keeps pixels close to
/// that color (the Background Eraser's Protect Foreground Color).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorMatch {
    pub sampling: Sampling,
    pub tolerance: f32,
    pub contiguous: bool,
    pub protect: Option<[u8; 3]>,
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
    /// Paint a pattern tiled from the document's origin (Pattern Stamp,
    /// Aligned).
    Pattern(TiledImage),
    /// Push the pixels along the stroke (Smudge): each dab pulls the
    /// colors under the previous dab with this strength (0–1).
    Smudge(f32),
    /// The Art History Brush: stylized strokes in the colors of `source`
    /// (the History panel's source state), scattered over `area` pixels
    /// around each dab, kept off areas within `tolerance` (0–1) of it.
    ArtHistory {
        source: TiledImage,
        style: ArtStyle,
        area: f32,
        tolerance: f32,
    },
    /// The Mixer Brush: paint loaded into the brush (`color`; None for a
    /// clean brush) laid down while it lasts (`load`), picking up the
    /// canvas's paint (`wet`) and mixing it in (`mix`), all 0–1.
    Mix {
        color: Option<[u8; 3]>,
        wet: f32,
        load: f32,
        mix: f32,
    },
    /// Erase the pixels matching a sampled color (Background Eraser).
    BackgroundErase(ColorMatch),
    /// Heal with the texture of `source` moved by (`dx`, `dy`) (Healing
    /// Brush: the Alt-clicked source).
    Heal {
        source: TiledImage,
        dx: i64,
        dy: i64,
    },
    /// Heal with texture found around each dab (Spot Healing Brush,
    /// Proximity Match).
    SpotHeal(TiledImage),
    /// Give the pixels matching a sampled color the hue, saturation,
    /// color or luminosity (`mode`) of `color` (Color Replacement).
    ReplaceColor {
        color: [u8; 3],
        mode: crate::layer::BlendMode,
        matching: ColorMatch,
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
    /// The color sampled once (Sampling: Once).
    sampled: Option<[u8; 3]>,
    /// Where the previous dab was (Smudge).
    last_dab: Option<(f32, f32)>,
    last: Option<(f32, f32)>,
    /// What the pen's pressure controls, and the pressure at the last
    /// point and at the dab being placed.
    pressure: Pressure,
    last_pressure: f32,
    dab_pressure: f32,
    /// Distance travelled since the last dab.
    since_dab: f32,
    retouch: Retouch,
    /// The Pattern Stamp's Impressionist, and the dab being placed.
    impressionist: bool,
    dab_center: (f32, f32),
    /// The Mixer Brush's paint: its color (None while clean) and how much
    /// of the load is left (0–1).
    mixer: (Option<[f32; 3]>, f32),
    /// The Clone Source panel's scale and angle for a `Source` stroke.
    source_transform: Option<SourceTransform>,
    /// The healing tools' Mode and Diffusion, and the Spot Healing Brush's
    /// Create Texture.
    heal_style: crate::heal::HealStyle,
    create_texture: bool,
    /// Sample All Layers (Blur, Sharpen, Smudge, the Mixer Brush): the
    /// merged image the stroke reads, kept up to date with what it lays
    /// down.
    sample: Option<TiledImage>,
    /// Where a `Pattern` stroke's pattern starts (the Pattern Stamp with
    /// Aligned off: the stroke's first point; otherwise the document's
    /// corner).
    pattern_origin: (u32, u32),
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
            sampled: None,
            last_dab: None,
            last: None,
            since_dab: 0.0,
            pressure: Pressure::default(),
            last_pressure: 1.0,
            dab_pressure: 1.0,
            retouch: Retouch::default(),
            impressionist: false,
            dab_center: (0.0, 0.0),
            mixer: (None, 1.0),
            source_transform: None,
            heal_style: crate::heal::HealStyle::default(),
            create_texture: false,
            sample: None,
            pattern_origin: (0, 0),
        })
    }

    /// The Pattern Stamp with Aligned off: the pattern starts at `origin`
    /// for this stroke instead of at the document's corner.
    pub fn with_pattern_origin(mut self, origin: (u32, u32)) -> Self {
        self.pattern_origin = origin;
        self
    }

    /// Sample All Layers: Blur, Sharpen, Smudge and the Mixer Brush read
    /// `merged` (the visible layers composited, the document's size)
    /// instead of the layer, and write their result into the layer.
    pub fn with_sample(mut self, merged: TiledImage) -> Self {
        self.sample = Some(merged);
        self
    }

    /// A healing stroke's Mode and Diffusion.
    pub fn with_heal_style(mut self, style: crate::heal::HealStyle) -> Self {
        self.heal_style = style;
        self
    }

    /// The Spot Healing Brush's Create Texture: each dab heals with a
    /// texture made from the pixels around it instead of a nearby spot.
    pub fn with_create_texture(mut self, on: bool) -> Self {
        self.create_texture = on;
        self
    }

    /// Scales and turns a `Source` stroke's source about its source point
    /// (the Clone Source panel's W, H and angle).
    pub fn with_source_transform(mut self, transform: SourceTransform) -> Self {
        self.source_transform = Some(transform);
        self
    }

    /// The Mixer Brush's color at the end of the stroke (None if clean):
    /// what the brush keeps when it isn't cleaned after the stroke.
    pub fn mixer_color(&self) -> Option<[u8; 3]> {
        self.mixer
            .0
            .map(|c| c.map(|v| (v * 255.0).round().clamp(0.0, 255.0) as u8))
    }

    /// The Pattern Stamp's Impressionist: each dab paints the pattern's
    /// color near its center, in daubs.
    pub fn with_impressionist(mut self, on: bool) -> Self {
        self.impressionist = on;
        self
    }

    /// The retouching tools' options.
    pub fn with_retouch(mut self, retouch: Retouch) -> Self {
        self.retouch = retouch;
        self
    }

    /// Another dab where the stroke is (the airbrush building up while the
    /// pointer holds still).
    pub fn build_up(&mut self, doc: &mut Document) {
        if let Some((x, y)) = self.last {
            self.dab(doc, x, y);
        }
    }

    /// Lets the pen's pressure control size, opacity and flow.
    pub fn with_pressure(mut self, pressure: Pressure) -> Self {
        self.pressure = pressure;
        self
    }

    /// Paints in `mode` (the Brush's and Pencil's Mode).
    pub fn with_mode(mut self, mode: PaintMode) -> Self {
        self.mode = mode;
        self
    }

    /// The tip's diameter at the current dab's pressure (Size: at least
    /// one pixel).
    fn diameter(&self) -> f32 {
        if self.pressure.size {
            (self.tip.diameter * self.dab_pressure).max(1.0)
        } else {
            self.tip.diameter
        }
    }

    /// Distance between dabs: the tip's spacing (Photoshop's default 25%)
    /// of the diameter at that moment.
    fn spacing(&self) -> f32 {
        // The Art History Brush scatters a handful of strokes every half
        // of its area
        if let StrokeKind::ArtHistory { area, .. } = self.kind {
            return (area * 0.5).max(1.0);
        }
        (self.diameter() * self.tip.spacing.clamp(0.01, 10.0)).max(1.0)
    }

    /// Continues the stroke to (`x`, `y`) in document pixels, placing dabs
    /// along the way. The first point places a single dab.
    pub fn add_point(&mut self, doc: &mut Document, x: f32, y: f32) {
        self.add_point_with_pressure(doc, x, y, 1.0);
    }

    /// [`Self::add_point`] with the pen's pressure there (0–1), interpolated
    /// from the last point's along the way.
    pub fn add_point_with_pressure(&mut self, doc: &mut Document, x: f32, y: f32, pressure: f32) {
        let pressure = pressure.clamp(0.0, 1.0);
        let Some((lx, ly)) = self.last else {
            self.dab_pressure = pressure;
            self.dab(doc, x, y);
            self.last = Some((x, y));
            self.last_pressure = pressure;
            return;
        };
        let (dx, dy) = (x - lx, y - ly);
        let dist = (dx * dx + dy * dy).sqrt();
        let p0 = self.last_pressure;
        let mut t = self.spacing() - self.since_dab;
        let mut placed = None;
        while t <= dist {
            let f = t / dist;
            self.dab_pressure = p0 + (pressure - p0) * f;
            self.dab(doc, lx + dx * f, ly + dy * f);
            placed = Some(t);
            t += self.spacing();
        }
        self.since_dab = match placed {
            Some(at) => dist - at,
            None => self.since_dab + dist,
        };
        self.last = Some((x, y));
        self.last_pressure = pressure;
    }

    fn dab(&mut self, doc: &mut Document, cx: f32, cy: f32) {
        if let StrokeKind::Smudge(strength) = self.kind {
            self.smudge_dab(doc, cx, cy, strength);
            return;
        }
        if let StrokeKind::ArtHistory { .. } = self.kind {
            self.art_dab(doc, cx, cy);
            return;
        }
        if let StrokeKind::Mix {
            color,
            wet,
            load,
            mix,
        } = self.kind
        {
            self.mixer_dab(doc, cx, cy, (color, wet, load, mix));
            return;
        }
        if matches!(self.kind, StrokeKind::Heal { .. } | StrokeKind::SpotHeal(_)) {
            self.heal_dab(doc, cx, cy);
            return;
        }
        let (w, h) = (doc.width, doc.height);
        self.dab_center = (cx, cy);
        let tip = BrushTip {
            diameter: self.diameter(),
            ..self.tip
        };
        // (an elliptical tip's long axis is the diameter)
        let r = tip.diameter / 2.0 + 1.0;
        // Pressure lowers this dab's flow, or the coverage it can reach
        let flow = if self.pressure.flow {
            self.flow * self.dab_pressure
        } else {
            self.flow
        };
        let cap = if self.pressure.opacity {
            self.dab_pressure
        } else {
            1.0
        };
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
        // The retouching tools work on the pixels as they are now, each dab
        // adding to the last, as Photoshop's do
        if matches!(
            self.kind,
            StrokeKind::Dodge(_)
                | StrokeKind::Burn(_)
                | StrokeKind::Sponge { .. }
                | StrokeKind::Blur
                | StrokeKind::Sharpen
        ) {
            let (mx0, my0) = (x0.saturating_sub(1), y0.saturating_sub(1));
            let (mx1, my1) = ((x1 + 1).min(w), (y1 + 1).min(h));
            let bw = mx1 - mx0;
            // Blur and Sharpen with Sample All Layers read the merged image
            let sampled = matches!(self.kind, StrokeKind::Blur | StrokeKind::Sharpen)
                .then_some(self.sample.as_mut())
                .flatten();
            let mut now = Vec::with_capacity((bw * (my1 - my0)) as usize);
            for y in my0..my1 {
                for x in mx0..mx1 {
                    now.push(match &sampled {
                        Some(m) => m.pixel(x, y),
                        None => image.pixel(x, y),
                    });
                }
            }
            let mut sampled = sampled;
            let at = |x: i64, y: i64| {
                let x = x.clamp(mx0 as i64, mx1 as i64 - 1) as u32;
                let y = y.clamp(my0 as i64, my1 as i64 - 1) as u32;
                now[((y - my0) * bw + (x - mx0)) as usize]
            };
            for y in y0..y1 {
                for x in x0..x1 {
                    let (px, py) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                    let a = tip.coverage(px, py);
                    if a <= 0.0 {
                        continue;
                    }
                    let selected = self
                        .selection
                        .as_ref()
                        .map_or(1.0, |s| s.get(x, y) as f32 / 255.0);
                    let amount = a * flow * cap * self.opacity * selected;
                    let cur = at(x as i64, y as i64);
                    let target = retouch_target(&self.kind, self.retouch, cur, |dx, dy| {
                        at(x as i64 + dx, y as i64 + dy)
                    });
                    if let Some(m) = sampled.as_deref_mut() {
                        m.set_pixel(x, y, mix(cur, target, amount, false));
                        let own = image.pixel(x, y);
                        image.set_pixel(x, y, mix(own, target, amount, self.preserve_alpha));
                    } else {
                        image.set_pixel(x, y, mix(cur, target, amount, self.preserve_alpha));
                    }
                }
            }
            doc.mark_dirty();
            return;
        }
        // The Background Eraser and Color Replacement change only the
        // pixels matching the sampled color
        let matched = self.match_mask(cx, cy, (x0, y0, x1, y1));
        for y in y0..y1 {
            for x in x0..x1 {
                let (px, py) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                let a = tip.coverage(px, py);
                if a <= 0.0 {
                    continue;
                }
                if let Some(m) = &matched
                    && !m[((y - y0) * (x1 - x0) + (x - x0)) as usize]
                {
                    continue;
                }
                let key = (x / TILE_SIZE, y / TILE_SIZE);
                let cov = self.coverage.entry(key).or_insert_with(|| {
                    vec![0.0; (TILE_SIZE * TILE_SIZE) as usize].into_boxed_slice()
                });
                let i = ((y % TILE_SIZE) * TILE_SIZE + x % TILE_SIZE) as usize;
                let c = &mut cov[i];
                if *c < cap {
                    *c += (cap - *c) * a * flow;
                }
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
                    StrokeKind::BackgroundErase(_) => {
                        let [r, g, b, a] = base;
                        [r, g, b, (a as f32 * (1.0 - amount)).round() as u8]
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
            StrokeKind::Dodge(_)
            | StrokeKind::Burn(_)
            | StrokeKind::Sponge { .. }
            | StrokeKind::Blur
            | StrokeKind::Sharpen => retouch_target(kind, self.retouch, base, |dx, dy| {
                let sx = (x as i64 + dx).clamp(0, self.base.width() as i64 - 1) as u32;
                let sy = (y as i64 + dy).clamp(0, self.base.height() as i64 - 1) as u32;
                self.base.pixel(sx, sy)
            }),
            StrokeKind::Source { image, dx, dy } => {
                let (sx, sy) = match self.source_transform {
                    // From the target point the source point lands on, back
                    // through the turn and the scale
                    // (about the middle of the source point's pixel, so an
                    // unchanged transform clones exactly as without one)
                    Some(t) => {
                        let (ox, oy) = (t.origin.0.floor() + 0.5, t.origin.1.floor() + 0.5);
                        let (tx, ty) = (ox + *dx as f32, oy + *dy as f32);
                        let (rx, ry) = (x as f32 + 0.5 - tx, y as f32 + 0.5 - ty);
                        let (s, c) = t.angle.to_radians().sin_cos();
                        // (screen y points down: counterclockwise is −y)
                        let (ux, uy) = (rx * c - ry * s, rx * s + ry * c);
                        let (sx, sy) = (ux / t.scale.0.max(0.01), uy / t.scale.1.max(0.01));
                        ((ox + sx).floor() as i64, (oy + sy).floor() as i64)
                    }
                    None => (x as i64 - dx, y as i64 - dy),
                };
                if sx < 0 || sy < 0 || sx >= image.width() as i64 || sy >= image.height() as i64 {
                    return base;
                }
                image.pixel(sx as u32, sy as u32)
            }
            StrokeKind::Pattern(image) => {
                let (pw, ph) = (image.width().max(1), image.height().max(1));
                // Impressionist: the color at the dab's center, a little off
                // by the pixel's quarter of the tip, so dabs read as daubs
                let (x, y) = if self.impressionist {
                    let (cx, cy) = self.dab_center;
                    let r = (self.tip.diameter / 4.0).max(1.0);
                    let jx = ((x / 3) % 3) as f32 - 1.0;
                    let jy = ((y / 3) % 3) as f32 - 1.0;
                    ((cx + jx * r).max(0.0) as u32, (cy + jy * r).max(0.0) as u32)
                } else {
                    (x, y)
                };
                let (ox, oy) = (self.pattern_origin.0 % pw, self.pattern_origin.1 % ph);
                let p = image.pixel((x + pw - ox) % pw, (y + ph - oy) % ph);
                [p[0], p[1], p[2], base[3].max(p[3])]
            }
            &StrokeKind::ReplaceColor { color, mode, .. } => {
                let c = color.map(|v| v as f32 / 255.0);
                out(crate::blend::blend(mode, rgb, c))
            }
            StrokeKind::Paint(_)
            | StrokeKind::Erase { .. }
            | StrokeKind::Smudge(_)
            | StrokeKind::Mix { .. }
            | StrokeKind::ArtHistory { .. }
            | StrokeKind::BackgroundErase(_)
            | StrokeKind::Heal { .. }
            | StrokeKind::SpotHeal(_) => base,
        }
    }

    /// For the Background Eraser and Color Replacement: which pixels of the
    /// dab's box (`x0`, `y0`) to (`x1`, `y1`) match the sampled color, row
    /// by row; `None` for the other strokes.
    fn match_mask(
        &mut self,
        cx: f32,
        cy: f32,
        (x0, y0, x1, y1): (u32, u32, u32, u32),
    ) -> Option<Vec<bool>> {
        let matching = match &self.kind {
            StrokeKind::BackgroundErase(m) => *m,
            StrokeKind::ReplaceColor { matching, .. } => *matching,
            _ => return None,
        };
        let (bw, bh) = (x1 - x0, y1 - y0);
        let center = (
            (cx.max(0.0) as u32).min(self.base.width().saturating_sub(1)),
            (cy.max(0.0) as u32).min(self.base.height().saturating_sub(1)),
        );
        let at = |x: u32, y: u32| {
            let p = self.base.pixel(x, y);
            [p[0], p[1], p[2]]
        };
        let sample = match matching.sampling {
            Sampling::Continuous => at(center.0, center.1),
            Sampling::Once => *self.sampled.get_or_insert(at(center.0, center.1)),
            Sampling::Swatch(c) => c,
        };
        let close = |p: [u8; 3], c: [u8; 3]| {
            let d = (0..3)
                .map(|i| (p[i] as i32 - c[i] as i32).unsigned_abs())
                .max()
                .unwrap_or(0);
            d as f32 / 255.0 <= matching.tolerance
        };
        let ok = |x: u32, y: u32| {
            let p = at(x, y);
            self.base.pixel(x, y)[3] > 0
                && close(p, sample)
                && matching.protect.is_none_or(|f| !close(p, f))
        };
        let mut mask = vec![false; (bw * bh) as usize];
        if !matching.contiguous {
            for y in y0..y1 {
                for x in x0..x1 {
                    mask[((y - y0) * bw + (x - x0)) as usize] = ok(x, y);
                }
            }
            return Some(mask);
        }
        // Contiguous: flood from the center through matching pixels
        if !(x0..x1).contains(&center.0) || !(y0..y1).contains(&center.1) || !ok(center.0, center.1)
        {
            return Some(mask);
        }
        let mut stack = vec![center];
        mask[((center.1 - y0) * bw + (center.0 - x0)) as usize] = true;
        while let Some((x, y)) = stack.pop() {
            let neighbors = [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ];
            for (nx, ny) in neighbors {
                if nx < x0 || ny < y0 || nx >= x1 || ny >= y1 {
                    continue;
                }
                let i = ((ny - y0) * bw + (nx - x0)) as usize;
                if !mask[i] && ok(nx, ny) {
                    mask[i] = true;
                    stack.push((nx, ny));
                }
            }
        }
        Some(mask)
    }

    /// One healing dab: the pixels under the tip are healed (`heal`) with
    /// the stroke's source, matched to their surroundings, by the tip's
    /// coverage × opacity × selection.
    fn heal_dab(&mut self, doc: &mut Document, cx: f32, cy: f32) {
        let (w, h) = (doc.width, doc.height);
        let r = self.tip.diameter / 2.0 + 1.0;
        let x0 = ((cx - r).floor() - 1.0).max(0.0) as u32;
        let y0 = ((cy - r).floor() - 1.0).max(0.0) as u32;
        let x1 = (((cx + r).ceil() + 1.0).max(0.0) as u32).min(w);
        let y1 = (((cy + r).ceil() + 1.0).max(0.0) as u32).min(h);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        let (bw, bh) = ((x1 - x0) as usize, (y1 - y0) as usize);
        let tip = self.tip;
        let cover: Vec<f32> = (0..bw * bh)
            .map(|k| {
                let (x, y) = (x0 + (k % bw) as u32, y0 + (k / bw) as u32);
                tip.coverage(x as f32 + 0.5 - cx, y as f32 + 0.5 - cy)
            })
            .collect();
        let inside: Vec<bool> = cover.iter().map(|&a| a > 0.0).collect();
        let Some(current) = doc.layer(self.layer).and_then(|l| l.image()) else {
            return;
        };
        let texture;
        let (source, offset) = match &self.kind {
            StrokeKind::Heal { source, dx, dy } => (source, (*dx, *dy)),
            StrokeKind::SpotHeal(source) if self.create_texture => {
                texture = crate::heal::mirrored_texture(
                    source,
                    (x0, y0, bw, bh),
                    (cx, cy),
                    tip.diameter / 2.0,
                );
                (&texture, (x0 as i64, y0 as i64))
            }
            StrokeKind::SpotHeal(source) => {
                let Some(o) = crate::heal::proximity_match(source, (cx, cy), tip.diameter / 2.0)
                else {
                    return;
                };
                (source, o)
            }
            _ => return,
        };
        let style = self.heal_style;
        let Some(healed) = crate::heal::heal_window_with(
            current,
            source,
            (x0, y0, bw, bh),
            &inside,
            offset,
            style,
        ) else {
            return;
        };
        let opacity = self.opacity;
        let selection = self.selection.clone();
        let preserve = self.preserve_alpha;
        let Some(image) = doc.layer_mut(self.layer).and_then(|l| l.image_mut()) else {
            return;
        };
        for k in 0..bw * bh {
            if !inside[k] {
                continue;
            }
            let (x, y) = (x0 + (k % bw) as u32, y0 + (k / bw) as u32);
            let selected = selection
                .as_ref()
                .map_or(1.0, |s| s.get(x, y) as f32 / 255.0);
            let mut amount = cover[k] * opacity * selected;
            // Replace keeps the grain: a pixel takes the healed color or
            // keeps its own
            if style.mode == crate::heal::HealMode::Replace {
                amount = if crate::heal::replace_takes(x, y, amount) {
                    1.0
                } else {
                    0.0
                };
            }
            let p = image.pixel(x, y);
            let [r, g, b] = healed[k].map(|v| v.round() as u8);
            image.set_pixel(x, y, mix(p, [r, g, b, p[3]], amount, preserve));
        }
        doc.mark_dirty();
    }

    /// One Smudge dab: the pixels under the tip take on the colors under
    /// the previous dab, by `strength` × coverage (the first dab only
    /// picks up).
    fn smudge_dab(&mut self, doc: &mut Document, cx: f32, cy: f32, strength: f32) {
        let Some((px, py)) = self.last_dab.replace((cx, cy)) else {
            return;
        };
        let (dx, dy) = ((cx - px).round() as i64, (cy - py).round() as i64);
        if dx == 0 && dy == 0 {
            return;
        }
        let (w, h) = (doc.width, doc.height);
        let r = self.tip.diameter / 2.0 + 1.0;
        let x0 = (cx - r).floor().max(0.0) as u32;
        let y0 = (cy - r).floor().max(0.0) as u32;
        let x1 = ((cx + r).ceil().max(0.0) as u32).min(w);
        let y1 = ((cy + r).ceil().max(0.0) as u32).min(h);
        let selection = self.selection.clone();
        let preserve = self.preserve_alpha;
        let tip = self.tip;
        let mut sample = self.sample.as_mut();
        let Some(image) = doc.layer_mut(self.layer).and_then(|l| l.image_mut()) else {
            return;
        };
        // Read the dragged colors before writing any (from the merged
        // image with Sample All Layers)
        let mut updates = Vec::new();
        for y in y0..y1 {
            for x in x0..x1 {
                let a = tip.coverage(x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                if a <= 0.0 {
                    continue;
                }
                let (sx, sy) = (x as i64 - dx, y as i64 - dy);
                if sx < 0 || sy < 0 || sx >= w as i64 || sy >= h as i64 {
                    continue;
                }
                let selected = selection
                    .as_ref()
                    .map_or(1.0, |s| s.get(x, y) as f32 / 255.0);
                let amount = a * strength * selected;
                let from = match &sample {
                    Some(m) => m.pixel(sx as u32, sy as u32),
                    None => image.pixel(sx as u32, sy as u32),
                };
                let merged = sample
                    .as_ref()
                    .map(|m| mix(m.pixel(x, y), from, amount, false));
                updates.push((x, y, mix(image.pixel(x, y), from, amount, preserve), merged));
            }
        }
        for (x, y, p, merged) in updates {
            image.set_pixel(x, y, p);
            if let (Some(m), Some(q)) = (sample.as_deref_mut(), merged) {
                m.set_pixel(x, y, q);
            }
        }
        doc.mark_dirty();
    }

    /// An Art History Brush dab: strokes scattered over the area around
    /// (`cx`, `cy`), each starting at a random point in the source's color
    /// there and running along the source's edges (across its luminosity
    /// gradient), shaped by the style.
    fn art_dab(&mut self, doc: &mut Document, cx: f32, cy: f32) {
        let StrokeKind::ArtHistory {
            source,
            style,
            area,
            tolerance,
        } = &self.kind
        else {
            return;
        };
        let (style, area, tolerance) = (*style, area.max(1.0), *tolerance);
        let source = source.clone();
        let (w, h) = (doc.width, doc.height);
        let tip = self.tip;
        let d = tip.diameter.max(1.0);
        let opacity = self.opacity;
        let preserve = self.preserve_alpha;
        let selection = self.selection.clone();
        // A repeatable sequence per dab
        let mut seed = (cx.to_bits() as u64) << 32 ^ cy.to_bits() as u64 ^ 0x9E37_79B9_7F4A_7C15;
        let mut rand = move || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            (seed >> 11) as f32 / (1u64 << 53) as f32
        };
        let Some(image) = doc.layer_mut(self.layer).and_then(|l| l.image_mut()) else {
            return;
        };
        let lum = |p: [u8; 4]| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32;
        let at = |x: f32, y: f32| {
            source.pixel(
                (x.max(0.0) as u32).min(w.saturating_sub(1)),
                (y.max(0.0) as u32).min(h.saturating_sub(1)),
            )
        };
        let count = ((area * area) / (d * d) * 0.3).clamp(1.0, 40.0) as usize;
        let (length, loose, curl) = style.shape();
        for _ in 0..count {
            // A start in the area (uniform over the disc)
            let (r, t) = (rand().sqrt() * area, rand() * std::f32::consts::TAU);
            let (sx, sy) = (cx + r * t.cos(), cy + r * t.sin());
            if sx < 0.0 || sy < 0.0 || sx >= w as f32 || sy >= h as f32 {
                continue;
            }
            let color = at(sx, sy);
            // Tolerance keeps the strokes off what already matches the source
            let now = image.pixel(sx as u32, sy as u32);
            let differ = (0..3)
                .map(|c| (color[c] as f32 - now[c] as f32).abs() / 255.0)
                .fold(0.0, f32::max);
            if tolerance > 0.0 && differ < tolerance {
                continue;
            }
            // Along the edges: across the luminosity gradient
            let gx = lum(at(sx + 1.0, sy)) - lum(at(sx - 1.0, sy));
            let gy = lum(at(sx, sy + 1.0)) - lum(at(sx, sy - 1.0));
            let mut dir = if gx.abs() + gy.abs() > 1.0 {
                gx.atan2(-gy)
            } else {
                rand() * std::f32::consts::TAU
            };
            dir += (rand() - 0.5) * loose * std::f32::consts::PI;
            let turn = if curl {
                (rand() - 0.5).signum() * 0.35
            } else {
                0.0
            };
            let steps = ((length * 4.0) as usize).max(1);
            let (mut x, mut y) = (sx, sy);
            let rgb = [color[0], color[1], color[2]];
            for _ in 0..steps {
                let r = d / 2.0 + 1.0;
                let x0 = (x - r).floor().max(0.0) as u32;
                let y0 = (y - r).floor().max(0.0) as u32;
                let x1 = ((x + r).ceil().max(0.0) as u32).min(w);
                let y1 = ((y + r).ceil().max(0.0) as u32).min(h);
                for py in y0..y1 {
                    for px in x0..x1 {
                        let a = tip.coverage(px as f32 + 0.5 - x, py as f32 + 0.5 - y);
                        if a <= 0.0 {
                            continue;
                        }
                        let selected = selection
                            .as_ref()
                            .map_or(1.0, |s| s.get(px, py) as f32 / 255.0);
                        let amount = a * opacity * selected;
                        let p = apply(
                            image.pixel(px, py),
                            amount,
                            &StrokeKind::Paint(rgb),
                            preserve,
                        );
                        image.set_pixel(px, py, p);
                    }
                }
                x += dir.cos() * d * 0.25;
                y += dir.sin() * d * 0.25;
                dir += turn;
            }
        }
        doc.mark_dirty();
    }

    /// A Mixer Brush dab: the brush picks up the paint under it by Wet,
    /// lays down its own mixed with the canvas's by Mix, at a strength that
    /// falls as the load runs out (wet paint keeps smearing).
    fn mixer_dab(
        &mut self,
        doc: &mut Document,
        cx: f32,
        cy: f32,
        (loaded, wet, load, mix): (Option<[u8; 3]>, f32, f32, f32),
    ) {
        if self.last_dab.is_none() {
            // The stroke's first dab loads the brush
            self.mixer.0 = loaded.map(|c| c.map(|v| v as f32 / 255.0)).or(self.mixer.0);
            self.mixer.1 = if loaded.is_some() { 1.0 } else { 0.0 };
        }
        self.last_dab = Some((cx, cy));
        let (w, h) = (doc.width, doc.height);
        let tip = self.tip;
        let r = tip.diameter / 2.0 + 1.0;
        let x0 = (cx - r).floor().max(0.0) as u32;
        let y0 = (cy - r).floor().max(0.0) as u32;
        let x1 = ((cx + r).ceil().max(0.0) as u32).min(w);
        let y1 = ((cy + r).ceil().max(0.0) as u32).min(h);
        let selection = self.selection.clone();
        let preserve = self.preserve_alpha;
        let flow = self.flow;
        let sample = self.sample.as_mut();
        let Some(image) = doc.layer_mut(self.layer).and_then(|l| l.image_mut()) else {
            return;
        };
        // The canvas's paint under the tip (the merged image's with Sample
        // All Layers)
        let (mut sum, mut weight) = ([0f32; 3], 0f32);
        for y in y0..y1 {
            for x in x0..x1 {
                let a = tip.coverage(x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                let p = match &sample {
                    Some(m) => m.pixel(x, y),
                    None => image.pixel(x, y),
                };
                let k = a * p[3] as f32 / 255.0;
                for c in 0..3 {
                    sum[c] += p[c] as f32 / 255.0 * k;
                }
                weight += k;
            }
        }
        let canvas = (weight > 1e-3).then(|| sum.map(|v| v / weight));
        // Pick up: the brush's paint takes on the canvas's, a tenth of Wet ×
        // Mix each dab
        let mut color = self.mixer.0;
        if let Some(c) = canvas
            && wet > 0.0
        {
            color = Some(match color {
                Some(b) => [0, 1, 2].map(|i| b[i] + (c[i] - b[i]) * wet * mix * 0.1),
                None => c,
            });
        }
        self.mixer.0 = color;
        let Some(brush) = color else {
            return;
        };
        let laid = match canvas {
            // (half of Mix: the brush's paint, which carries what it picked
            // up, always shows)
            Some(c) if wet > 0.0 => [0, 1, 2].map(|i| brush[i] + (c[i] - brush[i]) * mix * 0.5),
            _ => brush,
        };
        // Strength: the load left, and wet paint smearing even when dry
        let paint = self.mixer.1;
        let strength = flow * (paint + (1.0 - paint) * wet * 0.5).min(1.0);
        self.mixer.1 = (paint - 1.0 / (4.0 + load * 200.0)).max(0.0);
        let rgb = laid.map(|v| (v * 255.0).round().clamp(0.0, 255.0) as u8);
        let mut updates = Vec::new();
        for y in y0..y1 {
            for x in x0..x1 {
                let a = tip.coverage(x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                if a <= 0.0 {
                    continue;
                }
                let selected = selection
                    .as_ref()
                    .map_or(1.0, |s| s.get(x, y) as f32 / 255.0);
                let amount = a * strength * selected;
                updates.push((
                    x,
                    y,
                    apply(image.pixel(x, y), amount, &StrokeKind::Paint(rgb), preserve),
                ));
            }
        }
        for (x, y, p) in updates {
            image.set_pixel(x, y, p);
        }
        // The merged image takes the same paint, so later dabs pick it up
        if let Some(m) = sample {
            for y in y0..y1 {
                for x in x0..x1 {
                    let a = tip.coverage(x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                    if a <= 0.0 {
                        continue;
                    }
                    let selected = selection
                        .as_ref()
                        .map_or(1.0, |s| s.get(x, y) as f32 / 255.0);
                    let q = apply(
                        m.pixel(x, y),
                        a * strength * selected,
                        &StrokeKind::Paint(rgb),
                        false,
                    );
                    m.set_pixel(x, y, q);
                }
            }
        }
        doc.mark_dirty();
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

/// What a retouching tool turns pixel `p` into at full strength; `near`
/// gives the pixels around it (offsets −1–1) for Blur and Sharpen.
fn retouch_target(
    kind: &StrokeKind,
    options: Retouch,
    p: [u8; 4],
    near: impl Fn(i64, i64) -> [u8; 4],
) -> [u8; 4] {
    let rgb = [p[0], p[1], p[2]].map(|v| v as f32 / 255.0);
    let out = |c: [f32; 3]| {
        let [r, g, b] = c.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
        [r, g, b, p[3]]
    };
    let lum = |c: [f32; 3]| 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
    // Protect Tones: the luminosity changes and the color is scaled with
    // it, keeping its hue, and stays clear of clipping
    let relit = |to: f32| {
        let from = lum(rgb);
        let to = to.clamp(0.0, 1.0);
        let scaled = if from > 1e-4 {
            rgb.map(|c| c * to / from)
        } else {
            [to; 3]
        };
        let over = scaled.iter().fold(0f32, |m, &c| m.max(c));
        let scaled = if over > 1.0 {
            // Pull toward the gray of the same luminosity until it fits
            let t = (over - 1.0) / (over - to).max(1e-4);
            scaled.map(|c| c + (to - c) * t)
        } else {
            scaled
        };
        out(scaled)
    };
    match kind {
        StrokeKind::Dodge(range) if options.protect_tones => {
            let l = lum(rgb);
            relit(l + range.weight(l) * (1.0 - l))
        }
        StrokeKind::Burn(range) if options.protect_tones => {
            let l = lum(rgb);
            relit(l - range.weight(l) * l)
        }
        StrokeKind::Dodge(range) => out(rgb.map(|v| v + range.weight(v) * (1.0 - v))),
        StrokeKind::Burn(range) => out(rgb.map(|v| v - range.weight(v) * v)),
        StrokeKind::Sponge { saturate } => {
            let [h, s, l] = crate::adjust::rgb_to_hsl(rgb);
            // Vibrance: less on the colors near full (or no) saturation
            let s = match (*saturate, options.vibrance) {
                (true, false) => (s * 2.0).min(1.0),
                (true, true) => s + s * (1.0 - s),
                (false, false) => 0.0,
                (false, true) => s * s,
            };
            out(crate::adjust::hsl_to_rgb([h, s, l]))
        }
        StrokeKind::Blur | StrokeKind::Sharpen => {
            // 3×3 average, premultiplied
            let mut sum = [0f32; 4];
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let q = near(dx, dy);
                    let a = q[3] as f32 / 255.0;
                    for c in 0..3 {
                        sum[c] += q[c] as f32 / 255.0 * a;
                    }
                    sum[3] += a;
                }
            }
            if sum[3] <= 0.0 {
                return p;
            }
            let blurred = [sum[0] / sum[3], sum[1] / sum[3], sum[2] / sum[3]];
            if matches!(kind, StrokeKind::Blur) {
                out(blurred)
            } else if options.protect_detail {
                // Protect Detail: half the boost, and none below a level of
                // difference (noise)
                out([0, 1, 2].map(|c| {
                    let d = rgb[c] - blurred[c];
                    if d.abs() < 1.0 / 255.0 {
                        rgb[c]
                    } else {
                        rgb[c] + d * 0.5
                    }
                }))
            } else {
                out([0, 1, 2].map(|c| rgb[c] + (rgb[c] - blurred[c])))
            }
        }
        _ => p,
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
        angle: 0.0,
        roundness: 1.0,
        spacing: 0.25,
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
            angle: 0.0,
            roundness: 1.0,
            spacing: 0.25,
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
    fn unaligned_pattern_starts_at_the_stroke() {
        let mut pattern = TiledImage::new(3, 1);
        pattern.set_pixel(0, 0, [255, 0, 0, 255]);
        pattern.set_pixel(1, 0, [0, 255, 0, 255]);
        pattern.set_pixel(2, 0, [0, 0, 255, 255]);
        let mut doc = Document::new_with_background("t", 40, 40, Color::WHITE);
        let id = doc.layers[0].id;
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Pattern(pattern), 1.0, 1.0)
            .unwrap()
            .with_pattern_origin((20, 20));
        s.add_point(&mut doc, 20.0, 20.0);
        let image = doc.layer(id).unwrap().image().unwrap();
        // The pattern's first column lands on the stroke's first point
        assert_eq!(image.pixel(20, 20), [255, 0, 0, 255]);
        assert_eq!(image.pixel(21, 20), [0, 255, 0, 255]);
        assert_eq!(image.pixel(19, 20), [0, 0, 255, 255]);
    }

    #[test]
    fn sample_all_layers_reads_the_merged_image() {
        // Black and white stripes on the background, an empty layer on top
        let mut doc = Document::new_with_background("t", 40, 40, Color::WHITE);
        let bg = doc.layers[0].id;
        let image = doc.layer_mut(bg).unwrap().image_mut().unwrap();
        for y in 0..40 {
            for x in (0..40).step_by(2) {
                image.set_pixel(x, y, [0, 0, 0, 255]);
            }
        }
        let id = doc.new_layer_id();
        doc.layers
            .push(crate::Layer::raster(id, "top", TiledImage::new(40, 40)));
        doc.active_layer = Some(id);
        let merged = doc.sample_source(crate::SampleScope::All).unwrap();
        // Without it Blur has nothing to blur; with it the blurred stripes
        // land on the empty layer
        one_dab(&mut doc, StrokeKind::Blur, 1.0);
        assert_eq!(doc.layer(id).unwrap().image().unwrap().pixel(20, 20)[3], 0);
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Blur, 1.0, 1.0)
            .unwrap()
            .with_sample(merged);
        s.add_point(&mut doc, 20.0, 20.0);
        let p = doc.layer(id).unwrap().image().unwrap().pixel(20, 20);
        assert!(p[3] > 200 && (40..215).contains(&p[0]), "{p:?}");
        assert_eq!(doc.layer(id).unwrap().image().unwrap().pixel(2, 2)[3], 0);
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
    fn retouching_builds_up_and_its_options() {
        // Scrubbing back and forth keeps darkening
        let (mut doc, id) = doc_filled([128, 128, 128, 255]);
        let mut s =
            Stroke::begin(&doc, HARD, StrokeKind::Burn(ToneRange::Midtones), 0.2, 1.0).unwrap();
        s.add_point(&mut doc, 5.0, 20.0);
        s.add_point(&mut doc, 35.0, 20.0);
        let once = pixel(&doc, id, 20, 20)[0];
        s.add_point(&mut doc, 5.0, 20.0);
        let twice = pixel(&doc, id, 20, 20)[0];
        assert!(twice < once && once < 128, "{once} {twice}");
        // The airbrush builds up in place
        let held = pixel(&doc, id, 5, 20)[0];
        s.build_up(&mut doc);
        assert!(pixel(&doc, id, 5, 20)[0] < held);
        // Protect Tones keeps a color's hue as it lightens
        let protect = Retouch {
            protect_tones: true,
            ..Default::default()
        };
        let dodge = StrokeKind::Dodge(ToneRange::Highlights);
        let none = |_: i64, _: i64| [0u8; 4];
        let p = retouch_target(&dodge, protect, [80, 40, 20, 255], none);
        let plain = retouch_target(&dodge, Retouch::default(), [80, 40, 20, 255], none);
        let ratio = |q: [u8; 4]| q[0] as f32 / q[1] as f32;
        assert!(p[0] > 80 && (ratio(p) - 2.0).abs() < 0.06, "{p:?}");
        assert!((ratio(plain) - 2.0).abs() > 0.08, "{plain:?}");
        // Vibrance saturates a nearly saturated color less
        let vib = Retouch {
            vibrance: true,
            ..Default::default()
        };
        let sponge = StrokeKind::Sponge { saturate: true };
        let a = retouch_target(&sponge, vib, [200, 60, 60, 255], |_, _| [0; 4]);
        let b = retouch_target(&sponge, Retouch::default(), [200, 60, 60, 255], |_, _| {
            [0; 4]
        });
        assert!(a[1] > b[1], "{a:?} {b:?}");
        // Protect Detail sharpens half as hard
        let edge = |dx: i64, _: i64| {
            if dx < 0 {
                [0, 0, 0, 255]
            } else {
                [200, 200, 200, 255]
            }
        };
        let detail = Retouch {
            protect_detail: true,
            ..Default::default()
        };
        let soft = retouch_target(&StrokeKind::Sharpen, detail, [200, 200, 200, 255], edge);
        let hard = retouch_target(
            &StrokeKind::Sharpen,
            Retouch::default(),
            [200, 200, 200, 255],
            edge,
        );
        assert!(soft[0] > 200 && soft[0] < hard[0], "{soft:?} {hard:?}");
    }

    #[test]
    fn impressionist_pattern_daubs() {
        // A pattern of vertical stripes, one pixel each
        let mut pattern = TiledImage::new(2, 1);
        pattern.set_pixel(0, 0, [255, 0, 0, 255]);
        pattern.set_pixel(1, 0, [0, 0, 255, 255]);
        let (mut doc, id) = doc_with_layer();
        let mut s =
            Stroke::begin(&doc, HARD, StrokeKind::Pattern(pattern.clone()), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 20.0, 20.0);
        // Aligned: the stripes come through
        assert_ne!(pixel(&doc, id, 19, 20), pixel(&doc, id, 20, 20));
        // Impressionist: blocks of one color
        let (mut doc, id) = doc_with_layer();
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Pattern(pattern), 1.0, 1.0)
            .unwrap()
            .with_impressionist(true);
        s.add_point(&mut doc, 20.0, 20.0);
        assert_eq!(pixel(&doc, id, 18, 19), pixel(&doc, id, 19, 19));
    }

    #[test]
    fn mixer_brush_loads_picks_up_and_runs_dry() {
        let blue = |wet: f32, load: f32| StrokeKind::Mix {
            color: Some([0, 0, 255]),
            wet,
            load,
            mix: 0.5,
        };
        // Dry, light load: blue at first, fading as the paint runs out
        let (mut doc, id) = doc_filled([255, 255, 255, 255]);
        let mut s = Stroke::begin(&doc, HARD, blue(0.0, 0.0), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 5.0, 20.0);
        s.add_point(&mut doc, 35.0, 20.0);
        let (start, end) = (pixel(&doc, id, 5, 20), pixel(&doc, id, 34, 20));
        assert_eq!(start, [0, 0, 255, 255]);
        assert!(end[0] > 100, "{end:?}");
        // Wet on red: the blue mixes with the red it picks up
        let (mut doc, id) = doc_filled([255, 0, 0, 255]);
        let mut s = Stroke::begin(&doc, HARD, blue(1.0, 1.0), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 5.0, 20.0);
        s.add_point(&mut doc, 35.0, 20.0);
        let p = pixel(&doc, id, 20, 20);
        assert!(p[0] > 60 && p[2] > 60, "{p:?}");
        let kept = s.mixer_color().unwrap();
        assert!(kept[0] > 0 && kept[2] < 255, "{kept:?}");
        // A clean, wet brush smears what it picks up
        let (mut doc, id) = doc_filled([255, 255, 255, 255]);
        {
            let img = doc.layer_mut(id).unwrap().image_mut().unwrap();
            for y in 0..40 {
                for x in 0..12 {
                    img.set_pixel(x, y, [0, 0, 0, 255]);
                }
            }
        }
        let clean = StrokeKind::Mix {
            color: None,
            wet: 1.0,
            load: 0.5,
            mix: 1.0,
        };
        let mut s = Stroke::begin(&doc, HARD, clean, 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 8.0, 20.0);
        s.add_point(&mut doc, 24.0, 20.0);
        assert!(pixel(&doc, id, 16, 20)[0] < 200);
    }

    #[test]
    fn art_history_paints_the_source_in_strokes() {
        // Source: red; the layer now white
        let mut source = TiledImage::new(40, 40);
        for y in 0..40 {
            for x in 0..40 {
                source.set_pixel(x, y, [255, 0, 0, 255]);
            }
        }
        let small = BrushTip {
            diameter: 3.0,
            ..HARD
        };
        let art = |tolerance: f32| StrokeKind::ArtHistory {
            source: source.clone(),
            style: ArtStyle::TightMedium,
            area: 12.0,
            tolerance,
        };
        let (mut doc, id) = doc_filled([255, 255, 255, 255]);
        let mut s = Stroke::begin(&doc, small, art(0.0), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 20.0, 20.0);
        let red = (10..30)
            .flat_map(|y| (10..30).map(move |x| (x, y)))
            .filter(|&(x, y)| pixel(&doc, id, x, y) == [255, 0, 0, 255])
            .count();
        assert!(red > 20, "{red}");
        // Far from the area nothing changes
        assert_eq!(pixel(&doc, id, 2, 2), [255, 255, 255, 255]);
        // Already red: high tolerance keeps the strokes off
        let (mut doc, id) = doc_filled([255, 0, 0, 255]);
        let mut s = Stroke::begin(&doc, small, art(0.5), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 20.0, 20.0);
        assert_eq!(pixel(&doc, id, 20, 20), [255, 0, 0, 255]);
        assert_eq!(ArtStyle::ALL.len(), 10);
    }

    #[test]
    fn cloning_scaled_and_turned() {
        // A source with a red column at x = 5 (rows 0–40)
        let (mut doc, id) = doc_filled([100, 100, 100, 255]);
        {
            let img = doc.layer_mut(id).unwrap().image_mut().unwrap();
            for y in 0..40 {
                img.set_pixel(5, y, [255, 0, 0, 255]);
            }
        }
        let image = doc.layer(id).unwrap().image().unwrap().clone();
        let big = BrushTip {
            diameter: 30.0,
            ..HARD
        };
        // Source point (5.5, 20.5) lands at (20.5, 20.5), scaled 200% wide
        let source = StrokeKind::Source {
            image: image.clone(),
            dx: 15,
            dy: 0,
        };
        let t = SourceTransform {
            origin: (5.5, 20.5),
            scale: (2.0, 1.0),
            angle: 0.0,
        };
        let mut s = Stroke::begin(&doc, big, source, 1.0, 1.0)
            .unwrap()
            .with_source_transform(t);
        s.add_point(&mut doc, 20.5, 20.5);
        // The one-pixel column is now two wide (about the source point's
        // middle)
        assert_eq!(pixel(&doc, id, 19, 20), [255, 0, 0, 255]);
        assert_eq!(pixel(&doc, id, 20, 20), [255, 0, 0, 255]);
        assert_eq!(pixel(&doc, id, 22, 20), [100, 100, 100, 255]);
        // Turned 90°: the column becomes a row
        let (mut doc, id) = doc_filled([100, 100, 100, 255]);
        let source = StrokeKind::Source {
            image,
            dx: 15,
            dy: 0,
        };
        let t = SourceTransform {
            origin: (5.5, 20.5),
            scale: (1.0, 1.0),
            angle: 90.0,
        };
        let mut s = Stroke::begin(&doc, big, source, 1.0, 1.0)
            .unwrap()
            .with_source_transform(t);
        s.add_point(&mut doc, 20.5, 20.5);
        assert_eq!(pixel(&doc, id, 15, 20), [255, 0, 0, 255]);
        assert_eq!(pixel(&doc, id, 20, 15), [100, 100, 100, 255]);
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
    fn elliptical_tips_and_spacing() {
        // A flat tip (roundness 30%) at 0°: wide, short
        let tip = BrushTip {
            diameter: 20.0,
            roundness: 0.3,
            ..HARD
        };
        assert!(tip.coverage(9.0, 0.0) > 0.9);
        assert!(tip.coverage(0.0, 5.0) < 0.1);
        // Turned 90°: tall, narrow
        let tall = BrushTip { angle: 90.0, ..tip };
        assert!(tall.coverage(0.0, 9.0) > 0.9);
        assert!(tall.coverage(5.0, 0.0) < 0.1);
        // Spacing 100%: dabs a diameter apart along a line
        let (mut doc, id) = doc_with_layer();
        let tip = BrushTip {
            diameter: 4.0,
            spacing: 1.0,
            ..HARD
        };
        let mut s = Stroke::begin(&doc, tip, StrokeKind::Paint([0, 0, 0]), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 4.0, 20.0);
        s.add_point(&mut doc, 36.0, 20.0);
        // (dabs at 4, 8, 12...: a gap between 4 and 8 halfway is covered by
        // neither centre's core but the edges touch)
        assert_eq!(pixel(&doc, id, 8, 20)[3], 255);
        assert_eq!(pixel(&doc, id, 40, 20)[3], 0);
    }

    #[test]
    fn pressure_controls_size_and_opacity() {
        // Size: light pressure paints a thin line
        let (mut doc, id) = doc_with_layer();
        let tip = BrushTip {
            diameter: 20.0,
            ..HARD
        };
        let size = Pressure {
            size: true,
            ..Default::default()
        };
        let mut s = Stroke::begin(&doc, tip, StrokeKind::Paint([0, 0, 0]), 1.0, 1.0)
            .unwrap()
            .with_pressure(size);
        s.add_point_with_pressure(&mut doc, 10.0, 20.0, 0.2);
        s.add_point_with_pressure(&mut doc, 40.0, 20.0, 0.2);
        assert_eq!(pixel(&doc, id, 25, 20)[3], 255);
        assert_eq!(pixel(&doc, id, 25, 26)[3], 0);
        // Opacity: half pressure reaches half coverage, however often
        let (mut doc, id) = doc_with_layer();
        let opacity = Pressure {
            opacity: true,
            ..Default::default()
        };
        let mut s = Stroke::begin(&doc, tip, StrokeKind::Paint([0, 0, 0]), 1.0, 1.0)
            .unwrap()
            .with_pressure(opacity);
        for _ in 0..4 {
            s.add_point_with_pressure(&mut doc, 10.0, 20.0, 0.5);
            s.add_point_with_pressure(&mut doc, 40.0, 20.0, 0.5);
        }
        let a = pixel(&doc, id, 25, 20)[3];
        assert!(a.abs_diff(128) <= 2, "{a}");
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

    #[test]
    fn pattern_smudge_background_eraser_and_color_replacement() {
        use crate::BlendMode;
        // Pattern: a 2 × 2 checker tiled from the origin
        let (mut doc, id) = doc_with_layer();
        let mut pat = TiledImage::new(2, 2);
        pat.set_pixel(0, 0, [255, 0, 0, 255]);
        pat.set_pixel(1, 0, [0, 0, 255, 255]);
        pat.set_pixel(0, 1, [0, 0, 255, 255]);
        pat.set_pixel(1, 1, [255, 0, 0, 255]);
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Pattern(pat), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 20.0, 20.0);
        assert_eq!(pixel(&doc, id, 20, 20), [255, 0, 0, 255]);
        assert_eq!(pixel(&doc, id, 21, 20), [0, 0, 255, 255]);

        // Smudge drags red rightward into the empty layer
        let (mut doc, id) = doc_with_layer();
        for y in 0..40 {
            for x in 0..15 {
                doc.layer_mut(id)
                    .unwrap()
                    .image_mut()
                    .unwrap()
                    .set_pixel(x, y, [255, 0, 0, 255]);
            }
        }
        let mut s = Stroke::begin(&doc, HARD, StrokeKind::Smudge(1.0), 1.0, 1.0).unwrap();
        for x in [10.0, 13.0, 16.0, 19.0] {
            s.add_point(&mut doc, x, 20.0);
        }
        assert!(
            pixel(&doc, id, 17, 20)[3] > 0,
            "{:?}",
            pixel(&doc, id, 17, 20)
        );

        // Background Eraser: removes the gray it samples, keeps the black
        let (mut doc, id) = doc_with_layer();
        for y in 0..40 {
            for x in 0..40 {
                let c = if x < 20 {
                    [128, 128, 128, 255]
                } else {
                    [0, 0, 0, 255]
                };
                doc.layer_mut(id)
                    .unwrap()
                    .image_mut()
                    .unwrap()
                    .set_pixel(x, y, c);
            }
        }
        let matching = ColorMatch {
            sampling: Sampling::Continuous,
            tolerance: 0.1,
            contiguous: false,
            protect: None,
        };
        let mut s =
            Stroke::begin(&doc, HARD, StrokeKind::BackgroundErase(matching), 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 18.0, 20.0);
        assert_eq!(pixel(&doc, id, 17, 20)[3], 0);
        assert_eq!(pixel(&doc, id, 21, 20), [0, 0, 0, 255]);

        // Color Replacement (Color): the gray takes red's hue, the black stays
        let (mut doc, id) = doc_with_layer();
        for y in 0..40 {
            for x in 0..40 {
                let c = if x < 20 {
                    [128, 128, 128, 255]
                } else {
                    [0, 0, 0, 255]
                };
                doc.layer_mut(id)
                    .unwrap()
                    .image_mut()
                    .unwrap()
                    .set_pixel(x, y, c);
            }
        }
        let kind = StrokeKind::ReplaceColor {
            color: [255, 0, 0],
            mode: BlendMode::Color,
            matching,
        };
        let mut s = Stroke::begin(&doc, HARD, kind, 1.0, 1.0).unwrap();
        s.add_point(&mut doc, 18.0, 20.0);
        let p = pixel(&doc, id, 17, 20);
        assert!(p[0] > p[1] && p[0] > p[2], "{p:?}");
        assert_eq!(pixel(&doc, id, 21, 20), [0, 0, 0, 255]);
    }
}
