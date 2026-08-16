# Implementation Plan: Logging Subsystem (`ff-logging`)

## Overview

This plan covers the complete implementation of the `ff-logging` crate — the foundational logging subsystem for FileForgeWorkbench. Every other crate in the workspace depends on `ff-logging` for diagnostic output. The subsystem provides structured file-based logging with configurable levels, automatic rotation, async buffering, graceful degradation, and a plugin-accessible logging handle.

This is a **Wave 0 (Foundation)** sub-project with no upstream dependencies.

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-logging/Cargo.toml` with dependencies (chrono, thiserror, crossbeam-channel, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-logging/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `level.rs`, `record.rs`, `format.rs`, `config.rs`, `channel.rs`, `writer.rs`, `rotation.rs`, `sink.rs`, `handle.rs`, `panic_hook.rs`, `init.rs`
  - [x] 1.4 Add `ff-logging` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Log level types and filtering
  - [x] 2.1 Define `LogLevel` enum (Trace, Debug, Info, Warn, Error) with ordering and display
  - [x] 2.2 Implement `FromStr` for `LogLevel` with case-insensitive parsing and whitespace trimming
  - [x] 2.3 Implement level comparison operators for severity filtering
  - [x] 2.4 Write unit tests for level parsing and ordering
  - Covers: Requirement 3 (AC 3.1, 3.2, 3.3, 3.4)

- [x] 3. Log record data type
  - [x] 3.1 Define `LogRecord` struct with fields: timestamp, level, module_path, message
  - [x] 3.2 Implement `LogRecord::new()` constructor that captures current local time with millisecond precision
  - [x] 3.3 Implement message truncation at 8192 bytes with ellipsis marker
  - [x] 3.4 Implement control character escaping (ASCII 0x00–0x1F except LF) to `\u{XXXX}` format
  - [x] 3.5 Write unit tests for record construction, truncation, and escaping
  - Covers: Requirement 2 (AC 2.1, 2.3, 2.4)

- [x] 4. Log record formatting
  - [x] 4.1 Implement `format_record()` producing single-line output: `YYYY-MM-DDTHH:MM:SS.mmm±HH:MM LEVEL [module::path] message\n`
  - [x] 4.2 Implement level name padding to 5 characters (e.g., `INFO `, `WARN `, `ERROR`)
  - [x] 4.3 Ensure LF line ending on all platforms
  - [x] 4.4 Write unit tests for format correctness with all levels
  - Covers: Requirement 2 (AC 2.1, 2.2)

- [x] 5. Log record parsing (for round-trip property verification)
  - [x] 5.1 Implement `parse_record()` that extracts timestamp, level, module_path, and message from a formatted line
  - [x] 5.2 Write unit tests verifying round-trip equivalence
  - Covers: Requirement 2 (AC 2.5)

- [x] 6. Configuration data types
  - [x] 6.1 Define `LogConfig` struct with fields: level, directory, max_file_size_mb, max_retained_files
  - [x] 6.2 Implement `Default` for `LogConfig` (INFO level, platform default directory, 10 MB, 5 files)
  - [x] 6.3 Implement config validation with clamping for `max_file_size_mb` (1–1024) and `max_retained_files` (1–100)
  - [x] 6.4 Implement platform-default directory resolution (Windows: `%LOCALAPPDATA%/FileForgeWorkbench/logs`, Linux/macOS: `$XDG_DATA_HOME/file-forge-workbench/logs`)
  - [x] 6.5 Write unit tests for defaults, clamping, and platform path resolution
  - Covers: Requirement 3 (AC 3.1, 3.3, 3.4), Requirement 4 (AC 4.1, 4.3), Requirement 5 (AC 5.1, 5.2, 5.3, 5.6, 5.7, 5.8)

- [x] 7. Log directory creation and fallback
  - [x] 7.1 Implement `ensure_log_directory()` that creates directory with intermediate parents
  - [x] 7.2 Implement fallback chain: configured path → platform default → no-op sink
  - [x] 7.3 Write unit tests using `tempfile::TempDir` for directory creation success and failure paths
  - Covers: Requirement 1 (AC 1.5, 1.6), Requirement 4 (AC 4.2, 4.4)

- [x] 8. Log file writer
  - [x] 8.1 Implement `LogFileWriter` struct managing the active log file handle
  - [x] 8.2 Implement file naming pattern: `file_forge_workbench_YYYYMMDD_HHMMSS.log`
  - [x] 8.3 Implement buffered write with max 64 KB buffer
  - [x] 8.4 Implement immediate flush on ERROR and WARN level records
  - [x] 8.5 Write unit tests for file creation, naming, buffered writes, and flush behavior
  - Covers: Requirement 4 (AC 4.5), Requirement 6 (AC 6.1, 6.5)

- [x] 9. Log rotation
  - [x] 9.1 Implement size tracking — detect when a write would exceed `max_file_size_mb`
  - [x] 9.2 Implement rotation: write final record, close current file, open new file with fresh timestamp
  - [x] 9.3 Implement rotation failure fallback — continue writing to current file with WARN record
  - [x] 9.4 Implement retained file cleanup — delete oldest files by filename timestamp when count exceeds limit
  - [x] 9.5 Implement cleanup failure handling — log WARN and continue operating
  - [x] 9.6 Write unit tests for rotation trigger, file count enforcement, and failure paths
  - Covers: Requirement 5 (AC 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 5.10)

- [x] 10. Async channel and buffer management
  - [x] 10.1 Implement bounded async channel (capacity: 10,000 records) between producers and writer thread
  - [x] 10.2 Implement overflow handling — drop oldest unwritten records and increment dropped counter
  - [x] 10.3 Implement WARN record emission when overflow resolves (reporting total dropped count)
  - [x] 10.4 Implement thread-safe dropped-record counter (`AtomicU64`) with public accessor method
  - [x] 10.5 Implement periodic flush timer (≤1 second interval) for DEBUG/INFO records
  - [x] 10.6 Write unit tests for channel capacity, overflow behavior, and periodic flush
  - Covers: Requirement 8 (AC 8.1, 8.2, 8.3, 8.4, 8.5), Requirement 6 (AC 6.2)

- [x] 11. No-op sink implementation
  - [x] 11.1 Implement `NoOpSink` that discards all records silently
  - [x] 11.2 Implement diagnostic flag (`AtomicBool`) indicating logging is unavailable
  - [x] 11.3 Implement public method to query diagnostic flag status
  - [x] 11.4 Write unit tests for no-op behavior and flag state
  - Covers: Requirement 1 (AC 1.3, 1.4, 1.6), Requirement 7 (AC 7.7)

- [x] 12. Log subsystem initialization
  - [x] 12.1 Implement `LogSubsystem::init(config: LogConfig)` — full initialization sequence
  - [x] 12.2 Write startup INFO record with app name ("FileForgeWorkbench"), version, and RFC 3339 timestamp
  - [x] 12.3 Implement graceful fallback to no-op sink on initialization failure
  - [x] 12.4 Implement level filtering guard — atomic level check before any formatting/allocation
  - [x] 12.5 Implement invalid config level handling — default to INFO with WARN record
  - [x] 12.6 Write unit tests for successful init, fallback paths, and startup record content
  - Covers: Requirement 1 (AC 1.1, 1.2, 1.3, 1.4, 1.5, 1.6), Requirement 3 (AC 3.2, 3.3, 3.4, 3.5)

- [x] 13. Flush and shutdown logic
  - [x] 13.1 Implement `shutdown()` method — flush all buffered records, write final "Application shutdown complete" INFO record, close file
  - [x] 13.2 Implement shutdown signal that stops accepting new log calls
  - [x] 13.3 Implement 5-second timeout for flush during shutdown
  - [x] 13.4 Write unit tests for clean shutdown sequence and timeout behavior
  - Covers: Requirement 6 (AC 6.3, 6.6), Requirement 8 (AC 8.6)

- [x] 14. Panic hook integration
  - [x] 14.1 Implement custom panic hook that attempts to flush buffered records
  - [x] 14.2 Implement 500 ms timeout for panic flush — abandon and allow termination on timeout or I/O error
  - [x] 14.3 Install panic hook during `LogSubsystem::init()`
  - [x] 14.4 Write unit tests for panic hook flush behavior (using `std::panic::catch_unwind`)
  - Covers: Requirement 6 (AC 6.4)

- [x] 15. Plugin logging handle
  - [x] 15.1 Define `PluginLogHandle` trait with methods for all five log levels
  - [x] 15.2 Implement `PluginLogHandle` struct that auto-prefixes module path with plugin name (`[plugin:<name>::module]`)
  - [x] 15.3 Ensure same level filtering, formatting, rotation, and flushing rules apply to plugin records
  - [x] 15.4 Implement plugin flush-on-unload — flush buffered records before `shutdown` returns
  - [x] 15.5 Ensure `PluginLogHandle` is `Send + Sync` for cross-thread usage
  - [x] 15.6 Write unit tests for prefix formatting, level filtering, and thread-safety
  - Covers: Requirement 10 (AC 10.1, 10.2, 10.3, 10.4, 10.5, 10.6)

- [x] 16. Log macros
  - [x] 16.1 Define convenience macros: `log_trace!`, `log_debug!`, `log_info!`, `log_warn!`, `log_error!`
  - [x] 16.2 Ensure macros check level guard before evaluating format arguments (zero-cost filtering)
  - [x] 16.3 Macros automatically capture `module_path!()` for the source module field
  - [x] 16.4 Write unit tests verifying macros skip formatting when level is filtered out
  - Covers: Requirement 3 (AC 3.5), Requirement 9 (AC 9.5)

- [x] 17. GUI-independent process configuration
  - [x] 17.1 Document `#![windows_subsystem = "windows"]` requirement for the desktop binary (not in ff-logging itself)
  - [x] 17.2 Ensure Log_Subsystem does not write to stdout/stderr — all output goes exclusively to Log_File
  - [x] 17.3 Ensure no `AllocConsole` or child process spawning in logging code
  - [x] 17.4 Write unit tests verifying no stdout/stderr output during logging operations
  - Covers: Requirement 7 (AC 7.1, 7.4, 7.5, 7.6)

