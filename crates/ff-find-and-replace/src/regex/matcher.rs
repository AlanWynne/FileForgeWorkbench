use crate::case_folder::CaseFolder;
use crate::indexer::CharacterIndexer;
use crate::result::FindResult;
use crate::types::{BytePosition, MatchRange};

use super::{AnchorKind, CharClass, CompiledRegex, NfaInstruction, MAX_GROUPS};

/// Try to match the compiled regex at a specific position.
pub(super) fn try_match_at(
    compiled: &CompiledRegex,
    indexer: &dyn CharacterIndexer,
    pos: BytePosition,
    end: BytePosition,
    _case_folder: Option<&CaseFolder>,
    step_limit: u64,
) -> Option<FindResult> {
    let mut captures: Vec<Option<(u64, u64)>> = vec![None; MAX_GROUPS];
    let mut steps: u64 = 0;

    if execute_nfa(
        &compiled.instructions,
        &compiled.classes,
        indexer,
        pos.0,
        end.0,
        0, // start at instruction 0
        &mut captures,
        &mut steps,
        step_limit,
        _case_folder,
    ) {
        // Build FindResult from captures
        let full_match = captures[0]?;
        let match_start = BytePosition(full_match.0);
        let match_end = BytePosition(full_match.1);
        let line = indexer.line_from_position(match_start);

        let mut capture_ranges = Vec::new();
        capture_ranges.extend(
            captures.iter().skip(1).filter_map(|cap| {
                cap.map(|(s, e)| MatchRange::new(BytePosition(s), BytePosition(e)))
            }),
        );

        Some(FindResult::with_captures(
            match_start,
            match_end,
            line,
            capture_ranges,
        ))
    } else {
        None
    }
}

/// Recursive NFA execution with backtracking.
#[allow(clippy::too_many_arguments)]
pub(super) fn execute_nfa(
    instructions: &[NfaInstruction],
    classes: &[CharClass],
    indexer: &dyn CharacterIndexer,
    mut pos: u64,
    end: u64,
    mut pc: usize,
    captures: &mut Vec<Option<(u64, u64)>>,
    steps: &mut u64,
    step_limit: u64,
    case_folder: Option<&CaseFolder>,
) -> bool {
    loop {
        *steps += 1;
        if *steps > step_limit {
            return false;
        }

        if pc >= instructions.len() {
            return false;
        }

        match &instructions[pc] {
            NfaInstruction::Match => return true,
            NfaInstruction::Literal(expected) => {
                if pos >= end {
                    return false;
                }
                let byte = match indexer.char_at(BytePosition(pos)) {
                    Some(b) => b,
                    None => return false,
                };
                let matches = if let Some(_cf) = case_folder {
                    // Case-insensitive comparison
                    let doc_lower = (byte as char).to_ascii_lowercase() as u8;
                    let pat_lower = (*expected as char).to_ascii_lowercase() as u8;
                    doc_lower == pat_lower
                } else {
                    byte == *expected
                };
                if !matches {
                    return false;
                }
                pos += 1;
                pc += 1;
            }
            NfaInstruction::AnyChar => {
                if pos >= end {
                    return false;
                }
                match indexer.char_at(BytePosition(pos)) {
                    Some(b'\n') => return false, // . doesn't match newline
                    Some(_) => {}
                    None => return false,
                }
                pos += 1;
                pc += 1;
            }

            NfaInstruction::CharClass(idx) => {
                if pos >= end {
                    return false;
                }
                let byte = match indexer.char_at(BytePosition(pos)) {
                    Some(b) => b,
                    None => return false,
                };
                if !classes[*idx].matches(byte) {
                    return false;
                }
                pos += 1;
                pc += 1;
            }
            NfaInstruction::Anchor(kind) => {
                match kind {
                    AnchorKind::LineStart => {
                        if pos > 0 {
                            match indexer.char_at(BytePosition(pos - 1)) {
                                Some(b'\n') => {}
                                _ => return false,
                            }
                        }
                    }
                    AnchorKind::LineEnd => match indexer.char_at(BytePosition(pos)) {
                        Some(b'\n') | None => {}
                        _ => return false,
                    },
                    AnchorKind::WordBoundary => {
                        let before_word = if pos > 0 {
                            is_word_byte_nfa(indexer.char_at(BytePosition(pos - 1)))
                        } else {
                            false
                        };
                        let at_word = is_word_byte_nfa(indexer.char_at(BytePosition(pos)));
                        if before_word == at_word {
                            return false;
                        }
                    }
                    AnchorKind::WordStart => {
                        let before_word = if pos > 0 {
                            is_word_byte_nfa(indexer.char_at(BytePosition(pos - 1)))
                        } else {
                            false
                        };
                        let at_word = is_word_byte_nfa(indexer.char_at(BytePosition(pos)));
                        if before_word || !at_word {
                            return false;
                        }
                    }
                    AnchorKind::WordEnd => {
                        let before_word = if pos > 0 {
                            is_word_byte_nfa(indexer.char_at(BytePosition(pos - 1)))
                        } else {
                            false
                        };
                        let at_word = is_word_byte_nfa(indexer.char_at(BytePosition(pos)));
                        if !before_word || at_word {
                            return false;
                        }
                    }
                }
                pc += 1;
            }

            NfaInstruction::Split { first, second } => {
                let saved_captures = captures.clone();
                // Try first path (greedy)
                if execute_nfa(
                    instructions,
                    classes,
                    indexer,
                    pos,
                    end,
                    *first,
                    captures,
                    steps,
                    step_limit,
                    case_folder,
                ) {
                    return true;
                }
                // Restore and try second
                *captures = saved_captures;
                pc = *second;
            }
            NfaInstruction::SplitLazy { first, second } => {
                let saved_captures = captures.clone();
                // Try first path (skip/after - lazy prefers shorter)
                if execute_nfa(
                    instructions,
                    classes,
                    indexer,
                    pos,
                    end,
                    *first,
                    captures,
                    steps,
                    step_limit,
                    case_folder,
                ) {
                    return true;
                }
                // Restore and try second (consume)
                *captures = saved_captures;
                pc = *second;
            }
            NfaInstruction::Jump(target) => {
                pc = *target;
            }
            NfaInstruction::GroupStart(g) => {
                let idx = *g as usize;
                if idx < captures.len() {
                    // Save start position
                    captures[idx] = Some((pos, pos));
                }
                pc += 1;
            }
            NfaInstruction::GroupEnd(g) => {
                let idx = *g as usize;
                if idx < captures.len() {
                    if let Some((start, _)) = captures[idx] {
                        captures[idx] = Some((start, pos));
                    }
                }
                pc += 1;
            }
            NfaInstruction::BackRef(g) => {
                let idx = *g as usize;
                if idx >= captures.len() {
                    return false;
                }
                match captures[idx] {
                    Some((cap_start, cap_end)) => {
                        let cap_len = cap_end - cap_start;
                        // Match captured text at current position
                        for offset in 0..cap_len {
                            let cap_byte = indexer.char_at(BytePosition(cap_start + offset));
                            let doc_byte = indexer.char_at(BytePosition(pos + offset));
                            match (cap_byte, doc_byte) {
                                (Some(a), Some(b)) if a == b => {}
                                _ => return false,
                            }
                        }
                        pos += cap_len;
                        pc += 1;
                    }
                    None => return false,
                }
            }
        }
    }
}

pub(super) fn is_word_byte_nfa(byte: Option<u8>) -> bool {
    match byte {
        Some(b) => matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_'),
        None => false,
    }
}
