# Format Change Proposal -- Persist Settings_Namespace_View in Session (Task 14.5)

Status: SUPERSEDED BY CR-CH-012 (Phase DB) -- retained as analysis history only.
Change request: CR-CH-011 (see docs/status/change-log.md) -- superseded.
Blocks: menu-workspace Task 14.5; cw-requirements.md Req 10.6; configuration-system Req 15 criterion 15.9
Author context: Phase CW-impl follow-up

> IMPORTANT: Do NOT implement the recommendation in this document. The point fix
> proposed here (adding a `PersistedTabKind::SettingsPanel` variant plus a flat
> `settings_namespace_filter` field) is REPLACED by the descriptor-based
> persistence model in startup-and-session Requirement 21 (CR-CH-012). Under that
> model the closed `PersistedTabKind` enum is removed entirely, and the Settings
> namespace filter persists as a `CustomWorkspace { workspace_kind = settings,
> params = { namespace } }` Workspace_Descriptor (Requirement 21.3). This file is
> kept only for the baseline analysis in sections 2-4, which remains accurate.

---

## 1. Problem

menu-workspace Task 14.5 and cw-requirements.md Req 10.6 require that a
Settings_Namespace_View (a Settings panel opened with a namespace filter such as
`editor.`) survives an application restart: on next launch it must reopen as a
Settings panel with the same namespace filter applied. configuration-system
Req 15 criterion 15.9 states the same intent.

Task 14.5 is marked BLOCKED because implementing it changes the on-disk session
format owned by the `ff-session` crate, and the workflow rules require a
deliberate format decision plus human review before a persisted format changes.
This document is that decision record.

## 2. Verified baseline (what the code actually does today)

Confirmed by reading the source, not assumed:

- `crates/ff-session/src/session_state.rs`
  - `PersistedTabKind` is a plain (data-free) enum with variants: `FileEditor`
    (default), `PrimaryOptionMenu`, `FilesPanel`, `FileExplorerPanel`,
    `SearchResults`, `PluginManager`, `EventLog`, `Untitled`. It derives
    `Serialize, Deserialize` with `#[serde(rename_all = "snake_case")]`.
  - There is NO `MenuWorkspace` variant, and NO `SettingsPanel` variant, despite
    a comment in `crates/ff-desktop/src/menu_workspace/mod.rs` that refers to
    `PersistedTabKind::MenuWorkspace { file_path }`. That comment is aspirational
    and does not match the enum.
  - `TabState` stores all per-tab data in flat fields (`tab_id`, `tab_kind`,
    `uri`, viewport/caret fields, `selections`, `language_override`,
    `is_pinned`, `zoom_offset`, `workspace_name`). No variant carries its own
    payload today; `tab_kind` uses `#[serde(default)]`.

- `crates/ff-desktop/src/session_manager.rs`
  - On save, `TabKind::SettingsPanel` and `TabKind::MenuWorkspace` both map to
    `None` -- they are NOT persisted at all right now. Only `FileEditor` (with a
    path), `FilesPanel`, and `FileExplorerPanel` are written.

- `crates/ff-desktop/src/shell/update.rs`
  - The restore loop reconstructs a tab only when `session_tab.uri` is `Some`:
    `for session_tab in &state.tabs { if let Some(uri) = &session_tab.uri { open_file(uri) } }`.
  - Non-file tabs are never reconstructed by this loop. The POM is re-created
    separately by `insert_pom_tab` / `ensure_pom_tab_present`. So even the
    `FilesPanel` / `FileExplorerPanel` kinds that ARE saved are not currently
    re-opened on restore by this path.

Consequence: two gaps, not one. Restoring a namespace-filtered Settings tab
requires (a) a way to persist the filter and (b) a restore branch that
reconstructs non-file tabs. Adding an enum variant alone is insufficient.

## 3. Design goals

1. Persist enough to reopen a Settings tab with its namespace filter.
2. Do not break existing session.toml files written by prior versions
   (backward compatibility -- old files must still load).
3. Keep the change minimal and consistent with the existing flat-field style of
   `TabState`.
4. Avoid scope creep: fix Settings_Namespace_View persistence only. Do not, in
   this change, take on full restore of every non-file tab kind.

## 4. Options considered

### Option A -- Data-carrying enum variant

Add `PersistedTabKind::SettingsNamespaceView { namespace: String }`.

- Pros: self-describing; groups the filter with the kind.
- Cons: introduces the first data-carrying variant into a snake_case-tagged
  enum. TOML serialisation of an internally-untagged/externally-tagged enum
  variant with a struct payload is awkward and changes the shape of the
  `tab_kind` field from a bare string to a table. That is a heavier format shift
  and higher back-compat risk for a field currently written as a simple string.

### Option B -- New flat field on TabState (RECOMMENDED)

Add `PersistedTabKind::SettingsPanel` (data-free, consistent with the other
kinds) plus a new flat field on `TabState`:

```
/// Namespace filter for a restored Settings panel (e.g. "editor.").
/// None for the full/unfiltered Settings panel.
/// Validates: configuration-system Req 15.9; menu-workspace Req 10.6
#[serde(default, skip_serializing_if = "Option::is_none")]
pub settings_namespace_filter: Option<String>,
```

- Pros: matches the existing flat-field pattern (`workspace_name`,
  `language_override` are already `Option<String>` with `#[serde(default)]`);
  `tab_kind` stays a simple string; old files load unchanged because the field
  defaults to `None`; new files omit the field entirely when there is no filter.
