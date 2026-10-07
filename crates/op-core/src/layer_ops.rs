//! Layer menu operations: duplicating, Layer Via Copy/Cut, converting the
//! background, arranging, merging and flattening.

use crate::clipboard::{self, ClipError};
use crate::document::Document;
use crate::layer::{BlendMode, Layer, LayerId, LayerMask, Locks};
use crate::tile::TiledImage;

/// Layer > Arrange.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrange {
    BringToFront,
    BringForward,
    SendBackward,
    SendToBack,
}

fn active_index(doc: &Document) -> Option<usize> {
    let id = doc.active_layer?;
    doc.layers.iter().position(|l| l.id == id)
}

/// "Name copy", then "Name copy 2", "Name copy 3"... like Photoshop. A
/// copy of a copy counts on instead: "A copy" gives "A copy 2" and
/// "E copy 7" gives "E copy 8" (checked in Photoshop 2026).
fn copy_name(doc: &Document, name: &str) -> String {
    let free = |candidate: &String| !doc.layers.iter().any(|l| &l.name == candidate);
    // "<base> copy" or "<base> copy N"
    let numbered = name.rsplit_once(' ').and_then(|(head, n)| {
        let n: u32 = n.parse().ok()?;
        head.ends_with(" copy").then_some((head, n + 1))
    });
    let (base, first) = match numbered {
        Some((head, next)) => (head.to_string(), next),
        None if name.ends_with(" copy") => (name.to_string(), 2),
        None => {
            let base = format!("{name} copy");
            if free(&base) {
                return base;
            }
            (base, 2)
        }
    };
    (first..)
        .map(|n| format!("{base} {n}"))
        .find(free)
        .expect("some name is free")
}

/// Layer > Duplicate Layer: an identical layer named "Name copy" directly
/// above the active one, which becomes active.
pub fn duplicate(doc: &mut Document) -> Option<LayerId> {
    let source = doc.layers[active_index(doc)?].clone();
    duplicate_named(doc, &copy_name(doc, &source.name))
}

/// A copy of `id`'s block in `source` (a group with everything in it) with
/// ids new to `target`, parents inside the block following, the top one
/// named `name`. Copies of the background are regular layers.
fn cloned_block(
    source: &Document,
    id: LayerId,
    target: &Document,
    name: &str,
) -> Option<Vec<Layer>> {
    let range = source.block(id)?;
    let mut block: Vec<Layer> = source.layers[range].to_vec();
    let ids: Vec<(LayerId, LayerId)> = block
        .iter()
        .map(|l| (l.id, target.new_layer_id()))
        .collect();
    let new_id = |old: LayerId| ids.iter().find(|(o, _)| *o == old).map(|(_, n)| *n);
    for layer in &mut block {
        layer.id = new_id(layer.id).expect("in the block");
        layer.parent = layer.parent.map(|p| new_id(p).unwrap_or(p));
        layer.is_background = false;
        // A copy stays linked to the original's links, as in Photoshop;
        // link numbers mean nothing in another document
        if !std::ptr::eq(source, target) {
            layer.link = None;
        }
    }
    block.last_mut().expect("not empty").name = name.to_string();
    Some(block)
}

/// The Duplicate Layer dialog's default "As" name for the active layer.
pub fn duplicate_name(doc: &Document) -> Option<String> {
    let source = &doc.layers[active_index(doc)?];
    Some(copy_name(doc, &source.name))
}

/// Layer > Duplicate Layer... into the same document, named `name`.
/// A group is copied with everything in it, right above the original.
pub fn duplicate_named(doc: &mut Document, name: &str) -> Option<LayerId> {
    let source = doc.layers[active_index(doc)?].id;
    let block = cloned_block(doc, source, doc, name)?;
    let at = doc.block(source)?.end;
    let id = block.last().expect("not empty").id;
    doc.layers.splice(at..at, block);
    doc.select_layer(id);
    doc.mark_dirty();
    Some(id)
}

/// Dragging the selected layers onto the Layers panel's "Create a new
/// layer" button: each one (a group with everything in it) is copied and
/// named "Name copy" (or "copy 2", ...). One layer's copy goes right above
/// it; several copies go together, in order, above the topmost selected
/// layer, as in Photoshop 2026. The copies end up selected, the active
/// layer's copy active. Returns the copies, bottom to top.
pub fn duplicate_selected(doc: &mut Document) -> Vec<LayerId> {
    let index = |doc: &Document, id| doc.layers.iter().position(|l| l.id == id);
    let mut roots = selected_roots(doc);
    roots.sort_by_key(|&id| index(doc, id));
    let Some(&top) = roots.last() else {
        return Vec::new();
    };
    if roots.len() == 1 {
        return duplicate(doc).into_iter().collect();
    }
    let active = doc.active_layer;
    let parent = doc.layer(top).and_then(|l| l.parent);
    let mut copies: Vec<Layer> = Vec::new();
    let mut pairs: Vec<(LayerId, LayerId)> = Vec::new();
    for &root in &roots {
        let Some(name) = doc.layer(root).map(|l| l.name.clone()) else {
            continue;
        };
        let name = copy_name(doc, &name);
        let Some(mut block) = cloned_block(doc, root, doc, &name) else {
            continue;
        };
        let copy = block.last_mut().expect("not empty");
        copy.parent = parent;
        pairs.push((root, copy.id));
        copies.extend(block);
    }
    let Some(at) = doc.block(top).map(|b| b.end) else {
        return Vec::new();
    };
    let ids: Vec<LayerId> = pairs.iter().map(|&(_, c)| c).collect();
    doc.layers.splice(at..at, copies);
    // The active layer's copy last, so it stays active
    let mut selection: Vec<LayerId> = pairs
        .iter()
        .filter(|&&(root, _)| Some(root) != active)
        .map(|&(_, c)| c)
        .collect();
    selection.extend(
        pairs
            .iter()
            .find(|&&(root, _)| Some(root) == active)
            .map(|&(_, c)| c),
    );
    doc.set_selected_layers(selection);
    doc.mark_dirty();
    ids
}

/// Layer > Duplicate Layer... into another document: a copy of `source`'s
/// active layer named `name`, at the same pixel position, above `target`'s
/// active layer. Pixels past `target`'s canvas stay on the layer; the mask
/// is extended (revealing) or cut to the new canvas. A copy of the
/// background is a regular layer.
pub fn duplicate_into(source: &Document, target: &mut Document, name: &str) -> Option<LayerId> {
    let mut block = cloned_block(
        source,
        source.layers[active_index(source)?].id,
        target,
        name,
    )?;
    let (w, h) = (target.width, target.height);
    for layer in &mut block {
        if let Some(image) = layer.image_mut() {
            *image = image.with_canvas(w, h, 0, 0, [0; 4]);
        }
        if let Some(mask) = &mut layer.mask {
            mask.image = mask.image.with_canvas(w, h, 0, 0, [255; 4]).clipped();
        }
    }
    let (at, parent) = target.insertion_point();
    let root = block.last_mut().expect("not empty");
    root.parent = parent;
    let id = root.id;
    target.layers.splice(at..at, block);
    target.select_layer(id);
    target.mark_dirty();
    Some(id)
}

/// Layer > Duplicate Layer... to a new document: one the size and
/// resolution of `source`, titled `title`, holding only a copy of the
/// active layer named `name`.
pub fn duplicate_to_new(source: &Document, title: &str, name: &str) -> Option<Document> {
    let mut doc =
        Document::new_with_background(title, source.width, source.height, crate::Color::WHITE);
    doc.resolution = source.resolution;
    doc.layers.clear();
    doc.active_layer = None;
    duplicate_into(source, &mut doc, name)?;
    Some(doc)
}

