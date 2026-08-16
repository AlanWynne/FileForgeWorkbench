//! Encoding-aware character navigation.
//!
//! Provides UTF-8 and CRLF-aware navigation functions that ensure cursor
//! positions never land inside multi-byte sequences or between CR and LF.

use crate::gap_buffer::GapBuffer;
use crate::types::{CharacterExtracted, Direction};

/// Get the byte length of the character at position.
/// Returns 2 for CRLF, 1-4 for valid UTF-8, 1 for invalid bytes.
pub fn char_length_at(buffer: &GapBuffer, position: u64) -> u8 {
    let length = buffer.length();
    if position >= length {
        return 0;
    }

    let byte = match buffer.byte_at(position) {
        Some(b) => b,
        None => return 0,
    };

    // Check for CRLF
    if byte == 0x0D {
        if let Some(0x0A) = buffer.byte_at(position + 1) {
            return 2;
        }
        return 1;
    }

    // ASCII or invalid leading byte
    if byte < 0x80 {
        return 1;
    }

    // UTF-8 sequence: validate continuation bytes
    let expected_len = utf8_sequence_length(byte);
    if expected_len == 1 {
        return 1; // Invalid leading byte (e.g., 0x80-0xBF standalone)
    }

    // Check that we have enough bytes and they're valid continuations
    for i in 1..expected_len {
        match buffer.byte_at(position + i as u64) {
            Some(b) if is_utf8_continuation(b) => {}
            _ => return 1, // Invalid sequence, treat leading byte as single char
        }
    }

    expected_len
}

/// Move position outside a multi-byte sequence to the nearest boundary.
pub fn move_position_outside_char(buffer: &GapBuffer, position: u64, direction: Direction) -> u64 {
    let length = buffer.length();
    if position == 0 || position >= length {
        return position.min(length);
    }

    // Check if we're between CR and LF
    if is_between_crlf(buffer, position) {
        return match direction {
            Direction::Forward => position + 1,  // Move to after LF
            Direction::Backward => position - 1, // Move to CR
        };
    }

    // Check if we're inside a UTF-8 multi-byte sequence
    let byte = match buffer.byte_at(position) {
        Some(b) => b,
        None => return position,
    };

    if is_utf8_continuation(byte) {
        // We're inside a multi-byte sequence, find the start
        match direction {
            Direction::Backward => find_char_start(buffer, position),
            Direction::Forward => find_next_char_start(buffer, position),
        }
    } else {
        position
    }
}

/// Advance to the next valid character position.
pub fn next_position(buffer: &GapBuffer, position: u64, direction: Direction) -> Option<u64> {
    let length = buffer.length();

    match direction {
        Direction::Forward => {
            if position >= length {
                return None;
            }
            let char_len = char_length_at(buffer, position) as u64;
            let next = position + char_len.max(1);
            if next > length {
                Some(length)
            } else {
                Some(next)
            }
        }
        Direction::Backward => {
            if position == 0 {
                return None;
            }
            // Find the start of the character that ends at or before `position`
            let prev_byte_pos = position - 1;
            let start = find_char_start(buffer, prev_byte_pos);

            // Check if the character at `start` is a CRLF pair
            if let Some(0x0D) = buffer.byte_at(start) {
                if let Some(0x0A) = buffer.byte_at(start + 1) {
                    // It's a CRLF - return start of the pair
                    return Some(start);
                }
            }

            // Check if we landed on an LF that is part of a CRLF before it
            if let Some(0x0A) = buffer.byte_at(start) {
                if start > 0 {
                    if let Some(0x0D) = buffer.byte_at(start - 1) {
                        // The character before is actually a CRLF starting at start-1
                        return Some(start - 1);
                    }
                }
            }

            Some(start)
        }
    }
}

