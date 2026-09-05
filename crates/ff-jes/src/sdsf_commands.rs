//! SDSF extended commands: FIND, LOCATE, scroll, SET, WHO, QUERY AUTH.
//!
//! Validates: Requirement 17.3-17.11, 17.15-17.17

use crate::model::Job;
use crate::sdsf_panel::ScrollAmount;

// === FIND State =============================================================

/// Case sensitivity mode for FIND.
///
/// Validates: Requirement 17.15
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FindCase {
    /// Case-insensitive (default).
    #[default]
    Insensitive,
    /// Case-sensitive (FIND C string).
    Sensitive,
}

/// Active FIND state for a panel.
///
/// Validates: Requirement 17.3, 17.15, 17.16
#[derive(Debug, Clone, Default)]
pub struct FindState {
    pub pattern: String,
    pub case_mode: FindCase,
    /// Index into the visible row list of the current match (-1 = no match).
    pub current_match: Option<usize>,
}

impl FindState {
    /// Sets a new search pattern. Resets current match position.
    pub fn set_pattern(&mut self, pattern: &str, case_mode: FindCase) {
        self.pattern = pattern.to_string();
        self.case_mode = case_mode;
        self.current_match = None;
    }

    /// Returns true if the job name matches the current pattern.
    pub fn matches_job(&self, job: &Job) -> bool {
        if self.pattern.is_empty() {
            return false;
        }
        match self.case_mode {
            FindCase::Insensitive => job
                .name
                .to_uppercase()
                .contains(&self.pattern.to_uppercase()),
            FindCase::Sensitive => job.name.contains(&self.pattern),
        }
    }

    /// Finds the first matching row index in the given job list.
    ///
    /// Validates: Requirement 17.3
    pub fn find_first(&mut self, jobs: &[Job]) -> Option<usize> {
        let idx = jobs.iter().position(|j| self.matches_job(j));
        self.current_match = idx;
        idx
    }

    /// Advances to the next match after the current position.
    ///
    /// Validates: Requirement 17.3
    pub fn find_next(&mut self, jobs: &[Job]) -> Option<usize> {
        let start = self.current_match.map(|i| i + 1).unwrap_or(0);
        let idx = jobs[start..]
            .iter()
            .position(|j| self.matches_job(j))
            .map(|i| i + start);
        self.current_match = idx;
        idx
    }

    /// Moves to the previous match before the current position.
    ///
    /// Validates: Requirement 17.3
    pub fn find_prev(&mut self, jobs: &[Job]) -> Option<usize> {
        let end = self.current_match.unwrap_or(jobs.len());
        let idx = jobs[..end].iter().rposition(|j| self.matches_job(j));
        self.current_match = idx;
        idx
    }

    /// Returns true if no match was found (for "NOT FOUND" message).
    ///
    /// Validates: Requirement 17.16
    pub fn is_no_match(&self) -> bool {
        self.current_match.is_none() && !self.pattern.is_empty()
    }
}

// === LOCATE =================================================================

/// Result of a LOCATE command.
///
/// Validates: Requirement 17.4, 17.16
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocateResult {
    /// Exact or prefix match found at this row index.
    Found(usize),
    /// No match; nearest alphabetic position returned.
    Nearest(usize),
    /// List is empty.
    Empty,
}

/// Executes LOCATE jobname on a sorted job list.
///
/// Validates: Requirement 17.4
pub fn locate(jobs: &[Job], target: &str) -> LocateResult {
    if jobs.is_empty() {
        return LocateResult::Empty;
    }
    let target_up = target.to_uppercase();
    // Exact prefix match first
    if let Some(idx) = jobs
        .iter()
        .position(|j| j.name.to_uppercase().starts_with(&target_up))
    {
        return LocateResult::Found(idx);
    }
    // Nearest alphabetic position
    let idx = jobs.partition_point(|j| j.name.to_uppercase() < target_up);
    LocateResult::Nearest(idx.min(jobs.len().saturating_sub(1)))
}

