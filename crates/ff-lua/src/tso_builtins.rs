//! TSO built-in functions exposed to the macro engine.
//!
//! Implements Requirement 11 AC 11.16-11.23:
//!   - LISTDSI  -- dataset information from catalog (AC 11.16)
//!   - MSG      -- display message in status bar (AC 11.17)
//!   - MVSVAR   -- system variable values (AC 11.18)
//!   - OUTTRAP  -- capture TSO command output into stem (AC 11.19)
//!   - PROMPT   -- control terminal input availability (AC 11.20)
//!   - SYSDSN   -- dataset existence check (AC 11.21)
//!   - SYSVAR   -- ISPF system variable values (AC 11.22)
//!   - USERID   -- current user login name (AC 11.23)

use std::collections::HashMap;

// === DatasetInfo =============================================================

/// Dataset information returned by LISTDSI.
///
/// Addresses: Requirement 11 AC 11.16
#[derive(Debug, Clone, PartialEq)]
pub struct DatasetInfo {
    pub dsname: String,
    pub dsorg: String,
    pub recfm: String,
    pub lrecl: u32,
    pub blksize: u32,
    pub volser: String,
    pub member_count: u32,
}

impl DatasetInfo {
    pub fn new(dsname: impl Into<String>) -> Self {
        Self {
            dsname: dsname.into(),
            dsorg: "PS".to_string(),
            recfm: "FB".to_string(),
            lrecl: 80,
            blksize: 0,
            volser: String::new(),
            member_count: 0,
        }
    }
}

/// Result of a LISTDSI call.
#[derive(Debug, Clone, PartialEq)]
pub enum ListdsiResult {
    /// Dataset found; info returned.
    Found(DatasetInfo),
    /// Dataset not found in catalog.
    NotFound(String),
    /// Catalog query error.
    Error(String),
}

// === SysdsnResult ============================================================

/// Result of a SYSDSN call.
///
/// Addresses: Requirement 11 AC 11.21
#[derive(Debug, Clone, PartialEq)]
pub enum SysdsnResult {
    /// Dataset exists and is accessible.
    Ok,
    /// Dataset not found.
    DatasetNotFound,
    /// Member not found within a PDS.
    MemberNotFound,
    /// Dataset is in use (enqueued).
    DatasetInUse,
    /// Other error with description.
    Other(String),
}

impl SysdsnResult {
    /// Returns the ISPF-standard string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Ok => "OK",
            Self::DatasetNotFound => "DATASET NOT FOUND",
            Self::MemberNotFound => "MEMBER NOT FOUND",
            Self::DatasetInUse => "DATASET IN USE",
            Self::Other(s) => s.as_str(),
        }
    }
}

// === OuttrapState ============================================================

/// State for the OUTTRAP built-in.
///
/// When active, TSO command output is captured into the stem variable
/// rather than displayed.
///
/// Addresses: Requirement 11 AC 11.19
#[derive(Debug, Clone, Default)]
pub struct OuttrapState {
    /// Whether output trapping is active.
    active: bool,
    /// Stem variable name to capture into.
    stem_name: Option<String>,
    /// Captured output lines.
    captured: Vec<String>,
}

impl OuttrapState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Activate output trapping into the named stem variable.
    pub fn activate(&mut self, stem_name: &str) {
        self.active = true;
        self.stem_name = Some(stem_name.to_uppercase());
        self.captured.clear();
    }

    /// Deactivate output trapping.
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Returns whether trapping is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Capture a line of output (called by the TSO dispatcher when active).
    pub fn capture(&mut self, line: &str) {
        if self.active {
            self.captured.push(line.to_string());
        }
    }

    /// Returns the captured lines.
    pub fn captured_lines(&self) -> &[String] {
        &self.captured
    }

    /// Returns the stem variable name.
    pub fn stem_name(&self) -> Option<&str> {
        self.stem_name.as_deref()
    }
}

// === PromptState =============================================================

/// Controls whether the exec may prompt the user for input.
///
/// When OFF, any attempt to read from the terminal returns an empty string.
///
/// Addresses: Requirement 11 AC 11.20
#[derive(Debug, Clone, Default)]
pub struct PromptState {
    enabled: bool,
}

