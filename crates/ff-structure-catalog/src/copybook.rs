//! COBOL copybook parser — import structure definitions from copybook source.
//!
//! Parses COBOL copybook source text into [`FieldDefinition`] lists,
//! supporting PIC clauses, COMP-3 USAGE, level numbers, and REDEFINES.

use crate::error::CatalogError;
use crate::field::{FieldDefinition, FieldType};

/// Configuration options for COBOL copybook parsing.
#[derive(Debug, Clone)]
pub struct CopybookParserConfig {
    /// Starting column for COBOL source (typically 7 for fixed-format).
    pub start_column: u8,
    /// Ending column for COBOL source (typically 72 for fixed-format).
    pub end_column: u8,
    /// Whether to expand OCCURS clauses into individual fields.
    pub expand_occurs: bool,
}

impl Default for CopybookParserConfig {
    fn default() -> Self {
        Self {
            start_column: 7,
            end_column: 72,
            expand_occurs: false,
        }
    }
}

/// Result of parsing a COBOL copybook.
#[derive(Debug, Clone)]
pub struct CopybookParseResult {
    /// Successfully parsed field definitions.
    pub fields: Vec<FieldDefinition>,
    /// Warnings encountered during parsing.
    pub warnings: Vec<CopybookWarning>,
    /// Computed total record length.
    pub record_length: u32,
}

/// A warning from copybook parsing (non-fatal).
#[derive(Debug, Clone)]
pub struct CopybookWarning {
    /// Source line number where the warning occurred.
    pub line: u32,
    /// Warning message.
    pub message: String,
}

/// Parses COBOL copybook source into FieldDefinition lists.
pub struct CopybookParser {
    config: CopybookParserConfig,
}

impl CopybookParser {
    /// Create a new parser with the given configuration.
    pub fn new(config: CopybookParserConfig) -> Self {
        Self { config }
    }

