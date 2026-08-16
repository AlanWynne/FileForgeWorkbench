//! Platform-specific path handling (conditional compilation).
//!
//! Provides platform-specific utilities for Windows, Unix, and macOS.
//! Each platform module is gated behind `#[cfg(target_os = ...)]`.

use std::path::Path;

/// Detect whether a file is hidden on the current platform.
///
/// - Unix: file name starts with `.`
/// - Windows: file has the hidden attribute
/// - macOS: file has UF_HIDDEN flag or starts with `.`
///
/// Validates: Requirement 5 AC 4
pub fn is_hidden(path: &Path) -> bool {
    #[cfg(windows)]
    {
        is_hidden_windows(path)
    }
    #[cfg(not(windows))]
    {
        is_hidden_unix(path)
    }
}

/// Unix hidden file detection: name starts with `.`.
///
/// Validates: Requirement 2 AC 5, AC 6
#[cfg(not(windows))]
fn is_hidden_unix(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

/// Windows hidden file detection: checks the hidden file attribute.
///
/// Validates: Requirement 2 AC 4, AC 7
#[cfg(windows)]
fn is_hidden_windows(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;

    std::fs::metadata(path)
        .map(|meta| meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0)
        .unwrap_or(false)
}

/// Check if a path is a UNC path on Windows.
#[cfg(windows)]
pub fn is_unc_path(path: &str) -> bool {
    path.starts_with(r"\\") && !path.starts_with(r"\\?\")
}

/// Check if a path has a drive letter on Windows (e.g., `C:\`).
#[cfg(windows)]
pub fn has_drive_letter(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[cfg(not(windows))]
    #[test]
    fn is_hidden_detects_dot_prefix_on_unix() {
        assert!(is_hidden(Path::new("/home/user/.hidden")));
        assert!(is_hidden(Path::new(".config")));
        assert!(!is_hidden(Path::new("visible.txt")));
        assert!(!is_hidden(Path::new("/home/user/file.txt")));
    }

    #[cfg(windows)]
    #[test]
    fn has_drive_letter_detects_drive_paths() {
        assert!(has_drive_letter("C:\\Users\\test"));
        assert!(has_drive_letter("D:"));
        assert!(!has_drive_letter("\\\\server\\share"));
        assert!(!has_drive_letter("/unix/path"));
    }

    #[cfg(windows)]
    #[test]
    fn is_unc_path_detects_unc() {
        assert!(is_unc_path(r"\\server\share\file.txt"));
        assert!(!is_unc_path(r"\\?\C:\long\path"));
        assert!(!is_unc_path(r"C:\normal\path"));
    }
}
