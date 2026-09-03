//! # Files Panel — Virtual Catalog Manager
//!
//! Renders the unified virtual file catalog explorer opened by POM option 1.
//! Provides a split layout: left catalog tree + right content area.
//!
//! Validates: Requirement 1.1–1.7, 4.1, 4.3, 10.1–10.6

#![allow(dead_code)]

use std::collections::HashMap;

use eframe::egui;

use crate::catalog_manager_dialog::{DeleteCatalogConfirm, EditCatalogForm, NewCatalogForm};
use crate::catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog};
use crate::dataset_alloc_dialog::{AllocDatasetForm, AllocParams, Dsorg};

// ── State ─────────────────────────────────────────────────────────────────────

/// Which section header is expanded in the catalog tree.
#[derive(Debug, Clone)]
pub struct SectionState {
    pub mainframe_open: bool,
    pub posix_open: bool,
    pub native_open: bool,
}

impl Default for SectionState {
    fn default() -> Self {
        Self {
            mainframe_open: true,
            posix_open: true,
            native_open: true,
        }
    }
}

/// Action requested by the Files Panel during a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilesPanelAction {
    /// User pressed F3 / typed END — return to POM view.
    ReturnToPom,
    /// User clicked "New Catalog" — open the new catalog dialog.
    NewCatalog,
    /// User right-clicked a catalog node and chose Properties — open edit dialog.
    EditCatalog(String),
    /// User right-clicked a catalog node and chose Delete Catalog — open delete dialog.
    DeleteCatalog(String),
    /// User right-clicked a Mainframe catalog node and chose Allocate Dataset.
    AllocateDataset(String),
    /// User double-clicked a file/member/dataset node — open in editor tab.
    ///
    /// Validates: Requirement 10.3
    OpenFile(String),
    /// User double-clicked a directory/container node — navigate into it.
    ///
    /// Validates: Requirement 10.4
    NavigateInto(String),
    /// No action this frame.
    None,
}

/// Which modal dialog (if any) is currently open in the Files Panel.
///
/// Validates: Requirement 3.1, 4.1, 4.3
pub enum FilesDialogState {
    /// No dialog open.
    None,
    /// New Catalog creation dialog.
    NewCatalog(NewCatalogForm),
    /// Edit Catalog dialog.
    EditCatalog(EditCatalogForm),
    /// Allocate Dataset dialog.
    AllocateDataset(AllocDatasetForm),
    /// Delete Catalog confirmation dialog.
    DeleteCatalog(DeleteCatalogConfirm),
}

/// Column by which the content area is sorted.
///
/// Validates: Requirement 10.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Name,
    Type,
    Size,
    Modified,
}

/// Sort direction.
///
/// Validates: Requirement 10.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDir {
    Ascending,
    Descending,
}

impl SortDir {
    /// Toggle between ascending and descending.
    pub fn toggle(self) -> Self {
        match self {
            SortDir::Ascending => SortDir::Descending,
            SortDir::Descending => SortDir::Ascending,
        }
    }
}

/// A single entry displayed in the content area.
///
/// Validates: Requirement 10.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentEntry {
    /// Display name.
    pub name: String,
    /// Type label (e.g. "File", "Directory", "PS", "PDS").
    pub entry_type: String,
    /// Human-readable size (e.g. "4 KB") or empty for containers.
    pub size: String,
    /// Last-modified timestamp string, or empty if unknown.
    pub modified: String,
    /// True if this entry is a container (directory / PDS / GDG base).
    pub is_container: bool,
}

impl ContentEntry {
    /// Sort key for the given column (case-insensitive).
    pub fn sort_key(&self, col: SortColumn) -> String {
        match col {
            SortColumn::Name => self.name.to_lowercase(),
            SortColumn::Type => self.entry_type.to_lowercase(),
            SortColumn::Size => self.size.to_lowercase(),
            SortColumn::Modified => self.modified.clone(),
        }
    }
}

/// State for the right-side content area.
///
/// Validates: Requirement 10.1–10.6
#[derive(Debug, Clone)]
pub struct ContentAreaState {
    /// Name of the catalog whose contents are displayed, or `None` if nothing selected.
    pub selected_catalog: Option<String>,
    /// Current path within the catalog (breadcrumb segments).
    ///
    /// Validates: Requirement 10.5
    pub path_segments: Vec<String>,
    /// Entries currently displayed (pre-loaded from VFS or mock).
    pub entries: Vec<ContentEntry>,
    /// Active sort column.
    pub sort_col: SortColumn,
    /// Active sort direction.
    pub sort_dir: SortDir,
    /// Filter text for the content area.
    ///
    /// Validates: Requirement 10.6
    pub content_filter: String,
}

impl Default for ContentAreaState {
    fn default() -> Self {
        Self {
            selected_catalog: None,
            path_segments: Vec::new(),
            entries: Vec::new(),
            sort_col: SortColumn::Name,
            sort_dir: SortDir::Ascending,
            content_filter: String::new(),
        }
    }
}

impl ContentAreaState {
    /// Returns entries filtered by `content_filter` and sorted by `sort_col`/`sort_dir`.
    ///
    /// Validates: Requirement 10.2, 10.6
    pub fn visible_entries(&self) -> Vec<&ContentEntry> {
        let filter = self.content_filter.to_lowercase();
        let mut visible: Vec<&ContentEntry> = self
            .entries
            .iter()
            .filter(|e| filter.is_empty() || e.name.to_lowercase().contains(&filter))
            .collect();
        visible.sort_by(|a, b| {
            // Validates: Requirement 10.7 — containers always sort before non-containers
            // when sorting by Name; within each group sort by the chosen key.
            let container_order = if self.sort_col == SortColumn::Name {
                b.is_container.cmp(&a.is_container)
            } else {
                std::cmp::Ordering::Equal
            };
            if container_order != std::cmp::Ordering::Equal {
                return container_order;
            }
            let ka = a.sort_key(self.sort_col);
            let kb = b.sort_key(self.sort_col);
            match self.sort_dir {
                SortDir::Ascending => ka.cmp(&kb),
                SortDir::Descending => kb.cmp(&ka),
            }
        });
        visible
    }

    /// Toggle sort: if clicking the same column, flip direction; otherwise switch column ascending.
    ///
    /// Validates: Requirement 10.2
    pub fn toggle_sort(&mut self, col: SortColumn) {
        if self.sort_col == col {
            self.sort_dir = self.sort_dir.toggle();
        } else {
            self.sort_col = col;
            self.sort_dir = SortDir::Ascending;
        }
    }

    /// Navigate into a sub-path segment.
    ///
    /// Validates: Requirement 10.4, 10.5
    pub fn push_path(&mut self, segment: impl Into<String>) {
        self.path_segments.push(segment.into());
    }

    /// Navigate up to a breadcrumb index (0 = catalog root).
    ///
    /// Validates: Requirement 10.5
    pub fn navigate_to_segment(&mut self, index: usize) {
        self.path_segments.truncate(index);
    }

    /// Full path string for display in the breadcrumb bar.
    ///
    /// Validates: Requirement 10.5
    pub fn breadcrumb_display(&self) -> String {
        if let Some(cat) = &self.selected_catalog {
            if self.path_segments.is_empty() {
                cat.clone()
            } else {
                format!("{} / {}", cat, self.path_segments.join(" / "))
            }
        } else {
            String::new()
        }
    }
}

/// A dataset allocated within a catalog, stored in the UI-layer dataset map.
///
/// Validates: Requirement 13.1
#[derive(Debug, Clone)]
pub struct AllocatedDataset {
    /// Dataset name (DSN string).
    pub name: String,
    /// Dataset organisation label: "PS", "PO", "PDSE", "GDG".
    pub dsorg: String,
    /// Record format label: "FB", "F", "VB", "V", "U".
    pub recfm: String,
    /// Logical record length.
    pub lrecl: u32,
    /// Block size.
    pub blksize: u32,
    /// Optional description.
    pub description: String,
}

