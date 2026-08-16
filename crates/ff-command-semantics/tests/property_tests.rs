//! Property-based tests for ff-command-semantics.
//!
//! These tests validate invariant properties across all input spaces
//! using the proptest framework with a minimum of 100 iterations per property.

use proptest::prelude::*;

use ff_command_semantics::scope::{ScopeCandidate, ScopeLines};
use ff_command_semantics::{
    CommandConfig, CommandToken, LineCommandDescriptor, LineCommandKind, LineCommandParser,
    ParsedCommand, PrimaryCommandParser, QuoteStyle, ScopeResolver, ScopeSource, StatusMessage,
};

// ─── Strategies ──────────────────────────────────────────────────────────────

/// Generate a valid command name (1–15 uppercase ASCII letters).
fn command_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z]{1,15}".prop_map(|s| s)
}

/// Generate a bare word argument (alphanumeric, no spaces, no quotes).
fn bare_word_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_]{1,20}".prop_map(|s| s)
}

/// Generate content for a quoted string (no unescaped quotes).
fn quoted_content_strategy(quote_char: char) -> impl Strategy<Value = String> {
    let safe_chars: String = (32u8..127u8)
        .filter(|&c| c as char != quote_char)
        .map(|c| c as char)
        .collect();
    proptest::collection::vec(
        proptest::sample::select(safe_chars.chars().collect::<Vec<_>>()),
        0..30,
    )
    .prop_map(|chars| chars.into_iter().collect::<String>())
}

/// Generate a hex literal (even number of hex digits, 0-10 bytes).
fn hex_bytes_strategy() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(0u8..=255u8, 0..10)
}

/// Generate a CommandToken (any variant).
fn command_token_strategy() -> impl Strategy<Value = CommandToken> {
    prop_oneof![
        bare_word_strategy().prop_map(CommandToken::Word),
        quoted_content_strategy('\'').prop_map(|v| CommandToken::QuotedString {
            value: v,
            quote_style: QuoteStyle::Single,
        }),
        quoted_content_strategy('"').prop_map(|v| CommandToken::QuotedString {
            value: v,
            quote_style: QuoteStyle::Double,
        }),
        hex_bytes_strategy().prop_map(CommandToken::HexLiteral),
    ]
}

/// Generate a valid line command kind.
fn line_command_kind_strategy() -> impl Strategy<Value = LineCommandKind> {
    prop_oneof![
        Just(LineCommandKind::Copy),
        Just(LineCommandKind::Move),
        Just(LineCommandKind::Delete),
        Just(LineCommandKind::Repeat),
        Just(LineCommandKind::Exclude),
        Just(LineCommandKind::Insert),
        Just(LineCommandKind::After),
        Just(LineCommandKind::Before),
        Just(LineCommandKind::Overlay),
        Just(LineCommandKind::Show),
        Just(LineCommandKind::Select),
        Just(LineCommandKind::Tag),
        Just(LineCommandKind::ShiftRight),
        Just(LineCommandKind::ShiftLeft),
        Just(LineCommandKind::IndentIn),
        Just(LineCommandKind::IndentOut),
        Just(LineCommandKind::Bounds),
        Just(LineCommandKind::CopyBlock),
        Just(LineCommandKind::MoveBlock),
        Just(LineCommandKind::DeleteBlock),
        Just(LineCommandKind::RepeatBlock),
        Just(LineCommandKind::ExcludeBlock),
        Just(LineCommandKind::TagBlock),
    ]
}

/// Generate a ScopeSource variant.
fn scope_source_strategy() -> impl Strategy<Value = ScopeSource> {
    prop_oneof![
        Just(ScopeSource::ExplicitRange),
        Just(ScopeSource::BlockSource),
        Just(ScopeSource::SingleLineCommand),
        Just(ScopeSource::TaggedModifier),
        Just(ScopeSource::VisibilityModifier),
        Just(ScopeSource::CursorLine),
        Just(ScopeSource::EntireDocument),
    ]
}

/// Generate whitespace-only strings.
fn whitespace_strategy() -> impl Strategy<Value = String> {
    proptest::collection::vec(proptest::sample::select(vec![' ', '\t', '\r', '\n']), 0..50)
        .prop_map(|chars| chars.into_iter().collect::<String>())
}

// ─── Property 1: Primary Command Parser Round-Trip ───────────────────────────

