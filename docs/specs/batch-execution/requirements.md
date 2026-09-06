# Requirements Document -- Batch Command Execution

## Introduction

This sub-project specifies the Batch Command Execution subsystem for
FileForgeWorkbench. It provides a headless, non-interactive execution mode
analogous to the z/OS IKJEFT01 Terminal Monitor Program running in batch.

On z/OS, IKJEFT01 is the TSO Terminal Monitor Program. When invoked from a
JCL job step (PGM=IKJEFT01), it reads TSO commands from the SYSTSIN DD
statement and executes them sequentially without a terminal. ISPF sessions
run inside IKJEFT01 interactively; the same program drives batch automation.
FFWB provides an equivalent capability: a command file (analogous to SYSTSIN)
is supplied on the CLI or via stdin, and FFWB executes each command through
the same ff-command-semantics pipeline used interactively, then exits.

This enables:
- Scripted file operations (allocate, edit, save, submit) from CI/CD pipelines
- JCL-style job preparation and submission without opening a GUI
- Automated regression testing of FFWB command behaviour
- Integration with external schedulers and build systems

The subsystem spans:
- `ff-desktop` -- CLI entry point, batch runner orchestration
- `ff-command-semantics` -- command parsing and dispatch (unchanged)
- `ff-shell` -- output capture and routing (extended)
- `ff-workflow` -- optional sequencing for multi-step operations

**Source references:**
- **IKJEFT01** = z/OS TSO Terminal Monitor Program batch execution model
- **SYSTSIN** = z/OS DD name for TSO command input in batch
- **SYSTSPRT** = z/OS DD name for TSO command output in batch
- **WB** = Workbench Architecture Brief -- command-driven execution model

---

## Glossary

- **Batch_Mode**: A headless execution mode in which FFWB reads commands from
  a Batch_Input_Source, executes them through the command pipeline, writes
  output to a Batch_Output_Sink, and exits without opening a GUI window.
- **Batch_Input_Source**: The source of commands for batch execution. Either a
  named file path supplied via `--batch <file>` or standard input (`-`) via
  `--batch -`.
- **Batch_Output_Sink**: The destination for command output in batch mode.
  Either standard output (default) or a named file supplied via
  `--batch-output <file>`.
- **Batch_Command**: A single line from the Batch_Input_Source that is
  submitted to the command pipeline as a primary command. Blank lines and
  comment lines (beginning with `*` or `/*`) are skipped.
- **Batch_Runner**: The ff-desktop component that reads the Batch_Input_Source,
  submits each Batch_Command to the command pipeline, collects output, and
  manages the exit code.
- **Batch_Return_Code**: The integer process exit code returned by FFWB after
  batch execution. Follows z/OS conventions: 0 = success, 4 = warning,
  8 = error, 12 = severe error, 16 = catastrophic failure.
- **SYSTSIN_File**: Informal name for the Batch_Input_Source file, by analogy
  with the z/OS SYSTSIN DD statement.
- **SYSTSPRT_File**: Informal name for the Batch_Output_Sink file, by analogy
  with the z/OS SYSTSPRT DD statement.
- **Command_Echo**: The optional reproduction of each Batch_Command in the
  Batch_Output_Sink before its output, analogous to ISPF command echo.
- **Batch_Session**: The logical session context created for a batch run.
  Provides the same catalog registry, config, and session state as an
  interactive session, but with no GUI.
- **Step_Return_Code**: The return code produced by a single Batch_Command
  execution. The Batch_Return_Code is the maximum Step_Return_Code across all
  commands in the run.
- **Abort_On_Error**: A batch execution policy that stops processing further
  commands when any command produces a Step_Return_Code >= a configured
  threshold. Analogous to ISPF CONTROL ERRORS CANCEL.

---

## Requirements

### Requirement 1: Batch Mode CLI Entry Point

**User Story:** As a developer or operator, I want to invoke FFWB with a
command file and have it execute non-interactively, so that I can automate
FFWB operations from scripts, CI/CD pipelines, and schedulers without
opening a GUI.

**Source:** IKJEFT01 batch execution model. [IKJEFT01, WB]

#### Acceptance Criteria

1. WHEN `ffwb --batch <file>` is supplied on the command line, THE
   Batch_Runner SHALL execute in Batch_Mode, reading commands from the
   specified file, and SHALL NOT open any GUI window.
2. WHEN `ffwb --batch -` is supplied, THE Batch_Runner SHALL read commands
   from standard input (stdin) until EOF.