// === Scroll Commands ========================================================

/// Direction for a scroll command.
///
/// Validates: Requirement 17.5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDir {
    Up,
    Down,
    Left,
    Right,
}

impl ScrollDir {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "UP" => Some(Self::Up),
            "DOWN" => Some(Self::Down),
            "LEFT" => Some(Self::Left),
            "RIGHT" => Some(Self::Right),
            _ => None,
        }
    }
}

/// A parsed scroll command with direction and amount.
///
/// Validates: Requirement 17.5
#[derive(Debug, Clone)]
pub struct ScrollCommand {
    pub dir: ScrollDir,
    pub amount: ScrollAmount,
}

impl ScrollCommand {
    /// Parses "UP [amount]", "DOWN [amount]", etc.
    /// Amount defaults to the current SCROLL field value if omitted.
    pub fn parse(input: &str, default_amount: &ScrollAmount) -> Option<Self> {
        let mut parts = input.split_whitespace();
        let dir = ScrollDir::parse(parts.next()?)?;
        let amount = parts
            .next()
            .and_then(ScrollAmount::parse)
            .unwrap_or_else(|| default_amount.clone());
        Some(Self { dir, amount })
    }

    /// Computes the new first_visible_row given current state.
    ///
    /// Validates: Requirement 17.5
    pub fn apply_vertical(&self, first_row: usize, page_size: usize, total_rows: usize) -> usize {
        if total_rows == 0 {
            return 0;
        }
        let delta = match &self.amount {
            ScrollAmount::Lines(n) => *n,
            ScrollAmount::Page => page_size,
            ScrollAmount::Half => page_size / 2,
            ScrollAmount::Max => total_rows,
            ScrollAmount::Csr => 1,
        };
        match self.dir {
            ScrollDir::Up => first_row.saturating_sub(delta),
            ScrollDir::Down => {
                let max_first = total_rows.saturating_sub(page_size);
                (first_row + delta).min(max_first)
            }
            _ => first_row, // LEFT/RIGHT handled by caller
        }
    }
}

// === SET Commands ===========================================================

/// Persistent SET settings for the SDSF panel.
///
/// Validates: Requirement 17.6-17.11
#[derive(Debug, Clone)]
pub struct SdsfSetSettings {
    /// Whether SET ACTION display is active.
    pub action_display: bool,
    /// Default panel opened by MENU command.
    pub main_panel: String,
    /// Whether row numbers are shown in NP area.
    pub rownum_on: bool,
}

impl Default for SdsfSetSettings {
    fn default() -> Self {
        Self {
            action_display: false,
            main_panel: "I".to_string(),
            rownum_on: false,
        }
    }
}

impl SdsfSetSettings {
    /// Applies SET ACTION (toggle action display).
    ///
    /// Validates: Requirement 17.6
    pub fn set_action(&mut self, on: bool) {
        self.action_display = on;
    }

    /// Applies SET MAIN [panel-name].
    ///
    /// Validates: Requirement 17.7
    pub fn set_main(&mut self, panel: &str) {
        self.main_panel = panel.to_uppercase();
    }

    /// Applies SET ROWNUM ON/OFF.
    ///
    /// Validates: Requirement 17.8
    pub fn set_rownum(&mut self, on: bool) {
        self.rownum_on = on;
    }

    /// Serialises settings to a simple key=value string for persistence.
    ///
    /// Validates: Requirement 17.11
    pub fn serialise(&self) -> String {
        format!(
            "action_display={}\nmain_panel={}\nrownum_on={}",
            self.action_display, self.main_panel, self.rownum_on
        )
    }

    /// Deserialises from the key=value string produced by serialise().
    ///
    /// Validates: Requirement 17.11
    pub fn deserialise(s: &str) -> Self {
        let mut settings = Self::default();
        for line in s.lines() {
            if let Some((k, v)) = line.split_once('=') {
                match k.trim() {
                    "action_display" => settings.action_display = v.trim() == "true",
                    "main_panel" => settings.main_panel = v.trim().to_uppercase(),
                    "rownum_on" => settings.rownum_on = v.trim() == "true",
                    _ => {}
                }
            }
        }
        settings
    }
}

