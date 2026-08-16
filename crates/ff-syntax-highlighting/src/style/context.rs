//! StyleContext: helper structure for lexer authors providing convenient
//! character access, state tracking, and style assignment methods.

use crate::keywords::word_list::WordList;
use crate::types::{BytePosition, KeywordSetIndex, LexerState, StyleSlotIndex};

/// Helper structure for lexer authors providing convenient character access,
/// state tracking, and style assignment methods.
/// Addresses: Requirement 14
pub struct StyleContext<'a> {
    /// The text being styled.
    text: &'a str,
    /// Current byte position in the text.
    position: usize,
    /// Start position of the current token.
    token_start: usize,
    /// End position of the styling range.
    end: usize,
    /// Current lexer state.
    state: LexerState,
    /// Style assigned to current token region.
    current_style: StyleSlotIndex,
    /// Mutable reference to the style buffer data for this range.
    style_data: &'a mut Vec<u8>,
    /// Offset into the style buffer where our text starts.
    style_offset: usize,
}

impl<'a> StyleContext<'a> {
    /// Create a new StyleContext for the given text range.
    pub fn new(
        text: &'a str,
        start: usize,
        end: usize,
        initial_state: LexerState,
        style_data: &'a mut Vec<u8>,
        style_offset: usize,
    ) -> Self {
        Self {
            text,
            position: start,
            token_start: start,
            end: end.min(text.len()),
            state: initial_state,
            current_style: StyleSlotIndex::DEFAULT,
            style_data,
            style_offset,
        }
    }

    /// Get the current character.
    /// Addresses: Requirement 14, criterion 14.1
    pub fn ch(&self) -> char {
        if self.position >= self.text.len() {
            return '\0';
        }
        self.text[self.position..].chars().next().unwrap_or('\0')
    }

    /// Get the next character (lookahead). Returns '\0' at document end.
    /// Addresses: Requirement 14, criteria 14.1, 14.9
    pub fn ch_next(&self) -> char {
        if self.position >= self.text.len() {
            return '\0';
        }
        let current_char = self.text[self.position..].chars().next().unwrap_or('\0');
        let next_pos = self.position + current_char.len_utf8();
        if next_pos >= self.text.len() {
            return '\0';
        }
        self.text[next_pos..].chars().next().unwrap_or('\0')
    }

    /// Get the previous character. Returns '\0' at document start.
    /// Addresses: Requirement 14, criterion 14.1
    pub fn ch_prev(&self) -> char {
        if self.position == 0 {
            return '\0';
        }
        self.text[..self.position]
            .chars()
            .next_back()
            .unwrap_or('\0')
    }

    /// Get the current lexer state.
    /// Addresses: Requirement 14, criterion 14.1
    pub fn state(&self) -> LexerState {
        self.state
    }

    /// Get the byte position of the current token start.
    /// Addresses: Requirement 14, criterion 14.1
    pub fn start_position(&self) -> BytePosition {
        BytePosition(self.token_start)
    }

    /// Get the current byte position.
    pub fn current_position(&self) -> BytePosition {
        BytePosition(self.position)
    }

    /// Assign style to characters from token start to current position,
    /// then transition to new_state. The token start is reset to the current position.
    /// Addresses: Requirement 14, criterion 14.2
    pub fn set_state(&mut self, new_state: LexerState) {
        self.state = new_state;
        self.token_start = self.position;
    }

    /// Set the style for the current token being built.
    pub fn set_style(&mut self, style: StyleSlotIndex) {
        self.current_style = style;
    }

    /// Advance position by one character (handles multi-byte UTF-8).
    /// Addresses: Requirement 14, criterion 14.3
    pub fn forward(&mut self) {
        if self.position < self.text.len() {
            // Write current_style for the current character position
            let buf_pos = self.position.saturating_sub(self.style_offset);
            if buf_pos < self.style_data.len() {
                self.style_data[buf_pos] = self.current_style.0;
            }
            let ch = self.text[self.position..].chars().next().unwrap_or('\0');
            self.position += ch.len_utf8();
        }
    }

