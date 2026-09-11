#!/usr/bin/env python3
"""Byte-faithful splitter for ff-viewport-scrolling/src/viewport.rs (PA-STD-008, 572nt).

ViewportModel is a single struct with a 511-line impl over 15 PRIVATE fields.
To split the impl across sibling submodule impl blocks, the fields + the two
private helpers (clamp_top_line, emit_event) are widened to pub(super) -- visible
only within the `viewport` module tree, NOT re-exported (no external API change).

Source layout (1-based):
  1-13    : doc + imports
  14-43   : struct ViewportModel { 15 fields }   (fields -> pub(super))
  45      : impl ViewportModel {
  47-211  : constructors + getters + setters
  213-534 : scroll / wheel / scrollbar / cursor-move / pixel methods  -> scrolling.rs
  536-557 : Internal Helpers (clamp_top_line, emit_event) [+ section comment 536]
  557     : } (impl close)
  559-563 : impl Default
  566-572 : pub struct ScrollbarFeedback

mod.rs      = doc/imports + struct(pub(super) fields) + impl{ 47-211 + helpers }
              + `mod scrolling;` + Default + ScrollbarFeedback
scrolling.rs= imports + impl ViewportModel { 213-534 }
"""
import os
import re

SRC = os.path.join("crates", "ff-viewport-scrolling", "src", "viewport.rs")
OUTDIR = os.path.join("crates", "ff-viewport-scrolling", "src", "viewport")

SCROLLING_HEADER = (
    b"//! Scrolling, wheel, scrollbar, and cursor-movement operations for\n"
    b"//! `ViewportModel`. Split from the model for file-size; operates on the\n"
    b"//! same struct via pub(super) fields/helpers.\n"
    b"\n"
    b"use crate::caret_policy::CaretPolicyEngine;\n"
    b"use crate::cursor::CursorModel;\n"
    b"use crate::types::ScrollFraction;\n"
    b"\n"
    b"use super::{ScrollbarFeedback, ViewportModel};\n"
    b"\n"
    b"impl ViewportModel {\n"
)


def widen_fields(struct_body: bytes) -> bytes:
    """Prefix each `    <name>: <type>,` field line with `pub(super) `."""
    out = []
    for line in struct_body.split(b"\n"):
        # field lines look like 4-space indent, identifier, colon; skip doc/comments/braces
        if re.match(rb"^    [a-z_][a-zA-Z0-9_]*: ", line):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def widen_helpers(body: bytes) -> bytes:
    """Prefix `    fn clamp_top_line`/`    fn emit_event` with pub(super)."""
    out = []
    for line in body.split(b"\n"):
        if line.startswith(b"    fn clamp_top_line") or line.startswith(b"    fn emit_event"):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    doc_imports = b"\n".join(lines[0:13])       # 1-13
    struct_body = widen_fields(b"\n".join(lines[13:43]))  # 14-43
    ctor_get_set = b"\n".join(lines[46:211])    # 47-211 (impl body: after `impl {` .. setters)
    scroll_methods = b"\n".join(lines[212:534]) # 213-534
    helpers = widen_helpers(b"\n".join(lines[535:556]))   # 536-556 (helpers, EXCL original impl close at 557)
    default_impl = b"\n".join(lines[558:563])   # 559-563
    scrollbar_struct = b"\n".join(lines[565:])  # 566-end (ScrollbarFeedback + trailing)

    os.makedirs(OUTDIR, exist_ok=True)

    mod_bytes = (
        doc_imports + b"\n"
        + struct_body + b"\n\n"
        + b"impl ViewportModel {\n"
        + ctor_get_set + b"\n\n"
        + helpers + b"\n"
        + b"}\n\n"
        + b"mod scrolling;\n\n"
        + default_impl + b"\n\n"
        + scrollbar_struct + b"\n"
    )
    scrolling_bytes = SCROLLING_HEADER + scroll_methods + b"\n}\n"

    with open(os.path.join(OUTDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(OUTDIR, "scrolling.rs"), "wb") as f:
        f.write(scrolling_bytes)

    os.remove(SRC)
    print(f"mod.rs {len(mod_bytes)}  scrolling.rs {len(scrolling_bytes)}")
    print("removed original viewport.rs")


if __name__ == "__main__":
    main()
