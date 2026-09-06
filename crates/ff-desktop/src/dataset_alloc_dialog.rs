//! # Dataset Allocation Dialog
//!
//! ISPF-style modal dialog for allocating (creating) a new mainframe dataset
//! within a Mainframe catalog.
//!
//! BLKSIZE defaults to 0 (system-determined). IBM recommends specifying
//! `BLKSIZE=0` and allowing z/OS (or the host OS in FFWB's case) to determine
//! the optimal block size for the underlying storage device. A non-zero value
//! may be entered as a user override.
//!
//! Validates: Requirement 5.1–5.6

// from_like and allocate_like are wired in Task 8 context menus; suppress until then.
#![allow(dead_code)]

use eframe::egui;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Dataset organisation (DSORG).
///
/// Validates: Requirement 5.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dsorg {
    /// Sequential dataset.
    Ps,
    /// Partitioned dataset (PDS).
    Po,
    /// Partitioned dataset extended (PDSE).
    Pdse,
    /// Generation Data Group.
    Gdg,
}

impl Dsorg {
    fn label(self) -> &'static str {
        match self {
            Dsorg::Ps => "PS",
            Dsorg::Po => "PO",
            Dsorg::Pdse => "PDSE",
            Dsorg::Gdg => "GDG",
        }
    }
}

/// Record format (RECFM).
///
/// Validates: Requirement 5.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recfm {
    Fb,
    F,
    Vb,
    V,
    U,
}

impl Recfm {
    fn label(self) -> &'static str {
        match self {
            Recfm::Fb => "FB",
            Recfm::F => "F",
            Recfm::Vb => "VB",
            Recfm::V => "V",
            Recfm::U => "U",
        }
    }
}

/// Form state for the Dataset Allocation dialog.
///
/// Validates: Requirement 5.2, 5.6
#[derive(Debug, Clone)]
pub struct AllocDatasetForm {
    /// Dataset name (required).
    pub dataset_name: String,
    /// Dataset organisation.
    pub dsorg: Dsorg,
    /// Record format.
    pub recfm: Recfm,
    /// Logical record length (default 80).
    pub lrecl: String,
    /// Block size (default 0 — system-determined).
    pub blksize: String,
    /// Directory blocks — shown only for PO / PDSE (default 10).
    pub dir_blocks: String,
    /// GDG limit 1–255 — shown only for GDG.
    pub gdg_limit: String,
    /// Scratch on roll-off — shown only for GDG (default true).
    pub scratch: bool,
    /// Optional description.
    pub description: String,
    /// Inline error message, if any.
    pub error: Option<String>,
    /// When true the form was pre-populated via Allocate Like (Req 5.6).
    pub allocate_like: bool,
}

impl Default for AllocDatasetForm {
    fn default() -> Self {
        Self {
            dataset_name: String::new(),
            dsorg: Dsorg::Ps,
            recfm: Recfm::Fb,
            lrecl: "80".to_string(),
            blksize: "0".to_string(),
            dir_blocks: "10".to_string(),
            gdg_limit: "10".to_string(),
            scratch: true,
            description: String::new(),
            error: None,
            allocate_like: false,
        }
    }
}

impl AllocDatasetForm {
    /// Create a form with the Dataset Name pre-populated from the catalog's Default HLQ.
    ///
    /// The field is set to `"{hlq}."` so the user only needs to type the remaining qualifiers.
    ///
    /// Validates: Requirement 5.7
    pub fn with_hlq(hlq: &str) -> Self {
        Self {
            dataset_name: format!("{hlq}."),
            ..Default::default()
        }
    }

    /// Pre-populate the form from an existing dataset's attributes (Allocate Like).
    ///
    /// Validates: Requirement 5.6
    pub fn from_like(
        dsorg: Dsorg,
        recfm: Recfm,
        lrecl: u32,
        blksize: u32,
        dir_blocks: Option<u32>,
        gdg_limit: Option<u32>,
        scratch: bool,
    ) -> Self {
        Self {
            dataset_name: String::new(), // user must supply new DSN
            dsorg,
            recfm,
            lrecl: lrecl.to_string(),
            blksize: blksize.to_string(),
            dir_blocks: dir_blocks.unwrap_or(10).to_string(),
            gdg_limit: gdg_limit.unwrap_or(10).to_string(),
            scratch,
            description: String::new(),
            error: None,
            allocate_like: true,
        }
    }
}

