//! Undo history, modeled on Photoshop's History panel: a linear list of named
//! states, any of which can be jumped to. Each state holds a full document
//! snapshot; tiles are shared between snapshots, so only edited tiles cost memory.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::document::{Document, Snapshot};

fn next_state_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Photoshop's default "History States" preference.
pub const DEFAULT_LIMIT: usize = 50;

pub struct HistoryState {
    pub name: String,
    /// Unique for the life of the process; tells states apart even after
    /// older ones are dropped.
    pub id: u64,
    snapshot: Snapshot,
}

pub struct History {
    states: Vec<HistoryState>,
    current: usize,
    limit: usize,
    /// Where Toggle Last State jumps back to, while a toggle is in effect.
    toggle_origin: Option<usize>,
}

impl History {
    /// Starts a history whose first state (e.g. "Open" or "New") is the
    /// document as it is now.
    pub fn new(doc: &Document, name: impl Into<String>) -> Self {
        Self {
            states: vec![HistoryState {
                name: name.into(),
                id: next_state_id(),
                snapshot: doc.snapshot(),
            }],
            current: 0,
            limit: DEFAULT_LIMIT,
            toggle_origin: None,
        }
    }

    /// Records the document's current state after an edit. Any undone states
    /// are discarded, as in Photoshop.
    pub fn record(&mut self, doc: &Document, name: impl Into<String>) {
        self.toggle_origin = None;
        self.states.truncate(self.current + 1);
        self.states.push(HistoryState {
            name: name.into(),
            id: next_state_id(),
            snapshot: doc.snapshot(),
        });
        // The first state is kept so the document can always be reverted;
        // snapshots are complete, so dropping a middle state is safe.
        if self.states.len() > self.limit {
            self.states.remove(1);
        }
        self.current = self.states.len() - 1;
    }

    pub fn states(&self) -> &[HistoryState] {
        &self.states
    }

    /// The document as it was in state `index`.
    pub fn snapshot(&self, index: usize) -> Option<&Snapshot> {
        self.states.get(index).map(|s| &s.snapshot)
    }

    /// Id of the current state; compared with the id saved to tell whether
    /// the document has unsaved changes.
    pub fn current_id(&self) -> u64 {
        self.states[self.current].id
    }

    pub fn current(&self) -> usize {
        self.current
    }

    /// Where the state with `id` is now, if it is still kept.
    pub fn index_of(&self, id: u64) -> Option<usize> {
        self.states.iter().position(|s| s.id == id)
    }

    pub fn can_undo(&self) -> bool {
        self.current > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current + 1 < self.states.len()
    }

    /// Name of the step that undo would revert.
    pub fn undo_name(&self) -> Option<&str> {
        self.can_undo()
            .then(|| self.states[self.current].name.as_str())
    }

    /// Name of the step that redo would reapply.
    pub fn redo_name(&self) -> Option<&str> {
        self.can_redo()
            .then(|| self.states[self.current + 1].name.as_str())
    }

    pub fn undo(&mut self, doc: &mut Document) -> bool {
        self.can_undo() && self.jump(self.current - 1, doc)
    }

    pub fn redo(&mut self, doc: &mut Document) -> bool {
        self.can_redo() && self.jump(self.current + 1, doc)
    }

    /// Deletes state `index` and every state after it, then moves to the state
    /// before it (the trash button in the History panel). The first state can't
    /// be deleted.
    pub fn delete_from(&mut self, index: usize, doc: &mut Document) -> bool {
        if index == 0 || index >= self.states.len() {
            return false;
        }
        doc.restore(&self.states[index - 1].snapshot);
        self.states.truncate(index);
        self.current = index - 1;
        self.toggle_origin = None;
        true
    }

