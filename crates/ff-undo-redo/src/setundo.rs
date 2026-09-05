//! SETUNDO and RECOVERY primary command parsing and application.
//!
//! Validates: Requirement 19.1, 19.2

use crate::config::{MAX_MAX_LEVELS, MIN_MAX_LEVELS};
use crate::manager::DocumentUndoManager;

// === SETUNDO operand ========================================================

/// Parsed operand for the SETUNDO command.
///
/// Validates: Requirement 19.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetundoOperand {
    /// SETUNDO ON -- re-enable undo, restore configured max_levels.
    On,
    /// SETUNDO OFF -- disable undo (max_levels = 0).
    Off,
    /// SETUNDO n -- set max_levels to n (0-10000).
    Levels(u32),
}

impl SetundoOperand {
    /// Parses the operand string from the SETUNDO command line.
    ///
    /// Returns `None` for empty or unrecognised input.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "ON" => Some(Self::On),
            "OFF" => Some(Self::Off),
            other => {
                let n: u32 = other.parse().ok()?;
                if n <= MAX_MAX_LEVELS {
                    Some(Self::Levels(n))
                } else {
                    None
                }
            }
        }
    }
}

/// Applies a SETUNDO operand to a `DocumentUndoManager`.
///
/// `configured_max_levels` is the value from the persistent configuration,
/// used to restore the original limit when SETUNDO ON is issued.
///
/// Validates: Requirement 19.1
pub fn apply_setundo(
    mgr: &mut DocumentUndoManager,
    operand: &SetundoOperand,
    configured_max_levels: u32,
) {
    match operand {
        SetundoOperand::On => mgr.set_max_levels(configured_max_levels),
        SetundoOperand::Off => mgr.set_max_levels(MIN_MAX_LEVELS),
        SetundoOperand::Levels(n) => mgr.set_max_levels(*n),
    }
}

// === RECOVERY operand =======================================================

/// Parsed operand for the RECOVERY command.
///
/// Validates: Requirement 19.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryOperand {
    /// RECOVERY ON -- re-enable recovery, restore configured interval.
    On,
    /// RECOVERY OFF -- disable recovery (interval = 0).
    Off,
    /// RECOVERY n -- set recovery interval to n seconds.
    Interval(u32),
}

impl RecoveryOperand {
    /// Parses the operand string from the RECOVERY command line.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "ON" => Some(Self::On),
            "OFF" => Some(Self::Off),
            other => {
                let n: u32 = other.parse().ok()?;
                Some(Self::Interval(n))
            }
        }
    }
}

