//! Dataset name model and validation.
//!
//! Implements DSN syntax validation per z/OS rules:
//! - Maximum 44 characters total
//! - Qualifiers separated by dots, each 1–8 characters
//! - Qualifiers start with alpha or national character (@, #, $)
//! - No empty qualifiers (consecutive dots)

use crate::diagnostic::{DiagnosticCode, LintDiagnostic};

/// A validated dataset name.
///
/// Constructed via `DatasetName::parse()` which enforces z/OS DSN syntax rules.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DatasetName {
    /// The validated dataset name string (uppercase).
    name: String,
}

impl DatasetName {
    /// Parse and validate a dataset name string.
    ///
    /// # Rules
    /// - Maximum 44 characters
    /// - One or more qualifiers separated by dots
    /// - Each qualifier: 1–8 characters, starts with alpha or national (@, #, $)
    /// - Remaining chars: alphanumeric or national
    /// - No empty qualifiers (no consecutive dots, no leading/trailing dot)
    ///
    /// Returns the validated `DatasetName` or a `LintDiagnostic` describing the problem.
    pub fn parse(input: &str, line: usize, col_start: usize) -> Result<Self, LintDiagnostic> {
        let name = input.trim().to_uppercase();

        if name.is_empty() {
            return Err(LintDiagnostic::new(
                DiagnosticCode::InvalidDsnSyntax,
                line,
                (col_start, col_start),
                "DSN is empty",
            ));
        }

        if name.len() > 44 {
            return Err(LintDiagnostic::new(
                DiagnosticCode::InvalidDsnSyntax,
                line,
                (col_start, col_start + name.len()),
                format!("DSN exceeds 44 characters (length: {})", name.len()),
            ));
        }

        // Check for consecutive dots, leading dot, trailing dot
        if name.starts_with('.') || name.ends_with('.') || name.contains("..") {
            return Err(LintDiagnostic::new(
                DiagnosticCode::InvalidDsnSyntax,
                line,
                (col_start, col_start + name.len()),
                "DSN contains empty qualifier (consecutive dots, leading dot, or trailing dot)",
            ));
        }

        let qualifiers: Vec<&str> = name.split('.').collect();
        for qual in &qualifiers {
            if qual.len() > 8 {
                return Err(LintDiagnostic::new(
                    DiagnosticCode::InvalidDsnSyntax,
                    line,
                    (col_start, col_start + name.len()),
                    format!("Qualifier '{}' exceeds 8 characters", qual),
                ));
            }

            let first_char = qual.chars().next().unwrap();
            if !is_alpha_or_national(first_char) {
                return Err(LintDiagnostic::new(
                    DiagnosticCode::InvalidDsnSyntax,
                    line,
                    (col_start, col_start + name.len()),
                    format!(
                        "Qualifier '{}' starts with invalid character '{}' (must start with A-Z, @, #, $)",
                        qual, first_char
                    ),
                ));
            }

            for ch in qual.chars().skip(1) {
                if !is_dsn_char(ch) {
                    return Err(LintDiagnostic::new(
                        DiagnosticCode::InvalidDsnSyntax,
                        line,
                        (col_start, col_start + name.len()),
                        format!(
                            "Qualifier '{}' contains invalid character '{}' (allowed: A-Z, 0-9, @, #, $)",
                            qual, ch
                        ),
                    ));
                }
            }
        }

        Ok(Self { name })
    }

    /// Returns the validated dataset name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// Returns the number of qualifiers in this DSN.
    pub fn qualifier_count(&self) -> usize {
        self.name.split('.').count()
    }
}

impl std::fmt::Display for DatasetName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// A DSN reference extracted from a DD statement's DSN= operand.
///
/// Represents the different forms a DSN can take in JCL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DsnReference {
    /// A fully qualified dataset name: `DSN=MY.DATA.SET`.
    Simple { dsn: String },
    /// A PDS member reference: `DSN=MY.PDS(MEMBER)`.
    Member { pds_dsn: String, member: String },
    /// A temporary dataset: `DSN=&&TEMPNAME`.
    Temporary { name: String },
    /// A referback reference: `DSN=*.STEP1.DDNAME`.
    Referback {
        step_name: String,
        proc_step: Option<String>,
        ddname: String,
    },
    /// A GDG relative generation: `DSN=MY.GDG.BASE(+1)`.
    Gdg { base_name: String, generation: i32 },
}

