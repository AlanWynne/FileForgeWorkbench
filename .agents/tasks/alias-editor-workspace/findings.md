# Findings: Alias Editor Workspace Context (read-only investigation)

Investigation for the owner's follow-up to CR-NR-112: "We need a workspace for
creating and saving aliases." CR-NR-112 (parameterised aliases) treats aliases
purely as hand-edited TOML DATA; there is NO in-app surface to create, edit, or
save an alias. This findings pass grounds a NEW requirement -- an Alias Editor
Workspace Context -- in the real code it would build on, so the gate that follows
(step a) can write criteria without guessing.

ALL work below was read-only. No source file was changed and no spec doc was
edited. Every claim is grounded in the live source read during this
investigation; paths are to the current worktree.

Relationship to CR-NR-112 (settled): CR-NR-112 adds the 4-field alias record
(root, alias name, localisation key, parameter template) and the single-pass
expansion in the `rewrite_verb_to_root` pre-pass. This CR adds the UI/persistence
to AUTHOR those entries. It builds ON CR-NR-112's data model; it does not change
expansion or dispatch.

---

## Executive summary (read this first)

- There is NO alias-management UI today and NO alias-management spec. A grep for
  "alias" across `docs/specs/command-configurator/` and
  `docs/specs/command-completion/` returns nothing. Aliases are authored only as
  DATA in `i18n/<locale>/aliases/<env>.toml`. (Item 1)
- Alias loading is READ-ONLY and lives in the LOCALISATION layer
  (`shell/alias_reader.rs` -> `LocaleAliasStore`). There is NO production writer:
  the only code that writes an alias TOML is a `#[cfg(test)]` helper. A Save path
  must be built from scratch (an alias serialiser + a writer), mirroring the
  Menus Editor's serialise+write. (Item 2)
- The Menus Editor Context (`shell/menus_editor.rs` + `menus_editor_panel`) is the
  EXACT analog to copy: it lists editable TOML definitions, holds a working copy,
  edits rows (add/delete/move/field edits), VALIDATES with the loader's own
  validator (so the editor cannot produce a file the loader rejects), serialises,
  writes `<menus_dir>/<name>.toml`, and refreshes the list. The Alias Editor is
  the same shape over alias rows. (Item 3)
- The `WorkspaceContext` framework is the required wiring: implement
  `WorkspaceContext::render(&mut self, ui, &mut ShellServices) -> InteriorFocus`,
  dispatch via `render_workspace_context` (owned-panel swap), give the first
  control a stable `egui::Id`, and ship the full-shell first-Tab egui_kittest
  test. This is MANDATORY per `workspace-conformance.md` / `wiring-standard.md`.
  (Item 4)
- A navigable editor Context is registered as a `WorkspaceKind::CommandConfigurator`
  descriptor carrying an `editor` param (the Theme/Menus/Keys/Kinds editors all
  reuse `CommandConfigurator` with `editor = "theme"|"menus"|"keys"|"kinds"`). An
  Alias Editor most naturally adds `editor = "aliases"` under the SAME Kind -- no
  new WorkspaceKind, matching the existing configurator-family pattern. (Item 5)
