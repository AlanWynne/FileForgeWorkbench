//! Dataset types, record formats, allocation parameters.
//!
//! Defines the core data structures representing dataset metadata and
//! allocation parameters for the catalog database.

use std::fmt;
use std::str::FromStr;

use crate::dsn::Dsn;
use crate::error::CatalogError;
use crate::hierarchy::CatalogScope;

/// Dataset organization type.
///
/// Represents the three supported dataset organizations in the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Dsorg {
    /// Sequential — single flat file.
    PS,
    /// Partitioned — library of members (PDS or PDSE).
    PO,
    /// Generation Data Group — versioned dataset collection.
    GDG,
}

impl fmt::Display for Dsorg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PS => write!(f, "PS"),
            Self::PO => write!(f, "PO"),
            Self::GDG => write!(f, "GDG"),
        }
    }
}

impl FromStr for Dsorg {
    type Err = CatalogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PS" => Ok(Self::PS),
            "PO" => Ok(Self::PO),
            "GDG" => Ok(Self::GDG),
            _ => Err(CatalogError::InvalidAllocationParams {
                reason: format!("invalid DSORG '{}': must be PS, PO, or GDG", s),
                operation: "parse_dsorg".to_string(),
            }),
        }
    }
}

/// Record format for a dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Recfm {
    /// Fixed length records.
    F,
    /// Fixed blocked records.
    FB,
    /// Variable length records.
    V,
    /// Variable blocked records.
    VB,
    /// Undefined format.
    U,
}

impl fmt::Display for Recfm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::F => write!(f, "F"),
            Self::FB => write!(f, "FB"),
            Self::V => write!(f, "V"),
            Self::VB => write!(f, "VB"),
            Self::U => write!(f, "U"),
        }
    }
}

impl FromStr for Recfm {
    type Err = CatalogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "F" => Ok(Self::F),
            "FB" => Ok(Self::FB),
            "V" => Ok(Self::V),
            "VB" => Ok(Self::VB),
            "U" => Ok(Self::U),
            _ => Err(CatalogError::InvalidAllocationParams {
                reason: format!("invalid RECFM '{}': must be F, FB, V, VB, or U", s),
                operation: "parse_recfm".to_string(),
            }),
        }
    }
}

/// Distinguishes PDS from PDSE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PartitionedSubtype {
    /// Standard Partitioned Dataset.
    #[default]
    PDS,
    /// Partitioned Dataset Extended.
    PDSE,
}

impl fmt::Display for PartitionedSubtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PDS => write!(f, "PDS"),
            Self::PDSE => write!(f, "PDSE"),
        }
    }
}

impl FromStr for PartitionedSubtype {
    type Err = CatalogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "PDS" => Ok(Self::PDS),
            "PDSE" => Ok(Self::PDSE),
            _ => Err(CatalogError::InvalidAllocationParams {
                reason: format!("invalid subtype '{}': must be PDS or PDSE", s),
                operation: "parse_subtype".to_string(),
            }),
        }
    }
}

/// Allocation parameters for creating a new dataset.
#[derive(Debug, Clone)]
pub struct AllocParams {
    /// Dataset name to allocate.
    pub dsn: Dsn,
    /// Organization type.
    pub dsorg: Dsorg,
    /// Record format (optional -- defaults applied per dataset type).
    pub recfm: Option<Recfm>,
    /// Logical record length (optional -- defaults applied).
    pub lrecl: Option<u32>,
    /// Block size (optional -- defaults applied).
    pub blksize: Option<u32>,
    /// Directory blocks (PDS only).
    pub dir_blocks: Option<u32>,
    /// GDG limit (GDG only, 1-255).
    pub gdg_limit: Option<u8>,
    /// GDG scratch policy (GDG only).
    pub gdg_scratch: Option<bool>,
    /// PDS/PDSE subtype.
    pub subtype: Option<PartitionedSubtype>,
    /// Description.
    pub description: Option<String>,
    /// Catalogue scope (defaults to User).
    pub scope: CatalogScope,
}

impl AllocParams {
    /// Validate allocation parameters against constraints.
    ///
    /// - LRECL must be > 0 and ≤ 32760
    /// - BLKSIZE must be ≥ LRECL
    /// - GDG limit must be between 1 and 255
    ///
    /// # Errors
    ///
    /// Returns `CatalogError::InvalidAllocationParams` if any constraint is violated.
    pub fn validate(&self) -> Result<(), CatalogError> {
        if let Some(lrecl) = self.lrecl {
            if lrecl == 0 || lrecl > 32760 {
                return Err(CatalogError::InvalidAllocationParams {
                    reason: format!("LRECL must be between 1 and 32760, got {lrecl}"),
                    operation: "validate_alloc_params".to_string(),
                });
            }
        }

        if let Some(blksize) = self.blksize {
            if let Some(lrecl) = self.lrecl {
                if blksize < lrecl {
                    return Err(CatalogError::InvalidAllocationParams {
                        reason: format!("BLKSIZE ({blksize}) must be >= LRECL ({lrecl})"),
                        operation: "validate_alloc_params".to_string(),
                    });
                }
            }
        }

        if let Some(limit) = self.gdg_limit {
            if limit == 0 {
                return Err(CatalogError::InvalidAllocationParams {
                    reason: "GDG limit must be between 1 and 255".to_string(),
                    operation: "validate_alloc_params".to_string(),
                });
            }
        }

        if self.dsorg == Dsorg::GDG && self.gdg_limit.is_none() {
            return Err(CatalogError::InvalidAllocationParams {
                reason: "GDG datasets require a generation limit".to_string(),
                operation: "validate_alloc_params".to_string(),
            });
        }

        Ok(())
    }