- [x] 18. Thread safety and performance validation
  - [x] 18.1 Write multi-threaded stress test — spawn 10+ threads logging concurrently
  - [x] 18.2 Verify no data races using `cargo test` under Miri (if available) or thread sanitizer
  - [x] 18.3 Write test asserting log call returns within 1 ms (non-blocking check)
  - [x] 18.4 Verify `LogSubsystem` and all public types implement `Send + Sync`
  - Covers: Requirement 8 (AC 8.1, 8.2, 8.3)

- [x] 19. Integration API surface for platform-core
  - [x] 19.1 Define public `init`, `shutdown`, `is_logging_available`, `dropped_count` API surface
  - [x] 19.2 Implement subsystem error logging pattern: WARN/ERROR with subsystem name, operation, and error description
  - [x] 19.3 Document integration pattern for command executor (TRACE), file engine (DEBUG), and macro engine (DEBUG/INFO/ERROR)
  - [x] 19.4 Write integration test validating platform-core usage pattern
  - Covers: Requirement 9 (AC 9.1, 9.2, 9.3, 9.4, 9.5)

- [x] 20. Property-based tests
  - [x] 20.1 Write PBT: format round-trip property
  - [x] 20.2 Write PBT: rotation size invariant property
  - [x] 20.3 Write PBT: overflow handling property
  - [x] 20.4 Write PBT: level filtering property
  - [x] 20.5 Write PBT: message truncation property
  - [x] 20.6 Write PBT: control character escaping property
  - Covers: Requirement 2 (AC 2.5), Requirement 5, Requirement 8 (AC 8.4), Requirement 3 (AC 3.2), Requirement 2 (AC 2.3), Requirement 2 (AC 2.4)

