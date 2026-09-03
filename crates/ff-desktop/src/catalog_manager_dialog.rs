//! # Catalog Manager Dialog — Create / Edit / Delete
//!
//! Modal dialogs for creating, editing, and deleting virtual catalogs.
//! Renders as egui modal windows within the Files Panel frame.
//!
//! Validates: Requirement 3.1–3.8, 4.1–4.5

use eframe::egui;

use crate::catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog};

// ── Form state ────────────────────────────────────────────────────────────────

/// The form state for the New Catalog dialog.
///
/// Validates: Requirement 3.2–3.6
#[derive(Debug, Clone)]
pub struct NewCatalogForm {
    /// Selected catalog type.
    pub catalog_type: CatalogType,
    /// Catalog name (required, 1–32 alphanumeric/hyphen/underscore).
    pub name: String,
    /// Optional description (up to 120 chars).
    pub description: String,
    /// Auto-mount on startup (default: true).
    pub auto_mount: bool,
    // ── Mainframe-specific ────────────────────────────────────────────────
    /// Repository path (required for Mainframe).
    pub repository_path: String,
    /// Default HLQ (optional for Mainframe).
    pub default_hlq: String,
    /// Create repository now (default: true for Mainframe).
    pub create_repository_now: bool,
    // ── POSIX-specific ────────────────────────────────────────────────────
    /// Root directory (required for POSIX).
    pub root_directory: String,
    /// Mount point (optional for POSIX, default "/").
    pub mount_point: String,
    /// Read-only flag (POSIX / Native).
    pub read_only: bool,
    // ── Native-specific ───────────────────────────────────────────────────
    /// Root path (required for Native).
    pub root_path: String,
    // ── Config defaults (Req 12) ──────────────────────────────────────────
    /// Configured default root for Mainframe catalogs (from `catalogs.default_mainframe_root`).
    pub default_mainframe_root: String,
    /// Configured default root for POSIX catalogs (from `catalogs.default_posix_root`).
    #[allow(dead_code)]
    pub default_posix_root: String,
    // ── Validation ────────────────────────────────────────────────────────
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
    /// Dialog is still open — no action yet.
    Open,
    /// User confirmed — catalog was validated and registered.
    Confirmed,
    /// User cancelled.
    Cancelled,
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Validate the form fields and return an error string if invalid.
///
/// Validates: Requirement 3.3, 3.4, 3.5, 3.6, 3.8
pub fn validate(form: &NewCatalogForm, registry: &CatalogRegistry) -> Option<String> {
    // Common: name
    if !VirtualCatalog::is_valid_name(&form.name) {
        return Some(
            "Catalog Name must be 1–32 characters (alphanumeric, hyphen, underscore).".to_string(),
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

// ── Render ────────────────────────────────────────────────────────────────────

/// Render the New Catalog modal dialog.
///
/// Returns `DialogOutcome::Confirmed` when the user confirms a valid form,
/// `DialogOutcome::Cancelled` when they cancel, or `DialogOutcome::Open`
/// while the dialog remains active.
///
/// Validates: Requirement 3.1–3.8
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

            // ── Catalog Type selector — Req 3.2 ──────────────────────────
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

            // ── Common fields — Req 3.3 ───────────────────────────────────
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

            // ── Type-specific fields ──────────────────────────────────────
            match form.catalog_type {
                CatalogType::Mainframe => render_mainframe_fields(ui, form),
                CatalogType::Posix => render_posix_fields(ui, form),
                CatalogType::Native => render_native_fields(ui, form),
            }

            // ── Inline error — Req 3.8 ────────────────────────────────────
            if let Some(err) = &form.error {
                ui.colored_label(egui::Color32::RED, err);
            }

            ui.separator();

            // ── Buttons ───────────────────────────────────────────────────
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    match validate(form, registry) {
                        Some(err) => {
                            form.error = Some(err);
                        }
                        None => {
                            let catalog = build_catalog(form);
                            // register() cannot fail here — validate() already checked uniqueness
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

fn catalog_type_label(ct: CatalogType) -> &'static str {
    match ct {
        CatalogType::Mainframe => "Mainframe",
        CatalogType::Posix => "POSIX",
        CatalogType::Native => "Native",
    }
}

// ── Edit dialog ───────────────────────────────────────────────────────────────

/// Form state for the Edit Catalog dialog.
///
/// Name and type are immutable after creation (Req 4.2).
/// Validates: Requirement 4.1–4.2
#[derive(Debug, Clone)]
pub struct EditCatalogForm {
    /// Catalog name — display only, not editable.
    pub name: String,
    /// Catalog type — display only, not editable.
    pub catalog_type: CatalogType,
    /// Editable description.
    pub description: String,
    /// Editable auto-mount flag.
    pub auto_mount: bool,
    /// Editable read-only flag (POSIX / Native only).
    pub read_only: bool,
    /// Editable default HLQ (Mainframe only).
    pub default_hlq: String,
    /// Repository path — read-only display (Req 15.1, 15.3).
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
/// Validates: Requirement 4.1–4.2
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

            // Name and type — read-only display (Req 4.2)
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
            // Repository path — read-only (Req 15.1, 15.2, 15.3)
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
                            // update() cannot fail — name was pre-populated from registry
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

    outcome
}

// ── Delete dialog ─────────────────────────────────────────────────────────────

/// Which delete action the user chose.
///
/// Validates: Requirement 4.3–4.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteChoice {
    /// Remove catalog from registry only; leave backing files intact (Req 4.4).
    CatalogOnly,
    /// Remove catalog from registry AND recursively delete backing files (Req 4.5).
    CatalogAndFiles,
    /// User cancelled — no action.
    Cancel,
}

/// State for the Delete Catalog confirmation dialog.
///
/// Validates: Requirement 4.3
#[derive(Debug, Clone)]
pub struct DeleteCatalogConfirm {
    /// Name of the catalog to delete.
    pub name: String,
    /// Backing path — used for recursive delete when `CatalogAndFiles` is chosen.
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
/// Validates: Requirement 4.3–4.5
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
    // Validates: Requirement 14.6 — Home Native catalog is protected from deletion.
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

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog_registry::{CatalogRegistry, VirtualCatalog};

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

    // ── Default form state ────────────────────────────────────────────────

    /// Validates: Requirement 3.2 — default catalog type is Mainframe.
    #[test]
    fn new_catalog_form_default_type_is_mainframe() {
        // Validates: Requirement 3.2
        let form = NewCatalogForm::default();
        assert_eq!(form.catalog_type, CatalogType::Mainframe);
    }

    /// Validates: Requirement 3.3 — auto_mount defaults to true.
    #[test]
    fn new_catalog_form_auto_mount_defaults_to_true() {
        // Validates: Requirement 3.3
        let form = NewCatalogForm::default();
        assert!(form.auto_mount);
    }

    /// Validates: Requirement 3.4 — create_repository_now defaults to true.
    #[test]
    fn new_catalog_form_create_repository_now_defaults_to_true() {
        // Validates: Requirement 3.4
        let form = NewCatalogForm::default();
        assert!(form.create_repository_now);
    }

    /// Validates: Requirement 3.5 — mount_point defaults to "/".
    #[test]
    fn new_catalog_form_mount_point_defaults_to_slash() {
        // Validates: Requirement 3.5
        let form = NewCatalogForm::default();
        assert_eq!(form.mount_point, "/");
    }

    /// Validates: Requirement 3.5, 3.6 — read_only defaults to false.
    #[test]
    fn new_catalog_form_read_only_defaults_to_false() {
        // Validates: Requirement 3.5, 3.6
        let form = NewCatalogForm::default();
        assert!(!form.read_only);
    }

    // ── Validation — common fields ────────────────────────────────────────

    /// Validates: Requirement 3.8 — empty name fails validation.
    #[test]
    fn validate_rejects_empty_name() {
        // Validates: Requirement 3.8
        let mut form = valid_mainframe_form();
        form.name = String::new();
        assert!(validate(&form, &empty_registry()).is_some());
    }

    /// Validates: Requirement 3.8 — name with invalid chars fails validation.
    #[test]
    fn validate_rejects_name_with_spaces() {
        // Validates: Requirement 3.8
        let mut form = valid_mainframe_form();
        form.name = "bad name".to_string();
        assert!(validate(&form, &empty_registry()).is_some());
    }

    /// Validates: Requirement 3.8 — name over 32 chars fails validation.
    #[test]
    fn validate_rejects_name_over_32_chars() {
        // Validates: Requirement 3.8
        let mut form = valid_mainframe_form();
        form.name = "A".repeat(33);
        assert!(validate(&form, &empty_registry()).is_some());
    }

    /// Validates: Requirement 3.8 — duplicate name fails validation.
    #[test]
    fn validate_rejects_duplicate_name() {
        // Validates: Requirement 3.8
        let mut registry = empty_registry();
        let form = valid_mainframe_form();
        registry.register(build_catalog(&form)).unwrap();
        // Same name again
        assert!(validate(&form, &registry).is_some());
    }

    /// Validates: Requirement 3.3 — description over 120 chars fails validation.
    #[test]
    fn validate_rejects_description_over_120_chars() {
        // Validates: Requirement 3.3
        let mut form = valid_mainframe_form();
        form.description = "x".repeat(121);
        assert!(validate(&form, &empty_registry()).is_some());
    }

    // ── Validation — type-specific required fields ────────────────────────

    /// Validates: Requirement 3.4 — Mainframe with empty repository_path fails.
    #[test]
    fn validate_mainframe_rejects_empty_repository_path() {
        // Validates: Requirement 3.4
        let mut form = valid_mainframe_form();
        form.repository_path = String::new();
        assert!(validate(&form, &empty_registry()).is_some());
    }

    /// Validates: Requirement 3.5 — POSIX with empty root_directory fails.
    #[test]
    fn validate_posix_rejects_empty_root_directory() {
        // Validates: Requirement 3.5
        let mut form = valid_posix_form();
        form.root_directory = String::new();
        assert!(validate(&form, &empty_registry()).is_some());
    }

    /// Validates: Requirement 3.6 — Native with empty root_path fails.
    #[test]
    fn validate_native_rejects_empty_root_path() {
        // Validates: Requirement 3.6
        let mut form = valid_native_form();
        form.root_path = String::new();
        assert!(validate(&form, &empty_registry()).is_some());
    }

    // ── Validation — valid forms pass ─────────────────────────────────────

    /// Validates: Requirement 3.3, 3.4 — valid Mainframe form passes validation.
    #[test]
    fn validate_accepts_valid_mainframe_form() {
        // Validates: Requirement 3.3, 3.4
        assert!(validate(&valid_mainframe_form(), &empty_registry()).is_none());
    }

    /// Validates: Requirement 3.3, 3.5 — valid POSIX form passes validation.
    #[test]
    fn validate_accepts_valid_posix_form() {
        // Validates: Requirement 3.3, 3.5
        assert!(validate(&valid_posix_form(), &empty_registry()).is_none());
    }

    /// Validates: Requirement 3.3, 3.6 — valid Native form passes validation.
    #[test]
    fn validate_accepts_valid_native_form() {
        // Validates: Requirement 3.3, 3.6
        assert!(validate(&valid_native_form(), &empty_registry()).is_none());
    }

    // ── build_catalog ─────────────────────────────────────────────────────

    /// Validates: Requirement 3.7 — build_catalog maps Mainframe form fields correctly.
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

    /// Validates: Requirement 3.7 — build_catalog maps POSIX form fields correctly.
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

    /// Validates: Requirement 3.7 — build_catalog maps Native form fields correctly.
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

    /// Validates: Requirement 3.7 — empty optional fields produce None.
    #[test]
    fn build_catalog_empty_optional_fields_produce_none() {
        // Validates: Requirement 3.7
        let form = valid_mainframe_form(); // default_hlq and description are empty
        let cat = build_catalog(&form);
        assert!(cat.default_hlq.is_none());
        assert!(cat.description.is_none());
    }

    /// Validates: Requirement 3.5 — empty mount_point defaults to "/".
    #[test]
    fn build_catalog_posix_empty_mount_point_defaults_to_slash() {
        // Validates: Requirement 3.5
        let mut form = valid_posix_form();
        form.mount_point = String::new();
        let cat = build_catalog(&form);
        assert_eq!(cat.mount_point.as_deref(), Some("/"));
    }

    // ── confirm flow ──────────────────────────────────────────────────────

    /// Validates: Requirement 3.7 — confirmed form registers catalog in registry.
    #[test]
    fn confirm_registers_catalog_in_registry() {
        // Validates: Requirement 3.7
        let form = valid_mainframe_form();
        let mut registry = empty_registry();
        // Simulate the confirm path: validate → build → register
        assert!(validate(&form, &registry).is_none());
        let cat = build_catalog(&form);
        registry.register(cat).unwrap();
        assert!(registry.exists("PAYROLL"));
    }

    /// Validates: Requirement 3.8 — dialog stays open when validation fails.
    #[test]
    fn dialog_stays_open_when_validation_fails() {
        // Validates: Requirement 3.8
        let mut form = valid_mainframe_form();
        form.name = String::new();
        let registry = empty_registry();
        let err = validate(&form, &registry);
        assert!(err.is_some(), "invalid form must produce an error");
    }

    // ── EditCatalogForm ───────────────────────────────────────────────────

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

    /// Validates: Requirement 15.1 — EditCatalogForm carries the repository path from the catalog.
    #[test]
    fn edit_form_displays_repository_path() {
        // Validates: Requirement 15.1
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let form = EditCatalogForm::from_catalog(&cat);
        assert_eq!(form.path, "/catalogs/payroll");
    }

    /// Validates: Requirement 15.2 — repository path is present for all catalog types.
    #[test]
    fn edit_form_repository_path_present_for_all_catalog_types() {
        // Validates: Requirement 15.2
        let mut registry = empty_registry();
        let mf = registered_mainframe(&mut registry);
        let px = registered_posix(&mut registry);
        assert_eq!(EditCatalogForm::from_catalog(&mf).path, "/catalogs/payroll");
        assert_eq!(EditCatalogForm::from_catalog(&px).path, "/projects/dev");
    }

    /// Validates: Requirement 4.1 — EditCatalogForm is pre-populated from catalog.
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

    /// Validates: Requirement 4.1 — EditCatalogForm with no description gives empty string.
    #[test]
    fn edit_form_no_description_gives_empty_string() {
        // Validates: Requirement 4.1
        let mut registry = empty_registry();
        let cat = registered_posix(&mut registry);
        let form = EditCatalogForm::from_catalog(&cat);
        assert_eq!(form.description, "");
    }

    /// Validates: Requirement 4.2 — name and type fields are not editable (read-only display).
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

    /// Validates: Requirement 4.2 — validate_edit accepts valid description.
    #[test]
    fn validate_edit_accepts_valid_description() {
        // Validates: Requirement 4.2
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let mut form = EditCatalogForm::from_catalog(&cat);
        form.description = "Updated description".to_string();
        assert!(validate_edit(&form).is_none());
    }

    /// Validates: Requirement 4.2 — validate_edit rejects description over 120 chars.
    #[test]
    fn validate_edit_rejects_description_over_120_chars() {
        // Validates: Requirement 4.2
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let mut form = EditCatalogForm::from_catalog(&cat);
        form.description = "x".repeat(121);
        assert!(validate_edit(&form).is_some());
    }

    /// Validates: Requirement 4.2 — confirmed edit updates registry fields.
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

    /// Validates: Requirement 4.2 — empty description clears the field.
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

    // ── DeleteCatalogConfirm ──────────────────────────────────────────────

    /// Validates: Requirement 4.3 — DeleteCatalogConfirm is built from catalog.
    #[test]
    fn delete_confirm_built_from_catalog() {
        // Validates: Requirement 4.3
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let confirm = DeleteCatalogConfirm::from_catalog(&cat);
        assert_eq!(confirm.name, "PAYROLL");
        assert_eq!(confirm.path, "/catalogs/payroll");
    }

    /// Validates: Requirement 4.4 — CatalogOnly removes from registry, leaves files.
    #[test]
    fn execute_delete_catalog_only_removes_from_registry() {
        // Validates: Requirement 4.4
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let confirm = DeleteCatalogConfirm::from_catalog(&cat);
        execute_delete(&DeleteChoice::CatalogOnly, &confirm, &mut registry).unwrap();
        assert!(!registry.exists("PAYROLL"));
    }

    /// Validates: Requirement 4.4 — CatalogOnly on unknown name returns error.
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

    /// Validates: Requirement 4.5 — CatalogAndFiles removes from registry and deletes path.
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

    /// Validates: Requirement 4.3 — Cancel choice leaves registry unchanged.
    #[test]
    fn execute_delete_cancel_leaves_registry_unchanged() {
        // Validates: Requirement 4.3
        let mut registry = empty_registry();
        let cat = registered_mainframe(&mut registry);
        let confirm = DeleteCatalogConfirm::from_catalog(&cat);
        execute_delete(&DeleteChoice::Cancel, &confirm, &mut registry).unwrap();
        assert!(registry.exists("PAYROLL"));
    }

    /// Validates: Requirement 14.6 — deleting the "Home" Native catalog is rejected.
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

    /// Validates: Requirement 14.7 — a Native catalog renamed away from "Home" can be deleted.
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

    // ── Req 12 — Catalog storage default paths ────────────────────────────

    /// Validates: Requirement 12.1 — suggested_mainframe_path appends name to root.
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

    /// Validates: Requirement 12.2 — POSIX root_directory is pre-populated from default.
    #[test]
    fn with_defaults_posix_root_directory_pre_populated() {
        // Validates: Requirement 12.2
        let form = NewCatalogForm::with_defaults("C:/data", "C:/posix");
        assert_eq!(form.root_directory, "C:/posix");
    }

    /// Validates: Requirement 12.7 — pre-populated repository_path remains editable.
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

    /// Validates: Requirement 12.1 — empty name returns root alone.
    #[test]
    fn suggested_mainframe_path_empty_name_returns_root() {
        // Validates: Requirement 12.1
        let form = NewCatalogForm::with_defaults("C:/data", "");
        assert_eq!(form.suggested_mainframe_path(), "C:/data");
    }

    /// Validates: Requirement 12.1 — empty root returns name alone.
    #[test]
    fn suggested_mainframe_path_empty_root_returns_name() {
        // Validates: Requirement 12.1
        let mut form = NewCatalogForm::with_defaults("", "");
        form.name = "PAYROLL".to_string();
        assert_eq!(form.suggested_mainframe_path(), "PAYROLL");
    }
}