3. WHEN `--batch` is supplied alongside file path arguments (e.g.
   `ffwb --batch cmds.txt file.txt`), THE Batch_Runner SHALL reject the
   invocation with an error message stating that `--batch` is incompatible
   with file path arguments.
4. WHEN `--batch` is not supplied, THE workbench SHALL start in normal
   interactive GUI mode; batch mode SHALL NOT be activated implicitly.
5. THE `--batch` flag SHALL be documented in `ffwb --help` output with a
   one-line description and usage example.
6. WHEN the specified batch file does not exist or cannot be opened, THE
   Batch_Runner SHALL print an error message to stderr and exit with
   Batch_Return_Code 12.

---

### Requirement 2: Batch Input Format

**User Story:** As a user authoring a batch command file, I want a simple,
readable format for specifying commands, so that I can write and maintain
batch scripts without learning a new language.

**Source:** IKJEFT01 SYSTSIN format conventions. [IKJEFT01]

#### Acceptance Criteria

1. THE Batch_Input_Source SHALL be a plain text file encoded in UTF-8 (with
   or without BOM) or the platform default encoding; THE Batch_Runner SHALL
   detect and strip a UTF-8 BOM if present.
2. EACH non-blank, non-comment line in the Batch_Input_Source SHALL be
   treated as one Batch_Command and submitted to the command pipeline in
   order.
3. WHEN a line begins with `*` (asterisk) as the first non-whitespace
   character, THE Batch_Runner SHALL treat it as a comment and skip it.
4. WHEN a line begins with `/*` (slash-asterisk), THE Batch_Runner SHALL
   treat it as a comment and skip it (JCL-style comment compatibility).
5. WHEN a line is blank or contains only whitespace, THE Batch_Runner SHALL
   skip it without producing output or incrementing the command counter.
6. THE Batch_Runner SHALL support command continuation: WHEN a line ends
   with a single `-` (hyphen) as the last non-whitespace character, THE
   following line SHALL be appended to the current command (with the `-`
   removed and a single space inserted) before submission.
7. THE maximum supported line length SHALL be 32767 characters; lines
   exceeding this limit SHALL be truncated with a warning written to the
   Batch_Output_Sink.

---

### Requirement 3: Command Execution Pipeline

**User Story:** As a user, I want batch commands to be executed through the
same command pipeline as interactive commands, so that every command that
works interactively also works in batch.

**Source:** WB command-driven execution model. [WB]

#### Acceptance Criteria

1. EACH Batch_Command SHALL be submitted to the ff-command-semantics
   pipeline identically to a command entered in the interactive
   `Command ===>` field; no separate batch-only parser SHALL be used.
2. THE Batch_Runner SHALL create a Batch_Session that provides the same
   catalog registry, configuration layers, and session state as an
   interactive session, so that commands such as EDIT, FIND, SORT, SAVE,
   ALLOCATE, and SUBMIT behave identically in batch.
3. WHEN a Batch_Command produces output (e.g. LISTCAT, FIND results, error
   messages), THE Batch_Runner SHALL write that output to the
   Batch_Output_Sink.
4. WHEN a Batch_Command modifies a document (e.g. EDIT + CHANGE + SAVE),
   THE modification SHALL be applied to the real filesystem; batch mode
   SHALL NOT operate in a dry-run or sandbox mode unless explicitly
   configured.
5. WHEN a Batch_Command requires a GUI interaction (e.g. a modal dialog
   that cannot be answered from the command line), THE Batch_Runner SHALL
   treat the command as failed with Step_Return_Code 8 and write a
   diagnostic message to the Batch_Output_Sink explaining that the command
   requires interactive input.
6. THE Batch_Runner SHALL execute commands sequentially; concurrent
   execution of Batch_Commands is not supported.

---

### Requirement 4: Output Capture and Routing

**User Story:** As a user, I want batch output written to a file or stdout
so that I can capture, inspect, and process the results of batch runs.

**Source:** IKJEFT01 SYSTSPRT model. [IKJEFT01, WB]

#### Acceptance Criteria

1. BY DEFAULT, THE Batch_Runner SHALL write all command output to standard
   output (stdout).
2. WHEN `--batch-output <file>` is supplied, THE Batch_Runner SHALL write
   all command output to the specified file, creating it if it does not
   exist and overwriting it if it does.
3. WHEN `--batch-output-append <file>` is supplied, THE Batch_Runner SHALL
   append all command output to the specified file.
4. WHEN `--batch-echo` is supplied, THE Batch_Runner SHALL prefix each
   command's output block with a line showing the command text, formatted
   as: `===> <command text>`.
