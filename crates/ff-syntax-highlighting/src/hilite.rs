//! HILITE command state and operand parsing.
//!
//! Implements per-document HILITE ON/OFF toggle, LOGIC operator highlighting,
//! PAREN delimiter matching, and FIND persistent highlights.
//!
//! Addresses: Requirement 16 (AC 16.1-16.5)

// === HiliteMode flags =====================================================

/// Bitmask of active HILITE overlay modes for a single document.
///
/// Multiple modes may be active simultaneously (Requirement 16, AC 16.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HiliteModes(u8);

impl HiliteModes {
    /// No overlay modes active.
    pub const NONE: Self = Self(0);
    /// HILITE LOGIC -- boolean/comparison operator highlighting.
    pub const LOGIC: Self = Self(1 << 0);
    /// HILITE PAREN -- enclosing delimiter-pair highlighting.
    pub const PAREN: Self = Self(1 << 1);
    /// HILITE FIND -- persistent find-match highlighting.
    pub const FIND: Self = Self(1 << 2);

    /// Returns true if the given mode flag is active.
    pub fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }

    /// Enable a mode flag.
    pub fn insert(&mut self, flag: Self) {
        self.0 |= flag.0;
    }

    /// Disable a mode flag.
    pub fn remove(&mut self, flag: Self) {
        self.0 &= !flag.0;
    }
}

// === HiliteOperand ========================================================

/// Parsed operand(s) from a HILITE command.
///
/// Addresses: Requirement 16, AC 16.1-16.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HiliteOperand {
    /// HILITE ON [LOGIC] [PAREN] [FIND] -- enable highlighting with optional modes.
    On { modes: HiliteModes },
    /// HILITE OFF -- disable all highlighting and clear all modes.
    Off,
    /// HILITE LOGIC -- toggle LOGIC mode (syntax highlighting must be ON).
    Logic,
    /// HILITE PAREN -- toggle PAREN mode.
    Paren,
    /// HILITE FIND [OFF] -- toggle FIND mode; OFF clears persistent find highlights.
    Find { off: bool },
}

impl HiliteOperand {
    /// Parse a HILITE command operand string (the part after "HILITE").
    ///
    /// Accepts: ON, OFF, LOGIC, PAREN, FIND, FIND OFF,
    /// and combined: ON LOGIC PAREN, ON LOGIC, ON PAREN, etc.
    ///
    /// Returns `None` if the operand string is unrecognised.
    pub fn parse(operands: &str) -> Option<Self> {
        let mut tokens: Vec<&str> = operands.split_whitespace().collect();

        if tokens.is_empty() {
            return None;
        }

        match tokens[0].to_uppercase().as_str() {
            "OFF" => Some(HiliteOperand::Off),
            "LOGIC" => Some(HiliteOperand::Logic),
            "PAREN" => Some(HiliteOperand::Paren),
            "FIND" => {
                let off = tokens
                    .get(1)
                    .map(|t| t.to_uppercase() == "OFF")
                    .unwrap_or(false);
                Some(HiliteOperand::Find { off })
            }
            "ON" => {
                // Collect any additional mode tokens after ON
                let mut modes = HiliteModes::NONE;
                tokens.remove(0);
                for token in &tokens {
                    match token.to_uppercase().as_str() {
                        "LOGIC" => modes.insert(HiliteModes::LOGIC),
                        "PAREN" => modes.insert(HiliteModes::PAREN),
                        "FIND" => modes.insert(HiliteModes::FIND),
                        _ => return None,
                    }
                }
                Some(HiliteOperand::On { modes })
            }
            _ => None,
        }
    }
}

// === HiliteState ==========================================================

/// Per-document HILITE state: ON/OFF flag, active overlay modes, and FIND string.
///
/// Addresses: Requirement 16, AC 16.1-16.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HiliteState {
    /// Whether syntax highlighting is enabled for this document.
    pub enabled: bool,
    /// Active overlay modes (LOGIC, PAREN, FIND).
    pub modes: HiliteModes,
    /// The most recent FIND string for persistent find highlights.
    /// None when HILITE FIND is not active or has been cleared.
    pub find_string: Option<String>,
}

impl Default for HiliteState {
    fn default() -> Self {
        Self {
            enabled: true,
            modes: HiliteModes::NONE,
            find_string: None,
        }
    }
}

