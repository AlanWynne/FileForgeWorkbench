#!/usr/bin/env python3
"""Extract 6 heavy trait-method bodies from contraction_state/mod.rs into
inherent pub(super) helpers in contraction_state/ops.rs, leaving thin delegators.

For each target method NAME in the `impl DisplayLineMapping for ContractionState`
block, we:
  - locate `    fn NAME(<sig>) <ret> {` and its matching closing `    }`
  - move the whole `fn NAME(<sig>) <ret> { <body> }` into ops.rs as
    `    pub(super) fn NAME_impl(<sig>) <ret> { <body> }`
  - replace the trait method with a thin delegator:
    `    fn NAME(<sig>) <ret> { self.NAME_impl(<call-args>) }`

Delegators are declared explicitly per method (below) to keep call-arg extraction
exact and reviewable -- no signature guessing.
"""
import os

MOD = os.path.join("crates", "ff-display-line-mapping", "src", "contraction_state", "mod.rs")
OPS = os.path.join("crates", "ff-display-line-mapping", "src", "contraction_state", "ops.rs")

# name -> (full signature line(s) as they appear, delegator body)
# We match the `fn NAME(` start and find the matching brace to capture the block.
TARGETS = {
    "doc_from_display":
        "    fn doc_from_display(&self, display_line: DisplayLine) -> DocPosition {\n"
        "        self.doc_from_display_impl(display_line)\n    }\n",
    "set_visible":
        "    fn set_visible(&mut self, start: DocLine, end: DocLine, visible: bool) -> bool {\n"
        "        self.set_visible_impl(start, end, visible)\n    }\n",
    "set_expanded":
        "    fn set_expanded(&mut self, doc_line: DocLine, expanded: bool) -> bool {\n"
        "        self.set_expanded_impl(doc_line, expanded)\n    }\n",
    "set_height":
        "    fn set_height(&mut self, doc_line: DocLine, height: u32) -> bool {\n"
        "        self.set_height_impl(doc_line, height)\n    }\n",
    "insert_lines":
        "    fn insert_lines(&mut self, doc_line: DocLine, count: usize) {\n"
        "        self.insert_lines_impl(doc_line, count)\n    }\n",
    "delete_lines":
        "    fn delete_lines(&mut self, doc_line: DocLine, count: usize) {\n"
        "        self.delete_lines_impl(doc_line, count)\n    }\n",
}

ORDER = ["doc_from_display", "set_visible", "set_expanded", "set_height",
         "insert_lines", "delete_lines"]


def find_method_block(text, name):
    """Return (start_idx, end_idx) of the `    fn NAME(` ... matching `    }\n`."""
    needle = f"    fn {name}("
    start = text.index(needle)
    # find opening brace of the fn (first '{' at end of signature, before body)
    i = text.index("{", start)
    depth = 0
    j = i
    while j < len(text):
        c = text[j]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                # include trailing newline
                end = j + 1
                if end < len(text) and text[end] == "\n":
                    end += 1
                return start, end
        j += 1
    raise RuntimeError(f"unbalanced braces for {name}")


def main():
    with open(MOD, "r", encoding="utf-8") as f:
        mod = f.read()
    with open(OPS, "r", encoding="utf-8") as f:
        ops = f.read()

    helpers = []
    for name in ORDER:
        start, end = find_method_block(mod, name)
        block = mod[start:end]  # `    fn NAME(...) ... {\n ... \n    }\n`
        # inherent helper: rename fn NAME( -> pub(super) fn NAME_impl(
        helper = block.replace(f"    fn {name}(", f"    pub(super) fn {name}_impl(", 1)
        helpers.append(helper)
        # replace trait method with the thin delegator
        mod = mod[:start] + TARGETS[name] + mod[end:]

    # Insert helpers into ops.rs inside the `impl ContractionState {\n ... \n}` block.
    close = "\nimpl ContractionState {\n}\n"
    filled = "\nimpl ContractionState {\n" + "\n".join(helpers) + "}\n"
    assert close in ops, "ops.rs impl scaffold not found"
    ops = ops.replace(close, filled, 1)

    with open(MOD, "w", encoding="utf-8", newline="") as f:
        f.write(mod)
    with open(OPS, "w", encoding="utf-8", newline="") as f:
        f.write(ops)
    print(f"extracted {len(ORDER)} helpers; mod now {len(mod)} bytes, ops {len(ops)} bytes")


if __name__ == "__main__":
    main()