5. WHEN `--batch-echo` is not supplied, command text SHALL NOT appear in
   the output; only command results are written.
6. ALL error messages and diagnostic output from the Batch_Runner itself
   (not from commands) SHALL be written to standard error (stderr),
   separate from command output.
7. WHEN the Batch_Output_Sink file cannot be created or written, THE
   Batch_Runner SHALL print an error to stderr and exit with
   Batch_Return_Code 12 before executing any commands.

---

### Requirement 5: Return Codes

**User Story:** As a CI/CD pipeline operator, I want FFWB to exit with a
meaningful return code after a batch run, so that my pipeline can detect
failures and take appropriate action.

**Source:** z/OS return code conventions. [IKJEFT01]

#### Acceptance Criteria

1. WHEN all Batch_Commands complete with Step_Return_Code 0, THE
   Batch_Runner SHALL exit with Batch_Return_Code 0.
2. THE Batch_Return_Code SHALL be the maximum Step_Return_Code across all
   executed commands.
3. THE following Step_Return_Code values SHALL be used:
   - 0 = command completed successfully
   - 4 = command completed with warnings (e.g. FIND found no matches)
   - 8 = command completed with errors (e.g. file not found, syntax error)
   - 12 = command failed with a severe error (e.g. I/O failure, permission denied)
   - 16 = command caused a catastrophic failure (e.g. unrecoverable internal error)
4. WHEN the Batch_Runner itself fails to initialise (e.g. cannot read the
   input file, cannot create the output file), THE exit code SHALL be 12
   regardless of any commands executed.
5. THE Batch_Return_Code SHALL be written to stderr as a final summary line
   in the format: `FFWB BATCH RETURN CODE: <N>` before the process exits.

---

### Requirement 6: Abort-on-Error Policy

**User Story:** As a batch script author, I want to control whether
processing continues after a command error, so that I can choose between
fail-fast and best-effort execution strategies.

**Source:** ISPF CONTROL ERRORS CANCEL / NOCANCEL model. [IKJEFT01]

#### Acceptance Criteria

1. BY DEFAULT, THE Batch_Runner SHALL continue executing subsequent
   commands after a command failure (best-effort mode).
2. WHEN `--batch-abort-on-error <threshold>` is supplied (where threshold
   is 4, 8, 12, or 16), THE Batch_Runner SHALL stop executing further
   commands when any Step_Return_Code is >= the threshold.
3. WHEN execution is aborted due to Abort_On_Error, THE Batch_Runner SHALL
   write a message to the Batch_Output_Sink identifying the command that
   triggered the abort and the Step_Return_Code.
4. WHEN execution is aborted, THE Batch_Return_Code SHALL reflect the
   Step_Return_Code of the aborting command (not 0).
5. THE `CONTROL ERRORS CANCEL` and `CONTROL ERRORS NOCANCEL` Batch_Commands
   SHALL be recognised within the command stream as inline overrides of the
   abort policy, taking effect for all subsequent commands in the run.

---

### Requirement 7: Batch Session Initialisation

**User Story:** As a batch script author, I want the batch session to have
access to the same catalogs and configuration as my interactive session, so
that my scripts can reference the same datasets and settings.

**Source:** WB session model. [WB]

#### Acceptance Criteria

1. WHEN Batch_Mode starts, THE Batch_Runner SHALL load the same
   configuration layers (system, user, workspace) as an interactive
   session, using the same config file paths.
2. WHEN Batch_Mode starts, THE Batch_Runner SHALL load the catalog registry
   from the same `catalogs.toml` file used by the interactive session.
3. WHEN `--batch-profile <profile-name>` is supplied, THE Batch_Runner
   SHALL load the named configuration profile in addition to the default
   layers, allowing batch-specific settings (e.g. different default
   catalog, different timeout).
4. THE Batch_Session SHALL NOT restore open tabs, window geometry, or other
   GUI-specific session state; only catalog registry and configuration are
   loaded.
5. WHEN Batch_Mode completes, THE Batch_Runner SHALL NOT overwrite the
   interactive session state file; batch runs are non-destructive to the
   user's interactive session.
6. WHEN `--batch-no-catalog` is supplied, THE Batch_Runner SHALL start with
   an empty catalog registry, ignoring `catalogs.toml`; this is useful for
   isolated test runs.

---

### Requirement 8: Dry-Run Mode

**User Story:** As a batch script author, I want to validate my command
file without making any changes, so that I can check for syntax errors and
missing resources before running for real.

**Source:** Common batch tooling convention. [WB]

#### Acceptance Criteria

