//! Content-based language detection: magic bytes, shebang, first-line patterns.

use regex::Regex;

use crate::definition::{LanguageDefinition, LanguageId};
use crate::detection::{DetectionMethod, DetectionResult};

/// Maximum bytes inspected for content-based detection.
const MAX_CONTENT_BYTES: usize = 8192;

/// Content-based language detector using magic bytes, shebang patterns,
/// and first-line regex patterns.
#[derive(Debug)]
pub struct ContentDetector {
    /// Languages with magic byte patterns.
    magic_entries: Vec<MagicEntry>,
    /// Languages with shebang patterns.
    shebang_entries: Vec<ShebangEntry>,
    /// Languages with first-line patterns.
    first_line_entries: Vec<FirstLineEntry>,
}

#[derive(Debug, Clone)]
struct MagicEntry {
    language_id: LanguageId,
    magic_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct ShebangEntry {
    language_id: LanguageId,
    patterns: Vec<String>,
}

#[derive(Debug)]
struct FirstLineEntry {
    language_id: LanguageId,
    pattern: Regex,
}

impl ContentDetector {
    /// Build a content detector from loaded language definitions.
    pub fn from_definitions(definitions: &[LanguageDefinition]) -> Self {
        let mut magic_entries = Vec::new();
        let mut shebang_entries = Vec::new();
        let mut first_line_entries = Vec::new();

        for def in definitions {
            if let Some(bytes) = def.magic_bytes() {
                magic_entries.push(MagicEntry {
                    language_id: def.language_id().clone(),
                    magic_bytes: bytes.to_vec(),
                });
            }

            if !def.shebang_patterns().is_empty() {
                shebang_entries.push(ShebangEntry {
                    language_id: def.language_id().clone(),
                    patterns: def.shebang_patterns().to_vec(),
                });
            }

            if let Some(pattern_str) = def.first_line_pattern() {
                if let Ok(regex) = Regex::new(pattern_str) {
                    first_line_entries.push(FirstLineEntry {
                        language_id: def.language_id().clone(),
                        pattern: regex,
                    });
                }
            }
        }

        Self {
            magic_entries,
            shebang_entries,
            first_line_entries,
        }
    }

    /// Attempt content-based detection on file content.
    ///
    /// Detection applies in priority order:
    /// 1. Magic bytes (highest priority)
    /// 2. Shebang line
    /// 3. First-line pattern (lowest priority)
    ///
    /// Only the first 8192 bytes are inspected.
    pub fn detect(&self, first_bytes: Option<&[u8]>, first_line: Option<&str>) -> DetectionResult {
        // Limit inspection to MAX_CONTENT_BYTES
        let bytes = first_bytes.map(|b| {
            if b.len() > MAX_CONTENT_BYTES {
                &b[..MAX_CONTENT_BYTES]
            } else {
                b
            }
        });

        // 1. Magic bytes detection (highest priority)
        if let Some(content) = bytes {
            if let Some(result) = self.detect_magic_bytes(content) {
                return result;
            }
        }

        // 2. Shebang detection
        let line = first_line.or_else(|| {
            bytes.and_then(|b| std::str::from_utf8(b).ok().and_then(|s| s.lines().next()))
        });

        if let Some(first) = line {
            if let Some(result) = self.detect_shebang(first) {
                return result;
            }

            // 3. First-line pattern detection (lowest priority)
            if let Some(result) = self.detect_first_line_pattern(first) {
                return result;
            }
        }

        DetectionResult {
            language_id: LanguageId::plain_text(),
            method: DetectionMethod::Fallback,
        }
    }

    /// Check if content begins with known magic bytes.
    fn detect_magic_bytes(&self, content: &[u8]) -> Option<DetectionResult> {
        for entry in &self.magic_entries {
            if content.len() >= entry.magic_bytes.len() && content.starts_with(&entry.magic_bytes) {
                return Some(DetectionResult {
                    language_id: entry.language_id.clone(),
                    method: DetectionMethod::MagicBytes,
                });
            }
        }
        None
    }

