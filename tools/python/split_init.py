#!/usr/bin/env python3
"""Byte-faithful splitter for ff-config/src/init.rs (PA-STD-002 file 5/6, 445nt).

SPLITTABLE: small ConfigInitOptions builder + free fns (only trivial impl Default).
2-way by concern:
  init/mod.rs   : header + ConfigInitOptions struct + impl Default + impl (builder)
                  + lifecycle fns init/shutdown/auto_detect_project_config
                  + mod schema + `pub use schema::{register_core_schema,
                    register_catalog_schema}` + tests
  init/schema.rs: the two schema-registration free fns register_core_schema +
                  register_catalog_schema (the big schema-entry tables)

register_core_schema + register_catalog_schema are re-exported at the crate level
(lib.rs `pub use init::{register_core_schema, register_catalog_schema, ...}`), so
init/mod.rs re-exports them from schema.rs to keep those paths working; `init()`
(in mod.rs) calls register_core_schema via that re-export.

Source layout (1-based):
  1-85    : doc + imports + ConfigInitOptions struct + impl Default + impl(builder)
  87-277  : register_core_schema + register_catalog_schema           -> schema.rs
  279-445 : init + shutdown + auto_detect_project_config             (mod.rs)
  446-end : tests                                                     (mod.rs)
"""
import os

SRC = os.path.join("crates", "ff-config", "src", "init.rs")
NEWDIR = os.path.join("crates", "ff-config", "src", "init")

SCHEMA_IMPORTS = (
    b"use crate::error::ValueType;\n"
    b"use crate::schema::{SchemaEntry, SchemaRegistry};\n"
    b"use crate::value::ConfigValue;\n"
    b"\n"
)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    # NOTE: register_catalog_schema's fn-close brace is on line 278 (1-based),
    # so schema_fns must include index 277. lines[86:278] captures lines 87-278.
    head_options = b"\n".join(lines[0:85])    # 1-85 (doc/imports/options/impls)
    schema_fns = b"\n".join(lines[86:278])    # 87-278 (register_core_schema + catalog + close)
    lifecycle = b"\n".join(lines[278:445])    # 279-445 (init/shutdown/auto_detect)
    tests = b"\n".join(lines[445:])           # 446-end

    os.makedirs(NEWDIR, exist_ok=True)

    mod_bytes = (
        head_options + b"\n\n"
        + b"mod schema;\n"
        + b"pub use schema::{register_catalog_schema, register_core_schema};\n\n"
        + lifecycle + b"\n\n"
        + tests + b"\n"
    )
    schema_bytes = SCHEMA_IMPORTS + schema_fns + b"\n"

    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(NEWDIR, "schema.rs"), "wb") as f:
        f.write(schema_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} schema {len(schema_bytes)}")
    print("removed original init.rs")


if __name__ == "__main__":
    main()
