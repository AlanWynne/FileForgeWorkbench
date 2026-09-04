"""Phase BZ documentation patch script.

Updates:
1. docs/specs/menu-and-statusbar/tasks.md  -- mark Tasks 24-30 [x]
2. docs/quality/TCR.md                     -- mark BZ rows as covered
3. docs/specs/project-master/tasks.md      -- mark BZ entries [x]
"""

import os

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"
os.makedirs(os.path.dirname(LOG), exist_ok=True)
with open(LOG, "w", encoding="utf-8") as f:
    f.write("")


def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")


def patch_binary(path, replacements):
    with open(path, "rb") as f:
        data = f.read()
    log(f"  File size: {len(data)} bytes")
    for old, new in replacements:
        for sep in (b"\r\n", b"\n"):
            old_s = old.replace(b"\n", sep)
            new_s = new.replace(b"\n", sep)
            if old_s in data:
                data = data.replace(old_s, new_s, 1)
                log(f"  Patched (sep={repr(sep)}): {old[:50]!r}...")
                break
        else:
            log(f"  WARNING: pattern not found: {old[:60]!r}")
    with open(path, "wb") as f:
        f.write(data)


# ── 1. menu-and-statusbar/tasks.md ──────────────────────────────────────────
log("=== Patching menu-and-statusbar/tasks.md ===")
tasks_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\menu-and-statusbar\tasks.md"

patch_binary(tasks_path, [
    (b"- [ ] 24. SCROLL ===> field adjacent to Command ===> field",
     b"- [x] 24. SCROLL ===> field adjacent to Command ===> field"),
    (b"  - [ ] 24.1 Add `scroll_amount: ScrollAmount` field to `WorkbenchShell` state",
     b"  - [x] 24.1 Add `scroll_amount: ScrollAmount` field to `WorkbenchShell` state"),
    (b"  - [ ] 24.2 Render `SCROLL ===>` input field to the right of `Command ===>` in the command area",
     b"  - [x] 24.2 Render `SCROLL ===>` input field to the right of `Command ===>` in the command area"),
    (b"  - [ ] 24.3 On Enter in SCROLL field, update active scroll amount for current panel",
     b"  - [x] 24.3 On Enter in SCROLL field, update active scroll amount for current panel"),
    (b"  - [ ] 24.4 Persist scroll amount across command submissions and panel switches within session",
     b"  - [x] 24.4 Persist scroll amount across command submissions and panel switches within session"),
    (b"  - [ ] 24.5 Write unit tests for scroll field rendering, value update, and session retention",
     b"  - [x] 24.5 Write unit tests for scroll field rendering, value update, and session retention"),
    (b"- [ ] 25. Fastpath notation (dotted option path)",
     b"- [x] 25. Fastpath notation (dotted option path)"),
    (b"  - [ ] 25.1 Extend CommandEngine parser to recognise dotted notation",
     b"  - [x] 25.1 Extend CommandEngine parser to recognise dotted notation"),
    (b"  - [ ] 25.2 Implement fastpath navigation: resolve each dot-separated segment",
     b"  - [x] 25.2 Implement fastpath navigation: resolve each dot-separated segment"),
    (b"  - [ ] 25.3 Write unit tests for `3.1` navigating to POM option 3 sub-option 1",
     b"  - [x] 25.3 Write unit tests for `3.1` navigating to POM option 3 sub-option 1"),
    (b"- [ ] 26. Data entry panel and list panel layout conformance",
     b"- [x] 26. Data entry panel and list panel layout conformance"),
    (b"  - [ ] 26.1 Define `DataEntryPanel` layout contract",
     b"  - [x] 26.1 Define `DataEntryPanel` layout contract"),
    (b"  - [ ] 26.2 Define `ListPanel` layout contract",
     b"  - [x] 26.2 Define `ListPanel` layout contract"),
    (b"  - [ ] 26.3 Audit existing dialogs",
     b"  - [x] 26.3 Audit existing dialogs"),
    (b"  - [ ] 26.4 Write unit tests verifying panel layout structs expose required elements",
     b"  - [x] 26.4 Write unit tests verifying panel layout structs expose required elements"),
    (b"- [ ] 27. List panel LOCATE command",
     b"- [x] 27. List panel LOCATE command"),
    (b"  - [ ] 27.1 Implement `LOCATE name` handler for list panels",
     b"  - [x] 27.1 Implement `LOCATE name` handler for list panels"),
    (b"  - [ ] 27.2 Implement partial-name matching",
     b"  - [x] 27.2 Implement partial-name matching"),
    (b"  - [ ] 27.3 Write unit tests for exact match, partial match, no-match",
     b"  - [x] 27.3 Write unit tests for exact match, partial match, no-match"),
    (b"- [ ] 28. Extended scroll amounts (HALF, CSR, MAX, DATA)",
     b"- [x] 28. Extended scroll amounts (HALF, CSR, MAX, DATA)"),
    (b"  - [ ] 28.1 Extend `ScrollAmount` enum with `Half`, `Csr`, `Max`, `Data` variants",
     b"  - [x] 28.1 Extend `ScrollAmount` enum with `Half`, `Csr`, `Max`, `Data` variants"),
    (b"  - [ ] 28.2 Implement scroll distance calculation for each new variant",
     b"  - [x] 28.2 Implement scroll distance calculation for each new variant"),
    (b"  - [ ] 28.3 Write unit tests for each scroll amount variant",
     b"  - [x] 28.3 Write unit tests for each scroll amount variant"),
    (b"- [ ] 29. Split screen (PF2/PF9/PF3)",
     b"- [x] 29. Split screen (PF2/PF9/PF3)"),
    (b"  - [ ] 29.1 Add `split_screen: Option<SplitScreenState>` to `WorkbenchShell`",
     b"  - [x] 29.1 Add `split_screen: Option<SplitScreenState>` to `WorkbenchShell`"),
    (b"  - [ ] 29.2 On PF2: split active editor at cursor line into two independent halves",
     b"  - [x] 29.2 On PF2: split active editor at cursor line into two independent halves"),
    (b"  - [ ] 29.3 On PF9: swap focus between split halves",
     b"  - [x] 29.3 On PF9: swap focus between split halves"),
    (b"  - [ ] 29.4 On PF3 (END) while split: unsplit and restore single-panel view",
     b"  - [x] 29.4 On PF3 (END) while split: unsplit and restore single-panel view"),
    (b"  - [ ] 29.5 Each half maintains independent command field, scroll position, and cursor state",
     b"  - [x] 29.5 Each half maintains independent command field, scroll position, and cursor state"),
    (b"  - [ ] 29.6 Write unit tests for split/swap/unsplit state transitions",
     b"  - [x] 29.6 Write unit tests for split/swap/unsplit state transitions"),
    (b"- [ ] 30. TCR update for Requirement 19",
     b"- [x] 30. TCR update for Requirement 19"),
    (b"  - [ ] 30.1 Update docs/quality/TCR.md -- mark all Req 19.1-19.14 rows as covered once tests pass",
     b"  - [x] 30.1 Update docs/quality/TCR.md -- mark all Req 19.1-19.14 rows as covered once tests pass"),
])

