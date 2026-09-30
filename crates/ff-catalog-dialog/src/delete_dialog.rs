//! Delete Catalog sub-dialog: the delete-choice enum, the confirmation state,
//! the modal render, and the delete execution (registry removal + optional
//! recursive file delete, with Home-catalog protection).
//!
//! Validates: Requirement 4.3-4.5, 14.6, 14.7

use ff_catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog};

/// Which delete action the user chose.
///
/// Validates: Requirement 4.3-4.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteChoice {
    /// Remove catalog from registry only; leave backing files intact (Req 4.4).
    CatalogOnly,
    /// Remove catalog from registry AND recursively delete backing files (Req 4.5).
    CatalogAndFiles,
    /// User cancelled -- no action.
    Cancel,
}

/// State for the Delete Catalog confirmation dialog.
///
/// Validates: Requirement 4.3
#[derive(Debug, Clone)]
pub struct DeleteCatalogConfirm {
    /// Name of the catalog to delete.
    pub name: String,
    /// Backing path -- used for recursive delete when `CatalogAndFiles` is chosen.
    pub path: String,
}

impl DeleteCatalogConfirm {
    /// Create from an existing catalog.
    pub fn from_catalog(catalog: &VirtualCatalog) -> Self {
        Self {
            name: catalog.name.clone(),
            path: catalog.path.clone(),
        }
    }
}

/// Render the Delete Catalog confirmation dialog.
///
/// Returns the user's `DeleteChoice`; the caller is responsible for
/// executing the chosen action against the registry and filesystem.
///
/// Validates: Requirement 4.3-4.5
pub fn render_delete(ctx: &egui::Context, confirm: &DeleteCatalogConfirm) -> DeleteChoice {
    let mut choice = DeleteChoice::Cancel;
    let mut open = true;

    egui::Window::new("Delete Catalog")
        .collapsible(false)
        .resizable(false)
        .min_width(380.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.set_min_width(360.0);
            ui.label(format!(
                "Delete catalog \"{}\"? This will unmount it.",
                confirm.name
            ));
            ui.label("Optionally delete all backing files.");
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Delete Catalog Only").clicked() {
                    choice = DeleteChoice::CatalogOnly;
                }
                if ui.button("Delete Catalog and Files").clicked() {
                    choice = DeleteChoice::CatalogAndFiles;
                }
                if ui.button("Cancel").clicked() {
                    choice = DeleteChoice::Cancel;
                }
            });
        });

    // Window X button also cancels
    if !open {
        choice = DeleteChoice::Cancel;
    }
    choice
}

/// Execute the delete action chosen by the user.
///
/// - `CatalogOnly`: removes from registry, leaves files.
/// - `CatalogAndFiles`: removes from registry, then recursively deletes `path`.
///
/// The catalog named `"Home"` of type `Native` is protected and cannot be
/// deleted via this function.
///
/// Returns `Ok(())` on success or an error string on failure.
///
/// Validates: Requirement 4.4, 4.5, 14.6
pub fn execute_delete(
    choice: &DeleteChoice,
    confirm: &DeleteCatalogConfirm,
    registry: &mut CatalogRegistry,
) -> Result<(), String> {
    // Validates: Requirement 14.6 -- Home Native catalog is protected from deletion.
    if choice != &DeleteChoice::Cancel {
        if let Some(cat) = registry.get_by_name(&confirm.name) {
            if cat.name == "Home" && cat.catalog_type == CatalogType::Native {
                return Err(
                    "The Home catalog cannot be deleted. Rename or edit it instead.".to_string(),
                );
            }
        }
    }
    match choice {
        DeleteChoice::CatalogOnly => registry
            .remove(&confirm.name)
            .map(|_| ())
            .map_err(|e| e.to_string()),
        DeleteChoice::CatalogAndFiles => {
            registry.remove(&confirm.name).map_err(|e| e.to_string())?;
            if !confirm.path.is_empty() {
                std::fs::remove_dir_all(&confirm.path)
                    .map_err(|e| format!("Failed to delete '{}': {e}", confirm.path))?;
            }
            Ok(())
        }
        DeleteChoice::Cancel => Ok(()),
    }
}