impl DsnReference {
    /// Returns the raw DSN string for display (before resolution).
    pub fn display_name(&self) -> String {
        match self {
            Self::Simple { dsn } => dsn.clone(),
            Self::Member { pds_dsn, member } => format!("{}({})", pds_dsn, member),
            Self::Temporary { name } => format!("&&{}", name),
            Self::Referback {
                step_name,
                proc_step,
                ddname,
            } => match proc_step {
                Some(ps) => format!("*.{}.{}.{}", step_name, ps, ddname),
                None => format!("*.{}.{}", step_name, ddname),
            },
            Self::Gdg {
                base_name,
                generation,
            } => {
                if *generation >= 0 {
                    format!("{}(+{})", base_name, generation)
                } else {
                    format!("{}({})", base_name, generation)
                }
            }
        }
    }

    /// Returns true if this reference requires catalog lookup.
    pub fn requires_catalog_lookup(&self) -> bool {
        matches!(
            self,
            Self::Simple { .. } | Self::Member { .. } | Self::Gdg { .. }
        )
    }

    /// Returns true if this is a temporary dataset reference.
    pub fn is_temporary(&self) -> bool {
        matches!(self, Self::Temporary { .. })
    }

    /// Returns true if this is a referback reference.
    pub fn is_referback(&self) -> bool {
        matches!(self, Self::Referback { .. })
    }
}

/// Parse a raw DSN string from a DD statement into a `DsnReference`.
///
/// Recognises:
/// - Temporary: `&&name`
/// - Referback: `*.stepname.ddname` or `*.stepname.procstep.ddname`
/// - GDG relative: `base.name(+n)` or `base.name(-n)` or `base.name(0)`
/// - PDS member: `pds.name(MEMBER)` (non-numeric in parens)
/// - Simple: everything else
pub fn parse_dsn_reference(raw: &str) -> DsnReference {
    let trimmed = raw.trim().to_uppercase();

    // Temporary dataset
    if let Some(name_part) = trimmed.strip_prefix("&&") {
        let name = name_part.to_string();
        return DsnReference::Temporary { name };
    }

    // Referback
    if let Some(ref_part) = trimmed.strip_prefix("*.") {
        let parts: Vec<&str> = ref_part.split('.').collect();
        return match parts.len() {
            2 => DsnReference::Referback {
                step_name: parts[0].to_string(),
                proc_step: None,
                ddname: parts[1].to_string(),
            },
            3 => DsnReference::Referback {
                step_name: parts[0].to_string(),
                proc_step: Some(parts[1].to_string()),
                ddname: parts[2].to_string(),
            },
            _ => DsnReference::Simple { dsn: trimmed },
        };
    }

    // Check for parenthesised suffix: GDG or PDS member
    if let Some(paren_start) = trimmed.rfind('(') {
        if let Some(paren_end) = trimmed.rfind(')') {
            if paren_end > paren_start {
                let base = &trimmed[..paren_start];
                let inner = &trimmed[paren_start + 1..paren_end];

                // GDG relative generation: (+n), (-n), (0)
                if let Some(gen) = parse_gdg_generation(inner) {
                    return DsnReference::Gdg {
                        base_name: base.to_string(),
                        generation: gen,
                    };
                }

                // PDS member reference
                return DsnReference::Member {
                    pds_dsn: base.to_string(),
                    member: inner.to_string(),
                };
            }
        }
    }

    DsnReference::Simple { dsn: trimmed }
}

/// Try to parse a GDG generation number from parenthesised content.
/// Returns Some(n) for "+n", "-n", or "0".
fn parse_gdg_generation(inner: &str) -> Option<i32> {
    let trimmed = inner.trim();
    if trimmed == "0" {
        return Some(0);
    }
    if trimmed.starts_with('+') || trimmed.starts_with('-') {
        return trimmed.parse::<i32>().ok();
    }
    None
}

/// Returns true if the character is alphabetic or a national character.
fn is_alpha_or_national(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '@' || ch == '#' || ch == '$'
}

