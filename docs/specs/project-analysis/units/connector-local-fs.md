# Analysis Record: connector-local-fs (W5.14)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-connector-local-fs` (the PRIMARY VFS provider -- `LocalFsProvider`
  implementing the `VfsProvider` trait from `virtual-file-system` for native OS
  filesystem operations, async via Tokio)
- **Spec files**: requirements.md (179 lines, 7 requirements), tasks.md
  (89 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy + cap (PA-STD-069)

7 reqs / 179 lines -- not a split candidate. One cap file: `provider.rs` = 542 non-test
lines (well over the 400 cap -- the LocalFsProvider VfsProvider impl: read/write/list/
metadata/stream/etc. in one file). Recorded PA-STD-069 (MEDIUM -- cap): split provider.rs
by operation group (read/write / directory-list / metadata / streaming) into submodules,
leaving the LocalFsProvider impl as a thin coordinator. REFACTOR, no behaviour change.

---

## 2. Cross-unit consistency -- CLEAN + confirms PA-CONFLICT-021

### Correct layering: implements ff-vfs VfsProvider (WIRED, primary provider)

`LocalFsProvider` implements the `VfsProvider` trait from `virtual-file-system` (the
intro states it explicitly), async via Tokio. It is the PRIMARY VFS provider, WIRED
(7 external refs -- registered as the primary provider). Deps ff-vfs + ff-logging +
tokio -- exactly right for a VFS provider. Correct architecture.

### Confirms PA-CONFLICT-021 (connector-extensibility orphan) -- with a nuance

The local-fs spec NEVER mentions ff-connector-extensibility (0 refs in requirements/
design) and targets ff-vfs's VfsProvider directly. This CONFIRMS + refines W5.13's
PA-CONFLICT-021: the connector-EXTENSIBILITY framework was designed as a SEPARATE, higher
layer that even the PRIMARY connector was never meant to use -- so it is genuinely
speculative/orphaned, not merely "not wired yet." Strengthens PA-CONFLICT-021 option (b)
(the "future connectors MUST implement the framework" premise was never adopted by the
one real connector). Cross-referenced W5.13.

### 51 fs usages -- LEGITIMATE (this IS the filesystem layer)

The crate has 51 std/tokio fs usages -- but that is CORRECT: `ff-connector-local-fs` IS
the native-OS filesystem provider; performing real fs operations is its entire purpose.
This is the LEGITIMATE home for raw fs (via tokio::fs async), the layer that VFS-mediates
for everyone else. NOT a violation (contrast JES PA-CONFLICT-015, where a job engine
stored WORKBENCH data raw instead of going through the VFS). Positive: it centralises the
OS-fs interaction behind VfsProvider.

### Positive: ff-logging USED

9 ff-logging calls -- follows the logging standard (like connector-extensibility W5.13).
No dead-dep, no PA-LOG.

### Public types and ownership

- LocalFsProvider (VfsProvider impl), local-fs operation set -- sole-owned. No duplication
  (it is the primary provider; other connectors would be peers under ff-vfs).

### Cross-reference integrity

Cross-refs (ff-vfs VfsProvider, ff-logging, tokio) resolve + are used. Does NOT reference
connector-extensibility (by design -- PA-CONFLICT-021).

---

## 3. Completeness

Tracking: all 89 sub-tasks `[x]`. Implementation present across all 7 reqs (the
VfsProvider operation set for native OS fs). WIRED as the primary provider. Genuinely
complete. No PA-INCOMPLETE.

### TCR gap (PA-TCR-035)

TCR.md has 1 row for ff-connector-local-fs across 7 reqs -- thin, despite 89 tasks +
tests. Recorded PA-TCR-035.

---

## 4. Logging audit

- `ff_logging` / `log_*!`: 9 -- USED. ff-logging dep NOT dead. POSITIVE.
- `std::fs` / `tokio::fs`: 51 -- LEGITIMATE (this is the fs provider layer).

The primary fs provider logging its operations (or at least errors/retries) is
appropriate + present. No PA-LOG item. (Once more connectors exist, consistent
operation-level logging across providers would help -- but the foundation logs.)

---

## 5. Task revision proposals

- **PA-STD-069 (MEDIUM -- cap)**: split provider.rs (542) by operation group
  (read/write / list / metadata / streaming). REFACTOR.
- **PA-STD-070 (ASCII, comment mojibake)**: provider.rs doc-comment header has a mojibake
  em-dash (corrupted encoding, "LocalFsProvider <bad> implements VfsProvider..."). Same
  recurring Wave-5 comment-mojibake pattern (ff-jes/ffjcl, database-tool, toolchain).
  Replace with `--`. REFACTOR, no gate.
- **PA-TCR-035**: enumerate per-requirement TCR rows (1 for 7 reqs). No code.

No PA-LOG (logging present -- positive). No orphan (WIRED, primary provider). No raw-fs
VIOLATION (the 51 fs usages are the legitimate fs layer). No incompleteness.

---

## Summary

connector-local-fs (`ff-connector-local-fs`) is the PRIMARY VFS provider -- `LocalFsProvider`
implementing ff-vfs's `VfsProvider` trait for native OS filesystem operations (async via
Tokio; 7 reqs, 89/89, WIRED with 7 refs). A CLEAN, correct unit: right layering (implements
ff-vfs, centralises OS-fs behind VfsProvider), ff-logging USED (9 calls -- follows the
standard), and its 51 fs usages are LEGITIMATE (this IS the filesystem layer -- the correct
home for raw fs, contrast the JES data-store bypass). It also CONFIRMS + refines
PA-CONFLICT-021: the spec never mentions ff-connector-extensibility (0 refs) and targets
ff-vfs directly, so the connector-extensibility framework is a separate, speculative layer
even the primary connector was never designed to use (strengthens the delete-or-reconcile
option there). Findings are minor: provider.rs 542 over cap (PA-STD-069), a doc-comment
mojibake em-dash (PA-STD-070), and thin TCR (PA-TCR-035, 1/7). No orphan, no incompleteness,
no raw-fs violation, no dead logging.