/// Extract the character at position.
pub fn character_at(buffer: &GapBuffer, position: u64) -> Option<CharacterExtracted> {
    let length = buffer.length();
    if position >= length {
        return None;
    }

    let byte = buffer.byte_at(position)?;

    // CRLF
    if byte == 0x0D {
        if buffer.byte_at(position + 1) == Some(0x0A) {
            return Some(CharacterExtracted {
                character: '\n',
                byte_width: 2,
            });
        }
        return Some(CharacterExtracted {
            character: '\r',
            byte_width: 1,
        });
    }

    // ASCII
    if byte < 0x80 {
        return Some(CharacterExtracted {
            character: byte as char,
            byte_width: 1,
        });
    }

    // UTF-8 multi-byte
    let seq_len = utf8_sequence_length(byte);
    let mut bytes = [0u8; 4];
    bytes[0] = byte;
    for (i, byte_slot) in bytes.iter_mut().enumerate().take(seq_len as usize).skip(1) {
        match buffer.byte_at(position + i as u64) {
            Some(b) if is_utf8_continuation(b) => *byte_slot = b,
            _ => {
                // Invalid sequence: treat first byte as replacement character
                return Some(CharacterExtracted {
                    character: '\u{FFFD}',
                    byte_width: 1,
                });
            }
        }
    }

    match std::str::from_utf8(&bytes[..seq_len as usize]) {
        Ok(s) => {
            let ch = s.chars().next().unwrap_or('\u{FFFD}');
            Some(CharacterExtracted {
                character: ch,
                byte_width: seq_len,
            })
        }
        Err(_) => Some(CharacterExtracted {
            character: '\u{FFFD}',
            byte_width: 1,
        }),
    }
}

/// Extract the character before position.
pub fn character_before(buffer: &GapBuffer, position: u64) -> Option<CharacterExtracted> {
    if position == 0 {
        return None;
    }

    // Find the start of the character before this position
    let start = find_char_start(buffer, position - 1);

    // Check for CRLF
    if let Some(0x0D) = buffer.byte_at(start) {
        if let Some(0x0A) = buffer.byte_at(start + 1) {
            if start + 2 == position {
                return Some(CharacterExtracted {
                    character: '\n',
                    byte_width: 2,
                });
            }
        }
    }

    character_at(buffer, start)
}

/// Move by character offset from start position.
pub fn relative_position(buffer: &GapBuffer, start: u64, character_offset: i64) -> Option<u64> {
    let mut pos = start;

    if character_offset > 0 {
        for _ in 0..character_offset {
            pos = next_position(buffer, pos, Direction::Forward)?;
        }
    } else if character_offset < 0 {
        for _ in 0..(-character_offset) {
            pos = next_position(buffer, pos, Direction::Backward)?;
        }
    }

    Some(pos)
}

// --- Helper functions ---

/// Check if position is between a CR and its following LF.
fn is_between_crlf(buffer: &GapBuffer, position: u64) -> bool {
    if position == 0 {
        return false;
    }
    buffer.byte_at(position - 1) == Some(0x0D) && buffer.byte_at(position) == Some(0x0A)
}

/// Find the start of the character containing or before `position`.
fn find_char_start(buffer: &GapBuffer, position: u64) -> u64 {
    let mut pos = position;
    // Walk backwards through continuation bytes
    while pos > 0 {
        match buffer.byte_at(pos) {
            Some(b) if is_utf8_continuation(b) => pos -= 1,
            _ => break,
        }
    }
    pos
}

/// Find the next character start after position (skipping continuation bytes).
fn find_next_char_start(buffer: &GapBuffer, position: u64) -> u64 {
    let length = buffer.length();
    let mut pos = position + 1;
    while pos < length {
        match buffer.byte_at(pos) {
            Some(b) if is_utf8_continuation(b) => pos += 1,
            _ => break,
        }
    }
    pos
}

/// Check if a byte is a UTF-8 continuation byte (10xxxxxx).
fn is_utf8_continuation(byte: u8) -> bool {
    byte & 0xC0 == 0x80
}

