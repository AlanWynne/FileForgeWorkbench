# Current Work Dashboard

Use this page first to see what the project is working on. Keep the status
summary short and link to the detailed task list.

## Status key

| Status | Meaning |
|--------|---------|
| `ACTIVE` | Work is currently being implemented |
| `NEXT` | Ready to start after the active item |
| `BLOCKED` | Cannot proceed until a dependency or decision is resolved |
| `DONE` | Implemented and validated |
| `DEFERRED` | Deliberately outside the current release |

## Work areas

| Area | Status | Current focus | Detailed tracking |
|------|--------|---------------|-------------------|
| Workspace and project master | ACTIVE | Keep implementation status and dependencies current | [project-master tasks](../specs/project-master/tasks.md) |
| Dataset storage | NEXT | Implement SQLite-backed VSAM KSDS record provider | [dataset-catalog tasks](../specs/dataset-catalog/tasks.md) |
| Feature specifications | NEXT | Select the next incomplete feature task | [feature specs](../specs/) |
| Requirements quality | NEXT | Resolve review findings and maintain traceability | [requirements review](../reviews/requirements-review/) |
| Deferred connectors | DEFERRED | Network, FTP/SFTP, mainframe, and cloud connectors | [connector specs](../specs/) |

## Before starting work

1. Choose one feature from the table above.
2. Read its `requirements.md` and `design.md`.
3. Update its `tasks.md` before implementing.
4. Record any new source material in
   [`../requirements/source-register.md`](../requirements/source-register.md).
5. Update this dashboard when the active focus changes.

## Active work item

**Current focus:** EI-6 complete. EARS integration workflow fully done (EI-0 through EI-6).
All 13 EARS phases (BW-CI) are gated with requirements and tasks. No implementation yet.

Three parallel streams are now ready to proceed:
- Stream 1 (dataset architecture): BV.1 -> BS.8 -> ... -> BU.9
- Stream 2 (EARS P1 implementation): BW -> BX -> BY -> BZ -> CA -> CB -> CC -> CD
- Stream 3 (EARS P2 implementation): CE -> CF -> CG -> CH -> CI

Recommended next: BV.1 (small, no deps) or BS.8 (resumes dataset architecture stream).

**Completed:** EI-6 -- project-master reorganised, deferred-requirements.md created,
ff-vfs task gap filled (Tasks 13-16 added to virtual-file-system/tasks.md),
VCM Task 17 marked SUPERSEDED BY BU.7.

## Project records

- [Bug register](bugs.md)
- [Change log](change-log.md)
