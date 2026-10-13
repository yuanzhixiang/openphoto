//! Document annotations are persisted metadata; they never enter the composite.
use crate::Document;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub author: String,
    pub text: String,
    pub color: [u8; 3],
}
pub fn create_note(doc: &mut Document, x: u32, y: u32, author: &str) -> Option<u32> {
    if x >= doc.width || y >= doc.height {
        return None;
    }
    let id = doc
        .notes
        .iter()
        .map(|n| n.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)?;
    doc.notes.push(Note {
        id,
        x,
        y,
        author: author.into(),
        text: String::new(),
        color: [255; 3],
    });
    doc.mark_dirty();
    Some(id)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CountGroup {
    pub id: u32,
    pub name: String,
    pub visible: bool,
    pub color: [u8; 3],
    pub marker: u32,
    pub label: u32,
    pub points: Vec<(u32, u32)>,
}
pub fn create_count_group(doc: &mut Document) -> Option<u32> {
    let id = doc
        .count_groups
        .iter()
        .map(|g| g.id)
        .max()
        .unwrap_or(0)
        .checked_add(1)?;
    doc.count_groups.push(CountGroup {
        id,
        name: format!("Count Group {id}"),
        visible: true,
        color: [0x9c, 0xf2, 0xf4],
        marker: 2,
        label: 8,
        points: Vec::new(),
    });
    doc.mark_dirty();
    Some(id)
}
