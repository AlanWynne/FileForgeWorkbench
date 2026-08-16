//! Wrap mode enumeration.
//!
//! Defines the three wrap modes supported by the editor: None, Word, and Character.

/// The three wrap modes supported by the editor.
///
/// Addresses: Requirement 1 (Wrap Mode Enumeration)
///
/// # Variants
///
/// - `None` — No wrapping; each document line occupies exactly one display row.
/// - `Word` — Lines break at word boundaries (whitespace, punctuation adjacent to alphanumeric).
/// - `Character` — Lines break at the exact character position filling the boundary width.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[non_exhaustive]
pub enum WrapMode {
    /// No wrapping — each document line occupies exactly one display row.
    /// Long lines extend beyond the viewport edge (horizontal scroll required).
    #[default]
    None,

    /// Word-boundary wrapping — lines break at word boundaries (whitespace,
    /// punctuation adjacent to alphanumeric). Falls back to character-level
    /// for words exceeding the boundary width.
    Word,

    /// Character-boundary wrapping — lines break at the exact character
    /// position that fills the boundary width.
    Character,
}

impl WrapMode {
    /// The default enabled mode used by `WRAP ON` / `WRAP TOGGLE`.
    ///
    /// Per Requirement 1 AC 6: Word is the default wrapping style when the
    /// user enables wrapping without specifying a mode.
    pub const DEFAULT_ENABLED: Self = Self::Word;

    /// Whether wrap is currently active (not `None`).
    ///
    /// Returns `true` for `Word` and `Character`, `false` for `None`.
    pub fn is_active(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns a display label suitable for status messages.
    ///
    /// - `None` → `"Off"`
    /// - `Word` → `"Word"`
    /// - `Character` → `"Char"`
    pub fn display_label(self) -> &'static str {
        match self {
            Self::None => "Off",
            Self::Word => "Word",
            Self::Character => "Char",
        }
    }
}

impl std::fmt::Display for WrapMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_mode_default_is_none() {
        // Validates: Requirement 2.2 — default mode is None
        assert_eq!(WrapMode::default(), WrapMode::None);
    }

    #[test]
    fn is_active_returns_false_for_none() {
        // Validates: Requirement 1.2
        assert!(!WrapMode::None.is_active());
    }

    #[test]
    fn is_active_returns_true_for_word() {
        // Validates: Requirement 1.3
        assert!(WrapMode::Word.is_active());
    }

    #[test]
    fn is_active_returns_true_for_character() {
        // Validates: Requirement 1.5
        assert!(WrapMode::Character.is_active());
    }

    #[test]
    fn default_enabled_is_word() {
        // Validates: Requirement 1.6
        assert_eq!(WrapMode::DEFAULT_ENABLED, WrapMode::Word);
    }

    #[test]
    fn display_label_for_none_is_off() {
        assert_eq!(WrapMode::None.display_label(), "Off");
    }

    #[test]
    fn display_label_for_word() {
        assert_eq!(WrapMode::Word.display_label(), "Word");
    }

    #[test]
    fn display_label_for_character_is_char() {
        assert_eq!(WrapMode::Character.display_label(), "Char");
    }
}