/// Returns true if the character is valid in a DSN qualifier (after the first).
fn is_dsn_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '@' || ch == '#' || ch == '$'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_simple_dsn_parses_successfully() {
        // Validates: Requirement 1 AC 2
        let result = DatasetName::parse("MY.DATA.SET", 1, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "MY.DATA.SET");
    }

    #[test]
    fn valid_single_qualifier_dsn() {
        let result = DatasetName::parse("MYFILE", 1, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "MYFILE");
    }

    #[test]
    fn dsn_with_national_chars_is_valid() {
        // Validates: Requirement 10 AC 7
        let result = DatasetName::parse("@USER.#DATA.$SET", 1, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn dsn_exceeding_44_chars_is_invalid() {
        // Validates: Requirement 10 AC 7
        let long_dsn = "A.BCDEFGH.IJKLMNOP.QRSTUVWX.YZ123456.TOOLONG9";
        assert!(long_dsn.len() > 44);
        let result = DatasetName::parse(long_dsn, 1, 0);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("exceeds 44 characters"));
    }

    #[test]
    fn qualifier_exceeding_8_chars_is_invalid() {
        // Validates: Requirement 10 AC 7
        let result = DatasetName::parse("MY.TOOLONGQUALIFIER.SET", 1, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("exceeds 8 characters"));
    }

    #[test]
    fn qualifier_starting_with_digit_is_invalid() {
        // Validates: Requirement 10 AC 7
        let result = DatasetName::parse("MY.1INVALID.SET", 1, 0);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("starts with invalid character"));
    }

    #[test]
    fn empty_qualifier_consecutive_dots_is_invalid() {
        // Validates: Requirement 10 AC 7
        let result = DatasetName::parse("MY..SET", 1, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("empty qualifier"));
    }

    #[test]
    fn leading_dot_is_invalid() {
        let result = DatasetName::parse(".MY.SET", 1, 0);
        assert!(result.is_err());
    }

    #[test]
    fn trailing_dot_is_invalid() {
        let result = DatasetName::parse("MY.SET.", 1, 0);
        assert!(result.is_err());
    }

    #[test]
    fn parse_dsn_reference_temporary() {
        // Validates: Requirement 6 AC 1
        let result = parse_dsn_reference("&&TEMPFILE");
        assert_eq!(
            result,
            DsnReference::Temporary {
                name: "TEMPFILE".to_string()
            }
        );
    }

    #[test]
    fn parse_dsn_reference_referback_simple() {
        // Validates: Requirement 7 AC 1
        let result = parse_dsn_reference("*.STEP1.SYSUT1");
        assert_eq!(
            result,
            DsnReference::Referback {
                step_name: "STEP1".to_string(),
                proc_step: None,
                ddname: "SYSUT1".to_string(),
            }
        );
    }

    #[test]
    fn parse_dsn_reference_referback_proc_step() {
        // Validates: Requirement 7 AC 1
        let result = parse_dsn_reference("*.STEP1.PROC1.SYSUT1");
        assert_eq!(
            result,
            DsnReference::Referback {
                step_name: "STEP1".to_string(),
                proc_step: Some("PROC1".to_string()),
                ddname: "SYSUT1".to_string(),
            }
        );
    }

    #[test]
    fn parse_dsn_reference_gdg_positive() {
        // Validates: Requirement 8 AC 1
        let result = parse_dsn_reference("MY.GDG.BASE(+1)");
        assert_eq!(
            result,
            DsnReference::Gdg {
                base_name: "MY.GDG.BASE".to_string(),
                generation: 1,
            }
        );
    }

    #[test]
    fn parse_dsn_reference_gdg_negative() {
        // Validates: Requirement 8 AC 1
        let result = parse_dsn_reference("MY.GDG.BASE(-2)");
        assert_eq!(
            result,
            DsnReference::Gdg {
                base_name: "MY.GDG.BASE".to_string(),
                generation: -2,
            }
        );
    }

    #[test]
    fn parse_dsn_reference_gdg_zero() {
        // Validates: Requirement 8 AC 1
        let result = parse_dsn_reference("MY.GDG.BASE(0)");
        assert_eq!(
            result,
            DsnReference::Gdg {
                base_name: "MY.GDG.BASE".to_string(),
                generation: 0,
            }
        );
    }

    #[test]
    fn parse_dsn_reference_pds_member() {
        // Validates: Requirement 1 AC 3
        let result = parse_dsn_reference("MY.PDS(MEMBER1)");
        assert_eq!(
            result,
            DsnReference::Member {
                pds_dsn: "MY.PDS".to_string(),
                member: "MEMBER1".to_string(),
            }
        );
    }

    #[test]
    fn parse_dsn_reference_simple() {
        // Validates: Requirement 1 AC 2
        let result = parse_dsn_reference("MY.SIMPLE.DATASET");
        assert_eq!(
            result,
            DsnReference::Simple {
                dsn: "MY.SIMPLE.DATASET".to_string()
            }
        );
    }

    #[test]
    fn dsn_reference_display_name_all_variants() {
        let simple = DsnReference::Simple {
            dsn: "A.B".to_string(),
        };
        assert_eq!(simple.display_name(), "A.B");

        let member = DsnReference::Member {
            pds_dsn: "A.B".to_string(),
            member: "M".to_string(),
        };
        assert_eq!(member.display_name(), "A.B(M)");

        let temp = DsnReference::Temporary {
            name: "T1".to_string(),
        };
        assert_eq!(temp.display_name(), "&&T1");

        let rb = DsnReference::Referback {
            step_name: "S1".to_string(),
            proc_step: None,
            ddname: "DD1".to_string(),
        };
        assert_eq!(rb.display_name(), "*.S1.DD1");

        let gdg = DsnReference::Gdg {
            base_name: "G.B".to_string(),
            generation: 1,
        };
        assert_eq!(gdg.display_name(), "G.B(+1)");
    }
}
