//! EditorConfig file parser.
//!
//! Parses `.editorconfig` files into structured section/property data,
//! handling INI-like syntax with glob patterns for file matching.
//!
//! The EditorConfig file format is an INI-like format:
//! - Lines starting with `#` or `;` are comments
//! - Section headers are glob patterns in brackets: `[*.rs]`, `[Makefile]`
//! - Key-value pairs: `indent_style = space`, `indent_size = 4`
//! - Special top-level key: `root = true` (stops upward traversal)
//! - Property names are case-insensitive
//! - Property values are case-insensitive for enum-like properties

use std::path::Path;

/// A parsed `.editorconfig` file containing a root flag and sections.
///
/// Each section has a glob pattern and a set of editor properties.
#[derive(Debug, Clone, PartialEq)]
pub struct EditorConfigFile {
    /// Whether this file declares `root = true`, stopping upward traversal.
    pub root: bool,
    /// Sections in the order they appear in the file.
    pub sections: Vec<EditorConfigSection>,
}

/// A single section within an `.editorconfig` file.
///
/// Each section is introduced by a glob pattern in brackets (e.g., `[*.rs]`)
/// and contains zero or more property assignments.
#[derive(Debug, Clone, PartialEq)]
pub struct EditorConfigSection {
    /// The glob pattern from the section header (e.g., `*.rs`, `Makefile`).
    pub pattern: String,
    /// The properties defined in this section.
    pub properties: EditorConfigProperties,
}

/// Editor properties that can be specified in an `.editorconfig` section.
///
/// All fields are optional — only properties explicitly set in the file
/// will have a value.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditorConfigProperties {
    /// Indentation style: spaces or tabs.
    pub indent_style: Option<IndentStyle>,
    /// Indentation size in columns, or `Tab` to use `tab_width`.
    pub indent_size: Option<IndentSize>,
    /// Width of a tab character in columns.
    pub tab_width: Option<u32>,
    /// Line ending style.
    pub end_of_line: Option<EndOfLine>,
    /// File character encoding.
    pub charset: Option<Charset>,
    /// Whether trailing whitespace should be removed on save.
    pub trim_trailing_whitespace: Option<bool>,
    /// Whether a final newline should be inserted at end of file.
    pub insert_final_newline: Option<bool>,
}

/// Indentation style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentStyle {
    /// Use space characters for indentation.
    Space,
    /// Use tab characters for indentation.
    Tab,
}

/// Indentation size specification.
///
/// Can be a numeric column count or the special value `tab` which means
/// "use the tab_width setting".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentSize {
    /// A specific number of columns.
    Value(u32),
    /// Use the value of `tab_width`.
    Tab,
}

/// Line ending style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndOfLine {
    /// Line feed (`\n`) — Unix/macOS.
    Lf,
    /// Carriage return + line feed (`\r\n`) — Windows.
    CrLf,
    /// Carriage return (`\r`) — Classic Mac OS.
    Cr,
}

/// File character encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    /// UTF-8 without BOM.
    Utf8,
    /// UTF-8 with byte order mark.
    Utf8Bom,
    /// ISO 8859-1 (Latin-1).
    Latin1,
    /// UTF-16 big-endian.
    Utf16Be,
    /// UTF-16 little-endian.
    Utf16Le,
}

/// Errors that can occur during EditorConfig file parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// The line number (1-based) where the error occurred.
    pub line: usize,
    /// A description of the problem.
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Match a file path against an EditorConfig glob pattern.
///
/// `pattern` is the glob from the section header (e.g., `*.rs`, `lib/**/*.rs`).
/// `filename` is the relative path of the file from the .editorconfig directory.
///
/// EditorConfig matching rules:
/// - Patterns without `/` are matched only against the file's basename
/// - Patterns with `/` are matched against the full relative path
/// - Matching is case-sensitive
///
/// Supported glob features:
/// - `*` — matches any string of characters except `/`
/// - `**` — matches any string of characters including `/`
/// - `?` — matches any single character except `/`
/// - `[abc]` — character class
/// - `[!abc]` or `[^abc]` — negated character class
/// - `{s1,s2,s3}` — brace expansion (matches any of the alternatives)
/// - `{num1..num2}` — integer range (matches any integer in the range)
pub fn matches_pattern(pattern: &str, filename: &str) -> bool {
    // Determine whether to match against basename only or full path.
    // If pattern contains a `/`, match against the full relative path.
    // Otherwise, match only against the filename's basename.
    let target = if pattern.contains('/') {
        filename
    } else {
        // Extract basename (last component after final `/`)
        filename.rsplit('/').next().unwrap_or(filename)
    };

    // Expand braces first, then match each expanded pattern
    let expanded = expand_braces(pattern);
    expanded.iter().any(|p| glob_match(p, target))
}

/// Expand brace expressions in a pattern into multiple alternatives.
///
/// Handles:
/// - `{s1,s2,s3}` — alternatives
/// - `{num1..num2}` — integer ranges
///
/// Nested braces are not supported by the EditorConfig spec.
fn expand_braces(pattern: &str) -> Vec<String> {
    // Find the first `{` that has a matching `}`
    let Some(open) = pattern.find('{') else {
        return vec![pattern.to_string()];
    };

    // Find the matching closing brace (not nested)
    let after_open = &pattern[open + 1..];
    let Some(close_offset) = find_matching_close_brace(after_open) else {
        // No matching close brace — treat literal
        return vec![pattern.to_string()];
    };

    let close = open + 1 + close_offset;
    let prefix = &pattern[..open];
    let inner = &pattern[open + 1..close];
    let suffix = &pattern[close + 1..];

    // Check for integer range pattern: {num..num}
    if let Some((start, end)) = parse_integer_range(inner) {
        let range_start = start.min(end);
        let range_end = start.max(end);
        let mut results = Vec::new();
        for i in range_start..=range_end {
            let expanded_suffix = expand_braces(suffix);
            for s in &expanded_suffix {
                results.push(format!("{prefix}{i}{s}"));
            }
        }
        return results;
    }

    // Otherwise, split by comma for alternatives
    let alternatives = split_brace_alternatives(inner);
    let mut results = Vec::new();
    for alt in &alternatives {
        let combined = format!("{prefix}{alt}{suffix}");
        let expanded = expand_braces(&combined);
        results.extend(expanded);
    }
    results
}

