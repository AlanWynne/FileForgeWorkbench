#!/usr/bin/env python3
"""Byte-faithful splitter for ff-workflow/src/definition.rs (PA-STD-005 file 2/2, 462nt).

SPLITTABLE: type definitions + WorkflowBuilder + validation free fns (only trivial
impl Default for StepDefinition). 2-way by concern:
  definition/mod.rs     : all type defs (WorkflowDefinition/StepDefinition+Default/
                          StepKind/Transition/TransitionPredicate/ParameterDeclaration/
                          ContextKeyDeclaration/WorkflowBuilder + impl) + mod validate
                          + `pub use validate::validate_definition` + tests
  definition/validate.rs: find_unreachable_states (pub(super)) + validate_definition (pub)

WorkflowBuilder::build (mod.rs) calls find_unreachable_states (validate.rs) -> widen
pub(super) + `use validate::find_unreachable_states` in mod.rs. validate_definition is
referenced as crate::definition::validate_definition (builtin.rs tests), so mod.rs
re-exports it via `pub use validate::validate_definition`.

Source layout (1-based):
  1-359   : doc + imports + all type defs + WorkflowBuilder + impl (impl close 359)
  361-462 : find_unreachable_states + validate_definition                 -> validate.rs
  463-end : tests
"""
import os

SRC = os.path.join("crates", "ff-workflow", "src", "definition.rs")
NEWDIR = os.path.join("crates", "ff-workflow", "src", "definition")

VALIDATE_IMPORTS = (
    b"use std::collections::{HashMap, HashSet, VecDeque};\n"
    b"\n"
    b"use crate::error::WorkflowError;\n"
    b"\n"
    b"use super::{StepDefinition, Transition, WorkflowDefinition};\n"
    b"\n"
)


def widen_find(body: bytes) -> bytes:
    return body.replace(
        b"fn find_unreachable_states(", b"pub(super) fn find_unreachable_states(", 1)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    head = b"\n".join(lines[0:359])                     # 1-359 (through impl close)
    validate = widen_find(b"\n".join(lines[360:462]))   # 361-462
    tests = b"\n".join(lines[462:])                     # 463-end

    os.makedirs(NEWDIR, exist_ok=True)

    mod_bytes = (
        head + b"\n\n"
        + b"mod validate;\n\n"
        + b"pub use validate::validate_definition;\n"
        + b"use validate::find_unreachable_states;\n\n"
        + tests + b"\n"
    )
    validate_bytes = VALIDATE_IMPORTS + validate + b"\n"

    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(NEWDIR, "validate.rs"), "wb") as f:
        f.write(validate_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} validate {len(validate_bytes)}")
    print("removed original definition.rs")


if __name__ == "__main__":
    main()