- THE load-bearing open question is WHERE a user-authored alias is SAVED. Today
  the only alias surface is per-locale localisation DATA under
  `i18n/<locale>/aliases/`, which is SHIPPED data, arguably not a user-writable
  location, and is entangled with CR-NR-109/111 localisation. A language-
  independent shorthand (the owner's `ecob`) does not belong in a per-locale file.
  The owner has indicated a preference for a USER-SCOPED alias file separate from
  the localisation catalogues; this findings pass confirms that is feasible and
  the cleaner choice (Item 6), but it introduces a NEW load path the resolver must
  read, which is a design decision for the gate.

Next free change-request id: re-verify at gate time. At the parameterised-aliases
gate the next free was CR-NR-112 (now allocated), so this CR is most likely
**CR-NR-113** -- RE-VERIFY against `docs/status/change-log.md` before writing, as
other gates may allocate in parallel.

---

## Item 1: No alias-management UI or spec exists today

EVIDENCE:
- `docs/specs/command-configurator/requirements.md` -- grep for "alias"/"Alias":
  NO matches. The configurator spec covers user-defined COMMAND DEFINITIONS and
  external programs, not aliases.
- `docs/specs/command-completion/` -- (per the CR-NR-112 findings) grep for
  "alias" across its markdown: NO matches.
- So this CR does NOT modify an existing alias-management surface; it CREATES the
  first one. The alias DATA format is owned by `docs/specs/localization/`
  (CR-NR-109/111) and extended by CR-NR-112 (the `command_template` field).

IMPLICATION: the Alias Editor is a NEW Workspace Context. Its natural spec home is
`docs/specs/command-configurator/` (it already owns user-defined command
authoring and the Command Configurator Context), with the alias DATA format
cross-referenced to `localization` and the record cross-referenced to CR-NR-112
(command-semantics Req 14 / command-framework Req 19).

---

## Item 2: Alias loading is read-only and lives in the localisation layer -- NO writer exists

EVIDENCE (exact): `crates/ff-desktop/src/shell/alias_reader.rs`.

- The entire module is a READER. `LocaleAliasStore` seeds identity tables
  (`seed`), overlays the active locale's on-disk DATA (`apply_overlay_from_dir`
  -> `read_catalogue_from_dir` -> `parse_alias_toml`), and exposes `table_for`.
  There is no `save`, no `serialise`, no production `fs::write`.
- The ONLY code writing an alias TOML is the test helper `write_alias_file`
  (alias_reader.rs:336, under `#[cfg(test)]`). A repo-wide grep for
  `serialise.*alias` / `write.*alias` / `fn.*save.*alias` finds only that test
  helper. CONFIRMED: no production alias serialiser or writer.
- The on-disk row shape (`AliasRowToml`, alias_reader.rs:45): `surface`,
  `canonical`, `#[serde(default)] localisation_key`. CR-NR-112 adds a
  `#[serde(default)] command_template`. An Alias Editor Save must serialise these
  FOUR fields back to `[[alias]]` rows.
- The store is re-seeded+overlaid on a `ui.locale` change via the process-global
  `ACTIVE_ALIAS_STORE` slot (`reload_active_alias_store_for_locale`). A Save would
  need to trigger a reload (or an equivalent in-place refresh) so the edited alias
  is live without a restart -- mirror how the Menus Editor refreshes its list +
  reloads the working menu after a write.

IMPLICATION: a Save path is NET-NEW work: (a) an alias SERIALISER (rows -> TOML
text), (b) a WRITER to the chosen location (Item 6), (c) a post-write REFRESH of
the live `LocaleAliasStore`. Validation should reuse the overlay's existing
within-environment collision discipline (`AliasCatalogue::load_into_keyed` already
rejects a surface mapped to two roots) so the editor cannot save a file the
reader would reject -- the same "editor validates with the loader's validator"
contract the Menus Editor uses (Item 3).

---

## Item 3: The Menus Editor Context is the exact analog to copy

EVIDENCE (exact): `crates/ff-desktop/src/shell/menus_editor.rs` +
`crate::menus_editor_panel` (pure render) + menu-workspace Requirement 13.

Shape the Alias Editor mirrors one-to-one:
- `open_menus_editor` -- refreshes the available-definitions list, loads a working
  copy, and `navigate_to` a `CommandConfigurator` descriptor IN PLACE (so
  END/RETURN returns to the opener via the per-tab Navigation_Stack).
- `apply_menus_editor_action(MenusEditorAction)` -- the shell-side effect handler:
  `Select` (load working copy), `AddOption` / `DeleteOption` / `MoveOptionUp` /
  `MoveOptionDown` (structural edits), `Save` / `SaveAs`. Per-field edits are
  applied DIRECTLY on the mutable working copy by the render (B054), so only
  structural changes surface as actions. The Alias Editor's action enum is the
  same family: `Select`, `AddRow`, `DeleteRow`, `Save`, `SaveAs`.