/// Outcome of the dialog for a single frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllocOutcome {
    /// Dialog is still open.
    Open,
    /// User confirmed with valid parameters.
    Confirmed,
    /// User cancelled.
    Cancelled,
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Validated allocation parameters, produced by `validate()`.
///
/// Validates: Requirement 5.3
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllocParams {
    pub dataset_name: String,
    pub dsorg: Dsorg,
    pub recfm: Recfm,
    pub lrecl: u32,
    pub blksize: u32,
    pub dir_blocks: Option<u32>,
    pub gdg_limit: Option<u32>,
    pub scratch: bool,
    pub description: Option<String>,
}

/// Validate the allocation form.
///
/// Returns `Ok(AllocParams)` on success or `Err(message)` on failure.
///
/// Validates: Requirement 5.3, 5.8, dataset-catalog Requirement 7.10
pub fn validate(form: &AllocDatasetForm) -> Result<AllocParams, String> {
    // Dataset name required
    if form.dataset_name.trim().is_empty() {
        return Err("Dataset Name is required.".to_string());
    }

    // LRECL: integer, 1–32760
    let lrecl: u32 = form
        .lrecl
        .trim()
        .parse()
        .map_err(|_| "LRECL must be a positive integer.".to_string())?;
    if lrecl == 0 || lrecl > 32760 {
        return Err(format!("LRECL must be between 1 and 32760 (got {lrecl})."));
    }

    // BLKSIZE: 0 = system-determined (accepted as-is); otherwise must be >= LRECL
    let blksize: u32 = form
        .blksize
        .trim()
        .parse()
        .map_err(|_| "Block Size must be a non-negative integer.".to_string())?;
    if blksize != 0 && blksize < lrecl {
        return Err(format!(
            "Block Size ({blksize}) must be >= LRECL ({lrecl}), or 0 for system-determined."
        ));
    }

    // Directory Blocks — only for PO / PDSE
    let dir_blocks = match form.dsorg {
        Dsorg::Po | Dsorg::Pdse => {
            let db: u32 = form
                .dir_blocks
                .trim()
                .parse()
                .map_err(|_| "Directory Blocks must be a positive integer.".to_string())?;
            Some(db)
        }
        _ => None,
    };

    // GDG Limit — only for GDG, must be 1–255
    let gdg_limit = match form.dsorg {
        Dsorg::Gdg => {
            let limit: u32 = form
                .gdg_limit
                .trim()
                .parse()
                .map_err(|_| "GDG Limit must be an integer between 1 and 255.".to_string())?;
            if limit == 0 || limit > 255 {
                return Err(format!(
                    "GDG Limit must be between 1 and 255 (got {limit})."
                ));
            }
            Some(limit)
        }
        _ => None,
    };

    let description = if form.description.trim().is_empty() {
        None
    } else {
        Some(form.description.trim().to_string())
    };

    // Req 5.8 — Mainframe DSNs are always uppercase
    let dataset_name = form.dataset_name.trim().to_uppercase();

    Ok(AllocParams {
        dataset_name,
        dsorg: form.dsorg,
        recfm: form.recfm,
        lrecl,
        blksize,
        dir_blocks,
        gdg_limit,
        scratch: form.scratch,
        description,
    })
}

/// Validate the allocation form AND check for duplicate DSN within the existing dataset list.
///
/// `existing_names` is a slice of already-allocated DSNs for the target catalog.
/// Returns `Ok(AllocParams)` on success or `Err(message)` on failure.
///
/// Validates: Requirement 5.9
pub fn validate_for_catalog(
    form: &AllocDatasetForm,
    existing_names: &[String],
) -> Result<AllocParams, String> {
    let params = validate(form)?;
    let upper = params.dataset_name.to_uppercase();
    if existing_names.iter().any(|n| n.to_uppercase() == upper) {
        return Err(format!(
            "Dataset '{}' already exists in this catalog.",
            upper
        ));
    }
    Ok(params)
}

// ── Render ────────────────────────────────────────────────────────────────────

