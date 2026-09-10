# Analysis Record: virtual-file-system

- **Wave**: 0 (foundation -- FFW-ARCH-001, ALL file access flows through this)
- **Backing crate**: `ff-vfs`
- **Spec folder**: `docs/specs/virtual-file-system/`
- **Analysed**: Wave 0, task W0.11 (CR-NR-057 re-baseline)
- **Verdict**: Feature-COMPLETE (91/91 tasks, Req 1-12 TCR-PASS), BUT the
  LOGGING/COMPLIANCE DEFECT from the prior pass PERSISTS (unfixed): eprintln for
  diagnostics + unmet Req 3.3 WARN (PA-LOG-004). NOT a split candidate (weak).
  IWR-005 RESOLVED.
- **CR-NR-057 impact**: NONE (vfs specs untouched). Re-verified from code.

---

## 1. Scope summary

`ff-vfs` is the VFS abstraction (FFW-ARCH-001). 12 requirements: Req 1 abstraction
+ VfsError; Req 2 Resource_URI (`vfs://provider/path`); Req 3 provider registry;
Req 4 VfsProvider trait; Req 5 file ops; Req 6 dir/container ops; Req 7 file
watching; Req 8 search. CR-NR-016: Req 9 StorageProvider; Req 10 POSIX native;
Req 11 cross-resource staged transactions; Req 12 workspace backup/restore/
reconcile/diagnose.

286 req lines, 12 requirements, single backing crate.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 286 lines; exactly 12 reqs (not >12) | No |
| 3+ distinct responsibilities | VFS core (1-8) vs physical storage/txn/workspace (9-12) is a mild seam; Req 9 text even allows a future `ff-storage-provider` crate | Borderline |
| 2+ crates | one crate | No |
| low-cohesion clusters | mild seam Req 1-8 vs 9-12 | Weak |
| file-size pressure | posix_provider.rs 444, workspace.rs 410 over cap | Yes |

~1-2 criteria. **NOT recommended to split now** (weak PA-SPLIT-002). Revisit if
the storage layer grows.

## 3. Consistency / conflict

- Public types owned here (Vfs, ResourceUri, VfsProvider, StorageProvider,
  ProviderRegistry, VfsError, VfsCapabilities, VfsEntry, VfsMetadata, WatchHandle,
  WatchEvent, VfsStream, VfsTransaction, workspace commands) -- sole owner. Added
  to consistency-matrix.
- FFW-ARCH-001 reciprocal of document-model Req 4.8 (W0.9 verified consumers obey
  it; ff-vfs IS the crate permitted std::fs). Consistent.
- **PA-WATCH-003 (carried)**: Req 12 workspace-backup manifest (ff-vfs Req 12.2)
  overlaps dataset-catalog backup manifest (ff-dscatalog Req 26.3). Verify
  shared-vs-distinct manifest type at Wave 2.
- VfsError taxonomy (Req 1.4/9.4) is the common error surface; no provider types
  leak (Req 1.6). Consistent.

## 4. Completeness -- IWR-005 RESOLVED

- **IWR-005 (EI-3: "ff-vfs Req 9-12 had no tasks"): RESOLVED.** Tasks 13
  (StorageProvider Req 9), 14 (POSIX Req 10), 15 (staged txns Req 11), 16
  (workspace backup Req 12) all exist and are `[x]`; backing code
  (storage_provider/posix_provider/transaction/workspace.rs) present; 21 TCR rows
  for Req 9.1-12.5 all PASS. EI-3 predated this work.
- Tasks overall: 91 `[x]`, 0 `[ ]`. Req 1-8 covered in earlier phases.
- Completeness verdict: **feature-complete and well-tested.**

## 5. Logging audit -- DEFECT PERSISTS (PA-LOG-004)

- Log-macro call sites: ZERO. `ff-logging` is STILL NOT a declared dependency of
  ff-vfs (unfixed since prior pass).
- **`eprintln!` for diagnostics (3 real sites) -- GUI-independence violation**
  (logging-subsystem Req 7.4/7.5: NO stdout/stderr; all diagnostics via log file):
  - registry.rs:77 -- register SUCCESS path.
  - subsystem.rs:87 -- subsystem initialised.
  - subsystem.rs:105 -- subsystem shut down.
  (vfs.rs:1178,1211 `println!` are inside test byte-literal fixtures -- not
  violations.)
- **Req 3.3 WARN still UNMET.** registry.rs:71 returns `VfsError::DuplicateScheme`
  on duplicate registration but logs NOTHING; Req 3.3 mandates "return an error
  AND log a WARN-level record". The WARN half is missing.
- Logging verdict: **INADEQUATE -- defect persists.** PA-LOG-004 remains open.

## 6. Findings logged

- **PA-LOG-004** (LOGGING-GAP + Req 3.3 partial violation -- code fix, no gate;
  PERSISTS from prior pass): ff-vfs has no `ff-logging` dependency and uses
  `eprintln!` at registry.rs:77, subsystem.rs:87/105 (stderr -- violates Req
  7.4/7.5). Req 3.3's mandated WARN on duplicate scheme registration is not
  emitted (registry.rs:71). Fix: add `ff-logging` dep; replace the 3 `eprintln!`
  with `log_info!`; add `log_warn!` on the DuplicateScheme path. Follow the
  ff-plugin / ff-core exemplar logging pattern. Code-mode, no gate (criterion
  exists). HIGH value -- a foundation crate consumed everywhere.
- **IWR-005 RESOLVED**: Req 9-12 tasks 13-16 `[x]`; code + 21 TCR-PASS rows
  present. Register row updated.
- **PA-WATCH-003** (CONSISTENCY WATCH): Req 12 workspace-backup manifest overlaps
  dataset-catalog manifest (ff-dscatalog Req 26.3). Verify at Wave 2.
- **PA-SPLIT-002** (weak SPLIT PROPOSAL): borderline (Req 9-12 seam; posix_provider
  444 / workspace.rs 410 over cap; Req 9 permits a future `ff-storage-provider`).
  NOT recommended now. Apply 400-cap refactor to posix_provider.rs / workspace.rs.
  Owner-gated for a crate split.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-vfs contributes matches (doc
  prose, separators). Rolled into project-wide PA-LOG-001.
