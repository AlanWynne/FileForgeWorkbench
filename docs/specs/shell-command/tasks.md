# Implementation Plan: Shell Command Subsystem (`ff-shell`)

## Overview

This plan implements the `ff-shell` crate — the operating-system shell integration layer for FileForgeWorkbench. It provides command execution, document capture, stdin piping, interactive terminal sessions (VT100 emulation over PTY), an output panel with scrollback, environment and working directory management, security mode gating, and configuration integration.

The implementation is structured in 16 phases proceeding from crate scaffolding through integration tests. Each phase builds on the prior phase's public API surface.

**Crate path:** `crates/ff-shell`
**Requirements:** `.kiro/specs/shell-command/requirements.md` (18 requirements)
**Design:** `.kiro/specs/shell-command/design.md`

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-shell/Cargo.toml` with all dependencies from design
  - [x] 1.2 Create `crates/ff-shell/src/lib.rs` with module declarations and crate-level docs
  - [x] 1.3 Create `crates/ff-shell/src/error.rs` with `ShellError` enum (all variants from design)
  - [x] 1.4 Create module stub files for all submodules (engine, config, platform, profile, executor/*, capture, pipe, environment, working_dir, process, terminal/*, panel/*, commands)
  - [x] 1.5 Add `ff-shell` to workspace `Cargo.toml` members list
  - [x] 1.6 Verify `cargo check -p ff-shell` compiles with stubs

- [x] 2. Shell resolution and platform detection
  - [x] 2.1 Implement `PlatformDetector` in `src/platform.rs` with Windows default (`cmd.exe`)
  - [x] 2.2 Implement POSIX shell resolution (`$SHELL` → `bash` → `sh` fallback chain)
  - [x] 2.3 Implement `shell.default_shell` config override logic
  - [x] 2.4 Implement shell override parsing (first argument to SHELL as shell name)
  - [x] 2.5 Implement executable existence check (PATH lookup and absolute path validation)
  - [x] 2.6 Return `ShellError::ShellNotFound` when no valid shell is resolved
  - [x] 2.7 Write unit tests for platform detection (mock env vars and PATH)

- [x] 3. Shell profile resolver
  - [x] 3.1 Implement `ProfileResolver` in `src/profile.rs` with profile table lookup
  - [x] 3.2 Implement exact name matching against `[shell.profiles]` keys
  - [x] 3.3 Implement fallback to raw PATH resolution when no profile matches
  - [x] 3.4 Implement profile `args` and `env` extraction
  - [x] 3.5 Write unit tests for profile matching and fallback behaviour

- [x] 4. Configuration provider and hot-reload
  - [x] 4.1 Implement `ShellConfigProvider` in `src/config.rs` with `ff-config` integration
  - [x] 4.2 Implement typed reading of all `shell.*` keys with defaults
  - [x] 4.3 Implement validation: invalid `shell.mode` → warning + fallback to `"prompt"`
  - [x] 4.4 Implement validation: invalid `shell.timeout_seconds` → warning + fallback to 30
  - [x] 4.5 Implement `Reload_Callback` registration for hot-reload support
  - [x] 4.6 Implement unknown key logging (warn and continue)
  - [x] 4.7 Write unit tests for config parsing, defaults, and invalid value handling

- [x] 5. Environment builder
  - [x] 5.1 Implement `EnvironmentBuilder` in `src/environment.rs` with OS env inheritance
  - [x] 5.2 Implement `shell.env` table merging (override on collision)
  - [x] 5.3 Implement Windows variable expansion (`%VAR%` syntax)
  - [x] 5.4 Implement POSIX variable expansion (`$VAR` and `${VAR}` syntax)
  - [x] 5.5 Implement undefined variable substitution (empty string + DEBUG log)
  - [x] 5.6 Implement per-profile env overlay merging
  - [x] 5.7 Write unit tests for env merging and variable expansion

- [x] 6. Working directory resolver
  - [x] 6.1 Implement `WorkingDirResolver` in `src/working_dir.rs`
  - [x] 6.2 Implement `project_root` mode with project-open check
  - [x] 6.3 Implement `project_root` fallback to home directory when no project is open
  - [x] 6.4 Implement `file_directory` mode with active file path extraction
  - [x] 6.5 Implement `file_directory` fallback chain (project root → home)
  - [x] 6.6 Write unit tests for all resolution paths and fallback chains

- [x] 7. Command executor (async process spawning and output streaming)
  - [x] 7.1 Implement `CommandExecutor` in `src/executor/spawn.rs` with `tokio::process::Command`
  - [x] 7.2 Implement async stdout/stderr capture via `tokio::io::BufReader` line streaming
  - [x] 7.3 Implement `OutputCapture` struct with incremental append in `src/executor/output.rs`
  - [x] 7.4 Implement non-blocking execution (spawn on background task, never block GUI thread)
  - [x] 7.5 Implement process exit status collection (exit code, signal, force-kill)
  - [x] 7.6 Implement `ProcessHandle` in `src/process.rs` with state tracking
  - [x] 7.7 Write unit tests for spawn, output capture, and exit status parsing

- [x] 8. Document capture mode
  - [x] 8.1 Implement `CaptureHandler` in `src/capture.rs` with stdout-only collection
  - [x] 8.2 Implement line splitting (LF, CRLF, CR) into individual logical lines
  - [x] 8.3 Implement trailing newline suppression (no extra empty line)
  - [x] 8.4 Implement `CaptureTarget` with After/Before position calculation
  - [x] 8.5 Implement document insertion via `ff-document-model` line insertion API
  - [x] 8.6 Implement content preservation (no trimming or whitespace modification)
  - [x] 8.7 Implement non-zero exit code rejection (no insertion + retain target marker)
  - [x] 8.8 Implement partial output rejection on I/O error (no insertion)
  - [x] 8.9 Implement stderr routing to Output Panel (not into document)
  - [x] 8.10 Implement `CaptureResult` with lines_inserted count and undo_record
  - [x] 8.11 Write unit tests for line splitting, position calculation, and error cases

- [x] 9. Undo support for document capture
  - [x] 9.1 Implement `CaptureUndoRecord` implementing `ff-command::UndoRecord` trait
  - [x] 9.2 Store insertion position, line count, and command metadata in undo record
  - [x] 9.3 Implement undo action: remove inserted lines, restore document state
  - [x] 9.4 Return `CommandResult::OkUndoable` from capture handler
  - [x] 9.5 Write unit tests for undo/redo round-trip integrity

- [x] 10. Stdin piping from document
  - [x] 10.1 Implement `StdinPiper` in `src/pipe.rs` with document content delivery
  - [x] 10.2 Implement selection-only piping (pipe selected text when selection is active)
  - [x] 10.3 Implement stdin handle closure after content write (EOF signal)
  - [x] 10.4 Implement combined pipe + capture mode (stdin pipe with A/B target insertion)
  - [x] 10.5 Implement pipe without target (output to Output Panel)
  - [x] 10.6 Implement empty document piping (immediate EOF)
  - [x] 10.7 Write unit tests for stdin delivery and EOF signalling

- [x] 11. Terminal emulator (VT100 state machine)
  - [x] 11.1 Implement `Cell`, `CellAttributes`, and `TerminalColor` in `src/terminal/cell.rs`
  - [x] 11.2 Implement `TerminalGrid` in `src/terminal/grid.rs` with row-major storage
  - [x] 11.3 Implement grid operations: clear, scroll up, scroll down, line insert/delete
  - [x] 11.4 Implement `TerminalEmulator` in `src/terminal/emulator.rs` with parser state machine
  - [x] 11.5 Implement ANSI escape parsing: CSI sequences (cursor movement, erase, SGR)
  - [x] 11.6 Implement SGR attribute parsing (bold, italic, underline, foreground/background colors)
  - [x] 11.7 Implement cursor positioning (CUP, CUF, CUB, CUU, CUD, home, save/restore)
  - [x] 11.8 Implement screen clearing (ED — erase display, EL — erase line)
  - [x] 11.9 Implement scrollback buffer management (push lines above visible area)
  - [x] 11.10 Implement `feed()` method: byte stream → parsed sequences → grid state updates
  - [x] 11.11 Implement `resize()` method: reflow content to new dimensions
  - [x] 11.12 Write unit tests for escape sequence parsing and grid state transitions

- [x] 12. PTY abstraction and terminal manager
  - [x] 12.1 Define `PtyHandle` trait in `src/terminal/pty.rs`
  - [x] 12.2 Implement `UnixPty` for Linux/macOS using `nix` crate (`openpty`/`forkpty`)
  - [x] 12.3 Implement `WindowsConPty` for Windows using `windows-rs` ConPTY API
  - [x] 12.4 Implement PTY resize support on both platforms
  - [x] 12.5 Implement `TerminalManager` in `src/terminal/manager.rs` with session lifecycle
  - [x] 12.6 Implement session creation: spawn shell in PTY with resolved env/cwd
  - [x] 12.7 Implement session close: terminate PTY child, clean up resources
  - [x] 12.8 Implement `write_input()`: route keyboard bytes to active session PTY
  - [x] 12.9 Implement `poll_output()`: read PTY output, feed into emulator
  - [x] 12.10 Implement multiple concurrent sessions (tab model)
  - [x] 12.11 Write unit tests for session lifecycle and PTY mocking

- [x] 13. Output panel (scrollback and history)
  - [x] 13.1 Implement `OutputPanel` in `src/panel/output_panel.rs` with `DockablePanel` trait
  - [x] 13.2 Implement `OutputEntry` creation with command text, working directory, and timestamp header
  - [x] 13.3 Implement scrollback buffer with configurable maximum (`output_buffer_lines`)
  - [x] 13.4 Implement overflow trimming (discard oldest entries when limit exceeded)
  - [x] 13.5 Implement incremental line append (streaming output as it arrives)
  - [x] 13.6 Implement exit code display in entry header (success/error indicator)
  - [x] 13.7 Implement `clear()` action (command ID: `"shell.output.clear"`)
  - [x] 13.8 Implement file reference parsing (`<path>:<line>` and `<path>(<line>)` patterns)
  - [x] 13.9 Implement text selection and copy-to-clipboard via `clipboard-operations`
  - [x] 13.10 Implement separator line between command entries (command + timestamp)
  - [x] 13.11 Register panel with `ff-layout` PanelRegistry (panel ID: `"shell.output"`, zone: Bottom)
  - [x] 13.12 Write unit tests for scrollback management, overflow, and file reference parsing

- [x] 14. Terminal panel (tabbed sessions)
  - [x] 14.1 Implement `TerminalPanel` in `src/panel/terminal_panel.rs` with `DockablePanel` trait
  - [x] 14.2 Implement tabbed session display (one tab per active terminal session)
  - [x] 14.3 Implement keyboard focus capture: all input routed to active session when focused
  - [x] 14.4 Implement focus return: restore focus to previous editor panel on session close
  - [x] 14.5 Implement working directory display in panel title/status area
  - [x] 14.6 Implement shell profile selection for new terminal tabs
  - [x] 14.7 Register panel with `ff-layout` PanelRegistry (panel ID: `"shell.terminal"`, zone: Bottom)
  - [x] 14.8 Write unit tests for tab management and focus routing

- [x] 15. Cancellation, timeout, and signal delivery
  - [x] 15.1 Implement `TimeoutGuard` in `src/executor/timeout.rs` with configurable deadline
  - [x] 15.2 Implement timeout trigger: terminate process when `timeout_seconds` elapses
  - [x] 15.3 Implement signal delivery in `src/executor/signal.rs` (SIGTERM on POSIX, TerminateProcess on Windows)
  - [x] 15.4 Implement escalation sequence: SIGTERM → wait 5s → SIGKILL/force-kill
  - [x] 15.5 Implement `CancellationToken` integration with `ff-workflow` for user-triggered cancel
  - [x] 15.6 Implement cancel action: user triggers cancel → token signalled → process terminated
  - [x] 15.7 Implement timeout exclusion for interactive terminal sessions
  - [x] 15.8 Implement timeout disabled when `timeout_seconds` is 0
  - [x] 15.9 Implement cancellation message display in Output Panel
  - [x] 15.10 Implement document capture abort on cancellation (no partial output insertion)
  - [x] 15.11 Write unit tests for timeout triggering, escalation, and cancellation flow

- [x] 16. Security mode gate
  - [x] 16.1 Implement `check_security_gate()` in `ShellEngine` with mode dispatch
  - [x] 16.2 Implement `Disabled` mode: refuse all invocations with informative error
  - [x] 16.3 Implement `Prompt` mode: delegate confirmation dialog to UI layer
  - [x] 16.4 Implement `Enabled` mode: execute without prompting
  - [x] 16.5 Implement default fallback: absent config → `"prompt"`
  - [x] 16.6 Implement macro dual-gate: require BOTH `shell.mode` AND macro security to permit
  - [x] 16.7 Implement macro rejection when mode is `"prompt"` (cannot show UI prompt from macro)
  - [x] 16.8 Write unit tests for all mode/source combinations

- [x] 17. Command registration and shell engine
  - [x] 17.1 Implement `ShellEngine` struct in `src/engine.rs` as central coordinator
  - [x] 17.2 Implement command form validation (Requirements 9.1–9.6)
  - [x] 17.3 Implement mode routing: no args → terminal, args + no target → execute, args + A/B → capture
  - [x] 17.4 Implement invalid form rejection (source line commands, no args + target, multiple targets)
  - [x] 17.5 Implement `register_commands()` in `src/commands.rs` — register `shell.execute`, `shell.terminal`, `shell.capture`, `shell.output.clear`
  - [x] 17.6 Implement `CommandHandler` trait for each command ID with metadata
  - [x] 17.7 Implement TSO alias normalisation to canonical `"shell.execute"` command ID
  - [x] 17.8 Implement progress indicator emission via `ff-workflow::ProgressReporter`
  - [x] 17.9 Implement sequential execution queue (commands run sequentially by default)
  - [x] 17.10 Wire all components together: security gate → resolution → executor → output
  - [x] 17.11 Write unit tests for command form validation and mode routing

- [x] 18. Property-based tests
  - [x] 18.1 Write property test: Shell resolution always produces valid path or error (Property 1)
  - [x] 18.2 Write property test: Document capture preserves line content exactly (Property 2)
  - [x] 18.3 Write property test: Capture undo restores document state (Property 3)
  - [x] 18.4 Write property test: Environment builder produces superset of OS environment (Property 4)
  - [x] 18.5 Write property test: Environment variable expansion is idempotent on literals (Property 5)
  - [x] 18.6 Write property test: Output panel scrollback respects maximum size (Property 6)
  - [x] 18.7 Write property test: Shell mode gate consistency (Property 7)
  - [x] 18.8 Write property test: Command form validation is complete (Property 8)
  - [x] 18.9 Write property test: Timeout guard terminates within bounded time (Property 9)
  - [x] 18.10 Write property test: Working directory resolution is deterministic (Property 10)
  - [x] 18.11 Write property test: Terminal emulator grid dimensions are invariant (Property 11)
  - [x] 18.12 Write property test: Profile resolution falls back to PATH lookup (Property 12)
  - [x] 18.13 Write property test: Configuration hot-reload convergence (Property 13)
  - [x] 18.14 Write property test: Exit code is always reported for completed commands (Property 14)

- [x] 19. Integration tests
  - [x] 19.1 Write integration test: end-to-end command execution with real shell (echo, exit codes)
  - [x] 19.2 Write integration test: document capture with actual command output and undo verification
  - [x] 19.3 Write integration test: stdin piping from document content to command
  - [x] 19.4 Write integration test: terminal session lifecycle (open, write, read, close)
  - [x] 19.5 Write integration test: configuration hot-reload with in-memory config changes
  - [x] 19.6 Write integration test: command timeout and process termination
  - [x] 19.7 Write integration test: cancellation flow with long-running command
  - [x] 19.8 Write integration test: multiple sequential command execution with output panel history
  - [x] 19.9 Write integration test: shell profile resolution and override
  - [x] 19.10 Write integration test: working directory modes (project root vs file directory)

---

## Property-Based Test Definitions

| ID | Property | Strategy | Validates |
|----|----------|----------|-----------|
| P1 | Shell resolution always produces valid path or error | Generate random platform configs (with/without `$SHELL`, with/without `default_shell`, with/without override). Assert result is either `Ok(non-empty path)` or `Err(ShellNotFound)`. | Req 3.1–3.6 |
| P2 | Document capture preserves line content exactly | Generate arbitrary strings with mixed line endings (LF, CRLF, CR, trailing). Split and assert: each segment preserved verbatim, no extra empty trailing line. | Req 5.4, 5.5, 5.6 |
| P3 | Capture undo restores document state | Generate random document content + random command output. Perform capture at random position, then undo. Assert document is byte-for-byte identical to pre-capture state. | Req 6.1, 6.2 |
| P4 | Environment builder produces superset of OS environment | Generate random OS env map + random `shell.env` table. Build environment. Assert all OS keys present (unless overridden), all `shell.env` keys present. | Req 12.1, 12.2, 12.3 |
| P5 | Environment variable expansion is idempotent on literals | Generate strings with no `$`/`%` characters. Assert expansion output == input. Generate strings with only defined refs. Assert output contains no unexpanded `$`/`%` refs. | Req 12.4, 12.5 |
| P6 | Output panel scrollback respects maximum size | Generate sequences of N entries (N > max). Append all to panel. Assert `line_count() <= output_buffer_lines`. Assert oldest entries discarded. | Req 15.2, 15.3 |
| P7 | Shell mode gate consistency | Generate all (ShellMode, is_macro) combinations. Assert: Disabled→refuse, Enabled→permit, Prompt+direct→permit, Prompt+macro→refuse. | Req 2.1–2.4, 2.7 |
| P8 | Command form validation is complete | Generate all boolean tuples (has_args, has_A, has_B, has_source_cmd). Assert exactly one mode assigned or exactly one error returned. No unclassified input. | Req 9.1–9.6 |
| P9 | Timeout guard terminates within bounded time | Generate random `timeout_seconds` (1–120). Spawn a sleeping process. Assert termination occurs within `timeout + 5s + 1s` tolerance. Assert timeout does not fire for sessions. | Req 18.1, 18.2, 18.4 |
| P10 | Working directory resolution is deterministic | Generate all (mode, project_open, file_has_path) combinations with random paths. Assert result is always a non-empty valid path. Assert fallback chain is correct. | Req 11.1–11.4 |
| P11 | Terminal emulator grid dimensions are invariant | Generate random byte sequences (including escape sequences). Feed to emulator. Assert `grid.cols == initial_cols && grid.rows == initial_rows` after every feed. | Req 7.8 |
| P12 | Profile resolution falls back to PATH lookup | Generate random override strings and profile tables. Assert: matching name → profile path, non-matching name → raw executable lookup. Case-sensitive matching. | Req 16.1–16.3 |
| P13 | Configuration hot-reload convergence | Generate sequences of config change events (valid and invalid values). Apply via reload callback. Assert `get()` reflects latest valid values; invalid values use defaults with warning. | Req 10.1, 10.3, 10.5, 10.6 |
| P14 | Exit code is always reported for completed commands | Generate process completion events (normal exit, signal, timeout, cancel, force-kill). Assert `ExitStatus` always has either `code`, `signal`, or `force_killed=true`. Never all-None. | Req 17.1–17.5 |

---

## Acceptance Criteria Coverage Map

| Requirement | Criteria | Covered By Tasks |
|-------------|----------|-----------------|
| 1 (Command Recognition) | 1.1–1.5 | 17.5, 17.6, 17.7 |
| 2 (Security Mode) | 2.1–2.7 | 16.1–16.8, 18.7 |
| 3 (Platform Detection) | 3.1–3.6 | 2.1–2.7, 18.1 |
| 4 (Command Execution) | 4.1–4.7 | 7.1–7.7, 13.1–13.12, 15.1–15.11 |
| 5 (Document Capture) | 5.1–5.10 | 8.1–8.11, 18.2 |
| 6 (Undo Support) | 6.1–6.3 | 9.1–9.5, 18.3 |
| 7 (Interactive Terminal) | 7.1–7.8 | 11.1–11.12, 12.1–12.11, 14.1–14.8, 18.11 |
| 8 (Error Handling) | 8.1–8.5 | 1.3, 7.7, 8.7, 8.8 |
| 9 (Compatibility Matrix) | 9.1–9.6 | 17.2–17.4, 18.8 |
| 10 (Configuration) | 10.1–10.6 | 4.1–4.7, 18.13 |
| 11 (Working Directory) | 11.1–11.5 | 6.1–6.6, 14.5, 18.10 |
| 12 (Environment Variables) | 12.1–12.5 | 5.1–5.7, 18.4, 18.5 |
| 13 (Async + Cancellation) | 13.1–13.7 | 7.4, 15.1–15.11, 17.8–17.9, 18.9 |
| 14 (Stdin Piping) | 14.1–14.6 | 10.1–10.7 |
| 15 (Output Panel) | 15.1–15.7 | 13.1–13.12, 18.6 |
| 16 (Shell Profiles) | 16.1–16.6 | 3.1–3.5, 14.6, 18.12 |
| 17 (Exit Code Reporting) | 17.1–17.5 | 7.5, 13.6, 18.14 |
| 18 (Command Timeout) | 18.1–18.5 | 15.1–15.2, 15.7–15.8, 18.9 |

---

## Task Dependency Graph

```json
{
  "waves": [
    {
      "id": 1,
      "label": "Crate Scaffolding",
      "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6"],
      "dependsOn": []
    },
    {
      "id": 2,
      "label": "Shell Resolution and Platform Detection",
      "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7"],
      "dependsOn": [1]
    },
    {
      "id": 3,
      "label": "Shell Profile Resolver",
      "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5"],
      "dependsOn": [1]
    },
    {
      "id": 4,
      "label": "Configuration Provider",
      "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7"],
      "dependsOn": [1]
    },
    {
      "id": 5,
      "label": "Environment Builder",
      "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7"],
      "dependsOn": [4]
    },
    {
      "id": 6,
      "label": "Working Directory Resolver",
      "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6"],
      "dependsOn": [4]
    },
    {
      "id": 7,
      "label": "Command Executor",
      "tasks": ["7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7"],
      "dependsOn": [2, 3, 5, 6]
    },
    {
      "id": 8,
      "label": "Document Capture Mode",
      "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "8.8", "8.9", "8.10", "8.11"],
      "dependsOn": [7]
    },
    {
      "id": 9,
      "label": "Undo Support",
      "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5"],
      "dependsOn": [8]
    },
    {
      "id": 10,
      "label": "Stdin Piping",
      "tasks": ["10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7"],
      "dependsOn": [7, 8]
    },
    {
      "id": 11,
      "label": "Terminal Emulator (VT100)",
      "tasks": ["11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "11.7", "11.8", "11.9", "11.10", "11.11", "11.12"],
      "dependsOn": [1]
    },
    {
      "id": 12,
      "label": "PTY Abstraction and Terminal Manager",
      "tasks": ["12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7", "12.8", "12.9", "12.10", "12.11"],
      "dependsOn": [5, 6, 11]
    },
    {
      "id": 13,
      "label": "Output Panel",
      "tasks": ["13.1", "13.2", "13.3", "13.4", "13.5", "13.6", "13.7", "13.8", "13.9", "13.10", "13.11", "13.12"],
      "dependsOn": [7]
    },
    {
      "id": 14,
      "label": "Terminal Panel",
      "tasks": ["14.1", "14.2", "14.3", "14.4", "14.5", "14.6", "14.7", "14.8"],
      "dependsOn": [12]
    },
    {
      "id": 15,
      "label": "Cancellation, Timeout, and Signals",
      "tasks": ["15.1", "15.2", "15.3", "15.4", "15.5", "15.6", "15.7", "15.8", "15.9", "15.10", "15.11"],
      "dependsOn": [7, 13]
    },
    {
      "id": 16,
      "label": "Security Mode Gate",
      "tasks": ["16.1", "16.2", "16.3", "16.4", "16.5", "16.6", "16.7", "16.8"],
      "dependsOn": [4]
    },
    {
      "id": 17,
      "label": "Command Registration and Shell Engine",
      "tasks": ["17.1", "17.2", "17.3", "17.4", "17.5", "17.6", "17.7", "17.8", "17.9", "17.10", "17.11"],
      "dependsOn": [7, 8, 9, 10, 12, 13, 14, 15, 16]
    },
    {
      "id": 18,
      "label": "Property-Based Tests",
      "tasks": ["18.1", "18.2", "18.3", "18.4", "18.5", "18.6", "18.7", "18.8", "18.9", "18.10", "18.11", "18.12", "18.13", "18.14"],
      "dependsOn": [17]
    },
    {
      "id": 19,
      "label": "Integration Tests",
      "tasks": ["19.1", "19.2", "19.3", "19.4", "19.5", "19.6", "19.7", "19.8", "19.9", "19.10"],
      "dependsOn": [17]
    }
  ]
}
```

---

## Notes

- Phases 2, 3, 4, and 11 are independent of each other and can be implemented in parallel after Phase 1.
- The terminal emulator (Phase 11) is the most complex single component; it is isolated from process management and can be developed/tested independently.
- Platform-specific code (PTY implementations) uses conditional compilation (`#[cfg(unix)]` / `#[cfg(windows)]`).
- All property tests use the `proptest` crate with minimum 100 iterations and committed regression files.
- Integration tests spawn real shell processes (platform-dependent); CI must support process spawning.
- The security gate (Phase 16) is intentionally separated from the engine assembly (Phase 17) to allow isolated testing of the authorization logic.
- The Output Panel and Terminal Panel are both `DockablePanel` implementations but serve different purposes: Output Panel is append-only scrollback; Terminal Panel is interactive with full VT emulation.