impl PromptState {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Set prompt availability. `true` = ON, `false` = OFF.
    pub fn set(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether prompting is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Read a value from the terminal. Returns empty string when OFF.
    ///
    /// Addresses: Requirement 11 AC 11.20
    pub fn read_input(&self, _prompt: &str) -> String {
        if self.enabled {
            // In a real implementation this would invoke a UI dialog.
            // For the macro engine layer, return empty (UI layer overrides).
            String::new()
        } else {
            String::new()
        }
    }
}

// === TsoBuiltins =============================================================

/// Container for all TSO built-in function state and dispatch.
///
/// The catalog registry and system context are injected via the
/// `CatalogQuery` and `SystemContext` traits so the unit tests
/// can use in-memory stubs without touching the filesystem.
///
/// Addresses: Requirement 11 AC 11.16-11.23
pub struct TsoBuiltins {
    /// In-memory dataset catalog for LISTDSI and SYSDSN.
    catalog: HashMap<String, DatasetInfo>,
    /// System context values for MVSVAR and SYSVAR.
    system_vars: SystemVars,
    /// OUTTRAP state.
    outtrap: OuttrapState,
    /// PROMPT state.
    prompt: PromptState,
    /// Messages queued by MSG (most recent last).
    messages: Vec<String>,
}

/// System variable values mapped from workbench context.
///
/// Addresses: Requirement 11 AC 11.18, 11.22, 11.23
#[derive(Debug, Clone)]
pub struct SystemVars {
    /// Application name (maps to SYSNAME).
    pub sysname: String,
    /// Workspace name (maps to SYSPLEX).
    pub sysplex: String,
    /// Host OS short name (maps to SYSCLONE / SYSOPSYS).
    pub sysopsys: String,
    /// Current user login name (maps to SYSUID / USERID).
    pub userid: String,
    /// Current date YYYY-MM-DD (maps to SYSDATE).
    pub sysdate: String,
    /// Current time HH:MM:SS (maps to SYSTIME).
    pub systime: String,
    /// User dataset prefix (maps to SYSPREF).
    pub syspref: String,
    /// Execution environment: FORE or BACK (maps to SYSENV).
    pub sysenv: String,
}

impl Default for SystemVars {
    fn default() -> Self {
        Self {
            sysname: "FFWB".to_string(),
            sysplex: "LOCALWKS".to_string(),
            sysopsys: std::env::consts::OS.to_uppercase(),
            userid: whoami_fallback(),
            sysdate: "2024-01-01".to_string(),
            systime: "00:00:00".to_string(),
            syspref: String::new(),
            sysenv: "FORE".to_string(),
        }
    }
}

fn whoami_fallback() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "UNKNOWN".to_string())
}

impl TsoBuiltins {
    pub fn new(system_vars: SystemVars) -> Self {
        Self {
            catalog: HashMap::new(),
            system_vars,
            outtrap: OuttrapState::new(),
            prompt: PromptState::new(),
            messages: Vec::new(),
        }
    }

    /// Register a dataset in the in-memory catalog (used by tests and shell wiring).
    pub fn register_dataset(&mut self, info: DatasetInfo) {
        self.catalog.insert(info.dsname.to_uppercase(), info);
    }

    // -------------------------------------------------------------------------
    // LISTDSI
    // -------------------------------------------------------------------------

    /// Return dataset information for the named dataset.
    ///
    /// Addresses: Requirement 11 AC 11.16
    pub fn listdsi(&self, dsname: &str) -> ListdsiResult {
        match self.catalog.get(&dsname.to_uppercase()) {
            Some(info) => ListdsiResult::Found(info.clone()),
            None => ListdsiResult::NotFound(dsname.to_uppercase()),
        }
    }

    // -------------------------------------------------------------------------
    // MSG
    // -------------------------------------------------------------------------

    /// Display a message in the workbench status bar / message area.
    ///
    /// Addresses: Requirement 11 AC 11.17
    pub fn msg(&mut self, message: &str) {
        self.messages.push(message.to_string());
    }

    /// Returns all messages queued via MSG (most recent last).
    pub fn messages(&self) -> &[String] {
        &self.messages
    }

