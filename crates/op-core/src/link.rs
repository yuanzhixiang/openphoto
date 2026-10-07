//! Layer > Link Layers / Unlink Layers / Select Linked Layers: linked layers
//! move together, as in Photoshop. A layer's `link` number names its link
//! set; a number no other layer has links nothing, so deleting or
//! unlinking layers never leaves stale links to tidy up.

use crate::document::Document;
use crate::layer::LayerId;

/// The other layers linked to `id`.
/// A layer whose link is disabled moves on its own, and its partners move
/// without it.
pub fn linked_with(doc: &Document, id: LayerId) -> Vec<LayerId> {
    let Some(link) = doc
        .layer(id)
        .filter(|l| !l.link_disabled)
        .and_then(|l| l.link)
    else {
        return Vec::new();
    };
    doc.layers
        .iter()
        .filter(|l| l.id != id && l.link == Some(link) && !l.link_disabled)
        .map(|l| l.id)
        .collect()
}

/// The layers in `id`'s link set, disabled ones included (the Layers panel
/// shows their icons).
pub fn link_set(doc: &Document, id: LayerId) -> Vec<LayerId> {
    let Some(link) = doc.layer(id).and_then(|l| l.link) else {
        return Vec::new();
    };
    let set: Vec<LayerId> = doc
        .layers
        .iter()
        .filter(|l| l.link == Some(link))
        .map(|l| l.id)
        .collect();
    if set.len() > 1 { set } else { Vec::new() }
}

/// Shift-click on a layer's link icon: disables its link, or enables it
/// again. Returns whether the layer is linked at all.
pub fn toggle_disabled(doc: &mut Document, id: LayerId) -> bool {
    if link_set(doc, id).is_empty() {
        return false;
    }
    if let Some(l) = doc.layer_mut(id) {
        l.link_disabled = !l.link_disabled;
    }
    true
}

pub fn is_linked(doc: &Document, id: LayerId) -> bool {
    !linked_with(doc, id).is_empty()
}

/// The selected layers and every layer linked to one of them, bottom to
/// top. The Layers panel shows the link icon on these when any is linked.
pub fn with_linked(doc: &Document) -> Vec<LayerId> {
    let selected = doc.selected_layers();
    doc.layers
        .iter()
        .map(|l| l.id)
        .filter(|&id| {
            selected.contains(&id) || selected.iter().any(|&s| linked_with(doc, s).contains(&id))
        })
        .collect()
}

/// Whether the menu item reads "Unlink Layers": every selected layer is
/// linked (to anything). Otherwise it reads "Link Layers".
pub fn can_unlink(doc: &Document) -> bool {
    let selected = doc.selected_layers();
    !selected.is_empty() && selected.iter().all(|&id| is_linked(doc, id))
}

/// Link Layers needs two or more selected layers, not all already linked.
pub fn can_link(doc: &Document) -> bool {
    doc.selected_layers().len() >= 2 && !can_unlink(doc)
}

/// Links the selected layers, together with the layers already linked to
/// them, into one set. Returns whether anything changed.
pub fn link_selected(doc: &mut Document) -> bool {
    if !can_link(doc) {
        return false;
    }
    let members = with_linked(doc);
    let fresh = doc.layers.iter().filter_map(|l| l.link).max().unwrap_or(0) + 1;
    for id in members {
        if let Some(l) = doc.layer_mut(id) {
            l.link = Some(fresh);
        }
    }
    doc.mark_dirty();
    true
}

/// Takes the selected layers out of their link sets; the layers left in a
/// set stay linked to each other.
pub fn unlink_selected(doc: &mut Document) -> bool {
    if !can_unlink(doc) {
        return false;
    }
    for id in doc.selected_layers() {
        if let Some(l) = doc.layer_mut(id) {
            l.link = None;
        }
    }
    doc.mark_dirty();
    true
}

/// The Layers panel's link button and Layer > Link/Unlink Layers: unlinks
/// when every selected layer is linked, else links. Returns the history
/// name, or `None` when neither applies.
pub fn toggle(doc: &mut Document) -> Option<&'static str> {
    if unlink_selected(doc) {
        Some("Unlink Layers")
    } else if link_selected(doc) {
        Some("Link Layers")
    } else {
        None
    }
}

/// Whether Select Linked Layers would add anything to the selection.
pub fn can_select_linked(doc: &Document) -> bool {
    with_linked(doc).len() > doc.selected_layers().len()
}

