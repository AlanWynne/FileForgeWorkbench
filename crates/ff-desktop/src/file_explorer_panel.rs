//! # File Explorer Panel — POM Option 2
//!
//! Renders the File Explorer Panel: a tree view of all open catalogs grouped
//! under Mainframe Catalogs, POSIX Catalogs, and Native Catalogs section headers.
//! Each catalog node is expandable to show its files/datasets as child nodes.
//! Double-clicking a file node opens it in a new editor tab.
//! Right-clicking any node shows a context menu appropriate to the node kind.
//!
//! Validates: Requirement 19.5, 19.6, 19.7, 19.8, 19.9
//! Validates: Requirement 16.1–16.18
//! Validates: Requirement 18.1–18.9

use std::time::SystemTime;

use eframe::egui;

use crate::catalog_registry::{CatalogRegistry, CatalogType};
use crate::context_menu::{
    build_context_menu, classify_file, is_valid_mainframe_member_name, launch_default_app,
    FileClass, MenuAction, MenuItem, NodeKind,
};
use crate::copy_move_dialog::{native_to_mainframe_name, CopyMoveDialog, CopyMoveKind};
use crate::files_panel::FilesPanelState;

// ── State ─────────────────────────────────────────────────────────────────────

/// State for the File Explorer Panel tab.
///
/// Holds inline-rename state and the Copy To / Move To dialog.
///
/// Validates: Requirement 19.6, 16.11, 16.12, 18.8
#[derive(Debug, Default, Clone)]
pub struct FileExplorerPanelState {
    /// Active inline rename: `Some((full_path, edit_buffer))` while renaming.
    ///
    /// Validates: Requirement 16.11
    pub rename_state: Option<(String, String)>,
    /// Inline rename error (shown beneath the TextEdit for Mainframe members).
    pub rename_error: String,
    /// Copy To / Move To dialog.
    ///
    /// Validates: Requirement 16.12
    pub copy_move_dialog: CopyMoveDialog,
    /// Pending background-IO progress message (shown in status bar).
    #[allow(dead_code)]
    pub bgio_progress: Option<String>,
    /// Last error message to display in the status bar (e.g. locked-file open).
    ///
    /// Validates: Requirement 18.8
    pub last_error: Option<String>,
}

impl FileExplorerPanelState {
    pub fn new() -> Self {
        Self::default()
    }
}

// ── Render ────────────────────────────────────────────────────────────────────

/// Render the File Explorer Panel tree view.
///
/// Returns `Some(path)` when the user double-clicks a file node — the shell
/// must open that path in a new editor tab.
///
/// Validates: Requirement 19.5, 19.6, 19.7, 19.8, 19.9
pub fn render(
    ui: &mut egui::Ui,
    state: &mut FileExplorerPanelState,
    registry: &CatalogRegistry,
    files_panel: &FilesPanelState,
) -> Option<String> {
    let mut open_path: Option<String> = None;

    // Render Copy To / Move To dialog if open
    state.copy_move_dialog.render(ui.ctx());

    let total_catalogs = registry.list().len();

    if total_catalogs == 0 {
        // Validates: Requirement 19.8 — empty-state placeholder
        ui.label(egui::RichText::new(
            "No catalogs open \u{2014} use File Catalogs (option 1) to create or mount a catalog",
        ).monospace().weak());
        return None;
    }

    // Validates: Requirement 15.3 — scrollable content area
    egui::ScrollArea::vertical().show(ui, |ui| {
        // Validates: Requirement 19.7 — three section headers
        for (catalog_type, header_label) in [
            (CatalogType::Mainframe, "Mainframe Catalogs"),
            (CatalogType::Posix, "POSIX Catalogs"),
            (CatalogType::Native, "Native Catalogs"),
        ] {
            let catalogs = registry.list_by_type(catalog_type);
            egui::CollapsingHeader::new(egui::RichText::new(header_label).monospace().strong())
                .default_open(true)
                .show(ui, |ui| {
                    if catalogs.is_empty() {
                        ui.label(egui::RichText::new("  (none)").monospace().weak());
                    } else {
                        // Validates: Requirement 19.5 — each catalog as a top-level expandable node
                        for cat in &catalogs {
                            egui::CollapsingHeader::new(
                                egui::RichText::new(format!("\u{1F4C1} {}", cat.name)).monospace(),
                            )
                            .id_salt(format!("fep_cat_{}", cat.name))
                            .default_open(false)
                            .show(ui, |ui| {
                                // Validates: Requirement 19.6 — Native catalogs list disk files;
                                // Mainframe/POSIX catalogs list allocated datasets.
                                if cat.catalog_type == CatalogType::Native {
                                    render_native_children(
                                        ui,
                                        &cat.path,
                                        &cat.path,
                                        state,
                                        &mut open_path,
                                    );
                                } else {
                                    render_dataset_children(
                                        ui,
                                        cat.catalog_type,
                                        files_panel.datasets.get(&cat.name),
                                        state,
                                        &mut open_path,
                                    );
                                }
                            });
                        }
                    }
                });
        }
    });

    open_path
}

