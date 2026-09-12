#!/usr/bin/env python3
"""Byte-faithful splitter for ff-config/src/reload.rs (PA-STD-002 file 3/6, 509nt).

SPLITTABLE: ReloadEvent + ReloadManager structs + one big inherent impl + 2 free
fns (no trait-impl bulk). 3-way:
  reload/mod.rs : header + ReloadEvent + ReloadManager struct + impl(constructors +
                  accessors: new/with_callbacks/store/schema/schema_mut/callbacks/
                  set_watcher/watcher_mut/watcher/take_watcher) + mod decls + tests
  reload/ops.rs : impl ReloadManager { reload_file/load_project/unload_project/
                  open_project/has_project_layer/project_source_path/layer_values/
                  reload_all }                                            (137-452)
  reload/diff.rs: free fns flatten_config_table + compute_diff              (455-509)

Field-widening: ReloadManager's 5 fields -> pub(super) (ops.rs impl accesses them).
Free-fn-widening: compute_diff + flatten_config_table -> pub(super) (called from
ops.rs; compute_diff also used by tests in mod.rs).

Post-split fixups needed (applied by hand after run):
  - ops.rs: reload_all method-close brace (boundary off-by-one), + SystemTime +
    LayerData imports (now in SUB_IMPORTS above).
  - mod.rs test module (cargo fix over-trims): re-add
    `use super::diff::compute_diff; use crate::error::ConfigError;
     use crate::loader::load_toml_file; use std::path::PathBuf;`

Source layout (1-based):
  1-52    : doc + imports + ReloadEvent + ReloadManager struct
  53      : impl ReloadManager {
  54-136  : constructors + accessors
  137-452 : reload/project operations
  453     : } (impl close)
  455-509 : free fns flatten_config_table + compute_diff
  510-end : tests
"""
import os

SRC = os.path.join("crates", "ff-config", "src", "reload.rs")
NEWDIR = os.path.join("crates", "ff-config", "src", "reload")

SUB_IMPORTS = (
    b"use std::path::{Path, PathBuf};\n"
    b"use std::time::SystemTime;\n"
    b"\n"
    b"use crate::error::ConfigError;\n"
    b"use crate::layer::ConfigLayer;\n"
    b"use crate::loader::{load_toml_file, LayerData};\n"
    b"use crate::merger::merge_layers;\n"
    b"use crate::value::ConfigTable;\n"
    b"\n"
    b"use super::{ReloadEvent, ReloadManager};\n"
    b"use super::diff::compute_diff;\n"
    b"use super::diff::flatten_config_table;\n"
    b"\n"
)

DIFF_IMPORTS = (
    b"use crate::store::EffectiveStore;\n"
    b"\n"
)

FIELD_NAMES = (b"    layers:", b"    current_store:", b"    schema:",
               b"    callbacks:", b"    watcher:")


def widen_fields(head: bytes) -> bytes:
    out = []
    for line in head.split(b"\n"):
        if line.startswith(FIELD_NAMES):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def widen_free_fns(body: bytes) -> bytes:
    out = []
    for line in body.split(b"\n"):
        if line.startswith((b"fn flatten_config_table(", b"fn compute_diff(")):
            out.append(b"pub(super) " + line)
        else:
            out.append(line)
    return b"\n".join(out)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    head = widen_fields(b"\n".join(lines[0:52]))         # 1-52
    impl_accessors = b"\n".join(lines[52:136])           # 53-136 (impl { + ctors + accessors)
    ops_body = b"\n".join(lines[136:452])                # 137-452
    free_fns = widen_free_fns(b"\n".join(lines[454:509]))  # 455-509
    tests = b"\n".join(lines[509:])                       # 510-end

    os.makedirs(NEWDIR, exist_ok=True)

    mod_bytes = (
        head + b"\n\n"
        + impl_accessors + b"\n"
        + b"}\n\n"
        + b"mod diff;\nmod ops;\n\n"
        + tests + b"\n"
    )
    ops_bytes = SUB_IMPORTS + b"impl ReloadManager {\n" + ops_body + b"\n}\n"
    diff_bytes = DIFF_IMPORTS + free_fns + b"\n"

    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(NEWDIR, "ops.rs"), "wb") as f:
        f.write(ops_bytes)
    with open(os.path.join(NEWDIR, "diff.rs"), "wb") as f:
        f.write(diff_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} ops {len(ops_bytes)} diff {len(diff_bytes)}")
    print("removed original reload.rs")


if __name__ == "__main__":
    main()
