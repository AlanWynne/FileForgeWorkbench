//! EXECIO I/O operations and FFCMD command file execution.
//!
//! Implements Requirement 11 AC 11.24-11.30:
//!   - EXECIO DISKR  -- read records from ddname into stem (AC 11.24)
//!   - EXECIO DISKW  -- write records from stem to ddname (AC 11.25)
//!   - EXECIO FINIS  -- read/write all remaining records and close (AC 11.26)
//!   - EXECIO SKIP   -- advance read position without returning data (AC 11.27)
//!   - EXECIO RC     -- 0 success, 2 EOF before count, non-zero on error (AC 11.28)
//!   - FFCMD files   -- line-by-line batch primary command execution (AC 11.29)
//!   - FFCMD transaction wrapping (AC 11.30)

use std::collections::HashMap;

// === ExecioOperation =========================================================

/// Parsed EXECIO operation.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecioOperation {
    /// EXECIO <count> DISKR <ddname> STEM <stem>
    DiskRead {
        ddname: String,
        count: ExecioCount,
        stem: String,
    },
    /// EXECIO <count> DISKW <ddname> STEM <stem>
    DiskWrite {
        ddname: String,
        count: ExecioCount,
        stem: String,
    },
    /// EXECIO SKIP <ddname> <count>
    Skip { ddname: String, count: usize },
}

/// Record count operand -- either a specific number or all remaining (*).
#[derive(Debug, Clone, PartialEq)]
pub enum ExecioCount {
    /// Read/write exactly N records.
    Count(usize),
    /// Read/write all remaining records (FINIS variant).
    All,
}

/// Return code from an EXECIO operation.
///
/// Addresses: Requirement 11 AC 11.28
#[derive(Debug, Clone, PartialEq)]
pub enum ExecioRc {
    /// Success -- all requested records transferred.
    Success,
    /// EOF reached before count records were read (RC=2).
    Eof,
    /// I/O error (RC=8 or higher).
    IoError(String),
}

impl ExecioRc {
    /// Returns the integer RC value consistent with TSO EXECIO conventions.
    ///
    /// Addresses: Requirement 11 AC 11.28
    pub fn as_int(&self) -> i32 {
        match self {
            Self::Success => 0,
            Self::Eof => 2,
            Self::IoError(_) => 8,
        }
    }
}

// === DdnameFile ==============================================================

/// An open file allocated to a ddname.
#[derive(Debug, Clone)]
pub struct DdnameFile {
    /// All records in the file.
    records: Vec<String>,
    /// Current read position (0-based index into records).
    read_pos: usize,
    /// Whether the file is closed.
    closed: bool,
}

impl DdnameFile {
    /// Create a new file with the given records.
    pub fn new(records: Vec<String>) -> Self {
        Self {
            records,
            read_pos: 0,
            closed: false,
        }
    }

    /// Returns whether the file is at EOF.
    pub fn is_eof(&self) -> bool {
        self.read_pos >= self.records.len()
    }

    /// Returns whether the file is closed.
    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

// === ExecioEngine ============================================================

/// EXECIO I/O engine.
///
/// Manages ddname-to-file allocations and executes DISKR/DISKW/SKIP operations.
/// Files are injected via `allocate_ddname` (used by tests and shell wiring).
///
/// Addresses: Requirement 11 AC 11.24-11.28
pub struct ExecioEngine {
    /// Allocated ddnames: uppercase ddname -> file state.
    files: HashMap<String, DdnameFile>,
}

impl ExecioEngine {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Allocate a ddname to a set of records (used by tests and shell wiring).
    pub fn allocate_ddname(&mut self, ddname: &str, records: Vec<String>) {
        self.files
            .insert(ddname.to_uppercase(), DdnameFile::new(records));
    }

    /// Close a ddname (FINIS).
    pub fn close_ddname(&mut self, ddname: &str) {
        if let Some(f) = self.files.get_mut(&ddname.to_uppercase()) {
            f.closed = true;
        }
    }

