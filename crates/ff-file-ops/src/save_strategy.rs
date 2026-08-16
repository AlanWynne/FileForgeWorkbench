//! Save strategy definitions for persistence operations.
//!
//! Defines the three persistence strategies: Atomic (temp + rename),
//! DeleteFirst (delete then write), and Direct (overwrite in place).

/// The persistence strategy used when writing file content.
///
/// Addresses: Requirement 7, criteria 1, 6, 7
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum SaveStrategy {
    /// Write to temp file, fsync, atomic rename over target (default).
    ///
    /// Safest strategy: ensures no partial writes are visible. If the
    /// process crashes mid-write, only the temp file is affected.
    #[default]
    Atomic,

    /// Delete target first, then write new content, fsync.
    ///
    /// Used when the provider or configuration specifies `save.deletes.first`.
    /// Less safe than Atomic — a crash after delete but before write loses data.
    DeleteFirst,

    /// Overwrite target in place, fsync (for providers without rename).
    ///
    /// Fallback for VFS providers that do not support rename semantics.
    /// A crash mid-write may leave a partially written file.
    Direct,
}

impl SaveStrategy {
    /// Parse a strategy from a configuration string value.
    ///
    /// # Valid values
    /// - `"atomic"` → `SaveStrategy::Atomic`
    /// - `"delete_first"` → `SaveStrategy::DeleteFirst`
    /// - `"direct"` → `SaveStrategy::Direct`
    ///
    /// Returns `None` for unrecognised values.
    pub fn from_config_str(value: &str) -> Option<Self> {
        match value {
            "atomic" => Some(Self::Atomic),
            "delete_first" => Some(Self::DeleteFirst),
            "direct" => Some(Self::Direct),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_strategy_is_atomic() {
        assert_eq!(SaveStrategy::default(), SaveStrategy::Atomic);
    }

    #[test]
    fn from_config_str_parses_valid_values() {
        assert_eq!(
            SaveStrategy::from_config_str("atomic"),
            Some(SaveStrategy::Atomic)
        );
        assert_eq!(
            SaveStrategy::from_config_str("delete_first"),
            Some(SaveStrategy::DeleteFirst)
        );
        assert_eq!(
            SaveStrategy::from_config_str("direct"),
            Some(SaveStrategy::Direct)
        );
    }

    #[test]
    fn from_config_str_returns_none_for_invalid() {
        assert_eq!(SaveStrategy::from_config_str(""), None);
        assert_eq!(SaveStrategy::from_config_str("unknown"), None);
        assert_eq!(SaveStrategy::from_config_str("ATOMIC"), None);
    }
}
