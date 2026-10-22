//! Dataset allocation validation.
//!
//! Converts an `AllocDatasetForm` into validated `AllocParams`, enforcing the
//! field rules (LRECL, BLKSIZE, directory blocks, GDG limit) and the
//! uppercase/duplicate-name requirements. Pure logic, no egui dependency.
//!
//! Validates: Requirement 5.3, 5.8, 5.9

use crate::form::{AllocDatasetForm, Dsorg, Recfm};

// === Validation =============================================================

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
        return Err(ff_i18n::t("dataset-alloc-err-name-required"));
    }

    // LRECL: integer, 1-32760
    let lrecl: u32 = form
        .lrecl
        .trim()
        .parse()
        .map_err(|_| ff_i18n::t("dataset-alloc-err-lrecl-integer"))?;
    if lrecl == 0 || lrecl > 32760 {
        return Err(ff_i18n::t_args(
            "dataset-alloc-err-lrecl-range",
            &ff_i18n::MessageArgs::new().set_number("lrecl", lrecl as f64),
        ));
    }

    // BLKSIZE: 0 = system-determined (accepted as-is); otherwise must be >= LRECL
    let blksize: u32 = form
        .blksize
        .trim()
        .parse()
        .map_err(|_| ff_i18n::t("dataset-alloc-err-blksize-integer"))?;
    if blksize != 0 && blksize < lrecl {
        return Err(ff_i18n::t_args(
            "dataset-alloc-err-blksize-range",
            &ff_i18n::MessageArgs::new()
                .set_number("blksize", blksize as f64)
                .set_number("lrecl", lrecl as f64),
        ));
    }

    // Directory Blocks -- only for PO / PDSE
    let dir_blocks = match form.dsorg {
        Dsorg::Po | Dsorg::Pdse => {
            let db: u32 = form
                .dir_blocks
                .trim()
                .parse()
                .map_err(|_| ff_i18n::t("dataset-alloc-err-dir-blocks-integer"))?;
            Some(db)
        }
        _ => None,
    };

    // GDG Limit -- only for GDG, must be 1-255
    let gdg_limit = match form.dsorg {
        Dsorg::Gdg => {
            let limit: u32 = form
                .gdg_limit
                .trim()
                .parse()
                .map_err(|_| ff_i18n::t("dataset-alloc-err-gdg-limit-integer"))?;
            if limit == 0 || limit > 255 {
                return Err(ff_i18n::t_args(
                    "dataset-alloc-err-gdg-limit-range",
                    &ff_i18n::MessageArgs::new().set_number("limit", limit as f64),
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

    // Req 5.8 -- Mainframe DSNs are always uppercase
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
        return Err(ff_i18n::t_args(
            "dataset-alloc-err-duplicate",
            &ff_i18n::MessageArgs::new().set("name", upper.clone()),
        ));
    }
    Ok(params)
}
