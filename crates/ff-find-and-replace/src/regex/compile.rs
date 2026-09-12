use crate::error::FindReplaceError;

use super::parse::{apply_quantifier, hex_val, parse_char_class};
use super::{AnchorKind, CharClass, CompiledRegex, NfaInstruction, MAX_GROUPS};

/// Compile a pattern string into NFA instructions.
pub(super) fn compile_pattern(
    pattern: &str,
    max_size: usize,
) -> Result<CompiledRegex, FindReplaceError> {
    let mut instructions: Vec<NfaInstruction> = Vec::new();
    let mut classes: Vec<CharClass> = Vec::new();
    let mut group_count: u8 = 0;
    let mut group_stack: Vec<u8> = Vec::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    let mut literal_prefix: Option<Vec<u8>> = Some(Vec::new());

    // Track whether we're still building the literal prefix
    let mut prefix_done = false;

    instructions.push(NfaInstruction::GroupStart(0));

    while i < chars.len() {
        if instructions.len() > max_size {
            return Err(FindReplaceError::RegexPatternTooLong);
        }

        let ch = chars[i];
        match ch {
            '.' => {
                prefix_done = true;
                instructions.push(NfaInstruction::AnyChar);
                i += 1;
            }
            '^' => {
                prefix_done = true;
                instructions.push(NfaInstruction::Anchor(AnchorKind::LineStart));
                i += 1;
            }
            '$' => {
                prefix_done = true;
                instructions.push(NfaInstruction::Anchor(AnchorKind::LineEnd));
                i += 1;
            }
            '(' => {
                prefix_done = true;
                group_count += 1;
                if group_count >= MAX_GROUPS as u8 {
                    return Err(FindReplaceError::RegexCompile {
                        message: "too many groups (max 9)".to_string(),
                    });
                }
                group_stack.push(group_count);
                instructions.push(NfaInstruction::GroupStart(group_count));
                i += 1;
            }
            ')' => {
                match group_stack.pop() {
                    Some(g) => instructions.push(NfaInstruction::GroupEnd(g)),
                    None => {
                        return Err(FindReplaceError::RegexCompile {
                            message: "Unmatched )".to_string(),
                        })
                    }
                }
                i += 1;
            }

            '[' => {
                prefix_done = true;
                let (class, consumed) = parse_char_class(&chars[i..])?;
                let class_idx = classes.len();
                classes.push(class);
                instructions.push(NfaInstruction::CharClass(class_idx));
                i += consumed;
            }
            '\\' => {
                i += 1;
                if i >= chars.len() {
                    return Err(FindReplaceError::RegexCompile {
                        message: "trailing backslash".to_string(),
                    });
                }
                let escaped = chars[i];
                match escaped {
                    'd' => {
                        prefix_done = true;
                        let class_idx = classes.len();
                        classes.push(CharClass {
                            ranges: vec![(b'0', b'9')],
                            negated: false,
                        });
                        instructions.push(NfaInstruction::CharClass(class_idx));
                    }
                    'D' => {
                        prefix_done = true;
                        let class_idx = classes.len();
                        classes.push(CharClass {
                            ranges: vec![(b'0', b'9')],
                            negated: true,
                        });
                        instructions.push(NfaInstruction::CharClass(class_idx));
                    }
                    's' => {
                        prefix_done = true;
                        let class_idx = classes.len();
                        classes.push(CharClass {
                            ranges: vec![
                                (b' ', b' '),
                                (b'\t', b'\t'),
                                (b'\n', b'\n'),
                                (b'\r', b'\r'),
                                (0x0C, 0x0C),
                                (0x0B, 0x0B),
                            ],
                            negated: false,
                        });
                        instructions.push(NfaInstruction::CharClass(class_idx));
                    }
                    'S' => {
                        prefix_done = true;
                        let class_idx = classes.len();
                        classes.push(CharClass {
                            ranges: vec![
                                (b' ', b' '),
                                (b'\t', b'\t'),
                                (b'\n', b'\n'),
                                (b'\r', b'\r'),
                                (0x0C, 0x0C),
                                (0x0B, 0x0B),
                            ],
                            negated: true,
                        });
                        instructions.push(NfaInstruction::CharClass(class_idx));
                    }

                    'w' => {
                        prefix_done = true;
                        let class_idx = classes.len();
                        classes.push(CharClass {
                            ranges: vec![(b'a', b'z'), (b'A', b'Z'), (b'0', b'9'), (b'_', b'_')],
                            negated: false,
                        });
                        instructions.push(NfaInstruction::CharClass(class_idx));
                    }
                    'W' => {
                        prefix_done = true;
                        let class_idx = classes.len();
                        classes.push(CharClass {
                            ranges: vec![(b'a', b'z'), (b'A', b'Z'), (b'0', b'9'), (b'_', b'_')],
                            negated: true,
                        });
                        instructions.push(NfaInstruction::CharClass(class_idx));
                    }
                    'b' => {
                        prefix_done = true;
                        instructions.push(NfaInstruction::Anchor(AnchorKind::WordBoundary));
                    }
                    '<' => {
                        prefix_done = true;
                        instructions.push(NfaInstruction::Anchor(AnchorKind::WordStart));
                    }
                    '>' => {
                        prefix_done = true;
                        instructions.push(NfaInstruction::Anchor(AnchorKind::WordEnd));
                    }
                    'x' => {
                        // \xHH hex escape
                        prefix_done = true;
                        if i + 2 < chars.len() {
                            let h = hex_val(chars[i + 1]);
                            let l = hex_val(chars[i + 2]);
                            match (h, l) {
                                (Some(hv), Some(lv)) => {
                                    instructions.push(NfaInstruction::Literal((hv << 4) | lv));
                                    i += 2;
                                }
                                _ => {
                                    instructions.push(NfaInstruction::Literal(b'x'));
                                }
                            }
                        } else {
                            instructions.push(NfaInstruction::Literal(b'x'));
                        }
                    }

                    'a' => {
                        instructions.push(NfaInstruction::Literal(0x07));
                        prefix_done = true;
                    }
                    'f' => {
                        instructions.push(NfaInstruction::Literal(0x0C));
                        prefix_done = true;
                    }
                    'n' => {
                        instructions.push(NfaInstruction::Literal(b'\n'));
                        prefix_done = true;
                    }
                    'r' => {
                        instructions.push(NfaInstruction::Literal(b'\r'));
                        prefix_done = true;
                    }
                    't' => {
                        instructions.push(NfaInstruction::Literal(b'\t'));
                        prefix_done = true;
                    }
                    'v' => {
                        instructions.push(NfaInstruction::Literal(0x0B));
                        prefix_done = true;
                    }
                    '1'..='9' => {
                        prefix_done = true;
                        let group_ref = escaped as u8 - b'0';
                        if group_ref > group_count {
                            return Err(FindReplaceError::RegexCompile {
                                message: "Undetermined reference".to_string(),
                            });
                        }
                        // Check for cyclical reference
                        if group_stack.contains(&group_ref) {
                            return Err(FindReplaceError::RegexCompile {
                                message: "Cyclical reference".to_string(),
                            });
                        }
                        instructions.push(NfaInstruction::BackRef(group_ref));
                    }
                    _ => {
                        // Escaped literal
                        let byte = escaped as u8;
                        if !prefix_done {
                            if let Some(ref mut pf) = literal_prefix {
                                pf.push(byte);
                            }
                        }
                        instructions.push(NfaInstruction::Literal(byte));
                    }
                }
                i += 1;
            }

            '*' | '+' | '?' => {
                // Quantifier applied to previous instruction
                if instructions.len() <= 1 {
                    return Err(FindReplaceError::RegexCompile {
                        message: "Empty closure".to_string(),
                    });
                }
                prefix_done = true;

                // Check if lazy variant
                let lazy = if i + 1 < chars.len() && chars[i + 1] == '?' {
                    i += 1;
                    true
                } else {
                    false
                };

                let last_instr_idx = instructions.len() - 1;
                apply_quantifier(&mut instructions, last_instr_idx, ch, lazy);
                i += 1;
            }
            _ => {
                // Literal character
                let byte = ch as u8;
                if !prefix_done && ch.is_ascii() {
                    if let Some(ref mut pf) = literal_prefix {
                        pf.push(byte);
                    }
                } else {
                    prefix_done = true;
                }
                instructions.push(NfaInstruction::Literal(byte));
                i += 1;
            }
        }
    }

    if !group_stack.is_empty() {
        return Err(FindReplaceError::RegexCompile {
            message: "Unmatched (".to_string(),
        });
    }

    instructions.push(NfaInstruction::GroupEnd(0));
    instructions.push(NfaInstruction::Match);

    let prefix = match literal_prefix {
        Some(ref p) if !p.is_empty() => Some(p.clone()),
        _ => None,
    };

    Ok(CompiledRegex {
        instructions,
        classes,
        group_count,
        literal_prefix: prefix,
    })
}
