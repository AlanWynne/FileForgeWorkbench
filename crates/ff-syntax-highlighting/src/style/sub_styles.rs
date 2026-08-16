//! Sub-style allocation: manages contiguous style-index blocks from the
//! extended range (above base styles) for sub-style differentiation.

use crate::error::SyntaxHighlightError;
use crate::types::StyleSlotIndex;

/// A contiguous block of style-slot indices allocated for sub-style differentiation.
/// Addresses: Requirement 7, criterion 7.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubStyleRange {
    /// The base style this sub-style range belongs to.
    pub base_style: StyleSlotIndex,
    /// First allocated style index in the range.
    pub start: StyleSlotIndex,
    /// Number of allocated indices.
    pub count: u8,
}

impl SubStyleRange {
    /// Get the style index at position `offset` within this range.
    pub fn index_at(&self, offset: u8) -> Option<StyleSlotIndex> {
        if offset < self.count {
            Some(StyleSlotIndex(self.start.0.checked_add(offset)?))
        } else {
            None
        }
    }

    /// Check if a style index falls within this sub-style range.
    pub fn contains(&self, style: StyleSlotIndex) -> bool {
        style.0 >= self.start.0 && style.0 < self.start.0.saturating_add(self.count)
    }
}

/// Manages allocation of contiguous style-index blocks from the extended range.
/// The total budget is 256 indices shared between base styles and sub-styles.
/// Addresses: Requirement 7
pub struct SubStyleAllocator {
    /// Number of base styles already in use.
    _base_style_count: u8,
    /// Next available index for allocation.
    next_available: u8,
    /// Active allocations keyed by base style index.
    allocations: Vec<SubStyleRange>,
    /// Freed ranges available for reuse.
    freed: Vec<(u8, u8)>, // (start_index, count)
}

impl SubStyleAllocator {
    /// Create an allocator with the given number of base styles already in use.
    pub fn new(base_style_count: u8) -> Self {
        Self {
            _base_style_count: base_style_count,
            next_available: base_style_count,
            allocations: Vec::new(),
            freed: Vec::new(),
        }
    }

    /// Allocate a contiguous block of sub-style indices.
    /// Addresses: Requirement 7, criterion 7.2
    pub fn allocate(
        &mut self,
        base_style: StyleSlotIndex,
        count: u8,
    ) -> Result<SubStyleRange, SyntaxHighlightError> {
        if count == 0 {
            return Ok(SubStyleRange {
                base_style,
                start: StyleSlotIndex(self.next_available),
                count: 0,
            });
        }

        // Try to reuse a freed range
        let mut best_idx = None;
        for (i, &(_start, freed_count)) in self.freed.iter().enumerate() {
            if freed_count >= count {
                match best_idx {
                    None => best_idx = Some(i),
                    Some(bi) => {
                        if freed_count < self.freed[bi].1 {
                            best_idx = Some(i);
                        }
                    }
                }
            }
        }

        if let Some(idx) = best_idx {
            let (start, freed_count) = self.freed[idx];
            let range = SubStyleRange {
                base_style,
                start: StyleSlotIndex(start),
                count,
            };
            if freed_count > count {
                // Shrink the freed range
                self.freed[idx] = (start + count, freed_count - count);
            } else {
                self.freed.swap_remove(idx);
            }
            self.allocations.push(range.clone());
            return Ok(range);
        }

        // Allocate from the end
        let available = 255u16.saturating_sub(self.next_available as u16) + 1;
        if (count as u16) > available {
            return Err(SyntaxHighlightError::SubStyleAllocationExhausted {
                base_style: base_style.0,
                requested: count,
                available: available as u8,
            });
        }

        let start = self.next_available;
        let range = SubStyleRange {
            base_style,
            start: StyleSlotIndex(start),
            count,
        };
        self.next_available = start.saturating_add(count);
        self.allocations.push(range.clone());
        Ok(range)
    }

    /// Free all sub-style allocations for a base style.
    /// Addresses: Requirement 7, criterion 7.5
    pub fn free(&mut self, base_style: StyleSlotIndex) {
        let mut freed_ranges = Vec::new();
        self.allocations.retain(|alloc| {
            if alloc.base_style == base_style {
                freed_ranges.push((alloc.start.0, alloc.count));
                false
            } else {
                true
            }
        });
        self.freed.extend(freed_ranges);
    }

