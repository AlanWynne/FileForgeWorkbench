# Config-model gate -- summary (CR-CH-116)

Owner-confirmed FRAMEWORK CHANGE: switch the built-in menus (POM, Settings, SCRM)
and the command alias seed from CODE-ONLY-WITH-OVERRIDES to WRITE-THROUGH-CONFIG,
and formalise the ISPF END/RETURN/EXIT command model with X = EXIT everywhere.
SUPERSEDES CR-CH-021 / framework-conformance.md mechanism 4. Docs only; no `.rs`
changed. New CR id: **CR-CH-116** (CR-NR-116 is held by the parked branch
`alias-kind-gate`; different prefix, no collision).

Grounded in `.agents/tasks/config-model-change/decision.md` (owner decision) and
`.agents/tasks/config-model-change/investigation.md` (read-only investigation).

---

## 1. Criteria authored (all ids)

### configuration-system/requirements.md
- **NEW Requirement 23 -- Write-Through Config Model** (criteria 23.1-23.10 +
  glossary Recovery_Source / First_Run_Write / Config_Rebuild_Helper /
  Complete_Config_File; Out of scope).
  - 23.1 defaults are Recovery_Source only; on-disk file authoritative.
  - 23.2 first-run write of absent menu files, PER-FILE detection.
  - 23.3 first-run write of absent `aliases/<locale>/FFCMD.toml`.
  - 23.4 Complete_Config_File (every flag explicit; serialiser complete mode).
  - 23.5 all writes honour the dir seams (test isolation).
  - 23.6 first-run + Reset-bare SHARE ONE Config_Rebuild_Helper.
  - 23.7 parse failure -> recovery fallback + notice, NOT overwrite / not "absent".
  - 23.8 Degraded_Mode skips first-run write, falls back to Recovery_Source.
  - 23.9 `config_version` stamp (leaves room, no action taken now).
  - 23.10 deferred UPGRADE/merge NOT implemented (follow-up only).
- **AMENDED Requirement 19 (Reset bare)** -- NEW criteria:
  - 19.16 add `aliases/` to ARCHIVED_ITEMS.
  - 19.17 rebuild pom/settings/scrm + FFCMD seed from Recovery_Source via the helper.
  - 19.18 active-profile reopen reads the just-rebuilt files.