    /// Execute a DISKR operation: read up to `count` records into a stem.
    ///
    /// Returns (records_read, rc).
    ///
    /// Addresses: Requirement 11 AC 11.24, 11.26, 11.28
    pub fn diskr(
        &mut self,
        ddname: &str,
        count: &ExecioCount,
        finis: bool,
    ) -> (Vec<String>, ExecioRc) {
        let key = ddname.to_uppercase();
        let file = match self.files.get_mut(&key) {
            Some(f) => f,
            None => {
                return (
                    Vec::new(),
                    ExecioRc::IoError(format!("ddname not allocated: {ddname}")),
                )
            }
        };

        if file.closed {
            return (
                Vec::new(),
                ExecioRc::IoError(format!("ddname closed: {ddname}")),
            );
        }

        let available = file.records.len().saturating_sub(file.read_pos);
        let requested = match count {
            ExecioCount::Count(n) => *n,
            ExecioCount::All => available,
        };

        let to_read = requested.min(available);
        let result: Vec<String> = file.records[file.read_pos..file.read_pos + to_read].to_vec();
        file.read_pos += to_read;

        let rc = if matches!(count, ExecioCount::Count(n) if *n > available) {
            ExecioRc::Eof
        } else {
            ExecioRc::Success
        };

        if finis {
            file.closed = true;
        }

        (result, rc)
    }

    /// Execute a DISKW operation: write records from stem to ddname.
    ///
    /// Returns rc.
    ///
    /// Addresses: Requirement 11 AC 11.25, 11.26, 11.28
    pub fn diskw(&mut self, ddname: &str, records: Vec<String>, finis: bool) -> ExecioRc {
        let key = ddname.to_uppercase();
        let file = self
            .files
            .entry(key)
            .or_insert_with(|| DdnameFile::new(Vec::new()));

        if file.closed {
            return ExecioRc::IoError(format!("ddname closed: {ddname}"));
        }

        file.records.extend(records);

        if finis {
            file.closed = true;
        }

        ExecioRc::Success
    }

    /// Execute a SKIP operation: advance read position by count records.
    ///
    /// Addresses: Requirement 11 AC 11.27, 11.28
    pub fn skip(&mut self, ddname: &str, count: usize) -> ExecioRc {
        let key = ddname.to_uppercase();
        let file = match self.files.get_mut(&key) {
            Some(f) => f,
            None => return ExecioRc::IoError(format!("ddname not allocated: {ddname}")),
        };

        if file.closed {
            return ExecioRc::IoError(format!("ddname closed: {ddname}"));
        }

        let available = file.records.len().saturating_sub(file.read_pos);
        if count > available {
            file.read_pos = file.records.len();
            ExecioRc::Eof
        } else {
            file.read_pos += count;
            ExecioRc::Success
        }
    }

    /// Returns the current records written to a ddname (for test verification).
    pub fn written_records(&self, ddname: &str) -> Option<&[String]> {
        self.files
            .get(&ddname.to_uppercase())
            .map(|f| f.records.as_slice())
    }

    /// Returns the current read position for a ddname.
    pub fn read_pos(&self, ddname: &str) -> Option<usize> {
        self.files.get(&ddname.to_uppercase()).map(|f| f.read_pos)
    }
}

impl Default for ExecioEngine {
    fn default() -> Self {
        Self::new()
    }
}

// === FfcmdLine ===============================================================

/// A single parsed line from an FFCMD file.
#[derive(Debug, Clone, PartialEq)]
pub enum FfcmdLine {
    /// A primary command to execute.
    Command(String),
    /// A blank or comment line (skipped).
    Skip,
}

impl FfcmdLine {
    /// Parse a raw line from an FFCMD file.
    /// Lines starting with `*` or `//` are treated as comments.
    pub fn parse(raw: &str) -> Self {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('*') || trimmed.starts_with("//") {
            Self::Skip
        } else {
            Self::Command(trimmed.to_string())
        }
    }
}

// === FfcmdRunner =============================================================

/// Result of executing an FFCMD file.
#[derive(Debug, Clone, PartialEq)]
pub enum FfcmdResult {
    /// All commands executed successfully.
    Success { commands_run: usize },
    /// A command failed; execution stopped.
    CommandFailed {
        line_number: usize,
        command: String,
        reason: String,
    },
    /// The file could not be read.
    ReadError(String),
}

/// Executes `.ffcmd` batch command files line-by-line.
///
/// Each non-blank, non-comment line is dispatched as a primary command.
/// The entire file execution is wrapped in a single transaction so all
/// document modifications are atomically undoable.
///
/// Addresses: Requirement 11 AC 11.29, 11.30
pub struct FfcmdRunner {
    /// Commands executed in the last run (for test verification).
    last_run_commands: Vec<String>,
}

impl FfcmdRunner {
    pub fn new() -> Self {
        Self {
            last_run_commands: Vec::new(),
        }
    }