- Cons: the filter field is only meaningful for `SettingsPanel` tabs (a mild
  coupling), same as `uri` being meaningful mainly for `FileEditor`.

### Option C -- Reuse the existing `uri` field to carry the namespace

Store `settings:editor.` in `uri` for a Settings tab.

- Pros: zero schema change.
- Cons: overloads a field named `uri` with a non-URI value; the restore loop
  keys off `uri.is_some()` to call `open_file`, which would try to open the
  fake URI as a file. Rejected -- fragile and misleading.

Recommendation: Option B. It is the smallest change that preserves the
string-valued `tab_kind`, keeps full backward compatibility via `serde(default)`,
and follows the crate's established flat-field convention.

## 5. Recommended format change (Option B)

### 5.1 ff-session (`session_state.rs`)

1. Add a data-free variant to `PersistedTabKind`:
   ```
   /// The Settings panel tab (POM option 0). May carry a namespace filter via
   /// TabState::settings_namespace_filter.
   SettingsPanel,
   ```
2. Add the flat field to `TabState` (and to its `Default` impl as `None`):
   ```
   #[serde(default, skip_serializing_if = "Option::is_none")]
   pub settings_namespace_filter: Option<String>,
   ```

Serialised shape (new, filtered):
```
[[tabs]]
tab_id = "7"
tab_kind = "settings_panel"
settings_namespace_filter = "editor."
```
Serialised shape (new, unfiltered): `tab_kind = "settings_panel"` with the
filter field omitted. Old files: unchanged; the new field defaults to `None`
and unknown-to-old `tab_kind` values never appear in them.

### 5.2 ff-desktop save (`session_manager.rs`)

Change the two `TabKind::SettingsPanel => None` arms (in `save` and
`save_with_workspace`) to emit a `SessionTabState` with
`tab_kind: PersistedTabKind::SettingsPanel` and
`settings_namespace_filter: <the panel's current namespace_filter>`. (The
runtime filter already exists as `SettingsPanelState::namespace_filter`, added
in Task 14.1.) `MenuWorkspace` stays `None` in this change -- out of scope.

### 5.3 ff-desktop restore (`shell/update.rs`)

Extend the restore loop so that, in addition to the current
`if let Some(uri)` file branch, it reconstructs a Settings panel when
`session_tab.tab_kind == PersistedTabKind::SettingsPanel`, opening the Settings
tab and applying `settings_namespace_filter` (via the existing Task 14.2 routing
that pre-populates the filter). This is the new "restore a non-file tab" branch;
it is deliberately scoped to `SettingsPanel` only.

## 6. Backward and forward compatibility

- Old session.toml (no `settings_namespace_filter`, no `settings_panel` kind):
  loads unchanged; `serde(default)` fills the field with `None`. No migration
  step, no schema-version bump required.
- New session.toml read by an older build: the older `PersistedTabKind` has no
  `settings_panel` variant, so deserialisation of that tab would fail. Mitigation
  options to choose from at implementation time: (a) rely on the existing
  corrupt/partial-session fallback (empty state) -- acceptable because Settings
  is a re-openable panel, not user data; or (b) keep the whole-file load
  tolerant so one unknown tab is skipped rather than discarding the session.
  Recommend documenting (a) as the accepted behaviour since downgrades are rare
  and lossless for actual documents.

## 7. Test plan (TDD -- written before implementation)

ff-session unit tests:
- `settings_panel_tab_kind_round_trips` -- `tab_kind = "settings_panel"`
  serialises and deserialises. Validates: Req 10.6.
- `settings_namespace_filter_round_trips` -- a `TabState` with
  `settings_namespace_filter = Some("editor.")` survives TOML round-trip.
  Validates: Req 10.6.
- `old_session_without_filter_field_loads_as_none` -- a fixture TOML lacking the
  field deserialises with `settings_namespace_filter == None`. Validates:
  backward compatibility.

ff-desktop tests:
- `settings_namespace_view_persisted_on_save` -- saving a shell with a filtered
  Settings tab writes `tab_kind = "settings_panel"` and the filter.
- `settings_namespace_view_restored_with_filter` -- loading that state reopens a
  Settings tab with the namespace filter applied. Validates: Req 10.6,
  configuration-system Req 15.9.

## 8. Documentation updates required on approval

- `docs/specs/menu-workspace/tasks.md`: unblock Task 14.5 (remove BLOCKED note,
  add the format-change subtasks; keep `[ ]` until implemented).
- `docs/specs/menu-workspace/design.md`: add a design section referencing this
  proposal (the `SettingsPanel` variant, the flat filter field, the new restore
  branch).
- `docs/specs/configuration-system/requirements.md`: no criterion change --
  Req 15.9 already states the intended behaviour; note it is now covered.
- `docs/quality/TCR.md`: change the Req 10.6 row from NOT COVERED to reference
  the new tests once they pass; add ff-session rows for the round-trip tests.
- `docs/status/change-log.md`: CR-CH-011 (added by this proposal) moves to
  IN PROGRESS on approval, DONE when merged.

## 9. Decision requested

Approve Option B (data-free `SettingsPanel` variant plus flat
`settings_namespace_filter` field), or select Option A / another approach. No
source outside `docs/` changes until this is approved and a separate
implementation instruction is given.
