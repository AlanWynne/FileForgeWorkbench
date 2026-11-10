# Gate Summary -- CR-CH-072 Alias-Model Simplification

**Status:** GATE AUTHORED / PENDING OWNER APPROVAL (docs-only; NO source edits).
**Change id:** CR-CH-072 (next free id; last was CR-CH-071, re-verified against
`docs/status/change-log.md`).

## The simplified model (what the owner approved)

The whole point of the alias work was LOCALISATION CUSTOMISATION OF MENUS: a
foreign-language user maps menu commands to aliases in his own language, in-app,
and the menus localise from that. The delivered architecture failed that two ways
the owner rejected: (1) THREE interpretation layers for a localised command --
alias row -> `localisation_key` -> `verb-*` Fluent catalogue entry; (2) TWO alias
stores, and the one that actually localises the menus (shipped
`i18n/<locale>/aliases/*.toml`) was invisible and uneditable in-app.

CR-CH-072 collapses this to the SIMPLEST model that meets the requirement:

- **ONE-LAYER alias.** The alias row maps a SURFACE string DIRECTLY to a ROOT
  command; the localised surface (e.g. `INSTELLINGS`) lives IN the row. The
  `localisation_key -> verb-*` command-surface indirection is REMOVED. (`.ftl`
  still localises DESCRIPTIONS and TITLES -- unchanged.) The CR-NR-112 parameter
  template (`ecob -> EDIT ...(&1)`) is RETAINED on the row -- it is a shorthand
  feature, not a localisation layer.
- **TWO-LEVEL resolution.** The active Command Environment checks its alias table,
  then its roots; on reject FFCMD does the same. Exactly two levels, one path, for
  all commands -- the existing `rewrite_verb_to_root` pre-pass + active-env front
  door, no new dispatcher.
- **MENU OPTIONS REFERENCE AN ALIAS.** Render and dispatch both flow through the
  one alias -> root path; the bespoke render-time reverse-derivation is retired.
- **ONE locale-aware store.** Shipped read-only seed (`i18n/<locale>/aliases`)
  overlaid by a USER per-locale override store; the Alias Editor shows the active
  locale's EFFECTIVE table and writes edits to the user override. The Alias Editor
  becomes the single in-app place to view/customise localised menu-command aliases.

Observable behaviour preserved: POM + Settings render fully in `af`; commands
dispatch to the English root; the template expands; the CR-CH-071 ISPF save
semantics work over the override; the single front door, active-wins, code-only
built-in menus, and the WorkspaceContext focus contract are unchanged.

## Supersession / amendment map

| Prior | Disposition |
|-------|-------------|
| CR-NR-109 loc Req 11.1/11.2 (localisation_key + t(key) surface) | SUPERSEDED (surface in row, one lookup) |
| CR-NR-109 loc Req 11.3/11.4/11.6/11.11-11.13 | RETAINED (over in-row surface) |
| CR-NR-111 loc Req 12.2/12.4 (menu stores root + reverse-derive) | AMENDED (option references alias; one path) |
| CR-NR-111 loc Req 12.3 (mnemonic source) | UNCHANGED (mnemonics stay English) |
| CR-NR-112 semantics Req 14.1 (4-field record) | AMENDED (drop localisation_key; 3 fields) |
| CR-NR-112 template expansion 14.2-14.4 | RETAINED byte-for-byte |
| CR-NR-113 configurator Req 5.11 (user-store-only) | SUPERSEDED (edits per-locale aliases) |
| CR-NR-113 configurator Req 5.6/5.7 (3-overlay store) | SUPERSEDED (one seed + user override) |
| CR-CH-071 Req 20a (ISPF save semantics) | PRESERVED (over the override store) |

## OPEN DECISIONS (surfaced to the owner at the pause -- recommend, do not finalise)

1. **User override store location/shape.** RECOMMEND
   `<User_Data_Dir>/FileForgeWorkbench/aliases/<locale>/<ENV>.toml`, rows = surface
   + root + optional template, overlaying the shipped `i18n/<locale>/aliases` seed.
2. **How a menu option references its alias.** RECOMMEND store the ROOT + mark the
   option "resolve-as-alias" (render resolves the active-locale surface by the
   option's root; dispatch resolves the root), rather than storing an alias
   id/surface-key. Keeps the persisted identity = root (no saved-menu migration)
   and satisfies reservation #3 with the simplest change.
3. **Shipped catalogues: read-only seed + overlay vs copy-to-user-on-first-run.**
   RECOMMEND keep the shipped catalogues a READ-ONLY seed with the user override
   layered on top -- preserves out-of-box `af` and clean upgrades.
4. **Collapse vs retain CR-NR-112 `command_template`.** RECOMMEND RETAIN the
   template field on the unified AliasEntry (it is a legitimate shorthand feature).

## Sequencing note (owner action)

- Implementation follows approval + the four OPEN DECISIONS as a SEPARATE run.
- The CURRENT UNCOMMITTED `af` i18n data (the `verb-*`/`localisation_key` rows in
  `i18n/af/aliases/FFCMD.toml`, `af/menus.ftl`, `af`/`en` `ff-desktop-verbs.ftl`)
  should be COMMITTED or explicitly carried before the rework so it is not lost.
  The design BUILDS ON it (the `af` surfaces are reused verbatim; only the
  `localisation_key` lines and the `verb-*` command-surface keys are dropped) --
  it does NOT replace the surfaces.
- Several stale worktrees (`eqfiles-fix`, `alias-ghost-text`, `cmdenv-root-enum`,
  `crch071-save-cancel-refresh`) still need owner cleanup; `main` is ahead of
  `origin` (unpushed).

## Files created / amended (this gate)

- `docs/status/change-log.md` -- NEW CR-CH-072 entry.
- `docs/specs/localization/requirements.md` -- NEW Req 13; Req 11.1 supersession note.
- `docs/specs/command-environments/requirements.md` -- NEW Req 2b; Req 6a.1 note.
- `docs/specs/command-configurator/requirements.md` -- NEW Req 6; Req 5.24/5.11 note.
- `docs/specs/command-semantics/requirements.md` -- Req 14.1 amendment note.
- `docs/specs/command-framework/requirements.md` -- Req 18.1 amendment note.
- `docs/specs/localization/design.md` -- NEW CR-CH-072 design section (alias shape,
  store model, menu-option reference, migration, supersession table).
- `docs/specs/command-environments/design.md` -- NEW CR-CH-072 resolution-path section.
- `docs/specs/localization/tasks.md` -- NEW Task group 23-30 (TDD-first).
- `docs/specs/command-environments/tasks.md` -- NEW Task group 32 (resolution path).
- `docs/quality/TCR.md` -- NEW Phase (alias-model-simplification) NOT-COVERED rows.
- `docs/project-management/ROADMAP.md` -- NEW CR-CH-072 roadmap entry.
- `.agents/tasks/alias-model-simplification/gate-summary.md` -- this file.
- `TERMINAL_DEFECTS.md` -- one terminal-defect entry (unrelated to the model).