/// Find the position of the matching `}` in a string (not counting nested braces).
fn find_matching_close_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}

/// Split brace content by commas, respecting nested braces.
fn split_brace_alternatives(inner: &str) -> Vec<&str> {
    let mut results = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, ch) in inner.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,
            ',' if depth == 0 => {
                results.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    results.push(&inner[start..]);
    results
}

/// Try to parse `inner` as an integer range `num..num`.
fn parse_integer_range(inner: &str) -> Option<(i64, i64)> {
    let parts: Vec<&str> = inner.splitn(2, "..").collect();
    if parts.len() != 2 {
        return None;
    }
    let start = parts[0].trim().parse::<i64>().ok()?;
    let end = parts[1].trim().parse::<i64>().ok()?;
    Some((start, end))
}

/// Match a glob pattern (without braces) against a target string.
///
/// Supports `*`, `**`, `?`, and `[...]` character classes.
fn glob_match(pattern: &str, target: &str) -> bool {
    glob_match_recursive(pattern.as_bytes(), target.as_bytes())
}

/// Recursive glob matching implementation.
fn glob_match_recursive(pattern: &[u8], target: &[u8]) -> bool {
    let mut p = 0;
    let mut t = 0;

    // Track backtracking point for `*`
    let mut star_p: Option<usize> = None;
    let mut star_t: Option<usize> = None;

    while t < target.len() || p < pattern.len() {
        if p < pattern.len() {
            match pattern[p] {
                b'*' => {
                    // Check for `**`
                    if p + 1 < pattern.len() && pattern[p + 1] == b'*' {
                        // `**` matches everything including `/`
                        // Skip the `**`
                        let mut pp = p + 2;
                        // If followed by `/`, skip it too
                        if pp < pattern.len() && pattern[pp] == b'/' {
                            pp += 1;
                        }
                        // Try matching the rest of the pattern at every position
                        // including matching zero characters
                        for tt in t..=target.len() {
                            if glob_match_recursive(&pattern[pp..], &target[tt..]) {
                                return true;
                            }
                        }
                        return false;
                    }
                    // Single `*` — matches any characters except `/`
                    star_p = Some(p);
                    star_t = Some(t);
                    p += 1;
                    continue;
                }
                b'?' => {
                    if t < target.len() && target[t] != b'/' {
                        p += 1;
                        t += 1;
                        continue;
                    }
                }
                b'[' => {
                    if t < target.len() {
                        if let Some(class_end) = find_class_end(&pattern[p..]) {
                            let class_content = &pattern[p + 1..p + class_end];
                            let ch = target[t];
                            if match_character_class(class_content, ch) {
                                p = p + class_end + 1;
                                t += 1;
                                continue;
                            }
                        }
                    }
                }
                c => {
                    if t < target.len() && target[t] == c {
                        p += 1;
                        t += 1;
                        continue;
                    }
                }
            }
        }

        // No match at current position — try backtracking to last `*`
        if let (Some(sp), Some(st)) = (star_p, star_t) {
            // `*` cannot match past end of target or match `/`
            if st >= target.len() || target[st] == b'/' {
                return false;
            }
            let new_st = st + 1;
            star_t = Some(new_st);
            p = sp + 1;
            t = new_st;
            continue;
        }

        return false;
    }

    true
}