- `save_working_menu` -- VALIDATES with the loader's OWN validator
  (`loader::validate_menu`) BEFORE writing, so "the editor cannot produce a file
  the loader rejects"; on success writes, refreshes the list, reloads the working
  copy.
- `write_menu_file` -- `create_dir_all(menus_dir)`, slugify the name to a file
  stem, `serialiser::serialise(menu)`, `fs::write`.
- `refresh_menus_editor_list` -- rebuild the list from built-ins + every user
  `*.toml` on disk.

The pure render (`menus_editor_panel::render`) returns the action; the shell
stashes it on `pending_action` and applies it after the owned-panel swap (the
CR-NR-078 pattern). The Alias Editor render is pure the same way.

IMPLICATION: the Alias Editor is low-risk because the pattern is proven. The only
genuinely new pieces are the alias serialiser/writer (Item 2) and the save-
location decision (Item 6); the Context scaffolding, action enum, validate-then-
write flow, list refresh, and in-place navigation are a direct copy of the Menus
Editor.

---

## Item 4: The WorkspaceContext framework the editor MUST use

EVIDENCE (exact): `crates/ff-desktop/src/shell/workspace_context.rs`,
`shell/render.rs::render_workspace_context`, `theme_editor_panel.rs`.

- Implement `WorkspaceContext for AliasEditorState`:
  `fn render(&mut self, ui: &mut egui::Ui, services: &mut ShellServices) -> InteriorFocus`.
  The render stashes its action on `pending_action` (as Theme/Menus do) and
  returns `InteriorFocus`.
