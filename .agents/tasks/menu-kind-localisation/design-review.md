# Design Review -- Menu-Kind Localisation (CR-NR-111)

Reviewer: design-review subagent (fresh context, read-only).
Scope: the design.md files named in `design-files.md` and the EARS criteria they
satisfy. This is a documentation review; no code was written or run.

Documents reviewed:
- `docs/specs/localization/design.md` -- GOVERNING design section "Menu Kinds
  localise FULLY -- mnemonic + command alias + description (CR-NR-111,
  Requirement 12)" plus the two review-response logs (iterations 2 and 3).
- `docs/specs/menu-workspace/design.md` -- "Design Delta: Full menu-option
  localisation" (CR-NR-111).
- `docs/specs/menu-and-statusbar/design.md` -- "Design Delta: Menu row chrome
  localises fully" (CR-NR-111).
- Requirements checked for satisfaction: localization Req 11 + 12,
  menu-workspace Req 21 (incl. 21.8), menu-and-statusbar Req 20.
- `findings.md` (the investigation) for the framework-delta claim.

Source files independently read to verify the design's claims (see Verified
Assumptions):
- `crates/ff-desktop/src/shell/locale_resolver.rs`
- `crates/ff-desktop/src/shell/dispatch.rs`
- `crates/ff-desktop/src/shell/commands_menu.rs`
- `crates/ff-desktop/src/shell/commands_fastpath.rs`
- `crates/ff-desktop/src/shell/nav_reconstruct.rs`
- `crates/ff-desktop/src/shell/environment.rs`
- `crates/ff-desktop/src/menu_workspace/i18n_keys.rs`

---

## Verdict rationale

The design settles every point the step prompt names, grounds each load-bearing
claim in source that I independently verified, and satisfies every new EARS
criterion. No HIGH or MEDIUM finding was identified. The remaining items are NITs
that do not block implementation.

---

## Judgment against the required review points

1. **Storage-shape crux settled decisively, locale-INDEPENDENT value.** PASS.
   The governing design ("The crux ... DECISION") picks Option A (store the ROOT
   command) with four concrete justifications and records Option B (store a
   localisation key) as considered-and-rejected. The ROOT is explicitly the
   Locale_Independent_Stored_Value (Req 12.4), so first-prize auto-conversion
   works from a single saved definition. Decisive and justified.

2. **Whether `resolve_target` already resolves a per-locale alias, or a flagged
   FRAMEWORK CHANGE.** PASS, and it is a precise, source-grounded claim (not
   vague). The design states plainly: `ff_command::resolve_target` does NOT itself
   consult the alias table (its `TargetResolver` trait has no alias lookup), BUT
   the shell already translates a localised alias to its root BEFORE calling
   `resolve_target` via the single pre-pass `rewrite_verb_to_root` ->
   `AliasTable::canonical_verb`, run on both seams. It further states that moving
   the lookup INSIDE `resolve_target` WOULD be a framework change and is
   explicitly rejected. I verified `rewrite_verb_to_root` is called in both
   `dispatch.rs` (command line) and `commands_menu.rs::activate_menu_option`
   (menu selection), and is defined in `locale_resolver.rs`. The claim is correct
   and specific.

3. **Dispatch stays on the single existing `resolve_target` path.** PASS. The
   design adds no new or parallel dispatcher; the displayed surface is resolved
   back to the root by the existing pre-pass and dispatched through the unchanged
   front door. menu-and-statusbar Req 20.3 and menu-workspace Req 21.4 pin this,
   and the design's framework-conformance statements assert it.

4. **Covers mnemonic localisation, first prize, second prize, no-localised
   fallback, and migration/back-compat.** PASS on all five:
   - Mnemonic: a new `menu-<menu>-opt-<command>-key` catalogue key resolved by
     `localised_option_mnemonic` = `t_or(key, &option.key)`, mirroring the
     verified description seam; Option (i) derive-from-command-surface is rejected
     with reasons (digit mnemonics, coupling two columns). The REALISED CONTRACT
     is stated authoritatively in both the governing design and the
     menu-workspace delta.
   - First prize (Req 12.6): a single stored definition auto-converts; the
     end-to-end flow (re-seed on `on_locale_reload`, three derived surfaces next
     frame) is spelled out.
   - Second prize (Req 12.7): per-locale `menus/<L>/<name>.toml` resolved at
     load/reconstruct time, with the persistence contract (Req 12.7a) and the
     built-in CR-CH-021 clause (Req 12.7b) both explicit, and the POM load path
     explicitly scoped out with a verified rationale.
   - No-localised fallback (Req 12.8): per-part, per-option fallback to the
     Identity_Base, never a blank/unselectable row.
   - Migration/back-compat: none needed because Option A is the existing on-disk
     shape; stated explicitly.

5. **Respects the framework (code-only built-ins never written to disk; menus as
   content via the Kinds registry).** PASS. The design repeatedly forbids writing
   a built-in menu to disk (CR-CH-021), keeps the code-only `DEFAULT_*_TOML`
   fallback, adds no Workspace-Kinds-registry field (Req 12.10), and keeps menus
   as content through the one shared `render_menu_workspace`.

---

## Findings

