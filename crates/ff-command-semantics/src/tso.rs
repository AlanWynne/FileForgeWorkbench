//! TSO command routing and operand parsing (Requirement 9).
//!
//! Covers: ALLOCATE, FREE, DELETE, RENAME, LISTCAT, LISTDS, LISTALC,
//! SUBMIT, STATUS, EDIT extension, TSO operand parsing, SET PREFIX,
//! command continuation, ds:// URIs, namespace conflict resolution,
//! capability model, secret operand redaction, and audit events.

use std::collections::HashSet;

// === TSO Operand Parsing (Req 9.11) =========================================

/// A single parsed TSO operand -- either positional or keyword form.
///
/// Validates: Requirement 9.11
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TsoOperand {
    /// A positional operand (space-separated value in defined order).
    Positional(String),
    /// A keyword operand: `KEYWORD(value)` or `KEYWORD value`.
    Keyword { key: String, value: String },
}

/// Parses TSO-style operands from a command argument string.
///
/// Validates: Requirement 9.11
pub struct TsoOperandParser;

impl TsoOperandParser {
    /// Parse a slice of raw argument strings into TsoOperands.
    ///
    /// Keyword form `KEYWORD(value)` is detected by a `(` in the token.
    /// Keyword form `KEYWORD value` is detected when the next token is
    /// a plain value following an all-uppercase keyword.
    pub fn parse(args: &[&str]) -> Vec<TsoOperand> {
        let mut result = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let arg = args[i];
            // Keyword(value) form
            if let Some(paren) = arg.find('(') {
                if arg.ends_with(')') {
                    let key = arg[..paren].to_uppercase();
                    let value = arg[paren + 1..arg.len() - 1].to_string();
                    result.push(TsoOperand::Keyword { key, value });
                    i += 1;
                    continue;
                }
            }
            // Keyword value form: all-uppercase token followed by another token
            let is_keyword = arg.chars().all(|c| c.is_ascii_uppercase() || c == '_');
            if is_keyword && i + 1 < args.len() && !args[i + 1].starts_with('(') {
                let key = arg.to_uppercase();
                let value = args[i + 1].to_string();
                result.push(TsoOperand::Keyword { key, value });
                i += 2;
                continue;
            }
            result.push(TsoOperand::Positional(arg.to_string()));
            i += 1;
        }
        result
    }
}

// === Session Prefix (Req 9.12) ==============================================

/// Session-level dataset prefix state.
///
/// Validates: Requirement 9.12
#[derive(Debug, Clone, Default)]
pub struct SessionPrefix {
    prefix: Option<String>,
}

impl SessionPrefix {
    pub fn new() -> Self {
        Self { prefix: None }
    }

    /// Set the session prefix via `SET PREFIX dsn-prefix`.
    pub fn set(&mut self, prefix: &str) {
        if prefix.is_empty() {
            self.prefix = None;
        } else {
            self.prefix = Some(prefix.to_uppercase());
        }
    }

    /// Clear the session prefix.
    pub fn clear(&mut self) {
        self.prefix = None;
    }

    /// Get the current prefix, if any.
    pub fn get(&self) -> Option<&str> {
        self.prefix.as_deref()
    }

    /// Qualify a dataset name: if unqualified (no `.`) and prefix is set,
    /// prepend the prefix.
    pub fn qualify(&self, dsname: &str) -> String {
        match &self.prefix {
            Some(pfx) if !dsname.contains('.') => format!("{}.{}", pfx, dsname),
            _ => dsname.to_string(),
        }
    }
}

// === Command Continuation (Req 9.13) ========================================

/// Accumulates command lines that end with `\` until a complete command.
///
/// Validates: Requirement 9.13
#[derive(Debug, Default)]
pub struct CommandContinuation {
    buffer: Vec<String>,
}

impl CommandContinuation {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Submit a line. Returns `Some(full_command)` when the command is
    /// complete (no trailing `\`), or `None` if more lines are expected.
    pub fn submit(&mut self, line: &str) -> Option<String> {
        let trimmed = line.trim_end();
        if let Some(without_backslash) = trimmed.strip_suffix('\\') {
            self.buffer.push(without_backslash.trim_end().to_string());
            None
        } else {
            self.buffer.push(trimmed.to_string());
            let full = self.buffer.join(" ");
            self.buffer.clear();
            Some(full)
        }
    }

