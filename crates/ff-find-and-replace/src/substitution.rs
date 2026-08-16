//! Substitution template parsing and expansion for regex replacements.
//!
//! Addresses: Requirement 8

use crate::error::FindReplaceError;
use crate::indexer::CharacterIndexer;
use crate::types::MatchRange;

/// A segment within a substitution template.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TemplateSegment {
    /// Literal text to insert as-is.
    Literal(String),
    /// Group reference (0–9).
    GroupRef(u8),
}

/// A parsed replacement template with group references.
///
/// Addresses: Requirement 8 AC 2–4
#[derive(Debug, Clone)]
pub struct SubstitutionTemplate {
    segments: Vec<TemplateSegment>,
}

impl SubstitutionTemplate {
    /// Parse a replacement string into a template.
    ///
    /// Recognizes `\0`–`\9` and `$0`–`$9` as group references.
    ///
    /// Addresses: Requirement 8 AC 2–3
    pub fn parse(text: &str) -> Result<Self, FindReplaceError> {
        let mut segments = Vec::new();
        let mut current_literal = String::new();
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];
            match ch {
                '\\' => {
                    if i + 1 < chars.len() {
                        let next = chars[i + 1];
                        if next.is_ascii_digit() {
                            // Group reference \0–\9
                            if !current_literal.is_empty() {
                                segments.push(TemplateSegment::Literal(std::mem::take(
                                    &mut current_literal,
                                )));
                            }
                            let group = next as u8 - b'0';
                            segments.push(TemplateSegment::GroupRef(group));
                            i += 2;
                        } else {
                            // Escape sequence
                            match next {
                                'n' => current_literal.push('\n'),
                                'r' => current_literal.push('\r'),
                                't' => current_literal.push('\t'),
                                '\\' => current_literal.push('\\'),
                                '$' => current_literal.push('$'),
                                _ => {
                                    current_literal.push('\\');
                                    current_literal.push(next);
                                }
                            }
                            i += 2;
                        }
                    } else {
                        // Trailing backslash
                        current_literal.push('\\');
                        i += 1;
                    }
                }
                '$' => {
                    if i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                        // Group reference $0–$9
                        if !current_literal.is_empty() {
                            segments.push(TemplateSegment::Literal(std::mem::take(
                                &mut current_literal,
                            )));
                        }
                        let group = chars[i + 1] as u8 - b'0';
                        segments.push(TemplateSegment::GroupRef(group));
                        i += 2;
                    } else {
                        current_literal.push('$');
                        i += 1;
                    }
                }
                _ => {
                    current_literal.push(ch);
                    i += 1;
                }
            }
        }

        if !current_literal.is_empty() {
            segments.push(TemplateSegment::Literal(current_literal));
        }

        Ok(Self { segments })
    }

    /// Expand the template against captured groups, producing replacement text.
    ///
    /// `full_match` is group 0 (the entire match range).
    /// `captures` are groups 1–N from the regex.
    ///
    /// Addresses: Requirement 8 AC 8
    pub fn expand(
        &self,
        full_match: &MatchRange,
        captures: &[MatchRange],
        indexer: &dyn CharacterIndexer,
    ) -> String {
        let mut result = String::new();
        for segment in &self.segments {
            match segment {
                TemplateSegment::Literal(text) => result.push_str(text),
                TemplateSegment::GroupRef(group) => {
                    let range = if *group == 0 {
                        Some(full_match)
                    } else {
                        captures.get((*group as usize) - 1)
                    };
                    if let Some(r) = range {
                        if let Some(bytes) = indexer.slice(r.start, r.end) {
                            if let Ok(s) = std::str::from_utf8(&bytes) {
                                result.push_str(s);
                            }
                        }
                    }
                    // Unmatched group → empty string (Requirement 8 AC 4)
                }
            }
        }
        result
    }

    /// Whether this template contains any group references.
    pub fn has_group_refs(&self) -> bool {
        self.segments
            .iter()
            .any(|s| matches!(s, TemplateSegment::GroupRef(_)))
    }
}

/// Substitution engine for expanding templates.
///
/// Addresses: Requirement 8
pub struct SubstitutionEngine;

impl SubstitutionEngine {
    /// Parse a replacement string into a template.
    pub fn parse_template(replacement: &str) -> Result<SubstitutionTemplate, FindReplaceError> {
        SubstitutionTemplate::parse(replacement)
    }

    /// Expand a template using captured groups from a match.
    pub fn substitute(
        template: &SubstitutionTemplate,
        full_match: &MatchRange,
        captures: &[MatchRange],
        indexer: &dyn CharacterIndexer,
    ) -> String {
        template.expand(full_match, captures, indexer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::SliceIndexer;
    use crate::types::BytePosition;

    #[test]
    fn parse_template_with_no_group_refs_produces_literal() {
        let t = SubstitutionTemplate::parse("hello world").unwrap();
        assert!(!t.has_group_refs());
        let indexer = SliceIndexer::from_str("");
        let result = t.expand(
            &MatchRange::new(BytePosition(0), BytePosition(0)),
            &[],
            &indexer,
        );
        assert_eq!(result, "hello world");
    }

    #[test]
    fn parse_template_backslash_group_refs() {
        let t = SubstitutionTemplate::parse("\\1-\\2").unwrap();
        assert!(t.has_group_refs());
    }

    #[test]
    fn parse_template_dollar_group_refs() {
        let t = SubstitutionTemplate::parse("$1-$2").unwrap();
        assert!(t.has_group_refs());
    }

    #[test]
    fn expand_substitutes_group_text() {
        let indexer = SliceIndexer::from_str("hello world");
        let t = SubstitutionTemplate::parse("\\1 \\2").unwrap();
        let full = MatchRange::new(BytePosition(0), BytePosition(11));
        let caps = vec![
            MatchRange::new(BytePosition(0), BytePosition(5)), // group 1: "hello"
            MatchRange::new(BytePosition(6), BytePosition(11)), // group 2: "world"
        ];
        let result = t.expand(&full, &caps, &indexer);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn expand_unmatched_group_produces_empty_string() {
        let indexer = SliceIndexer::from_str("hello");
        let t = SubstitutionTemplate::parse("\\1-\\9").unwrap();
        let full = MatchRange::new(BytePosition(0), BytePosition(5));
        let caps = vec![MatchRange::new(BytePosition(0), BytePosition(5))];
        let result = t.expand(&full, &caps, &indexer);
        // group 1 exists, group 9 does not → empty
        assert_eq!(result, "hello-");
    }

    #[test]
    fn expand_group_zero_uses_full_match() {
        let indexer = SliceIndexer::from_str("hello");
        let t = SubstitutionTemplate::parse("[\\0]").unwrap();
        let full = MatchRange::new(BytePosition(0), BytePosition(5));
        let result = t.expand(&full, &[], &indexer);
        assert_eq!(result, "[hello]");
    }

    #[test]
    fn parse_escape_sequences_in_replacement() {
        let t = SubstitutionTemplate::parse("a\\nb").unwrap();
        let indexer = SliceIndexer::from_str("");
        let result = t.expand(
            &MatchRange::new(BytePosition(0), BytePosition(0)),
            &[],
            &indexer,
        );
        assert_eq!(result, "a\nb");
    }
}