proptest! {
    /// **Validates: Requirement 3.6**
    ///
    /// Feature: command-semantics, Property 1: Primary command parser round-trip property
    ///
    /// For any valid command line input, parsing and reconstructing SHALL produce
    /// output that re-parses to the same token sequence.
    #[test]
    fn primary_parser_round_trip(
        name in command_name_strategy(),
        args in proptest::collection::vec(command_token_strategy(), 0..8),
    ) {
        // Build the command
        let command = ParsedCommand::Command {
            name: name.clone(),
            args: args.clone(),
        };

        // Reconstruct text from parsed command
        let reconstructed = PrimaryCommandParser::reconstruct(&command);

        // Re-parse the reconstructed text
        let reparsed = PrimaryCommandParser::parse(&reconstructed).unwrap();

        // Invariant: parse(reconstruct(command)) == command
        prop_assert_eq!(reparsed, command);
    }
}

// ─── Property 2: Case-Insensitive Normalization ──────────────────────────────

proptest! {
    /// **Validates: Requirement 3.4**
    ///
    /// Feature: command-semantics, Property 2: Primary command parser case-insensitive normalization
    ///
    /// Any case variation of the same characters resolves to the identical
    /// normalized (uppercase) command name.
    #[test]
    fn primary_parser_case_normalization(
        base_name in "[a-zA-Z]{1,15}",
    ) {
        let upper = base_name.to_uppercase();
        let lower = base_name.to_lowercase();

        let parsed_upper = PrimaryCommandParser::parse(&upper).unwrap();
        let parsed_lower = PrimaryCommandParser::parse(&lower).unwrap();
        let parsed_original = PrimaryCommandParser::parse(&base_name).unwrap();

        // All should normalize to the same uppercase name
        let name_upper = match &parsed_upper {
            ParsedCommand::Command { name, .. } => name.clone(),
            _ => panic!("expected Command"),
        };
        let name_lower = match &parsed_lower {
            ParsedCommand::Command { name, .. } => name.clone(),
            _ => panic!("expected Command"),
        };
        let name_original = match &parsed_original {
            ParsedCommand::Command { name, .. } => name.clone(),
            _ => panic!("expected Command"),
        };

        prop_assert_eq!(&name_upper, &name_lower);
        prop_assert_eq!(&name_upper, &name_original);
        prop_assert_eq!(&name_upper, &upper.to_uppercase());
    }
}

// ─── Property 3: Line Command Parser Kind/Count Decomposition ────────────────

proptest! {
    /// **Validates: Requirement 4.5**
    ///
    /// Feature: command-semantics, Property 3: Line command parser kind/count decomposition
    ///
    /// For any valid alphabetic kind followed by digits, the parser unambiguously
    /// splits into kind and count, and reconstructing matches the original (normalized).
    #[test]
    fn line_command_kind_count_decomposition(
        kind in line_command_kind_strategy(),
        count in 1u32..=99999u32,
    ) {
        // Only test alphabetic kinds for this property (special chars don't have count suffix)
        let kind_str = kind.as_str();
        if !kind_str.chars().all(|c| c.is_ascii_alphabetic()) {
            // Skip non-alphabetic kinds (>, <, (, ), ])
            return Ok(());
        }

        let input = format!("{}{}", kind_str, count);
        let result = LineCommandParser::parse(&input).unwrap().unwrap();

        match result {
            LineCommandDescriptor::Known { kind: parsed_kind, count: parsed_count } => {
                prop_assert_eq!(parsed_kind, kind);
                prop_assert_eq!(parsed_count, count);
            }
            LineCommandDescriptor::Unknown(_) => {
                prop_assert!(false, "Expected Known variant for valid kind '{}'", kind_str);
            }
        }
    }
}

// ─── Property 4: Line Command Parser Count Range Validation ──────────────────

proptest! {
    /// **Validates: Requirement 4.7**
    ///
    /// Feature: command-semantics, Property 4: Line command parser count range validation
    ///
    /// Counts in [1, 99999] succeed, counts > 99999 produce an error.
    #[test]
    fn line_command_count_range_validation(
        count in 1u64..200000u64,
    ) {
        let input = format!("D{}", count);
        let result = LineCommandParser::parse(&input);

        if count <= 99999 {
            // Should succeed
            let desc = result.unwrap().unwrap();
            match desc {
                LineCommandDescriptor::Known { count: parsed_count, .. } => {
                    prop_assert_eq!(parsed_count, count as u32);
                }
                _ => prop_assert!(false, "Expected Known variant"),
            }
        } else {
            // Should return error
            prop_assert!(result.is_err(), "Expected error for count {}", count);
        }
    }
}

// ─── Property 5: Scope Resolution Priority Ordering ──────────────────────────

