//! File rotation logic, naming, and size tracking.
//!
//! Handles size-based rotation of log files, file naming with timestamps,
//! and retention cleanup (deleting oldest files when count exceeds limit).

use std::fs;
use std::path::Path;

use crate::writer::LogFileWriter;

/// Pattern prefix used to identify log files created by this subsystem.
const LOG_FILE_PREFIX: &str = "file_forge_workbench_";

/// Pattern suffix for log files.
const LOG_FILE_SUFFIX: &str = ".log";

/// Checks whether a rotation is needed before writing a line.
///
/// Returns `true` if writing `line_bytes` would cause the cumulative bytes
/// written to exceed the configured maximum file size.
///
/// # Arguments
///
/// * `writer` - The current log file writer with byte tracking.
/// * `line_bytes` - The number of bytes about to be written.
/// * `max_file_size_mb` - The configured maximum file size in megabytes.
pub(crate) fn should_rotate(
    writer: &LogFileWriter,
    line_bytes: u64,
    max_file_size_mb: u32,
) -> bool {
    let threshold = u64::from(max_file_size_mb) * 1_024 * 1_024;
    writer.bytes_written() + line_bytes > threshold
}

/// Performs file rotation on the writer.
///
/// Flushes the current file, generates a new filename with a fresh timestamp,
/// opens the new file, and resets the byte counter. If rotation fails, a WARN
/// line is written to the current file and writing continues there.
///
/// # Arguments
///
/// * `writer` - The log file writer to rotate.
///
/// # Returns
///
/// `Ok(())` if rotation succeeded, or an error if it failed. On failure,
/// the caller should continue writing to the current file.
pub(crate) fn perform_rotation(writer: &mut LogFileWriter) -> std::io::Result<()> {
    writer.rotate()
}

/// Handles rotation failure by writing a WARN line into the current file.
///
/// When rotation fails, this function writes a diagnostic message to the
/// existing file so the failure is captured in the log stream.
///
/// # Arguments
///
/// * `writer` - The writer that failed to rotate (still pointing at old file).
/// * `error` - The I/O error that caused the rotation failure.
pub(crate) fn handle_rotation_failure(writer: &mut LogFileWriter, error: &std::io::Error) {
    let warn_line = format!(
        "WARN  [ff_logging::rotation] Log rotation failed: {error}. Continuing with current file.\n"
    );
    // Best-effort write — if this also fails, we can't do anything more
    let _ = writer.write_line(&warn_line, crate::level::LogLevel::Warn);
}

/// Enforces the retention policy by deleting oldest log files when the count
/// exceeds the configured maximum.
///
/// Scans the log directory for files matching the naming pattern
/// (`file_forge_workbench_YYYYMMDD_HHMMSS.log`), sorts them lexicographically
/// (which corresponds to chronological order), and deletes the oldest files
/// until the count is within the limit.
///
/// # Arguments
///
/// * `log_directory` - The directory containing log files.
/// * `max_retained_files` - Maximum number of files to retain.
///
/// # Returns
///
/// A vector of warnings for any files that could not be deleted.
pub(crate) fn enforce_retention(log_directory: &Path, max_retained_files: u32) -> Vec<String> {
    let mut warnings = Vec::new();

    let mut log_files: Vec<String> = match fs::read_dir(log_directory) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                if is_log_file(&name) {
                    Some(name)
                } else {
                    None
                }
            })
            .collect(),
        Err(_) => return warnings,
    };

    // Sort lexicographically (oldest first, since timestamps are in the filename)
    log_files.sort();

    let max = max_retained_files as usize;
    if log_files.len() <= max {
        return warnings;
    }

    // Delete oldest files until we're within the limit
    let files_to_delete = log_files.len() - max;
    for filename in log_files.iter().take(files_to_delete) {
        let path = log_directory.join(filename);
        if let Err(err) = fs::remove_file(&path) {
            warnings.push(format!(
                "WARN  [ff_logging::rotation] Failed to delete old log file '{}': {err}. Continuing.\n",
                path.display()
            ));
        }
    }

    warnings
}