/// Get the expected UTF-8 sequence length from the leading byte.
fn utf8_sequence_length(byte: u8) -> u8 {
    if byte < 0x80 {
        1
    } else if byte & 0xE0 == 0xC0 {
        2
    } else if byte & 0xF0 == 0xE0 {
        3
    } else if byte & 0xF8 == 0xF0 {
        4
    } else {
        1 // Invalid leading byte, treat as single byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer_from(data: &[u8]) -> GapBuffer {
        let mut buf = GapBuffer::new(data.len() as u64 + 64);
        buf.insert(0, data);
        buf
    }

    #[test]
    fn char_length_at_ascii() {
        let buf = buffer_from(b"hello");
        assert_eq!(char_length_at(&buf, 0), 1);
    }

    #[test]
    fn char_length_at_crlf() {
        let buf = buffer_from(b"a\r\nb");
        assert_eq!(char_length_at(&buf, 1), 2); // CRLF at position 1
    }

    #[test]
    fn char_length_at_utf8_multibyte() {
        // 'é' is 0xC3 0xA9 (2 bytes)
        let buf = buffer_from("é".as_bytes());
        assert_eq!(char_length_at(&buf, 0), 2);
    }

    #[test]
    fn char_length_at_four_byte_utf8() {
        // '𝄞' (musical symbol) is 4 bytes
        let buf = buffer_from("𝄞".as_bytes());
        assert_eq!(char_length_at(&buf, 0), 4);
    }

    #[test]
    fn next_position_forward_ascii() {
        let buf = buffer_from(b"abc");
        assert_eq!(next_position(&buf, 0, Direction::Forward), Some(1));
        assert_eq!(next_position(&buf, 1, Direction::Forward), Some(2));
        assert_eq!(next_position(&buf, 2, Direction::Forward), Some(3));
        assert_eq!(next_position(&buf, 3, Direction::Forward), None);
    }

    #[test]
    fn next_position_backward_ascii() {
        let buf = buffer_from(b"abc");
        assert_eq!(next_position(&buf, 3, Direction::Backward), Some(2));
        assert_eq!(next_position(&buf, 2, Direction::Backward), Some(1));
        assert_eq!(next_position(&buf, 1, Direction::Backward), Some(0));
        assert_eq!(next_position(&buf, 0, Direction::Backward), None);
    }

    #[test]
    fn next_position_skips_crlf_atomically() {
        let buf = buffer_from(b"a\r\nb");
        assert_eq!(next_position(&buf, 0, Direction::Forward), Some(1));
        assert_eq!(next_position(&buf, 1, Direction::Forward), Some(3)); // skip CRLF
        assert_eq!(next_position(&buf, 3, Direction::Backward), Some(1)); // back to CR
    }

    #[test]
    fn next_position_multibyte_utf8() {
        let buf = buffer_from("aé".as_bytes());
        assert_eq!(next_position(&buf, 0, Direction::Forward), Some(1));
        assert_eq!(next_position(&buf, 1, Direction::Forward), Some(3)); // é is 2 bytes
    }

    #[test]
    fn character_at_ascii() {
        let buf = buffer_from(b"hello");
        let ch = character_at(&buf, 0).unwrap();
        assert_eq!(ch.character, 'h');
        assert_eq!(ch.byte_width, 1);
    }

    #[test]
    fn character_at_multibyte() {
        let buf = buffer_from("日".as_bytes());
        let ch = character_at(&buf, 0).unwrap();
        assert_eq!(ch.character, '日');
        assert_eq!(ch.byte_width, 3);
    }

    #[test]
    fn character_at_invalid_utf8() {
        let buf = buffer_from(&[0xFF, b'a']);
        let ch = character_at(&buf, 0).unwrap();
        assert_eq!(ch.character, '\u{FFFD}');
        assert_eq!(ch.byte_width, 1);
    }

    #[test]
    fn character_before_at_various_positions() {
        let buf = buffer_from(b"ab");
        let ch = character_before(&buf, 2).unwrap();
        assert_eq!(ch.character, 'b');
        let ch = character_before(&buf, 1).unwrap();
        assert_eq!(ch.character, 'a');
        assert_eq!(character_before(&buf, 0), None);
    }

    #[test]
    fn relative_position_forward() {
        let buf = buffer_from(b"hello");
        assert_eq!(relative_position(&buf, 0, 3), Some(3));
    }

    #[test]
    fn relative_position_backward() {
        let buf = buffer_from(b"hello");
        assert_eq!(relative_position(&buf, 3, -2), Some(1));
    }

    #[test]
    fn relative_position_out_of_bounds() {
        let buf = buffer_from(b"hi");
        assert_eq!(relative_position(&buf, 0, 10), None);
        assert_eq!(relative_position(&buf, 2, -5), None);
    }

    #[test]
    fn move_position_outside_char_crlf() {
        let buf = buffer_from(b"a\r\nb");
        // Position 2 is between CR and LF
        let adjusted_back = move_position_outside_char(&buf, 2, Direction::Backward);
        assert_eq!(adjusted_back, 1); // Move to CR

        let adjusted_fwd = move_position_outside_char(&buf, 2, Direction::Forward);
        assert_eq!(adjusted_fwd, 3); // Move to after LF
    }

    #[test]
    fn move_position_outside_char_multibyte() {
        // 'é' = 0xC3 0xA9
        let buf = buffer_from("aé".as_bytes());
        // Position 2 is the continuation byte of 'é'
        let adjusted_back = move_position_outside_char(&buf, 2, Direction::Backward);
        assert_eq!(adjusted_back, 1); // start of 'é'

        let adjusted_fwd = move_position_outside_char(&buf, 2, Direction::Forward);
        assert_eq!(adjusted_fwd, 3); // after 'é'
    }
}