// === WHO Command ============================================================

/// Session information displayed by the WHO command.
///
/// Validates: Requirement 17.9
#[derive(Debug, Clone)]
pub struct WhoInfo {
    pub user: String,
    pub session_start: String,
    pub prefix_filter: Option<String>,
    pub owner_filter: Option<String>,
    pub dest_filter: Option<String>,
    pub rownum_on: bool,
    pub main_panel: String,
    pub provider: String,
}

impl WhoInfo {
    /// Formats the WHO output as a multi-line string.
    pub fn format(&self) -> String {
        let mut lines = vec![
            format!("User:          {}", self.user),
            format!("Session start: {}", self.session_start),
            format!("Provider:      {}", self.provider),
            format!(
                "SET ROWNUM:    {}",
                if self.rownum_on { "ON" } else { "OFF" }
            ),
            format!("SET MAIN:      {}", self.main_panel),
        ];
        if let Some(ref p) = self.prefix_filter {
            lines.push(format!("PREFIX:        {p}"));
        }
        if let Some(ref o) = self.owner_filter {
            lines.push(format!("OWNER:         {o}"));
        }
        if let Some(ref d) = self.dest_filter {
            lines.push(format!("DEST:          {d}"));
        }
        lines.join("\n")
    }
}

// === QUERY AUTH =============================================================

