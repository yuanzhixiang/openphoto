use std::collections::HashMap;
use std::sync::Arc;

use op_color::Hsb;
use op_core::{Color, DocId, Document, History, LayerId};
use op_render::CanvasImage;
use op_tools::Tool;

/// How a document is shown in its viewport.
#[derive(Clone, Copy, Debug)]
pub struct View {
    /// Physical pixels per document pixel; 1.0 = 100%.
    pub zoom: f32,
    /// Offset of the document center from the viewport center, in points.
    /// Keeps the document centered when the window is resized.
    pub offset: egui::Vec2,
    /// The viewport size is unknown on first show; the initial zoom is picked once it is.
    pub initialized: bool,
    /// Viewport of the last frame, in points; used by keyboard zoom.
    pub viewport: egui::Rect,
    /// The Rotate View tool's angle: the canvas turned clockwise by this
    /// many degrees about the document's center (−180 to 180).
    pub rotation: f32,
    /// View › Flip Horizontal: the canvas shown mirrored left to right
    /// (the pixels stay as they are).
    pub flip: bool,
}

impl Default for View {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: egui::Vec2::ZERO,
            initialized: false,
            viewport: egui::Rect::NOTHING,
            rotation: 0.0,
            flip: false,
        }
    }
}

/// What the transform controls box depends on: the document revision, the
/// active layer and the selection revision.
type ControlsKey = (u64, Option<LayerId>, u64);

/// A box in document pixels: (x0, y0, x1, y1).
type Bounds = (f32, f32, f32, f32);

type MovableKey = (u64, u64, Vec<op_core::LayerId>);

pub struct DocState {
    pub doc: Document,
    /// The file the document was opened from or last saved to.
    pub path: Option<std::path::PathBuf>,
    /// History state that matches the file on disk (or the new document);
    /// any other current state means there are unsaved changes.
    saved_state: u64,
    /// The document has no embedded color profile (shown as "#" in its tab).
    /// Opened files are untagged because profiles aren't read; new documents
    /// are sRGB.
    pub untagged: bool,
    pub history: History,
    pub view: View,
    /// Set by continuous edits (e.g. dragging opacity) that haven't been
    /// recorded in the history yet.
    pending_edit: bool,
    /// A marquee being dragged on the canvas.
    pub marquee_drag: Option<MarqueeDrag>,
    /// A Move tool drag: the move and where it started (document pixels).
    pub move_drag: Option<(op_core::move_tool::Move, egui::Pos2)>,
    /// The paint stroke in progress, and the tool painting it.
    pub stroke: Option<(op_core::paint::Stroke, Tool)>,
    /// Where the last stroke ended; Shift-click draws a line from here.
    pub last_paint_point: Option<(f32, f32)>,
    /// A lasso outline being drawn.
    pub lasso: Option<LassoPath>,
    /// The Patch or Content-Aware Move tool dragging the selection: where
    /// the drag started and where it is (document pixels).
    pub patch_drag: Option<(egui::Pos2, egui::Pos2)>,
    /// The Magnetic Lasso under way (its outline is `lasso`).
    pub magnetic: Option<MagneticPath>,
    /// A Quick Selection stroke under way.
    pub quick: Option<QuickStroke>,
    /// The Object Selection tool's rectangle being dragged (document
    /// pixels).
    pub object_drag: Option<(egui::Pos2, egui::Pos2)>,
    /// What the drag in progress snaps to (View › Snap).
    pub snap: Option<crate::snap::Targets>,
    /// A Selection Brush stroke under way, and the overlay texture showing
    /// the selection with that tool (keyed by selection revision and color).
    pub selection_stroke: Option<op_core::selection_brush::SelectionStroke>,
    pub overlay: Option<((u64, [u8; 3]), egui::TextureHandle)>,
    /// The Color Sampler tool's points (document pixels, at most
    /// `MAX_SAMPLERS`), and the one being dragged.
    pub color_samplers: Vec<egui::Pos2>,
    pub sampler_drag: Option<usize>,
    /// The rulers' zero point, in document pixels (moved by dragging out of
    /// the rulers' corner).
    pub ruler_origin: egui::Pos2,
    /// The pointer while the origin is dragged out of the corner.
    pub origin_drag: Option<egui::Pos2>,
    /// A guide being dragged.
    pub guide_drag: Option<GuideDrag>,
    /// Where the pointer is over the document (document pixels), for the
    /// Info panel.
    pub pointer: Option<egui::Pos2>,
    /// The merged image's luminosity histogram and thumbnail texture, per
    /// revision (Histogram and Navigator panels).
    histogram: Option<(u64, [u64; 256])>,
    composite_tex: Option<(u64, egui::TextureHandle)>,
    /// The Clone Stamp's source point (set with Alt-click), and the offset
    /// from it to the strokes once painting has started (Aligned).
    pub clone_source: Option<egui::Pos2>,
    pub clone_offset: Option<egui::Vec2>,
    /// The button went down to pick the source; no painting until it's up.
    pub picking_clone_source: bool,
    /// The Crop tool's box, while the Crop tool is in use.
    pub crop: Option<CropBox>,
    /// Edit > Free Transform, while in progress.
    pub free_transform: Option<FreeTransform>,
    /// A Gradient tool drag: start and current point, in document pixels.
    pub gradient_drag: Option<(egui::Pos2, egui::Pos2)>,
    /// Text being typed, while the Type tool edits.
    pub text_edit: Option<TextEdit>,
    /// A shape tool drag: start and current point, in document pixels.
    pub shape_drag: Option<(egui::Pos2, egui::Pos2)>,
    /// A layer name being edited in the Layers panel, and the text so far.
    pub renaming: Option<(LayerId, String)>,
    /// The Move tool's transform controls box (document pixels), cached per
    /// (revision, active layer, selection revision); `None` inside when the
    /// layer can't be transformed.
    controls_bounds: Option<(ControlsKey, Option<Bounds>)>,
    /// How many selected layers Align/Distribute would move, for the
    /// buttons' and menu items' enabled state (asked many times a frame).
    movable: std::cell::RefCell<Option<(MovableKey, usize)>>,
    /// Marching-ants outline of the selection, cached per selection revision.
    outline: Option<(u64, Arc<Vec<[u32; 4]>>)>,
    canvas: Option<Arc<CanvasImage>>,
    /// Layer (and, with `true`, layer mask) thumbnails per revision.
    thumbs: HashMap<(LayerId, bool), (u64, egui::TextureHandle)>,
    /// Thumbnail of the document as opened, for the History panel's snapshot
    /// row. Kept as pixels until a texture can be created.
    snapshot_thumb: Option<egui::ColorImage>,
    snapshot_texture: Option<egui::TextureHandle>,
}

impl DocState {
    /// The box Show Transform Controls draws around the active layer's
    /// pixels (or the selection), as Free Transform would start with it.
    /// Whether a tool is in the middle of something Escape would cancel
    /// (so Escape is not taken to leave Full Screen Mode).
    pub fn busy(&self) -> bool {
        self.crop.is_some()
            || self.free_transform.is_some()
            || self.lasso.is_some()
            || self.magnetic.is_some()
            || self.text_edit.is_some()
            || self.marquee_drag.is_some()
            || self.shape_drag.is_some()
            || self.move_drag.is_some()
    }

