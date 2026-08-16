//! Sequence number detection engine.
//!
//! Samples file content and determines whether sequence numbers are present
//! in defined column ranges using configurable heuristic rules.
//! The detection algorithm is purely read-only — it never modifies the edit buffer.

use crate::config::SeqNumConfig;
use crate::traits::{DocumentAccess, LanguageProfile};
use crate::types::{ColumnRange, DetectedFormat, DetectionResult};

/// Samples file content to detect sequence number presence.
///
/// Read-only operation — never modifies the edit buffer or source file.
#[derive(Debug)]
pub struct SequenceDetector {
    config: SeqNumConfig,
}

/// Full detection outcome for a document, covering both front and back ranges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullDetectionResult {
    /// Detection result for the front column range.
    pub front: DetectionResult,
    /// Detection result for the back column range.
    pub back: DetectionResult,
    /// The front column range that was checked.
    pub front_columns: Option<ColumnRange>,
    /// The back column range that was checked.
    pub back_columns: Option<ColumnRange>,
    /// Detected format for front columns (if present).
    pub front_format: Option<DetectedFormat>,
    /// Detected format for back columns (if present).
    pub back_format: Option<DetectedFormat>,
    /// Number of non-blank lines sampled.
    pub lines_sampled: usize,
}

impl SequenceDetector {
    /// Create a new detector with the given configuration.
    pub fn new(config: &SeqNumConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Update configuration (on hot-reload).
    pub fn update_config(&mut self, config: &SeqNumConfig) {
        self.config = config.clone();
    }

    /// Detect sequence numbers in the given document using the language profile.
    ///
    /// Evaluates front and back column ranges independently.
    /// Returns `FullDetectionResult` with detection status for each range.
    pub fn detect(
        &self,
        document: &dyn DocumentAccess,
        profile: &dyn LanguageProfile,
    ) -> FullDetectionResult {
        let front_cols = profile.sequence_cols_front();
        let back_cols = profile.sequence_cols_back();

        // Collect non-blank lines (up to sample_size)
        let lines = self.sample_lines(document);
        let lines_sampled = lines.len();

        let (front_result, front_format) = if let Some(ref cols) = front_cols {
            self.detect_range(&lines, cols)
        } else {
            (DetectionResult::Absent, None)
        };

        let (back_result, back_format) = if let Some(ref cols) = back_cols {
            self.detect_range(&lines, cols)
        } else {
            (DetectionResult::Absent, None)
        };

        FullDetectionResult {
            front: front_result,
            back: back_result,
            front_columns: front_cols,
            back_columns: back_cols,
            front_format,
            back_format,
            lines_sampled,
        }
    }

    /// Detect sequence numbers in a specific column range from pre-collected lines.
    ///
    /// Returns the detection result and detected format.
    pub fn detect_range(
        &self,
        lines: &[&str],
        range: &ColumnRange,
    ) -> (DetectionResult, Option<DetectedFormat>) {
        if lines.is_empty() {
            return (DetectionResult::Absent, None);
        }

        let threshold = self.effective_threshold(lines.len());
        let mut match_count = 0;
        let mut has_all_digit_line = false;
        let mut alpha_prefix: Option<String> = None;
        let mut alpha_match_count = 0;

        for line in lines {
            let col_content = self.extract_column_content(line, range);
            if let Some(content) = col_content {
                if self.is_numeric_match(&content) {
                    match_count += 1;
                    if content.chars().all(|c| c.is_ascii_digit()) {
                        has_all_digit_line = true;
                    }
                }
                // Check alpha-prefix pattern
                if let Some(prefix) = self.extract_alpha_prefix(&content) {
                    match &alpha_prefix {
                        None => {
                            alpha_prefix = Some(prefix);
                            alpha_match_count = 1;
                        }
                        Some(existing) if *existing == prefix => {
                            alpha_match_count += 1;
                        }
                        _ => {} // Different prefix, don't count
                    }
                }
            }
        }

        let required_matches = (lines.len() as f64 * threshold as f64 / 100.0).ceil() as usize;

        // Check pure numeric detection first
        if match_count >= required_matches && has_all_digit_line {
            return (DetectionResult::Present, Some(DetectedFormat::Numeric));
        }

        // Check alphanumeric prefix detection
        if alpha_match_count >= required_matches {
            if let Some(prefix) = alpha_prefix {
                return (
                    DetectionResult::Present,
                    Some(DetectedFormat::AlphaPrefix { prefix }),
                );
            }
        }

        (DetectionResult::Absent, None)
    }

    /// Sample up to `sample_size` non-blank lines from the document.
    fn sample_lines<'a>(&self, document: &'a dyn DocumentAccess) -> Vec<&'a str> {
        let mut lines = Vec::new();
        let total = document.line_count();
        for i in 0..total {
            if lines.len() >= self.config.sample_size as usize {
                break;
            }
            if let Some(content) = document.line_content(i) {
                if !content.trim().is_empty() {
                    lines.push(content);
                }
            }
        }
        lines
    }

    /// Calculate the effective threshold percentage.
    /// For files with fewer than 5 non-blank lines, requires 100% match.
    fn effective_threshold(&self, sample_count: usize) -> u8 {
        if sample_count < 5 {
            100
        } else {
            self.config.detection_threshold
        }
    }

    /// Extract the content of a column range from a line.
    /// Returns None if the line is shorter than the start of the range.
    fn extract_column_content(&self, line: &str, range: &ColumnRange) -> Option<String> {
        let start = range.start_offset();
        let end = range.end_offset();
        if line.len() < end {
            return None;
        }
        Some(line[start..end].to_string())
    }

    /// Check if a column content string matches the numeric criterion:
    /// all characters are digits (0-9) or spaces, with the string not being all spaces.
    fn is_numeric_match(&self, content: &str) -> bool {
        if content.chars().all(|c| c == ' ') {
            return false;
        }
        content.chars().all(|c| c.is_ascii_digit() || c == ' ')
    }

    /// Extract an alphabetic prefix from content if the content follows
    /// the pattern: alphabetic prefix + digits.
    fn extract_alpha_prefix(&self, content: &str) -> Option<String> {
        if content.is_empty() {
            return None;
        }
        let prefix_end = content
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .count();
        if prefix_end == 0 || prefix_end >= content.len() {
            return None;
        }
        let remainder = &content[prefix_end..];
        if remainder.chars().all(|c| c.is_ascii_digit()) && !remainder.is_empty() {
            Some(content[..prefix_end].to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Test Helpers ───────────────────────────────────────────────────────

    struct MockDocument {
        lines: Vec<String>,
    }

    impl MockDocument {
        fn new(lines: &[&str]) -> Self {
            Self {
                lines: lines.iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl DocumentAccess for MockDocument {
        fn line_count(&self) -> usize {
            self.lines.len()
        }

        fn line_content(&self, index: usize) -> Option<&str> {
            self.lines.get(index).map(|s| s.as_str())
        }
    }

    struct MockProfile {
        front: Option<ColumnRange>,
        back: Option<ColumnRange>,
        auto_unnum: bool,
    }

    impl LanguageProfile for MockProfile {
        fn sequence_cols_front(&self) -> Option<ColumnRange> {
            self.front
        }

        fn sequence_cols_back(&self) -> Option<ColumnRange> {
            self.back
        }

        fn auto_unnum(&self) -> bool {
            self.auto_unnum
        }

        fn language_id(&self) -> &str {
            "test"
        }
    }

    fn cobol_profile() -> MockProfile {
        MockProfile {
            front: Some(ColumnRange::new(1, 6).unwrap()),
            back: Some(ColumnRange::new(73, 80).unwrap()),
            auto_unnum: true,
        }
    }

    fn make_cobol_line(seq_front: &str, body: &str, seq_back: &str) -> String {
        // COBOL: cols 1-6 (front), cols 7-72 (body), cols 73-80 (back)
        let front = format!("{:<6}", seq_front);
        let body_padded = format!("{:<66}", body);
        let back = format!("{:<8}", seq_back);
        format!("{}{}{}", &front[..6], &body_padded[..66], &back[..8])
    }

    // ─── Detection Tests ────────────────────────────────────────────────────

    #[test]
    fn detect_cobol_with_valid_sequence_numbers() {
        // Validates: Requirements 2.1, 2.2
        let lines: Vec<String> = (1..=10)
            .map(|i| {
                make_cobol_line(
                    &format!("{:06}", i * 100),
                    " MOVE A TO B.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let doc = MockDocument::new(&line_refs);
        let config = SeqNumConfig::default();
        let detector = SequenceDetector::new(&config);
        let result = detector.detect(&doc, &cobol_profile());

        assert_eq!(result.front, DetectionResult::Present);
        assert_eq!(result.back, DetectionResult::Present);
    }

    #[test]
    fn detect_file_without_sequence_numbers() {
        // Validates: Requirements 2.1, 2.2
        let lines: Vec<String> = (1..=10)
            .map(|_| make_cobol_line("      ", " MOVE A TO B.", "        "))
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let doc = MockDocument::new(&line_refs);
        let config = SeqNumConfig::default();
        let detector = SequenceDetector::new(&config);
        let result = detector.detect(&doc, &cobol_profile());

        assert_eq!(result.front, DetectionResult::Absent);
        assert_eq!(result.back, DetectionResult::Absent);
    }

    #[test]
    fn detect_short_file_requires_100_percent() {
        // Validates: Requirement 2.3
        // 4 lines (< 5), one doesn't match → should be Absent
        let mut lines: Vec<String> = (1..=3)
            .map(|i| {
                make_cobol_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        lines.push(make_cobol_line("ABCDEF", " CODE.", "COMMENTS"));
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let doc = MockDocument::new(&line_refs);
        let config = SeqNumConfig::default();
        let detector = SequenceDetector::new(&config);
        let result = detector.detect(&doc, &cobol_profile());

        assert_eq!(result.front, DetectionResult::Absent);
    }

    #[test]
    fn detect_short_file_all_matching() {
        // Validates: Requirement 2.3
        let lines: Vec<String> = (1..=4)
            .map(|i| {
                make_cobol_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let doc = MockDocument::new(&line_refs);
        let config = SeqNumConfig::default();
        let detector = SequenceDetector::new(&config);
        let result = detector.detect(&doc, &cobol_profile());

        assert_eq!(result.front, DetectionResult::Present);
        assert_eq!(result.back, DetectionResult::Present);
    }

    #[test]
    fn detect_lines_shorter_than_range_do_not_match() {
        // Validates: Requirement 2.5
        let range = ColumnRange::new(73, 80).unwrap();
        let config = SeqNumConfig::default();
        let detector = SequenceDetector::new(&config);

        // Lines that are only 50 chars long — shorter than col 73
        let short_lines: Vec<&str> = vec![
            "      MOVE A TO B.                                ",
            "      MOVE C TO D.                                ",
            "      MOVE E TO F.                                ",
            "      MOVE G TO H.                                ",
            "      MOVE I TO J.                                ",
        ];
        let (result, _) = detector.detect_range(&short_lines, &range);
        assert_eq!(result, DetectionResult::Absent);
    }

    #[test]
    fn detect_front_and_back_independently() {
        // Validates: Requirement 2.4
        // Front has seq nums, back does not
        let lines: Vec<String> = (1..=10)
            .map(|i| make_cobol_line(&format!("{:06}", i * 100), " CODE.", "COMMENTS"))
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let doc = MockDocument::new(&line_refs);
        let config = SeqNumConfig::default();
        let detector = SequenceDetector::new(&config);
        let result = detector.detect(&doc, &cobol_profile());

        assert_eq!(result.front, DetectionResult::Present);
        assert_eq!(result.back, DetectionResult::Absent);
    }

    #[test]
    fn detect_alphanumeric_prefix_pattern() {
        // Validates: Requirement 2.9
        let range = ColumnRange::new(73, 80).unwrap();
        let config = SeqNumConfig::default();
        let detector = SequenceDetector::new(&config);

        // Build lines that are exactly 80 chars with alpha-prefix seq nums in cols 73-80
        let lines: Vec<String> = (1..=6)
            .map(|i| {
                let body = format!("{:<72}", format!("      MOVE LINE{i} TO DEST."));
                let seq = format!("ABC{:05}", i * 100);
                format!("{}{}", &body[..72], seq)
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let (result, format) = detector.detect_range(&line_refs, &range);
        assert_eq!(result, DetectionResult::Present);
        assert_eq!(
            format,
            Some(DetectedFormat::AlphaPrefix {
                prefix: "ABC".to_string()
            })
        );
    }

    #[test]
    fn detect_read_only_guarantee() {
        // Validates: Requirement 2.7
        let lines: Vec<String> = (1..=10)
            .map(|i| {
                make_cobol_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let doc = MockDocument::new(&line_refs);
        let config = SeqNumConfig::default();
        let detector = SequenceDetector::new(&config);

        // Store original content
        let original_lines: Vec<String> = (0..doc.line_count())
            .map(|i| doc.line_content(i).unwrap().to_string())
            .collect();

        let _result = detector.detect(&doc, &cobol_profile());

        // Verify document is unchanged
        for (i, original) in original_lines.iter().enumerate() {
            assert_eq!(doc.line_content(i).unwrap(), original.as_str());
        }
    }

    #[test]
    fn detect_no_columns_defined_returns_absent() {
        // Validates: Requirement 1.9
        let lines: Vec<String> = (1..=10)
            .map(|i| {
                make_cobol_line(
                    &format!("{:06}", i * 100),
                    " CODE.",
                    &format!("{:08}", i * 100),
                )
            })
            .collect();
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let doc = MockDocument::new(&line_refs);
        let config = SeqNumConfig::default();
        let detector = SequenceDetector::new(&config);

        let no_cols_profile = MockProfile {
            front: None,
            back: None,
            auto_unnum: true,
        };
        let result = detector.detect(&doc, &no_cols_profile);

        assert_eq!(result.front, DetectionResult::Absent);
        assert_eq!(result.back, DetectionResult::Absent);
    }

    #[test]
    fn detect_empty_document() {
        // Validates: Requirement 2.1
        let doc = MockDocument::new(&[]);
        let config = SeqNumConfig::default();
        let detector = SequenceDetector::new(&config);
        let result = detector.detect(&doc, &cobol_profile());

        assert_eq!(result.front, DetectionResult::Absent);
        assert_eq!(result.back, DetectionResult::Absent);
        assert_eq!(result.lines_sampled, 0);
    }
}
