//! SYSIN input processing and reading modes.
//!
//! Supports three input modes: SYSIN DD resolution, string buffer, and file input.

/// Input source specification for IDCAMS invocations.
#[derive(Debug, Clone, PartialEq)]
pub enum InputSource {
    /// Commands from a SYSIN DD (JCL execution context).
    SysinDd(String),
    /// Commands from an in-memory string buffer (scripting API).
    StringBuffer(String),
    /// Commands from a file path (standalone execution).
    FileInput(String),
}

/// Preprocesses SYSIN input: strips sequence numbers and blank lines.
pub fn preprocess_sysin(input: &str) -> String {
    let mut result = String::with_capacity(input.len());

    for line in input.lines() {
        // Strip sequence numbers from columns 73-80
        let processed = if line.len() >= 80 {
            let seq_area = &line[72..80];
            if seq_area.chars().all(|c| c.is_ascii_digit() || c == ' ') {
                &line[..72]
            } else {
                line
            }
        } else {
            line
        };

        // Skip blank lines
        if processed.trim().is_empty() {
            continue;
        }

        result.push_str(processed);
        result.push('\n');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preprocess_strips_sequence_numbers() {
        let input =
            "DEFINE CLUSTER (NAME(MY.DATA))                                          00000100";
        let result = preprocess_sysin(input);
        assert!(!result.contains("00000100"));
        assert!(result.contains("DEFINE CLUSTER"));
    }

    #[test]
    fn preprocess_skips_blank_lines() {
        let input = "DEFINE\n\n\nDELETE\n";
        let result = preprocess_sysin(input);
        assert_eq!(result, "DEFINE\nDELETE\n");
    }

    #[test]
    fn preprocess_empty_input() {
        let result = preprocess_sysin("");
        assert_eq!(result, "");
    }
}