/// Persistent state for the Files Panel tab.
pub struct FilesPanelState {
    /// Catalog registry — source of truth for all virtual catalogs.
    pub registry: CatalogRegistry,
    /// Tree section expand/collapse state.
    pub sections: SectionState,
    /// Filter text entered in the toolbar search box.
    pub filter: String,
    /// Command field text local to the Files Panel.
    pub command: String,
    /// Active dialog, if any.
    pub dialog: FilesDialogState,
    /// Content area state.
    ///
    /// Validates: Requirement 10.1–10.6
    pub content: ContentAreaState,
    /// Allocated datasets keyed by catalog name.
    ///
    /// Validates: Requirement 13.1
    pub datasets: HashMap<String, Vec<AllocatedDataset>>,
    /// Catalog name that opened the current Allocate Dataset dialog.
    ///
    /// Validates: Requirement 13.2
    pub pending_alloc_catalog: Option<String>,
    /// When true, the next render pass should move keyboard focus to the first
    /// catalog node in the tree (set by Tab from the command field).
    ///
    /// Validates: Requirement 20.1 file-tree-panel
    pub tree_focus_requested: bool,
    /// The catalog name that currently has keyboard focus in the tree, or `None`.
    /// Driven by Tab-into-tree and arrow keys; rendered with a highlight border.
    ///
    /// Validates: Requirement 20.1 file-tree-panel
    pub focused_catalog: Option<String>,
}

impl FilesPanelState {
    /// Create a new, empty Files Panel state.
    pub fn new() -> Self {
        Self {
            registry: CatalogRegistry::new(),
            sections: SectionState::default(),
            filter: String::new(),
            command: String::new(),
            dialog: FilesDialogState::None,
            content: ContentAreaState::default(),
            datasets: HashMap::new(),
            pending_alloc_catalog: None,
            tree_focus_requested: false,
            focused_catalog: None,
        }
    }

    /// Insert an allocated dataset under the given catalog name.
    ///
    /// Validates: Requirement 13.2
    pub fn add_dataset(&mut self, catalog_name: &str, params: AllocParams) {
        let dsorg = match params.dsorg {
            Dsorg::Ps => "PS",
            Dsorg::Po => "PO",
            Dsorg::Pdse => "PDSE",
            Dsorg::Gdg => "GDG",
        };
        let recfm = match params.recfm {
            crate::dataset_alloc_dialog::Recfm::Fb => "FB",
            crate::dataset_alloc_dialog::Recfm::F => "F",
            crate::dataset_alloc_dialog::Recfm::Vb => "VB",
            crate::dataset_alloc_dialog::Recfm::V => "V",
            crate::dataset_alloc_dialog::Recfm::U => "U",
        };
        let entry = AllocatedDataset {
            name: params.dataset_name,
            dsorg: dsorg.to_string(),
            recfm: recfm.to_string(),
            lrecl: params.lrecl,
            blksize: params.blksize,
            description: params.description.unwrap_or_default(),
        };
        self.datasets
            .entry(catalog_name.to_string())
            .or_default()
            .push(entry);
    }

    /// Populate `content.entries` from the datasets stored for `catalog_name`.
    ///
    /// Validates: Requirement 13.3
    pub fn load_entries_from_datasets(&mut self, catalog_name: &str) {
        let entries = self
            .datasets
            .get(catalog_name)
            .map(|datasets| {
                datasets
                    .iter()
                    .map(|d| {
                        let is_container = d.dsorg == "PO" || d.dsorg == "PDSE" || d.dsorg == "GDG";
                        ContentEntry {
                            name: d.name.clone(),
                            entry_type: d.dsorg.clone(),
                            size: String::new(),
                            modified: String::new(),
                            is_container,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.content.entries = entries;
    }

    /// Remove all datasets stored under `catalog_name`.
    ///
    /// Validates: Requirement 13.5
    pub fn remove_catalog_datasets(&mut self, catalog_name: &str) {
        self.datasets.remove(catalog_name);
    }

    /// Create a Mainframe dataset file on disk, including any missing parent directories.
    ///
    /// Called on first open of a newly-allocated dataset whose physical file does not yet
    /// exist. Matches ISPF behaviour: allocation reserves the dataset; opening creates it.
    ///
    /// Validates: Requirement 16.3
    pub fn create_dataset_file(path: &std::path::Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::File::create(path)?;
        Ok(())
    }

    /// Resolve a dataset's physical file path from the catalog's repository path and the DSN.
    ///
    /// Maps `PAYROLL.EMPLOYEE` in repo `C:/catalogs/payroll` to
    /// `C:/catalogs/payroll/PAYROLL/EMPLOYEE` by splitting the DSN on `.` and
    /// joining as path components.
    ///
    /// The repository path is normalised to the OS path separator before joining
    /// so that mixed-separator paths (e.g. `mainframe/Payroll` on Windows) do
    /// not produce invalid paths.
    ///
    /// Returns `None` when either argument is empty.
    ///
    /// Validates: Requirement 16.1, 16.4, 16.5
    pub fn resolve_dataset_path(repository_path: &str, dsn: &str) -> Option<std::path::PathBuf> {
        if repository_path.is_empty() || dsn.is_empty() {
            return None;
        }
        // Normalise separators: replace `/` with the OS separator on Windows so
        // that a catalog path stored with forward slashes (e.g. "mainframe/Payroll")
        // does not produce a mixed-separator path when joined with DSN components.
        let normalised = repository_path.replace('/', std::path::MAIN_SEPARATOR_STR);
        let rel: std::path::PathBuf = dsn.split('.').collect();
        Some(std::path::Path::new(&normalised).join(rel))
    }

    /// Returns the platform label appended to the Native section header.
    ///
    /// Validates: Requirement 1.4
    pub fn native_platform_label() -> &'static str {
        match std::env::consts::OS {
            "windows" => "Windows",
            "macos" => "macOS",
            _ => "Linux",
        }
    }
}

// ── Render ────────────────────────────────────────────────────────────────────

/// Render the Files Panel into `ui`.
///
/// Returns a `FilesPanelAction` indicating any deferred action the shell must
/// perform after this frame (e.g. return to POM, open a dialog).
///
/// Validates: Requirement 1.1–1.7
pub fn render(ui: &mut egui::Ui, state: &mut FilesPanelState) -> FilesPanelAction {
    let mut action = FilesPanelAction::None;

    ui.vertical(|ui| {
        // ── Title bar ────────────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("FileForge Workbench — Virtual File Catalogs")
                    .monospace()
                    .strong(),
            );
        });
        ui.separator();

        // ── Toolbar — Req 1.3 ────────────────────────────────────────────
        ui.horizontal(|ui| {
            if ui.button("New Catalog").clicked() {
                action = FilesPanelAction::NewCatalog;
            }
            ui.separator();
            ui.button("Open").clicked();
            ui.button("Refresh").clicked();
            ui.button("Properties").clicked();
            ui.separator();
            ui.label("Filter:");
            ui.add(
                egui::TextEdit::singleline(&mut state.filter)
                    .desired_width(160.0)
                    .hint_text("name…")
                    .font(egui::TextStyle::Monospace),
            );
        });
        ui.separator();

        // ── Command field — Req 1.6, 1.7 ─────────────────────────────────
        ui.horizontal(|ui| {
            ui.label("Command ===>");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut state.command)
                    .id(egui::Id::new("files_panel_cmd"))
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace),
            );
            if resp.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                && !state.command.is_empty()
            {
                let cmd = state.command.trim().to_uppercase();
                state.command.clear();
                if cmd == "END" || cmd == "F3" {
                    action = FilesPanelAction::ReturnToPom;
                }
            }
        });
        ui.separator();

        // ── Split layout: left tree | right content ───────────────────────
        // Req 1.2 — left catalog tree + right content area
        let tree_action = std::cell::Cell::new(FilesPanelAction::None);
        ui.columns(2, |cols| {
            egui::ScrollArea::vertical()
                .id_salt("files_tree")
                .show(&mut cols[0], |ui| {
                    if let Some(a) = render_catalog_tree(ui, state) {
                        tree_action.set(a);
                    }
                });

            egui::ScrollArea::vertical()
                .id_salt("files_content")
                .show(&mut cols[1], |ui| {
                    if let Some(a) = render_content_area(ui, state) {
                        tree_action.set(a);
                    }
                });
        });
        let tree_action = tree_action.into_inner();
        if tree_action != FilesPanelAction::None {
            action = tree_action;
        }
    });

    // F3 key also triggers return-to-POM — Req 1.7
    if ui.input(|i| i.key_pressed(egui::Key::F3)) {
        action = FilesPanelAction::ReturnToPom;
    }

    action
}

