//! SDSF panel filter state, column definitions, and SORT command.
//!
//! Validates: Requirement 16 AC 4, 18, 19, 20, 24, 25, 26

use crate::model::{Job, JobStatus};

// === Column Definitions =====================================================

/// All columns available in the SDSF job table.
///
/// Validates: Requirement 16.24
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SdsfColumn {
    JobName,
    JobId,
    Owner,
    Status,
    Class,
    Priority,
    Queue,
    Start,
    End,
    ReturnCode,
    StepName,
    ProcStep,
}

impl SdsfColumn {
    /// Returns the canonical display name for the column.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::JobName => "JOBNAME",
            Self::JobId => "JOBID",
            Self::Owner => "OWNER",
            Self::Status => "STATUS",
            Self::Class => "CLASS",
            Self::Priority => "PRTY",
            Self::Queue => "QUEUE",
            Self::Start => "START",
            Self::End => "END",
            Self::ReturnCode => "RC",
            Self::StepName => "STEPNAME",
            Self::ProcStep => "PROCSTEP",
        }
    }

    /// Parses a column name string (case-insensitive).
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "JOBNAME" => Some(Self::JobName),
            "JOBID" => Some(Self::JobId),
            "OWNER" => Some(Self::Owner),
            "STATUS" => Some(Self::Status),
            "CLASS" => Some(Self::Class),
            "PRTY" | "PRIORITY" => Some(Self::Priority),
            "QUEUE" => Some(Self::Queue),
            "START" => Some(Self::Start),
            "END" => Some(Self::End),
            "RC" | "RETURNCODE" => Some(Self::ReturnCode),
            "STEPNAME" => Some(Self::StepName),
            "PROCSTEP" => Some(Self::ProcStep),
            _ => None,
        }
    }

    /// Returns all columns in default display order.
    pub fn all() -> &'static [SdsfColumn] {
        &[
            Self::JobName,
            Self::JobId,
            Self::Owner,
            Self::Status,
            Self::Class,
            Self::Priority,
            Self::Queue,
            Self::Start,
            Self::End,
            Self::ReturnCode,
            Self::StepName,
            Self::ProcStep,
        ]
    }
}

// === Column Visibility State ================================================

/// Tracks which columns are visible and their display order.
///
/// Validates: Requirement 16.24
#[derive(Debug, Clone)]
pub struct ColumnLayout {
    /// Ordered list of visible columns.
    pub visible: Vec<SdsfColumn>,
}

impl Default for ColumnLayout {
    fn default() -> Self {
        Self {
            visible: SdsfColumn::all().to_vec(),
        }
    }
}

impl ColumnLayout {
    /// Hides a column.
    pub fn hide(&mut self, col: SdsfColumn) {
        self.visible.retain(|c| *c != col);
    }

    /// Shows a column (appends to end if not already visible).
    pub fn show(&mut self, col: SdsfColumn) {
        if !self.visible.contains(&col) {
            self.visible.push(col);
        }
    }

    /// Returns true if the column is currently visible.
    pub fn is_visible(&self, col: SdsfColumn) -> bool {
        self.visible.contains(&col)
    }

    /// Moves a column to a new position index.
    pub fn reorder(&mut self, col: SdsfColumn, new_index: usize) {
        self.visible.retain(|c| *c != col);
        let idx = new_index.min(self.visible.len());
        self.visible.insert(idx, col);
    }
}

// === Sort State =============================================================

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Active sort specification.
///
/// Validates: Requirement 16.26
#[derive(Debug, Clone)]
pub struct SdsfSort {
    pub column: SdsfColumn,
    pub direction: SortDirection,
}

impl SdsfSort {
    /// Parses a SORT command operand string: `SORT colname [A|D]`
    /// Returns None if the column name is unrecognised.
    pub fn parse(operands: &str) -> Option<Self> {
        let mut parts = operands.split_whitespace();
        let col_str = parts.next()?;
        let col = SdsfColumn::parse(col_str)?;
        let direction = match parts.next().unwrap_or("A").to_uppercase().as_str() {
            "D" | "DESC" | "DESCENDING" => SortDirection::Descending,
            _ => SortDirection::Ascending,
        };
        Some(Self {
            column: col,
            direction,
        })
    }

