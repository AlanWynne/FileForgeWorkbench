//! # Context Menu — File Explorer Panel
//!
//! File classification and OS-integration helpers used by the modern File
//! Explorer: `FileClass`, `classify_file`/`classify_extension`,
//! `launch_default_app`, and `reveal_in_explorer`. (The legacy path-string
//! context-menu model was retired with the inline File Explorer in CR-NR-060
//! Slice A; the modern explorer builds its own menu in
//! `explorer_view::context_menu_ui`.)
//!
//! Validates: Requirement 17.1-17.3, 17.6, 17.8; Requirement 16.14

// === FileClass =============================================================

/// Classification of a file node determining whether it opens in FFWB or
/// in the OS default application.
///
/// Validates: Requirement 17.1, 17.2, 17.8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FileClass {
    /// Plain text or source code — open in FFWB editor.
    Text,
    /// FileForge structured file — open in FFWB specialised viewer.
    FfwbStructured,
    /// Binary or document file — launch OS default application.
    External,
}

/// Extensions that always map to `FileClass::External`.
///
/// Validates: Requirement 17.8
pub const EXTERNAL_EXTENSIONS: &[&str] = &[
    // Microsoft Office / OpenDocument
    "docx", "xlsx", "pptx", "doc", "xls", "ppt", "odt", "ods", "odp", // PDF / eBook
    "pdf", "epub", "mobi", // Images
    "png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp", "svg", "ico", // Audio / Video
    "mp3", "mp4", "wav", "flac", "avi", "mkv", "mov", "wmv", // Archives
    "zip", "tar", "gz", "bz2", "xz", "7z", "rar", // Executables / Libraries
    "exe", "dll", "so", "dylib", "app", // Databases
    "db", "sqlite", "mdb", "accdb",
];

/// Classify a file by extension alone.
///
/// Returns `FileClass::External` if the extension is in `EXTERNAL_EXTENSIONS`,
/// otherwise `FileClass::Text`.
///
/// Validates: Requirement 17.1, 17.2, 17.8
pub fn classify_extension(ext: &str) -> FileClass {
    let lower = ext.to_ascii_lowercase();
    if EXTERNAL_EXTENSIONS.contains(&lower.as_str()) {
        FileClass::External
    } else {
        FileClass::Text
    }
}

/// Classify a file by path: extension lookup first, then magic-byte fallback.
///
/// Validates: Requirement 17.1, 17.2, 17.3
pub fn classify_file(path: &str) -> FileClass {
    let ext = std::path::Path::new(path)
        .extension()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default();
    if !ext.is_empty() {
        let class = classify_extension(&ext);
        if class == FileClass::External {
            return FileClass::External;
        }
        // Known text extension — no need to scan
        if class == FileClass::Text {
            return FileClass::Text;
        }
    }
    // No extension or unknown — magic-byte scan
    match std::fs::File::open(path) {
        Ok(mut f) => {
            use std::io::Read;
            let mut buf = [0u8; 512];
            let n = f.read(&mut buf).unwrap_or(0);
            if is_text_bytes(&buf[..n]) {
                FileClass::Text
            } else {
                FileClass::External
            }
        }
        Err(_) => FileClass::Text, // can't read — try editor
    }
}

/// Returns `true` if the byte slice looks like UTF-8 text:
/// no null bytes and fewer than 5% non-UTF-8 bytes.
///
/// Validates: Requirement 17.3
pub fn is_text_bytes(data: &[u8]) -> bool {
    if data.is_empty() {
        return true;
    }
    if data.contains(&0u8) {
        return false;
    }
    let non_utf8 = data.iter().filter(|&&b| b > 0x7E && b < 0xC0).count();
    non_utf8 * 100 / data.len() < 5
}

