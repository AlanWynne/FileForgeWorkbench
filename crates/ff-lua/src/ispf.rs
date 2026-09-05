//! ISPF host command environments and related services.
//!
//! Implements Requirement 11 AC 11.1-11.6:
//!   - ISREDIT host command environment (AC 11.1)
//!   - ISPEXEC host command environment (AC 11.2)
//!   - IMACRO initial macro execution (AC 11.3, 11.4)
//!   - LINENUM label/relative-ref resolver (AC 11.5)
//!   - CURSOR get/set API (AC 11.6)

use std::collections::HashMap;

// === IsreditService ==========================================================

/// Result of dispatching an ISREDIT service call.
#[derive(Debug, Clone, PartialEq)]
pub enum IsreditResult {
    /// Service executed successfully; optional string output.
    Ok(Option<String>),
    /// Service call was not recognised.
    UnknownService(String),
    /// Service call had invalid arguments.
    InvalidArgs(String),
}

/// Parsed ISREDIT service call.
#[derive(Debug, Clone, PartialEq)]
pub struct IsreditCall {
    /// Service name in uppercase (e.g. "CURSOR", "LINE", "LABEL").
    pub service: String,
    /// Raw operand string after the service name.
    pub operands: String,
}

impl IsreditCall {
    /// Parse an ISREDIT call string such as `"CURSOR = 5 10"` or `"LINE 3"`.
    /// Returns None if the string is empty or whitespace-only.
    pub fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return None;
        }
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let service = parts.next()?.to_uppercase();
        let operands = parts.next().unwrap_or("").trim().to_string();
        Some(Self { service, operands })
    }
}

/// ISREDIT host command environment.
///
/// Accepts ISPF Edit macro service call strings and dispatches them to
/// the corresponding editor operations via a simple in-process model.
/// In the full integration the cursor/line state is injected by the shell;
/// here we maintain it internally so the unit tests are self-contained.
///
/// Addresses: Requirement 11 AC 11.1
pub struct IsreditEnv {
    /// Current cursor position (1-based line, 1-based col).
    cursor: (usize, usize),
    /// Line labels: label -> 1-based line number.
    labels: HashMap<String, usize>,
}

impl IsreditEnv {
    /// Create a new ISREDIT environment with the given buffer size.
    pub fn new(total_lines: usize) -> Self {
        let _ = total_lines;
        Self {
            cursor: (1, 1),
            labels: HashMap::new(),
        }
    }