    pub fn transform_controls_bounds(&mut self) -> Option<Bounds> {
        let key = (
            self.doc.revision(),
            self.doc.active_layer,
            self.doc.selection_revision(),
        );
        match self.controls_bounds {
            Some((k, b)) if k == key => b,
            _ => {
                let b = op_core::transform::bounds(&self.doc).ok();
                self.controls_bounds = Some((key, b));
                b
            }
        }
    }

    /// Layers Align and Distribute would move, cached per document
    /// revision, selection and selected layers.
    pub fn movable_layers(&self) -> usize {
        let key = (
            self.doc.revision(),
            self.doc.selection_revision(),
            self.doc.selected_layers(),
        );
        if let Some((k, n)) = &*self.movable.borrow()
            && *k == key
        {
            return *n;
        }
        let n = op_core::align::movable_count(&self.doc);
        *self.movable.borrow_mut() = Some((key, n));
        n
    }

    /// `initial` names the first history state, e.g. "Open" or "New".
    pub fn new(doc: Document, initial: &str) -> Self {
        let snapshot_thumb = Some(composite_thumbnail(&doc, SNAPSHOT_THUMB_PX));
        let history = History::new(&doc, initial);
        Self {
            untagged: initial == "Open",
            path: None,
            saved_state: history.current_id(),
            history,
            snapshot_thumb,
            snapshot_texture: None,
            doc,
            view: View::default(),
            pending_edit: false,
            marquee_drag: None,
            move_drag: None,
            controls_bounds: None,
            movable: Default::default(),
            stroke: None,
            last_paint_point: None,
            renaming: None,
            lasso: None,
            patch_drag: None,
            magnetic: None,
            quick: None,
            object_drag: None,
            snap: None,
            selection_stroke: None,
            overlay: None,
            color_samplers: Vec::new(),
            sampler_drag: None,
            ruler_origin: egui::Pos2::ZERO,
            origin_drag: None,
            free_transform: None,
            crop: None,
            guide_drag: None,
            pointer: None,
            histogram: None,
            composite_tex: None,
            clone_source: None,
            clone_offset: None,
            picking_clone_source: false,
            gradient_drag: None,
            shape_drag: None,
            text_edit: None,
            outline: None,
            canvas: None,
            thumbs: HashMap::new(),
        }
    }

    /// Whether the document differs from its file (or, for a new document,
    /// from how it was created). Undoing back to the saved state counts as
    /// unchanged, as in Photoshop.
    pub fn is_dirty(&self) -> bool {
        self.history.current_id() != self.saved_state
    }

    /// The document now matches its file.
    pub fn mark_saved(&mut self) {
        self.saved_state = self.history.current_id();
    }

    /// Records the current document as a new history state.
    pub fn record(&mut self, name: &str) {
        self.history.record(&self.doc, name);
        self.pending_edit = false;
    }

    /// Marks an in-progress edit that will be recorded by [`Self::commit_pending`].
    pub fn mark_pending(&mut self) {
        self.pending_edit = true;
    }

    /// Records a pending continuous edit, if there is one.
    pub fn commit_pending(&mut self, name: &str) {
        if self.pending_edit {
            self.record(name);
        }
    }

    pub fn undo(&mut self) -> bool {
        self.pending_edit = false;
        self.history.undo(&mut self.doc)
    }

    pub fn redo(&mut self) -> bool {
        self.pending_edit = false;
        self.history.redo(&mut self.doc)
    }

    pub fn toggle_last_state(&mut self) -> bool {
        self.pending_edit = false;
        self.history.toggle_last_state(&mut self.doc)
    }

    pub fn jump_to_state(&mut self, index: usize) -> bool {
        self.pending_edit = false;
        self.history.jump(index, &mut self.doc)
    }

    pub fn delete_states_from(&mut self, index: usize) -> bool {
        self.pending_edit = false;
        self.history.delete_from(index, &mut self.doc)
    }

    /// Outline of the current selection (see `Selection::outline`), cached.
    pub fn selection_outline(&mut self) -> Option<Arc<Vec<[u32; 4]>>> {
        let selection = self.doc.selection()?;
        let rev = self.doc.selection_revision();
        if let Some((r, segs)) = &self.outline
            && *r == rev
        {
            return Some(segs.clone());
        }
        let segs = Arc::new(selection.outline());
        self.outline = Some((rev, segs.clone()));
        Some(segs)
    }

    /// The current composite, recomputed when the document changes.
    pub fn canvas_image(&mut self) -> Arc<CanvasImage> {
        let rev = self.doc.revision();
        match &self.canvas {
            Some(img) if img.revision == rev => img.clone(),
            _ => {
                let mut pixels = self.doc.composite_rgba8();
                // Quick Mask: unselected areas tinted with 50% red, the
                // default "Masked Areas" display
                if let Some(q) = &self.doc.quick_mask {
                    let w = self.doc.width;
                    for (i, px) in pixels.chunks_mut(4).enumerate() {
                        let (x, y) = (i as u32 % w, i as u32 / w);
                        let t = 0.5 * (1.0 - q.pixel(x, y)[0] as f32 / 255.0);
                        if t <= 0.0 {
                            continue;
                        }
                        let a = px[3] as f32 / 255.0;
                        let red = [255.0, 0.0, 0.0];
                        for c in 0..3 {
                            px[c] = (px[c] as f32 * a * (1.0 - t) + red[c] * t)
                                .round()
                                .clamp(0.0, 255.0) as u8;
                        }
                        px[3] = 255;
                    }
                }
                let img = Arc::new(CanvasImage {
                    key: self.doc.id.0,
                    revision: rev,
                    width: self.doc.width,
                    height: self.doc.height,
                    pixels,
                });
                self.canvas = Some(img.clone());
                img
            }
        }
    }

    /// The Eyedropper's color at (`x`, `y`): the average of a `size` ×
    /// `size` square (clipped to the canvas) of the merged image or, without
    /// `all_layers`, of the active layer. Averaged with alpha weighting, and
    /// opaque; `None` outside the canvas or where everything sampled is
    /// transparent.
    pub fn sample_average(
        &mut self,
        x: u32,
        y: u32,
        size: u32,
        scope: op_core::SampleScope,
    ) -> Option<Color> {
        use op_core::SampleScope;
        let (w, h) = (self.doc.width, self.doc.height);
        if x >= w || y >= h {
            return None;
        }
        let below = (scope == SampleScope::CurrentAndBelow)
            .then(|| self.doc.sample_source(scope))
            .flatten();
        let merged = (scope == SampleScope::All).then(|| self.canvas_image());
        let layer = match scope {
            SampleScope::Current => {
                let id = self.doc.active_layer?;
                Some(self.doc.layer(id)?.image()?)
            }
            SampleScope::CurrentAndBelow => Some(below.as_ref()?),
            SampleScope::All => None,
        };
        let half = (size / 2) as i64;
        let mut sum = [0u64; 3];
        let mut alpha = 0u64;
        for sy in (y as i64 - half).max(0)..=(y as i64 + half).min(h as i64 - 1) {
            for sx in (x as i64 - half).max(0)..=(x as i64 + half).min(w as i64 - 1) {
                let (sx, sy) = (sx as u32, sy as u32);
                let px: [u8; 4] = match (&merged, layer) {
                    (Some(img), _) => {
                        let i = ((sy * img.width + sx) * 4) as usize;
                        img.pixels[i..i + 4].try_into().ok()?
                    }
                    (None, Some(image)) => image.pixel(sx, sy),
                    (None, None) => return None,
                };
                for c in 0..3 {
                    sum[c] += px[c] as u64 * px[3] as u64;
                }
                alpha += px[3] as u64;
            }
        }
        if alpha == 0 {
            return None;
        }
        // Picked colors are always opaque
        let [r, g, b] = sum.map(|v| ((v + alpha / 2) / alpha) as u8);
        Some(Color::from_rgba8([r, g, b, 255]))
    }

