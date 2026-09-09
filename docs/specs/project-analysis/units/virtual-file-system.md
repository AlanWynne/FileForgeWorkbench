# Analysis Record: virtual-file-system

- **Wave**: 0 (foundation -- FFW-ARCH-001, ALL file access flows through this)
- **Backing crate**: `ff-vfs`
- **Spec folder**: `docs/specs/virtual-file-system/`
- **Analysed**: Wave 0, task W0.11
- **Verdict**: COMPLETE for feature tracking (91/91 tasks `[x]`, Req 1-12
  TCR-PASS), BUT with a real LOGGING/COMPLIANCE DEFECT: the crate uses
  `eprintln!` for diagnostics (violates GUI-independence) and does NOT log the
  WARN mandated by Req 3.3 (duplicate scheme). NOT a split candidate.
- **IWR-005 RESOLVED** -- see section 4.

---

## 1. Scope summary

`ff-vfs` is the VFS abstraction (FFW-ARCH-001). 12 requirements:

- Req 1 abstraction layer + unified VfsError; Req 2 Resource_URI scheme
  (`vfs://provider/path`); Req 3 provider registry (routing, discovery,
  dedup); Req 4 VfsProvider trait (async, object-safe, capabilities); Req 5
  file operations; Req 6 directory/container ops; Req 7 file watching; Req 8
  search (native + fallback). Added by CR-NR-016: Req 9 StorageProvider trait;
  Req 10 POSIX native-file constraints; Req 11 cross-resource staged
  transactions; Req 12 workspace backup/restore/reconcile/diagnose.

286 req lines, 12 requirements, single backing crate. Cohesive foundation.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 286 lines; exactly 12 reqs (not >12) | No |
| 3+ distinct responsibilities | abstraction/URI/registry/provider-trait/ops/watch/search + StorageProvider/POSIX/txn/backup are layers of ONE VFS; the CR-NR-016 additions (Req 9-12) are the closest to separable | Borderline |
| 2+ crates | one crate `ff-vfs`; Req 9 text even allows "or a new ff-storage-provider crate" | Borderline |
| low-cohesion clusters | Req 1-8 (VFS core) vs Req 9-12 (physical storage / txn / workspace) is a mild seam | Weak |
| file-size pressure | `posix_provider.rs` 444, `workspace.rs` 410 over the 400 cap; `transaction.rs` 393 near | Yes |

~1-2 criteria. **Borderline, but NOT recommended to split now.** The Req 9 spec
itself offers an optional future `ff-storage-provider` crate; if the physical
storage layer grows, revisit. For now the seam is mild and the registry ties
VfsProvider + StorageProvider together. Record as a WEAK split proposal
(PA-SPLIT-002), not an action.

## 3. Consistency / conflict

- Public types owned here (Vfs, ResourceUri, VfsProvider, StorageProvider,
  ProviderRegistry, VfsError, VfsCapabilities, VfsEntry, VfsMetadata,
  WatchHandle, WatchEvent, VfsStream, VfsTransaction, workspace commands) --
  sole owner `ff-vfs`. Added to consistency-matrix.
- FFW-ARCH-001 is the reciprocal of document-model's Req 4.8: consumers must not
  call std::fs; ff-vfs IS the crate permitted to. document-model W0.9 verified
  it obeys this. Consistent.
- Req 12 workspace commands (`workspace.backup/restore/reconcile/diagnose`)
  overlap dataset-catalog backup/manifest (dataset-catalog Req 26.3, ff-dscatalog
  integrity) and virtual-catalog-manager. TCR shows ff-vfs and ff-dscatalog BOTH
  have "backup manifest contains schema version/provider/inventory" rows (ff-vfs
  Req 12.2 vs ff-dscatalog Req 26.3). WATCH: confirm the manifest formats are one
  shared type or intentionally distinct layers when analysing dataset-catalog
  (Wave 2). Recorded PA-WATCH-003.
- VfsError taxonomy (Req 1.4/9.4) is the common error surface; providers map into
  it. Consistent, no leakage of provider types (Req 1.6).

## 4. Completeness -- IWR-005 verification

