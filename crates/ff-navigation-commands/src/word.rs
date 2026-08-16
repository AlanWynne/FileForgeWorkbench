//! Word navigation implementation.
//!
//! Moves the caret by word boundaries using character class transitions.
//! Supports line-boundary crossing and document boundary clamping.

use crate::char_class::{CharClassifier, CharacterClass};
use crate::types::SelectionModifier;

/// Word navigation executor.
pub struct WordNav;

impl WordNav {
    /// Move caret to the start of the previous word (word-left).
    ///
    /// Skips whitespace backwards, then skips same-class characters backwards
    /// until a class transition is reached. Crosses line boundaries.
    ///
    /// Returns the new (line, column) position (both 1-based).
    pub fn word_left(
        lines: &[&str],
        line: u64,
        column: u64,
        classifier: &CharClassifier,
        _selection: SelectionModifier,
    ) -> (u64, u64) {
        let total_lines = lines.len() as u64;
        if total_lines == 0 {
            return (1, 1);
        }

        let mut cur_line = line.min(total_lines);
        let mut cur_col = column; // 1-based

        // Get the current line content
        let get_line =
            |l: u64| -> &str { lines.get((l as usize).saturating_sub(1)).unwrap_or(&"") };

        let line_content = get_line(cur_line);
        let chars: Vec<char> = line_content.chars().collect();

        // Convert to 0-based position in the character array
        let mut pos = (cur_col as usize).saturating_sub(1).min(chars.len());

        // If at start of line, move to end of previous line
        if pos == 0 {
            if cur_line <= 1 {
                return (1, 1);
            }
            cur_line -= 1;
            let prev_chars: Vec<char> = get_line(cur_line).chars().collect();
            return (cur_line, (prev_chars.len() + 1) as u64);
        }

        // Skip whitespace backwards
        while pos > 0 && classifier.classify(chars[pos - 1]) == CharacterClass::Space {
            pos -= 1;
        }

        // If we've gone to start of line after skipping space, check previous line
        if pos == 0 {
            if cur_line <= 1 {
                return (1, 1);
            }
            cur_line -= 1;
            let prev_chars: Vec<char> = get_line(cur_line).chars().collect();
            pos = prev_chars.len();

            // Skip whitespace at end of previous line
            while pos > 0 && classifier.classify(prev_chars[pos - 1]) == CharacterClass::Space {
                pos -= 1;
            }

            if pos == 0 {
                return (cur_line, 1);
            }

            // Skip same-class chars backwards
            let target_class = classifier.classify(prev_chars[pos - 1]);
            while pos > 0 && classifier.classify(prev_chars[pos - 1]) == target_class {
                pos -= 1;
            }

            cur_col = (pos + 1) as u64;
            return (cur_line, cur_col);
        }

        // Skip same-class characters backwards
        let target_class = classifier.classify(chars[pos - 1]);
        while pos > 0 && classifier.classify(chars[pos - 1]) == target_class {
            pos -= 1;
        }

        cur_col = (pos + 1) as u64;
        (cur_line, cur_col)
    }

    /// Move caret to the start of the next word (word-right).
    ///
    /// Skips current-class characters forwards, then skips whitespace forwards
    /// until a non-whitespace character is reached. Crosses line boundaries.
    ///
    /// Returns the new (line, column) position (both 1-based).
    pub fn word_right(
        lines: &[&str],
        line: u64,
        column: u64,
        classifier: &CharClassifier,
        _selection: SelectionModifier,
    ) -> (u64, u64) {
        let total_lines = lines.len() as u64;
        if total_lines == 0 {
            return (1, 1);
        }

        let mut cur_line = line.min(total_lines);

        let get_line =
            |l: u64| -> &str { lines.get((l as usize).saturating_sub(1)).unwrap_or(&"") };

        let line_content = get_line(cur_line);
        let chars: Vec<char> = line_content.chars().collect();
        let mut pos = (column as usize).saturating_sub(1).min(chars.len());

        // Skip current-class characters forwards
        if pos < chars.len() {
            let current_class = classifier.classify(chars[pos]);
            while pos < chars.len() && classifier.classify(chars[pos]) == current_class {
                pos += 1;
            }
        }

        // Skip whitespace forwards
        while pos < chars.len() && classifier.classify(chars[pos]) == CharacterClass::Space {
            pos += 1;
        }

        // If at end of line, cross to next line
        if pos >= chars.len() {
            if cur_line >= total_lines {
                // At document end
                let last_chars: Vec<char> = get_line(total_lines).chars().collect();
                return (total_lines, (last_chars.len() + 1) as u64);
            }
            cur_line += 1;
            let next_chars: Vec<char> = get_line(cur_line).chars().collect();

            // Skip whitespace at start of next line
            pos = 0;
            while pos < next_chars.len()
                && classifier.classify(next_chars[pos]) == CharacterClass::Space
            {
                pos += 1;
            }

            return (cur_line, (pos + 1) as u64);
        }

        (cur_line, (pos + 1) as u64)
    }

