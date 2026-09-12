use crate::error::FindReplaceError;

use super::{CharClass, NfaInstruction};

/// Apply a quantifier (*, +, ?) to the last instruction.
pub(super) fn apply_quantifier(
    instructions: &mut Vec<NfaInstruction>,
    last_idx: usize,
    quant: char,
    lazy: bool,
) {
    let body_start = last_idx;
    let body_end = instructions.len();

    match quant {
        '*' => {
            // Zero or more: Split(body, after) + body + Jump(split)
            let split_pos = body_start;
            let after_pos = body_end + 2; // +1 for split, +1 for jump

            let split = if lazy {
                NfaInstruction::SplitLazy {
                    first: after_pos,
                    second: split_pos + 1,
                }
            } else {
                NfaInstruction::Split {
                    first: split_pos + 1,
                    second: after_pos,
                }
            };
            instructions.insert(split_pos, split);
            instructions.push(NfaInstruction::Jump(split_pos));
        }
        '+' => {
            // One or more: body + Split(body, after)
            let split_pos = instructions.len();
            let split = if lazy {
                NfaInstruction::SplitLazy {
                    first: split_pos + 1,
                    second: body_start,
                }
            } else {
                NfaInstruction::Split {
                    first: body_start,
                    second: split_pos + 1,
                }
            };
            instructions.push(split);
        }
        '?' => {
            // Zero or one: Split(body, after) + body
            let after_pos = body_end + 1;
            let split = if lazy {
                NfaInstruction::SplitLazy {
                    first: after_pos,
                    second: body_start + 1,
                }
            } else {
                NfaInstruction::Split {
                    first: body_start + 1,
                    second: after_pos,
                }
            };
            instructions.insert(body_start, split);
        }
        _ => {}
    }
}

/// Parse a character class [...] from the pattern.
/// Returns the CharClass and number of characters consumed.
pub(super) fn parse_char_class(chars: &[char]) -> Result<(CharClass, usize), FindReplaceError> {
    // chars[0] == '['
    let mut i = 1;
    let negated = if i < chars.len() && chars[i] == '^' {
        i += 1;
        true
    } else {
        false
    };

    let mut ranges: Vec<(u8, u8)> = Vec::new();

    // Handle ] at start (literal ])
    if i < chars.len() && chars[i] == ']' {
        ranges.push((b']', b']'));
        i += 1;
    }

    while i < chars.len() && chars[i] != ']' {
        let ch = chars[i];
        if ch == '\\' && i + 1 < chars.len() {
            i += 1;
            let escaped = escape_to_byte(chars[i]);
            if i + 1 < chars.len()
                && chars[i + 1] == '-'
                && i + 2 < chars.len()
                && chars[i + 2] != ']'
            {
                let end_byte = if chars[i + 2] == '\\' && i + 3 < chars.len() {
                    i += 2;
                    escape_to_byte(chars[i + 1])
                } else {
                    chars[i + 2] as u8
                };
                ranges.push((escaped, end_byte));
                i += 3;
            } else {
                ranges.push((escaped, escaped));
                i += 1;
            }
        } else if i + 2 < chars.len() && chars[i + 1] == '-' && chars[i + 2] != ']' {
            // Range like a-z
            let start = ch as u8;
            let end = chars[i + 2] as u8;
            ranges.push((start, end));
            i += 3;
        } else {
            ranges.push((ch as u8, ch as u8));
            i += 1;
        }
    }

    if i >= chars.len() {
        return Err(FindReplaceError::RegexCompile {
            message: "Unmatched [".to_string(),
        });
    }

    // Skip closing ]
    i += 1;

    Ok((CharClass { ranges, negated }, i))
}

pub(super) fn escape_to_byte(ch: char) -> u8 {
    match ch {
        'n' => b'\n',
        'r' => b'\r',
        't' => b'\t',
        'a' => 0x07,
        'f' => 0x0C,
        'v' => 0x0B,
        _ => ch as u8,
    }
}

pub(super) fn hex_val(ch: char) -> Option<u8> {
    match ch {
        '0'..='9' => Some(ch as u8 - b'0'),
        'a'..='f' => Some(ch as u8 - b'a' + 10),
        'A'..='F' => Some(ch as u8 - b'A' + 10),
        _ => None,
    }
}
