//! Dataset allocation form state and option enums.
//!
//! Pure data types for the Dataset Allocation dialog: the DSORG/RECFM option
//! enums and the `AllocDatasetForm` input state (plus its constructors and the
//! `AllocOutcome` result enum). No egui dependency -- this module is pure data.
//!
//! Validates: Requirement 5.2, 5.6, 5.7

// === Types ==================================================================

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
    pub(crate) fn label(self) -> &'static str {
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
    pub(crate) fn label(self) -> &'static str {
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
    /// Block size (default 0 -- system-determined).
    pub blksize: String,
    /// Directory blocks -- shown only for PO / PDSE (default 10).
    pub dir_blocks: String,
    /// GDG limit 1-255 -- shown only for GDG.
    pub gdg_limit: String,
    /// Scratch on roll-off -- shown only for GDG (default true).
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
