//! Optimised literal search algorithm.
//!
//! Uses byte-by-byte scanning for fast literal text matching with support
//! for forward/backward search, case-insensitive matching via CaseFolder,
//! and column-bounded search.
//!
//! Addresses: Requirement 1 AC 1–10, Requirement 19 AC 3, 19.7

use crate::case_folder::CaseFolder;
use crate::indexer::CharacterIndexer;
use crate::request::WordMatchMode;
use crate::result::FindResult;
use crate::types::BytePosition;
use crate::word_boundary::check_word_boundary;

/// Find the first forward match of a literal byte pattern starting from `from`.
///
/// Addresses: Requirement 1 AC 1–2
pub fn find_literal_forward(
    pattern: &[u8],
    indexer: &dyn CharacterIndexer,
    from: BytePosition,
    end: BytePosition,
    word_mode: WordMatchMode,
) -> Option<FindResult> {
    if pattern.is_empty() || from.0 >= end.0 {
        return None;
    }

    let pattern_len = pattern.len() as u64;
    let search_end = end.0.saturating_sub(pattern_len - 1);

    let mut pos = from.0;
    while pos < search_end {
        if matches_at(pattern, indexer, BytePosition(pos)) {
            let match_start = BytePosition(pos);
            let match_end = BytePosition(pos + pattern_len);

            if check_word_boundary(word_mode, match_start, match_end, indexer) {
                let line = indexer.line_from_position(match_start);
                return Some(FindResult::simple(match_start, match_end, line));
            }
        }
        pos += 1;
    }

    None
}

/// Find the last backward match of a literal byte pattern before `from`.
///
/// Addresses: Requirement 1 AC 3
pub fn find_literal_backward(
    pattern: &[u8],
    indexer: &dyn CharacterIndexer,
    from: BytePosition,
    start: BytePosition,
    word_mode: WordMatchMode,
) -> Option<FindResult> {
    if pattern.is_empty() {
        return None;
    }

    let pattern_len = pattern.len() as u64;
    // Start searching from just before `from` position
    let search_start = if from.0 >= pattern_len {
        from.0 - pattern_len
    } else {
        return None;
    };

    let mut pos = search_start;
    loop {
        if pos < start.0 {
            break;
        }

        if matches_at(pattern, indexer, BytePosition(pos)) {
            let match_start = BytePosition(pos);
            let match_end = BytePosition(pos + pattern_len);

            if check_word_boundary(word_mode, match_start, match_end, indexer) {
                let line = indexer.line_from_position(match_start);
                return Some(FindResult::simple(match_start, match_end, line));
            }
        }

        if pos == start.0 {
            break;
        }
        pos -= 1;
    }

    None
}

/// Find all non-overlapping forward matches of a literal pattern.
///
/// Addresses: Requirement 1 AC 6
pub fn find_literal_all(
    pattern: &[u8],
    indexer: &dyn CharacterIndexer,
    start: BytePosition,
    end: BytePosition,
    word_mode: WordMatchMode,
) -> Vec<FindResult> {
    let mut results = Vec::new();
    let mut pos = start;

    while pos.0 < end.0 {
        match find_literal_forward(pattern, indexer, pos, end, word_mode) {
            Some(result) => {
                let next_pos = result.match_range.end;
                results.push(result);
                pos = next_pos;
            }
            None => break,
        }
    }

    results
}

/// Find a literal pattern using case-insensitive matching via CaseFolder.
///
/// The search term should be pre-folded. Document segments are folded lazily.
///
/// Addresses: Requirement 1 AC 10, Requirement 10 AC 2, AC 6
pub fn find_literal_case_insensitive_forward(
    folded_pattern: &[u8],
    indexer: &dyn CharacterIndexer,
    from: BytePosition,
    end: BytePosition,
    case_folder: &CaseFolder,
    word_mode: WordMatchMode,
) -> Option<FindResult> {
    if folded_pattern.is_empty() || from.0 >= end.0 {
        return None;
    }

    let doc_len = end.0;
    let mut pos = from.0;

    while pos < doc_len {
        // Try to match starting at this position by folding document chars
        if let Some(match_end) =
            try_case_insensitive_match(folded_pattern, indexer, BytePosition(pos), end, case_folder)
        {
            let match_start = BytePosition(pos);

            if check_word_boundary(word_mode, match_start, match_end, indexer) {
                let line = indexer.line_from_position(match_start);
                return Some(FindResult::simple(match_start, match_end, line));
            }
        }
        pos += 1;
    }

    None
}

