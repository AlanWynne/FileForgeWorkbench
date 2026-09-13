# Requirements Document -- Project Analysis and Re-Structuring

## Introduction

FileForgeWorkbench has grown to 79 sub-project specs and ~69 crates. Some
sub-projects have themselves grown large (for example `file-tree-panel` with 674
requirement lines, `dataset-catalog` 516, `virtual-catalog-manager` 443,
`idcams-emulator` 410, `jes-emulator` 398) and may be candidates for splitting
into smaller logical units.

This sub-project defines a systematic, repeatable analysis pass over the WHOLE
project. It is a meta / analysis sub-project (like `ears-integration` and
`workbench-requirements-merge`): it produces analysis artifacts and revised task
lists; it ships no crate of its own. Its purpose is to leave the project in a
state where every sub-project is either an appropriately-sized logical unit or
has an approved split plan, every logical unit is internally consistent and free
of cross-unit conflicts, every logical unit's development state (complete /
incomplete) is known and logged, proper logging is enabled throughout the code
so bugs can be diagnosed from logs, and the incomplete-task backlog is revised
and re-ordered to reflect the findings.

The analysis MUST build on the existing review artifacts under
`docs/specs/ears-integration/` (`incomplete-work-audit.md`, `gap-analysis.md`,
`source-of-truth-map.md`, `coverage-classification.md`) and the
`docs/reviews/requirements-review/` outputs, rather than duplicating them.

### Source References

- **[CR-NR-056]** = Change-log entry for this analysis pass.
- **[EI-3]** = `docs/specs/ears-integration/incomplete-work-audit.md` (prior audit).
- **[RR]** = `docs/reviews/requirements-review/` (Phase BQ requirements review).
- **[WF]** = `.kiro/steering/workflow.md` (gate + operating modes).
- **[LOG]** = `docs/specs/logging-subsystem/requirements.md`.

### Glossary

| Term | Definition |
|------|-----------|
| **Sub_Project** | A folder under `docs/specs/<name>/` with a `requirements.md` (and usually `design.md`, `tasks.md`). |
| **Logical_Unit** | A cohesive, single-responsibility slice of functionality. A Sub_Project is either already one Logical_Unit or decomposes into several. |
| **Split_Candidate** | A Sub_Project whose size, responsibility count, or crate count indicates it SHOULD be evaluated for decomposition. |
| **Split_Plan** | An approved proposal to divide a Sub_Project into named Logical_Units (new or existing sub-project folders), with requirement/criterion reassignment. |
| **Analysis_Record** | The per-Sub_Project written output of one analysis pass, stored under `docs/specs/project-analysis/units/<name>.md`. |
| **Consistency_Matrix** | The cross-unit record of shared types, commands, config keys, and terminology, flagging conflicts and duplicate ownership. |
| **Incomplete_Work_Register** | The consolidated, living list of every pending/incomplete task across the project, with status and re-ordered sequence. |
| **Logging_Audit** | The per-crate assessment that appropriate log records exist at the right levels for bug diagnosis (per logging-subsystem). |
| **Analysis_Wave** | A group of Sub_Projects analysed together, ordered by architectural dependency (foundations before consumers). |

---

## Requirements

### Requirement 1: Analysis Scope and Sequencing

**User Story:** As the project owner, I want every sub-project analysed in a
logical, dependency-aware order, so that foundational units are settled before
the units that depend on them and no sub-project is missed.

**Source:** [CR-NR-056], [WF]

#### Acceptance Criteria

1. THE analysis SHALL cover every Sub_Project folder under `docs/specs/` that
   contains a `requirements.md`, plus the `project-master` aggregate; a checklist
   SHALL record each Sub_Project as PENDING, IN PROGRESS, or DONE.
2. THE analysis SHALL proceed in Analysis_Waves ordered by architectural
   dependency: foundation/platform units first (platform-core, configuration-
   system, logging-subsystem, command-framework, document-model, virtual-file-
   system), then middle layers, then application/tool and UI units last.
3. WHEN a Sub_Project is analysed, THE analysis SHALL NOT be considered started
   for that unit until its upstream dependencies (per `project-master`
   dependency data and `source-of-truth-map.md`) have been analysed or explicitly
   deferred with a recorded reason.
4. THE analysis SHALL produce one Analysis_Record per Sub_Project under
   `docs/specs/project-analysis/units/<name>.md`, following the template defined
   in the design document.
5. THE analysis SHALL reuse and cross-reference the existing artifacts
   (`ears-integration/incomplete-work-audit.md`, `gap-analysis.md`,
   `source-of-truth-map.md`, `coverage-classification.md`,
   `reviews/requirements-review/*`) and SHALL NOT re-derive findings those
   artifacts already establish; it SHALL instead note where they are now stale.