    /// Whether a continuation is in progress.
    pub fn is_pending(&self) -> bool {
        !self.buffer.is_empty()
    }
}

// === ds:// URI detection (Req 9.14) =========================================

/// Returns true if the argument is a `ds://` URI.
///
/// Validates: Requirement 9.14
pub fn is_ds_uri(arg: &str) -> bool {
    arg.to_lowercase().starts_with("ds://")
}

/// Strip the `ds://` prefix from a URI, returning the raw dataset path.
pub fn strip_ds_uri(arg: &str) -> &str {
    &arg[5..]
}

// === Namespace Conflict Resolution (Req 9.15) ================================

/// Priority level for a registered command.
///
/// Validates: Requirement 9.15
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommandPriority {
    /// Macro-defined command (lowest priority).
    Macro = 0,
    /// Plugin-defined command.
    Plugin = 1,
    /// Built-in command (highest priority).
    BuiltIn = 2,
}

/// A command registration entry with name, priority, and optional qualifier.
#[derive(Debug, Clone)]
pub struct CommandEntry {
    pub name: String,
    pub priority: CommandPriority,
    /// Qualified name for lower-priority access, e.g. `plugin:commandname`.
    pub qualified_name: Option<String>,
}

/// Resolve a command name from a set of registrations, applying priority order.
///
/// Returns the highest-priority entry, or `None` if no match.
///
/// Validates: Requirement 9.15
pub fn resolve_command<'a>(name: &str, entries: &'a [CommandEntry]) -> Option<&'a CommandEntry> {
    let upper = name.to_uppercase();
    entries
        .iter()
        .filter(|e| e.name.to_uppercase() == upper)
        .max_by_key(|e| e.priority)
}

// === Capability Model (Req 9.16) ============================================

/// A set of capabilities available in the current session context.
///
/// Validates: Requirement 9.16
#[derive(Debug, Clone, Default)]
pub struct CapabilitySet {
    capabilities: HashSet<String>,
}

impl CapabilitySet {
    pub fn new() -> Self {
        Self {
            capabilities: HashSet::new(),
        }
    }

    pub fn grant(&mut self, cap: &str) {
        self.capabilities.insert(cap.to_lowercase());
    }

    pub fn has(&self, cap: &str) -> bool {
        self.capabilities.contains(&cap.to_lowercase())
    }
}

/// Check that all required capabilities are present.
///
/// Returns `Ok(())` if all are present, or `Err(missing)` listing the first
/// missing capability.
///
/// Validates: Requirement 9.16
pub fn check_capabilities(required: &[&str], available: &CapabilitySet) -> Result<(), String> {
    for cap in required {
        if !available.has(cap) {
            return Err(format!("missing capability: {}", cap));
        }
    }
    Ok(())
}

// === Secret Operand Redaction (Req 9.17) =====================================

/// Redact secret operand values from a command string.
///
/// Replaces the value of any operand whose key is in `secret_keys` with `***`.
/// Handles both `KEY(value)` and `KEY value` forms.
///
/// Validates: Requirement 9.17
pub fn redact_secrets(command: &str, secret_keys: &[&str]) -> String {
    let mut result = command.to_string();
    for key in secret_keys {
        let upper_key = key.to_uppercase();
        // KEY(value) form
        let paren_pattern = format!("{}(", upper_key);
        if let Some(start) = result.to_uppercase().find(&paren_pattern) {
            let after_paren = start + paren_pattern.len();
            if let Some(end) = result[after_paren..].find(')') {
                let replacement = format!("{}(***)", upper_key);
                result = format!(
                    "{}{}{}",
                    &result[..start],
                    replacement,
                    &result[after_paren + end + 1..]
                );
                continue;
            }
        }
        // KEY value form (space-separated)
        let space_pattern = format!("{} ", upper_key);
        if let Some(start) = result.to_uppercase().find(&space_pattern) {
            let value_start = start + space_pattern.len();
            let value_end = result[value_start..]
                .find(' ')
                .map(|p| value_start + p)
                .unwrap_or(result.len());
            result = format!(
                "{}{} ***{}",
                &result[..start],
                upper_key,
                &result[value_end..]
            );
        }
    }
    result
}

