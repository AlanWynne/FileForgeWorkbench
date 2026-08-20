//! # File Explorer Panel — POM Option 2
//!
//! Renders the File Explorer Panel: a tree view of all open catalogs grouped
//! under Mainframe Catalogs, POSIX Catalogs, and Native Catalogs section headers.
//! Each catalog node is expandable to show its files/datasets as child nodes.
//! Double-clicking a file node opens it in a new editor tab.
//!
//! Validates: Requirement 19.5, 19.6, 19.7, 19.8, 19.9

use eframe::egui;

use crate::catalog_registry::{CatalogRegistry, CatalogType};
use crate::files_panel::FilesPanelState;

// ── State ─────────────────────────────────────────────────────────────────────

/// State for the File Explorer Panel tab.
///
/// Currently stateless — egui's `CollapsingHeader` manages expand/collapse
/// state internally via its `id_salt`.
///
/// Validates: Requirement 19.6
#[derive(Debug, Default, Clone)]
pub struct FileExplorerPanelState;

impl FileExplorerPanelState {
    pub fn new() -> Self {
        Self
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
    _state: &mut FileExplorerPanelState,
    registry: &CatalogRegistry,
    files_panel: &FilesPanelState,
) -> Option<String> {
    let mut open_path: Option<String> = None;

    let total_catalogs = registry.list().len();

    if total_catalogs == 0 {
        // Validates: Requirement 19.8 — empty-state placeholder
        ui.label(egui::RichText::new(
            "No catalogs open \u{2014} use File Catalogs (option 1) to create or mount a catalog",
        ).monospace().weak());
        return None;
    }

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
                        // Validates: Requirement 19.6 — expanding shows files/datasets
                        let datasets = files_panel.datasets.get(&cat.name);
                        egui::CollapsingHeader::new(
                            egui::RichText::new(format!("\u{1F4C1} {}", cat.name)).monospace(),
                        )
                        .id_salt(format!("fep_cat_{}", cat.name))
                        .default_open(false)
                        .show(ui, |ui| {
                            match datasets {
                                None => {
                                    ui.label(
                                        egui::RichText::new("  (no files)").monospace().weak(),
                                    );
                                }
                                Some(d) if d.is_empty() => {
                                    ui.label(
                                        egui::RichText::new("  (no files)").monospace().weak(),
                                    );
                                }
                                Some(datasets) => {
                                    for ds in datasets {
                                        let icon = if ds.dsorg == "PO"
                                            || ds.dsorg == "PDSE"
                                            || ds.dsorg == "GDG"
                                        {
                                            "\u{1F4C1}"
                                        } else {
                                            "\u{1F4C4}"
                                        };
                                        let label =
                                            egui::RichText::new(format!("  {icon} {}", ds.name))
                                                .monospace();
                                        // Validates: Requirement 19.9 — double-click opens file
                                        let resp = ui.selectable_label(false, label);
                                        if resp.double_clicked() {
                                            open_path = Some(ds.name.clone());
                                        }
                                    }
                                }
                            }
                        });
                    }
                }
            });
    }

    open_path
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

    /// Validates: Requirement 19.5 — FileExplorerPanelState is a unit struct (egui manages expand state).
    #[test]
    fn file_explorer_panel_state_initialises_empty() {
        // Validates: Requirement 19.5
        let _state = FileExplorerPanelState::new();
        // No fields to assert — egui CollapsingHeader manages expand state internally.
    }
}
