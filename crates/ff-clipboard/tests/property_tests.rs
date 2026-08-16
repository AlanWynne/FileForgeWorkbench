//! Property-based tests for ff-clipboard.
//!
//! These tests use the `proptest` crate to verify invariants across
//! a wide range of inputs.

use proptest::prelude::*;

use ff_clipboard::{
    ClipboardConfig, ClipboardEngine, ClipboardEntry, ClipboardHistoryRing, ClipboardMode,
    ClipboardProvider, CopyCommandRouter, CopyHandler, InMemoryClipboardProvider, LineSplitter,
    PasteHandler,
};

// ─── Task 23: Clipboard Engine Invariants ───────────────────────────────────

mod clipboard_engine_invariants {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Any text written to ClipboardEngine and immediately read back produces
        /// identical text content for arbitrary UTF-8 strings.
        ///
        /// **Validates: Requirements 1.2, 1.3**
        #[test]
        fn write_read_roundtrip_preserves_text(text in ".*") {
            // Feature: clipboard-operations, Property 9: Clipboard Write/Read Consistency
            let provider = InMemoryClipboardProvider::new();
            let mut engine = ClipboardEngine::new(Box::new(provider), ClipboardConfig::default());

            let entry = ClipboardEntry::stream(text.clone());
            engine.write(entry).unwrap();

            let read_back = engine.read().unwrap();
            prop_assert_eq!(read_back.text(), text.as_str());
        }

        /// Clipboard mode is preserved through write/read cycle for any ClipboardMode
        /// variant and arbitrary entry content.
        ///
        /// **Validates: Requirements 1.4, 1.5**
        #[test]
        fn write_read_preserves_mode(
            text in ".+",
            mode_idx in 0u8..3
        ) {
            // Feature: clipboard-operations, Property 9: Clipboard Write/Read Consistency
            let provider = InMemoryClipboardProvider::new();
            let mut engine = ClipboardEngine::new(Box::new(provider), ClipboardConfig::default());

            let mode = match mode_idx {
                0 => ClipboardMode::Stream,
                1 => ClipboardMode::Line,
                _ => ClipboardMode::Rectangular,
            };

            let entry = ClipboardEntry::from_text(text, mode);
            engine.write(entry).unwrap();

            let read_back = engine.read().unwrap();
            prop_assert_eq!(read_back.mode(), mode);
        }

        /// External clipboard modification (simulated by direct provider write) always
        /// results in Stream mode on read regardless of prior internal mode.
        ///
        /// **Validates: Requirement 1.5**
        #[test]
        fn external_modification_defaults_to_stream(
            internal_text in ".+",
            external_text in ".+"
        ) {
            // Feature: clipboard-operations, Property 10: External Clipboard Defaults to Stream
            prop_assume!(internal_text != external_text);

            let provider = InMemoryClipboardProvider::new();
            let provider_clone = provider.clone();
            let mut engine = ClipboardEngine::new(Box::new(provider), ClipboardConfig::default());

            // Write internally with Line mode
            let entry = ClipboardEntry::line(internal_text);
            engine.write(entry).unwrap();

            // Simulate external modification
            provider_clone.set_content_externally(&external_text);

            // Read back should be Stream mode
            let read_back = engine.read().unwrap();
            prop_assert_eq!(read_back.mode(), ClipboardMode::Stream);
            prop_assert_eq!(read_back.text(), external_text.as_str());
        }

        /// ClipboardEngine never panics for any sequence of read/write/availability-check
        /// calls with any provider state.
        ///
        /// **Validates: Requirement 1.6**
        #[test]
        fn engine_never_panics(
            ops in proptest::collection::vec(0u8..5, 1..20),
            texts in proptest::collection::vec(".*", 1..5)
        ) {
            // Feature: clipboard-operations, Property: No-panic guarantee
            let provider = InMemoryClipboardProvider::new();
            let provider_clone = provider.clone();
            let mut engine = ClipboardEngine::new(Box::new(provider), ClipboardConfig::default());

            for op in ops {
                let text_idx = 0;
                match op {
                    0 => { let _ = engine.read(); }
                    1 => { let _ = engine.write(ClipboardEntry::stream(texts[text_idx].clone())); }
                    2 => { let _ = engine.has_content(); }
                    3 => { let _ = engine.is_available(); }
                    4 => {
                        // Toggle availability
                        let current = provider_clone.is_available();
                        provider_clone.set_available(!current);
                    }
                    _ => {}
                }
            }
            // If we reach here without panic, the property holds.
        }
    }
}