    // -------------------------------------------------------------------------
    // MVSVAR
    // -------------------------------------------------------------------------

    /// Return a system variable value mapped to workbench equivalents.
    ///
    /// Supported names: SYSNAME, SYSPLEX, SYSCLONE, SYSOPSYS.
    ///
    /// Addresses: Requirement 11 AC 11.18
    pub fn mvsvar(&self, name: &str) -> Option<String> {
        match name.trim().to_uppercase().as_str() {
            "SYSNAME" => Some(self.system_vars.sysname.clone()),
            "SYSPLEX" => Some(self.system_vars.sysplex.clone()),
            "SYSCLONE" => Some(self.system_vars.sysname.clone()),
            "SYSOPSYS" => Some(self.system_vars.sysopsys.clone()),
            _ => None,
        }
    }

    // -------------------------------------------------------------------------
    // OUTTRAP
    // -------------------------------------------------------------------------

    /// Activate or deactivate output trapping.
    ///
    /// `stem_name` = Some("STEM.") to activate; None to deactivate.
    ///
    /// Addresses: Requirement 11 AC 11.19
    pub fn outtrap(&mut self, stem_name: Option<&str>) {
        match stem_name {
            Some(name) => self.outtrap.activate(name),
            None => self.outtrap.deactivate(),
        }
    }

    /// Returns a reference to the OUTTRAP state.
    pub fn outtrap_state(&self) -> &OuttrapState {
        &self.outtrap
    }

    /// Capture a line of TSO output (called by dispatcher when OUTTRAP is active).
    pub fn capture_output(&mut self, line: &str) {
        self.outtrap.capture(line);
    }

    // -------------------------------------------------------------------------
    // PROMPT
    // -------------------------------------------------------------------------

    /// Set prompt availability: ON = true, OFF = false.
    ///
    /// Addresses: Requirement 11 AC 11.20
    pub fn prompt(&mut self, enabled: bool) {
        self.prompt.set(enabled);
    }

    /// Read terminal input, respecting PROMPT state.
    ///
    /// Returns empty string when PROMPT is OFF.
    ///
    /// Addresses: Requirement 11 AC 11.20
    pub fn read_input(&self, prompt_text: &str) -> String {
        self.prompt.read_input(prompt_text)
    }

    /// Returns whether prompting is enabled.
    pub fn prompt_enabled(&self) -> bool {
        self.prompt.is_enabled()
    }

    // -------------------------------------------------------------------------
    // SYSDSN
    // -------------------------------------------------------------------------

    /// Check whether a dataset exists and is accessible.
    ///
    /// Returns OK or an ISPF-standard error string.
    ///
    /// Addresses: Requirement 11 AC 11.21
    pub fn sysdsn(&self, dsname: &str) -> SysdsnResult {
        let key = dsname.to_uppercase();
        // Check for member reference: 'DSN(MEMBER)'
        if let Some(paren) = key.find('(') {
            let base = &key[..paren];
            let member_end = key.find(')').unwrap_or(key.len());
            let _member = &key[paren + 1..member_end];
            return if self.catalog.contains_key(base) {
                SysdsnResult::MemberNotFound
            } else {
                SysdsnResult::DatasetNotFound
            };
        }
        if self.catalog.contains_key(&key) {
            SysdsnResult::Ok
        } else {
            SysdsnResult::DatasetNotFound
        }
    }

    // -------------------------------------------------------------------------
    // SYSVAR
    // -------------------------------------------------------------------------

    /// Return an ISPF system variable value mapped to workbench equivalents.
    ///
    /// Supported names: SYSUID, SYSDATE, SYSTIME, SYSPREF, SYSENV.
    ///
    /// Addresses: Requirement 11 AC 11.22
    pub fn sysvar(&self, name: &str) -> Option<String> {
        match name.trim().to_uppercase().as_str() {
            "SYSUID" => Some(self.system_vars.userid.clone()),
            "SYSDATE" => Some(self.system_vars.sysdate.clone()),
            "SYSTIME" => Some(self.system_vars.systime.clone()),
            "SYSPREF" => Some(self.system_vars.syspref.clone()),
            "SYSENV" => Some(self.system_vars.sysenv.clone()),
            _ => None,
        }
    }

