//! File › Open Recent: the files opened and saved lately, newest first,
//! kept across launches in a plain text file (one path per line).

use std::path::{Path, PathBuf};

/// Photoshop's default "Recent File List Contains" preference.
pub const LIMIT: usize = 20;

#[derive(Default)]
pub struct RecentFiles {
    files: Vec<PathBuf>,
    /// Where the list is kept; `None` keeps it in memory only (tests).
    store: Option<PathBuf>,
    /// Bumped on every change, for the menu to rebuild.
    revision: u64,
}

impl RecentFiles {
    /// The list kept at `store`, read now (an unreadable file reads as an
    /// empty list).
    pub fn load(store: PathBuf) -> Self {
        let files = std::fs::read_to_string(&store)
            .map(|s| {
                s.lines()
                    .filter(|l| !l.is_empty())
                    .map(PathBuf::from)
                    .take(LIMIT)
                    .collect()
            })
            .unwrap_or_default();
        Self {
            files,
            store: Some(store),
            revision: 1,
        }
    }

    /// Where the real app keeps the list: OpenPhoto's folder in the
    /// user's Application Support.
    pub fn default_store() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(
            Path::new(&home)
                .join("Library/Application Support/OpenPhoto")
                .join("recent-files.txt"),
        )
    }

    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// `path` was opened or saved: it moves to the top (once), the list
    /// keeping at most `LIMIT`.
    pub fn add(&mut self, path: &Path) {
        let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        self.files.retain(|p| *p != path);
        self.files.insert(0, path);
        self.files.truncate(LIMIT);
        self.changed();
    }

    /// Clear Recent File List.
    pub fn clear(&mut self) {
        self.files.clear();
        self.changed();
    }

    fn changed(&mut self) {
        self.revision += 1;
        if let Some(store) = &self.store {
            if let Some(dir) = store.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let text: String = self
                .files
                .iter()
                .map(|p| format!("{}\n", p.display()))
                .collect();
            if let Err(e) = std::fs::write(store, text) {
                log::warn!("could not keep the recent files: {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_first_without_repeats_and_kept_on_disk() {
        let dir = std::env::temp_dir().join(format!("openphoto-recent-{}", std::process::id()));
        let store = dir.join("recent-files.txt");
        let _ = std::fs::remove_file(&store);
        let mut r = RecentFiles::load(store.clone());
        assert!(r.files().is_empty());
        for i in 0..25 {
            r.add(Path::new(&format!("/tmp/none-{i}.png")));
        }
        assert_eq!(r.files().len(), LIMIT);
        assert_eq!(r.files()[0], Path::new("/tmp/none-24.png"));
        r.add(Path::new("/tmp/none-10.png"));
        assert_eq!(r.files()[0], Path::new("/tmp/none-10.png"));
        assert_eq!(
            r.files()
                .iter()
                .filter(|p| p.ends_with("none-10.png"))
                .count(),
            1
        );
        // Read back as written
        let again = RecentFiles::load(store.clone());
        assert_eq!(again.files(), r.files());
        r.clear();
        assert!(RecentFiles::load(store).files().is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }
}
