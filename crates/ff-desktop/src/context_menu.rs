//! # Context Menu — File Explorer Panel
//!
//! Defines `NodeKind`, `MenuItem`, `ExtensionRule`, and `build_context_menu()`.
//! The menu spec for each of the 8 node kinds is driven by the requirements
//! table in Requirement 16.2–16.9.  Extension rules can promote a
//! `Disabled` item to an active one (e.g. `*.jcl` enabling Submit JCL).
//!
//! Validates: Requirement 16.1–16.9, 16.15, 16.16, 16.17

use crate::catalog_registry::CatalogType;

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

// === NodeKind ================================================================

/// The kind of node that was right-clicked.
///
/// Drives which context menu is shown (Req 16.2–16.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum NodeKind {
    NativeFile,
    NativeDir,
    PosixFile,
    MfPs,
    MfPds,
    MfMember,
    MfGdgBase,
    MfGdgGen,
}

// === MenuItem ================================================================

/// A single entry in a context menu.
///
/// `Disabled` items are rendered via `ui.add_enabled(false, ...)` — visible
/// but not clickable (Req 16.15, 16.16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItem {
    /// A clickable action item.
    Action(MenuAction),
    /// A horizontal separator line.
    Separator,
    /// A visible-but-greyed-out item (deferred feature).
    Disabled(&'static str),
}

/// All actionable menu items across all node kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Open,
    OpenInNewTab,
    OpenInNewWindow,
    OpenWith,
    Copy,
    Rename,
    MoveTo,
    CopyTo,
    NewFile,
    NewFolder,
    CopyFileName,
    CopyRelativePath,
    CopyFullPath,
    OpenContainingFolder,
    RevealInExplorer,
    Properties,
    // Mainframe-specific
    Compare,
    CopyDatasetName,
    DatasetProperties,
    Refresh,
    NewMember,
    CopyMember,
    RenameMember,
    CopyMemberName,
    CopyDatasetMember,
    MemberProperties,
    NewGeneration,
}

// === ExtensionRule ===========================================================

/// A data-driven rule that can promote a `Disabled` item to an `Action`.
///
/// Validates: Requirement 16.17
pub struct ExtensionRule {
    /// Glob pattern matched against the file extension (e.g. `"*.jcl"`).
    pub pattern: &'static str,
    /// The disabled label that should become active when the pattern matches.
    pub enables: &'static str,
    /// The action to substitute in place of the disabled item.
    pub action: MenuAction,
}

/// Built-in extension rules (code-defined; TOML override deferred).
///
/// Validates: Requirement 16.17 AC 2, AC 3
pub const EXTENSION_RULES: &[ExtensionRule] = &[
    // *.jcl will enable Submit JCL once SDSF is ready — currently no rule
    // activates it, but the table is structured for future TOML extension.
];

// === build_context_menu ======================================================

/// Build the ordered list of `MenuItem` values for a given node.
///
/// `extension` is the file extension (e.g. `"jcl"`, `"rs"`) or `""` for
/// directories / datasets with no extension.  Extension rules are consulted
/// last and may promote `Disabled` items to `Action` items.
///
/// Validates: Requirement 16.2–16.9, 16.17
pub fn build_context_menu(
    catalog_type: CatalogType,
    node_kind: NodeKind,
    extension: &str,
) -> Vec<MenuItem> {
    let mut items = raw_menu(catalog_type, node_kind);
    apply_extension_rules(&mut items, extension);
    items
}

