//! ASA carriage control detection and display.
//!
//! Interprets column 1 of FBA/VBA records as ASA carriage control characters
//! used by IBM mainframe report spool files.

/// ASA carriage control character interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsaControl {
    /// Space — single space before printing.
    SingleSpace,
    /// '0' — double space.
    DoubleSpace,
    /// '-' — triple space.
    TripleSpace,
    /// '1' — new page (form feed).
    NewPage,
    /// '+' — overprint (no advance).
    Overprint,
    /// 'H' — halt.
    Halt,
    /// Unknown character in column 1.
    Unknown(u8),
}

impl AsaControl {
    /// Returns the 2-character display abbreviation for this control.
    pub fn abbreviation(&self) -> &'static str {
        match self {
            Self::SingleSpace => "SP",
            Self::DoubleSpace => "DS",
            Self::TripleSpace => "TS",
            Self::NewPage => "NP",
            Self::Overprint => "OP",
            Self::Halt => "HT",
            Self::Unknown(_) => "??",
        }
    }

    /// Returns a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::SingleSpace => "single space",
            Self::DoubleSpace => "double space",
            Self::TripleSpace => "triple space",
            Self::NewPage => "new page",
            Self::Overprint => "overprint",
            Self::Halt => "halt",
            Self::Unknown(_) => "unknown",
        }
    }

    /// Returns true if this is a known (valid) ASA control character.
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Unknown(_))
    }
}

/// Parses a column-1 byte as an ASA carriage control character.
pub fn parse_asa_char(byte: u8) -> AsaControl {
    match byte {
        b' ' => AsaControl::SingleSpace,
        b'0' => AsaControl::DoubleSpace,
        b'-' => AsaControl::TripleSpace,
        b'1' => AsaControl::NewPage,
        b'+' => AsaControl::Overprint,
        b'H' => AsaControl::Halt,
        _ => AsaControl::Unknown(byte),
    }
}

/// Result of ASA auto-detection analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct AsaDetectionResult {
    /// Whether ASA carriage control was detected.
    pub detected: bool,
    /// Confidence level (ratio of matching lines to total sampled lines).
    pub confidence: f32,
    /// Number of non-blank lines sampled.
    pub sample_size: usize,
}

/// Detects whether records use ASA carriage control.
///
/// Samples up to `max_sample` non-blank records and checks if >= 80% have
/// a known ASA character in column 1.
///
/// # Arguments
///
/// * `records` - Slice of record byte slices to sample
/// * `max_sample` - Maximum number of non-blank records to examine (default: 20)
pub fn detect_asa(records: &[&[u8]], max_sample: usize) -> AsaDetectionResult {
    let mut sampled = 0;
    let mut asa_matches = 0;

    for &record in records {
        if sampled >= max_sample {
            break;
        }

        // Skip blank (empty) records
        if record.is_empty() {
            continue;
        }

        // Skip records that are all whitespace
        if record.iter().all(|&b| b == b' ' || b == b'\t') {
            continue;
        }

        sampled += 1;
        let control = parse_asa_char(record[0]);
        if control.is_known() {
            asa_matches += 1;
        }
    }

    if sampled == 0 {
        return AsaDetectionResult {
            detected: false,
            confidence: 0.0,
            sample_size: 0,
        };
    }

    let confidence = asa_matches as f32 / sampled as f32;
    let detected = confidence >= 0.8;

    AsaDetectionResult {
        detected,
        confidence,
        sample_size: sampled,
    }
}