impl HiliteState {
    /// Create a new HiliteState with highlighting enabled and no active modes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a parsed HILITE operand to this state.
    ///
    /// Addresses: Requirement 16, AC 16.1-16.5
    pub fn apply(&mut self, operand: &HiliteOperand) {
        match operand {
            HiliteOperand::On { modes } => {
                self.enabled = true;
                // Enable any modes specified alongside ON
                if modes.contains(HiliteModes::LOGIC) {
                    self.modes.insert(HiliteModes::LOGIC);
                }
                if modes.contains(HiliteModes::PAREN) {
                    self.modes.insert(HiliteModes::PAREN);
                }
                if modes.contains(HiliteModes::FIND) {
                    self.modes.insert(HiliteModes::FIND);
                }
            }
            HiliteOperand::Off => {
                // OFF clears all modes and disables highlighting
                self.enabled = false;
                self.modes = HiliteModes::NONE;
                self.find_string = None;
            }
            HiliteOperand::Logic => {
                // Toggle LOGIC mode independently
                if self.modes.contains(HiliteModes::LOGIC) {
                    self.modes.remove(HiliteModes::LOGIC);
                } else {
                    self.modes.insert(HiliteModes::LOGIC);
                }
            }
            HiliteOperand::Paren => {
                // Toggle PAREN mode independently
                if self.modes.contains(HiliteModes::PAREN) {
                    self.modes.remove(HiliteModes::PAREN);
                } else {
                    self.modes.insert(HiliteModes::PAREN);
                }
            }
            HiliteOperand::Find { off } => {
                if *off {
                    // HILITE FIND OFF clears persistent find highlights
                    self.modes.remove(HiliteModes::FIND);
                    self.find_string = None;
                } else {
                    // Toggle FIND mode independently
                    if self.modes.contains(HiliteModes::FIND) {
                        self.modes.remove(HiliteModes::FIND);
                        self.find_string = None;
                    } else {
                        self.modes.insert(HiliteModes::FIND);
                    }
                }
            }
        }
    }

    /// Set the persistent FIND string (called when a FIND command is executed
    /// while HILITE FIND mode is active).
    ///
    /// Addresses: Requirement 16, AC 16.4
    pub fn set_find_string(&mut self, s: &str) {
        if self.modes.contains(HiliteModes::FIND) {
            self.find_string = Some(s.to_string());
        }
    }
}

// === HiliteLogicScanner ===================================================

/// Scans a line of text and returns byte ranges of logic/comparison operators.
///
/// Detected operators: `&&`, `||`, `!`, `AND`, `OR`, `NOT`,
/// `==`, `!=`, `<`, `>`, `<=`, `>=`.
///
/// Addresses: Requirement 16, AC 16.2
pub struct HiliteLogicScanner;

impl HiliteLogicScanner {
    /// Scan `text` and return a list of `(start, end)` byte ranges for each
    /// logic or comparison operator found.
    pub fn scan(text: &str) -> Vec<(usize, usize)> {
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut spans: Vec<(usize, usize)> = Vec::new();
        let mut i = 0;

        while i < len {
            // Two-character operators first
            if i + 1 < len {
                match (bytes[i], bytes[i + 1]) {
                    (b'&', b'&') => {
                        spans.push((i, i + 2));
                        i += 2;
                        continue;
                    }
                    (b'|', b'|') => {
                        spans.push((i, i + 2));
                        i += 2;
                        continue;
                    }
                    (b'!', b'=') => {
                        spans.push((i, i + 2));
                        i += 2;
                        continue;
                    }
                    (b'<', b'=') => {
                        spans.push((i, i + 2));
                        i += 2;
                        continue;
                    }
                    (b'>', b'=') => {
                        spans.push((i, i + 2));
                        i += 2;
                        continue;
                    }
                    (b'=', b'=') => {
                        spans.push((i, i + 2));
                        i += 2;
                        continue;
                    }
                    _ => {}
                }
            }
            // Single-character operators (not part of a two-char sequence)
            match bytes[i] {
                b'!' | b'<' | b'>' => {
                    spans.push((i, i + 1));
                    i += 1;
                    continue;
                }
                _ => {}
            }
            // Word operators: AND, OR, NOT (case-insensitive, whole-word only)
            if bytes[i].is_ascii_alphabetic() {
                let word_start = i;
                while i < len && bytes[i].is_ascii_alphabetic() {
                    i += 1;
                }
                let word = &text[word_start..i];
                let before_ok = word_start == 0 || !bytes[word_start - 1].is_ascii_alphanumeric();
                let after_ok = i >= len || !bytes[i].is_ascii_alphanumeric();
                if before_ok && after_ok {
                    match word.to_uppercase().as_str() {
                        "AND" | "OR" | "NOT" => {
                            spans.push((word_start, i));
                        }
                        _ => {}
                    }
                }
                continue;
            }
            i += 1;
        }
        spans
    }
}

