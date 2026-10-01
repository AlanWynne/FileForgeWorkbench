//! # Files Panel persistent state and dataset resolution
//!
//! The `FilesPanelState` struct (registry, sections, filter, command, active
//! dialog, content area, keyboard focus) plus the pure dataset path-resolution
//! and SQLite catalog-loading helpers.
//!
//! Validates: Requirement 1.1-1.7, 13.2, 16.1-16.6

use ff_catalog_registry::{CatalogRegistry, CatalogType};

use crate::state::{ContentAreaState, ContentEntry, FilesDialogState, SectionState};

/// Persistent state for the Files Panel tab.
pub struct FilesPanelState {
    /// Catalog registry -- source of truth for all virtual catalogs.
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
    /// Validates: Requirement 10.1-10.6
    pub content: ContentAreaState,
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
            pending_alloc_catalog: None,
            tree_focus_requested: false,
            focused_catalog: None,
        }
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
    /// Returns `None` when either argument is empty.
    ///
    /// Validates: Requirement 16.1, 16.4, 16.5
    pub fn resolve_dataset_path(repository_path: &str, dsn: &str) -> Option<std::path::PathBuf> {
        if repository_path.is_empty() || dsn.is_empty() {
            return None;
        }
        let normalised = repository_path.replace('/', std::path::MAIN_SEPARATOR_STR);
        let rel: std::path::PathBuf = dsn.split('.').collect();
        Some(std::path::Path::new(&normalised).join(rel))
    }

    /// Resolve a DSN via the SQLite catalog and open or create the physical file.
    ///
    /// Searches all mounted Mainframe catalogs in the registry for the DSN.
    /// On success returns the physical `PathBuf` (creating the file if absent).
    /// On failure returns a human-readable error string.
    ///
    /// Validates: Requirement 16.1, 16.3, 16.4, 16.6
    pub fn resolve_and_open_dataset(
        registry: &ff_dscatalog::catalog_registry::CatalogRegistry,
        dsn: &str,
    ) -> Result<std::path::PathBuf, String> {
        let parsed = ff_dscatalog::dsn::Dsn::parse(dsn)
            .map_err(|_| format!("'{}': invalid dataset name", dsn))?;
        match registry.resolve(&parsed) {
            Ok(result) => {
                let path = result.physical_path;
                if !path.exists() {
                    Self::create_dataset_file(&path).map_err(|e| {
                        format!("'{}': cannot create dataset file at {:?}: {}", dsn, path, e)
                    })?;
                }
                Ok(path)
            }
            Err(_) => Err(format!(
                "'{}': dataset not found in any mounted catalog",
                dsn
            )),
        }
    }

    /// Populate `content.entries` from the SQLite catalog for `catalog_name`.
    ///
    /// Replaces `load_entries_from_datasets` for Mainframe catalogs.
    /// Non-Mainframe catalogs are unaffected.
    ///
    /// Validates: Requirement 13.2
    pub fn load_entries_from_catalog(
        &mut self,
        catalog_name: &str,
        registry: &ff_dscatalog::catalog_registry::CatalogRegistry,
    ) {
        // Try the passed-in ff-dscatalog registry first (used in tests and
        // in the wired-up shell path once BU.4 is complete).
        let records = if let Some(catalog) = registry.get_catalog(catalog_name) {
            catalog.list_datasets().unwrap_or_default()
        } else {
            // Fall back: look up the repository path from self.registry and
            // open the catalog directly.
            let repo_path = match self
                .registry
                .list_by_type(CatalogType::Mainframe)
                .iter()
                .find(|c| c.name == catalog_name)
                .map(|c| c.path.clone())
            {
                Some(p) => p,
                None => return,
            };
            match ff_dscatalog::catalog::Catalog::mount(std::path::Path::new(&repo_path), 1) {
                Ok(c) => c.list_datasets().unwrap_or_default(),
                Err(_) => return,
            }
        };

        self.content.entries = records
            .into_iter()
            .map(|d| {
                let is_container = matches!(
                    d.dsorg,
                    ff_dscatalog::dataset::Dsorg::PO | ff_dscatalog::dataset::Dsorg::GDG
                );
                ContentEntry {
                    name: d.dsn.as_str().to_string(),
                    entry_type: d.dsorg.to_string(),
                    size: String::new(),
                    modified: d.modified.unwrap_or_default(),
                    is_container,
                }
            })
            .collect();
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

impl Default for FilesPanelState {
    fn default() -> Self {
        Self::new()
    }
}
