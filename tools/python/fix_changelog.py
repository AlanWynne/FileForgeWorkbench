"""Fix stale IN PROGRESS statuses in change-log.md."""
import sys

LOG = r"C:\workspace\VSC\FileForgeWorkbench\tools\logs\script-out.txt"
PATH = r"C:\workspace\VSC\FileForgeWorkbench\docs\status\change-log.md"

def log(msg):
    print(msg, flush=True)
    with open(LOG, "w", encoding="utf-8") as f:
        f.write(msg + "\n")

def logappend(msg):
    print(msg, flush=True)
    with open(LOG, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

with open(PATH, "rb") as f:
    data = f.read()

log(f"File size: {len(data)} bytes")

replacements = [
    # CR-NR-017 BV
    (
        b"- **Status**: IN PROGRESS\r\n- **Linked spec**: `docs/specs/dataset-catalog/requirements.md` (new Requirement 31)",
        b"- **Status**: DONE -- Phase BV complete, CatalogLocation enum added to ff-dscatalog\r\n- **Linked spec**: `docs/specs/dataset-catalog/requirements.md` (new Requirement 31)"
    ),
    # CR-NR-016 BS
    (
        b"- **Status**: IN PROGRESS\r\n- **Linked spec**: `docs/specs/dataset-catalog/requirements.md` (new Requirements 16-30), `docs/specs/virtual-file-system/requirements.md` (new Requirements 9-12)",
        b"- **Status**: DONE -- Phase BS complete (BS.1-BS.15), all 15 deliverables implemented and tested\r\n- **Linked spec**: `docs/specs/dataset-catalog/requirements.md` (new Requirements 16-30), `docs/specs/virtual-file-system/requirements.md` (new Requirements 9-12)"
    ),
    # CR-NR-015 BQ -- first IN PROGRESS entry
    (
        b"- **Status**: IN PROGRESS \xe2\x80\x94 Task 1 (Inventory) complete\r\n- **Linked spec**: `docs/reviews/requirements-review/inventory.md` (Task 1 complete)",
        b"- **Status**: DONE -- Phase BQ complete, all 10 tasks done, 8 artefacts delivered\r\n- **Linked spec**: `docs/reviews/requirements-review/` (all 10 output files complete)"
    ),
    # CR-NR-032 CJ bootstrap
    (
        b"- **Status**: IN PROGRESS\r\n- **Linked spec**: `docs/specs/bootstrap-scripts/requirements.md` (new sub-project)",
        b"- **Status**: DONE -- Phase CJ complete, bootstrap/ scripts for Windows/Linux/macOS\r\n- **Linked spec**: `docs/specs/bootstrap-scripts/requirements.md` (new sub-project)"
    ),
    # CR-NR-033 CK FFTest
    (
        b"- **Status**: IN PROGRESS -- gate running\r\n- **Linked spec**: `docs/specs/automated-dialog-testing/requirements.md` (new sub-project)",
        b"- **Status**: DONE -- Phase CK complete (CK.1-CK.4), ff-fftest crate wired, 429 tests passing\r\n- **Linked spec**: `docs/specs/automated-dialog-testing/requirements.md` (new sub-project)"
    ),
    # CR-CH-006 BU SQLite
    (
        b"- **Status**: IN PROGRESS\n",
        b"- **Status**: DONE -- Phase BU complete (BU.1-BU.9), SQLite integration live\n"
    ),
    # CR-NR-002 PENDING GATE (superseded by Phase AS)
    (
        b"- **Status**: PENDING GATE\r\n- **Linked spec**: `docs/specs/startup-and-session/requirements.md` Requirement 19",
        b"- **Status**: DONE -- superseded and implemented by Phase AS (File Explorer Panel)\r\n- **Linked spec**: `docs/specs/startup-and-session/requirements.md` Requirement 19"
    ),
    # CR-CH-001 PENDING GATE (superseded by Phase AS/AC)
    (
        b"- **Status**: PENDING GATE\r\n\r\n### CR-CH-003",
        b"- **Status**: DONE -- implemented by Phase AS/AC (POM option 2 relabelled)\r\n\r\n### CR-CH-003"
    ),
    # CR-NR-008 PENDING GATE (implemented by Phase BB)
    (
        b"- **Status**: PENDING GATE\r\n- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new Requirement 18)",
        b"- **Status**: DONE -- implemented by Phase BB (sorted listing with file attributes)\r\n- **Linked spec**: `docs/specs/file-tree-panel/requirements.md` (new Requirement 18)"
    ),
]

count = 0
for old, new in replacements:
    if old in data:
        data = data.replace(old, new, 1)
        logappend(f"Replaced: {old[:60]!r}...")
        count += 1
    else:
        logappend(f"NOT FOUND: {old[:60]!r}...")

with open(PATH, "wb") as f:
    f.write(data)

logappend(f"Done. {count}/{len(replacements)} replacements made.")