    /// Apply default values for omitted parameters based on dataset organization.
    ///
    /// PS/PO defaults: RECFM=FB, LRECL=80, BLKSIZE=27920.
    pub fn with_defaults(mut self) -> Self {
        match self.dsorg {
            Dsorg::PS | Dsorg::PO => {
                if self.recfm.is_none() {
                    self.recfm = Some(Recfm::FB);
                }
                if self.lrecl.is_none() {
                    self.lrecl = Some(80);
                }
                if self.blksize.is_none() {
                    self.blksize = Some(27920);
                }
            }
            Dsorg::GDG => {
                // GDG base doesn't need record attributes
            }
        }
        self
    }
}

/// A dataset entry as stored in the catalog database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetRecord {
    /// Unique identifier (database row ID).
    pub id: i64,
    /// The dataset name.
    pub dsn: Dsn,
    /// Dataset organization.
    pub dsorg: Dsorg,
    /// Relative path from repository root to physical content.
    pub storage_path: String,
    /// Record format.
    pub recfm: Option<Recfm>,
    /// Logical record length.
    pub lrecl: Option<u32>,
    /// Block size.
    pub blksize: Option<u32>,
    /// PDS/PDSE subtype (only for PO datasets).
    pub subtype: Option<PartitionedSubtype>,
    /// Catalogue scope.
    pub scope: CatalogScope,
    /// Creation timestamp (ISO 8601).
    pub created: Option<String>,
    /// Last modification timestamp (ISO 8601).
    pub modified: Option<String>,
    /// Last access timestamp (ISO 8601).
    pub accessed: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsorg_display_and_parse() {
        // Validates: Requirement 3 AC 1
        assert_eq!(Dsorg::PS.to_string(), "PS");
        assert_eq!(Dsorg::PO.to_string(), "PO");
        assert_eq!(Dsorg::GDG.to_string(), "GDG");

        assert_eq!("ps".parse::<Dsorg>().unwrap(), Dsorg::PS);
        assert_eq!("PO".parse::<Dsorg>().unwrap(), Dsorg::PO);
        assert_eq!("gdg".parse::<Dsorg>().unwrap(), Dsorg::GDG);
        assert!("INVALID".parse::<Dsorg>().is_err());
    }

    #[test]
    fn recfm_display_and_parse() {
        // Validates: Requirement 7 AC 1
        assert_eq!(Recfm::FB.to_string(), "FB");
        assert_eq!("fb".parse::<Recfm>().unwrap(), Recfm::FB);
        assert_eq!("V".parse::<Recfm>().unwrap(), Recfm::V);
        assert!("X".parse::<Recfm>().is_err());
    }

    #[test]
    fn alloc_params_validate_lrecl_range() {
        // Validates: Requirement 7 AC 10
        let params = AllocParams {
            dsn: Dsn::parse("TEST.DSN").unwrap(),
            dsorg: Dsorg::PS,
            recfm: Some(Recfm::FB),
            lrecl: Some(0),
            blksize: Some(80),
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope: CatalogScope::User,
        };
        assert!(params.validate().is_err());

        let params = AllocParams {
            dsn: Dsn::parse("TEST.DSN").unwrap(),
            dsorg: Dsorg::PS,
            recfm: Some(Recfm::FB),
            lrecl: Some(32761),
            blksize: Some(32761),
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope: CatalogScope::User,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn alloc_params_validate_blksize_ge_lrecl() {
        // Validates: Requirement 7 AC 10
        let params = AllocParams {
            dsn: Dsn::parse("TEST.DSN").unwrap(),
            dsorg: Dsorg::PS,
            recfm: Some(Recfm::FB),
            lrecl: Some(80),
            blksize: Some(40), // less than LRECL
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope: CatalogScope::User,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn alloc_params_validate_gdg_requires_limit() {
        // Validates: Requirement 7 AC 10
        let params = AllocParams {
            dsn: Dsn::parse("TEST.GDG").unwrap(),
            dsorg: Dsorg::GDG,
            recfm: None,
            lrecl: None,
            blksize: None,
            dir_blocks: None,
            gdg_limit: None, // Missing limit
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope: CatalogScope::User,
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn alloc_params_valid_succeeds() {
        // Validates: Requirement 7 AC 1, AC 2
        let params = AllocParams {
            dsn: Dsn::parse("TEST.DSN").unwrap(),
            dsorg: Dsorg::PS,
            recfm: Some(Recfm::FB),
            lrecl: Some(80),
            blksize: Some(27920),
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope: CatalogScope::User,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn alloc_params_with_defaults_ps() {
        // Validates: Requirement 15 AC 1
        let params = AllocParams {
            dsn: Dsn::parse("TEST.DSN").unwrap(),
            dsorg: Dsorg::PS,
            recfm: None,
            lrecl: None,
            blksize: None,
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope: CatalogScope::User,
        };
        let with_defaults = params.with_defaults();
        assert_eq!(with_defaults.recfm, Some(Recfm::FB));
        assert_eq!(with_defaults.lrecl, Some(80));
        assert_eq!(with_defaults.blksize, Some(27920));
    }

    #[test]
    fn alloc_params_explicit_overrides_defaults() {
        // Validates: Requirement 15 AC 5
        let params = AllocParams {
            dsn: Dsn::parse("TEST.DSN").unwrap(),
            dsorg: Dsorg::PS,
            recfm: Some(Recfm::V),
            lrecl: Some(255),
            blksize: Some(27998),
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope: CatalogScope::User,
        };
        let with_defaults = params.with_defaults();
        assert_eq!(with_defaults.recfm, Some(Recfm::V));
        assert_eq!(with_defaults.lrecl, Some(255));
        assert_eq!(with_defaults.blksize, Some(27998));
    }
}
