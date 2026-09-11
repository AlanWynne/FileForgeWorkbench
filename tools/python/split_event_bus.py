#!/usr/bin/env python3
"""Byte-faithful splitter for ff-core/src/event_bus.rs (PA-STD cap, 434 nt).

Splits the payload value-types and the WorkbenchEvent enum out of event_bus.rs
into submodules, keeping EventBus + EventSubscription (private-field coupled) and
the tests in mod.rs. Operates on RAW BYTES so the box-drawing separators + any
non-ASCII survive (they will be ASCII-fixed separately if needed).

Line ranges (1-based inclusive) from the source:
  1-116   : mod head (imports, const, EventBus struct + core impl)
  117-178 : payload value types  -> payloads.rs
  179-287 : WorkbenchEvent enum + EventCategory + impl WorkbenchEvent -> events.rs
  288-432 : EventFilter + EventSubscription + impl EventBus(subscribe)  (mod)
  433-end : tests (mod)

mod.rs = head[1-116] + wiring + tail[288-432] + tests[433-]
"""
import os

SRC = os.path.join("crates", "ff-core", "src", "event_bus.rs")
OUTDIR = os.path.join("crates", "ff-core", "src", "event_bus")

PAYLOADS_HEADER = (
    b"//! Locally-defined event payload value types.\n"
    b"//!\n"
    b"//! Defined in ff-core to avoid circular dependencies / layer violations;\n"
    b"//! re-exported from the parent module.\n"
    b"\n"
    b"use std::collections::HashMap;\n"
    b"\n"
)

EVENTS_HEADER = (
    b"//! The WorkbenchEvent enum, its categories, and category classification.\n"
    b"\n"
    b"use super::{CommandOutcome, CommandParams, DocumentId, NotificationSeverity,\n"
    b"    OperationId, ProgressInfo};\n"
    b"\n"
)

# Wiring inserted into mod.rs right after the head block (line 116).
MOD_WIRING = (
    b"\n"
    b"mod events;\n"
    b"mod payloads;\n"
    b"\n"
    b"pub use events::{EventCategory, WorkbenchEvent};\n"
    b"pub use payloads::{\n"
    b"    CommandOutcome, CommandParams, DocumentId, NotificationSeverity, OperationId,\n"
    b"    ParamValue, ProgressInfo,\n"
    b"};\n"
    b"\n"
)


def join(lines, a, b):
    # a,b are 1-based inclusive; lines is 0-based list.
    return b"\n".join(lines[a - 1:b])


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    head = join(lines, 1, 116)
    payloads = join(lines, 117, 178)
    events = join(lines, 179, 287)
    tail = join(lines, 288, 432)
    tests = b"\n".join(lines[432:])  # 433-end

    os.makedirs(OUTDIR, exist_ok=True)

    mod_bytes = head + b"\n" + MOD_WIRING + b"\n" + tail + b"\n\n" + tests
    payloads_bytes = PAYLOADS_HEADER + payloads + b"\n"
    events_bytes = EVENTS_HEADER + events + b"\n"

    with open(os.path.join(OUTDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(OUTDIR, "payloads.rs"), "wb") as f:
        f.write(payloads_bytes)
    with open(os.path.join(OUTDIR, "events.rs"), "wb") as f:
        f.write(events_bytes)

    os.remove(SRC)
    print(f"mod.rs {len(mod_bytes)}  payloads.rs {len(payloads_bytes)}  events.rs {len(events_bytes)}")
    print("removed original event_bus.rs")


if __name__ == "__main__":
    main()
