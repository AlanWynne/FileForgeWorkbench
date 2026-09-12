#!/usr/bin/env python3
"""Byte-faithful splitter for ff-config/src/editorconfig/parser.rs (PA-STD-002 file 2/6, 625nt).

SPLITTABLE: data types + free fns (only trivial impl Display/Error for ParseError).
2-way by concern:
  parser/mod.rs : doc + imports + data types (EditorConfigFile/Section/Properties/
                  IndentStyle/IndentSize/EndOfLine/Charset/ParseError + Display/Error)
                  + the INI parse cluster (parse/parse_key_value/parse_bool/parse_u32/
                  apply_property/load_editorconfig_file) + mod glob + re-export + tests
  parser/glob.rs: the self-contained glob/brace-expansion engine (matches_pattern +
                  expand_braces/find_matching_close_brace/split_brace_alternatives/
                  parse_integer_range/glob_match/glob_match_recursive/find_class_end/
                  match_character_class)

The glob cluster (lines 125-407) has ZERO crate::/super::/use refs -- pure str/byte
logic, no imports needed. Only `matches_pattern` is public API (re-exported at the
editorconfig level via `pub use parser::{... matches_pattern ...}`), so parser/mod.rs
does `pub use glob::matches_pattern;`. The other glob fns stay private to glob.rs.

Source layout (1-based):
  1-124   : doc + `use std::path::Path;` + all data types + ParseError
            + impl Display for ParseError + impl Error for ParseError  (mod.rs)
  125-407 : glob/brace engine (matches_pattern + 8 private helpers)     -> glob.rs
  408-625 : parse cluster                                                (mod.rs)
  626-end : tests                                                        (mod.rs)
"""
import os

SRC = os.path.join("crates", "ff-config", "src", "editorconfig", "parser.rs")
NEWDIR = os.path.join("crates", "ff-config", "src", "editorconfig", "parser")


def main():
    with open(SRC, "rb") as f:
        data = f.read()
    lines = data.split(b"\n")

    head_types = b"\n".join(lines[0:124])    # 1-124
    glob_cluster = b"\n".join(lines[124:407])  # 125-407
    parse_cluster = b"\n".join(lines[407:625])  # 408-625
    tests = b"\n".join(lines[625:])            # 626-end

    os.makedirs(NEWDIR, exist_ok=True)

    mod_bytes = (
        head_types + b"\n\n"
        + b"mod glob;\n"
        + b"pub use glob::matches_pattern;\n\n"
        + parse_cluster + b"\n\n"
        + tests + b"\n"
    )
    # glob cluster is pure str/byte logic: no imports required.
    glob_bytes = (
        b"//! Glob and brace-expansion matching for EditorConfig section patterns.\n"
        b"//!\n"
        b"//! Self-contained str/byte matching engine used by `matches_pattern`.\n\n"
        + glob_cluster + b"\n"
    )

    with open(os.path.join(NEWDIR, "mod.rs"), "wb") as f:
        f.write(mod_bytes)
    with open(os.path.join(NEWDIR, "glob.rs"), "wb") as f:
        f.write(glob_bytes)

    os.remove(SRC)
    print(f"mod {len(mod_bytes)} glob {len(glob_bytes)}")
    print("removed original parser.rs")


if __name__ == "__main__":
    main()