/// Layer > New > Layer Via Copy (Cmd+J). Without a selection the whole layer
/// is duplicated (named "Name copy", or "Layer N" for the background); with
/// one, the selected pixels go to a new "Layer N" in place. Either way the
/// new layer keeps the source's opacity and blend mode.
pub fn via_copy(doc: &mut Document) -> Result<LayerId, ClipError> {
    let index = active_index(doc).ok_or(ClipError::NoLayer)?;
    let source = doc.layers[index].clone();
    if doc.selection().is_none() {
        let name = if source.is_background {
            doc.next_layer_name()
        } else {
            copy_name(doc, &source.name)
        };
        return duplicate_named(doc, &name).ok_or(ClipError::NoLayer);
    }
    let clip = clipboard::copy(doc)?;
    Ok(paste_like(doc, &clip, &source))
}

/// Layer > New > Layer Via Cut (Shift+Cmd+J): moves the selected pixels to
/// a new "Layer N" in place; the background is left with `background`.
pub fn via_cut(doc: &mut Document, background: [u8; 3]) -> Result<LayerId, ClipError> {
    let index = active_index(doc).ok_or(ClipError::NoLayer)?;
    let source = doc.layers[index].clone();
    let clip = clipboard::cut(doc, background)?;
    Ok(paste_like(doc, &clip, &source))
}

/// Puts `clip` back at its origin on a new layer that takes `source`'s
/// opacity and blend mode, keeping the selection (unlike Paste).
fn paste_like(doc: &mut Document, clip: &clipboard::Clip, source: &Layer) -> LayerId {
    let selection = doc.selection().cloned();
    let id = clipboard::paste(doc, clip, clip.origin.unwrap_or((0, 0)));
    doc.set_selection(selection);
    let layer = doc.layer_mut(id).expect("just added");
    if !source.is_background {
        layer.opacity = source.opacity;
        layer.fill = source.fill;
        layer.blend_mode = source.blend_mode;
    }
    id
}

/// Layer > New > Layer from Background: the background becomes a regular,
/// unlocked "Layer 0" that can hold transparency and be moved.
pub fn layer_from_background(doc: &mut Document) -> bool {
    let Some(layer) = doc.layers.iter_mut().find(|l| l.is_background) else {
        return false;
    };
    layer.is_background = false;
    layer.name = "Layer 0".into();
    layer.set_locks(Locks::default());
    doc.mark_dirty();
    true
}

/// Where Layer > Arrange would move the active layer: the gap (as in
/// [`move_block`]) among the layers of its own group, if it would move.
/// Nothing moves the background, and no layer goes below it.
pub fn arrange_target(doc: &Document, arrange: Arrange) -> Option<usize> {
    let id = doc.active_layer?;
    let layer = doc.layer(id)?;
    if layer.is_background {
        return None;
    }
    // The layers of the same group, bottom to top (the background can't be
    // passed)
    let siblings: Vec<LayerId> = doc
        .layers
        .iter()
        .filter(|l| l.parent == layer.parent && !l.is_background)
        .map(|l| l.id)
        .collect();
    let k = siblings.iter().position(|&s| s == id)?;
    let gap = match arrange {
        Arrange::BringToFront if k + 1 < siblings.len() => doc.block(*siblings.last()?)?.end,
        Arrange::BringForward if k + 1 < siblings.len() => doc.block(siblings[k + 1])?.end,
        Arrange::SendBackward if k > 0 => doc.block(siblings[k - 1])?.start,
        Arrange::SendToBack if k > 0 => doc.block(siblings[0])?.start,
        _ => return None,
    };
    can_move_block(doc, id, gap).then_some(gap)
}

/// Layer > Arrange. Returns whether the layer moved.
pub fn arrange(doc: &mut Document, arrange: Arrange) -> bool {
    let (Some(id), Some(gap)) = (doc.active_layer, arrange_target(doc, arrange)) else {
        return false;
    };
    move_block(doc, id, gap)
}

/// Whether Layer > Arrange > Reverse can run: two or more selected layers
/// (with what's in them) in one group, none the background.
pub fn can_reverse(doc: &Document) -> bool {
    let roots = selected_roots(doc);
    let parent = roots
        .first()
        .and_then(|&id| doc.layer(id))
        .and_then(|l| l.parent);
    roots.len() >= 2
        && roots.iter().all(|&id| {
            doc.layer(id)
                .is_some_and(|l| l.parent == parent && !l.is_background)
        })
}

/// Layer > Arrange > Reverse: the selected layers swap places so their
/// order is reversed. Returns whether anything moved.
pub fn reverse_selected(doc: &mut Document) -> bool {
    if !can_reverse(doc) {
        return false;
    }
    let roots = selected_roots(doc);
    // Lift the blocks out and drop them back into the same slots, reversed
    let blocks: Vec<std::ops::Range<usize>> =
        roots.iter().filter_map(|&id| doc.block(id)).collect();
    let lifted: Vec<Vec<Layer>> = blocks
        .iter()
        .map(|r| doc.layers[r.clone()].to_vec())
        .collect();
    let mut out: Vec<Layer> = Vec::with_capacity(doc.layers.len());
    let (mut i, mut slot) = (0, 0);
    while i < doc.layers.len() {
        if let Some(b) = blocks.iter().find(|b| b.start == i) {
            out.extend(lifted[lifted.len() - 1 - slot].iter().cloned());
            slot += 1;
            i = b.end;
        } else {
            out.push(doc.layers[i].clone());
            i += 1;
        }
    }
    doc.layers = out;
    doc.mark_dirty();
    true
}

/// The selected layers that aren't inside another selected layer (the
/// blocks a group or move takes along), bottom to top.
fn selected_roots(doc: &Document) -> Vec<LayerId> {
    let selected = doc.selected_layers();
    selected
        .iter()
        .copied()
        .filter(|&id| {
            let mut parent = doc.layer(id).and_then(|l| l.parent);
            while let Some(p) = parent {
                if selected.contains(&p) {
                    return false;
                }
                parent = doc.layer(p).and_then(|l| l.parent);
            }
            true
        })
        .collect()
}

/// Whether Layer > Group Layers can run: layers are selected, none of them
/// the background.
pub fn can_group(doc: &Document) -> bool {
    let roots = selected_roots(doc);
    !roots.is_empty()
        && roots
            .iter()
            .all(|&id| doc.layer(id).is_some_and(|l| !l.is_background))
}

/// Layer > Group Layers (Cmd+G): the selected layers (with what's in them)
/// go into a new "Group N" where the topmost of them was, in its group.
/// The new group is selected.
pub fn group_selected(doc: &mut Document) -> Option<LayerId> {
    if !can_group(doc) {
        return None;
    }
    let roots = selected_roots(doc);
    let top = *roots.last().expect("not empty");
    let parent = doc.layer(top).and_then(|l| l.parent);
    let top_index = doc.layers.iter().position(|l| l.id == top).expect("exists");
    let mut taken: Vec<LayerId> = Vec::new();
    for &root in &roots {
        taken.push(root);
        taken.extend(doc.descendants(root));
    }
    let before = doc.layers[..top_index]
        .iter()
        .filter(|l| !taken.contains(&l.id))
        .count();
    let (mut moved, rest): (Vec<Layer>, Vec<Layer>) = std::mem::take(&mut doc.layers)
        .into_iter()
        .partition(|l| taken.contains(&l.id));
    doc.layers = rest;
    let id = doc.new_layer_id();
    for layer in &mut moved {
        if roots.contains(&layer.id) {
            layer.parent = Some(id);
        }
    }
    let mut group = Layer::group(id, doc.next_group_name());
    group.parent = parent;
    moved.push(group);
    doc.layers.splice(before..before, moved);
    doc.select_layer(id);
    doc.mark_dirty();
    Some(id)
}