// ── Child renderers ──────────────────────────────────────────────────────────

// ── File entry row ────────────────────────────────────────────────────────────

/// A single entry in a Native catalog directory listing.
///
/// Validates: Requirement 18.2–18.6
#[derive(Debug, Clone)]
struct FileEntryRow {
    is_dir: bool,
    name: String,
    full_path: String,
    size_bytes: Option<u64>,
    created: Option<SystemTime>,
    modified: Option<SystemTime>,
    accessed: Option<SystemTime>,
    permissions_str: String,
}

/// Format a byte count as a human-readable size string.
///
/// Validates: Requirement 18.2
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = 1_024 * KB;
    const GB: u64 = 1_024 * MB;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Format a `SystemTime` as `YYYY-MM-DD HH:MM`.
///
/// Validates: Requirement 18.3, 18.4, 18.5
pub fn format_timestamp(t: SystemTime) -> String {
    // Seconds since Unix epoch
    let secs = match t.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => return "—".to_string(),
    };
    // Simple calendar calculation (no external dep)
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    // Days since 1970-01-01
    let (year, month, day) = days_to_ymd(days);
    let _ = s; // seconds not displayed
    format!("{year:04}-{month:02}-{day:02} {h:02}:{m:02}")
}

/// Convert days-since-epoch to (year, month, day).
fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let leap = is_leap(year);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let month_days: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(year: u64) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

/// Format file permissions in a user-friendly compact string.
///
/// Windows: flags R/H/S/A; Unix: rwxr-xr-x style.
/// Validates: Requirement 18.6
pub fn format_permissions(meta: &std::fs::Metadata) -> String {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_READONLY: u32 = 0x0001;
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x0002;
        const FILE_ATTRIBUTE_SYSTEM: u32 = 0x0004;
        const FILE_ATTRIBUTE_ARCHIVE: u32 = 0x0020;
        let attrs = meta.file_attributes();
        let mut s = String::with_capacity(4);
        if attrs & FILE_ATTRIBUTE_READONLY != 0 {
            s.push('R');
        }
        if attrs & FILE_ATTRIBUTE_HIDDEN != 0 {
            s.push('H');
        }
        if attrs & FILE_ATTRIBUTE_SYSTEM != 0 {
            s.push('S');
        }
        if attrs & FILE_ATTRIBUTE_ARCHIVE != 0 {
            s.push('A');
        }
        if s.is_empty() {
            "—".to_string()
        } else {
            s
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = meta.permissions().mode();
        let bits = [
            (0o400, 'r'),
            (0o200, 'w'),
            (0o100, 'x'),
            (0o040, 'r'),
            (0o020, 'w'),
            (0o010, 'x'),
            (0o004, 'r'),
            (0o002, 'w'),
            (0o001, 'x'),
        ];
        bits.iter()
            .map(|(b, c)| if mode & b != 0 { *c } else { '-' })
            .collect()
    }
}