proptest! {
    /// **Validates: Requirement 2.1, 2.9**
    ///
    /// Feature: command-semantics, Property 5: Scope resolution priority ordering
    ///
    /// The resolver always selects the highest-priority source, regardless
    /// of presentation order.
    #[test]
    fn scope_resolution_priority_ordering(
        sources in proptest::collection::vec(scope_source_strategy(), 2..5),
    ) {
        // Deduplicate sources
        let mut unique_sources: Vec<ScopeSource> = Vec::new();
        for s in &sources {
            if !unique_sources.contains(s) {
                unique_sources.push(*s);
            }
        }

        if unique_sources.len() < 2 {
            return Ok(());
        }

        // Build candidates in original order
        let candidates: Vec<ScopeCandidate> = unique_sources
            .iter()
            .map(|&source| ScopeCandidate {
                source,
                lines: ScopeLines::CursorLine(0),
            })
            .collect();

        // Build candidates in reversed order
        let mut reversed_candidates = candidates.clone();
        reversed_candidates.reverse();

        let result_forward = ScopeResolver::resolve(&candidates, None, false).unwrap();
        let result_reversed = ScopeResolver::resolve(&reversed_candidates, None, false).unwrap();

        // The result should be the same regardless of order
        prop_assert_eq!(result_forward.source, result_reversed.source);

        // The result should be the highest priority (lowest ordinal)
        let expected = unique_sources.iter().copied().min().unwrap();
        prop_assert_eq!(result_forward.source, expected);
    }
}

// ─── Property 6: Status Message Length Invariant ─────────────────────────────

proptest! {
    /// **Validates: Requirement 5.4**
    ///
    /// Feature: command-semantics, Property 6: Status message length invariant
    ///
    /// All StatusMessages are at most 200 characters. If the original would exceed
    /// 200, the message ends with "...".
    #[test]
    fn status_message_length_invariant(
        command_name in "[A-Z]{1,50}",
        description in ".{1,500}",
        severity in 0u8..3u8,
    ) {
        let msg = match severity {
            0 => StatusMessage::syntax_error(&command_name, &description),
            1 => StatusMessage::structure_error(&command_name, &description),
            _ => StatusMessage::runtime_error(&command_name, &description),
        };

        // Invariant: length is always ≤ 200
        prop_assert!(
            msg.text.len() <= 200,
            "Message length {} exceeds 200: {:?}",
            msg.text.len(),
            msg.text
        );

        // If the formatted message would have exceeded 200, it must end with "..."
        let untruncated = match severity {
            0 => format!("Syntax error in {}: {}", command_name, description),
            1 => format!("Structure error in {}: {}", command_name, description),
            _ => format!("Error in {}: {}", command_name, description),
        };

        if untruncated.len() > 200 {
            prop_assert!(
                msg.text.ends_with("..."),
                "Truncated message should end with '...': {:?}",
                msg.text
            );
        }
    }
}

// ─── Property 7: Configuration Clamping ──────────────────────────────────────

proptest! {
    /// **Validates: Requirement 6.2, 6.6**
    ///
    /// Feature: command-semantics, Property 7: Configuration clamping
    ///
    /// `commands.default_shift_width` is always clamped to [1, 72].
    #[test]
    fn configuration_clamping(
        input in -1000i64..1000i64,
    ) {
        let effective = CommandConfig::clamp_shift_width(input);

        // Invariant: effective value is always in [1, 72]
        prop_assert!(effective >= 1, "effective {} < 1 for input {}", effective, input);
        prop_assert!(effective <= 72, "effective {} > 72 for input {}", effective, input);

        // If input is in range, value is unchanged
        if input >= 1 && input <= 72 {
            prop_assert_eq!(effective, input as u32);
        }

        // If input < 1, clamped to 1
        if input < 1 {
            prop_assert_eq!(effective, 1);
        }

        // If input > 72, clamped to 72
        if input > 72 {
            prop_assert_eq!(effective, 72);
        }
    }
}

// ─── Property 8: Empty Input Detection ───────────────────────────────────────

proptest! {
    /// **Validates: Requirement 3.5, 4.6**
    ///
    /// Feature: command-semantics, Property 8: Empty input detection
    ///
    /// For any whitespace-only or empty string, both parsers return None/Empty.
    #[test]
    fn empty_input_detection(
        whitespace in whitespace_strategy(),
    ) {
        // Primary parser returns Empty for whitespace-only input
        let primary_result = PrimaryCommandParser::parse(&whitespace).unwrap();
        prop_assert_eq!(primary_result, ParsedCommand::Empty);

        // Line command parser returns None for whitespace-only input
        let line_result = LineCommandParser::parse(&whitespace).unwrap();
        prop_assert_eq!(line_result, None);
    }
}
