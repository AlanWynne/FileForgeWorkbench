import sys, os

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

# Clear log
open(LOG, "w").close()
log("CR TCR/tasks update script started")

# === 1. Update TCR.md: flip 16.1-16.4 and 12.1-12.5 rows from NOT COVERED to PASS ===
tcr_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
with open(tcr_path, "rb") as f:
    data = f.read()

log(f"TCR.md size: {len(data)} bytes")

replacements = [
    # theme-and-appearance Req 16.x (OS follow)
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | \xe2\x80\x94 | Req 16.1:", b"| `ff-desktop` | \xe2\x9c\x85 | theme_follow_os_key_is_registered_in_schema | Req 16.1:"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | \xe2\x80\x94 | Req 16.2:", b"| `ff-desktop` | \xe2\x9c\x85 | theme_follow_os_defaults_to_false | Req 16.2:"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | \xe2\x80\x94 | Req 16.3:", b"| `ff-desktop` | \xe2\x9c\x85 | theme_follow_os_can_be_set_to_true | Req 16.3:"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | \xe2\x80\x94 | Req 16.4:", b"| `ff-desktop` | \xf0\x9f\x94\xb2 | manual | Req 16.4:"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | \xe2\x80\x94 | Req 16.5:", b"| `ff-desktop` | \xe2\x9c\x85 | theme_follow_os_false_does_not_change_palette | Req 16.5:"),
    # lua-macro-engine Req 12.x (macro library)
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | \xe2\x80\x94 | Req 12.1:", b"| `ff-desktop` | \xe2\x9c\x85 | option_6_routes_to_macro_library | Req 12.1:"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | \xe2\x80\x94 | Req 12.2:", b"| `ff-desktop` | \xe2\x9c\x85 | refresh_discovers_lua_files_in_directory | Req 12.2:"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | \xe2\x80\x94 | Req 12.3:", b"| `ff-desktop` | \xf0\x9f\x94\xb2 | manual | Req 12.3:"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | \xe2\x80\x94 | Req 12.4:", b"| `ff-desktop` | \xe2\x9c\x85 | filter_narrows_visible_entries | Req 12.4:"),
    (b"| `ff-desktop` | \xf0\x9f\x94\xb4 | \xe2\x80\x94 | Req 12.5:", b"| `ff-desktop` | \xe2\x9c\x85 | macro_library_tab_not_persisted_in_session | Req 12.5:"),
]

changed = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        changed += 1
        log(f"  Replaced: {old[:50]}")
    else:
        log(f"  NOT FOUND: {old[:50]}")

with open(tcr_path, "wb") as f:
    f.write(data)
log(f"TCR.md updated: {changed} replacements")

# === 2. Update theme-and-appearance/tasks.md: mark Task 20 subtasks done ===
theme_tasks = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\theme-and-appearance\tasks.md"
with open(theme_tasks, "rb") as f:
    td = f.read()

log(f"theme tasks size: {len(td)} bytes")

# Mark subtasks 20.1-20.5 as done
for sub in [b"20.1", b"20.2", b"20.3", b"20.4", b"20.5"]:
    old = b"- [ ] " + sub
    new = b"- [x] " + sub
    if old in td:
        td = td.replace(old, new)
        log(f"  Checked: {sub}")
    else:
        log(f"  NOT FOUND: {sub}")

with open(theme_tasks, "wb") as f:
    f.write(td)
log("theme-and-appearance/tasks.md updated")

# === 3. Update lua-macro-engine/tasks.md: mark Task 25 subtasks done ===
lua_tasks = r"C:\workspace\VSC\FileForgeWorkbench\docs\specs\lua-macro-engine\tasks.md"
with open(lua_tasks, "rb") as f:
    ld = f.read()

log(f"lua tasks size: {len(ld)} bytes")

for sub in [b"25.1", b"25.2", b"25.3", b"25.4", b"25.5", b"25.6", b"25.7", b"25.8", b"25.9", b"25.10"]:
    old = b"- [ ] " + sub
    new = b"- [x] " + sub
    if old in ld:
        ld = ld.replace(old, new)
        log(f"  Checked: {sub}")
    else:
        log(f"  NOT FOUND: {sub}")

with open(lua_tasks, "wb") as f:
    f.write(ld)
log("lua-macro-engine/tasks.md updated")

log("Done")
