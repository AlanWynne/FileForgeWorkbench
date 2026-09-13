# Analysis Record: connector-network-fs (W5.15)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: NONE (no `ff-connector-network-fs` crate -- DEFERRED, not in
  initial release)
- **Spec files**: requirements.md (174 lines, 12 requirements, 10 EARS criteria),
  design.md -- NO tasks.md
- **Analysed**: Wave 5 pass (DEFERRED future spec)

---

## 1. Split candidacy

N/A -- no crate. Nothing to split.

---

## 2. Cross-unit consistency -- REFINES PA-CONFLICT-021 (connector-extensibility is intentional, not abandoned)

connector-network-fs is a DEFERRED, well-formed FUTURE spec: design.md is marked
"STATUS: DEFERRED -- Not in initial release ... a placeholder design documenting future
integration points only." No crate, no tasks.md (nothing claimed done), 12 reqs /
10 EARS criteria. It is referenced/depended-on by 0 crates (expected -- unbuilt).

CRUCIAL cross-finding: the network-fs spec states the future connector "implements the
`connector-extensibility` trait (defined in ff-connector-extensibility), which combines
`VfsProvider` (from ff-vfs) with connector LIFECYCLE, AUTHENTICATION, and CAPABILITY."

This substantially REFINES W5.13's PA-CONFLICT-021 (connector-extensibility "orphan"):
- The connector-extensibility framework is the DESIGNED BASE for the DEFERRED REMOTE
  connectors (network-fs, ftp-sftp, cloud, mainframe) -- which combine VfsProvider WITH
  lifecycle/auth/capability. Local-fs (initial release) correctly implements ff-vfs
  DIRECTLY (it needs no remote lifecycle/auth). So the architecture is intentional:
  local = plain VfsProvider; remote = connector-extensibility (VfsProvider + lifecycle +
  auth + capability).
- Therefore ff-connector-extensibility is NOT an abandoned orphan -- it is a framework
  built AHEAD of its intended implementers, all of which are DEFERRED/unbuilt. Its
  "0 implementers" is because the remote connectors do not exist yet, by plan.

Recorded as a REFINEMENT to PA-CONFLICT-021 (DOWNGRADE to LOW-MEDIUM / WATCH): the
framework is speculative-but-intentional. Action: keep it, but (a) add a note in its own
spec that its implementers are the DEFERRED remote connectors (so future readers do not
mistake it for an orphan -- as W5.13 initially did), and (b) re-confirm the framework's
API when the first remote connector is actually built (the trait may need adjustment
against real FTP/SFTP semantics). NOT a delete candidate (contrast the harder orphan
reading in W5.13). Cross-referenced W5.13 + W5.14.

### Consistent VFS layering (positive)

network-fs targets `VfsProvider` (ff-vfs) + connector-extensibility -- consistent with the
local-fs layering (W5.14) and the extensibility framework (W5.13). The connector
architecture is COHERENT across the family once read together; the confusion was only
that the base shipped before its (deferred) consumers.

---

## 3. Completeness

N/A for a deferred spec (no crate, no tasks.md, nothing claimed done). This is HONEST
deferral (contrast database-tool W5.4, which claimed 157/157 complete on a skeleton).
Recorded PA-TRACK-006 (docs, LOW): the deferred connector specs (network-fs + the others
in W5.16-18) have requirements.md + design.md but no tasks.md -- fine for deferred work,
but the project-master should clearly list them as DEFERRED/future so they are not
counted as either complete or incomplete. Inventory hygiene only.

---

## 4. Logging audit

N/A -- no crate.

---

## 5. Task revision proposals

- **PA-CONFLICT-021 REFINEMENT (downgrade to LOW-MEDIUM / WATCH)**: ff-connector-
  extensibility is the intentional base for the DEFERRED remote connectors (network-fs
  states it implements the connector-extensibility trait) -- NOT an abandoned orphan.
  Keep it; add an implementers-are-deferred note to its spec; re-confirm its API when
  the first remote connector is built. (Updates the W5.13 record's severity.)
- **PA-TRACK-006 (docs, LOW)**: mark the deferred connector specs (network-fs + others)
  clearly as DEFERRED/future in project-master so they are not miscounted.
- **PA-STD-071 (docs ASCII, design.md)**: design.md has mojibake -- a corrupted
  warning-emoji sequence + em-dash in the STATUS line. Replace the emoji with an ASCII
  marker (e.g. "STATUS: DEFERRED") and the em-dash with `--`. Docs; no gate.

No code (no crate). No PA-LOG / PA-TCR / PA-SPLIT (nothing to scan).

---

## Summary

connector-network-fs is a DEFERRED, honestly-scoped FUTURE spec (no crate, no tasks.md,
design.md marked "DEFERRED -- Not in initial release"; 12 reqs / 10 EARS criteria). Its
key value to the analysis is a REFINEMENT of PA-CONFLICT-021: the spec states the future
connector implements the `connector-extensibility` trait (VfsProvider + lifecycle + auth +
capability), which reveals the connector architecture is INTENTIONAL -- local-fs (initial
release) implements ff-vfs directly; the DEFERRED remote connectors (network-fs, ftp-sftp,
cloud, mainframe) are meant to implement ff-connector-extensibility. So that framework is
NOT an abandoned orphan (as W5.13 first read it) but a base built AHEAD of its deferred
implementers -- DOWNGRADE PA-CONFLICT-021 to LOW-MEDIUM/WATCH (keep it; note that its
implementers are deferred; re-confirm the API when the first remote connector lands).
Minor: PA-TRACK-006 (mark deferred connector specs clearly so they are not miscounted --
honest deferral, contrast database-tool's false-complete) and PA-STD-071 (design.md
mojibake in the STATUS line). No code.