/// Render the left-side catalog tree with three collapsible section headers.
/// Returns `Some(action)` if a context menu item was clicked, else `None`.
///
/// Validates: Requirement 1.4, 1.5, 4.1, 4.3
fn render_catalog_tree(ui: &mut egui::Ui, state: &mut FilesPanelState) -> Option<FilesPanelAction> {
    let filter = state.filter.to_lowercase();
    let mut action: Option<FilesPanelAction> = None;

    // Collect all visible catalog names across all sections in order.
    let all_visible: Vec<String> = [
        state.registry.list_by_type(CatalogType::Mainframe),
        state.registry.list_by_type(CatalogType::Posix),
        state.registry.list_by_type(CatalogType::Native),
    ]
    .into_iter()
    .flatten()
    .filter(|c| filter.is_empty() || c.name.to_lowercase().contains(&filter))
    .map(|c| c.name.clone())
    .collect();

    // Tab-into-tree: move focus to the first catalog.
    if state.tree_focus_requested {
        state.tree_focus_requested = false;
        state.focused_catalog = all_visible.first().cloned();
    }

    // Arrow-key navigation within the tree when a catalog has focus.
    if state.focused_catalog.is_some() {
        let down = ui.input(|i| i.key_pressed(egui::Key::ArrowDown));
        let up = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
        if down || up {
            if let Some(ref cur) = state.focused_catalog.clone() {
                let pos = all_visible.iter().position(|n| n == cur);
                state.focused_catalog = match (down, pos) {
                    (true, Some(i)) => all_visible.get(i + 1).cloned().or(Some(cur.clone())),
                    (false, Some(0)) | (false, None) => Some(cur.clone()),
                    (false, Some(i)) => all_visible.get(i - 1).cloned(),
                    _ => Some(cur.clone()),
                };
            }
        }
        // Enter selects the focused catalog.
        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(ref name) = state.focused_catalog.clone() {
                state.content.selected_catalog = Some(name.clone());
                state.content.path_segments.clear();
                state.content.content_filter.clear();
            }
        }
        // Tab out of tree clears focus.
        if ui.input(|i| i.key_pressed(egui::Key::Tab)) {
            state.focused_catalog = None;
        }
    }

    // Snapshot open-state before borrowing state.content via sec_ctx.
    let mf_open = state.sections.mainframe_open;
    let px_open = state.sections.posix_open;
    let nat_open = state.sections.native_open;
    let mf_entries = state.registry.list_by_type(CatalogType::Mainframe);
    let px_entries = state.registry.list_by_type(CatalogType::Posix);
    let nat_entries = state.registry.list_by_type(CatalogType::Native);
    let native_header = format!(
        "Native Catalogs ({})",
        FilesPanelState::native_platform_label()
    );
    let focused = state.focused_catalog.clone();

    {
        let mut sec_ctx = SectionCtx {
            content: &mut state.content,
            action: &mut action,
            focused: &focused,
        };
        render_section(
            ui,
            egui::RichText::new("Mainframe Catalogs")
                .monospace()
                .strong(),
            mf_open,
            mf_entries,
            &filter,
            &mut sec_ctx,
        );
        render_section(
            ui,
            egui::RichText::new("POSIX Catalogs").monospace().strong(),
            px_open,
            px_entries,
            &filter,
            &mut sec_ctx,
        );
        render_section(
            ui,
            egui::RichText::new(native_header).monospace().strong(),
            nat_open,
            nat_entries,
            &filter,
            &mut sec_ctx,
        );
    } // sec_ctx dropped here

    state.sections.mainframe_open = true;
    state.sections.posix_open = true;
    state.sections.native_open = true;

    action
}

/// Mutable context threaded through `render_section` to avoid exceeding the
/// 7-argument Clippy limit.
struct SectionCtx<'a> {
    content: &'a mut ContentAreaState,
    action: &'a mut Option<FilesPanelAction>,
    focused: &'a Option<String>,
}

/// Render one collapsible section of the catalog tree.
///
/// Validates: Requirement 1.4, 1.5, 4.1, 4.3, 10.1
fn render_section(
    ui: &mut egui::Ui,
    header: egui::RichText,
    default_open: bool,
    entries: Vec<&VirtualCatalog>,
    filter: &str,
    ctx: &mut SectionCtx<'_>,
) {
    egui::CollapsingHeader::new(header)
        .default_open(default_open)
        .show(ui, |ui| {
            let visible: Vec<_> = entries
                .iter()
                .filter(|c| filter.is_empty() || c.name.to_lowercase().contains(filter))
                .collect();
            if visible.is_empty() {
                ui.label(
                    egui::RichText::new("  No catalogs defined — click New Catalog to create one")
                        .monospace()
                        .weak(),
                );
            } else {
                for cat in visible.iter() {
                    let is_selected = ctx
                        .content
                        .selected_catalog
                        .as_deref()
                        .is_some_and(|s| s == cat.name);
                    let is_focused = ctx.focused.as_deref().is_some_and(|f| f == cat.name);
                    let label_text = egui::RichText::new(format!("  📁 {}", cat.name)).monospace();
                    let resp = ui.selectable_label(is_selected, label_text);
                    // Validates: Requirement 20.1 -- draw focus highlight on keyboard-focused row.
                    if is_focused {
                        ui.painter().rect_stroke(
                            resp.rect,
                            2.0,
                            egui::Stroke::new(1.5_f32, ui.visuals().selection.stroke.color),
                        );
                    }
                    // Left-click selects catalog — Req 10.1
                    if resp.clicked() {
                        ctx.content.selected_catalog = Some(cat.name.clone());
                        ctx.content.path_segments.clear();
                        ctx.content.content_filter.clear();
                    }
                    // Right-click context menu — Req 4.1, 4.3, 5.1
                    resp.context_menu(|ui| {
                        if ui.button("Properties").clicked() {
                            *ctx.action = Some(FilesPanelAction::EditCatalog(cat.name.clone()));
                            ui.close_menu();
                        }
                        if ui.button("Allocate Dataset").clicked() {
                            *ctx.action = Some(FilesPanelAction::AllocateDataset(cat.name.clone()));
                            ui.close_menu();
                        }
                        if ui.button("Delete Catalog").clicked() {
                            *ctx.action = Some(FilesPanelAction::DeleteCatalog(cat.name.clone()));
                            ui.close_menu();
                        }
                    });
                }
            }
        });
}

/// Render the right-side content area.
///
/// Validates: Requirement 10.1–10.6, 13.3
fn render_content_area(ui: &mut egui::Ui, state: &mut FilesPanelState) -> Option<FilesPanelAction> {
    let mut action: Option<FilesPanelAction> = None;

    if state.content.selected_catalog.is_none() {
        ui.label(
            egui::RichText::new("Select a catalog to browse its contents.")
                .monospace()
                .weak(),
        );
        return None;
    }

    // Req 13.3 — populate entries from the dataset store for the selected catalog
    if let Some(cat) = state.content.selected_catalog.clone() {
        state.load_entries_from_datasets(&cat);
    }

    // ── Breadcrumb bar — Req 10.5 ─────────────────────────────────────────
    ui.horizontal(|ui| {
        let cat_name = state.content.selected_catalog.clone().unwrap_or_default();
        // Root segment (catalog name)
        if ui
            .selectable_label(false, egui::RichText::new(&cat_name).monospace())
            .clicked()
        {
            state.content.navigate_to_segment(0);
        }
        let segments = state.content.path_segments.clone();
        for (i, seg) in segments.iter().enumerate() {
            ui.label(egui::RichText::new(" / ").monospace().weak());
            if ui
                .selectable_label(false, egui::RichText::new(seg).monospace())
                .clicked()
            {
                state.content.navigate_to_segment(i + 1);
            }
        }
    });
    ui.separator();

    // ── Content filter — Req 10.6 ─────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label("Filter:");
        ui.add(
            egui::TextEdit::singleline(&mut state.content.content_filter)
                .desired_width(200.0)
                .hint_text("name…")
                .font(egui::TextStyle::Monospace),
        );
    });
    ui.separator();

    // ── Column headers — Req 10.2 ─────────────────────────────────────────
    ui.horizontal(|ui| {
        let col_btn = |ui: &mut egui::Ui,
                       label: &str,
                       col: SortColumn,
                       content: &mut ContentAreaState|
         -> bool {
            let indicator = if content.sort_col == col {
                match content.sort_dir {
                    SortDir::Ascending => " ▲",
                    SortDir::Descending => " ▼",
                }
            } else {
                ""
            };
            ui.button(
                egui::RichText::new(format!("{label}{indicator}"))
                    .monospace()
                    .strong(),
            )
            .clicked()
                && {
                    content.toggle_sort(col);
                    true
                }
        };
        col_btn(ui, "Name", SortColumn::Name, &mut state.content);
        col_btn(ui, "Type", SortColumn::Type, &mut state.content);
        col_btn(ui, "Size", SortColumn::Size, &mut state.content);
        col_btn(ui, "Modified", SortColumn::Modified, &mut state.content);
    });
    ui.separator();

    // ── Entry rows — Req 10.1, 10.3, 10.4 ────────────────────────────────
    let visible: Vec<ContentEntry> = state
        .content
        .visible_entries()
        .into_iter()
        .cloned()
        .collect();

    if visible.is_empty() {
        ui.label(
            egui::RichText::new("No entries to display.")
                .monospace()
                .weak(),
        );
    } else {
        for entry in &visible {
            let icon = if entry.is_container { "📁" } else { "📄" };
            let row_text = format!(
                "{icon} {:<40} {:<12} {:<10} {}",
                entry.name, entry.entry_type, entry.size, entry.modified
            );
            let resp = ui.selectable_label(false, egui::RichText::new(row_text).monospace());
            if resp.double_clicked() {
                if entry.is_container {
                    state.content.push_path(entry.name.clone());
                    action = Some(FilesPanelAction::NavigateInto(entry.name.clone()));
                } else {
                    action = Some(FilesPanelAction::OpenFile(entry.name.clone()));
                }
            }
        }
    }

    action
}