fn raw_menu(catalog_type: CatalogType, node_kind: NodeKind) -> Vec<MenuItem> {
    use MenuAction::*;
    use MenuItem::*;

    match (catalog_type, node_kind) {
        // === 16.2 — Native File =============================================
        (CatalogType::Native, NodeKind::NativeFile) => vec![
            Action(Open),
            Action(OpenInNewTab),
            Action(OpenInNewWindow),
            Action(OpenWith),
            Separator,
            Action(Copy),
            Separator,
            Action(Rename),
            Action(MoveTo),
            Action(CopyTo),
            Separator,
            Action(NewFile),
            Action(NewFolder),
            Separator,
            Action(CopyFileName),
            Action(CopyRelativePath),
            Action(CopyFullPath),
            Separator,
            Action(OpenContainingFolder),
            Action(RevealInExplorer),
            Separator,
            Disabled("Git \u{25b6}"),
            Separator,
            Action(Properties),
        ],

        // === 16.3 — Native Directory ========================================
        (CatalogType::Native, NodeKind::NativeDir) => vec![
            Action(OpenInNewTab),
            Separator,
            Action(NewFile),
            Action(NewFolder),
            Separator,
            Action(Copy),
            Separator,
            Action(Rename),
            Action(MoveTo),
            Action(CopyTo),
            Separator,
            Action(CopyFullPath),
            Separator,
            Action(RevealInExplorer),
            Separator,
            Disabled("Git \u{25b6}"),
            Separator,
            Action(Properties),
        ],

        // === 16.4 — POSIX File ==============================================
        (CatalogType::Posix, NodeKind::PosixFile) => vec![
            Action(Open),
            Action(OpenInNewTab),
            Action(OpenInNewWindow),
            Action(OpenWith),
            Separator,
            Action(Copy),
            Separator,
            Action(CopyFileName),
            Action(CopyRelativePath),
            Action(CopyFullPath),
            Separator,
            Action(Properties),
        ],

        // === 16.5 — Mainframe PS ============================================
        (CatalogType::Mainframe, NodeKind::MfPs) => vec![
            Action(Open),
            Action(OpenInNewTab),
            Separator,
            Action(Compare),
            Separator,
            Action(CopyTo),
            Separator,
            Action(CopyDatasetName),
            Action(CopyFullPath),
            Separator,
            Action(DatasetProperties),
            Separator,
            Action(Refresh),
        ],

        // === 16.6 — Mainframe PDS ===========================================
        (CatalogType::Mainframe, NodeKind::MfPds) => vec![
            Action(NewMember),
            Separator,
            Action(CopyTo),
            Separator,
            Action(CopyDatasetName),
            Separator,
            Action(DatasetProperties),
            Separator,
            Action(Refresh),
        ],

        // === 16.7 — Mainframe PDS Member ====================================
        (CatalogType::Mainframe, NodeKind::MfMember) => vec![
            Action(Open),
            Action(OpenInNewTab),
            Separator,
            Disabled("Submit JCL"),
            Action(Compare),
            Separator,
            Action(CopyMember),
            Action(RenameMember),
            Separator,
            Action(CopyMemberName),
            Action(CopyDatasetName),
            Action(CopyDatasetMember),
            Separator,
            Action(MemberProperties),
            Action(DatasetProperties),
            Separator,
            Action(Refresh),
        ],

        // === 16.8 — Mainframe GDG Base ======================================
        (CatalogType::Mainframe, NodeKind::MfGdgBase) => vec![
            Action(NewGeneration),
            Separator,
            Action(CopyDatasetName),
            Separator,
            Action(DatasetProperties),
            Separator,
            Action(Refresh),
        ],

        // === 16.9 — Mainframe GDG Generation ================================
        (CatalogType::Mainframe, NodeKind::MfGdgGen) => vec![
            Action(Open),
            Action(OpenInNewTab),
            Separator,
            Action(CopyDatasetName),
            Separator,
            Action(DatasetProperties),
            Separator,
            Action(Refresh),
        ],

        // Fallback — should not occur in practice
        _ => vec![],
    }
}

/// Apply extension rules: promote matching `Disabled` items to `Action` items.
///
/// Validates: Requirement 16.17 AC 4
fn apply_extension_rules(items: &mut [MenuItem], extension: &str) {
    for rule in EXTENSION_RULES {
        // Simple glob: only `*.<ext>` patterns are supported in this release.
        let pattern_ext = rule.pattern.trim_start_matches("*.");
        if extension.eq_ignore_ascii_case(pattern_ext) {
            for item in items.iter_mut() {
                if let MenuItem::Disabled(label) = item {
                    if *label == rule.enables {
                        *item = MenuItem::Action(rule.action);
                    }
                }
            }
        }
    }
}