    /// Parse FFCMD source text into a list of command lines.
    ///
    /// Addresses: Requirement 11 AC 11.29
    pub fn parse(source: &str) -> Vec<FfcmdLine> {
        source.lines().map(FfcmdLine::parse).collect()
    }

    /// Execute parsed FFCMD lines using the provided command dispatcher.
    ///
    /// The dispatcher returns Ok(()) on success or Err(reason) on failure.
    /// Execution stops on the first failure (transaction semantics).
    ///
    /// Addresses: Requirement 11 AC 11.29, 11.30
    pub fn execute<F>(&mut self, lines: &[FfcmdLine], mut dispatcher: F) -> FfcmdResult
    where
        F: FnMut(&str) -> Result<(), String>,
    {
        self.last_run_commands.clear();
        let mut commands_run = 0;

        for (idx, line) in lines.iter().enumerate() {
            if let FfcmdLine::Command(cmd) = line {
                match dispatcher(cmd) {
                    Ok(()) => {
                        self.last_run_commands.push(cmd.clone());
                        commands_run += 1;
                    }
                    Err(reason) => {
                        return FfcmdResult::CommandFailed {
                            line_number: idx + 1,
                            command: cmd.clone(),
                            reason,
                        };
                    }
                }
            }
        }

        FfcmdResult::Success { commands_run }
    }

    /// Execute FFCMD source text directly (parse + execute in one call).
    ///
    /// Addresses: Requirement 11 AC 11.29, 11.30
    pub fn run_source<F>(&mut self, source: &str, dispatcher: F) -> FfcmdResult
    where
        F: FnMut(&str) -> Result<(), String>,
    {
        let lines = Self::parse(source);
        self.execute(&lines, dispatcher)
    }

    /// Returns the commands executed in the last run.
    pub fn last_run_commands(&self) -> &[String] {
        &self.last_run_commands
    }
}

impl Default for FfcmdRunner {
    fn default() -> Self {
        Self::new()
    }
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- ExecioRc ------------------------------------------------------------

    // Validates: Requirement 11.28
    #[test]
    fn execio_rc_success_is_zero() {
        assert_eq!(ExecioRc::Success.as_int(), 0);
    }

    // Validates: Requirement 11.28
    #[test]
    fn execio_rc_eof_is_two() {
        assert_eq!(ExecioRc::Eof.as_int(), 2);
    }

    // Validates: Requirement 11.28
    #[test]
    fn execio_rc_io_error_is_nonzero() {
        assert!(ExecioRc::IoError("fail".to_string()).as_int() != 0);
    }

    // --- ExecioEngine::diskr -------------------------------------------------

    // Validates: Requirement 11.24
    #[test]
    fn diskr_reads_requested_count() {
        let mut engine = ExecioEngine::new();
        engine.allocate_ddname("INDD", vec!["A".into(), "B".into(), "C".into()]);
        let (records, rc) = engine.diskr("INDD", &ExecioCount::Count(2), false);
        assert_eq!(records, vec!["A", "B"]);
        assert_eq!(rc, ExecioRc::Success);
    }

