//! DD statement operand models: DISP, DCB, SPACE.
//!
//! These types represent the parsed sub-parameters of DD statement operands.

use serde::Deserialize;

/// Dataset status at step initiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DispStatus {
    /// New dataset — must not already exist.
    New,
    /// Old dataset — must already exist, exclusive access.
    Old,
    /// Shared dataset — must already exist, shared access.
    Shr,
    /// Modify dataset — append access; create if not exists and SPACE given.
    Mod,
}

impl DispStatus {
    /// Parse a DISP status keyword.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "NEW" => Some(Self::New),
            "OLD" => Some(Self::Old),
            "SHR" => Some(Self::Shr),
            "MOD" => Some(Self::Mod),
            _ => None,
        }
    }
}

impl std::fmt::Display for DispStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::New => write!(f, "NEW"),
            Self::Old => write!(f, "OLD"),
            Self::Shr => write!(f, "SHR"),
            Self::Mod => write!(f, "MOD"),
        }
    }
}

/// Conditional disposition action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DispAction {
    /// Keep the dataset.
    Keep,
    /// Delete the dataset.
    Delete,
    /// Catalog the dataset.
    Catlg,
    /// Uncatalog the dataset.
    Uncatlg,
    /// Pass to a subsequent step.
    Pass,
}

impl DispAction {
    /// Parse a DISP action keyword.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "KEEP" => Some(Self::Keep),
            "DELETE" => Some(Self::Delete),
            "CATLG" => Some(Self::Catlg),
            "UNCATLG" => Some(Self::Uncatlg),
            "PASS" => Some(Self::Pass),
            _ => None,
        }
    }
}

impl std::fmt::Display for DispAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keep => write!(f, "KEEP"),
            Self::Delete => write!(f, "DELETE"),
            Self::Catlg => write!(f, "CATLG"),
            Self::Uncatlg => write!(f, "UNCATLG"),
            Self::Pass => write!(f, "PASS"),
        }
    }
}

/// Parsed DISP operand with up to three positional sub-parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispParameter {
    /// Status at step start: NEW, OLD, SHR, MOD.
    pub status: DispStatus,
    /// Normal-end disposition: KEEP, DELETE, CATLG, UNCATLG, PASS.
    pub normal_disp: Option<DispAction>,
    /// Abnormal-end disposition: KEEP, DELETE, CATLG, UNCATLG.
    pub abnormal_disp: Option<DispAction>,
}

impl DispParameter {
    /// Returns the default DISP when no DISP operand is specified: (NEW, DELETE).
    pub fn default_disp() -> Self {
        Self {
            status: DispStatus::New,
            normal_disp: Some(DispAction::Delete),
            abnormal_disp: None,
        }
    }

    /// Returns true if this disposition requires the dataset to already exist.
    pub fn requires_existing(&self) -> bool {
        matches!(self.status, DispStatus::Old | DispStatus::Shr)
    }

    /// Returns true if this disposition creates a new dataset.
    pub fn creates_new(&self) -> bool {
        matches!(self.status, DispStatus::New)
    }

    /// Parse a DISP operand string like "(NEW,CATLG,DELETE)" or "SHR".
    pub fn parse(text: &str) -> Option<Self> {
        let trimmed = text.trim().to_uppercase();

        // Simple form: just a status keyword
        if !trimmed.contains('(') {
            let status = DispStatus::parse(&trimmed)?;
            return Some(Self {
                status,
                normal_disp: None,
                abnormal_disp: None,
            });
        }

        // Parenthesised form: (status,normal,abnormal)
        let inner = trimmed
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))?;

        let parts: Vec<&str> = inner.split(',').collect();
        if parts.is_empty() {
            return None;
        }

        let status = DispStatus::parse(parts[0])?;
        let normal_disp = parts.get(1).and_then(|s| {
            let s = s.trim();
            if s.is_empty() {
                None
            } else {
                DispAction::parse(s)
            }
        });
        let abnormal_disp = parts.get(2).and_then(|s| {
            let s = s.trim();
            if s.is_empty() {
                None
            } else {
                DispAction::parse(s)
            }
        });

        Some(Self {
            status,
            normal_disp,
            abnormal_disp,
        })
    }
}