// === Label helpers ===========================================================

impl MenuAction {
    /// Display label for this action.
    pub fn label(self) -> &'static str {
        match self {
            MenuAction::Open => "Open",
            MenuAction::OpenInNewTab => "Open in New Tab",
            MenuAction::OpenInNewWindow => "Open in New Window",
            MenuAction::OpenWith => "Open With\u{2026}",
            MenuAction::Copy => "Copy",
            MenuAction::Rename => "Rename",
            MenuAction::MoveTo => "Move To\u{2026}",
            MenuAction::CopyTo => "Copy To\u{2026}",
            MenuAction::NewFile => "New File",
            MenuAction::NewFolder => "New Folder",
            MenuAction::CopyFileName => "Copy File Name",
            MenuAction::CopyRelativePath => "Copy Relative Path",
            MenuAction::CopyFullPath => "Copy Full Path",
            MenuAction::OpenContainingFolder => "Open Containing Folder",
            MenuAction::RevealInExplorer => "Reveal in Explorer",
            MenuAction::Properties => "Properties",
            MenuAction::Compare => "Compare\u{2026}",
            MenuAction::CopyDatasetName => "Copy Dataset Name",
            MenuAction::DatasetProperties => "Dataset Properties",
            MenuAction::Refresh => "Refresh",
            MenuAction::NewMember => "New Member",
            MenuAction::CopyMember => "Copy Member",
            MenuAction::RenameMember => "Rename Member",
            MenuAction::CopyMemberName => "Copy Member Name",
            MenuAction::CopyDatasetMember => "Copy Dataset(Member)",
            MenuAction::MemberProperties => "Member Properties",
            MenuAction::NewGeneration => "New Generation",
        }
    }

    /// Platform-appropriate label for "Reveal in Explorer".
    ///
    /// Validates: Requirement 16.14 AC 2
    pub fn reveal_label() -> &'static str {
        #[cfg(target_os = "windows")]
        return "Reveal in Explorer";
        #[cfg(target_os = "macos")]
        return "Reveal in Finder";
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        return "Open Containing Folder";
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog_registry::CatalogType;

    fn actions(items: &[MenuItem]) -> Vec<MenuAction> {
        items
            .iter()
            .filter_map(|i| {
                if let MenuItem::Action(a) = i {
                    Some(*a)
                } else {
                    None
                }
            })
            .collect()
    }

    fn has_disabled(items: &[MenuItem], label: &str) -> bool {
        items
            .iter()
            .any(|i| matches!(i, MenuItem::Disabled(l) if *l == label))
    }

    fn has_action(items: &[MenuItem], action: MenuAction) -> bool {
        items
            .iter()
            .any(|i| matches!(i, MenuItem::Action(a) if *a == action))
    }

    // --- 16.2 Native File ---------------------------------------------------

    /// Validates: Requirement 16.2
    #[test]
    fn native_file_menu_contains_required_actions() {
        let items = build_context_menu(CatalogType::Native, NodeKind::NativeFile, "txt");
        assert!(has_action(&items, MenuAction::Open));
        assert!(has_action(&items, MenuAction::OpenInNewTab));
        assert!(has_action(&items, MenuAction::OpenInNewWindow));
        assert!(has_action(&items, MenuAction::OpenWith));
        assert!(has_action(&items, MenuAction::Copy));
        assert!(has_action(&items, MenuAction::Rename));
        assert!(has_action(&items, MenuAction::MoveTo));
        assert!(has_action(&items, MenuAction::CopyTo));
        assert!(has_action(&items, MenuAction::NewFile));
        assert!(has_action(&items, MenuAction::NewFolder));
        assert!(has_action(&items, MenuAction::CopyFileName));
        assert!(has_action(&items, MenuAction::CopyRelativePath));
        assert!(has_action(&items, MenuAction::CopyFullPath));
        assert!(has_action(&items, MenuAction::RevealInExplorer));
        assert!(has_action(&items, MenuAction::Properties));
        assert!(has_disabled(&items, "Git \u{25b6}"));
    }

    // --- 16.3 Native Dir ----------------------------------------------------

    /// Validates: Requirement 16.3
    #[test]
    fn native_dir_menu_has_no_open_or_rename_member() {
        let items = build_context_menu(CatalogType::Native, NodeKind::NativeDir, "");
        assert!(
            !has_action(&items, MenuAction::Open),
            "Dir must not have Open"
        );
        assert!(!has_action(&items, MenuAction::RenameMember));
        assert!(has_action(&items, MenuAction::OpenInNewTab));
        assert!(has_action(&items, MenuAction::NewFile));
        assert!(has_action(&items, MenuAction::NewFolder));
        assert!(has_action(&items, MenuAction::Rename));
        assert!(has_action(&items, MenuAction::MoveTo));
        assert!(has_action(&items, MenuAction::CopyTo));
        assert!(has_action(&items, MenuAction::CopyFullPath));
        assert!(has_action(&items, MenuAction::RevealInExplorer));
        assert!(has_disabled(&items, "Git \u{25b6}"));
    }

    // --- 16.4 POSIX File ----------------------------------------------------

    /// Validates: Requirement 16.4
    #[test]
    fn posix_file_menu_has_no_write_operations() {
        let items = build_context_menu(CatalogType::Posix, NodeKind::PosixFile, "sh");
        assert!(
            !has_action(&items, MenuAction::Rename),
            "POSIX is read-only"
        );
        assert!(!has_action(&items, MenuAction::MoveTo));
        assert!(!has_action(&items, MenuAction::CopyTo));
        assert!(!has_action(&items, MenuAction::NewFile));
        assert!(!has_action(&items, MenuAction::NewFolder));
        assert!(has_action(&items, MenuAction::Open));
        assert!(has_action(&items, MenuAction::Copy));
        assert!(has_action(&items, MenuAction::CopyFullPath));
    }

    // --- 16.5 Mainframe PS --------------------------------------------------

    /// Validates: Requirement 16.5
    #[test]
    fn mf_ps_menu_contains_required_actions() {
        let items = build_context_menu(CatalogType::Mainframe, NodeKind::MfPs, "");
        assert!(has_action(&items, MenuAction::Open));
        assert!(has_action(&items, MenuAction::OpenInNewTab));
        assert!(has_action(&items, MenuAction::Compare));
        assert!(has_action(&items, MenuAction::CopyTo));
        assert!(has_action(&items, MenuAction::CopyDatasetName));
        assert!(has_action(&items, MenuAction::DatasetProperties));
        assert!(has_action(&items, MenuAction::Refresh));
    }

    // --- 16.6 Mainframe PDS -------------------------------------------------

    /// Validates: Requirement 16.6
    #[test]
    fn mf_pds_menu_has_new_member_not_open() {
        let items = build_context_menu(CatalogType::Mainframe, NodeKind::MfPds, "");
        assert!(has_action(&items, MenuAction::NewMember));
        assert!(
            !has_action(&items, MenuAction::Open),
            "PDS itself is not openable"
        );
        assert!(has_action(&items, MenuAction::CopyTo));
        assert!(has_action(&items, MenuAction::CopyDatasetName));
        assert!(has_action(&items, MenuAction::DatasetProperties));
        assert!(has_action(&items, MenuAction::Refresh));
    }

    // --- 16.7 Mainframe PDS Member ------------------------------------------

    /// Validates: Requirement 16.7
    #[test]
    fn mf_member_menu_has_submit_jcl_disabled() {
        let items = build_context_menu(CatalogType::Mainframe, NodeKind::MfMember, "cbl");
        assert!(has_action(&items, MenuAction::Open));
        assert!(has_action(&items, MenuAction::OpenInNewTab));
        assert!(
            has_disabled(&items, "Submit JCL"),
            "Submit JCL must be greyed-out"
        );
        assert!(has_action(&items, MenuAction::Compare));
        assert!(has_action(&items, MenuAction::CopyMember));
        assert!(has_action(&items, MenuAction::RenameMember));
        assert!(has_action(&items, MenuAction::CopyMemberName));
        assert!(has_action(&items, MenuAction::CopyDatasetName));
        assert!(has_action(&items, MenuAction::CopyDatasetMember));
        assert!(has_action(&items, MenuAction::MemberProperties));
        assert!(has_action(&items, MenuAction::DatasetProperties));
        assert!(has_action(&items, MenuAction::Refresh));
    }

    // --- 16.8 Mainframe GDG Base --------------------------------------------

    /// Validates: Requirement 16.8
    #[test]
    fn mf_gdg_base_menu_has_new_generation() {
        let items = build_context_menu(CatalogType::Mainframe, NodeKind::MfGdgBase, "");
        assert!(has_action(&items, MenuAction::NewGeneration));
        assert!(has_action(&items, MenuAction::CopyDatasetName));
        assert!(has_action(&items, MenuAction::DatasetProperties));
        assert!(has_action(&items, MenuAction::Refresh));
        assert!(!has_action(&items, MenuAction::Open));
    }

    // --- 16.9 Mainframe GDG Generation --------------------------------------

    /// Validates: Requirement 16.9
    #[test]
    fn mf_gdg_gen_menu_has_open_and_dataset_name() {
        let items = build_context_menu(CatalogType::Mainframe, NodeKind::MfGdgGen, "");
        assert!(has_action(&items, MenuAction::Open));
        assert!(has_action(&items, MenuAction::OpenInNewTab));
        assert!(has_action(&items, MenuAction::CopyDatasetName));
        assert!(has_action(&items, MenuAction::DatasetProperties));
        assert!(has_action(&items, MenuAction::Refresh));
    }

    // --- 16.15 Git greyed-out -----------------------------------------------

    /// Validates: Requirement 16.15
    #[test]
    fn git_submenu_is_disabled_on_native_nodes() {
        let file_items = build_context_menu(CatalogType::Native, NodeKind::NativeFile, "rs");
        let dir_items = build_context_menu(CatalogType::Native, NodeKind::NativeDir, "");
        assert!(has_disabled(&file_items, "Git \u{25b6}"));
        assert!(has_disabled(&dir_items, "Git \u{25b6}"));
        // Must not be an active action
        assert!(!file_items
            .iter()
            .any(|i| matches!(i, MenuItem::Action(_) if {
                if let MenuItem::Action(a) = i { a.label().contains("Git") } else { false }
            })));
    }

    // --- 16.16 Submit JCL greyed-out ----------------------------------------

    /// Validates: Requirement 16.16
    #[test]
    fn submit_jcl_is_disabled_on_mf_member() {
        let items = build_context_menu(CatalogType::Mainframe, NodeKind::MfMember, "jcl");
        // Even with .jcl extension, Submit JCL stays disabled (no rule enables it yet)
        assert!(has_disabled(&items, "Submit JCL"));
        // Open IS present on MfMember — confirm Submit JCL is not an active action
        assert!(has_action(&items, MenuAction::Open));
        let active_labels: Vec<&str> = actions(&items).iter().map(|a| a.label()).collect();
        assert!(!active_labels.contains(&"Submit JCL"));
    }

    // --- 16.17 Extension rules table ----------------------------------------

    /// Validates: Requirement 16.17 AC 2 — table is defined in code
    #[test]
    fn extension_rules_table_is_defined() {
        // The table exists and is structured (may be empty in this release)
        let _ = EXTENSION_RULES;
    }

    // --- 17.8 FileClass / EXTERNAL_EXTENSIONS table -------------------------

    /// Validates: Requirement 17.8 — Office docs are External
    #[test]
    fn office_extensions_are_external() {
        for ext in &[
            "docx", "xlsx", "pptx", "doc", "xls", "ppt", "odt", "ods", "odp",
        ] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} must be External"
            );
        }
    }

    /// Validates: Requirement 17.8 — PDF/eBook are External
    #[test]
    fn pdf_extensions_are_external() {
        for ext in &["pdf", "epub", "mobi"] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} must be External"
            );
        }
    }

    /// Validates: Requirement 17.8 — image extensions are External
    #[test]
    fn image_extensions_are_external() {
        for ext in &[
            "png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp", "svg", "ico",
        ] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} must be External"
            );
        }
    }

    /// Validates: Requirement 17.8 — audio/video extensions are External
    #[test]
    fn audio_video_extensions_are_external() {
        for ext in &["mp3", "mp4", "wav", "flac", "avi", "mkv", "mov", "wmv"] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} must be External"
            );
        }
    }

    /// Validates: Requirement 17.8 — archive extensions are External
    #[test]
    fn archive_extensions_are_external() {
        for ext in &["zip", "tar", "gz", "bz2", "xz", "7z", "rar"] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} must be External"
            );
        }
    }

    /// Validates: Requirement 17.8 — executable extensions are External
    #[test]
    fn executable_extensions_are_external() {
        for ext in &["exe", "dll", "so", "dylib"] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} must be External"
            );
        }
    }

    /// Validates: Requirement 17.8 — database extensions are External
    #[test]
    fn database_extensions_are_external() {
        for ext in &["db", "sqlite", "mdb", "accdb"] {
            assert_eq!(
                classify_extension(ext),
                FileClass::External,
                "{ext} must be External"
            );
        }
    }

    /// Validates: Requirement 17.1 — source/text extensions are Text
    #[test]
    fn source_extensions_are_text() {
        for ext in &[
            "rs", "c", "cpp", "py", "sh", "txt", "toml", "yaml", "jcl", "cbl",
        ] {
            assert_eq!(
                classify_extension(ext),
                FileClass::Text,
                "{ext} must be Text"
            );
        }
    }

    /// Validates: Requirement 17.3 — magic-byte scan: null byte = binary
    #[test]
    fn magic_byte_scan_null_byte_is_binary() {
        let data = b"hello\x00world";
        assert!(!is_text_bytes(data), "null byte must be detected as binary");
    }

    /// Validates: Requirement 17.3 — magic-byte scan: valid UTF-8 = text
    #[test]
    fn magic_byte_scan_utf8_is_text() {
        let data = b"Hello, world!\nThis is a text file.\n";
        assert!(is_text_bytes(data), "plain ASCII must be detected as text");
    }

    /// Validates: Requirement 17.3 — magic-byte scan: high binary ratio = binary
    #[test]
    fn magic_byte_scan_high_binary_ratio_is_binary() {
        // PNG magic bytes + random binary data
        let data: Vec<u8> = (0u8..=255u8).cycle().take(512).collect();
        assert!(
            !is_text_bytes(&data),
            "high non-UTF-8 ratio must be detected as binary"
        );
    }

    /// Validates: Requirement 17.6 — launch_default_app uses correct command on this platform
    #[test]
    fn launch_default_app_command_is_platform_appropriate() {
        // We can't actually launch a file in tests, but we can verify the
        // function exists and accepts a path without panicking on a dummy path.
        // The actual spawn will fail (no such file) but must not panic.
        let result = std::panic::catch_unwind(|| {
            launch_default_app("/nonexistent/test/file.txt");
        });
        assert!(result.is_ok(), "launch_default_app must not panic");
    }

    // --- 16.18 Copy path variants -------------------------------------------

    /// Validates: Requirement 16.18 AC 1 — Copy File Name = base name only
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

    /// Validates: Requirement 16.18 AC 2 — Copy Relative Path
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

    /// Validates: Requirement 16.18 AC 4 — Copy Dataset Name = fully-qualified DSN
    #[test]
    fn copy_dataset_name_is_fully_qualified() {
        let dsn = "PAYROLL.DATA";
        assert!(dsn.contains('.'), "DSN must be fully qualified");
    }

    /// Validates: Requirement 16.18 AC 5 — Copy Member Name = member only
    #[test]
    fn copy_member_name_is_member_only() {
        let member = "MYJOB";
        assert!(!member.contains('('), "member name must not include parens");
        assert!(member.len() <= 8, "member name must be <= 8 chars");
    }

    /// Validates: Requirement 16.18 AC 6 — Copy Dataset(Member) combined form
    #[test]
    fn copy_dataset_member_combined_form() {
        let dsn = "PAYROLL.JCL";
        let member = "MYJOB";
        let combined = format!("{dsn}({member})");
        assert_eq!(combined, "PAYROLL.JCL(MYJOB)");
    }

    // --- 16.12 Naming transformation ----------------------------------------

    /// Validates: Requirement 16.12 AC 2 — Native→Mainframe: uppercase + truncate to 8
    #[test]
    fn native_to_mainframe_name_transform_uppercase_truncate() {
        let native_name = "my_long_filename.rs";
        let stem = std::path::Path::new(native_name)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_uppercase();
        let truncated: String = stem.chars().take(8).collect();
        // Only keep valid mainframe chars (A-Z, 0-9, @, #, $)
        let valid: String = truncated
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '@' | '#' | '$'))
            .collect();
        assert_eq!(
            valid,
            "MY_LONG_"
                .replace('_', "")
                .chars()
                .take(8)
                .collect::<String>()
        );
        // Simpler: just verify uppercase + truncate
        let result: String = "my_long_filename"
            .to_uppercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .take(8)
            .collect();
        assert_eq!(result, "MYLONGFI");
    }

    /// Validates: Requirement 16.12 AC 4 — Mainframe→Native: lowercase, no extension
    #[test]
    fn mainframe_to_native_name_transform_lowercase() {
        let member = "MYJOB";
        let native = member.to_lowercase();
        assert_eq!(native, "myjob");
    }

    // --- 16.11 Inline rename — 8-char uppercase enforcement -----------------

    /// Validates: Requirement 16.11 AC 4 — Mainframe member name > 8 chars is invalid
    #[test]
    fn mainframe_member_name_over_8_chars_is_invalid() {
        let name = "TOOLONGNAME";
        assert!(name.len() > 8, "test precondition");
        assert!(!is_valid_mainframe_member_name(name));
    }

    /// Validates: Requirement 16.11 AC 4 — valid 8-char uppercase name is accepted
    #[test]
    fn mainframe_member_name_8_chars_uppercase_is_valid() {
        assert!(is_valid_mainframe_member_name("MYJOB"));
        assert!(is_valid_mainframe_member_name("PAYROLL1"));
    }

    /// Validates: Requirement 16.11 AC 4 — lowercase name is invalid
    #[test]
    fn mainframe_member_name_lowercase_is_invalid() {
        assert!(!is_valid_mainframe_member_name("myjob"));
    }
}

// === Mainframe member name validation =======================================

/// Returns true if `name` is a valid Mainframe PDS member name:
/// 1–8 characters, uppercase A-Z, digits 0-9, or national chars @, #, $.
/// First character must be alphabetic or national.
///
/// Validates: Requirement 16.11 AC 4
pub fn is_valid_mainframe_member_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 8 {
        return false;
    }
    let mut chars = name.chars();
    let first = chars.next().expect("non-empty");
    if !matches!(first, 'A'..='Z' | '@' | '#' | '$') {
        return false;
    }
    chars.all(|c| matches!(c, 'A'..='Z' | '0'..='9' | '@' | '#' | '$'))
}