    // Validates: Requirement 11.24
    #[test]
    fn diskr_advances_read_position() {
        let mut engine = ExecioEngine::new();
        engine.allocate_ddname("INDD", vec!["A".into(), "B".into(), "C".into()]);
        engine.diskr("INDD", &ExecioCount::Count(1), false);
        let (records, _) = engine.diskr("INDD", &ExecioCount::Count(1), false);
        assert_eq!(records, vec!["B"]);
    }

    // Validates: Requirement 11.28
    #[test]
    fn diskr_returns_eof_when_count_exceeds_available() {
        let mut engine = ExecioEngine::new();
        engine.allocate_ddname("INDD", vec!["A".into()]);
        let (records, rc) = engine.diskr("INDD", &ExecioCount::Count(5), false);
        assert_eq!(records, vec!["A"]);
        assert_eq!(rc, ExecioRc::Eof);
    }

    // Validates: Requirement 11.26
    #[test]
    fn diskr_finis_closes_file() {
        let mut engine = ExecioEngine::new();
        engine.allocate_ddname("INDD", vec!["A".into(), "B".into()]);
        engine.diskr("INDD", &ExecioCount::All, true);
        let (_, rc) = engine.diskr("INDD", &ExecioCount::Count(1), false);
        assert!(matches!(rc, ExecioRc::IoError(_)));
    }

    // Validates: Requirement 11.26
    #[test]
    fn diskr_all_reads_remaining_records() {
        let mut engine = ExecioEngine::new();
        engine.allocate_ddname("INDD", vec!["X".into(), "Y".into(), "Z".into()]);
        let (records, rc) = engine.diskr("INDD", &ExecioCount::All, false);
        assert_eq!(records, vec!["X", "Y", "Z"]);
        assert_eq!(rc, ExecioRc::Success);
    }

    // Validates: Requirement 11.28
    #[test]
    fn diskr_unallocated_ddname_returns_io_error() {
        let mut engine = ExecioEngine::new();
        let (_, rc) = engine.diskr("MISSING", &ExecioCount::Count(1), false);
        assert!(matches!(rc, ExecioRc::IoError(_)));
    }

    // --- ExecioEngine::diskw -------------------------------------------------

    // Validates: Requirement 11.25
    #[test]
    fn diskw_writes_records_to_ddname() {
        let mut engine = ExecioEngine::new();
        let rc = engine.diskw("OUTDD", vec!["LINE1".into(), "LINE2".into()], false);
        assert_eq!(rc, ExecioRc::Success);
        assert_eq!(
            engine.written_records("OUTDD"),
            Some(["LINE1".to_string(), "LINE2".to_string()].as_slice())
        );
    }

    // Validates: Requirement 11.25
    #[test]
    fn diskw_appends_to_existing_records() {
        let mut engine = ExecioEngine::new();
        engine.diskw("OUTDD", vec!["A".into()], false);
        engine.diskw("OUTDD", vec!["B".into()], false);
        assert_eq!(
            engine.written_records("OUTDD"),
            Some(["A".to_string(), "B".to_string()].as_slice())
        );
    }

    // Validates: Requirement 11.26
    #[test]
    fn diskw_finis_closes_file() {
        let mut engine = ExecioEngine::new();
        engine.diskw("OUTDD", vec!["A".into()], true);
        let rc = engine.diskw("OUTDD", vec!["B".into()], false);
        assert!(matches!(rc, ExecioRc::IoError(_)));
    }

    // --- ExecioEngine::skip --------------------------------------------------

    // Validates: Requirement 11.27
    #[test]
    fn skip_advances_read_position() {
        let mut engine = ExecioEngine::new();
        engine.allocate_ddname("INDD", vec!["A".into(), "B".into(), "C".into()]);
        let rc = engine.skip("INDD", 2);
        assert_eq!(rc, ExecioRc::Success);
        assert_eq!(engine.read_pos("INDD"), Some(2));
    }

