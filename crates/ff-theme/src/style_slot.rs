//! Style slot system: 256 indexed entries with font/colour/attribute combinations.
//!
//! Adapted from Scintilla's 256-style architecture for efficient
//! syntax-highlighting integration. Each slot defines the visual
//! attributes for rendering a particular token type.

use serde::{Deserialize, Serialize};

use crate::colour::ColourRGBA;
use crate::error::ThemeError;

/// Reserved style slot index: Default (all undefined slots inherit from this).
pub const DEFAULT_STYLE_INDEX: u8 = 32;
/// Reserved style slot index: Line Number gutter.
pub const LINE_NUMBER_STYLE_INDEX: u8 = 33;
/// Reserved style slot index: Brace Highlight.
pub const BRACE_HIGHLIGHT_STYLE_INDEX: u8 = 34;
/// Reserved style slot index: Brace Mismatch.
pub const BRACE_MISMATCH_STYLE_INDEX: u8 = 35;
/// Reserved style slot index: Control Character.
pub const CONTROL_CHAR_STYLE_INDEX: u8 = 36;
/// Reserved style slot index: Indent Guide.
pub const INDENT_GUIDE_STYLE_INDEX: u8 = 37;
/// Reserved style slot index: Call Tip.
pub const CALL_TIP_STYLE_INDEX: u8 = 38;
/// Reserved style slot index: Fold Display Text.
pub const FOLD_DISPLAY_STYLE_INDEX: u8 = 39;

/// First index available for dynamic allocation (after reserved range).
const FIRST_ALLOCATABLE_INDEX: u8 = 40;

/// Case transformation applied when rendering a style slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CaseTransform {
    /// No transformation.
    #[default]
    None,
    /// Convert to UPPERCASE.
    Upper,
    /// Convert to lowercase.
    Lower,
    /// Convert to camelCase.
    Camel,
}

/// A single style slot defining visual attributes for a syntax token type.
///
/// Each slot specifies foreground/background colours, font attributes
/// (bold, italic, underline), an optional font family override, and a
/// case transformation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleSlot {
    /// Foreground colour for this slot.
    pub foreground: ColourRGBA,
    /// Background colour for this slot.
    pub background: ColourRGBA,
    /// Optional font family override. `None` means use the default monospace stack.
    pub font_family: Option<String>,
    /// Bold text attribute.
    pub bold: bool,
    /// Italic text attribute.
    pub italic: bool,
    /// Underline text attribute.
    pub underline: bool,
    /// Case transformation applied to rendered text.
    pub case_transform: CaseTransform,
}

impl Default for StyleSlot {
    fn default() -> Self {
        Self {
            foreground: ColourRGBA::rgb(204, 204, 204),
            background: ColourRGBA::rgb(30, 30, 46),
            font_family: None,
            bold: false,
            italic: false,
            underline: false,
            case_transform: CaseTransform::None,
        }
    }
}

/// The 256-entry indexed style slot table.
///
/// Unset slots inherit all attributes from the Default slot (index 32).
/// The table tracks which slots have been explicitly defined versus inherited.
#[derive(Debug, Clone, PartialEq)]
pub struct StyleSlotTable {
    /// The 256 style slots.
    slots: Vec<StyleSlot>,
    /// Tracks which slots have been explicitly defined.
    defined: Vec<bool>,
    /// Next available index for dynamic allocation.
    next_available: u8,
}

impl StyleSlotTable {
    /// Create a new style slot table with the default slot initialised.
    pub fn new(default_slot: StyleSlot) -> Self {
        let mut slots = vec![default_slot.clone(); 256];
        let mut defined = vec![false; 256];

        // Mark the default slot as defined
        slots[DEFAULT_STYLE_INDEX as usize] = default_slot;
        defined[DEFAULT_STYLE_INDEX as usize] = true;

        Self {
            slots,
            defined,
            next_available: FIRST_ALLOCATABLE_INDEX,
        }
    }

    /// Get the style slot at the given index.
    ///
    /// Undefined slots return values inherited from the Default slot (index 32).
    pub fn get(&self, index: u8) -> &StyleSlot {
        &self.slots[index as usize]
    }

    /// Set the style slot at the given index.
    pub fn set(&mut self, index: u8, slot: StyleSlot) {
        self.slots[index as usize] = slot;
        self.defined[index as usize] = true;
    }

    /// Check if a slot has been explicitly defined (vs inherited from default).
    pub fn is_defined(&self, index: u8) -> bool {
        self.defined[index as usize]
    }

    /// Get the Default style slot (index 32).
    pub fn default_slot(&self) -> &StyleSlot {
        &self.slots[DEFAULT_STYLE_INDEX as usize]
    }