    /// Move caret to the end of the current or next word (word-end-right).
    ///
    /// Skips whitespace forwards, then skips word characters forwards
    /// until a class transition is reached.
    ///
    /// Returns the new (line, column) position (both 1-based).
    pub fn word_end_right(
        lines: &[&str],
        line: u64,
        column: u64,
        classifier: &CharClassifier,
        _selection: SelectionModifier,
    ) -> (u64, u64) {
        let total_lines = lines.len() as u64;
        if total_lines == 0 {
            return (1, 1);
        }

        let mut cur_line = line.min(total_lines);

        let get_line =
            |l: u64| -> &str { lines.get((l as usize).saturating_sub(1)).unwrap_or(&"") };

        let line_content = get_line(cur_line);
        let chars: Vec<char> = line_content.chars().collect();
        let mut pos = (column as usize).saturating_sub(1).min(chars.len());

        // Advance at least one position
        if pos < chars.len() {
            pos += 1;
        }

        // Skip whitespace forwards
        while pos < chars.len() && classifier.classify(chars[pos]) == CharacterClass::Space {
            pos += 1;
        }

        // If at end of line, cross to next line
        if pos >= chars.len() {
            if cur_line >= total_lines {
                let last_chars: Vec<char> = get_line(total_lines).chars().collect();
                return (total_lines, (last_chars.len() + 1) as u64);
            }
            cur_line += 1;
            let next_chars: Vec<char> = get_line(cur_line).chars().collect();
            pos = 0;

            // Skip whitespace at start of next line
            while pos < next_chars.len()
                && classifier.classify(next_chars[pos]) == CharacterClass::Space
            {
                pos += 1;
            }

            if pos >= next_chars.len() {
                return (cur_line, (next_chars.len() + 1) as u64);
            }

            // Skip same-class forwards to end of word
            let target_class = classifier.classify(next_chars[pos]);
            while pos < next_chars.len() && classifier.classify(next_chars[pos]) == target_class {
                pos += 1;
            }

            return (cur_line, (pos + 1) as u64);
        }

        // Skip same-class forwards to end of word
        let target_class = classifier.classify(chars[pos]);
        while pos < chars.len() && classifier.classify(chars[pos]) == target_class {
            pos += 1;
        }

        (cur_line, (pos + 1) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_left_from_middle_of_word() {
        // Validates: Requirement 7.2
        let classifier = CharClassifier::new();
        let lines = vec!["hello world"];
        // Position at 'o' in "world" (column 8)
        let (line, col) = WordNav::word_left(&lines, 1, 8, &classifier, SelectionModifier::Move);
        assert_eq!(line, 1);
        assert_eq!(col, 7); // Start of "world"
    }

    #[test]
    fn word_left_from_start_of_word_skips_to_previous() {
        // Validates: Requirement 7.2
        let classifier = CharClassifier::new();
        let lines = vec!["hello world"];
        // Position at start of "world" (column 7)
        let (line, col) = WordNav::word_left(&lines, 1, 7, &classifier, SelectionModifier::Move);
        assert_eq!(line, 1);
        assert_eq!(col, 1); // Start of "hello"
    }

    #[test]
    fn word_right_from_middle_of_word() {
        // Validates: Requirement 7.3
        let classifier = CharClassifier::new();
        let lines = vec!["hello world"];
        // Position at 'e' in "hello" (column 2)
        let (line, col) = WordNav::word_right(&lines, 1, 2, &classifier, SelectionModifier::Move);
        assert_eq!(line, 1);
        assert_eq!(col, 7); // Start of "world"
    }

    #[test]
    fn word_right_crosses_line_boundary() {
        // Validates: Requirement 7.5
        let classifier = CharClassifier::new();
        let lines = vec!["hello", "world"];
        // Position at end of first line (column 6, past 'o')
        let (line, col) = WordNav::word_right(&lines, 1, 5, &classifier, SelectionModifier::Move);
        assert_eq!(line, 2);
        assert_eq!(col, 1); // Start of "world" on line 2
    }

    #[test]
    fn word_left_crosses_line_boundary() {
        // Validates: Requirement 7.5
        let classifier = CharClassifier::new();
        let lines = vec!["hello", "world"];
        // Position at start of line 2 (column 1)
        let (line, col) = WordNav::word_left(&lines, 2, 1, &classifier, SelectionModifier::Move);
        assert_eq!(line, 1);
        assert_eq!(col, 6); // End of "hello" on line 1
    }

    #[test]
    fn word_left_at_document_start_clamps() {
        // Validates: Requirement 7.6
        let classifier = CharClassifier::new();
        let lines = vec!["hello"];
        let (line, col) = WordNav::word_left(&lines, 1, 1, &classifier, SelectionModifier::Move);
        assert_eq!(line, 1);
        assert_eq!(col, 1);
    }

    #[test]
    fn word_right_at_document_end_clamps() {
        // Validates: Requirement 7.7
        let classifier = CharClassifier::new();
        let lines = vec!["hello"];
        let (line, col) = WordNav::word_right(&lines, 1, 5, &classifier, SelectionModifier::Move);
        assert_eq!(line, 1);
        assert_eq!(col, 6); // Past end of line
    }

    #[test]
    fn word_end_right_basic() {
        // Validates: Requirement 7.4
        let classifier = CharClassifier::new();
        let lines = vec!["hello world"];
        // Position at 'h' (column 1)
        let (line, col) =
            WordNav::word_end_right(&lines, 1, 1, &classifier, SelectionModifier::Move);
        assert_eq!(line, 1);
        assert_eq!(col, 6); // Past end of "hello"
    }

    #[test]
    fn word_navigation_with_punctuation() {
        // Validates: Requirement 7.1
        let classifier = CharClassifier::new();
        let lines = vec!["foo.bar + baz"];
        // From column 1 ('f'), word_right skips "foo" then the dot is punctuation
        let (line, col) = WordNav::word_right(&lines, 1, 1, &classifier, SelectionModifier::Move);
        assert_eq!(line, 1);
        assert_eq!(col, 4); // Lands on '.'
    }
}