    /// Set a label on a line (used by tests and LABEL service).
    pub fn set_label(&mut self, label: &str, line: usize) {
        self.labels.insert(label.to_uppercase(), line);
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// Dispatch an ISREDIT service call string.
    ///
    /// Addresses: Requirement 11 AC 11.1
    pub fn dispatch(&mut self, call_str: &str) -> IsreditResult {
        let call = match IsreditCall::parse(call_str) {
            Some(c) => c,
            None => return IsreditResult::InvalidArgs("empty service call".to_string()),
        };

        match call.service.as_str() {
            "CURSOR" => self.service_cursor(&call.operands),
            "LABEL" => self.service_label(&call.operands),
            "LINE_BEFORE" | "LINE_AFTER" => IsreditResult::Ok(None),
            _ => IsreditResult::UnknownService(call.service),
        }
    }

    // CURSOR = <line> <col>  or  CURSOR = (<line>,<col>)
    fn service_cursor(&mut self, operands: &str) -> IsreditResult {
        let ops = operands.trim_start_matches('=').trim();
        let parts: Vec<&str> = ops.split_whitespace().collect();
        if parts.len() < 2 {
            return IsreditResult::InvalidArgs(format!(
                "CURSOR requires line and col, got: '{operands}'"
            ));
        }
        let line: usize = match parts[0].parse() {
            Ok(n) => n,
            Err(_) => return IsreditResult::InvalidArgs(format!("invalid line: '{}'", parts[0])),
        };
        let col: usize = match parts[1].parse() {
            Ok(n) => n,
            Err(_) => return IsreditResult::InvalidArgs(format!("invalid col: '{}'", parts[1])),
        };
        self.cursor = (line, col);
        IsreditResult::Ok(None)
    }

    // LABEL .name <line>
    fn service_label(&mut self, operands: &str) -> IsreditResult {
        let parts: Vec<&str> = operands.split_whitespace().collect();
        if parts.len() < 2 {
            return IsreditResult::InvalidArgs(format!(
                "LABEL requires name and line, got: '{operands}'"
            ));
        }
        let line: usize = match parts[1].parse() {
            Ok(n) => n,
            Err(_) => return IsreditResult::InvalidArgs(format!("invalid line: '{}'", parts[1])),
        };
        let key = parts[0].trim_start_matches('.').to_uppercase();
        self.labels.insert(key, line);
        IsreditResult::Ok(None)
    }
}

// === IspexecEnv ==============================================================

/// Result of dispatching an ISPEXEC service call.
#[derive(Debug, Clone, PartialEq)]
pub enum IspexecResult {
    /// Service executed; optional message output.
    Ok(Option<String>),
    /// Service not recognised.
    UnknownService(String),
    /// Invalid arguments.
    InvalidArgs(String),
}

/// ISPEXEC host command environment.
///
/// Routes ISPF dialog service calls to workbench services.
/// In this implementation the variable pool and message store are
/// maintained in-process; the shell layer injects real services.
///
/// Addresses: Requirement 11 AC 11.2
pub struct IspexecEnv {
    /// ISPF variable pool (name -> value).
    variables: HashMap<String, String>,
    /// Last message set via MSG service.
    last_message: Option<String>,
}

impl IspexecEnv {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            last_message: None,
        }
    }

    /// Retrieve the last message set via the MSG service.
    pub fn last_message(&self) -> Option<&str> {
        self.last_message.as_deref()
    }

    /// Get a variable from the pool.
    pub fn get_var(&self, name: &str) -> Option<&str> {
        self.variables.get(&name.to_uppercase()).map(|s| s.as_str())
    }

    /// Dispatch an ISPEXEC service call string.
    ///
    /// Addresses: Requirement 11 AC 11.2
    pub fn dispatch(&mut self, call_str: &str) -> IspexecResult {
        let trimmed = call_str.trim();
        if trimmed.is_empty() {
            return IspexecResult::InvalidArgs("empty service call".to_string());
        }
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let service = parts.next().unwrap_or("").to_uppercase();
        let operands = parts.next().unwrap_or("").trim().to_string();

        match service.as_str() {
            "VPUT" | "VGET" => self.service_vput_vget(&service, &operands),
            "SETMSG" => self.service_setmsg(&operands),
            "DISPLAY" => IspexecResult::Ok(None),
            "SELECT" => IspexecResult::Ok(None),
            _ => IspexecResult::UnknownService(service),
        }
    }

    fn service_vput_vget(&mut self, service: &str, operands: &str) -> IspexecResult {
        // VPUT varname value  /  VGET varname
        let parts: Vec<&str> = operands.splitn(2, char::is_whitespace).collect();
        if parts.is_empty() || parts[0].is_empty() {
            return IspexecResult::InvalidArgs(format!("{service} requires a variable name"));
        }
        let name = parts[0].to_uppercase();
        if service == "VPUT" {
            let value = parts.get(1).copied().unwrap_or("").to_string();
            self.variables.insert(name, value);
        }
        IspexecResult::Ok(None)
    }

    fn service_setmsg(&mut self, operands: &str) -> IspexecResult {
        self.last_message = Some(operands.trim().to_string());
        IspexecResult::Ok(Some(operands.trim().to_string()))
    }
}

impl Default for IspexecEnv {
    fn default() -> Self {
        Self::new()
    }
}

// === ImacroState =============================================================

/// Stores the IMACRO setting for an edit profile.
///
/// Addresses: Requirement 11 AC 11.3, 11.4
#[derive(Debug, Clone, Default)]
pub struct ImacroState {
    /// Name of the initial macro to run on edit session open.
    /// None or empty string means no initial macro.
    macro_name: Option<String>,
}

impl ImacroState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the IMACRO name. Empty string clears it.
    ///
    /// Addresses: Requirement 11 AC 11.4
    pub fn set(&mut self, name: &str) {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            self.macro_name = None;
        } else {
            self.macro_name = Some(trimmed.to_string());
        }
    }

    /// Get the IMACRO name, if set.
    pub fn get(&self) -> Option<&str> {
        self.macro_name.as_deref()
    }

    /// Returns true if an initial macro is configured.
    ///
    /// Addresses: Requirement 11 AC 11.3
    pub fn is_active(&self) -> bool {
        self.macro_name.is_some()
    }
}

// === LineNumResolver =========================================================

/// Resolves label or relative line references to absolute 1-based line numbers.
///
/// Addresses: Requirement 11 AC 11.5
pub struct LineNumResolver {
    /// Label map: uppercase label -> 1-based line number.
    labels: HashMap<String, usize>,
    /// Total lines in the buffer.
    total_lines: usize,
    /// Current cursor line (1-based).
    cursor_line: usize,
}

impl LineNumResolver {
    pub fn new(total_lines: usize, cursor_line: usize) -> Self {
        Self {
            labels: HashMap::new(),
            total_lines,
            cursor_line,
        }
    }

    /// Register a label. The leading dot is optional and stripped on storage.
    pub fn set_label(&mut self, label: &str, line: usize) {
        let key = label.trim_start_matches('.').to_uppercase();
        self.labels.insert(key, line);
    }