    /// The merged image's pixel at (`x`, `y`).
    #[cfg(test)]
    pub fn sample(&mut self, x: u32, y: u32) -> Option<Color> {
        if x >= self.doc.width || y >= self.doc.height {
            return None;
        }
        let img = self.canvas_image();
        let i = ((y * img.width + x) * 4) as usize;
        Some(Color::from_rgba8(img.pixels[i..i + 4].try_into().ok()?))
    }

    /// Luminosity histogram of the merged image, cached per revision.
    pub fn composite_histogram(&mut self) -> [u64; 256] {
        let rev = self.doc.revision();
        if let Some((r, h)) = &self.histogram
            && *r == rev
        {
            return *h;
        }
        let img = self.canvas_image();
        let mut hist = [0u64; 256];
        for px in img.pixels.chunks(4) {
            if px[3] > 0 {
                let l = op_core::adjust::luminosity([px[0], px[1], px[2], 255]);
                hist[l as usize] += 1;
            }
        }
        self.histogram = Some((rev, hist));
        hist
    }

    /// The merged image as a texture at most `max_px` on a side, cached
    /// per revision (the Navigator's thumbnail).
    pub fn composite_texture(
        &mut self,
        ctx: &egui::Context,
        max_px: u32,
    ) -> Option<egui::TextureHandle> {
        let rev = self.doc.revision();
        if let Some((r, tex)) = &self.composite_tex
            && *r == rev
        {
            return Some(tex.clone());
        }
        // The canvas's composite, unless Quick Mask tints it
        let image = if self.doc.quick_mask.is_none() {
            let canvas = self.canvas_image();
            scaled_thumbnail(&canvas.pixels, canvas.width, canvas.height, max_px)
        } else {
            composite_thumbnail(&self.doc, max_px)
        };
        let tex = ctx.load_texture(
            format!("navigator-{}", self.doc.id.0),
            image,
            egui::TextureOptions::LINEAR,
        );
        self.composite_tex = Some((rev, tex.clone()));
        Some(tex)
    }

    /// Thumbnail of the document as it was opened (History panel snapshot).
    pub fn snapshot_thumbnail(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        if let Some(image) = self.snapshot_thumb.take() {
            let name = format!("snapshot-{}", self.doc.id.0);
            self.snapshot_texture =
                Some(ctx.load_texture(name, image, egui::TextureOptions::LINEAR));
        }
        self.snapshot_texture.clone()
    }

    /// Layer thumbnail, cached per document revision.
    pub fn layer_thumbnail(
        &mut self,
        ctx: &egui::Context,
        layer: LayerId,
        max_px: u32,
    ) -> Option<egui::TextureHandle> {
        self.thumbnail(ctx, layer, false, max_px)
    }

    /// Thumbnail of a layer's mask, if it has one.
    pub fn mask_thumbnail(
        &mut self,
        ctx: &egui::Context,
        layer: LayerId,
        max_px: u32,
    ) -> Option<egui::TextureHandle> {
        self.thumbnail(ctx, layer, true, max_px)
    }

    fn thumbnail(
        &mut self,
        ctx: &egui::Context,
        layer: LayerId,
        mask: bool,
        max_px: u32,
    ) -> Option<egui::TextureHandle> {
        let rev = self.doc.revision();
        if let Some((r, tex)) = self.thumbs.get(&(layer, mask))
            && *r == rev
        {
            return Some(tex.clone());
        }
        let l = self.doc.layer(layer)?;
        let img = if mask {
            &l.mask.as_ref()?.image
        } else {
            // A group has no thumbnail of its own
            l.image()?
        };
        let scale = (max_px as f32 / img.width().max(img.height()) as f32).min(1.0);
        let tw = ((img.width() as f32 * scale).round() as u32).max(1);
        let th = ((img.height() as f32 * scale).round() as u32).max(1);
        let mut pixels = Vec::with_capacity((tw * th) as usize);
        for y in 0..th {
            for x in 0..tw {
                let sx = ((x as f32 + 0.5) / scale) as u32;
                let sy = ((y as f32 + 0.5) / scale) as u32;
                let [r, g, b, a] = img.pixel(sx.min(img.width() - 1), sy.min(img.height() - 1));
                pixels.push(egui::Color32::from_rgba_unmultiplied(r, g, b, a));
            }
        }
        let image = egui::ColorImage::new([tw as usize, th as usize], pixels);
        let tex = ctx.load_texture(
            format!("thumb-{}-{}-{mask}", self.doc.id.0, layer.0),
            image,
            egui::TextureOptions::LINEAR,
        );
        self.thumbs.insert((layer, mask), (rev, tex.clone()));
        Some(tex)
    }
}

const SNAPSHOT_THUMB_PX: u32 = 96;

/// Downscaled composite of the whole document (nearest-neighbor).
fn composite_thumbnail(doc: &Document, max_px: u32) -> egui::ColorImage {
    scaled_thumbnail(&doc.composite_rgba8(), doc.width, doc.height, max_px)
}

/// `pixels` (`width` × `height` straight RGBA8) at most `max_px` on a side,
/// nearest-neighbor.
fn scaled_thumbnail(pixels: &[u8], width: u32, height: u32, max_px: u32) -> egui::ColorImage {
    let scale = (max_px as f32 / width.max(height) as f32).min(1.0);
    let tw = ((width as f32 * scale).round() as u32).max(1);
    let th = ((height as f32 * scale).round() as u32).max(1);
    let mut out = Vec::with_capacity((tw * th) as usize);
    for y in 0..th {
        for x in 0..tw {
            let sx = (((x as f32 + 0.5) / scale) as u32).min(width - 1);
            let sy = (((y as f32 + 0.5) / scale) as u32).min(height - 1);
            let i = ((sy * width + sx) * 4) as usize;
            out.push(egui::Color32::from_rgba_unmultiplied(
                pixels[i],
                pixels[i + 1],
                pixels[i + 2],
                pixels[i + 3],
            ));
        }
    }
    egui::ColorImage::new([tw as usize, th as usize], out)
}

/// A marquee drag in progress, in document pixels.
#[derive(Clone, Copy, Debug)]
pub struct MarqueeDrag {
    pub start: egui::Pos2,
    pub current: egui::Pos2,
    pub op: op_core::SelectionOp,
    /// Shift/Alt were held when the drag started with an existing selection,
    /// so they chose the combine mode and don't constrain the shape.
    pub shift_for_op: bool,
    pub alt_for_op: bool,
}

/// Type tool options: the font style and size in points (Photoshop's
/// default 12 pt; at 72 ppi a point is a pixel).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeOptions {
    pub semibold: bool,
    pub size_pt: f32,
}

impl Default for TypeOptions {
    fn default() -> Self {
        Self {
            semibold: false,
            size_pt: 12.0,
        }
    }
}