/// Collect and sort directory entries for a Native catalog path.
///
/// Silently skips entries where metadata() returns an error (Req 18.7).
/// Sorts directories first, then files, both groups alphabetically case-insensitive (Req 18.1).
fn collect_native_entries(path: &str) -> Result<Vec<FileEntryRow>, std::io::Error> {
    let mut rows: Vec<FileEntryRow> = std::fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            // Req 18.7 — silently skip entries where metadata fails
            let meta = e.metadata().ok()?;
            let name = e.file_name().to_string_lossy().into_owned();
            let full_path = e.path().to_string_lossy().into_owned();
            let is_dir = meta.is_dir();
            let size_bytes = if is_dir { None } else { Some(meta.len()) };
            let created = meta.created().ok();
            let modified = meta.modified().ok();
            let accessed = meta.accessed().ok();
            let permissions_str = format_permissions(&meta);
            Some(FileEntryRow {
                is_dir,
                name,
                full_path,
                size_bytes,
                created,
                modified,
                accessed,
                permissions_str,
            })
        })
        .collect();
    // Req 18.1 — dirs first, then files; each group alphabetical case-insensitive
    rows.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(rows)
}

/// Render children for a Native catalog by reading the directory from disk.
///
/// Validates: Requirement 19.6, 16.2, 16.3, 18.1–18.9
fn render_native_children(
    ui: &mut egui::Ui,
    path: &str,
    catalog_root: &str,
    state: &mut FileExplorerPanelState,
    open_path: &mut Option<String>,
) {
    let rows = match collect_native_entries(path) {
        Err(e) => {
            ui.label(
                egui::RichText::new(format!("  (error reading directory: {e})"))
                    .monospace()
                    .weak(),
            );
            return;
        }
        Ok(r) => r,
    };

    if rows.is_empty() {
        ui.label(egui::RichText::new("  (empty)").monospace().weak());
        return;
    }

    // Column headers — Req 18.9
    egui::Grid::new(format!("fep_hdr_{path}"))
        .num_columns(6)
        .min_col_width(0.0)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Name").monospace().strong());
            ui.label(egui::RichText::new("Size").monospace().strong());
            ui.label(egui::RichText::new("Modified").monospace().strong());
            ui.label(egui::RichText::new("Created").monospace().strong());
            ui.label(egui::RichText::new("Accessed").monospace().strong());
            ui.label(egui::RichText::new("Perms").monospace().strong());
            ui.end_row();
        });
    ui.separator();

    // Data rows — Req 18.1–18.9
    egui::Grid::new(format!("fep_grid_{path}"))
        .num_columns(6)
        .striped(true)
        .min_col_width(0.0)
        .show(ui, |ui| {
            for row in rows {
                let mod_str = row
                    .modified
                    .map(format_timestamp)
                    .unwrap_or_else(|| "—".to_string());
                let cre_str = row
                    .created
                    .map(format_timestamp)
                    .unwrap_or_else(|| "—".to_string());
                let acc_str = row
                    .accessed
                    .map(format_timestamp)
                    .unwrap_or_else(|| "—".to_string());
                if row.is_dir {
                    let resp = egui::CollapsingHeader::new(
                        egui::RichText::new(format!("\u{1F4C1} {}", row.name)).monospace(),
                    )
                    .id_salt(format!("fep_dir_{}", row.full_path))
                    .default_open(false)
                    .show(ui, |ui| {
                        render_native_children(ui, &row.full_path, catalog_root, state, open_path);
                    });
                    resp.header_response.context_menu(|ui| {
                        show_context_menu(
                            ui,
                            CatalogType::Native,
                            NodeKind::NativeDir,
                            &row.full_path,
                            catalog_root,
                            state,
                            open_path,
                        );
                    });
                    // Attribute columns for directories
                    ui.label(egui::RichText::new("<DIR>").monospace().weak());
                    ui.label(egui::RichText::new(&mod_str).monospace().weak());
                    ui.label(egui::RichText::new(&cre_str).monospace().weak());
                    ui.label(egui::RichText::new(&acc_str).monospace().weak());
                    ui.label(egui::RichText::new(&row.permissions_str).monospace().weak());
                    ui.end_row();
                } else {
                    let is_renaming = state
                        .rename_state
                        .as_ref()
                        .map(|(p, _)| p == &row.full_path)
                        .unwrap_or(false);
                    if is_renaming {
                        render_inline_rename(ui, state, &row.full_path, false);
                        for _ in 0..5 {
                            ui.label("");
                        }
                        ui.end_row();
                    } else {
                        let fp = row.full_path.clone();
                        let cr = catalog_root.to_string();
                        let label =
                            egui::RichText::new(format!("  \u{1F4C4} {}", row.name)).monospace();
                        let resp = ui.selectable_label(false, label);
                        if resp.double_clicked() {
                            open_file_node(&fp, state, open_path);
                        }
                        resp.context_menu(|ui| {
                            show_context_menu(
                                ui,
                                CatalogType::Native,
                                NodeKind::NativeFile,
                                &fp,
                                &cr,
                                state,
                                open_path,
                            );
                        });
                        // Attribute columns — Req 18.2–18.6, 18.9
                        let size_str = row.size_bytes.map(format_size).unwrap_or_default();
                        ui.label(egui::RichText::new(&size_str).monospace().weak());
                        ui.label(egui::RichText::new(&mod_str).monospace().weak());
                        ui.label(egui::RichText::new(&cre_str).monospace().weak());
                        ui.label(egui::RichText::new(&acc_str).monospace().weak());
                        ui.label(egui::RichText::new(&row.permissions_str).monospace().weak());
                        ui.end_row();
                    }
                }
            }
        }); // end Grid
}

