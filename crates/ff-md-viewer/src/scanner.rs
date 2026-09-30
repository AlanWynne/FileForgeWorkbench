use std::path::{Path, PathBuf};

const EXCLUDED: &[&str] = &[
    "node_modules", ".git", ".vscode", "__pycache__",
    ".next", "dist", "build", "bin", "obj", ".vs", ".hg", ".svn",
    ".kiro", "target",
];

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub relative_path: String,
    pub full_path: PathBuf,
}

pub struct Scanner;

impl Scanner {
    pub fn scan(root: &Path) -> Vec<FileEntry> {
        let mut entries = Vec::new();
        scan_dir(root, root, &mut entries);
        entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        entries
    }
}

fn scan_dir(dir: &Path, root: &Path, out: &mut Vec<FileEntry>) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
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
            out.push(FileEntry { relative_path: relative, full_path: path });
        }
    }
}
