"""Gate step 6: TCR rows for Phase CR + fix CQ ? rows. Also update change-log and current-work."""
import re

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\cr_gate_docs.txt"
with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

log("=== cr_gate_docs.py ===")

# -----------------------------------------------------------------------
# 1. TCR.md -- fix CQ ? rows to checkmark, append CR rows
# -----------------------------------------------------------------------
tcr_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
tcr = open(tcr_path, encoding="utf-8").read()
log(f"TCR size: {len(tcr)}")

# Fix CQ rows: replace "| ? |" with checkmark in rows that contain "Req 16." or "Req 17." or "Req 18."
# These rows currently show "?" as status
fixed = 0
lines = tcr.split("\n")
new_lines = []
for line in lines:
    if "| ? |" in line and ("Req 16." in line or "Req 17." in line or "Req 18." in line):
        line = line.replace("| ? |", "| \u2705 |", 1)
        fixed += 1
    new_lines.append(line)
tcr = "\n".join(new_lines)
log(f"Fixed {fixed} CQ TCR rows to checkmark")

# Append new CR rows
cr_rows = """
### Phase CR -- OS Theme Follow + Macro Library Management

| Crate | Status | Test | Requirement |
|-------|--------|------|-------------|
| `ff-desktop` | \U0001f534 | -- | Req 16.1: theme.follow_os config key (boolean, default false) |
| `ff-desktop` | \U0001f534 | -- | Req 16.2: follow_os=true + OS dark -> Visual_Mode set to Dark |
| `ff-desktop` | \U0001f534 | -- | Req 16.3: follow_os=true + OS light -> Visual_Mode set to Light |
| `ff-desktop` | \U0001f534 | -- | Req 16.4: follow_os=false -> OS preference ignored; theme.mode used |
| `ff-desktop` | \U0001f534 | -- | Req 16.5: OS preference change detected within one egui frame |
| `ff-desktop` | \U0001f534 | -- | Req 16.6: OS preference read from egui ctx.style().visuals.dark_mode |
| `ff-desktop` | \U0001f534 | -- | Req 16.7: auto-applied mode not persisted to theme.mode config key |
| `ff-desktop` | \U0001f534 | -- | Req 16.8: Settings panel exposes theme.follow_os checkbox |
| `ff-desktop` | \U0001f534 | -- | Req 12.1: Macro Library panel via POM option 6, MACROS command, =6 fastpath |
| `ff-desktop` | \U0001f534 | -- | Req 12.2: panel lists all macros with name, source directory, full path |
| `ff-desktop` | \U0001f534 | -- | Req 12.3: Run action dispatches MACRO <name> and shows result in status bar |
| `ff-desktop` | \U0001f534 | -- | Req 12.4: Edit action opens .lua file in new editor tab |
| `ff-desktop` | \U0001f534 | -- | Req 12.5: Delete action prompts confirmation then removes file and inventory entry |
| `ff-desktop` | \U0001f534 | -- | Req 12.6: filter input narrows list by case-insensitive name substring |
| `ff-desktop` | \U0001f534 | -- | Req 12.7: panel state (selection, filter) not persisted across sessions |
| `ff-desktop` | \U0001f534 | -- | Req 12.8: panel list refreshes within one frame when macro inventory changes |
"""
tcr = tcr.rstrip() + "\n" + cr_rows
with open(tcr_path, "w", encoding="utf-8") as f:
    f.write(tcr)
log("TCR.md updated OK")

# -----------------------------------------------------------------------
# 2. change-log.md -- append CR-NR-043
# -----------------------------------------------------------------------
cl_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"
cl = open(cl_path, encoding="utf-8").read()
log(f"change-log size: {len(cl)}")

cr_entry = """
### CR-NR-043 -- Phase CR: OS Theme Follow + Macro Library Management
- **Date/Phase**: Phase CR
- **Prompt**: "Proceed with Phase CR"
- **Description**: Adds two medium-priority gap features: (1) OS dark/light mode follow -- the workbench detects the OS dark/light preference via egui and automatically switches the active Visual_Mode when theme.follow_os is enabled; (2) Macro Library Management panel -- POM option 6 opens a panel listing all discovered Lua scripts with Run, Edit, Delete, and filter capabilities.
- **Status**: IN PROGRESS
- **Linked spec**: `docs/specs/theme-and-appearance/requirements.md` (new Req 16), `docs/specs/lua-macro-engine/requirements.md` (new Req 12)
"""
cl = cl.rstrip() + "\n" + cr_entry
with open(cl_path, "w", encoding="utf-8") as f:
    f.write(cl)
log("change-log.md updated OK")

# -----------------------------------------------------------------------
# 3. current-work.md -- add Phase CR row and update active focus
# -----------------------------------------------------------------------
cw_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\current-work.md"
cw = open(cw_path, encoding="utf-8").read()
log(f"current-work size: {len(cw)}")

# Add Phase CR row to the work areas table after the CQ row
old_row = "| Phase CQ -- Enterprise Features | DONE | audit-logging, settings export/import, locked config keys | [project-master tasks](../specs/project-master/tasks.md) |"
new_row = old_row + "\n| Phase CR -- OS Theme Follow + Macro Library | ACTIVE | OS dark/light follow, Macro Library panel | [project-master tasks](../specs/project-master/tasks.md) |"
if old_row in cw:
    cw = cw.replace(old_row, new_row, 1)
    log("Added Phase CR row to work areas table")
else:
    log("WARNING: CQ row not found in current-work.md")

# Update active focus line
old_focus = "**Current focus:** Phase CQ -- Enterprise Features COMPLETE. All 5 deliverables done."
new_focus = "**Current focus:** Phase CR -- OS Theme Follow + Macro Library Management. Requirements gate complete."
if old_focus in cw:
    cw = cw.replace(old_focus, new_focus, 1)
    log("Updated active focus line")
else:
    log("WARNING: old focus line not found")

# Add Phase CR deliverable table after the CQ table
old_cq_table_end = "| Integration tests + TCR | All new criteria | [x] CQ.5 |"
new_section = old_cq_table_end + """

### Phase CR -- OS Theme Follow + Macro Library Management (next)

| Deliverable | Spec | Status |
|-------------|------|--------|
| Requirements gate | theme-and-appearance Req 16, lua-macro-engine Req 12 | [x] CR.1 |
| OS dark/light mode follow | theme.follow_os config key + frame detection | [ ] CR.2 |
| Macro Library panel | TabKind::MacroLibrary, MACROS/=6, list/run/edit/delete | [ ] CR.3 |
| Integration tests + TCR | All new criteria | [ ] CR.4 |"""
if old_cq_table_end in cw:
    cw = cw.replace(old_cq_table_end, new_section, 1)
    log("Added Phase CR deliverable table")
else:
    log("WARNING: CQ table end not found")

with open(cw_path, "w", encoding="utf-8") as f:
    f.write(cw)
log("current-work.md updated OK")

log("Done.")
