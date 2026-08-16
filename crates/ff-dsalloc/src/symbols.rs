//! Symbol table and symbolic substitution engine.
//!
//! Implements scoped symbol lookup, system symbol population, and left-to-right
//! substitution with dot-terminator, double-ampersand, and substring support.

use std::collections::HashMap;

use crate::config::ResolverConfig;
use crate::diagnostic::{DiagnosticCode, LintDiagnostic};

/// A scoped collection of symbolic parameter definitions.
///
/// Supports hierarchical scoping: job-level → proc-level → step-level.
/// Lookup searches from innermost to outermost scope.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Stack of scopes (innermost last). Each scope is a name→value map.
    scopes: Vec<HashMap<String, String>>,
}

impl SymbolTable {
    /// Create a new symbol table with a single empty scope.
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Create a new symbol table pre-populated with system symbols.
    pub fn new_with_system_symbols(config: &ResolverConfig) -> Self {
        let mut table = Self::new();

        // System symbols
        let now = chrono::Local::now();
        table.define("SYSDATE", &now.format("%y%m%d").to_string());
        table.define("SYSDATE4", &now.format("%Y%m%d").to_string());
        table.define("SYSTIME", &now.format("%H%M%S").to_string());
        table.define("SYSUID", "USER");
        table.define("SYSJOBNAME", "NOJOB");
        table.define("SYSSTEP", "");

        // Load persistent user-defined symbols from config
        for (name, value) in &config.symbols {
            table.define(&name.to_uppercase(), value);
        }

        table
    }

