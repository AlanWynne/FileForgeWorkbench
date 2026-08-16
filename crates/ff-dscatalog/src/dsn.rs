//! Dataset Name (DSN) validation and parsing.
//!
//! Implements mainframe dataset naming rules: qualifiers separated by dots,
//! each 1–8 characters, total ≤44 characters. Case-insensitive, stored uppercase.

use std::fmt;
use std::str::FromStr;

use crate::error::CatalogError;

/// Maximum total length of a dataset name (including dots).
const MAX_DSN_LENGTH: usize = 44;

/// Maximum length of a single qualifier.
const MAX_QUALIFIER_LENGTH: usize = 8;

/// A validated mainframe dataset name in HLQ.qualifier format.
///
/// Stored internally in uppercase. Maximum 44 characters total.
///
/// # Examples
///
/// ```
/// use ff_dscatalog::dsn::Dsn;
///
/// let dsn = Dsn::parse("PAYROLL.INPUT.FILE").unwrap();
/// assert_eq!(dsn.hlq(), "PAYROLL");
/// assert_eq!(dsn.as_str(), "PAYROLL.INPUT.FILE");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dsn {
    /// The full DSN string in uppercase (e.g., "PAYROLL.INPUT.FILE")
    normalized: String,
    /// Individual qualifiers (e.g., ["PAYROLL", "INPUT", "FILE"])
    qualifiers: Vec<String>,
}

impl Dsn {
    /// Parse and validate a DSN string.
    ///
    /// Returns error with position info on failure. Performs case-insensitive
    /// normalization (stores uppercase).
    ///
    /// # Errors
    ///
    /// Returns `CatalogError::DsnValidation` if:
    /// - Total length exceeds 44 characters
    /// - Any qualifier exceeds 8 characters
    /// - Any qualifier starts with a non-alphabetic/national character
    /// - Any qualifier contains invalid characters
    /// - The string starts/ends with a dot or contains consecutive dots
    /// - The string is empty
    pub fn parse(input: &str) -> Result<Self, CatalogError> {
        Self::parse_inner(input, "parse")
    }

    /// Internal parsing with configurable operation name for error context.
    fn parse_inner(input: &str, operation: &str) -> Result<Self, CatalogError> {
        if input.is_empty() {
            return Err(CatalogError::DsnValidation {
                input: input.to_string(),
                reason: "dataset name cannot be empty".to_string(),
                position: 0,
                operation: operation.to_string(),
            });
        }

        let upper = input.to_uppercase();

        if upper.len() > MAX_DSN_LENGTH {
            return Err(CatalogError::DsnValidation {
                input: input.to_string(),
                reason: format!(
                    "total length {} exceeds maximum of {MAX_DSN_LENGTH}",
                    upper.len()
                ),
                position: MAX_DSN_LENGTH,
                operation: operation.to_string(),
            });
        }

        if upper.starts_with('.') {
            return Err(CatalogError::DsnValidation {
                input: input.to_string(),
                reason: "dataset name cannot start with a dot".to_string(),
                position: 0,
                operation: operation.to_string(),
            });
        }

        if upper.ends_with('.') {
            return Err(CatalogError::DsnValidation {
                input: input.to_string(),
                reason: "dataset name cannot end with a dot".to_string(),
                position: upper.len() - 1,
                operation: operation.to_string(),
            });
        }

        if upper.contains("..") {
            let pos = upper.find("..").unwrap_or(0);
            return Err(CatalogError::DsnValidation {
                input: input.to_string(),
                reason: "dataset name cannot contain consecutive dots".to_string(),
                position: pos,
                operation: operation.to_string(),
            });
        }

        let qualifiers: Vec<&str> = upper.split('.').collect();
        let mut current_pos = 0;

        for qualifier in &qualifiers {
            if qualifier.is_empty() {
                return Err(CatalogError::DsnValidation {
                    input: input.to_string(),
                    reason: "qualifier cannot be empty".to_string(),
                    position: current_pos,
                    operation: operation.to_string(),
                });
            }

            if qualifier.len() > MAX_QUALIFIER_LENGTH {
                return Err(CatalogError::DsnValidation {
                    input: input.to_string(),
                    reason: format!(
                        "qualifier '{}' exceeds maximum length of {MAX_QUALIFIER_LENGTH}",
                        qualifier
                    ),
                    position: current_pos,
                    operation: operation.to_string(),
                });
            }

            let first_char = qualifier.chars().next().unwrap();
            if !is_alpha_or_national(first_char) {
                return Err(CatalogError::DsnValidation {
                    input: input.to_string(),
                    reason: format!(
                        "qualifier '{}' must start with alphabetic (A-Z) or national (@, #, $) character",
                        qualifier
                    ),
                    position: current_pos,
                    operation: operation.to_string(),
                });
            }

            for (i, ch) in qualifier.chars().enumerate().skip(1) {
                if !is_alphanumeric_or_national(ch) {
                    return Err(CatalogError::DsnValidation {
                        input: input.to_string(),
                        reason: format!("invalid character '{}' in qualifier '{}'", ch, qualifier),
                        position: current_pos + i,
                        operation: operation.to_string(),
                    });
                }
            }

            current_pos += qualifier.len() + 1; // +1 for the dot separator
        }

        let owned_qualifiers: Vec<String> = qualifiers.iter().map(|q| q.to_string()).collect();

        Ok(Self {
            normalized: upper,
            qualifiers: owned_qualifiers,
        })
    }