// === HiliteParenMatcher ===================================================

/// Result of a parenthesis/bracket/brace matching operation.
///
/// Addresses: Requirement 16, AC 16.3
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParenMatchResult {
    /// A matched pair was found: open byte offset and close byte offset.
    Matched { open: usize, close: usize },
    /// An unmatched delimiter was found at the given byte offset.
    Mismatched { position: usize },
    /// The cursor is not inside any delimiter pair.
    None,
}

/// Finds the innermost enclosing delimiter pair around a cursor position.
///
/// Supported delimiter pairs: `()`, `[]`, `{}`.
///
/// Addresses: Requirement 16, AC 16.3
pub struct HiliteParenMatcher;

impl HiliteParenMatcher {
    /// Find the innermost enclosing delimiter pair containing `cursor_byte`.
    ///
    /// Returns `ParenMatchResult::Matched` when a balanced pair is found,
    /// `ParenMatchResult::Mismatched` when an unmatched opener is found,
    /// or `ParenMatchResult::None` when the cursor is not inside any pair.
    pub fn find_enclosing(text: &str, cursor_byte: usize) -> ParenMatchResult {
        let bytes = text.as_bytes();
        let len = bytes.len();
        let cursor = cursor_byte.min(len);

        // Scan backwards from cursor to find the nearest unmatched opener
        let mut depth: i32 = 0;
        let mut open_pos: Option<usize> = None;
        let mut open_ch: u8 = 0;

        let mut i = cursor;
        while i > 0 {
            i -= 1;
            match bytes[i] {
                b')' | b']' | b'}' => depth += 1,
                b'(' | b'[' | b'{' => {
                    if depth == 0 {
                        open_pos = Some(i);
                        open_ch = bytes[i];
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }

        let open_pos = match open_pos {
            Some(p) => p,
            None => return ParenMatchResult::None,
        };

        let expected_close: u8 = match open_ch {
            b'(' => b')',
            b'[' => b']',
            b'{' => b'}',
            _ => return ParenMatchResult::None,
        };

        // Scan forward from open_pos to find the matching closer
        let mut depth: i32 = 0;
        let mut j = open_pos;
        while j < len {
            if bytes[j] == open_ch {
                depth += 1;
            } else if bytes[j] == expected_close {
                depth -= 1;
                if depth == 0 {
                    return ParenMatchResult::Matched {
                        open: open_pos,
                        close: j,
                    };
                }
            }
            j += 1;
        }

        // Opener found but no matching closer
        ParenMatchResult::Mismatched { position: open_pos }
    }
}

// === Tests ================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- HiliteOperand::parse ---

    #[test]
    fn hilite_operand_parse_on() {
        // Validates: Requirement 16.1
        let op = HiliteOperand::parse("ON").unwrap();
        assert_eq!(
            op,
            HiliteOperand::On {
                modes: HiliteModes::NONE
            }
        );
    }

    #[test]
    fn hilite_operand_parse_off() {
        // Validates: Requirement 16.1
        let op = HiliteOperand::parse("OFF").unwrap();
        assert_eq!(op, HiliteOperand::Off);
    }

    #[test]
    fn hilite_operand_parse_logic() {
        // Validates: Requirement 16.2
        let op = HiliteOperand::parse("LOGIC").unwrap();
        assert_eq!(op, HiliteOperand::Logic);
    }

    #[test]
    fn hilite_operand_parse_paren() {
        // Validates: Requirement 16.3
        let op = HiliteOperand::parse("PAREN").unwrap();
        assert_eq!(op, HiliteOperand::Paren);
    }

    #[test]
    fn hilite_operand_parse_find() {
        // Validates: Requirement 16.4
        let op = HiliteOperand::parse("FIND").unwrap();
        assert_eq!(op, HiliteOperand::Find { off: false });
    }

    #[test]
    fn hilite_operand_parse_find_off() {
        // Validates: Requirement 16.4
        let op = HiliteOperand::parse("FIND OFF").unwrap();
        assert_eq!(op, HiliteOperand::Find { off: true });
    }

    #[test]
    fn hilite_operand_parse_on_logic_paren() {
        // Validates: Requirement 16.5
        let op = HiliteOperand::parse("ON LOGIC PAREN").unwrap();
        let mut expected = HiliteModes::NONE;
        expected.insert(HiliteModes::LOGIC);
        expected.insert(HiliteModes::PAREN);
        assert_eq!(op, HiliteOperand::On { modes: expected });
    }

    #[test]
    fn hilite_operand_parse_on_logic() {
        // Validates: Requirement 16.5
        let op = HiliteOperand::parse("ON LOGIC").unwrap();
        let mut expected = HiliteModes::NONE;
        expected.insert(HiliteModes::LOGIC);
        assert_eq!(op, HiliteOperand::On { modes: expected });
    }

    #[test]
    fn hilite_operand_parse_case_insensitive() {
        // Validates: Requirement 16.1
        assert!(HiliteOperand::parse("on").is_some());
        assert!(HiliteOperand::parse("off").is_some());
        assert!(HiliteOperand::parse("logic").is_some());
    }

    #[test]
    fn hilite_operand_parse_unknown_returns_none() {
        assert!(HiliteOperand::parse("BOGUS").is_none());
        assert!(HiliteOperand::parse("").is_none());
    }

    // --- HiliteState::apply ---

    #[test]
    fn hilite_state_on_enables_highlighting() {
        // Validates: Requirement 16.1
        let mut state = HiliteState::new();
        state.enabled = false;
        state.apply(&HiliteOperand::On {
            modes: HiliteModes::NONE,
        });
        assert!(state.enabled);
    }

    #[test]
    fn hilite_state_off_disables_highlighting_and_clears_modes() {
        // Validates: Requirement 16.1
        let mut state = HiliteState::new();
        state.modes.insert(HiliteModes::LOGIC);
        state.modes.insert(HiliteModes::PAREN);
        state.find_string = Some("foo".to_string());
        state.apply(&HiliteOperand::Off);
        assert!(!state.enabled);
        assert_eq!(state.modes, HiliteModes::NONE);
        assert!(state.find_string.is_none());
    }

    #[test]
    fn hilite_state_logic_toggles_independently() {
        // Validates: Requirement 16.2, 16.5
        let mut state = HiliteState::new();
        assert!(!state.modes.contains(HiliteModes::LOGIC));
        state.apply(&HiliteOperand::Logic);
        assert!(state.modes.contains(HiliteModes::LOGIC));
        state.apply(&HiliteOperand::Logic);
        assert!(!state.modes.contains(HiliteModes::LOGIC));
    }

    #[test]
    fn hilite_state_paren_toggles_independently() {
        // Validates: Requirement 16.3, 16.5
        let mut state = HiliteState::new();
        state.apply(&HiliteOperand::Paren);
        assert!(state.modes.contains(HiliteModes::PAREN));
        // LOGIC unaffected
        assert!(!state.modes.contains(HiliteModes::LOGIC));
        state.apply(&HiliteOperand::Paren);
        assert!(!state.modes.contains(HiliteModes::PAREN));
    }

    #[test]
    fn hilite_state_find_off_clears_find_string() {
        // Validates: Requirement 16.4
        let mut state = HiliteState::new();
        state.modes.insert(HiliteModes::FIND);
        state.find_string = Some("hello".to_string());
        state.apply(&HiliteOperand::Find { off: true });
        assert!(!state.modes.contains(HiliteModes::FIND));
        assert!(state.find_string.is_none());
    }

    #[test]
    fn hilite_state_on_logic_paren_enables_both_modes() {
        // Validates: Requirement 16.5
        let mut state = HiliteState::new();
        let mut modes = HiliteModes::NONE;
        modes.insert(HiliteModes::LOGIC);
        modes.insert(HiliteModes::PAREN);
        state.apply(&HiliteOperand::On { modes });
        assert!(state.enabled);
        assert!(state.modes.contains(HiliteModes::LOGIC));
        assert!(state.modes.contains(HiliteModes::PAREN));
    }

    #[test]
    fn hilite_state_modes_toggle_independently_after_on() {
        // Validates: Requirement 16.5 -- modes toggle independently
        let mut state = HiliteState::new();
        let mut modes = HiliteModes::NONE;
        modes.insert(HiliteModes::LOGIC);
        modes.insert(HiliteModes::PAREN);
        state.apply(&HiliteOperand::On { modes });
        // Toggle LOGIC off -- PAREN unaffected
        state.apply(&HiliteOperand::Logic);
        assert!(!state.modes.contains(HiliteModes::LOGIC));
        assert!(state.modes.contains(HiliteModes::PAREN));
        assert!(state.enabled);
    }

    #[test]
    fn hilite_state_set_find_string_only_when_find_active() {
        // Validates: Requirement 16.4
        let mut state = HiliteState::new();
        // FIND mode not active -- set_find_string is a no-op
        state.set_find_string("hello");
        assert!(state.find_string.is_none());
        // Enable FIND mode
        state.modes.insert(HiliteModes::FIND);
        state.set_find_string("hello");
        assert_eq!(state.find_string.as_deref(), Some("hello"));
    }

    // --- HiliteLogicScanner ---

    #[test]
    fn logic_scanner_detects_double_ampersand() {
        // Validates: Requirement 16.2
        let spans = HiliteLogicScanner::scan("a && b");
        assert!(spans.contains(&(2, 4)));
    }

    #[test]
    fn logic_scanner_detects_double_pipe() {
        // Validates: Requirement 16.2
        let spans = HiliteLogicScanner::scan("x || y");
        assert!(spans.contains(&(2, 4)));
    }

    #[test]
    fn logic_scanner_detects_comparison_operators() {
        // Validates: Requirement 16.2
        let spans = HiliteLogicScanner::scan("a == b != c <= d >= e");
        let starts: Vec<usize> = spans.iter().map(|&(s, _)| s).collect();
        assert!(starts.contains(&2)); // ==
        assert!(starts.contains(&7)); // !=
        assert!(starts.contains(&12)); // <=
        assert!(starts.contains(&17)); // >=
    }

    #[test]
    fn logic_scanner_detects_word_operators() {
        // Validates: Requirement 16.2
        let spans = HiliteLogicScanner::scan("x AND y OR z NOT w");
        assert_eq!(spans.len(), 3);
    }

    #[test]
    fn logic_scanner_word_operators_whole_word_only() {
        // Validates: Requirement 16.2 -- ANDROID should not match AND
        let spans = HiliteLogicScanner::scan("ANDROID");
        assert!(spans.is_empty());
    }

    #[test]
    fn logic_scanner_empty_text_returns_empty() {
        assert!(HiliteLogicScanner::scan("").is_empty());
    }

    // --- HiliteParenMatcher ---

    #[test]
    fn paren_matcher_finds_enclosing_parens() {
        // Validates: Requirement 16.3
        let text = "foo(bar)baz";
        // cursor inside the parens
        let result = HiliteParenMatcher::find_enclosing(text, 5);
        assert_eq!(result, ParenMatchResult::Matched { open: 3, close: 7 });
    }

    #[test]
    fn paren_matcher_finds_enclosing_brackets() {
        // Validates: Requirement 16.3
        let text = "arr[idx]";
        let result = HiliteParenMatcher::find_enclosing(text, 5);
        assert_eq!(result, ParenMatchResult::Matched { open: 3, close: 7 });
    }

    #[test]
    fn paren_matcher_finds_enclosing_braces() {
        // Validates: Requirement 16.3
        let text = "fn { body }";
        let result = HiliteParenMatcher::find_enclosing(text, 6);
        assert_eq!(result, ParenMatchResult::Matched { open: 3, close: 10 });
    }

    #[test]
    fn paren_matcher_returns_none_outside_any_pair() {
        // Validates: Requirement 16.3
        let text = "hello world";
        let result = HiliteParenMatcher::find_enclosing(text, 5);
        assert_eq!(result, ParenMatchResult::None);
    }

    #[test]
    fn paren_matcher_returns_mismatched_for_unclosed_opener() {
        // Validates: Requirement 16.3 -- HILITE_PAREN_ERROR style
        let text = "foo(bar";
        let result = HiliteParenMatcher::find_enclosing(text, 5);
        assert_eq!(result, ParenMatchResult::Mismatched { position: 3 });
    }

    #[test]
    fn paren_matcher_finds_innermost_pair() {
        // Validates: Requirement 16.3 -- nested parens
        let text = "outer(inner(x)y)z";
        // cursor at 'x' (position 12)
        let result = HiliteParenMatcher::find_enclosing(text, 12);
        assert_eq!(
            result,
            ParenMatchResult::Matched {
                open: 11,
                close: 13
            }
        );
    }
}
