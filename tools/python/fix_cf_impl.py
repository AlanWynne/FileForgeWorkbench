"""Mark CF.impl tasks complete in syntax-highlighting/tasks.md and project-master/tasks.md."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\cf-impl-fixes.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

def replace_in_file(path, replacements):
    with open(path, "rb") as f:
        data = f.read()
    log(f"  File size: {len(data)} bytes")
    count = 0
    for old_str, new_str in replacements:
        for sep in (b"\r\n", b"\n"):
            old = old_str.replace(b"\n", sep)
            new = new_str.replace(b"\n", sep)
            if old in data:
                data = data.replace(old, new)
                count += 1
                log(f"  Replaced (sep={repr(sep)}): {old_str[:60]}")
                break
        else:
            log(f"  WARNING: pattern not found: {old_str[:60]}")
    with open(path, "wb") as f:
        f.write(data)
    log(f"  {count}/{len(replacements)} replacements written")
    return count

log("=== CF.impl doc fixes ===")

# --- syntax-highlighting/tasks.md ---
sh_tasks = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\syntax-highlighting\tasks.md"
log(f"\nProcessing: {sh_tasks}")

sh_replacements = [
    (
        b"- [ ] 21. HILITE ON/OFF command and HILITE LOGIC mode",
        b"- [x] 21. HILITE ON/OFF command and HILITE LOGIC mode",
    ),
    (
        b"  - [ ] 21.1 Register HILITE primary command with ON/OFF operand parsing",
        b"  - [x] 21.1 Register HILITE primary command with ON/OFF operand parsing",
    ),
    (
        b"  - [ ] 21.2 Implement HILITE ON -- re-enable syntax highlighting for active document",
        b"  - [x] 21.2 Implement HILITE ON -- re-enable syntax highlighting for active document",
    ),
    (
        b"  - [ ] 21.3 Implement HILITE OFF -- disable syntax highlighting, revert all text to default style 0",
        b"  - [x] 21.3 Implement HILITE OFF -- disable syntax highlighting, revert all text to default style 0",
    ),
    (
        b"  - [ ] 21.4 Implement HILITE LOGIC mode -- highlight boolean and comparison operators with HILITE_LOGIC style slot",
        b"  - [x] 21.4 Implement HILITE LOGIC mode -- highlight boolean and comparison operators with HILITE_LOGIC style slot",
    ),
    (
        b"  - [ ] 21.5 Write unit tests for HILITE ON/OFF toggle, HILITE LOGIC operator detection, style slot assignment",
        b"  - [x] 21.5 Write unit tests for HILITE ON/OFF toggle, HILITE LOGIC operator detection, style slot assignment",
    ),
    (
        b"- [ ] 22. HILITE PAREN, HILITE FIND, and combined operands",
        b"- [x] 22. HILITE PAREN, HILITE FIND, and combined operands",
    ),
    (
        b"  - [ ] 22.1 Implement HILITE PAREN mode -- highlight enclosing delimiter pair at cursor; update on cursor move",
        b"  - [x] 22.1 Implement HILITE PAREN mode -- highlight enclosing delimiter pair at cursor; update on cursor move",
    ),
    (
        b"  - [ ] 22.2 Implement HILITE_PAREN_ERROR style for mismatched delimiters",
        b"  - [x] 22.2 Implement HILITE_PAREN_ERROR style for mismatched delimiters",
    ),
    (
        b"  - [ ] 22.3 Implement HILITE FIND -- persist find-match highlights for most recent FIND string; HILITE FIND OFF clears",
        b"  - [x] 22.3 Implement HILITE FIND -- persist find-match highlights for most recent FIND string; HILITE FIND OFF clears",
    ),
    (
        b"  - [ ] 22.4 Implement combined operands: HILITE ON LOGIC PAREN enables multiple modes simultaneously; modes toggle independently",
        b"  - [x] 22.4 Implement combined operands: HILITE ON LOGIC PAREN enables multiple modes simultaneously; modes toggle independently",
    ),
    (
        b"  - [ ] 22.5 Write unit tests for HILITE PAREN (match/mismatch), HILITE FIND (set/clear), combined operand parsing",
        b"  - [x] 22.5 Write unit tests for HILITE PAREN (match/mismatch), HILITE FIND (set/clear), combined operand parsing",
    ),
]

replace_in_file(sh_tasks, sh_replacements)

# --- project-master/tasks.md ---
pm_tasks = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"
log(f"\nProcessing: {pm_tasks}")

pm_replacements = [
    (
        b"- [ ] CF.impl syntax-highlighting: HILITE ON/OFF/LOGIC/PAREN/FIND (Tasks 21-22 in syntax-highlighting/tasks.md)",
        b"- [x] CF.impl syntax-highlighting: HILITE ON/OFF/LOGIC/PAREN/FIND (Tasks 21-22 in syntax-highlighting/tasks.md)",
    ),
]

replace_in_file(pm_tasks, pm_replacements)

log("\nDone.")
