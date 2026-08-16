//! ASA carriage control character parsing and classification.
//!
//! The ASA (ANSI) carriage control standard defines characters in column 1
//! of each record that control the printer's vertical paper motion before
//! printing the line's data content.

/// ASA carriage control character classification.
///
/// Represents the printer action encoded in column 1 of each record.
///
/// # Variants
///
/// Each variant corresponds to a standard ASA control character that
/// directs the line printer's vertical motion before printing.
// Validates: Requirement 1.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AsaControl {
    /// Space (` `) — single space before printing (normal line advance).
    Space,
    /// `0` — double space before printing (skip one blank line).
    DoubleSpace,
    /// `-` — triple space before printing (skip two blank lines).
    TripleSpace,
    /// `1` — page eject (advance to top of next page before printing).
    PageEject,
    /// `+` — no advance (overstrike/overprint on previous line).
    Overstrike,
    /// `H` — halt (printer halt indication).
    Halt,
    /// Unrecognised character — treated as single space with a warning.
    Unknown(char),
}

/// The set of valid ASA control characters for detection purposes.
pub const ASA_VALID_CHARS: &[char] = &[' ', '0', '-', '1', '+', 'H'];

impl AsaControl {
    /// Parse a single character into an ASA control classification.
    ///
    /// Returns the appropriate variant for known ASA characters,
    /// or `Unknown(ch)` for unrecognised characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_asa::control::AsaControl;
    ///
    /// assert_eq!(AsaControl::from_char(' '), AsaControl::Space);
    /// assert_eq!(AsaControl::from_char('1'), AsaControl::PageEject);
    /// assert_eq!(AsaControl::from_char('X'), AsaControl::Unknown('X'));
    /// ```
    // Validates: Requirement 1.1, 1.9
    pub fn from_char(ch: char) -> Self {
        match ch {
            ' ' => Self::Space,
            '0' => Self::DoubleSpace,
            '-' => Self::TripleSpace,
            '1' => Self::PageEject,
            '+' => Self::Overstrike,
            'H' => Self::Halt,
            other => Self::Unknown(other),
        }
    }

    /// Convert back to the original column-1 character.
    ///
    /// For `Unknown` variants, returns the stored character.
    pub fn to_char(self) -> char {
        match self {
            Self::Space => ' ',
            Self::DoubleSpace => '0',
            Self::TripleSpace => '-',
            Self::PageEject => '1',
            Self::Overstrike => '+',
            Self::Halt => 'H',
            Self::Unknown(ch) => ch,
        }
    }

    /// Number of blank lines to insert before this line's content.
    ///
    /// - Space / Overstrike / PageEject / Halt / Unknown → 0
    /// - DoubleSpace → 1
    /// - TripleSpace → 2
    // Validates: Requirement 1.2, 1.3, 1.4
    pub fn spacing_lines(self) -> u8 {
        match self {
            Self::DoubleSpace => 1,
            Self::TripleSpace => 2,
            _ => 0,
        }
    }

    /// Whether this control starts a new page.
    pub fn is_page_break(self) -> bool {
        matches!(self, Self::PageEject)
    }

    /// Whether this control indicates overstrike merging.
    pub fn is_overstrike(self) -> bool {
        matches!(self, Self::Overstrike)
    }

    /// Whether this is a recognised (valid) ASA control character.
    pub fn is_known(self) -> bool {
        !matches!(self, Self::Unknown(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Validates: Requirement 1.1
    fn from_char_parses_all_valid_asa_characters() {
        assert_eq!(AsaControl::from_char(' '), AsaControl::Space);
        assert_eq!(AsaControl::from_char('0'), AsaControl::DoubleSpace);
        assert_eq!(AsaControl::from_char('-'), AsaControl::TripleSpace);
        assert_eq!(AsaControl::from_char('1'), AsaControl::PageEject);
        assert_eq!(AsaControl::from_char('+'), AsaControl::Overstrike);
        assert_eq!(AsaControl::from_char('H'), AsaControl::Halt);
    }

    #[test]
    // Validates: Requirement 1.9
    fn from_char_returns_unknown_for_unrecognised_characters() {
        assert_eq!(AsaControl::from_char('A'), AsaControl::Unknown('A'));
        assert_eq!(AsaControl::from_char('X'), AsaControl::Unknown('X'));
        assert_eq!(AsaControl::from_char('\t'), AsaControl::Unknown('\t'));
        assert_eq!(AsaControl::from_char('2'), AsaControl::Unknown('2'));
    }

    #[test]
    // Validates: Requirement 1.1
    fn to_char_round_trips_known_controls() {
        for &ch in ASA_VALID_CHARS {
            let control = AsaControl::from_char(ch);
            assert_eq!(control.to_char(), ch);
        }
    }

    #[test]
    // Validates: Requirement 1.2, 1.3, 1.4
    fn spacing_lines_returns_correct_counts() {
        assert_eq!(AsaControl::Space.spacing_lines(), 0);
        assert_eq!(AsaControl::DoubleSpace.spacing_lines(), 1);
        assert_eq!(AsaControl::TripleSpace.spacing_lines(), 2);
        assert_eq!(AsaControl::PageEject.spacing_lines(), 0);
        assert_eq!(AsaControl::Overstrike.spacing_lines(), 0);
        assert_eq!(AsaControl::Halt.spacing_lines(), 0);
        assert_eq!(AsaControl::Unknown('X').spacing_lines(), 0);
    }

    #[test]
    fn is_page_break_only_for_page_eject() {
        assert!(AsaControl::PageEject.is_page_break());
        assert!(!AsaControl::Space.is_page_break());
        assert!(!AsaControl::DoubleSpace.is_page_break());
        assert!(!AsaControl::Overstrike.is_page_break());
    }

    #[test]
    fn is_overstrike_only_for_overstrike() {
        assert!(AsaControl::Overstrike.is_overstrike());
        assert!(!AsaControl::Space.is_overstrike());
        assert!(!AsaControl::PageEject.is_overstrike());
    }

    #[test]
    fn is_known_returns_false_for_unknown() {
        assert!(AsaControl::Space.is_known());
        assert!(AsaControl::PageEject.is_known());
        assert!(!AsaControl::Unknown('X').is_known());
    }
}