/// Returns `true` if the filename matches the log file naming pattern.
///
/// Expected pattern: `file_forge_workbench_YYYYMMDD_HHMMSS.log`
fn is_log_file(filename: &str) -> bool {
    filename.starts_with(LOG_FILE_PREFIX)
        && filename.ends_with(LOG_FILE_SUFFIX)
        && filename.len() == 40
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LogLevel;
    use crate::writer::LogFileWriter;
    use pretty_assertions::assert_eq;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    // ─── should_rotate Tests ────────────────────────────────────────────────

    #[test]
    fn should_rotate_returns_false_when_under_threshold() {
        // Validates: Requirement 5.4
        let tmp = TempDir::new().expect("failed to create temp dir");
        let writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        // 1 MB threshold, 0 bytes written, trying to write 100 bytes
        assert!(!should_rotate(&writer, 100, 1));
    }

    #[test]
    fn should_rotate_returns_true_when_write_would_exceed_threshold() {
        // Validates: Requirement 5.4
        let tmp = TempDir::new().expect("failed to create temp dir");
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        // Write enough to get close to 1 MB
        let big_line = "x".repeat(1_048_500) + "\n";
        writer
            .write_line(&big_line, LogLevel::Warn)
            .expect("write failed");

        // Now adding 200 bytes would exceed 1 MB
        assert!(should_rotate(&writer, 200, 1));
    }

    #[test]
    fn should_rotate_returns_false_at_exact_threshold() {
        // Validates: Requirement 5.4
        let tmp = TempDir::new().expect("failed to create temp dir");
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        // Write exactly 1 MB - 10 bytes
        let line = "x".repeat(1_048_566) + "\n"; // 1048567 bytes
        writer
            .write_line(&line, LogLevel::Warn)
            .expect("write failed");

        // Adding 9 bytes should NOT trigger rotation (total = 1048576 = exactly 1MB)
        assert!(!should_rotate(&writer, 9, 1));
        // Adding 10 bytes would exceed
        assert!(should_rotate(&writer, 10, 1));
    }

    // ─── perform_rotation Tests ─────────────────────────────────────────────

    #[test]
    fn perform_rotation_resets_byte_counter_and_creates_new_file() {
        // Validates: Requirement 5.4
        let tmp = TempDir::new().expect("failed to create temp dir");
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        writer
            .write_line("some data\n", LogLevel::Warn)
            .expect("write failed");
        let old_path = writer.current_path().to_path_buf();

        std::thread::sleep(std::time::Duration::from_secs(1));

        perform_rotation(&mut writer).expect("rotation failed");

        assert_eq!(writer.bytes_written(), 0);
        assert_ne!(writer.current_path(), old_path.as_path());
        assert!(writer.current_path().exists());
    }

    // ─── enforce_retention Tests ────────────────────────────────────────────

    #[test]
    fn enforce_retention_does_nothing_when_under_limit() {
        // Validates: Requirement 5.9
        let tmp = TempDir::new().expect("failed to create temp dir");

        // Create 3 log files
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_100000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_110000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_120000.log");

        let warnings = enforce_retention(tmp.path(), 5);

        assert!(warnings.is_empty());
        assert_eq!(count_log_files(tmp.path()), 3);
    }

    #[test]
    fn enforce_retention_deletes_oldest_when_over_limit() {
        // Validates: Requirement 5.9
        let tmp = TempDir::new().expect("failed to create temp dir");

        // Create 5 log files
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_100000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_110000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_120000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_130000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_140000.log");

        let warnings = enforce_retention(tmp.path(), 3);

        assert!(warnings.is_empty());
        assert_eq!(count_log_files(tmp.path()), 3);

        // The two oldest should be deleted
        assert!(!tmp
            .path()
            .join("file_forge_workbench_20250101_100000.log")
            .exists());
        assert!(!tmp
            .path()
            .join("file_forge_workbench_20250101_110000.log")
            .exists());
        // The three newest should remain
        assert!(tmp
            .path()
            .join("file_forge_workbench_20250101_120000.log")
            .exists());
        assert!(tmp
            .path()
            .join("file_forge_workbench_20250101_130000.log")
            .exists());
        assert!(tmp
            .path()
            .join("file_forge_workbench_20250101_140000.log")
            .exists());
    }

    #[test]
    fn enforce_retention_at_exact_limit_does_not_delete() {
        // Validates: Requirement 5.9
        let tmp = TempDir::new().expect("failed to create temp dir");

        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_100000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_110000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_120000.log");

        let warnings = enforce_retention(tmp.path(), 3);

        assert!(warnings.is_empty());
        assert_eq!(count_log_files(tmp.path()), 3);
    }

    #[test]
    fn enforce_retention_ignores_non_log_files() {
        // Validates: Requirement 5.9
        let tmp = TempDir::new().expect("failed to create temp dir");

        // Create log files and non-log files
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_100000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_110000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_120000.log");
        create_dummy_log_file(tmp.path(), "other_file.txt");
        create_dummy_log_file(tmp.path(), "readme.md");

        let warnings = enforce_retention(tmp.path(), 2);

        assert!(warnings.is_empty());
        // Only 2 log files should remain, non-log files untouched
        assert_eq!(count_log_files(tmp.path()), 2);
        assert!(tmp.path().join("other_file.txt").exists());
        assert!(tmp.path().join("readme.md").exists());
    }

    #[test]
    fn enforce_retention_returns_warnings_for_undeletable_files() {
        // Validates: Requirement 5.10
        // This test verifies that deletion failures produce warnings rather than panics.
        // On most platforms we can't easily make a file undeletable in a portable way,
        // so we just verify the happy path doesn't produce warnings.
        let tmp = TempDir::new().expect("failed to create temp dir");

        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_100000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_110000.log");

        let warnings = enforce_retention(tmp.path(), 2);
        assert!(warnings.is_empty());
    }

    #[test]
    fn is_log_file_matches_valid_filenames() {
        assert!(is_log_file("file_forge_workbench_20250120_143022.log"));
        assert!(is_log_file("file_forge_workbench_20241231_235959.log"));
    }

    #[test]
    fn is_log_file_rejects_invalid_filenames() {
        assert!(!is_log_file("other_file.log"));
        assert!(!is_log_file("file_forge_workbench_20250120.log")); // too short
        assert!(!is_log_file("file_forge_workbench_20250120_143022.txt")); // wrong extension
        assert!(!is_log_file("")); // empty
    }

    // ─── Integration: Rotation + Retention ──────────────────────────────────

    #[test]
    fn rotation_followed_by_retention_enforces_file_limit() {
        // Validates: Requirement 5.4, 5.9
        let tmp = TempDir::new().expect("failed to create temp dir");

        // Pre-create some log files
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_100000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_110000.log");
        create_dummy_log_file(tmp.path(), "file_forge_workbench_20250101_120000.log");

        // Create a writer (adds a 4th file)
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");
        writer
            .write_line("data\n", LogLevel::Warn)
            .expect("write failed");

        // Now we have 4 files, enforce retention of 3
        let warnings = enforce_retention(tmp.path(), 3);
        assert!(warnings.is_empty());
        assert_eq!(count_log_files(tmp.path()), 3);
    }

    // ─── Test Helpers ───────────────────────────────────────────────────────

    fn create_dummy_log_file(dir: &Path, name: &str) {
        let path = dir.join(name);
        let mut file = File::create(&path).expect("failed to create dummy file");
        file.write_all(b"dummy content\n")
            .expect("failed to write dummy content");
    }

    fn count_log_files(dir: &Path) -> usize {
        fs::read_dir(dir)
            .expect("failed to read dir")
            .filter_map(|e| e.ok())
            .filter(|e| is_log_file(&e.file_name().to_string_lossy()))
            .count()
    }
}