    /// Parse a DSN with optional member reference: `DSN(MEMBER)`.
    ///
    /// Returns `(Dsn, Option<MemberName>)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_dscatalog::dsn::Dsn;
    ///
    /// let (dsn, member) = Dsn::parse_member_ref("SYS1.MACLIB(OPEN)").unwrap();
    /// assert_eq!(dsn.as_str(), "SYS1.MACLIB");
    /// assert_eq!(member.unwrap().as_str(), "OPEN");
    /// ```
    pub fn parse_member_ref(input: &str) -> Result<(Self, Option<MemberName>), CatalogError> {
        if let Some(paren_start) = input.find('(') {
            if !input.ends_with(')') {
                return Err(CatalogError::DsnValidation {
                    input: input.to_string(),
                    reason: "member reference must end with ')'".to_string(),
                    position: input.len() - 1,
                    operation: "parse_member_ref".to_string(),
                });
            }

            let dsn_part = &input[..paren_start];
            let member_part = &input[paren_start + 1..input.len() - 1];

            let dsn = Self::parse_inner(dsn_part, "parse_member_ref")?;
            let member = MemberName::parse(member_part)?;

            Ok((dsn, Some(member)))
        } else {
            let dsn = Self::parse_inner(input, "parse_member_ref")?;
            Ok((dsn, None))
        }
    }

    /// Prepend a default HLQ to a bare qualifier to form a full DSN.
    ///
    /// # Examples
    ///
    /// ```
    /// use ff_dscatalog::dsn::Dsn;
    ///
    /// let dsn = Dsn::with_default_hlq("INPUT.FILE", "PAYROLL").unwrap();
    /// assert_eq!(dsn.as_str(), "PAYROLL.INPUT.FILE");
    /// ```
    pub fn with_default_hlq(bare: &str, hlq: &str) -> Result<Self, CatalogError> {
        let full = format!("{hlq}.{bare}");
        Self::parse_inner(&full, "with_default_hlq")
    }

    /// Get the High Level Qualifier (first qualifier).
    pub fn hlq(&self) -> &str {
        &self.qualifiers[0]
    }

    /// Get all qualifiers as a slice.
    pub fn qualifiers(&self) -> &[String] {
        &self.qualifiers
    }

    /// Get the full normalized DSN string.
    pub fn as_str(&self) -> &str {
        &self.normalized
    }

    /// Check if this DSN matches a filter pattern (with `*` and `%` wildcards).
    ///
    /// - `*` matches zero or more characters across qualifiers
    /// - `%` matches exactly one qualifier
    pub fn matches_pattern(&self, pattern: &str) -> bool {
        let pattern_upper = pattern.to_uppercase();
        wildcard_match(&self.normalized, &pattern_upper)
    }

