# Analysis Record: workbench-requirements-merge (W6.2)

- **Wave**: 6 (master, meta, finalization)
- **Backing crate**: NONE -- architecture + validation META sub-project (no deliverable
  crate; no requirements.md)
- **Files**: architecture-brief.md (394), dataset-catalog-brief.md (309),
  final-inventory.md (419), tasks.md (152, 95/95 done), + verification reports:
  consistency (162), dbeaver (618), ffe-coverage (198), gap-analysis (143),
  vfs-principle (143)
- **Analysed**: Wave 6 pass (meta / cross-cutting)

---

## 1. Split candidacy

N/A -- documentation, not a crate.

---

## 2. Cross-unit consistency -- PA-TRACK-008 (verification reports are SPEC-level; over-claim vs CODE reality)

This sub-project holds the project's foundational architecture briefs + the "Task 18.x"
FINAL-VALIDATION verification reports. The reports are well-structured, BUT they verify
SPEC / REQUIREMENTS coverage, not IMPLEMENTATION -- and read as completeness claims they
OVER-CLAIM relative to the code-level analysis findings. Two concrete contradictions:

1. **verification-vfs-principle.md** asserts (universal): "All file-related specs honour
   FFW-ARCH-001. Every spec that performs [file ops routes through the VFS]" (FFW-ARCH-001
   = "no direct std::fs / tokio::fs"). This verifies the SPECS say VFS -- but the CODE has
   TWO raw-fs WRITE violations the analysis found: JES job-queue persistence
   (PA-CONFLICT-015) and global-search cross-file REPLACE (PA-CONFLICT-020), both raw
   std::fs bypassing the VFS. So the "all honour FFW-ARCH-001" claim is true at the spec
   level but FALSE at the code level.

2. **verification-dbeaver.md** presents a Capability Coverage Matrix with 268 "COVERED"
   entries; its scope is "Verify DBeaver-derived database tool REQUIREMENTS ... compare
   RESEARCH files" -- i.e. COVERED = a requirement is SPECIFIED, not implemented. But W5.4
   found the database-tool IMPLEMENTATION is a thiserror+serde SKELETON (~14/17 reqs
   unbuilt, PA-INCOMPLETE-014). So 268 "COVERED" reads as "built" but means "specified".

3. **verification-gap-analysis.md** similarly verifies gap-analysis items are "Addressed"
   at the requirements level (a spec exists), not implemented.

Root cause (single meta-finding): these are SPEC-COVERAGE verifications from the "Task
18.x" FINAL-VALIDATION vintage (same era as project-master's readiness-summary, W6.1
PA-TRACK-007). They validate that specs exist + cover the research/gaps -- a legitimate +
useful check -- but they are NOT implementation verification, and their universal/"COVERED"
phrasing invites misreading as completeness. Recorded PA-TRACK-008 (docs, MEDIUM): add an
explicit SCOPE banner to the verification reports ("this verifies SPEC coverage, not
implementation") and cross-reference the code-level exceptions the analysis found
(vfs-principle -> PA-CONFLICT-015/020; dbeaver -> PA-INCOMPLETE-014). Pairs with
PA-TRACK-007 (both reconcile the pre-analysis validation narrative with the code reality).

### Path drift (rolls into PA-DOC-011)

The reports reference `.kiro/specs/` / `FileForgeEditor/.kiro/specs/` paths; specs now live
under `docs/specs/`. Same stale-path issue as readiness-summary (PA-DOC-011) -- fold in.

### The architecture briefs are VALUABLE + consistent (positive)

architecture-brief.md + dataset-catalog-brief.md + final-inventory.md are coherent
foundational documents; final-inventory (419) is a useful crate/sub-project inventory. The
VFS PRINCIPLE itself (FFW-ARCH-001) is sound + is the correct standard the analysis
measures against (external-mod/file-ops/local-fs honour it; JES/global-search violate it).
So the ARCHITECTURE is well-documented; only the VERIFICATION reports' completeness framing
is stale (PA-TRACK-008).

---

## 3. Completeness

Tracking: 95/95 tasks `[x]`. The meta docs + verification reports exist + are structurally
complete. The gap is that the verification reports are spec-level (PA-TRACK-008), not a
missing document. No PA-INCOMPLETE.

---

## 4. Logging audit

N/A -- documentation.

---

## 5. Task revision proposals

- **PA-TRACK-008 (docs, MEDIUM)**: add a SCOPE banner to the verification reports (verifies
  SPEC coverage, not implementation) + cross-reference the code-level exceptions the
  analysis found -- vfs-principle -> PA-CONFLICT-015/020 (JES + global-search raw-fs
  writes); dbeaver -> PA-INCOMPLETE-014 (database-tool skeleton); gap-analysis -> the
  relevant PA-INCOMPLETE items. Pairs with PA-TRACK-007. Docs.
- **PA-DOC-011 (extend)**: fold the `.kiro/specs/` -> `docs/specs/` path fix here too
  (the verification reports share the stale path).

No code (no crate). No PA-CONFLICT/PA-LOG/PA-STD/PA-SPLIT (documentation).

---

## Summary

workbench-requirements-merge is an architecture + validation META sub-project (no crate):
foundational briefs (architecture, dataset-catalog, final-inventory) + the "Task 18.x"
verification reports (consistency, dbeaver, ffe-coverage, gap-analysis, vfs-principle;
95/95). The architecture briefs are valuable + consistent, and FFW-ARCH-001 (the VFS
principle) is the correct standard the analysis measures against. The finding is
PA-TRACK-008 (MEDIUM): the verification reports verify SPEC / requirements coverage, not
IMPLEMENTATION, so their universal/"COVERED" phrasing OVER-CLAIMS vs the code-level analysis
-- concretely, vfs-principle asserts "all honour FFW-ARCH-001" but the code has two raw-fs
WRITE bypasses (JES PA-CONFLICT-015, global-search PA-CONFLICT-020), and dbeaver's 268
"COVERED" entries mean "specified" while database-tool is an unbuilt skeleton
(PA-INCOMPLETE-014). Add a scope banner + cross-reference the code-level exceptions; this
pairs with PA-TRACK-007 (project-master) as the Wave-6 reconciliation of the pre-analysis
validation narrative with code reality. Stale `.kiro/specs/` paths fold into PA-DOC-011.
No code; the docs are structurally complete, just spec-scoped.