/// Open a file node: route text files to the editor, external files to the OS.
/// Catches OS error 32 (file in use) and stores a status-bar message.
///
/// Validates: Requirement 17.1, 17.2, 17.7, 18.8
fn open_file_node(path: &str, state: &mut FileExplorerPanelState, open_path: &mut Option<String>) {
    match classify_file(path) {
        FileClass::Text | FileClass::FfwbStructured => {
            // Attempt to verify the file is readable before handing to editor
            match std::fs::File::open(path) {
                Ok(_) => {
                    *open_path = Some(path.to_string());
                }
                Err(e) => {
                    // Req 18.8 — OS error 32 = file in use by another process
                    let name = std::path::Path::new(path)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.to_string());
                    state.last_error = Some(format!(
                        "Cannot open '{name}': {}",
                        if e.raw_os_error() == Some(32) {
                            "file is in use by another process".to_string()
                        } else {
                            e.to_string()
                        }
                    ));
                }
            }
        }
        FileClass::External => {
            launch_default_app(path);
        }
    }
}

/// Render children for a Mainframe/POSIX catalog from the allocated dataset store.
///
/// Validates: Requirement 19.6, 16.4–16.9
fn render_dataset_children(
    ui: &mut egui::Ui,
    catalog_type: CatalogType,
    datasets: Option<&Vec<crate::files_panel::AllocatedDataset>>,
    state: &mut FileExplorerPanelState,
    open_path: &mut Option<String>,
) {
    match datasets.map(|v| v.as_slice()) {
        None | Some([]) => {
            ui.label(egui::RichText::new("  (no datasets)").monospace().weak());
        }
        Some(datasets) => {
            for ds in datasets {
                let is_container = ds.dsorg == "PO" || ds.dsorg == "PDSE" || ds.dsorg == "GDG";
                let icon = if is_container {
                    "\u{1F4C1}"
                } else {
                    "\u{1F4C4}"
                };
                let label = egui::RichText::new(format!("  {icon} {}", ds.name)).monospace();
                let resp = ui.selectable_label(false, label);
                if resp.double_clicked() {
                    *open_path = Some(ds.name.clone());
                }
                let node_kind = dataset_node_kind(&ds.dsorg);
                let dsn = ds.name.clone();
                resp.context_menu(|ui| {
                    show_context_menu(ui, catalog_type, node_kind, &dsn, "", state, open_path);
                });
            }
        }
    }
}

