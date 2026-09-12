#!/usr/bin/env python3
"""Byte-faithful splitter for ff-find-and-replace/src/engine.rs (PA-STD-011 half, 837nt).

SPLITTABLE (only trivial impl Default; bulk = FindEngine inherent impl + one free fn).
3-way: mod.rs (config/struct + public-API impl + Default + tests), finders.rs
(private find/change helper methods, pub(super)), stateless.rs (execute_find_stateless).
Field-widening: FindEngine private fields + private helper methods -> pub(super).

Source layout (1-based):
  1-63    : doc + imports + FindEngineConfig(+Default) + FindEngine struct
  64      : impl FindEngine {
  65-328  : public API methods (new/with_config/state/find/rfind/change/
            change_all/rchange/find_for_filter/highlight_all)          (mod.rs)
  330-708 : private helpers (execute_find/find_literal/find_hex/find_regex/
            find_all_in_range/execute_change_all/compute_replacement)  -> finders.rs
  709     : } (impl close)
  711-716 : impl Default for FindEngine
  717-837 : execute_find_stateless (free fn)                           -> stateless.rs
  838-end : tests                                                       (mod.rs)
"""
import os
import re

SRC = os.path.join("crates", "ff-find-and-replace", "src", "engine.rs")
OUTDIR = os.path.join("crates", "ff-find-and-replace", "src", "engine")

# Superset of imports for submodules; cargo fix trims, re-add test-only after.
SUB_IMPORTS = (
    b"use crate::case_folder::CaseFolder;\n"
    b"use crate::direction::SearchDirection;\n"
    b"use crate::error::FindReplaceError;\n"
    b"use crate::hex_search::parse_hex_pattern;\n"
    b"use crate::indexer::{CharacterIndexer, CharacterIndexerMut};\n"
    b"use crate::literal;\n"
    b"use crate::request::{ChangeRequest, FindRequest, WordMatchMode};\n"
    b"use crate::result::{ChangeOutcome, ChangeResult, FindOutcome, FindResult};\n"
    b"use crate::scope::{resolve_column_range, Bounds, ColumnRange, ScopeFilterProvider, ScopeModifier};\n"
    b"use crate::search_mode::SearchMode;\n"
    b"use crate::substitution::SubstitutionTemplate;\n"
    b"use crate::types::{BytePosition, LineNumber, MatchRange};\n"
    b"\n"
    b"use super::FindEngine;\n"
    b"\n"
)


def widen_fields_struct(body: bytes) -> bytes:
    out = []
    for line in body.split(b"\n"):
        if re.match(rb"^    [a-z_][a-zA-Z0-9_]*: ", line):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def widen_methods(body: bytes) -> bytes:
    out = []
    for line in body.split(b"\n"):
        if line.startswith(b"    fn "):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def widen_free_fn(body: bytes) -> bytes:
    out = []
    for line in body.split(b"\n"):
        if line.startswith(b"fn "):
            out.append(b"pub(super) " + line)
        else:
            out.append(line)
    return b"\n".join(out)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    head = widen_fields_struct(b"\n".join(lines[0:63]))    # 1-63 (doc/imports/config/struct)
    impl_open_public = b"\n".join(lines[63:328])           # 64-328 (impl { + public methods)
    helpers = widen_methods(b"\n".join(lines[329:708]))    # 330-708 (private helpers)
    default_impl = b"\n".join(lines[710:716])              # 711-716
    stateless = widen_free_fn(b"\n".join(lines[716:837]))  # 717-837
    tests = b"\n".join(lines[837:])                        # 838-end

    os.makedirs(OUTDIR, exist_ok=True)

    mod_bytes = (
        head + b"\n\n"
        + impl_open_public + b"\n"
        + b"}\n\n"
        + default_impl + b"\n\n"
        + b"mod finders;\nmod stateless;\n\n"
        + tests + b"\n"
    )
    finders_bytes = SUB_IMPORTS + b"use super::stateless::execute_find_stateless;\n\nimpl FindEngine {\n" + helpers + b"\n}\n"
    stateless_bytes = SUB_IMPORTS + stateless + b"\n"

    with open(os.path.join(OUTDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(OUTDIR, "finders.rs"), "wb") as f:
        f.write(finders_bytes)
    with open(os.path.join(OUTDIR, "stateless.rs"), "wb") as f:
        f.write(stateless_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} finders {len(finders_bytes)} stateless {len(stateless_bytes)}")
    print("removed original engine.rs")


if __name__ == "__main__":
    main()