impl TypeOptions {
    pub fn font(&self) -> &'static [u8] {
        if self.semibold {
            crate::theme::SOURCE_SANS_SEMIBOLD
        } else {
            crate::theme::SOURCE_SANS_REGULAR
        }
    }
}

/// Text being typed with the Type tool: where its first baseline starts,
/// what has been typed, and the document before it (the text shows live as
/// a layer, rebuilt from `before` on every change).
pub struct TextEdit {
    pub origin: egui::Pos2,
    pub text: String,
    pub before: op_core::Snapshot,
    /// The text as last shown on the document.
    pub shown: String,
}

/// Move tool options. Both are off by default, as in Photoshop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MoveOptions {
    /// Clicking picks the topmost layer with a pixel under the pointer
    /// (Cmd inverts this while held).
    pub auto_select: bool,
    /// Draw the transform box around the active layer; dragging one of its
    /// handles starts Free Transform.
    pub show_transform_controls: bool,
}

/// Shape tool options: the Polygon's sides (5) and the Line's weight
/// (1 px), Photoshop's defaults.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapeOptions {
    pub sides: u32,
    pub weight: f32,
}

impl Default for ShapeOptions {
    fn default() -> Self {
        Self {
            sides: 5,
            weight: 1.0,
        }
    }
}

/// Options of the retouching tools beyond their brushes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RetouchOptions {
    pub dodge_range: op_core::paint::ToneRange,
    pub burn_range: op_core::paint::ToneRange,
    /// Sponge: "Saturate" rather than "Desaturate" (the default).
    pub sponge_saturate: bool,
    /// Clone Stamp: "Aligned" (on by default): the offset from the source
    /// point stays the same from stroke to stroke.
    pub clone_aligned: bool,
}

impl Default for RetouchOptions {
    fn default() -> Self {
        Self {
            dodge_range: Default::default(),
            burn_range: Default::default(),
            sponge_saturate: false,
            clone_aligned: true,
        }
    }
}

/// View menu switches. Photoshop's defaults: rulers and grid off, Extras
/// and guides on, guides unlocked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewOptions {
    pub rulers: bool,
    /// View > Extras: selection edges, guides and the grid together.
    pub extras: bool,
    pub guides: bool,
    pub grid: bool,
    pub lock_guides: bool,
    /// View > Show > Smart Guides (on in Photoshop by default); nothing
    /// draws them yet.
    pub smart_guides: bool,
    /// View > Show > Pixel Grid (on by default): the grid between image
    /// pixels from 600% up.
    pub pixel_grid: bool,
    /// View > Show > Selection Edges (on by default).
    pub selection_edges: bool,
    /// View > Show > Layer Edges (off by default).
    pub layer_edges: bool,
    /// View > Snap and the Snap To items (all on by default, as in
    /// Photoshop 2026).
    pub snap: bool,
    pub snap_guides: bool,
    pub snap_grid: bool,
    pub snap_layers: bool,
    /// Slices: a switch only, there are no slices yet.
    pub snap_slices: bool,
    pub snap_bounds: bool,
}

impl Default for ViewOptions {
    fn default() -> Self {
        Self {
            rulers: false,
            extras: true,
            guides: true,
            grid: false,
            lock_guides: false,
            smart_guides: true,
            pixel_grid: true,
            selection_edges: true,
            layer_edges: false,
            snap: true,
            snap_guides: true,
            snap_grid: true,
            snap_layers: true,
            snap_slices: true,
            snap_bounds: true,
        }
    }
}

/// A guide being dragged: out of a ruler (`index` is `None`) or an
/// existing one, and where it is now.
#[derive(Clone, Copy, Debug)]
pub struct GuideDrag {
    pub index: Option<usize>,
    pub guide: op_core::Guide,
}

/// The Crop tool's box, in document pixels.
#[derive(Clone, Copy, Debug)]
pub struct CropBox {
    /// In box space: the image turned by −`angle` about its center, where
    /// the box is upright (the same as the image when `angle` is 0).
    pub rect: egui::Rect,
    /// How far the box is turned on the image, clockwise radians.
    pub angle: f32,
    pub drag: Option<CropDrag>,
}

/// A crop box drag: the handle (−1, 0 or 1 per axis; `None` moves the
/// box), where the pointer was pressed (screen) and the box at that time.
#[derive(Clone, Copy, Debug)]
pub struct CropDrag {
    pub handle: Option<(i8, i8)>,
    pub pointer: egui::Pos2,
    pub rect: egui::Rect,
    /// The box's angle when the drag started.
    pub angle: f32,
    /// What the drag does besides moving the box or a handle.
    pub kind: CropDragKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CropDragKind {
    Box,
    /// Outside the box: turning the image under it.
    Rotate,
    /// Straighten: a line to make level.
    Straighten,
}

/// What a Free Transform drag grabbed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformHandle {
    Move,
    Rotate,
    /// A scale handle: −1, 0 or 1 along each axis of the box (0 for the
    /// middle of a side).
    Scale(i8, i8),
}

/// A Free Transform drag: the handle and the box as it was at mouse-down.
#[derive(Clone, Copy, Debug)]
pub struct TransformDrag {
    pub handle: TransformHandle,
    pub start: egui::Pos2,
    pub offset: (f32, f32),
    pub scale: (f32, f32),
    pub angle: f32,
    /// The box's corners at mouse-down, when it has free corners.
    pub quad: Option<[egui::Pos2; 4]>,
    /// Warp: the mesh at mouse-down, and the grabbed control point or the
    /// surface point (u, v) being pulled.
    pub mesh: Option<op_core::transform::WarpMesh>,
    pub warp_grab: Option<WarpGrab>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WarpGrab {
    Point(usize),
    Surface(f32, f32),
}

/// Edit > Transform's modes: what dragging a handle does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TransformMode {
    /// Free Transform, Scale and Rotate: handles scale.
    #[default]
    Free,
    /// Handles slide along the box's sides.
    Skew,
    /// Corners move freely.
    Distort,
    /// A corner and its neighbor move apart or together.
    Perspective,
    /// A Bézier mesh over the box (Edit > Transform > Warp).
    Warp,
}

/// Edit > Free Transform in progress: the box over the original bounds,
/// scaled and rotated around its center and moved by `offset`. The
/// document shows the result live; `before` is restored to cancel.
pub struct FreeTransform {
    pub before: op_core::Snapshot,
    pub bounds: (f32, f32, f32, f32),
    pub offset: (f32, f32),
    pub scale: (f32, f32),
    pub angle: f32,
    /// Free corners (top left, top right, bottom right, bottom left, in
    /// document pixels) once the box is skewed, distorted or put in
    /// perspective; then they describe the box instead of the above.
    pub quad: Option<[egui::Pos2; 4]>,
    pub mode: TransformMode,
    /// Horizontal and vertical skew, radians.
    pub skew: (f32, f32),
    /// The reference point: −1, 0 or 1 along each axis of the box.
    pub reference: (i8, i8),
    /// The options bar's reference point checkbox (Photoshop 2026: off).
    pub show_reference: bool,
    /// X and Y relative to where the reference point started.
    pub relative: bool,
    /// W and H change together.
    pub linked: bool,
    pub interpolation: op_core::transform::Interpolation,
    /// Select > Transform Selection: only the selection's outline moves.
    pub selection_only: bool,
    /// Warp's mesh, once warping (document pixels).
    pub warp: Option<op_core::transform::WarpMesh>,
    pub drag: Option<TransformDrag>,
    /// The transform the document currently shows, and how it was
    /// resampled.
    pub applied: op_core::transform::Projective,
    pub applied_interpolation: op_core::transform::Interpolation,
    pub applied_warp: Option<op_core::transform::WarpMesh>,
}