/// Layer > Select Linked Layers: adds the layers linked to the selected
/// ones to the selection; the active layer stays active.
pub fn select_linked(doc: &mut Document) -> bool {
    if !can_select_linked(doc) {
        return false;
    }
    let active = doc.active_layer;
    let mut ids: Vec<LayerId> = with_linked(doc)
        .into_iter()
        .filter(|&id| Some(id) != active)
        .collect();
    ids.extend(active);
    doc.set_selected_layers(ids);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Layer, TiledImage};

    /// Background plus A, B, C, D (bottom to top).
    fn doc() -> (Document, [LayerId; 5]) {
        let mut doc = Document::new_with_background("t", 4, 4, Color::WHITE);
        let bg = doc.layers[0].id;
        let mut ids = [bg; 5];
        for (k, name) in ["A", "B", "C", "D"].into_iter().enumerate() {
            let id = doc.new_layer_id();
            doc.layers
                .push(Layer::raster(id, name, TiledImage::new(4, 4)));
            ids[k + 1] = id;
        }
        (doc, ids)
    }

    fn select(doc: &mut Document, ids: &[LayerId]) {
        doc.set_selected_layers(ids.to_vec());
    }

    fn linked(doc: &Document, id: LayerId) -> Vec<LayerId> {
        let mut v = linked_with(doc, id);
        v.sort_by_key(|i| i.0);
        v
    }

    /// The sequence checked in Photoshop 2026.
    #[test]
    fn matches_photoshop() {
        let (mut doc, [bg, a, b, c, d]) = doc();
        // One layer: nothing to link
        select(&mut doc, &[b]);
        assert!(!can_link(&doc) && !can_unlink(&doc));
        // Link A and C
        select(&mut doc, &[a, c]);
        assert!(can_link(&doc));
        assert_eq!(toggle(&mut doc), Some("Link Layers"));
        assert_eq!(linked(&doc, a), [c]);
        // Now both are linked: the item reads Unlink, and B alone shows none
        assert!(can_unlink(&doc));
        select(&mut doc, &[b]);
        assert_eq!(with_linked(&doc), [b]);
        select(&mut doc, &[a]);
        assert_eq!(with_linked(&doc), [a, c]);
        assert!(can_unlink(&doc) && can_select_linked(&doc));
        // Linking B with A brings C along
        select(&mut doc, &[b, a]);
        assert_eq!(toggle(&mut doc), Some("Link Layers"));
        assert_eq!(linked(&doc, b), [a, c]);
        // Unlinking C alone leaves A and B linked
        select(&mut doc, &[c]);
        assert_eq!(toggle(&mut doc), Some("Unlink Layers"));
        assert!(!is_linked(&doc, c));
        assert_eq!(linked(&doc, a), [b]);
        // C and the background
        select(&mut doc, &[c, bg]);
        assert_eq!(toggle(&mut doc), Some("Link Layers"));
        assert_eq!(linked(&doc, bg), [c]);
        // A linked and D not: Link Layers
        select(&mut doc, &[d, a]);
        assert!(can_link(&doc) && !can_unlink(&doc));
        // Every selected layer linked (to different sets): Unlink Layers
        select(&mut doc, &[a, c]);
        assert!(can_unlink(&doc));
        // Select Linked Layers: only when it adds something
        select(&mut doc, &[a, b]);
        assert!(!can_select_linked(&doc));
        select(&mut doc, &[c, a]);
        assert!(select_linked(&mut doc));
        assert_eq!(doc.active_layer, Some(a));
        let mut sel = doc.selected_layers();
        sel.sort_by_key(|i| i.0);
        let mut want = vec![bg, a, b, c];
        want.sort_by_key(|i| i.0);
        assert_eq!(sel, want);
    }

    #[test]
    fn deleting_a_linked_layer_leaves_no_stale_link() {
        let (mut doc, [_, a, b, _, _]) = doc();
        select(&mut doc, &[a, b]);
        link_selected(&mut doc);
        doc.layers.retain(|l| l.id != b);
        assert!(!is_linked(&doc, a));
    }

    #[test]
    fn a_disabled_link_moves_on_its_own() {
        let (mut doc, [_, a, b, c, _]) = doc();
        select(&mut doc, &[a, b, c]);
        assert!(link_selected(&mut doc));
        assert!(toggle_disabled(&mut doc, b));
        // B is out of the set for now; A and C stay linked
        assert!(linked(&doc, b).is_empty());
        assert_eq!(linked(&doc, a), [c]);
        assert_eq!(link_set(&doc, a).len(), 3);
        // Enabled again
        assert!(toggle_disabled(&mut doc, b));
        assert_eq!(linked(&doc, a), [b, c]);
        // An unlinked layer has nothing to disable
        let (mut lone, [_, x, ..]) = super::tests::doc();
        assert!(!toggle_disabled(&mut lone, x));
    }
}