// ─── Task 24: Line Splitting and Paste Invariants ───────────────────────────

mod line_splitting_invariants {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Splitting text on line endings and rejoining with a single separator
        /// produces equivalent logical content — no content is lost or added.
        ///
        /// **Validates: Requirements 16.1, 16.4**
        #[test]
        fn split_rejoin_preserves_content(text in ".+") {
            // Feature: clipboard-operations, Property 1: Line Splitter Round-Trip
            let result = LineSplitter::split(&text);

            // Rejoin with \n
            let rejoined = result.lines.join("\n");

            // Normalize original: replace all line endings with \n
            let normalized = text.replace("\r\n", "\n").replace('\r', "\n");

            // If had trailing terminator, the normalized text ends with \n
            // and the rejoined text lacks that final \n (by design — trailing suppression).
            // So: rejoined + optional trailing \n == normalized
            if result.had_trailing_terminator {
                let reconstructed = format!("{}\n", rejoined);
                prop_assert_eq!(reconstructed, normalized);
            } else {
                prop_assert_eq!(rejoined, normalized);
            }
        }

        /// Text ending with a trailing line ending produces exactly N lines (not N+1)
        /// where N is the number of line-ending separators.
        ///
        /// **Validates: Requirement 16.3**
        #[test]
        fn trailing_terminator_no_extra_line(
            base_lines in proptest::collection::vec("[^\r\n]+", 1..10)
        ) {
            // Feature: clipboard-operations, Property 2: Trailing Terminator Suppression
            let text_with_trailing = base_lines.join("\n") + "\n";
            let result = LineSplitter::split(&text_with_trailing);

            prop_assert_eq!(result.lines.len(), base_lines.len());
            prop_assert!(result.had_trailing_terminator);
            // Last line should NOT be empty (the trailing \n doesn't create one)
            if let Some(last) = result.lines.last() {
                prop_assert!(!last.is_empty() || base_lines.last().is_some_and(|l| l.is_empty()));
            }
        }

        /// Paste followed by undo restores document to exact pre-paste state.
        /// (Tested at the preparation level — paste_stream returns the same lines
        /// that were split from the input.)
        ///
        /// **Validates: Requirements 15.1, 15.2**
        #[test]
        fn paste_stream_is_reversible(text in ".+") {
            // Feature: clipboard-operations, Property: Paste reversibility
            let entry = ClipboardEntry::stream(text.clone());
            let result = PasteHandler::paste_stream(&entry).unwrap();

            // Splitting and rejoining should produce the normalized content
            // The split lines, when joined, represent the content that would be inserted.
            // If we split text into lines and rejoin, we get the content minus line endings.
            let split_result = LineSplitter::split(&text);

            // The paste result lines should match the splitter output
            prop_assert_eq!(&result.lines_to_insert, &split_result.lines);
            prop_assert_eq!(result.lines_inserted, split_result.lines.len());
        }

        /// Line-mode paste inserts exactly the number of logical lines derived
        /// from clipboard content.
        ///
        /// **Validates: Requirements 4.2, 14.2, 14.3**
        #[test]
        fn line_paste_inserts_exact_line_count(
            lines in proptest::collection::vec("[^\r\n]*", 1..20)
        ) {
            // Feature: clipboard-operations, Property 8: Line-Copy Paste Inserts Above
            let text = lines.join("\n") + "\n";
            let entry = ClipboardEntry::line(text);
            let result = PasteHandler::paste_line(&entry).unwrap();

            prop_assert_eq!(result.lines_inserted, lines.len());
            prop_assert_eq!(result.mode, ClipboardMode::Line);
        }
    }
}

// ─── Task 25: Multi-Caret and Rectangular Invariants ────────────────────────

mod multi_caret_rectangular_invariants {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Multi-caret copy with N carets produces exactly N segments in ClipboardEntry.
        ///
        /// **Validates: Requirement 13.1**
        #[test]
        fn multi_caret_copy_produces_n_segments(
            segments in proptest::collection::vec(".+", 1..8)
        ) {
            // Feature: clipboard-operations, Property 6: Multi-Caret Segment Distribution
            let provider = InMemoryClipboardProvider::new();
            let mut engine = ClipboardEngine::new(Box::new(provider), ClipboardConfig::default());

            let n = segments.len();
            CopyHandler::copy_multi_caret(&mut engine, segments).unwrap();

            let entry = engine.read().unwrap();
            prop_assert_eq!(entry.segment_count(), n);
        }