---

## Property-Based Test Definitions

### Property 1: Format Round-Trip

**Validates: Requirement 2.5**

- **Statement:** For all valid `LogRecord` values, formatting the record and then parsing the formatted output SHALL produce field values equivalent to the original record.
- **Strategy:** Generate arbitrary `LogRecord` instances with:
  - Timestamps: random local datetimes with millisecond precision
  - Levels: uniform selection from {Trace, Debug, Info, Warn, Error}
  - Module paths: generated strings matching `[a-z_][a-z0-9_:]*` pattern (1–64 chars)
  - Messages: arbitrary UTF-8 strings (0–10000 bytes, including control chars)
- **Invariant:** `parse_record(format_record(record)) == record` (field-by-field equivalence after normalization of truncation and escaping)

### Property 2: Rotation Size Invariant

**Validates: Requirement 5.4**

- **Statement:** For any sequence of log writes, no log file SHALL contain more bytes than `max_file_size_mb` + the size of the final record that triggered rotation.
- **Strategy:** Generate:
  - `max_file_size_mb`: integer in [1, 10]
  - Record sequence: 50–500 records with message sizes in [1, 4096] bytes
- **Invariant:** For every closed log file, `file_size <= max_file_size_mb * 1024 * 1024 + max_single_record_size`

### Property 3: Overflow Handling

**Validates: Requirement 8.4**