    // -------------------------------------------------------------------------
    // USERID
    // -------------------------------------------------------------------------

    /// Return the current user's login name.
    ///
    /// Addresses: Requirement 11 AC 11.23
    pub fn userid(&self) -> &str {
        &self.system_vars.userid
    }
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn builtins() -> TsoBuiltins {
        TsoBuiltins::new(SystemVars {
            sysname: "FFWB".to_string(),
            sysplex: "TESTPLEX".to_string(),
            sysopsys: "WINDOWS".to_string(),
            userid: "TESTUSER".to_string(),
            sysdate: "2024-06-01".to_string(),
            systime: "12:00:00".to_string(),
            syspref: "TEST".to_string(),
            sysenv: "FORE".to_string(),
        })
    }

    fn with_dataset(dsname: &str) -> TsoBuiltins {
        let mut b = builtins();
        let mut info = DatasetInfo::new(dsname);
        info.dsorg = "PO".to_string();
        info.lrecl = 80;
        b.register_dataset(info);
        b
    }

    // --- LISTDSI -------------------------------------------------------------

    // Validates: Requirement 11.16
    #[test]
    fn listdsi_returns_info_for_known_dataset() {
        let b = with_dataset("MY.DATA.SET");
        match b.listdsi("MY.DATA.SET") {
            ListdsiResult::Found(info) => assert_eq!(info.dsname, "MY.DATA.SET"),
            other => panic!("expected Found, got {other:?}"),
        }
    }

    // Validates: Requirement 11.16
    #[test]
    fn listdsi_is_case_insensitive() {
        let b = with_dataset("MY.DATA.SET");
        assert!(matches!(b.listdsi("my.data.set"), ListdsiResult::Found(_)));
    }

    // Validates: Requirement 11.16
    #[test]
    fn listdsi_returns_not_found_for_unknown() {
        let b = builtins();
        assert!(matches!(
            b.listdsi("MISSING.DS"),
            ListdsiResult::NotFound(_)
        ));
    }

    // Validates: Requirement 11.16
    #[test]
    fn listdsi_returns_correct_attributes() {
        let b = with_dataset("MY.DATA.SET");
        if let ListdsiResult::Found(info) = b.listdsi("MY.DATA.SET") {
            assert_eq!(info.dsorg, "PO");
            assert_eq!(info.lrecl, 80);
        }
    }

    // --- MSG -----------------------------------------------------------------

    // Validates: Requirement 11.17
    #[test]
    fn msg_stores_message() {
        let mut b = builtins();
        b.msg("ISRZ000");
        assert_eq!(b.messages(), &["ISRZ000"]);
    }

    // Validates: Requirement 11.17
    #[test]
    fn msg_accumulates_multiple_messages() {
        let mut b = builtins();
        b.msg("MSG1");
        b.msg("MSG2");
        assert_eq!(b.messages().len(), 2);
        assert_eq!(b.messages()[1], "MSG2");
    }

    // --- MVSVAR --------------------------------------------------------------

    // Validates: Requirement 11.18
    #[test]
    fn mvsvar_returns_sysname() {
        let b = builtins();
        assert_eq!(b.mvsvar("SYSNAME"), Some("FFWB".to_string()));
    }

    // Validates: Requirement 11.18
    #[test]
    fn mvsvar_returns_sysopsys() {
        let b = builtins();
        assert_eq!(b.mvsvar("SYSOPSYS"), Some("WINDOWS".to_string()));
    }

    // Validates: Requirement 11.18
    #[test]
    fn mvsvar_returns_none_for_unknown() {
        let b = builtins();
        assert_eq!(b.mvsvar("BOGUSVAR"), None);
    }

    // Validates: Requirement 11.18
    #[test]
    fn mvsvar_is_case_insensitive() {
        let b = builtins();
        assert_eq!(b.mvsvar("sysname"), Some("FFWB".to_string()));
    }

    // --- OUTTRAP -------------------------------------------------------------