// === Tests =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ff_catalog_registry::CatalogRegistry;

    fn empty_registry() -> CatalogRegistry {
        CatalogRegistry::new()
    }

    fn registered_mainframe(registry: &mut CatalogRegistry) -> VirtualCatalog {
        let cat = VirtualCatalog {
            name: "PAYROLL".to_string(),
            catalog_type: CatalogType::Mainframe,
            path: "/catalogs/payroll".to_string(),
            description: Some("Payroll datasets".to_string()),
            auto_mount: true,
            default_hlq: Some("PAYROLL".to_string()),
            mount_point: None,
            read_only: false,
        };
        registry.register(cat.clone()).unwrap();
        cat
    }

    /// Validates: Requirement 4.3 -- DeleteCatalogConfirm is built from catalog.
    #[test]
    fn delete_confirm_built_from_catalog() {
        // Validates: Requirement 4.3
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let confirm = DeleteCatalogConfirm::from_catalog(&cat);
        assert_eq!(confirm.name, "PAYROLL");
        assert_eq!(confirm.path, "/catalogs/payroll");
    }

    /// Validates: Requirement 4.4 -- CatalogOnly removes from registry, leaves files.
    #[test]
    fn execute_delete_catalog_only_removes_from_registry() {
        // Validates: Requirement 4.4
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let confirm = DeleteCatalogConfirm::from_catalog(&cat);
        execute_delete(&DeleteChoice::CatalogOnly, &confirm, &mut registry).unwrap();
        assert!(!registry.exists("PAYROLL"));
    }

    /// Validates: Requirement 4.4 -- CatalogOnly on unknown name returns error.
    #[test]
    fn execute_delete_catalog_only_unknown_name_returns_error() {
        // Validates: Requirement 4.4
        let mut registry = empty_registry();
        let confirm = DeleteCatalogConfirm {
            name: "NOSUCH".to_string(),
            path: "/some/path".to_string(),
        };
        let result = execute_delete(&DeleteChoice::CatalogOnly, &confirm, &mut registry);
        assert!(result.is_err());
    }

    /// Validates: Requirement 4.5 -- CatalogAndFiles removes from registry and deletes path.
    #[test]
    fn execute_delete_catalog_and_files_removes_registry_and_deletes_dir() {
        // Validates: Requirement 4.5
        use tempfile::TempDir;
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_string_lossy().into_owned();

        let mut registry = empty_registry();
        let cat = VirtualCatalog {
            name: "TMPCAT".to_string(),
            catalog_type: CatalogType::Native,
            path: path.clone(),
            description: None,
            auto_mount: false,
            default_hlq: None,
            mount_point: None,
            read_only: false,
        };
        registry.register(cat).unwrap();

        let confirm = DeleteCatalogConfirm {
            name: "TMPCAT".to_string(),
            path: path.clone(),
        };
        execute_delete(&DeleteChoice::CatalogAndFiles, &confirm, &mut registry).unwrap();

        assert!(!registry.exists("TMPCAT"));
        assert!(!std::path::Path::new(&path).exists());
        // Prevent TempDir from trying to clean up the already-deleted dir
        let _ = tmp.keep();
    }

    /// Validates: Requirement 4.3 -- Cancel choice leaves registry unchanged.
    #[test]
    fn execute_delete_cancel_leaves_registry_unchanged() {
        // Validates: Requirement 4.3
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let confirm = DeleteCatalogConfirm::from_catalog(&cat);
        execute_delete(&DeleteChoice::Cancel, &confirm, &mut registry).unwrap();
        assert!(registry.exists("PAYROLL"));
    }

    /// Validates: Requirement 14.6 -- deleting the "Home" Native catalog is rejected.
    #[test]
    fn delete_home_native_catalog_is_rejected() {
        // Validates: Requirement 14.6
        let mut registry = empty_registry();
        registry
            .register(VirtualCatalog {
                name: "Home".to_string(),
                catalog_type: CatalogType::Native,
                path: "C:/Users/user".to_string(),
                description: Some("Default home directory catalog".to_string()),
                auto_mount: true,
                default_hlq: None,
                mount_point: None,
                read_only: false,
            })
            .unwrap();
        let confirm = DeleteCatalogConfirm {
            name: "Home".to_string(),
            path: "C:/Users/user".to_string(),
        };
        let result = execute_delete(&DeleteChoice::CatalogOnly, &confirm, &mut registry);
        assert!(
            result.is_err(),
            "deleting Home Native catalog must be rejected"
        );
        let msg = result.unwrap_err();
        assert!(
            msg.contains("cannot be deleted"),
            "error message must mention cannot be deleted, got: {msg}"
        );
        // Registry must be unchanged
        assert!(registry.exists("Home"));
    }

    /// Validates: Requirement 14.7 -- a Native catalog renamed away from "Home" can be deleted.
    #[test]
    fn delete_renamed_home_catalog_is_permitted() {
        // Validates: Requirement 14.7
        // A catalog that was once "Home" but is now named "MyHome" is not protected.
        let mut registry = empty_registry();
        registry
            .register(VirtualCatalog {
                name: "MyHome".to_string(),
                catalog_type: CatalogType::Native,
                path: "C:/Users/user".to_string(),
                description: None,
                auto_mount: true,
                default_hlq: None,
                mount_point: None,
                read_only: false,
            })
            .unwrap();
        let confirm = DeleteCatalogConfirm {
            name: "MyHome".to_string(),
            path: "C:/Users/user".to_string(),
        };
        let result = execute_delete(&DeleteChoice::CatalogOnly, &confirm, &mut registry);
        assert!(result.is_ok(), "renamed catalog must be deletable");
        assert!(!registry.exists("MyHome"));
    }
}
