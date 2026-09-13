# Analysis Record: project-master (W6.1)

- **Wave**: 6 (master, meta, finalization)
- **Backing crate**: NONE -- master ORCHESTRATION documents (no deliverable crate)
- **Files**: requirements.md (444 lines, master req + 73 sub-project inventory rows),
  tasks.md (1453 lines: impl Phases A-Q + spec phases + the analysis Phases PA-W0..PA-W5),
  readiness-summary.md (114), sub-project-audit.md (177), api-consistency-report.md (274),
  phase-s-tasks.md (32), validation-19.2/19.3/19.4 (86/132/122)
- **Analysed**: Wave 6 pass (meta / cross-cutting)

---

## 1. Split candidacy

N/A -- documentation, not a crate. (tasks.md is 1453 lines but it is a master task ledger,
not source; the 400-line cap does not apply.)

---

## 2. Cross-unit consistency -- PA-TRACK-007 (master narrative STALE vs the analysis)

The master's ANALYSIS integration is confined to tasks.md: all 160 analysis references
(PA-W0..PA-W5 phases, PA-CONFLICT/orphan mentions) live in tasks.md -- which is correct +
current (all six PA-W phases present, coherent, dependency-ordered). GOOD.

BUT the master's TOP-LEVEL NARRATIVE documents do NOT reflect the analysis at all:
- readiness-summary.md: 0 analysis refs.
- sub-project-audit.md, api-consistency-report.md, validation-19.2/19.3/19.4.md: 0 analysis
  refs.

So the master's readiness/audit/validation story is STALE -- it presents the PRE-analysis
picture and never mentions the analysis's material findings: the SIX orphan crates, the
database-tool FALSE-POSITIVE-COMPLETE (PA-INCOMPLETE-014), the DUPLICATION cluster (ASA x3,
EBCDIC x2, viewers/PREVIEW, language-service), the DATA-SAFETY raw-fs write bypasses
(PA-CONFLICT-015/020), or the CR-NR-057 command-chaining bundle (Phase DH). A reader of
readiness-summary.md would conclude the project is essentially complete/ready, which the
analysis contradicts in specific, actionable ways.

Recorded PA-TRACK-007 (docs, MEDIUM): reconcile the master narrative with the analysis --
update readiness-summary.md (and note in sub-project-audit.md) to REFERENCE
`docs/specs/project-analysis/` (checklist + incomplete-work-register + consistency-matrix)
and summarise the headline findings (orphan cluster, database-tool re-open, duplication
consolidation, data-safety writes, Phase DH, PA-W0..W5 remediation phases). The master
should point to the analysis as the current source of truth for "what still needs doing,"
not present a stale "all specified / ready" story.

### PA-DOC-011 (master doc accuracy) -- stale path + incomplete deferred list

readiness-summary.md has two concrete inaccuracies:
- It scopes to `.kiro/specs/` -- but the specs now live under `docs/specs/`. Stale path.
- It states "Deferred (4 sub-projects)" but NAMES only 3 (connector-network-fs,
  connector-ftp-sftp, connector-mainframe) -- OMITS connector-cloud (W5.17 confirmed it is
  the 4th deferred connector). The count (4) is right; the enumeration is missing cloud.

Recorded PA-DOC-011 (docs, LOW): fix the `.kiro/specs/` -> `docs/specs/` path and add
connector-cloud to the named Deferred list. Also fold in the PA-TRACK-005 correction
(jcl-resolver scope NOT in FFW-JES Req 11) since that lives in a master doc. Docs only.

### The master IS an accurate INVENTORY (positive)

requirements.md enumerates 73 sub-project rows -- a proper master inventory of the family.
The tasks.md phase structure (A-Q impl + spec phases + PA-W0..W5) is coherent + current.
So the master's STRUCTURE is sound; only its READINESS/VALIDATION NARRATIVE is stale
(PA-TRACK-007). The fix is reconciliation, not restructuring.

---

## 3. Completeness (of the master as a document)

The master orchestration docs are present + structurally complete (inventory, phases,
summary, validation reports). The gap is CURRENCY, not existence (PA-TRACK-007) -- the
narrative predates the analysis. No PA-INCOMPLETE (nothing is missing; the readiness story
is out of date).

---

## 4. Logging audit

N/A -- documentation.

---

## 5. Task revision proposals

- **PA-TRACK-007 (docs, MEDIUM)**: reconcile the master narrative (readiness-summary +
  sub-project-audit) with the analysis -- reference project-analysis as the current source
  of truth for outstanding work; summarise the headline findings + the PA-W0..W5 phases.
  This is a Wave-6 finalization deliverable.
- **PA-DOC-011 (docs, LOW)**: fix readiness-summary `.kiro/specs/` -> `docs/specs/`; add
  connector-cloud to the named Deferred list (4 stated, only 3 named); fold in PA-TRACK-005
  (jcl-resolver scope correction).

No code (no crate). No PA-CONFLICT/PA-LOG/PA-STD/PA-SPLIT (documentation).

---

## Summary

project-master is the master ORCHESTRATION layer (no crate): a requirements.md inventory
(73 sub-project rows), a tasks.md ledger (Phases A-Q + spec phases + the analysis
Phases PA-W0..PA-W5 -- all present, coherent, current), and readiness/audit/validation
reports. Its STRUCTURE is sound and the analysis Phases are correctly integrated INTO
tasks.md (160 refs). The finding is PA-TRACK-007 (MEDIUM): the master's TOP-LEVEL NARRATIVE
(readiness-summary + audit + validation docs -- 0 analysis refs) is STALE relative to the
analysis, presenting a pre-analysis "all specified / ready" picture that never mentions the
six orphan crates, the database-tool false-complete, the duplication cluster, the
data-safety raw-fs bypasses, or Phase DH. Reconcile it to reference project-analysis as the
current source of truth for outstanding work. Plus PA-DOC-011 (LOW): readiness-summary uses
the stale `.kiro/specs/` path and names only 3 of its stated 4 deferred connectors (omits
connector-cloud). The fix throughout is reconciliation/currency, not restructuring -- the
master is an accurate inventory; its readiness story just predates the 6-wave analysis.