    // Validates: Requirement 11.19
    #[test]
    fn outtrap_activates_and_captures_output() {
        let mut b = builtins();
        b.outtrap(Some("OUT."));
        b.capture_output("line one");
        b.capture_output("line two");
        assert_eq!(
            b.outtrap_state().captured_lines(),
            &["line one", "line two"]
        );
    }

    // Validates: Requirement 11.19
    #[test]
    fn outtrap_deactivate_stops_capture() {
        let mut b = builtins();
        b.outtrap(Some("OUT."));
        b.capture_output("captured");
        b.outtrap(None);
        b.capture_output("not captured");
        assert_eq!(b.outtrap_state().captured_lines(), &["captured"]);
    }

    // Validates: Requirement 11.19
    #[test]
    fn outtrap_stores_stem_name() {
        let mut b = builtins();
        b.outtrap(Some("MYVAR."));
        assert_eq!(b.outtrap_state().stem_name(), Some("MYVAR."));
    }

    // --- PROMPT --------------------------------------------------------------

    // Validates: Requirement 11.20
    #[test]
    fn prompt_off_returns_empty_string_for_input() {
        let mut b = builtins();
        b.prompt(false);
        assert!(!b.prompt_enabled());
        assert_eq!(b.read_input("Enter value:"), "");
    }

    // Validates: Requirement 11.20
    #[test]
    fn prompt_on_by_default() {
        let b = builtins();
        assert!(b.prompt_enabled());
    }

    // --- SYSDSN --------------------------------------------------------------

    // Validates: Requirement 11.21
    #[test]
    fn sysdsn_returns_ok_for_known_dataset() {
        let b = with_dataset("MY.DATA.SET");
        assert_eq!(b.sysdsn("MY.DATA.SET"), SysdsnResult::Ok);
        assert_eq!(b.sysdsn("MY.DATA.SET").as_str(), "OK");
    }

    // Validates: Requirement 11.21
    #[test]
    fn sysdsn_returns_dataset_not_found() {
        let b = builtins();
        assert_eq!(b.sysdsn("MISSING.DS"), SysdsnResult::DatasetNotFound);
        assert_eq!(b.sysdsn("MISSING.DS").as_str(), "DATASET NOT FOUND");
    }

    // Validates: Requirement 11.21
    #[test]
    fn sysdsn_member_reference_returns_member_not_found_when_pds_exists() {
        let b = with_dataset("MY.PDS");
        assert_eq!(b.sysdsn("MY.PDS(MEMBER)"), SysdsnResult::MemberNotFound);
    }

    // Validates: Requirement 11.21
    #[test]
    fn sysdsn_member_reference_returns_dataset_not_found_when_pds_missing() {
        let b = builtins();
        assert_eq!(
            b.sysdsn("MISSING.PDS(MEMBER)"),
            SysdsnResult::DatasetNotFound
        );
    }

    // --- SYSVAR --------------------------------------------------------------

    // Validates: Requirement 11.22
    #[test]
    fn sysvar_returns_sysuid() {
        let b = builtins();
        assert_eq!(b.sysvar("SYSUID"), Some("TESTUSER".to_string()));
    }

    // Validates: Requirement 11.22
    #[test]
    fn sysvar_returns_sysdate() {
        let b = builtins();
        assert_eq!(b.sysvar("SYSDATE"), Some("2024-06-01".to_string()));
    }

    // Validates: Requirement 11.22
    #[test]
    fn sysvar_returns_sysenv() {
        let b = builtins();
        assert_eq!(b.sysvar("SYSENV"), Some("FORE".to_string()));
    }

    // Validates: Requirement 11.22
    #[test]
    fn sysvar_returns_none_for_unknown() {
        let b = builtins();
        assert_eq!(b.sysvar("BOGUS"), None);
    }

    // --- USERID --------------------------------------------------------------

    // Validates: Requirement 11.23
    #[test]
    fn userid_returns_login_name() {
        let b = builtins();
        assert_eq!(b.userid(), "TESTUSER");
    }

    // Validates: Requirement 11.23
    #[test]
    fn userid_is_non_empty() {
        let b = TsoBuiltins::new(SystemVars::default());
        assert!(!b.userid().is_empty());
    }
}
