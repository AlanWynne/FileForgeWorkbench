use std::path::{Path, PathBuf};

/// Directory names that are never descended into during a scan (build output,
/// VCS metadata, tool caches).
const EXCLUDED: &[&str] = &[
    "node_modules",
    ".git",
    ".vscode",
    "__pycache__",
    ".next",
    "dist",
    "build",
    "bin",
    "obj",
    ".vs",
    ".hg",
    ".svn",
    ".kiro",
    "target",
];

/// A single Markdown file discovered by a scan.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Path relative to the scan root, with separators normalised to `/`.
    pub relative_path: String,
    /// Absolute path of the file on disk.
    pub full_path: PathBuf,
}

/// Recursively scans a directory tree for Markdown files.
pub struct Scanner;

impl Scanner {
    /// Walk `root` recursively and return one [`FileEntry`] per `.md` file,
    /// skipping excluded directories, sorted lexicographically by
    /// `relative_path`. Returns an empty vector if `root` cannot be read.
    pub fn scan(root: &Path) -> Vec<FileEntry> {
        let mut entries = Vec::new();
        scan_dir(root, root, &mut entries);
        entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        entries
    }
}

fn scan_dir(dir: &Path, root: &Path, out: &mut Vec<FileEntry>) {
    let Ok(read) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if !EXCLUDED.contains(&name) {
                scan_dir(&path, root, out);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let relative = path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            out.push(FileEntry {
                relative_path: relative,
                full_path: path,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Scanner;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    fn write(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dir");
        }
        fs::write(path, contents).expect("write file");
    }

    fn relative_paths(root: &Path) -> Vec<String> {
        Scanner::scan(root)
            .into_iter()
            .map(|e| e.relative_path)
            .collect()
    }

    #[test]
    fn includes_only_md_files() {
        // Validates: Requirement 12.1, 12.2
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path();
        write(&root.join("a.md"), "# a");
        write(&root.join("b.markdown"), "b");
        write(&root.join("c.txt"), "c");
        write(&root.join("d"), "d");

        assert_eq!(relative_paths(root), vec!["a.md".to_string()]);
    }

    #[test]
    fn excluded_directories_are_not_descended() {
        // Validates: Requirement 12.3
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path();
        write(&root.join("keep.md"), "k");
        write(&root.join("node_modules").join("dep.md"), "x");
        write(&root.join(".git").join("hook.md"), "x");
        write(&root.join("target").join("out.md"), "x");

        assert_eq!(relative_paths(root), vec!["keep.md".to_string()]);
    }

    #[test]
    fn relative_path_uses_forward_slashes_and_full_path_is_absolute() {
        // Validates: Requirement 12.4
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path();
        write(&root.join("docs").join("guide.md"), "g");

        let entries = Scanner::scan(root);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].relative_path, "docs/guide.md");
        assert!(!entries[0].relative_path.contains('\\'));
        assert!(entries[0].full_path.is_absolute());
    }

    #[test]
    fn results_are_sorted_lexicographically() {
        // Validates: Requirement 12.5
        let dir = TempDir::new().expect("tempdir");
        let root = dir.path();
        write(&root.join("zebra.md"), "z");
        write(&root.join("alpha.md"), "a");
        write(&root.join("mid").join("mango.md"), "m");

        let paths = relative_paths(root);
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted);
    }

    #[test]
    fn unreadable_root_returns_empty_vector() {
        // Validates: Requirement 12.6
        let missing = PathBuf::from("this-path-does-not-exist-xyzzy");
        assert!(Scanner::scan(&missing).is_empty());
    }
}