/// Render the Dataset Allocation modal dialog.
///
/// Returns `AllocOutcome::Confirmed` (with validated params stored in `form`)
/// when the user confirms, `AllocOutcome::Cancelled` when they cancel, or
/// `AllocOutcome::Open` while the dialog remains active.
///
/// Validates: Requirement 5.1–5.5
pub fn render(ctx: &egui::Context, form: &mut AllocDatasetForm) -> AllocOutcome {
    let mut outcome = AllocOutcome::Open;

    egui::Window::new("Allocate Dataset")
        .collapsible(false)
        .resizable(false)
        .min_width(460.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_min_width(440.0);

            // ── Dataset Name ─────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Dataset Name:      ");
                ui.text_edit_singleline(&mut form.dataset_name);
            });

            // ── DSORG selector ───────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Dataset Org (DSORG):");
                for dsorg in [Dsorg::Ps, Dsorg::Po, Dsorg::Pdse, Dsorg::Gdg] {
                    ui.selectable_value(&mut form.dsorg, dsorg, dsorg.label());
                }
            });

            // ── RECFM selector ───────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Record Format:     ");
                for recfm in [Recfm::Fb, Recfm::F, Recfm::Vb, Recfm::V, Recfm::U] {
                    ui.selectable_value(&mut form.recfm, recfm, recfm.label());
                }
            });

            // ── LRECL / BLKSIZE ──────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("LRECL:             ");
                ui.add(
                    egui::TextEdit::singleline(&mut form.lrecl)
                        .desired_width(60.0)
                        .font(egui::TextStyle::Monospace),
                );
                ui.label("  Block Size:");
                ui.add(
                    egui::TextEdit::singleline(&mut form.blksize)
                        .desired_width(80.0)
                        .font(egui::TextStyle::Monospace),
                );
            });

            // ── Conditional fields ───────────────────────────────────────
            match form.dsorg {
                Dsorg::Po | Dsorg::Pdse => {
                    ui.horizontal(|ui| {
                        ui.label("Directory Blocks:  ");
                        ui.add(
                            egui::TextEdit::singleline(&mut form.dir_blocks)
                                .desired_width(60.0)
                                .font(egui::TextStyle::Monospace),
                        );
                    });
                }
                Dsorg::Gdg => {
                    ui.horizontal(|ui| {
                        ui.label("GDG Limit (1-255): ");
                        ui.add(
                            egui::TextEdit::singleline(&mut form.gdg_limit)
                                .desired_width(60.0)
                                .font(egui::TextStyle::Monospace),
                        );
                    });
                    ui.checkbox(&mut form.scratch, "Scratch on roll-off");
                }
                _ => {}
            }

            // ── Description ──────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.label("Description:       ");
                ui.text_edit_singleline(&mut form.description);
            });

            // ── Inline error — Req 5.5 ───────────────────────────────────
            if let Some(err) = &form.error {
                ui.colored_label(egui::Color32::RED, err);
            }

            ui.separator();

            // ── Buttons ──────────────────────────────────────────────────
            ui.horizontal(|ui| {
                if ui.button("Allocate").clicked() {
                    match validate(form) {
                        Ok(_) => {
                            form.error = None;
                            outcome = AllocOutcome::Confirmed;
                        }
                        Err(e) => {
                            form.error = Some(e);
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    outcome = AllocOutcome::Cancelled;
                }
            });
        });

    // Validates: accessibility Requirement 2.1, 2.3 -- Escape closes the dialog.
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        outcome = AllocOutcome::Cancelled;
    }

    outcome
}