/// Dataset organisation types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum DsOrg {
    /// Physical sequential.
    Ps,
    /// Partitioned (PDS/PDSE).
    Po,
    /// Direct access.
    Da,
    /// VSAM (informational — not fully emulated).
    Vsam,
}

impl DsOrg {
    /// Parse a DSORG keyword.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "PS" => Some(Self::Ps),
            "PO" => Some(Self::Po),
            "DA" => Some(Self::Da),
            "VSAM" => Some(Self::Vsam),
            _ => None,
        }
    }
}

/// Dataset Control Block attributes extracted from the DCB operand.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DcbAttributes {
    /// Record format: F, FB, V, VB, U, etc.
    pub recfm: Option<String>,
    /// Logical record length.
    pub lrecl: Option<u32>,
    /// Block size.
    pub blksize: Option<u32>,
    /// Dataset organisation.
    pub dsorg: Option<DsOrg>,
}

impl DcbAttributes {
    /// Returns hardcoded defaults: RECFM=FB, LRECL=80, BLKSIZE=27920.
    ///
    /// Used when neither DCB nor catalog.defaults are specified.
    pub fn hardcoded_defaults() -> Self {
        Self {
            recfm: Some("FB".to_string()),
            lrecl: Some(80),
            blksize: Some(27920),
            dsorg: None,
        }
    }

    /// Parse a DCB operand from key=value pairs.
    ///
    /// Input: the content inside DCB parentheses, e.g., "RECFM=FB,LRECL=80,BLKSIZE=27920".
    pub fn parse(text: &str) -> Self {
        let mut attrs = Self::default();
        for part in text.split(',') {
            let part = part.trim().to_uppercase();
            if let Some((key, value)) = part.split_once('=') {
                match key.trim() {
                    "RECFM" => attrs.recfm = Some(value.trim().to_string()),
                    "LRECL" => attrs.lrecl = value.trim().parse().ok(),
                    "BLKSIZE" => attrs.blksize = value.trim().parse().ok(),
                    "DSORG" => attrs.dsorg = DsOrg::parse(value.trim()),
                    _ => {} // ignore unknown DCB sub-params
                }
            }
        }
        attrs
    }
}

/// Space allocation unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceUnit {
    /// Tracks.
    Trk,
    /// Cylinders.
    Cyl,
    /// Average block size (integer).
    Blksize(u32),
}

/// Parsed SPACE operand specifying allocation size for new datasets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceAllocation {
    /// Allocation unit.
    pub unit: SpaceUnit,
    /// Primary quantity.
    pub primary: u32,
    /// Secondary quantity (for extension).
    pub secondary: Option<u32>,
    /// Directory blocks (for PDS).
    pub directory: Option<u32>,
}

impl SpaceAllocation {
    /// Parse a SPACE operand string.
    ///
    /// Formats supported:
    /// - `(TRK,(primary,secondary,directory))`
    /// - `(CYL,(primary,secondary))`
    /// - `(blocksize,(primary,secondary,directory))`
    pub fn parse(text: &str) -> Option<Self> {
        let trimmed = text.trim().to_uppercase();
        let inner = trimmed.strip_prefix('(')?.strip_suffix(')')?;

        // Split on first comma that is not inside nested parens
        let (unit_str, quantities_str) = split_first_comma_outside_parens(inner)?;

        let unit = match unit_str.trim() {
            "TRK" => SpaceUnit::Trk,
            "CYL" => SpaceUnit::Cyl,
            s => {
                let blk: u32 = s.parse().ok()?;
                SpaceUnit::Blksize(blk)
            }
        };

        // Parse quantities: (primary,secondary,directory)
        let qty_str = quantities_str.trim();
        let qty_inner = qty_str.strip_prefix('(')?.strip_suffix(')')?;
        let qty_parts: Vec<&str> = qty_inner.split(',').collect();

        let primary: u32 = qty_parts.first()?.trim().parse().ok()?;
        let secondary = qty_parts.get(1).and_then(|s| s.trim().parse().ok());
        let directory = qty_parts.get(2).and_then(|s| s.trim().parse().ok());

        Some(Self {
            unit,
            primary,
            secondary,
            directory,
        })
    }
}