/// Layer > Ungroup Layers (Shift+Cmd+G): the active group goes away and its
/// layers take its place in its own group; they become selected.
pub fn ungroup(doc: &mut Document) -> bool {
    let Some(group) = doc
        .active_layer
        .and_then(|id| doc.layer(id))
        .filter(|l| l.is_group())
    else {
        return false;
    };
    let (id, parent) = (group.id, group.parent);
    let children: Vec<LayerId> = doc
        .layers
        .iter()
        .filter(|l| l.parent == Some(id))
        .map(|l| l.id)
        .collect();
    doc.layers.retain(|l| l.id != id);
    for layer in &mut doc.layers {
        if layer.parent == Some(id) {
            layer.parent = parent;
        }
    }
    doc.set_selected_layers(children);
    doc.mark_dirty();
    true
}

/// Layer > New > Group: an empty "Group N" above the active layer (inside
/// it when it's an expanded group).
pub fn new_group(doc: &mut Document) -> LayerId {
    let id = doc.new_layer_id();
    let group = Layer::group(id, doc.next_group_name());
    doc.insert_above_active(group);
    doc.select_layer(id);
    id
}

pub fn can_move_block(doc: &Document, id: LayerId, gap: usize) -> bool {
    can_move_blocks(doc, &[id], gap)
}

/// Moves a layer, with everything in it, so it sits at `gap` (an index into
/// `layers` before the move: between `gap - 1` and `gap`). It joins the
/// innermost group the gap lies inside. Nothing goes below
/// the background, the background doesn't move, and a group can't go into
/// itself. Returns whether anything moved.
pub fn move_block(doc: &mut Document, id: LayerId, gap: usize) -> bool {
    move_blocks(doc, &[id], gap)
}

/// The layers of `ids` that aren't inside another one of them, bottom to
/// top.
fn block_roots(doc: &Document, ids: &[LayerId]) -> Vec<LayerId> {
    let mut roots: Vec<LayerId> = ids
        .iter()
        .copied()
        .filter(|&id| {
            let mut parent = doc.layer(id).and_then(|l| l.parent);
            while let Some(p) = parent {
                if ids.contains(&p) {
                    return false;
                }
                parent = doc.layer(p).and_then(|l| l.parent);
            }
            doc.layer(id).is_some()
        })
        .collect();
    roots.sort_by_key(|&id| doc.layers.iter().position(|l| l.id == id));
    roots.dedup();
    roots
}

/// Whether `move_blocks` would do something.
pub fn can_move_blocks(doc: &Document, ids: &[LayerId], gap: usize) -> bool {
    let roots = block_roots(doc, ids);
    if roots.is_empty()
        || gap > doc.layers.len()
        || (gap == 0 && doc.layers.first().is_some_and(|l| l.is_background))
    {
        return false;
    }
    let mut ranges = Vec::new();
    for &id in &roots {
        let Some(range) = doc.block(id) else {
            return false;
        };
        // The background stays, and a group can't go inside itself
        if doc.layers[range.end - 1].is_background || (range.start < gap && gap < range.end) {
            return false;
        }
        ranges.push(range);
    }
    // Moving already-adjacent blocks to their own edge changes nothing
    let together = ranges.windows(2).all(|w| w[0].end == w[1].start);
    let (start, end) = (ranges[0].start, ranges[ranges.len() - 1].end);
    !(together && (start..=end).contains(&gap))
}

/// Dragging several selected rows: moves the layers of `ids` (each with
/// everything in it) together, in their order, to `gap` as `move_block`
/// does with one.
pub fn move_blocks(doc: &mut Document, ids: &[LayerId], gap: usize) -> bool {
    if !can_move_blocks(doc, ids, gap) {
        return false;
    }
    let roots = block_roots(doc, ids);
    // The gap's group: the innermost group whose layers lie on both sides
    // of it (its block holds the layer below, and the layer above or the
    // group itself)
    let parent = doc
        .layers
        .iter()
        .filter(|g| g.is_group() && !roots.contains(&g.id))
        .filter_map(|g| doc.block(g.id).map(|b| (g.id, b)))
        .filter(|(_, b)| b.start < gap && gap < b.end)
        .min_by_key(|(_, b)| b.len())
        .map(|(g, _)| g);
    let mut moving: Vec<Layer> = Vec::new();
    let mut before_gap = 0;
    // Take the blocks out from the top down so earlier ranges stay valid
    let mut blocks: Vec<Vec<Layer>> = Vec::new();
    for &id in roots.iter().rev() {
        let range = doc.block(id).expect("checked");
        if range.end <= gap {
            before_gap += range.len();
        }
        blocks.push(doc.layers.drain(range).collect());
    }
    for block in blocks.into_iter().rev() {
        moving.extend(block);
    }
    let at = gap - before_gap;
    doc.layers.splice(at..at, moving);
    for id in roots {
        if let Some(layer) = doc.layer_mut(id) {
            layer.parent = parent;
        }
    }
    doc.mark_dirty();
    true
}

/// Layer > Rename Layer. Returns whether the name changed; empty names are
/// refused.
pub fn rename(doc: &mut Document, id: LayerId, name: &str) -> bool {
    let name = name.trim();
    let Some(layer) = doc.layer_mut(id) else {
        return false;
    };
    if name.is_empty() || layer.name == name {
        return false;
    }
    layer.name = name.into();
    doc.mark_dirty();
    true
}

/// The visible pixels of `layers` merged, as one image. Pixels outside the
/// canvas are merged and kept too, as in Photoshop (the background, which
/// ends at the canvas, adds none there).
fn merged(doc: &Document, layers: &[Layer]) -> TiledImage {
    let (cw, ch) = (doc.width as i64, doc.height as i64);
    let (x0, y0, x1, y1) = layers
        .iter()
        .filter_map(|l| l.image()?.content_bounds())
        .fold((0, 0, cw, ch), |a, b| {
            (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3))
        });
    if (x0, y0, x1, y1) == (0, 0, cw, ch) {
        return TiledImage::from_rgba8(doc.width, doc.height, &doc.composite_layers_rgba8(layers));
    }
    // Composite on a canvas grown to cover them all, then put the result
    // back in place
    let (w, h) = ((x1 - x0) as u32, (y1 - y0) as u32);
    let mut grown = Document::empty("", doc.width, doc.height);
    grown.layers = layers.to_vec();
    for layer in &mut grown.layers {
        layer.is_background = false;
    }
    grown.place_canvas(w, h, -x0, -y0, crate::Color::WHITE);
    let pixels = grown.composite_layers_rgba8(&grown.layers);
    TiledImage::from_rgba8(w, h, &pixels).with_canvas(doc.width, doc.height, x0, y0, [0; 4])
}

/// Gives `layer` the merged `image`; the background keeps only what is on
/// the canvas.
fn set_merged(layer: &mut Layer, image: TiledImage) {
    let image = if layer.is_background {
        image.clipped()
    } else {
        image
    };
    layer.kind = crate::layer::LayerKind::Raster(image);
}

/// Whether Layer > Merge Down can run: the active layer and the one below
/// it are both visible.
pub fn can_merge_down(doc: &Document) -> bool {
    active_index(doc).is_some_and(|i| i > 0 && doc.layers[i].visible && doc.layers[i - 1].visible)
}

/// Layer > Merge Down (Cmd+E): merges the active layer into the one below,
/// which keeps its name and properties and becomes active.
pub fn merge_down(doc: &mut Document) -> bool {
    if !can_merge_down(doc) {
        return false;
    }
    let i = active_index(doc).expect("checked");
    let mut lower = doc.layers[i - 1].clone();
    // The upper layer is merged into the lower layer's own pixels
    lower.opacity = 1.0;
    lower.fill = 1.0;
    lower.blend_mode = BlendMode::Normal;
    let image = merged(doc, &[lower, doc.layers[i].clone()]);
    doc.layers.remove(i);
    let target = &mut doc.layers[i - 1];
    set_merged(target, image);
    // Both layers' masks are in the merged pixels
    target.mask = None;
    doc.active_layer = Some(target.id);
    doc.mark_dirty();
    true
}

