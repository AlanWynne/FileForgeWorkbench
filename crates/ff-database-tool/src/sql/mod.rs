//! SQL statement parsing — boundary detection and parameter extraction.

/// A parsed SQL statement boundary within a script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementBoundary {
    /// Start byte offset in the script (inclusive).
    pub start: usize,
    /// End byte offset in the script (exclusive, includes delimiter).
    pub end: usize,
    /// The SQL text of this statement (without trailing delimiter).
    pub text: String,
}

/// SQL statement parser — detects statement boundaries in a script.
///
/// Respects string literals, quoted identifiers, comments, and nested blocks
/// so that delimiters inside these constructs are not treated as terminators.
pub struct SqlParser {
    /// Statement delimiter. Default: ';'.
    delimiter: char,
}

impl SqlParser {
    /// Create a new parser with the default delimiter (';').
    pub fn new() -> Self {
        Self { delimiter: ';' }
    }

    /// Create a parser with a custom delimiter.
    pub fn with_delimiter(delimiter: char) -> Self {
        Self { delimiter }
    }

    /// Split a SQL script into individual statement boundaries.
    ///
    /// Respects:
    /// - Single-quoted string literals (`'...'`)
    /// - Double-quoted identifiers (`"..."`)
    /// - Line comments (`-- ...`)
    /// - Block comments (`/* ... */`)
    pub fn split_statements(&self, script: &str) -> Vec<StatementBoundary> {
        let mut statements = Vec::new();
        let chars: Vec<char> = script.chars().collect();
        let len = chars.len();
        let mut i = 0;
        let mut stmt_start = 0;

        while i < len {
            match chars[i] {
                // Single-quoted string literal
                '\'' => {
                    i += 1;
                    while i < len {
                        if chars[i] == '\'' {
                            i += 1;
                            // Handle escaped quote ''
                            if i < len && chars[i] == '\'' {
                                i += 1;
                            } else {
                                break;
                            }
                        } else {
                            i += 1;
                        }
                    }
                }
                // Double-quoted identifier
                '"' => {
                    i += 1;
                    while i < len && chars[i] != '"' {
                        i += 1;
                    }
                    if i < len {
                        i += 1;
                    }
                }
                // Line comment
                '-' if i + 1 < len && chars[i + 1] == '-' => {
                    while i < len && chars[i] != '\n' {
                        i += 1;
                    }
                }
                // Block comment
                '/' if i + 1 < len && chars[i + 1] == '*' => {
                    i += 2;
                    while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                        i += 1;
                    }
                    if i + 1 < len {
                        i += 2;
                    }
                }
                // Statement delimiter
                c if c == self.delimiter => {
                    let text = script[stmt_start..i].trim().to_string();
                    if !text.is_empty() {
                        statements.push(StatementBoundary {
                            start: stmt_start,
                            end: i + 1,
                            text,
                        });
                    }
                    i += 1;
                    stmt_start = i;
                }
                _ => {
                    i += 1;
                }
            }
        }

        // Handle trailing statement without delimiter
        let text = script[stmt_start..].trim().to_string();
        if !text.is_empty() {
            statements.push(StatementBoundary {
                start: stmt_start,
                end: len,
                text,
            });
        }

        statements
    }

    /// Find the statement containing the given byte offset.
    pub fn statement_at_offset(&self, script: &str, offset: usize) -> Option<StatementBoundary> {
        let statements = self.split_statements(script);
        statements
            .into_iter()
            .find(|s| s.start <= offset && offset < s.end)
    }

    /// Detect parameter placeholders in a SQL statement.
    ///
    /// Returns a list of (name_or_position, start_offset) tuples.
    pub fn detect_parameters(&self, sql: &str) -> Vec<(String, usize)> {
        let mut params = Vec::new();
        let chars: Vec<char> = sql.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            match chars[i] {
                // Skip string literals
                '\'' => {
                    i += 1;
                    while i < len && chars[i] != '\'' {
                        i += 1;
                    }
                    if i < len {
                        i += 1;
                    }
                }
                // $N positional parameter
                '$' if i + 1 < len && chars[i + 1].is_ascii_digit() => {
                    let start = i;
                    i += 1;
                    let mut num = String::new();
                    while i < len && chars[i].is_ascii_digit() {
                        num.push(chars[i]);
                        i += 1;
                    }
                    params.push((format!("${num}"), start));
                }
                // :name named parameter
                ':' if i + 1 < len && (chars[i + 1].is_alphabetic() || chars[i + 1] == '_') => {
                    let start = i;
                    i += 1;
                    let mut name = String::new();
                    while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        name.push(chars[i]);
                        i += 1;
                    }
                    params.push((format!(":{name}"), start));
                }
                // @variable named parameter
                '@' if i + 1 < len && (chars[i + 1].is_alphabetic() || chars[i + 1] == '_') => {
                    let start = i;
                    i += 1;
                    let mut name = String::new();
                    while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        name.push(chars[i]);
                        i += 1;
                    }
                    params.push((format!("@{name}"), start));
                }
                _ => {
                    i += 1;
                }
            }
        }

        params
    }
}