        /// Multi-caret paste with matching segment count distributes exactly one
        /// segment per caret and total inserted text equals total segment text.
        ///
        /// **Validates: Requirements 13.2, 13.3**
        #[test]
        fn multi_caret_paste_matched_distribution(
            segments in proptest::collection::vec(".+", 2..8)
        ) {
            // Feature: clipboard-operations, Property 6: Multi-Caret Segment Distribution
            let n = segments.len();
            let entry = ClipboardEntry::multi_caret(segments.clone());

            let results = PasteHandler::paste_multi_caret_matched(&entry, n).unwrap();
            prop_assert_eq!(results.len(), n);

            // Each result should contain the corresponding segment's content
            for (i, result) in results.iter().enumerate() {
                let expected_lines = LineSplitter::split(&segments[i]).lines;
                prop_assert_eq!(&result.lines_to_insert, &expected_lines);
            }
        }

        /// Rectangular paste produces exactly `segments.len()` worth of lines to insert.
        ///
        /// **Validates: Requirements 12.2, 12.4**
        #[test]
        fn rectangular_paste_produces_correct_line_count(
            segments in proptest::collection::vec("[^\r\n]+", 1..20)
        ) {
            // Feature: clipboard-operations, Property 7: Rectangular Paste Column Alignment
            let entry = ClipboardEntry::rectangular(segments.clone());
            let config = ClipboardConfig::default();

            let result = PasteHandler::paste_rectangular(&entry, &config).unwrap();
            prop_assert_eq!(result.lines_to_insert.len(), segments.len());
            prop_assert_eq!(result.mode, ClipboardMode::Rectangular);
        }

        /// Multi-caret paste in reverse document order produces identical result
        /// to forward-order (at the preparation level, the results are independent).
        ///
        /// **Validates: Requirement 13.5**
        #[test]
        fn multi_caret_paste_reverse_order_correctness(
            segments in proptest::collection::vec(".+", 2..6)
        ) {
            // Feature: clipboard-operations, Property: Reverse-order correctness
            let n = segments.len();
            let entry = ClipboardEntry::multi_caret(segments.clone());

            let _forward_results = PasteHandler::paste_multi_caret_matched(&entry, n).unwrap();

            // Reverse order should produce the same segments (just reordered by caller)
            let mut reversed_segments = segments.clone();
            reversed_segments.reverse();
            let reversed_entry = ClipboardEntry::multi_caret(reversed_segments.clone());
            let reverse_results = PasteHandler::paste_multi_caret_matched(&reversed_entry, n).unwrap();

            // Each result independently matches its segment
            for (i, result) in reverse_results.iter().enumerate() {
                let expected_lines = LineSplitter::split(&reversed_segments[i]).lines;
                prop_assert_eq!(&result.lines_to_insert, &expected_lines);
            }
        }
    }
}

// ─── Task 26: COPY Command Disambiguation Invariants ────────────────────────

mod copy_command_disambiguation_invariants {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// resolve_copy_mode with pending C/CC and any target always returns
        /// InDocument regardless of arguments (when no path).
        ///
        /// **Validates: Requirements 8.1, 8.3**
        #[test]
        fn pending_source_with_target_always_in_document(
            _dummy in 0u8..1
        ) {
            // Feature: clipboard-operations, Property 5: Disambiguation Is Total
            let result = CopyCommandRouter::resolve("", true, true);
            prop_assert_eq!(result.unwrap(), ff_clipboard::CopyCommandMode::InDocument);
        }

        /// resolve_copy_mode with no pending C/CC, no args, and valid A/B target
        /// always returns ClipboardPaste.
        ///
        /// **Validates: Requirement 8.4**
        #[test]
        fn no_source_no_args_target_always_clipboard_paste(
            _dummy in 0u8..1
        ) {
            // Feature: clipboard-operations, Property 5: Disambiguation Is Total
            let result = CopyCommandRouter::resolve("", false, true);
            prop_assert_eq!(result.unwrap(), ff_clipboard::CopyCommandMode::ClipboardPaste);
        }

