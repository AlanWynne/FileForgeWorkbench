//! NFA-based regular expression engine with group capture.
//!
//! Supports POSIX-like syntax with extensions: character classes,
//! lazy/greedy quantifiers, backreferences, and anchors.
//!
//! Addresses: Requirements 4, 12

use crate::case_folder::CaseFolder;
use crate::error::FindReplaceError;
use crate::indexer::CharacterIndexer;
use crate::request::WordMatchMode;
use crate::result::FindResult;
use crate::types::{BytePosition, MatchRange};
use crate::word_boundary::check_word_boundary;

/// Maximum number of NFA instructions allowed.
const DEFAULT_MAX_NFA_SIZE: usize = 10_000;

/// Default step limit per position to prevent catastrophic backtracking.
const DEFAULT_STEP_LIMIT: u64 = 10_000;

/// Maximum number of capture groups (0 = full match, 1–9 = sub-groups).
const MAX_GROUPS: usize = 10;

/// NFA instruction set.
#[derive(Debug, Clone, PartialEq, Eq)]
enum NfaInstruction {
    /// Match a specific byte.
    Literal(u8),
    /// Match any character except newline.
    AnyChar,
    /// Match a character in a character class (index into classes vec).
    CharClass(usize),
    /// Anchor check.
    Anchor(AnchorKind),
    /// Split execution: greedy (try first path first).
    Split { first: usize, second: usize },
    /// Split execution: lazy (try second path first).
    SplitLazy { first: usize, second: usize },
    /// Unconditional jump.
    Jump(usize),
    /// Start of capture group.
    GroupStart(u8),
    /// End of capture group.
    GroupEnd(u8),
    /// Backreference to group N.
    BackRef(u8),
    /// Successful match.
    Match,
}

/// Anchor types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnchorKind {
    LineStart,
    LineEnd,
    WordBoundary,
    WordStart,
    WordEnd,
}

/// A character class definition (set of byte ranges).
#[derive(Debug, Clone)]
struct CharClass {
    /// Ranges of bytes included in the class.
    ranges: Vec<(u8, u8)>,
    /// Whether this class is negated ([^...]).
    negated: bool,
}

impl CharClass {
    fn matches(&self, byte: u8) -> bool {
        let in_set = self.ranges.iter().any(|&(lo, hi)| byte >= lo && byte <= hi);
        if self.negated {
            !in_set
        } else {
            in_set
        }
    }
}

/// Compiled NFA ready for execution.
///
/// Addresses: Requirement 12 AC 1
#[derive(Debug, Clone)]
pub struct CompiledRegex {
    instructions: Vec<NfaInstruction>,
    classes: Vec<CharClass>,
    #[allow(dead_code)]
    group_count: u8,
    /// Optional literal prefix for fast-path scanning.
    pub literal_prefix: Option<Vec<u8>>,
}

/// NFA-based regular expression engine.
///
/// Addresses: Requirements 4, 12
pub struct RegexEngine {
    last_compiled: Option<CompiledRegex>,
    max_nfa_size: usize,
    step_limit: u64,
}

impl RegexEngine {
    /// Create with default limits.
    pub fn new() -> Self {
        Self {
            last_compiled: None,
            max_nfa_size: DEFAULT_MAX_NFA_SIZE,
            step_limit: DEFAULT_STEP_LIMIT,
        }
    }

    /// Create with custom limits.
    pub fn with_limits(max_nfa_size: usize, step_limit: u64) -> Self {
        Self {
            last_compiled: None,
            max_nfa_size,
            step_limit,
        }
    }

    /// Compile a regex pattern into NFA bytecode.
    ///
    /// Addresses: Requirement 12 AC 1–9
    pub fn compile(&mut self, pattern: &str) -> Result<&CompiledRegex, FindReplaceError> {
        if pattern.is_empty() {
            return match &self.last_compiled {
                Some(_) => Ok(self.last_compiled.as_ref().unwrap()),
                None => Err(FindReplaceError::NoPreviousRegex),
            };
        }

        let compiled = compile_pattern(pattern, self.max_nfa_size)?;
        self.last_compiled = Some(compiled);
        Ok(self.last_compiled.as_ref().unwrap())
    }

    /// Get the last compiled regex (if any).
    pub fn last_compiled(&self) -> Option<&CompiledRegex> {
        self.last_compiled.as_ref()
    }

