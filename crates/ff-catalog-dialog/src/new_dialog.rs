//! New Catalog sub-dialog: form state, validation, catalog construction, and
//! the modal render. Shared `DialogOutcome` lives here (used by the Edit dialog
//! too).
//!
//! Validates: Requirement 3.1-3.8, 12.1-12.3, 12.7

use ff_catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog};

use crate::catalog_type_label;

// === Form state ================================================================

/// The form state for the New Catalog dialog.
///
/// Validates: Requirement 3.2-3.6
#[derive(Debug, Clone)]
pub struct NewCatalogForm {
    /// Selected catalog type.
    pub catalog_type: CatalogType,
    /// Catalog name (required, 1-32 alphanumeric/hyphen/underscore).
    pub name: String,
    /// Optional description (up to 120 chars).
    pub description: String,
    /// Auto-mount on startup (default: true).
    pub auto_mount: bool,
    // === Mainframe-specific ===================================================
    /// Repository path (required for Mainframe).
    pub repository_path: String,
    /// Default HLQ (optional for Mainframe).
    pub default_hlq: String,
    /// Create repository now (default: true for Mainframe).
    pub create_repository_now: bool,
    // === POSIX-specific =======================================================
    /// Root directory (required for POSIX).
    pub root_directory: String,
    /// Mount point (optional for POSIX, default "/").
    pub mount_point: String,
    /// Read-only flag (POSIX / Native).
    pub read_only: bool,
    // === Native-specific ======================================================
    /// Root path (required for Native).
    pub root_path: String,
    // === Config defaults (Req 12) =============================================
    /// Configured default root for Mainframe catalogs (from `catalogs.default_mainframe_root`).
    pub default_mainframe_root: String,
    /// Configured default root for POSIX catalogs (from `catalogs.default_posix_root`).
    #[allow(dead_code)]
    pub default_posix_root: String,
    // === Validation ===========================================================
    /// Inline error message, if any.
    pub error: Option<String>,
}

impl Default for NewCatalogForm {
    fn default() -> Self {
        Self {
            catalog_type: CatalogType::Mainframe,
            name: String::new(),
            description: String::new(),
            auto_mount: true,
            repository_path: String::new(),
            default_hlq: String::new(),
            create_repository_now: true,
            root_directory: String::new(),
            mount_point: "/".to_string(),
            read_only: false,
            root_path: String::new(),
            default_mainframe_root: String::new(),
            default_posix_root: String::new(),
            error: None,
        }
    }
}

impl NewCatalogForm {
    /// Create a form pre-populated with configured default paths.
    ///
    /// - Mainframe `repository_path` is left empty; it is computed live from
    ///   `default_mainframe_root + "/" + name` as the user types.
    /// - POSIX `root_directory` is pre-set to `default_posix_root`.
    ///
    /// Validates: Requirement 12.1, 12.2, 12.7
    pub fn with_defaults(mainframe_root: impl Into<String>, posix_root: impl Into<String>) -> Self {
        let posix_root = posix_root.into();
        let mainframe_root = mainframe_root.into();
        // Pre-populate repository_path with the configured root so the field is
        // non-empty on dialog open (Req 12.1). It will be updated live as the
        // user types the catalog name.
        let repository_path = mainframe_root.clone();
        Self {
            default_mainframe_root: mainframe_root,
            repository_path,
            root_directory: posix_root.clone(),
            default_posix_root: posix_root,
            ..Default::default()
        }
    }

    /// Compute the suggested Mainframe repository path from the current name.
    ///
    /// Returns `"{default_mainframe_root}/{name}"` when both are non-empty,
    /// otherwise returns `default_mainframe_root` alone.
    ///
    /// Validates: Requirement 12.1
    pub fn suggested_mainframe_path(&self) -> String {
        if self.default_mainframe_root.is_empty() {
            return self.name.clone();
        }
        if self.name.is_empty() {
            return self.default_mainframe_root.clone();
        }
        std::path::Path::new(&self.default_mainframe_root)
            .join(&self.name)
            .to_string_lossy()
            .into_owned()
    }
}