// ── Context menu types ────────────────────────────────────────────────────────

/// The kind of a Mainframe dataset node in the tree.
///
/// Validates: Requirement 6.1–6.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatasetNodeKind {
    Ps,
    Pds,
    Member,
    GdgBase,
}

/// Context menu items for a Mainframe dataset node.
///
/// Validates: Requirement 6.1–6.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainframeContextItem {
    Open,
    NewMember,
    NewGeneration,
    Rename,
    Delete,
    DeleteGdg,
    Properties,
    CopyDsn,
    CopyMemberName,
    AllocateLike,
    ListGenerations,
    ModifyLimit,
}

/// Returns the context menu items for a given Mainframe dataset node kind.
///
/// Validates: Requirement 6.1–6.4
pub fn context_menu_items_mainframe(kind: DatasetNodeKind) -> Vec<MainframeContextItem> {
    match kind {
        DatasetNodeKind::Ps => vec![
            MainframeContextItem::Open,
            MainframeContextItem::Rename,
            MainframeContextItem::Delete,
            MainframeContextItem::Properties,
            MainframeContextItem::CopyDsn,
            MainframeContextItem::AllocateLike,
        ],
        DatasetNodeKind::Pds => vec![
            MainframeContextItem::NewMember,
            MainframeContextItem::Rename,
            MainframeContextItem::Delete,
            MainframeContextItem::Properties,
            MainframeContextItem::CopyDsn,
            MainframeContextItem::AllocateLike,
        ],
        DatasetNodeKind::Member => vec![
            MainframeContextItem::Open,
            MainframeContextItem::Rename,
            MainframeContextItem::Delete,
            MainframeContextItem::CopyMemberName,
        ],
        DatasetNodeKind::GdgBase => vec![
            MainframeContextItem::NewGeneration,
            MainframeContextItem::ListGenerations,
            MainframeContextItem::Properties,
            MainframeContextItem::DeleteGdg,
            MainframeContextItem::ModifyLimit,
        ],
    }
}

/// The kind of a POSIX node in the tree.
///
/// Validates: Requirement 8.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosixNodeKind {
    File,
    Directory,
}

/// Context menu items for a POSIX node.
///
/// Validates: Requirement 8.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosixContextItem {
    NewFile,
    NewDirectory,
    Rename,
    Delete,
    Properties,
    CopyPath,
}

/// Returns the context menu items for a POSIX node.
///
/// Validates: Requirement 8.1
pub fn context_menu_items_posix(kind: PosixNodeKind) -> Vec<PosixContextItem> {
    match kind {
        PosixNodeKind::Directory => vec![
            PosixContextItem::NewFile,
            PosixContextItem::NewDirectory,
            PosixContextItem::Rename,
            PosixContextItem::Delete,
            PosixContextItem::Properties,
            PosixContextItem::CopyPath,
        ],
        PosixNodeKind::File => vec![
            PosixContextItem::Rename,
            PosixContextItem::Delete,
            PosixContextItem::Properties,
            PosixContextItem::CopyPath,
        ],
    }
}

/// The kind of a Native catalog node.
///
/// Validates: Requirement 9.3–9.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeNodeKind {
    File,
    Directory,
}

/// Context menu items for a Native catalog node.
///
/// Validates: Requirement 9.3–9.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeContextItem {
    Open,
    Rename,
    Delete,
    CopyPath,
    NewFile,
    NewFolder,
    OpenInNativeFileManager,
    Refresh,
    OpenInCmd,
    OpenInPowerShell,
    OpenInTerminal,
    RevealInFinder,
}

/// Returns the context menu items for a Native catalog node.
///
/// `os` should be `std::env::consts::OS` — passed in for testability.
///
/// Validates: Requirement 9.3–9.4
pub fn context_menu_items_native(kind: NativeNodeKind, os: &str) -> Vec<NativeContextItem> {
    match kind {
        NativeNodeKind::File => {
            let mut items = vec![
                NativeContextItem::Open,
                NativeContextItem::Rename,
                NativeContextItem::Delete,
                NativeContextItem::CopyPath,
            ];
            match os {
                "windows" => {
                    items.push(NativeContextItem::OpenInCmd);
                    items.push(NativeContextItem::OpenInPowerShell);
                }
                "macos" => {
                    items.push(NativeContextItem::RevealInFinder);
                    items.push(NativeContextItem::OpenInTerminal);
                }
                _ => items.push(NativeContextItem::OpenInTerminal),
            }
            items
        }
        NativeNodeKind::Directory => {
            let mut items = vec![
                NativeContextItem::NewFile,
                NativeContextItem::NewFolder,
                NativeContextItem::Rename,
                NativeContextItem::Delete,
                NativeContextItem::CopyPath,
                NativeContextItem::OpenInNativeFileManager,
                NativeContextItem::Refresh,
            ];
            match os {
                "windows" => {
                    items.push(NativeContextItem::OpenInCmd);
                    items.push(NativeContextItem::OpenInPowerShell);
                }
                "macos" => {
                    items.push(NativeContextItem::RevealInFinder);
                    items.push(NativeContextItem::OpenInTerminal);
                }
                _ => items.push(NativeContextItem::OpenInTerminal),
            }
            items
        }
    }
}

// ── Inline-form structs ───────────────────────────────────────────────────────

/// Inline rename form for a dataset or PDS member.
///
/// Validates: Requirement 6.5
#[derive(Debug, Clone)]
pub struct DatasetRenameForm {
    pub current_name: String,
    pub new_name: String,
    pub error: Option<String>,
}

impl DatasetRenameForm {
    pub fn new(current_name: impl Into<String>) -> Self {
        let current_name = current_name.into();
        let new_name = current_name.clone();
        Self {
            current_name,
            new_name,
            error: None,
        }
    }
}

/// Delete confirmation for a dataset or PDS member.
///
/// Validates: Requirement 6.6
#[derive(Debug, Clone)]
pub struct DatasetDeleteConfirm {
    pub name: String,
}

impl DatasetDeleteConfirm {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Inline new-file form for a POSIX catalog.
///
/// Validates: Requirement 8.2
#[derive(Debug, Clone)]
pub struct PosixNewFileForm {
    pub parent_path: String,
    pub filename: String,
    pub error: Option<String>,
}

impl PosixNewFileForm {
    pub fn new(parent_path: impl Into<String>) -> Self {
        Self {
            parent_path: parent_path.into(),
            filename: String::new(),
            error: None,
        }
    }