/// Whether Layer > Merge Group (Cmd+E on a group) can run: the active
/// layer is a group with layers in it.
pub fn can_merge_group(doc: &Document) -> bool {
    doc.active_layer
        .and_then(|id| doc.layer(id))
        .is_some_and(|l| l.is_group() && !doc.descendants(l.id).is_empty())
}

/// Layer > Merge Group: the group's layers merged into one pixel layer that
/// takes the group's place, name, opacity, mask and blend mode (Pass
/// Through becoming Normal).
pub fn merge_group(doc: &mut Document) -> bool {
    if !can_merge_group(doc) {
        return false;
    }
    let id = doc.active_layer.expect("checked");
    let range = doc.block(id).expect("exists");
    let inside: Vec<Layer> = doc.layers[range.start..range.end - 1].to_vec();
    let image = merged(doc, &inside);
    let group = doc.layers[range.end - 1].clone();
    let mut layer = Layer::raster(group.id, group.name, image);
    layer.parent = group.parent;
    layer.visible = group.visible;
    layer.opacity = group.opacity;
    layer.fill = group.fill;
    layer.mask = group.mask;
    layer.color = group.color;
    layer.blend_mode = match group.blend_mode {
        BlendMode::PassThrough => BlendMode::Normal,
        m => m,
    };
    doc.layers.splice(range, [layer]);
    doc.select_layer(id);
    doc.mark_dirty();
    true
}

/// Whether Layer > Merge Layers (Cmd+E with several layers selected) can
/// run: at least two of the selected layers are visible.
pub fn can_merge_selected(doc: &Document) -> bool {
    let selected = doc.selected_layers();
    selected.len() > 1
        && selected
            .iter()
            .filter(|&&id| doc.layer(id).is_some_and(|l| l.visible))
            .count()
            > 1
}

/// Layer > Merge Layers: the visible selected layers merged into one, in
/// the topmost one's place and with its name (or into the background when
/// it is among them). Hidden selected layers stay as they are.
pub fn merge_selected(doc: &mut Document) -> bool {
    if !can_merge_selected(doc) {
        return false;
    }
    let selected = doc.selected_layers();
    let merging: Vec<Layer> = doc
        .layers
        .iter()
        .filter(|l| l.visible && selected.contains(&l.id))
        .cloned()
        .collect();
    let target_id = merging
        .iter()
        .find(|l| l.is_background)
        .or(merging.last())
        .map(|l| l.id)
        .expect("two or more");
    let image = merged(doc, &merging);
    doc.layers
        .retain(|l| l.id == target_id || !merging.iter().any(|m| m.id == l.id));
    let layer = doc.layer_mut(target_id).expect("kept");
    set_merged(layer, image);
    layer.mask = None;
    layer.opacity = 1.0;
    layer.fill = 1.0;
    layer.blend_mode = BlendMode::Normal;
    doc.select_layer(target_id);
    doc.mark_dirty();
    true
}

/// Deletes every selected layer, unless that would leave none. The layer
/// below the lowest deleted one (else the lowest left) becomes active.
pub fn delete_selected(doc: &mut Document) -> bool {
    let selected = doc.selected_layers();
    if selected.is_empty()
        || selected.len() >= doc.layers.len()
        || selected.iter().any(|&id| doc.in_locked_group(id))
    {
        return false;
    }
    // A group goes with everything in it
    let mut doomed = selected.clone();
    for &id in &selected {
        doomed.extend(doc.descendants(id));
    }
    if doomed.len() >= doc.layers.len() {
        return false;
    }
    let lowest = doc
        .layers
        .iter()
        .position(|l| doomed.contains(&l.id))
        .expect("selected layers exist");
    doc.layers.retain(|l| !doomed.contains(&l.id));
    let next = doc.layers[lowest.saturating_sub(1).min(doc.layers.len() - 1)].id;
    doc.select_layer(next);
    doc.mark_dirty();
    true
}

/// Deleting with "Group Only": the selected groups go but their layers
/// stay (in the groups' places); other selected layers are deleted. The
/// layer that took the place of the topmost removed one becomes active.
pub fn delete_selected_keep_contents(doc: &mut Document) -> bool {
    let selected = doc.selected_layers();
    if selected.is_empty() {
        return false;
    }
    let (groups, others): (Vec<LayerId>, Vec<LayerId>) = selected
        .iter()
        .partition(|&&id| doc.layer(id).is_some_and(|l| l.is_group()));
    if others.len() + groups.len() >= doc.layers.len()
        && doc.layers.iter().all(|l| selected.contains(&l.id))
    {
        return false;
    }
    for &g in &groups {
        let parent = doc.layer(g).and_then(|l| l.parent);
        for layer in &mut doc.layers {
            if layer.parent == Some(g) {
                layer.parent = parent;
            }
        }
    }
    let lowest = doc
        .layers
        .iter()
        .position(|l| selected.contains(&l.id))
        .expect("selected layers exist");
    doc.layers.retain(|l| !selected.contains(&l.id));
    if doc.layers.is_empty() {
        return true;
    }
    let next = doc.layers[lowest.saturating_sub(1).min(doc.layers.len() - 1)].id;
    doc.select_layer(next);
    doc.mark_dirty();
    true
}

/// Whether deleting the selection would take a group's layers along (and
/// Photoshop asks first).
pub fn deleting_groups_with_contents(doc: &Document) -> Option<String> {
    doc.selected_layers()
        .into_iter()
        .filter_map(|id| doc.layer(id))
        .find(|l| l.is_group() && !doc.descendants(l.id).is_empty())
        .map(|l| l.name.clone())
}

/// Layer > Hide Layers / Show Layers: hides every selected layer, or shows
/// them all when all are hidden. Returns whether they are now visible.
pub fn toggle_selected_visibility(doc: &mut Document) -> Option<bool> {
    let selected = doc.selected_layers();
    if selected.is_empty() {
        return None;
    }
    let show = selected
        .iter()
        .all(|&id| doc.layer(id).is_some_and(|l| !l.visible));
    for id in selected {
        if let Some(l) = doc.layer_mut(id) {
            l.visible = show;
        }
    }
    doc.mark_dirty();
    Some(show)
}

/// The selected layers that can be locked (all but the background).
fn lockable(doc: &Document) -> Vec<LayerId> {
    doc.selected_layers()
        .into_iter()
        .filter(|&id| doc.layer(id).is_some_and(|l| !l.is_background))
        .collect()
}

/// Layer > Lock Layers...: the locks every selected layer has (a box is
/// ticked only when all of them have it), or `None` when only the
/// background (or nothing) is selected.
pub fn selected_locks(doc: &Document) -> Option<Locks> {
    let locks: Vec<Locks> = lockable(doc)
        .into_iter()
        .filter_map(|id| Some(doc.layer(id)?.locks()))
        .collect();
    if locks.is_empty() {
        return None;
    }
    let every = |f: fn(&Locks) -> bool| locks.iter().all(f);
    Some(Locks {
        transparency: every(|l| l.transparency),
        pixels: every(|l| l.pixels),
        position: every(|l| l.position),
        nesting: every(|l| l.nesting),
        all: every(|l| l.all),
    })
}

/// Sets the locks on every selected layer but the background. Returns
/// whether anything changed.
pub fn set_selected_locks(doc: &mut Document, locks: Locks) -> bool {
    let mut changed = false;
    for id in lockable(doc) {
        if let Some(l) = doc.layer_mut(id)
            && l.locks() != locks
        {
            l.set_locks(locks);
            changed = true;
        }
    }
    if changed {
        doc.mark_dirty();
    }
    changed
}