1. WHEN `--batch-dry-run` is supplied, THE Batch_Runner SHALL parse and
   validate each Batch_Command but SHALL NOT execute any command that
   modifies the filesystem, catalog, or any external resource.
2. IN dry-run mode, THE Batch_Runner SHALL write a line to the
   Batch_Output_Sink for each command indicating whether it would succeed
   or fail, formatted as: `[DRY-RUN] <command> -> <OK|ERROR: reason>`.
3. IN dry-run mode, THE Batch_Return_Code SHALL reflect validation results:
   0 if all commands are valid, 8 if any command has a syntax error or
   references a missing resource.
4. Read-only commands (e.g. LISTCAT, FIND, LOCATE) SHALL execute normally
   in dry-run mode and produce their real output.

---

### Requirement 9: Compatibility with FFCMD

**User Story:** As a Lua macro author, I want batch command files to use
the same format as FFCMD files, so that I can reuse scripts between the
macro engine and the batch runner.

**Source:** lua-macro-engine Requirement 11.29 (FFCMD). [WB]

#### Acceptance Criteria

1. THE Batch_Input_Source format SHALL be identical to the `.ffcmd` file
   format defined in lua-macro-engine Requirement 11.29: plain text, one
   command per line, `*` and `/*` comments, `-` continuation.
2. A `.ffcmd` file SHALL be directly usable as a `--batch` input without
   modification.
3. THE Batch_Runner SHALL recognise the `.ffcmd` file extension and apply
   the FFCMD format rules automatically; other extensions are also accepted
   with the same rules.
4. WHEN a `.ffcmd` file is executed via `--batch`, THE Batch_Runner SHALL
   NOT invoke the Lua macro engine; commands are executed directly through
   the command pipeline.

---

### Requirement 10: Logging and Diagnostics

**User Story:** As a batch operator, I want a structured log of the batch
run so that I can diagnose failures after the fact.

**Source:** WB logging subsystem. [WB]

#### Acceptance Criteria

1. WHEN Batch_Mode runs, THE Batch_Runner SHALL write a structured log to
   the standard FFWB log file (as configured by ff-logging), including:
   batch start time, input file path, each command submitted, each
   Step_Return_Code, and the final Batch_Return_Code.
2. WHEN `--batch-log <file>` is supplied, THE Batch_Runner SHALL write the
   structured log to the specified file instead of the default log location.
3. THE log SHALL include the wall-clock duration of each command execution.
4. WHEN a command produces a Step_Return_Code >= 8, THE Batch_Runner SHALL
   log the full error detail at ERROR level.
5. THE log format SHALL be the same structured format used by ff-logging
   for all other FFWB subsystems (timestamp, level, component, message).

---

## Cross-References

| Dependency | Relationship |
|------------|-------------|
| `command-semantics` | Batch_Commands parsed and dispatched through the same pipeline as interactive commands. Requirement 9 (TSO commands) and Requirement 10 (P2 commands) apply in batch. |
| `shell-command` | Output capture from commands routed to Batch_Output_Sink via the same Output_Panel mechanism, adapted for headless use. |
| `workflow-engine` | Multi-step batch operations (e.g. allocate + edit + save + submit) may be modelled as workflows for progress reporting and error recovery. |
| `lua-macro-engine` | FFCMD file format (Requirement 11.29) is the canonical batch input format; the two subsystems share the same file format but use different execution engines. |
| `startup-and-session` | Batch_Session loads config and catalog registry from the same paths as an interactive session (Requirements 7.1, 7.2). |
| `dataset-catalog` | Batch commands that allocate, list, or open datasets use the same CatalogRegistry API as the interactive session. |
| `logging-subsystem` | Batch run log written via ff-logging (Requirement 10). |
| `configuration-system` | Batch profile loading (Requirement 7.3) uses the ff-config layered system. |

---

## Notes

- The IKJEFT01 analogy is intentional and guides the design: SYSTSIN = Batch_Input_Source,
  SYSTSPRT = Batch_Output_Sink, MAXCC = Batch_Return_Code.
- The batch runner does NOT implement a full TSO session; it uses the FFWB command
  pipeline. TSO-specific commands (ALLOCATE, SUBMIT, etc.) are available because
  they are registered in ff-command-semantics (Requirements 9 and 10 of that spec).
- GUI-requiring commands (modal dialogs) fail gracefully in batch (Requirement 3.5)
  rather than hanging or crashing.
- The dry-run mode (Requirement 8) is specifically designed to support the FFTest
  automated testing framework's headless validation use case.
- Batch mode and interactive mode are mutually exclusive; the same binary serves both.