    /// Allocate a contiguous block of style slots for extended syntax styles.
    ///
    /// Returns the starting index of the allocated block, or an error if
    /// insufficient contiguous slots are available.
    ///
    /// # Errors
    ///
    /// Returns `ThemeError::SlotAllocationExhausted` if there are not enough
    /// contiguous slots available.
    pub fn allocate_range(&mut self, count: u8) -> Result<u8, ThemeError> {
        if count == 0 {
            return Ok(self.next_available);
        }

        let available = 255u8.saturating_sub(self.next_available).saturating_add(1);
        if count > available {
            return Err(ThemeError::SlotAllocationExhausted {
                requested: count,
                available,
            });
        }

        let start = self.next_available;
        self.next_available = self.next_available.saturating_add(count);
        Ok(start)
    }

    /// Resolve a slot's font family through the font stack mechanism.
    ///
    /// If the slot has a font family set, returns it. Otherwise returns `None`
    /// to indicate the default monospace stack should be used.
    pub fn resolved_font_family(&self, index: u8) -> Option<&str> {
        self.slots[index as usize].font_family.as_deref()
    }
}

impl Default for StyleSlotTable {
    fn default() -> Self {
        Self::new(StyleSlot::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undefined_slots_inherit_from_default() {
        // Validates: Requirement 3.4
        let default_slot = StyleSlot {
            foreground: ColourRGBA::rgb(255, 255, 0),
            background: ColourRGBA::rgb(0, 0, 128),
            font_family: None,
            bold: true,
            italic: false,
            underline: true,
            case_transform: CaseTransform::Upper,
        };
        let table = StyleSlotTable::new(default_slot.clone());

        // Any undefined slot should return the default values
        assert_eq!(table.get(0), &default_slot);
        assert_eq!(table.get(100), &default_slot);
        assert_eq!(table.get(255), &default_slot);
    }

    #[test]
    fn defined_slot_returns_its_own_values() {
        // Validates: Requirement 3.4
        let mut table = StyleSlotTable::default();
        let custom = StyleSlot {
            foreground: ColourRGBA::rgb(255, 0, 0),
            background: ColourRGBA::rgb(0, 255, 0),
            font_family: Some("Consolas".to_string()),
            bold: true,
            italic: true,
            underline: false,
            case_transform: CaseTransform::Lower,
        };
        table.set(50, custom.clone());

        assert_eq!(table.get(50), &custom);
        assert!(table.is_defined(50));
        assert!(!table.is_defined(51));
    }

    #[test]
    fn reserved_indices_are_correct() {
        // Validates: Requirement 3.3
        assert_eq!(DEFAULT_STYLE_INDEX, 32);
        assert_eq!(LINE_NUMBER_STYLE_INDEX, 33);
        assert_eq!(BRACE_HIGHLIGHT_STYLE_INDEX, 34);
        assert_eq!(BRACE_MISMATCH_STYLE_INDEX, 35);
        assert_eq!(CONTROL_CHAR_STYLE_INDEX, 36);
        assert_eq!(INDENT_GUIDE_STYLE_INDEX, 37);
        assert_eq!(CALL_TIP_STYLE_INDEX, 38);
        assert_eq!(FOLD_DISPLAY_STYLE_INDEX, 39);
    }

    #[test]
    fn allocate_range_returns_contiguous_block() {
        // Validates: Requirement 3.5
        let mut table = StyleSlotTable::default();
        let start = table.allocate_range(10).unwrap();
        assert_eq!(start, FIRST_ALLOCATABLE_INDEX);

        let start2 = table.allocate_range(5).unwrap();
        assert_eq!(start2, FIRST_ALLOCATABLE_INDEX + 10);
    }

    #[test]
    fn allocate_range_fails_when_exhausted() {
        // Validates: Requirement 3.5
        let mut table = StyleSlotTable::default();
        // Allocate most of the available space
        let _ = table.allocate_range(200).unwrap();
        // Try to allocate more than remaining
        let result = table.allocate_range(100);
        assert!(result.is_err());
    }

    #[test]
    fn font_family_resolution_returns_none_for_default() {
        // Validates: Requirement 3.6
        let table = StyleSlotTable::default();
        assert_eq!(table.resolved_font_family(0), None);
    }

    #[test]
    fn font_family_resolution_returns_set_family() {
        // Validates: Requirement 3.6
        let mut table = StyleSlotTable::default();
        let slot = StyleSlot {
            font_family: Some("Fira Code".to_string()),
            ..StyleSlot::default()
        };
        table.set(50, slot);
        assert_eq!(table.resolved_font_family(50), Some("Fira Code"));
    }
}
