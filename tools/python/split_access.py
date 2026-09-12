#!/usr/bin/env python3
"""Byte-faithful splitter for ff-config/src/access.rs (PA-STD-002 file 4/6, 485nt).

SPLITTABLE: single ConfigAccess<'a> struct + one big inherent impl + 2 free fns
(no trait-impl bulk). 2-way by concern:
  access/mod.rs             : header + ConfigAccess struct + impl { new + typed
                              getters get/get_with_provenance/get_string/get_int/
                              get_float/get_bool/get_array/get_table/all_values +
                              private resolve_value/validate_if_schema_exists }
                              + mod decl + tests
  access/editorconfig_access.rs : impl<'a> ConfigAccess<'a> { resolve_editorconfig
                              + get_for_file + get_string_for_file/get_int_for_file/
                              get_bool_for_file } + free fns is_editor_key +
                              editorconfig_value_for_key

Field-widening: ConfigAccess's 2 fields (store/schema) -> pub(super) (defensive;
the file-aware chunk only calls self.get_* public methods, but keep module-tree
access open). The file-aware getters call pub methods on self; no private access.

Post-split fixups (applied by hand after run):
  - editorconfig_access.rs: widen is_editor_key + editorconfig_value_for_key to
    pub(super) (tests call them via super::).
  - mod.rs: add `#[cfg(test)] use editorconfig_access::{editorconfig_value_for_key,
    is_editor_key};` (so super::<fn> resolves from the test module) + test-module
    imports `EditorConfigProperties, IndentStyle` (cargo fix trimmed the file-top
    imports these tests used unqualified).

Source layout (1-based):
  1-32    : doc + imports + ConfigAccess struct
  33      : impl<'a> ConfigAccess<'a> {
  34-333  : typed getters + resolve_value + validate_if_schema_exists
  335-423 : resolve_editorconfig + *_for_file getters
  424     : } (impl close)
  426-485 : free fns is_editor_key + editorconfig_value_for_key
  486-end : tests
"""
import os

SRC = os.path.join("crates", "ff-config", "src", "access.rs")
NEWDIR = os.path.join("crates", "ff-config", "src", "access")

SUB_IMPORTS = (
    b"use std::path::Path;\n"
    b"\n"
    b"use crate::editorconfig::parser::{\n"
    b"    Charset, EditorConfigProperties, EndOfLine, IndentSize, IndentStyle,\n"
    b"};\n"
    b"use crate::editorconfig::resolver::resolve_editorconfig as resolve_editorconfig_for_path;\n"
    b"use crate::error::ConfigError;\n"
    b"use crate::value::ConfigValue;\n"
    b"\n"
    b"use super::ConfigAccess;\n"
    b"\n"
)

FIELD_NAMES = (b"    store:", b"    schema:")


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

    head = widen_fields(b"\n".join(lines[0:32]))       # 1-32
    impl_getters = b"\n".join(lines[32:333])           # 33-333 (impl { + typed getters)
    fileaware = b"\n".join(lines[334:423])             # 335-423 (file-aware methods)
    free_fns = b"\n".join(lines[425:485])              # 426-485
    tests = b"\n".join(lines[485:])                    # 486-end

    os.makedirs(NEWDIR, exist_ok=True)

    mod_bytes = (
        head + b"\n\n"
        + impl_getters + b"\n"
        + b"}\n\n"
        + b"mod editorconfig_access;\n\n"
        + tests + b"\n"
    )
    ec_bytes = (
        SUB_IMPORTS
        + b"impl<'a> ConfigAccess<'a> {\n"
        + fileaware + b"\n"
        + b"}\n\n"
        + free_fns + b"\n"
    )

    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(NEWDIR, "editorconfig_access.rs"), "wb") as f:
        f.write(ec_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} editorconfig_access {len(ec_bytes)}")
    print("removed original access.rs")


if __name__ == "__main__":
    main()