/// Split a string on the first comma that is not inside parentheses.
fn split_first_comma_outside_parens(s: &str) -> Option<(&str, &str)> {
    let mut depth = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => return Some((&s[..i], &s[i + 1..])),
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disp_parse_simple_status() {
        // Validates: Requirement 1 AC 4
        let disp = DispParameter::parse("SHR").unwrap();
        assert_eq!(disp.status, DispStatus::Shr);
        assert_eq!(disp.normal_disp, None);
        assert_eq!(disp.abnormal_disp, None);
    }

    #[test]
    fn disp_parse_full_parenthesised() {
        // Validates: Requirement 1 AC 4
        let disp = DispParameter::parse("(NEW,CATLG,DELETE)").unwrap();
        assert_eq!(disp.status, DispStatus::New);
        assert_eq!(disp.normal_disp, Some(DispAction::Catlg));
        assert_eq!(disp.abnormal_disp, Some(DispAction::Delete));
    }

    #[test]
    fn disp_parse_partial_parenthesised() {
        // Validates: Requirement 1 AC 4
        let disp = DispParameter::parse("(OLD,KEEP)").unwrap();
        assert_eq!(disp.status, DispStatus::Old);
        assert_eq!(disp.normal_disp, Some(DispAction::Keep));
        assert_eq!(disp.abnormal_disp, None);
    }

    #[test]
    fn disp_default_is_new_delete() {
        // Validates: Requirement 4 AC 7
        let disp = DispParameter::default_disp();
        assert_eq!(disp.status, DispStatus::New);
        assert_eq!(disp.normal_disp, Some(DispAction::Delete));
    }

    #[test]
    fn disp_requires_existing_for_old_and_shr() {
        let old = DispParameter {
            status: DispStatus::Old,
            normal_disp: None,
            abnormal_disp: None,
        };
        assert!(old.requires_existing());

        let shr = DispParameter {
            status: DispStatus::Shr,
            normal_disp: None,
            abnormal_disp: None,
        };
        assert!(shr.requires_existing());

        let new = DispParameter {
            status: DispStatus::New,
            normal_disp: None,
            abnormal_disp: None,
        };
        assert!(!new.requires_existing());
    }

    #[test]
    fn dcb_parse_all_fields() {
        // Validates: Requirement 1 AC 5
        let dcb = DcbAttributes::parse("RECFM=FB,LRECL=80,BLKSIZE=27920,DSORG=PS");
        assert_eq!(dcb.recfm.as_deref(), Some("FB"));
        assert_eq!(dcb.lrecl, Some(80));
        assert_eq!(dcb.blksize, Some(27920));
        assert_eq!(dcb.dsorg, Some(DsOrg::Ps));
    }

    #[test]
    fn dcb_hardcoded_defaults() {
        // Validates: Requirement 4 AC 2
        let defaults = DcbAttributes::hardcoded_defaults();
        assert_eq!(defaults.recfm.as_deref(), Some("FB"));
        assert_eq!(defaults.lrecl, Some(80));
        assert_eq!(defaults.blksize, Some(27920));
    }

    #[test]
    fn space_parse_tracks() {
        // Validates: Requirement 1 AC 6
        let space = SpaceAllocation::parse("(TRK,(100,50,5))").unwrap();
        assert_eq!(space.unit, SpaceUnit::Trk);
        assert_eq!(space.primary, 100);
        assert_eq!(space.secondary, Some(50));
        assert_eq!(space.directory, Some(5));
    }

    #[test]
    fn space_parse_cylinders_no_directory() {
        // Validates: Requirement 1 AC 6
        let space = SpaceAllocation::parse("(CYL,(10,5))").unwrap();
        assert_eq!(space.unit, SpaceUnit::Cyl);
        assert_eq!(space.primary, 10);
        assert_eq!(space.secondary, Some(5));
        assert_eq!(space.directory, None);
    }

    #[test]
    fn space_parse_blocksize_unit() {
        // Validates: Requirement 1 AC 6
        let space = SpaceAllocation::parse("(6160,(100,20))").unwrap();
        assert_eq!(space.unit, SpaceUnit::Blksize(6160));
        assert_eq!(space.primary, 100);
        assert_eq!(space.secondary, Some(20));
    }
}
