use std::path::{Path, PathBuf};

use crate::channel::{ChannelMessage, FormattedRecord};
use crate::level::LogLevel;
use crate::rotation::{
    enforce_retention, handle_rotation_failure, perform_rotation, should_rotate,
};
use crate::writer::LogFileWriter;

// === Writer Thread Loop =====================================================

/// The main loop of the dedicated writer thread.
///
/// Continuously reads messages from the channel receiver and processes them:
/// - `Record`: checks rotation, performs rotation if needed, writes the line
/// - `Flush`: flushes the writer buffer to disk
/// - `Shutdown`: drains remaining records, flushes, and exits
/// - Timeout (Err): flushes buffer periodically (every ~1 second)
pub(super) fn writer_thread_loop(
    mut writer: LogFileWriter,
    receiver: crate::channel::LogReceiver,
    initial_max_file_size_mb: u32,
    initial_max_retained_files: u32,
    initial_log_directory: &Path,
) {
    // Rotation settings and the active directory are mutable so a Reconfigure
    // message can update them at runtime (Requirement 11).
    let mut max_file_size_mb = initial_max_file_size_mb;
    let mut max_retained_files = initial_max_retained_files;
    let mut log_directory = initial_log_directory.to_path_buf();

    loop {
        match receiver.recv_timeout() {
            Ok(ChannelMessage::Record(record)) => {
                handle_record(
                    &mut writer,
                    &record,
                    max_file_size_mb,
                    max_retained_files,
                    &log_directory,
                );
            }
            Ok(ChannelMessage::Flush) => {
                let _ = writer.flush();
            }
            Ok(ChannelMessage::Reconfigure(request)) => {
                handle_reconfigure(
                    &mut writer,
                    request,
                    &mut max_file_size_mb,
                    &mut max_retained_files,
                    &mut log_directory,
                );
            }
            Ok(ChannelMessage::Shutdown) => {
                // Drain all remaining records from the channel
                let remaining = receiver.drain();
                for msg in remaining {
                    match msg {
                        ChannelMessage::Record(record) => {
                            handle_record(
                                &mut writer,
                                &record,
                                max_file_size_mb,
                                max_retained_files,
                                &log_directory,
                            );
                        }
                        ChannelMessage::Flush => {
                            let _ = writer.flush();
                        }
                        ChannelMessage::Reconfigure(request) => {
                            handle_reconfigure(
                                &mut writer,
                                request,
                                &mut max_file_size_mb,
                                &mut max_retained_files,
                                &mut log_directory,
                            );
                        }
                        ChannelMessage::Shutdown => {
                            // Ignore duplicate shutdown signals
                        }
                    }
                }
                // Final flush before exit
                let _ = writer.flush();
                break;
            }
            Err(()) => {
                // Timeout -- periodic flush for buffered DEBUG/INFO records
                let _ = writer.flush();
            }
        }
    }
}

/// Applies a reconfiguration request on the writer thread.
///
/// Updates the rotation settings, and if the request carries a new directory
/// that differs from the current one, switches the writer to a fresh file
/// under it. On a successful directory switch, writes an INFO record to the
/// new file (Requirement 11.8). On failure, retains the current file and
/// writes a WARN record (Requirement 11.5); no buffered records are lost.
fn handle_reconfigure(
    writer: &mut LogFileWriter,
    request: crate::channel::ReconfigureRequest,
    max_file_size_mb: &mut u32,
    max_retained_files: &mut u32,
    log_directory: &mut PathBuf,
) {
    // Rotation settings always apply.
    *max_file_size_mb = request.max_file_size_mb;
    *max_retained_files = request.max_retained_files;

    // Directory change only when a new, different directory is requested.
    if let Some(new_dir) = request.directory {
        if new_dir != *log_directory {
            match writer.switch_directory(&new_dir) {
                Ok(()) => {
                    *log_directory = new_dir.clone();
                    let info_line = format!(
                        "INFO  [ff_logging::init] Logging reconfigured; directory = {}\n",
                        new_dir.display()
                    );
                    let _ = writer.write_line(&info_line, LogLevel::Info);
                }
                Err(err) => {
                    let warn_line = format!(
                        "WARN  [ff_logging::init] Logging reconfigure failed to switch to '{}': {}. Retaining current directory '{}'.\n",
                        new_dir.display(),
                        err,
                        log_directory.display()
                    );
                    let _ = writer.write_line(&warn_line, LogLevel::Warn);
                }
            }
        }
    }
}

/// Handles a single record: checks rotation, performs it if needed, then writes.
fn handle_record(
    writer: &mut LogFileWriter,
    record: &FormattedRecord,
    max_file_size_mb: u32,
    max_retained_files: u32,
    log_directory: &Path,
) {
    let line_bytes = record.line.len() as u64;

    // Check if rotation is needed before writing
    if should_rotate(writer, line_bytes, max_file_size_mb) {
        match perform_rotation(writer) {
            Ok(()) => {
                // Enforce retention policy after successful rotation
                let warnings = enforce_retention(log_directory, max_retained_files);
                for warn_line in warnings {
                    let _ = writer.write_line(&warn_line, LogLevel::Warn);
                }
            }
            Err(err) => {
                handle_rotation_failure(writer, &err);
            }
        }
    }

    // Write the record line
    let _ = writer.write_line(&record.line, record.level);
}
