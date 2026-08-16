# Implementation Plan: Lua Macro Engine (`ff-macro`)

## Overview

This plan covers the complete implementation of the `ff-macro` crate — the scripting and automation layer for FileForgeWorkbench. The macro engine embeds a Lua 5.4 runtime (via `mlua`), exposes a rich editor API for buffer manipulation, provides a comprehensive event hook system, manages per-buffer Lua state, discovers and auto-reloads macro scripts, registers MACRO/EXEC/RUN commands, enforces security modes, and supports script debugging.

This is a **Wave 10 (Extensions and Macros)** sub-project. It depends on `ff-command` (scripting bridge, command registration), `ff-document` (buffer access), `ff-undo` (macro transactions), `ff-config` (configuration), `ff-logging` (diagnostics), and `ff-plugin` (plugin lifecycle).

---

## Tasks

- [ ] 1. Crate scaffolding and module structure
  - [ ] 1.1 Create `crates/ff-macro/Cargo.toml` with dependencies (mlua with lua54+send features, ff-command, ff-document, ff-undo, ff-config, ff-logging, ff-plugin, thiserror, notify, serde, proptest dev-dep)
  - [ ] 1.2 Create `crates/ff-macro/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `engine.rs`, `runtime.rs`, `editor_api.rs`, `hooks.rs`, `hook_registry.rs`, `buffer_state.rs`, `discovery.rs`, `auto_reload.rs`, `commands.rs`, `security.rs`, `config.rs`, `transaction.rs`, `debug.rs`, `error.rs`
  - [ ] 1.4 Add `ff-macro` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [ ] 2. Error types and configuration model
  - [ ] 2.1 Define `MacroError` enum with variants: RuntimeError, ScriptNotFound, SecurityDenied, InstructionLimitExceeded, MemoryLimitExceeded, RollbackFailed, ReloadError, InvalidLineNumber, ConfigError
  - [ ] 2.2 Implement `Display` and `thiserror::Error` derives with descriptive messages including context (script name, line number, limits)
  - [ ] 2.3 Define `MacroConfig` struct with fields: security_mode, directories, auto_reload, startup_script, debug_traceback, instruction_limit, memory_limit_mb, trusted_paths, auto_load_for extensions
  - [ ] 2.4 Implement `MacroConfig::from_config(ff_config)` loading all `macro.*` configuration keys with defaults
  - [ ] 2.5 Write unit tests for error display formatting and config loading with defaults
  - Covers: Requirement 7 (AC 7.1, 7.7), Requirement 1 (AC 1.3, 1.4)

- [ ] 3. Security mode gate
  - [ ] 3.1 Define `SecurityMode` enum with variants: Disabled, Prompt, TrustedOnly, Enabled
  - [ ] 3.2 Implement `SecurityGate` struct that evaluates whether a script path is permitted to execute under the current security mode
  - [ ] 3.3 Implement `Disabled` mode: reject all execution with error message "Macro execution is disabled by security policy."
  - [ ] 3.4 Implement `Prompt` mode: return a `SecurityDecision::NeedsPrompt` with options AllowOnce, AlwaysTrust, Deny
  - [ ] 3.5 Implement `TrustedOnly` mode: allow only scripts in `macro.trusted_paths` or user-level Macro_Directories
  - [ ] 3.6 Implement `Enabled` mode: allow all scripts without restriction
  - [ ] 3.7 Implement restricted stdlib policy: block `os.execute`, `io.popen`, `loadfile` (arbitrary paths), `dofile` (arbitrary paths) for non-trusted scripts regardless of mode
  - [ ] 3.8 Write unit tests for each security mode decision path and restricted function blocking
  - Covers: Requirement 7 (AC 7.1, 7.2, 7.3, 7.4, 7.5, 7.6)

- [ ] 4. Lua runtime initialization
  - [ ] 4.1 Implement `LuaRuntimeBuilder` that creates an `mlua::Lua` instance with Lua 5.4 and configurable standard library loading
  - [ ] 4.2 Implement standard library filtering: always load `base`, `string`, `table`, `math`, `utf8`, `coroutine`; conditionally load `io`, `os`, `debug` only when SecurityMode is `Enabled`
  - [ ] 4.3 Implement configurable instruction count limit (default 10M) using mlua's hook mechanism
  - [ ] 4.4 Implement configurable memory limit (default 64 MB) using mlua's memory limit API
  - [ ] 4.5 Implement limit violation handling: terminate script, generate ScriptError with violation details, signal transaction rollback
  - [ ] 4.6 Implement single-instance lifecycle: runtime created once at engine startup, reused across invocations, global state preserved
  - [ ] 4.7 Write unit tests for runtime creation, stdlib availability by mode, instruction limit enforcement, memory limit enforcement
  - Covers: Requirement 1 (AC 1.1, 1.2, 1.3, 1.4, 1.5, 1.6)

- [ ] 5. LuaMacroEngine core struct and plugin registration
  - [ ] 5.1 Define `LuaMacroEngine` struct owning the Lua runtime, hook registry, buffer state map, config, and security gate
  - [ ] 5.2 Implement `LuaMacroEngine::new(config: MacroConfig) -> Result<Self, MacroError>` performing full initialization
  - [ ] 5.3 Implement `MacroCapability` trait for plugin registration via `ff-plugin`
  - [ ] 5.4 Implement plugin lifecycle methods: `on_activate()`, `on_deactivate()`, `on_shutdown()` managing engine state
  - [ ] 5.5 Write unit tests for engine construction, plugin capability registration, and lifecycle transitions
  - Covers: Requirement 1 (AC 1.6, 1.7)

- [ ] 6. Editor API — buffer content functions
  - [ ] 6.1 Register Lua global table `editor` in the runtime with all required API functions
  - [ ] 6.2 Implement `editor.lines()` returning total line count as Lua integer
  - [ ] 6.3 Implement `editor.get_line(n)` returning 1-based line content as Lua string (excluding terminator)
  - [ ] 6.4 Implement `editor.set_line(n, text)` replacing line content and recording change in Macro_Transaction
  - [ ] 6.5 Implement `editor.insert_line(n, text)` inserting before line n, shifting subsequent lines, recording in transaction
  - [ ] 6.6 Implement `editor.delete_line(n)` removing line n, shifting subsequent lines up, recording in transaction
  - [ ] 6.7 Implement `editor.tag(n)` setting the tagged metadata flag on line n
  - [ ] 6.8 Implement out-of-range line number validation: raise Lua error with descriptive message including invalid index and valid range
  - [ ] 6.9 Write unit tests for each buffer function with valid inputs and boundary/error cases
  - Covers: Requirement 2 (AC 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.11)

- [ ] 7. Editor API — command dispatch and state queries
  - [ ] 7.1 Implement `editor.command(str)` dispatching through the command framework's Scripting_Bridge, returning boolean success
  - [ ] 7.2 Implement `editor.cursor_line()` returning 1-based cursor line number
  - [ ] 7.3 Implement `editor.cursor_col()` returning 1-based cursor column
  - [ ] 7.4 Implement `editor.selection()` returning start_line, start_col, end_line, end_col or nil if no selection
  - [ ] 7.5 Implement `editor.language()` returning detected language name as string
  - [ ] 7.6 Implement `editor.file_path()` returning absolute path of active buffer or nil for untitled
  - [ ] 7.7 Implement `editor.config(key)` reading configuration value via ff-config and returning appropriate Lua type
  - [ ] 7.8 Write unit tests for command dispatch integration, state query return values, and nil cases
  - Covers: Requirement 2 (AC 2.8, 2.9, 2.10)

- [ ] 8. Event hook registry
  - [ ] 8.1 Define `HookRegistry` struct with internal storage mapping event names to ordered handler lists
  - [ ] 8.2 Define `HookEvent` enum with variants: OnOpen, OnBeforeSave, OnAfterSave, OnClose, OnSwitchBuffer, OnChar, OnKey, OnCommand, OnError
  - [ ] 8.3 Define `HookHandler` struct containing: handler function reference, source script path, registration order, cancellable flag
  - [ ] 8.4 Implement `register(event: HookEvent, handler: HookHandler)` adding handler in script-load order
  - [ ] 8.5 Implement `unregister_by_script(script_path: &Path)` removing all handlers from a specific script
  - [ ] 8.6 Implement `handlers_for(event: &HookEvent) -> &[HookHandler]` returning ordered handler list
  - [ ] 8.7 Write unit tests for registration, ordering, per-script unregistration, and empty event queries
  - Covers: Requirement 3 (AC 3.2, 3.3)

- [ ] 9. Event hook discovery and invocation
  - [ ] 9.1 Implement automatic discovery of global Lua functions matching hook names after script load (e.g., `function OnChar(ch)` → register as OnChar handler)
  - [ ] 9.2 Implement hook invocation loop: invoke all handlers in load order, short-circuit on `false` for cancellable hooks
  - [ ] 9.3 Implement `OnBeforeSave` as cancellable: returning false cancels save and displays status bar message identifying the cancelling macro
  - [ ] 9.4 Implement `OnCommand` as cancellable: returning false prevents command execution
  - [ ] 9.5 Implement `OnKey` as cancellable: returning false suppresses the default key action
  - [ ] 9.6 Implement `OnChar` as non-cancellable: fires after character insertion, provides single-char string
  - [ ] 9.7 Implement `OnOpen(file_path)` firing after file is fully loaded into buffer
  - [ ] 9.8 Implement `OnClose(file_path)` firing before buffer is discarded
  - [ ] 9.9 Implement `OnSwitchBuffer(file_path)` firing on tab/buffer switch with new buffer path
  - [ ] 9.10 Implement `OnAfterSave(file_path)` firing after successful save
  - [ ] 9.11 Implement `OnError(error_message)` firing after any Script_Error occurs
  - [ ] 9.12 Write unit tests for hook discovery, cancellable vs non-cancellable behaviour, multi-handler ordering, and error hook cascading
  - Covers: Requirement 3 (AC 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 3.10, 3.11)

- [ ] 10. Per-buffer state management
  - [ ] 10.1 Implement `BufferStateMap` struct maintaining a `HashMap<BufferId, mlua::Table>` of per-buffer Lua tables
  - [ ] 10.2 Implement creation of fresh empty Lua table when a new buffer is opened
  - [ ] 10.3 Implement buffer switch: save departing buffer's `buffer` global, restore arriving buffer's table as the global `buffer`
  - [ ] 10.4 Implement buffer close: discard associated table and release memory
  - [ ] 10.5 Implement nil `buffer` global during engine startup (before any buffer is active), ensuring scripts accessing it receive nil without crash
  - [ ] 10.6 Implement timing guarantee: new buffer's table is active before `OnSwitchBuffer` handlers execute
  - [ ] 10.7 Implement preservation of buffer tables across script auto-reloads
  - [ ] 10.8 Write unit tests for table creation, switch lifecycle, close cleanup, nil-at-startup, and reload preservation
  - Covers: Requirement 4 (AC 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7)

- [ ] 11. Macro transaction integration
  - [ ] 11.1 Implement `MacroTransaction` struct wrapping ff-undo transaction lifecycle for a single macro invocation
  - [ ] 11.2 Implement automatic transaction begin when MACRO/EXEC/RUN execution starts
  - [ ] 11.3 Implement automatic transaction commit on successful completion (all changes become one undo unit)
  - [ ] 11.4 Implement automatic transaction rollback on Lua runtime error (all partial changes reverted)
  - [ ] 11.5 Implement rollback failure handling: log ERROR-level diagnostic, display user warning about manual undo
  - [ ] 11.6 Write unit tests for transaction commit on success, rollback on error, and rollback failure path
  - Covers: Requirement 5 (AC 5.4), Requirement 6 (AC 6.1, 6.4, 6.7)

- [ ] 12. MACRO, EXEC, and RUN command handlers
  - [ ] 12.1 Implement `MacroCommandHandler` for `MACRO <name>`: locate named `.lua` file in configured directories, load and execute
  - [ ] 12.2 Implement `ExecCommandHandler` for `EXEC <lua_expression>`: evaluate inline Lua expression via dostring, display return value in status bar
  - [ ] 12.3 Implement `RunCommandHandler` for `RUN <path>`: load and execute .lua file at absolute or workspace-relative path
  - [ ] 12.4 Implement macro name resolution: search directories in priority order, match by filename without extension
  - [ ] 12.5 Implement error reporting for `MACRO`: "Macro not found: <name>" when name doesn't resolve
  - [ ] 12.6 Implement error reporting for `RUN`: "Cannot open macro file: <path>" when path is invalid
  - [ ] 12.7 Register commands with framework: `"macro.run_named"`, `"macro.exec_inline"`, `"macro.run_file"` — invocable from shortcuts, menus, and macros
  - [ ] 12.8 Write unit tests for name resolution, inline execution, path execution, error messages, and command registration
  - Covers: Requirement 5 (AC 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7)

- [ ] 13. Macro error handling and rollback
  - [ ] 13.1 Implement Lua runtime error capture: catch all errors from macro execution without panic or crash
  - [ ] 13.2 Implement error message formatting: include macro name/expression, Lua error message, and conditional stack traceback
  - [ ] 13.3 Implement error propagation to status bar display
  - [ ] 13.4 Implement runtime resilience: subsequent macro invocations continue functioning after any error
  - [ ] 13.5 Implement cancellable hook error policy: on handler error, treat as returning `true` (do not cancel), report error, continue invoking subsequent handlers
  - [ ] 13.6 Implement `OnError(error_message)` hook firing after any macro error
  - [ ] 13.7 Implement debug traceback inclusion when `macro.debug_traceback = true`
  - [ ] 13.8 Write unit tests for error capture, message formatting, runtime continuity, hook error policy, and OnError firing
  - Covers: Requirement 6 (AC 6.1, 6.2, 6.3, 6.4, 6.5, 6.6)

- [ ] 14. Macro directory scanning and discovery
  - [ ] 14.1 Implement `MacroDiscovery` struct managing configured Macro_Directories and discovered script inventory
  - [ ] 14.2 Implement startup scan: discover all `.lua` files recursively up to 3 directory levels
  - [ ] 14.3 Implement name keying: register scripts by filename without extension (e.g., `format_cobol.lua` → `"format_cobol"`)
  - [ ] 14.4 Implement directory priority: workspace macros override user macros on name collision, log DEBUG message for shadowing
  - [ ] 14.5 Implement default directory configuration: user-level `~/.config/ffworkbench/macros/` and workspace-level `macros/` subdirectory
  - [ ] 14.6 Implement startup script execution: run `macro.startup_script` once at initialization before any buffer is loaded
  - [ ] 14.7 Implement per-extension auto-load: execute configured script when file with matching extension is opened
  - [ ] 14.8 Implement hot-discovery: detect new `.lua` files appearing in monitored directories within 5 seconds
  - [ ] 14.9 Write unit tests for recursive scan, name resolution, priority shadowing, startup script, extension auto-load, and hot-discovery
  - Covers: Requirement 9 (AC 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7)

- [ ] 15. Auto-reload on file change
  - [ ] 15.1 Implement file watcher integration for all loaded macro script paths using `notify` crate (or ff-vfs watcher)
  - [ ] 15.2 Implement change detection: re-execute modified script within 2 seconds of disk change
  - [ ] 15.3 Implement hook cleanup on reload: unregister all event hooks from the specific reloading script before re-execution
  - [ ] 15.4 Implement fresh hook registration: re-run script to register new hooks, preventing duplicate handlers
  - [ ] 15.5 Implement reload error handling: retain previous version's hooks on failure, display error in status bar, log WARN diagnostic
  - [ ] 15.6 Implement per-buffer state preservation: `buffer` tables unaffected by script reloads
  - [ ] 15.7 Implement `macro.auto_reload` configuration gate: when disabled, no file monitoring occurs
  - [ ] 15.8 Write unit tests for reload trigger, hook cleanup, error retention, buffer state preservation, and config gate
  - Covers: Requirement 8 (AC 8.1, 8.2, 8.3, 8.4, 8.5, 8.6)

- [ ] 16. Script debugging support
  - [ ] 16.1 Implement `trace(message)` Lua global function: output to diagnostic log at INFO level with calling script name and line number prefix
  - [ ] 16.2 Implement `print(...)` Lua global function: concatenate arguments with tabs, output to output panel or diagnostic log
  - [ ] 16.3 Implement execution timing: measure and log duration of MACRO/EXEC/RUN invocations at DEBUG level
  - [ ] 16.4 Implement EXEC return value display: format with `tostring()` and show in status bar
  - [ ] 16.5 Implement instruction limit exceeded reporting: include instruction count in error message
  - [ ] 16.6 Implement full stack traceback when `macro.debug_traceback = true`: include source file paths and line numbers
  - [ ] 16.7 Write unit tests for trace output, print output, timing measurement, EXEC display, and traceback content
  - Covers: Requirement 10 (AC 10.1, 10.2, 10.3, 10.4, 10.5, 10.6)

- [ ] 17. Configuration integration
  - [ ] 17.1 Wire `macro.security_mode` to SecurityGate with default `Prompt`
  - [ ] 17.2 Wire `macro.directories` array to MacroDiscovery with default user + workspace directories
  - [ ] 17.3 Wire `macro.auto_reload` boolean to auto-reload system with default `true`
  - [ ] 17.4 Wire `macro.startup_script` path to startup execution
  - [ ] 17.5 Wire `macro.debug_traceback` boolean to error formatting
  - [ ] 17.6 Wire `macro.instruction_limit` integer to runtime hook with default 10_000_000
  - [ ] 17.7 Wire `macro.memory_limit_mb` integer to runtime memory limit with default 64
  - [ ] 17.8 Wire `macro.trusted_paths` array to security gate for TrustedOnly mode
  - [ ] 17.9 Wire `macro.auto_load_for.<extension>` map to per-extension auto-load
  - [ ] 17.10 Write unit tests for each config key wiring, default values, and invalid config handling
  - Covers: Requirement 1 (AC 1.3, 1.4), Requirement 7 (AC 7.7), Requirement 8 (AC 8.1, 8.5), Requirement 9 (AC 9.1, 9.5, 9.6)

- [ ] 18. Command registration and integration wiring
  - [ ] 18.1 Register `macro.run_named` command (MACRO) with command framework including metadata (display name, category, description)
  - [ ] 18.2 Register `macro.exec_inline` command (EXEC) with command framework
  - [ ] 18.3 Register `macro.run_file` command (RUN) with command framework
  - [ ] 18.4 Wire OnChar/OnKey hooks to editor keypress pipeline integration point
  - [ ] 18.5 Wire OnOpen/OnClose/OnSwitchBuffer hooks to document lifecycle events
  - [ ] 18.6 Wire OnBeforeSave/OnAfterSave hooks to file save pipeline
  - [ ] 18.7 Wire OnCommand hook to command dispatch pre-execution point
  - [ ] 18.8 Write integration tests for command invocation end-to-end and hook firing from lifecycle events
  - Covers: Requirement 5 (AC 5.7), Requirement 3 (AC 3.1)

- [ ] 19. Property-based tests
  - [ ] 19.1 Write PBT: Editor API line number validation property
  - [ ] 19.2 Write PBT: Security mode decision consistency property
  - [ ] 19.3 Write PBT: Hook invocation order preservation property
  - [ ] 19.4 Write PBT: Per-buffer state isolation property
  - [ ] 19.5 Write PBT: Macro transaction atomicity property
  - [ ] 19.6 Write PBT: Instruction limit enforcement property
  - [ ] 19.7 Write PBT: Directory scan name resolution property
  - [ ] 19.8 Write PBT: Auto-reload hook deduplication property
  - Covers: All requirements (property-based validation)

- [ ] 20. Integration tests
  - [ ] 20.1 Write integration test: full macro lifecycle — load script, execute, verify buffer modifications, undo all changes
  - [ ] 20.2 Write integration test: hook cascade — register multiple OnBeforeSave handlers, verify ordering and cancellation
  - [ ] 20.3 Write integration test: per-buffer state across tab switches — set state, switch, verify isolation, switch back, verify restoration
  - [ ] 20.4 Write integration test: security gate — attempt execution in each mode, verify allow/deny decisions
  - [ ] 20.5 Write integration test: error rollback — macro that modifies 5 lines then errors, verify all 5 changes reverted
  - [ ] 20.6 Write integration test: auto-reload — modify script on disk, verify hooks re-registered without duplication
  - [ ] 20.7 Write integration test: EXEC command — evaluate expressions, verify return value display
  - [ ] 20.8 Write integration test: startup script and per-extension auto-load execution order
  - Covers: All requirements (end-to-end validation)

---

## Property-Based Test Definitions

### Property 1: Editor API Line Number Validation

**Validates: Requirement 2.11**

- **Statement:** For any line number `n` and buffer of size `L`, `editor.get_line(n)` succeeds if and only if `1 <= n <= L`; all other values raise a Lua error containing both the invalid index and the valid range.
- **Strategy:** Generate:
  - Buffer sizes: integer in [1, 500]
  - Line numbers: integer in [-10, L+10] (covering valid, zero, negative, and overflow)
- **Invariant:** `get_line(n).is_ok() ⟺ (1 <= n <= L)`; on error, message contains `n` and range `[1, L]`

### Property 2: Security Mode Decision Consistency

**Validates: Requirement 7.1, 7.2, 7.3, 7.4, 7.5**

- **Statement:** For any security mode `M`, script path `P`, and trusted path set `T`, the security decision is deterministic and follows: Disabled → always deny; Enabled → always allow; TrustedOnly → allow iff `P ∈ T` or P is in user-level directory; Prompt → always NeedsPrompt for non-trusted scripts.
- **Strategy:** Generate:
  - Security modes: uniform choice from {Disabled, Prompt, TrustedOnly, Enabled}
  - Script paths: mix of paths inside/outside trusted directories
  - Trusted path sets: 0–10 random directory paths
- **Invariant:** `decide(M, P, T)` matches the mode specification exactly; no mode ever produces an unexpected decision variant

### Property 3: Hook Invocation Order Preservation

**Validates: Requirement 3.3**

- **Statement:** For any sequence of N scripts loaded in order, each defining a handler for the same event, invoking that event calls handlers in load order (first loaded = index 0, second = index 1, etc.).
- **Strategy:** Generate:
  - Number of scripts: integer in [1, 20]
  - Event type: uniform choice from all hook events
  - Each script defines one handler that records its index
- **Invariant:** After invocation, the recorded indices form the sequence [0, 1, 2, ..., N-1]

### Property 4: Per-Buffer State Isolation

**Validates: Requirement 4.1, 4.3, 4.5**

- **Statement:** For any sequence of buffer switches and `buffer` table writes, each buffer's table contains only the keys written while that buffer was active — keys written to one buffer are never visible in another buffer's table.
- **Strategy:** Generate:
  - Number of buffers: integer in [2, 10]
  - Operations: sequence of 10–100 actions from {SwitchTo(buf_id), Write(key, value), Read(key)}
  - Track expected state per buffer
- **Invariant:** After all operations, each buffer's table matches its expected key-value set; no cross-contamination

### Property 5: Macro Transaction Atomicity

**Validates: Requirement 5.4, Requirement 6.1**

- **Statement:** For any macro that performs K document modifications then either succeeds or fails, the buffer state is either all-committed (on success, K changes applied) or all-rolled-back (on failure, buffer identical to pre-execution state).
- **Strategy:** Generate:
  - Number of modifications K: integer in [1, 50]
  - Failure point: None (success) or integer in [1, K] (error after N modifications)
  - Initial buffer content: 10–100 lines of random text
- **Invariant:** On success: buffer has all K changes applied. On failure at step N: buffer equals initial state exactly (byte-for-byte)

### Property 6: Instruction Limit Enforcement

**Validates: Requirement 1.3, 1.5**

- **Statement:** For any configurable instruction limit L and a Lua script that executes exactly N instructions, the script completes successfully if N <= L and is terminated with an InstructionLimitExceeded error if N > L. The buffer state is rolled back on termination.
- **Strategy:** Generate:
  - Instruction limits L: integer in [100, 10_000]
  - Script loops generating approximately N instructions: N drawn from [L-50, L+50] (boundary region)
- **Invariant:** Script succeeds ⟺ actual instructions ≤ L; on limit exceeded, error message reports instruction count, and buffer state equals pre-execution state

### Property 7: Directory Scan Name Resolution

**Validates: Requirement 9.3, 9.4**

- **Statement:** For any set of macro directories with priority ordering and any set of `.lua` files distributed across them, name resolution produces exactly one script per unique base name, always preferring the highest-priority directory.
- **Strategy:** Generate:
  - Number of directories: integer in [1, 5] with distinct priorities
  - Scripts per directory: 1–10 `.lua` files with names drawn from pool of 5–15 unique base names
  - Track expected resolution (highest priority wins)
- **Invariant:** `resolve(name)` returns the path from the highest-priority directory containing that name; total unique resolved names equals the union of all names across directories

### Property 8: Auto-Reload Hook Deduplication

**Validates: Requirement 8.3**

- **Statement:** After any number of sequential auto-reloads of the same script, the hook registry contains exactly one set of handlers from that script (no duplicates from repeated load cycles).
- **Strategy:** Generate:
  - Number of reload cycles: integer in [1, 20]
  - Script defines K hooks: K in [1, 5]
  - After each reload, count handlers registered by the script
- **Invariant:** For all reload counts, handlers registered by the script == K (not K * reload_count)

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding and Foundation", "tasks": ["1", "2"] },
    { "id": 1, "label": "Security and Runtime", "tasks": ["3", "4"], "dependsOn": [0] },
    { "id": 2, "label": "Engine Core", "tasks": ["5", "8", "10", "11"], "dependsOn": [1] },
    { "id": 3, "label": "Editor API", "tasks": ["6", "7"], "dependsOn": [2] },
    { "id": 4, "label": "Hook System", "tasks": ["9"], "dependsOn": [2, 3] },
    { "id": 5, "label": "Commands and Discovery", "tasks": ["12", "14"], "dependsOn": [3, 4] },
    { "id": 6, "label": "Auto-Reload and Debugging", "tasks": ["15", "16"], "dependsOn": [4, 5] },
    { "id": 7, "label": "Error Handling", "tasks": ["13"], "dependsOn": [4, 5] },
    { "id": 8, "label": "Configuration and Wiring", "tasks": ["17", "18"], "dependsOn": [5, 6, 7] },
    { "id": 9, "label": "Property-Based Tests", "tasks": ["19"], "dependsOn": [8] },
    { "id": 10, "label": "Integration Tests", "tasks": ["20"], "dependsOn": [8, 9] }
  ]
}
```

