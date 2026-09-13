# Analysis Record: ears-integration (W6.3)

- **Wave**: 6 (master, meta, finalization)
- **Backing crate**: NONE -- EARS requirements-integration WORKFLOW + outputs (review-only
  meta sub-project; no crate, no tasks.md)
- **Files**: workflow.md (283), + outputs: coverage-classification.md (521),
  gap-analysis.md (523), incomplete-work-audit.md (245), integration-plan.md (531),
  minix-ftso-reconciliation.md (204), source-of-truth-map.md (323),
  deferred-requirements.md (78)
- **Analysed**: Wave 6 pass (meta / cross-cutting; review-only)

---

## 1. Split candidacy

N/A -- documentation, not a crate.

---

## 2. Cross-unit consistency -- CLEAN + PA-TRACK-009 (two-level audit disambiguation)

ears-integration is the EARS-format WORKFLOW for merging ISPF + TSO/SDSF source
requirements (from `docs/source-documents/ispf-ears/` + `tso-ears/`) into the workbench,
with reconciliation outputs (source-of-truth-map, coverage-classification, gap-analysis,
incomplete-work-audit, integration-plan, minix-ftso-reconciliation, deferred-requirements).

### Positive: current paths, sound process

Unlike project-master + workbench-requirements-merge (stale `.kiro/specs/` paths,
PA-DOC-011), ears-integration uses CURRENT `docs/` paths (0 `.kiro/specs` / FileForgeEditor
refs). The EARS workflow is sound + well-structured. deferred-requirements.md records P3
DEFERRED requirements with a promotion process -- consistent with the deferred-connector
handling (W5.15-18). No stale-path finding here.

### PA-TRACK-009 -- TWO "incomplete work" audits at DIFFERENT levels (disambiguate + cross-ref)

There are now TWO incomplete-work audits AND two gap-analyses in the repo, at DIFFERENT
levels:
- ears-integration/incomplete-work-audit.md ("EI-3 Output") + gap-analysis.md =
  REQUIREMENTS-LEVEL: pending phases, "orphaned REQUIREMENTS" (requirements with no
  task/coverage -- it reports "No orphaned task references found", "orphaned requirement
  refs = 0"), TCR gaps. Inputs: project-master/dataset-catalog/VCM tasks + TCR.
- project-analysis/incomplete-work-register.md + consistency-matrix.md =
  CODE/ARCHITECTURE-LEVEL: orphan CRATES, false-positive-complete implementations, raw-fs
  bypasses, capability duplication, cap violations.

These are COMPLEMENTARY, not contradictory (only 3 incidental term-overlaps; the two are
largely disjoint). BUT the word "ORPHAN" is OVERLOADED: ears = "orphaned REQUIREMENTS"
(req with no task); project-analysis = "orphan CRATES" (crate used by nobody). A reader
could conflate "No orphaned [requirements] found" (ears) with the SIX orphan CRATES the
code analysis found. Recorded PA-TRACK-009 (docs, LOW): add a cross-reference between the
two audit sets clarifying the REQUIREMENTS-level vs CODE-level split, and disambiguate the
"orphan" term (orphaned-requirement vs orphan-crate). Docs only; hygiene.

### Consistency of findings

Where the two audits touch the same subjects they do NOT contradict: ears finds 0 orphaned
*requirements* (every requirement maps to a task) -- which is consistent with the
code-level finding that the ISSUE is not missing requirements but unwired/duplicated/
skeleton CODE. The two levels together give the full picture: requirements are well-mapped
(ears), but implementation has orphan crates + a skeleton + duplication (project-analysis).
This is itself a useful meta-observation for Wave-6 finalization.

---

## 3. Completeness

Review-only meta sub-project (no crate, no tasks.md -- correct). The EARS outputs are
present + current. No PA-INCOMPLETE.

---

## 4. Logging audit

N/A -- documentation.

---

## 5. Task revision proposals

- **PA-TRACK-009 (docs, LOW)**: cross-reference the ears-integration
  (requirements-level) and project-analysis (code-level) incomplete-work/gap audits;
  disambiguate the overloaded "orphan" term (orphaned-requirement vs orphan-crate). Docs.
- POSITIVE (no action): ears-integration uses CURRENT `docs/` paths (contrast the stale
  `.kiro/specs/` in project-master + workbench-requirements-merge, PA-DOC-011). The EARS
  workflow + deferred-requirements process are sound.

No code (no crate). No PA-CONFLICT/PA-LOG/PA-STD/PA-SPLIT/PA-DOC (paths are current).

---

## Summary

ears-integration is a review-only META sub-project (no crate): the EARS WORKFLOW for
merging ISPF + TSO/SDSF source requirements into the workbench, with reconciliation outputs
(source-of-truth-map, coverage-classification, gap-analysis, incomplete-work-audit,
integration-plan, deferred-requirements). It is CLEAN + a positive: it uses CURRENT `docs/`
paths (unlike the stale `.kiro/specs/` in project-master + workbench-requirements-merge),
and the EARS workflow + P3 deferred-requirements promotion process are sound. The one
finding is PA-TRACK-009 (LOW): there are now TWO incomplete-work / gap audits at DIFFERENT
levels -- ears (REQUIREMENTS-level: orphaned requirements, pending phases, TCR gaps; reports
0 orphaned requirements) vs project-analysis (CODE-level: orphan crates, false-complete,
raw-fs, duplication). They are complementary (largely disjoint) but the "orphan" term is
OVERLOADED (orphaned-requirement vs orphan-crate); cross-reference the two + disambiguate.
Usefully, the two levels TOGETHER tell the full story: requirements are well-mapped (ears
finds 0 orphaned requirements), but the IMPLEMENTATION has 6 orphan crates + a skeleton +
duplication (project-analysis) -- the gap is code, not requirements. No code, no conflict.
This completes the Wave-6 unit analysis (all 81 sub-projects DONE).