    /// Construct from components without re-parsing (internal use).
    #[allow(dead_code)]
    pub(crate) fn from_qualifiers(qualifiers: Vec<String>) -> Self {
        let normalized = qualifiers.join(".");
        Self {
            normalized,
            qualifiers,
        }
    }
}

impl fmt::Display for Dsn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.normalized)
    }
}

impl FromStr for Dsn {
    type Err = CatalogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// A validated PDS member name (1–8 characters, same rules as a single qualifier).
///
/// # Examples
///
/// ```
/// use ff_dscatalog::dsn::MemberName;
///
/// let member = MemberName::parse("OPEN").unwrap();
/// assert_eq!(member.as_str(), "OPEN");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemberName {
    /// The member name in uppercase.
    normalized: String,
}

impl MemberName {
    /// Parse and validate a member name.
    ///
    /// # Errors
    ///
    /// Returns `CatalogError::InvalidMemberName` if the name doesn't conform
    /// to single-qualifier rules (1–8 chars, starts with alphabetic/national,
    /// followed by alphanumeric/national).
    pub fn parse(input: &str) -> Result<Self, CatalogError> {
        if input.is_empty() {
            return Err(CatalogError::InvalidMemberName {
                input: input.to_string(),
                reason: "member name cannot be empty".to_string(),
                operation: "parse".to_string(),
            });
        }

        let upper = input.to_uppercase();

        if upper.len() > MAX_QUALIFIER_LENGTH {
            return Err(CatalogError::InvalidMemberName {
                input: input.to_string(),
                reason: format!(
                    "member name length {} exceeds maximum of {MAX_QUALIFIER_LENGTH}",
                    upper.len()
                ),
                operation: "parse".to_string(),
            });
        }

        let first_char = upper.chars().next().unwrap();
        if !is_alpha_or_national(first_char) {
            return Err(CatalogError::InvalidMemberName {
                input: input.to_string(),
                reason: "must start with alphabetic (A-Z) or national (@, #, $) character"
                    .to_string(),
                operation: "parse".to_string(),
            });
        }

        for (i, ch) in upper.chars().enumerate().skip(1) {
            if !is_alphanumeric_or_national(ch) {
                return Err(CatalogError::InvalidMemberName {
                    input: input.to_string(),
                    reason: format!("invalid character '{}' at position {}", ch, i),
                    operation: "parse".to_string(),
                });
            }
        }

        Ok(Self { normalized: upper })
    }

    /// Get the normalized member name string.
    pub fn as_str(&self) -> &str {
        &self.normalized
    }
}

impl fmt::Display for MemberName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.normalized)
    }
}