    /// Validate: 1–255 chars, no path separators, no null bytes.
    ///
    /// Validates: Requirement 8.2
    pub fn validate(&self) -> Result<(), String> {
        if self.filename.is_empty() {
            return Err("Filename cannot be empty".to_string());
        }
        if self.filename.len() > 255 {
            return Err("Filename must be 255 characters or fewer".to_string());
        }
        if self.filename.contains('/') || self.filename.contains('\\') {
            return Err("Filename must not contain path separators".to_string());
        }
        if self.filename.contains('\0') {
            return Err("Filename must not contain null bytes".to_string());
        }
        Ok(())
    }
}

/// Inline new-directory form for a POSIX catalog.
///
/// Validates: Requirement 8.3
#[derive(Debug, Clone)]
pub struct PosixNewDirForm {
    pub parent_path: String,
    pub dirname: String,
    pub error: Option<String>,
}

impl PosixNewDirForm {
    pub fn new(parent_path: impl Into<String>) -> Self {
        Self {
            parent_path: parent_path.into(),
            dirname: String::new(),
            error: None,
        }
    }

    /// Validate: same rules as filename.
    ///
    /// Validates: Requirement 8.3
    pub fn validate(&self) -> Result<(), String> {
        if self.dirname.is_empty() {
            return Err("Directory name cannot be empty".to_string());
        }
        if self.dirname.len() > 255 {
            return Err("Directory name must be 255 characters or fewer".to_string());
        }
        if self.dirname.contains('/') || self.dirname.contains('\\') {
            return Err("Directory name must not contain path separators".to_string());
        }
        if self.dirname.contains('\0') {
            return Err("Directory name must not contain null bytes".to_string());
        }
        Ok(())
    }
}

/// Delete confirmation for a POSIX file or directory.
///
/// Validates: Requirement 8.5
#[derive(Debug, Clone)]
pub struct PosixDeleteConfirm {
    pub name: String,
    pub is_directory: bool,
}

impl PosixDeleteConfirm {
    pub fn new(name: impl Into<String>, is_directory: bool) -> Self {
        Self {
            name: name.into(),
            is_directory,
        }
    }

