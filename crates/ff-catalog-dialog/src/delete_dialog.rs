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

    egui::Window::new(ff_i18n::t("catalog-dialog-delete-title"))
        .collapsible(false)
        .resizable(false)
        .min_width(380.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.set_min_width(360.0);
            ui.label(ff_i18n::t_args(
                "catalog-dialog-delete-confirm",
                &ff_i18n::MessageArgs::new().set("name", confirm.name.clone()),
            ));
            ui.label(ff_i18n::t("catalog-dialog-delete-optional-files"));
            ui.separator();
            ui.horizontal(|ui| {
                if ui
                    .button(ff_i18n::t("catalog-dialog-delete-btn-catalog-only"))
                    .clicked()
                {
                    choice = DeleteChoice::CatalogOnly;
                }
                if ui
                    .button(ff_i18n::t("catalog-dialog-delete-btn-catalog-and-files"))
                    .clicked()
                {
                    choice = DeleteChoice::CatalogAndFiles;
                }
                if ui.button(ff_i18n::t("catalog-dialog-btn-cancel")).clicked() {
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
                // The comparison `"Home"` + `CatalogType::Native` above is a LOGIC
                // test, never localised (Req 4.4); only the user-facing SENTENCE
                // is resolved through the seam (Req 4.1).
                return Err(ff_i18n::t("catalog-dialog-delete-err-home-protected"));
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
                std::fs::remove_dir_all(&confirm.path).map_err(|e| {
                    ff_i18n::t_args(
                        "catalog-dialog-delete-err-delete-failed",
                        &ff_i18n::MessageArgs::new()
                            .set("path", confirm.path.clone())
                            .set("error", e.to_string()),
                    )
                })?;
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

    /// Install the shipped English Identity_Base catalogue into the process-wide
    /// seam so lookups resolve to their English prose (not the key fallback).
    /// Idempotent and safe to call from multiple tests; the seam is a swappable
    /// global handle. The `i18n/` tree lives in the sibling `ff-i18n` crate.
    fn install_english_catalogue() {
        let i18n_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("ff-i18n")
            .join("i18n");
        let catalogue = ff_i18n::Catalogue::load_identity_base(&i18n_root)
            .expect("English Identity_Base must load for the catalog-dialog tests");
        ff_i18n::set_catalogue(catalogue);
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
        // The rejection MESSAGE is now localised through the ff_i18n seam
        // (Req 4.1); install the English catalogue so the text assertion below
        // reflects the ui.locale=en behaviour (Req 10.2).
        install_english_catalogue();
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

    // === Localisation of converted render sites (FEAT-catalog-dialog) =========

    /// With the English catalogue installed, a converted render site resolves to
    /// its EXACT original English literal, byte-for-byte -- including the
    /// alignment-padding spaces preserved via Fluent string-literal placeables.
    ///
    /// Validates: localization Requirement 4.1, 10.1, 10.2
    #[test]
    fn converted_sites_resolve_to_their_english_literals() {
        install_english_catalogue();
        // Plain labels / buttons.
        assert_eq!(ff_i18n::t("catalog-dialog-delete-title"), "Delete Catalog");
        assert_eq!(ff_i18n::t("catalog-dialog-btn-cancel"), "Cancel");
        assert_eq!(ff_i18n::t("catalog-dialog-btn-ok"), "OK");
        assert_eq!(ff_i18n::t("catalog-dialog-new-title"), "New Catalog");
        assert_eq!(ff_i18n::t("catalog-dialog-edit-title"), "Edit Catalog");
        // The type DISPLAY labels (distinct from the CatalogType enum variants).
        assert_eq!(ff_i18n::t("catalog-dialog-type-mainframe"), "Mainframe");
        assert_eq!(ff_i18n::t("catalog-dialog-type-posix"), "POSIX");
        assert_eq!(ff_i18n::t("catalog-dialog-type-native"), "Native");
        // Significant trailing/leading alignment whitespace is preserved exactly.
        assert_eq!(
            ff_i18n::t("catalog-dialog-new-label-name"),
            "Catalog Name:  "
        );
        assert_eq!(
            ff_i18n::t("catalog-dialog-new-label-root-path"),
            "Root Path:      "
        );
        assert_eq!(
            ff_i18n::t("catalog-dialog-new-label-root-directory"),
            "Root Directory: "
        );
    }

    /// An argument-bearing converted site interpolates its runtime value, and the
    /// `catalog_type_label` helper routes the CatalogType logic key through the
    /// seam to the English display label.
    ///
    /// Validates: localization Requirement 4.1, 9.1
    #[test]
    fn converted_argument_site_interpolates_and_type_label_resolves() {
        install_english_catalogue();
        let confirm = ff_i18n::t_args(
            "catalog-dialog-delete-confirm",
            &ff_i18n::MessageArgs::new().set("name", "PAYROLL"),
        );
        assert_eq!(confirm, "Delete catalog \"PAYROLL\"? This will unmount it.");
        assert_eq!(
            crate::catalog_type_label(CatalogType::Mainframe),
            "Mainframe"
        );
    }

    /// A key that is absent in every catalogue falls back to the key itself (a
    /// stable, visible, diagnosable sentinel) rather than panicking or rendering
    /// blank text -- the ff_i18n seam fallback discipline.
    ///
    /// Validates: localization Requirement 3.2, 3.3, 10.2
    #[test]
    fn missing_key_falls_back_to_the_key_itself() {
        install_english_catalogue();
        assert_eq!(
            ff_i18n::t("catalog-dialog-no-such-key-zzz"),
            "catalog-dialog-no-such-key-zzz"
        );
    }
}