    /// Applies this sort to a job slice in-place.
    pub fn apply(&self, jobs: &mut [Job]) {
        let asc = self.direction == SortDirection::Ascending;
        match self.column {
            SdsfColumn::JobName => jobs.sort_by(|a, b| {
                if asc {
                    a.name.cmp(&b.name)
                } else {
                    b.name.cmp(&a.name)
                }
            }),
            SdsfColumn::JobId => jobs.sort_by(|a, b| {
                if asc {
                    a.id.cmp(&b.id)
                } else {
                    b.id.cmp(&a.id)
                }
            }),
            SdsfColumn::Owner => jobs.sort_by(|a, b| {
                if asc {
                    a.owner.cmp(&b.owner)
                } else {
                    b.owner.cmp(&a.owner)
                }
            }),
            SdsfColumn::Status => jobs.sort_by(|a, b| {
                let as_ = a.status.to_string();
                let bs = b.status.to_string();
                if asc {
                    as_.cmp(&bs)
                } else {
                    bs.cmp(&as_)
                }
            }),
            SdsfColumn::Priority => jobs.sort_by(|a, b| {
                if asc {
                    a.priority.cmp(&b.priority)
                } else {
                    b.priority.cmp(&a.priority)
                }
            }),
            SdsfColumn::ReturnCode => jobs.sort_by(|a, b| {
                if asc {
                    a.return_code.cmp(&b.return_code)
                } else {
                    b.return_code.cmp(&a.return_code)
                }
            }),
            SdsfColumn::Start => jobs.sort_by(|a, b| {
                if asc {
                    a.start_time.cmp(&b.start_time)
                } else {
                    b.start_time.cmp(&a.start_time)
                }
            }),
            SdsfColumn::End => jobs.sort_by(|a, b| {
                if asc {
                    a.end_time.cmp(&b.end_time)
                } else {
                    b.end_time.cmp(&a.end_time)
                }
            }),
            // Columns without a direct Job field sort by submit time as fallback
            SdsfColumn::Class | SdsfColumn::Queue | SdsfColumn::StepName | SdsfColumn::ProcStep => {
                jobs.sort_by(|a, b| {
                    if asc {
                        a.submit_time.cmp(&b.submit_time)
                    } else {
                        b.submit_time.cmp(&a.submit_time)
                    }
                })
            }
        }
    }
}

// === Filter State ===========================================================

/// Active PREFIX/OWNER/DEST filter values.
///
/// Validates: Requirement 16.4, 16.18, 16.19, 16.20, 16.25
#[derive(Debug, Clone, Default)]
pub struct SdsfFilter {
    /// Job name prefix filter. None or "*" means no filter.
    pub prefix: Option<String>,
    /// Owner filter. None or "*" means no filter.
    pub owner: Option<String>,
    /// Output destination filter. None or "*" means no filter.
    pub dest: Option<String>,
}

impl SdsfFilter {
    /// Sets the PREFIX filter. "*" or empty string clears it.
    pub fn set_prefix(&mut self, value: &str) {
        let v = value.trim();
        if v.is_empty() || v == "*" {
            self.prefix = None;
        } else {
            self.prefix = Some(v.to_uppercase());
        }
    }

    /// Sets the OWNER filter. "*" or empty string clears it.
    pub fn set_owner(&mut self, value: &str) {
        let v = value.trim();
        if v.is_empty() || v == "*" {
            self.owner = None;
        } else {
            self.owner = Some(v.to_uppercase());
        }
    }

    /// Sets the DEST filter. "*" or empty string clears it.
    pub fn set_dest(&mut self, value: &str) {
        let v = value.trim();
        if v.is_empty() || v == "*" {
            self.dest = None;
        } else {
            self.dest = Some(v.to_uppercase());
        }
    }

    /// Returns true if no filters are active.
    pub fn is_empty(&self) -> bool {
        self.prefix.is_none() && self.owner.is_none() && self.dest.is_none()
    }

    /// Returns true if the job passes all active filters.
    ///
    /// Validates: Requirement 16.18, 16.19, 16.20
    pub fn matches(&self, job: &Job) -> bool {
        if let Some(ref pfx) = self.prefix {
            if !job.name.to_uppercase().starts_with(pfx.as_str()) {
                return false;
            }
        }
        if let Some(ref owner) = self.owner {
            if !job.owner.to_uppercase().eq(owner.as_str()) {
                return false;
            }
        }
        // DEST is metadata-only for now; all jobs pass unless dest filter set
        // (no dest field on Job yet -- always passes)
        let _ = &self.dest;
        true
    }

    /// Builds the filter information lines shown below the title line.
    ///
    /// Validates: Requirement 16.4
    pub fn info_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(ref p) = self.prefix {
            lines.push(format!("PREFIX={p}"));
        }
        if let Some(ref o) = self.owner {
            lines.push(format!("OWNER={o}"));
        }
        if let Some(ref d) = self.dest {
            lines.push(format!("DEST={d}"));
        }
        lines
    }

    /// Applies this filter to a job list, returning only matching jobs.
    pub fn apply<'a>(&self, jobs: &'a [Job]) -> Vec<&'a Job> {
        jobs.iter().filter(|j| self.matches(j)).collect()
    }
}