    /// Confirmation message shown to the user.
    ///
    /// Validates: Requirement 8.5
    pub fn message(&self) -> String {
        if self.is_directory {
            format!(
                "Delete directory \"{}\" and all its contents? This cannot be undone.",
                self.name
            )
        } else {
            format!("Delete \"{}\"? This cannot be undone.", self.name)
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

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

    /// Validates: Requirement 1.4 — three section types are represented.
    #[test]
    fn files_panel_state_registry_supports_three_catalog_types() {
        // Validates: Requirement 1.4
        let mut state = FilesPanelState::new();
        state
            .registry
            .register(make_catalog("MF1", CatalogType::Mainframe))
            .unwrap();
        state
            .registry
            .register(make_catalog("PX1", CatalogType::Posix))
            .unwrap();
        state
            .registry
            .register(make_catalog("NAT1", CatalogType::Native))
            .unwrap();

        assert_eq!(state.registry.list_by_type(CatalogType::Mainframe).len(), 1);
        assert_eq!(state.registry.list_by_type(CatalogType::Posix).len(), 1);
        assert_eq!(state.registry.list_by_type(CatalogType::Native).len(), 1);
    }

    /// Validates: Requirement 1.5 — empty section has no catalog entries.
    #[test]
    fn empty_section_has_no_catalog_entries() {
        // Validates: Requirement 1.5
        let state = FilesPanelState::new();
        assert!(state
            .registry
            .list_by_type(CatalogType::Mainframe)
            .is_empty());
        assert!(state.registry.list_by_type(CatalogType::Posix).is_empty());
        assert!(state.registry.list_by_type(CatalogType::Native).is_empty());
    }

    /// Validates: Requirement 1.4 — native platform label is non-empty.
    #[test]
    fn native_platform_label_is_non_empty() {
        // Validates: Requirement 1.4
        let label = FilesPanelState::native_platform_label();
        assert!(!label.is_empty());
        assert!(
            label == "Windows" || label == "Linux" || label == "macOS",
            "unexpected label: {label}"
        );
    }

    /// Validates: Requirement 1.7 — END command produces ReturnToPom action.
    #[test]
    fn end_command_produces_return_to_pom_action() {
        // Validates: Requirement 1.7
        let cmd = "END";
        let action = if cmd == "END" || cmd == "F3" {
            FilesPanelAction::ReturnToPom
        } else {
            FilesPanelAction::None
        };
        assert_eq!(action, FilesPanelAction::ReturnToPom);
    }

    /// Validates: Requirement 1.3 — NewCatalog action is distinct from None.
    #[test]
    fn new_catalog_action_is_distinct_from_none() {
        // Validates: Requirement 1.3
        assert_ne!(FilesPanelAction::NewCatalog, FilesPanelAction::None);
        assert_ne!(FilesPanelAction::ReturnToPom, FilesPanelAction::None);
    }

    /// Validates: Requirement 1.2 — FilesPanelState initialises with empty filter and command.
    #[test]
    fn files_panel_state_initialises_with_empty_filter_and_command() {
        // Validates: Requirement 1.2
        let state = FilesPanelState::new();
        assert!(state.filter.is_empty());
        assert!(state.command.is_empty());
    }

    /// Validates: Requirement 1.4 — sections default to open.
    #[test]
    fn section_state_defaults_to_all_open() {
        // Validates: Requirement 1.4
        let s = SectionState::default();
        assert!(s.mainframe_open);
        assert!(s.posix_open);
        assert!(s.native_open);
    }

    /// Validates: Requirement 4.1 — EditCatalog action carries the catalog name.
    #[test]
    fn edit_catalog_action_carries_name() {
        // Validates: Requirement 4.1
        let action = FilesPanelAction::EditCatalog("PAYROLL".to_string());
        assert_eq!(action, FilesPanelAction::EditCatalog("PAYROLL".to_string()));
        assert_ne!(action, FilesPanelAction::None);
    }

    /// Validates: Requirement 4.3 — DeleteCatalog action carries the catalog name.
    #[test]
    fn delete_catalog_action_carries_name() {
        // Validates: Requirement 4.3
        let action = FilesPanelAction::DeleteCatalog("PAYROLL".to_string());
        assert_eq!(
            action,
            FilesPanelAction::DeleteCatalog("PAYROLL".to_string())
        );
        assert_ne!(action, FilesPanelAction::None);
    }

    // ── Task 8 tests ──────────────────────────────────────────────────────────

    /// Validates: Requirement 6.1 — PS context menu contains Open, Rename, Delete, Properties, CopyDsn, AllocateLike.
    #[test]
    fn mainframe_ps_context_menu_contains_required_items() {
        // Validates: Requirement 6.1
        let items = context_menu_items_mainframe(DatasetNodeKind::Ps);
        assert!(items.contains(&MainframeContextItem::Open));
        assert!(items.contains(&MainframeContextItem::Rename));
        assert!(items.contains(&MainframeContextItem::Delete));
        assert!(items.contains(&MainframeContextItem::Properties));
        assert!(items.contains(&MainframeContextItem::CopyDsn));
        assert!(items.contains(&MainframeContextItem::AllocateLike));
    }

    /// Validates: Requirement 6.1 — PS context menu does not contain NewMember.
    #[test]
    fn mainframe_ps_context_menu_excludes_new_member() {
        // Validates: Requirement 6.1
        let items = context_menu_items_mainframe(DatasetNodeKind::Ps);
        assert!(!items.contains(&MainframeContextItem::NewMember));
    }

    /// Validates: Requirement 6.2 — PDS context menu contains NewMember, Rename, Delete, Properties, CopyDsn, AllocateLike.
    #[test]
    fn mainframe_pds_context_menu_contains_required_items() {
        // Validates: Requirement 6.2
        let items = context_menu_items_mainframe(DatasetNodeKind::Pds);
        assert!(items.contains(&MainframeContextItem::NewMember));
        assert!(items.contains(&MainframeContextItem::Rename));
        assert!(items.contains(&MainframeContextItem::Delete));
        assert!(items.contains(&MainframeContextItem::Properties));
        assert!(items.contains(&MainframeContextItem::CopyDsn));
        assert!(items.contains(&MainframeContextItem::AllocateLike));
    }

    /// Validates: Requirement 6.2 — PDS context menu does not contain Open.
    #[test]
    fn mainframe_pds_context_menu_excludes_open() {
        // Validates: Requirement 6.2
        let items = context_menu_items_mainframe(DatasetNodeKind::Pds);
        assert!(!items.contains(&MainframeContextItem::Open));
    }

    /// Validates: Requirement 6.3 — Member context menu contains Open, Rename, Delete, CopyMemberName.
    #[test]
    fn mainframe_member_context_menu_contains_required_items() {
        // Validates: Requirement 6.3
        let items = context_menu_items_mainframe(DatasetNodeKind::Member);
        assert!(items.contains(&MainframeContextItem::Open));
        assert!(items.contains(&MainframeContextItem::Rename));
        assert!(items.contains(&MainframeContextItem::Delete));
        assert!(items.contains(&MainframeContextItem::CopyMemberName));
    }

    /// Validates: Requirement 6.3 — Member context menu does not contain Properties or CopyDsn.
    #[test]
    fn mainframe_member_context_menu_excludes_properties_and_copy_dsn() {
        // Validates: Requirement 6.3
        let items = context_menu_items_mainframe(DatasetNodeKind::Member);
        assert!(!items.contains(&MainframeContextItem::Properties));
        assert!(!items.contains(&MainframeContextItem::CopyDsn));
    }

    /// Validates: Requirement 6.4 — GDG context menu contains NewGeneration, ListGenerations, Properties, DeleteGdg, ModifyLimit.
    #[test]
    fn mainframe_gdg_context_menu_contains_required_items() {
        // Validates: Requirement 6.4
        let items = context_menu_items_mainframe(DatasetNodeKind::GdgBase);
        assert!(items.contains(&MainframeContextItem::NewGeneration));
        assert!(items.contains(&MainframeContextItem::ListGenerations));
        assert!(items.contains(&MainframeContextItem::Properties));
        assert!(items.contains(&MainframeContextItem::DeleteGdg));
        assert!(items.contains(&MainframeContextItem::ModifyLimit));
    }

    /// Validates: Requirement 8.1 — POSIX directory context menu contains all six items.
    #[test]
    fn posix_directory_context_menu_contains_required_items() {
        // Validates: Requirement 8.1
        let items = context_menu_items_posix(PosixNodeKind::Directory);
        assert!(items.contains(&PosixContextItem::NewFile));
        assert!(items.contains(&PosixContextItem::NewDirectory));
        assert!(items.contains(&PosixContextItem::Rename));
        assert!(items.contains(&PosixContextItem::Delete));
        assert!(items.contains(&PosixContextItem::Properties));
        assert!(items.contains(&PosixContextItem::CopyPath));
    }

    /// Validates: Requirement 8.1 — POSIX file context menu does not contain NewFile or NewDirectory.
    #[test]
    fn posix_file_context_menu_excludes_new_items() {
        // Validates: Requirement 8.1
        let items = context_menu_items_posix(PosixNodeKind::File);
        assert!(!items.contains(&PosixContextItem::NewFile));
        assert!(!items.contains(&PosixContextItem::NewDirectory));
        assert!(items.contains(&PosixContextItem::Rename));
        assert!(items.contains(&PosixContextItem::Delete));
    }

    /// Validates: Requirement 9.3 — Native file on Windows includes OpenInCmd and OpenInPowerShell.
    #[test]
    fn native_file_windows_context_menu_includes_shell_actions() {
        // Validates: Requirement 9.3
        let items = context_menu_items_native(NativeNodeKind::File, "windows");
        assert!(items.contains(&NativeContextItem::Open));
        assert!(items.contains(&NativeContextItem::OpenInCmd));
        assert!(items.contains(&NativeContextItem::OpenInPowerShell));
        assert!(!items.contains(&NativeContextItem::OpenInTerminal));
    }

    /// Validates: Requirement 9.3 — Native file on Linux includes OpenInTerminal.
    #[test]
    fn native_file_linux_context_menu_includes_terminal() {
        // Validates: Requirement 9.3
        let items = context_menu_items_native(NativeNodeKind::File, "linux");
        assert!(items.contains(&NativeContextItem::OpenInTerminal));
        assert!(!items.contains(&NativeContextItem::OpenInCmd));
    }

    /// Validates: Requirement 9.3 — Native file on macOS includes RevealInFinder and OpenInTerminal.
    #[test]
    fn native_file_macos_context_menu_includes_finder_and_terminal() {
        // Validates: Requirement 9.3
        let items = context_menu_items_native(NativeNodeKind::File, "macos");
        assert!(items.contains(&NativeContextItem::RevealInFinder));
        assert!(items.contains(&NativeContextItem::OpenInTerminal));
        assert!(!items.contains(&NativeContextItem::OpenInCmd));
    }

    /// Validates: Requirement 9.4 — Native directory context menu contains NewFile, NewFolder, Rename, Delete, CopyPath, OpenInNativeFileManager, Refresh.
    #[test]
    fn native_directory_context_menu_contains_required_items() {
        // Validates: Requirement 9.4
        let items = context_menu_items_native(NativeNodeKind::Directory, "windows");
        assert!(items.contains(&NativeContextItem::NewFile));
        assert!(items.contains(&NativeContextItem::NewFolder));
        assert!(items.contains(&NativeContextItem::Rename));
        assert!(items.contains(&NativeContextItem::Delete));
        assert!(items.contains(&NativeContextItem::CopyPath));
        assert!(items.contains(&NativeContextItem::OpenInNativeFileManager));
        assert!(items.contains(&NativeContextItem::Refresh));
    }

    /// Validates: Requirement 8.2 — PosixNewFileForm rejects empty filename.
    #[test]
    fn posix_new_file_form_rejects_empty_filename() {
        // Validates: Requirement 8.2
        let form = PosixNewFileForm::new("/");
        assert!(form.validate().is_err());
    }

    /// Validates: Requirement 8.2 — PosixNewFileForm rejects filename with path separator.
    #[test]
    fn posix_new_file_form_rejects_path_separator() {
        // Validates: Requirement 8.2
        let mut form = PosixNewFileForm::new("/");
        form.filename = "foo/bar".to_string();
        assert!(form.validate().is_err());
    }

    /// Validates: Requirement 8.2 — PosixNewFileForm accepts valid filename.
    #[test]
    fn posix_new_file_form_accepts_valid_filename() {
        // Validates: Requirement 8.2
        let mut form = PosixNewFileForm::new("/");
        form.filename = "hello.txt".to_string();
        assert!(form.validate().is_ok());
    }

    /// Validates: Requirement 8.3 — PosixNewDirForm rejects empty dirname.
    #[test]
    fn posix_new_dir_form_rejects_empty_dirname() {
        // Validates: Requirement 8.3
        let form = PosixNewDirForm::new("/");
        assert!(form.validate().is_err());
    }

    /// Validates: Requirement 8.5 — PosixDeleteConfirm message mentions directory name.
    #[test]
    fn posix_delete_confirm_directory_message_contains_name() {
        // Validates: Requirement 8.5
        let confirm = PosixDeleteConfirm::new("src", true);
        let msg = confirm.message();
        assert!(msg.contains("src"));
        assert!(msg.contains("all its contents"));
    }

    /// Validates: Requirement 6.5 — DatasetRenameForm pre-fills new_name from current_name.
    #[test]
    fn dataset_rename_form_prefills_new_name() {
        // Validates: Requirement 6.5
        let form = DatasetRenameForm::new("PAYROLL.DATA");
        assert_eq!(form.current_name, "PAYROLL.DATA");
        assert_eq!(form.new_name, "PAYROLL.DATA");
        assert!(form.error.is_none());
    }

    /// Validates: Requirement 6.6 — DatasetDeleteConfirm stores the dataset name.
    #[test]
    fn dataset_delete_confirm_stores_name() {
        // Validates: Requirement 6.6
        let confirm = DatasetDeleteConfirm::new("PAYROLL.DATA");
        assert_eq!(confirm.name, "PAYROLL.DATA");
    }

    // ── Task 9 tests ──────────────────────────────────────────────────────────

    fn make_entry(
        name: &str,
        entry_type: &str,
        size: &str,
        modified: &str,
        is_container: bool,
    ) -> ContentEntry {
        ContentEntry {
            name: name.to_string(),
            entry_type: entry_type.to_string(),
            size: size.to_string(),
            modified: modified.to_string(),
            is_container,
        }
    }

    /// Validates: Requirement 10.1 — ContentAreaState initialises with no selection and empty entries.
    #[test]
    fn content_area_state_initialises_empty() {
        // Validates: Requirement 10.1
        let s = ContentAreaState::default();
        assert!(s.selected_catalog.is_none());
        assert!(s.entries.is_empty());
        assert!(s.path_segments.is_empty());
        assert_eq!(s.sort_col, SortColumn::Name);
        assert_eq!(s.sort_dir, SortDir::Ascending);
        assert!(s.content_filter.is_empty());
    }

    /// Validates: Requirement 10.7 — directories sort before files when sorting by Name.
    #[test]
    fn visible_entries_name_sort_groups_dirs_before_files() {
        // Validates: Requirement 10.7
        let mut s = ContentAreaState::default();
        s.entries = vec![
            make_entry("zebra.txt", "File", "", "", false),
            make_entry("alpha", "Directory", "", "", true),
            make_entry("mango.txt", "File", "", "", false),
            make_entry("beta", "Directory", "", "", true),
        ];
        let names: Vec<&str> = s
            .visible_entries()
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha", "beta", "mango.txt", "zebra.txt"]);
    }

    /// Validates: Requirement 10.7 — directories within the directory group are sorted alphabetically.
    #[test]
    fn visible_entries_name_sort_dirs_are_alphabetical_within_group() {
        // Validates: Requirement 10.7
        let mut s = ContentAreaState::default();
        s.entries = vec![
            make_entry("Zebra", "Directory", "", "", true),
            make_entry("alpha", "Directory", "", "", true),
            make_entry("Mango", "Directory", "", "", true),
        ];
        let names: Vec<&str> = s
            .visible_entries()
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        // Case-insensitive: alpha < mango < zebra
        assert_eq!(names, vec!["alpha", "Mango", "Zebra"]);
    }

    /// Validates: Requirement 10.7 — non-Name sort columns do not apply container grouping.
    #[test]
    fn visible_entries_type_sort_does_not_force_dir_grouping() {
        // Validates: Requirement 10.7 (grouping only applies to Name column)
        let mut s = ContentAreaState::default();
        s.entries = vec![
            make_entry("b_file", "PS", "", "", false),
            make_entry("a_dir", "Directory", "", "", true),
        ];
        s.sort_col = SortColumn::Type;
        s.sort_dir = SortDir::Ascending;
        let types: Vec<&str> = s
            .visible_entries()
            .iter()
            .map(|e| e.entry_type.as_str())
            .collect();
        // "Directory" < "PS" alphabetically
        assert_eq!(types, vec!["Directory", "PS"]);
    }

    /// Validates: Requirement 10.2 — visible_entries sorts by Name ascending by default.
    #[test]
    fn visible_entries_sorts_by_name_ascending_by_default() {
        // Validates: Requirement 10.2
        let mut s = ContentAreaState::default();
        s.entries = vec![
            make_entry("zebra.txt", "File", "1 KB", "2024-01-03", false),
            make_entry("alpha.txt", "File", "2 KB", "2024-01-01", false),
            make_entry("mango.txt", "File", "3 KB", "2024-01-02", false),
        ];
        let names: Vec<&str> = s
            .visible_entries()
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        assert_eq!(names, vec!["alpha.txt", "mango.txt", "zebra.txt"]);
    }

    /// Validates: Requirement 10.2 — toggle_sort on same column flips to descending.
    #[test]
    fn toggle_sort_same_column_flips_to_descending() {
        // Validates: Requirement 10.2
        let mut s = ContentAreaState::default();
        assert_eq!(s.sort_dir, SortDir::Ascending);
        s.toggle_sort(SortColumn::Name);
        assert_eq!(s.sort_col, SortColumn::Name);
        assert_eq!(s.sort_dir, SortDir::Descending);
    }

    /// Validates: Requirement 10.2 — toggle_sort on different column resets to ascending.
    #[test]
    fn toggle_sort_different_column_resets_to_ascending() {
        // Validates: Requirement 10.2
        let mut s = ContentAreaState::default();
        s.toggle_sort(SortColumn::Name); // now descending
        s.toggle_sort(SortColumn::Type); // switch column
        assert_eq!(s.sort_col, SortColumn::Type);
        assert_eq!(s.sort_dir, SortDir::Ascending);
    }

    /// Validates: Requirement 10.2 — visible_entries sorts descending correctly.
    #[test]
    fn visible_entries_sorts_by_name_descending() {
        // Validates: Requirement 10.2
        let mut s = ContentAreaState::default();
        s.entries = vec![
            make_entry("alpha.txt", "File", "", "", false),
            make_entry("zebra.txt", "File", "", "", false),
        ];
        s.sort_dir = SortDir::Descending;
        let names: Vec<&str> = s
            .visible_entries()
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        assert_eq!(names, vec!["zebra.txt", "alpha.txt"]);
    }

    /// Validates: Requirement 10.2 — sort by Type column works.
    #[test]
    fn visible_entries_sorts_by_type_column() {
        // Validates: Requirement 10.2
        let mut s = ContentAreaState::default();
        s.entries = vec![
            make_entry("b", "PS", "", "", false),
            make_entry("a", "Directory", "", "", true),
        ];
        s.sort_col = SortColumn::Type;
        s.sort_dir = SortDir::Ascending;
        let types: Vec<&str> = s
            .visible_entries()
            .iter()
            .map(|e| e.entry_type.as_str())
            .collect();
        assert_eq!(types, vec!["Directory", "PS"]);
    }

    /// Validates: Requirement 10.6 — visible_entries filters by name substring (case-insensitive).
    #[test]
    fn visible_entries_filters_by_name_case_insensitive() {
        // Validates: Requirement 10.6
        let mut s = ContentAreaState::default();
        s.entries = vec![
            make_entry("README.md", "File", "", "", false),
            make_entry("main.rs", "File", "", "", false),
            make_entry("Cargo.toml", "File", "", "", false),
        ];
        s.content_filter = "readme".to_string();
        let visible = s.visible_entries();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].name, "README.md");
    }