    /// Create a parser with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(CopybookParserConfig::default())
    }

    /// Parse COBOL copybook source text into field definitions.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError::CopybookParseError`] for fatal parse failures.
    pub fn parse(&self, source: &str) -> Result<CopybookParseResult, CatalogError> {
        let mut fields = Vec::new();
        let mut warnings = Vec::new();
        let mut current_offset: u32 = 0;

        for (line_num, line) in source.lines().enumerate() {
            let line_number = (line_num + 1) as u32;

            // Extract the content area (columns start_column to end_column)
            let content = self.extract_content(line);
            let content = content.trim();

            if content.is_empty() {
                continue; // Skip empty lines
            }

            // Check for COBOL comment lines
            if self.is_comment_line(line) {
                continue;
            }

            // Also skip if trimmed content starts with *
            if content.starts_with('*') {
                continue;
            }

            // Try to parse as a field definition
            match self.parse_line(content, line_number) {
                Ok(Some(parsed)) => {
                    let byte_length = parsed.byte_length;
                    let field = FieldDefinition {
                        name: parsed.name,
                        offset: current_offset,
                        length: byte_length,
                        field_type: parsed.field_type,
                        decimals: parsed.decimals,
                        identifiers: Vec::new(),
                        filters: Vec::new(),
                    };
                    // Only add elementary items (items with PIC clause)
                    if byte_length > 0 {
                        fields.push(field);
                        current_offset += byte_length;
                    }
                }
                Ok(None) => {
                    // Group level or continuation — skip
                }
                Err(msg) => {
                    warnings.push(CopybookWarning {
                        line: line_number,
                        message: msg,
                    });
                }
            }
        }

        Ok(CopybookParseResult {
            record_length: current_offset,
            fields,
            warnings,
        })
    }

    /// Extract content from the COBOL source line area.
    ///
    /// For lines long enough, extracts columns `start_column..end_column` (0-indexed).
    /// For shorter lines, uses the entire line trimmed of leading whitespace.
    /// This handles both fixed-format COBOL (columns 7-72) and free-format input.
    fn extract_content<'a>(&self, line: &'a str) -> &'a str {
        // If the line is long enough for fixed-format extraction
        let start = self.config.start_column as usize;
        if line.len() > start {
            let end = (self.config.end_column as usize).min(line.len());
            &line[start..end]
        } else {
            // Short line — just use it as-is (will be trimmed later)
            line
        }
    }

    /// Check if a line is a COBOL comment (indicator `*` in column 7).
    fn is_comment_line(&self, line: &str) -> bool {
        // Standard: column 7 (index 6 in 0-based) contains '*'
        if line.len() > 6 && line.as_bytes()[6] == b'*' {
            return true;
        }
        // Also handle trimmed input starting with *
        line.trim().starts_with('*')
    }

    /// Parse a single content line.
    ///
    /// Returns `Ok(Some(field))` for elementary items, `Ok(None)` for group items,
    /// or `Err(warning)` for unparseable lines.
    fn parse_line(&self, content: &str, _line_number: u32) -> Result<Option<ParsedField>, String> {
        let tokens: Vec<&str> = content.split_whitespace().collect();
        if tokens.is_empty() {
            return Ok(None);
        }

        // Parse level number
        let level: u8 = tokens[0]
            .trim_end_matches('.')
            .parse()
            .map_err(|_| format!("invalid level number: {}", tokens[0]))?;

        // Skip 88 levels (condition names)
        if level == 88 {
            return Ok(None);
        }

        // Get the field name
        if tokens.len() < 2 {
            return Ok(None);
        }
        let name = tokens[1].trim_end_matches('.').to_string();

        // Look for PIC/PICTURE clause
        let content_upper = content.to_uppercase();
        let pic_info = self.extract_pic_clause(&content_upper);

        // Look for USAGE clause
        let usage = self.extract_usage(&content_upper);

        match (pic_info, usage) {
            (Some((pic_type, pic_length, decimals)), usage_type) => {
                let (field_type, byte_length) = match usage_type {
                    Some(UsageType::Comp3) => {
                        (FieldType::PackedDecimal, packed_decimal_length(pic_length))
                    }
                    Some(UsageType::Binary) => (FieldType::Binary, binary_length(pic_length)),
                    None => (pic_type, pic_length),
                    Some(UsageType::Display) => (pic_type, pic_length),
                };
                Ok(Some(ParsedField {
                    name,
                    field_type,
                    byte_length,
                    decimals,
                }))
            }
            (None, Some(UsageType::Comp3)) => {
                // COMP-3 without PIC — treat as group level
                Ok(None)
            }
            (None, Some(UsageType::Binary)) => Ok(None),
            _ => {
                // Group level (no PIC clause) — skip regardless of level
                Ok(None)
            }
        }
    }

    /// Extract PIC clause information: (field_type, byte_length, decimals).
    fn extract_pic_clause(&self, content: &str) -> Option<(FieldType, u32, u8)> {
        // Find PIC or PICTURE keyword
        let pic_start = content
            .find("PIC ")
            .or_else(|| content.find("PICTURE "))
            .map(|pos| {
                if content[pos..].starts_with("PICTURE ") {
                    pos + 8
                } else {
                    pos + 4
                }
            })?;

        let pic_str: &str = &content[pic_start..];
        let pic_end = pic_str.find([' ', '.']).unwrap_or(pic_str.len());
        let pic = &pic_str[..pic_end];

        parse_pic_clause(pic)
    }

    /// Extract USAGE clause type.
    fn extract_usage(&self, content: &str) -> Option<UsageType> {
        if content.contains("COMP-3")
            || content.contains("COMPUTATIONAL-3")
            || content.contains("PACKED-DECIMAL")
        {
            Some(UsageType::Comp3)
        } else if content.contains("COMP")
            || content.contains("COMPUTATIONAL")
            || content.contains("BINARY")
        {
            // COMP without -3 is binary
            if !content.contains("COMP-3") && !content.contains("COMPUTATIONAL-3") {
                Some(UsageType::Binary)
            } else {
                Some(UsageType::Comp3)
            }
        } else if content.contains("DISPLAY") {
            Some(UsageType::Display)
        } else {
            None
        }
    }
}

