# Analysis Record: configuration-system

- **Wave**: 0 (foundation -- Platform Architecture, consumed by all higher crates)
- **Backing crate**: `ff-config`
- **Spec folder**: `docs/specs/configuration-system/`
- **Analysed**: Wave 0, task W0.3
- **Verdict**: COMPLETE (all 24 task groups [x], TCR PASS for ff-config rows).
  SPLIT CANDIDATE -- recommend splitting the spec into two sub-projects
  (core engine vs. enterprise/UI features). Crate split optional, deferred.

---

## 1. Scope summary

`ff-config` is the central settings-management layer. Requirements group cleanly
into two bands:

- **Core engine (Req 1-9)**: TOML format, six-layer override model
  (Defaults -> System -> User -> Profile -> Project -> Workspace), hot-reload
  with debounced file watching, named profiles, per-project overrides,
  EditorConfig integration, typed access API, plugin namespace scoping,
  schema + validation.
- **Enterprise / UI band (Req 15-18)**: Settings Context interactive dialog
  (Req 15, largely realised in `ff-desktop`), configuration audit logging
  (Req 16), settings export/import (Req 17), locked configuration keys (Req 18).

Requirement numbers 10-14 are absent from this spec (allocated to other
sub-projects / cross-cutting reqs). Not a defect, but noted (see PA-DOC-002).

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 365 req lines; 13 requirements (1-9, 15-18) | YES |
| 3+ distinct responsibilities | config engine; file-watch/reload; EditorConfig; plugin scoping; audit; export/import; locked keys; Settings UI | YES |
| 2+ crates | `ff-config` (engine + audit + export/import + locked) AND `ff-desktop` (Settings Context UI Req 15, 18.6) | YES |
| low-cohesion concern clusters | Req 1-9 (engine) vs Req 15-18 (enterprise/UI) are separable bands | YES |

4 of 4 criteria met -> **SPLIT CANDIDATE**.

### Recommended split (proposal only -- needs owner approval + own gate)

- **configuration-system** (retain): Req 1-9 core engine. Backing crate
  `ff-config` core modules (value, layer, loader, merger, store, access,
  provenance, watcher, reload, callback, profile, project, namespace,
  plugin_handle, paths, keys, init, schema/*, editorconfig/*, provider, error).
- **configuration-enterprise** (new spec folder): Req 16 audit, Req 17
  export/import, Req 18 locked keys. Currently `audit.rs`, `export_import.rs`,
  and the locked-key logic in `merger.rs`/`config_handle.rs`. These are additive
  enterprise features (Phase CQ roadmap) with low coupling to the core merge.
- **settings-context** (new spec folder OR fold into `menu-workspace`): Req 15
  + Req 18.6 UI. This is `ff-desktop` work (settings_panel, Settings_Menu),
  already partly tracked under `menu-workspace` Phase CW. Recommend folding
  Req 15/18.6 UI criteria into `menu-workspace` rather than a standalone folder,
  since Phase CW-impl already owns the Settings_Menu restructure.

Crate split is NOT recommended at this time: `ff-config` remains one cohesive
crate; the audit/export/locked modules are small (377 + 356 lines) and share the
`ConfigHandle` surface. Splitting the *spec* improves traceability without
churning the crate. Record as proposal; no action without gate.

## 3. Consistency / conflict

- Public types owned here (ConfigValue, ConfigLayer, ConfigHandle, ConfigError,
  EffectiveValue, Provenance, SchemaEntry, PluginConfigHandle, AuditEntry,
  ExportScope, ImportTarget, ImportSummary) -- see consistency-matrix.md. No
  duplicate ownership found; `ConfigProvider` trait is defined in `ff-core` and
  implemented here (correct direction, no conflict).
- Req 15.9 correctly annotates the Phase DB CR-CH-012 change (tab-kind ->
  Workspace_Kind `settings`) -- consistent with startup-and-session Req 21.3.
- Reserved namespaces (`logging`, `editor`, `theme`, `vfs`, `commands`, `layout`,
  `core`, `_session`) are consistent with the crate names those namespaces feed.
- No cross-unit conflict detected.

## 4. Completeness

- Tasks: all 24 task groups and subtasks marked `[x]`.
- TCR: ff-config rows for Req 15.4, 15.6, 16.1-16.6, 17.1-17.9, 18.1-18.5,
  18.7-18.8 all PASS (checkmark). Manual/UI rows deferred to ff-desktop:
  Req 18.6 (LOCKED badge) MANUAL, Req 12.6 (Settings persistence) MANUAL,
  Req 13.8 (About dialog) MANUAL -- these are ff-desktop UI-verification items,
  not ff-config gaps.
- Integration + property tests present (24.1-24.12, twelve documented
  properties, proptest >=100 iters). No incomplete engine work found.
- Completeness verdict: **COMPLETE** for the engine + enterprise bands. Only
  open items are manual UI verifications owned by ff-desktop/menu-workspace.

## 5. Logging audit

- `ff_logging` / `log_warn!` / `log_error!` / `log_info!` / `log_debug!` usage:
  31 hits across the crate -- WARN on parse/validation/IO failures (Req 1.6,
  3.6, 4.6, 5.7, 7.5, 7.6, 9.4, 16.4), DEBUG on unknown keys (Req 9.6) and
  locked-key override attempts (Req 18.4). Levels match the requirements.
- `println!`/`eprintln!`: 3 hits, ALL inside `///` doc-comment examples
  (callback.rs:103, plugin_handle.rs:228, profile.rs:312). NOT runtime code --
  no logging violation.
- Logging verdict: **adequate**; failure paths are logged at the required levels.

## 6. Findings logged

- **PA-STD-001** (rust-standards 400-line cap): six non-test source files exceed
  400 lines of non-test code -- `config_handle.rs` (809), `editorconfig/parser.rs`
  (626), `reload.rs` (510), `access.rs` (486), `init.rs` (446),
  `plugin_handle.rs` (432). REFACTOR (split by concern per rust-standards.md);
  no behaviour change, no gate. Logged in incomplete-work-register.
- **PA-DOC-002** (tracking): requirement numbering skips 10-14 in this spec.
  Confirm these are intentionally allocated elsewhere (cross-cutting / other
  sub-projects) and add a one-line note to the spec Introduction. Doc-only.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-config contributes 641
  non-ASCII matches in `.rs` (comments / doc-comment prose / separators).
  Rolled into the existing project-wide PA-LOG-001 cleanup item; not fixed here.
- **SPLIT proposal**: split the SPEC into core (Req 1-9) + enterprise (Req 16-18)
  + fold Settings UI (Req 15, 18.6) into menu-workspace. Proposal only; owner
  approval + own gate required (Req 8 of project-analysis).
