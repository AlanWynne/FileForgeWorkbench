# Analysis Record: database-tool (W5.4)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-database-tool` (integrated Database IDE, delivered as a
  workbench plugin -- spec adapts "DBeaver Community Edition" capabilities)
- **Spec files**: requirements.md (664 lines, 17 requirements), tasks.md
  (157 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy

NOT actionable as a split NOW -- the implementation is a foundation skeleton (see
PA-INCOMPLETE-014). The SPEC (17 reqs, 664 lines, 7 distinct panels) WOULD be a strong
split candidate once built (connection/driver/pool vs SQL-editor/execution vs
result-grid vs schema-browser vs ER-diagram vs data-transfer vs administration). But
there is no oversized code to split today (6 files, none over 350). Re-evaluate the
split after the IDE is actually implemented. No split proposed now.

---

## 2. Completeness -- PA-INCOMPLETE-014 (FALSE-POSITIVE-COMPLETE, HIGH)

The spec describes a FULL-FEATURED Database IDE (Req 1-17): plugin lifecycle, driver
registry, connection management + POOLING, SQL editor with dialect-aware highlighting +
autocomplete, query execution + parameter binding, sortable/filterable RESULT GRID with
cell editing, lazy-loading SCHEMA BROWSER, DATA TRANSFER wizards, zoomable ER DIAGRAM
canvas, ADMINISTRATION dashboards, async I/O (Req 13), multi-database (Req 14), command
integration (Req 15), VFS integration (Req 16), layout integration (Req 17).

The IMPLEMENTATION is a foundation skeleton:
- 6 source files, ~716 non-test lines TOTAL. Modules present: `connection/`, `driver/`,
  `sql/` -- 3 of the 17 subsystems, and those are TYPE-MODEL + registry logic
  (DriverRegistry, DriverCapabilities, DriverDefinition, error types), not working DB
  access.
- Cargo deps are ONLY `thiserror` + `serde`. There is NO dependency capable of
  implementing the spec: no sqlx/rusqlite (drivers/queries Req 2/6), no tokio (async
  I/O Req 13), no egui (ALL 7 panels Req 5/8/9/11/12), no ff-vfs (Req 16), no ff-command
  (Req 15), no ff-layout (Req 17). You cannot have a result grid, ER canvas, connection
  pool, or live DB connection with thiserror+serde alone.
- tasks.md frames the built work as "Wave A (Foundation): crate scaffold, error types,
  driver abstraction" -- i.e. the implementation deliberately stopped at the foundation.

YET all 157 sub-tasks are `[x]` and the unit is presented as complete. This is the
LARGEST false-positive-complete found in the analysis: ~14 of 17 requirements (all 7
panels, pooling, async, ER, transfer, admin, multi-DB, VFS/layout/command integration)
are UNIMPLEMENTED while tracking says done. Recorded PA-INCOMPLETE-014 (HIGH): re-open
the database-tool tasks -- mark the foundation subset actually done, RE-OPEN the panel /
pooling / async / integration tasks as `[ ]`, and correct the "complete" framing. This
is a tracking-integrity issue (the code is honest scaffold; the CHECKBOXES lie).

### External-product reference (note)

lib.rs + the spec state the tool "adapts DBeaver Community Edition capabilities". DBeaver
CE is EPL/Apache-licensed; "adapts capabilities" (feature parity) is fine, but if any
CODE/assets are ported, a licence review is needed. Flagging for the owner (no evidence
of copied code -- deps are only thiserror+serde). Non-blocking.

---

## 3. Cross-unit consistency

### Plugin -- registered (NOT an orphan)

`ff-database-tool` registers as a workbench plugin (Req 1) and is wired by name 5x in
shell/plugin. Not an orphan (like JES W5.1; unlike idcams W5.2). The Command Integration
(Req 15), VFS Integration (Req 16), Layout Integration (Req 17) are SPEC'd but the deps
(ff-command/ff-vfs/ff-layout) are absent -- consistent with PA-INCOMPLETE-014 (those
integrations are unbuilt).

### VFS discipline

0 raw std::fs -- but that is because there is almost no I/O yet (skeleton). Req 16 (VFS
integration) is unbuilt. Neutral (re-check once implemented; the JES raw-fs bypass
PA-CONFLICT-015 is the pattern to avoid when the DB file I/O lands).

### Public types and ownership

- DriverRegistry / DriverDefinition / DriverCapabilities, connection + SQL type models,
  error types -- sole-owned by `ff-database-tool`. No duplication (nothing else does DB).

---

## 4. Logging audit

- `ff_logging` / `log_*!`: 0; `ff-logging` dep: ABSENT (not dead).
- A database IDE is a very high-value logging site (connection open/close/pool, query
  execute + timing, transaction, errors) -- but with the implementation at skeleton
  stage, logging is premature. Recorded PA-LOG-043 (deferred, MEDIUM-when-built): add
  ff-logging + dev-logging on connection/pool/query lifecycle when the IDE is actually
  implemented (PA-INCOMPLETE-014). Deferred, not a dead-dep issue now.

---

## 5. Task revision proposals

- **PA-INCOMPLETE-014 (HIGH)**: correct the false-positive-complete tracking -- re-open
  the ~14 unbuilt requirements (panels, pooling, async, ER, transfer, admin, multi-DB,
  VFS/layout/command integration) as `[ ]`; keep only the foundation (driver/connection/
  sql type models + error) as done; fix the "complete" framing. Tracking + honest
  re-scoping. (Then a real implementation effort, owner-prioritised.)
- **PA-STD-058 (ASCII, doc-comment mojibake)**: lib.rs header has mojibake em-dashes
  (UTF-8 corruption, e.g. "ff-database-tool <bad> Integrated Database IDE Plugin";
  "Connection Management <bad> Create..."). Replace with `--`. REFACTOR, no gate.
- **PA-LOG-043 (deferred, MEDIUM-when-built)**: connection/pool/query dev-logging once
  implemented.
- **PA-TCR-029**: TCR has 0 rows -- but with the IDE unbuilt, defer full TCR to the
  implementation; add rows for the foundation types now. No code.
- **DBeaver-CE licence note**: owner review if any code/assets are ported (feature
  parity alone is fine). Non-blocking.

---

## Summary

database-tool (`ff-database-tool`) is specified as a full-featured integrated Database
IDE plugin (17 reqs, 664 lines -- connection pooling, SQL editor, result grid, schema
browser, ER diagram, data transfer, administration; adapts DBeaver CE). The headline
finding is PA-INCOMPLETE-014 (HIGH -- the LARGEST false-positive-complete in the
analysis): despite 157/157 tasks `[x]`, the implementation is a FOUNDATION SKELETON --
6 files, ~716 non-test lines, modules only for driver/connection/sql TYPE MODELS, and
Cargo deps of ONLY thiserror+serde (no sqlx/rusqlite/tokio/egui/ff-vfs/ff-command/
ff-layout -- nothing that could implement the panels, pooling, async, or integrations).
tasks.md itself frames the built work as "Wave A (Foundation)". ~14 of 17 requirements
are unimplemented while tracking says done -- re-open them. The code is honest scaffold;
the checkboxes overstate it. It IS a registered plugin (wired 5x -- not an orphan).
Minor: lib.rs doc-comment mojibake (PA-STD-058); logging + full TCR deferred to the
real implementation (PA-LOG-043, PA-TCR-029); a non-blocking DBeaver-CE licence note.