/// An authorised command or action entry.
///
/// Validates: Requirement 17.10
#[derive(Debug, Clone)]
pub struct AuthEntry {
    pub name: String,
    pub kind: AuthKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthKind {
    Command,
    ActionChar,
}

/// Returns the default authorised command and action list.
///
/// Validates: Requirement 17.10
pub fn default_auth_list() -> Vec<AuthEntry> {
    let commands = [
        "PREFIX",
        "OWNER",
        "DEST",
        "SORT",
        "FIND",
        "LOCATE",
        "SET",
        "MENU",
        "WHO",
        "QUERY AUTH",
        "FILTER",
    ];
    let actions = ["S", "?", "C", "H", "A", "P", "D", "E", "J", "W"];
    let mut list: Vec<AuthEntry> = commands
        .iter()
        .map(|n| AuthEntry {
            name: n.to_string(),
            kind: AuthKind::Command,
        })
        .collect();
    list.extend(actions.iter().map(|n| AuthEntry {
        name: n.to_string(),
        kind: AuthKind::ActionChar,
    }));
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffjcl::{FfjclDefinition, FfjclStep};
    use crate::model::{Job, JobId};

    fn make_job(id: u64, name: &str) -> Job {
        let def = FfjclDefinition {
            job_name: name.to_string(),
            owner: None,
            priority: None,
            class: None,
            steps: vec![FfjclStep {
                name: "S1".to_string(),
                program: "PGM".to_string(),
                args: vec![],
                dds: vec![],
                condition: None,
            }],
            source: String::new(),
        };
        let mut job = Job::new(JobId::new(id), def, "user");
        job.name = name.to_string();
        job
    }

    // --- FindState tests ---

    #[test]
    fn find_first_returns_correct_index() {
        // Validates: Requirement 17.3
        let jobs = vec![
            make_job(1, "ALPHA"),
            make_job(2, "PAYROLL"),
            make_job(3, "BETA"),
        ];
        let mut fs = FindState::default();
        fs.set_pattern("PAY", FindCase::Insensitive);
        assert_eq!(fs.find_first(&jobs), Some(1));
    }

    #[test]
    fn find_next_advances_past_current() {
        // Validates: Requirement 17.3
        let jobs = vec![
            make_job(1, "PAY1"),
            make_job(2, "OTHER"),
            make_job(3, "PAY2"),
        ];
        let mut fs = FindState::default();
        fs.set_pattern("PAY", FindCase::Insensitive);
        fs.find_first(&jobs);
        assert_eq!(fs.find_next(&jobs), Some(2));
    }

    #[test]
    fn find_prev_moves_backward() {
        // Validates: Requirement 17.3
        let jobs = vec![
            make_job(1, "PAY1"),
            make_job(2, "OTHER"),
            make_job(3, "PAY2"),
        ];
        let mut fs = FindState::default();
        fs.set_pattern("PAY", FindCase::Insensitive);
        fs.current_match = Some(2);
        assert_eq!(fs.find_prev(&jobs), Some(0));
    }

    #[test]
    fn find_case_sensitive() {
        // Validates: Requirement 17.15
        let jobs = vec![make_job(1, "payroll"), make_job(2, "PAYROLL")];
        let mut fs = FindState::default();
        fs.set_pattern("PAYROLL", FindCase::Sensitive);
        assert_eq!(fs.find_first(&jobs), Some(1));
    }

    #[test]
    fn find_no_match_sets_is_no_match() {
        // Validates: Requirement 17.16
        let jobs = vec![make_job(1, "ALPHA")];
        let mut fs = FindState::default();
        fs.set_pattern("ZZZZZ", FindCase::Insensitive);
        fs.find_first(&jobs);
        assert!(fs.is_no_match());
    }

    // --- LOCATE tests ---

    #[test]
    fn locate_exact_prefix_match() {
        // Validates: Requirement 17.4
        let jobs = vec![
            make_job(1, "ALPHA"),
            make_job(2, "PAYROLL"),
            make_job(3, "ZEBRA"),
        ];
        assert_eq!(locate(&jobs, "PAY"), LocateResult::Found(1));
    }

    #[test]
    fn locate_nearest_alphabetic() {
        // Validates: Requirement 17.4
        let jobs = vec![
            make_job(1, "ALPHA"),
            make_job(2, "GAMMA"),
            make_job(3, "ZEBRA"),
        ];
        // "DELTA" falls between GAMMA and ZEBRA
        let result = locate(&jobs, "DELTA");
        assert!(matches!(result, LocateResult::Nearest(_)));
    }

    #[test]
    fn locate_empty_list() {
        assert_eq!(locate(&[], "PAY"), LocateResult::Empty);
    }

    // --- ScrollCommand tests ---

    #[test]
    fn scroll_down_page() {
        // Validates: Requirement 17.5
        let cmd = ScrollCommand::parse("DOWN PAGE", &ScrollAmount::Page).unwrap();
        assert_eq!(cmd.dir, ScrollDir::Down);
        let new_row = cmd.apply_vertical(0, 25, 100);
        assert_eq!(new_row, 25);
    }

    #[test]
    fn scroll_up_half() {
        // Validates: Requirement 17.5
        let cmd = ScrollCommand::parse("UP HALF", &ScrollAmount::Page).unwrap();
        let new_row = cmd.apply_vertical(50, 25, 100);
        assert_eq!(new_row, 37); // 50 - 25/2 = 37 (integer)
    }

    #[test]
    fn scroll_down_clamps_at_max() {
        // Validates: Requirement 17.5
        let cmd = ScrollCommand::parse("DOWN MAX", &ScrollAmount::Page).unwrap();
        let new_row = cmd.apply_vertical(0, 25, 30);
        assert_eq!(new_row, 5); // 30 - 25 = 5
    }

    #[test]
    fn scroll_up_clamps_at_zero() {
        // Validates: Requirement 17.5
        let cmd = ScrollCommand::parse("UP 100", &ScrollAmount::Page).unwrap();
        let new_row = cmd.apply_vertical(5, 25, 100);
        assert_eq!(new_row, 0);
    }

    #[test]
    fn scroll_uses_default_amount_when_omitted() {
        // Validates: Requirement 17.5, 17.17
        let default = ScrollAmount::Lines(10);
        let cmd = ScrollCommand::parse("DOWN", &default).unwrap();
        assert_eq!(cmd.amount, ScrollAmount::Lines(10));
    }

    #[test]
    fn scroll_updates_scroll_field() {
        // Validates: Requirement 17.17
        let cmd = ScrollCommand::parse("DOWN HALF", &ScrollAmount::Page).unwrap();
        assert_eq!(cmd.amount, ScrollAmount::Half);
    }

    #[test]
    fn scroll_invalid_direction_returns_none() {
        assert!(ScrollCommand::parse("SIDEWAYS", &ScrollAmount::Page).is_none());
    }

    // --- SdsfSetSettings tests ---

    #[test]
    fn set_action_toggles() {
        // Validates: Requirement 17.6
        let mut s = SdsfSetSettings::default();
        assert!(!s.action_display);
        s.set_action(true);
        assert!(s.action_display);
    }

    #[test]
    fn set_main_updates_panel() {
        // Validates: Requirement 17.7
        let mut s = SdsfSetSettings::default();
        s.set_main("ST");
        assert_eq!(s.main_panel, "ST");
    }

    #[test]
    fn set_rownum_toggles() {
        // Validates: Requirement 17.8
        let mut s = SdsfSetSettings::default();
        s.set_rownum(true);
        assert!(s.rownum_on);
        s.set_rownum(false);
        assert!(!s.rownum_on);
    }

    #[test]
    fn settings_serialise_round_trip() {
        // Validates: Requirement 17.11
        let mut s = SdsfSetSettings::default();
        s.set_action(true);
        s.set_main("ST");
        s.set_rownum(true);
        let serialised = s.serialise();
        let restored = SdsfSetSettings::deserialise(&serialised);
        assert_eq!(restored.action_display, true);
        assert_eq!(restored.main_panel, "ST");
        assert_eq!(restored.rownum_on, true);
    }

    #[test]
    fn settings_default_round_trip() {
        // Validates: Requirement 17.11
        let s = SdsfSetSettings::default();
        let restored = SdsfSetSettings::deserialise(&s.serialise());
        assert_eq!(restored.action_display, s.action_display);
        assert_eq!(restored.main_panel, s.main_panel);
        assert_eq!(restored.rownum_on, s.rownum_on);
    }

    // --- WHO tests ---

    #[test]
    fn who_format_contains_required_fields() {
        // Validates: Requirement 17.9
        let info = WhoInfo {
            user: "ALICE".to_string(),
            session_start: "10:00".to_string(),
            prefix_filter: Some("PAY".to_string()),
            owner_filter: None,
            dest_filter: None,
            rownum_on: false,
            main_panel: "I".to_string(),
            provider: "desktop".to_string(),
        };
        let output = info.format();
        assert!(output.contains("ALICE"));
        assert!(output.contains("10:00"));
        assert!(output.contains("desktop"));
        assert!(output.contains("PREFIX"));
        assert!(output.contains("PAY"));
    }

    #[test]
    fn who_omits_unset_filters() {
        // Validates: Requirement 17.9
        let info = WhoInfo {
            user: "BOB".to_string(),
            session_start: "09:00".to_string(),
            prefix_filter: None,
            owner_filter: None,
            dest_filter: None,
            rownum_on: false,
            main_panel: "I".to_string(),
            provider: "desktop".to_string(),
        };
        let output = info.format();
        assert!(!output.contains("PREFIX"));
        assert!(!output.contains("OWNER"));
        assert!(!output.contains("DEST"));
    }

    // --- QUERY AUTH tests ---

    #[test]
    fn query_auth_contains_commands_and_actions() {
        // Validates: Requirement 17.10
        let list = default_auth_list();
        assert!(list
            .iter()
            .any(|e| e.name == "FIND" && e.kind == AuthKind::Command));
        assert!(list
            .iter()
            .any(|e| e.name == "S" && e.kind == AuthKind::ActionChar));
        assert!(list
            .iter()
            .any(|e| e.name == "C" && e.kind == AuthKind::ActionChar));
    }

    #[test]
    fn query_auth_list_non_empty() {
        // Validates: Requirement 17.10
        assert!(!default_auth_list().is_empty());
    }
}