    /// Resolve a reference string to an absolute 1-based line number.
    ///
    /// Supported forms:
    /// - `.LABEL`  — named label
    /// - `+N`      — N lines after cursor
    /// - `-N`      — N lines before cursor
    /// - `N`       — absolute line number
    ///
    /// Returns None if the reference is invalid or out of range.
    ///
    /// Addresses: Requirement 11 AC 11.5
    pub fn resolve(&self, reference: &str) -> Option<usize> {
        let r = reference.trim();
        if r.is_empty() {
            return None;
        }

        if r.starts_with('.') {
            // Label reference
            let label = r.strip_prefix('.').unwrap_or(r).to_uppercase();
            return self.labels.get(&label).copied();
        }

        if let Some(offset_str) = r.strip_prefix('+') {
            let offset: usize = offset_str.parse().ok()?;
            let result = self.cursor_line.checked_add(offset)?;
            return if result <= self.total_lines {
                Some(result)
            } else {
                None
            };
        }

        if let Some(offset_str) = r.strip_prefix('-') {
            let offset: usize = offset_str.parse().ok()?;
            let result = self.cursor_line.checked_sub(offset)?;
            return if result >= 1 { Some(result) } else { None };
        }

        // Absolute line number
        let n: usize = r.parse().ok()?;
        if n >= 1 && n <= self.total_lines {
            Some(n)
        } else {
            None
        }
    }
}

// === CursorApi ===============================================================

/// Get/set cursor position API.
///
/// Addresses: Requirement 11 AC 11.6
#[derive(Debug, Clone, PartialEq)]
pub struct CursorPosition {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub col: usize,
}

impl CursorPosition {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

/// Manages cursor position for the CURSOR() Lua function.
///
/// Addresses: Requirement 11 AC 11.6
pub struct CursorApi {
    position: CursorPosition,
    total_lines: usize,
}

impl CursorApi {
    pub fn new(total_lines: usize) -> Self {
        Self {
            position: CursorPosition::new(1, 1),
            total_lines,
        }
    }

    /// Get the current cursor position.
    pub fn get(&self) -> &CursorPosition {
        &self.position
    }

    /// Set the cursor position. Returns false if out of range.
    ///
    /// Addresses: Requirement 11 AC 11.6
    pub fn set(&mut self, line: usize, col: usize) -> bool {
        if line < 1 || line > self.total_lines || col < 1 {
            return false;
        }
        self.position = CursorPosition::new(line, col);
        true
    }
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- IsreditCall::parse --------------------------------------------------

    // Validates: Requirement 11.1
    #[test]
    fn isredit_call_parse_extracts_service_and_operands() {
        let call = IsreditCall::parse("CURSOR = 5 10").unwrap();
        assert_eq!(call.service, "CURSOR");
        assert_eq!(call.operands, "= 5 10");
    }

    // Validates: Requirement 11.1
    #[test]
    fn isredit_call_parse_uppercases_service() {
        let call = IsreditCall::parse("cursor = 3 1").unwrap();
        assert_eq!(call.service, "CURSOR");
    }

    // Validates: Requirement 11.1
    #[test]
    fn isredit_call_parse_returns_none_for_empty() {
        assert!(IsreditCall::parse("").is_none());
        assert!(IsreditCall::parse("   ").is_none());
    }

    // --- IsreditEnv::dispatch ------------------------------------------------

    // Validates: Requirement 11.1
    #[test]
    fn isredit_cursor_service_sets_position() {
        let mut env = IsreditEnv::new(100);
        let result = env.dispatch("CURSOR = 5 10");
        assert_eq!(result, IsreditResult::Ok(None));
        assert_eq!(env.cursor(), (5, 10));
    }

    // Validates: Requirement 11.1
    #[test]
    fn isredit_cursor_service_invalid_args_returns_error() {
        let mut env = IsreditEnv::new(100);
        let result = env.dispatch("CURSOR = 5");
        assert!(matches!(result, IsreditResult::InvalidArgs(_)));
    }

    // Validates: Requirement 11.1
    #[test]
    fn isredit_unknown_service_returns_unknown() {
        let mut env = IsreditEnv::new(100);
        let result = env.dispatch("BOGUS operands");
        assert!(matches!(result, IsreditResult::UnknownService(_)));
    }

    // Validates: Requirement 11.1
    #[test]
    fn isredit_label_service_stores_label() {
        let mut env = IsreditEnv::new(100);
        env.dispatch("LABEL .TOP 1");
        assert_eq!(env.labels.get("TOP"), Some(&1usize));
    }

    // --- IspexecEnv::dispatch ------------------------------------------------