### Requirement 2: Split Analysis (Decomposition into Logical Units)

**User Story:** As a maintainer, I want each sub-project evaluated for whether it
should be split into smaller logical units, so that no single sub-project carries
too many responsibilities to reason about or maintain.

**Source:** [CR-NR-056]

#### Acceptance Criteria

1. WHEN a Sub_Project is analysed, THE analysis SHALL classify it as a
   Split_Candidate or not, using explicit criteria (defined in the design
   document): requirement-line size, number of distinct responsibilities, number
   of backing crates, and the presence of unrelated concern clusters.
2. WHEN a Sub_Project is a Split_Candidate, THE analysis SHALL identify the
   candidate Logical_Units within it, name each, and state the single
   responsibility of each.
3. WHEN a split is proposed, THE analysis SHALL produce a Split_Plan that maps
   every existing requirement/criterion of the Sub_Project to exactly one target
   Logical_Unit (no criterion dropped, none duplicated), and SHALL identify which
   Logical_Units become new sub-project folders and which remain in place.
4. WHEN a split is proposed, THE Split_Plan SHALL identify the impact on backing
   crates (rename/split/no-change) and on downstream references, and SHALL flag
   any change that requires a separate requirements gate.
5. WHEN a Sub_Project is NOT a Split_Candidate, THE Analysis_Record SHALL state
   why (cohesive single responsibility, acceptable size) so the decision is
   auditable.
6. NO split SHALL be executed (no requirement/criterion moved, no folder created)
   as part of this analysis; a Split_Plan is a proposal that requires owner
   approval and its own requirements gate before execution.

### Requirement 3: Cross-Unit Consistency and Conflict Resolution

**User Story:** As an architect, I want each logical unit checked against all
others for consistency and conflicts, so that shared types, commands, config
keys, and terminology have a single owner and no contradictions.

**Source:** [CR-NR-056], [RR]

#### Acceptance Criteria

1. THE analysis SHALL build a Consistency_Matrix recording, across all Logical_
   Units: shared/public types, Command_IDs, configuration keys, panel/context
   names, and glossary terms, with the owning unit for each.
2. WHEN two Logical_Units define, own, or claim the same type, Command_ID, config
   key, panel name, or term, THE analysis SHALL record it as a Conflict with a
   proposed resolution (single owner + references) referencing the
   `terminology-map.md` and `api-consistency-report.md` where they exist.
3. WHEN a requirement in one unit contradicts a requirement in another
   (behaviour, ownership, or precedence), THE analysis SHALL record the
   contradiction and a proposed reconciliation, and SHALL NOT silently alter
   either requirement.
4. THE analysis SHALL verify that every cross-reference between units resolves to
   an existing requirement/criterion, and SHALL log dangling cross-references for
   correction.
5. THE Consistency_Matrix and its conflicts SHALL be stored under
   `docs/specs/project-analysis/consistency-matrix.md`.

### Requirement 4: Completeness and Incomplete-Work Register

**User Story:** As the project owner, I want each logical unit's development state
verified -- is the work complete and tested -- with every incomplete item logged
for action, so that the true remaining backlog is known.

**Source:** [CR-NR-056], [EI-3], [WF]

#### Acceptance Criteria

1. WHEN a Logical_Unit is analysed, THE analysis SHALL determine its development
   state by cross-checking its `tasks.md` markers, its TCR rows
   (`docs/quality/TCR.md`), and the presence of the corresponding crate code and
   tests; discrepancies between these three sources SHALL be recorded.
2. THE analysis SHALL classify each Logical_Unit as COMPLETE (all criteria have
   PASS/MANUAL TCR coverage and all tasks `[x]`), PARTIAL (some coverage/tasks
   outstanding), or NOT STARTED.
3. THE analysis SHALL produce a consolidated Incomplete_Work_Register under
   `docs/specs/project-analysis/incomplete-work-register.md` listing every
   pending task, orphaned requirement (requirement with no task), and NOT COVERED
   TCR row that lacks a task, extending the prior
   `ears-integration/incomplete-work-audit.md` rather than replacing it.
4. WHEN a task marker is inconsistent with actual completion (e.g. subtasks `[x]`
   but parent `[ ]`), THE analysis SHALL record it as a tracking fix (no gate
   required) with the exact correction.
5. THE analysis SHALL NOT mark any task complete that is not genuinely complete;
   verifying completion requires the criterion to have real test coverage, not
   merely a `[x]` marker.

### Requirement 5: Logging Enablement Audit

**User Story:** As a developer diagnosing bugs from logs, I want each logical
unit's code audited to confirm proper logging is enabled, so that runtime
behaviour and failures are diagnosable from log files.

