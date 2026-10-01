//! # Files Panel state and data types
//!
//! Pure data model for the Files Panel: section expand/collapse state, the
//! deferred-action enum, the open-dialog enum, sort column/direction, the
//! content-area entry and content-area state.
//!
//! Validates: Requirement 1.1-1.7, 10.1-10.7

use ff_catalog_dialog::{DeleteCatalogConfirm, EditCatalogForm, NewCatalogForm};
use ff_dataset_alloc_dialog::AllocDatasetForm;

// === State ==================================================================

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
    /// User pressed F3 / typed END -- return to POM view.
    ReturnToPom,
    /// User clicked "New Catalog" -- open the new catalog dialog.
    NewCatalog,
    /// User right-clicked a catalog node and chose Properties -- open edit dialog.
    EditCatalog(String),
    /// User right-clicked a catalog node and chose Delete Catalog -- open delete dialog.
    DeleteCatalog(String),
    /// User right-clicked a Mainframe catalog node and chose Allocate Dataset.
    AllocateDataset(String),
    /// User double-clicked a file/member/dataset node -- open in editor tab.
    ///
    /// Validates: Requirement 10.3
    OpenFile(String),
    /// User double-clicked a directory/container node -- navigate into it.
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
/// Validates: Requirement 10.1-10.6
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
            // Validates: Requirement 10.7 -- containers always sort before non-containers
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
