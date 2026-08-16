# Requirements Document

## Introduction

This feature specifies the structured file-based logging subsystem for FileForgeWorkbench (`ff-logging` crate). The logging subsystem is a **foundational dependency** — every other crate in the workspace (platform-core, command-framework, plugin-architecture, workflow-engine, document-model, and all plugins) depends on `ff-logging` for diagnostic output. Because the workbench application is built as a GUI-independent platform with a replaceable rendering shell (see Architecture Brief §3 Principle 1), no console or terminal is guaranteed at runtime. All application log output (informational messages, warnings, errors, and debug traces) must be written to a persistent log file.

The logging subsystem supports configurable log levels, structured timestamps, automatic log rotation, and graceful degradation. It integrates with the workbench `platform-core` layer and is accessible to all plugins via the `plugin-architecture` trait without tight coupling — plugins obtain a logging handle through `PluginContext` at initialization time.

The application's process model is also formally specified: the workbench runs as a standalone GUI process without spawning or requiring a terminal window, while remaining fully functional when launched from a command-line shell.

**Source references:**
- **FFE** = FileForgeEditor `file-based-logging` specification (original source)
- **WB** = Workbench Architecture Brief (architectural adaptations)

## Glossary

- **Log_Subsystem**: The `ff-logging` crate responsible for initializing, configuring, and managing all log output within the application, routing log records to the log file sink. [FFE]
- **Log_File**: The on-disk file to which the Log_Subsystem writes formatted log records. [FFE]
- **Log_Record**: A single structured log entry containing a timestamp, severity level, source module path, and message body. [FFE]
- **Log_Level**: A severity classification for log records. The supported levels in ascending severity are: TRACE, DEBUG, INFO, WARN, ERROR. [FFE]
- **Log_Rotation**: The process of closing the current Log_File and creating a new one when the current file exceeds a configured size threshold. [FFE]
- **Log_Directory**: The filesystem directory in which all Log_Files (current and rotated) are stored. [FFE]
- **Application_Process**: The FileForgeWorkbench operating system process, configured to run without an attached console window on desktop platforms. [FFE, WB]
- **Workbench_Config**: The application configuration managed by the `configuration-system` crate (TOML-based), which stores user-adjustable settings including logging parameters. Replaces the monolithic `config/editor.toml` from FileForgeEditor. [WB]
- **Plugin_Context**: The context object passed to plugins during initialization via the `plugin-architecture` trait, through which plugins obtain services including logging handles. [WB]
- **Platform_Core**: The GUI-independent workbench core layer (`ff-core`) that orchestrates all services; logging integrates at this level to be available before any GUI or plugin code executes. [WB]

## Requirements

### Requirement 1: Log Subsystem Initialization

**User Story:** As a developer, I want the logging subsystem to initialize early in the application startup sequence, so that all subsequent operations (including error paths) have a functioning log sink available.

**Source:** FFE Req 1 — adapted for workbench platform-core startup sequence. [FFE, WB]

#### Acceptance Criteria

1. WHEN the application starts, THE Log_Subsystem SHALL initialize and be ready to accept log records before any other subsystem (Platform_Core services, plugin loader, command registry, layout engine, GUI shell) is constructed or invoked.
2. WHEN the Log_Subsystem initializes successfully, THE Log_Subsystem SHALL write an INFO-level record containing the application name ("FileForgeWorkbench"), version, and the current timestamp in RFC 3339 format to the Log_File.
3. IF the Log_Subsystem fails to create or open the Log_File (due to permission errors, invalid path, or disk full), THEN THE Log_Subsystem SHALL fall back to a no-op sink that discards all log records and SHALL NOT cause the application to terminate.
4. IF the Log_Subsystem falls back to a no-op sink, THEN THE Application_Process SHALL set an internal diagnostic flag that any GUI status bar can query to display a warning indicating that logging is unavailable.
5. WHEN the Log_Subsystem initializes, THE Log_Subsystem SHALL create the Log_Directory if it does not already exist, including any intermediate parent directories.
6. IF the Log_Subsystem fails to create the Log_Directory (due to permission errors or a read-only filesystem), THEN THE Log_Subsystem SHALL fall back to a no-op sink and SHALL NOT cause the application to terminate.

---

### Requirement 2: Log Record Format

**User Story:** As a developer, I want log records to follow a consistent structured format with timestamps and severity levels, so that I can efficiently search, filter, and correlate log entries during debugging.

**Source:** FFE Req 2 — unchanged format specification. [FFE]

#### Acceptance Criteria