    // Validates: Requirement 11.2
    #[test]
    fn ispexec_setmsg_stores_message() {
        let mut env = IspexecEnv::new();
        env.dispatch("SETMSG ISRZ000");
        assert_eq!(env.last_message(), Some("ISRZ000"));
    }

    // Validates: Requirement 11.2
    #[test]
    fn ispexec_vput_stores_variable() {
        let mut env = IspexecEnv::new();
        env.dispatch("VPUT MYVAR hello");
        assert_eq!(env.get_var("MYVAR"), Some("hello"));
    }

    // Validates: Requirement 11.2
    #[test]
    fn ispexec_unknown_service_returns_unknown() {
        let mut env = IspexecEnv::new();
        let result = env.dispatch("TBOPEN MYTABLE");
        assert!(matches!(result, IspexecResult::UnknownService(_)));
    }

    // Validates: Requirement 11.2
    #[test]
    fn ispexec_display_returns_ok() {
        let mut env = IspexecEnv::new();
        let result = env.dispatch("DISPLAY PANEL(MYPANEL)");
        assert_eq!(result, IspexecResult::Ok(None));
    }

    // --- ImacroState ---------------------------------------------------------

    // Validates: Requirement 11.3
    #[test]
    fn imacro_is_active_when_name_set() {
        let mut state = ImacroState::new();
        state.set("MYMACRO");
        assert!(state.is_active());
        assert_eq!(state.get(), Some("MYMACRO"));
    }

    // Validates: Requirement 11.4
    #[test]
    fn imacro_blank_clears_setting() {
        let mut state = ImacroState::new();
        state.set("MYMACRO");
        state.set("");
        assert!(!state.is_active());
        assert_eq!(state.get(), None);
    }

    // Validates: Requirement 11.4
    #[test]
    fn imacro_default_is_inactive() {
        let state = ImacroState::new();
        assert!(!state.is_active());
    }

    // --- LineNumResolver -----------------------------------------------------

    // Validates: Requirement 11.5
    #[test]
    fn linenum_resolves_absolute_number() {
        let resolver = LineNumResolver::new(50, 10);
        assert_eq!(resolver.resolve("25"), Some(25));
    }

    // Validates: Requirement 11.5
    #[test]
    fn linenum_resolves_label_reference() {
        let mut resolver = LineNumResolver::new(50, 10);
        resolver.set_label(".TOP", 1);
        assert_eq!(resolver.resolve(".TOP"), Some(1));
    }

    // Validates: Requirement 11.5
    #[test]
    fn linenum_resolves_positive_relative_offset() {
        let resolver = LineNumResolver::new(50, 10);
        assert_eq!(resolver.resolve("+5"), Some(15));
    }

    // Validates: Requirement 11.5
    #[test]
    fn linenum_resolves_negative_relative_offset() {
        let resolver = LineNumResolver::new(50, 10);
        assert_eq!(resolver.resolve("-3"), Some(7));
    }

    // Validates: Requirement 11.5
    #[test]
    fn linenum_returns_none_for_out_of_range() {
        let resolver = LineNumResolver::new(50, 10);
        assert_eq!(resolver.resolve("99"), None);
        assert_eq!(resolver.resolve("0"), None);
    }

    // Validates: Requirement 11.5
    #[test]
    fn linenum_returns_none_for_unknown_label() {
        let resolver = LineNumResolver::new(50, 10);
        assert_eq!(resolver.resolve(".MISSING"), None);
    }

    // Validates: Requirement 11.5
    #[test]
    fn linenum_relative_offset_clamped_at_bounds() {
        let resolver = LineNumResolver::new(50, 2);
        assert_eq!(resolver.resolve("-5"), None);
        let resolver2 = LineNumResolver::new(50, 48);
        assert_eq!(resolver2.resolve("+5"), None);
    }

    // --- CursorApi -----------------------------------------------------------

    // Validates: Requirement 11.6
    #[test]
    fn cursor_api_get_returns_initial_position() {
        let api = CursorApi::new(100);
        assert_eq!(api.get(), &CursorPosition::new(1, 1));
    }

    // Validates: Requirement 11.6
    #[test]
    fn cursor_api_set_updates_position() {
        let mut api = CursorApi::new(100);
        assert!(api.set(5, 10));
        assert_eq!(api.get(), &CursorPosition::new(5, 10));
    }

    // Validates: Requirement 11.6
    #[test]
    fn cursor_api_set_rejects_out_of_range_line() {
        let mut api = CursorApi::new(10);
        assert!(!api.set(11, 1));
        assert_eq!(api.get(), &CursorPosition::new(1, 1));
    }

    // Validates: Requirement 11.6
    #[test]
    fn cursor_api_set_rejects_zero_line_or_col() {
        let mut api = CursorApi::new(10);
        assert!(!api.set(0, 1));
        assert!(!api.set(1, 0));
    }
}
