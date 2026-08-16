//! Line end mode configuration and detection utilities.
//!
//! Provides `LineEndMode` and helper functions for detecting line endings
//! in byte sequences.

/// Configures which byte sequences are recognised as line endings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEndMode {
    /// Recognises CR (0x0D), LF (0x0A), and CRLF (0x0D 0x0A).
    #[default]
    Default,
    /// Additionally recognises LS (U+2028), PS (U+2029), NEL (U+0085).
    Unicode,
}

// Line ending byte sequences (used for documentation/reference):
// NEL (Next Line) in UTF-8: 0xC2 0x85
// LS (Line Separator) in UTF-8: 0xE2 0x80 0xA8
// PS (Paragraph Separator) in UTF-8: 0xE2 0x80 0xA9

/// Check if a byte sequence contains any line ending for the given mode.
pub fn contains_line_end(text: &[u8], mode: LineEndMode) -> bool {
    let mut i = 0;
    while i < text.len() {
        match text[i] {
            0x0A | 0x0D => return true,
            0xC2 if mode == LineEndMode::Unicode => {
                if i + 1 < text.len() && text[i + 1] == 0x85 {
                    return true;
                }
                i += 1;
            }
            0xE2 if mode == LineEndMode::Unicode
                && i + 2 < text.len()
                && text[i + 1] == 0x80
                && (text[i + 2] == 0xA8 || text[i + 2] == 0xA9) =>
            {
                return true;
            }
            _ => {
                i += 1;
            }
        }
    }
    false
}

/// Count line endings in a byte slice according to the given mode.
/// CRLF counts as a single line ending.
pub fn count_line_endings(text: &[u8], mode: LineEndMode) -> u64 {
    let mut count = 0u64;
    let mut i = 0;
    while i < text.len() {
        match text[i] {
            0x0D => {
                // Check for CRLF
                if i + 1 < text.len() && text[i + 1] == 0x0A {
                    count += 1;
                    i += 2;
                } else {
                    count += 1;
                    i += 1;
                }
            }
            0x0A => {
                count += 1;
                i += 1;
            }
            0xC2 if mode == LineEndMode::Unicode => {
                if i + 1 < text.len() && text[i + 1] == 0x85 {
                    count += 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            0xE2 if mode == LineEndMode::Unicode
                && i + 2 < text.len()
                && text[i + 1] == 0x80
                && (text[i + 2] == 0xA8 || text[i + 2] == 0xA9) =>
            {
                count += 1;
                i += 3;
            }
            _ => {
                i += 1;
            }
        }
    }
    count
}

/// Returns the byte length of a line ending at the given position, or 0 if none.
pub fn line_ending_length_at(text: &[u8], pos: usize, mode: LineEndMode) -> usize {
    if pos >= text.len() {
        return 0;
    }
    match text[pos] {
        0x0D => {
            if pos + 1 < text.len() && text[pos + 1] == 0x0A {
                2 // CRLF
            } else {
                1 // lone CR
            }
        }
        0x0A => 1, // LF
        0xC2 if mode == LineEndMode::Unicode => {
            if pos + 1 < text.len() && text[pos + 1] == 0x85 {
                2 // NEL
            } else {
                0
            }
        }
        0xE2 if mode == LineEndMode::Unicode
            && pos + 2 < text.len()
            && text[pos + 1] == 0x80
            && (text[pos + 2] == 0xA8 || text[pos + 2] == 0xA9) =>
        {
            3 // LS or PS
        }
        _ => 0,
    }
}

/// Check if position is inside a multi-byte line ending sequence (Unicode mode).
/// Returns the start of the line ending if inside, None otherwise.
pub fn inside_unicode_line_ending(text: &[u8], pos: usize, mode: LineEndMode) -> Option<usize> {
    if mode != LineEndMode::Unicode || pos == 0 {
        return None;
    }
    // Check if we're in the middle of NEL (0xC2 0x85)
    if pos >= 1 && text.get(pos - 1) == Some(&0xC2) && text.get(pos) == Some(&0x85) {
        return Some(pos - 1);
    }
    // Check if we're in the middle of LS/PS (0xE2 0x80 0xA8/0xA9)
    if pos >= 1
        && text.get(pos - 1) == Some(&0x80)
        && pos >= 2
        && text.get(pos - 2) == Some(&0xE2)
        && text.get(pos).is_some_and(|b| *b == 0xA8 || *b == 0xA9)
    {
        return Some(pos - 2);
    }
    if pos >= 2
        && text.get(pos - 2) == Some(&0xE2)
        && text.get(pos - 1) == Some(&0x80)
        && text.get(pos).is_some_and(|b| *b == 0xA8 || *b == 0xA9)
    {
        return Some(pos - 2);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_detects_cr_lf_crlf() {
        assert!(contains_line_end(b"hello\nworld", LineEndMode::Default));
        assert!(contains_line_end(b"hello\rworld", LineEndMode::Default));
        assert!(contains_line_end(b"hello\r\nworld", LineEndMode::Default));
        assert!(!contains_line_end(b"hello world", LineEndMode::Default));
    }

    #[test]
    fn unicode_mode_detects_nel_ls_ps() {
        // NEL = 0xC2 0x85
        assert!(contains_line_end(
            &[b'a', 0xC2, 0x85, b'b'],
            LineEndMode::Unicode
        ));
        // LS = 0xE2 0x80 0xA8
        assert!(contains_line_end(
            &[b'a', 0xE2, 0x80, 0xA8, b'b'],
            LineEndMode::Unicode
        ));
        // PS = 0xE2 0x80 0xA9
        assert!(contains_line_end(
            &[b'a', 0xE2, 0x80, 0xA9, b'b'],
            LineEndMode::Unicode
        ));
        // These should NOT be detected in Default mode
        assert!(!contains_line_end(
            &[b'a', 0xC2, 0x85, b'b'],
            LineEndMode::Default
        ));
    }

    #[test]
    fn count_line_endings_crlf_as_one() {
        assert_eq!(count_line_endings(b"a\r\nb\r\nc", LineEndMode::Default), 2);
        assert_eq!(count_line_endings(b"a\rb\nc", LineEndMode::Default), 2);
        assert_eq!(count_line_endings(b"no endings", LineEndMode::Default), 0);
    }

    #[test]
    fn count_line_endings_unicode_mode() {
        let text: Vec<u8> = [
            b"a".as_slice(),
            &[0xC2, 0x85],
            b"b",
            &[0xE2, 0x80, 0xA8],
            b"c",
        ]
        .concat();
        assert_eq!(count_line_endings(&text, LineEndMode::Unicode), 2);
    }

    #[test]
    fn line_ending_length_at_various() {
        assert_eq!(line_ending_length_at(b"\r\n", 0, LineEndMode::Default), 2);
        assert_eq!(line_ending_length_at(b"\r", 0, LineEndMode::Default), 1);
        assert_eq!(line_ending_length_at(b"\n", 0, LineEndMode::Default), 1);
        assert_eq!(line_ending_length_at(b"a", 0, LineEndMode::Default), 0);
        let nel = [0xC2, 0x85];
        assert_eq!(line_ending_length_at(&nel, 0, LineEndMode::Unicode), 2);
        assert_eq!(line_ending_length_at(&nel, 0, LineEndMode::Default), 0);
    }
}
