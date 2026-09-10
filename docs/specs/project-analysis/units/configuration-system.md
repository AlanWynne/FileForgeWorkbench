# Analysis Record: configuration-system

- **Wave**: 0 (foundation -- central settings management, consumed by all crates)
- **Backing crate**: `ff-config`
- **Spec folder**: `docs/specs/configuration-system/`
- **Analysed**: Wave 0, task W0.3 (CR-NR-057 re-baseline)
- **Verdict**: FUNCTIONALLY COMPLETE (Req 1-9, 15-18 implemented, TCR PASS), with a
  TRACKING GAP: Phase CQ tasks 30-31 (Req 16 Audit, Req 17 Export/Import) are
  implemented and TCR-PASS but their task checkboxes remain `[ ]` (17 open).
  SPLIT CANDIDATE (13 reqs, 365 lines). Six 400-cap refactors.
- **CR-NR-057 impact**: NONE (config specs untouched). Re-verified from code.

---

## 1. Scope summary

`ff-config` is the central settings layer: TOML files, six-layer override
(Defaults->System->User->Profile->Project->Workspace), hot-reload, profiles,
per-project overrides, EditorConfig, typed access, plugin namespace scoping,
schema+validation, plus enterprise bands (Settings Context UI, audit, export/
import, locked keys). 13 requirements (1-9, 15-18; numbers 10-14 absent --
allocated elsewhere, PA-DOC-002).

365 req lines, 13 requirements, single backing crate.

## 2. Split candidacy -- CANDIDATE

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 365 lines AND 13 reqs | YES (both) |
| 3+ distinct responsibilities | config engine (Req 1-9); Settings UI (Req 15); audit (Req 16); export/import (Req 17); locked keys (Req 18) | YES |
| 2+ crates | `ff-config` (engine + enterprise) AND `ff-desktop` (Settings Context UI Req 15, 18.6) | YES |
| low-cohesion clusters | Req 1-9 (engine) vs Req 15-18 (enterprise/UI) | YES |

4/4. **SPLIT CANDIDATE.** Recorded as PA-SPLIT-001 (proposal only).

### Proposed split (proposal, owner-gated)

- Retain `configuration-system` = Req 1-9 core engine.
- New `configuration-enterprise` = Req 16 audit + Req 17 export/import + Req 18
  locked keys (Phase CQ features; `audit.rs`, `export_import.rs`, locked-key logic
  in merger/config_handle).
- Fold Settings UI (Req 15, 18.6) into `menu-workspace` (Phase CW already owns the
  Settings_Menu restructure).
Crate split NOT recommended (cohesive crate; modules small). SPEC split improves
traceability for the 13-req doc. Owner approval + own gate.

## 3. Consistency / conflict

- Public types owned here (ConfigValue, ConfigTable, ConfigLayer, ConfigHandle,
  ConfigError, EffectiveValue, Provenance, SchemaEntry, PluginConfigHandle,
  AuditEntry, ExportScope, ImportTarget, ImportSummary) -- sole owner ff-config.
  Added to consistency-matrix.
- `ConfigProvider` trait defined in ff-core, implemented here -- correct direction
  (confirmed W0.1). No conflict.
- Reserved namespaces (`logging`, `editor`, `theme`, `vfs`, `commands`, `layout`,
  `core`, `_session`) align with the crates those namespaces feed; consistent with
  plugin-architecture Req 8 scoping. No conflict.
- Req 15.9 correctly annotates the Phase DB CR-CH-012 change (tab-kind ->
  Workspace_Kind `settings`) consistent with startup-and-session Req 21.3.
- **PA-DOC-002 (re-confirmed)**: requirement numbering skips 10-14 (jumps 9 ->
  15). Confirm allocated elsewhere; add a one-line note to the spec Introduction.

## 4. Completeness -- TRACKING GAP

- Tasks: 233 `[x]`, 17 `[ ]`. The 17 open are Phase CQ Task 30 (Audit Logging,
  Req 16; 30.1-30.8) and Task 31 (Settings Export/Import, Req 17; 31.1-31.7).
- BUT both features ARE implemented and TCR-PASS:
  - `audit.rs` (377 lines) -- `AuditEntry`, `AuditFilter`, ring buffer,
    `query_audit_log`/`clear_audit_log` on ConfigHandle. TCR Req 16.1-16.6 PASS.
  - `export_import.rs` (356 lines) -- `ExportScope`, `ImportTarget`,
    `ImportSummary`, `export_settings`/`import_settings` on ConfigHandle. TCR
    Req 17.x PASS.
- So tasks 30-31 are STALE checkboxes, not missing code (same class as
  logging-subsystem Req 12 last pass). Req 18 (task 32) is `[x]`.
- Completeness verdict: **implementation complete; task tracking stale.** Logged
  PA-TRACK-001. Bookkeeping only (check tasks 30-31); no code, no gate.

## 5. Logging audit

- 31 log call sites; `ff-logging` declared and used. WARN on parse/validation/IO
  failures (Req 1.6, 3.6, 4.6, 5.7, 7.5, 7.6, 9.4, 16.4), DEBUG on unknown keys
  (Req 9.6) and locked-key override attempts (Req 18.4). Levels match the spec.
- `println!`/`eprintln!`: 3 hits, ALL inside `///` doc-comment examples
  (callback.rs, plugin_handle.rs, profile.rs). NOT runtime code -- no violation.
- Logging verdict: **adequate.**

## 6. Findings logged

- **PA-TRACK-001** (TRACKING-FIX): Phase CQ tasks 30 (Req 16 Audit) and 31 (Req 17
  Export/Import) remain `[ ]` (17 subtasks) although `audit.rs` + `export_import.rs`
  exist and their TCR rows (Req 16.1-16.6, 17.x) are PASS. Owner action: check
  tasks 30.1-31.7. Bookkeeping only; no code, no gate.
- **PA-SPLIT-001** (SPLIT PROPOSAL): 4/4 criteria. Split SPEC into core (Req 1-9)
  + `configuration-enterprise` (Req 16-18); fold Settings UI (Req 15, 18.6) into
  menu-workspace. Crate split NOT recommended. Owner approval + own gate.
- **PA-STD-002** (REFACTOR -- 400-line cap): six non-test files over the cap:
  `config_handle.rs` (809), `editorconfig/parser.rs` (626), `reload.rs` (510),
  `access.rs` (486), `init.rs` (446), `plugin_handle.rs` (432). Split by concern.
  REFACTOR, no gate.
- **PA-DOC-002** (TRACKING-FIX): requirement numbering skips Req 10-14 (9 -> 15).
  Confirm allocated elsewhere; add a note to the spec Introduction. Doc-only.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-config contributes 641
  matches (comments / doc-comment prose / separators). Rolled into project-wide
  PA-LOG-001; not fixed here.
