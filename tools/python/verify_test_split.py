"""Verify the shell/tests.rs split preserved every #[test] function name.

Compares the set of test-fn names in the pre-split tests.rs (git HEAD) against
the union of test-fn names across the new shell/tests_*.rs files.

Usage: python tools/python/verify_test_split.py
Output: tools/logs/verify_test_split.txt (also printed).
Read-only: runs `git show` and reads working-tree files; writes only its log.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(r"c:\workspace\VSC\FileForgeWorkbench")
LOG = ROOT / "tools" / "logs" / "verify_test_split.txt"
LOG.parent.mkdir(parents=True, exist_ok=True)

# Match an fn name on the line that follows a #[test] attribute.
FN_RE = re.compile(r"^\s*(?:async\s+)?fn\s+([A-Za-z0-9_]+)\s*\(")
TEST_ATTR_RE = re.compile(r"^\s*#\[(?:test|tokio::test|.*kittest.*)\]\s*$")


def log(msg: str) -> None:
    print(msg)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")


def extract_test_names(text: str) -> list[str]:
    """A test fn is any fn whose immediately-preceding non-doc line is #[test]."""
    names: list[str] = []
    lines = text.splitlines()
    pending_test = False
    for line in lines:
        stripped = line.strip()
        if stripped == "#[test]":
            pending_test = True
            continue
        if pending_test:
            m = FN_RE.match(line)
            if m:
                names.append(m.group(1))
                pending_test = False
            elif stripped.startswith("#["):
                # another attribute between #[test] and fn; keep waiting
                continue
            elif stripped == "" or stripped.startswith("//"):
                continue
            else:
                # not a fn line; drop the pending flag
                pending_test = False
    return names


def main() -> int:
    open(LOG, "w", encoding="utf-8").close()
    log("=== Verify shell/tests.rs split: test-name invariant ===")

    # BEFORE: git HEAD version of the deleted file.
    before_text = subprocess.run(
        ["git", "-C", str(ROOT), "show", "HEAD:crates/ff-desktop/src/shell/tests.rs"],
        capture_output=True, text=True, encoding="utf-8",
    ).stdout
    before = extract_test_names(before_text)

    # AFTER: union across the new split files.
    after: list[str] = []
    per_file = {}
    shell_dir = ROOT / "crates" / "ff-desktop" / "src" / "shell"
    for p in sorted(shell_dir.glob("tests_*.rs")):
        names = extract_test_names(p.read_text(encoding="utf-8"))
        per_file[p.name] = len(names)
        after.extend(names)

    before_set = set(before)
    after_set = set(after)

    log(f"BEFORE #[test] count (git HEAD tests.rs): {len(before)} (unique {len(before_set)})")
    log(f"AFTER  #[test] count (union tests_*.rs):  {len(after)} (unique {len(after_set)})")
    log("Per-file after counts:")
    for name, cnt in per_file.items():
        log(f"  {name}: {cnt}")

    missing = sorted(before_set - after_set)
    added = sorted(after_set - before_set)
    dupes_before = sorted({n for n in before if before.count(n) > 1})
    dupes_after = sorted({n for n in after if after.count(n) > 1})

    log(f"MISSING (in before, not after): {len(missing)}")
    for n in missing:
        log(f"  - {n}")
    log(f"ADDED (in after, not before): {len(added)}")
    for n in added:
        log(f"  + {n}")
    log(f"Duplicate names BEFORE: {dupes_before}")
    log(f"Duplicate names AFTER:  {dupes_after}")

    ok = (not missing and not added and len(before) == len(after)
          and not dupes_before and not dupes_after)
    log(f"RESULT: {'PASS -- identical test-name set and count' if ok else 'FAIL -- divergence detected'}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