/// Applies a RECOVERY operand to a `DocumentUndoManager`.
///
/// `configured_interval` is the value from the persistent configuration,
/// used to restore the original interval when RECOVERY ON is issued.
///
/// Validates: Requirement 19.2
pub fn apply_recovery(
    mgr: &mut DocumentUndoManager,
    operand: &RecoveryOperand,
    configured_interval: u32,
) {
    match operand {
        RecoveryOperand::On => mgr.set_recovery_interval(configured_interval),
        RecoveryOperand::Off => mgr.set_recovery_interval(0),
        RecoveryOperand::Interval(n) => mgr.set_recovery_interval(*n),
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UndoConfig;

    fn mgr_with(max_levels: u32, recovery: u32) -> DocumentUndoManager {
        DocumentUndoManager::new(UndoConfig {
            max_levels,
            recovery_interval_seconds: recovery,
            ..UndoConfig::default()
        })
    }

    // --- SetundoOperand parsing ---

    #[test]
    fn setundo_parse_on() {
        // Validates: Requirement 19.1
        assert_eq!(SetundoOperand::parse("ON"), Some(SetundoOperand::On));
        assert_eq!(SetundoOperand::parse("on"), Some(SetundoOperand::On));
    }

    #[test]
    fn setundo_parse_off() {
        // Validates: Requirement 19.1
        assert_eq!(SetundoOperand::parse("OFF"), Some(SetundoOperand::Off));
    }

    #[test]
    fn setundo_parse_numeric() {
        // Validates: Requirement 19.1
        assert_eq!(
            SetundoOperand::parse("50"),
            Some(SetundoOperand::Levels(50))
        );
        assert_eq!(SetundoOperand::parse("0"), Some(SetundoOperand::Levels(0)));
        assert_eq!(
            SetundoOperand::parse("10000"),
            Some(SetundoOperand::Levels(10000))
        );
    }

    #[test]
    fn setundo_parse_out_of_range_returns_none() {
        // Validates: Requirement 19.1 -- range 0-10000
        assert_eq!(SetundoOperand::parse("10001"), None);
        assert_eq!(SetundoOperand::parse("99999"), None);
    }

    #[test]
    fn setundo_parse_empty_returns_none() {
        assert_eq!(SetundoOperand::parse(""), None);
    }

    #[test]
    fn setundo_parse_garbage_returns_none() {
        assert_eq!(SetundoOperand::parse("BOGUS"), None);
    }

    // --- apply_setundo ---

    #[test]
    fn setundo_off_disables_undo() {
        // Validates: Requirement 19.1 -- SETUNDO OFF equivalent to max_levels=0
        let mut mgr = mgr_with(100, 60);
        apply_setundo(&mut mgr, &SetundoOperand::Off, 100);
        assert!(!mgr.can_undo());
        // Recording should be a no-op when disabled
        mgr.record_insert(0, b"hello");
        assert!(!mgr.can_undo());
    }

    #[test]
    fn setundo_on_restores_configured_max_levels() {
        // Validates: Requirement 19.1 -- SETUNDO ON restores configured value
        let mut mgr = mgr_with(100, 60);
        apply_setundo(&mut mgr, &SetundoOperand::Off, 100);
        assert_eq!(mgr.max_levels(), 0);
        apply_setundo(&mut mgr, &SetundoOperand::On, 100);
        assert_eq!(mgr.max_levels(), 100);
    }

    #[test]
    fn setundo_n_sets_max_levels_immediately() {
        // Validates: Requirement 19.1 -- immediate effect
        let mut mgr = mgr_with(100, 60);
        apply_setundo(&mut mgr, &SetundoOperand::Levels(5), 100);
        assert_eq!(mgr.max_levels(), 5);
        // Push 6 transactions -- only 5 should be retained
        for i in 0..6u64 {
            mgr.break_coalesce();
            mgr.record_insert(i, b"x");
        }
        assert_eq!(mgr.undo_depth(), 5);
    }

    #[test]
    fn setundo_zero_disables_undo() {
        // Validates: Requirement 19.1 -- SETUNDO 0 same as SETUNDO OFF
        let mut mgr = mgr_with(100, 60);
        apply_setundo(&mut mgr, &SetundoOperand::Levels(0), 100);
        mgr.record_insert(0, b"x");
        assert!(!mgr.can_undo());
    }

    #[test]
    fn setundo_shrink_trims_oldest_entries() {
        // Validates: Requirement 19.1 -- shrinking takes immediate effect
        let mut mgr = mgr_with(10, 60);
        for i in 0..8u64 {
            mgr.break_coalesce();
            mgr.record_insert(i, b"x");
        }
        assert_eq!(mgr.undo_depth(), 8);
        apply_setundo(&mut mgr, &SetundoOperand::Levels(3), 10);
        assert_eq!(mgr.undo_depth(), 3);
    }

    // --- RecoveryOperand parsing ---

    #[test]
    fn recovery_parse_on() {
        // Validates: Requirement 19.2
        assert_eq!(RecoveryOperand::parse("ON"), Some(RecoveryOperand::On));
        assert_eq!(RecoveryOperand::parse("on"), Some(RecoveryOperand::On));
    }

    #[test]
    fn recovery_parse_off() {
        // Validates: Requirement 19.2
        assert_eq!(RecoveryOperand::parse("OFF"), Some(RecoveryOperand::Off));
    }

    #[test]
    fn recovery_parse_numeric() {
        // Validates: Requirement 19.2
        assert_eq!(
            RecoveryOperand::parse("30"),
            Some(RecoveryOperand::Interval(30))
        );
        assert_eq!(
            RecoveryOperand::parse("0"),
            Some(RecoveryOperand::Interval(0))
        );
    }

    #[test]
    fn recovery_parse_empty_returns_none() {
        assert_eq!(RecoveryOperand::parse(""), None);
    }

    #[test]
    fn recovery_parse_garbage_returns_none() {
        assert_eq!(RecoveryOperand::parse("BOGUS"), None);
    }

    // --- apply_recovery ---

    #[test]
    fn recovery_off_sets_interval_to_zero() {
        // Validates: Requirement 19.2 -- RECOVERY OFF disables
        let mut mgr = mgr_with(100, 60);
        apply_recovery(&mut mgr, &RecoveryOperand::Off, 60);
        assert_eq!(mgr.recovery_interval(), 0);
    }

    #[test]
    fn recovery_on_restores_configured_interval() {
        // Validates: Requirement 19.2 -- RECOVERY ON restores configured value
        let mut mgr = mgr_with(100, 60);
        apply_recovery(&mut mgr, &RecoveryOperand::Off, 60);
        assert_eq!(mgr.recovery_interval(), 0);
        apply_recovery(&mut mgr, &RecoveryOperand::On, 60);
        assert_eq!(mgr.recovery_interval(), 60);
    }

    #[test]
    fn recovery_n_sets_interval_immediately() {
        // Validates: Requirement 19.2 -- immediate effect
        let mut mgr = mgr_with(100, 60);
        apply_recovery(&mut mgr, &RecoveryOperand::Interval(120), 60);
        assert_eq!(mgr.recovery_interval(), 120);
    }

    #[test]
    fn recovery_zero_disables_recovery() {
        // Validates: Requirement 19.2 -- interval=0 disables
        let mut mgr = mgr_with(100, 60);
        apply_recovery(&mut mgr, &RecoveryOperand::Interval(0), 60);
        assert_eq!(mgr.recovery_interval(), 0);
    }
}
