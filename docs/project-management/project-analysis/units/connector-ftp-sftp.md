# Analysis Record: connector-ftp-sftp (W5.16)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: NONE (no `ff-connector-ftp-sftp` crate -- DEFERRED, not in initial
  release)
- **Spec files**: requirements.md (180 lines, 12 requirements, 9 EARS criteria),
  design.md -- NO tasks.md
- **Analysed**: Wave 5 pass (DEFERRED future spec)

---

## 1. Split candidacy

N/A -- no crate. Nothing to split.

---

## 2. Cross-unit consistency -- CONFIRMS the deferred-connector pattern + PA-CONFLICT-021 refinement

connector-ftp-sftp is the SAME deferred-spec pattern established for network-fs (W5.15):
- NO crate, NO tasks.md; design.md marked "STATUS: DEFERRED -- Not in initial release ...
  a placeholder design documenting future integration points only."
- 12 reqs / 180 lines / 9 EARS criteria; referenced/depended-on by 0 crates (expected).
- Explicitly targets the `connector-extensibility` trait (25 refs to
  connector-extensibility / VfsProvider) -- "the `connector-extensibility` trait (defined
  in ff-connector-extensibility) provides [the base]".

This is a CONFIRMING data point for the PA-CONFLICT-021 refinement (W5.15): ff-connector-
extensibility is the intentional base for the DEFERRED remote connectors, of which
FTP/FTPS/SFTP is one. No new conflict -- it strengthens the "framework built ahead of its
deferred implementers" reading. FTP/FTPS/SFTP is a natural connector-extensibility
consumer (remote auth, connection lifecycle, capability advertisement all apply).

### Consistent layering

Like network-fs, ftp-sftp targets connector-extensibility (VfsProvider + lifecycle + auth
+ capability). The connector architecture is coherent across the deferred family. When the
first remote connector is built, FTP/SFTP semantics (passive/active mode, key auth,
resume) will be the acid test of the connector-extensibility API (per the PA-CONFLICT-021
re-confirm-API action).

---

## 3. Completeness

N/A for a deferred spec (no crate, no tasks.md, nothing claimed done). HONEST deferral
(contrast database-tool W5.4). Rolls under PA-TRACK-006 (mark deferred connector specs
clearly in project-master) -- this is the SECOND such spec (w/ network-fs).

---

## 4. Logging audit

N/A -- no crate.

---

## 5. Task revision proposals

- **PA-TRACK-006 (extend)**: connector-ftp-sftp is a second DEFERRED spec-only connector
  (w/ network-fs) -- include it in the "mark deferred connector specs clearly" action.
- **PA-STD-071 (extend, docs ASCII)**: design.md has the same mojibake STATUS line
  (corrupted warning-emoji + em-dash) as network-fs. Same ASCII fix. Docs; no gate.
- **PA-CONFLICT-021 (confirming evidence)**: no new action -- ftp-sftp confirms
  connector-extensibility's intended (deferred) implementers.

No code (no crate). No new PA-CONFLICT / PA-LOG / PA-TCR / PA-SPLIT.

---

## Summary

connector-ftp-sftp is a DEFERRED, honestly-scoped FUTURE spec (no crate, no tasks.md,
design.md "DEFERRED -- not in initial release"; 12 reqs / 9 EARS). It repeats the
network-fs (W5.15) pattern and CONFIRMS the PA-CONFLICT-021 refinement: it explicitly
targets the `connector-extensibility` trait (25 refs), so it is one of the deferred remote
connectors the framework was intentionally built for -- no new conflict, just confirming
evidence that ff-connector-extensibility is a base awaiting its deferred implementers.
FTP/FTPS/SFTP (remote auth, connection lifecycle, resume) will be a good acid test of the
connector-extensibility API when built. Rolls under PA-TRACK-006 (mark deferred specs) +
PA-STD-071 (design.md mojibake STATUS line, same as network-fs). No code, no new findings.