1. THE Log_Subsystem SHALL format each Log_Record as a single line containing the following fields in order, separated by a single space character: ISO 8601 timestamp with millisecond precision in local time (format `YYYY-MM-DDTHH:MM:SS.mmm±HH:MM`), the Log_Level name in uppercase padded to 5 characters, the source module path enclosed in square brackets, and the message body.
2. THE Log_Subsystem SHALL include a newline character (LF, `0x0A`, on all platforms) at the end of each Log_Record.
3. THE Log_Subsystem SHALL truncate any single message body that exceeds 8192 bytes at the 8192nd byte boundary, appending an ellipsis marker ("...") to indicate truncation.
4. THE Log_Subsystem SHALL escape or replace control characters (ASCII 0x00–0x1F except 0x0A) within the message body with their Unicode escape representation (`\u{XXXX}`) to ensure each record remains on one line.
5. FOR ALL valid Log_Records, parsing the formatted output and extracting the timestamp, level, module path, and message fields SHALL produce values equivalent to the original Log_Record fields (round-trip property).

---

### Requirement 3: Log Level Configuration

**User Story:** As a user, I want to configure the minimum log level, so that I can control the verbosity of log output without recompiling the application.

**Source:** FFE Req 3 — adapted to reference Workbench_Config (configuration-system). [FFE, WB]

#### Acceptance Criteria

1. THE Workbench_Config SHALL support a `logging.level` setting that accepts the values "trace", "debug", "info", "warn", and "error" (case-insensitive, leading and trailing whitespace trimmed before comparison).
2. WHEN the `logging.level` setting is present in Workbench_Config, THE Log_Subsystem SHALL filter out all Log_Records with a severity below the configured level, where severity ordering from lowest to highest is: TRACE < DEBUG < INFO < WARN < ERROR.
3. IF the `logging.level` setting is absent from Workbench_Config, THEN THE Log_Subsystem SHALL default to the INFO level, determined once at initialization time.
4. IF the `logging.level` setting contains an unrecognized value (after trimming and case-insensitive comparison), THEN THE Log_Subsystem SHALL default to the INFO level and write a WARN-level Log_Record indicating the invalid configuration value and the fallback behavior.
5. WHEN a log macro is invoked at a level below the configured minimum, THE Log_Subsystem SHALL not perform string formatting or allocation for the filtered record; the level check guard SHALL be evaluated before any formatting closure is executed.

---

### Requirement 4: Log File Location

**User Story:** As a user, I want to configure where log files are stored, so that I can place them on an appropriate disk or partition for my environment.

**Source:** FFE Req 4 — adapted path defaults and naming for workbench. [FFE, WB]

#### Acceptance Criteria

1. THE Workbench_Config SHALL support a `logging.directory` setting that accepts an absolute path or a path relative to the application's working directory.
2. WHEN the `logging.directory` setting is present and the path is valid and writable, THE Log_Subsystem SHALL write Log_Files to that directory.
3. IF the `logging.directory` setting is absent from Workbench_Config, THEN THE Log_Subsystem SHALL use a platform-appropriate default: on Windows, `%LOCALAPPDATA%/FileForgeWorkbench/logs`; on Linux and macOS, `$XDG_DATA_HOME/file-forge-workbench/logs` (falling back to `~/.local/share/file-forge-workbench/logs` if `XDG_DATA_HOME` is unset).
4. IF the configured `logging.directory` path cannot be created or is not writable, THEN THE Log_Subsystem SHALL attempt the platform default path before falling back to the no-op sink.
5. THE Log_Subsystem SHALL name each Log_File using the pattern `file_forge_workbench_YYYYMMDD_HHMMSS.log`, where the timestamp reflects the moment the file was created in local time.

---

### Requirement 5: Log Rotation

**User Story:** As a user, I want log files to rotate automatically when they grow large, so that disk space consumption remains bounded without manual intervention.

**Source:** FFE Req 5 — adapted config key paths to workbench configuration-system. [FFE, WB]

#### Acceptance Criteria