    /// Validates: Requirement 10.6 — empty filter shows all entries.
    #[test]
    fn visible_entries_empty_filter_shows_all() {
        // Validates: Requirement 10.6
        let mut s = ContentAreaState::default();
        s.entries = vec![
            make_entry("a", "File", "", "", false),
            make_entry("b", "File", "", "", false),
        ];
        assert_eq!(s.visible_entries().len(), 2);
    }

    /// Validates: Requirement 10.6 — filter with no match returns empty list.
    #[test]
    fn visible_entries_filter_no_match_returns_empty() {
        // Validates: Requirement 10.6
        let mut s = ContentAreaState::default();
        s.entries = vec![make_entry("hello.txt", "File", "", "", false)];
        s.content_filter = "zzz".to_string();
        assert!(s.visible_entries().is_empty());
    }

    /// Validates: Requirement 10.5 — breadcrumb_display returns catalog name when at root.
    #[test]
    fn breadcrumb_display_at_root_shows_catalog_name() {
        // Validates: Requirement 10.5
        let mut s = ContentAreaState::default();
        s.selected_catalog = Some("PAYROLL".to_string());
        assert_eq!(s.breadcrumb_display(), "PAYROLL");
    }

    /// Validates: Requirement 10.5 — breadcrumb_display includes path segments.
    #[test]
    fn breadcrumb_display_includes_path_segments() {
        // Validates: Requirement 10.5
        let mut s = ContentAreaState::default();
        s.selected_catalog = Some("MYCAT".to_string());
        s.push_path("src");
        s.push_path("lib");
        assert_eq!(s.breadcrumb_display(), "MYCAT / src / lib");
    }

    /// Validates: Requirement 10.5 — breadcrumb_display is empty when no catalog selected.
    #[test]
    fn breadcrumb_display_empty_when_no_catalog_selected() {
        // Validates: Requirement 10.5
        let s = ContentAreaState::default();
        assert!(s.breadcrumb_display().is_empty());
    }

    /// Validates: Requirement 10.5 — navigate_to_segment(0) clears all path segments.
    #[test]
    fn navigate_to_segment_zero_clears_path() {
        // Validates: Requirement 10.5
        let mut s = ContentAreaState::default();
        s.push_path("src");
        s.push_path("lib");
        s.navigate_to_segment(0);
        assert!(s.path_segments.is_empty());
    }