// === Audit Events (Req 9.18) =================================================

/// Outcome of a command execution.
///
/// Validates: Requirement 9.18
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditOutcome {
    Success,
    Failure(String),
}

/// A structured audit event emitted for every command execution.
///
/// Validates: Requirement 9.18
#[derive(Debug, Clone)]
pub struct AuditEvent {
    /// Command name.
    pub command: String,
    /// Arguments with secrets redacted.
    pub args_redacted: String,
    /// Timestamp (milliseconds since Unix epoch, or 0 in tests).
    pub timestamp_ms: u64,
    /// User context identifier.
    pub user: String,
    /// Outcome of the execution.
    pub outcome: AuditOutcome,
}

impl AuditEvent {
    pub fn new(
        command: &str,
        args_redacted: &str,
        timestamp_ms: u64,
        user: &str,
        outcome: AuditOutcome,
    ) -> Self {
        Self {
            command: command.to_string(),
            args_redacted: args_redacted.to_string(),
            timestamp_ms,
            user: user.to_string(),
            outcome,
        }
    }
}

// === TSO Command Router (Req 9.1-9.10, 10.1-10.5) ==========================

/// Target for SEND command routing.
///
/// Validates: Requirement 10.3
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendTarget {
    /// Send to a specific user: `USER(userid)`.
    User(String),
    /// Send to all logged-on users.
    Logon,
    /// Send to the system broadcast queue.
    Broadcast,
    /// No target specified (default).
    Default,
}

/// Parse the SEND target from the args string.
///
/// Validates: Requirement 10.3
pub fn parse_send_target(args: &str) -> SendTarget {
    let upper = args.to_uppercase();
    if upper.contains("BROADCAST") {
        return SendTarget::Broadcast;
    }
    if upper.contains("LOGON") {
        return SendTarget::Logon;
    }
    // USER(userid) form
    if let Some(start) = upper.find("USER(") {
        let after = start + 5;
        if let Some(end) = upper[after..].find(')') {
            let userid = args[after..after + end].to_string();
            return SendTarget::User(userid);
        }
    }
    SendTarget::Default
}

/// Whether CANCEL should also purge job output.
///
/// Validates: Requirement 10.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelPurge {
    /// Cancel only; retain output.
    Cancel,
    /// Cancel and purge all output.
    CancelAndPurge,
}

/// Parse whether PURGE operand is present in CANCEL args.
///
/// Validates: Requirement 10.2
pub fn parse_cancel_purge(args: &str) -> CancelPurge {
    if args.to_uppercase().contains("PURGE") {
        CancelPurge::CancelAndPurge
    } else {
        CancelPurge::Cancel
    }
}

/// The routing target for a TSO command.
///
/// Validates: Requirement 9.1-9.10, 10.1-10.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TsoRoute {
    /// Route to dataset allocator (ALLOCATE, FREE, LISTALC).
    DatasetAllocator { command: String, args: String },
    /// Route to VFS/catalog layer (DELETE, RENAME, LISTCAT, LISTDS).
    VfsCatalog { command: String, args: String },
    /// Route to FFW-JES subsystem (SUBMIT, STATUS, OUTPUT, CANCEL).
    FfwJes { command: String, args: String },
    /// Route to file-operations pipeline (EDIT, PRINTDS).
    FileOperations { command: String, args: String },
    /// Route to messaging subsystem (SEND).
    Messaging { command: String, args: String },
    /// Route to session profile subsystem (PROFILE).
    SessionProfile { command: String, args: String },
    /// Unknown command -- not a TSO command.
    Unknown(String),
}

