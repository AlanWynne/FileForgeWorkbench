#!/usr/bin/env python3
"""Byte-faithful splitter for ff-workflow/src/runner.rs (PA-STD-005 file 1/2, 482nt).

SPLITTABLE: WorkflowResult enum + WorkflowHandle/WorkflowRunner structs + small
inherent impls + a cluster of free async execution fns (no trait-impl bulk). 2-way:
  runner/mod.rs : header + WorkflowResult + WorkflowHandle + impl + WorkflowRunner
                  + impl (new/start) + mod exec + tests
  runner/exec.rs: the execution engine free fns execute_workflow + StepOutcome enum +
                  execute_step_with_policy + execute_rollback + resolve_execution_order

WorkflowRunner::start (mod.rs) spawns execute_workflow (exec.rs) -> widen it to
pub(super). WorkflowHandle/WorkflowResult/WorkflowRunner are re-exported at the crate
level (lib.rs pub use runner::{...}); they stay in mod.rs, paths unchanged.

Source layout (1-based):
  1-24    : doc + imports + WorkflowResult doc start
  25-178  : WorkflowResult enum + WorkflowHandle struct + impl + WorkflowRunner struct
            + impl WorkflowRunner (new/start), impl close at 177
  180-482 : execute_workflow + StepOutcome + execute_step_with_policy +
            execute_rollback + resolve_execution_order                      -> exec.rs
  483-end : tests
"""
import os

SRC = os.path.join("crates", "ff-workflow", "src", "runner.rs")
NEWDIR = os.path.join("crates", "ff-workflow", "src", "runner")

EXEC_IMPORTS = (
    b"use std::collections::HashMap;\n"
    b"use std::sync::Arc;\n"
    b"\n"
    b"use tokio::sync::RwLock;\n"
    b"\n"
    b"use crate::cancellation::CancellationToken;\n"
    b"use crate::definition::WorkflowDefinition;\n"
    b"use crate::error::{RollbackStatus, WorkflowError, WorkflowErrorReport};\n"
    b"use crate::error_policy::{self, ErrorPolicy, ErrorStrategy};\n"
    b"use crate::progress::{self, ProgressReporter, WorkflowExecutionId};\n"
    b"use crate::state::{StepStatus, WorkflowPhase, WorkflowState};\n"
    b"use crate::step::{CompensatingAction, WorkflowEventDispatcher, WorkflowStep};\n"
    b"\n"
    b"use super::{WorkflowHandle, WorkflowResult};\n"
    b"\n"
)


def widen_execute_workflow(body: bytes) -> bytes:
    return body.replace(
        b"async fn execute_workflow(", b"pub(super) async fn execute_workflow(", 1)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    head = b"\n".join(lines[0:178])                     # 1-178 (through impl close)
    exec_body = widen_execute_workflow(b"\n".join(lines[179:482]))  # 180-482
    tests = b"\n".join(lines[482:])                     # 483-end

    os.makedirs(NEWDIR, exist_ok=True)

    mod_bytes = (
        head + b"\n\n"
        + b"mod exec;\n\n"
        + tests + b"\n"
    )
    exec_bytes = EXEC_IMPORTS + exec_body + b"\n"

    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(NEWDIR, "exec.rs"), "wb") as f:
        f.write(exec_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} exec {len(exec_bytes)}")
    print("removed original runner.rs")


if __name__ == "__main__":
    main()