    /// Advance position by the specified number of bytes.
    /// Addresses: Requirement 14, criterion 14.4
    pub fn forward_bytes(&mut self, count: usize) {
        let target = (self.position + count).min(self.text.len());
        while self.position < target {
            let buf_pos = self.position.saturating_sub(self.style_offset);
            if buf_pos < self.style_data.len() {
                self.style_data[buf_pos] = self.current_style.0;
            }
            self.position += 1;
        }
    }

    /// Check current token against keyword sets. Returns matching set index.
    /// Addresses: Requirement 14, criterion 14.5
    pub fn match_keyword(&self, word_lists: &[WordList]) -> Option<KeywordSetIndex> {
        let token = self.current_token();
        if token.is_empty() {
            return None;
        }
        for (idx, wl) in word_lists.iter().enumerate() {
            if wl.contains(token) {
                return KeywordSetIndex::new(idx as u8);
            }
        }
        None
    }

    /// Returns true if current position is at the beginning of a line.
    /// Addresses: Requirement 14, criterion 14.6
    pub fn at_line_start(&self) -> bool {
        if self.position == 0 {
            return true;
        }
        let prev = self.text.as_bytes().get(self.position - 1).copied();
        prev == Some(b'\n')
    }

    /// Returns true if current character is a line-ending character.
    /// Addresses: Requirement 14, criterion 14.7
    pub fn at_line_end(&self) -> bool {
        let ch = self.ch();
        ch == '\n' || ch == '\r' || ch == '\0'
    }

    /// Returns true if there are more characters to process.
    /// Addresses: Requirement 14, criterion 14.8
    pub fn more(&self) -> bool {
        self.position < self.end
    }

    /// Get the text of the current token (from start to current position).
    pub fn current_token(&self) -> &str {
        let start = self.token_start.min(self.text.len());
        let end = self.position.min(self.text.len());
        &self.text[start..end]
    }

    /// Get the full text being styled.
    pub fn text(&self) -> &str {
        self.text
    }