impl FreeTransform {
    pub fn new(before: op_core::Snapshot, bounds: (f32, f32, f32, f32)) -> Self {
        Self {
            before,
            bounds,
            offset: (0.0, 0.0),
            scale: (1.0, 1.0),
            angle: 0.0,
            quad: None,
            mode: TransformMode::Free,
            skew: (0.0, 0.0),
            reference: (0, 0),
            show_reference: false,
            relative: false,
            linked: true,
            interpolation: op_core::transform::Interpolation::Bicubic,
            selection_only: false,
            warp: None,
            drag: None,
            applied: op_core::transform::Projective::IDENTITY,
            applied_interpolation: op_core::transform::Interpolation::Bicubic,
            applied_warp: None,
        }
    }

    pub fn center(&self) -> (f32, f32) {
        let (x0, y0, x1, y1) = self.bounds;
        ((x0 + x1) / 2.0, (y0 + y1) / 2.0)
    }

    /// The box's map: the free corners' when there are some.
    pub fn mapping(&self) -> op_core::transform::Projective {
        match self.quad {
            Some(q) => {
                op_core::transform::Projective::rect_to_quad(self.bounds, q.map(|p| (p.x, p.y)))
            }
            None => op_core::transform::Projective::from_affine(self.affine()),
        }
    }

    pub fn affine(&self) -> op_core::transform::Affine {
        use op_core::transform::Affine;
        let (cx, cy) = self.center();
        Affine::translate(cx + self.offset.0, cy + self.offset.1)
            .after(Affine::rotate(self.angle))
            .after(Affine::skew(self.skew.0, self.skew.1))
            .after(Affine::scale(self.scale.0, self.scale.1))
            .after(Affine::translate(-cx, -cy))
    }

    /// The reference point on the original box.
    pub fn reference_point(&self) -> (f32, f32) {
        let (x0, y0, x1, y1) = self.bounds;
        let pick = |k: i8, a: f32, b: f32| match k {
            -1 => a,
            0 => (a + b) / 2.0,
            _ => b,
        };
        (
            pick(self.reference.0, x0, x1),
            pick(self.reference.1, y0, y1),
        )
    }

    /// Where the reference point is now.
    pub fn reference_now(&self) -> (f32, f32) {
        self.mapping().apply(self.reference_point())
    }

    /// Changes the box's numbers with `change`, keeping the reference
    /// point where it is (the options bar's edits pivot on it).
    pub fn pivoting(&mut self, change: impl FnOnce(&mut Self)) {
        let before = self.reference_now();
        change(self);
        let after = self.reference_now();
        self.offset.0 += before.0 - after.0;
        self.offset.1 += before.1 - after.1;
    }
}

/// A Lasso or Polygonal Lasso outline being drawn, in document pixels.
#[derive(Clone, Debug)]
pub struct LassoPath {
    pub points: Vec<egui::Pos2>,
    pub op: op_core::SelectionOp,
    pub polygonal: bool,
    /// The Lasso let go with Alt held: straight edges by clicking until
    /// Alt is released (Photoshop's temporary Polygonal Lasso).
    pub held: bool,
}

/// The Magnetic Lasso's progress: the edge map it snaps to, which points
/// of the outline (`DocState::lasso`) are anchors, and how many points are
/// fixed (the rest is the live wire to the pointer).
#[derive(Clone)]
pub struct MagneticPath {
    pub edges: Arc<op_core::magnetic::EdgeMap>,
    pub anchors: Vec<usize>,
    pub fixed: usize,
}

/// A Quick Selection stroke: the region it grows, the selection it
/// started from and how it combines with it, and where its last dab was.
pub struct QuickStroke {
    pub select: op_core::smart_select::QuickSelect,
    pub base: Option<op_core::Selection>,
    pub op: op_core::SelectionOp,
    pub last: egui::Pos2,
}

/// Eyedropper options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EyedropperOptions {
    /// Side of the averaged square in pixels: 1 is "Point Sample".
    pub size: u32,
    /// "Sample:": the current layer, the layers up to it, or all merged.
    pub sample: op_core::SampleScope,
}

impl EyedropperOptions {
    /// Photoshop's "Sample Size" choices.
    pub const SIZES: [(u32, &'static str); 7] = [
        (1, "Point Sample"),
        (3, "3 by 3 Average"),
        (5, "5 by 5 Average"),
        (11, "11 by 11 Average"),
        (31, "31 by 31 Average"),
        (51, "51 by 51 Average"),
        (101, "101 by 101 Average"),
    ];
}

impl Default for EyedropperOptions {
    fn default() -> Self {
        Self {
            size: 1,
            sample: op_core::SampleScope::All,
        }
    }
}

/// Magic Wand options: combine mode plus the region rule it shares with the
/// Paint Bucket (tolerance 32, anti-alias and contiguous on by default).
#[derive(Clone, Copy, Debug, Default)]
pub struct WandOptions {
    pub mode: SelectionMode,
    pub region: op_core::fill::BucketOptions,
}

/// Selection combine mode (the four buttons in the options bar).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SelectionMode {
    #[default]
    New,
    Add,
    Subtract,
    Intersect,
}

impl SelectionMode {
    pub fn op(self) -> op_core::SelectionOp {
        match self {
            Self::New => op_core::SelectionOp::Replace,
            Self::Add => op_core::SelectionOp::Add,
            Self::Subtract => op_core::SelectionOp::Subtract,
            Self::Intersect => op_core::SelectionOp::Intersect,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MarqueeStyle {
    #[default]
    Normal,
    FixedRatio,
    FixedSize,
}

impl MarqueeStyle {
    pub const ALL: [Self; 3] = [Self::Normal, Self::FixedRatio, Self::FixedSize];
}

pub struct MarqueeOptions {
    pub mode: SelectionMode,
    pub feather: f32,
    /// Photoshop has Anti-alias on by default (it applies to the elliptical
    /// marquee).
    pub anti_alias: bool,
    pub style: MarqueeStyle,
}

impl Default for MarqueeOptions {
    fn default() -> Self {
        Self {
            mode: SelectionMode::New,
            feather: 0.0,
            anti_alias: true,
            style: MarqueeStyle::Normal,
        }
    }
}

/// What a confirmed Color Picker color is applied to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerTarget {
    Foreground,
    Background,
    /// The "Other..." canvas extension color in the Canvas Size dialog.
    CanvasExtension,
    /// "Color..." in the Fill dialog.
    FillColor,
}

pub struct PickerSession {
    pub picker: crate::dialogs::ColorPicker,
    pub target: PickerTarget,
}

/// Settings of a painting tool. Photoshop keeps them per tool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintOptions {
    /// Brush diameter in pixels (1–5000).
    pub size: f32,
    /// 0..=1; not used by the Pencil.
    pub hardness: f32,
    pub opacity: f32,
    /// 0..=1; not used by the Pencil.
    pub flow: f32,
}

impl PaintOptions {
    pub const MAX_SIZE: f32 = 5000.0;