/// Find all case-insensitive forward matches.
pub fn find_literal_case_insensitive_all(
    folded_pattern: &[u8],
    indexer: &dyn CharacterIndexer,
    start: BytePosition,
    end: BytePosition,
    case_folder: &CaseFolder,
    word_mode: WordMatchMode,
) -> Vec<FindResult> {
    let mut results = Vec::new();
    let mut pos = start;

    while pos.0 < end.0 {
        match find_literal_case_insensitive_forward(
            folded_pattern,
            indexer,
            pos,
            end,
            case_folder,
            word_mode,
        ) {
            Some(result) => {
                let next_pos = result.match_range.end;
                results.push(result);
                pos = next_pos;
            }
            None => break,
        }
    }

    results
}

/// Find case-insensitive backward match.
pub fn find_literal_case_insensitive_backward(
    folded_pattern: &[u8],
    indexer: &dyn CharacterIndexer,
    from: BytePosition,
    start: BytePosition,
    end: BytePosition,
    case_folder: &CaseFolder,
    word_mode: WordMatchMode,
) -> Option<FindResult> {
    if folded_pattern.is_empty() || from.0 == 0 {
        return None;
    }

    let mut pos = from.0 - 1;
    loop {
        if pos < start.0 {
            break;
        }

        if let Some(match_end) =
            try_case_insensitive_match(folded_pattern, indexer, BytePosition(pos), end, case_folder)
        {
            let match_start = BytePosition(pos);

            if check_word_boundary(word_mode, match_start, match_end, indexer) {
                let line = indexer.line_from_position(match_start);
                return Some(FindResult::simple(match_start, match_end, line));
            }
        }

        if pos == start.0 {
            break;
        }
        pos -= 1;
    }

    None
}

/// Try to match folded_pattern at a given position using case folding.
///
/// Returns the end position of the match if successful.
fn try_case_insensitive_match(
    folded_pattern: &[u8],
    indexer: &dyn CharacterIndexer,
    start: BytePosition,
    end: BytePosition,
    case_folder: &CaseFolder,
) -> Option<BytePosition> {
    let mut pattern_idx = 0;
    let mut doc_pos = start.0;

    while pattern_idx < folded_pattern.len() && doc_pos < end.0 {
        // Read a UTF-8 character from the document
        indexer.char_at(BytePosition(doc_pos))?;
        let (ch, char_len) = decode_utf8_char(indexer, BytePosition(doc_pos));

        // Fold the document character
        let folded_chars = case_folder.fold_char(ch);
        let mut folded_doc_bytes = Vec::new();
        for fc in &folded_chars {
            let mut buf = [0u8; 4];
            let s = fc.encode_utf8(&mut buf);
            folded_doc_bytes.extend_from_slice(s.as_bytes());
        }

        // Check if folded doc bytes match the pattern at current position
        let remaining_pattern = &folded_pattern[pattern_idx..];
        if remaining_pattern.len() < folded_doc_bytes.len() {
            return None;
        }
        if remaining_pattern[..folded_doc_bytes.len()] != folded_doc_bytes[..] {
            return None;
        }

        pattern_idx += folded_doc_bytes.len();
        doc_pos += char_len as u64;
    }

    if pattern_idx == folded_pattern.len() {
        Some(BytePosition(doc_pos))
    } else {
        None
    }
}

/// Decode a UTF-8 character from the indexer at the given position.
/// Returns the character and its byte length.
fn decode_utf8_char(indexer: &dyn CharacterIndexer, position: BytePosition) -> (char, u8) {
    let first = match indexer.char_at(position) {
        Some(b) => b,
        None => return ('\u{FFFD}', 1),
    };

    if first < 0x80 {
        return (first as char, 1);
    }

    let len = if first & 0xE0 == 0xC0 {
        2
    } else if first & 0xF0 == 0xE0 {
        3
    } else if first & 0xF8 == 0xF0 {
        4
    } else {
        return ('\u{FFFD}', 1);
    };

    let mut bytes = [0u8; 4];
    bytes[0] = first;
    for (i, byte) in bytes.iter_mut().enumerate().take(len).skip(1) {
        match indexer.char_at(position + i as u64) {
            Some(b) if (b & 0xC0) == 0x80 => *byte = b,
            _ => return ('\u{FFFD}', len as u8),
        }
    }

    match std::str::from_utf8(&bytes[..len]) {
        Ok(s) => (s.chars().next().unwrap_or('\u{FFFD}'), len as u8),
        Err(_) => ('\u{FFFD}', len as u8),
    }
}