/// Routes a TSO command to the appropriate subsystem.
///
/// Validates: Requirement 9.1-9.10, 10.1-10.5
pub fn route_tso_command(name: &str, args: &str) -> TsoRoute {
    let upper = name.trim().to_uppercase();
    let args = args.trim().to_string();
    match upper.as_str() {
        "ALLOCATE" | "FREE" | "LISTALC" => TsoRoute::DatasetAllocator {
            command: upper,
            args,
        },
        "DELETE" | "RENAME" | "LISTCAT" | "LISTDS" => TsoRoute::VfsCatalog {
            command: upper,
            args,
        },
        "SUBMIT" | "STATUS" | "OUTPUT" | "CANCEL" => TsoRoute::FfwJes {
            command: upper,
            args,
        },
        "EDIT" | "PRINTDS" => TsoRoute::FileOperations {
            command: upper,
            args,
        },
        "SEND" => TsoRoute::Messaging {
            command: upper,
            args,
        },
        "PROFILE" => TsoRoute::SessionProfile {
            command: upper,
            args,
        },
        _ => TsoRoute::Unknown(upper),
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Req 9.1-9.7: dataset management routing ---

    #[test]
    fn allocate_routes_to_dataset_allocator() {
        // Validates: Requirement 9.1
        let route = route_tso_command("ALLOCATE", "DATASET(MY.DATA) TRACKS SPACE(5,1)");
        assert!(matches!(route, TsoRoute::DatasetAllocator { .. }));
        if let TsoRoute::DatasetAllocator { command, .. } = route {
            assert_eq!(command, "ALLOCATE");
        }
    }

    #[test]
    fn free_routes_to_dataset_allocator() {
        // Validates: Requirement 9.2
        let route = route_tso_command("FREE", "DATASET(MY.DATA)");
        assert!(matches!(route, TsoRoute::DatasetAllocator { .. }));
    }

    #[test]
    fn delete_routes_to_vfs_catalog() {
        // Validates: Requirement 9.3
        let route = route_tso_command("DELETE", "MY.DATA");
        assert!(matches!(route, TsoRoute::VfsCatalog { .. }));
        if let TsoRoute::VfsCatalog { command, .. } = route {
            assert_eq!(command, "DELETE");
        }
    }

    #[test]
    fn rename_routes_to_vfs_catalog() {
        // Validates: Requirement 9.4
        let route = route_tso_command("RENAME", "OLD.NAME NEW.NAME");
        assert!(matches!(route, TsoRoute::VfsCatalog { .. }));
        if let TsoRoute::VfsCatalog { command, args } = route {
            assert_eq!(command, "RENAME");
            assert_eq!(args, "OLD.NAME NEW.NAME");
        }
    }

    #[test]
    fn listcat_routes_to_vfs_catalog() {
        // Validates: Requirement 9.5
        let route = route_tso_command("LISTCAT", "MY.*");
        assert!(matches!(route, TsoRoute::VfsCatalog { .. }));
    }

    #[test]
    fn listds_routes_to_vfs_catalog() {
        // Validates: Requirement 9.6
        let route = route_tso_command("LISTDS", "MY.DATA MEMBERS");
        assert!(matches!(route, TsoRoute::VfsCatalog { .. }));
        if let TsoRoute::VfsCatalog { command, args } = route {
            assert_eq!(command, "LISTDS");
            assert_eq!(args, "MY.DATA MEMBERS");
        }
    }

    #[test]
    fn listalc_routes_to_dataset_allocator() {
        // Validates: Requirement 9.7
        let route = route_tso_command("LISTALC", "");
        assert!(matches!(route, TsoRoute::DatasetAllocator { .. }));
    }

    // --- Req 9.8-9.10: job/edit routing ---

    #[test]
    fn submit_routes_to_ffwjes() {
        // Validates: Requirement 9.8
        let route = route_tso_command("SUBMIT", "MY.JCL");
        assert!(matches!(route, TsoRoute::FfwJes { .. }));
        if let TsoRoute::FfwJes { command, args } = route {
            assert_eq!(command, "SUBMIT");
            assert_eq!(args, "MY.JCL");
        }
    }

    #[test]
    fn status_routes_to_ffwjes() {
        // Validates: Requirement 9.9
        let route = route_tso_command("STATUS", "");
        assert!(matches!(route, TsoRoute::FfwJes { .. }));
    }

    #[test]
    fn status_with_jobname_routes_to_ffwjes_with_args() {
        // Validates: Requirement 9.9
        let route = route_tso_command("STATUS", "MYJOB");
        if let TsoRoute::FfwJes { command, args } = route {
            assert_eq!(command, "STATUS");
            assert_eq!(args, "MYJOB");
        }
    }

    #[test]
    fn edit_routes_to_file_operations() {
        // Validates: Requirement 9.10
        let route = route_tso_command("EDIT", "MY.SOURCE");
        assert!(matches!(route, TsoRoute::FileOperations { .. }));
        if let TsoRoute::FileOperations { command, args } = route {
            assert_eq!(command, "EDIT");
            assert_eq!(args, "MY.SOURCE");
        }
    }

    #[test]
    fn unknown_command_returns_unknown_route() {
        // Validates: Requirement 9.10 (negative case)
        let route = route_tso_command("NOSUCHCMD", "");
        assert!(matches!(route, TsoRoute::Unknown(_)));
    }

    // --- Req 9.11: TSO operand parsing ---

    #[test]
    fn tso_operand_parser_positional() {
        // Validates: Requirement 9.11
        let operands = TsoOperandParser::parse(&["MY.DATA", "MEMBERS"]);
        assert_eq!(operands[0], TsoOperand::Positional("MY.DATA".to_string()));
        assert_eq!(operands[1], TsoOperand::Positional("MEMBERS".to_string()));
    }

    #[test]
    fn tso_operand_parser_keyword_paren_form() {
        // Validates: Requirement 9.11
        let operands = TsoOperandParser::parse(&["DATASET(MY.DATA)", "SPACE(5,1)"]);
        assert_eq!(
            operands[0],
            TsoOperand::Keyword {
                key: "DATASET".to_string(),
                value: "MY.DATA".to_string()
            }
        );
        assert_eq!(
            operands[1],
            TsoOperand::Keyword {
                key: "SPACE".to_string(),
                value: "5,1".to_string()
            }
        );
    }

    #[test]
    fn tso_operand_parser_keyword_space_form() {
        // Validates: Requirement 9.11
        let operands = TsoOperandParser::parse(&["UNIT", "SYSDA"]);
        assert_eq!(
            operands[0],
            TsoOperand::Keyword {
                key: "UNIT".to_string(),
                value: "SYSDA".to_string()
            }
        );
    }

    // --- Req 9.12: SET PREFIX ---

    #[test]
    fn session_prefix_set_and_qualify() {
        // Validates: Requirement 9.12
        let mut prefix = SessionPrefix::new();
        prefix.set("MYUSER");
        assert_eq!(prefix.get(), Some("MYUSER"));
        assert_eq!(prefix.qualify("DATA"), "MYUSER.DATA");
    }

    #[test]
    fn session_prefix_qualified_name_not_modified() {
        // Validates: Requirement 9.12 -- already-qualified names unchanged
        let mut prefix = SessionPrefix::new();
        prefix.set("MYUSER");
        assert_eq!(prefix.qualify("OTHER.DATA"), "OTHER.DATA");
    }

    #[test]
    fn session_prefix_clear_removes_prefix() {
        // Validates: Requirement 9.12
        let mut prefix = SessionPrefix::new();
        prefix.set("MYUSER");
        prefix.clear();
        assert!(prefix.get().is_none());
        assert_eq!(prefix.qualify("DATA"), "DATA");
    }

    // --- Req 9.13: command continuation ---

    #[test]
    fn continuation_accumulates_lines_with_backslash() {
        // Validates: Requirement 9.13
        let mut cont = CommandContinuation::new();
        assert!(cont.submit("ALLOCATE DATASET(MY.DATA) \\").is_none());
        assert!(cont.is_pending());
        let result = cont.submit("SPACE(5,1) TRACKS");
        assert_eq!(
            result,
            Some("ALLOCATE DATASET(MY.DATA) SPACE(5,1) TRACKS".to_string())
        );
        assert!(!cont.is_pending());
    }

    #[test]
    fn continuation_single_line_no_backslash_completes_immediately() {
        // Validates: Requirement 9.13
        let mut cont = CommandContinuation::new();
        let result = cont.submit("ALLOCATE DATASET(MY.DATA)");
        assert_eq!(result, Some("ALLOCATE DATASET(MY.DATA)".to_string()));
    }

    #[test]
    fn continuation_multiple_continuations() {
        // Validates: Requirement 9.13
        let mut cont = CommandContinuation::new();
        assert!(cont.submit("A \\").is_none());
        assert!(cont.submit("B \\").is_none());
        let result = cont.submit("C");
        assert_eq!(result, Some("A B C".to_string()));
    }

    // --- Req 9.14: ds:// URI ---

    #[test]
    fn ds_uri_detected() {
        // Validates: Requirement 9.14
        assert!(is_ds_uri("ds://MY.DATASET"));
        assert!(is_ds_uri("DS://MY.DATASET"));
    }

    #[test]
    fn non_ds_uri_not_detected() {
        // Validates: Requirement 9.14
        assert!(!is_ds_uri("MY.DATASET"));
        assert!(!is_ds_uri("vfs://local/path"));
    }

    #[test]
    fn ds_uri_strip_returns_path() {
        // Validates: Requirement 9.14
        assert_eq!(strip_ds_uri("ds://MY.DATASET"), "MY.DATASET");
    }

    // --- Req 9.15: namespace conflict resolution ---

    #[test]
    fn builtin_wins_over_plugin_and_macro() {
        // Validates: Requirement 9.15
        let entries = vec![
            CommandEntry {
                name: "FIND".to_string(),
                priority: CommandPriority::Macro,
                qualified_name: Some("macro:FIND".to_string()),
            },
            CommandEntry {
                name: "FIND".to_string(),
                priority: CommandPriority::Plugin,
                qualified_name: Some("plugin:FIND".to_string()),
            },
            CommandEntry {
                name: "FIND".to_string(),
                priority: CommandPriority::BuiltIn,
                qualified_name: None,
            },
        ];
        let resolved = resolve_command("FIND", &entries).unwrap();
        assert_eq!(resolved.priority, CommandPriority::BuiltIn);
    }

    #[test]
    fn plugin_wins_over_macro() {
        // Validates: Requirement 9.15
        let entries = vec![
            CommandEntry {
                name: "MYFIND".to_string(),
                priority: CommandPriority::Macro,
                qualified_name: Some("macro:MYFIND".to_string()),
            },
            CommandEntry {
                name: "MYFIND".to_string(),
                priority: CommandPriority::Plugin,
                qualified_name: Some("plugin:MYFIND".to_string()),
            },
        ];
        let resolved = resolve_command("MYFIND", &entries).unwrap();
        assert_eq!(resolved.priority, CommandPriority::Plugin);
    }

    #[test]
    fn resolve_command_case_insensitive() {
        // Validates: Requirement 9.15
        let entries = vec![CommandEntry {
            name: "FIND".to_string(),
            priority: CommandPriority::BuiltIn,
            qualified_name: None,
        }];
        assert!(resolve_command("find", &entries).is_some());
        assert!(resolve_command("Find", &entries).is_some());
    }

    #[test]
    fn resolve_command_unknown_returns_none() {
        // Validates: Requirement 9.15
        let entries: Vec<CommandEntry> = vec![];
        assert!(resolve_command("NOSUCH", &entries).is_none());
    }

    // --- Req 9.16: capability model ---

    #[test]
    fn capability_check_passes_when_all_present() {
        // Validates: Requirement 9.16
        let mut caps = CapabilitySet::new();
        caps.grant("dataset.write");
        caps.grant("jes.submit");
        assert!(check_capabilities(&["dataset.write", "jes.submit"], &caps).is_ok());
    }

    #[test]
    fn capability_check_fails_when_missing() {
        // Validates: Requirement 9.16
        let caps = CapabilitySet::new();
        let result = check_capabilities(&["jes.submit"], &caps);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("jes.submit"));
    }

    #[test]
    fn capability_check_empty_requirements_always_passes() {
        // Validates: Requirement 9.16
        let caps = CapabilitySet::new();
        assert!(check_capabilities(&[], &caps).is_ok());
    }

    // --- Req 9.17: secret operand redaction ---

    #[test]
    fn redact_secrets_keyword_paren_form() {
        // Validates: Requirement 9.17
        let cmd = "LOGON USER(ALICE) PASSWORD(SECRET123)";
        let redacted = redact_secrets(cmd, &["PASSWORD"]);
        assert!(redacted.contains("PASSWORD(***)"));
        assert!(!redacted.contains("SECRET123"));
        assert!(redacted.contains("USER(ALICE)"));
    }

    #[test]
    fn redact_secrets_keyword_space_form() {
        // Validates: Requirement 9.17
        let cmd = "LOGON USER ALICE PASSWORD SECRET123";
        let redacted = redact_secrets(cmd, &["PASSWORD"]);
        assert!(redacted.contains("PASSWORD ***"));
        assert!(!redacted.contains("SECRET123"));
    }

    #[test]
    fn redact_secrets_no_match_unchanged() {
        // Validates: Requirement 9.17
        let cmd = "ALLOCATE DATASET(MY.DATA)";
        let redacted = redact_secrets(cmd, &["PASSWORD"]);
        assert_eq!(redacted, cmd);
    }

    // --- Req 10.1: OUTPUT routing ---

    #[test]
    fn output_routes_to_ffwjes() {
        // Validates: Requirement 10.1
        let route = route_tso_command("OUTPUT", "MYJOB");
        assert!(matches!(route, TsoRoute::FfwJes { .. }));
        if let TsoRoute::FfwJes { command, args } = route {
            assert_eq!(command, "OUTPUT");
            assert_eq!(args, "MYJOB");
        }
    }

    #[test]
    fn output_with_options_routes_to_ffwjes() {
        // Validates: Requirement 10.1
        let route = route_tso_command("OUTPUT", "MYJOB PRINT");
        assert!(matches!(route, TsoRoute::FfwJes { .. }));
    }

    // --- Req 10.2: CANCEL routing and PURGE operand ---

    #[test]
    fn cancel_routes_to_ffwjes() {
        // Validates: Requirement 10.2
        let route = route_tso_command("CANCEL", "MYJOB");
        assert!(matches!(route, TsoRoute::FfwJes { .. }));
        if let TsoRoute::FfwJes { command, args } = route {
            assert_eq!(command, "CANCEL");
            assert_eq!(args, "MYJOB");
        }
    }

    #[test]
    fn cancel_without_purge_is_cancel_only() {
        // Validates: Requirement 10.2
        assert_eq!(parse_cancel_purge("MYJOB"), CancelPurge::Cancel);
    }

    #[test]
    fn cancel_with_purge_operand_is_cancel_and_purge() {
        // Validates: Requirement 10.2
        assert_eq!(
            parse_cancel_purge("MYJOB PURGE"),
            CancelPurge::CancelAndPurge
        );
    }

    #[test]
    fn cancel_purge_case_insensitive() {
        // Validates: Requirement 10.2
        assert_eq!(
            parse_cancel_purge("MYJOB purge"),
            CancelPurge::CancelAndPurge
        );
    }

    // --- Req 10.3: SEND routing and target variants ---

    #[test]
    fn send_routes_to_messaging() {
        // Validates: Requirement 10.3
        let route = route_tso_command("SEND", "'hello' USER(ALICE)");
        assert!(matches!(route, TsoRoute::Messaging { .. }));
        if let TsoRoute::Messaging { command, .. } = route {
            assert_eq!(command, "SEND");
        }
    }

    #[test]
    fn send_target_user_parsed() {
        // Validates: Requirement 10.3
        let target = parse_send_target("'hello' USER(ALICE)");
        assert_eq!(target, SendTarget::User("ALICE".to_string()));
    }

    #[test]
    fn send_target_logon_parsed() {
        // Validates: Requirement 10.3
        let target = parse_send_target("'hello' LOGON");
        assert_eq!(target, SendTarget::Logon);
    }

    #[test]
    fn send_target_broadcast_parsed() {
        // Validates: Requirement 10.3
        let target = parse_send_target("'hello' BROADCAST");
        assert_eq!(target, SendTarget::Broadcast);
    }

    #[test]
    fn send_target_default_when_no_target() {
        // Validates: Requirement 10.3
        let target = parse_send_target("'hello'");
        assert_eq!(target, SendTarget::Default);
    }

    // --- Req 10.4: PROFILE routing ---

    #[test]
    fn profile_routes_to_session_profile() {
        // Validates: Requirement 10.4
        let route = route_tso_command("PROFILE", "");
        assert!(matches!(route, TsoRoute::SessionProfile { .. }));
        if let TsoRoute::SessionProfile { command, .. } = route {
            assert_eq!(command, "PROFILE");
        }
    }

    #[test]
    fn profile_with_msgid_operand_routes_to_session_profile() {
        // Validates: Requirement 10.4
        let route = route_tso_command("PROFILE", "MSGID");
        assert!(matches!(route, TsoRoute::SessionProfile { .. }));
        if let TsoRoute::SessionProfile { args, .. } = route {
            assert_eq!(args, "MSGID");
        }
    }

    #[test]
    fn profile_with_intercom_operand_routes_correctly() {
        // Validates: Requirement 10.4
        let route = route_tso_command("PROFILE", "INTERCOM");
        assert!(matches!(route, TsoRoute::SessionProfile { .. }));
    }

    #[test]
    fn profile_with_nointercom_operand_routes_correctly() {
        // Validates: Requirement 10.4
        let route = route_tso_command("PROFILE", "NOINTERCOM");
        assert!(matches!(route, TsoRoute::SessionProfile { .. }));
    }

    #[test]
    fn profile_with_prefix_operand_routes_correctly() {
        // Validates: Requirement 10.4
        let route = route_tso_command("PROFILE", "PREFIX(MYUSER)");
        assert!(matches!(route, TsoRoute::SessionProfile { .. }));
    }

    // --- Req 10.5: PRINTDS routing ---

    #[test]
    fn printds_routes_to_file_operations() {
        // Validates: Requirement 10.5
        let route = route_tso_command("PRINTDS", "DATASET(MY.DATA)");
        assert!(matches!(route, TsoRoute::FileOperations { .. }));
        if let TsoRoute::FileOperations { command, args } = route {
            assert_eq!(command, "PRINTDS");
            assert_eq!(args, "DATASET(MY.DATA)");
        }
    }

    #[test]
    fn printds_with_options_routes_to_file_operations() {
        // Validates: Requirement 10.5
        let route = route_tso_command("PRINTDS", "DATASET(MY.DATA) SYSOUT(A)");
        assert!(matches!(route, TsoRoute::FileOperations { .. }));
    }

    #[test]
    fn printds_case_insensitive_command_name() {
        // Validates: Requirement 10.5
        let route = route_tso_command("printds", "DATASET(MY.DATA)");
        assert!(matches!(route, TsoRoute::FileOperations { .. }));
    }

    #[test]
    fn audit_event_contains_required_fields() {
        // Validates: Requirement 9.18
        let event = AuditEvent::new(
            "SUBMIT",
            "MY.JCL",
            1_700_000_000_000,
            "ALICE",
            AuditOutcome::Success,
        );
        assert_eq!(event.command, "SUBMIT");
        assert_eq!(event.args_redacted, "MY.JCL");
        assert_eq!(event.user, "ALICE");
        assert_eq!(event.outcome, AuditOutcome::Success);
        assert_eq!(event.timestamp_ms, 1_700_000_000_000);
    }

    #[test]
    fn audit_event_failure_records_reason() {
        // Validates: Requirement 9.18
        let event = AuditEvent::new(
            "DELETE",
            "MY.DATA",
            0,
            "BOB",
            AuditOutcome::Failure("dataset not found".to_string()),
        );
        assert!(matches!(event.outcome, AuditOutcome::Failure(_)));
        if let AuditOutcome::Failure(reason) = &event.outcome {
            assert!(reason.contains("not found"));
        }
    }

    #[test]
    fn audit_event_secrets_are_redacted_before_recording() {
        // Validates: Requirement 9.17, 9.18
        let raw_args = "USER(ALICE) PASSWORD(SECRET)";
        let redacted = redact_secrets(raw_args, &["PASSWORD"]);
        let event = AuditEvent::new("LOGON", &redacted, 0, "ALICE", AuditOutcome::Success);
        assert!(!event.args_redacted.contains("SECRET"));
        assert!(event.args_redacted.contains("PASSWORD(***)"));
    }
}
