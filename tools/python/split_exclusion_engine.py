#!/usr/bin/env python3
"""Byte-faithful splitter for ff-exclude-show-filter/src/exclusion_engine.rs (PA-STD cap, 534nt).

SPLITTABLE: 2 small traits + ExclusionEngine<D,A> struct + one big generic inherent
impl (no `impl Trait for` bulk). NO test module in this file. 2-way by concern:
  exclusion_engine/mod.rs : head + DocumentAccess + ExclusionListener traits +
                            ExclusionEngine struct + impl { new/add_listener/accessors/
                            is_excluded/counts/iters/exclude_line/exclude_range/show_line/
                            show_range/show_all/execute_exclude + private exclude_text/
                            exclude_regex/exclude_all/exclude_tagged/exclude_range_by_number }
                            + mod ops
  exclusion_engine/ops.rs : impl { execute_show + show_all_lines/show_excluded/show_text/
                            show_regex + execute_reset + execute_line_command +
                            exclusion_blocks/block_count/block_at_doc_line + notify_change }

Both sibling impl blocks repeat `impl<D: DisplayLineMapping, A: DocumentAccess>
ExclusionEngine<D, A>`. Field-widening: the 3 struct fields (display_mapping/document/
listeners) -> pub(super) (ops.rs impl accesses them; ops methods also call the pub
exclude_line/exclude_range/is_excluded/excluded_line_count in mod.rs). All private
methods are called only within their own chunk (notify_change used at execute_reset,
both in ops). ExclusionEngine/DocumentAccess/ExclusionListener re-exported at crate
level; they stay in mod.rs, paths unchanged.

Source layout (1-based):
  1-51    : doc + imports + DocumentAccess + ExclusionListener + ExclusionEngine struct
  52      : impl<...> ExclusionEngine<D, A> {
  53-328  : new/accessors/exclude/show-basic + execute_exclude + private exclude_* helpers
  330-534 : SHOW sep + execute_show + show_* + execute_reset + execute_line_command +
            block methods + notify_change (method close 534)
  535     : } (impl close)  [no tests follow]
"""
import os

SRC = os.path.join("crates", "ff-exclude-show-filter", "src", "exclusion_engine.rs")
NEWDIR = os.path.join("crates", "ff-exclude-show-filter", "src", "exclusion_engine")

OPS_IMPORTS = (
    b"use ff_display_line_mapping::{DisplayLineMapping, DocLine};\n"
    b"\n"
    b"use crate::error::ExcludeFilterError;\n"
    b"use crate::text_matcher::TextMatcher;\n"
    b"use crate::types::*;\n"
    b"\n"
    b"use super::{DocumentAccess, ExclusionEngine};\n"
    b"\n"
)

FIELD_NAMES = (b"    display_mapping:", b"    document:", b"    listeners:")


def widen_fields(head: bytes) -> bytes:
    out = []
    for line in head.split(b"\n"):
        if line.startswith(FIELD_NAMES):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    head_impl = widen_fields(b"\n".join(lines[0:328]))  # 1-328 (head + traits + struct + impl{ + basic/exclude)
    ops_body = b"\n".join(lines[329:534])               # 330-534 (SHOW..notify_change method close)

    os.makedirs(NEWDIR, exist_ok=True)

    mod_bytes = (
        head_impl + b"\n"
        + b"}\n\n"
        + b"mod ops;\n"
    )
    ops_bytes = (
        OPS_IMPORTS
        + b"impl<D: DisplayLineMapping, A: DocumentAccess> ExclusionEngine<D, A> {\n"
        + ops_body + b"\n"
        + b"}\n"
    )

    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(NEWDIR, "ops.rs"), "wb") as f:
        f.write(ops_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} ops {len(ops_bytes)}")
    print("removed original exclusion_engine.rs")


if __name__ == "__main__":
    main()
