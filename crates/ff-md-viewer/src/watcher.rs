use std::path::{Path, PathBuf};
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::MdViewerError;

/// The quiet window used to coalesce a burst of change events into a single
/// emitted path. This is the viewer's own live-reload debounce and is distinct
/// from the framework's 300ms refresh debounce.
pub const DEBOUNCE: Duration = Duration::from_millis(400);

/// Returns whether `path` denotes a Markdown file (extension exactly `md`).
///
/// This is the pure predicate the watcher uses to filter raw filesystem events
/// so that only Markdown changes are forwarded.
pub fn is_markdown_path(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("md")
}

/// Coalesce a burst of changed paths into the single most-recent path.
///
/// Given the paths observed within one quiet window (in arrival order), the
/// debounced result is the last path. Returns `None` for an empty burst.
pub fn coalesce_burst(burst: &[PathBuf]) -> Option<PathBuf> {
    burst.last().cloned()
}

/// Watches a root directory recursively for Markdown file changes and emits the
/// path of each changed Markdown file on a receiver channel after a debounce
/// quiet window.
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<PathBuf>,
}

impl FileWatcher {
    /// Watch `root` for `.md` changes. Emits debounced paths (~400 ms window).
    ///
    /// # Errors
    ///
    /// Returns [`MdViewerError::Watch`] if the platform watcher cannot be
    /// created or cannot begin watching `root`.
    pub fn new(root: &Path) -> Result<Self, MdViewerError> {
        let (raw_tx, raw_rx): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();
        let (debounced_tx, debounced_rx): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        for path in event.paths {
                            if is_markdown_path(&path) {
                                let _ = raw_tx.send(path);
                            }
                        }
                    }
                }
            },
            Config::default(),
        )
        .map_err(|source| MdViewerError::Watch {
            path: root.to_path_buf(),
            source,
        })?;

        watcher
            .watch(root, RecursiveMode::Recursive)
            .map_err(|source| MdViewerError::Watch {
                path: root.to_path_buf(),
                source,
            })?;

        std::thread::spawn(move || {
            while let Ok(path) = raw_rx.recv() {
                let mut burst = vec![path];
                while let Ok(p) = raw_rx.recv_timeout(DEBOUNCE) {
                    burst.push(p);
                }
                if let Some(last) = coalesce_burst(&burst) {
                    let _ = debounced_tx.send(last);
                }
            }
        });

        Ok(Self {
            _watcher: watcher,
            rx: debounced_rx,
        })
    }

    /// Returns the receiver of debounced changed-Markdown paths.
    pub fn changes(&self) -> &Receiver<PathBuf> {
        &self.rx
    }
}

#[cfg(test)]
mod tests {
    use super::{coalesce_burst, is_markdown_path};
    use std::path::PathBuf;

    #[test]
    fn markdown_path_predicate_accepts_only_md_extension() {
        // Validates: Requirement 15.2
        assert!(is_markdown_path(&PathBuf::from("notes.md")));
        assert!(is_markdown_path(&PathBuf::from("sub/dir/readme.md")));
        assert!(!is_markdown_path(&PathBuf::from("notes.markdown")));
        assert!(!is_markdown_path(&PathBuf::from("notes.txt")));
        assert!(!is_markdown_path(&PathBuf::from("notes")));
        assert!(!is_markdown_path(&PathBuf::from("README.MD")));
    }

    #[test]
    fn burst_coalesces_to_most_recent_path() {
        // Validates: Requirement 15.1, 15.3
        let burst = vec![
            PathBuf::from("a.md"),
            PathBuf::from("b.md"),
            PathBuf::from("c.md"),
        ];
        assert_eq!(coalesce_burst(&burst), Some(PathBuf::from("c.md")));
    }

    #[test]
    fn empty_burst_coalesces_to_none() {
        // Validates: Requirement 15.3
        assert_eq!(coalesce_burst(&[]), None);
    }
}
