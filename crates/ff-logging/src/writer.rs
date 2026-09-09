//! Writer thread, buffered I/O, and flush strategies.
//!
//! Manages the dedicated OS thread that consumes log records from the
//! channel and writes them to the file sink via a buffered writer.
//! The writer thread is the ONLY thread that performs file I/O.

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::level::LogLevel;

/// Size of the internal write buffer in bytes (64 KB).
/// Limits potential data loss during abnormal termination.
/// Validates: Requirement 6, criterion 5.
const BUFFER_CAPACITY: usize = 65_536;

/// Manages the active log file handle with buffered writes and level-based flushing.
///
/// The writer is owned exclusively by the dedicated writer thread. It holds a
/// `BufWriter<File>` with a 64 KB buffer, tracks cumulative bytes written for
/// rotation decisions, and flushes immediately for WARN/ERROR records.
pub(crate) struct LogFileWriter {
    /// Buffered writer wrapping the active log file.
    writer: BufWriter<File>,
    /// Path to the currently active log file.
    current_path: PathBuf,
    /// Cumulative bytes written to the current file (for rotation tracking).
    bytes_written: u64,
    /// Directory where log files are stored.
    log_directory: PathBuf,
    /// Flush immediately for this level and above (WARN by default).
    level_for_flush: LogLevel,
}

impl LogFileWriter {
    /// Creates a new `LogFileWriter` by opening a log file in the specified directory.
    ///
    /// The file is created with the standard naming convention
    /// (`file_forge_workbench_YYYYMMDD_HHMMSS.log`) and wrapped in a `BufWriter`
    /// with 64 KB capacity.
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the directory cannot be created or the file
    /// cannot be opened.
    pub(crate) fn new(log_directory: &Path) -> std::io::Result<Self> {
        fs::create_dir_all(log_directory)?;

        let filename = generate_log_filename();
        let path = log_directory.join(&filename);

        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        let writer = BufWriter::with_capacity(BUFFER_CAPACITY, file);

        Ok(Self {
            writer,
            current_path: path,
            bytes_written: 0,
            log_directory: log_directory.to_path_buf(),
            level_for_flush: LogLevel::Warn,
        })
    }

    /// Writes a pre-formatted log line to the buffered writer.
    ///
    /// If the record's level is >= the flush threshold (WARN), the buffer is
    /// flushed immediately after writing. This ensures high-severity records
    /// are persisted to disk without delay.
    ///
    /// Returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the write or flush operation fails.
    pub(crate) fn write_line(&mut self, line: &str, level: LogLevel) -> std::io::Result<usize> {
        let bytes = line.as_bytes();
        self.writer.write_all(bytes)?;
        self.bytes_written += bytes.len() as u64;

        // Flush immediately for WARN and ERROR records
        if level >= self.level_for_flush {
            self.writer.flush()?;
        }

        Ok(bytes.len())
    }

    /// Flushes the internal buffer to disk.
    ///
    /// Called periodically (every 1 second) by the writer thread for
    /// DEBUG/INFO records that haven't triggered an immediate flush.
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the flush fails.
    pub(crate) fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }

    /// Returns the cumulative bytes written to the current log file.
    pub(crate) fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Returns the path to the currently active log file.
    #[allow(dead_code)]
    pub(crate) fn current_path(&self) -> &Path {
        &self.current_path
    }

    /// Returns a reference to the log directory.
    #[allow(dead_code)]
    pub(crate) fn log_directory(&self) -> &Path {
        &self.log_directory
    }

    /// Rotates to a new log file.
    ///
    /// Flushes the current buffer, generates a new filename with a fresh
    /// timestamp, opens the new file, and resets `bytes_written` to zero.
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the new file cannot be created.
    /// On failure, the writer continues using the current file.
    pub(crate) fn rotate(&mut self) -> std::io::Result<()> {
        // Flush current buffer before closing
        self.writer.flush()?;

        let filename = generate_log_filename();
        let new_path = self.log_directory.join(&filename);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&new_path)?;

        self.writer = BufWriter::with_capacity(BUFFER_CAPACITY, file);
        self.current_path = new_path;
        self.bytes_written = 0;

        Ok(())
    }

    /// Switches the writer to a new log directory, opening a fresh log file.
    ///
    /// Flushes the current buffer, ensures the new directory exists (creating
    /// intermediate parents), opens a new log file under it with a fresh
    /// timestamped filename, and updates the writer's directory and byte
    /// counter. Used by runtime reconfiguration (Requirement 11, criterion 2).
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the new directory cannot be created or
    /// the new file cannot be opened. On failure, the writer is left unchanged
    /// and continues using the current file (Requirement 11, criterion 5).
    pub(crate) fn switch_directory(&mut self, new_directory: &Path) -> std::io::Result<()> {
        // Flush current buffer before switching so no buffered records are lost.
        self.writer.flush()?;

        // Create the new directory (and parents) before opening the file.
        fs::create_dir_all(new_directory)?;

        let filename = generate_log_filename();
        let new_path = new_directory.join(&filename);

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&new_path)?;

        self.writer = BufWriter::with_capacity(BUFFER_CAPACITY, file);
        self.current_path = new_path;
        self.log_directory = new_directory.to_path_buf();
        self.bytes_written = 0;

        Ok(())
    }
}