/// Check if the pattern matches at the given position (byte-by-byte).
fn matches_at(pattern: &[u8], indexer: &dyn CharacterIndexer, position: BytePosition) -> bool {
    for (i, &pattern_byte) in pattern.iter().enumerate() {
        match indexer.char_at(position + i as u64) {
            Some(doc_byte) if doc_byte == pattern_byte => continue,
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::SliceIndexer;
    use crate::types::LineNumber;

    #[test]
    fn find_literal_forward_finds_first_occurrence() {
        let indexer = SliceIndexer::from_str("hello world hello");
        let result = find_literal_forward(
            b"hello",
            &indexer,
            BytePosition(0),
            BytePosition(indexer.length()),
            WordMatchMode::None,
        );
        let r = result.unwrap();
        assert_eq!(r.match_range.start, BytePosition(0));
        assert_eq!(r.match_range.end, BytePosition(5));
    }

    #[test]
    fn find_literal_forward_finds_from_position() {
        let indexer = SliceIndexer::from_str("hello world hello");
        let result = find_literal_forward(
            b"hello",
            &indexer,
            BytePosition(1),
            BytePosition(indexer.length()),
            WordMatchMode::None,
        );
        let r = result.unwrap();
        assert_eq!(r.match_range.start, BytePosition(12));
    }

    #[test]
    fn find_literal_forward_returns_none_when_not_found() {
        let indexer = SliceIndexer::from_str("hello world");
        let result = find_literal_forward(
            b"xyz",
            &indexer,
            BytePosition(0),
            BytePosition(indexer.length()),
            WordMatchMode::None,
        );
        assert!(result.is_none());
    }

    #[test]
    fn find_literal_backward_finds_nearest_before_position() {
        let indexer = SliceIndexer::from_str("hello world hello");
        let result = find_literal_backward(
            b"hello",
            &indexer,
            BytePosition(17),
            BytePosition(0),
            WordMatchMode::None,
        );
        let r = result.unwrap();
        assert_eq!(r.match_range.start, BytePosition(12));
    }

    #[test]
    fn find_literal_backward_skips_match_at_from_position() {
        let indexer = SliceIndexer::from_str("hello world hello");
        // from=12 means we search for match ending before position 12
        let result = find_literal_backward(
            b"hello",
            &indexer,
            BytePosition(12),
            BytePosition(0),
            WordMatchMode::None,
        );
        let r = result.unwrap();
        assert_eq!(r.match_range.start, BytePosition(0));
    }

    #[test]
    fn find_literal_all_finds_all_non_overlapping_matches() {
        let indexer = SliceIndexer::from_str("abcabcabc");
        let results = find_literal_all(
            b"abc",
            &indexer,
            BytePosition(0),
            BytePosition(indexer.length()),
            WordMatchMode::None,
        );
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].match_range.start, BytePosition(0));
        assert_eq!(results[1].match_range.start, BytePosition(3));
        assert_eq!(results[2].match_range.start, BytePosition(6));
    }

    #[test]
    fn find_literal_case_insensitive_matches_different_cases() {
        let indexer = SliceIndexer::from_str("Hello WORLD hello");
        let folder = CaseFolder::new();
        let folded = folder.fold_bytes(b"hello");
        let result = find_literal_case_insensitive_forward(
            &folded,
            &indexer,
            BytePosition(0),
            BytePosition(indexer.length()),
            &folder,
            WordMatchMode::None,
        );
        let r = result.unwrap();
        assert_eq!(r.match_range.start, BytePosition(0));
        assert_eq!(r.match_range.end, BytePosition(5));
    }

    #[test]
    fn find_literal_handles_empty_document() {
        let indexer = SliceIndexer::from_str("");
        let result = find_literal_forward(
            b"hello",
            &indexer,
            BytePosition(0),
            BytePosition(0),
            WordMatchMode::None,
        );
        assert!(result.is_none());
    }

    #[test]
    fn find_literal_handles_null_bytes() {
        let data = b"hello\x00world";
        let indexer = SliceIndexer::new(data);
        let result = find_literal_forward(
            b"\x00",
            &indexer,
            BytePosition(0),
            BytePosition(indexer.length()),
            WordMatchMode::None,
        );
        let r = result.unwrap();
        assert_eq!(r.match_range.start, BytePosition(5));
    }

    #[test]
    fn find_literal_forward_reports_correct_line_number() {
        let indexer = SliceIndexer::from_str("line1\nline2\ntarget here");
        let result = find_literal_forward(
            b"target",
            &indexer,
            BytePosition(0),
            BytePosition(indexer.length()),
            WordMatchMode::None,
        );
        let r = result.unwrap();
        assert_eq!(r.line, LineNumber(2));
    }
}