    /// Photoshop's default round brush: 30 px, 0% hardness.
    const fn brush() -> Self {
        Self {
            size: 30.0,
            hardness: 0.0,
            opacity: 1.0,
            flow: 1.0,
        }
    }

    const fn pencil() -> Self {
        Self {
            size: 1.0,
            hardness: 1.0,
            opacity: 1.0,
            flow: 1.0,
        }
    }

    /// A 13 px hard brush (the Smudge, Pattern Stamp, Background Eraser
    /// and Color Replacement tools) with `opacity` (the Smudge's Strength).
    const fn small(opacity: f32) -> Self {
        Self {
            size: 13.0,
            hardness: 1.0,
            opacity,
            flow: 1.0,
        }
    }

    /// A retouching tool: the round brush with `opacity` as its strength
    /// (Exposure for Dodge and Burn, Strength for Blur and Sharpen) and
    /// `flow` (the Sponge's Flow).
    const fn retouch(opacity: f32, flow: f32) -> Self {
        Self {
            opacity,
            flow,
            ..Self::brush()
        }
    }

    /// The `]` step at this size, as in Photoshop.
    pub fn size_step(size: f32) -> f32 {
        match size {
            s if s < 10.0 => 1.0,
            s if s < 50.0 => 5.0,
            s if s < 100.0 => 10.0,
            s if s < 200.0 => 25.0,
            s if s < 300.0 => 50.0,
            _ => 100.0,
        }
    }
}

pub struct AppState {
    pub docs: HashMap<DocId, DocState>,
    /// Tab order of the open documents.
    pub doc_order: Vec<DocId>,
    pub active_doc: Option<DocId>,
    pub tool: Tool,
    /// The tool each toolbar slot shows (the last one used in its group).
    pub tool_slots: Vec<Tool>,
    pub foreground: Color,
    pub background: Color,
    pub marquee: MarqueeOptions,
    pub view: ViewOptions,
    pub eyedropper: EyedropperOptions,
    pub gradient: op_core::gradient::GradientOptions,
    pub wand: WandOptions,
    pub brush: PaintOptions,
    pub pencil: PaintOptions,
    pub eraser: PaintOptions,
    /// Each retouching tool keeps its own brush, as in Photoshop. Dodge and
    /// Burn start at Exposure 50%, Blur and Sharpen at Strength 50%, the
    /// Sponge at Flow 50%.
    pub dodge: PaintOptions,
    pub burn: PaintOptions,
    pub sponge: PaintOptions,
    pub blur: PaintOptions,
    pub sharpen: PaintOptions,
    pub clone_stamp: PaintOptions,
    pub history_brush: PaintOptions,
    pub smudge: PaintOptions,
    pub pattern_stamp: PaintOptions,
    pub background_eraser: PaintOptions,
    pub color_replacement: PaintOptions,
    /// The Selection Brush (Photoshop's 200 px hard tip).
    pub selection_brush: PaintOptions,
    pub healing_brush: PaintOptions,
    pub spot_healing: PaintOptions,
    pub retouch: RetouchOptions,
    pub shape: ShapeOptions,
    pub move_options: MoveOptions,
    /// The Crop tool's options.
    pub crop_options: crate::crop_tool::CropOptions,
    pub type_options: TypeOptions,
    /// Whether the Color panel edits the background or the foreground color.
    pub editing_background: bool,
    /// Cached HSB so the hue doesn't snap back to 0 for grays.
    pub picker_hsb: Hsb,
    pub untitled_counter: u32,
    /// Error message to show to the user.
    pub alert: Option<String>,
    /// The rulers' unit, and a right-click on a ruler asking for its menu.
    pub ruler_units: crate::rulers::RulerUnit,
    pub ruler_menu: bool,
    /// What the status bar shows beside the zoom box.
    pub status_info: crate::status_info::StatusInfo,
    /// The status bar's arrow was clicked: its menu opens after the frame.
    pub status_menu: bool,
    /// How long the last command that changed a document took, in seconds
    /// (the status bar's Timing).
    pub last_timing: f32,
    /// View › Screen Mode.
    pub screen_mode: ScreenMode,
    /// Tab: the toolbar and options bar hidden (with the panels).
    pub hide_tools: bool,
    /// Tab or Shift+Tab: the panels and the icon column hidden.
    pub hide_panels: bool,
    /// Entering Full Screen Mode: Photoshop's warning, while asked.
    pub full_screen_prompt: Option<crate::dialogs::alert::Alert>,
    /// "Don't show again" was ticked in that warning.
    pub skip_full_screen_prompt: bool,
    /// Layer > Flatten Image's "Discard hidden layers?" while asked.
    pub flatten_prompt: Option<crate::dialogs::alert::Alert>,
    /// "Don't show again" was ticked in that prompt.
    pub skip_flatten_prompt: bool,
    /// Deleting a group with layers in it: "Group and Contents", "Group
    /// Only" or Cancel.
    pub delete_group_prompt: Option<crate::dialogs::alert::Alert>,
    /// Window menu panels that float (Info, Navigator, Histogram).
    pub floating: crate::panels::floating::FloatingPanels,
    /// Whether the History panel is popped out from the icon strip.
    pub history_open: bool,
    pub history_panel: crate::panels::history::PanelState,
    /// Image > Canvas Size, while open.
    pub canvas_size_dialog: Option<crate::dialogs::CanvasSizeDialog>,
    /// Edit > Fill, while open.
    pub fill_dialog: Option<crate::dialogs::FillDialog>,
    /// The last transform applied, for Edit > Transform > Again.
    pub last_transform: Option<op_core::transform::Projective>,
    /// The last filter applied, for Filter > Last Filter.
    pub last_filter: Option<op_core::filter::Filter>,
    /// Options bar settings that nothing reads yet (Photoshop's options
    /// the app doesn't implement), by key, so they keep what's typed.
    pub tool_settings: std::collections::HashMap<&'static str, String>,
    /// Each filter dialog's last settings, to open with next time.
    pub filter_settings: std::collections::HashMap<crate::dialogs::AdjustKind, Vec<String>>,
    /// An adjustment or filter dialog, while open.
    pub adjust_dialog: Option<crate::dialogs::AdjustDialog>,
    /// Select > Modify > Border/Smooth/Expand/Contract/Feather, while open.
    pub modify_dialog: Option<crate::dialogs::ModifyDialog>,
    /// File > New..., while open.
    pub new_document_dialog: Option<crate::dialogs::NewDocumentDialog>,
    /// The New Document dialog's Recent (newest first) and Saved presets,
    /// and whether its welcome box was closed; kept for this run.
    pub new_document_recent: Vec<crate::dialogs::DocumentPreset>,
    pub new_document_saved: Vec<crate::dialogs::DocumentPreset>,
    pub new_document_welcome_closed: bool,
    /// View > Guides > New Guide..., while open.
    pub new_guide_dialog: Option<crate::dialogs::NewGuideDialog>,
    /// Layer > New > Layer... while open.
    pub new_layer_dialog: Option<crate::dialogs::NewLayerDialog>,
    /// Layer > Duplicate Layer... while open.
    pub duplicate_dialog: Option<crate::dialogs::DuplicateLayerDialog>,
    /// Layer > Lock Layers...
    pub lock_dialog: Option<crate::dialogs::LockLayersDialog>,
    /// Image > Image Rotation > Arbitrary...
    pub rotate_dialog: Option<crate::dialogs::RotateCanvasDialog>,
    /// Image > Image Size, while open.
    pub image_size_dialog: Option<crate::dialogs::ImageSizeDialog>,
    /// Image > Trim, while open.
    pub trim_dialog: Option<crate::dialogs::TrimDialog>,
    /// Equalize's question when there is a selection.
    pub equalize_dialog: Option<crate::dialogs::EqualizeDialog>,
    /// Paint Bucket options.
    pub bucket: op_core::fill::BucketOptions,
    /// The Color Picker, while open. It can sit on top of Canvas Size.
    pub color_picker: Option<PickerSession>,
    /// Swatches panel contents; "Add to Swatches" appends here.
    pub swatches: Vec<Color>,
    /// Documents waiting to be closed (Close All, quitting); each one with
    /// unsaved changes asks first.
    pub close_queue: Vec<DocId>,
    /// The document whose "Save changes?" prompt is showing.
    pub save_prompt: Option<DocId>,
    /// Quit once the close queue is done (it was started by quitting).
    pub quit_after_close: bool,
    /// Every document was dealt with; the window may close now.
    pub quit_approved: bool,
    pub clipboard: crate::clipboard::Clipboard,
    /// A text field has keyboard focus (as of the last frame).
    pub typing: bool,
    /// Input events for egui's next frame: menu Cut/Copy/Paste passed on to
    /// the focused text field.
    pub forward_events: Vec<egui::Event>,
}

impl Default for AppState {
    fn default() -> Self {
        let foreground = Color::from_rgba8([0x14, 0xa5, 0xdc, 0xff]);
        Self {
            docs: HashMap::new(),
            doc_order: Vec::new(),
            active_doc: None,
            tool: Tool::RectangularMarquee,
            tool_slots: op_tools::TOOLBAR
                .iter()
                .map(|group| op_tools::default_in_group(group))
                .collect(),
            foreground,
            background: Color::WHITE,
            marquee: MarqueeOptions::default(),
            view: ViewOptions::default(),
            eyedropper: EyedropperOptions::default(),
            gradient: Default::default(),
            wand: WandOptions::default(),
            brush: PaintOptions::brush(),
            pencil: PaintOptions::pencil(),
            eraser: PaintOptions::brush(),
            dodge: PaintOptions::retouch(0.5, 1.0),
            burn: PaintOptions::retouch(0.5, 1.0),
            sponge: PaintOptions::retouch(1.0, 0.5),
            blur: PaintOptions::retouch(0.5, 1.0),
            sharpen: PaintOptions::retouch(0.5, 1.0),
            clone_stamp: PaintOptions::brush(),
            history_brush: PaintOptions::brush(),
            smudge: PaintOptions::small(0.5),
            pattern_stamp: PaintOptions::small(1.0),
            background_eraser: PaintOptions::small(1.0),
            color_replacement: PaintOptions::small(1.0),
            selection_brush: PaintOptions {
                size: 200.0,
                ..PaintOptions::small(1.0)
            },
            healing_brush: PaintOptions::small(1.0),
            spot_healing: PaintOptions::small(1.0),
            retouch: RetouchOptions::default(),
            shape: ShapeOptions::default(),
            move_options: MoveOptions::default(),
            crop_options: Default::default(),
            type_options: TypeOptions::default(),
            editing_background: false,
            picker_hsb: Hsb::from_color(foreground),
            untitled_counter: 0,
            alert: None,
            history_open: false,
            floating: Default::default(),
            history_panel: Default::default(),
            canvas_size_dialog: None,
            fill_dialog: None,
            trim_dialog: None,
            equalize_dialog: None,
            image_size_dialog: None,
            new_guide_dialog: None,
            new_layer_dialog: None,
            ruler_units: Default::default(),
            ruler_menu: false,
            status_info: Default::default(),
            status_menu: false,
            last_timing: 0.0,
            screen_mode: ScreenMode::Standard,
            hide_tools: false,
            hide_panels: false,
            full_screen_prompt: None,
            skip_full_screen_prompt: false,
            flatten_prompt: None,
            skip_flatten_prompt: false,
            delete_group_prompt: None,
            duplicate_dialog: None,
            lock_dialog: None,
            rotate_dialog: None,
            new_document_dialog: None,
            new_document_recent: Vec::new(),
            new_document_saved: Vec::new(),
            new_document_welcome_closed: false,
            modify_dialog: None,
            adjust_dialog: None,
            last_filter: None,
            filter_settings: Default::default(),
            tool_settings: Default::default(),
            last_transform: None,
            bucket: Default::default(),
            color_picker: None,
            swatches: crate::panels::DEFAULT_SWATCHES
                .iter()
                .map(|&hex| {
                    Color::from_rgba8([(hex >> 16) as u8, (hex >> 8) as u8, hex as u8, 255])
                })
                .collect(),
            close_queue: Vec::new(),
            save_prompt: None,
            quit_after_close: false,
            quit_approved: false,
            clipboard: crate::clipboard::Clipboard::new(false),
            typing: false,
            forward_events: Vec::new(),
        }
    }
}

impl AppState {
    /// Options of the painting tool `tool`, if it is one.
    pub fn paint_options(&mut self, tool: Tool) -> Option<&mut PaintOptions> {
        match tool {
            Tool::Brush => Some(&mut self.brush),
            Tool::Pencil => Some(&mut self.pencil),
            Tool::Eraser => Some(&mut self.eraser),
            Tool::Dodge => Some(&mut self.dodge),
            Tool::Burn => Some(&mut self.burn),
            Tool::Sponge => Some(&mut self.sponge),
            Tool::Blur => Some(&mut self.blur),
            Tool::Sharpen => Some(&mut self.sharpen),
            Tool::CloneStamp => Some(&mut self.clone_stamp),
            Tool::HistoryBrush => Some(&mut self.history_brush),
            Tool::Smudge => Some(&mut self.smudge),
            Tool::PatternStamp => Some(&mut self.pattern_stamp),
            Tool::BackgroundEraser => Some(&mut self.background_eraser),
            Tool::ColorReplacement => Some(&mut self.color_replacement),
            Tool::HealingBrush => Some(&mut self.healing_brush),
            Tool::SpotHealingBrush => Some(&mut self.spot_healing),
            Tool::SelectionBrush => Some(&mut self.selection_brush),
            _ => None,
        }
    }

