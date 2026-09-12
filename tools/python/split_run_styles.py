#!/usr/bin/env python3
"""Byte-faithful splitter for ff-text-decorations/src/run_styles.rs (PA-STD cap, 410nt).

SPLITTABLE: Run<T> + RunStyles<T> structs + one generic inherent impl + 3 trait impls
(Clone/PartialEq/Debug). 410 non-test = only 10 over cap. 2-way: move the private
helper methods out of the inherent impl into a sibling internal.rs impl block.
  run_styles/mod.rs     : head + Run + RunStyles struct + impl (public API only:
                          new/value_at/run_start/run_end/fill_range/insert_space/
                          delete_range/is_empty/total_length/runs_in_range/runs) +
                          impl Clone + impl PartialEq + impl Debug + mod internal + tests
  run_styles/internal.rs: sibling impl<T> RunStyles<T> { find_run_index /
                          find_run_index_for_insert / split_at / merge_adjacent /
                          rebuild_cumulative } (private helpers, widened pub(super))

Field-widening: RunStyles's 3 fields (runs/cumulative/total_length) -> pub(super)
(internal.rs impl + the trait impls in mod.rs access them); the 5 private helper
methods -> pub(super) (public methods in mod.rs call them). Both inherent impl blocks
repeat `impl<T: Clone + Eq + Default + Debug> RunStyles<T>`. Run/RunStyles re-exported
at crate level; they stay in mod.rs, paths unchanged.

Source layout (1-based):
  1-31    : doc + `use std::fmt::Debug` + Run struct + RunStyles struct
  32      : impl<T> RunStyles<T> {
  33-293  : public API methods (through runs() close at 293)
  294     : // === Private Helpers === separator
  296-383 : find_run_index / find_run_index_for_insert / split_at / merge_adjacent /
            rebuild_cumulative                                            -> internal.rs
  384     : } (inherent impl close)
  386-409 : impl Clone + impl PartialEq + impl Debug for RunStyles         (mod.rs)
  411-end : tests                                                          (mod.rs)
"""
import os

SRC = os.path.join("crates", "ff-text-decorations", "src", "run_styles.rs")
NEWDIR = os.path.join("crates", "ff-text-decorations", "src", "run_styles")

INTERNAL_IMPORTS = (
    b"use std::fmt::Debug;\n"
    b"\n"
    b"use super::{Run, RunStyles};\n"
    b"\n"
)

FIELD_NAMES = (b"    runs:", b"    cumulative:", b"    total_length:")
HELPER_FNS = (b"    fn find_run_index(", b"    fn find_run_index_for_insert(",
              b"    fn split_at(", b"    fn merge_adjacent(",
              b"    fn rebuild_cumulative(")


def widen_fields(head: bytes) -> bytes:
    out = []
    for line in head.split(b"\n"):
        if line.startswith(FIELD_NAMES):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def widen_helpers(body: bytes) -> bytes:
    out = []
    for line in body.split(b"\n"):
        if line.startswith(HELPER_FNS):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    head_pub = widen_fields(b"\n".join(lines[0:293]))     # 1-293 (head + structs + impl{ + public API)
    internal = widen_helpers(b"\n".join(lines[295:383]))  # 296-383 (private helpers)
    trait_impls = b"\n".join(lines[385:409])              # 386-409 (Clone/PartialEq/Debug)
    tests = b"\n".join(lines[410:])                       # 411-end

    os.makedirs(NEWDIR, exist_ok=True)

    mod_bytes = (
        head_pub + b"\n"
        + b"}\n\n"
        + trait_impls + b"\n\n"
        + b"mod internal;\n\n"
        + tests + b"\n"
    )
    internal_bytes = (
        INTERNAL_IMPORTS
        + b"impl<T: Clone + Eq + Default + Debug> RunStyles<T> {\n"
        + b"    // === Private Helpers ====================================================\n\n"
        + internal + b"\n"
        + b"}\n"
    )

    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(NEWDIR, "internal.rs"), "wb") as f:
        f.write(internal_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} internal {len(internal_bytes)}")
    print("removed original run_styles.rs")


if __name__ == "__main__":
    main()
