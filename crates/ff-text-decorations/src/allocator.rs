//! Indicator number allocation and namespace management.
//!
//! Manages the container range (8–31) for plugin indicator allocation,
//! preventing conflicts between independent producers.

use crate::error::DecorationError;
use crate::IndicatorNumber;

/// Manages indicator number allocation and namespace enforcement.
///
/// Addresses: Requirement 13 AC 1–6
pub struct IndicatorAllocator {
    /// Tracks which container-range indicators (8–31) are allocated.
    allocated: [bool; 24],
    /// Plugin ID associated with each allocated slot.
    owners: [Option<String>; 24],
}

impl IndicatorAllocator {
    /// Create a new allocator with no indicators allocated.
    pub fn new() -> Self {
        Self {
            allocated: [false; 24],
            owners: std::array::from_fn(|_| None),
        }
    }

    /// Allocate an indicator number from the container range (8–31) for a plugin.
    ///
    /// Addresses: Requirement 13 AC 4, 5
    pub fn allocate(&mut self, plugin_id: &str) -> Result<IndicatorNumber, DecorationError> {
        for i in 0..24 {
            if !self.allocated[i] {
                self.allocated[i] = true;
                self.owners[i] = Some(plugin_id.to_string());
                return Ok(IndicatorNumber(i as u8 + 8));
            }
        }
        Err(DecorationError::NoAvailableIndicators)
    }

    /// Release a previously allocated indicator number.
    pub fn release(&mut self, indicator: IndicatorNumber) -> Result<(), DecorationError> {
        if !Self::is_container_range(indicator) {
            return Err(DecorationError::NotAllocated {
                number: indicator.0,
            });
        }
        let idx = (indicator.0 - 8) as usize;
        if !self.allocated[idx] {
            return Err(DecorationError::NotAllocated {
                number: indicator.0,
            });
        }
        self.allocated[idx] = false;
        self.owners[idx] = None;
        Ok(())
    }

    /// Check if an indicator number is in the lexer range (0–7).
    ///
    /// Addresses: Requirement 13 AC 6
    pub fn is_lexer_range(indicator: IndicatorNumber) -> bool {
        indicator.0 <= 7
    }

    /// Check if an indicator number is in the container range (8–31).
    pub fn is_container_range(indicator: IndicatorNumber) -> bool {
        indicator.0 >= 8 && indicator.0 <= 31
    }

    /// Check if an indicator number is in the IME range (32–35).
    pub fn is_ime_range(indicator: IndicatorNumber) -> bool {
        indicator.0 >= 32 && indicator.0 <= 35
    }

    /// Check if an indicator number is in the history range (36–43).
    pub fn is_history_range(indicator: IndicatorNumber) -> bool {
        indicator.0 >= 36 && indicator.0 <= 43
    }
}

impl Default for IndicatorAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_returns_first_available() {
        // Validates: Requirement 13 AC 4
        let mut alloc = IndicatorAllocator::new();
        let indicator = alloc.allocate("test-plugin").unwrap();
        assert_eq!(indicator.0, 8);
    }

    #[test]
    fn allocate_returns_sequential_numbers() {
        let mut alloc = IndicatorAllocator::new();
        let i1 = alloc.allocate("plugin-a").unwrap();
        let i2 = alloc.allocate("plugin-b").unwrap();
        assert_eq!(i1.0, 8);
        assert_eq!(i2.0, 9);
    }

    #[test]
    fn release_frees_indicator_for_reuse() {
        let mut alloc = IndicatorAllocator::new();
        let indicator = alloc.allocate("test-plugin").unwrap();
        alloc.release(indicator).unwrap();
        let reused = alloc.allocate("another-plugin").unwrap();
        assert_eq!(reused.0, 8);
    }

    #[test]
    fn allocate_exhaustion_returns_error() {
        // Validates: Requirement 13 AC 5
        let mut alloc = IndicatorAllocator::new();
        for i in 0..24 {
            alloc.allocate(&format!("plugin-{i}")).unwrap();
        }
        let result = alloc.allocate("one-too-many");
        assert!(result.is_err());
    }

    #[test]
    fn release_unallocated_returns_error() {
        let mut alloc = IndicatorAllocator::new();
        let result = alloc.release(IndicatorNumber(8));
        assert!(result.is_err());
    }

    #[test]
    fn range_predicates_are_correct() {
        // Validates: Requirement 13 AC 6
        assert!(IndicatorAllocator::is_lexer_range(IndicatorNumber(0)));
        assert!(IndicatorAllocator::is_lexer_range(IndicatorNumber(7)));
        assert!(!IndicatorAllocator::is_lexer_range(IndicatorNumber(8)));

        assert!(IndicatorAllocator::is_container_range(IndicatorNumber(8)));
        assert!(IndicatorAllocator::is_container_range(IndicatorNumber(31)));
        assert!(!IndicatorAllocator::is_container_range(IndicatorNumber(32)));

        assert!(IndicatorAllocator::is_ime_range(IndicatorNumber(32)));
        assert!(IndicatorAllocator::is_ime_range(IndicatorNumber(35)));
        assert!(!IndicatorAllocator::is_ime_range(IndicatorNumber(36)));

        assert!(IndicatorAllocator::is_history_range(IndicatorNumber(36)));
        assert!(IndicatorAllocator::is_history_range(IndicatorNumber(43)));
        assert!(!IndicatorAllocator::is_history_range(IndicatorNumber(44)));
    }
}
