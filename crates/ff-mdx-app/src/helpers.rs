use std::path::{Path, PathBuf};

/// Classification of a file dropped onto the application window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DropAction {
    /// Open the dropped directory as the active folder.
    OpenFolder(PathBuf),
    /// A Markdown file was dropped; open its parent directory.
    OpenParentOf(PathBuf),
    /// The drop is not actionable (not a directory, not a `.md` file).
    Ignore,
}

/// Returns whether `relative_path` matches `filter`, matched case-insensitively
/// as a substring. An empty filter matches everything.
pub fn filter_matches(relative_path: &str, filter: &str) -> bool {
    filter.is_empty()
        || relative_path
            .to_lowercase()
            .contains(&filter.to_lowercase())
}

/// Classify a dropped path into the action the app should take.
pub fn classify_drop(path: &Path) -> DropAction {
    if path.is_dir() {
        DropAction::OpenFolder(path.to_path_buf())
    } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
        DropAction::OpenParentOf(path.to_path_buf())
    } else {
        DropAction::Ignore
    }
}

/// Returns whether a watcher-reported `changed` path should trigger a reload of
/// the currently `open` document: only when they are the same file.
pub fn should_reload(open: Option<&Path>, changed: &Path) -> bool {
    open == Some(changed)
}

/// Compute the display title for a file relative to `root`, with path
/// separators normalised to `/`. Falls back to the full path when `path` is not
/// under `root` or no root is set.
pub fn relative_title(path: &Path, root: &Option<PathBuf>) -> String {
    root.as_ref()
        .and_then(|r| path.strip_prefix(r).ok())
        .map(|p| p.display().to_string().replace('\\', "/"))
        .unwrap_or_else(|| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::{classify_drop, filter_matches, relative_title, DropAction};
    use std::path::PathBuf;

    #[test]
    fn empty_filter_matches_everything() {
        // Validates: Requirement 16.2
        assert!(filter_matches("docs/guide.md", ""));
    }

    #[test]
    fn filter_is_case_insensitive_substring() {
        // Validates: Requirement 16.2
        assert!(filter_matches("docs/Guide.md", "guide"));
        assert!(filter_matches("docs/Guide.md", "DOCS"));
        assert!(!filter_matches("docs/guide.md", "zzz"));
    }

    #[test]
    fn markdown_file_drop_opens_parent() {
        // Validates: Requirement 16.6 (drop-path classification)
        let p = PathBuf::from("some/missing/file.md");
        assert_eq!(classify_drop(&p), DropAction::OpenParentOf(p));
    }

    #[test]
    fn non_markdown_file_drop_is_ignored() {
        // Validates: Requirement 16.6
        assert_eq!(
            classify_drop(&PathBuf::from("file.txt")),
            DropAction::Ignore
        );
    }

    #[test]
    fn relative_title_strips_root_and_normalises_separators() {
        // Validates: Requirement 16.6 (title strip_prefix logic)
        let root = Some(PathBuf::from("/home/user/proj"));
        let path = PathBuf::from("/home/user/proj/docs/guide.md");
        assert_eq!(relative_title(&path, &root), "docs/guide.md");
    }

    #[test]
    fn reload_only_when_changed_file_is_the_open_file() {
        // Validates: Requirement 16.5, 15.4
        use super::should_reload;
        let open = PathBuf::from("/root/open.md");
        assert!(should_reload(Some(open.as_path()), &open));
        assert!(!should_reload(
            Some(open.as_path()),
            &PathBuf::from("/root/other.md")
        ));
        assert!(!should_reload(None, &open));
    }

    #[test]
    fn relative_title_falls_back_to_full_path_outside_root() {
        // Validates: Requirement 16.6
        let root = Some(PathBuf::from("/home/user/proj"));
        let path = PathBuf::from("/elsewhere/readme.md");
        assert_eq!(relative_title(&path, &root), path.display().to_string());
    }
}