/// Generates a log filename using the current local timestamp.
///
/// Format: `file_forge_workbench_YYYYMMDD_HHMMSS.log`
///
/// # Examples
///
/// The returned filename will look like:
/// ```text
/// file_forge_workbench_20250120_143022.log
/// ```
pub(crate) fn generate_log_filename() -> String {
    let now = chrono::Local::now();
    now.format("file_forge_workbench_%Y%m%d_%H%M%S.log")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use proptest::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    // ─── Filename Generation Tests ──────────────────────────────────────────

    #[test]
    fn generate_log_filename_matches_expected_pattern() {
        // Validates: Requirement 4.5
        let filename = generate_log_filename();
        assert!(
            filename.starts_with("file_forge_workbench_"),
            "Filename should start with 'file_forge_workbench_', got: {filename}"
        );
        assert!(
            filename.ends_with(".log"),
            "Filename should end with '.log', got: {filename}"
        );
        // Total length: "file_forge_workbench_" (21) + "YYYYMMDD_HHMMSS" (15) + ".log" (4) = 40
        assert_eq!(
            filename.len(),
            40,
            "Filename length should be 40, got: {}",
            filename.len()
        );
    }

    #[test]
    fn generate_log_filename_contains_valid_timestamp_digits() {
        // Validates: Requirement 4.5
        let filename = generate_log_filename();
        let timestamp_part = &filename[21..36]; // "YYYYMMDD_HHMMSS"
        assert_eq!(timestamp_part.len(), 15);
        assert_eq!(&timestamp_part[8..9], "_");
        // All other chars should be digits
        for (i, ch) in timestamp_part.chars().enumerate() {
            if i == 8 {
                assert_eq!(ch, '_');
            } else {
                assert!(
                    ch.is_ascii_digit(),
                    "Expected digit at position {i}, got '{ch}'"
                );
            }
        }
    }

    // ─── File Creation Tests ────────────────────────────────────────────────

    #[test]
    fn new_creates_log_file_in_specified_directory() {
        // Validates: Requirement 1.5, 4.2
        let tmp = TempDir::new().expect("failed to create temp dir");
        let log_dir = tmp.path().join("logs");

        let writer = LogFileWriter::new(&log_dir).expect("failed to create writer");

        assert!(
            writer.current_path().exists(),
            "Log file should exist on disk"
        );
        assert!(
            writer.current_path().starts_with(&log_dir),
            "File should be in the log directory"
        );
    }

    #[test]
    fn new_creates_directory_if_it_does_not_exist() {
        // Validates: Requirement 1.5
        let tmp = TempDir::new().expect("failed to create temp dir");
        let log_dir = tmp.path().join("nested").join("logs");

        let _writer = LogFileWriter::new(&log_dir).expect("failed to create writer");

        assert!(log_dir.exists(), "Log directory should be created");
    }

    #[test]
    fn new_initializes_bytes_written_to_zero() {
        // Validates: Requirement 5.4 (size tracking)
        let tmp = TempDir::new().expect("failed to create temp dir");

        let writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        assert_eq!(writer.bytes_written(), 0);
    }

    // ─── Buffered Write Tests ───────────────────────────────────────────────

    #[test]
    fn write_line_tracks_bytes_written_correctly() {
        // Validates: Requirement 5.4 (size tracking for rotation)
        let tmp = TempDir::new().expect("failed to create temp dir");
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        let line = "2025-01-20T14:30:22.456+10:00 INFO  [test] Hello\n";
        let written = writer
            .write_line(line, LogLevel::Info)
            .expect("write failed");

        assert_eq!(written, line.len());
        assert_eq!(writer.bytes_written(), line.len() as u64);
    }

    #[test]
    fn write_line_accumulates_bytes_across_multiple_writes() {
        // Validates: Requirement 5.4 (cumulative size tracking)
        let tmp = TempDir::new().expect("failed to create temp dir");
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        let line1 = "first line\n";
        let line2 = "second line\n";
        writer
            .write_line(line1, LogLevel::Info)
            .expect("write failed");
        writer
            .write_line(line2, LogLevel::Debug)
            .expect("write failed");

        assert_eq!(writer.bytes_written(), (line1.len() + line2.len()) as u64);
    }

    #[test]
    fn write_line_uses_64kb_buffer() {
        // Validates: Requirement 6.5 (64 KB buffer)
        let tmp = TempDir::new().expect("failed to create temp dir");
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        // Write a small line without flushing level — data should be buffered
        let line = "short info line\n";
        writer
            .write_line(line, LogLevel::Info)
            .expect("write failed");

        // Read the file — with buffering, it may be empty unless flushed
        // (We can't easily verify buffer capacity programmatically, but
        // we verify the BufWriter is created with correct capacity by
        // checking that small writes don't immediately appear on disk)
        let content = fs::read_to_string(writer.current_path()).expect("read failed");
        // With a 64KB buffer, a 16-byte write should stay buffered
        assert!(
            content.is_empty(),
            "Small INFO write should be buffered, not yet on disk"
        );
    }

    // ─── Flush Behavior Tests ───────────────────────────────────────────────

    #[test]
    fn write_line_flushes_immediately_for_warn_level() {
        // Validates: Requirement 6.1
        let tmp = TempDir::new().expect("failed to create temp dir");
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        let line = "2025-01-20T14:30:22.456+10:00 WARN  [test] Warning!\n";
        writer
            .write_line(line, LogLevel::Warn)
            .expect("write failed");

        // After WARN write, data should be flushed to disk
        let content = fs::read_to_string(writer.current_path()).expect("read failed");
        assert_eq!(content, line);
    }

    #[test]
    fn write_line_flushes_immediately_for_error_level() {
        // Validates: Requirement 6.1
        let tmp = TempDir::new().expect("failed to create temp dir");
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        let line = "2025-01-20T14:30:22.456+10:00 ERROR [test] Error!\n";
        writer
            .write_line(line, LogLevel::Error)
            .expect("write failed");

        // After ERROR write, data should be flushed to disk
        let content = fs::read_to_string(writer.current_path()).expect("read failed");
        assert_eq!(content, line);
    }

    #[test]
    fn write_line_does_not_flush_for_info_level() {
        // Validates: Requirement 6.1 (flush only for WARN and above)
        let tmp = TempDir::new().expect("failed to create temp dir");
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        let line = "2025-01-20T14:30:22.456+10:00 INFO  [test] Info msg\n";
        writer
            .write_line(line, LogLevel::Info)
            .expect("write failed");

        // INFO should NOT trigger immediate flush — data stays in buffer
        let content = fs::read_to_string(writer.current_path()).expect("read failed");
        assert!(content.is_empty(), "INFO should not trigger flush");
    }

    #[test]
    fn flush_persists_buffered_data_to_disk() {
        // Validates: Requirement 6.2 (periodic flush)
        let tmp = TempDir::new().expect("failed to create temp dir");
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        let line = "buffered info line\n";
        writer
            .write_line(line, LogLevel::Info)
            .expect("write failed");

        // Explicitly flush (simulates periodic flush)
        writer.flush().expect("flush failed");

        let content = fs::read_to_string(writer.current_path()).expect("read failed");
        assert_eq!(content, line);
    }

    // ─── Rotation Tests ─────────────────────────────────────────────────────

    #[test]
    fn rotate_creates_new_file_and_resets_bytes_written() {
        // Validates: Requirement 5.4
        let tmp = TempDir::new().expect("failed to create temp dir");
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        let line = "some data\n";
        writer
            .write_line(line, LogLevel::Warn)
            .expect("write failed");
        assert!(writer.bytes_written() > 0);

        let old_path = writer.current_path().to_path_buf();

        // Sleep briefly to ensure different timestamp in filename
        std::thread::sleep(std::time::Duration::from_secs(1));

        writer.rotate().expect("rotation failed");

        assert_eq!(writer.bytes_written(), 0);
        assert_ne!(writer.current_path(), old_path.as_path());
        assert!(writer.current_path().exists());
        assert!(old_path.exists()); // Old file should still exist
    }

    // ─── Directory Switch Tests (Requirement 11) ────────────────────────────

    #[test]
    fn switch_directory_opens_file_in_new_directory() {
        // Validates: Requirement 11.2
        let tmp = TempDir::new().expect("failed to create temp dir");
        let dir1 = tmp.path().join("dir1");
        let dir2 = tmp.path().join("dir2");

        let mut writer = LogFileWriter::new(&dir1).expect("failed to create writer");
        let old_path = writer.current_path().to_path_buf();

        writer
            .switch_directory(&dir2)
            .expect("switch_directory failed");

        assert!(dir2.exists(), "New directory should be created");
        assert!(
            writer.current_path().starts_with(&dir2),
            "New file should be under the new directory"
        );
        assert_eq!(writer.log_directory(), dir2.as_path());
        assert_eq!(writer.bytes_written(), 0);
        assert!(old_path.exists(), "Old file should still exist");
    }

    #[test]
    fn switch_directory_creates_nested_parents() {
        // Validates: Requirement 11.2
        let tmp = TempDir::new().expect("failed to create temp dir");
        let mut writer = LogFileWriter::new(tmp.path()).expect("failed to create writer");

        let nested = tmp.path().join("a").join("b").join("c");
        writer
            .switch_directory(&nested)
            .expect("switch_directory failed");

        assert!(nested.exists(), "Nested directory chain should be created");
        assert!(writer.current_path().starts_with(&nested));
    }

    #[test]
    fn switch_directory_flushes_current_buffer_before_switching() {
        // Validates: Requirement 11.2 (no buffered records lost across the swap)
        let tmp = TempDir::new().expect("failed to create temp dir");
        let dir1 = tmp.path().join("dir1");
        let dir2 = tmp.path().join("dir2");

        let mut writer = LogFileWriter::new(&dir1).expect("failed to create writer");
        let line = "buffered info before switch\n";
        writer
            .write_line(line, LogLevel::Info)
            .expect("write failed");
        let old_path = writer.current_path().to_path_buf();

        writer
            .switch_directory(&dir2)
            .expect("switch_directory failed");

        // The buffered INFO line must have been flushed to the old file.
        let old_content = fs::read_to_string(&old_path).expect("read old file failed");
        assert_eq!(old_content, line);
    }

    #[test]
    fn switch_directory_failure_leaves_writer_on_current_file() {
        // Validates: Requirement 11.5 (failure retains current file)
        let tmp = TempDir::new().expect("failed to create temp dir");
        let dir1 = tmp.path().join("dir1");
        let mut writer = LogFileWriter::new(&dir1).expect("failed to create writer");
        let old_path = writer.current_path().to_path_buf();

        // Create a regular file, then try to switch into a path *inside* it,
        // which cannot be created as a directory.
        let blocker = tmp.path().join("blocker_file");
        fs::write(&blocker, "x").expect("failed to write blocker");
        let invalid_dir = blocker.join("cannot_create");

        let result = writer.switch_directory(&invalid_dir);
        assert!(result.is_err(), "switch into a file path should fail");

        // Writer must still point at the original file and remain usable.
        assert_eq!(writer.current_path(), old_path.as_path());
        writer
            .write_line("still works\n", LogLevel::Warn)
            .expect("writer should still be usable after failed switch");
    }

    // ─── Property 11: Reconfigure Directory Swap ────────────────────────────

    proptest! {
        // Feature: logging-subsystem, Property 11: Reconfigure Directory Swap.
        // For any records written before a directory switch and any records
        // written after, the pre-switch records remain under the old directory
        // and every post-switch record lands under the new directory. No
        // pre-switch record is lost across the swap.
        // Validates: Requirement 11.2, 11.9
        #[test]
        fn prop_switch_directory_routes_records_to_new_dir(
            before in prop::collection::vec("[a-zA-Z0-9 ]{1,40}", 1..8),
            after in prop::collection::vec("[a-zA-Z0-9 ]{1,40}", 1..8),
        ) {
            let base = TempDir::new().expect("tempdir");
            let dir1 = base.path().join("d1");
            let dir2 = base.path().join("d2");

            let mut writer = LogFileWriter::new(&dir1).expect("writer");
            let old_path = writer.current_path().to_path_buf();

            // Write the "before" records (WARN forces a flush to disk).
            for (i, msg) in before.iter().enumerate() {
                let line = format!("BEFORE-{i}-{msg}\n");
                writer.write_line(&line, LogLevel::Warn).expect("write before");
            }

            writer.switch_directory(&dir2).expect("switch");
            let new_path = writer.current_path().to_path_buf();

            // Write the "after" records.
            for (i, msg) in after.iter().enumerate() {
                let line = format!("AFTER-{i}-{msg}\n");
                writer.write_line(&line, LogLevel::Warn).expect("write after");
            }
            writer.flush().expect("flush");

            let old_content = fs::read_to_string(&old_path).expect("read old");
            let new_content = fs::read_to_string(&new_path).expect("read new");

            // Every "before" record is preserved in the old file (no loss).
            for (i, msg) in before.iter().enumerate() {
                let marker = format!("BEFORE-{i}-{msg}");
                prop_assert!(
                    old_content.contains(&marker),
                    "pre-switch record missing from old file: {marker}"
                );
            }
            // Every "after" record is in the new file, and none in the old file.
            for (i, msg) in after.iter().enumerate() {
                let marker = format!("AFTER-{i}-{msg}");
                prop_assert!(
                    new_content.contains(&marker),
                    "post-switch record missing from new file: {marker}"
                );
                prop_assert!(
                    !old_content.contains(&marker),
                    "post-switch record leaked into old file: {marker}"
                );
            }
            // The two files are distinct and the active dir is the new one.
            prop_assert_ne!(old_path.as_path(), new_path.as_path());
            prop_assert!(new_path.starts_with(&dir2));
        }
    }
}