    /// Execute the compiled regex forward from start within [start, end).
    ///
    /// Addresses: Requirement 12 AC 10–13
    pub fn execute_forward(
        &self,
        compiled: &CompiledRegex,
        indexer: &dyn CharacterIndexer,
        start: BytePosition,
        end: BytePosition,
        case_folder: Option<&CaseFolder>,
        word_mode: WordMatchMode,
    ) -> Option<FindResult> {
        let mut pos = start.0;
        while pos <= end.0 {
            if let Some(result) = try_match_at(
                compiled,
                indexer,
                BytePosition(pos),
                end,
                case_folder,
                self.step_limit,
            ) {
                // Validate word boundaries
                if check_word_boundary(
                    word_mode,
                    result.match_range.start,
                    result.match_range.end,
                    indexer,
                ) {
                    return Some(result);
                }
            }
            pos += 1;
            if pos > end.0 {
                break;
            }
        }
        None
    }

    /// Execute in reverse (backward search).
    pub fn execute_backward(
        &self,
        compiled: &CompiledRegex,
        indexer: &dyn CharacterIndexer,
        start: BytePosition,
        end: BytePosition,
        case_folder: Option<&CaseFolder>,
        word_mode: WordMatchMode,
    ) -> Option<FindResult> {
        // For backward search, try positions from end-1 down to start
        if end.0 == 0 {
            return None;
        }
        let mut pos = end.0 - 1;
        loop {
            if let Some(result) = try_match_at(
                compiled,
                indexer,
                BytePosition(pos),
                end,
                case_folder,
                self.step_limit,
            ) {
                if check_word_boundary(
                    word_mode,
                    result.match_range.start,
                    result.match_range.end,
                    indexer,
                ) {
                    return Some(result);
                }
            }
            if pos == start.0 {
                break;
            }
            pos -= 1;
        }
        None
    }

    /// Find all non-overlapping matches within a range.
    pub fn find_all(
        &self,
        compiled: &CompiledRegex,
        indexer: &dyn CharacterIndexer,
        start: BytePosition,
        end: BytePosition,
        case_folder: Option<&CaseFolder>,
        word_mode: WordMatchMode,
    ) -> Vec<FindResult> {
        let mut results = Vec::new();
        let mut pos = start.0;

        while pos <= end.0 {
            if let Some(result) = try_match_at(
                compiled,
                indexer,
                BytePosition(pos),
                end,
                case_folder,
                self.step_limit,
            ) {
                if check_word_boundary(
                    word_mode,
                    result.match_range.start,
                    result.match_range.end,
                    indexer,
                ) {
                    let next = if result.match_range.is_empty() {
                        pos + 1
                    } else {
                        result.match_range.end.0
                    };
                    results.push(result);
                    pos = next;
                } else {
                    pos += 1;
                }
            } else {
                pos += 1;
            }
        }

        results
    }
}

