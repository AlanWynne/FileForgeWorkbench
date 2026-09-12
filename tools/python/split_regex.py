#!/usr/bin/env python3
"""Byte-faithful splitter for ff-find-and-replace/src/regex.rs (PA-STD-011, 992nt).

Biggest offender. SPLITTABLE (only trivial `impl Default`; bulk is free fns).
4-way: mod.rs (types + RegexEngine + impl + Default + tests), compile.rs
(compile_pattern), parse.rs (quantifier/char-class/escape helpers), matcher.rs
(try_match_at/execute_nfa/is_word_byte_nfa). Private types + free fns -> pub(super).

Source layout (1-based):
  1-25    : module doc + imports + 3 consts
  26-276  : private types (NfaInstruction/AnchorKind/CharClass+impl/CompiledRegex)
            + RegexEngine + impl RegexEngine + impl Default            (mod.rs)
  277-569 : compile_pattern (+ its doc at 277-278)                     -> compile.rs
  570-722 : apply_quantifier/parse_char_class/escape_to_byte/hex_val   -> parse.rs
  723-992 : try_match_at/execute_nfa/is_word_byte_nfa                   -> matcher.rs
  993-end : tests                                                       (mod.rs)

Private free fns -> pub(super). Private type decls + their fields -> pub(super).
"""
import os
import re

SRC = os.path.join("crates", "ff-find-and-replace", "src", "regex.rs")
OUTDIR = os.path.join("crates", "ff-find-and-replace", "src", "regex")

# Imports the submodules may need; unused ones trimmed by cargo fix afterward.
SUB_IMPORTS = (
    b"use crate::error::FindReplaceError;\n"
    b"use crate::indexer::CharacterIndexer;\n"
    b"use crate::result::FindResult;\n"
    b"use crate::types::MatchRange;\n"
    b"\n"
    b"use super::{AnchorKind, CharClass, CompiledRegex, NfaInstruction};\n"
    b"\n"
)


def widen_free_fns(body: bytes) -> bytes:
    """Prefix top-level `fn ` (column 0) with pub(super)."""
    out = []
    for line in body.split(b"\n"):
        if line.startswith(b"fn "):
            out.append(b"pub(super) " + line)
        else:
            out.append(line)
    return b"\n".join(out)


def widen_types(body: bytes) -> bytes:
    """Widen private type decls + their fields + CharClass::matches to pub(super)."""
    out = []
    for line in body.split(b"\n"):
        if line.startswith(b"enum ") or line.startswith(b"struct "):
            out.append(b"pub(super) " + line)
        elif re.match(rb"^    [a-z_][a-zA-Z0-9_]*: ", line):
            out.append(b"    pub(super) " + line[4:])
        elif line.startswith(b"    fn matches("):
            out.append(b"    pub(super) " + line[4:])
        else:
            out.append(line)
    return b"\n".join(out)


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    mod_head = widen_types(b"\n".join(lines[0:276]))   # 1-276
    compile_body = widen_free_fns(b"\n".join(lines[276:569]))  # 277-569
    parse_body = widen_free_fns(b"\n".join(lines[569:722]))    # 570-722
    matcher_body = widen_free_fns(b"\n".join(lines[722:992]))  # 723-992
    tests = b"\n".join(lines[992:])                    # 993-end

    os.makedirs(OUTDIR, exist_ok=True)

    mod_bytes = (
        mod_head + b"\n\n"
        + b"mod compile;\nmod matcher;\nmod parse;\n\n"
        + b"use compile::compile_pattern;\n"
        + b"use matcher::try_match_at;\n\n"
        + tests + b"\n"
    )
    compile_bytes = SUB_IMPORTS + b"use super::parse::{apply_quantifier, parse_char_class};\n\n" + compile_body + b"\n"
    parse_bytes = SUB_IMPORTS + parse_body + b"\n"
    matcher_bytes = SUB_IMPORTS + matcher_body + b"\n"

    with open(os.path.join(OUTDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(OUTDIR, "compile.rs"), "wb") as f:
        f.write(compile_bytes)
    with open(os.path.join(OUTDIR, "parse.rs"), "wb") as f:
        f.write(parse_bytes)
    with open(os.path.join(OUTDIR, "matcher.rs"), "wb") as f:
        f.write(matcher_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} compile {len(compile_bytes)} parse {len(parse_bytes)} matcher {len(matcher_bytes)}")
    print("removed original regex.rs")


if __name__ == "__main__":
    main()