    /// Get the end position of the styling range.
    pub fn end_position(&self) -> usize {
        self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context<'a>(text: &'a str, style_data: &'a mut Vec<u8>) -> StyleContext<'a> {
        StyleContext::new(text, 0, text.len(), LexerState::INITIAL, style_data, 0)
    }

    #[test]
    fn ch_returns_current_character() {
        // Validates: Requirement 14, criterion 14.1
        let mut data = vec![0u8; 5];
        let ctx = make_context("hello", &mut data);
        assert_eq!(ctx.ch(), 'h');
    }

    #[test]
    fn ch_next_returns_lookahead() {
        // Validates: Requirement 14, criterion 14.1
        let mut data = vec![0u8; 5];
        let ctx = make_context("hello", &mut data);
        assert_eq!(ctx.ch_next(), 'e');
    }

    #[test]
    fn ch_prev_returns_null_at_start() {
        // Validates: Requirement 14, criterion 14.1
        let mut data = vec![0u8; 5];
        let ctx = make_context("hello", &mut data);
        assert_eq!(ctx.ch_prev(), '\0');
    }

    #[test]
    fn ch_prev_returns_previous_after_forward() {
        let mut data = vec![0u8; 5];
        let mut ctx = make_context("hello", &mut data);
        ctx.forward();
        assert_eq!(ctx.ch_prev(), 'h');
        assert_eq!(ctx.ch(), 'e');
    }

    #[test]
    fn forward_advances_one_char() {
        // Validates: Requirement 14, criterion 14.3
        let mut data = vec![0u8; 5];
        let mut ctx = make_context("hello", &mut data);
        ctx.forward();
        assert_eq!(ctx.ch(), 'e');
        ctx.forward();
        assert_eq!(ctx.ch(), 'l');
    }

    #[test]
    fn forward_handles_multibyte_utf8() {
        // Validates: Requirement 14, criterion 14.3
        let text = "héllo";
        let mut data = vec![0u8; text.len()];
        let mut ctx = make_context(text, &mut data);
        assert_eq!(ctx.ch(), 'h');
        ctx.forward();
        assert_eq!(ctx.ch(), 'é');
        ctx.forward();
        assert_eq!(ctx.ch(), 'l');
    }

    #[test]
    fn forward_bytes_advances_by_count() {
        // Validates: Requirement 14, criterion 14.4
        let mut data = vec![0u8; 5];
        let mut ctx = make_context("hello", &mut data);
        ctx.forward_bytes(3);
        assert_eq!(ctx.ch(), 'l');
    }

    #[test]
    fn more_returns_false_at_end() {
        // Validates: Requirement 14, criterion 14.8
        let mut data = vec![0u8; 2];
        let mut ctx = make_context("hi", &mut data);
        assert!(ctx.more());
        ctx.forward();
        assert!(ctx.more());
        ctx.forward();
        assert!(!ctx.more());
    }

    #[test]
    fn at_line_start_at_beginning() {
        // Validates: Requirement 14, criterion 14.6
        let mut data = vec![0u8; 5];
        let ctx = make_context("hello", &mut data);
        assert!(ctx.at_line_start());
    }

    #[test]
    fn at_line_start_after_newline() {
        let text = "hi\nworld";
        let mut data = vec![0u8; text.len()];
        let mut ctx = make_context(text, &mut data);
        ctx.forward_bytes(3); // position at 'w'
        assert!(ctx.at_line_start());
    }

    #[test]
    fn at_line_end_at_newline() {
        // Validates: Requirement 14, criterion 14.7
        let text = "hi\nworld";
        let mut data = vec![0u8; text.len()];
        let mut ctx = make_context(text, &mut data);
        ctx.forward_bytes(2); // position at '\n'
        assert!(ctx.at_line_end());
    }

    #[test]
    fn set_state_assigns_style_and_resets_token_start() {
        // Validates: Requirement 14, criterion 14.2
        let mut data = vec![0u8; 5];
        let mut ctx = make_context("hello", &mut data);
        ctx.set_style(StyleSlotIndex(3));
        ctx.forward(); // writes style 3 at pos 0
        ctx.forward(); // writes style 3 at pos 1
        ctx.set_state(LexerState(1));
        assert_eq!(ctx.state(), LexerState(1));
        // Verify token start moved
        assert_eq!(ctx.start_position(), BytePosition(2));
        // Drop ctx to release borrow, then check data
        drop(ctx);
        // forward() wrote style 3 at positions 0 and 1
        assert_eq!(data[0], 3);
        assert_eq!(data[1], 3);
        assert_eq!(data[2], 0); // not yet styled
    }

    #[test]
    fn current_token_returns_text_slice() {
        let mut data = vec![0u8; 5];
        let mut ctx = make_context("hello", &mut data);
        ctx.forward();
        ctx.forward();
        ctx.forward();
        assert_eq!(ctx.current_token(), "hel");
    }

    #[test]
    fn ch_at_end_returns_null() {
        let mut data = vec![0u8; 2];
        let mut ctx = make_context("hi", &mut data);
        ctx.forward();
        ctx.forward();
        assert_eq!(ctx.ch(), '\0');
        assert_eq!(ctx.ch_next(), '\0');
    }

    #[test]
    fn start_position_tracks_token_start() {
        // Validates: Requirement 14, criterion 14.1
        let mut data = vec![0u8; 5];
        let mut ctx = make_context("hello", &mut data);
        assert_eq!(ctx.start_position(), BytePosition(0));
        ctx.forward();
        ctx.forward();
        ctx.set_state(LexerState(1));
        assert_eq!(ctx.start_position(), BytePosition(2));
    }
}
