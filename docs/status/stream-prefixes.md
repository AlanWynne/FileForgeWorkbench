# Work-Stream ID Prefix Registry

**Purpose:** When more than one work stream is active at the same time (for
example, mainline FFWB development plus a feature worktree), their bug and
change-request numbering would otherwise collide -- two different items could be
assigned the same `B###` / `CR-NR-###` / `CR-CH-###` id on different branches,
and every new id would also cause a merge conflict on `bugs.md` /
`change-log.md`.

To prevent this, each work stream owns a UNIQUE id PREFIX. New ids logged while
working in a stream use that stream's prefix. Prefixed ids never collide across
streams and merge cleanly, because two streams never touch the same id.

This file is the SINGLE SOURCE OF TRUTH for the prefix assignment. Before logging
a new bug or change request, a session MUST consult this registry to find the
prefix for the stream it is working in (determined by the current git branch /
worktree), then number within that prefix.

## How a prefix is applied

The prefix is inserted into the id after the category token:

| Category | Mainline (no prefix) | Prefixed stream (prefix `V`) |
|----------|----------------------|------------------------------|
| Bug | `B001`, `B002`, ... | `BV001`, `BV002`, ... |
| New Requirement | `CR-NR-001`, ... | `CR-NR-V001`, ... |
| Change Request | `CR-CH-001`, ... | `CR-CH-V001`, ... |

Within a prefix, numbering is sequential from `001` and independent of every
other stream. To find the next id: read the relevant log, filter to the current
stream's prefix, take the highest number, add 1.

The same prefixed id is used everywhere the item is referenced: the test plan
(`core-acceptance-test-plan.md`) Req / Backing column, the TCR backing column,
the Regression Traceability map, and spec cross-references. Traceability stays
clean because the prefixed id is globally unique.

## Mainline is the unprefixed default

The main FFWB development stream (branch `main`) uses NO prefix. All existing ids
(`B001...`, `CR-NR-001...`, `CR-CH-001...`) stay exactly as they are -- there is
no retroactive renumbering. Only parallel feature streams take a prefix.

## Registry

| Prefix | Stream | Branch / worktree | Status | Owner notes |
|--------|--------|-------------------|--------|-------------|
| (none) | Mainline FFWB | `main` (workspace root) | ACTIVE | Default stream; keeps all historical ids. |
| `V` | VSAM service wiring | `feature/vsam-service-wiring` (`.worktrees/vsam-wiring`) | ON HOLD -- REDIRECT | OWNER-CONFIRMED (CR-CH-059 gate approval): stays ON HOLD, worktree NOT deleted. Its original job (wire `ff-vsam-services::VsamService` to `ff-dscatalog` KSDS/ESDS/RRDS) is accepted as THROWAWAY: `ff-vsam-services` is slated to RETIRE and VSAM must instead be wired under the single `DatasetAccess` trait on the unified `ff-vfs::StorageProvider` seam (rationalisation report `.agents/tasks/dataset-vision-fit/report.md` section E, "must NOT build before consolidation"; task phase RC.B step 7). Do NOT resume this stream against the old trait; redirect it to the reconciled `ff-dscatalog` `VsamService` under `DatasetAccess` once the consolidation lands. |

## Adding a new stream

When a new parallel worktree/branch is created for a distinct body of work:

1. Choose a short, mnemonic, UNUSED prefix (one or two uppercase letters tied to
   the stream, e.g. `V` for VSAM, `M` for the ffmdx standalone app).
2. Add a row to the Registry table above (prefix, stream name, branch/worktree
   path, `ACTIVE`, a one-line note).
3. From then on, log all new bugs / CRs in that stream with the chosen prefix.

When a stream merges back to mainline, leave its prefixed ids as they are (they
remain globally unique) and set its Registry row `Status` to `MERGED`. Do not
reuse a retired prefix for a different stream.