/// Strips column 1 ASA characters from all records, shifting content left.
///
/// Returns the modified records with the ASA byte removed.
pub fn strip_asa(records: &[Vec<u8>]) -> Vec<Vec<u8>> {
    records
        .iter()
        .map(|record| {
            if record.is_empty() {
                Vec::new()
            } else {
                record[1..].to_vec()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 7.4
    #[test]
    fn parse_asa_char_known_characters() {
        assert_eq!(parse_asa_char(b' '), AsaControl::SingleSpace);
        assert_eq!(parse_asa_char(b'0'), AsaControl::DoubleSpace);
        assert_eq!(parse_asa_char(b'-'), AsaControl::TripleSpace);
        assert_eq!(parse_asa_char(b'1'), AsaControl::NewPage);
        assert_eq!(parse_asa_char(b'+'), AsaControl::Overprint);
        assert_eq!(parse_asa_char(b'H'), AsaControl::Halt);
    }

    #[test]
    fn parse_asa_char_unknown_character() {
        let control = parse_asa_char(b'X');
        assert_eq!(control, AsaControl::Unknown(b'X'));
        assert!(!control.is_known());
    }

    // Validates: Requirement 7.4
    #[test]
    fn asa_control_abbreviations() {
        assert_eq!(AsaControl::SingleSpace.abbreviation(), "SP");
        assert_eq!(AsaControl::DoubleSpace.abbreviation(), "DS");
        assert_eq!(AsaControl::TripleSpace.abbreviation(), "TS");
        assert_eq!(AsaControl::NewPage.abbreviation(), "NP");
        assert_eq!(AsaControl::Overprint.abbreviation(), "OP");
        assert_eq!(AsaControl::Halt.abbreviation(), "HT");
        assert_eq!(AsaControl::Unknown(b'X').abbreviation(), "??");
    }

    // Validates: Requirement 7.3
    #[test]
    fn detect_asa_at_80_percent_threshold() {
        // 16 out of 20 lines have ASA chars = 80% → detected
        let mut records: Vec<Vec<u8>> = Vec::new();
        for _ in 0..16 {
            records.push(b" DATA LINE".to_vec()); // space = ASA
        }
        for _ in 0..4 {
            records.push(b"XDATA LINE".to_vec()); // X = not ASA
        }

        let refs: Vec<&[u8]> = records.iter().map(|r| r.as_slice()).collect();
        let result = detect_asa(&refs, 20);
        assert!(result.detected);
        assert_eq!(result.confidence, 0.8);
        assert_eq!(result.sample_size, 20);
    }

    #[test]
    fn detect_asa_below_threshold_not_detected() {
        // 15 out of 20 lines = 75% → NOT detected
        let mut records: Vec<Vec<u8>> = Vec::new();
        for _ in 0..15 {
            records.push(b" DATA LINE".to_vec());
        }
        for _ in 0..5 {
            records.push(b"XDATA LINE".to_vec());
        }

        let refs: Vec<&[u8]> = records.iter().map(|r| r.as_slice()).collect();
        let result = detect_asa(&refs, 20);
        assert!(!result.detected);
        assert!(result.confidence < 0.8);
    }

    #[test]
    fn detect_asa_all_asa_chars_detected() {
        let records: Vec<Vec<u8>> = vec![
            b" line1".to_vec(),
            b"0line2".to_vec(),
            b"-line3".to_vec(),
            b"1line4".to_vec(),
            b"+line5".to_vec(),
            b"Hline6".to_vec(),
        ];
        let refs: Vec<&[u8]> = records.iter().map(|r| r.as_slice()).collect();
        let result = detect_asa(&refs, 20);
        assert!(result.detected);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn detect_asa_empty_records_skipped() {
        let records: Vec<Vec<u8>> = vec![vec![], b" data".to_vec(), vec![], b" more".to_vec()];
        let refs: Vec<&[u8]> = records.iter().map(|r| r.as_slice()).collect();
        let result = detect_asa(&refs, 20);
        assert!(result.detected);
        assert_eq!(result.sample_size, 2); // Only non-empty lines counted
    }

    #[test]
    fn detect_asa_no_records_returns_not_detected() {
        let records: Vec<&[u8]> = vec![];
        let result = detect_asa(&records, 20);
        assert!(!result.detected);
        assert_eq!(result.sample_size, 0);
    }

    // Validates: Requirement 7.8
    #[test]
    fn strip_asa_removes_column_1() {
        let records = vec![
            b" Hello World".to_vec(),
            b"0Second Line".to_vec(),
            b"1New Page".to_vec(),
        ];
        let stripped = strip_asa(&records);
        assert_eq!(stripped[0], b"Hello World");
        assert_eq!(stripped[1], b"Second Line");
        assert_eq!(stripped[2], b"New Page");
    }

    #[test]
    fn strip_asa_empty_record_stays_empty() {
        let records = vec![vec![]];
        let stripped = strip_asa(&records);
        assert_eq!(stripped[0], Vec::<u8>::new());
    }

    // Validates: Requirement 7.5
    #[test]
    fn strip_asa_shifts_content_left_by_one_byte() {
        let records = vec![b" ABCDEF".to_vec()];
        let stripped = strip_asa(&records);
        assert_eq!(stripped[0].len(), 6); // Original was 7, now 6
        assert_eq!(stripped[0], b"ABCDEF");
    }
}