- Give the FIRST interactive control (e.g. the definition/surface selector combo,
  or the first row's surface field) a STABLE `egui::Id` captured each frame on
  `first_interior_id`; return `InteriorFocus { first, last }` (or
  `InteriorFocus::single(id)` when one control guarantees presence).
- Dispatch via the owned-panel swap: `mem::take` the panel ->
  `render_workspace_context(ctx, ui, &mut panel)` -> put it back -> apply the
  stashed action. Do NOT hand-write the focus latch.
- MANDATORY test (`workspace-conformance.md`): a full-shell egui_kittest
  `build_eframe` first-Tab test asserting the first Tab from the command field
  lands EXACTLY on the reported first interior control (no phantom stop). Model on
  `full_shell_config_first_tab_focuses_filter_field` /
  `full_shell_theme_editor_first_tab_focuses_theme_selector`.

IMPLICATION: this is boilerplate the project already enforces; no new mechanism.
The focus contract is compiler-enforced by the trait return type.

---

## Item 5: Registration as a navigable Workspace Kind

EVIDENCE (exact): `crates/ff-desktop/src/shell/nav_stack.rs:76-95`.

- The Theme / Menus / Keys / Kinds editors ALL reuse
  `WorkspaceKind::CommandConfigurator` with a distinct `editor` marker param:
  `editor = "theme" | "menus" | "keys" | "kinds"`. The descriptor reconstruct arm
  routes on that param to open the right editor.
- An Alias Editor fits the SAME pattern: `WorkspaceKind::CommandConfigurator` with
  `editor = "aliases"`. No new `WorkspaceKind` variant is needed, matching the
  configurator-family convention and keeping session persistence
  (`WorkspaceDescriptor`) unchanged in shape.
- A command verb opens it (e.g. `ALIASES`), registered the framework-conformant
  way (classified by `resolve_target` as a `CustomWorkspace`/menu target, or a
  menu option in `menus/*.toml`), reaching `open_alias_editor` which `navigate_to`
  the descriptor in place -- exactly as `open_menus_editor` does. No new
  dispatcher.

IMPLICATION: registration is a small additive change: one `editor = "aliases"`
descriptor arm in `nav_stack.rs` + `nav_reconstruct.rs`, one `open_alias_editor`,
one command verb / menu option. Framework-conformant; no framework change.

---

## Item 6: THE open decision -- where user-authored aliases are SAVED

The owner (this conversation) chose a USER-SCOPED alias file SEPARATE from the
per-locale localisation catalogues. This findings pass confirms that is feasible
and names the consequences the gate must settle.

Today's only alias surface:
- `i18n/<locale>/aliases/<env>.toml` -- per-locale localisation DATA, read by
  `LocaleAliasStore`. This is SHIPPED data (under the resolved `i18n/` root), it
  is entangled with CR-NR-109/111 localisation intent, and a language-independent
  shorthand (`ecob`) has no natural locale home there. Writing user edits into the
  shipped tree is also questionable (it may be a read-only install location).

Recommended (owner-confirmed direction) -- a user-scoped alias store:
- A user-writable file, e.g. under the workbench data dir (the same place macros
  use: `dirs::data_dir()/FileForgeWorkbench/...`, see `shell/actions.rs:28`) or a
  workspace-local `aliases/` dir, holding `[[alias]]` rows with the 4 fields.
- The resolver (`LocaleAliasStore`) must gain a load of this user file layered on
  top of the identity base, analogous to `apply_overlay_from_dir` but from the
  user location and locale-independent. This is a NEW load path -- a design
  decision the gate must specify (precedence vs the per-locale overlay; whether a
  user alias can shadow a localisation surface; collision discipline).
- SEPARATION from CR-NR-111: a per-locale localisation alias (surface -> root with
  a localisation key) stays in `i18n/<locale>/aliases/`; a user shorthand (with a
  parameter template, no localisation key) lives in the user file. The Alias
  Editor primarily authors the USER file; whether it can ALSO edit the per-locale
  localisation catalogues is a scope decision (recommend: v1 edits the user file
  only; localisation catalogues remain DATA authored out-of-band, to avoid
  colliding with CR-NR-111).

DESIGN QUESTIONS the gate must answer (surfaced, not decided here):
1. Exact save location + format of the user alias file (data dir vs workspace;
   one file vs per-environment files).
2. The resolver's load + precedence order for the user file relative to the
   per-locale overlay and the identity base (and whether a user alias may shadow a
   built-in root or a localisation surface -- recommend it may NOT shadow a
   built-in root, mirroring the Target_Resolution shadowing rule).
3. Scope: does the editor author ONLY the user file (recommended for v1), or also
   the per-locale localisation catalogues (risks CR-NR-111 overlap)?
4. Validation reuse: reuse the overlay's within-environment collision rejection so
   a saved file cannot break the reader.
5. Live refresh after Save: re-seed/overlay the `LocaleAliasStore` in place (reuse
   the `ACTIVE_ALIAS_STORE` reload seam) so a saved alias works immediately.

---

## Spec areas and deliverables the follow-on gate (step a) will target

- Primary spec: `docs/specs/command-configurator/` (the Alias Editor Context:
  requirements for listing, creating, editing the 4-field record, validating, and
  saving; the open/navigate verb; the first-Tab focus criterion). READ its current
  `requirements.md` / `design.md` / `tasks.md` first; number new EARS to avoid
  collision.
- Cross-reference (do NOT contradict): `docs/specs/localization/` (the alias DATA
  format + CR-NR-111 Req 12), command-semantics Req 14 + command-framework Req 19
  (CR-NR-112 record + expansion), and `workspace-kinds` / `menu-workspace` Req 13
  (the editor-Context pattern + CommandConfigurator Kind).
- Deliverables will be: a change-log entry (RE-VERIFY next free CR-NR, likely
  CR-NR-113); requirements.md criteria; design.md (the AliasEditorState + action
  enum modelled on MenusEditorAction, the serialiser/writer, the user-file save
  location + resolver load path, the WorkspaceContext wiring + first-Tab test, the
  CommandConfigurator `editor = "aliases"` registration, and an explicit
  framework-conformance statement); tasks.md; NOT-COVERED TCR rows; ROADMAP
  tracking; a gate-summary.

NO framework change is anticipated: the Context uses the existing
`WorkspaceContext` + `render_workspace_context` + `CommandConfigurator` Kind +
single-dispatch verb. The one genuinely new mechanism is the USER alias file +
its resolver load path (Item 6) -- that is a NEW persistence/load location, which
the gate must specify and the owner must confirm, but it is additive and does not
reshape a core framework type.