# ── 2. TCR.md ────────────────────────────────────────────────────────────────
log("=== Patching TCR.md ===")
tcr_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"

patch_binary(tcr_path, [
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.1: SCROLL ===> field rendered adjacent to Command ===> field |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `scroll_amount.rs` + `shell/render.rs` unit tests | Req 19.1: SCROLL ===> field rendered adjacent to Command ===> field |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.2: SCROLL field value update on Enter sets active scroll amount |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 19.2: SCROLL field value update on Enter sets active scroll amount |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.3: SCROLL field value retained across command submissions and panel switches |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 19.3: SCROLL field value retained across command submissions and panel switches |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.4: fastpath notation (e.g., 3.1) navigates directly to nested option |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 19.4: fastpath notation (e.g., 3.1) navigates directly to nested option |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.5: data entry panel conforms to ISPF layout (title, command, ===> fields, key bar) |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `panel_layout.rs` unit tests | Req 19.5: data entry panel conforms to ISPF layout (title, command, ===> fields, key bar) |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.6: list panel conforms to ISPF layout (title, command, filter lines, NP column, rows) |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `panel_layout.rs` unit tests | Req 19.6: list panel conforms to ISPF layout (title, command, filter lines, NP column, rows) |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.7: LOCATE on list panel scrolls to nearest alphabetic match |",
     b"| `ff-desktop` | \xe2\x9c\x85 | existing `nav_manager` LOCATE tests | Req 19.7: LOCATE on list panel scrolls to nearest alphabetic match |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.8: LOCATE accepts partial names on list panel |",
     b"| `ff-desktop` | \xe2\x9c\x85 | existing `nav_manager` LOCATE tests | Req 19.8: LOCATE accepts partial names on list panel |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.9: LOCATE scrolls panel so matching item is visible |",
     b"| `ff-desktop` | \xf0\x9f\x94\xb2 | -- | Req 19.9: LOCATE scrolls panel so matching item is visible (manual UI verification) |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.10: scroll amounts HALF/CSR/MAX/DATA supported in all panel scroll commands |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `scroll_amount.rs` unit tests | Req 19.10: scroll amounts HALF/CSR/MAX/DATA supported in all panel scroll commands |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.11: PF2 splits screen at cursor line into two independent halves |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 19.11: PF2 splits screen at cursor line into two independent halves |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.12: PF9 swaps focus between split-screen halves |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 19.12: PF9 swaps focus between split-screen halves |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.13: each split-screen half operates independently |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 19.13: each split-screen half operates independently |"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | -- | Req 19.14: END (PF3) while split unsplits the screen |",
     b"| `ff-desktop` | \xe2\x9c\x85 | `shell/tests.rs` unit tests | Req 19.14: END (PF3) while split unsplits the screen |"),
])

# ── 3. project-master/tasks.md ───────────────────────────────────────────────
log("=== Patching project-master/tasks.md ===")
master_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"

with open(master_path, "rb") as f:
    master_data = f.read()
log(f"  File size: {len(master_data)} bytes")

for old, new in [
    (b"- [ ] BZ.1", b"- [x] BZ.1"),
    (b"- [ ] BZ.2", b"- [x] BZ.2"),
    (b"- [ ] BZ.3", b"- [x] BZ.3"),
    (b"- [ ] BZ.4", b"- [x] BZ.4"),
    (b"- [ ] BZ.5", b"- [x] BZ.5"),
    (b"- [ ] BZ.6", b"- [x] BZ.6"),
    (b"- [ ] BZ.7", b"- [x] BZ.7"),
    (b"- [ ] BZ.impl", b"- [x] BZ.impl"),
]:
    if old in master_data:
        master_data = master_data.replace(old, new, 1)
        log(f"  Patched: {old!r}")
    else:
        log(f"  Not found: {old!r}")

with open(master_path, "wb") as f:
    f.write(master_data)

log("=== Done ===")
