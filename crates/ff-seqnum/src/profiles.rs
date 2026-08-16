//! Built-in language profile constants for sequence number column definitions.
//!
//! These provide the default column ranges for COBOL, FORTRAN, JCL, and PL/I.

use crate::types::ColumnRange;

/// Built-in COBOL profile: front=1-6, back=73-80, auto_unnum=true.
pub struct CobolProfile;

impl CobolProfile {
    /// COBOL front sequence columns (1-6).
    pub fn front() -> ColumnRange {
        ColumnRange::new(1, 6).expect("COBOL front range is always valid")
    }

    /// COBOL back sequence columns (73-80).
    pub fn back() -> ColumnRange {
        ColumnRange::new(73, 80).expect("COBOL back range is always valid")
    }

    /// COBOL auto_unnum default.
    pub fn auto_unnum() -> bool {
        true
    }
}

/// Built-in FORTRAN profile: front=1-5, back=73-80, auto_unnum=true.
pub struct FortranProfile;

impl FortranProfile {
    /// FORTRAN front sequence columns (1-5).
    pub fn front() -> ColumnRange {
        ColumnRange::new(1, 5).expect("FORTRAN front range is always valid")
    }

    /// FORTRAN back sequence columns (73-80).
    pub fn back() -> ColumnRange {
        ColumnRange::new(73, 80).expect("FORTRAN back range is always valid")
    }

    /// FORTRAN auto_unnum default.
    pub fn auto_unnum() -> bool {
        true
    }
}

/// Built-in JCL profile: no front, back=73-80, auto_unnum=true.
pub struct JclProfile;

impl JclProfile {
    /// JCL has no front sequence columns.
    pub fn front() -> Option<ColumnRange> {
        None
    }

    /// JCL back sequence columns (73-80).
    pub fn back() -> ColumnRange {
        ColumnRange::new(73, 80).expect("JCL back range is always valid")
    }

    /// JCL auto_unnum default.
    pub fn auto_unnum() -> bool {
        true
    }
}

/// Built-in PL/I profile: no front, back=73-80, auto_unnum=true.
pub struct PliProfile;

impl PliProfile {
    /// PL/I has no front sequence columns.
    pub fn front() -> Option<ColumnRange> {
        None
    }

    /// PL/I back sequence columns (73-80).
    pub fn back() -> ColumnRange {
        ColumnRange::new(73, 80).expect("PL/I back range is always valid")
    }

    /// PL/I auto_unnum default.
    pub fn auto_unnum() -> bool {
        true
    }
}

/// A language with no sequence columns defined.
pub struct NoSequenceProfile;

impl NoSequenceProfile {
    /// No front columns.
    pub fn front() -> Option<ColumnRange> {
        None
    }

    /// No back columns.
    pub fn back() -> Option<ColumnRange> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cobol_profile_columns() {
        // Validates: Requirement 1.5
        let front = CobolProfile::front();
        assert_eq!(front.start(), 1);
        assert_eq!(front.end(), 6);
        assert_eq!(front.width(), 6);

        let back = CobolProfile::back();
        assert_eq!(back.start(), 73);
        assert_eq!(back.end(), 80);
        assert_eq!(back.width(), 8);

        assert!(CobolProfile::auto_unnum());
    }

    #[test]
    fn fortran_profile_columns() {
        // Validates: Requirement 1.6
        let front = FortranProfile::front();
        assert_eq!(front.start(), 1);
        assert_eq!(front.end(), 5);
        assert_eq!(front.width(), 5);

        let back = FortranProfile::back();
        assert_eq!(back.start(), 73);
        assert_eq!(back.end(), 80);

        assert!(FortranProfile::auto_unnum());
    }

    #[test]
    fn jcl_profile_columns() {
        // Validates: Requirement 1.7
        assert!(JclProfile::front().is_none());

        let back = JclProfile::back();
        assert_eq!(back.start(), 73);
        assert_eq!(back.end(), 80);

        assert!(JclProfile::auto_unnum());
    }

    #[test]
    fn pli_profile_columns() {
        // Validates: Requirement 1.8
        assert!(PliProfile::front().is_none());

        let back = PliProfile::back();
        assert_eq!(back.start(), 73);
        assert_eq!(back.end(), 80);

        assert!(PliProfile::auto_unnum());
    }

    #[test]
    fn no_sequence_profile() {
        // Validates: Requirement 1.9
        assert!(NoSequenceProfile::front().is_none());
        assert!(NoSequenceProfile::back().is_none());
    }
}