- **IWR-005 (from EI-3): "ff-vfs Req 9-12 had no tasks -- gap; add tasks before
  BS.8." STATUS: RESOLVED.** The tasks now exist and are done:
  - Task 13 StorageProvider (Req 9) `[x]`; backing `storage_provider.rs`.
  - Task 14 POSIX native (Req 10) `[x]`; backing `posix_provider.rs`.
  - Task 15 staged transactions (Req 11) `[x]`; backing `transaction.rs`.
  - Task 16 workspace backup/restore/reconcile/diagnose (Req 12) `[x]`; backing
    `workspace.rs`.
  Added under "Tasks Added by CR-NR-016 -- StorageProvider and VFS Extensions".
  The EI-3 audit predated this work; the gap has been closed. IWR-005 can be
  marked SUPERSEDED/RESOLVED in the register.
- Tasks overall: 91 `[x]`, 0 `[ ]`.
- TCR: Req 9.1-9.5, 10.1-10.6, 11.1-11.5, 12.1-12.5 ALL PASS with named tests
  (storage_provider/posix_provider/transaction/workspace test modules). Req 1-8
  covered in earlier phases.
- Completeness verdict: **feature-complete and well-tested.**

## 5. Logging audit -- DEFECT FOUND

- Log-macro call sites: ZERO. `ff-logging` is NOT even a declared dependency of
  `ff-vfs` (unlike ff-document-model which at least declares it).
- **`eprintln!` used for diagnostics (3 real sites)** -- a GUI-independence
  violation (logging-subsystem Req 7.4/7.5: NO stdout/stderr output; all
  diagnostics via the log file):
  - `registry.rs:77` -- `eprintln!("[vfs] registry: registered provider ...")`
    on the register SUCCESS path.
  - `subsystem.rs:87` -- `eprintln!("[vfs] subsystem initialized")`.
  - `subsystem.rs:105` -- `eprintln!("[vfs] subsystem shut down ...")`.
  (The 2 `println!` in vfs.rs:1178,1211 are inside test byte-literal fixtures,
  not output -- not violations.)
- **Req 3.3 WARN is UNMET.** Req 3.3: on duplicate scheme registration THE
  registry SHALL "return an error AND log a WARN-level record". The code
  (registry.rs:71) correctly returns `VfsError::DuplicateScheme` but logs
  NOTHING on the duplicate path -- the only diagnostic output is the `eprintln!`
  on the SUCCESS path. So the WARN half of Req 3.3 is not satisfied.
- This is the most material logging finding in Wave 0 so far: a foundation crate
  that (a) has no logging wired, (b) writes to stderr against the platform's
  GUI-independence rule, and (c) fails a specific WARN acceptance criterion.
- Logging verdict: **INADEQUATE -- defect.** Logged PA-LOG-004 (code-mode fix).

## 6. Findings logged

- **PA-LOG-004** (LOGGING-GAP + Req 3.3 partial violation -- code fix, no gate):
  `ff-vfs` has no `ff-logging` dependency and uses `eprintln!` at registry.rs:77,
  subsystem.rs:87, subsystem.rs:105 (stderr -- violates logging-subsystem Req
  7.4/7.5 GUI-independence). Additionally Req 3.3's mandated WARN on duplicate
  scheme registration is not emitted (registry.rs:71 returns the error silently).
  Fix: add `ff-logging` dep; replace the 3 `eprintln!` with `log_info!`
  (register/init/shutdown are informational); add a `log_warn!` on the
  DuplicateScheme path to satisfy Req 3.3. Existing behaviour/tests unaffected
  (adds logging only). Code-mode change, no requirements gate (criterion exists).
- **IWR-005 RESOLVED**: Req 9-12 tasks (13-16) now exist and are `[x]`; backing
  code and TCR-PASS present. Mark IWR-005 SUPERSEDED/RESOLVED in the register.
- **PA-WATCH-003** (consistency watch): Req 12 workspace-backup manifest
  (ff-vfs Req 12.2) overlaps dataset-catalog backup manifest (ff-dscatalog Req
  26.3). Verify shared-vs-distinct manifest type when analysing dataset-catalog
  (Wave 2).
- **PA-SPLIT-002** (weak split proposal): `ff-vfs` is borderline (Req 9-12
  physical-storage seam + `posix_provider.rs` 444 / `workspace.rs` 410 over the
  400 cap). Req 9 spec text even permits a future `ff-storage-provider` crate.
  NOT recommended now; revisit if the storage layer grows. Also apply the 400
  cap refactor to posix_provider.rs / workspace.rs (rolled here, split by
  concern per rust-standards).
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-vfs contributes 24 matches
  (doc-comment prose, separators). Rolled into project-wide PA-LOG-001.