// == Tests ====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_ps_form() -> AllocDatasetForm {
        AllocDatasetForm {
            dataset_name: "PAYROLL.INPUT".to_string(),
            dsorg: Dsorg::Ps,
            recfm: Recfm::Fb,
            lrecl: "80".to_string(),
            blksize: "0".to_string(),
            ..Default::default()
        }
    }

    fn valid_po_form() -> AllocDatasetForm {
        AllocDatasetForm {
            dataset_name: "PAYROLL.LIB".to_string(),
            dsorg: Dsorg::Po,
            recfm: Recfm::Fb,
            lrecl: "80".to_string(),
            blksize: "0".to_string(),
            dir_blocks: "10".to_string(),
            ..Default::default()
        }
    }

    fn valid_gdg_form() -> AllocDatasetForm {
        AllocDatasetForm {
            dataset_name: "PAYROLL.MONTHLY".to_string(),
            dsorg: Dsorg::Gdg,
            recfm: Recfm::Fb,
            lrecl: "80".to_string(),
            blksize: "0".to_string(),
            gdg_limit: "10".to_string(),
            scratch: true,
            ..Default::default()
        }
    }

    // ── Default form state ────────────────────────────────────────────────

    /// Validates: Requirement 5.2 — default DSORG is PS.
    #[test]
    fn default_form_dsorg_is_ps() {
        // Validates: Requirement 5.2
        let form = AllocDatasetForm::default();
        assert_eq!(form.dsorg, Dsorg::Ps);
    }

    /// Validates: Requirement 5.2 — default RECFM is FB.
    #[test]
    fn default_form_recfm_is_fb() {
        // Validates: Requirement 5.2
        let form = AllocDatasetForm::default();
        assert_eq!(form.recfm, Recfm::Fb);
    }

    /// Validates: Requirement 5.2 — default LRECL is 80.
    #[test]
    fn default_form_lrecl_is_80() {
        // Validates: Requirement 5.2
        let form = AllocDatasetForm::default();
        assert_eq!(form.lrecl, "80");
    }

    /// Validates: Requirement 5.2 — default BLKSIZE is 0 (system-determined).
    #[test]
    fn default_form_blksize_is_zero() {
        // Validates: Requirement 5.2
        let form = AllocDatasetForm::default();
        assert_eq!(form.blksize, "0");
    }

    /// Validates: Requirement 5.2 — default dir_blocks is 10.
    #[test]
    fn default_form_dir_blocks_is_10() {
        // Validates: Requirement 5.2
        let form = AllocDatasetForm::default();
        assert_eq!(form.dir_blocks, "10");
    }

    /// Validates: Requirement 5.2 — scratch defaults to true.
    #[test]
    fn default_form_scratch_is_true() {
        // Validates: Requirement 5.2
        let form = AllocDatasetForm::default();
        assert!(form.scratch);
    }

    // ── Validation — valid forms ──────────────────────────────────────────

    /// Validates: Requirement 5.3 — valid PS form passes validation.
    #[test]
    fn validate_accepts_valid_ps_form() {
        // Validates: Requirement 5.3
        let params = validate(&valid_ps_form()).unwrap();
        assert_eq!(params.dataset_name, "PAYROLL.INPUT");
        assert_eq!(params.dsorg, Dsorg::Ps);
        assert_eq!(params.lrecl, 80);
        assert_eq!(params.blksize, 0);
        assert!(params.dir_blocks.is_none());
        assert!(params.gdg_limit.is_none());
    }

    /// Validates: Requirement 5.3 — valid PO form passes validation.
    #[test]
    fn validate_accepts_valid_po_form() {
        // Validates: Requirement 5.3
        let params = validate(&valid_po_form()).unwrap();
        assert_eq!(params.dsorg, Dsorg::Po);
        assert_eq!(params.dir_blocks, Some(10));
        assert!(params.gdg_limit.is_none());
    }

    /// Validates: Requirement 5.3 — valid GDG form passes validation.
    #[test]
    fn validate_accepts_valid_gdg_form() {
        // Validates: Requirement 5.3
        let params = validate(&valid_gdg_form()).unwrap();
        assert_eq!(params.dsorg, Dsorg::Gdg);
        assert_eq!(params.gdg_limit, Some(10));
        assert!(params.scratch);
        assert!(params.dir_blocks.is_none());
    }

    // ── Validation — dataset name ─────────────────────────────────────────

    /// Validates: Requirement 5.3 — empty dataset name fails.
    #[test]
    fn validate_rejects_empty_dataset_name() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.dataset_name = String::new();
        assert!(validate(&form).is_err());
    }

    /// Validates: Requirement 5.3 — whitespace-only dataset name fails.
    #[test]
    fn validate_rejects_whitespace_dataset_name() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.dataset_name = "   ".to_string();
        assert!(validate(&form).is_err());
    }

    // ── Validation — LRECL ───────────────────────────────────────────────

    /// Validates: Requirement 5.3, dataset-catalog Req 7.10 — LRECL 0 fails.
    #[test]
    fn validate_rejects_lrecl_zero() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.lrecl = "0".to_string();
        assert!(validate(&form).is_err());
    }

    /// Validates: Requirement 5.3, dataset-catalog Req 7.10 — LRECL > 32760 fails.
    #[test]
    fn validate_rejects_lrecl_over_32760() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.lrecl = "32761".to_string();
        assert!(validate(&form).is_err());
    }

    /// Validates: Requirement 5.3 — LRECL 32760 is accepted.
    #[test]
    fn validate_accepts_lrecl_at_max() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.lrecl = "32760".to_string();
        form.blksize = "0".to_string();
        assert!(validate(&form).is_ok());
    }

    /// Validates: Requirement 5.3 — non-numeric LRECL fails.
    #[test]
    fn validate_rejects_non_numeric_lrecl() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.lrecl = "abc".to_string();
        assert!(validate(&form).is_err());
    }

    // ── Validation — BLKSIZE ─────────────────────────────────────────────

    /// Validates: Requirement 5.2 — BLKSIZE 0 is accepted (system-determined).
    #[test]
    fn validate_accepts_blksize_zero() {
        // Validates: Requirement 5.2
        let mut form = valid_ps_form();
        form.blksize = "0".to_string();
        assert!(validate(&form).is_ok());
    }

    /// Validates: Requirement 5.3, dataset-catalog Req 7.10 — BLKSIZE < LRECL fails.
    #[test]
    fn validate_rejects_blksize_less_than_lrecl() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.lrecl = "80".to_string();
        form.blksize = "79".to_string();
        assert!(validate(&form).is_err());
    }

    /// Validates: Requirement 5.3 — BLKSIZE == LRECL is accepted.
    #[test]
    fn validate_accepts_blksize_equal_to_lrecl() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.lrecl = "80".to_string();
        form.blksize = "80".to_string();
        assert!(validate(&form).is_ok());
    }

    /// Validates: Requirement 5.3 — non-numeric BLKSIZE fails.
    #[test]
    fn validate_rejects_non_numeric_blksize() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.blksize = "xyz".to_string();
        assert!(validate(&form).is_err());
    }

    // ── Validation — GDG limit ────────────────────────────────────────────

    /// Validates: Requirement 5.3, dataset-catalog Req 7.10 — GDG limit 0 fails.
    #[test]
    fn validate_rejects_gdg_limit_zero() {
        // Validates: Requirement 5.3
        let mut form = valid_gdg_form();
        form.gdg_limit = "0".to_string();
        assert!(validate(&form).is_err());
    }

    /// Validates: Requirement 5.3, dataset-catalog Req 7.10 — GDG limit > 255 fails.
    #[test]
    fn validate_rejects_gdg_limit_over_255() {
        // Validates: Requirement 5.3
        let mut form = valid_gdg_form();
        form.gdg_limit = "256".to_string();
        assert!(validate(&form).is_err());
    }

    /// Validates: Requirement 5.3 — GDG limit 255 is accepted.
    #[test]
    fn validate_accepts_gdg_limit_at_max() {
        // Validates: Requirement 5.3
        let mut form = valid_gdg_form();
        form.gdg_limit = "255".to_string();
        assert!(validate(&form).is_ok());
    }

    /// Validates: Requirement 5.3 — GDG limit 1 is accepted.
    #[test]
    fn validate_accepts_gdg_limit_at_min() {
        // Validates: Requirement 5.3
        let mut form = valid_gdg_form();
        form.gdg_limit = "1".to_string();
        assert!(validate(&form).is_ok());
    }

    // ── Conditional field visibility ──────────────────────────────────────

    /// Validates: Requirement 5.2 — dir_blocks not included for PS.
    #[test]
    fn validate_ps_does_not_include_dir_blocks() {
        // Validates: Requirement 5.2
        let params = validate(&valid_ps_form()).unwrap();
        assert!(params.dir_blocks.is_none());
    }

    /// Validates: Requirement 5.2 — gdg_limit not included for PO.
    #[test]
    fn validate_po_does_not_include_gdg_limit() {
        // Validates: Requirement 5.2
        let params = validate(&valid_po_form()).unwrap();
        assert!(params.gdg_limit.is_none());
    }

    /// Validates: Requirement 5.2 — dir_blocks not included for GDG.
    #[test]
    fn validate_gdg_does_not_include_dir_blocks() {
        // Validates: Requirement 5.2
        let params = validate(&valid_gdg_form()).unwrap();
        assert!(params.dir_blocks.is_none());
    }

    // ── Allocate Like ─────────────────────────────────────────────────────

    /// Validates: Requirement 5.6 — from_like pre-populates all fields except DSN.
    #[test]
    fn from_like_prepopulates_all_fields_except_dsn() {
        // Validates: Requirement 5.6
        let form =
            AllocDatasetForm::from_like(Dsorg::Po, Recfm::Vb, 256, 2048, Some(20), None, false);
        assert!(
            form.dataset_name.is_empty(),
            "DSN must be empty for user to fill"
        );
        assert_eq!(form.dsorg, Dsorg::Po);
        assert_eq!(form.recfm, Recfm::Vb);
        assert_eq!(form.lrecl, "256");
        assert_eq!(form.blksize, "2048");
        assert_eq!(form.dir_blocks, "20");
        assert!(!form.scratch);
        assert!(form.allocate_like);
    }

    /// Validates: Requirement 5.6 — from_like GDG sets gdg_limit.
    #[test]
    fn from_like_gdg_sets_gdg_limit() {
        // Validates: Requirement 5.6
        let form = AllocDatasetForm::from_like(Dsorg::Gdg, Recfm::Fb, 80, 0, None, Some(5), true);
        assert_eq!(form.gdg_limit, "5");
        assert!(form.scratch);
    }

    // ── Optional description ──────────────────────────────────────────────

    /// Validates: Requirement 5.2 — empty description produces None.
    #[test]
    fn validate_empty_description_produces_none() {
        // Validates: Requirement 5.2
        let params = validate(&valid_ps_form()).unwrap();
        assert!(params.description.is_none());
    }

    /// Validates: Requirement 5.2 — non-empty description is preserved.
    #[test]
    fn validate_non_empty_description_is_preserved() {
        // Validates: Requirement 5.2
        let mut form = valid_ps_form();
        form.description = "Payroll input file".to_string();
        let params = validate(&form).unwrap();
        assert_eq!(params.description.as_deref(), Some("Payroll input file"));
    }

    // ── Req 5.8: uppercase ────────────────────────────────────────────────

    /// Validates: Requirement 5.8 — dataset name is uppercased by validate.
    #[test]
    fn validate_uppercases_dataset_name() {
        // Validates: Requirement 5.8
        let mut form = valid_ps_form();
        form.dataset_name = "payroll.input".to_string();
        let params = validate(&form).unwrap();
        assert_eq!(params.dataset_name, "PAYROLL.INPUT");
    }

    /// Validates: Requirement 5.8 — mixed-case name is uppercased.
    #[test]
    fn validate_uppercases_mixed_case_name() {
        // Validates: Requirement 5.8
        let mut form = valid_ps_form();
        form.dataset_name = "Payroll.Input".to_string();
        let params = validate(&form).unwrap();
        assert_eq!(params.dataset_name, "PAYROLL.INPUT");
    }

    // ── Req 5.7: HLQ pre-population ──────────────────────────────────────

    /// Validates: Requirement 5.7 — with_hlq pre-populates dataset_name with HLQ dot.
    #[test]
    fn with_hlq_prepopulates_dataset_name_with_hlq_dot() {
        // Validates: Requirement 5.7
        let form = AllocDatasetForm::with_hlq("PAYROLL");
        assert_eq!(form.dataset_name, "PAYROLL.");
    }

    /// Validates: Requirement 5.7 — with_hlq empty string gives just a dot.
    #[test]
    fn with_hlq_empty_string_gives_dot() {
        // Validates: Requirement 5.7
        let form = AllocDatasetForm::with_hlq("");
        assert_eq!(form.dataset_name, ".");
    }

    // ── Req 5.9: duplicate detection ─────────────────────────────────────

    /// Validates: Requirement 5.9 — validate_for_catalog rejects duplicate DSN.
    #[test]
    fn validate_for_catalog_rejects_duplicate_dsn() {
        // Validates: Requirement 5.9
        let form = valid_ps_form(); // dataset_name = "PAYROLL.INPUT"
        let existing = vec!["PAYROLL.INPUT".to_string()];
        assert!(validate_for_catalog(&form, &existing).is_err());
    }

    /// Validates: Requirement 5.9 — duplicate check is case-insensitive.
    #[test]
    fn validate_for_catalog_duplicate_check_is_case_insensitive() {
        // Validates: Requirement 5.9
        let mut form = valid_ps_form();
        form.dataset_name = "payroll.input".to_string();
        let existing = vec!["PAYROLL.INPUT".to_string()];
        assert!(validate_for_catalog(&form, &existing).is_err());
    }

    /// Validates: Requirement 5.9 — unique DSN passes catalog validation.
    #[test]
    fn validate_for_catalog_accepts_unique_dsn() {
        // Validates: Requirement 5.9
        let form = valid_ps_form(); // PAYROLL.INPUT
        let existing = vec!["PAYROLL.OTHER".to_string()];
        assert!(validate_for_catalog(&form, &existing).is_ok());
    }

    /// Validates: Requirement 5.9 — empty existing list always passes.
    #[test]
    fn validate_for_catalog_empty_existing_always_passes() {
        // Validates: Requirement 5.9
        let form = valid_ps_form();
        assert!(validate_for_catalog(&form, &[]).is_ok());
    }
}