    /// Edit > Toggle Last State: switches between the current state and the
    /// one before it. Toggling again returns to where it started.
    pub fn toggle_last_state(&mut self, doc: &mut Document) -> bool {
        match self.toggle_origin.take() {
            Some(origin) if origin < self.states.len() => self.go_to(origin, doc),
            _ if self.current > 0 => {
                let origin = self.current;
                let moved = self.go_to(origin - 1, doc);
                self.toggle_origin = Some(origin);
                moved
            }
            _ => false,
        }
    }

    /// Restores the document to state `index` (clicking a row in the History panel).
    pub fn jump(&mut self, index: usize, doc: &mut Document) -> bool {
        self.toggle_origin = None;
        self.go_to(index, doc)
    }

    fn go_to(&mut self, index: usize, doc: &mut Document) -> bool {
        if index >= self.states.len() || index == self.current {
            return false;
        }
        doc.restore(&self.states[index].snapshot);
        self.current = index;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Anchor, Color};

    #[test]
    fn undo_redo_and_truncate() {
        let mut doc = Document::new_with_background("t", 2, 2, Color::WHITE);
        let mut h = History::new(&doc, "New");

        doc.resize_canvas(4, 4, Anchor::CENTER, Color::BLACK);
        h.record(&doc, "Canvas Size");
        assert_eq!(h.undo_name(), Some("Canvas Size"));

        assert!(h.undo(&mut doc));
        assert_eq!((doc.width, doc.height), (2, 2));
        assert!(!h.undo(&mut doc));

        assert!(h.redo(&mut doc));
        assert_eq!((doc.width, doc.height), (4, 4));

        h.undo(&mut doc);
        doc.resize_canvas(3, 3, Anchor::CENTER, Color::BLACK);
        h.record(&doc, "Canvas Size");
        assert!(!h.can_redo());
        assert_eq!(h.states().len(), 2);
    }

    #[test]
    fn delete_from_drops_later_states() {
        let mut doc = Document::new_with_background("t", 1, 1, Color::WHITE);
        let mut h = History::new(&doc, "Open");
        for w in 2..5 {
            doc.resize_canvas(w, 1, Anchor::CENTER, Color::BLACK);
            h.record(&doc, "Canvas Size");
        }
        assert!(h.delete_from(2, &mut doc));
        assert_eq!(h.states().len(), 2);
        assert_eq!(h.current(), 1);
        assert_eq!(doc.width, 2);
        assert!(!h.delete_from(0, &mut doc));
    }

    #[test]
    fn toggle_last_state_round_trips() {
        let mut doc = Document::new_with_background("t", 1, 1, Color::WHITE);
        let mut h = History::new(&doc, "Open");
        doc.resize_canvas(2, 1, Anchor::CENTER, Color::BLACK);
        h.record(&doc, "Canvas Size");
        doc.resize_canvas(3, 1, Anchor::CENTER, Color::BLACK);
        h.record(&doc, "Canvas Size");

        assert!(h.toggle_last_state(&mut doc));
        assert_eq!((h.current(), doc.width), (1, 2));
        assert!(h.toggle_last_state(&mut doc));
        assert_eq!((h.current(), doc.width), (2, 3));

        // A regular undo ends the toggle
        h.toggle_last_state(&mut doc);
        h.undo(&mut doc);
        assert_eq!(h.current(), 0);
        assert!(!h.toggle_last_state(&mut doc));
    }

    #[test]
    fn limit_keeps_first_state() {
        let mut doc = Document::new_with_background("t", 1, 1, Color::WHITE);
        let mut h = History::new(&doc, "Open");
        for i in 0..DEFAULT_LIMIT + 5 {
            doc.resize_canvas(i as u32 + 2, 1, Anchor::CENTER, Color::BLACK);
            h.record(&doc, format!("Step {i}"));
        }
        assert_eq!(h.states().len(), DEFAULT_LIMIT);
        assert_eq!(h.states()[0].name, "Open");
        assert!(h.jump(0, &mut doc));
        assert_eq!(doc.width, 1);
    }
}