    /// Check if the first line is a shebang matching a known interpreter.
    fn detect_shebang(&self, first_line: &str) -> Option<DetectionResult> {
        if !first_line.starts_with("#!") {
            return None;
        }

        // Extract the interpreter name from the shebang line
        // Handles: #!/usr/bin/python, #!/usr/bin/env python, #!/usr/bin/env -S python
        let shebang_content = &first_line[2..];
        let interpreter = Self::extract_interpreter(shebang_content);

        for entry in &self.shebang_entries {
            for pattern in &entry.patterns {
                if interpreter.contains(pattern.as_str()) {
                    return Some(DetectionResult {
                        language_id: entry.language_id.clone(),
                        method: DetectionMethod::Shebang,
                    });
                }
            }
        }
        None
    }

    /// Extract the interpreter name from a shebang line content.
    fn extract_interpreter(content: &str) -> String {
        let trimmed = content.trim();
        // Handle `/usr/bin/env [-S] interpreter`
        if let Some(after_env) = trimmed.strip_prefix("/usr/bin/env") {
            let args = after_env.trim();
            // Skip -S or other flags
            let parts: Vec<&str> = args.split_whitespace().collect();
            for part in parts {
                if !part.starts_with('-') {
                    return part.to_string();
                }
            }
            return String::new();
        }
        // Handle direct path: /usr/bin/python3
        trimmed
            .rsplit('/')
            .next()
            .unwrap_or("")
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    }