### menu-workspace/requirements.md (REVISED, in place)
- 2.1g POM terminate option command `X` -> `EXIT`; nav_x retired; no self-reference.
- 4.1 / 4.2 first-run WRITES pom/settings/scrm; files authoritative (was "never
  materialise, COMPILED-ONLY").
- 12.1 Recovery_Baseline now the Recovery_Source (written, not code-only).
- 12.2 POM `X` -> `EXIT` option (displayed `X EXIT`).
- 13.2 Menus-editor selector built-in set = `[POM, Settings, SCRM]` union on-disk.
- 13.3 editor load fallback maps `SCRM -> recovery_scrm_menu()`.
- 14.10 RETURN = unwind to the tab's PRIMARY PANEL (Tab_Visual_Root), reclassified.
- 14.13 bare `X` = EXIT on non-editor Contexts (FFEDIT EXCLUDE exception); nav_x retired.
- 14.14 `=X` exits via the alias.
- 14.15 the END/RETURN/EXIT model, no POM special-casing.
- 14.16 Detached_Workspace parity (END/RETURN close; X/EXIT terminate).
- Tab_Visual_Root glossary updated (X no longer collapses; RETURN does).

### command-configurator/requirements.md
- **NEW Requirement 8 -- FFCMD `X -> EXIT` seed** (criteria 8.1-8.7 + Out of scope).
  - 8.1 the single genuine seed alias `X -> EXIT`; no other verb seeded.
  - 8.2 first-run write of `aliases/<locale>/FFCMD.toml`, locale = active `ui.locale`.
  - 8.3 write through the alias base seam.
  - 8.4 Reset bare archives `aliases/` and rebuilds the seed.
  - 8.5 coexists with FFEDIT `X -> EXCLUDE` (active-wins).
  - 8.6 the seed row is visible/editable in the Alias Editor.
  - 8.7 resolves through the existing overlay path; `X` is not a command.

### startup-and-session/requirements.md
- **NEW Requirement 23 -- First-Run Write in the Startup_Sequence** (23.1-23.6):
  hook after Phase 4, per active profile, through seams, Degraded_Mode skip,
  shared helper, directory auto-create.

### navigation-commands/requirements.md
- **NEW Requirement 21 -- ISPF END / RETURN / EXIT model** (21.1-21.8 + Out of scope):
  END one level (PF3 stays END); RETURN to primary panel; EXIT unconditional;
  X = EXIT via FFCMD seed (FFEDIT EXCLUDE exception); nav_x retired; single front
  door.

### screen-snapshot-scrm/requirements.md
- 22.8 REVISED: Reset bare rebuilds `menus/scrm.toml` + the SCRM-bearing POM from
  the Recovery_Source; also written on first run (supersedes the CR-CH-021 "never
  materialise" clause).

---

## 2. The one-helper design

A single `Config_Rebuild_Helper` (recommended
`shell/config_bootstrap.rs::write_builtin_defaults(seams, mode)`) is the ONLY code
that serialises the Recovery_Source into complete config files. Two modes:
`FirstRunMissingOnly` (write-if-absent, per-file) for startup; `RebuildOverwrite`
for Reset bare (which has already archived the old files). It writes pom/settings/
scrm (complete) + `aliases/<locale>/FFCMD.toml`, always through the dir seams
(`dir_overrides.menus/.scrm/.user_aliases`, `FFWB_USER_ALIASES_DIR`). First-run and
Reset-bare call the SAME helper, so their output cannot diverge (configuration-system
Req 23.6 / 19.17). Hook points: first run in `construct.rs::new_with_history_store`
(the POM first-run write moves here so there is ONE hook, not `ensure_pom_menu_loaded`);
rebuild in `reset_bare.rs` after `archive_config` (+ in-memory reset for the active
profile). Complete-mode serialisation is a new `serialise_complete(menu)` on
`menu_workspace::serialiser` (chosen over writer post-fill so the "explicit flags"
contract lives next to the loader in one testable place).

---

## 3. The reroute plan (8 sites, investigation Q7)

The write-through switch is driven by the first-run write (making files reliably
present) + the Reset-bare rebuild. The eight production readers keep their existing
"prefer on-disk file, recover from constant" shape; only site 6 needs a real edit:

1. `commands_fastpath.rs::ensure_pom_menu_loaded` -- recovery fallback retained; no edit.
2. `commands_menu.rs::open_menu_by_name` (SCRM) -- recovery fallback retained; no edit.
3. `nav_reconstruct.rs` (Settings) -- recovery fallback retained; no edit.
4. `render_menu_bar.rs::active_pom_menu` / `menu_bar_dropdown_options` -- already
   prefer on-disk; no edit.
5. `render_chrome.rs::active_pom_menu` -- already prefers on-disk; no edit.
6. `menus_editor.rs::load_menu_for_editor` + `refresh_menus_editor_list` -- **EDIT**:
   built-in list gains SCRM; load fallback maps `SCRM -> recovery_scrm_menu()`.
7. `alias_reader.rs::builtin_ffcmd_roots` -- UNCHANGED (reads recovery fns to compute
   the root set deliberately; confirm `EXIT` is a root so `X -> EXIT` targets a real root).
8. `command_config/mod.rs::BUILTIN_MENU_NAMES` -- UNCHANGED (names already Menu targets).

Blast radius: the first-run hook, the Reset-bare hook, the serialiser complete-mode,
and the ONE Menus-editor edit. Contained in `ff-desktop`.

---

## 4. END / RETURN / EXIT reconciliation (+ X = EXIT, nav_x retirement)

Built on the existing per-tab Navigation_Stack and the single dispatch front door;
NO new dispatcher, NO new stack.

- **END (PF3)**: UNCHANGED -- pop ONE level; close-when-last at the Tab_Visual_Root.
  PF3 stays END, never remapped to EXIT (authentic ISPF).
- **RETURN**: UNCHANGED in behaviour -- already collapses to the Tab_Visual_Root in
  one step (CR-CH-052). This IS the ISPF "back out to the primary panel" semantics;
  reclassified as such, stale "converges with X" note removed. The primary panel is
  the per-tab Tab_Visual_Root (POM for a default tab), reconciled with mechanism 3.
- **EXIT**: UNCHANGED -- `try_exit_family` -> `file.exit`, unconditional
  (startup-and-session Req 20.3). No new command.
- **X = EXIT everywhere**: the `upper == "X"` arm in `commands_ladder_a.rs` (calling
  `nav_x`) is REMOVED; `X` resolves via the FFCMD seed `X -> EXIT` through the
  existing `rewrite_verb_to_root` -> `resolve_target` pre-pass. Active-wins keeps
  FFEDIT bare `X` = EXCLUDE on an editor. `=X` reinitialises to FFCMD then runs `X`,
  which now exits.
- **nav_x RETIRED**: `shell/nav_stack.rs::nav_x` and its single caller removed;
  `close_workspace_or_exit` is RETAINED (END/RETURN still use it at the root). No
  other caller found (re-confirm with a reference search at implementation).

---

## 5. Owner-decision / confirm points (with recommendation)

- **D1 -- Confirm X = EXIT everywhere.** Owner CONFIRMED (prompt x2). Recommendation:
  implement as specified (X is the FFCMD seed alias of EXIT; unconditional exit on
  non-editor Contexts; FFEDIT EXCLUDE preserved via active-wins). No open question.
- **D2 -- Complete serialiser vs writer post-fill.** RECOMMEND the complete-mode
  serialiser (`serialise_complete`) so the explicit-flags contract is one testable
  place beside the loader. (Owner confirm not strictly required; a design choice.)
- **D3 -- First-run hook location (construct.rs vs ensure_pom_menu_loaded).**
  RECOMMEND `construct.rs::new_with_history_store` for ONE consistent hook; the POM
  first-run write moves there. (Design choice; flag for awareness.)
- **D4 -- `config_version` stamp.** RECOMMEND adding the optional defaulted field now
  (additive, backward compatible) to leave room for the deferred UPGRADE. No action
  taken on it in this change. Owner confirm that a stamp is acceptable.
- **D5 -- Alias seed locale.** RECOMMEND the active `ui.locale` (default `en`) and
  file `aliases/<locale>/FFCMD.toml`, matching the ENV-in-filename model. Owner
  confirm the `en` default seed is correct.
- **D6 -- Seed as a user-override row (editable) vs a shipped read-only seed.**
  RECOMMEND the user-override row (visible/editable in the Alias Editor, rebuilt by
  Reset bare), matching "expose everything in config files". Owner confirm.
- **D7 -- Retire nav_x entirely vs keep it for a close-only path.** RECOMMEND full
  retirement (owner decision: X just exits; terminate-when-last nuance gone).
  Confirmed by the decision doc.

No decision point was left unresolved; all are either owner-confirmed or carry a
clear recommendation for approval.

---

## 6. Deferred UPGRADE note

A software update that adds a NEW default option/alias will NOT reach a user who
already has authoritative config files (files win). Owner decision: "worry about it
when one happens." NOT built here. The design leaves room via the `config_version`
stamp (Req 23.9) for a future UPGRADE command that rebuilds the Reset-bare defaults
and merges the existing user configuration into them (Req 23.10) -- recorded as a
follow-up only.

---

## 7. Framework-rule supersession

`.kiro/steering/framework-conformance.md`:
- **Mechanism 4** rewritten from "Code-only built-in menus (CR-CH-021), NEVER written
  to disk" to "Write-through built-in menus + alias seed (CR-CH-116)": defaults are
  the Recovery_Source, first run writes complete files, the app runs from the files,
  Reset bare rebuilds, one shared helper, seams honoured, graceful fallback, deferred
  UPGRADE. The themes code-only rule is explicitly NOT affected.
- **Mechanism 3** gains an ISPF END/RETURN/EXIT note (END one level / RETURN to the
  primary panel / EXIT and X terminate; PF3 stays END; nav_x retired; FFEDIT EXCLUDE
  the sole exception).

ONE additive data-shape change strictly needed: optional defaulted `config_version`
on the crate-private `RawMenuFile` / alias seed (NOT a public cross-crate type;
backward compatible). No other public type reshaped; no new dispatcher or navigation
stack; everything else on existing seams (single dispatch, per-tab Navigation_Stack,
three-layer alias resolver, ScreenProvider).

---

## 8. Files created or changed (docs only)

- `docs/specs/configuration-system/requirements.md` (NEW Req 23; Req 19.16-19.18)
- `docs/specs/configuration-system/design.md` (NEW write-through delta)
- `docs/specs/configuration-system/tasks.md` (NEW Task 47)
- `docs/specs/menu-workspace/requirements.md` (REVISED Req 2.1g, 4, 12, 13, 14 + glossary)
- `docs/specs/menu-workspace/design.md` (NEW CR-CH-116 delta)
- `docs/specs/menu-workspace/tasks.md` (NEW Task 56)
- `docs/specs/command-configurator/requirements.md` (NEW Req 8)
- `docs/specs/command-configurator/design.md` (NEW CR-CH-116 delta)
- `docs/specs/command-configurator/tasks.md` (NEW Task 8)
- `docs/specs/startup-and-session/requirements.md` (NEW Req 23)
- `docs/specs/startup-and-session/tasks.md` (NEW Task 37)
- `docs/specs/navigation-commands/requirements.md` (NEW Req 21)
- `docs/specs/navigation-commands/tasks.md` (NEW Task 21)
- `docs/specs/screen-snapshot-scrm/requirements.md` (REVISED Req 22.8)
- `.kiro/steering/framework-conformance.md` (mechanism 4 superseded; mechanism 3 note)
- `docs/status/change-log.md` (NEW CR-CH-116 entry)
- `.agents/tasks/config-model-gate/gate-summary.md` (this file)

---

## 9. Review-pass responses (review.json round 1 -> addressed)

Round-1 verdict was CHANGES_REQUESTED (2 HIGH, 1 MEDIUM, 2 NIT). Each scored
finding is now resolved in the worktree deliverables:

- **Finding 1 (HIGH) -- startup-and-session 11.9 contradicted new Req 23.**
  FIXED. Criterion 11.9 rewritten: the built-in menus ARE written on first launch
  (Req 23.1-23.2); the compiled Recovery_Source is consumed only for the first-run
  write, Reset-bare rebuild, and last-ditch runtime fallback. The "(CR-CH-021)" tag
  on the companion criterion 11.8 is qualified to the write-through model
  (Recovery_Source). No "SHALL NOT be materialised" clause remains.
- **Finding 2 (HIGH) -- screen-snapshot-scrm 25.5 contradicted revised 22.8.**
  FIXED. Criterion 25.5 rewritten to align with 22.8: `menus/scrm.toml` is written
  on first run and rebuilt by Reset bare from `DEFAULT_SCRM_MENU_TOML` via the
  shared Config_Rebuild_Helper; the app runs from the on-disk file; a user
  `menus/scrm.toml` stays authoritative; the compiled default is the Recovery_Source
  only. No "SHALL NOT be materialised" clause remains.
- **Finding 3 (MEDIUM) -- five wrong "command-configurator Requirement 6" cites.**
  FIXED. All five references (configuration-system Req 19 criterion 17 and Req 23.3;
  menu-workspace criterion 2.1g, the POM note, and criterion 14.13) now read
  "command-configurator Requirement 8" (the FFCMD `X -> EXIT` seed). Verified zero
  residual "command-configurator Requirement 6" occurrences; Requirement 6 is the
  unrelated Alias Editor localisation requirement.
- **Finding 4 (NIT) -- pre-existing non-ASCII in touched files.** Acknowledged,
  not actioned. These characters are on pre-existing lines not authored by
  CR-CH-116; the gate's own added/revised content is ASCII-clean. A standing-repo
  cleanup is out of scope for this gate (would expand blast radius into unrelated
  pre-existing prose).
- **Finding 5 (NIT) -- main-root vs worktree divergence.** Clarified. The 16
  tracked doc deliverables live on the `config-model-gate` worktree branch (what
  merges); the `.agents/tasks/config-model-gate/` scratch (this summary, review.json,
  review.md, decision/investigation) is repo-root session scratch shared across
  worktrees and is not branch content. The two are intentionally in different trees;
  implementers/reviewers use the worktree for the docs and `.agents` for the gate
  record. Nothing to merge from `.agents`.

GATE_READY_FOR_APPROVAL