- **Statement:** When the internal buffer reaches capacity (10,000 records), the subsystem SHALL drop the oldest records and the total of (written records + dropped records + pending records) SHALL equal the total number of records submitted.
- **Strategy:** Generate:
  - Number of producer threads: [2, 8]
  - Records per thread: [5000, 20000]
  - Writer processing delay: [0, 5] ms per record (simulating slow I/O)
- **Invariant:** `records_written + records_dropped + records_pending == records_submitted`

### Property 4: Level Filtering

**Validates: Requirement 3.2**

- **Statement:** For any configured minimum level and any submitted record, the record SHALL appear in the output if and only if its level is >= the configured minimum.
- **Strategy:** Generate:
  - Configured level: uniform from {Trace, Debug, Info, Warn, Error}
  - Record level: uniform from {Trace, Debug, Info, Warn, Error}
  - 100–500 records with random levels
- **Invariant:** Record is present in output ⟺ `record.level >= configured_level`

### Property 5: Message Truncation

**Validates: Requirement 2.3**

- **Statement:** For any message body, the formatted output message field SHALL have at most 8192 + 3 bytes (the 3 being the "..." ellipsis marker), and messages ≤ 8192 bytes SHALL appear unchanged.
- **Strategy:** Generate:
  - Message bodies: arbitrary byte sequences of length [0, 20000]
- **Invariant:** If `input.len() <= 8192` then `output_message == input`; if `input.len() > 8192` then `output_message == input[..8192] + "..."` and `output_message.len() == 8195`

### Property 6: Control Character Escaping

**Validates: Requirement 2.4**

- **Statement:** For any message body containing control characters, the formatted output SHALL contain no raw control characters (0x00–0x1F) except LF at the record terminator, and all such characters SHALL be replaced with their `\u{XXXX}` representation.
- **Strategy:** Generate:
  - Message bodies: arbitrary byte sequences including control characters, length [1, 1000]
- **Invariant:** The formatted record line (excluding trailing LF) contains no bytes in range 0x00–0x1F, and each original control character maps to exactly one `\u{XXXX}` escape sequence

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types", "tasks": ["2", "3", "6", "11"], "dependsOn": [0] },
    { "id": 2, "label": "Formatting and Config", "tasks": ["4", "5", "7"], "dependsOn": [1] },
    { "id": 3, "label": "File I/O", "tasks": ["8", "9", "10"], "dependsOn": [2] },
    { "id": 4, "label": "Subsystem Assembly", "tasks": ["12", "13", "14", "16"], "dependsOn": [3] },
    { "id": 5, "label": "Integration and Plugins", "tasks": ["15", "17", "19"], "dependsOn": [4] },
    { "id": 6, "label": "Validation and PBT", "tasks": ["18", "20"], "dependsOn": [5] }
  ]
}
```

---

## Notes

- This is a Wave 0 (Foundation) crate with zero upstream dependencies
- All other workspace crates will depend on `ff-logging` — the public API surface must be stable before downstream work begins
- The `configuration-system` crate does not exist yet; `LogConfig` accepts values directly and will be wired to TOML config in a later wave
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- Thread-safety tests should use `std::thread::spawn` with join handles rather than Tokio, since `ff-logging` itself does not depend on an async runtime
- The `#![windows_subsystem = "windows"]` attribute belongs on the desktop binary crate, not on `ff-logging` — Task 17 documents this requirement without implementing it in the logging crate
- Plugin handle trait definition lives in `ff-logging` to avoid circular dependencies; the `plugin-architecture` crate will re-export it

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Initialization | AC 1.1–1.6 | Tasks 7, 11, 12 |
| Req 2: Record Format | AC 2.1–2.5 | Tasks 3, 4, 5, 20 |
| Req 3: Level Config | AC 3.1–3.5 | Tasks 2, 6, 12, 16, 20 |
| Req 4: File Location | AC 4.1–4.5 | Tasks 6, 7, 8 |
| Req 5: Rotation | AC 5.1–5.10 | Tasks 6, 9, 20 |
| Req 6: Flush/Shutdown | AC 6.1–6.5 | Tasks 8, 10, 13, 14 |
| Req 7: GUI-Independent | AC 7.1–7.7 | Tasks 11, 17 |
| Req 8: Thread Safety | AC 8.1–8.6 | Tasks 10, 13, 18, 20 |
| Req 9: Platform Integration | AC 9.1–9.5 | Tasks 16, 19 |
| Req 10: Plugin Integration | AC 10.1–10.6 | Task 15 |
