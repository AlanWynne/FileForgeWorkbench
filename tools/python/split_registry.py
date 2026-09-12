#!/usr/bin/env python3
"""Byte-faithful splitter for ff-plugin/src/registry.rs (PA-STD cap, 697nt/945 total).

SPLITTABLE: PluginEntry + PluginLoadResult + PluginRegistry structs + one big
inherent impl + extract_panic_message free fn (no trait-impl bulk). 3-way by
lifecycle concern:
  registry/mod.rs      : header + PluginEntry + PluginLoadResult + PluginRegistry
                         struct + impl { new/with_capability_registry/plugin_directory/
                         plugin_state/plugin_metadata/list_plugins/register_plugin/
                         discover_plugins } + mod decls + extract_panic_message +
                         Send/Sync const assertion + tests
  registry/load.rs     : impl PluginRegistry { load_all/load_plugin/load_single_plugin }
  registry/lifecycle.rs: impl PluginRegistry { unload_plugin/deactivate_plugin/
                         shutdown_plugin/shutdown_all/hot_reload }

Field-widening: PluginRegistry's 3 private fields (plugins/plugin_directory/services)
-> pub(super) (load.rs + lifecycle.rs impls access them; capability_registry is
already pub(crate)). extract_panic_message -> pub(super) (called from load.rs at
load_single_plugin + lifecycle.rs at deactivate_plugin). The private methods
load_single_plugin (load.rs) / deactivate_plugin+shutdown_plugin (lifecycle.rs) are
each called only within their own submodule, so no method-widening needed.

Source layout (1-based):
  1-62    : doc + imports + PluginEntry + PluginLoadResult + PluginRegistry struct
  63      : impl PluginRegistry {
  64-201  : query/registration methods (through discover_plugins close)
  203-437 : load_all + load_plugin + load_single_plugin
  439-678 : unload_plugin + deactivate_plugin + shutdown_plugin + shutdown_all + hot_reload
  679     : } (impl close)
  681-696 : extract_panic_message fn + Send/Sync const assertion
  698-end : tests
"""
import os

SRC = os.path.join("crates", "ff-plugin", "src", "registry.rs")
NEWDIR = os.path.join("crates", "ff-plugin", "src", "registry")

SUB_IMPORTS = (
    b"use std::sync::Arc;\n"
    b"use std::time::{Duration, Instant};\n"
    b"\n"
    b"use crate::context::PluginContext;\n"
    b"use crate::dependency::DependencyGraph;\n"
    b"use crate::error::PluginError;\n"
    b"use crate::lifecycle::PluginState;\n"
    b"use crate::metadata::PluginMetadata;\n"
    b"use crate::version::{is_api_compatible, PLUGIN_API_VERSION};\n"
    b"\n"
    b"use super::{extract_panic_message, PluginLoadResult, PluginRegistry};\n"
    b"\n"
)

FIELD_NAMES = (b"    plugins:", b"    plugin_directory:", b"    services:")


def widen_fields(head: bytes) -> bytes:
    out = []
    for line in head.split(b"\n"):
        if line.startswith(FIELD_NAMES):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def widen_extract(tail: bytes) -> bytes:
    return tail.replace(
        b"fn extract_panic_message(", b"pub(super) fn extract_panic_message(", 1)


def widen_load_single(body: bytes) -> bytes:
    # load_single_plugin is called from lifecycle.rs (hot_reload) -> pub(super).
    return body.replace(
        b"    fn load_single_plugin(", b"    pub(super) fn load_single_plugin(", 1)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    # NOTE: load_single_plugin method-close brace is on line 437 -- lines[202:438]
    # captures 203-438 incl. that close (off-by-one guard). load_single_plugin is
    # widened to pub(super) below (called from lifecycle.rs hot_reload).
    head = widen_fields(b"\n".join(lines[0:62]))       # 1-62
    query = b"\n".join(lines[62:201])                  # 63-201 (impl { + query methods)
    load = widen_load_single(b"\n".join(lines[202:438]))  # 203-438
    lifecycle = b"\n".join(lines[438:678])             # 439-678
    tail = widen_extract(b"\n".join(lines[680:696]))   # 681-696 (extract fn + const assert)
    tests = b"\n".join(lines[697:])                    # 698-end

    os.makedirs(NEWDIR, exist_ok=True)

    mod_bytes = (
        head + b"\n\n"
        + query + b"\n"
        + b"}\n\n"
        + b"mod lifecycle;\nmod load;\n\n"
        + tail + b"\n\n"
        + tests + b"\n"
    )
    load_bytes = SUB_IMPORTS + b"impl PluginRegistry {\n" + load + b"\n}\n"
    lifecycle_bytes = SUB_IMPORTS + b"impl PluginRegistry {\n" + lifecycle + b"\n}\n"

    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(NEWDIR, "load.rs"), "wb") as f:
        f.write(load_bytes)
    with open(os.path.join(NEWDIR, "lifecycle.rs"), "wb") as f:
        f.write(lifecycle_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} load {len(load_bytes)} lifecycle {len(lifecycle_bytes)}")
    print("removed original registry.rs")


if __name__ == "__main__":
    main()