impl FromStr for MemberName {
    type Err = CatalogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Returns true if the character is alphabetic (A-Z) or a national character (@, #, $).
fn is_alpha_or_national(ch: char) -> bool {
    ch.is_ascii_uppercase() || ch == '@' || ch == '#' || ch == '$'
}

/// Returns true if the character is alphanumeric (A-Z, 0-9) or a national character.
fn is_alphanumeric_or_national(ch: char) -> bool {
    ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '@' || ch == '#' || ch == '$'
}

/// Simple wildcard matching: `*` matches zero or more chars, `%` matches exactly one qualifier.
fn wildcard_match(text: &str, pattern: &str) -> bool {
    // Handle `%` as matching exactly one qualifier by converting the pattern
    // to work with qualifiers for `%` and chars for `*`.
    if pattern.contains('%') {
        // Split both into qualifiers and match qualifier-by-qualifier
        let text_qualifiers: Vec<&str> = text.split('.').collect();
        let pattern_qualifiers: Vec<&str> = pattern.split('.').collect();
        return qualifier_match(&text_qualifiers, &pattern_qualifiers);
    }

    // Simple `*` matching across the whole string
    simple_wildcard_match(text, pattern)
}

/// Match text qualifiers against pattern qualifiers where `%` matches exactly one qualifier.
fn qualifier_match(text_quals: &[&str], pattern_quals: &[&str]) -> bool {
    if pattern_quals.is_empty() {
        return text_quals.is_empty();
    }
    if text_quals.is_empty() {
        // Pattern can match empty only if all remaining are `*`
        return pattern_quals.iter().all(|p| *p == "*");
    }

    let pat = pattern_quals[0];
    if pat == "%" {
        // `%` matches exactly one qualifier
        qualifier_match(&text_quals[1..], &pattern_quals[1..])
    } else if pat == "*" || pat.contains('*') {
        if pat == "*" {
            // `*` alone matches zero or more qualifiers
            for i in 0..=text_quals.len() {
                if qualifier_match(&text_quals[i..], &pattern_quals[1..]) {
                    return true;
                }
            }
            false
        } else {
            // Qualifier contains `*` — match within the qualifier text
            if simple_wildcard_match(text_quals[0], pat) {
                qualifier_match(&text_quals[1..], &pattern_quals[1..])
            } else {
                false
            }
        }
    } else {
        // Literal qualifier match
        if text_quals[0] == pat {
            qualifier_match(&text_quals[1..], &pattern_quals[1..])
        } else {
            false
        }
    }
}

/// Simple wildcard match where `*` matches zero or more characters.
fn simple_wildcard_match(text: &str, pattern: &str) -> bool {
    let text_bytes = text.as_bytes();
    let pattern_bytes = pattern.as_bytes();
    let (tlen, plen) = (text_bytes.len(), pattern_bytes.len());

    let mut dp = vec![vec![false; plen + 1]; tlen + 1];
    dp[0][0] = true;

    // Handle leading `*` patterns
    for j in 1..=plen {
        if pattern_bytes[j - 1] == b'*' {
            dp[0][j] = dp[0][j - 1];
        }
    }

    for i in 1..=tlen {
        for j in 1..=plen {
            if pattern_bytes[j - 1] == b'*' {
                dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
            } else if pattern_bytes[j - 1] == text_bytes[i - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            }
        }
    }

    dp[tlen][plen]
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Valid DSN tests ──

    #[test]
    fn parse_simple_dsn() {
        // Validates: Requirement 2 AC 1, AC 2
        let dsn = Dsn::parse("PAYROLL.INPUT.FILE").unwrap();
        assert_eq!(dsn.as_str(), "PAYROLL.INPUT.FILE");
        assert_eq!(dsn.hlq(), "PAYROLL");
        assert_eq!(dsn.qualifiers().len(), 3);
    }

    #[test]
    fn parse_single_qualifier() {
        // Validates: Requirement 2 AC 1
        let dsn = Dsn::parse("SINGLE").unwrap();
        assert_eq!(dsn.as_str(), "SINGLE");
        assert_eq!(dsn.qualifiers().len(), 1);
    }

    #[test]
    fn parse_case_insensitive_stores_uppercase() {
        // Validates: Requirement 2 AC 5
        let dsn = Dsn::parse("payroll.input.file").unwrap();
        assert_eq!(dsn.as_str(), "PAYROLL.INPUT.FILE");
    }

    #[test]
    fn parse_mixed_case_produces_same_result() {
        // Validates: Requirement 2 AC 5
        let upper = Dsn::parse("PAYROLL.INPUT").unwrap();
        let lower = Dsn::parse("payroll.input").unwrap();
        let mixed = Dsn::parse("Payroll.Input").unwrap();
        assert_eq!(upper, lower);
        assert_eq!(upper, mixed);
    }

    #[test]
    fn parse_national_characters() {
        // Validates: Requirement 2 AC 2
        let dsn = Dsn::parse("@USER.#TEMP.$DATA").unwrap();
        assert_eq!(dsn.as_str(), "@USER.#TEMP.$DATA");
    }

    #[test]
    fn parse_dsn_with_numbers() {
        // Validates: Requirement 2 AC 2
        let dsn = Dsn::parse("SYS1.MACLIB").unwrap();
        assert_eq!(dsn.as_str(), "SYS1.MACLIB");
    }

    #[test]
    fn parse_maximum_qualifier_length() {
        // Validates: Requirement 2 AC 2 — 8 chars per qualifier
        let dsn = Dsn::parse("ABCDEFGH.IJKLMNOP").unwrap();
        assert_eq!(dsn.qualifiers()[0], "ABCDEFGH");
        assert_eq!(dsn.qualifiers()[1], "IJKLMNOP");
    }

    #[test]
    fn parse_dsn_at_44_chars() {
        // Validates: Requirement 2 AC 1 — max 44 chars
        // "ABCDEFGH.ABCDEFGH.ABCDEFGH.ABCDEFGH.ABCDEFGH" = 5*8 + 4 dots = 44
        let dsn = Dsn::parse("ABCDEFGH.ABCDEFGH.ABCDEFGH.ABCDEFGH.ABCDEFGH").unwrap();
        assert_eq!(dsn.as_str().len(), 44);
    }

    // ── Invalid DSN tests ──

    #[test]
    fn reject_empty_dsn() {
        // Validates: Requirement 2 AC 4
        let err = Dsn::parse("").unwrap_err();
        match err {
            CatalogError::DsnValidation { reason, .. } => {
                assert!(reason.contains("empty"));
            }
            _ => panic!("expected DsnValidation"),
        }
    }

    #[test]
    fn reject_dsn_exceeding_44_chars() {
        // Validates: Requirement 2 AC 1, AC 4
        // 6*8 + 5 dots = 53 chars, definitely over 44
        let long = "ABCDEFGH.ABCDEFGH.ABCDEFGH.ABCDEFGH.ABCDEFGH.A";
        assert!(long.len() > 44);
        let err = Dsn::parse(long).unwrap_err();
        match err {
            CatalogError::DsnValidation { reason, .. } => {
                assert!(reason.contains("exceeds maximum"));
            }
            _ => panic!("expected DsnValidation"),
        }
    }

    #[test]
    fn reject_qualifier_exceeding_8_chars() {
        // Validates: Requirement 2 AC 2, AC 4
        let err = Dsn::parse("TOOLONGQU.DATA").unwrap_err();
        match err {
            CatalogError::DsnValidation { reason, .. } => {
                assert!(reason.contains("exceeds maximum length"));
            }
            _ => panic!("expected DsnValidation"),
        }
    }

    #[test]
    fn reject_qualifier_starting_with_digit() {
        // Validates: Requirement 2 AC 2, AC 4
        let err = Dsn::parse("1BAD.DATA").unwrap_err();
        match err {
            CatalogError::DsnValidation { reason, .. } => {
                assert!(reason.contains("must start with"));
            }
            _ => panic!("expected DsnValidation"),
        }
    }

    #[test]
    fn reject_leading_dot() {
        // Validates: Requirement 2 AC 7
        let err = Dsn::parse(".LEADING").unwrap_err();
        match err {
            CatalogError::DsnValidation { reason, .. } => {
                assert!(reason.contains("start with a dot"));
            }
            _ => panic!("expected DsnValidation"),
        }
    }

    #[test]
    fn reject_trailing_dot() {
        // Validates: Requirement 2 AC 7
        let err = Dsn::parse("TRAILING.").unwrap_err();
        match err {
            CatalogError::DsnValidation { reason, .. } => {
                assert!(reason.contains("end with a dot"));
            }
            _ => panic!("expected DsnValidation"),
        }
    }

    #[test]
    fn reject_consecutive_dots() {
        // Validates: Requirement 2 AC 7
        let err = Dsn::parse("BAD..NAME").unwrap_err();
        match err {
            CatalogError::DsnValidation { reason, .. } => {
                assert!(reason.contains("consecutive dots"));
            }
            _ => panic!("expected DsnValidation"),
        }
    }

    #[test]
    fn reject_invalid_character_in_qualifier() {
        // Validates: Requirement 2 AC 4
        let err = Dsn::parse("BAD-NAME.DATA").unwrap_err();
        match err {
            CatalogError::DsnValidation { reason, .. } => {
                assert!(reason.contains("invalid character"));
            }
            _ => panic!("expected DsnValidation"),
        }
    }

    // ── Member ref tests ──

    #[test]
    fn parse_member_ref_with_member() {
        // Validates: Requirement 2 AC 9
        let (dsn, member) = Dsn::parse_member_ref("SYS1.MACLIB(OPEN)").unwrap();
        assert_eq!(dsn.as_str(), "SYS1.MACLIB");
        assert_eq!(member.unwrap().as_str(), "OPEN");
    }

    #[test]
    fn parse_member_ref_without_member() {
        // Validates: Requirement 2 AC 9
        let (dsn, member) = Dsn::parse_member_ref("SYS1.MACLIB").unwrap();
        assert_eq!(dsn.as_str(), "SYS1.MACLIB");
        assert!(member.is_none());
    }

    #[test]
    fn parse_member_ref_case_insensitive() {
        // Validates: Requirement 2 AC 5, AC 9
        let (dsn, member) = Dsn::parse_member_ref("sys1.maclib(open)").unwrap();
        assert_eq!(dsn.as_str(), "SYS1.MACLIB");
        assert_eq!(member.unwrap().as_str(), "OPEN");
    }

    // ── Default HLQ tests ──

    #[test]
    fn with_default_hlq_prepends() {
        // Validates: Requirement 2 AC 6
        let dsn = Dsn::with_default_hlq("INPUT.FILE", "PAYROLL").unwrap();
        assert_eq!(dsn.as_str(), "PAYROLL.INPUT.FILE");
    }

    // ── MemberName tests ──

    #[test]
    fn member_name_valid() {
        // Validates: Requirement 2 AC 8
        let m = MemberName::parse("ABCDEFGH").unwrap();
        assert_eq!(m.as_str(), "ABCDEFGH");
    }

    #[test]
    fn member_name_case_insensitive() {
        // Validates: Requirement 2 AC 8
        let m = MemberName::parse("open").unwrap();
        assert_eq!(m.as_str(), "OPEN");
    }

    #[test]
    fn member_name_rejects_empty() {
        // Validates: Requirement 2 AC 8
        assert!(MemberName::parse("").is_err());
    }

    #[test]
    fn member_name_rejects_too_long() {
        // Validates: Requirement 2 AC 8
        assert!(MemberName::parse("TOOLONGMM").is_err());
    }

    #[test]
    fn member_name_rejects_starting_digit() {
        // Validates: Requirement 2 AC 8
        assert!(MemberName::parse("1BAD").is_err());
    }

    // ── Display/FromStr tests ──

    #[test]
    fn dsn_display_round_trip() {
        // Validates: Requirement 2 AC 1
        let dsn = Dsn::parse("PAYROLL.INPUT").unwrap();
        let displayed = dsn.to_string();
        let reparsed = Dsn::parse(&displayed).unwrap();
        assert_eq!(dsn, reparsed);
    }

    #[test]
    fn dsn_from_str() {
        // Validates: Requirement 2 AC 1
        let dsn: Dsn = "SYS1.MACLIB".parse().unwrap();
        assert_eq!(dsn.as_str(), "SYS1.MACLIB");
    }

    // ── Wildcard matching tests ──

    #[test]
    fn wildcard_star_matches_prefix() {
        // Validates: Requirement 13 AC 9
        let dsn = Dsn::parse("PAYROLL.INPUT.FILE").unwrap();
        assert!(dsn.matches_pattern("PAYROLL.*"));
    }

    #[test]
    fn wildcard_star_matches_all() {
        let dsn = Dsn::parse("ANYTHING.HERE").unwrap();
        assert!(dsn.matches_pattern("*"));
    }

    #[test]
    fn wildcard_percent_matches_one_qualifier() {
        // Validates: Requirement 13 AC 9
        let dsn = Dsn::parse("PAYROLL.INPUT.FILE").unwrap();
        assert!(dsn.matches_pattern("PAYROLL.%.FILE"));
        assert!(!dsn.matches_pattern("PAYROLL.%")); // % matches exactly one qualifier, not two
    }

    #[test]
    fn wildcard_no_match() {
        let dsn = Dsn::parse("PAYROLL.INPUT.FILE").unwrap();
        assert!(!dsn.matches_pattern("OTHER.*"));
    }
}
