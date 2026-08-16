//! ASA auto-detection engine.
//!
//! Examines the first column of a file's lines to determine whether the file
//! contains ASA carriage control characters. The heuristic checks that a
//! sufficient percentage of first-column characters match the valid ASA set
//! and that at least one page eject (`1`) is present.

use crate::control::ASA_VALID_CHARS;

/// Configuration for the ASA detection heuristic.
///
/// Controls the sensitivity and scope of the detection algorithm.
// Validates: Requirement 2.6
#[derive(Debug, Clone, PartialEq)]
pub struct DetectionConfig {
    /// Minimum ratio of valid ASA chars for positive detection.
    /// Range: 0.5–1.0. Default: 0.8.
    pub threshold: f64,
    /// Number of non-blank lines to sample.
    /// Range: 10–500. Default: 50.
    pub sample_size: usize,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            threshold: 0.8,
            sample_size: 50,
        }
    }
}

/// Result of the ASA auto-detection heuristic.
// Validates: Requirement 2.1–2.3
#[derive(Debug, Clone, PartialEq)]
pub struct DetectionResult {
    /// Whether the file is classified as ASA-controlled.
    pub is_asa: bool,
    /// Confidence ratio (0.0–1.0) of valid ASA characters in column 1.
    pub confidence: f64,
    /// Whether at least one page eject (`1`) was found in the sample.
    pub has_page_eject: bool,
    /// Number of non-blank lines actually sampled.
    pub lines_sampled: usize,
    /// Whether detection was forced by RECFM metadata.
    pub forced_by_recfm: bool,
}

/// Run the ASA auto-detection heuristic on a set of lines.
///
/// Examines column 1 of the first N non-blank lines and determines
/// whether the file likely contains ASA carriage control characters.
///
/// Classification rule: file is ASA-controlled when:
/// - confidence >= threshold AND
/// - at least one `1` (page eject) character is present
// Validates: Requirement 2.1, 2.2
pub fn detect_asa(lines: &[&str], config: &DetectionConfig) -> DetectionResult {
    if lines.is_empty() {
        return DetectionResult {
            is_asa: false,
            confidence: 0.0,
            has_page_eject: false,
            lines_sampled: 0,
            forced_by_recfm: false,
        };
    }

    let mut valid_count: usize = 0;
    let mut has_page_eject = false;
    let mut sampled = 0;

    for line in lines.iter() {
        if line.is_empty() {
            continue;
        }
        if sampled >= config.sample_size {
            break;
        }
        sampled += 1;

        let first_char = line.chars().next().unwrap_or(' ');
        if ASA_VALID_CHARS.contains(&first_char) {
            valid_count += 1;
        }
        if first_char == '1' {
            has_page_eject = true;
        }
    }

    let confidence = if sampled > 0 {
        valid_count as f64 / sampled as f64
    } else {
        0.0
    };

    let is_asa = confidence >= config.threshold && has_page_eject;

    DetectionResult {
        is_asa,
        confidence,
        has_page_eject,
        lines_sampled: sampled,
        forced_by_recfm: false,
    }
}

/// Check if a file should be treated as ASA based on RECFM metadata.
///
/// Returns true for "FBA" or "VBA" record formats, which unconditionally
/// indicate ASA carriage control presence.
// Validates: Requirement 2.3
pub fn is_asa_by_recfm(recfm: &str) -> bool {
    let upper = recfm.trim().to_uppercase();
    upper == "FBA" || upper == "VBA"
}