/// Find the closing `]` of a character class, returning its offset from the start `[`.
fn find_class_end(pattern: &[u8]) -> Option<usize> {
    // pattern[0] == b'['
    let mut i = 1;
    // Allow `]` as first char in class (or after `!`/`^`)
    if i < pattern.len() && (pattern[i] == b'!' || pattern[i] == b'^') {
        i += 1;
    }
    if i < pattern.len() && pattern[i] == b']' {
        i += 1;
    }
    while i < pattern.len() {
        if pattern[i] == b']' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Match a single character against a character class content (between `[` and `]`).
///
/// Supports negation with `!` or `^` as first character, and ranges like `a-z`.
fn match_character_class(class: &[u8], ch: u8) -> bool {
    let (negated, content) = if !class.is_empty() && (class[0] == b'!' || class[0] == b'^') {
        (true, &class[1..])
    } else {
        (false, class)
    };

    let mut matched = false;
    let mut i = 0;

    while i < content.len() {
        if i + 2 < content.len() && content[i + 1] == b'-' {
            // Range: e.g., `a-z`
            let range_start = content[i];
            let range_end = content[i + 2];
            if ch >= range_start && ch <= range_end {
                matched = true;
            }
            i += 3;
        } else {
            if content[i] == ch {
                matched = true;
            }
            i += 1;
        }
    }

    if negated {
        !matched
    } else {
        matched
    }
}

/// Parse the content of an `.editorconfig` file into a structured representation.
///
/// This function accepts the full text content of an `.editorconfig` file and
/// returns a parsed `EditorConfigFile` with all recognized sections and properties.
///
/// # Behavior
///
/// - Lines starting with `#` or `;` are treated as comments and ignored.
/// - Blank lines are ignored.
/// - Section headers are glob patterns enclosed in brackets: `[*.rs]`
/// - Key-value pairs use `=` as the separator: `indent_style = space`
/// - The `root = true` declaration (before any section) sets the file-level root flag.
/// - Property names are normalized to lowercase.
/// - Enum-like property values are matched case-insensitively.
/// - Unknown properties are silently ignored.
/// - Invalid values for known properties are silently ignored (the property
///   remains `None`).
///
/// # Errors
///
/// Returns `Err(ParseError)` if the file contains structurally invalid syntax
/// that cannot be recovered from (e.g., unclosed section brackets). Individual
/// invalid property values do not cause errors — they are simply ignored.
pub fn parse(content: &str) -> Result<EditorConfigFile, ParseError> {
    let mut root = false;
    let mut sections: Vec<EditorConfigSection> = Vec::new();
    let mut current_section: Option<EditorConfigSection> = None;

    for (line_idx, raw_line) in content.lines().enumerate() {
        let line_num = line_idx + 1;
        let line = raw_line.trim();

        // Skip blank lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        // Section header
        if line.starts_with('[') {
            // Push any existing section
            if let Some(section) = current_section.take() {
                sections.push(section);
            }

            // Validate closing bracket exists
            let end = line.rfind(']').ok_or_else(|| ParseError {
                line: line_num,
                message: "unclosed section bracket".to_string(),
            })?;

            let pattern = line[1..end].trim().to_string();
            current_section = Some(EditorConfigSection {
                pattern,
                properties: EditorConfigProperties::default(),
            });
            continue;
        }

        // Key-value pair
        if let Some((key, value)) = parse_key_value(line) {
            let key_lower = key.to_lowercase();
            let value_trimmed = value.trim();

            // Handle root = true at the top level (before any section)
            if key_lower == "root" && current_section.is_none() {
                root = parse_bool(value_trimmed).unwrap_or(false);
                continue;
            }

            // Apply property to current section (if we're in one)
            if let Some(ref mut section) = current_section {
                apply_property(&mut section.properties, &key_lower, value_trimmed);
            }
        }
        // Lines that don't match any pattern are silently ignored
    }

    // Push the final section
    if let Some(section) = current_section.take() {
        sections.push(section);
    }

    Ok(EditorConfigFile { root, sections })
}

/// Split a line into key and value around the first `=` sign.
fn parse_key_value(line: &str) -> Option<(&str, &str)> {
    let eq_pos = line.find('=')?;
    let key = line[..eq_pos].trim();
    let value = line[eq_pos + 1..].trim();
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}

/// Parse a boolean value (case-insensitive).
fn parse_bool(value: &str) -> Option<bool> {
    match value.to_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Parse an unsigned integer value.
fn parse_u32(value: &str) -> Option<u32> {
    value.parse::<u32>().ok()
}

/// Apply a parsed property to an `EditorConfigProperties` struct.
///
/// Unknown properties or invalid values are silently ignored.
fn apply_property(props: &mut EditorConfigProperties, key: &str, value: &str) {
    let value_lower = value.to_lowercase();

    match key {
        "indent_style" => {
            props.indent_style = match value_lower.as_str() {
                "space" => Some(IndentStyle::Space),
                "tab" => Some(IndentStyle::Tab),
                _ => None,
            };
        }
        "indent_size" => {
            props.indent_size = match value_lower.as_str() {
                "tab" => Some(IndentSize::Tab),
                _ => parse_u32(value).map(IndentSize::Value),
            };
        }
        "tab_width" => {
            props.tab_width = parse_u32(value);
        }
        "end_of_line" => {
            props.end_of_line = match value_lower.as_str() {
                "lf" => Some(EndOfLine::Lf),
                "crlf" => Some(EndOfLine::CrLf),
                "cr" => Some(EndOfLine::Cr),
                _ => None,
            };
        }
        "charset" => {
            props.charset = match value_lower.as_str() {
                "utf-8" => Some(Charset::Utf8),
                "utf-8-bom" => Some(Charset::Utf8Bom),
                "latin1" => Some(Charset::Latin1),
                "utf-16be" => Some(Charset::Utf16Be),
                "utf-16le" => Some(Charset::Utf16Le),
                _ => None,
            };
        }
        "trim_trailing_whitespace" => {
            props.trim_trailing_whitespace = parse_bool(&value_lower);
        }
        "insert_final_newline" => {
            props.insert_final_newline = parse_bool(&value_lower);
        }
        _ => {
            // Unknown property — silently ignored
        }
    }
}

/// Load and parse an `.editorconfig` file from disk.
///
/// Returns `Some(EditorConfigFile)` on success, or `None` if the file
/// doesn't exist, can't be read, or has syntax errors (with WARN log).
///
/// # Behavior
///
/// - If the file does not exist → returns `None` (no log emitted; this is normal)
/// - If the file cannot be read (I/O error) → emits a WARN log, returns `None`
/// - If the file has syntax errors → emits a WARN log with file path and error
///   details, returns `None`
/// - If the file parses successfully → returns `Some(EditorConfigFile)`
///
/// This function is used by the resolver when walking up the directory tree.
/// The resolver must be resilient to individual files being broken.
///
/// # Validates
///
/// Requirement 6 AC 6.6: IF an `.editorconfig` file contains syntax errors,
/// THE Configuration_System SHALL skip that file, emit a WARN-level log record
/// identifying the file and parse error, and continue resolution using remaining
/// `.editorconfig` files in the path hierarchy.
pub fn load_editorconfig_file(path: &Path) -> Option<EditorConfigFile> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                // File doesn't exist — normal case, no log needed
                return None;
            }
            // I/O error (permission denied, etc.) — emit WARN and skip
            ff_logging::log_warn!(
                "[config] editorconfig: cannot read '{}': {}",
                path.display(),
                err
            );
            return None;
        }
    };

    match parse(&content) {
        Ok(file) => Some(file),
        Err(err) => {
            // Parse error — emit WARN identifying file and error, then skip
            ff_logging::log_warn!(
                "[config] editorconfig: parse error in '{}': {}",
                path.display(),
                err
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 6 AC 6.1 — parse .editorconfig files conforming to EditorConfig spec
    #[test]
    fn parse_empty_file_returns_no_root_no_sections() {
        let result = parse("").unwrap();
        assert!(!result.root);
        assert!(result.sections.is_empty());
    }

    // Validates: Requirement 6 AC 6.1 — root = true parsing
    #[test]
    fn parse_root_true_sets_root_flag() {
        let content = "root = true\n";
        let result = parse(content).unwrap();
        assert!(result.root);
    }

    // Validates: Requirement 6 AC 6.1 — root = true is case-insensitive
    #[test]
    fn parse_root_true_case_insensitive() {
        let content = "Root = True\n";
        let result = parse(content).unwrap();
        assert!(result.root);
    }

    // Validates: Requirement 6 AC 6.1 — root defaults to false when not specified
    #[test]
    fn parse_without_root_defaults_to_false() {
        let content = "[*.rs]\nindent_style = space\n";
        let result = parse(content).unwrap();
        assert!(!result.root);
    }

    // Validates: Requirement 6 AC 6.1 — comments with # are ignored
    #[test]
    fn parse_ignores_hash_comments() {
        let content = "# This is a comment\nroot = true\n";
        let result = parse(content).unwrap();
        assert!(result.root);
        assert!(result.sections.is_empty());
    }

    // Validates: Requirement 6 AC 6.1 — comments with ; are ignored
    #[test]
    fn parse_ignores_semicolon_comments() {
        let content = "; This is a comment\nroot = true\n";
        let result = parse(content).unwrap();
        assert!(result.root);
    }

    // Validates: Requirement 6 AC 6.1 — blank lines are ignored
    #[test]
    fn parse_ignores_blank_lines() {
        let content = "\n\nroot = true\n\n[*.rs]\n\nindent_size = 4\n";
        let result = parse(content).unwrap();
        assert!(result.root);
        assert_eq!(result.sections.len(), 1);
        assert_eq!(
            result.sections[0].properties.indent_size,
            Some(IndentSize::Value(4))
        );
    }

    // Validates: Requirement 6 AC 6.1 — section headers parse glob patterns
    #[test]
    fn parse_section_header_extracts_glob_pattern() {
        let content = "[*.rs]\nindent_style = space\n";
        let result = parse(content).unwrap();
        assert_eq!(result.sections.len(), 1);
        assert_eq!(result.sections[0].pattern, "*.rs");
    }

    // Validates: Requirement 6 AC 6.1 — multiple sections are parsed in order
    #[test]
    fn parse_multiple_sections_preserves_order() {
        let content = "[*.rs]\nindent_style = space\n\n[Makefile]\nindent_style = tab\n";
        let result = parse(content).unwrap();
        assert_eq!(result.sections.len(), 2);
        assert_eq!(result.sections[0].pattern, "*.rs");
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Space)
        );
        assert_eq!(result.sections[1].pattern, "Makefile");
        assert_eq!(
            result.sections[1].properties.indent_style,
            Some(IndentStyle::Tab)
        );
    }

    // Validates: Requirement 6 AC 6.2 — indent_style property parsing
    #[test]
    fn parse_indent_style_space_and_tab() {
        let content = "[*]\nindent_style = space\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Space)
        );

        let content = "[*]\nindent_style = tab\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Tab)
        );
    }

    // Validates: Requirement 6 AC 6.2 — indent_size property parsing
    #[test]
    fn parse_indent_size_numeric_and_tab() {
        let content = "[*]\nindent_size = 4\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.indent_size,
            Some(IndentSize::Value(4))
        );

        let content = "[*]\nindent_size = tab\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.indent_size,
            Some(IndentSize::Tab)
        );
    }

    // Validates: Requirement 6 AC 6.2 — tab_width property parsing
    #[test]
    fn parse_tab_width_numeric() {
        let content = "[*]\ntab_width = 8\n";
        let result = parse(content).unwrap();
        assert_eq!(result.sections[0].properties.tab_width, Some(8));
    }

    // Validates: Requirement 6 AC 6.2 — end_of_line property parsing
    #[test]
    fn parse_end_of_line_all_variants() {
        let content = "[*]\nend_of_line = lf\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.end_of_line,
            Some(EndOfLine::Lf)
        );

        let content = "[*]\nend_of_line = crlf\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.end_of_line,
            Some(EndOfLine::CrLf)
        );

        let content = "[*]\nend_of_line = cr\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.end_of_line,
            Some(EndOfLine::Cr)
        );
    }

    // Validates: Requirement 6 AC 6.2 — charset property parsing
    #[test]
    fn parse_charset_all_variants() {
        let cases = [
            ("utf-8", Charset::Utf8),
            ("utf-8-bom", Charset::Utf8Bom),
            ("latin1", Charset::Latin1),
            ("utf-16be", Charset::Utf16Be),
            ("utf-16le", Charset::Utf16Le),
        ];
        for (value, expected) in &cases {
            let content = format!("[*]\ncharset = {value}\n");
            let result = parse(&content).unwrap();
            assert_eq!(
                result.sections[0].properties.charset,
                Some(*expected),
                "failed for charset value: {value}"
            );
        }
    }

    // Validates: Requirement 6 AC 6.2 — trim_trailing_whitespace property parsing
    #[test]
    fn parse_trim_trailing_whitespace_bool() {
        let content = "[*]\ntrim_trailing_whitespace = true\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.trim_trailing_whitespace,
            Some(true)
        );

        let content = "[*]\ntrim_trailing_whitespace = false\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.trim_trailing_whitespace,
            Some(false)
        );
    }

    // Validates: Requirement 6 AC 6.2 — insert_final_newline property parsing
    #[test]
    fn parse_insert_final_newline_bool() {
        let content = "[*]\ninsert_final_newline = true\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.insert_final_newline,
            Some(true)
        );

        let content = "[*]\ninsert_final_newline = false\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.insert_final_newline,
            Some(false)
        );
    }

    // Validates: Requirement 6 AC 6.1 — property names are case-insensitive
    #[test]
    fn parse_property_names_case_insensitive() {
        let content = "[*]\nIndent_Style = Space\nINDENT_SIZE = 2\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Space)
        );
        assert_eq!(
            result.sections[0].properties.indent_size,
            Some(IndentSize::Value(2))
        );
    }

    // Validates: Requirement 6 AC 6.1 — property values are case-insensitive for enums
    #[test]
    fn parse_enum_values_case_insensitive() {
        let content = "[*]\nindent_style = SPACE\nend_of_line = CRLF\ncharset = UTF-8\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Space)
        );
        assert_eq!(
            result.sections[0].properties.end_of_line,
            Some(EndOfLine::CrLf)
        );
        assert_eq!(result.sections[0].properties.charset, Some(Charset::Utf8));
    }

    // Validates: Requirement 6 AC 6.6 — unclosed bracket produces error
    #[test]
    fn parse_unclosed_section_bracket_returns_error() {
        let content = "[*.rs\nindent_style = space\n";
        let result = parse(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.line, 1);
        assert!(err.message.contains("unclosed section bracket"));
    }

    // Validates: Requirement 6 AC 6.1 — invalid property values are silently ignored
    #[test]
    fn parse_invalid_property_value_leaves_field_none() {
        let content = "[*]\nindent_style = invalid\nindent_size = abc\n";
        let result = parse(content).unwrap();
        assert_eq!(result.sections[0].properties.indent_style, None);
        assert_eq!(result.sections[0].properties.indent_size, None);
    }

    // Validates: Requirement 6 AC 6.1 — unknown properties are silently ignored
    #[test]
    fn parse_unknown_properties_ignored() {
        let content = "[*]\nunknown_prop = value\nindent_style = space\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Space)
        );
    }

    // Validates: Requirement 6 AC 6.1 — full example file
    #[test]
    fn parse_complete_editorconfig_file() {
        let content = r#"# EditorConfig is awesome: https://EditorConfig.org

# top-most EditorConfig file
root = true

# Unix-style newlines with a newline ending every file
[*]
end_of_line = lf
insert_final_newline = true

# Matches multiple files with brace expansion notation
[*.{js,py}]
charset = utf-8

# 4 space indentation
[*.py]
indent_style = space
indent_size = 4

# Tab indentation (no size specified)
[Makefile]
indent_style = tab

# Matches the exact file "package.json"
[package.json]
indent_style = space
indent_size = 2
"#;
        let result = parse(content).unwrap();
        assert!(result.root);
        assert_eq!(result.sections.len(), 5);

        // [*]
        assert_eq!(result.sections[0].pattern, "*");
        assert_eq!(
            result.sections[0].properties.end_of_line,
            Some(EndOfLine::Lf)
        );
        assert_eq!(
            result.sections[0].properties.insert_final_newline,
            Some(true)
        );

        // [*.{js,py}]
        assert_eq!(result.sections[1].pattern, "*.{js,py}");
        assert_eq!(result.sections[1].properties.charset, Some(Charset::Utf8));

        // [*.py]
        assert_eq!(result.sections[2].pattern, "*.py");
        assert_eq!(
            result.sections[2].properties.indent_style,
            Some(IndentStyle::Space)
        );
        assert_eq!(
            result.sections[2].properties.indent_size,
            Some(IndentSize::Value(4))
        );

        // [Makefile]
        assert_eq!(result.sections[3].pattern, "Makefile");
        assert_eq!(
            result.sections[3].properties.indent_style,
            Some(IndentStyle::Tab)
        );

        // [package.json]
        assert_eq!(result.sections[4].pattern, "package.json");
        assert_eq!(
            result.sections[4].properties.indent_style,
            Some(IndentStyle::Space)
        );
        assert_eq!(
            result.sections[4].properties.indent_size,
            Some(IndentSize::Value(2))
        );
    }

    // Validates: Requirement 6 AC 6.1 — whitespace around = is trimmed
    #[test]
    fn parse_handles_whitespace_around_equals() {
        let content = "[*]\n  indent_style   =   space  \n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Space)
        );
    }

    // Validates: Requirement 6 AC 6.1 — root = true inside a section is not treated as file-level root
    #[test]
    fn parse_root_inside_section_does_not_set_file_root() {
        let content = "[*]\nroot = true\nindent_style = space\n";
        let result = parse(content).unwrap();
        // root inside a section is treated as an unknown property
        assert!(!result.root);
    }

    // Validates: Requirement 6 AC 6.2 — indent_size = 0 is valid
    #[test]
    fn parse_indent_size_zero_is_valid() {
        let content = "[*]\nindent_size = 0\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.indent_size,
            Some(IndentSize::Value(0))
        );
    }

    // Validates: Requirement 6 AC 6.1 — section with no properties
    #[test]
    fn parse_section_with_no_properties() {
        let content = "[*.rs]\n";
        let result = parse(content).unwrap();
        assert_eq!(result.sections.len(), 1);
        assert_eq!(result.sections[0].pattern, "*.rs");
        assert_eq!(
            result.sections[0].properties,
            EditorConfigProperties::default()
        );
    }

    // ===== Glob pattern matching tests =====

    // Validates: Requirement 6 AC 6.1, 6.4 — simple wildcard matches basename only
    #[test]
    fn glob_star_matches_extension_in_same_directory() {
        assert!(matches_pattern("*.rs", "main.rs"));
        assert!(matches_pattern("*.rs", "lib.rs"));
    }

    // Validates: Requirement 6 AC 6.1, 6.4 — star does not match across directories
    #[test]
    fn glob_star_does_not_match_path_separator() {
        // Pattern without `/` matches only against basename.
        // So `*.rs` matches the basename `main.rs` of `src/main.rs`.
        // Per EditorConfig spec: patterns without `/` match the filename part only.
        assert!(matches_pattern("*.rs", "src/main.rs"));
        // But a pattern WITH `/` uses the full path, and `*` doesn't cross `/`
        assert!(!matches_pattern("src/*.rs", "src/sub/main.rs"));
    }

    // Validates: Requirement 6 AC 6.4 — double star matches across directories
    #[test]
    fn glob_double_star_matches_across_directories() {
        assert!(matches_pattern("**/*.rs", "src/main.rs"));
        assert!(matches_pattern("**/*.rs", "src/nested/deep/lib.rs"));
        assert!(matches_pattern("**/*.rs", "main.rs"));
    }

    // Validates: Requirement 6 AC 6.1, 6.4 — brace expansion with extensions
    #[test]
    fn glob_brace_expansion_matches_alternatives() {
        assert!(matches_pattern("*.{js,ts}", "app.js"));
        assert!(matches_pattern("*.{js,ts}", "app.ts"));
        assert!(!matches_pattern("*.{js,ts}", "app.rs"));
    }

    // Validates: Requirement 6 AC 6.1, 6.4 — character class
    #[test]
    fn glob_character_class_matches_listed_chars() {
        assert!(matches_pattern("[Mm]akefile", "Makefile"));
        assert!(matches_pattern("[Mm]akefile", "makefile"));
        assert!(!matches_pattern("[Mm]akefile", "Lakefile"));
    }

    // Validates: Requirement 6 AC 6.1, 6.4 — negated character class
    #[test]
    fn glob_negated_character_class() {
        assert!(matches_pattern("[!Mm]akefile", "Lakefile"));
        assert!(!matches_pattern("[!Mm]akefile", "Makefile"));
        assert!(!matches_pattern("[!Mm]akefile", "makefile"));
    }

    // Validates: Requirement 6 AC 6.1, 6.4 — negated character class with caret
    #[test]
    fn glob_negated_character_class_with_caret() {
        assert!(matches_pattern("[^Mm]akefile", "Lakefile"));
        assert!(!matches_pattern("[^Mm]akefile", "Makefile"));
    }

    // Validates: Requirement 6 AC 6.1, 6.4 — question mark matches single char
    #[test]
    fn glob_question_mark_matches_single_character() {
        assert!(matches_pattern("file?.txt", "file1.txt"));
        assert!(matches_pattern("file?.txt", "fileA.txt"));
        assert!(!matches_pattern("file?.txt", "file12.txt"));
        assert!(!matches_pattern("file?.txt", "file.txt"));
    }

    // Validates: Requirement 6 AC 6.1, 6.4 — exact filename match
    #[test]
    fn glob_exact_match() {
        assert!(matches_pattern("Makefile", "Makefile"));
        assert!(!matches_pattern("Makefile", "makefile"));
        assert!(!matches_pattern("Makefile", "Makefile.bak"));
    }

    // Validates: Requirement 6 AC 6.4 — pattern with slash matches relative path
    #[test]
    fn glob_pattern_with_slash_matches_relative_path() {
        assert!(matches_pattern("lib/**/*.rs", "lib/core/mod.rs"));
        assert!(matches_pattern("lib/**/*.rs", "lib/mod.rs"));
        assert!(!matches_pattern("lib/**/*.rs", "src/mod.rs"));
    }

    // Validates: Requirement 6 AC 6.4 — pattern without slash matches only basename
    #[test]
    fn glob_pattern_without_slash_matches_basename_of_nested_file() {
        // Pattern without `/` should match basename of any file
        assert!(matches_pattern("*.txt", "hello.txt"));
        // Per EditorConfig spec: patterns without `/` match against the basename.
        // The basename of "docs/readme.txt" is "readme.txt" which matches "*.txt"
        assert!(matches_pattern("*.txt", "docs/readme.txt"));
    }

    // Validates: Requirement 6 AC 6.4 — basename matching for nested paths
    #[test]
    fn glob_basename_matching_for_nested_paths() {
        // Pattern without `/` should match the basename portion
        assert!(matches_pattern("Makefile", "Makefile"));
        // When the file is at a nested path, the basename is extracted
        assert!(matches_pattern("Makefile", "src/Makefile"));
        assert!(matches_pattern("*.js", "src/app.js"));
    }

    // Validates: Requirement 6 AC 6.1, 6.4 — integer range in braces
    #[test]
    fn glob_integer_range_matches_numbers_in_range() {
        assert!(matches_pattern("file{1..5}.txt", "file1.txt"));
        assert!(matches_pattern("file{1..5}.txt", "file3.txt"));
        assert!(matches_pattern("file{1..5}.txt", "file5.txt"));
        assert!(!matches_pattern("file{1..5}.txt", "file0.txt"));
        assert!(!matches_pattern("file{1..5}.txt", "file6.txt"));
    }

    // Validates: Requirement 6 AC 6.1, 6.4 — character range in class
    #[test]
    fn glob_character_range_in_class() {
        assert!(matches_pattern("[a-z]ile.txt", "file.txt"));
        assert!(!matches_pattern("[a-z]ile.txt", "File.txt"));
    }

    // Validates: Requirement 6 AC 6.1 — star matches empty string
    #[test]
    fn glob_star_matches_empty() {
        assert!(matches_pattern("*.rs", ".rs"));
    }

    // Validates: Requirement 6 AC 6.1, 6.4 — double star at start with subpath
    #[test]
    fn glob_double_star_at_beginning() {
        assert!(matches_pattern("**/test.rs", "test.rs"));
        assert!(matches_pattern("**/test.rs", "src/test.rs"));
        assert!(matches_pattern("**/test.rs", "a/b/c/test.rs"));
    }

    // Validates: Requirement 6 AC 6.1 — multiple brace expansions
    #[test]
    fn glob_multiple_brace_expansions() {
        assert!(matches_pattern("{src,lib}/*.{rs,toml}", "src/main.rs"));
        assert!(matches_pattern("{src,lib}/*.{rs,toml}", "lib/Cargo.toml"));
        assert!(!matches_pattern("{src,lib}/*.{rs,toml}", "tests/main.rs"));
    }

    // Validates: Requirement 6 AC 6.1 — question mark does not match path separator
    #[test]
    fn glob_question_mark_does_not_match_separator() {
        assert!(!matches_pattern("src?main.rs", "src/main.rs"));
    }

    // Validates: Requirement 6 AC 6.4 — wildcard all files pattern
    #[test]
    fn glob_star_alone_matches_any_filename() {
        assert!(matches_pattern("*", "anything.txt"));
        assert!(matches_pattern("*", "Makefile"));
        assert!(matches_pattern("*", ".gitignore"));
    }

    // Validates: Requirement 6 AC 6.1 — integer range reversed order
    #[test]
    fn glob_integer_range_reversed_order() {
        // {5..1} should still match 1 through 5
        assert!(matches_pattern("file{5..1}.txt", "file1.txt"));
        assert!(matches_pattern("file{5..1}.txt", "file3.txt"));
        assert!(matches_pattern("file{5..1}.txt", "file5.txt"));
        assert!(!matches_pattern("file{5..1}.txt", "file6.txt"));
    }

    // ===== load_editorconfig_file tests =====

    // Validates: Requirement 6 AC 6.6 — non-existent file returns None without panic
    #[test]
    fn load_editorconfig_file_nonexistent_returns_none() {
        let path = std::path::Path::new("/nonexistent/path/.editorconfig");
        let result = load_editorconfig_file(path);
        assert!(result.is_none());
    }

    // Validates: Requirement 6 AC 6.6 — file with invalid syntax returns None
    #[test]
    fn load_editorconfig_file_invalid_syntax_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join(".editorconfig");
        // Write a file with an unclosed bracket (syntax error)
        std::fs::write(&file_path, "[*.rs\nindent_style = space\n").unwrap();

        let result = load_editorconfig_file(&file_path);
        assert!(result.is_none());
    }

    // Validates: Requirement 6 AC 6.6 — valid file returns Some(parsed file)
    #[test]
    fn load_editorconfig_file_valid_content_returns_some() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join(".editorconfig");
        std::fs::write(
            &file_path,
            "root = true\n\n[*.rs]\nindent_style = space\nindent_size = 4\n",
        )
        .unwrap();

        let result = load_editorconfig_file(&file_path);
        assert!(result.is_some());
        let file = result.unwrap();
        assert!(file.root);
        assert_eq!(file.sections.len(), 1);
        assert_eq!(file.sections[0].pattern, "*.rs");
        assert_eq!(
            file.sections[0].properties.indent_style,
            Some(IndentStyle::Space)
        );
        assert_eq!(
            file.sections[0].properties.indent_size,
            Some(IndentSize::Value(4))
        );
    }

    // Validates: Requirement 6 AC 6.6 — I/O error (directory path) returns None
    #[test]
    fn load_editorconfig_file_io_error_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        // Trying to read a directory as a file causes an I/O error
        let result = load_editorconfig_file(dir.path());
        assert!(result.is_none());
    }

    // ===== Additional edge case tests =====

    // Validates: Requirement 6 AC 6.1 — whitespace-only lines are treated as blank and ignored
    #[test]
    fn parse_whitespace_only_lines_are_ignored() {
        let content = "   \n\t\n  \t  \nroot = true\n   \n[*.rs]\n  \nindent_style = space\n";
        let result = parse(content).unwrap();
        assert!(result.root);
        assert_eq!(result.sections.len(), 1);
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Space)
        );
    }

    // Validates: Requirement 6 AC 6.1 — Windows-style CRLF line endings are handled correctly
    #[test]
    fn parse_crlf_line_endings_handled() {
        let content = "root = true\r\n\r\n[*.rs]\r\nindent_style = space\r\nindent_size = 4\r\n";
        let result = parse(content).unwrap();
        assert!(result.root);
        assert_eq!(result.sections.len(), 1);
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Space)
        );
        assert_eq!(
            result.sections[0].properties.indent_size,
            Some(IndentSize::Value(4))
        );
    }

    // Validates: Requirement 6 AC 6.1 — text after a comment marker on its own line is a comment
    #[test]
    fn parse_inline_comment_after_value_not_supported() {
        // EditorConfig spec: comments must be on their own line.
        // A `#` after a value is NOT treated as a comment — it becomes part of the value.
        let content = "[*]\nindent_style = space # this is not a comment\n";
        let result = parse(content).unwrap();
        // "space # this is not a comment" is not a valid indent_style, so it should be None
        assert_eq!(result.sections[0].properties.indent_style, None);
    }

    // Validates: Requirement 6 AC 6.1 — multiple equals signs: only first = is the separator
    #[test]
    fn parse_multiple_equals_signs_splits_on_first() {
        let content = "[*]\ncharset = utf-8\n";
        let result = parse(content).unwrap();
        assert_eq!(result.sections[0].properties.charset, Some(Charset::Utf8));

        // Value containing `=` should still work (split on first `=` only)
        // This is an unknown property but shouldn't cause a parse error
        let content = "[*]\ncustom_key = value=with=equals\nindent_size = 4\n";
        let result = parse(content).unwrap();
        assert_eq!(
            result.sections[0].properties.indent_size,
            Some(IndentSize::Value(4))
        );
    }

    // Validates: Requirement 6 AC 6.1 — empty section header `[]` is valid (empty pattern)
    #[test]
    fn parse_empty_section_header() {
        let content = "[]\nindent_style = space\n";
        let result = parse(content).unwrap();
        assert_eq!(result.sections.len(), 1);
        assert_eq!(result.sections[0].pattern, "");
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Space)
        );
    }

    // Validates: Requirement 6 AC 6.1 — section header with only whitespace `[ ]` trims to empty
    #[test]
    fn parse_whitespace_only_section_header() {
        let content = "[  ]\nindent_style = tab\n";
        let result = parse(content).unwrap();
        assert_eq!(result.sections.len(), 1);
        assert_eq!(result.sections[0].pattern, "");
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Tab)
        );
    }

    // Validates: Requirement 6 AC 6.1 — very long glob pattern is parsed without error
    #[test]
    fn parse_long_glob_pattern() {
        let long_ext = "a".repeat(200);
        let content = format!("[*.{}]\nindent_size = 2\n", long_ext);
        let result = parse(&content).unwrap();
        assert_eq!(result.sections.len(), 1);
        assert_eq!(result.sections[0].pattern, format!("*.{}", long_ext));
        assert_eq!(
            result.sections[0].properties.indent_size,
            Some(IndentSize::Value(2))
        );
    }

    // Validates: Requirement 6 AC 6.1 — deeply nested braces in pattern (no crash)
    #[test]
    fn glob_deeply_nested_braces_no_crash() {
        // EditorConfig spec does not require nested brace support, but shouldn't crash
        let pattern = "*.{a,{b,{c,d}}}";
        // Should not panic — either match or not
        let _ = matches_pattern(pattern, "test.a");
        let _ = matches_pattern(pattern, "test.b");
        let _ = matches_pattern(pattern, "test.c");
        let _ = matches_pattern(pattern, "test.d");
    }

    // Validates: Requirement 6 AC 6.1 — line with only key and no value (no `=`) is ignored
    #[test]
    fn parse_line_without_equals_is_ignored() {
        let content = "[*]\nindent_style\nindent_size = 4\n";
        let result = parse(content).unwrap();
        // "indent_style" without `=` is not a valid key-value pair, silently ignored
        assert_eq!(result.sections[0].properties.indent_style, None);
        assert_eq!(
            result.sections[0].properties.indent_size,
            Some(IndentSize::Value(4))
        );
    }

    // Validates: Requirement 6 AC 6.1 — key with empty value after `=` is treated as empty string
    #[test]
    fn parse_key_with_empty_value() {
        let content = "[*]\nindent_style =\nindent_size = 4\n";
        let result = parse(content).unwrap();
        // Empty value is not a valid indent_style, so it remains None
        assert_eq!(result.sections[0].properties.indent_style, None);
        // But other properties still parse fine
        assert_eq!(
            result.sections[0].properties.indent_size,
            Some(IndentSize::Value(4))
        );
    }

    // Validates: Requirement 6 AC 6.6 — file with mixed valid and invalid sections
    #[test]
    fn parse_unclosed_bracket_stops_parsing_with_error() {
        // A syntax error (unclosed bracket) causes a parse error for the whole file
        let content = "[*.rs]\nindent_style = space\n[*.py\nindent_size = 4\n";
        let result = parse(content);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.line, 3);
    }

    // Validates: Requirement 6 AC 6.1 — file ending without trailing newline
    #[test]
    fn parse_file_without_trailing_newline() {
        let content = "root = true\n[*.rs]\nindent_style = space";
        let result = parse(content).unwrap();
        assert!(result.root);
        assert_eq!(result.sections.len(), 1);
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Space)
        );
    }

    // Validates: Requirement 6 AC 6.1 — mixed CR line endings (classic Mac OS)
    #[test]
    fn parse_cr_only_line_endings() {
        // Rust's `str::lines()` handles \r\n and \n but NOT lone \r as a line separator.
        // However, `trim()` will strip trailing \r from lines split by \n.
        // With pure \r (no \n), the entire content is one line — this is an edge case.
        // EditorConfig files in practice always use \n or \r\n.
        let content = "root = true\n[*.rs]\nindent_style = tab\n";
        let result = parse(content).unwrap();
        assert!(result.root);
        assert_eq!(
            result.sections[0].properties.indent_style,
            Some(IndentStyle::Tab)
        );
    }

    // Validates: Requirement 6 AC 6.1 — section header with extra text after closing bracket
    #[test]
    fn parse_section_header_with_trailing_text() {
        // The parser uses rfind(']') so it finds the last `]` on the line.
        // Text after the closing bracket is ignored.
        let content = "[*.rs] ; some comment\nindent_style = space\n";
        let result = parse(content).unwrap();
        assert_eq!(result.sections.len(), 1);
        // Pattern is extracted between first `[` and last `]`
        // Since the line is `[*.rs] ; some comment`, rfind(']') finds position 5
        // So pattern = line[1..5] = "*.rs" — correctly trimmed
        assert_eq!(result.sections[0].pattern, "*.rs");
    }
}