**Source:** [CR-NR-056], [LOG]

#### Acceptance Criteria

1. WHEN a Logical_Unit backed by a crate is analysed, THE Logging_Audit SHALL
   assess whether the crate emits log records via `ff-logging` at appropriate
   levels: ERROR for failures, WARN for recoverable/degraded conditions, INFO for
   significant state transitions, and DEBUG for diagnostic detail, per
   logging-subsystem requirements.
2. THE Logging_Audit SHALL flag error paths that currently return or swallow an
   error without a log record (including `let _ = ...` discards and silent
   fallbacks) as logging gaps, recording file and location.
3. THE Logging_Audit SHALL flag any use of `println!` / `eprintln!` or ad-hoc
   stderr writes in library/binary code (outside tests and tools) that should be
   a structured log record.
4. THE Logging_Audit SHALL confirm that the two-phase logging init (logging-
   subsystem Requirement 11, CR-CH-015) is respected so records emitted before
   config load are not lost and are re-pointed to the configured directory.
5. THE analysis SHALL record logging gaps as actionable items in the
   Incomplete_Work_Register (Requirement 4.3), one entry per unit, so they can be
   fixed under the normal code operating mode.
6. THE Logging_Audit SHALL NOT modify source code; it records findings only.

### Requirement 6: Task Revision and Re-Ordering

**User Story:** As the project owner, I want existing incomplete tasks revised and
re-ordered to suit the analysis findings, so that the backlog reflects the new
logical-unit structure and dependency order.

**Source:** [CR-NR-056], [WF]

#### Acceptance Criteria

1. WHEN the analysis is complete for an Analysis_Wave, THE analysis SHALL produce
   a revised, dependency-ordered task sequence for the incomplete work in that
   wave, superseding stale ordering in the affected `tasks.md` files and in
   `project-master/tasks.md`.
2. WHEN a Split_Plan is approved for a Sub_Project, THE revised task sequence
   SHALL re-assign that Sub_Project's incomplete tasks to the target Logical_
   Units.
3. THE revised task sequence SHALL preserve every incomplete task (none dropped)
   unless a task is explicitly recorded as SUPERSEDED with the superseding task
   named (consistent with the prior audit's treatment of VCM Task 17).
4. Task-list edits that only revise ordering, add tracking fixes, or record
   supersession SHALL be treated as documentation maintenance (no requirements
   gate); any edit that adds new behaviour or new criteria SHALL trigger the
   normal requirements gate before it is written.
5. THE revised task sequence SHALL be reflected in `project-master/tasks.md` so a
   single master ordering exists.

### Requirement 7: Per-Sub-Project Commit Checkpoints

**User Story:** As the project owner, I want the analysis committed to Git at the
end of each sub-project, so that progress is incrementally saved and reviewable.

**Source:** [CR-NR-056]

#### Acceptance Criteria

1. WHEN the analysis of a Sub_Project is complete (its Analysis_Record written,
   its consistency/completeness/logging findings recorded, and its task revisions
   applied), THE work SHALL be committed to Git with a message identifying the
   Sub_Project and the analysis pass, and pushed.
2. Each per-Sub_Project commit SHALL contain only the analysis artifacts and
   task/doc revisions for that Sub_Project (and the shared registers updated for
   it), so history reads as one commit per analysed Sub_Project.
3. THE commit SHALL NOT include unrelated in-progress code changes; if the tree
   contains interleaved work, only the analysis Sub_Project's files SHALL be
   staged.
4. A final consolidation commit SHALL record the completed Consistency_Matrix,
   Incomplete_Work_Register, and revised master task ordering once all waves are
   analysed.

### Requirement 8: Analysis Integrity (No Silent Changes)

**User Story:** As the project owner, I want the analysis to be non-destructive
and auditable, so that findings are proposals I can review before anything
material changes.

**Source:** [CR-NR-056], [WF]

#### Acceptance Criteria

1. THE analysis SHALL NOT modify any source file under `crates/` (logging fixes,
   splits, and behaviour changes are executed later under the normal operating
   modes, not during analysis).
2. THE analysis SHALL NOT delete or rewrite existing requirement criteria; it
   records proposed reconciliations for owner approval.
3. THE analysis SHALL NOT execute any Split_Plan; splits require owner approval
   and their own requirements gate.
4. ALL analysis outputs SHALL use plain ASCII per `documentation.md` (box-drawing
   permitted in fenced diagrams only).
5. WHEN the analysis uncovers a finding that requires a decision (a split, a
   conflict reconciliation, a superseded task, a new requirement), THE analysis
   SHALL surface it to the owner rather than resolving it unilaterally.