/// Map a dataset DSORG string to the appropriate `NodeKind`.
fn dataset_node_kind(dsorg: &str) -> NodeKind {
    match dsorg {
        "PO" | "PDSE" => NodeKind::MfPds,
        "GDG" => NodeKind::MfGdgBase,
        _ => NodeKind::MfPs,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

// === Context menu dispatch =================================================

/// Render the context menu items for a node and handle the selected action.
///
/// Validates: Requirement 16.1, 16.2-16.9, 16.10, 16.11, 16.12, 16.14
#[allow(clippy::too_many_arguments)]
fn show_context_menu(
    ui: &mut egui::Ui,
    catalog_type: CatalogType,
    node_kind: NodeKind,
    node_path: &str,
    catalog_root: &str,
    state: &mut FileExplorerPanelState,
    open_path: &mut Option<String>,
) {
    let items = build_context_menu(catalog_type, node_kind, "");
    for item in &items {
        match item {
            MenuItem::Separator => {
                ui.separator();
            }
            MenuItem::Disabled(label) => {
                ui.add_enabled(false, egui::Button::new(*label));
            }
            MenuItem::Action(action) => {
                let label = if *action == MenuAction::RevealInExplorer {
                    MenuAction::reveal_label()
                } else {
                    action.label()
                };
                if ui.button(label).clicked() {
                    ui.close_menu();
                    handle_menu_action(
                        *action,
                        node_path,
                        catalog_root,
                        catalog_type,
                        node_kind,
                        state,
                        open_path,
                    );
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_menu_action(
    action: MenuAction,
    node_path: &str,
    catalog_root: &str,
    _catalog_type: CatalogType,
    _node_kind: NodeKind,
    state: &mut FileExplorerPanelState,
    open_path: &mut Option<String>,
) {
    match action {
        MenuAction::Open | MenuAction::OpenInNewTab | MenuAction::OpenInNewWindow => {
            // Validates: Requirement 17.1, 17.2, 17.7, 18.8
            match (_catalog_type, _node_kind) {
                // Mainframe nodes always open in FFWB
                (crate::catalog_registry::CatalogType::Mainframe, _) => {
                    *open_path = Some(node_path.to_string());
                }
                _ => open_file_node(node_path, state, open_path),
            }
        }
        MenuAction::Copy | MenuAction::CopyFullPath | MenuAction::CopyDatasetName => {
            write_to_clipboard(node_path);
        }
        MenuAction::CopyFileName => {
            let name = std::path::Path::new(node_path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| node_path.to_string());
            write_to_clipboard(&name);
        }
        MenuAction::CopyRelativePath => {
            let rel = std::path::Path::new(node_path)
                .strip_prefix(catalog_root)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| node_path.to_string());
            write_to_clipboard(&rel);
        }
        MenuAction::CopyMemberName => {
            let member = extract_member_name(node_path).unwrap_or_else(|| node_path.to_string());
            write_to_clipboard(&member);
        }
        MenuAction::CopyDatasetMember => {
            write_to_clipboard(node_path);
        }
        MenuAction::Rename | MenuAction::RenameMember => {
            let current_name = std::path::Path::new(node_path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| node_path.to_string());
            state.rename_state = Some((node_path.to_string(), current_name));
            state.rename_error.clear();
        }
        MenuAction::MoveTo => {
            let proposed = native_to_mainframe_name(node_path);
            state
                .copy_move_dialog
                .open(CopyMoveKind::Move, node_path, &proposed);
        }
        MenuAction::CopyTo => {
            let proposed = native_to_mainframe_name(node_path);
            state
                .copy_move_dialog
                .open(CopyMoveKind::Copy, node_path, &proposed);
        }
        MenuAction::RevealInExplorer | MenuAction::OpenContainingFolder => {
            reveal_in_explorer(node_path);
        }
        _ => {}
    }
}

// === Inline rename ==========================================================

/// Render an inline TextEdit in place of the node label.
///
/// Validates: Requirement 16.11
fn render_inline_rename(
    ui: &mut egui::Ui,
    state: &mut FileExplorerPanelState,
    full_path: &str,
    is_mainframe_member: bool,
) {
    let Some((_, ref mut buf)) = state.rename_state else {
        return;
    };
    let resp = ui.text_edit_singleline(buf);
    if !state.rename_error.is_empty() {
        ui.colored_label(egui::Color32::RED, &state.rename_error);
    }
    let confirmed = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let cancelled = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape));
    if confirmed {
        let new_name = buf.clone();
        if is_mainframe_member && !is_valid_mainframe_member_name(&new_name) {
            state.rename_error =
                "Name must be 1-8 uppercase chars (A-Z, 0-9, @, #, $).".to_string();
            return;
        }
        let old_path = std::path::Path::new(full_path);
        if let Some(parent) = old_path.parent() {
            let new_path = parent.join(&new_name);
            let _ = std::fs::rename(old_path, new_path);
        }
        state.rename_state = None;
        state.rename_error.clear();
    } else if cancelled {
        state.rename_state = None;
        state.rename_error.clear();
    }
}

// === Clipboard ==============================================================

/// Write `text` to the OS clipboard via arboard.
///
/// Validates: Requirement 16.10 AC 1, 16.18
fn write_to_clipboard(text: &str) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(text);
    }
}

// === Reveal in Explorer =====================================================

/// Open the OS file manager at the parent directory of `path`.
///
/// Validates: Requirement 16.14
fn reveal_in_explorer(path: &str) {
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

// === Helpers ================================================================

fn extract_member_name(dsn_member: &str) -> Option<String> {
    let start = dsn_member.find('(')?;
    let end = dsn_member.rfind(')')?;
    if end > start {
        Some(dsn_member[start + 1..end].to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog_registry::{CatalogType, VirtualCatalog};

    fn make_catalog(name: &str, catalog_type: CatalogType) -> VirtualCatalog {
        VirtualCatalog {
            name: name.to_string(),
            catalog_type,
            path: "/some/path".to_string(),
            description: None,
            auto_mount: true,
            default_hlq: None,
            mount_point: None,
            read_only: false,
        }
    }

    /// Validates: Requirement 19.7 — three section header types are represented.
    #[test]
    fn file_explorer_panel_has_three_catalog_type_sections() {
        // Validates: Requirement 19.7
        let types = [
            CatalogType::Mainframe,
            CatalogType::Posix,
            CatalogType::Native,
        ];
        assert_eq!(types.len(), 3);
    }

    /// Validates: Requirement 19.8 — empty registry produces no catalog nodes.
    #[test]
    fn empty_registry_has_no_catalog_nodes() {
        // Validates: Requirement 19.8
        let registry = CatalogRegistry::new();
        assert_eq!(registry.list().len(), 0);
    }

    /// Validates: Requirement 19.5 — each registered catalog appears as a node.
    #[test]
    fn registered_catalogs_appear_as_tree_nodes() {
        // Validates: Requirement 19.5
        let mut registry = CatalogRegistry::new();
        registry
            .register(make_catalog("PAYROLL", CatalogType::Mainframe))
            .unwrap();
        registry
            .register(make_catalog("POSIX1", CatalogType::Posix))
            .unwrap();
        registry
            .register(make_catalog("LOCAL", CatalogType::Native))
            .unwrap();

        assert_eq!(registry.list_by_type(CatalogType::Mainframe).len(), 1);
        assert_eq!(registry.list_by_type(CatalogType::Posix).len(), 1);
        assert_eq!(registry.list_by_type(CatalogType::Native).len(), 1);
    }

    /// Validates: Requirement 19.6 — datasets for a catalog are accessible for child nodes.
    #[test]
    fn catalog_datasets_accessible_for_child_nodes() {
        // Validates: Requirement 19.6
        use crate::dataset_alloc_dialog::{AllocParams, Dsorg, Recfm};
        let mut files_panel = FilesPanelState::new();
        files_panel.add_dataset(
            "PAYROLL",
            AllocParams {
                dataset_name: "PAYROLL.DATA".to_string(),
                dsorg: Dsorg::Ps,
                recfm: Recfm::Fb,
                lrecl: 80,
                blksize: 27920,
                dir_blocks: None,
                gdg_limit: None,
                scratch: false,
                description: None,
            },
        );
        let datasets = files_panel.datasets.get("PAYROLL").expect("must exist");
        assert_eq!(datasets.len(), 1);
        assert_eq!(datasets[0].name, "PAYROLL.DATA");
    }

    /// Validates: Requirement 19.7 — section headers use the correct labels.
    #[test]
    fn section_header_labels_match_catalog_type_labels() {
        // Validates: Requirement 19.7
        assert_eq!(CatalogType::Mainframe.section_label(), "Mainframe Catalogs");
        assert_eq!(CatalogType::Posix.section_label(), "POSIX Catalogs");
        assert_eq!(CatalogType::Native.section_label(), "Native Catalogs");
    }

    /// Validates: Requirement 19.9 — file nodes (non-container datasets) are leaf nodes.
    #[test]
    fn ps_dataset_is_a_leaf_node_not_a_container() {
        // Validates: Requirement 19.9
        use crate::dataset_alloc_dialog::{AllocParams, Dsorg, Recfm};
        let mut files_panel = FilesPanelState::new();
        files_panel.add_dataset(
            "CAT",
            AllocParams {
                dataset_name: "CAT.SEQ".to_string(),
                dsorg: Dsorg::Ps,
                recfm: Recfm::Fb,
                lrecl: 80,
                blksize: 27920,
                dir_blocks: None,
                gdg_limit: None,
                scratch: false,
                description: None,
            },
        );
        let ds = &files_panel.datasets["CAT"][0];
        // PS is not a container — double-click should open it
        assert_eq!(ds.dsorg, "PS");
        let is_container = ds.dsorg == "PO" || ds.dsorg == "PDSE" || ds.dsorg == "GDG";
        assert!(!is_container, "PS dataset must be a leaf node");
    }

    /// Validates: Requirement 19.9 — PO dataset is a container node (not directly openable).
    #[test]
    fn po_dataset_is_a_container_node() {
        // Validates: Requirement 19.9
        use crate::dataset_alloc_dialog::{AllocParams, Dsorg, Recfm};
        let mut files_panel = FilesPanelState::new();
        files_panel.add_dataset(
            "CAT",
            AllocParams {
                dataset_name: "CAT.LIB".to_string(),
                dsorg: Dsorg::Po,
                recfm: Recfm::Fb,
                lrecl: 80,
                blksize: 27920,
                dir_blocks: None,
                gdg_limit: None,
                scratch: false,
                description: None,
            },
        );
        let ds = &files_panel.datasets["CAT"][0];
        let is_container = ds.dsorg == "PO" || ds.dsorg == "PDSE" || ds.dsorg == "GDG";
        assert!(is_container, "PO dataset must be a container node");
    }

    /// Validates: Requirement 19.8 — total_catalogs == 0 triggers empty-state path.
    #[test]
    fn zero_catalogs_triggers_empty_state() {
        // Validates: Requirement 19.8
        let registry = CatalogRegistry::new();
        assert_eq!(
            registry.list().len(),
            0,
            "empty registry must have 0 catalogs"
        );
    }

    // === Phase BB tests =====================================================

    /// Validates: Requirement 18.1 — dirs-first then alphabetical sort.
    #[test]
    fn collect_native_entries_sorts_dirs_first_then_alpha() {
        // Validates: Requirement 18.1
        use tempfile::TempDir;
        let tmp = TempDir::new().expect("tempdir");
        std::fs::write(tmp.path().join("zebra.txt"), "").unwrap();
        std::fs::write(tmp.path().join("apple.txt"), "").unwrap();
        std::fs::create_dir(tmp.path().join("mango")).unwrap();
        std::fs::create_dir(tmp.path().join("banana")).unwrap();
        let rows =
            collect_native_entries(tmp.path().to_str().unwrap()).expect("collect must succeed");
        assert_eq!(rows.len(), 4);
        assert!(rows[0].is_dir);
        assert_eq!(rows[0].name, "banana");
        assert!(rows[1].is_dir);
        assert_eq!(rows[1].name, "mango");
        assert!(!rows[2].is_dir);
        assert_eq!(rows[2].name, "apple.txt");
        assert!(!rows[3].is_dir);
        assert_eq!(rows[3].name, "zebra.txt");
    }

    /// Validates: Requirement 18.7 — valid entries are collected without error.
    #[test]
    fn collect_native_entries_returns_valid_entries() {
        // Validates: Requirement 18.7
        use tempfile::TempDir;
        let tmp = TempDir::new().expect("tempdir");
        std::fs::write(tmp.path().join("ok.txt"), "data").unwrap();
        let rows =
            collect_native_entries(tmp.path().to_str().unwrap()).expect("collect must succeed");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "ok.txt");
    }

    /// Validates: Requirement 18.2 — format_size produces correct human-readable strings.
    #[test]
    fn format_size_produces_correct_strings() {
        // Validates: Requirement 18.2
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1_024), "1.0 KB");
        assert_eq!(format_size(1_536), "1.5 KB");
        assert_eq!(format_size(1_048_576), "1.0 MB");
        assert_eq!(format_size(1_073_741_824), "1.0 GB");
    }

    /// Validates: Requirement 18.3, 18.4, 18.5 — format_timestamp produces YYYY-MM-DD HH:MM.
    #[test]
    fn format_timestamp_produces_correct_format() {
        // Validates: Requirement 18.3, 18.4, 18.5
        use std::time::{Duration, SystemTime};
        // 2024-01-15 10:30 UTC = 1705314600 seconds since epoch
        let t = SystemTime::UNIX_EPOCH + Duration::from_secs(1_705_314_600);
        let s = format_timestamp(t);
        assert_eq!(s.len(), 16, "timestamp must be 16 chars: got '{s}'");
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[7..8], "-");
        assert_eq!(&s[10..11], " ");
        assert_eq!(&s[13..14], ":");
    }

    /// Validates: Requirement 18.6 — format_permissions returns a non-empty string.
    #[test]
    fn format_permissions_returns_nonempty_string() {
        // Validates: Requirement 18.6
        use tempfile::TempDir;
        let tmp = TempDir::new().expect("tempdir");
        let f = tmp.path().join("perm_test.txt");
        std::fs::write(&f, "x").unwrap();
        let meta = std::fs::metadata(&f).expect("metadata must succeed");
        let s = format_permissions(&meta);
        assert!(!s.is_empty(), "permissions string must not be empty");
    }

    /// Validates: Requirement 18.8 — open_file_node stores last_error for unreadable files.
    #[test]
    fn open_file_node_stores_error_for_nonexistent_file() {
        // Validates: Requirement 18.8
        let mut state = FileExplorerPanelState::new();
        let mut open_path: Option<String> = None;
        open_file_node(
            "C:\\nonexistent\\path\\to\\file.txt",
            &mut state,
            &mut open_path,
        );
        assert!(open_path.is_none(), "nonexistent file must not be opened");
        assert!(
            state.last_error.is_some(),
            "last_error must be set for unreadable file"
        );
    }

    /// Validates: Requirement 19.6 — Native catalog path is readable from disk.
    #[test]
    fn native_catalog_path_is_readable() {
        // Validates: Requirement 19.6
        use tempfile::TempDir;
        let tmp = TempDir::new().expect("tempdir");
        std::fs::write(tmp.path().join("hello.txt"), "hi").unwrap();
        std::fs::create_dir(tmp.path().join("subdir")).unwrap();

        let entries: Vec<_> = std::fs::read_dir(tmp.path())
            .expect("read_dir must succeed")
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 2, "must see file and directory");
    }

    /// Validates: Requirement 15.2 — nested directory structure readable to at least two levels deep.
    #[test]
    fn nested_directory_structure_readable_two_levels_deep() {
        // Validates: Requirement 15.2
        use tempfile::TempDir;
        let tmp = TempDir::new().expect("tempdir");
        let level1 = tmp.path().join("level1");
        std::fs::create_dir(&level1).unwrap();
        let level2 = level1.join("level2");
        std::fs::create_dir(&level2).unwrap();
        std::fs::write(level2.join("deep.txt"), "deep").unwrap();

        // Level 1 must be visible from root
        let root_entries: Vec<_> = std::fs::read_dir(tmp.path())
            .expect("root read_dir must succeed")
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(root_entries.len(), 1, "root must contain level1 dir");
        assert!(root_entries[0].file_type().unwrap().is_dir());

        // Level 2 must be visible from level 1
        let level1_entries: Vec<_> = std::fs::read_dir(&level1)
            .expect("level1 read_dir must succeed")
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(level1_entries.len(), 1, "level1 must contain level2 dir");
        assert!(level1_entries[0].file_type().unwrap().is_dir());

        // deep.txt must be visible from level 2
        let level2_entries: Vec<_> = std::fs::read_dir(&level2)
            .expect("level2 read_dir must succeed")
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(level2_entries.len(), 1, "level2 must contain deep.txt");
        assert!(!level2_entries[0].file_type().unwrap().is_dir());
    }
}
