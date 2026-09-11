#!/usr/bin/env python3
"""Byte-faithful splitter for ff-undo-redo/src/manager.rs (PA-STD-007, 599nt).

DocumentUndoManager: single struct, 11 private fields, one 555-line impl.
Field-widening technique: fields + private helpers -> pub(super), split impl into
a sibling `ops` submodule. Public path `manager::` + re-exports unchanged.

Source layout (1-based):
  1-16    : module doc + imports
  17-33   : struct DocumentUndoManager (doc + fields)   fields -> pub(super)
  35      : impl DocumentUndoManager {
  37-146  : new + Transaction API + Edit Recording          (mod.rs)
  147-267 : Undo/Redo Execution                             -> ops.rs
  268-477 : Save Point/Coalescing/Tentative/Selection/
            Recovery/History/Notifications-registration     (mod.rs)
  478-589 : Private helpers (record_operation, add_to_transaction,
            commit_transaction, notify_*)                   -> ops.rs (pub(super))
  590     : } (impl close)
  592-598 : fn format_op_name (free fn)
  600-783 : tests

mod.rs = doc/imports + struct(pub(super) fields) + impl{ 37-146 + 268-477 }
         + `mod ops;` + format_op_name + tests
ops.rs = imports + impl DocumentUndoManager { <exec 147-267> <helpers 478-589 pub(super)> }
"""
import os
import re

SRC = os.path.join("crates", "ff-undo-redo", "src", "manager.rs")
OUTDIR = os.path.join("crates", "ff-undo-redo", "src", "manager")

OPS_HEADER = (
    b"//! Undo/redo execution + the private state-mutation helpers for\n"
    b"//! `DocumentUndoManager`. Split from the model for file-size; operates on\n"
    b"//! the same struct via pub(super) fields/helpers.\n"
    b"\n"
    b"use crate::coalesce::CoalesceOpType;\n"
    b"use crate::edit_op::EditOperation;\n"
    b"use crate::error::UndoError;\n"
    b"use crate::transaction::Transaction;\n"
    b"\n"
    b"use super::{format_op_name, DocumentUndoManager};\n"
    b"\n"
    b"impl DocumentUndoManager {\n"
)


def widen_struct_fields(body: bytes) -> bytes:
    out = []
    for line in body.split(b"\n"):
        if re.match(rb"^    [a-z_][a-zA-Z0-9_]*: ", line):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def widen_helper_fns(body: bytes) -> bytes:
    """Prefix 4-space-indented `fn ` (private methods) with pub(super)."""
    out = []
    for line in body.split(b"\n"):
        if line.startswith(b"    fn "):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    doc_imports = b"\n".join(lines[0:16])            # 1-16
    struct_block = widen_struct_fields(b"\n".join(lines[16:33]))  # 17-33
    body_new_txn_rec = b"\n".join(lines[36:146])     # 37-146 (impl body start)
    execution = b"\n".join(lines[146:267])           # 147-267
    body_state = b"\n".join(lines[267:477])          # 268-477
    helpers = widen_helper_fns(b"\n".join(lines[477:589]))  # 478-589 (EXCL impl close 590)
    format_fn = b"\n".join(lines[591:598])           # 592-598
    tests = b"\n".join(lines[599:])                  # 600-end

    os.makedirs(OUTDIR, exist_ok=True)

    mod_bytes = (
        doc_imports + b"\n\n"
        + struct_block + b"\n\n"
        + b"impl DocumentUndoManager {\n"
        + body_new_txn_rec + b"\n\n"
        + body_state + b"\n"
        + b"}\n\n"
        + b"mod ops;\n\n"
        + format_fn + b"\n\n"
        + tests + b"\n"
    )
    ops_bytes = OPS_HEADER + execution + b"\n\n" + helpers + b"\n}\n"

    with open(os.path.join(OUTDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(OUTDIR, "ops.rs"), "wb") as f:
        f.write(ops_bytes)

    os.remove(SRC)
    print(f"mod.rs {len(mod_bytes)}  ops.rs {len(ops_bytes)}")
    print("removed original manager.rs")


if __name__ == "__main__":
    main()
