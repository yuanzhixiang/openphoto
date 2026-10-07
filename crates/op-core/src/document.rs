use std::sync::atomic::{AtomicU64, Ordering};

use crate::blend;
use crate::color::Color;
use crate::layer::{BlendMode, Layer, LayerId, LayerKind};
use crate::pixel::{BitDepth, ColorMode};
use crate::selection::Selection;
use crate::tile::{TILE_SIZE, TiledImage};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DocId(pub u64);

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Where the existing image is pinned when the canvas size changes
/// (the 3×3 grid in the Canvas Size dialog).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Anchor {
    /// 0 = left, 1 = center, 2 = right.
    pub x: u8,
    /// 0 = top, 1 = middle, 2 = bottom.
    pub y: u8,
}

impl Anchor {
    pub const CENTER: Self = Self { x: 1, y: 1 };

    /// Offset of the old image inside the new canvas along one axis.
    fn offset(pos: u8, old: u32, new: u32) -> i64 {
        let diff = new as i64 - old as i64;
        match pos {
            0 => 0,
            // Odd differences put the extra pixel on the right/bottom
            1 => diff.div_euclid(2),
            _ => diff,
        }
    }
}

/// A ruler guide: a horizontal or vertical line at `position` document
/// pixels (it may lie outside the canvas).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Guide {
    pub vertical: bool,
    pub position: f32,
    /// Its own color (New Guide's Color), or None for the guides' color.
    pub color: Option<[u8; 3]>,
}

/// The undoable part of a document. Cheap to clone because tiles are shared.
#[derive(Clone)]
pub struct Snapshot {
    guides: Vec<Guide>,
    quick_mask: Option<TiledImage>,
    width: u32,
    height: u32,
    resolution: f32,
    layers: Vec<Layer>,
    active_layer: Option<LayerId>,
    selection: Option<Selection>,
    last_selection: Option<Selection>,
}

/// Which pixels a tool samples: its "Sample:" option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SampleScope {
    #[default]
    Current,
    CurrentAndBelow,
    All,
}

pub struct Document {
    pub id: DocId,
    pub title: String,
    pub width: u32,
    pub height: u32,
    /// Resolution in pixels per inch.
    pub resolution: f32,
    pub color_mode: ColorMode,
    pub bit_depth: BitDepth,
    /// Ordered bottom to top.
    pub layers: Vec<Layer>,
    pub active_layer: Option<LayerId>,
    /// Bumped on every pixel or layer property change; the renderer uses it to
    /// decide whether to re-upload.
    revision: u64,
    /// The current selection; `None` means nothing is selected (Photoshop then
    /// treats the whole document as the target of edits).
    selection: Option<Selection>,
    /// The selection before the last Deselect, for Select > Reselect.
    last_selection: Option<Selection>,
    /// Bumped when the selection changes, so its outline can be cached.
    selection_revision: u64,
    /// Ruler guides; undoable like edits, as in Photoshop.
    pub guides: Vec<Guide>,
    /// Edits go to the active layer's mask (its thumbnail was clicked in
    /// the Layers panel) rather than its pixels. Not part of the history.
    pub mask_target: bool,
    /// Layers selected together in the Layers panel (Shift- or Cmd-click).
    /// Only counts while it contains the active layer: setting
    /// `active_layer` to another layer makes that the only selected one.
    /// Not part of the history, like Photoshop's layer selection.
    selected_layers: Vec<LayerId>,
    /// Quick Mask mode (Q): the selection as a gray image being painted
    /// (white selected, black not). Takes every pixel edit while on.
    pub quick_mask: Option<TiledImage>,
}

/// Where pixel edits go: the active layer's pixels, or its mask.
pub struct EditTarget<'a> {
    pub image: &'a mut TiledImage,
    /// The mask is being edited: colors become grays.
    pub mask: bool,
    /// Alpha must be kept: the background layer, locked transparency, or a
    /// mask (always opaque).
    pub keep_alpha: bool,
}

