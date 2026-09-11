# Analysis Record: connector-mainframe (W5.18)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: NONE (no `ff-connector-mainframe` crate -- DEFERRED, not in initial
  release)
- **Spec files**: requirements.md (208 lines, 12 requirements, 8 EARS criteria),
  design.md -- NO tasks.md
- **Analysed**: Wave 5 pass (DEFERRED future spec)

---

## 1. Split candidacy

N/A -- no crate. Nothing to split.

---

## 2. Cross-unit consistency -- CONFIRMS deferred pattern + a CLEAN local-vs-remote boundary

Same deferred-spec pattern as network-fs (W5.15) / ftp-sftp (W5.16) / cloud (W5.17):
- NO crate, NO tasks.md; design.md marked "STATUS: DEFERRED -- Not in initial release."
- 12 reqs / 208 lines / 8 EARS; 0 crate dependents (expected).
- Targets the `connector-extensibility` trait (ConnectorPlugin) -- confirming the
  PA-CONFLICT-021 refinement (deferred remote connectors are the intended implementers).
- Transports: z/OS FTP (`zos-ftp://`) for MVS dataset transfer + JES spool, and z/OSMF
  REST (`zosmf://`) for datasets/jobs.

### CLEAN local-emulation vs remote-connectivity boundary (POSITIVE)

The intro draws an EXPLICIT, well-designed boundary: "the `dataset-catalog` sub-project
provides LOCAL mainframe filesystem EMULATION (MVS datasets, PDS/PDSE, GDG, sequential
files) in the INITIAL release; this [connector] is for REAL mainframe connectivity
(FUTURE)." So:
- dataset-catalog (built, W2) + JES (built, W5.1) + IDCAMS (W5.2) = LOCAL EMULATION of
  the mainframe environment.
- connector-mainframe (deferred) = REMOTE connection to a REAL z/OS system.

These are COMPLEMENTARY, not duplicative -- a clean separation of "emulate locally" vs
"connect to the real thing." Recorded as a POSITIVE consistency finding (no conflict):
the mainframe-emulation crates and the future mainframe CONNECTOR occupy distinct,
well-bounded roles. (Contrast the ff-viewers-vs-ff-asa duplication W5.10 -- here the
boundary is drawn correctly.)

### Consistent layering

Targets connector-extensibility (VfsProvider + lifecycle + auth + capability), consistent
with the deferred family. z/OS auth (RACF/passwords/certs) is another justification for
the extensibility auth layer (like cloud OAuth W5.17).

---

## 3. Completeness

N/A for a deferred spec (no crate, no tasks.md, nothing claimed done). HONEST deferral.
FOURTH deferred connector spec (w/ network-fs, ftp-sftp, cloud) -- rolls under
PA-TRACK-006.

---

## 4. Logging audit

N/A -- no crate. (When built, connection/transfer/auth logging will matter, avoiding
secret values -- same note as cloud.)

---

## 5. Task revision proposals

- **PA-TRACK-006 (extend)**: connector-mainframe is the FOURTH deferred spec-only
  connector -- include in the "mark deferred connector specs clearly" action. All four
  deferred connectors (network-fs, ftp-sftp, cloud, mainframe) are now catalogued.
- **PA-STD-071 (extend, docs ASCII)**: design.md has the same mojibake STATUS line
  (corrupted warning-emoji + em-dash). Same ASCII fix. Docs; no gate.
- **PA-CONFLICT-021 (confirming evidence)**: mainframe confirms connector-extensibility's
  deferred implementers; z/OS auth exercises the auth layer. No action now.
- POSITIVE (no action): clean local-emulation (dataset-catalog/JES/IDCAMS) vs
  remote-connectivity (this connector) boundary -- record as an exemplar of correct
  scope separation.

No code (no crate). No new PA-CONFLICT / PA-LOG / PA-TCR / PA-SPLIT.

---

## Summary

connector-mainframe is a DEFERRED, honestly-scoped FUTURE spec (no crate, no tasks.md,
design.md "DEFERRED -- not in initial release"; 12 reqs / 8 EARS) for REAL z/OS
connectivity via z/OS FTP (`zos-ftp://`) + z/OSMF REST (`zosmf://`). It repeats the
deferred-connector pattern and CONFIRMS the PA-CONFLICT-021 refinement (targets the
connector-extensibility trait). Its standout value is a POSITIVE consistency finding: the
spec draws an EXPLICIT, well-designed boundary between LOCAL mainframe EMULATION
(dataset-catalog + JES + IDCAMS, built in the initial release) and REMOTE real-mainframe
CONNECTIVITY (this connector, future) -- complementary, non-duplicative roles (a clean
scope separation, contrast the ff-viewers/ff-asa duplication W5.10). z/OS auth further
justifies the connector-extensibility auth layer (like cloud OAuth W5.17). Rolls under
PA-TRACK-006 (fourth + final deferred connector) + PA-STD-071 (design.md mojibake). No
code, no new findings. This completes the connector family analysis: 1 built (local-fs),
1 intentional base (connector-extensibility), 4 deferred remote (network-fs, ftp-sftp,
cloud, mainframe).
