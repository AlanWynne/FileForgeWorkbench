#!/usr/bin/env python3
"""Byte-faithful splitter for ff-logging/src/init.rs (PA-STD cap, 639nt/746 total).

SPLITTABLE: all free fns + LoggingStatus enum + LogSubsystem struct + statics
(no trait-impl bulk). 3-way by concern:
  init/mod.rs        : header + LoggingStatus enum + LogSubsystem struct + statics
                       (IS_SHUTDOWN/SUBSYSTEM) + dir helpers (ensure_log_directory/
                       resolve_log_directory) + get_sender/take_writer_handle +
                       mod decls + `pub use api::{...}` re-exports + tests
  init/api.rs        : the public API fns init/init_default/reconfigure/
                       install_panic_hook/log/log_lazy/is_fallback/
                       is_logging_available/dropped_count/current_level
  init/writer_loop.rs: the private writer-thread internals writer_thread_loop/
                       handle_reconfigure/handle_record

The 10 public API fns are re-exported at the crate level (lib.rs pub use init::{...}),
so mod.rs re-exports them from api to keep those paths working. get_sender/
take_writer_handle stay in mod.rs (crate::init::get_sender is imported by shutdown.rs
+ plugin_handle.rs). init (api.rs) spawns the writer thread -> calls writer_thread_loop
(writer_loop.rs, widened pub(super)) and constructs LogSubsystem (fields widened
pub(super)); it reads/sets the SUBSYSTEM static (widened pub(super)).

Post-split fixups (applied by hand): api.rs needs `use std::sync::Mutex` +
`use std::path::{Path, PathBuf}` (init builds LogSubsystem); writer_loop.rs needs
`use std::path::{Path, PathBuf}` + `use crate::level::LogLevel` (handle_reconfigure/
handle_record); mod.rs test module re-add `use crate::level::LogLevel` after
cargo-fix trim. (Import blocks above already updated to match.)

Source layout (1-based):
  1-131   : doc + imports + LoggingStatus + LogSubsystem + statics + dir helpers
  133-459 : init/init_default/reconfigure/install_panic_hook/log/log_lazy/
            is_fallback/is_logging_available/dropped_count/current_level      -> api.rs
  461-476 : get_sender + take_writer_handle                                    (mod.rs)
  478-639 : // Writer Thread Loop sep + writer_thread_loop/handle_reconfigure/
            handle_record                                                       -> writer_loop.rs
  640-end : tests                                                              (mod.rs)
"""
import os

SRC = os.path.join("crates", "ff-logging", "src", "init.rs")
NEWDIR = os.path.join("crates", "ff-logging", "src", "init")

API_IMPORTS = (
    b"use std::path::{Path, PathBuf};\n"
    b"use std::sync::atomic::{AtomicU8, Ordering};\n"
    b"\n"
    b"use crate::channel::{create_log_channel, ChannelMessage, FormattedRecord};\n"
    b"use crate::config::LogConfig;\n"
    b"use crate::format::format_record;\n"
    b"use crate::level::LogLevel;\n"
    b"use crate::record::LogRecord;\n"
    b"use crate::sink::{is_fallback_active, NoOpSink};\n"
    b"use crate::writer::LogFileWriter;\n"
    b"\n"
    b"use super::writer_loop::writer_thread_loop;\n"
    b"use super::{\n"
    b"    ensure_log_directory, get_sender, resolve_log_directory, LogSubsystem, LoggingStatus,\n"
    b"    IS_SHUTDOWN, SUBSYSTEM,\n"
    b"};\n"
    b"\n"
)

WRITER_IMPORTS = (
    b"use std::path::{Path, PathBuf};\n"
    b"\n"
    b"use crate::channel::{ChannelMessage, FormattedRecord};\n"
    b"use crate::format::format_record;\n"
    b"use crate::level::LogLevel;\n"
    b"use crate::record::LogRecord;\n"
    b"use crate::rotation::{\n"
    b"    enforce_retention, handle_rotation_failure, perform_rotation, should_rotate,\n"
    b"};\n"
    b"use crate::writer::LogFileWriter;\n"
    b"\n"
)

API_RE_EXPORT = (
    b"pub use api::{\n"
    b"    current_level, dropped_count, init, init_default, install_panic_hook, is_fallback,\n"
    b"    is_logging_available, log, log_lazy, reconfigure,\n"
    b"};\n"
)


def widen_state(state: bytes) -> bytes:
    state = state.replace(
        b"static SUBSYSTEM: OnceLock<LogSubsystem>",
        b"pub(super) static SUBSYSTEM: OnceLock<LogSubsystem>", 1)
    # LogSubsystem fields -> pub(super) so api.rs `init` can construct it.
    out = []
    for line in state.split(b"\n"):
        if line.startswith((b"    level:", b"    sender:", b"    writer_handle:")):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def widen_writer(writer: bytes) -> bytes:
    return writer.replace(
        b"fn writer_thread_loop(", b"pub(super) fn writer_thread_loop(", 1)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    state = widen_state(b"\n".join(lines[0:131]))     # 1-131
    api = b"\n".join(lines[132:459])                  # 133-459
    helpers = b"\n".join(lines[460:476])              # 461-476 (get_sender/take_writer_handle)
    writer = widen_writer(b"\n".join(lines[477:639])) # 478-639
    tests = b"\n".join(lines[639:])                   # 640-end

    os.makedirs(NEWDIR, exist_ok=True)

    mod_bytes = (
        state + b"\n\n"
        + helpers + b"\n\n"
        + b"mod api;\nmod writer_loop;\n\n"
        + API_RE_EXPORT + b"\n"
        + tests + b"\n"
    )
    api_bytes = API_IMPORTS + api + b"\n"
    writer_bytes = WRITER_IMPORTS + writer + b"\n"

    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(NEWDIR, "api.rs"), "wb") as f:
        f.write(api_bytes)
    with open(os.path.join(NEWDIR, "writer_loop.rs"), "wb") as f:
        f.write(writer_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} api {len(api_bytes)} writer_loop {len(writer_bytes)}")
    print("removed original init.rs")


if __name__ == "__main__":
    main()