---

## Notes

- This is a Wave 10 (Extensions and Macros) crate depending on `ff-command` (Wave 2), `ff-document` (Wave 4), `ff-undo` (Wave 4), `ff-config` (Wave 2), `ff-logging` (Wave 0), and `ff-plugin` (Wave 2)
- The `mlua` crate is used with `lua54` and `send` features to enable Lua 5.4 embedding with thread-safe runtime
- The `notify` crate provides cross-platform file system watching for auto-reload and hot-discovery
- The editor API functions (`editor.*`) access the active buffer through the `ff-document` crate's line access interface; a reference to the active document is injected into Lua userdata before each invocation
- Macro transactions wrap `ff-undo` transaction primitives; the engine begins a transaction before script execution and commits or rolls back based on outcome
- Security mode decisions for `Prompt` mode require UI interaction (confirmation dialog); the engine returns a `SecurityDecision::NeedsPrompt` that the UI layer handles asynchronously
- Per-buffer state is stored as mlua `RegistryKey` references to Lua tables; swapping is implemented by updating the global `buffer` reference in the Lua registry
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The `OnError` hook is invoked outside the transaction scope to prevent infinite error loops (an error in OnError is logged but does not trigger another OnError invocation)
- Hot-discovery (Task 14.8) uses the same file watcher infrastructure as auto-reload (Task 15.1) — a single watcher monitors all configured directories
- The startup script (Task 14.6) runs with `buffer` set to nil since no document is loaded yet; scripts must guard against nil buffer access
- Command framework integration uses the `ScriptingBridge` defined by `ff-command` — the macro engine is the primary consumer of this bridge interface

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Lua Runtime | AC 1.1 | Task 4 |
| Req 1: Lua Runtime | AC 1.2 | Tasks 3, 4 |
| Req 1: Lua Runtime | AC 1.3 | Tasks 4, 17 |
| Req 1: Lua Runtime | AC 1.4 | Tasks 4, 17 |
| Req 1: Lua Runtime | AC 1.5 | Tasks 4, 11 |
| Req 1: Lua Runtime | AC 1.6 | Tasks 4, 5 |
| Req 1: Lua Runtime | AC 1.7 | Task 5 |
| Req 2: Editor API | AC 2.1 | Task 6 |
| Req 2: Editor API | AC 2.2 | Task 6 |
| Req 2: Editor API | AC 2.3 | Task 6 |
| Req 2: Editor API | AC 2.4 | Task 6 |
| Req 2: Editor API | AC 2.5 | Task 6 |
| Req 2: Editor API | AC 2.6 | Task 6 |
| Req 2: Editor API | AC 2.7 | Task 6 |
| Req 2: Editor API | AC 2.8 | Task 7 |
| Req 2: Editor API | AC 2.9 | Task 7 |
| Req 2: Editor API | AC 2.10 | Task 7 |
| Req 2: Editor API | AC 2.11 | Task 6 |
| Req 3: Event Hooks | AC 3.1 | Tasks 8, 9, 18 |
| Req 3: Event Hooks | AC 3.2 | Tasks 8, 9 |
| Req 3: Event Hooks | AC 3.3 | Tasks 8, 9 |
| Req 3: Event Hooks | AC 3.4 | Task 9 |
| Req 3: Event Hooks | AC 3.5 | Task 9 |
| Req 3: Event Hooks | AC 3.6 | Task 9 |
| Req 3: Event Hooks | AC 3.7 | Task 9 |
| Req 3: Event Hooks | AC 3.8 | Task 9 |
| Req 3: Event Hooks | AC 3.9 | Task 9 |
| Req 3: Event Hooks | AC 3.10 | Task 9 |
| Req 3: Event Hooks | AC 3.11 | Task 9 |
| Req 4: Per-Buffer State | AC 4.1 | Task 10 |
| Req 4: Per-Buffer State | AC 4.2 | Task 10 |
| Req 4: Per-Buffer State | AC 4.3 | Task 10 |
| Req 4: Per-Buffer State | AC 4.4 | Task 10 |
| Req 4: Per-Buffer State | AC 4.5 | Task 10 |
| Req 4: Per-Buffer State | AC 4.6 | Task 10 |
| Req 4: Per-Buffer State | AC 4.7 | Task 10 |
| Req 5: Commands | AC 5.1 | Task 12 |
| Req 5: Commands | AC 5.2 | Task 12 |
| Req 5: Commands | AC 5.3 | Task 12 |
| Req 5: Commands | AC 5.4 | Tasks 11, 12 |
| Req 5: Commands | AC 5.5 | Task 12 |
| Req 5: Commands | AC 5.6 | Task 12 |
| Req 5: Commands | AC 5.7 | Tasks 12, 18 |
| Req 6: Error Handling | AC 6.1 | Tasks 11, 13 |
| Req 6: Error Handling | AC 6.2 | Task 13 |
| Req 6: Error Handling | AC 6.3 | Task 13 |
| Req 6: Error Handling | AC 6.4 | Tasks 11, 13 |
| Req 6: Error Handling | AC 6.5 | Task 13 |
| Req 6: Error Handling | AC 6.6 | Task 13 |
| Req 6: Error Handling | AC 6.7 | Task 11 |
| Req 7: Security Modes | AC 7.1 | Tasks 2, 3 |
| Req 7: Security Modes | AC 7.2 | Task 3 |
| Req 7: Security Modes | AC 7.3 | Task 3 |
| Req 7: Security Modes | AC 7.4 | Task 3 |
| Req 7: Security Modes | AC 7.5 | Task 3 |
| Req 7: Security Modes | AC 7.6 | Task 3 |
| Req 7: Security Modes | AC 7.7 | Tasks 2, 17 |
| Req 8: Auto-Reload | AC 8.1 | Tasks 15, 17 |
| Req 8: Auto-Reload | AC 8.2 | Task 15 |
| Req 8: Auto-Reload | AC 8.3 | Task 15 |
| Req 8: Auto-Reload | AC 8.4 | Task 15 |
| Req 8: Auto-Reload | AC 8.5 | Tasks 15, 17 |
| Req 8: Auto-Reload | AC 8.6 | Task 15 |
| Req 9: Directory Scanning | AC 9.1 | Tasks 14, 17 |
| Req 9: Directory Scanning | AC 9.2 | Tasks 14, 17 |
| Req 9: Directory Scanning | AC 9.3 | Task 14 |
| Req 9: Directory Scanning | AC 9.4 | Task 14 |
| Req 9: Directory Scanning | AC 9.5 | Tasks 14, 17 |
| Req 9: Directory Scanning | AC 9.6 | Tasks 14, 17 |
| Req 9: Directory Scanning | AC 9.7 | Task 14 |
| Req 10: Debugging | AC 10.1 | Task 16 |
| Req 10: Debugging | AC 10.2 | Task 16 |
| Req 10: Debugging | AC 10.3 | Task 16 |
| Req 10: Debugging | AC 10.4 | Task 16 |
| Req 10: Debugging | AC 10.5 | Task 16 |
| Req 10: Debugging | AC 10.6 | Task 16 |
