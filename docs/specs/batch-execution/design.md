# Design Document -- Batch Command Execution

## Overview

Batch execution adds a headless mode to the `ff-desktop` binary. When
`--batch <file>` is supplied, the binary skips eframe/egui initialisation
entirely and runs a `BatchRunner` that reads commands from the input source,
dispatches them through the existing `ff-command-semantics` pipeline, and
exits with a z/OS-style return code.

No new crate is required. The `BatchRunner` lives in `ff-desktop` as a
sibling to `WorkbenchShell`. The two share the same startup wiring
(logging, config, catalog registry) but diverge at the point where the
interactive path calls `eframe::run_native`.

---

## Architecture

```
ffwb --batch cmds.txt
        |
        v
  main.rs
    detect --batch flag
        |
        +-- interactive path --> WorkbenchShell --> eframe::run_native
        |
        +-- batch path -------> BatchRunner::run(input, output, opts)
                                    |
                                    v
                              read line from BatchInputSource
                                    |
                                    v
                              CommandSemantics::parse_and_dispatch()
                              (same pipeline as Command ===> field)
                                    |
                                    v
                              collect CommandResult
                                    |
                                    v
                              write output to BatchOutputSink
                                    |
                                    v
                              update max Step_Return_Code
                                    |
                                    v
                              check AbortOnError policy
                                    |
                                    v
                              exit(Batch_Return_Code)
```

---

## Key Design Decisions

### 1. No new crate

The `BatchRunner` is a module within `ff-desktop` (`src/batch/mod.rs`).
It reuses all existing crates without adding a new dependency wave.

### 2. Shared command pipeline

`BatchRunner` calls the same `handle_command()` entry point used by
`WorkbenchShell`. This guarantees identical behaviour between interactive
and batch execution. No batch-specific command parser is introduced.

### 3. Headless session context

`BatchRunner` constructs a `BatchSession` struct that provides the
interfaces expected by the command pipeline (catalog registry, config
handle, document store) without any egui context. Commands that attempt
to open a GUI dialog receive a `CommandError::RequiresInteractiveInput`
response, which maps to Step_Return_Code 8.

### 4. Output routing

In interactive mode, command output goes to the status bar or Output_Panel.
In batch mode, a `BatchOutputSink` adapter implements the same output
trait and writes to stdout or a file. No changes to the command pipeline
are needed -- only the output sink is swapped.

### 5. Return code model

The z/OS MAXCC convention (0/4/8/12/16) is used. This is already familiar
to the target audience and maps cleanly to the severity levels used in
`ff-command-semantics` error types.

### 6. FFCMD format reuse

The `.ffcmd` format (defined in lua-macro-engine Req 11.29) is adopted
as the canonical batch input format. The `BatchRunner` parser is a
standalone reader -- it does NOT invoke the Lua engine. This keeps the
two execution paths independent while sharing the same file format.

---

## Module Layout

```
crates/ff-desktop/src/
  batch/
    mod.rs          -- BatchRunner struct, run() entry point
    input.rs        -- BatchInputSource (file / stdin reader, comment/blank skip)
    output.rs       -- BatchOutputSink (stdout / file writer)
    session.rs      -- BatchSession (headless session context)
    return_code.rs  -- StepReturnCode, BatchReturnCode, AbortPolicy
```

---

## Integration Points

| Component | Change |
|-----------|--------|
| `main.rs` | Detect `--batch` flag; branch to `BatchRunner::run()` before `eframe::run_native` |
| `shell/commands.rs` | No change -- `handle_command()` called as-is |
| `ff-command-semantics` | No change -- pipeline unchanged |
| `ff-session` | `BatchSession` loads config + catalog registry; does NOT restore GUI state |
| `ff-logging` | Batch run events logged via existing `ff-logging` API |

---

## Non-Goals

- Parallel command execution (sequential only, matching IKJEFT01 behaviour)
- A separate batch-only command language (all commands are standard FFWB commands)
- Persistent batch job queuing (each `--batch` invocation is a single run)
- GUI rendering in batch mode (strictly headless)
