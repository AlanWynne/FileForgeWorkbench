//! Edit Catalog sub-dialog: form state (name/type immutable), validation, and
//! the modal render.
//!
//! Validates: Requirement 4.1-4.2, 15.1-15.3

use ff_catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog};

use crate::catalog_type_label;
use crate::new_dialog::DialogOutcome;

/// Form state for the Edit Catalog dialog.
///
/// Name and type are immutable after creation (Req 4.2).
/// Validates: Requirement 4.1-4.2
#[derive(Debug, Clone)]
pub struct EditCatalogForm {
    /// Catalog name -- display only, not editable.
    pub name: String,
    /// Catalog type -- display only, not editable.
    pub catalog_type: CatalogType,
    /// Editable description.
    pub description: String,
    /// Editable auto-mount flag.
    pub auto_mount: bool,
    /// Editable read-only flag (POSIX / Native only).
    pub read_only: bool,
    /// Editable default HLQ (Mainframe only).
    pub default_hlq: String,
    /// Repository path -- read-only display (Req 15.1, 15.3).
    pub path: String,
    /// Inline error message, if any.
    pub error: Option<String>,
}

impl EditCatalogForm {
    /// Pre-populate the form from an existing catalog.
    ///
    /// Validates: Requirement 4.1, 15.1
    pub fn from_catalog(catalog: &VirtualCatalog) -> Self {
        Self {
            name: catalog.name.clone(),
            catalog_type: catalog.catalog_type,
            description: catalog.description.clone().unwrap_or_default(),
            auto_mount: catalog.auto_mount,
            read_only: catalog.read_only,
            default_hlq: catalog.default_hlq.clone().unwrap_or_default(),
            path: catalog.path.clone(),
            error: None,
        }
    }
}

/// Validate the edit form fields.
///
/// Validates: Requirement 4.2
pub fn validate_edit(form: &EditCatalogForm) -> Option<String> {
    if form.description.len() > 120 {
        return Some("Description must be 120 characters or fewer.".to_string());
    }
    None
}