    /// An options bar setting nothing reads yet, starting at `default`.
    pub fn setting(&mut self, key: &'static str, default: &str) -> &mut String {
        self.tool_settings
            .entry(key)
            .or_insert_with(|| default.to_owned())
    }

    /// A checkbox setting nothing reads yet.
    pub fn flag(&mut self, key: &'static str, default: bool) -> bool {
        self.setting(key, if default { "1" } else { "0" }) == "1"
    }

    pub fn set_flag(&mut self, key: &'static str, on: bool) {
        *self.setting(key, "0") = if on { "1" } else { "0" }.to_owned();
    }

    /// Makes `tool` current and the tool shown in its toolbar slot.
    pub fn select_tool(&mut self, tool: Tool) {
        self.tool = tool;
        self.tool_slots[tool.slot()] = tool;
    }

    /// Adds a document as the last tab and makes it active.
    pub fn add_document(&mut self, doc: Document, initial: &str) {
        let id = doc.id;
        self.docs.insert(id, DocState::new(doc, initial));
        self.doc_order.push(id);
        self.active_doc = Some(id);
    }

    /// Closes a document; the tab to its left becomes active if it was active.
    pub fn close_document(&mut self, id: DocId) {
        let index = self.doc_order.iter().position(|d| *d == id);
        self.doc_order.retain(|d| *d != id);
        self.docs.remove(&id);
        if self.active_doc == Some(id) {
            self.active_doc = index.and_then(|i| {
                self.doc_order
                    .get(i.saturating_sub(1))
                    .or(self.doc_order.first())
                    .copied()
            });
        }
    }

