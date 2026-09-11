#!/usr/bin/env python3
"""Byte-faithful splitter for ff-line-commands/src/resolution.rs (PA-STD-010, 422 nt).

ResolutionEngine is a FIELDLESS unit struct, so its associated fns can be split
across sibling `impl ResolutionEngine` blocks freely (helpers -> pub(super)).

Source layout (1-based):
  1-15    : doc + imports
  16-29   : ResolutionResult struct + ResolutionEngine struct
  30      : `impl ResolutionEngine {`
  39-146  : pub fn resolve (+ its doc from ~31)
  147-418 : 4 private helper fns (resolve_immediate/_block_pair/_source_target/
            block_kind_to_line_cmd_kind)
  419     : `}`  (impl close)
  422-end : tests

mod.rs   = doc+imports + types + `impl { <lines 30-146> }` + wiring + tests
kinds.rs = doc+imports + `impl ResolutionEngine { <helpers 147-418, pub(super)> }`
"""
import os

SRC = os.path.join("crates", "ff-line-commands", "src", "resolution.rs")
OUTDIR = os.path.join("crates", "ff-line-commands", "src", "resolution")

# Full import block copied to kinds.rs; clippy will flag any unused -> trim after.
KINDS_HEADER = (
    b"//! Per-kind resolution helpers for `ResolutionEngine`.\n"
    b"//!\n"
    b"//! Immediate commands, block pairs, and source+target resolution.\n"
    b"\n"
    b"use crate::block_pair::BlockPairValidator;\n"
    b"use crate::command::{\n"
    b"    classify, BlockCommandKind, ExecutableCommand, LineCommandCategory, LineCommandKind,\n"
    b"    SourceOperation, SourceTarget, TargetPosition,\n"
    b"};\n"
    b"use crate::compatibility::CommandCompatibilityMatrix;\n"
    b"use crate::config::LineCommandConfig;\n"
    b"use crate::error::LineCommandError;\n"
    b"use crate::parser::LineCommandParser;\n"
    b"use crate::pending::{PendingCommand, PendingCommandStore, PendingReason};\n"
    b"\n"
    b"use super::{ResolutionEngine, ResolutionResult};\n"
    b"\n"
    b"impl ResolutionEngine {\n"
)


def make_pub_super(body: bytes) -> bytes:
    """Prefix `    fn ` (4-space-indented method) with `pub(super) `."""
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

    head_types = b"\n".join(lines[0:29])       # 1-29 (doc, imports, both structs)
    impl_open_resolve = b"\n".join(lines[29:146])  # 30-146 (impl { + resolve)
    helpers = b"\n".join(lines[146:418])       # 147-418 (4 helper fns)
    tests = b"\n".join(lines[421:])            # 422-end

    os.makedirs(OUTDIR, exist_ok=True)

    mod_bytes = (
        head_types + b"\n"
        + impl_open_resolve + b"\n"
        + b"}\n"                     # close the resolve impl block
        + b"\n"
        + b"mod kinds;\n"
        + b"\n"
        + tests + b"\n"
    )
    kinds_bytes = KINDS_HEADER + make_pub_super(helpers) + b"\n}\n"

    with open(os.path.join(OUTDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(OUTDIR, "kinds.rs"), "wb") as f:
        f.write(kinds_bytes)

    os.remove(SRC)
    print(f"mod.rs {len(mod_bytes)}  kinds.rs {len(kinds_bytes)}")
    print("removed original resolution.rs")


if __name__ == "__main__":
    main()