1. **NIT -- Second-prize resolution-order wording differs slightly between the
   two documents for the user-menu case.** `localization/design.md` states the
   order as `menus/<L>/<name>.toml` -> base `menus/<name>.toml` -> code-only
   `DEFAULT_*_TOML`. `menu-workspace/design.md` states the same order. Both are
   consistent. However, Req 12.7 (localization requirements) describes the
   second-prize fallback only as "render the saved definition that matches the
   active locale when one exists, falling back to the Identity_Base definition
   otherwise" and does not mention the intermediate base `menus/<name>.toml` step
   for a user menu. The design's three-step order is a correct and reasonable
   realisation, but the requirement's two-step wording ("locale file else
   Identity_Base") could be read as skipping the base file. This is a NIT because
   the design's order is a strict superset that still ends at the Identity_Base;
   suggested fix: add one sentence to the design noting that for a USER menu the
   "Identity_Base definition" of Req 12.7 is the base `menus/<name>.toml` (and,
   when that too is absent for a built-in, the code-only `DEFAULT_*_TOML`), so the
   mapping from the two-step requirement to the three-step design is explicit.

2. **NIT -- "re-reading the THEN-active locale `L` each time" relies on an
   implicit assumption that `menus_dir()` and the active-locale accessor are both
   available at the two reconstruct seams.** The design asserts
   `reconstruct_settings_menu` and `reconstruct_named_menu` become
   `menus_dir().join(L).join("<name>.toml")`. I verified both functions exist and
   today build `menus_dir().join("<name>.toml")` (confirmed for
   `reconstruct_named_menu`). The design does not name HOW `L` (the active locale)
   is obtained at those two seams (e.g. the `ff-config` `ui.locale` read or a
   cached locale on the shell). This is a NIT, not a gap that blocks coding,
   because the locale is already read elsewhere (`on_locale_reload`), but naming
   the exact accessor the reconstruct seams will call would remove one
   implementation guess. Suggested fix: add a half-sentence naming the active-
   locale source the two seams read (the same one `menu_option_command_surfaces`
   / `on_locale_reload` use).

---

## Verified Assumptions

Each claim below was checked against the cited source and found accurate.

- **The alias->root pre-pass runs on BOTH seams before `resolve_target`.**
  Verified: `rewrite_verb_to_root` is called in `shell/dispatch.rs`
  (`dispatch_command_string`, command line) and in
  `shell/commands_menu.rs` (`activate_menu_option`, menu selection); defined in
  `shell/locale_resolver.rs`. The design's central framework-delta claim is
  correct.
- **`slug` lower-cases, collapses non-alphanumeric runs to a single `-`, trims.**
  Verified in `menu_workspace/i18n_keys.rs` (`fn slug`, with doc examples
  `"Reset bare" -> "reset-bare"`). The design's mandatory slug-convention claim
  (both `menu_option_key_key` and `menu_option_desc_key` slug the command
  identically, giving `menu-settings-opt-reset-bare-key`) is correct.
- **The description seam the mnemonic mirrors exists.** Verified:
  `menu_option_desc_key` builds `menu-<menu>-opt-<command>-desc`,
  `localised_option_description` = `t_or(menu_option_desc_key(...), &option.description)`,
  and `t_or` returns the fallback on a catalogue miss. The mnemonic seam is a
  faithful mirror.
- **The command-surface derivation the design reuses exists.** Verified:
  `AliasTable::display_surface_for_canonical` (`shell/environment.rs`) and
  `menu_option_command_surfaces` (`shell/locale_resolver.rs`) exist, and
  `localised_option_command` resolves the alias surface else the root.
- **The POM load path is distinct from the two reconstruct seams and builds
  `pom.toml` directly.** Verified: `ensure_pom_menu_loaded`
  (`shell/commands_fastpath.rs`) builds `self.menus_dir().join("pom.toml")`
  directly and falls back to `recovery_pom_menu()` (code-only), confirming the
  design's rationale for scoping the POM out of the second-prize file override.
- **`reconstruct_named_menu` / `reconstruct_settings_menu` are the user-menu
  reconstruct seams.** Verified both exist in `shell/nav_reconstruct.rs`;
  `reconstruct_named_menu` builds `menus_dir().join(format!("{name}.toml"))`.
- **menu-workspace Req 21.8 exists and pins the DISPLAY-vs-MATCH split.**
  Verified in `docs/specs/menu-workspace/requirements.md` (Req 21.8: displayed
  mnemonic is display-only; the stored `key` is the selection token).
- **menu-and-statusbar Req 20 exists and owns only the no-new-dispatch-path
  guarantee.** Verified (Req 20.1-20.3).
- **localization Req 12 criteria 12.1-12.10** are each addressed by a
  corresponding design section (full three-part derivation, locale-independent
  stored value, mnemonic source + fallback, routing invariance, first prize,
  second prize, per-part fallback, all-menu-kinds, no Kinds-registry change).

---

## Unverified / Wrong Assumptions

None. Every load-bearing claim the design makes about existing source
(`rewrite_verb_to_root` on both seams, `slug`, the description/command seams,
`display_surface_for_canonical`, the three distinct menu load paths, the
reconstruct-seam path construction) was independently verified and found correct.
No claim in the design was contradicted by the source read.

---

## Verdict

Findings: 0 HIGH, 0 MEDIUM, 2 NIT. Per the mechanical rule (HIGH+MEDIUM == 0),
the verdict is APPROVED. The two NITs are recorded for the implementer's benefit
and do not block implementation.