/// Create a detection result forced by RECFM metadata.
// Validates: Requirement 2.3
pub fn detect_by_recfm(recfm: &str) -> DetectionResult {
    DetectionResult {
        is_asa: is_asa_by_recfm(recfm),
        confidence: 1.0,
        has_page_eject: true,
        lines_sampled: 0,
        forced_by_recfm: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Validates: Requirement 2.1
    fn detect_empty_file_returns_not_asa() {
        let result = detect_asa(&[], &DetectionConfig::default());
        assert!(!result.is_asa);
        assert_eq!(result.confidence, 0.0);
        assert_eq!(result.lines_sampled, 0);
    }

    #[test]
    // Validates: Requirement 2.2
    fn detect_file_with_all_valid_asa_and_page_eject_is_asa() {
        let lines: Vec<&str> = vec![
            "1REPORT TITLE",
            " DATA LINE 1",
            " DATA LINE 2",
            "0DOUBLE SPACED",
            "-TRIPLE SPACED",
            "+OVERPRINT",
            "1PAGE 2",
            " MORE DATA",
        ];
        let result = detect_asa(&lines, &DetectionConfig::default());
        assert!(result.is_asa);
        assert_eq!(result.confidence, 1.0);
        assert!(result.has_page_eject);
        assert_eq!(result.lines_sampled, 8);
    }

    #[test]
    // Validates: Requirement 2.2
    fn detect_file_below_threshold_is_not_asa() {
        let lines: Vec<&str> = vec!["AHELLO", "BWORLD", "CTEST", " DATA", "1PAGE"];
        let result = detect_asa(&lines, &DetectionConfig::default());
        // 2 out of 5 valid = 0.4 confidence, below 0.8 threshold
        assert!(!result.is_asa);
        assert!(result.confidence < 0.8);
    }

    #[test]
    // Validates: Requirement 2.2
    fn detect_file_with_high_confidence_but_no_page_eject_is_not_asa() {
        let lines: Vec<&str> = vec![
            " DATA LINE 1",
            " DATA LINE 2",
            " DATA LINE 3",
            "0DOUBLE SPACE",
            " DATA LINE 4",
        ];
        let result = detect_asa(&lines, &DetectionConfig::default());
        assert!(!result.is_asa);
        assert!(result.confidence >= 0.8);
        assert!(!result.has_page_eject);
    }

    #[test]
    // Validates: Requirement 2.6
    fn detect_respects_sample_size_limit() {
        let lines: Vec<&str> = vec![
            "1PAGE1", " A", " B", " C", " D", " E", " F", " G", " H", " I",
        ];
        let config = DetectionConfig {
            threshold: 0.8,
            sample_size: 3,
        };
        let result = detect_asa(&lines, &config);
        assert_eq!(result.lines_sampled, 3);
    }

    #[test]
    // Validates: Requirement 2.6
    fn detect_skips_empty_lines() {
        let lines: Vec<&str> = vec!["", "1PAGE", "", " DATA", "", " MORE"];
        let result = detect_asa(&lines, &DetectionConfig::default());
        assert_eq!(result.lines_sampled, 3);
        assert!(result.is_asa);
    }

    #[test]
    // Validates: Requirement 2.3
    fn is_asa_by_recfm_recognises_fba_and_vba() {
        assert!(is_asa_by_recfm("FBA"));
        assert!(is_asa_by_recfm("VBA"));
        assert!(is_asa_by_recfm("fba"));
        assert!(is_asa_by_recfm("vba"));
        assert!(is_asa_by_recfm(" FBA "));
        assert!(!is_asa_by_recfm("FB"));
        assert!(!is_asa_by_recfm("VB"));
        assert!(!is_asa_by_recfm(""));
    }

    #[test]
    // Validates: Requirement 2.3
    fn detect_by_recfm_forces_asa_for_fba() {
        let result = detect_by_recfm("FBA");
        assert!(result.is_asa);
        assert!(result.forced_by_recfm);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    // Validates: Requirement 2.6
    fn detect_with_custom_threshold() {
        let lines: Vec<&str> = vec!["1PAGE", " DATA", " MORE", "XINVALID", " OK"];
        // 4/5 valid = 0.8. With threshold 0.9, should fail
        let config = DetectionConfig {
            threshold: 0.9,
            sample_size: 50,
        };
        let result = detect_asa(&lines, &config);
        assert!(!result.is_asa);

        // With threshold 0.7, should pass
        let config = DetectionConfig {
            threshold: 0.7,
            sample_size: 50,
        };
        let result = detect_asa(&lines, &config);
        assert!(result.is_asa);
    }
}