1. THE Workbench_Config SHALL support a `logging.max_file_size_mb` setting that accepts an integer value specifying the maximum size in megabytes for a single Log_File before rotation occurs.
2. IF the `logging.max_file_size_mb` setting is absent from Workbench_Config, THEN THE Log_Subsystem SHALL default to 10 megabytes.
3. IF the `logging.max_file_size_mb` setting contains a value less than 1 or greater than 1024, THEN THE Log_Subsystem SHALL clamp the value to the nearest bound (1 or 1024) and write a WARN-level Log_Record indicating the adjustment.
4. WHEN a Log_Record write would cause the current Log_File size to exceed the configured maximum, THE Log_Subsystem SHALL first write that record to the current file, then close the current file and create a new Log_File with a fresh timestamp in its filename.
5. IF the Log_Subsystem fails to create a new Log_File during rotation (due to permission errors, disk full, or invalid path), THEN THE Log_Subsystem SHALL continue writing to the current Log_File and write a WARN-level Log_Record indicating the rotation failure reason.
6. THE Workbench_Config SHALL support a `logging.max_retained_files` setting that accepts an integer value specifying the maximum number of Log_Files to retain in the Log_Directory, with valid values ranging from 1 to 100.
7. IF the `logging.max_retained_files` setting is absent from Workbench_Config, THEN THE Log_Subsystem SHALL default to retaining 5 files.
8. IF the `logging.max_retained_files` setting contains a value less than 1 or greater than 100, THEN THE Log_Subsystem SHALL clamp the value to the nearest bound (1 or 100) and write a WARN-level Log_Record indicating the adjustment.
9. WHEN a new Log_File is created due to rotation and the number of existing Log_Files in the Log_Directory exceeds `logging.max_retained_files`, THE Log_Subsystem SHALL delete the oldest Log_Files (determined by the timestamp in their filenames) until the count equals `logging.max_retained_files`.
10. IF the Log_Subsystem fails to delete an old Log_File during rotation cleanup, THEN THE Log_Subsystem SHALL log a WARN-level record indicating the filename and error reason, and SHALL continue operating with the new Log_File.

---

### Requirement 6: Log Flushing and Shutdown

**User Story:** As a developer, I want log records to be flushed to disk reliably, so that diagnostic information is not lost during crashes or unexpected termination.

**Source:** FFE Req 6 — unchanged core semantics. [FFE]

#### Acceptance Criteria

1. WHEN an ERROR-level or WARN-level Log_Record is written, THE Log_Subsystem SHALL flush all buffered Log_Records to the Log_File before returning from the write call.
2. WHILE buffered Log_Records of DEBUG-level or INFO-level exist unflushed, THE Log_Subsystem SHALL flush them to the Log_File at a periodic interval not exceeding 1 second.
3. WHEN the application exits normally, THE Log_Subsystem SHALL flush all buffered records and write a final INFO-level record containing "Application shutdown complete" before closing the Log_File.
4. IF a panic occurs, THEN THE Log_Subsystem SHALL attempt to flush any buffered records within 500 milliseconds using a custom panic hook installed during initialization; IF the flush does not complete within the timeout or encounters an I/O error, THEN THE Log_Subsystem SHALL abandon the flush and allow the process to terminate without blocking.
5. THE Log_Subsystem SHALL use a write buffer no larger than 64 kilobytes to limit potential data loss during abnormal termination.

---

### Requirement 7: GUI-Independent Process Execution

**User Story:** As a user, I want FileForgeWorkbench to run as a standalone GUI window without a console window appearing, so that the application behaves like a native desktop application when launched from the Start Menu, desktop shortcut, or file association.

**Source:** FFE Req 7 — adapted for workbench GUI-independent platform-core architecture. The workbench separates the GUI shell from the core; logging operates at the core level regardless of which GUI shell is active. [FFE, WB]

#### Acceptance Criteria

1. THE Application_Process SHALL be compiled with the `#![windows_subsystem = "windows"]` attribute on the desktop binary (`fileforge-desktop`), causing Windows to suppress automatic console window allocation when the process starts.
2. WHEN the Application_Process is launched from a desktop shortcut, Start Menu entry, or file association, THE Application_Process SHALL display only the workbench GUI window with no visible console or terminal window.
3. WHEN the Application_Process is launched from an existing terminal (cmd.exe, PowerShell, or a Unix shell), THE Application_Process SHALL start successfully and display the workbench GUI window, with the terminal returning to an interactive prompt within 3 seconds of the launch command being issued.
4. WHILE the Application_Process is running without an attached console, THE Log_Subsystem SHALL serve as the exclusive diagnostic output channel, with no log records lost due to the absence of stdout or stderr.
5. IF the Application_Process is launched from a terminal and the user redirects stdout or stderr (e.g., via `> file.txt` or `2>&1`), THEN THE Application_Process SHALL not produce output on those streams since all diagnostic output is routed to the Log_File.
6. THE Application_Process SHALL not spawn child processes, allocate new console windows, or call `AllocConsole` (or platform equivalent) at any point during its lifetime, except when explicitly requested by the shell-command subsystem for user-initiated terminal operations.
7. IF the Log_Subsystem cannot create or write to the Log_File at startup (due to permission errors or disk space exhaustion), THEN THE Application_Process SHALL continue launching the GUI window and display a non-blocking notification in the status bar indicating that logging is unavailable, without terminating the application.

---

### Requirement 8: Thread Safety and Performance

**User Story:** As a developer, I want the logging subsystem to be safe for use from multiple threads without impacting UI responsiveness, so that background tasks (Tokio workers, file I/O, plugin operations) and the render loop can log freely.