    /// Check if the first line matches a known first-line pattern.
    fn detect_first_line_pattern(&self, first_line: &str) -> Option<DetectionResult> {
        for entry in &self.first_line_entries {
            if entry.pattern.is_match(first_line) {
                return Some(DetectionResult {
                    language_id: entry.language_id.clone(),
                    method: DetectionMethod::FirstLinePattern,
                });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{ConfigLayer, DefinitionSource};
    use crate::keyword_set::KeywordSets;
    use std::collections::HashMap;

    fn make_definition_with_detection(
        id: &str,
        shebang: &[&str],
        magic: Option<Vec<u8>>,
        first_line: Option<&str>,
    ) -> LanguageDefinition {
        LanguageDefinition {
            language_id: LanguageId::new(id).unwrap(),
            name: id.to_string(),
            extensions: Vec::new(),
            priority: 0,
            case_sensitive_keywords: true,
            keyword_sets: KeywordSets::empty(),
            line_comments: Vec::new(),
            block_comment_start: None,
            block_comment_end: None,
            string_delimiters: Vec::new(),
            character_delimiter: None,
            escape_character: None,
            heredoc_patterns: Vec::new(),
            shebang_patterns: shebang.iter().map(|s| s.to_string()).collect(),
            magic_bytes: magic,
            first_line_pattern: first_line.map(|s| s.to_string()),
            embedded_languages: Vec::new(),
            properties: HashMap::new(),
            fold_keywords: None,
            source: DefinitionSource::File {
                path: "test.toml".to_string(),
                layer: ConfigLayer::BuiltIn,
            },
        }
    }

    #[test]
    fn detect_shebang_python() {
        // Validates: Requirement 3.2
        let defs = vec![make_definition_with_detection(
            "python",
            &["python", "python3"],
            None,
            None,
        )];
        let detector = ContentDetector::from_definitions(&defs);

        let result = detector.detect(None, Some("#!/usr/bin/env python3"));
        assert_eq!(result.language_id.as_str(), "python");
        assert_eq!(result.method, DetectionMethod::Shebang);
    }

    #[test]
    fn detect_shebang_direct_path() {
        // Validates: Requirement 3.2
        let defs = vec![make_definition_with_detection(
            "python",
            &["python", "python3"],
            None,
            None,
        )];
        let detector = ContentDetector::from_definitions(&defs);

        let result = detector.detect(None, Some("#!/usr/bin/python3"));
        assert_eq!(result.language_id.as_str(), "python");
        assert_eq!(result.method, DetectionMethod::Shebang);
    }

    #[test]
    fn detect_shebang_with_env_s_flag() {
        // Validates: Requirement 3.2
        let defs = vec![make_definition_with_detection(
            "python",
            &["python"],
            None,
            None,
        )];
        let detector = ContentDetector::from_definitions(&defs);

        let result = detector.detect(None, Some("#!/usr/bin/env -S python"));
        assert_eq!(result.language_id.as_str(), "python");
    }

    #[test]
    fn detect_magic_bytes_elf() {
        // Validates: Requirement 3.3
        let defs = vec![make_definition_with_detection(
            "elf",
            &[],
            Some(vec![0x7F, 0x45, 0x4C, 0x46]),
            None,
        )];
        let detector = ContentDetector::from_definitions(&defs);

        let content = vec![0x7F, 0x45, 0x4C, 0x46, 0x00, 0x00];
        let result = detector.detect(Some(&content), None);
        assert_eq!(result.language_id.as_str(), "elf");
        assert_eq!(result.method, DetectionMethod::MagicBytes);
    }

    #[test]
    fn detect_first_line_pattern_xml() {
        // Validates: Requirement 3.4
        let defs = vec![make_definition_with_detection(
            "xml",
            &[],
            None,
            Some(r"^<\?xml"),
        )];
        let detector = ContentDetector::from_definitions(&defs);

        let result = detector.detect(None, Some("<?xml version=\"1.0\"?>"));
        assert_eq!(result.language_id.as_str(), "xml");
        assert_eq!(result.method, DetectionMethod::FirstLinePattern);
    }

    #[test]
    fn detect_priority_magic_over_shebang() {
        // Validates: Requirement 3.5
        let defs = vec![
            make_definition_with_detection("binary", &[], Some(vec![0x7F, 0x45]), None),
            make_definition_with_detection("shell", &["sh", "bash"], None, None),
        ];
        let detector = ContentDetector::from_definitions(&defs);

        // Content starts with magic bytes AND has a shebang-like line
        let mut content = vec![0x7F, 0x45];
        content.extend_from_slice(b"#!/bin/bash\n");
        let result = detector.detect(Some(&content), None);
        assert_eq!(result.language_id.as_str(), "binary");
        assert_eq!(result.method, DetectionMethod::MagicBytes);
    }

    #[test]
    fn detect_priority_shebang_over_first_line() {
        // Validates: Requirement 3.5
        let defs = vec![
            make_definition_with_detection("python", &["python"], None, None),
            make_definition_with_detection("xml", &[], None, Some(r"^#!")),
        ];
        let detector = ContentDetector::from_definitions(&defs);

        let result = detector.detect(None, Some("#!/usr/bin/env python"));
        assert_eq!(result.language_id.as_str(), "python");
        assert_eq!(result.method, DetectionMethod::Shebang);
    }

    #[test]
    fn detect_no_match_returns_plain_text() {
        // Validates: Requirement 3.6
        let defs = vec![make_definition_with_detection(
            "python",
            &["python"],
            None,
            None,
        )];
        let detector = ContentDetector::from_definitions(&defs);

        let result = detector.detect(None, Some("just some text"));
        assert!(result.language_id.is_plain_text());
        assert_eq!(result.method, DetectionMethod::Fallback);
    }

    #[test]
    fn detect_respects_byte_limit() {
        // Validates: Requirement 3.7
        let defs = vec![make_definition_with_detection(
            "special",
            &[],
            Some(vec![0xFF, 0xFE]),
            None,
        )];
        let detector = ContentDetector::from_definitions(&defs);

        // Magic bytes beyond 8192 limit should not be detected
        let mut content = vec![0x00; 8200];
        content[8193] = 0xFF;
        content[8194] = 0xFE;
        let result = detector.detect(Some(&content), None);
        assert!(result.language_id.is_plain_text());
    }

    #[test]
    fn detect_non_shebang_first_line_not_matched_as_shebang() {
        // Validates: Requirement 3.2
        let defs = vec![make_definition_with_detection(
            "python",
            &["python"],
            None,
            None,
        )];
        let detector = ContentDetector::from_definitions(&defs);

        let result = detector.detect(None, Some("# This is a comment about python"));
        assert!(result.language_id.is_plain_text());
    }
}