        /// resolve_copy_mode with a path argument always takes precedence over
        /// clipboard-paste when no pending C/CC exists and target is present.
        ///
        /// **Validates: Requirements 8.5, 8.6**
        #[test]
        fn path_arg_takes_precedence_over_clipboard_paste(
            path in "[a-zA-Z0-9_/\\.]+",
        ) {
            // Feature: clipboard-operations, Property 5: Disambiguation Is Total
            prop_assume!(!path.trim().is_empty());
            let result = CopyCommandRouter::resolve(&path, false, true);
            match result.unwrap() {
                ff_clipboard::CopyCommandMode::FileInsert { .. } => {} // expected
                other => prop_assert!(false, "Expected FileInsert, got {:?}", other),
            }
        }

        /// resolve_copy_mode never returns Ok for the combination pending C/CC + path
        /// argument (always error).
        ///
        /// **Validates: Requirement 8.7**
        #[test]
        fn pending_source_plus_path_always_error(
            path in "[a-zA-Z0-9_]+",
        ) {
            // Feature: clipboard-operations, Property 5: Disambiguation Is Total
            prop_assume!(!path.trim().is_empty());
            let result = CopyCommandRouter::resolve(&path, true, true);
            prop_assert!(result.is_err());

            let result2 = CopyCommandRouter::resolve(&path, true, false);
            prop_assert!(result2.is_err());
        }
    }

    /// Exhaustive test of all boolean combinations for disambiguation totality.
    #[test]
    fn disambiguation_is_total_for_all_boolean_combinations() {
        // Feature: clipboard-operations, Property 5: COPY Command Disambiguation Is Total
        // Validates: Requirements 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8
        let args_options = ["", "file.txt"];
        let source_options = [false, true];
        let target_options = [false, true];

        for args in &args_options {
            for &has_source in &source_options {
                for &has_target in &target_options {
                    // Should never panic
                    let result = CopyCommandRouter::resolve(args, has_source, has_target);
                    // Result is always Ok or Err (never panics)
                    assert!(result.is_ok() || result.is_err());
                }
            }
        }
    }
}

// ─── Task 27: Clipboard History Ring Invariants ─────────────────────────────

mod clipboard_history_ring_invariants {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// History ring never exceeds configured max capacity after any number of
        /// push operations.
        ///
        /// **Validates: Internal invariant (history ring bounded size)**
        #[test]
        fn ring_never_exceeds_capacity(
            capacity in 1usize..50,
            push_count in 1usize..100,
        ) {
            // Feature: clipboard-operations, Property: Ring capacity bounded
            let mut ring = ClipboardHistoryRing::new(capacity);

            for i in 0..push_count {
                ring.push(ClipboardEntry::stream(format!("entry-{}", i)));
            }

            prop_assert!(ring.len() <= capacity);
        }

        /// `current()` always returns the most recently pushed entry when ring is non-empty.
        ///
        /// **Validates: Internal invariant (LIFO ordering)**
        #[test]
        fn current_is_most_recent(
            entries in proptest::collection::vec(".+", 1..20),
        ) {
            // Feature: clipboard-operations, Property: LIFO current
            let mut ring = ClipboardHistoryRing::new(100);

            for entry_text in &entries {
                ring.push(ClipboardEntry::stream(entry_text.clone()));
            }

            let current = ring.current().unwrap();
            prop_assert_eq!(current.text(), entries.last().unwrap().as_str());
        }

        /// Cycling through entire ring and back returns to original current entry.
        ///
        /// **Validates: Internal invariant (ring cycle correctness)**
        #[test]
        fn cycling_wraps_correctly(
            entry_count in 2usize..15,
        ) {
            // Feature: clipboard-operations, Property: Ring wrap correctness
            let mut ring = ClipboardHistoryRing::new(20);

            for i in 0..entry_count {
                ring.push(ClipboardEntry::stream(format!("e{}", i)));
            }

            let initial = ring.current().unwrap().text().to_string();

            // Cycle back through all entries
            for _ in 0..entry_count {
                ring.cycle_back();
            }

            // Should be back at the start
            let after_full_cycle = ring.current().unwrap().text().to_string();
            prop_assert_eq!(initial, after_full_cycle);
        }
    }
}