impl Default for SqlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_simple_statements() {
        // Validates: Requirement 5 AC 2, AC 3
        let parser = SqlParser::new();
        let script = "SELECT 1; SELECT 2; SELECT 3";
        let stmts = parser.split_statements(script);
        assert_eq!(stmts.len(), 3);
        assert_eq!(stmts[0].text, "SELECT 1");
        assert_eq!(stmts[1].text, "SELECT 2");
        assert_eq!(stmts[2].text, "SELECT 3");
    }

    #[test]
    fn delimiter_inside_string_not_split() {
        // Validates: Requirement 5 AC 3
        let parser = SqlParser::new();
        let script = "SELECT 'hello; world'";
        let stmts = parser.split_statements(script);
        assert_eq!(stmts.len(), 1);
        assert_eq!(stmts[0].text, "SELECT 'hello; world'");
    }

    #[test]
    fn delimiter_inside_line_comment_not_split() {
        // Validates: Requirement 5 AC 3
        let parser = SqlParser::new();
        let script = "SELECT 1 -- this; is a comment\n";
        let stmts = parser.split_statements(script);
        assert_eq!(stmts.len(), 1);
    }

    #[test]
    fn delimiter_inside_block_comment_not_split() {
        // Validates: Requirement 5 AC 3
        let parser = SqlParser::new();
        let script = "SELECT /* ; */ 1";
        let stmts = parser.split_statements(script);
        assert_eq!(stmts.len(), 1);
    }

    #[test]
    fn empty_statements_skipped() {
        let parser = SqlParser::new();
        let script = "SELECT 1;;SELECT 2";
        let stmts = parser.split_statements(script);
        assert_eq!(stmts.len(), 2);
    }

    #[test]
    fn trailing_statement_without_delimiter() {
        let parser = SqlParser::new();
        let script = "SELECT 1; SELECT 2";
        let stmts = parser.split_statements(script);
        assert_eq!(stmts.len(), 2);
        assert_eq!(stmts[1].text, "SELECT 2");
    }

    #[test]
    fn statement_at_offset() {
        // Validates: Requirement 5 AC 11
        let parser = SqlParser::new();
        let script = "SELECT 1; SELECT 2";
        let stmt = parser.statement_at_offset(script, 12).unwrap();
        assert_eq!(stmt.text, "SELECT 2");
    }

    #[test]
    fn detect_positional_parameters() {
        // Validates: Requirement 7 AC 1, AC 4
        let parser = SqlParser::new();
        let params = parser.detect_parameters("SELECT $1, $2 FROM t WHERE id = $1");
        let names: Vec<_> = params.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"$1"));
        assert!(names.contains(&"$2"));
    }

    #[test]
    fn detect_named_colon_parameters() {
        // Validates: Requirement 7 AC 1, AC 3
        let parser = SqlParser::new();
        let params = parser.detect_parameters("SELECT :name, :age FROM t");
        let names: Vec<_> = params.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&":name"));
        assert!(names.contains(&":age"));
    }

    #[test]
    fn detect_at_variable_parameters() {
        // Validates: Requirement 7 AC 1
        let parser = SqlParser::new();
        let params = parser.detect_parameters("SELECT @salary FROM emp WHERE @dept = dept");
        let names: Vec<_> = params.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"@salary"));
        assert!(names.contains(&"@dept"));
    }

    #[test]
    fn no_parameters_in_plain_sql() {
        let parser = SqlParser::new();
        let params = parser.detect_parameters("SELECT id, name FROM users");
        assert!(params.is_empty());
    }

    #[test]
    fn parameters_not_detected_in_string_literals() {
        // Validates: Requirement 7 AC 8
        let parser = SqlParser::new();
        let params = parser.detect_parameters("SELECT '$1 is not a param'");
        assert!(params.is_empty());
    }
}
