#!/usr/bin/env python3
"""Byte-faithful splitter for ff-config/src/plugin_handle.rs (PA-STD-002 file 6/6, 431nt).

SPLITTABLE: PluginConfigHandle<'a> struct + impl Debug (trivial) + one inherent
impl + a cluster of lifecycle free fns + PluginDefault struct. 2-way by concern:
  plugin_handle/mod.rs      : header + PluginConfigHandle struct + impl Debug +
                              impl<'a> PluginConfigHandle<'a> (new + getters/set +
                              on_reload + private resolve_key/make_full_key/
                              check_namespace) + mod lifecycle + `pub use lifecycle::
                              {create_plugin_config_handle, ..., PluginDefault}` + tests
  plugin_handle/lifecycle.rs: create_plugin_config_handle[_with_callbacks] +
                              PluginDefault struct + register_plugin_defaults +
                              unload_plugin

lib.rs re-exports create_*/register_plugin_defaults/unload_plugin/PluginDefault from
plugin_handle, so mod.rs re-exports them from lifecycle to keep those paths working.
PluginConfigHandle stays in mod.rs. The lifecycle create_* fns call the private
`PluginConfigHandle::new`, so `new` is widened to pub(super).

Post-split fixups (applied by hand after run):
  - impl PluginConfigHandle close brace off-by-one (line 273) -- add `}` before
    `mod lifecycle;` (guarded now by lines[0:273]).
  - mod.rs test module (cargo fix over-trim): re-add `use crate::error::ValueType;`
    and change `use crate::schema::SchemaEntry;` -> `{Constraints, SchemaEntry}`.

Source layout (1-based):
  1-22    : doc + imports
  24-38   : PluginConfigHandle struct
  39-48   : impl Debug for PluginConfigHandle
  50-272  : impl<'a> PluginConfigHandle<'a> (new + methods), impl close at 272
  274-431 : create_plugin_config_handle[_with_callbacks] + PluginDefault +
            register_plugin_defaults + unload_plugin
  432-end : tests
"""
import os

SRC = os.path.join("crates", "ff-config", "src", "plugin_handle.rs")
NEWDIR = os.path.join("crates", "ff-config", "src", "plugin_handle")

LIFECYCLE_IMPORTS = (
    b"use std::sync::Arc;\n"
    b"\n"
    b"use crate::callback::{CallbackHandle, CallbackRegistry, ReloadCallback};\n"
    b"use crate::error::{ConfigError, ValueType};\n"
    b"use crate::namespace::{is_reserved_namespace, plugin_namespace_prefix, validate_plugin_name};\n"
    b"use crate::schema::{Constraints, SchemaEntry, SchemaRegistry};\n"
    b"use crate::store::EffectiveStore;\n"
    b"use crate::value::ConfigValue;\n"
    b"\n"
    b"use super::PluginConfigHandle;\n"
    b"\n"
)


def widen_new(body: bytes) -> bytes:
    # The private constructor `    fn new(` -> pub(super) so lifecycle.rs can call it.
    return body.replace(b"    fn new(\n", b"    pub(super) fn new(\n", 1)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    # NOTE: impl PluginConfigHandle close brace is on line 273 (1-based) --
    # lines[0:273] captures lines 1-273 incl. the impl close (off-by-one guard).
    head_handle = widen_new(b"\n".join(lines[0:273]))   # 1-273 (doc/imports/struct/Debug/impl+close)
    lifecycle = b"\n".join(lines[273:431])              # 274-431 (free fns + PluginDefault)
    tests = b"\n".join(lines[431:])                     # 432-end

    os.makedirs(NEWDIR, exist_ok=True)

    mod_bytes = (
        head_handle + b"\n\n"
        + b"mod lifecycle;\n"
        + b"pub use lifecycle::{\n"
        + b"    create_plugin_config_handle, create_plugin_config_handle_with_callbacks,\n"
        + b"    register_plugin_defaults, unload_plugin, PluginDefault,\n"
        + b"};\n\n"
        + tests + b"\n"
    )
    lifecycle_bytes = LIFECYCLE_IMPORTS + lifecycle + b"\n"

    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(NEWDIR, "lifecycle.rs"), "wb") as f:
        f.write(lifecycle_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} lifecycle {len(lifecycle_bytes)}")
    print("removed original plugin_handle.rs")


if __name__ == "__main__":
    main()
