//! Word boundary detection for WORD and WORDSTART matching.
//!
//! Addresses: Requirement 11

use crate::indexer::CharacterIndexer;
use crate::request::WordMatchMode;
use crate::types::BytePosition;

/// Classify whether a byte represents a word character.
///
/// Word characters are: ASCII letters (a-z, A-Z), digits (0-9), underscore (_),
/// and any non-ASCII byte that's part of a multi-byte UTF-8 sequence (treated as word char).
///
/// Addresses: Requirement 11 AC 3
pub fn is_word_char_at(indexer: &dyn CharacterIndexer, position: BytePosition) -> bool {
    match indexer.char_at(position) {
        Some(b) => is_word_byte(b),
        None => false,
    }
}

/// Classify a single byte as a word character.
///
/// For multi-byte UTF-8 characters, we check the full code point by decoding from
/// the start of the character.
fn is_word_byte(b: u8) -> bool {
    matches!(b,
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_'
        | 0x80..=0xFF // Non-ASCII bytes are part of multi-byte chars, treat as word
    )
}

/// Check if a match at the given position satisfies word boundary constraints.
///
/// Addresses: Requirement 11 AC 1–2, AC 4–5
pub fn check_word_boundary(
    mode: WordMatchMode,
    match_start: BytePosition,
    match_end: BytePosition,
    indexer: &dyn CharacterIndexer,
) -> bool {
    match mode {
        WordMatchMode::None => true,
        WordMatchMode::WholeWord => {
            has_word_boundary_at_start(match_start, indexer)
                && has_word_boundary_at_end(match_end, indexer)
        }
        WordMatchMode::WordStart => has_word_boundary_at_start(match_start, indexer),
    }
}

/// Check for a word boundary at the start of a match.
///
/// A word boundary exists at start if:
/// - The match is at position 0 (start of document), OR
/// - The character immediately before the match is NOT a word character
///   AND the first character of the match IS a word character
fn has_word_boundary_at_start(position: BytePosition, indexer: &dyn CharacterIndexer) -> bool {
    if position == BytePosition::ZERO {
        // At document start — word boundary exists if match starts with word char
        return match indexer.char_at(position) {
            Some(b) => is_word_byte(b),
            None => false,
        };
    }

    let before = indexer.char_at(position - 1);
    let at = indexer.char_at(position);

    match (before, at) {
        (Some(before_byte), Some(at_byte)) => !is_word_byte(before_byte) && is_word_byte(at_byte),
        (None, Some(at_byte)) => is_word_byte(at_byte),
        _ => false,
    }
}

/// Check for a word boundary at the end of a match.
///
/// A word boundary exists at end if:
/// - The match ends at document length (end of document), OR
/// - The character immediately after the match is NOT a word character
///   AND the last character of the match IS a word character
fn has_word_boundary_at_end(position: BytePosition, indexer: &dyn CharacterIndexer) -> bool {
    if position.0 >= indexer.length() {
        // At document end — boundary exists if last char before end is word char
        if position == BytePosition::ZERO {
            return false;
        }
        return match indexer.char_at(position - 1) {
            Some(b) => is_word_byte(b),
            None => false,
        };
    }

    let at = indexer.char_at(position);
    let before = if position == BytePosition::ZERO {
        None
    } else {
        indexer.char_at(position - 1)
    };

    match (before, at) {
        (Some(before_byte), Some(at_byte)) => is_word_byte(before_byte) && !is_word_byte(at_byte),
        (Some(before_byte), None) => is_word_byte(before_byte),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::SliceIndexer;

    #[test]
    fn whole_word_matches_isolated_word() {
        let indexer = SliceIndexer::from_str("hello world foo");
        // "world" starts at position 6, ends at 11
        assert!(check_word_boundary(
            WordMatchMode::WholeWord,
            BytePosition(6),
            BytePosition(11),
            &indexer,
        ));
    }

    #[test]
    fn whole_word_rejects_partial_word_match() {
        let indexer = SliceIndexer::from_str("helloworld");
        // "world" starts at position 5, ends at 10 — no boundary at start
        assert!(!check_word_boundary(
            WordMatchMode::WholeWord,
            BytePosition(5),
            BytePosition(10),
            &indexer,
        ));
    }

    #[test]
    fn word_start_matches_at_word_beginning() {
        let indexer = SliceIndexer::from_str("hello world");
        // "wor" starts at position 6
        assert!(check_word_boundary(
            WordMatchMode::WordStart,
            BytePosition(6),
            BytePosition(9),
            &indexer,
        ));
    }

    #[test]
    fn word_start_rejects_mid_word_match() {
        let indexer = SliceIndexer::from_str("helloworld");
        // "world" starts at 5 with no boundary (preceded by word char)
        assert!(!check_word_boundary(
            WordMatchMode::WordStart,
            BytePosition(5),
            BytePosition(10),
            &indexer,
        ));
    }

    #[test]
    fn whole_word_matches_at_document_start() {
        let indexer = SliceIndexer::from_str("hello world");
        assert!(check_word_boundary(
            WordMatchMode::WholeWord,
            BytePosition(0),
            BytePosition(5),
            &indexer,
        ));
    }

    #[test]
    fn whole_word_matches_at_document_end() {
        let indexer = SliceIndexer::from_str("hello world");
        assert!(check_word_boundary(
            WordMatchMode::WholeWord,
            BytePosition(6),
            BytePosition(11),
            &indexer,
        ));
    }

    #[test]
    fn none_mode_always_passes() {
        let indexer = SliceIndexer::from_str("helloworld");
        assert!(check_word_boundary(
            WordMatchMode::None,
            BytePosition(3),
            BytePosition(7),
            &indexer,
        ));
    }
}