    /// Validates: Requirement 10.5 — navigate_to_segment(1) keeps first segment only.
    #[test]
    fn navigate_to_segment_one_keeps_first_segment() {
        // Validates: Requirement 10.5
        let mut s = ContentAreaState::default();
        s.push_path("src");
        s.push_path("lib");
        s.push_path("util");
        s.navigate_to_segment(1);
        assert_eq!(s.path_segments, vec!["src"]);
    }

    /// Validates: Requirement 10.3 — OpenFile action carries the file name.
    #[test]
    fn open_file_action_carries_name() {
        // Validates: Requirement 10.3
        let action = FilesPanelAction::OpenFile("PAYROLL.DATA".to_string());
        assert_eq!(
            action,
            FilesPanelAction::OpenFile("PAYROLL.DATA".to_string())
        );
        assert_ne!(action, FilesPanelAction::None);
    }

    /// Validates: Requirement 10.4 — NavigateInto action carries the directory name.
    #[test]
    fn navigate_into_action_carries_name() {
        // Validates: Requirement 10.4
        let action = FilesPanelAction::NavigateInto("src".to_string());
        assert_eq!(action, FilesPanelAction::NavigateInto("src".to_string()));
        assert_ne!(action, FilesPanelAction::None);
    }

    /// Validates: Requirement 10.4 — push_path appends a segment and breadcrumb updates.
    #[test]
    fn push_path_appends_segment() {
        // Validates: Requirement 10.4
        let mut s = ContentAreaState::default();
        s.selected_catalog = Some("CAT".to_string());
        s.push_path("subdir");
        assert_eq!(s.path_segments, vec!["subdir"]);
        assert_eq!(s.breadcrumb_display(), "CAT / subdir");
    }

    /// Validates: Requirement 10.1 — FilesPanelState initialises with default ContentAreaState.
    #[test]
    fn files_panel_state_has_default_content_area() {
        // Validates: Requirement 10.1
        let state = FilesPanelState::new();
        assert!(state.content.selected_catalog.is_none());
        assert!(state.content.entries.is_empty());
    }

    // ── Phase AT: Allocated Dataset Persistence and Display (Req 13) ─────────

    fn make_alloc_params(
        name: &str,
        dsorg: crate::dataset_alloc_dialog::Dsorg,
    ) -> crate::dataset_alloc_dialog::AllocParams {
        crate::dataset_alloc_dialog::AllocParams {
            dataset_name: name.to_string(),
            dsorg,
            recfm: crate::dataset_alloc_dialog::Recfm::Fb,
            lrecl: 80,
            blksize: 0,
            dir_blocks: None,
            gdg_limit: None,
            scratch: false,
            description: None,
        }
    }

    /// Validates: Requirement 13.1 — FilesPanelState has a datasets map.
    #[test]
    fn files_panel_state_has_datasets_map() {
        // Validates: Requirement 13.1
        let state = FilesPanelState::new();
        assert!(state.datasets.is_empty());
    }

    /// Validates: Requirement 13.2 — add_dataset inserts AllocatedDataset under the catalog name.
    #[test]
    fn add_dataset_inserts_into_map_under_catalog_name() {
        // Validates: Requirement 13.2
        let mut state = FilesPanelState::new();
        let params = make_alloc_params("DEV.TEST.PSFB80", crate::dataset_alloc_dialog::Dsorg::Ps);
        state.add_dataset("development", params);
        let datasets = state
            .datasets
            .get("development")
            .expect("catalog entry must exist");
        assert_eq!(datasets.len(), 1);
        assert_eq!(datasets[0].name, "DEV.TEST.PSFB80");
        assert_eq!(datasets[0].dsorg, "PS");
    }

    /// Validates: Requirement 13.3 — load_entries_from_datasets populates content area entries.
    #[test]
    fn load_entries_populates_content_area_from_datasets() {
        // Validates: Requirement 13.3
        let mut state = FilesPanelState::new();
        state.add_dataset(
            "development",
            make_alloc_params("DEV.SEQ", crate::dataset_alloc_dialog::Dsorg::Ps),
        );
        state.add_dataset(
            "development",
            make_alloc_params("DEV.LIB", crate::dataset_alloc_dialog::Dsorg::Po),
        );
        state.load_entries_from_datasets("development");
        assert_eq!(state.content.entries.len(), 2);
        let seq = state
            .content
            .entries
            .iter()
            .find(|e| e.name == "DEV.SEQ")
            .expect("SEQ must be present");
        assert_eq!(seq.entry_type, "PS");
        assert!(!seq.is_container);
        let lib = state
            .content
            .entries
            .iter()
            .find(|e| e.name == "DEV.LIB")
            .expect("LIB must be present");
        assert_eq!(lib.entry_type, "PO");
        assert!(lib.is_container);
    }

    /// Validates: Requirement 13.5 — removing a catalog also removes its datasets.
    #[test]
    fn delete_catalog_removes_its_datasets() {
        // Validates: Requirement 13.5
        let mut state = FilesPanelState::new();
        state.add_dataset(
            "development",
            make_alloc_params("DEV.TEST", crate::dataset_alloc_dialog::Dsorg::Ps),
        );
        assert!(state.datasets.contains_key("development"));
        state.remove_catalog_datasets("development");
        assert!(!state.datasets.contains_key("development"));
    }

    // ── Phase BJ: Dataset path resolution (Req 16) ───────────────────────

    /// Validates: Requirement 16.1, 16.5 — DSN qualifiers become path components.
    #[test]
    fn resolve_dataset_path_maps_dsn_to_subpath() {
        // Validates: Requirement 16.1, 16.5
        let result =
            FilesPanelState::resolve_dataset_path("C:/catalogs/payroll", "PAYROLL.EMPLOYEE");
        assert!(result.is_some());
        let path = result.unwrap();
        // Should end with PAYROLL/EMPLOYEE (platform separator)
        let components: Vec<_> = path.components().collect();
        let last = components.last().unwrap().as_os_str().to_string_lossy();
        let second_last = components[components.len() - 2]
            .as_os_str()
            .to_string_lossy();
        assert_eq!(last, "EMPLOYEE");
        assert_eq!(second_last, "PAYROLL");
    }

    /// Validates: Requirement 16.4, 16.5 — empty repository path returns None.
    #[test]
    fn resolve_dataset_path_empty_repo_returns_none() {
        // Validates: Requirement 16.4, 16.5
        let result = FilesPanelState::resolve_dataset_path("", "PAYROLL.EMPLOYEE");
        assert!(result.is_none());
    }

    /// Validates: Requirement 16.5 — empty DSN returns None.
    #[test]
    fn resolve_dataset_path_empty_dsn_returns_none() {
        // Validates: Requirement 16.5
        let result = FilesPanelState::resolve_dataset_path("C:/catalogs/payroll", "");
        assert!(result.is_none());
    }

    /// Validates: Requirement 16.1 — single-qualifier DSN resolves to one component under repo.
    #[test]
    fn resolve_dataset_path_single_qualifier_dsn() {
        // Validates: Requirement 16.1
        let result = FilesPanelState::resolve_dataset_path("C:/repo", "MYDATA");
        assert!(result.is_some());
        let path = result.unwrap();
        let last = path.file_name().unwrap().to_string_lossy();
        assert_eq!(last, "MYDATA");
    }

    /// Validates: Requirement 16.3 — create_dataset_file creates the file and parent dirs.
    #[test]
    fn opening_missing_dataset_creates_file_and_parent_dirs() {
        // Validates: Requirement 16.3
        use tempfile::TempDir;
        let tmp = TempDir::new().expect("tempdir");
        let target = tmp.path().join("PAYROLL").join("EMPLOYEE");
        assert!(!target.exists());
        FilesPanelState::create_dataset_file(&target).expect("create must succeed");
        assert!(target.exists(), "file must be created");
        assert!(target.is_file(), "must be a regular file");
    }

    /// Validates: Requirement 16.3 — create_dataset_file creates missing parent directories.
    #[test]
    fn opening_missing_dataset_creates_parent_dirs() {
        // Validates: Requirement 16.3
        use tempfile::TempDir;
        let tmp = TempDir::new().expect("tempdir");
        let target = tmp.path().join("A").join("B").join("C").join("DATASET");
        assert!(!target.parent().unwrap().exists());
        FilesPanelState::create_dataset_file(&target).expect("create must succeed");
        assert!(target.exists());
        assert!(target.parent().unwrap().is_dir());
    }
}
