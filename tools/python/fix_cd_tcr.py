"""Update TCR rows for FFW-JES Requirement 17 (CD.impl) -- 17 rows from red to green."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\fix_cd_tcr.txt"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

with open(LOG, "w", encoding="utf-8") as f:
    f.write("")

log("fix_cd_tcr.py started")

tcr_path = r"C:\workspace\VSC\FileForgeWorkbench\docs\quality\TCR.md"
with open(tcr_path, "rb") as f:
    data = f.read()
log(f"TCR.md size: {len(data)} bytes")

# Each replacement: old red row -> new green row
# Using UTF-8 bytes for the emoji: red circle = \xf0\x9f\x94\xb4, green check = \xe2\x9c\x85
RED = b"\xf0\x9f\x94\xb4"
GREEN = b"\xe2\x9c\x85"

rows = [
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.1: ST panel shows all jobs with STATUS column |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.1: ST panel shows all jobs with STATUS column |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.2: FILTER command -- advanced filter expression; FILTER clears |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.2: FILTER command -- advanced filter expression; FILTER clears |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.3: FIND command -- search panel data; FIND NEXT/PREV |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.3: FIND command -- search panel data; FIND NEXT/PREV |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.4: LOCATE command -- scroll to first JOBNAME match, nearest alpha on no match |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.4: LOCATE command -- scroll to first JOBNAME match, nearest alpha on no match |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.5: UP/DOWN/LEFT/RIGHT scroll commands with n/HALF/PAGE/MAX amounts |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.5: UP/DOWN/LEFT/RIGHT scroll commands with n/HALF/PAGE/MAX amounts |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.6: SET ACTION displays valid action characters with descriptions |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.6: SET ACTION displays valid action characters with descriptions |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.7: SET MAIN [panel-name] sets default MENU panel |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.7: SET MAIN [panel-name] sets default MENU panel |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.8: SET ROWNUM ON/OFF toggles row numbers in NP area |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.8: SET ROWNUM ON/OFF toggles row numbers in NP area |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.9: WHO displays session info (user, start time, filters, SET settings, provider) |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.9: WHO displays session info (user, start time, filters, SET settings, provider) |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.10: QUERY AUTH displays authorised commands and action characters |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.10: QUERY AUTH displays authorised commands and action characters |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.11: SET settings (ACTION/MAIN/ROWNUM) persist across restarts |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.11: SET settings (ACTION/MAIN/ROWNUM) persist across restarts |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.12: FILTER supports =, !=, >, <, >=, <= operators and wildcard * |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.12: FILTER supports =, !=, >, <, >=, <= operators and wildcard * |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.13: FILTER supports AND and OR logical operators |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.13: FILTER supports AND and OR logical operators |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.14: ST panel accessible via ST command and S action on main panel |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.14: ST panel accessible via ST command and S action on main panel |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.15: FIND case-insensitive by default; FIND C for case-sensitive |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.15: FIND case-insensitive by default; FIND C for case-sensitive |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.16: LOCATE/FIND no-match shows \"string NOT FOUND\" in message area |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.16: LOCATE/FIND no-match shows \"string NOT FOUND\" in message area |"),
    (b"| `ff-jes` | " + RED + b" | -- | Req 17.17: scroll commands update SCROLL ===> field to last-used amount |",
     b"| `ff-jes` | " + GREEN + b" | `sdsf_filter.rs` unit tests | Req 17.17: scroll commands update SCROLL ===> field to last-used amount |"),
]

count = 0
for old, new in rows:
    if old in data:
        data = data.replace(old, new, 1)
        count += 1
    else:
        log(f"  WARNING: not found: {old[20:70]}")

with open(tcr_path, "wb") as f:
    f.write(data)
log(f"TCR.md: {count}/{len(rows)} rows updated")
log("Done.")
