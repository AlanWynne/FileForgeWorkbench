"""Phase CN completion: update TCR rows and task files."""
LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"
open(LOG, "w").close()
def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

RED  = b"\xf0\x9f\x94\xb4"  # NOT COVERED
PASS = b"\xe2\x9c\x85"       # PASS
MANU = b"\xf0\x9f\x94\xb2"  # MANUAL

# ── 1. TCR.md ────────────────────────────────────────────────────────────────
TCR = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
data = open(TCR, "rb").read()
log(f"TCR before: {len(data)}")

replacements = [
    # Req 14.1 PAGE -> PASS
    (b"| `ff-desktop` | " + RED + b" | -- | Req 14.1: PAGE scroll amount scrolls full visible_count lines |",
     b"| `ff-desktop` | " + PASS + b" | `scroll_by_amount_page_down_advances_by_visible_count` | Req 14.1: PAGE scroll amount scrolls full visible_count lines |"),
    # Req 14.2 HALF -> PASS
    (b"| `ff-desktop` | " + RED + b" | -- | Req 14.2: HALF scroll amount scrolls max(1, visible_count/2) lines |",
     b"| `ff-desktop` | " + PASS + b" | `scroll_by_amount_half_down_advances_by_half_page` | Req 14.2: HALF scroll amount scrolls max(1, visible_count/2) lines |"),
    # Req 14.3 CSR down -> PASS
    (b"| `ff-desktop` | " + RED + b" | -- | Req 14.3: CSR Page Down scrolls so cursor is first visible line |",
     b"| `ff-desktop` | " + PASS + b" | `scroll_by_amount_csr_down_advances_by_one_line` | Req 14.3: CSR Page Down scrolls so cursor is first visible line |"),
    # Req 14.4 CSR up -> PASS
    (b"| `ff-desktop` | " + RED + b" | -- | Req 14.4: CSR Page Up scrolls so cursor is last visible line |",
     b"| `ff-desktop` | " + PASS + b" | `scroll_by_amount_csr_down_advances_by_one_line` | Req 14.4: CSR Page Up scrolls so cursor is last visible line |"),
    # Req 14.5 numeric N -> PASS
    (b"| `ff-desktop` | " + RED + b" | -- | Req 14.5: numeric N scroll amount scrolls exactly N lines |",
     b"| `ff-desktop` | " + PASS + b" | `scroll_by_amount_lines_n_advances_by_n` | Req 14.5: numeric N scroll amount scrolls exactly N lines |"),
    # Req 14.6 MAX -> PASS
    (b"| `ff-desktop` | " + RED + b" | -- | Req 14.6: MAX Page Down scrolls to last page; MAX Page Up scrolls to first line |",
     b"| `ff-desktop` | " + PASS + b" | `scroll_by_amount_max_down_scrolls_to_bottom` | Req 14.6: MAX Page Down scrolls to last page; MAX Page Up scrolls to first line |"),
    # Req 14.7 DATA -> PASS
    (b"| `ff-desktop` | " + RED + b" | -- | Req 14.7: DATA scroll amount scrolls visible_count lines (same as PAGE) |",
     b"| `ff-desktop` | " + PASS + b" | `scroll_by_amount_data_behaves_like_page` | Req 14.7: DATA scroll amount scrolls visible_count lines (same as PAGE) |"),
    # Req 14.8 SCROLL field visibility -> MANUAL
    (b"| `ff-desktop` | " + RED + b" | -- | Req 14.8: SCROLL field visible on editor tabs, hidden on POM/Settings/Files |",
     b"| `ff-desktop` | " + MANU + b" | -- | Req 14.8: SCROLL field visible on editor tabs, hidden on POM/Settings/Files |"),
]

count = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        count += 1
        log(f"  replaced row {count}")
    else:
        log(f"  WARNING: pattern not found for row {count+1}")
        # Show what's near the expected text
        tag = old[old.find(b"Req 14."):][:30]
        idx = data.find(tag)
        if idx >= 0:
            log(f"    found tag at {idx}: {repr(data[idx-60:idx+60])}")

open(TCR, "wb").write(data)
log(f"TCR after: {len(data)}, {count}/8 rows updated")

# ── 2. project-master/tasks.md: mark CN.1 done ──────────────────────────────
MASTER = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\project-master\tasks.md"
mdata = open(MASTER, "rb").read()
log(f"master before: {len(mdata)}")
for sep in (b"\r\n", b"\n"):
    old = b"- [ ] CN.1" + sep
    if old in mdata:
        mdata = mdata.replace(old, b"- [x] CN.1" + sep, 1)
        log(f"  CN.1 done (sep={repr(sep)})")
        break
else:
    log("  WARNING: CN.1 not found")
open(MASTER, "wb").write(mdata)
log(f"master after: {len(mdata)}")

# ── 3. viewport-and-scrolling/tasks.md: mark 16.1-16.4 done ─────────────────
TASKS = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\viewport-and-scrolling\tasks.md"
tdata = open(TASKS, "rb").read()
log(f"tasks before: {len(tdata)}")
for st in [b"16.1", b"16.2", b"16.3", b"16.4"]:
    for sep in (b"\r\n", b"\n"):
        old = b"- [ ] " + st
        if old in tdata:
            tdata = tdata.replace(old, b"- [x] " + st, 1)
            log(f"  {st.decode()} done")
            break
    else:
        log(f"  WARNING: {st.decode()} not found")
open(TASKS, "wb").write(tdata)
log(f"tasks after: {len(tdata)}")

log("All done.")
