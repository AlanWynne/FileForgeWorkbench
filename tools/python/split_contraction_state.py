#!/usr/bin/env python3
"""Delegation-refactor setup for ff-display-line-mapping/src/contraction_state.rs
(PA-STD-009, 614nt). The bulk is a single `impl DisplayLineMapping for ContractionState`
trait impl -- INDIVISIBLE across files. So we move contraction_state.rs -> a
contraction_state/ module (mod.rs), widen the struct fields + private helpers
(ensure_data/notify_change) to pub(super), and declare `mod ops;`. The 6 largest
trait-method bodies are then extracted (by hand, via str_replace) into an inherent
`impl ContractionState` block in ops.rs (pub(super) helpers), with the trait methods
becoming thin delegators -- getting mod.rs under the 400 cap while keeping the trait
impl in one file (only thin wrappers).

This script does the mechanical part: move file -> mod.rs (byte-faithful) with the
field/helper visibility widened. ops.rs is created empty-scaffold; bodies are moved by
subsequent str_replace edits.
"""
import os

SRC = os.path.join("crates", "ff-display-line-mapping", "src", "contraction_state.rs")
NEWDIR = os.path.join("crates", "ff-display-line-mapping", "src", "contraction_state")

FIELD_NAMES = (
    b"    line_count:", b"    one_to_one:", b"    large_document:",
    b"    data:", b"    fold_text:", b"    listeners:", b"    next_handle_id:",
)


def widen(data: bytes) -> bytes:
    out = []
    for line in data.split(b"\n"):
        if line.startswith(FIELD_NAMES):
            out.append(b"    pub(super) " + line[4:])
        elif line == b"struct FullTrackingData {":
            out.append(b"pub(super) struct FullTrackingData {")
        elif line == b"struct ListenerEntry {":
            out.append(b"pub(super) struct ListenerEntry {")
        elif line == b"    fn ensure_data(&mut self) {":
            out.append(b"    pub(super) fn ensure_data(&mut self) {")
        elif line == b"    fn notify_change(&self, old_count: usize, new_count: usize) {":
            out.append(b"    pub(super) fn notify_change(&self, old_count: usize, new_count: usize) {")
        else:
            out.append(line)
    return b"\n".join(out)


def main():
    with open(SRC, "rb") as f:
        data = f.read()

    body = widen(data)
    # Declare the ops submodule right after the inherent-impl block closes and
    # before the trait impl. Insert `mod ops;` just before `impl DisplayLineMapping`.
    marker = b"\nimpl DisplayLineMapping for ContractionState {"
    assert marker in body, "trait impl header not found"
    body = body.replace(marker, b"\nmod ops;\n" + marker, 1)

    os.makedirs(NEWDIR, exist_ok=True)
    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(body)
    # ops.rs scaffold: imports + empty inherent impl (bodies added by str_replace).
    ops = (
        b"//! Extracted inherent helpers for the heavier ContractionState operations.\n"
        b"//!\n"
        b"//! These are `pub(super)` helpers that the thin `DisplayLineMapping` trait\n"
        b"//! methods in `mod.rs` delegate to (a trait impl cannot span files, so the\n"
        b"//! heavy logic lives here and the trait methods are thin wrappers).\n\n"
        b"use crate::traits::DisplayLineMapping;\n"
        b"use crate::types::{DisplayLine, DocLine, DocPosition, SubLine};\n\n"
        b"use super::ContractionState;\n\n"
        b"impl ContractionState {\n"
        b"}\n"
    )
    with open(os.path.join(NEWDIR, "ops.rs"), "wb") as f:
        f.write(ops)

    os.remove(SRC)
    print(f"mod {len(body)} ops {len(ops)}")
    print("removed original contraction_state.rs")


if __name__ == "__main__":
    main()