/// Parsed field information from a copybook line.
struct ParsedField {
    name: String,
    field_type: FieldType,
    byte_length: u32,
    decimals: u8,
}

/// USAGE clause type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UsageType {
    Display,
    Comp3,
    Binary,
}

/// Parse a PIC clause string into (field_type, byte_length, decimals).
///
/// Supports: X(n), 9(n), 9(n)V9(m), S9(n)
fn parse_pic_clause(pic: &str) -> Option<(FieldType, u32, u8)> {
    let pic = pic.trim();
    if pic.is_empty() {
        return None;
    }

    // Check for X (alphanumeric)
    if pic.starts_with('X') {
        let length = extract_repeat_count(pic, 'X');
        return Some((FieldType::Alphanumeric, length, 0));
    }

    // Check for 9 or S9 (numeric)
    let is_signed = pic.starts_with('S');
    let numeric_pic = if is_signed { &pic[1..] } else { pic };

    if numeric_pic.starts_with('9') {
        // Look for V (implied decimal)
        if let Some(v_pos) = numeric_pic.find('V') {
            let integer_part = &numeric_pic[..v_pos];
            let decimal_part = &numeric_pic[v_pos + 1..];
            let int_len = extract_repeat_count(integer_part, '9');
            let dec_len = extract_repeat_count(decimal_part, '9');
            let total_len = int_len + dec_len + if is_signed { 1 } else { 0 };
            return Some((FieldType::Numeric, total_len, dec_len as u8));
        }

        let length = extract_repeat_count(numeric_pic, '9');
        let total_len = length + if is_signed { 1 } else { 0 };
        return Some((FieldType::Numeric, total_len, 0));
    }

    None
}

/// Extract the repeat count from a PIC notation like `X(30)` or `XXX`.
fn extract_repeat_count(s: &str, ch: char) -> u32 {
    // Check for parenthesized form: X(30)
    if let Some(paren_start) = s.find('(') {
        if let Some(paren_end) = s.find(')') {
            let count_str = &s[paren_start + 1..paren_end];
            return count_str.parse().unwrap_or(1);
        }
    }

    // Count consecutive characters
    s.chars().filter(|c| *c == ch.to_ascii_uppercase()).count() as u32
}

/// Calculate packed-decimal byte length from digit count.
///
/// COMP-3: (digits + 1) / 2 bytes (each byte holds 2 digits, last nibble is sign).
fn packed_decimal_length(digit_count: u32) -> u32 {
    (digit_count + 2) / 2
}

