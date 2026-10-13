//! User slices are non-destructive, undoable document-space export regions.
use crate::Document;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slice {
    pub id: u64,
    pub name: String,
    pub bounds: (u32, u32, u32, u32),
    pub url: String,
    pub target: String,
    pub alt: String,
}

pub fn create(doc: &mut Document, bounds: (u32, u32, u32, u32)) -> Option<u64> {
    let (x0, y0, x1, y1) = bounds;
    let bounds = (
        x0.min(doc.width),
        y0.min(doc.height),
        x1.min(doc.width),
        y1.min(doc.height),
    );
    if bounds.0 >= bounds.2 || bounds.1 >= bounds.3 {
        return None;
    }
    let id = doc
        .slices
        .iter()
        .map(|s| s.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)?;
    if id > u32::MAX as u64 {
        return None;
    }
    doc.slices.push(Slice {
        id,
        name: format!("{}_{}", doc.title, id),
        bounds,
        url: String::new(),
        target: String::new(),
        alt: String::new(),
    });
    doc.mark_dirty();
    Some(id)
}

/// Pixel export is deliberately independent of a filesystem or image encoder.
pub fn rgba8(doc: &Document, id: u64) -> Option<(u32, u32, Vec<u8>)> {
    let s = doc.slices.iter().find(|s| s.id == id)?;
    let (x0, y0, x1, y1) = s.bounds;
    let (x1, y1) = (x1.min(doc.width), y1.min(doc.height));
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let source = doc.composite_rgba8();
    let mut out = Vec::with_capacity(((x1 - x0) * (y1 - y0) * 4) as usize);
    for y in y0..y1 {
        out.extend_from_slice(
            &source[((y * doc.width + x0) * 4) as usize..((y * doc.width + x1) * 4) as usize],
        );
    }
    Some((x1 - x0, y1 - y0, out))
}

/// Replace user slices with the grid defined by in-canvas ruler guides.
pub fn from_guides(doc: &mut Document) -> bool {
    if doc.guides.is_empty() {
        return false;
    }
    let mut xs = vec![0, doc.width];
    let mut ys = vec![0, doc.height];
    for g in &doc.guides {
        let bound = if g.vertical { doc.width } else { doc.height };
        if g.position.is_finite() && g.position > 0.0 && g.position < bound as f32 {
            let at = g.position.round() as u32;
            if g.vertical {
                xs.push(at);
            } else {
                ys.push(at);
            }
        }
    }
    xs.sort_unstable();
    xs.dedup();
    ys.sort_unstable();
    ys.dedup();
    let bounds: Vec<_> = ys
        .windows(2)
        .flat_map(|y| xs.windows(2).map(move |x| (x[0], y[0], x[1], y[1])))
        .collect();
    if bounds
        .iter()
        .copied()
        .eq(doc.slices.iter().map(|s| s.bounds))
    {
        return false;
    }
    doc.slices.clear();
    for b in bounds {
        create(doc, b);
    }
    true
}