**Source:** FFE Req 8 — adapted for workbench async model (Tokio-based background workers per Architecture Brief §9). [FFE, WB]

#### Acceptance Criteria

1. THE Log_Subsystem SHALL be safe to invoke from any thread (including Tokio worker threads) without requiring the caller to acquire an external lock, and without causing data races or panicked threads regardless of the number of concurrent callers.
2. WHILE the Log_Subsystem is writing a Log_Record to the Log_File, THE Log_Subsystem SHALL not block the GUI render thread for more than 1 millisecond per individual log call.
3. THE Log_Subsystem SHALL use an internal buffer or asynchronous channel to decouple log record production from file I/O, ensuring that callers return from log macros without waiting for disk writes to complete and within the 1 millisecond budget defined in criterion 2.
4. IF the internal log buffer reaches capacity (defined as 10,000 pending records), THEN THE Log_Subsystem SHALL drop the oldest unwritten records and increment a dropped-record counter, and SHALL write a single WARN-level record indicating the total number of records dropped in that overflow episode once at least one buffer slot becomes free, where the WARN record itself counts toward buffer capacity.
5. THE Log_Subsystem SHALL expose a thread-safe method to retrieve the current cumulative count of dropped records for diagnostic display in the UI status bar, returning a value of type unsigned integer that is safe to read from any thread without blocking.
6. IF the Log_Subsystem receives a shutdown signal, THEN THE Log_Subsystem SHALL flush all buffered records to the Log_File within 5 seconds before completing shutdown, and SHALL not accept new log calls after the shutdown signal is received.

---

### Requirement 9: Integration with Platform-Core Subsystems

**User Story:** As a developer, I want all platform-core subsystems (commands, file engine, macros, editor sessions, workflows) to use the logging subsystem for diagnostic output, so that there is a single consistent log stream for the entire workbench.

**Source:** FFE Req 9 — adapted for workbench multi-crate architecture and command-framework. [FFE, WB]

#### Acceptance Criteria

1. WHEN any platform-core subsystem encounters a recoverable error, THE subsystem SHALL write a WARN-level or ERROR-level Log_Record containing the subsystem name, the operation that was attempted, and the error description before returning an error to the caller.
2. WHEN the macro engine executes a Lua script, THE macro engine SHALL write a DEBUG-level Log_Record containing the script filename before execution begins, and an INFO-level Log_Record containing the script filename and execution duration in milliseconds after execution completes; IF the script execution fails, THEN THE macro engine SHALL write an ERROR-level Log_Record containing the script filename and the error message.
3. WHEN the file engine opens or processes a file through the virtual-file-system layer, THE file engine SHALL write a DEBUG-level Log_Record containing the resource URI and file size in bytes.
4. WHEN the command executor processes a command via the command-framework, THE command executor SHALL write a TRACE-level Log_Record containing the command ID and parameters.
5. IF a subsystem's log call occurs at a level below the configured minimum, THEN THE Log_Subsystem SHALL skip the call with negligible overhead (no string allocation, no lock acquisition beyond an atomic level check).

---

### Requirement 10: Plugin Integration

**User Story:** As a plugin developer, I want to use the workbench logging subsystem from my plugin without tight coupling to logging implementation details, so that all plugin diagnostic output is unified in the same log stream as the platform-core.

**Source:** NEW — derived from workbench plugin-architecture principle (Architecture Brief §10). [WB]

#### Acceptance Criteria

1. WHEN a plugin is initialized via the `FileForgePlugin::initialize` method, THE Plugin_Context SHALL provide a logging handle that the plugin can use to emit Log_Records without importing or depending on the `ff-logging` crate's internal types directly.
2. THE logging handle provided to plugins SHALL support all five Log_Levels (TRACE, DEBUG, INFO, WARN, ERROR) and SHALL automatically prefix each Log_Record's module path with the plugin's registered name (e.g., `[plugin:my-plugin::module]`).
3. THE Log_Subsystem SHALL apply the same level filtering, formatting, rotation, and flushing rules to Log_Records originating from plugins as it does to records from platform-core subsystems.
4. IF a plugin emits log records at a rate exceeding the internal buffer capacity, THEN THE Log_Subsystem SHALL apply the same overflow handling (drop oldest, increment counter, emit WARN) as specified in Requirement 8 criterion 4, without distinguishing between plugin and core records.
5. WHEN a plugin is unloaded or shut down, THE Log_Subsystem SHALL flush any buffered records from that plugin before the plugin's `shutdown` method returns.
6. THE plugin logging handle SHALL be safe to use from any thread spawned by the plugin, maintaining the same thread-safety guarantees as specified in Requirement 8.