    /// Get the base style for a given sub-style index.
    /// Addresses: Requirement 7, criterion 7.7
    pub fn base_for(&self, sub_style: StyleSlotIndex) -> Option<StyleSlotIndex> {
        for alloc in &self.allocations {
            if alloc.contains(sub_style) {
                return Some(alloc.base_style);
            }
        }
        None
    }

    /// Get the allocated range for a base style.
    pub fn range_for(&self, base_style: StyleSlotIndex) -> Option<&SubStyleRange> {
        self.allocations
            .iter()
            .find(|alloc| alloc.base_style == base_style)
    }

    /// Get the number of available style indices remaining.
    pub fn available(&self) -> u8 {
        let freed_total: u16 = self.freed.iter().map(|(_, c)| *c as u16).sum();
        let end_available = 255u16.saturating_sub(self.next_available as u16) + 1;
        (freed_total + end_available).min(255) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_basic() {
        // Validates: Requirement 7, criterion 7.2
        let mut alloc = SubStyleAllocator::new(10);
        let range = alloc.allocate(StyleSlotIndex(1), 5).unwrap();
        assert_eq!(range.base_style, StyleSlotIndex(1));
        assert_eq!(range.start, StyleSlotIndex(10));
        assert_eq!(range.count, 5);
    }

    #[test]
    fn allocate_multiple_non_overlapping() {
        // Validates: Requirement 7, criterion 7.1
        let mut alloc = SubStyleAllocator::new(10);
        let r1 = alloc.allocate(StyleSlotIndex(1), 5).unwrap();
        let r2 = alloc.allocate(StyleSlotIndex(2), 3).unwrap();
        // Ranges must not overlap
        assert!(r1.start.0 + r1.count <= r2.start.0 || r2.start.0 + r2.count <= r1.start.0);
    }

    #[test]
    fn allocate_exhausts_pool() {
        // Validates: Requirement 7, criterion 7.6
        let mut alloc = SubStyleAllocator::new(200);
        // 56 remaining (200..255 = 56 indices)
        let result = alloc.allocate(StyleSlotIndex(1), 100);
        assert!(result.is_err());
        if let Err(SyntaxHighlightError::SubStyleAllocationExhausted { available, .. }) = result {
            assert_eq!(available, 56);
        }
    }

    #[test]
    fn free_releases_indices_for_reuse() {
        // Validates: Requirement 7, criterion 7.5
        let mut alloc = SubStyleAllocator::new(10);
        let _r1 = alloc.allocate(StyleSlotIndex(1), 5).unwrap();
        alloc.free(StyleSlotIndex(1));
        // Should be able to reuse those indices
        let r2 = alloc.allocate(StyleSlotIndex(2), 5).unwrap();
        assert_eq!(r2.start, StyleSlotIndex(10)); // reused
    }

    #[test]
    fn base_for_returns_correct_base() {
        // Validates: Requirement 7, criterion 7.7
        let mut alloc = SubStyleAllocator::new(10);
        let range = alloc.allocate(StyleSlotIndex(3), 5).unwrap();
        assert_eq!(alloc.base_for(range.start), Some(StyleSlotIndex(3)));
        assert_eq!(
            alloc.base_for(StyleSlotIndex(range.start.0 + 2)),
            Some(StyleSlotIndex(3))
        );
        assert_eq!(alloc.base_for(StyleSlotIndex(0)), None);
    }

    #[test]
    fn sub_style_range_index_at() {
        let range = SubStyleRange {
            base_style: StyleSlotIndex(1),
            start: StyleSlotIndex(20),
            count: 5,
        };
        assert_eq!(range.index_at(0), Some(StyleSlotIndex(20)));
        assert_eq!(range.index_at(4), Some(StyleSlotIndex(24)));
        assert_eq!(range.index_at(5), None);
    }

    #[test]
    fn sub_style_range_contains() {
        let range = SubStyleRange {
            base_style: StyleSlotIndex(1),
            start: StyleSlotIndex(20),
            count: 5,
        };
        assert!(!range.contains(StyleSlotIndex(19)));
        assert!(range.contains(StyleSlotIndex(20)));
        assert!(range.contains(StyleSlotIndex(24)));
        assert!(!range.contains(StyleSlotIndex(25)));
    }

    #[test]
    fn allocate_zero_count_succeeds() {
        let mut alloc = SubStyleAllocator::new(10);
        let range = alloc.allocate(StyleSlotIndex(1), 0).unwrap();
        assert_eq!(range.count, 0);
    }
}
