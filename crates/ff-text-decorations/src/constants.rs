//! Well-known indicator and marker number constants.
//!
//! These constants define the fixed allocations for built-in features
//! (search, diagnostics, IME, history) and prevent accidental conflicts.

use crate::{IndicatorNumber, MarkerNumber};

/// Well-known indicator number allocations.
///
/// Addresses: Requirement 13 AC 3
pub mod indicators {
    use super::IndicatorNumber;

    // Container range (8–31): application-managed
    /// Current search match indicator (StraightBox bright orange).
    pub const SEARCH_CURRENT: IndicatorNumber = IndicatorNumber(8);
    /// All other search matches indicator (RoundBox pale yellow).
    pub const SEARCH_ALL: IndicatorNumber = IndicatorNumber(9);
    /// Error diagnostic indicator (Squiggle red).
    pub const ERROR: IndicatorNumber = IndicatorNumber(10);
    /// Warning diagnostic indicator (Squiggle amber).
    pub const WARNING: IndicatorNumber = IndicatorNumber(11);
    /// Information diagnostic indicator (Plain blue).
    pub const INFO: IndicatorNumber = IndicatorNumber(12);
    /// Hint diagnostic indicator (Dots grey).
    pub const HINT: IndicatorNumber = IndicatorNumber(13);
    // 14–31: available for plugins

    // IME range (32–35)
    /// IME input composition.
    pub const IME_INPUT: IndicatorNumber = IndicatorNumber(32);
    /// IME target (selected in composition).
    pub const IME_TARGET: IndicatorNumber = IndicatorNumber(33);
    /// IME converted text.
    pub const IME_CONVERTED: IndicatorNumber = IndicatorNumber(34);
    /// IME target non-converted.
    pub const IME_TARGET_NON_CONVERTED: IndicatorNumber = IndicatorNumber(35);

    // History range (36–43)
    /// Modified line, insertion indicator.
    pub const HISTORY_MODIFIED_INSERTION: IndicatorNumber = IndicatorNumber(36);
    /// Modified line, deletion indicator.
    pub const HISTORY_MODIFIED_DELETION: IndicatorNumber = IndicatorNumber(37);
    /// Saved line, insertion indicator.
    pub const HISTORY_SAVED_INSERTION: IndicatorNumber = IndicatorNumber(38);
    /// Saved line, deletion indicator.
    pub const HISTORY_SAVED_DELETION: IndicatorNumber = IndicatorNumber(39);
    /// Reverted to origin, insertion indicator.
    pub const HISTORY_REVERTED_ORIGIN_INSERTION: IndicatorNumber = IndicatorNumber(40);
    /// Reverted to origin, deletion indicator.
    pub const HISTORY_REVERTED_ORIGIN_DELETION: IndicatorNumber = IndicatorNumber(41);
    /// Reverted to modified, insertion indicator.
    pub const HISTORY_REVERTED_MODIFIED_INSERTION: IndicatorNumber = IndicatorNumber(42);
    /// Reverted to modified, deletion indicator.
    pub const HISTORY_REVERTED_MODIFIED_DELETION: IndicatorNumber = IndicatorNumber(43);
}

/// Well-known marker number allocations.
///
/// Addresses: Requirements 7, 8
pub mod markers {
    use super::MarkerNumber;

    /// Bookmark marker.
    pub const BOOKMARK: MarkerNumber = MarkerNumber(0);
    /// Modified line (unsaved changes).
    pub const HISTORY_MODIFIED: MarkerNumber = MarkerNumber(1);
    /// Saved line (modified then saved).
    pub const HISTORY_SAVED: MarkerNumber = MarkerNumber(2);
    /// Reverted to original file content.
    pub const HISTORY_REVERTED_ORIGIN: MarkerNumber = MarkerNumber(3);
    /// Reverted to previously modified state.
    pub const HISTORY_REVERTED_MODIFIED: MarkerNumber = MarkerNumber(4);
    // 5–31: available for fold markers, plugins, etc.
}