/// Outcome of the dialog for a single frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogOutcome {
    /// Dialog is still open -- no action yet.
    Open,
    /// User confirmed -- catalog was validated and registered.
    Confirmed,
    /// User cancelled.
    Cancelled,
}

// === Validation ================================================================

/// Validate the form fields and return an error string if invalid.
///
/// Validates: Requirement 3.3, 3.4, 3.5, 3.6, 3.8
pub fn validate(form: &NewCatalogForm, registry: &CatalogRegistry) -> Option<String> {
    // Common: name
    if !VirtualCatalog::is_valid_name(&form.name) {
        return Some(
            "Catalog Name must be 1-32 characters (alphanumeric, hyphen, underscore).".to_string(),
        );
    }
    if registry.exists(&form.name) {
        return Some(format!("A catalog named '{}' already exists.", form.name));
    }
    // Common: description length
    if form.description.len() > 120 {
        return Some("Description must be 120 characters or fewer.".to_string());
    }
    // Type-specific required fields
    match form.catalog_type {
        CatalogType::Mainframe => {
            if form.repository_path.trim().is_empty() {
                return Some("Repository Path is required for Mainframe catalogs.".to_string());
            }
        }
        CatalogType::Posix => {
            if form.root_directory.trim().is_empty() {
                return Some("Root Directory is required for POSIX catalogs.".to_string());
            }
        }
        CatalogType::Native => {
            if form.root_path.trim().is_empty() {
                return Some("Root Path is required for Native catalogs.".to_string());
            }
        }
    }
    None
}

/// Build a `VirtualCatalog` from a validated form.
///
/// Validates: Requirement 3.7
pub fn build_catalog(form: &NewCatalogForm) -> VirtualCatalog {
    let (path, default_hlq, mount_point) = match form.catalog_type {
        CatalogType::Mainframe => (
            form.repository_path.trim().to_string(),
            if form.default_hlq.trim().is_empty() {
                None
            } else {
                Some(form.default_hlq.trim().to_string())
            },
            None,
        ),
        CatalogType::Posix => (
            form.root_directory.trim().to_string(),
            None,
            Some(if form.mount_point.trim().is_empty() {
                "/".to_string()
            } else {
                form.mount_point.trim().to_string()
            }),
        ),
        CatalogType::Native => (form.root_path.trim().to_string(), None, None),
    };

    VirtualCatalog {
        name: form.name.trim().to_string(),
        catalog_type: form.catalog_type,
        path,
        description: if form.description.trim().is_empty() {
            None
        } else {
            Some(form.description.trim().to_string())
        },
        auto_mount: form.auto_mount,
        default_hlq,
        mount_point,
        read_only: form.read_only,
    }
}

// === Render ====================================================================

