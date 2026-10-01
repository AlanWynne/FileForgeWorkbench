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
//! Extracted from `ff-desktop/src/dataset_alloc_dialog.rs` into this standalone
//! crate (decomposition Wave 7, Task 23). The non-test body is split by concern
//! into `form` (data types), `validate` (validation logic), and `render` (egui
//! dialog) to respect the 400-line-per-file limit; this `lib.rs` re-exports the
//! flat public surface so `ff_dataset_alloc_dialog::*` matches the old module.
//!
//! Validates: Requirement 5.1-5.6

// from_like and allocate_like are wired in Task 8 context menus; suppress until then.
#![allow(dead_code)]

mod form;
mod render;
mod validate;

pub use form::{AllocDatasetForm, AllocOutcome, Dsorg, Recfm};
pub use render::render;
pub use validate::{validate, validate_for_catalog, AllocParams};

// === Tests ==================================================================

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

    // === Default form state =================================================

    /// Validates: Requirement 5.2 -- default DSORG is PS.
    #[test]
    fn default_form_dsorg_is_ps() {
        // Validates: Requirement 5.2
        let form = AllocDatasetForm::default();
        assert_eq!(form.dsorg, Dsorg::Ps);
    }

    /// Validates: Requirement 5.2 -- default RECFM is FB.
    #[test]
    fn default_form_recfm_is_fb() {
        // Validates: Requirement 5.2
        let form = AllocDatasetForm::default();
        assert_eq!(form.recfm, Recfm::Fb);
    }

    /// Validates: Requirement 5.2 -- default LRECL is 80.
    #[test]
    fn default_form_lrecl_is_80() {
        // Validates: Requirement 5.2
        let form = AllocDatasetForm::default();
        assert_eq!(form.lrecl, "80");
    }

    /// Validates: Requirement 5.2 -- default BLKSIZE is 0 (system-determined).
    #[test]
    fn default_form_blksize_is_zero() {
        // Validates: Requirement 5.2
        let form = AllocDatasetForm::default();
        assert_eq!(form.blksize, "0");
    }

    /// Validates: Requirement 5.2 -- default dir_blocks is 10.
    #[test]
    fn default_form_dir_blocks_is_10() {
        // Validates: Requirement 5.2
        let form = AllocDatasetForm::default();
        assert_eq!(form.dir_blocks, "10");
    }

    /// Validates: Requirement 5.2 -- scratch defaults to true.
    #[test]
    fn default_form_scratch_is_true() {
        // Validates: Requirement 5.2
        let form = AllocDatasetForm::default();
        assert!(form.scratch);
    }

    // === Validation -- valid forms ==========================================

    /// Validates: Requirement 5.3 -- valid PS form passes validation.
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

    /// Validates: Requirement 5.3 -- valid PO form passes validation.
    #[test]
    fn validate_accepts_valid_po_form() {
        // Validates: Requirement 5.3
        let params = validate(&valid_po_form()).unwrap();
        assert_eq!(params.dsorg, Dsorg::Po);
        assert_eq!(params.dir_blocks, Some(10));
        assert!(params.gdg_limit.is_none());
    }

    /// Validates: Requirement 5.3 -- valid GDG form passes validation.
    #[test]
    fn validate_accepts_valid_gdg_form() {
        // Validates: Requirement 5.3
        let params = validate(&valid_gdg_form()).unwrap();
        assert_eq!(params.dsorg, Dsorg::Gdg);
        assert_eq!(params.gdg_limit, Some(10));
        assert!(params.scratch);
        assert!(params.dir_blocks.is_none());
    }

    // === Validation -- dataset name =========================================

    /// Validates: Requirement 5.3 -- empty dataset name fails.
    #[test]
    fn validate_rejects_empty_dataset_name() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.dataset_name = String::new();
        assert!(validate(&form).is_err());
    }

    /// Validates: Requirement 5.3 -- whitespace-only dataset name fails.
    #[test]
    fn validate_rejects_whitespace_dataset_name() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.dataset_name = "   ".to_string();
        assert!(validate(&form).is_err());
    }

    // === Validation -- LRECL ================================================

    /// Validates: Requirement 5.3, dataset-catalog Req 7.10 -- LRECL 0 fails.
    #[test]
    fn validate_rejects_lrecl_zero() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.lrecl = "0".to_string();
        assert!(validate(&form).is_err());
    }

    /// Validates: Requirement 5.3, dataset-catalog Req 7.10 -- LRECL > 32760 fails.
    #[test]
    fn validate_rejects_lrecl_over_32760() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.lrecl = "32761".to_string();
        assert!(validate(&form).is_err());
    }

    /// Validates: Requirement 5.3 -- LRECL 32760 is accepted.
    #[test]
    fn validate_accepts_lrecl_at_max() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.lrecl = "32760".to_string();
        form.blksize = "0".to_string();
        assert!(validate(&form).is_ok());
    }

    /// Validates: Requirement 5.3 -- non-numeric LRECL fails.
    #[test]
    fn validate_rejects_non_numeric_lrecl() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.lrecl = "abc".to_string();
        assert!(validate(&form).is_err());
    }

    // === Validation -- BLKSIZE ==============================================

    /// Validates: Requirement 5.2 -- BLKSIZE 0 is accepted (system-determined).
    #[test]
    fn validate_accepts_blksize_zero() {
        // Validates: Requirement 5.2
        let mut form = valid_ps_form();
        form.blksize = "0".to_string();
        assert!(validate(&form).is_ok());
    }

    /// Validates: Requirement 5.3, dataset-catalog Req 7.10 -- BLKSIZE < LRECL fails.
    #[test]
    fn validate_rejects_blksize_less_than_lrecl() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.lrecl = "80".to_string();
        form.blksize = "79".to_string();
        assert!(validate(&form).is_err());
    }

    /// Validates: Requirement 5.3 -- BLKSIZE == LRECL is accepted.
    #[test]
    fn validate_accepts_blksize_equal_to_lrecl() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.lrecl = "80".to_string();
        form.blksize = "80".to_string();
        assert!(validate(&form).is_ok());
    }

    /// Validates: Requirement 5.3 -- non-numeric BLKSIZE fails.
    #[test]
    fn validate_rejects_non_numeric_blksize() {
        // Validates: Requirement 5.3
        let mut form = valid_ps_form();
        form.blksize = "xyz".to_string();
        assert!(validate(&form).is_err());
    }

    // === Validation -- GDG limit ============================================

    /// Validates: Requirement 5.3, dataset-catalog Req 7.10 -- GDG limit 0 fails.
    #[test]
    fn validate_rejects_gdg_limit_zero() {
        // Validates: Requirement 5.3
        let mut form = valid_gdg_form();
        form.gdg_limit = "0".to_string();
        assert!(validate(&form).is_err());
    }

    /// Validates: Requirement 5.3, dataset-catalog Req 7.10 -- GDG limit > 255 fails.
    #[test]
    fn validate_rejects_gdg_limit_over_255() {
        // Validates: Requirement 5.3
        let mut form = valid_gdg_form();
        form.gdg_limit = "256".to_string();
        assert!(validate(&form).is_err());
    }

    /// Validates: Requirement 5.3 -- GDG limit 255 is accepted.
    #[test]
    fn validate_accepts_gdg_limit_at_max() {
        // Validates: Requirement 5.3
        let mut form = valid_gdg_form();
        form.gdg_limit = "255".to_string();
        assert!(validate(&form).is_ok());
    }

    /// Validates: Requirement 5.3 -- GDG limit 1 is accepted.
    #[test]
    fn validate_accepts_gdg_limit_at_min() {
        // Validates: Requirement 5.3
        let mut form = valid_gdg_form();
        form.gdg_limit = "1".to_string();
        assert!(validate(&form).is_ok());
    }

    // === Conditional field visibility =======================================

    /// Validates: Requirement 5.2 -- dir_blocks not included for PS.
    #[test]
    fn validate_ps_does_not_include_dir_blocks() {
        // Validates: Requirement 5.2
        let params = validate(&valid_ps_form()).unwrap();
        assert!(params.dir_blocks.is_none());
    }

    /// Validates: Requirement 5.2 -- gdg_limit not included for PO.
    #[test]
    fn validate_po_does_not_include_gdg_limit() {
        // Validates: Requirement 5.2
        let params = validate(&valid_po_form()).unwrap();
        assert!(params.gdg_limit.is_none());
    }

    /// Validates: Requirement 5.2 -- dir_blocks not included for GDG.
    #[test]
    fn validate_gdg_does_not_include_dir_blocks() {
        // Validates: Requirement 5.2
        let params = validate(&valid_gdg_form()).unwrap();
        assert!(params.dir_blocks.is_none());
    }

    // === Allocate Like ======================================================

    /// Validates: Requirement 5.6 -- from_like pre-populates all fields except DSN.
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

    /// Validates: Requirement 5.6 -- from_like GDG sets gdg_limit.
    #[test]
    fn from_like_gdg_sets_gdg_limit() {
        // Validates: Requirement 5.6
        let form = AllocDatasetForm::from_like(Dsorg::Gdg, Recfm::Fb, 80, 0, None, Some(5), true);
        assert_eq!(form.gdg_limit, "5");
        assert!(form.scratch);
    }

    // === Optional description ===============================================

    /// Validates: Requirement 5.2 -- empty description produces None.
    #[test]
    fn validate_empty_description_produces_none() {
        // Validates: Requirement 5.2
        let params = validate(&valid_ps_form()).unwrap();
        assert!(params.description.is_none());
    }

    /// Validates: Requirement 5.2 -- non-empty description is preserved.
    #[test]
    fn validate_non_empty_description_is_preserved() {
        // Validates: Requirement 5.2
        let mut form = valid_ps_form();
        form.description = "Payroll input file".to_string();
        let params = validate(&form).unwrap();
        assert_eq!(params.description.as_deref(), Some("Payroll input file"));
    }

    // === Req 5.8: uppercase =================================================

    /// Validates: Requirement 5.8 -- dataset name is uppercased by validate.
    #[test]
    fn validate_uppercases_dataset_name() {
        // Validates: Requirement 5.8
        let mut form = valid_ps_form();
        form.dataset_name = "payroll.input".to_string();
        let params = validate(&form).unwrap();
        assert_eq!(params.dataset_name, "PAYROLL.INPUT");
    }

    /// Validates: Requirement 5.8 -- mixed-case name is uppercased.
    #[test]
    fn validate_uppercases_mixed_case_name() {
        // Validates: Requirement 5.8
        let mut form = valid_ps_form();
        form.dataset_name = "Payroll.Input".to_string();
        let params = validate(&form).unwrap();
        assert_eq!(params.dataset_name, "PAYROLL.INPUT");
    }

    // === Req 5.7: HLQ pre-population =========================================

    /// Validates: Requirement 5.7 -- with_hlq pre-populates dataset_name with HLQ dot.
    #[test]
    fn with_hlq_prepopulates_dataset_name_with_hlq_dot() {
        // Validates: Requirement 5.7
        let form = AllocDatasetForm::with_hlq("PAYROLL");
        assert_eq!(form.dataset_name, "PAYROLL.");
    }

    /// Validates: Requirement 5.7 -- with_hlq empty string gives just a dot.
    #[test]
    fn with_hlq_empty_string_gives_dot() {
        // Validates: Requirement 5.7
        let form = AllocDatasetForm::with_hlq("");
        assert_eq!(form.dataset_name, ".");
    }

    // === Req 5.9: duplicate detection =======================================

    /// Validates: Requirement 5.9 -- validate_for_catalog rejects duplicate DSN.
    #[test]
    fn validate_for_catalog_rejects_duplicate_dsn() {
        // Validates: Requirement 5.9
        let form = valid_ps_form(); // dataset_name = "PAYROLL.INPUT"
        let existing = vec!["PAYROLL.INPUT".to_string()];
        assert!(validate_for_catalog(&form, &existing).is_err());
    }

    /// Validates: Requirement 5.9 -- duplicate check is case-insensitive.
    #[test]
    fn validate_for_catalog_duplicate_check_is_case_insensitive() {
        // Validates: Requirement 5.9
        let mut form = valid_ps_form();
        form.dataset_name = "payroll.input".to_string();
        let existing = vec!["PAYROLL.INPUT".to_string()];
        assert!(validate_for_catalog(&form, &existing).is_err());
    }

    /// Validates: Requirement 5.9 -- unique DSN passes catalog validation.
    #[test]
    fn validate_for_catalog_accepts_unique_dsn() {
        // Validates: Requirement 5.9
        let form = valid_ps_form(); // PAYROLL.INPUT
        let existing = vec!["PAYROLL.OTHER".to_string()];
        assert!(validate_for_catalog(&form, &existing).is_ok());
    }

    /// Validates: Requirement 5.9 -- empty existing list always passes.
    #[test]
    fn validate_for_catalog_empty_existing_always_passes() {
        // Validates: Requirement 5.9
        let form = valid_ps_form();
        assert!(validate_for_catalog(&form, &[]).is_ok());
    }
}
