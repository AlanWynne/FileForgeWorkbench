#!/usr/bin/env python3
"""Byte-faithful splitter for ff-config/src/config_handle.rs (PA-STD cap, 808nt/1262 total).

SPLITTABLE: single ConfigHandle struct + one huge inherent impl (no trait-impl bulk)
+ a cluster of TOML-IO free fns. 4-way:
  mod.rs       : header + ConfigSystem + ConfigHandle struct + impl(constructors+accessors)
                 + mod decls + tests
  runtime.rs   : impl ConfigHandle { write access + locked + schema + audit }  (209-493)
  export_ext.rs: impl ConfigHandle { export/import + internal helpers }         (495-656)
  toml_io.rs   : the free fns (collect_*, extract_locked_keys, TOML file IO)     (658-807)

Field-widening: ConfigHandle.inner -> pub(super) (sibling impl blocks access it).
Free-fn-widening: the 5 free fns called from impl blocks -> pub(super); the
5 intra-toml_io helpers stay private.

Source layout (1-based):
  1-78    : doc + imports + ConfigSystem struct + ConfigHandle doc + struct {inner}
  79      : impl ConfigHandle {
  80-208  : constructors (new/with_profile_manager) + read accessors (get*/resolve/get_for_file)
  209-493 : write access + Locked Key + Schema query + Audit Log sections
  495-656 : Export/Import API + internal helpers (upsert/remove/rebuild)
  657     : } (impl close)
  658-807 : free fns (collect_all_effective_values/collect_layer_values/
            extract_locked_keys/write_key_to_toml_file/remove_key_from_toml_file/
            read_toml_as_value/write_toml_value/set_dotted_key/remove_dotted_key/
            config_value_to_toml)
  808     : blank
  809-end : tests
"""
import os
import re

SRC = os.path.join("crates", "ff-config", "src", "config_handle.rs")
NEWDIR = os.path.join("crates", "ff-config", "src", "config_handle")

# Superset imports for the impl submodules (runtime.rs, export_ext.rs).
# cargo fix trims unused; re-add test-only never needed (tests stay in mod.rs).
SUB_IMPORTS = (
    b"use std::path::Path;\n"
    b"use std::sync::Arc;\n"
    b"\n"
    b"use crate::access::ConfigAccess;\n"
    b"use crate::audit::{AuditEntry, AuditFilter};\n"
    b"use crate::error::ConfigError;\n"
    b"use crate::layer::ConfigLayer;\n"
    b"use crate::loader::LayerData;\n"
    b"use crate::provenance::EffectiveValue;\n"
    b"use crate::reload::{ReloadEvent, ReloadManager};\n"
    b"use crate::value::{ConfigTable, ConfigValue};\n"
    b"\n"
    b"use super::{ConfigHandle, ConfigSystem};\n"
    b"use super::toml_io::{\n"
    b"    collect_all_effective_values, collect_layer_values, extract_locked_keys,\n"
    b"    remove_key_from_toml_file, write_key_to_toml_file,\n"
    b"};\n"
    b"\n"
)

# Imports for toml_io.rs (free fns). Superset; cargo fix trims.
TOML_IMPORTS = (
    b"use crate::layer::ConfigLayer;\n"
    b"use crate::value::ConfigTable;\n"
    b"\n"
)

# Free fns callable from impl submodules -> pub(super).
# set_dotted_key/remove_dotted_key are also used by the tests in mod.rs.
WIDEN_FREE_FNS = (
    b"fn collect_all_effective_values(",
    b"fn collect_layer_values(",
    b"fn extract_locked_keys(",
    b"fn write_key_to_toml_file(",
    b"fn remove_key_from_toml_file(",
    b"fn set_dotted_key(",
    b"fn remove_dotted_key(",
)

# Private impl helper methods in the export chunk that are called from the
# runtime chunk (set_active_profile) -> widen to pub(super).
WIDEN_METHODS = (
    b"    fn upsert_layer(",
    b"    fn remove_layer(",
    b"    fn rebuild_store(",
)


def widen_inner_field(head: bytes) -> bytes:
    return head.replace(b"    inner: Arc<RwLock<ConfigSystem>>,",
                        b"    pub(super) inner: Arc<RwLock<ConfigSystem>>,", 1)


def widen_free_fns(body: bytes) -> bytes:
    out = []
    for line in body.split(b"\n"):
        if line.startswith(WIDEN_FREE_FNS):
            out.append(b"pub(super) " + line)
        else:
            out.append(line)
    return b"\n".join(out)


def widen_export_methods(body: bytes) -> bytes:
    out = []
    for line in body.split(b"\n"):
        if line.startswith(WIDEN_METHODS):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    head = widen_inner_field(b"\n".join(lines[0:78]))       # 1-78
    impl_open_core = b"\n".join(lines[78:208])              # 79-208 (impl { + ctors + accessors)
    runtime_body = b"\n".join(lines[208:493])               # 209-493
    export_body = widen_export_methods(b"\n".join(lines[494:656]))  # 495-656
    free_fns = widen_free_fns(b"\n".join(lines[657:807]))   # 658-807
    tests = b"\n".join(lines[808:])                          # 809-end

    os.makedirs(NEWDIR, exist_ok=True)

    mod_bytes = (
        head + b"\n\n"
        + impl_open_core + b"\n"
        + b"}\n\n"
        + b"mod export_ext;\nmod runtime;\nmod toml_io;\n\n"
        + tests + b"\n"
    )
    runtime_bytes = SUB_IMPORTS + b"impl ConfigHandle {\n" + runtime_body + b"\n}\n"
    export_bytes = SUB_IMPORTS + b"impl ConfigHandle {\n" + export_body + b"\n}\n"
    toml_bytes = TOML_IMPORTS + free_fns + b"\n"

    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(NEWDIR, "runtime.rs"), "wb") as f:
        f.write(runtime_bytes)
    with open(os.path.join(NEWDIR, "export_ext.rs"), "wb") as f:
        f.write(export_bytes)
    with open(os.path.join(NEWDIR, "toml_io.rs"), "wb") as f:
        f.write(toml_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} runtime {len(runtime_bytes)} "
          f"export_ext {len(export_bytes)} toml_io {len(toml_bytes)}")
    print("removed original config_handle.rs")


if __name__ == "__main__":
    main()
