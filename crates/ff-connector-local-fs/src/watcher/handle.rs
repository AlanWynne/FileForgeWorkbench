//! WatchId — opaque identifier for a watch registration.
//!
//! Addresses: Requirement 3, criterion 8

/// Opaque identifier for a watch registration.
///
/// Returned when a watch is registered, used to remove or query the watch later.
///
/// Addresses: Requirement 3, criterion 8
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WatchId(pub(crate) u64);

impl WatchId {
    /// Returns the raw numeric value of this watch ID.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for WatchId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WatchId({})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_id_equality() {
        let a = WatchId(1);
        let b = WatchId(1);
        let c = WatchId(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn watch_id_display() {
        let id = WatchId(42);
        assert_eq!(format!("{}", id), "WatchId(42)");
    }

    #[test]
    fn watch_id_as_u64() {
        let id = WatchId(99);
        assert_eq!(id.as_u64(), 99);
    }
}
