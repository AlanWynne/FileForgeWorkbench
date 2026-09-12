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
use crate::types::BytePosition;
use crate::word_boundary::check_word_boundary;

/// Maximum number of NFA instructions allowed.
const DEFAULT_MAX_NFA_SIZE: usize = 10_000;

/// Default step limit per position to prevent catastrophic backtracking.
const DEFAULT_STEP_LIMIT: u64 = 10_000;

/// Maximum number of capture groups (0 = full match, 1-9 = sub-groups).
pub(super) const MAX_GROUPS: usize = 10;

/// NFA instruction set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum NfaInstruction {
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
pub(super) enum AnchorKind {
    LineStart,
    LineEnd,
    WordBoundary,
    WordStart,
    WordEnd,
}

/// A character class definition (set of byte ranges).
#[derive(Debug, Clone)]
pub(super) struct CharClass {
    /// Ranges of bytes included in the class.
    pub(super) ranges: Vec<(u8, u8)>,
    /// Whether this class is negated ([^...]).
    pub(super) negated: bool,
}

impl CharClass {
    pub(super) fn matches(&self, byte: u8) -> bool {
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
    pub(super) instructions: Vec<NfaInstruction>,
    pub(super) classes: Vec<CharClass>,
    #[allow(dead_code)]
    pub(super) group_count: u8,
    /// Optional literal prefix for fast-path scanning.
    pub literal_prefix: Option<Vec<u8>>,
}

/// NFA-based regular expression engine.
///
/// Addresses: Requirements 4, 12
pub struct RegexEngine {
    pub(super) last_compiled: Option<CompiledRegex>,
    pub(super) max_nfa_size: usize,
    pub(super) step_limit: u64,
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
    /// Addresses: Requirement 12 AC 1-9
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
    /// Addresses: Requirement 12 AC 10-13
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

mod compile;
mod matcher;
mod parse;

use compile::compile_pattern;
use matcher::try_match_at;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::SliceIndexer;
    use crate::types::MatchRange;

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