/// Cmd+/: Lock all on the selected layers, or, when every one of them is
/// already fully locked, no locks at all (the individual ones are cleared
/// too, unlike the Lock all button). Returns whether the layers are now
/// locked, or `None` when only the background is selected.
pub fn toggle_lock_all(doc: &mut Document) -> Option<bool> {
    let ids = lockable(doc);
    if ids.is_empty() {
        return None;
    }
    let lock = !ids
        .iter()
        .all(|&id| doc.layer(id).is_some_and(|l| l.lock_all));
    for id in ids {
        if let Some(l) = doc.layer_mut(id) {
            if lock {
                l.lock_all = true;
            } else {
                l.set_locks(Locks::default());
            }
        }
    }
    doc.mark_dirty();
    Some(lock)
}

/// Whether Layer > Merge Visible has anything to merge.
pub fn can_merge_visible(doc: &Document) -> bool {
    doc.layers.iter().filter(|l| l.visible).count() > 1
}

/// Layer > Merge Visible (Shift+Cmd+E): merges every visible layer into one.
/// The result takes the background's place if it is visible, otherwise the
/// active layer's (or the top visible layer's when the active one is
/// hidden). Hidden layers stay as they are.
pub fn merge_visible(doc: &mut Document) -> bool {
    if !can_merge_visible(doc) {
        return false;
    }
    let active = active_index(doc).filter(|&i| doc.layers[i].visible);
    let target = doc
        .layers
        .iter()
        .position(|l| l.visible && l.is_background)
        .or(active)
        .or_else(|| doc.layers.iter().rposition(|l| l.visible))
        .expect("there are visible layers");
    let image = merged(doc, &doc.layers);
    let target_id = doc.layers[target].id;
    doc.layers.retain(|l| !l.visible || l.id == target_id);
    let layer = doc.layer_mut(target_id).expect("kept");
    set_merged(layer, image);
    layer.mask = None;
    layer.opacity = 1.0;
    layer.fill = 1.0;
    layer.blend_mode = BlendMode::Normal;
    doc.active_layer = Some(target_id);
    doc.mark_dirty();
    true
}

/// Layer > Flatten Image: the visible layers merged onto white become a
/// single "Background" layer; hidden layers are discarded.
pub fn flatten(doc: &mut Document) {
    let mut pixels = doc.composite_rgba8();
    for px in pixels.as_chunks_mut::<4>().0 {
        let a = px[3] as u32;
        for c in &mut px[..3] {
            *c = ((*c as u32 * a + 255 * (255 - a) + 127) / 255) as u8;
        }
        px[3] = 255;
    }
    let image = TiledImage::from_rgba8(doc.width, doc.height, &pixels);
    let mut layer = Layer::raster(doc.new_layer_id(), "Background", image);
    layer.is_background = true;
    doc.active_layer = Some(layer.id);
    doc.layers = vec![layer];
    doc.mark_dirty();
}

/// Layer > Layer Mask > Reveal All / Hide All / Reveal Selection / Hide
/// Selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewMask {
    RevealAll,
    HideAll,
    RevealSelection,
    HideSelection,
}

/// Whether the active layer can get a mask: an existing, non-background
/// layer without one.
pub fn can_add_mask(doc: &Document) -> bool {
    doc.active_layer
        .and_then(|id| doc.layer(id))
        .is_some_and(|l| !l.is_background && l.mask.is_none())
}

/// Adds a mask to the active layer and makes it the edit target, as in
/// Photoshop. The selection kinds need a selection (and keep it).
pub fn add_mask(doc: &mut Document, kind: NewMask) -> bool {
    if !can_add_mask(doc) {
        return false;
    }
    let (w, h) = (doc.width, doc.height);
    let mask = match (kind, doc.selection()) {
        (NewMask::RevealAll, _) => LayerMask::filled(w, h, 255),
        (NewMask::HideAll, _) => LayerMask::filled(w, h, 0),
        (NewMask::RevealSelection, Some(s)) => LayerMask::from_values(w, h, |x, y| s.get(x, y)),
        (NewMask::HideSelection, Some(s)) => LayerMask::from_values(w, h, |x, y| 255 - s.get(x, y)),
        _ => return false,
    };
    let id = doc.active_layer.expect("checked");
    doc.layer_mut(id).expect("checked").mask = Some(mask);
    doc.mask_target = true;
    doc.mark_dirty();
    true
}

/// Layer > Layer Mask > Delete: removes the mask without applying it.
pub fn delete_mask(doc: &mut Document) -> bool {
    let Some(layer) = doc.active_layer.and_then(|id| doc.layer_mut(id)) else {
        return false;
    };
    if layer.mask.take().is_none() {
        return false;
    }
    doc.mask_target = false;
    doc.mark_dirty();
    true
}

/// Layer > Layer Mask > Apply: the mask is multiplied into the layer's
/// alpha and removed.
pub fn apply_mask(doc: &mut Document) -> bool {
    let (w, h) = (doc.width, doc.height);
    let Some(layer) = doc.active_layer.and_then(|id| doc.layer_mut(id)) else {
        return false;
    };
    let Some(mask) = layer.mask.take() else {
        return false;
    };
    let Some(image) = layer.image_mut() else {
        // A group's mask is just removed
        doc.mark_dirty();
        return true;
    };
    if mask.enabled {
        for y in 0..h {
            for x in 0..w {
                let m = mask.value(x, y) as u32;
                let px = image.pixel(x, y);
                if m < 255 && px[3] > 0 {
                    let a = (px[3] as u32 * m + 127) / 255;
                    image.set_pixel(x, y, [px[0], px[1], px[2], a as u8]);
                }
            }
        }
    }
    doc.mask_target = false;
    doc.mark_dirty();
    true
}

/// Layer > Layer Mask > Disable / Enable. Returns the new state, or `None`
/// without a mask.
pub fn toggle_mask(doc: &mut Document) -> Option<bool> {
    let mask = doc
        .active_layer
        .and_then(|id| doc.layer_mut(id))?
        .mask
        .as_mut()?;
    mask.enabled = !mask.enabled;
    let enabled = mask.enabled;
    doc.mark_dirty();
    Some(enabled)
}