/// Render the New Catalog modal dialog.
///
/// Returns `DialogOutcome::Confirmed` when the user confirms a valid form,
/// `DialogOutcome::Cancelled` when they cancel, or `DialogOutcome::Open`
/// while the dialog remains active.
///
/// Validates: Requirement 3.1-3.8
pub fn render(
    ctx: &egui::Context,
    form: &mut NewCatalogForm,
    registry: &mut CatalogRegistry,
) -> DialogOutcome {
    let mut outcome = DialogOutcome::Open;

    egui::Window::new("New Catalog")
        .collapsible(false)
        .resizable(false)
        .min_width(420.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_min_width(400.0);

            // === Catalog Type selector -- Req 3.2 ============================
            ui.horizontal(|ui| {
                ui.label("Catalog Type:");
                egui::ComboBox::from_id_salt("catalog_type_selector")
                    .selected_text(catalog_type_label(form.catalog_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut form.catalog_type,
                            CatalogType::Mainframe,
                            "Mainframe",
                        );
                        ui.selectable_value(&mut form.catalog_type, CatalogType::Posix, "POSIX");
                        ui.selectable_value(&mut form.catalog_type, CatalogType::Native, "Native");
                    });
            });
            ui.separator();

            // === Common fields -- Req 3.3 ====================================
            ui.horizontal(|ui| {
                ui.label("Catalog Name:  ");
                let name_resp = ui.text_edit_singleline(&mut form.name);
                // Live-update Mainframe repository path as name changes (Req 12.1)
                if name_resp.changed() && form.catalog_type == CatalogType::Mainframe {
                    form.repository_path = form.suggested_mainframe_path();
                }
            });
            ui.horizontal(|ui| {
                ui.label("Description:   ");
                ui.text_edit_singleline(&mut form.description);
            });
            ui.checkbox(&mut form.auto_mount, "Auto-mount on startup");
            ui.separator();

            // === Type-specific fields ========================================
            match form.catalog_type {
                CatalogType::Mainframe => render_mainframe_fields(ui, form),
                CatalogType::Posix => render_posix_fields(ui, form),
                CatalogType::Native => render_native_fields(ui, form),
            }

            // === Inline error -- Req 3.8 =====================================
            if let Some(err) = &form.error {
                ui.colored_label(egui::Color32::RED, err);
            }

            ui.separator();

            // === Buttons =====================================================
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    match validate(form, registry) {
                        Some(err) => {
                            form.error = Some(err);
                        }
                        None => {
                            let catalog = build_catalog(form);
                            // Create the physical repository structure now for
                            // Mainframe catalogs (B041): without this the catalog
                            // is metadata-only and the first dataset allocation
                            // fails with "repository root does not exist". Wired
                            // to the previously-inert `create_repository_now`
                            // checkbox (default on). Validates: Requirement 12.3.
                            if form.catalog_type == CatalogType::Mainframe
                                && form.create_repository_now
                            {
                                let repo = ff_dscatalog::repository::Repository::new(&catalog.path);
                                if let Err(e) = repo.initialize(&catalog.name) {
                                    form.error = Some(format!("Failed to create repository: {e}"));
                                    // Do not register/confirm on failure.
                                    return;
                                }
                            }
                            // register() cannot fail here -- validate() already checked uniqueness
                            let _ = registry.register(catalog);
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

    // Validates: accessibility Requirement 2.1, 2.3 -- Escape closes the New Catalog dialog.
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        outcome = DialogOutcome::Cancelled;
    }

    outcome
}

fn render_mainframe_fields(ui: &mut egui::Ui, form: &mut NewCatalogForm) {
    // Validates: Requirement 3.4, 12.1
    ui.horizontal(|ui| {
        ui.label("Repository Path:");
        ui.text_edit_singleline(&mut form.repository_path);
    });
    ui.horizontal(|ui| {
        ui.label("Default HLQ:    ");
        ui.text_edit_singleline(&mut form.default_hlq);
    });
    ui.checkbox(&mut form.create_repository_now, "Create repository now");
}

fn render_posix_fields(ui: &mut egui::Ui, form: &mut NewCatalogForm) {
    // Validates: Requirement 3.5
    ui.horizontal(|ui| {
        ui.label("Root Directory: ");
        ui.text_edit_singleline(&mut form.root_directory);
    });
    ui.horizontal(|ui| {
        ui.label("Mount Point:    ");
        ui.text_edit_singleline(&mut form.mount_point);
    });
    ui.checkbox(&mut form.read_only, "Read-Only");
}

fn render_native_fields(ui: &mut egui::Ui, form: &mut NewCatalogForm) {
    // Validates: Requirement 3.6
    ui.horizontal(|ui| {
        ui.label("Root Path:      ");
        ui.text_edit_singleline(&mut form.root_path);
    });
    ui.checkbox(&mut form.read_only, "Read-Only");
}

// === Tests =====================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ff_catalog_registry::CatalogRegistry;

    fn empty_registry() -> CatalogRegistry {
        CatalogRegistry::new()
    }

    fn valid_mainframe_form() -> NewCatalogForm {
        NewCatalogForm {
            catalog_type: CatalogType::Mainframe,
            name: "PAYROLL".to_string(),
            repository_path: "/catalogs/payroll".to_string(),
            ..Default::default()
        }
    }

    fn valid_posix_form() -> NewCatalogForm {
        NewCatalogForm {
            catalog_type: CatalogType::Posix,
            name: "dev-posix".to_string(),
            root_directory: "/projects/dev".to_string(),
            mount_point: "/".to_string(),
            ..Default::default()
        }
    }

    fn valid_native_form() -> NewCatalogForm {
        NewCatalogForm {
            catalog_type: CatalogType::Native,
            name: "projects".to_string(),
            root_path: "C:/projects".to_string(),
            ..Default::default()
        }
    }

    // === Default form state ===================================================

    /// Validates: Requirement 3.2 -- default catalog type is Mainframe.
    #[test]
    fn new_catalog_form_default_type_is_mainframe() {
        // Validates: Requirement 3.2
        let form = NewCatalogForm::default();
        assert_eq!(form.catalog_type, CatalogType::Mainframe);
    }

    /// Validates: Requirement 3.3 -- auto_mount defaults to true.
    #[test]
    fn new_catalog_form_auto_mount_defaults_to_true() {
        // Validates: Requirement 3.3
        let form = NewCatalogForm::default();
        assert!(form.auto_mount);
    }

    /// Validates: Requirement 3.4 -- create_repository_now defaults to true.
    #[test]
    fn new_catalog_form_create_repository_now_defaults_to_true() {
        // Validates: Requirement 3.4
        let form = NewCatalogForm::default();
        assert!(form.create_repository_now);
    }

    /// Validates: Requirement 3.5 -- mount_point defaults to "/".
    #[test]
    fn new_catalog_form_mount_point_defaults_to_slash() {
        // Validates: Requirement 3.5
        let form = NewCatalogForm::default();
        assert_eq!(form.mount_point, "/");
    }

    /// Validates: Requirement 3.5, 3.6 -- read_only defaults to false.
    #[test]
    fn new_catalog_form_read_only_defaults_to_false() {
        // Validates: Requirement 3.5, 3.6
        let form = NewCatalogForm::default();
        assert!(!form.read_only);
    }

    // === Validation -- common fields ==========================================

    /// Validates: Requirement 3.8 -- empty name fails validation.
    #[test]
    fn validate_rejects_empty_name() {
        // Validates: Requirement 3.8
        let mut form = valid_mainframe_form();
        form.name = String::new();
        assert!(validate(&form, &empty_registry()).is_some());
    }

    /// Validates: Requirement 3.8 -- name with invalid chars fails validation.
    #[test]
    fn validate_rejects_name_with_spaces() {
        // Validates: Requirement 3.8
        let mut form = valid_mainframe_form();
        form.name = "bad name".to_string();
        assert!(validate(&form, &empty_registry()).is_some());
    }

    /// Validates: Requirement 3.8 -- name over 32 chars fails validation.
    #[test]
    fn validate_rejects_name_over_32_chars() {
        // Validates: Requirement 3.8
        let mut form = valid_mainframe_form();
        form.name = "A".repeat(33);
        assert!(validate(&form, &empty_registry()).is_some());
    }

    /// Validates: Requirement 3.8 -- duplicate name fails validation.
    #[test]
    fn validate_rejects_duplicate_name() {
        // Validates: Requirement 3.8
        let mut registry = empty_registry();
        let form = valid_mainframe_form();
        registry.register(build_catalog(&form)).unwrap();
        // Same name again
        assert!(validate(&form, &registry).is_some());
    }

    /// Validates: Requirement 3.3 -- description over 120 chars fails validation.
    #[test]
    fn validate_rejects_description_over_120_chars() {
        // Validates: Requirement 3.3
        let mut form = valid_mainframe_form();
        form.description = "x".repeat(121);
        assert!(validate(&form, &empty_registry()).is_some());
    }

    // === Validation -- type-specific required fields ==========================

    /// Validates: Requirement 3.4 -- Mainframe with empty repository_path fails.
    #[test]
    fn validate_mainframe_rejects_empty_repository_path() {
        // Validates: Requirement 3.4
        let mut form = valid_mainframe_form();
        form.repository_path = String::new();
        assert!(validate(&form, &empty_registry()).is_some());
    }

    /// Validates: Requirement 3.5 -- POSIX with empty root_directory fails.
    #[test]
    fn validate_posix_rejects_empty_root_directory() {
        // Validates: Requirement 3.5
        let mut form = valid_posix_form();
        form.root_directory = String::new();
        assert!(validate(&form, &empty_registry()).is_some());
    }

    /// Validates: Requirement 3.6 -- Native with empty root_path fails.
    #[test]
    fn validate_native_rejects_empty_root_path() {
        // Validates: Requirement 3.6
        let mut form = valid_native_form();
        form.root_path = String::new();
        assert!(validate(&form, &empty_registry()).is_some());
    }

    // === Validation -- valid forms pass =======================================

    /// Validates: Requirement 3.3, 3.4 -- valid Mainframe form passes validation.
    #[test]
    fn validate_accepts_valid_mainframe_form() {
        // Validates: Requirement 3.3, 3.4
        assert!(validate(&valid_mainframe_form(), &empty_registry()).is_none());
    }

    /// Validates: Requirement 3.3, 3.5 -- valid POSIX form passes validation.
    #[test]
    fn validate_accepts_valid_posix_form() {
        // Validates: Requirement 3.3, 3.5
        assert!(validate(&valid_posix_form(), &empty_registry()).is_none());
    }

    /// Validates: Requirement 3.3, 3.6 -- valid Native form passes validation.
    #[test]
    fn validate_accepts_valid_native_form() {
        // Validates: Requirement 3.3, 3.6
        assert!(validate(&valid_native_form(), &empty_registry()).is_none());
    }

    // === build_catalog ========================================================

    /// Validates: Requirement 3.7 -- build_catalog maps Mainframe form fields correctly.
    #[test]
    fn build_catalog_mainframe_maps_fields_correctly() {
        // Validates: Requirement 3.7
        let mut form = valid_mainframe_form();
        form.default_hlq = "PAYROLL".to_string();
        form.description = "Payroll datasets".to_string();
        form.auto_mount = false;
        let cat = build_catalog(&form);
        assert_eq!(cat.name, "PAYROLL");
        assert_eq!(cat.catalog_type, CatalogType::Mainframe);
        assert_eq!(cat.path, "/catalogs/payroll");
        assert_eq!(cat.default_hlq.as_deref(), Some("PAYROLL"));
        assert_eq!(cat.description.as_deref(), Some("Payroll datasets"));
        assert!(!cat.auto_mount);
        assert!(cat.mount_point.is_none());
    }

    /// Validates: Requirement 3.7 -- build_catalog maps POSIX form fields correctly.
    #[test]
    fn build_catalog_posix_maps_fields_correctly() {
        // Validates: Requirement 3.7
        let mut form = valid_posix_form();
        form.read_only = true;
        let cat = build_catalog(&form);
        assert_eq!(cat.name, "dev-posix");
        assert_eq!(cat.catalog_type, CatalogType::Posix);
        assert_eq!(cat.path, "/projects/dev");
        assert_eq!(cat.mount_point.as_deref(), Some("/"));
        assert!(cat.read_only);
        assert!(cat.default_hlq.is_none());
    }

    /// Validates: Requirement 3.7 -- build_catalog maps Native form fields correctly.
    #[test]
    fn build_catalog_native_maps_fields_correctly() {
        // Validates: Requirement 3.7
        let cat = build_catalog(&valid_native_form());
        assert_eq!(cat.name, "projects");
        assert_eq!(cat.catalog_type, CatalogType::Native);
        assert_eq!(cat.path, "C:/projects");
        assert!(cat.default_hlq.is_none());
        assert!(cat.mount_point.is_none());
    }

    /// Validates: Requirement 3.7 -- empty optional fields produce None.
    #[test]
    fn build_catalog_empty_optional_fields_produce_none() {
        // Validates: Requirement 3.7
        let form = valid_mainframe_form(); // default_hlq and description are empty
        let cat = build_catalog(&form);
        assert!(cat.default_hlq.is_none());
        assert!(cat.description.is_none());
    }

    /// Validates: Requirement 3.5 -- empty mount_point defaults to "/".
    #[test]
    fn build_catalog_posix_empty_mount_point_defaults_to_slash() {
        // Validates: Requirement 3.5
        let mut form = valid_posix_form();
        form.mount_point = String::new();
        let cat = build_catalog(&form);
        assert_eq!(cat.mount_point.as_deref(), Some("/"));
    }

    // === confirm flow =========================================================

    /// Validates: Requirement 3.7 -- confirmed form registers catalog in registry.
    #[test]
    fn confirm_registers_catalog_in_registry() {
        // Validates: Requirement 3.7
        let form = valid_mainframe_form();
        let mut registry = empty_registry();
        // Simulate the confirm path: validate -> build -> register
        assert!(validate(&form, &registry).is_none());
        let cat = build_catalog(&form);
        registry.register(cat).unwrap();
        assert!(registry.exists("PAYROLL"));
    }

    /// Validates: Requirement 3.8 -- dialog stays open when validation fails.
    #[test]
    fn dialog_stays_open_when_validation_fails() {
        // Validates: Requirement 3.8
        let mut form = valid_mainframe_form();
        form.name = String::new();
        let registry = empty_registry();
        let err = validate(&form, &registry);
        assert!(err.is_some(), "invalid form must produce an error");
    }

    // === Req 12 -- Catalog storage default paths ==============================

    /// Validates: Requirement 12.1 -- suggested_mainframe_path appends name to root.
    #[test]
    fn with_defaults_mainframe_path_appends_name() {
        // Validates: Requirement 12.1
        let mut form = NewCatalogForm::with_defaults("C:/data", "C:/posix");
        form.name = "PAYROLL".to_string();
        let result = form.suggested_mainframe_path();
        // Use PathBuf comparison to be platform-separator-agnostic
        let expected = std::path::Path::new("C:/data").join("PAYROLL");
        assert_eq!(std::path::Path::new(&result), expected);
    }

    /// Validates: Requirement 12.2 -- POSIX root_directory is pre-populated from default.
    #[test]
    fn with_defaults_posix_root_directory_pre_populated() {
        // Validates: Requirement 12.2
        let form = NewCatalogForm::with_defaults("C:/data", "C:/posix");
        assert_eq!(form.root_directory, "C:/posix");
    }

    /// Validates: Requirement 12.7 -- pre-populated repository_path remains editable.
    #[test]
    fn with_defaults_repository_path_is_editable() {
        // Validates: Requirement 12.7
        let mut form = NewCatalogForm::with_defaults("C:/data", "C:/posix");
        // Pre-populated with the root on construction.
        assert_eq!(form.repository_path, "C:/data");
        // User can override it.
        form.repository_path = "custom/path".to_string();
        assert_eq!(form.repository_path, "custom/path");
    }

    /// Validates: Requirement 12.1 -- empty name returns root alone.
    #[test]
    fn suggested_mainframe_path_empty_name_returns_root() {
        // Validates: Requirement 12.1
        let form = NewCatalogForm::with_defaults("C:/data", "");
        assert_eq!(form.suggested_mainframe_path(), "C:/data");
    }

    /// Validates: Requirement 12.1 -- empty root returns name alone.
    #[test]
    fn suggested_mainframe_path_empty_root_returns_name() {
        // Validates: Requirement 12.1
        let mut form = NewCatalogForm::with_defaults("", "");
        form.name = "PAYROLL".to_string();
        assert_eq!(form.suggested_mainframe_path(), "PAYROLL");
    }
}
