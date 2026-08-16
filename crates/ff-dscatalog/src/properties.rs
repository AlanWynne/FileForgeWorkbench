//! Dataset properties panel data provider.
//!
//! Produces structured property sets for display in the properties panel.

use std::path::PathBuf;

use crate::dataset::{Dsorg, PartitionedSubtype, Recfm};
use crate::dsn::Dsn;

/// Complete property set for the properties panel display.
#[derive(Debug, Clone)]
pub struct DatasetProperties {
    /// The dataset name.
    pub dsn: Dsn,
    /// Organization type.
    pub dsorg: Dsorg,
    /// Record format.
    pub recfm: Option<Recfm>,
    /// Logical record length.
    pub lrecl: Option<u32>,
    /// Block size.
    pub blksize: Option<u32>,
    /// PDS/PDSE subtype.
    pub subtype: Option<PartitionedSubtype>,
    /// Creation date.
    pub created: Option<String>,
    /// Last modified date.
    pub modified: Option<String>,
    /// Last access date.
    pub accessed: Option<String>,
    /// Physical file size in bytes.
    pub physical_size: Option<u64>,
    /// Physical path on disk.
    pub physical_path: Option<PathBuf>,
    /// Name of the containing catalog.
    pub catalog_name: String,
    /// Member count (PDS only).
    pub member_count: Option<usize>,
    /// GDG limit.
    pub gdg_limit: Option<u8>,
    /// GDG scratch policy.
    pub gdg_scratch: Option<bool>,
    /// GDG active generations count.
    pub gdg_active_generations: Option<usize>,
}