/// Layer > Delete > Hidden Layers. Returns whether any were deleted; the
/// last layer is never deleted.
pub fn delete_hidden(doc: &mut Document) -> bool {
    let before = doc.layers.len();
    if doc.layers.iter().all(|l| !l.visible) {
        return false;
    }
    doc.layers.retain(|l| l.visible);
    if doc.layers.len() == before {
        return false;
    }
    if active_index(doc).is_none() {
        doc.active_layer = doc.layers.last().map(|l| l.id);
    }
    doc.mark_dirty();
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;
    use crate::selection::{Rect, Selection};

    /// 4×4 white background plus "Layer 1" with a red pixel at (1, 1).
    fn doc() -> Document {
        let mut doc = Document::new_with_background("t", 4, 4, Color::WHITE);
        let mut image = TiledImage::new(4, 4);
        image.set_pixel(1, 1, [255, 0, 0, 255]);
        doc.insert_above_active(Layer::raster(doc.new_layer_id(), "Layer 1", image));
        doc
    }

    fn names(doc: &Document) -> Vec<&str> {
        doc.layers.iter().map(|l| l.name.as_str()).collect()
    }

    fn pixel(doc: &Document, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * doc.width + x) * 4) as usize;
        doc.composite_rgba8()[i..i + 4].try_into().unwrap()
    }

    #[test]
    fn lock_layers_sets_every_selected_layer_but_the_background() {
        let mut doc = doc();
        let bg = doc.layers[0].id;
        let a = doc.layers[1].id;
        let b = doc.new_layer_id();
        doc.insert_above_active(Layer::raster(b, "Layer 2", TiledImage::new(4, 4)));
        let locks = |transparency, pixels, position| Locks {
            transparency,
            pixels,
            position,
            ..Locks::default()
        };
        // Only the background: nothing to lock
        doc.select_layer(bg);
        assert_eq!(selected_locks(&doc), None);
        assert!(!set_selected_locks(&mut doc, locks(true, true, true)));
        assert_eq!(toggle_lock_all(&mut doc), None);
        assert!(!doc.layers[0].lock_pixels);
        // A box is ticked only when every selected layer has that lock
        doc.layer_mut(a).unwrap().lock_position = true;
        doc.select_layer(a);
        doc.toggle_layer_selection(b);
        doc.toggle_layer_selection(bg);
        assert_eq!(selected_locks(&doc), Some(Locks::default()));
        assert!(set_selected_locks(&mut doc, locks(true, false, true)));
        assert_eq!(selected_locks(&doc), Some(locks(true, false, true)));
        for id in [a, b] {
            let l = doc.layer(id).unwrap();
            assert!(l.lock_transparency && !l.lock_pixels && l.lock_position);
        }
        assert!(!doc.layer(bg).unwrap().lock_transparency);
        // Setting the same locks again changes nothing
        assert!(!set_selected_locks(&mut doc, locks(true, false, true)));
    }

    #[test]
    fn lock_all_is_a_flag_of_its_own() {
        let mut doc = doc();
        let id = doc.layers[1].id;
        doc.layer_mut(id).unwrap().lock_position = true;
        // Cmd+/ locks everything...
        assert_eq!(toggle_lock_all(&mut doc), Some(true));
        let l = doc.layer(id).unwrap();
        assert!(l.lock_all && l.pixels_locked() && l.transparency_locked());
        assert!(!l.lock_pixels, "the individual locks stay as they were");
        // ...which stops painting and moving
        assert!(l.is_locked());
        // and a second Cmd+/ clears every lock, like Photoshop
        assert_eq!(toggle_lock_all(&mut doc), Some(false));
        assert_eq!(doc.layer(id).unwrap().locks(), Locks::default());
    }

    /// Dragging A and "Background copy" onto the new-layer button in
    /// Photoshop 2026 gave "A copy" and "Background copy 2" on top.
    #[test]
    fn duplicating_several_layers_puts_the_copies_on_top() {
        let mut doc = doc();
        let bg = doc.layers[0].id;
        doc.select_layer(bg);
        let bg_copy = duplicate(&mut doc).unwrap();
        let a = doc.layers[2].id;
        assert_eq!(names(&doc), ["Background", "Background copy", "Layer 1"]);
        doc.set_selected_layers(vec![bg_copy, a]);
        let copies = duplicate_selected(&mut doc);
        assert_eq!(
            names(&doc),
            [
                "Background",
                "Background copy",
                "Layer 1",
                "Background copy 2",
                "Layer 1 copy"
            ]
        );
        assert_eq!(copies.len(), 2);
        let mut selected = doc.selected_layers();
        selected.sort_by_key(|i| i.0);
        assert_eq!(selected, copies);
        assert_eq!(doc.active_layer, Some(copies[1]));
        // One layer: its copy goes right above it
        doc.select_layer(bg_copy);
        duplicate_selected(&mut doc);
        assert_eq!(doc.layers[2].name, "Background copy 3");
    }

    #[test]
    fn copies_of_copies_count_on() {
        let mut doc = doc();
        let named = |doc: &mut Document, name: &str| {
            let id = doc.new_layer_id();
            doc.layers
                .push(Layer::raster(id, name, TiledImage::new(4, 4)));
            id
        };
        // As Photoshop 2026 named them
        for (name, copy) in [
            ("B copy", "B copy 2"),
            ("E copy 7", "E copy 8"),
            ("Gcopy", "Gcopy copy"),
            ("H copy x", "H copy x copy"),
        ] {
            named(&mut doc, name);
            assert_eq!(copy_name(&doc, name), copy, "{name}");
        }
        // The next free number from there
        named(&mut doc, "B copy 2");
        assert_eq!(copy_name(&doc, "B copy"), "B copy 3");
    }

    #[test]
    fn moving_several_blocks_together() {
        let mut doc = doc();
        let ids: Vec<LayerId> = (0..3)
            .map(|k| {
                let id = doc.new_layer_id();
                doc.layers
                    .push(Layer::raster(id, format!("L{k}"), TiledImage::new(4, 4)));
                id
            })
            .collect();
        // Background, Layer 1, L0, L1, L2: L0 and L2 to the top
        assert!(move_blocks(&mut doc, &[ids[2], ids[0]], 5));
        assert_eq!(names(&doc), ["Background", "Layer 1", "L1", "L0", "L2"]);
        // ...and just above the background
        assert!(move_blocks(&mut doc, &[ids[0], ids[2]], 1));
        assert_eq!(names(&doc), ["Background", "L0", "L2", "Layer 1", "L1"]);
        // Where they already are: nothing
        assert!(!can_move_blocks(&doc, &[ids[0], ids[2]], 1));
        assert!(!can_move_blocks(&doc, &[ids[0], ids[2]], 3));
        // Never below the background, nor the background itself
        assert!(!can_move_blocks(&doc, &[ids[0]], 0));
        let bg = doc.layers[0].id;
        assert!(!can_move_blocks(&doc, &[bg, ids[1]], 5));
    }

    #[test]
    fn a_locked_group_locks_its_layers() {
        let mut doc = doc();
        let child = doc.layers[1].id;
        let group = doc.new_layer_id();
        let mut g = Layer::group(group, "G");
        g.lock_position = true;
        doc.layer_mut(child).unwrap().parent = Some(group);
        doc.layers.push(g);
        // Lock position on the group: the child can't move but can be
        // painted and deleted
        assert!(doc.position_locked(child) && !doc.pixels_locked(child));
        assert!(!doc.in_locked_group(child));
        // Lock all: nothing, not even deleting it
        doc.layer_mut(group).unwrap().lock_all = true;
        assert!(doc.pixels_locked(child) && doc.transparency_locked(child));
        assert!(doc.in_locked_group(child));
        doc.select_layer(child);
        assert!(!delete_selected(&mut doc));
        assert!(crate::move_tool::Move::begin(&doc, [0; 3]).is_err());
        let stroke = crate::paint::Stroke::begin(
            &doc,
            crate::paint::BrushTip {
                diameter: 2.0,
                hardness: 1.0,
                aliased: false,
                square: false,
                angle: 0.0,
                roundness: 1.0,
                spacing: 0.25,
            },
            crate::paint::StrokeKind::Paint([0, 0, 0]),
            1.0,
            1.0,
        );
        assert!(stroke.is_err());
    }

    #[test]
    fn duplicates_are_named_like_photoshop() {
        let mut doc = doc();
        duplicate(&mut doc);
        doc.active_layer = Some(doc.layers[1].id);
        duplicate(&mut doc);
        assert_eq!(
            names(&doc),
            ["Background", "Layer 1", "Layer 1 copy 2", "Layer 1 copy"]
        );
        // Cmd+J on the background makes "Layer N", not a second background
        doc.active_layer = Some(doc.layers[0].id);
        via_copy(&mut doc).unwrap();
        assert_eq!(doc.layers[1].name, "Layer 2");
        assert!(!doc.layers[1].is_background);
    }

    #[test]
    fn via_copy_and_cut_move_the_selection_in_place() {
        let mut doc = doc();
        doc.layers[1].opacity = 0.5;
        let s = Selection::rect(4, 4, Rect::new(0.0, 0.0, 2.0, 2.0));
        doc.set_selection(Some(s));
        let id = via_copy(&mut doc).unwrap();
        assert_eq!(doc.active_layer, Some(id));
        assert_eq!(doc.layers[2].name, "Layer 2");
        assert_eq!(doc.layers[2].opacity, 0.5);
        assert!(doc.selection().is_some());

        doc.active_layer = Some(doc.layers[1].id);
        via_cut(&mut doc, [0, 0, 0]).unwrap();
        let image = doc.layers[1].image().unwrap();
        assert_eq!(image.pixel(1, 1)[3], 0);
        assert_eq!(doc.layers[2].name, "Layer 3");
    }

    #[test]
    fn arrange_keeps_the_background_at_the_bottom() {
        let mut doc = doc();
        duplicate(&mut doc);
        assert!(arrange(&mut doc, Arrange::SendToBack));
        assert_eq!(names(&doc), ["Background", "Layer 1 copy", "Layer 1"]);
        assert!(!arrange(&mut doc, Arrange::SendBackward));
        assert!(arrange(&mut doc, Arrange::BringToFront));
        assert_eq!(names(&doc), ["Background", "Layer 1", "Layer 1 copy"]);
        doc.active_layer = Some(doc.layers[0].id);
        assert!(!arrange(&mut doc, Arrange::BringForward));
    }

    #[test]
    fn dragging_and_renaming() {
        let mut doc = doc();
        duplicate(&mut doc);
        let copy = doc.layers[2].id;
        assert!(move_block(&mut doc, copy, 1));
        assert_eq!(names(&doc), ["Background", "Layer 1 copy", "Layer 1"]);
        assert!(!move_block(&mut doc, copy, 0));
        let background = doc.layers[0].id;
        assert!(!move_block(&mut doc, background, 3));
        let id = doc.layers[2].id;
        assert!(rename(&mut doc, id, "  Sky "));
        assert_eq!(doc.layers[2].name, "Sky");
        assert!(!rename(&mut doc, id, ""));
        assert!(!rename(&mut doc, id, "Sky"));
    }

    #[test]
    fn masks_hide_reveal_and_apply() {
        let mut d = doc();
        // The background can't have a mask
        d.active_layer = Some(d.layers[0].id);
        assert!(!add_mask(&mut d, NewMask::RevealAll));
        d.active_layer = Some(d.layers[1].id);
        assert!(add_mask(&mut d, NewMask::HideAll));
        assert!(d.editing_mask());
        assert_eq!(pixel(&d, 1, 1), [255, 255, 255, 255]);
        // Disabled, the mask is ignored
        assert_eq!(toggle_mask(&mut d), Some(false));
        assert_eq!(pixel(&d, 1, 1), [255, 0, 0, 255]);
        toggle_mask(&mut d);
        // Painting white on the mask reveals the red pixel
        crate::fill::fill(&mut d, [255, 255, 255], Default::default()).unwrap();
        assert_eq!(pixel(&d, 1, 1), [255, 0, 0, 255]);
        let image = d.layers[1].image().unwrap();
        assert_eq!(image.pixel(0, 0)[3], 0, "the layer's pixels are untouched");
        // Applying a half-gray mask halves the alpha
        crate::fill::fill(&mut d, [128, 128, 128], Default::default()).unwrap();
        assert!(apply_mask(&mut d));
        assert!(d.layers[1].mask.is_none() && !d.editing_mask());
        let image = d.layers[1].image().unwrap();
        assert_eq!(image.pixel(1, 1)[3], 128);

        // From the selection
        let mut d = doc();
        d.set_selection(Some(Selection::rect(4, 4, Rect::new(0.0, 0.0, 1.0, 1.0))));
        assert!(add_mask(&mut d, NewMask::HideSelection));
        assert_eq!(d.layers[1].mask.as_ref().unwrap().value(0, 0), 0);
        assert_eq!(d.layers[1].mask.as_ref().unwrap().value(1, 1), 255);
        assert!(delete_mask(&mut d));
    }

    #[test]
    fn merge_down_keeps_the_lower_layer() {
        let mut doc = doc();
        doc.layers[1].opacity = 0.5;
        assert!(merge_down(&mut doc));
        assert_eq!(names(&doc), ["Background"]);
        assert!(doc.layers[0].is_background);
        assert_eq!(pixel(&doc, 1, 1), [255, 128, 128, 255]);
        assert!(!can_merge_down(&doc));
    }

    #[test]
    fn merge_visible_leaves_hidden_layers() {
        let mut doc = doc();
        duplicate(&mut doc);
        doc.layers[2].visible = false;
        assert!(merge_visible(&mut doc));
        assert_eq!(names(&doc), ["Background", "Layer 1 copy"]);
        assert_eq!(pixel(&doc, 1, 1), [255, 0, 0, 255]);
        assert_eq!(doc.active_layer, Some(doc.layers[0].id));
    }

    #[test]
    fn flatten_fills_transparency_with_white() {
        let mut doc = doc();
        layer_from_background(&mut doc);
        assert_eq!(doc.layers[0].name, "Layer 0");
        doc.layers[0].visible = false;
        flatten(&mut doc);
        assert_eq!(names(&doc), ["Background"]);
        assert_eq!(pixel(&doc, 0, 0), [255, 255, 255, 255]);
        assert_eq!(pixel(&doc, 1, 1), [255, 0, 0, 255]);
    }

    #[test]
    fn delete_hidden_keeps_visible_layers() {
        let mut doc = doc();
        doc.layers[1].visible = false;
        assert!(delete_hidden(&mut doc));
        assert_eq!(names(&doc), ["Background"]);
        assert_eq!(doc.active_layer, Some(doc.layers[0].id));
        assert!(!delete_hidden(&mut doc));
    }

    #[test]
    fn duplicate_layer_dialog_targets() {
        let mut a = Document::new_with_background("a", 4, 4, crate::Color::WHITE);
        assert_eq!(duplicate_name(&a).as_deref(), Some("Background copy"));
        let id = duplicate_named(&mut a, "Sky").unwrap();
        assert_eq!(a.layer(id).unwrap().name, "Sky");
        assert!(!a.layer(id).unwrap().is_background);

        // Into a smaller document: same position, the rest kept outside
        let mut b = Document::new_with_background("b", 2, 2, crate::Color::BLACK);
        let copied = duplicate_into(&a, &mut b, "From A").unwrap();
        assert_eq!(b.layers.len(), 2);
        assert_eq!(b.active_layer, Some(copied));
        let image = b.layer(copied).unwrap().image().unwrap();
        assert_eq!(image.content_bounds(), Some((0, 0, 4, 4)));

        // To a new document
        let c = duplicate_to_new(&a, "Untitled-1", "Sky").unwrap();
        assert_eq!((c.title.as_str(), c.width, c.height), ("Untitled-1", 4, 4));
        assert_eq!(c.layers.len(), 1);
        assert_eq!(c.layers[0].name, "Sky");
        assert_eq!(c.active_layer, Some(c.layers[0].id));
    }

    #[test]
    fn selected_layers_merge_delete_and_hide() {
        let mut doc = Document::new_with_background("t", 2, 1, crate::Color::WHITE);
        let add = |doc: &mut Document, name: &str, px: [u8; 4]| {
            let id = doc.new_layer_id();
            let mut image = TiledImage::new(2, 1);
            image.set_pixel(0, 0, px);
            doc.layers.push(Layer::raster(id, name, image));
            id
        };
        let a = add(&mut doc, "A", [255, 0, 0, 255]);
        let b = add(&mut doc, "B", [0, 0, 255, 128]);
        let c = add(&mut doc, "C", [0, 255, 0, 255]);
        doc.select_layer(a);
        doc.toggle_layer_selection(b);
        assert!(can_merge_selected(&doc));
        assert!(merge_selected(&mut doc));
        // B (the top one) holds A and B merged; C is untouched
        let names: Vec<&str> = doc.layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["Background", "B", "C"]);
        assert_eq!(doc.selected_layers(), [b]);
        let image = doc.layer(b).unwrap().image().unwrap();
        assert_eq!(image.pixel(0, 0)[3], 255);

        doc.toggle_layer_selection(c);
        assert_eq!(toggle_selected_visibility(&mut doc), Some(false));
        assert!(!doc.layer(b).unwrap().visible && !doc.layer(c).unwrap().visible);
        assert_eq!(toggle_selected_visibility(&mut doc), Some(true));

        assert!(delete_selected(&mut doc));
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.active_layer, Some(doc.layers[0].id));
        // The last layer can't go
        assert!(!delete_selected(&mut doc));
    }

    #[test]
    fn grouping_ungrouping_and_moving_blocks() {
        let mut doc = Document::new_with_background("t", 2, 2, crate::Color::WHITE);
        let ids: Vec<LayerId> = (0..3)
            .map(|i| {
                let id = doc.new_layer_id();
                doc.layers
                    .push(Layer::raster(id, format!("L{i}"), TiledImage::new(2, 2)));
                id
            })
            .collect();
        // Group L0 and L2: they go together where L2 was, L1 stays below
        doc.select_layer(ids[0]);
        doc.toggle_layer_selection(ids[2]);
        let g = group_selected(&mut doc).unwrap();
        let names = |doc: &Document| {
            doc.layers
                .iter()
                .map(|l| l.name.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(names(&doc), ["Background", "L1", "L0", "L2", "Group 1"]);
        assert_eq!(doc.layer(ids[0]).unwrap().parent, Some(g));
        assert_eq!(doc.layer(ids[1]).unwrap().parent, None);
        assert_eq!(doc.selected_layers(), [g]);
        // A new layer with the expanded group active goes on top inside it
        let mut l3 = Layer::raster(doc.new_layer_id(), "L3", TiledImage::new(2, 2));
        l3.parent = Some(ids[1]); // overwritten
        doc.insert_above_active(l3);
        assert_eq!(
            names(&doc),
            ["Background", "L1", "L0", "L2", "L3", "Group 1"]
        );
        assert_eq!(doc.layers[4].parent, Some(g));
        // Move L1 to the top of the group's layers (gap below the group)
        assert!(move_block(&mut doc, ids[1], 5));
        assert_eq!(
            names(&doc),
            ["Background", "L0", "L2", "L3", "L1", "Group 1"]
        );
        assert_eq!(doc.layer(ids[1]).unwrap().parent, Some(g));
        // The group is already right above the background: no move; not
        // below the background, and not into itself either
        assert!(!move_block(&mut doc, g, 1));
        assert!(!move_block(&mut doc, g, 0));
        assert!(!move_block(&mut doc, g, 3));
        // Ungroup: the layers stay, top-level now
        doc.select_layer(g);
        assert!(ungroup(&mut doc));
        assert!(
            doc.layers
                .iter()
                .all(|l| l.parent.is_none() && !l.is_group())
        );
        assert_eq!(doc.selected_layers().len(), 4);
        // Deleting a group deletes what's in it
        doc.select_all_layers();
        let g = group_selected(&mut doc).unwrap();
        assert!(delete_selected(&mut doc));
        assert_eq!(names(&doc), ["Background"]);
        assert!(doc.layer(g).is_none());
    }

    #[test]
    fn deleting_only_the_group_keeps_its_layers() {
        let mut doc = Document::new_with_background("t", 2, 2, crate::Color::WHITE);
        let a = doc.new_layer_id();
        doc.layers
            .push(Layer::raster(a, "A", TiledImage::new(2, 2)));
        doc.select_layer(a);
        let g = group_selected(&mut doc).unwrap();
        assert_eq!(
            deleting_groups_with_contents(&doc).as_deref(),
            Some("Group 1")
        );
        assert!(delete_selected_keep_contents(&mut doc));
        let names: Vec<&str> = doc.layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["Background", "A"]);
        assert_eq!(doc.layer(a).unwrap().parent, None);
        assert!(doc.layer(g).is_none());
        assert_eq!(deleting_groups_with_contents(&doc), None);
    }

    #[test]
    fn duplicating_a_group_copies_its_layers() {
        let mut doc = Document::new_with_background("t", 2, 2, crate::Color::WHITE);
        let a = doc.new_layer_id();
        doc.layers
            .push(Layer::raster(a, "A", TiledImage::new(2, 2)));
        doc.select_layer(a);
        let g = group_selected(&mut doc).unwrap();
        let copy = duplicate(&mut doc).unwrap();
        let names: Vec<&str> = doc.layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["Background", "A", "Group 1", "A", "Group 1 copy"]);
        assert_ne!(copy, g);
        assert_eq!(doc.layers[3].parent, Some(copy));
        assert_eq!(doc.layers[1].parent, Some(g));
        assert_eq!(doc.descendants(copy).len(), 1);
        // Into another document, with its layer
        let mut other = Document::new_with_background("o", 2, 2, crate::Color::WHITE);
        let into = duplicate_into(&doc, &mut other, "Copied").unwrap();
        assert_eq!(other.layers.len(), 3);
        assert_eq!(other.descendants(into).len(), 1);
    }

    #[test]
    fn arranging_stays_within_the_group_and_reverse() {
        let mut doc = Document::new_with_background("t", 2, 2, crate::Color::WHITE);
        let ids: Vec<LayerId> = (0..4)
            .map(|i| {
                let id = doc.new_layer_id();
                doc.layers
                    .push(Layer::raster(id, format!("L{i}"), TiledImage::new(2, 2)));
                id
            })
            .collect();
        // L1 and L2 in a group; L0 below it, L3 above it
        doc.select_layer(ids[1]);
        doc.toggle_layer_selection(ids[2]);
        let g = group_selected(&mut doc).unwrap();
        let names = |doc: &Document| {
            doc.layers
                .iter()
                .map(|l| l.name.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            names(&doc),
            ["Background", "L0", "L1", "L2", "Group 1", "L3"]
        );
        // Bring L1 to front: the top of its group, not of the document
        doc.select_layer(ids[1]);
        assert!(arrange(&mut doc, Arrange::BringToFront));
        assert_eq!(
            names(&doc),
            ["Background", "L0", "L2", "L1", "Group 1", "L3"]
        );
        assert_eq!(doc.layer(ids[1]).unwrap().parent, Some(g));
        assert!(arrange_target(&doc, Arrange::BringForward).is_none());
        // Send the group backward: past L0, with its layers
        doc.select_layer(g);
        assert!(arrange(&mut doc, Arrange::SendBackward));
        assert_eq!(
            names(&doc),
            ["Background", "L2", "L1", "Group 1", "L0", "L3"]
        );
        assert!(arrange_target(&doc, Arrange::SendBackward).is_none());
        // Bring L0 forward past L3 (top level)
        doc.select_layer(ids[0]);
        assert!(arrange(&mut doc, Arrange::BringForward));
        assert_eq!(
            names(&doc),
            ["Background", "L2", "L1", "Group 1", "L3", "L0"]
        );
        assert_eq!(doc.layer(ids[0]).unwrap().parent, None);
        // Reverse the group, L3 and L0
        doc.select_layer(g);
        doc.toggle_layer_selection(ids[3]);
        doc.toggle_layer_selection(ids[0]);
        assert!(reverse_selected(&mut doc));
        assert_eq!(
            names(&doc),
            ["Background", "L0", "L3", "L2", "L1", "Group 1"]
        );
        assert_eq!(doc.layer(ids[1]).unwrap().parent, Some(g));
    }

    #[test]
    fn merging_a_group() {
        let mut doc = Document::new_with_background("t", 1, 1, crate::Color::WHITE);
        let (a, b) = (doc.new_layer_id(), doc.new_layer_id());
        doc.layers.push(Layer::raster(
            a,
            "A",
            TiledImage::filled(1, 1, [255, 0, 0, 255]),
        ));
        let mut half = Layer::raster(b, "B", TiledImage::filled(1, 1, [0, 0, 255, 255]));
        half.opacity = 0.5;
        doc.layers.push(half);
        doc.select_layer(a);
        doc.toggle_layer_selection(b);
        let g = group_selected(&mut doc).unwrap();
        let before = doc.composite_rgba8();
        assert!(can_merge_group(&doc));
        assert!(merge_group(&mut doc));
        let names: Vec<&str> = doc.layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["Background", "Group 1"]);
        assert_eq!(doc.layers[1].id, g);
        assert!(!doc.layers[1].is_group());
        assert_eq!(doc.composite_rgba8(), before);
    }
}