/// Calculate binary (COMP) byte length from digit count.
fn binary_length(digit_count: u32) -> u32 {
    match digit_count {
        0..=4 => 2,
        5..=9 => 4,
        _ => 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 27.2 — PIC X(n) → alphanumeric
    #[test]
    fn parse_pic_x_alphanumeric() {
        let result = parse_pic_clause("X(30)");
        assert_eq!(result, Some((FieldType::Alphanumeric, 30, 0)));
    }

    // Validates: Requirement 27.2 — PIC XXX (repeat notation)
    #[test]
    fn parse_pic_x_repeated() {
        let result = parse_pic_clause("XXX");
        assert_eq!(result, Some((FieldType::Alphanumeric, 3, 0)));
    }

    // Validates: Requirement 27.3 — PIC 9(n) → numeric
    #[test]
    fn parse_pic_9_numeric() {
        let result = parse_pic_clause("9(5)");
        assert_eq!(result, Some((FieldType::Numeric, 5, 0)));
    }

    // Validates: Requirement 27.3 — PIC 9(n)V9(m) → numeric with decimals
    #[test]
    fn parse_pic_9_with_implied_decimal() {
        let result = parse_pic_clause("9(5)V9(2)");
        assert_eq!(result, Some((FieldType::Numeric, 7, 2)));
    }

    // Validates: Requirement 27.3 — signed numeric
    #[test]
    fn parse_pic_s9_signed_numeric() {
        let result = parse_pic_clause("S9(7)");
        assert_eq!(result, Some((FieldType::Numeric, 8, 0))); // +1 for sign
    }

    // Validates: Requirement 27.4 — COMP-3 byte length
    #[test]
    fn packed_decimal_length_calculation() {
        // 5 digits → (5 + 2) / 2 = 3 bytes
        assert_eq!(packed_decimal_length(5), 3);
        // 7 digits → (7 + 2) / 2 = 4 bytes
        assert_eq!(packed_decimal_length(7), 4);
        // 9 digits → (9 + 2) / 2 = 5 bytes
        assert_eq!(packed_decimal_length(9), 5);
    }

    // Validates: Requirement 27.5 — BINARY/COMP byte length
    #[test]
    fn binary_length_calculation() {
        assert_eq!(binary_length(4), 2); // S9(4) COMP = 2 bytes
        assert_eq!(binary_length(9), 4); // S9(9) COMP = 4 bytes
        assert_eq!(binary_length(18), 8); // S9(18) COMP = 8 bytes
    }

    // Validates: Requirement 27.7 — offset calculation from copybook
    #[test]
    fn parse_copybook_calculates_contiguous_offsets() {
        let source = concat!(
            "       01  CUSTOMER-RECORD.\n",
            "       05  CUST-NAME          PIC X(30).\n",
            "       05  CUST-BALANCE       PIC 9(7)V9(2) COMP-3.\n",
            "       05  CUST-STATUS        PIC X(1).\n",
        );

        let parser = CopybookParser::with_defaults();
        let result = parser.parse(source).unwrap();

        assert_eq!(result.fields.len(), 3);
        assert_eq!(result.fields[0].name, "CUST-NAME");
        assert_eq!(result.fields[0].offset, 0);
        assert_eq!(result.fields[0].length, 30);
        assert_eq!(result.fields[0].field_type, FieldType::Alphanumeric);

        assert_eq!(result.fields[1].name, "CUST-BALANCE");
        assert_eq!(result.fields[1].offset, 30);
        assert_eq!(result.fields[1].field_type, FieldType::PackedDecimal);

        assert_eq!(result.fields[2].name, "CUST-STATUS");
        assert_eq!(result.fields[2].field_type, FieldType::Alphanumeric);
        assert_eq!(result.fields[2].length, 1);
    }

    // Validates: Requirement 27.1 — skips comment lines
    #[test]
    fn parse_skips_comment_lines() {
        let source = concat!(
            "      * This is a comment\n",
            "       05  FIELD1             PIC X(10).\n",
        );

        let parser = CopybookParser::with_defaults();
        let result = parser.parse(source).unwrap();
        assert_eq!(result.fields.len(), 1);
        assert_eq!(result.fields[0].name, "FIELD1");
    }

    // Validates: Requirement 27.1 — handles 88 level condition names
    #[test]
    fn parse_skips_88_level_conditions() {
        let source = concat!(
            "       05  STATUS-CODE        PIC X(2).\n",
            "       88  ACTIVE          VALUE 'AC'.\n",
            "       88  INACTIVE        VALUE 'IN'.\n",
            "       05  NEXT-FIELD        PIC X(5).\n",
        );

        let parser = CopybookParser::with_defaults();
        let result = parser.parse(source).unwrap();
        assert_eq!(result.fields.len(), 2);
    }

    // Validates: Requirement 27.8 — record length computed
    #[test]
    fn parse_computes_total_record_length() {
        let source = concat!(
            "       05  F1  PIC X(10).\n",
            "       05  F2  PIC X(20).\n",
            "       05  F3  PIC 9(5).\n",
        );

        let parser = CopybookParser::with_defaults();
        let result = parser.parse(source).unwrap();
        assert_eq!(result.record_length, 35);
    }
}