impl Default for RegexEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Compile a pattern string into NFA instructions.
fn compile_pattern(pattern: &str, max_size: usize) -> Result<CompiledRegex, FindReplaceError> {
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

/// Apply a quantifier (*, +, ?) to the last instruction.
fn apply_quantifier(
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
fn parse_char_class(chars: &[char]) -> Result<(CharClass, usize), FindReplaceError> {
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

fn escape_to_byte(ch: char) -> u8 {
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

fn hex_val(ch: char) -> Option<u8> {
    match ch {
        '0'..='9' => Some(ch as u8 - b'0'),
        'a'..='f' => Some(ch as u8 - b'a' + 10),
        'A'..='F' => Some(ch as u8 - b'A' + 10),
        _ => None,
    }
}

/// Try to match the compiled regex at a specific position.
fn try_match_at(
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
fn execute_nfa(
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

fn is_word_byte_nfa(byte: Option<u8>) -> bool {
    match byte {
        Some(b) => matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_'),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::SliceIndexer;

    #[test]
    fn compile_simple_literal_pattern() {
        let mut engine = RegexEngine::new();
        let compiled = engine.compile("abc").unwrap();
        assert!(compiled.literal_prefix.is_some());
        assert_eq!(
            compiled.literal_prefix.as_ref().unwrap(),
            &vec![b'a', b'b', b'c']
        );
    }

    #[test]
    fn compile_rejects_unmatched_opening_paren() {
        let mut engine = RegexEngine::new();
        let err = engine.compile("(abc").unwrap_err();
        assert!(err.to_string().contains("Unmatched ("));
    }

    #[test]
    fn compile_rejects_unmatched_closing_paren() {
        let mut engine = RegexEngine::new();
        let err = engine.compile("abc)").unwrap_err();
        assert!(err.to_string().contains("Unmatched )"));
    }

    #[test]
    fn compile_rejects_empty_closure() {
        let mut engine = RegexEngine::new();
        let err = engine.compile("*abc").unwrap_err();
        assert!(err.to_string().contains("Empty closure"));
    }

    #[test]
    fn empty_pattern_reuses_previous() {
        let mut engine = RegexEngine::new();
        engine.compile("abc").unwrap();
        let compiled = engine.compile("").unwrap();
        assert!(compiled.literal_prefix.is_some());
    }

    #[test]
    fn empty_pattern_with_no_previous_returns_error() {
        let mut engine = RegexEngine::new();
        let err = engine.compile("").unwrap_err();
        assert!(matches!(err, FindReplaceError::NoPreviousRegex));
    }

    #[test]
    fn execute_simple_literal_match() {
        let mut engine = RegexEngine::new();
        let compiled = engine.compile("world").unwrap().clone();
        let indexer = SliceIndexer::from_str("hello world");
        let result = engine.execute_forward(
            &compiled,
            &indexer,
            BytePosition(0),
            BytePosition(11),
            None,
            WordMatchMode::None,
        );
        let r = result.unwrap();
        assert_eq!(r.match_range.start, BytePosition(6));
        assert_eq!(r.match_range.end, BytePosition(11));
    }

    #[test]
    fn execute_dot_metacharacter_matches_any_non_newline() {
        let mut engine = RegexEngine::new();
        let compiled = engine.compile("h.llo").unwrap().clone();
        let indexer = SliceIndexer::from_str("hello");
        let result = engine.execute_forward(
            &compiled,
            &indexer,
            BytePosition(0),
            BytePosition(5),
            None,
            WordMatchMode::None,
        );
        assert!(result.is_some());
    }

    #[test]
    fn execute_character_class() {
        let mut engine = RegexEngine::new();
        let compiled = engine.compile("[abc]").unwrap().clone();
        let indexer = SliceIndexer::from_str("xbz");
        let result = engine.execute_forward(
            &compiled,
            &indexer,
            BytePosition(0),
            BytePosition(3),
            None,
            WordMatchMode::None,
        );
        let r = result.unwrap();
        assert_eq!(r.match_range.start, BytePosition(1));
    }

    #[test]
    fn execute_captures_groups() {
        let mut engine = RegexEngine::new();
        let compiled = engine.compile("(ab)(cd)").unwrap().clone();
        let indexer = SliceIndexer::from_str("xabcdy");
        let result = engine.execute_forward(
            &compiled,
            &indexer,
            BytePosition(0),
            BytePosition(6),
            None,
            WordMatchMode::None,
        );
        let r = result.unwrap();
        assert_eq!(r.match_range.start, BytePosition(1));
        assert_eq!(r.match_range.end, BytePosition(5));
        assert_eq!(r.captures.len(), 2);
        assert_eq!(
            r.captures[0],
            MatchRange::new(BytePosition(1), BytePosition(3))
        );
        assert_eq!(
            r.captures[1],
            MatchRange::new(BytePosition(3), BytePosition(5))
        );
    }

    #[test]
    fn execute_undetermined_reference_error() {
        let mut engine = RegexEngine::new();
        let err = engine.compile("\\1").unwrap_err();
        assert!(err.to_string().contains("Undetermined reference"));
    }

    #[test]
    fn execute_digit_class_shorthand() {
        let mut engine = RegexEngine::new();
        let compiled = engine.compile("\\d+").unwrap().clone();
        let indexer = SliceIndexer::from_str("abc123def");
        let result = engine.execute_forward(
            &compiled,
            &indexer,
            BytePosition(0),
            BytePosition(9),
            None,
            WordMatchMode::None,
        );
        let r = result.unwrap();
        assert_eq!(r.match_range.start, BytePosition(3));
        assert_eq!(r.match_range.end, BytePosition(6));
    }
}