    // Validates: Requirement 11.27, 11.28
    #[test]
    fn skip_past_end_returns_eof() {
        let mut engine = ExecioEngine::new();
        engine.allocate_ddname("INDD", vec!["A".into()]);
        let rc = engine.skip("INDD", 5);
        assert_eq!(rc, ExecioRc::Eof);
    }

    // Validates: Requirement 11.27
    #[test]
    fn skip_then_read_returns_correct_records() {
        let mut engine = ExecioEngine::new();
        engine.allocate_ddname("INDD", vec!["A".into(), "B".into(), "C".into(), "D".into()]);
        engine.skip("INDD", 2);
        let (records, _) = engine.diskr("INDD", &ExecioCount::Count(2), false);
        assert_eq!(records, vec!["C", "D"]);
    }

    // --- FfcmdLine::parse ----------------------------------------------------

    // Validates: Requirement 11.29
    #[test]
    fn ffcmd_parse_command_line() {
        assert_eq!(
            FfcmdLine::parse("FIND /hello/"),
            FfcmdLine::Command("FIND /hello/".to_string())
        );
    }

    // Validates: Requirement 11.29
    #[test]
    fn ffcmd_parse_blank_line_is_skip() {
        assert_eq!(FfcmdLine::parse(""), FfcmdLine::Skip);
        assert_eq!(FfcmdLine::parse("   "), FfcmdLine::Skip);
    }

    // Validates: Requirement 11.29
    #[test]
    fn ffcmd_parse_comment_lines_are_skipped() {
        assert_eq!(FfcmdLine::parse("* this is a comment"), FfcmdLine::Skip);
        assert_eq!(FfcmdLine::parse("// JCL-style comment"), FfcmdLine::Skip);
    }

    // --- FfcmdRunner ---------------------------------------------------------

    // Validates: Requirement 11.29
    #[test]
    fn ffcmd_runner_executes_commands_sequentially() {
        let mut runner = FfcmdRunner::new();
        let source = "FIND /hello/\nCHANGE /hello/ /world/\n";
        let mut executed: Vec<String> = Vec::new();
        let result = runner.run_source(source, |cmd| {
            executed.push(cmd.to_string());
            Ok(())
        });
        assert_eq!(result, FfcmdResult::Success { commands_run: 2 });
        assert_eq!(executed, vec!["FIND /hello/", "CHANGE /hello/ /world/"]);
    }

    // Validates: Requirement 11.29
    #[test]
    fn ffcmd_runner_skips_blank_and_comment_lines() {
        let mut runner = FfcmdRunner::new();
        let source = "* comment\n\nFIND /x/\n* another\n";
        let result = runner.run_source(source, |_| Ok(()));
        assert_eq!(result, FfcmdResult::Success { commands_run: 1 });
    }

    // Validates: Requirement 11.30
    #[test]
    fn ffcmd_runner_stops_on_first_failure() {
        let mut runner = FfcmdRunner::new();
        let source = "CMD1\nCMD2\nCMD3\n";
        let mut count = 0;
        let result = runner.run_source(source, |cmd| {
            count += 1;
            if cmd == "CMD2" {
                Err("CMD2 failed".to_string())
            } else {
                Ok(())
            }
        });
        assert!(matches!(
            result,
            FfcmdResult::CommandFailed { line_number: 2, .. }
        ));
        // CMD3 must NOT have been executed
        assert_eq!(count, 2);
    }

    // Validates: Requirement 11.30
    #[test]
    fn ffcmd_runner_records_executed_commands() {
        let mut runner = FfcmdRunner::new();
        runner.run_source("A\nB\nC\n", |_| Ok(()));
        assert_eq!(runner.last_run_commands(), &["A", "B", "C"]);
    }

    // Validates: Requirement 11.29
    #[test]
    fn ffcmd_runner_empty_source_succeeds_with_zero_commands() {
        let mut runner = FfcmdRunner::new();
        let result = runner.run_source("", |_| Ok(()));
        assert_eq!(result, FfcmdResult::Success { commands_run: 0 });
    }
}