impl Document {
    pub(crate) fn empty(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            id: DocId(next_id()),
            title: title.into(),
            width,
            height,
            resolution: 72.0,
            color_mode: ColorMode::Rgb,
            bit_depth: BitDepth::U8,
            layers: Vec::new(),
            active_layer: None,
            revision: 0,
            selection: None,
            last_selection: None,
            selection_revision: 0,
            guides: Vec::new(),
            mask_target: false,
            selected_layers: Vec::new(),
            quick_mask: None,
        }
    }

    /// File > New: a single background layer filled with `background`.
    pub fn new_with_background(
        title: impl Into<String>,
        width: u32,
        height: u32,
        background: Color,
    ) -> Self {
        let mut doc = Self::empty(title, width, height);
        let image = TiledImage::filled(width, height, background.to_rgba8());
        let mut layer = Layer::raster(doc.new_layer_id(), "Background", image);
        layer.is_background = true;
        doc.active_layer = Some(layer.id);
        doc.layers.push(layer);
        doc
    }

    /// Opens a flat bitmap file as a single layer, like Photoshop: an opaque
    /// image becomes the locked "Background" layer, while an image with any
    /// transparency becomes a regular "Layer 0".
    pub fn from_rgba8(title: impl Into<String>, width: u32, height: u32, pixels: &[u8]) -> Self {
        let mut doc = Self::empty(title, width, height);
        let opaque = pixels.as_chunks::<4>().0.iter().all(|p| p[3] == 255);
        let image = TiledImage::from_rgba8(width, height, pixels);
        let name = if opaque { "Background" } else { "Layer 0" };
        let mut layer = Layer::raster(doc.new_layer_id(), name, image);
        layer.is_background = opaque;
        doc.active_layer = Some(layer.id);
        doc.layers.push(layer);
        doc
    }

    pub fn new_layer_id(&self) -> LayerId {
        LayerId(next_id())
    }

    /// Name for a new layer, like Photoshop: one more than the highest
    /// existing "Layer N".
    pub fn next_layer_name(&self) -> String {
        let n = self
            .layers
            .iter()
            .filter_map(|l| l.name.strip_prefix("Layer ")?.parse::<u32>().ok())
            .max()
            .map_or(1, |max| max + 1);
        format!("Layer {n}")
    }

    /// Adds `layer` directly above the active layer (on top without one)
    /// and makes it active.
    pub fn insert_above_active(&mut self, mut layer: Layer) {
        let (index, parent) = self.insertion_point();
        layer.parent = parent;
        self.active_layer = Some(layer.id);
        self.layers.insert(index, layer);
        self.mark_dirty();
    }

    /// Where a new layer goes, and into which group: onto the top of the
    /// active layer's layers when it's an expanded group (as in
    /// Photoshop), else directly above it in its group; on top without an
    /// active layer.
    pub fn insertion_point(&self) -> (usize, Option<LayerId>) {
        match self
            .active_layer
            .and_then(|a| self.layers.iter().position(|l| l.id == a))
        {
            Some(i) if matches!(self.layers[i].kind, LayerKind::Group { collapsed: false }) => {
                (i, Some(self.layers[i].id))
            }
            Some(i) => (i + 1, self.layers[i].parent),
            None => (self.layers.len(), None),
        }
    }

    /// "Group N" for a new group: one more than the highest N in use.
    pub fn next_group_name(&self) -> String {
        let n = self
            .layers
            .iter()
            .filter_map(|l| l.name.strip_prefix("Group ")?.parse::<u32>().ok())
            .max()
            .map_or(1, |max| max + 1);
        format!("Group {n}")
    }

    /// Where a layer's block (the layers in it, then the layer itself)
    /// sits in `layers`.
    pub fn block(&self, id: LayerId) -> Option<std::ops::Range<usize>> {
        let end = self.layers.iter().position(|l| l.id == id)? + 1;
        let start = self
            .descendants(id)
            .into_iter()
            .filter_map(|d| self.layers.iter().position(|l| l.id == d))
            .min()
            .unwrap_or(end - 1);
        Some(start..end)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn mark_dirty(&mut self) {
        self.revision += 1;
    }

    /// A layer as it is in `snapshot` (the History Brush paints from it).
    pub fn snapshot_layer(snapshot: &Snapshot, id: LayerId) -> Option<&Layer> {
        snapshot.layers.iter().find(|l| l.id == id)
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            guides: self.guides.clone(),
            quick_mask: self.quick_mask.clone(),
            width: self.width,
            height: self.height,
            resolution: self.resolution,
            layers: self.layers.clone(),
            active_layer: self.active_layer,
            selection: self.selection.clone(),
            last_selection: self.last_selection.clone(),
        }
    }

    pub fn restore(&mut self, snapshot: &Snapshot) {
        let s = snapshot.clone();
        self.width = s.width;
        self.height = s.height;
        self.resolution = s.resolution;
        self.layers = s.layers;
        self.active_layer = s.active_layer;
        self.selection = s.selection;
        self.last_selection = s.last_selection;
        self.guides = s.guides;
        self.quick_mask = s.quick_mask;
        self.selection_revision += 1;
        self.mark_dirty();
    }

    /// Moves each guide through `f` (document pixels, for the given
    /// orientation); used when the canvas is cropped, extended, resized,
    /// rotated or flipped. A rotation by 90° turns guides around, so `f`
    /// also returns the new orientation.
    pub fn map_guides(&mut self, f: impl Fn(Guide) -> Guide) {
        self.guides = self.guides.iter().map(|&g| f(g)).collect();
    }

    /// Image > Canvas Size. The background layer is extended with `fill`
    /// and cut to the new canvas; other layers are extended with
    /// transparency and keep their pixels outside it, as in Photoshop.
    pub fn resize_canvas(&mut self, width: u32, height: u32, anchor: Anchor, fill: Color) {
        let dx = Anchor::offset(anchor.x, self.width, width);
        let dy = Anchor::offset(anchor.y, self.height, height);
        self.place_canvas(width, height, dx, dy, fill);
    }

    /// The box around every layer's pixels and the canvas, as (x0, y0, x1,
    /// y1) in canvas coordinates: what Image > Reveal All shows.
    pub fn content_bounds(&self) -> (i64, i64, i64, i64) {
        let mut b = (0, 0, self.width as i64, self.height as i64);
        for image in self.layers.iter().filter_map(|l| l.image()) {
            if let Some((x0, y0, x1, y1)) = image.content_bounds() {
                b = (b.0.min(x0), b.1.min(y0), b.2.max(x1), b.3.max(y1));
            }
        }
        b
    }

    /// Whether `id` or a group it is in has `lock` set: a locked group locks
    /// everything in it, as in Photoshop.
    fn locked_by(&self, id: LayerId, lock: fn(&Layer) -> bool) -> bool {
        let mut at = self.layer(id);
        while let Some(layer) = at {
            if lock(layer) {
                return true;
            }
            at = layer.parent.and_then(|p| self.layer(p));
        }
        false
    }

    /// Whether `id` is inside a group with Lock all: Photoshop then won't
    /// delete it, change its blend mode or opacity, or give it a style.
    pub fn in_locked_group(&self, id: LayerId) -> bool {
        let mut parent = self.layer(id).and_then(|l| l.parent);
        while let Some(g) = parent.and_then(|p| self.layer(p)) {
            if g.lock_all {
                return true;
            }
            parent = g.parent;
        }
        false
    }

    /// Whether `id`'s transparent pixels are protected, by its own lock or
    /// one of its groups'.
    pub fn transparency_locked(&self, id: LayerId) -> bool {
        self.locked_by(id, Layer::transparency_locked)
    }

    /// Whether `id`'s pixels can't be edited, by its own lock or one of its
    /// groups'.
    pub fn pixels_locked(&self, id: LayerId) -> bool {
        self.locked_by(id, Layer::pixels_locked)
    }

    /// Whether `id` can't move, by its own lock or one of its groups'.
    pub fn position_locked(&self, id: LayerId) -> bool {
        self.locked_by(id, Layer::position_locked)
    }

    /// A `width`×`height` canvas with the old one at (`dx`, `dy`): the
    /// common part of Canvas Size and Reveal All.
    pub fn place_canvas(&mut self, width: u32, height: u32, dx: i64, dy: i64, fill: Color) {
        let fill = fill.to_rgba8();
        for layer in &mut self.layers {
            let extension = if layer.is_background {
                [fill[0], fill[1], fill[2], 255]
            } else {
                [0; 4]
            };
            let background = layer.is_background;
            if let Some(image) = layer.image_mut() {
                *image = image.with_canvas(width, height, dx, dy, extension);
                if background {
                    *image = image.clipped();
                }
            }
            // New canvas areas are revealed by the mask
            if let Some(mask) = &mut layer.mask {
                mask.image = mask.image.with_canvas(width, height, dx, dy, [255; 4]);
            }
        }
        for sel in [&mut self.selection, &mut self.last_selection]
            .into_iter()
            .flatten()
        {
            *sel = sel.with_canvas(width, height, dx, dy);
        }
        self.map_guides(|g| Guide {
            position: g.position + if g.vertical { dx } else { dy } as f32,
            ..g
        });
        self.selection_revision += 1;
        self.width = width;
        self.height = height;
        self.mark_dirty();
    }

    /// Replaces the canvas with a `width`×`height` one: every layer's image
    /// and both selections (current and the one Reselect brings back) are
    /// passed through `image` and `selection`. Used by Crop, Trim, Image
    /// Rotation and the canvas flips.
    pub fn transform_canvas(
        &mut self,
        width: u32,
        height: u32,
        image: impl Fn(&TiledImage) -> TiledImage,
        selection: impl Fn(&Selection) -> Selection,
    ) {
        for layer in &mut self.layers {
            if let Some(img) = layer.image_mut() {
                *img = image(img);
            }
            if let Some(mask) = &mut layer.mask {
                mask.image = image(&mask.image);
            }
        }
        for sel in [&mut self.selection, &mut self.last_selection]
            .into_iter()
            .flatten()
        {
            *sel = selection(sel);
        }
        if let Some(q) = &mut self.quick_mask {
            *q = image(q);
        }
        self.selection_revision += 1;
        self.width = width;
        self.height = height;
        self.mark_dirty();
    }

    /// Select > Edit in Quick Mask Mode: the selection becomes a gray
    /// image (no selection: everything white) and the selection goes away
    /// until Quick Mask is left.
    pub fn enter_quick_mask(&mut self) {
        let (w, h) = (self.width, self.height);
        let image = match &self.selection {
            Some(s) => {
                let mut pixels = Vec::with_capacity((w * h * 4) as usize);
                for y in 0..h {
                    for x in 0..w {
                        let v = s.get(x, y);
                        pixels.extend_from_slice(&[v, v, v, 255]);
                    }
                }
                TiledImage::from_rgba8(w, h, &pixels)
            }
            None => TiledImage::filled(w, h, [255; 4]),
        };
        self.quick_mask = Some(image);
        self.selection = None;
        self.selection_revision += 1;
        self.mark_dirty();
    }

    /// Leaves Quick Mask: the painted grays become the selection. A mask
    /// that is all white or all black leaves no selection.
    pub fn exit_quick_mask(&mut self) {
        let Some(image) = self.quick_mask.take() else {
            return;
        };
        let (w, h) = (self.width, self.height);
        let mut mask = Vec::with_capacity((w * h) as usize);
        for y in 0..h {
            for x in 0..w {
                mask.push(image.pixel(x, y)[0]);
            }
        }
        let all = mask.iter().all(|&v| v == 255);
        let s = Selection::from_mask(w, h, mask, false);
        self.selection = (!all && !s.is_empty()).then_some(s);
        self.selection_revision += 1;
        self.mark_dirty();
    }

    /// Whether edits go to the active layer's mask.
    pub fn editing_mask(&self) -> bool {
        self.mask_target
            && self
                .active_layer
                .and_then(|id| self.layer(id))
                .is_some_and(|l| l.mask.is_some())
    }

    /// The image pixel edits (painting, fills, gradients) go to.
    pub fn edit_target(&mut self) -> Option<EditTarget<'_>> {
        match self.quick_mask {
            Some(ref mut image) => Some(EditTarget {
                image,
                mask: true,
                keep_alpha: true,
            }),
            None => self.layer_target(),
        }
    }

    /// The active layer's pixels or mask (outside Quick Mask).
    fn layer_target(&mut self) -> Option<EditTarget<'_>> {
        let editing_mask = self.editing_mask();
        let id = self.active_layer?;
        let transparency_locked = self.transparency_locked(id);
        let layer = self.layers.iter_mut().find(|l| l.id == id)?;
        if editing_mask {
            let mask = layer.mask.as_mut()?;
            return Some(EditTarget {
                image: &mut mask.image,
                mask: true,
                keep_alpha: true,
            });
        }
        let keep_alpha = layer.is_background || transparency_locked;
        // A group has no pixels to edit
        let image = layer.image_mut()?;
        Some(EditTarget {
            image,
            mask: false,
            keep_alpha,
        })
    }

    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }

    pub fn selection_revision(&self) -> u64 {
        self.selection_revision
    }

    /// Replaces the selection. An empty selection counts as no selection.
    /// Clearing a selection remembers it for [`Self::reselect`].
    pub fn set_selection(&mut self, selection: Option<Selection>) {
        let selection = selection.filter(|s| !s.is_empty());
        if selection.is_none() && self.selection.is_some() {
            self.last_selection = self.selection.take();
        }
        self.selection = selection;
        self.selection_revision += 1;
    }

    /// Select > Reselect: restores the selection cleared by the last Deselect.
    pub fn reselect(&mut self) -> bool {
        match self.last_selection.take() {
            Some(s) => {
                self.selection = Some(s);
                self.selection_revision += 1;
                true
            }
            None => false,
        }
    }

    pub fn can_reselect(&self) -> bool {
        self.selection.is_none() && self.last_selection.is_some()
    }

    /// Whether the document has a background layer (which decides whether the
    /// canvas extension color applies).
    pub fn has_background(&self) -> bool {
        self.layers.iter().any(|l| l.is_background)
    }

    pub fn layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    pub fn layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }

    /// Composites all visible layers on the CPU into tightly packed straight RGBA8.
    ///
    /// Blending happens in gamma-encoded space, which is Photoshop's default,
    /// with each layer's blend mode (see [`crate::blend`]).
    /// The selected layers, bottom to top: the multi-selection when it
    /// includes the active layer, else just the active layer (or none).
    pub fn selected_layers(&self) -> Vec<LayerId> {
        let Some(active) = self.active_layer else {
            return Vec::new();
        };
        if !self.selected_layers.contains(&active) {
            return vec![active];
        }
        self.layers
            .iter()
            .map(|l| l.id)
            .filter(|id| self.selected_layers.contains(id))
            .collect()
    }

    /// Whether the layer is selected (alone or with others).
    pub fn is_layer_selected(&self, id: LayerId) -> bool {
        self.selected_layers().contains(&id)
    }

    /// A click on a layer: it alone is selected and active.
    pub fn select_layer(&mut self, id: LayerId) {
        self.active_layer = Some(id);
        self.selected_layers = vec![id];
    }

    /// Cmd-click: adds the layer to the selection (and makes it active), or
    /// takes it out; the last selected layer stays.
    pub fn toggle_layer_selection(&mut self, id: LayerId) {
        let mut selected = self.selected_layers();
        if let Some(i) = selected.iter().position(|&s| s == id) {
            if selected.len() == 1 {
                return;
            }
            selected.remove(i);
            if self.active_layer == Some(id) {
                self.active_layer = selected.last().copied();
            }
        } else {
            selected.push(id);
            self.active_layer = Some(id);
        }
        self.selected_layers = selected;
    }

    /// Shift-click: selects every layer between the active one and `id`
    /// (in the stack), and makes `id` active.
    pub fn select_layer_range(&mut self, id: LayerId) {
        let index = |l: Option<LayerId>| l.and_then(|l| self.layers.iter().position(|x| x.id == l));
        let (Some(from), Some(to)) = (index(self.active_layer), index(Some(id))) else {
            self.select_layer(id);
            return;
        };
        let (a, b) = (from.min(to), from.max(to));
        self.selected_layers = self.layers[a..=b].iter().map(|l| l.id).collect();
        self.active_layer = Some(id);
    }

    /// Selects exactly `ids`, the last one active (none: nothing selected).
    pub fn set_selected_layers(&mut self, ids: Vec<LayerId>) {
        self.active_layer = ids.last().copied();
        self.selected_layers = ids;
    }

    /// Select > All Layers (Alt+Cmd+A): every layer but the background.
    /// Returns false when there's no such layer.
    pub fn select_all_layers(&mut self) -> bool {
        let ids: Vec<LayerId> = self
            .layers
            .iter()
            .filter(|l| !l.is_background)
            .map(|l| l.id)
            .collect();
        let Some(&top) = ids.last() else {
            return false;
        };
        self.selected_layers = ids;
        self.active_layer = Some(top);
        true
    }

    /// Select > Deselect Layers: no layer is selected.
    pub fn deselect_layers(&mut self) {
        self.selected_layers.clear();
        self.active_layer = None;
    }

    /// The layers inside a group, at any depth (empty for other layers).
    pub fn descendants(&self, id: LayerId) -> Vec<LayerId> {
        let mut out = Vec::new();
        let mut frontier = vec![id];
        while let Some(parent) = frontier.pop() {
            for l in self.layers.iter().filter(|l| l.parent == Some(parent)) {
                out.push(l.id);
                frontier.push(l.id);
            }
        }
        out
    }

    /// `ids` with every group replaced by the pixel layers inside it, in
    /// stack order (bottom to top), without duplicates.
    pub fn pixel_layers(&self, ids: &[LayerId]) -> Vec<LayerId> {
        let mut wanted: Vec<LayerId> = Vec::new();
        for &id in ids {
            wanted.push(id);
            wanted.extend(self.descendants(id));
        }
        self.layers
            .iter()
            .filter(|l| !l.is_group() && wanted.contains(&l.id))
            .map(|l| l.id)
            .collect()
    }

    /// Whether the layer and every group it is in are visible.
    pub fn is_shown(&self, id: LayerId) -> bool {
        let mut current = self.layer(id);
        while let Some(layer) = current {
            if !layer.visible {
                return false;
            }
            current = layer.parent.and_then(|p| self.layer(p));
        }
        true
    }

    /// The topmost visible layer showing a pixel at (x, y): the Move tool's
    /// Auto-Select. Pixels hidden by the layer's mask or a zero opacity
    /// don't count.
    pub fn layer_at(&self, x: u32, y: u32) -> Option<LayerId> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.layers
            .iter()
            .rev()
            .filter(|l| self.is_shown(l.id) && l.opacity * l.fill > 0.0)
            .find(|l| {
                let Some(image) = l.image() else {
                    return false;
                };
                let masked = l
                    .mask
                    .as_ref()
                    .is_some_and(|m| m.enabled && m.value(x, y) == 0);
                image.pixel(x, y)[3] > 0 && !masked
            })
            .map(|l| l.id)
    }

    /// The pixels a tool samples ("Sample:" in its bar): the active layer,
    /// the layers up to and including it, or all of them merged. `None`
    /// without an active pixel layer (for the current layer only).
    pub fn sample_source(&self, scope: SampleScope) -> Option<TiledImage> {
        let (w, h) = (self.width, self.height);
        match scope {
            SampleScope::Current => self.layer(self.active_layer?)?.image().cloned(),
            SampleScope::CurrentAndBelow => {
                let id = self.active_layer?;
                let i = self.layers.iter().position(|l| l.id == id)?;
                Some(TiledImage::from_rgba8(
                    w,
                    h,
                    &self.composite_layers_rgba8(&self.layers[..=i]),
                ))
            }
            SampleScope::All => Some(TiledImage::from_rgba8(w, h, &self.composite_rgba8())),
        }
    }

    pub fn composite_rgba8(&self) -> Vec<u8> {
        self.composite_layers_rgba8(&self.layers)
    }

    /// Like [`Self::composite_rgba8`] for a given list of layers (bottom to
    /// top, each the size of the document), e.g. the layers a merge
    /// combines. Groups composite their layers: straight into what's below
    /// when passing through at full opacity without a mask, else on their
    /// own first and then blended in as one. A layer whose group isn't in
    /// the list counts as top-level.
    ///
    /// Bands of tile rows composite on separate threads: a 3000 × 1080
    /// document has to composite well within a frame while a layer is
    /// dragged.
    pub fn composite_layers_rgba8(&self, layers: &[Layer]) -> Vec<u8> {
        let (w, h) = (self.width as usize, self.height as usize);
        let mut bytes = vec![0u8; w * h * 4];
        if w == 0 || h == 0 {
            return bytes;
        }
        let top = |l: &Layer| l.parent.is_none_or(|p| !layers.iter().any(|x| x.id == p));
        let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
        let tile = TILE_SIZE as usize;
        let tile_rows = h.div_ceil(tile);
        let band = tile_rows.div_ceil(threads).max(1) * tile;
        std::thread::scope(|scope| {
            for (i, chunk) in bytes.chunks_mut(band * w * 4).enumerate() {
                let top = &top;
                scope.spawn(move || {
                    let y0 = i * band;
                    let rows = chunk.len() / (w * 4);
                    let mut out = vec![0f32; chunk.len()];
                    self.composite_children(layers, top, &mut out, (y0, y0 + rows));
                    for (b, v) in chunk.iter_mut().zip(&out) {
                        *b = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
                    }
                });
            }
        });
        bytes
    }

    /// Composites the layers of `layers` that `belongs` picks, in order,
    /// onto `out` (straight RGBA, 0..=1): the document's rows `band.0`
    /// to `band.1`, a whole number of tile rows from a tile row's top.
    fn composite_children(
        &self,
        layers: &[Layer],
        belongs: &dyn Fn(&Layer) -> bool,
        out: &mut [f32],
        band: (usize, usize),
    ) {
        for layer in layers.iter().filter(|l| belongs(l) && l.visible) {
            match &layer.kind {
                LayerKind::Raster(image) => self.composite_image(layer, image, out, band),
                LayerKind::Group { .. } => {
                    let id = layer.id;
                    let child = move |l: &Layer| l.parent == Some(id);
                    let alpha = layer.opacity * layer.fill;
                    let mask = layer.mask.as_ref().filter(|m| m.enabled);
                    if layer.blend_mode == BlendMode::PassThrough && alpha >= 1.0 && mask.is_none()
                    {
                        self.composite_children(layers, &child, out, band);
                    } else if alpha > 0.0 {
                        let mut own = vec![0f32; out.len()];
                        self.composite_children(layers, &child, &mut own, band);
                        self.blend_buffer(layer, &own, out, band.0);
                    }
                }
            }
        }
    }

    /// Blends a group's own composite onto `out` with the group's mode,
    /// opacity and mask (Pass Through then acts as Normal).
    fn blend_buffer(&self, group: &Layer, own: &[f32], out: &mut [f32], y0: usize) {
        let w = self.width as usize;
        let alpha = group.opacity * group.fill;
        let mask = group.mask.as_ref().filter(|m| m.enabled);
        let mode = match group.blend_mode {
            BlendMode::PassThrough => BlendMode::Normal,
            m => m,
        };
        for (i, (s, d)) in own
            .as_chunks::<4>()
            .0
            .iter()
            .zip(out.as_chunks_mut::<4>().0.iter_mut())
            .enumerate()
        {
            let (x, y) = ((i % w) as u32, (i / w + y0) as u32);
            let m = mask.map_or(1.0, |m| m.value(x, y) as f32 / 255.0);
            let sa = s[3] * alpha * m;
            if sa <= 0.0 {
                continue;
            }
            let result =
                blend::composite(mode, [d[0], d[1], d[2], d[3]], [s[0], s[1], s[2]], sa, x, y);
            d.copy_from_slice(&result);
        }
    }

    /// Composites one pixel layer onto `out`, the rows of `band`.
    fn composite_image(
        &self,
        layer: &Layer,
        image: &TiledImage,
        out: &mut [f32],
        band: (usize, usize),
    ) {
        let (w, h) = (self.width as usize, band.1);
        let layer_alpha = layer.opacity * layer.fill;
        if layer_alpha <= 0.0 {
            return;
        }
        let mask = layer.mask.as_ref().filter(|m| m.enabled);
        let normal = layer.blend_mode == BlendMode::Normal;
        let tile_rows = band.0 / TILE_SIZE as usize..band.1.div_ceil(TILE_SIZE as usize);
        for ty in tile_rows.map(|t| t as u32).filter(|&t| t < image.tiles_y()) {
            for tx in 0..image.tiles_x() {
                let Some(tile) = image.tile(tx, ty) else {
                    continue;
                };
                // The mask uses the same tile grid; a missing tile hides
                let mask_tile = match mask {
                    Some(m) => match m.image.tile(tx, ty) {
                        Some(t) => Some(t),
                        None => continue,
                    },
                    None => None,
                };
                let (x0, y0) = ((tx * TILE_SIZE) as usize, (ty * TILE_SIZE) as usize);
                let tw = (TILE_SIZE as usize).min(w - x0);
                let th = (TILE_SIZE as usize).min(h - y0);
                for row in 0..th {
                    let src_row = &tile.data[row * TILE_SIZE as usize * 4..];
                    let dst_row = &mut out[((y0 + row - band.0) * w + x0) * 4..];
                    for col in 0..tw {
                        let s = &src_row[col * 4..col * 4 + 4];
                        let m = mask_tile.map_or(1.0, |t| {
                            t.data[(row * TILE_SIZE as usize + col) * 4] as f32 / 255.0
                        });
                        let sa = s[3] as f32 / 255.0 * layer_alpha * m;
                        if sa <= 0.0 {
                            continue;
                        }
                        let d = &mut dst_row[col * 4..col * 4 + 4];
                        let src = [s[0], s[1], s[2]].map(|v| v as f32 / 255.0);
                        // Normal: plain source-over, the common case
                        if normal {
                            if sa >= 1.0 {
                                d.copy_from_slice(&[src[0], src[1], src[2], 1.0]);
                            } else {
                                let da = d[3] * (1.0 - sa);
                                let oa = sa + da;
                                for c in 0..3 {
                                    d[c] = (src[c] * sa + d[c] * da) / oa;
                                }
                                d[3] = oa;
                            }
                            continue;
                        }
                        let (x, y) = ((x0 + col) as u32, (y0 + row) as u32);
                        let out = blend::composite(
                            layer.blend_mode,
                            [d[0], d[1], d[2], d[3]],
                            src,
                            sa,
                            x,
                            y,
                        );
                        d.copy_from_slice(&out);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_bitmap_opens_as_background() {
        let doc = Document::from_rgba8("t", 2, 1, &[1, 2, 3, 255, 4, 5, 6, 255]);
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.layers[0].name, "Background");
        assert!(doc.layers[0].is_background);
        assert!(doc.has_background());
    }

    #[test]
    fn transparent_bitmap_opens_as_regular_layer() {
        let doc = Document::from_rgba8("t", 2, 1, &[1, 2, 3, 255, 4, 5, 6, 128]);
        assert_eq!(doc.layers[0].name, "Layer 0");
        assert!(!doc.layers[0].is_background);
        assert!(!doc.has_background());
        assert_eq!(doc.active_layer, Some(doc.layers[0].id));
    }

    #[test]
    fn resize_canvas_centered() {
        let mut doc = Document::new_with_background("t", 2, 2, Color::WHITE);
        doc.resize_canvas(4, 5, Anchor::CENTER, Color::BLACK);
        assert_eq!((doc.width, doc.height), (4, 5));
        let px = doc.composite_rgba8();
        let at = |x: usize, y: usize| &px[(y * 4 + x) * 4..(y * 4 + x) * 4 + 4];
        assert_eq!(at(0, 0), &[0, 0, 0, 255]);
        assert_eq!(at(1, 1), &[255, 255, 255, 255]);
        assert_eq!(at(2, 2), &[255, 255, 255, 255]);
        assert_eq!(at(2, 3), &[0, 0, 0, 255]);
    }

    #[test]
    fn resize_canvas_keeps_layers_transparent() {
        let mut doc = Document::new_with_background("t", 2, 2, Color::WHITE);
        let id = doc.new_layer_id();
        doc.layers.push(Layer::raster(
            id,
            "Layer 1",
            TiledImage::filled(2, 2, [255, 0, 0, 255]),
        ));
        doc.resize_canvas(3, 3, Anchor { x: 0, y: 0 }, Color::BLACK);
        let img = doc.layer(id).unwrap().image().unwrap();
        assert_eq!(img.pixel(2, 2), [0; 4]);
        assert_eq!(img.pixel(1, 1), [255, 0, 0, 255]);
    }

    #[test]
    fn selection_deselect_reselect_and_undo() {
        use crate::selection::Rect;
        let mut doc = Document::new_with_background("t", 10, 10, Color::WHITE);
        let snapshot = doc.snapshot();
        doc.set_selection(Some(Selection::rect(10, 10, Rect::new(1.0, 1.0, 4.0, 4.0))));
        assert!(doc.selection().is_some());
        doc.set_selection(None);
        assert!(doc.can_reselect());
        assert!(doc.reselect());
        assert_eq!(doc.selection().unwrap().bounds(), Some((1, 1, 4, 4)));
        // Restoring an earlier snapshot restores its (empty) selection
        doc.restore(&snapshot);
        assert!(doc.selection().is_none());
        // An empty selection counts as none
        doc.set_selection(Some(Selection::rect(10, 10, Rect::new(3.0, 3.0, 3.0, 3.0))));
        assert!(doc.selection().is_none());
    }

    #[test]
    fn composite_uses_blend_mode() {
        let mut doc =
            Document::new_with_background("t", 1, 1, Color::from_rgba8([128, 128, 128, 255]));
        let id = doc.new_layer_id();
        let mut layer = Layer::raster(
            id,
            "Layer 1",
            TiledImage::filled(1, 1, [128, 128, 128, 255]),
        );
        layer.blend_mode = crate::BlendMode::Multiply;
        doc.layers.push(layer);
        assert_eq!(&doc.composite_rgba8()[..4], &[64, 64, 64, 255]);
    }

    #[test]
    fn composite_half_opacity_over_white() {
        let mut doc = Document::new_with_background("t", 4, 4, Color::WHITE);
        let id = doc.new_layer_id();
        let mut layer = Layer::raster(id, "Layer 1", TiledImage::filled(4, 4, [0, 0, 0, 255]));
        layer.opacity = 0.5;
        doc.layers.push(layer);
        let px = doc.composite_rgba8();
        assert_eq!(&px[0..4], &[128, 128, 128, 255]);
    }

    #[test]
    fn layer_at_finds_the_topmost_visible_pixel() {
        let mut doc = Document::new_with_background("t", 4, 4, Color::WHITE);
        let bg = doc.layers[0].id;
        let id = doc.new_layer_id();
        let mut image = TiledImage::new(4, 4);
        image.set_pixel(1, 1, [0, 0, 0, 255]);
        doc.layers.push(Layer::raster(id, "Layer 1", image));
        assert_eq!(doc.layer_at(1, 1), Some(id));
        assert_eq!(
            doc.layer_at(2, 2),
            Some(bg),
            "transparent pixels fall through"
        );
        doc.layers[1].visible = false;
        assert_eq!(doc.layer_at(1, 1), Some(bg));
        assert_eq!(doc.layer_at(9, 9), None);
    }

    #[test]
    fn canvas_size_keeps_hidden_pixels_except_on_the_background() {
        let mut doc = Document::new_with_background("t", 4, 4, Color::WHITE);
        let id = doc.new_layer_id();
        doc.layers.push(Layer::raster(
            id,
            "L",
            TiledImage::filled(4, 4, [1, 2, 3, 255]),
        ));
        doc.resize_canvas(2, 2, Anchor::CENTER, Color::BLACK);
        let image = doc.layer(id).unwrap().image().unwrap();
        assert_eq!(image.content_bounds(), Some((-1, -1, 3, 3)));
        let bg = doc.layers[0].image().unwrap();
        assert!(!bg.has_pixels_outside());
        // Growing back brings the hidden pixels back
        doc.resize_canvas(4, 4, Anchor::CENTER, Color::BLACK);
        let image = doc.layer(id).unwrap().image().unwrap();
        assert_eq!(image.pixel(0, 0), [1, 2, 3, 255]);
    }

    #[test]
    fn layer_multi_selection() {
        let mut doc = Document::new_with_background("t", 2, 2, Color::WHITE);
        let bg = doc.layers[0].id;
        let ids: Vec<LayerId> = (0..3)
            .map(|i| {
                let id = doc.new_layer_id();
                doc.layers
                    .push(Layer::raster(id, format!("L{i}"), TiledImage::new(2, 2)));
                id
            })
            .collect();
        doc.select_layer(ids[0]);
        assert_eq!(doc.selected_layers(), [ids[0]]);
        // Shift-click: a range, the clicked one active
        doc.select_layer_range(ids[2]);
        assert_eq!(doc.selected_layers(), ids);
        assert_eq!(doc.active_layer, Some(ids[2]));
        // Cmd-click takes one out (the active one: the last left becomes active)
        doc.toggle_layer_selection(ids[2]);
        assert_eq!(doc.selected_layers(), [ids[0], ids[1]]);
        assert_eq!(doc.active_layer, Some(ids[1]));
        // ...and puts it back, active
        doc.toggle_layer_selection(bg);
        assert_eq!(doc.selected_layers(), [bg, ids[0], ids[1]]);
        assert_eq!(doc.active_layer, Some(bg));
        // Setting the active layer directly makes it the only selected one
        doc.active_layer = Some(ids[2]);
        assert_eq!(doc.selected_layers(), [ids[2]]);
        // All Layers leaves the background out
        assert!(doc.select_all_layers());
        assert_eq!(doc.selected_layers(), ids);
        doc.deselect_layers();
        assert!(doc.selected_layers().is_empty());
    }

    #[test]
    fn groups_composite_their_layers() {
        // White background; a group holding a black layer
        let mut doc = Document::new_with_background("t", 1, 1, Color::WHITE);
        let group = doc.new_layer_id();
        let child = doc.new_layer_id();
        let mut black = Layer::raster(child, "black", TiledImage::filled(1, 1, [0, 0, 0, 255]));
        black.parent = Some(group);
        doc.layers.push(black);
        doc.layers.push(Layer::group(group, "Group 1"));
        // Pass Through at full opacity: as if not grouped
        assert_eq!(&doc.composite_rgba8()[..4], &[0, 0, 0, 255]);
        assert!(doc.is_shown(child));
        // A hidden group hides its layers
        doc.layer_mut(group).unwrap().visible = false;
        assert_eq!(&doc.composite_rgba8()[..4], &[255, 255, 255, 255]);
        assert!(!doc.is_shown(child));
        assert_eq!(doc.layer_at(0, 0), Some(doc.layers[0].id));
        // Half opacity: the group's result is blended in as one
        let g = doc.layer_mut(group).unwrap();
        g.visible = true;
        g.opacity = 0.5;
        assert_eq!(&doc.composite_rgba8()[..4], &[128, 128, 128, 255]);
        assert_eq!(doc.descendants(group), [child]);
        assert_eq!(doc.pixel_layers(&[group]), [child]);
    }
}