    /// Deletes the selected layers, asking first (like Photoshop) when that
    /// would take a group's layers along.
    pub fn delete_layers(&mut self) {
        let Some(state) = self.active() else {
            return;
        };
        match op_core::layer_ops::deleting_groups_with_contents(&state.doc) {
            Some(name) => {
                self.delete_group_prompt = Some(crate::dialogs::alert::Alert::choose(
                    format!(
                        "Delete the group \u{201c}{name}\u{201d} and its contents or delete only the group?"
                    ),
                    &["Group and Contents", "Group Only", "Cancel"],
                ));
            }
            None => crate::panels::delete_active_layer(state),
        }
    }

    /// Whether a modal dialog is open; menus and shortcuts are disabled meanwhile.
    pub fn modal_open(&self) -> bool {
        self.canvas_size_dialog.is_some()
            || self.fill_dialog.is_some()
            || self.trim_dialog.is_some()
            || self.equalize_dialog.is_some()
            || self.image_size_dialog.is_some()
            || self.new_guide_dialog.is_some()
            || self.new_layer_dialog.is_some()
            || self.flatten_prompt.is_some()
            || self.full_screen_prompt.is_some()
            || self.delete_group_prompt.is_some()
            || self.duplicate_dialog.is_some()
            || self.lock_dialog.is_some()
            || self.rotate_dialog.is_some()
            || self.new_document_dialog.is_some()
            || self.modify_dialog.is_some()
            || self.adjust_dialog.is_some()
            || self.color_picker.is_some()
            || self.save_prompt.is_some()
            || self.alert.is_some()
            || self.transforming()
            || self.typing_text()
    }

    /// Whether the active document has text being typed (menus and tool
    /// keys are off meanwhile; the keys go into the text).
    pub fn typing_text(&self) -> bool {
        self.active_doc
            .and_then(|id| self.docs.get(&id))
            .is_some_and(|d| d.text_edit.is_some())
    }

    /// Whether the active document is in Free Transform (menus and tool
    /// keys are off meanwhile, as in Photoshop).
    pub fn transforming(&self) -> bool {
        self.active_doc
            .and_then(|id| self.docs.get(&id))
            .is_some_and(|d| d.free_transform.is_some())
    }

    /// Opens the Color Picker for the foreground or background color, titled
    /// like Photoshop's ("Color Picker (Foreground Color)").
    pub fn open_color_picker(&mut self, target: PickerTarget) {
        let (title, color) = match target {
            PickerTarget::Foreground => ("Color Picker (Foreground Color)", self.foreground),
            PickerTarget::Background => ("Color Picker (Background Color)", self.background),
            PickerTarget::CanvasExtension => ("Color Picker", self.background),
            PickerTarget::FillColor => ("Color Picker (Fill Color)", self.foreground),
        };
        self.color_picker = Some(PickerSession {
            picker: crate::dialogs::ColorPicker::new(title, color),
            target,
        });
    }

    pub fn active(&mut self) -> Option<&mut DocState> {
        self.docs.get_mut(&self.active_doc?)
    }

    pub fn editing_color(&self) -> Color {
        if self.editing_background {
            self.background
        } else {
            self.foreground
        }
    }

    pub fn set_editing_color(&mut self, c: Color) {
        if self.editing_background {
            self.background = c;
        } else {
            self.foreground = c;
        }
    }
}

/// The Pattern Stamp's pattern: Photoshop's default, dots of green on dark
/// green (18 × 22 pixels, tiled).
pub fn default_pattern() -> op_core::TiledImage {
    let (w, h) = (18u32, 22u32);
    let mut image = op_core::TiledImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            image.set_pixel(x, y, [0x16, 0x34, 0x18, 255]);
        }
    }
    // Two rows of dots, the second shifted half a period
    for (cx, cy) in [
        (4.5f32, 5.5f32),
        (13.5, 5.5),
        (9.0, 16.5),
        (0.0, 16.5),
        (18.0, 16.5),
    ] {
        for y in 0..h {
            for x in 0..w {
                let (dx, dy) = (x as f32 + 0.5 - cx, y as f32 + 0.5 - cy);
                if dx * dx + dy * dy <= 2.6 * 2.6 {
                    image.set_pixel(x, y, [0x3f, 0x7a, 0x3a, 255]);
                }
            }
        }
    }
    image
}

/// View › Screen Mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ScreenMode {
    /// The window with its title bar and document tabs.
    #[default]
    Standard,
    /// The window filling the screen below the menu bar, without its
    /// title bar or document tabs.
    FullWithMenus,
    /// The whole screen: no menu bar, no panels, a black pasteboard.
    Full,
}

impl ScreenMode {
    /// The next mode (F), or the previous one (Shift+F).
    pub fn cycle(self, back: bool) -> Self {
        use ScreenMode::*;
        match (self, back) {
            (Standard, false) | (Full, true) => FullWithMenus,
            (FullWithMenus, false) | (Standard, true) => Full,
            (Full, false) | (FullWithMenus, true) => Standard,
        }
    }
}

/// Photoshop's warning on entering Full Screen Mode.
pub const FULL_SCREEN_WARNING: &str = "In Full Screen Mode, panels are hidden. They can be accessed on the sides of the screen, or revealed by pressing Tab.\n\nWhile in Full Screen Mode, you can return to Standard Screen Mode by pressing 'F' or Esc.";

impl AppState {
    /// Switches to `mode`; Full Screen Mode asks first, unless told not to.
    pub fn set_screen_mode(&mut self, mode: ScreenMode) {
        if mode == ScreenMode::Full && !self.skip_full_screen_prompt {
            let mut alert = crate::dialogs::alert::Alert::caution(FULL_SCREEN_WARNING);
            alert.icon = crate::dialogs::alert::Icon::App;
            alert.ok_label = "Full Screen";
            self.full_screen_prompt = Some(alert);
            return;
        }
        self.enter_screen_mode(mode);
    }

    /// Switches to `mode` without asking. Full Screen Mode hides the panels
    /// and tools (Tab shows them); leaving it shows them again.
    pub fn enter_screen_mode(&mut self, mode: ScreenMode) {
        let was = self.screen_mode;
        self.screen_mode = mode;
        if mode == ScreenMode::Full {
            self.hide_tools = true;
            self.hide_panels = true;
        } else if was == ScreenMode::Full {
            self.hide_tools = false;
            self.hide_panels = false;
        }
    }

    /// Tab: hides the toolbar, options bar and panels, or shows them all
    /// when they are all hidden. Shift+Tab: the same for the panels only.
    pub fn toggle_hidden(&mut self, panels_only: bool) {
        if panels_only {
            self.hide_panels = !self.hide_panels;
        } else {
            let all = self.hide_tools && self.hide_panels;
            self.hide_tools = !all;
            self.hide_panels = !all;
        }
    }
}