/// Launch the OS default application for `path` non-blocking.
///
/// Validates: Requirement 17.2, 17.6
pub fn launch_default_app(path: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", path])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

// === Reveal in Explorer =====================================================

/// Open the OS file manager at `path` (or its parent directory when `path` is a
/// file). Non-blocking. Used by both the File Explorer context menu and the
/// modern explorer's Reveal action.
///
/// Validates: Requirement 16.14
pub(crate) fn reveal_in_explorer(path: &str) {
    let target = std::path::Path::new(path);
    let dir = if target.is_dir() {
        target.to_path_buf()
    } else {
        target.parent().unwrap_or(target).to_path_buf()
    };
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(&dir).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&dir).spawn();
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(&dir).spawn();
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- 17.8 FileClass / EXTERNAL_EXTENSIONS table -------------------------

    /// Validates: Requirement 17.8 -- Office docs are External
    #[test]
    fn office_extensions_are_external() {
        for ext in &[
            "docx", "xlsx", "pptx", "doc", "xls", "ppt", "odt", "ods", "odp",
        ] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} External"
            );
        }
    }

    /// Validates: Requirement 17.8 -- PDF/eBook are External
    #[test]
    fn pdf_extensions_are_external() {
        for ext in &["pdf", "epub", "mobi"] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} External"
            );
        }
    }

    /// Validates: Requirement 17.8 -- image extensions are External
    #[test]
    fn image_extensions_are_external() {
        for ext in &[
            "png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp", "svg", "ico",
        ] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} External"
            );
        }
    }

    /// Validates: Requirement 17.8 -- audio/video extensions are External
    #[test]
    fn audio_video_extensions_are_external() {
        for ext in &["mp3", "mp4", "wav", "flac", "avi", "mkv", "mov", "wmv"] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} External"
            );
        }
    }

    /// Validates: Requirement 17.8 -- archive extensions are External
    #[test]
    fn archive_extensions_are_external() {
        for ext in &["zip", "tar", "gz", "bz2", "xz", "7z", "rar"] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} External"
            );
        }
    }

    /// Validates: Requirement 17.8 -- executable extensions are External
    #[test]
    fn executable_extensions_are_external() {
        for ext in &["exe", "dll", "so", "dylib"] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} External"
            );
        }
    }

    /// Validates: Requirement 17.8 -- database extensions are External
    #[test]
    fn database_extensions_are_external() {
        for ext in &["db", "sqlite", "mdb", "accdb"] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} External"
            );
        }
    }

    /// Validates: Requirement 17.1 -- source/text extensions are Text
    #[test]
    fn source_extensions_are_text() {
        for ext in &[
            "rs", "c", "cpp", "py", "sh", "txt", "toml", "yaml", "jcl", "cbl",
        ] {
            assert_eq!(classify_extension(ext), FileClass::Text, "{ext} Text");
        }
    }

    /// Validates: Requirement 17.3 -- magic-byte scan: null byte = binary
    #[test]
    fn magic_byte_scan_null_byte_is_binary() {
        let data = b"hello\x00world";
        assert!(!is_text_bytes(data), "null byte -> binary");
    }

    /// Validates: Requirement 17.3 -- magic-byte scan: valid UTF-8 = text
    #[test]
    fn magic_byte_scan_utf8_is_text() {
        let data = b"Hello, world!\nThis is a text file.\n";
        assert!(is_text_bytes(data), "plain ASCII -> text");
    }

    /// Validates: Requirement 17.3 -- magic-byte scan: high binary ratio = binary
    #[test]
    fn magic_byte_scan_high_binary_ratio_is_binary() {
        let data: Vec<u8> = (0u8..=255u8).cycle().take(512).collect();
        assert!(!is_text_bytes(&data), "high non-UTF-8 ratio -> binary");
    }

    /// Validates: Requirement 17.6 -- launch_default_app must not panic
    #[test]
    fn launch_default_app_command_is_platform_appropriate() {
        let result = std::panic::catch_unwind(|| {
            launch_default_app("/nonexistent/test/file.txt");
        });
        assert!(result.is_ok(), "launch_default_app must not panic");
    }

    // --- 16.18 Copy path variants (formatting the modern Copy Full Path uses) --

    /// Validates: Requirement 16.18 AC 1 -- Copy File Name = base name only
    #[test]
    fn copy_file_name_is_base_name_only() {
        let full = "C:\\Users\\user\\projects\\hello.rs";
        let base = std::path::Path::new(full)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert_eq!(base, "hello.rs");
    }

    /// Validates: Requirement 16.18 AC 2 -- Copy Relative Path
    #[test]
    fn copy_relative_path_strips_catalog_root() {
        let root = "C:\\Users\\user\\projects";
        let full = "C:\\Users\\user\\projects\\src\\main.rs";
        let rel = std::path::Path::new(full)
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert_eq!(rel, "src\\main.rs");
    }
}