    /// Push a new scope (e.g., entering a procedure).
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the innermost scope (e.g., leaving a procedure).
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Define a symbol in the current (innermost) scope.
    pub fn define(&mut self, name: &str, value: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_uppercase(), value.to_string());
        }
    }

    /// Look up a symbol value, searching from innermost to outermost scope.
    pub fn lookup(&self, name: &str) -> Option<&str> {
        let upper = name.to_uppercase();
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(&upper) {
                return Some(value.as_str());
            }
        }
        None
    }

    /// Returns true if the symbol is defined in any scope.
    pub fn contains(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Load persistent symbols from configuration map.
    pub fn load_from_config(&mut self, symbols: &HashMap<String, String>) {
        for (name, value) in symbols {
            self.define(&name.to_uppercase(), value);
        }
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of symbolic substitution.
#[derive(Debug, Clone)]
pub struct SubstitutionResult {
    /// The text after substitution.
    pub text: String,
    /// Diagnostics produced during substitution (e.g., unresolved symbols).
    pub diagnostics: Vec<LintDiagnostic>,
}

/// Perform symbolic substitution on a text string.
///
/// Single left-to-right pass replacing `&symbol` and `&symbol.` references
/// with values from the symbol table.
///
/// # Rules
/// - `&SYM` is replaced with the value of SYM
/// - `&SYM.REST` replaces SYM value and consumes the dot (result: value + REST)
/// - `&&` is treated as literal ampersand (or temp dataset prefix) — not substituted
/// - `&SYM(start,length)` extracts a substring of the symbol's value
/// - Unresolved symbols produce ERROR diagnostics
pub fn substitute_symbols(
    text: &str,
    table: &SymbolTable,
    line_number: usize,
) -> SubstitutionResult {
    let mut result = String::with_capacity(text.len());
    let mut diagnostics = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '&' {
            // Double ampersand — literal or temp dataset
            if i + 1 < chars.len() && chars[i + 1] == '&' {
                result.push('&');
                result.push('&');
                i += 2;
                continue;
            }

            // Extract symbol name
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && is_symbol_char(chars[end]) {
                end += 1;
            }

            if end == start {
                // Lone ampersand — keep as-is
                result.push('&');
                i += 1;
                continue;
            }

            let sym_name: String = chars[start..end].iter().collect();

            // Check for substring notation: &SYM(start,length)
            let (sub_start, sub_len) = if end < chars.len() && chars[end] == '(' {
                let paren_end = chars[end..].iter().position(|c| *c == ')');
                if let Some(pe) = paren_end {
                    let inner: String = chars[end + 1..end + pe].iter().collect();
                    let parts: Vec<&str> = inner.split(',').collect();
                    if parts.len() == 2 {
                        let s = parts[0].trim().parse::<usize>().ok();
                        let l = parts[1].trim().parse::<usize>().ok();
                        if let (Some(s_val), Some(l_val)) = (s, l) {
                            end = end + pe + 1; // skip past closing paren
                            (Some(s_val), Some(l_val))
                        } else {
                            (None, None)
                        }
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            // Look up symbol
            if let Some(value) = table.lookup(&sym_name) {
                let effective_value = if let (Some(s), Some(l)) = (sub_start, sub_len) {
                    // Substring notation (1-based index)
                    let start_idx = s.saturating_sub(1);
                    let end_idx = (start_idx + l).min(value.len());
                    value.get(start_idx..end_idx).unwrap_or("").to_string()
                } else {
                    value.to_string()
                };

                result.push_str(&effective_value);

                // Consume dot terminator if present
                if end < chars.len() && chars[end] == '.' {
                    end += 1;
                }
            } else {
                // Unresolved symbolic
                diagnostics.push(LintDiagnostic::new(
                    DiagnosticCode::UnresolvedSymbolic,
                    line_number,
                    (i, end),
                    format!("Unresolved symbolic: &{}", sym_name),
                ));
                // Keep original text
                result.push('&');
                result.push_str(&sym_name);
            }

            i = end;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    SubstitutionResult {
        text: result,
        diagnostics,
    }
}

/// Returns true if the character is valid in a symbolic parameter name.
fn is_symbol_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '@' || ch == '#' || ch == '$'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_symbol_substitution() {
        // Validates: Requirement 3 AC 1
        let mut table = SymbolTable::new();
        table.define("HLQ", "PROD");
        let result = substitute_symbols("&HLQ.DATA.FILE", &table, 1);
        assert_eq!(result.text, "PRODDATA.FILE");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn dot_terminator_consumed() {
        // Validates: Requirement 3 AC 6
        let mut table = SymbolTable::new();
        table.define("SYM", "VALUE");
        let result = substitute_symbols("&SYM.REST", &table, 1);
        assert_eq!(result.text, "VALUEREST");
    }

    #[test]
    fn double_ampersand_not_substituted() {
        // Validates: Requirement 3 AC 7
        let table = SymbolTable::new();
        let result = substitute_symbols("&&TEMPFILE", &table, 1);
        assert_eq!(result.text, "&&TEMPFILE");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn substring_notation() {
        // Validates: Requirement 3 AC 8
        let mut table = SymbolTable::new();
        table.define("LONGVAL", "ABCDEFGH");
        let result = substitute_symbols("&LONGVAL(1,3)", &table, 1);
        assert_eq!(result.text, "ABC");
    }

    #[test]
    fn unresolved_symbol_produces_diagnostic() {
        // Validates: Requirement 3 AC 5
        let table = SymbolTable::new();
        let result = substitute_symbols("DSN=&UNKNOWN.DATA", &table, 5);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            DiagnosticCode::UnresolvedSymbolic
        );
        assert!(result.diagnostics[0].message.contains("UNKNOWN"));
    }

    #[test]
    fn scoped_lookup_inner_overrides_outer() {
        // Validates: Requirement 3 AC 3, AC 4
        let mut table = SymbolTable::new();
        table.define("VAR", "OUTER");
        table.push_scope();
        table.define("VAR", "INNER");
        assert_eq!(table.lookup("VAR"), Some("INNER"));
        table.pop_scope();
        assert_eq!(table.lookup("VAR"), Some("OUTER"));
    }

    #[test]
    fn system_symbols_populated() {
        // Validates: Requirement 3 AC 2
        let config = ResolverConfig::default();
        let table = SymbolTable::new_with_system_symbols(&config);
        assert!(table.contains("SYSDATE"));
        assert!(table.contains("SYSDATE4"));
        assert!(table.contains("SYSTIME"));
        assert!(table.contains("SYSUID"));
        assert!(table.contains("SYSJOBNAME"));
    }

    #[test]
    fn config_symbols_loaded() {
        // Validates: Requirement 3 AC 10
        let mut config = ResolverConfig::default();
        config
            .symbols
            .insert("MYVAR".to_string(), "MYVAL".to_string());
        let table = SymbolTable::new_with_system_symbols(&config);
        assert_eq!(table.lookup("MYVAR"), Some("MYVAL"));
    }

    #[test]
    fn substitution_single_pass_no_recursion() {
        // Validates: Requirement 3 AC 9
        let mut table = SymbolTable::new();
        table.define("A", "&B");
        table.define("B", "FINAL");
        let result = substitute_symbols("&A", &table, 1);
        // Single pass: &A → "&B" (literal), not "FINAL"
        assert_eq!(result.text, "&B");
    }
}
