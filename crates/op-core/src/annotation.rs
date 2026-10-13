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