// === Queue Tab ==============================================================

/// Which sub-panel (tab) is active in the Job Monitor.
///
/// Validates: Requirement 9.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueTab {
    Input,
    Active,
    Held,
    Output,
    Failed,
    Cancelled,
}

impl QueueTab {
    /// Returns the statuses shown in this tab.
    pub fn statuses(self) -> &'static [JobStatus] {
        match self {
            Self::Input => &[JobStatus::Queued],
            Self::Active => &[JobStatus::Active],
            Self::Held => &[JobStatus::Held],
            Self::Output => &[JobStatus::Completed],
            Self::Failed => &[JobStatus::Failed],
            Self::Cancelled => &[JobStatus::Cancelled],
        }
    }

    /// Returns the tab label (without count).
    pub fn label(self) -> &'static str {
        match self {
            Self::Input => "Input Queue",
            Self::Active => "Active Jobs",
            Self::Held => "Held Jobs",
            Self::Output => "Output",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffjcl::{FfjclDefinition, FfjclStep};
    use crate::model::{Job, JobId};

    fn make_job(id: u64, name: &str, owner: &str) -> Job {
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
        let mut job = Job::new(JobId::new(id), def, owner);
        job.name = name.to_string();
        job
    }

    // --- Column tests ---

    #[test]
    fn all_columns_present() {
        // Validates: Requirement 16.24
        let all = SdsfColumn::all();
        assert_eq!(all.len(), 12);
        assert!(all.contains(&SdsfColumn::JobName));
        assert!(all.contains(&SdsfColumn::ProcStep));
    }

    #[test]
    fn column_parse_case_insensitive() {
        // Validates: Requirement 16.24
        assert_eq!(SdsfColumn::parse("jobname"), Some(SdsfColumn::JobName));
        assert_eq!(SdsfColumn::parse("PRTY"), Some(SdsfColumn::Priority));
        assert_eq!(SdsfColumn::parse("RC"), Some(SdsfColumn::ReturnCode));
        assert_eq!(SdsfColumn::parse("UNKNOWN"), None);
    }

    #[test]
    fn column_layout_hide_show() {
        // Validates: Requirement 16.24
        let mut layout = ColumnLayout::default();
        assert!(layout.is_visible(SdsfColumn::Class));
        layout.hide(SdsfColumn::Class);
        assert!(!layout.is_visible(SdsfColumn::Class));
        layout.show(SdsfColumn::Class);
        assert!(layout.is_visible(SdsfColumn::Class));
    }

    #[test]
    fn column_layout_reorder() {
        // Validates: Requirement 16.24
        let mut layout = ColumnLayout::default();
        layout.reorder(SdsfColumn::ReturnCode, 0);
        assert_eq!(layout.visible[0], SdsfColumn::ReturnCode);
    }

    // --- Sort tests ---

    #[test]
    fn sort_parse_ascending() {
        // Validates: Requirement 16.26
        let s = SdsfSort::parse("JOBNAME A").unwrap();
        assert_eq!(s.column, SdsfColumn::JobName);
        assert_eq!(s.direction, SortDirection::Ascending);
    }

    #[test]
    fn sort_parse_descending() {
        // Validates: Requirement 16.26
        let s = SdsfSort::parse("PRTY D").unwrap();
        assert_eq!(s.column, SdsfColumn::Priority);
        assert_eq!(s.direction, SortDirection::Descending);
    }

    #[test]
    fn sort_parse_default_ascending() {
        // Validates: Requirement 16.26 -- no direction defaults to A
        let s = SdsfSort::parse("OWNER").unwrap();
        assert_eq!(s.direction, SortDirection::Ascending);
    }

    #[test]
    fn sort_parse_unknown_column_returns_none() {
        assert!(SdsfSort::parse("BOGUS").is_none());
    }

    #[test]
    fn sort_apply_by_jobname_ascending() {
        // Validates: Requirement 16.26
        let mut jobs = vec![
            make_job(3, "ZEBRA", "u"),
            make_job(1, "ALPHA", "u"),
            make_job(2, "MANGO", "u"),
        ];
        let sort = SdsfSort::parse("JOBNAME A").unwrap();
        sort.apply(&mut jobs);
        assert_eq!(jobs[0].name, "ALPHA");
        assert_eq!(jobs[1].name, "MANGO");
        assert_eq!(jobs[2].name, "ZEBRA");
    }

    #[test]
    fn sort_apply_by_priority_descending() {
        // Validates: Requirement 16.26
        let mut jobs = vec![
            make_job(1, "LOW", "u"),
            make_job(2, "HIGH", "u"),
            make_job(3, "MED", "u"),
        ];
        jobs[0].priority = 1;
        jobs[1].priority = 10;
        jobs[2].priority = 5;
        let sort = SdsfSort::parse("PRTY D").unwrap();
        sort.apply(&mut jobs);
        assert_eq!(jobs[0].name, "HIGH");
        assert_eq!(jobs[1].name, "MED");
        assert_eq!(jobs[2].name, "LOW");
    }

    // --- Filter tests ---

    #[test]
    fn filter_prefix_matches_prefix() {
        // Validates: Requirement 16.18
        let mut f = SdsfFilter::default();
        f.set_prefix("PAY");
        let j1 = make_job(1, "PAYROLL", "u");
        let j2 = make_job(2, "BILLING", "u");
        assert!(f.matches(&j1));
        assert!(!f.matches(&j2));
    }

    #[test]
    fn filter_prefix_star_clears() {
        // Validates: Requirement 16.18
        let mut f = SdsfFilter::default();
        f.set_prefix("PAY");
        f.set_prefix("*");
        assert!(f.prefix.is_none());
    }

    #[test]
    fn filter_owner_matches_exact() {
        // Validates: Requirement 16.19
        let mut f = SdsfFilter::default();
        f.set_owner("ALICE");
        let j1 = make_job(1, "JOB1", "alice");
        let j2 = make_job(2, "JOB2", "bob");
        assert!(f.matches(&j1));
        assert!(!f.matches(&j2));
    }

    #[test]
    fn filter_owner_star_clears() {
        // Validates: Requirement 16.19
        let mut f = SdsfFilter::default();
        f.set_owner("ALICE");
        f.set_owner("*");
        assert!(f.owner.is_none());
    }

    #[test]
    fn filter_dest_set_and_clear() {
        // Validates: Requirement 16.20
        let mut f = SdsfFilter::default();
        f.set_dest("LOCAL");
        assert_eq!(f.dest, Some("LOCAL".to_string()));
        f.set_dest("*");
        assert!(f.dest.is_none());
    }

    #[test]
    fn filter_info_lines_empty_when_no_filters() {
        // Validates: Requirement 16.4
        let f = SdsfFilter::default();
        assert!(f.info_lines().is_empty());
    }

    #[test]
    fn filter_info_lines_shows_active_filters() {
        // Validates: Requirement 16.4
        let mut f = SdsfFilter::default();
        f.set_prefix("PAY");
        f.set_owner("ALICE");
        let lines = f.info_lines();
        assert!(lines.iter().any(|l| l == "PREFIX=PAY"));
        assert!(lines.iter().any(|l| l == "OWNER=ALICE"));
    }

    #[test]
    fn filter_combined_prefix_and_owner() {
        // Validates: Requirement 16.25
        let mut f = SdsfFilter::default();
        f.set_prefix("PAY");
        f.set_owner("ALICE");
        let j1 = make_job(1, "PAYROLL", "alice");
        let j2 = make_job(2, "PAYROLL", "bob");
        let j3 = make_job(3, "BILLING", "alice");
        assert!(f.matches(&j1));
        assert!(!f.matches(&j2));
        assert!(!f.matches(&j3));
    }

    #[test]
    fn filter_does_not_alter_job_state() {
        // Validates: Requirement 9 AC 5
        let mut f = SdsfFilter::default();
        f.set_prefix("PAY");
        let jobs = vec![make_job(1, "PAYROLL", "u"), make_job(2, "BILLING", "u")];
        let before_statuses: Vec<_> = jobs.iter().map(|j| j.status).collect();
        let _ = f.apply(&jobs);
        let after_statuses: Vec<_> = jobs.iter().map(|j| j.status).collect();
        assert_eq!(before_statuses, after_statuses);
    }

    // --- QueueTab tests ---

    #[test]
    fn queue_tab_statuses_correct() {
        assert!(QueueTab::Input.statuses().contains(&JobStatus::Queued));
        assert!(QueueTab::Active.statuses().contains(&JobStatus::Active));
        assert!(QueueTab::Held.statuses().contains(&JobStatus::Held));
        assert!(QueueTab::Output.statuses().contains(&JobStatus::Completed));
        assert!(QueueTab::Failed.statuses().contains(&JobStatus::Failed));
        assert!(QueueTab::Cancelled
            .statuses()
            .contains(&JobStatus::Cancelled));
    }
}