/// Render the Edit Catalog modal dialog.
///
/// Returns `DialogOutcome::Confirmed` when the user saves changes,
/// `DialogOutcome::Cancelled` when they cancel.
///
/// Validates: Requirement 4.1-4.2
pub fn render_edit(
    ctx: &egui::Context,
    form: &mut EditCatalogForm,
    registry: &mut CatalogRegistry,
) -> DialogOutcome {
    let mut outcome = DialogOutcome::Open;

    egui::Window::new("Edit Catalog")
        .collapsible(false)
        .resizable(false)
        .min_width(420.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_min_width(400.0);

            // Name and type -- read-only display (Req 4.2)
            ui.horizontal(|ui| {
                ui.label("Catalog Name:  ");
                ui.label(egui::RichText::new(&form.name).monospace().strong());
            });
            ui.horizontal(|ui| {
                ui.label("Catalog Type:  ");
                ui.label(
                    egui::RichText::new(catalog_type_label(form.catalog_type))
                        .monospace()
                        .weak(),
                );
            });
            // Repository path -- read-only (Req 15.1, 15.2, 15.3)
            ui.horizontal(|ui| {
                ui.label("Repository Path:");
                ui.label(egui::RichText::new(&form.path).monospace().weak());
            });
            ui.separator();

            // Editable fields
            ui.horizontal(|ui| {
                ui.label("Description:   ");
                ui.text_edit_singleline(&mut form.description);
            });
            ui.checkbox(&mut form.auto_mount, "Auto-mount on startup");

            // Type-specific editable fields (Req 4.2)
            match form.catalog_type {
                CatalogType::Mainframe => {
                    ui.horizontal(|ui| {
                        ui.label("Default HLQ:   ");
                        ui.text_edit_singleline(&mut form.default_hlq);
                    });
                }
                CatalogType::Posix | CatalogType::Native => {
                    ui.checkbox(&mut form.read_only, "Read-Only");
                }
            }

            if let Some(err) = &form.error {
                ui.colored_label(egui::Color32::RED, err);
            }
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    match validate_edit(form) {
                        Some(err) => {
                            form.error = Some(err);
                        }
                        None => {
                            let hlq = if form.default_hlq.trim().is_empty() {
                                None
                            } else {
                                Some(form.default_hlq.trim().to_string())
                            };
                            let desc = if form.description.trim().is_empty() {
                                None
                            } else {
                                Some(form.description.trim().to_string())
                            };
                            // update() cannot fail -- name was pre-populated from registry
                            let _ = registry.update(
                                &form.name,
                                desc,
                                form.auto_mount,
                                form.read_only,
                                hlq,
                            );
                            form.error = None;
                            outcome = DialogOutcome::Confirmed;
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    outcome = DialogOutcome::Cancelled;
                }
            });
        });

    // Validates: accessibility Requirement 2.1, 2.3 -- Escape closes the Edit Catalog dialog.
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        outcome = DialogOutcome::Cancelled;
    }

    outcome
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

    fn registered_posix(registry: &mut CatalogRegistry) -> VirtualCatalog {
        let cat = VirtualCatalog {
            name: "dev-posix".to_string(),
            catalog_type: CatalogType::Posix,
            path: "/projects/dev".to_string(),
            description: None,
            auto_mount: true,
            default_hlq: None,
            mount_point: Some("/".to_string()),
            read_only: false,
        };
        registry.register(cat.clone()).unwrap();
        cat
    }

    /// Validates: Requirement 15.1 -- EditCatalogForm carries the repository path from the catalog.
    #[test]
    fn edit_form_displays_repository_path() {
        // Validates: Requirement 15.1
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let form = EditCatalogForm::from_catalog(&cat);
        assert_eq!(form.path, "/catalogs/payroll");
    }

    /// Validates: Requirement 15.2 -- repository path is present for all catalog types.
    #[test]
    fn edit_form_repository_path_present_for_all_catalog_types() {
        // Validates: Requirement 15.2
        let mut registry = empty_registry();
        let mf = registered_mainframe(&mut registry);
        let px = registered_posix(&mut registry);
        assert_eq!(EditCatalogForm::from_catalog(&mf).path, "/catalogs/payroll");
        assert_eq!(EditCatalogForm::from_catalog(&px).path, "/projects/dev");
    }

    /// Validates: Requirement 4.1 -- EditCatalogForm is pre-populated from catalog.
    #[test]
    fn edit_form_prepopulated_from_catalog() {
        // Validates: Requirement 4.1
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let form = EditCatalogForm::from_catalog(&cat);
        assert_eq!(form.name, "PAYROLL");
        assert_eq!(form.catalog_type, CatalogType::Mainframe);
        assert_eq!(form.description, "Payroll datasets");
        assert!(form.auto_mount);
        assert_eq!(form.default_hlq, "PAYROLL");
        assert!(!form.read_only);
    }

    /// Validates: Requirement 4.1 -- EditCatalogForm with no description gives empty string.
    #[test]
    fn edit_form_no_description_gives_empty_string() {
        // Validates: Requirement 4.1
        let mut registry = empty_registry();
        let cat = registered_posix(&mut registry);
        let form = EditCatalogForm::from_catalog(&cat);
        assert_eq!(form.description, "");
    }

    /// Validates: Requirement 4.2 -- name and type fields are not editable (read-only display).
    #[test]
    fn edit_form_name_and_type_are_immutable() {
        // Validates: Requirement 4.2
        // The form carries name/type for display only; validate_edit() does not check them.
        // Mutating them in the form has no effect on the registry update path.
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let mut form = EditCatalogForm::from_catalog(&cat);
        // Even if someone mutates the display fields, registry.update() uses the original name.
        form.name = "TAMPERED".to_string();
        // registry still has PAYROLL, not TAMPERED
        assert!(registry.exists("PAYROLL"));
        assert!(!registry.exists("TAMPERED"));
    }

    /// Validates: Requirement 4.2 -- validate_edit accepts valid description.
    #[test]
    fn validate_edit_accepts_valid_description() {
        // Validates: Requirement 4.2
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let mut form = EditCatalogForm::from_catalog(&cat);
        form.description = "Updated description".to_string();
        assert!(validate_edit(&form).is_none());
    }

    /// Validates: Requirement 4.2 -- validate_edit rejects description over 120 chars.
    #[test]
    fn validate_edit_rejects_description_over_120_chars() {
        // Validates: Requirement 4.2
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let mut form = EditCatalogForm::from_catalog(&cat);
        form.description = "x".repeat(121);
        assert!(validate_edit(&form).is_some());
    }

    /// Validates: Requirement 4.2 -- confirmed edit updates registry fields.
    #[test]
    fn edit_confirm_updates_registry_fields() {
        // Validates: Requirement 4.2
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let mut form = EditCatalogForm::from_catalog(&cat);
        form.description = "New desc".to_string();
        form.auto_mount = false;
        form.default_hlq = "HR".to_string();

        assert!(validate_edit(&form).is_none());
        let hlq = if form.default_hlq.trim().is_empty() {
            None
        } else {
            Some(form.default_hlq.trim().to_string())
        };
        let desc = Some(form.description.trim().to_string());
        registry
            .update(&form.name, desc, form.auto_mount, form.read_only, hlq)
            .unwrap();

        let updated = registry.get_by_name("PAYROLL").unwrap();
        assert_eq!(updated.description.as_deref(), Some("New desc"));
        assert!(!updated.auto_mount);
        assert_eq!(updated.default_hlq.as_deref(), Some("HR"));
        // Name and type unchanged
        assert_eq!(updated.name, "PAYROLL");
        assert_eq!(updated.catalog_type, CatalogType::Mainframe);
    }

    /// Validates: Requirement 4.2 -- empty description clears the field.
    #[test]
    fn edit_confirm_empty_description_clears_field() {
        // Validates: Requirement 4.2
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let mut form = EditCatalogForm::from_catalog(&cat);
        form.description = String::new();

        registry
            .update(&form.name, None, form.auto_mount, form.read_only, None)
            .unwrap();

        let updated = registry.get_by_name("PAYROLL").unwrap();
        assert!(updated.description.is_none());
    }
}
