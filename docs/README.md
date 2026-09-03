# FileForge Workbench Documentation

This directory separates project inputs, approved specifications, decisions,
and work tracking.

## Where to look

| Area | Purpose |
|------|---------|
| [source-documents](source-documents/) | Original briefs, imported references, and research material |
| [requirements](requirements/) | Product-level requirements and source traceability |
| [specs](specs/) | Canonical feature requirements, designs, and implementation tasks |
| [reviews](reviews/) | Audits, gap analysis, and requirements review reports |
| [decisions](decisions/) | Architecture decision records |
| [status](status/) | Current project status, active work, and blockers |
| [working-notes](working-notes/) | Temporary notes that may later become formal documentation |
| [`../tools/`](../tools/) | Reusable project development and maintenance tools |

## Documentation rules

- Documents in `source-documents/` are preserved as reference material and
  should not be edited in place.
- Documents in `specs/` are the implementation source of truth.
- Every active feature specification should contain `requirements.md`,
  `design.md`, and `tasks.md` where applicable.
- Link new requirements back to their source using
  `requirements/source-register.md`.
- Move superseded material to `archive/` rather than deleting it.

## Current entry points

- [Current work dashboard](status/current-work.md)
- [Source register](requirements/source-register.md)
- [Project master status](specs/project-master/tasks.md)
- [Project readiness summary](specs/project-master/readiness-summary.md)
