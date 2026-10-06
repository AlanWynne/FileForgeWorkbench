# CR-CH-056 Phase 5 (Task 33) -- Implementation Note

The FINAL phase of the egui-native theme rework: egui-native theme-authoring
documentation + a test-migration audit + the TCR coverage sweep. Phase 5 is
almost entirely docs plus a coverage record; it changes NO observable behaviour
and no shipping `.rs` was modified. Phases 1-4 were already complete and
owner-confirmed (full gate CLEAN, 9613 tests at Phase 4) and were NOT re-done.

First-iteration run (no `phase5-review.json` present). A prior Phase 5 attempt had
already produced the on-disk deliverables (the authoring guide, the audit
conclusion, and the TCR edits) but was interrupted before finishing the
bookkeeping; this run independently re-verified every deliverable on disk, ran
its own scoped checks, and completed the task/TCR/change-log markers.

## 33.1 -- egui-native theme-authoring documentation

Deliverable: `docs/specs/theme-and-appearance/theme-authoring.md` (Theme Authoring
Guide, egui-Native Theme Model). Satisfies Requirement 23.4 (terminology honesty).

The guide uses egui `Style` / `Visuals` vocabulary as the PRIMARY model and frames
ISPF / Legacy as ONE retrofitted instance, not the model's defining shape:

- Section 1 "What a Theme is": a Theme = a CHROME layer (configures the full
  `egui::Style` / `Visuals` / `WidgetVisuals` surface) PLUS the retained DOMAIN
  groups egui does not model (`syntax`, `gutter`, `file_tree`, `decorations`,
  `indicators`, `style_slots`, `elements`); carries `name` + `VisualMode`;
  "Theme" stays the user-facing word (egui `Style`/`Visuals` is never exposed as
  the user concept). Includes the "Theme vs egui Style / Visuals" glossary table
  (Theme / egui `Style` / `Visuals` / `WidgetVisuals` / chrome layer / domain
  groups / the `gutter` group renamed from `chrome`).
- Section 2: the single `WorkbenchShell::apply_theme` seam
  (`shell/render_theme.rs`) doing a wholesale `apply_to_egui(&mut egui::Style)`,
  with Design_Tokens (spacing / radii / shadows) wired on at that seam and
  `visuals.dark_mode` matching the `VisualMode`.
- Section 3: the versioned-TOML v2 format -- `version = 2`,
  `egui_version = "0.33"`, `name`, `mode`; the FLAT AUTHORING GROUPS are
  AUTHORITATIVE and the chrome `egui::Style` is DERIVED from them on load; the
  embedded `[chrome_style]` sub-table is an additive round-trip snapshot read
  tolerantly, never authoritative; the `[chrome]` TOML section header is retained
  for file-format compatibility but loads into the renamed in-memory `gutter`
  group; v1 (no `version`) backward-compatibility and version-tolerant Style
  deserialise.
- Section 4: `base` inheritance actually resolved (built-in or user base, mode
  default last; unresolvable base WARNs + falls back; cycle detection).
- Section 5: the five built-in instances (Default Dark/Light = Solarized; High
  Contrast; Default Legacy toned `#000060`; Legacy Soft `#33FF66`).
- Sections 6-8: `THEME` selection, the Theme Editor (derived editable surface,
  Copy / Save / Save As with the B081 fix, Import/Export in FFWB's own format with
  command parity `THEME EXPORT` / `THEME IMPORT`), and an authoring checklist.

ASCII-clean: a non-ASCII grep over the file returns no matches.

No OTHER theme-authoring doc with stale pre-egui vocabulary exists -- a docs-wide
search for an authoring/how-to page found only this guide. (The
`menus/*.toml`-era `palette.ui.primary_menu_bg` reference in
`docs/specs/menu-and-statusbar/design.md` is an architecture-doc mapping note, not
an authoring guide, so it is out of 33.1 scope.)

## 33.2 -- test-migration audit (RESULT: nothing left to migrate)

The chrome-field test migration was COMPLETED across Phases 1-4. This audit
confirms NO leftover test asserts a REMOVED / renamed OLD chrome field. Evidence
(grep over `crates/**/*.rs`):

1. **Production chrome read sites already migrated (Phase 1).** `render_tab_bar.rs`,
   `render_split_region.rs`, and `render_command_line.rs` read
   `self.palette.chrome_style.*` accessors (`tab_active_bg_color()`,
   `title_band_bg_color()`, `focus_ring_color()`, ...), NOT the flat
   `palette.tab_bar.*` / `palette.ui.*` fields. Each carries a CR-CH-056 Req
   23.3/23.9/23.10 comment recording the migration.

2. **`chrome` group fully renamed to `gutter`.** A grep for `palette.chrome.` /
   `ChromeColours` finds only the `/// RENAMED from ChromeColours` doc comment in
   `palette.rs`. The in-memory group is `ThemePalette.gutter: GutterColours`; the
   only retained literal `chrome` strings are the `ColourToken` display names in
   `token.rs` and the frozen `[chrome]` TOML section header (file-format
   compatibility), both intentional.

3. **Hardcoded `== 14` editor-token-count already migrated.** `ff-theme-editor`
   `lib.rs` (`all_editable_tokens_have_labels`) and `editable_surface.rs`
   (`derived_surface_is_larger_than_the_former_fixed_fourteen`) assert
   `EditableToken::all().len() > 14` against the DERIVED surface. No `== 14`
   remains.

4. **Remaining `palette.ui.*` / `palette.tab_bar.*` / `ui.focus_ring` test
   references assert RETAINED authoring groups, not removed fields.** Per
   design.md section C56.2 the flat authoring groups stay AUTHORITATIVE (the chrome
   `Style` is derived FROM them -- `chrome_style.rs` sets `focus_ring: ui.focus_ring`,
   `tab_active_bg: tab_bar.active_bg`, ...). These tests exercise the authoring
   surface / loader parse / Theme Editor live-preview round-trip, which is the
   correct thing to assert. The `focus_ring` tests in `ff-desktop tests_focus.rs`
   validate ACCESSIBILITY Req 3.1-3.3 (focus-ring colour exists, differs from
   panel bg, is opaque) via the stable `ColourToken::UiFocusRing` / `ui.focus_ring`
   API -- they are not CR-CH-056 chrome-field tests.

5. **`.mode` / `.name` THEME-command tests left UNCHANGED** (command tests, not
   chrome-field tests), per the task instruction.

Conclusion: nothing migrated, nothing deferred -- the audit is a confirmation,
not a change. (This is the same conclusion the interrupted prior attempt reached;
it was re-derived independently here from the grep evidence.)

## 33.3 -- TCR sweep

`docs/quality/TCR.md` CR-CH-056 section carries, as PASS:
- Req 23.1-23.11 (23.4 cites the ff-theme model tests + the new authoring guide,
  per the testing.md MANUAL-exception rationale for terminology/doc coverage);
- Req 24.1, 24.2, 24.3, 24.5;
- Req 25.1-25.7;
- the reworded rows Req 13.2, 18.3, 20.5, 20.11, 20.12, 20.13.

Req 24.4 is intentionally left RED with a note: external-format import
(base16 / VS Code / tmTheme) is OUT OF SCOPE for CR-CH-056 (recorded as future).
No coverage was invented for it.

## Scoped checks (this run)

All via the clean non-interactive pwsh7 wrapper, redirected to `tools/logs/` and
read back (the terminal reports `Exit Code: -1` due to known PSReadLine mangling;
the logs are the source of truth).

- `cargo test -p ff-theme -p ff-theme-editor` -> `tools/logs/p5-smalltests.txt`:
  ff-theme **149 unit tests pass, 0 failed** (incl. all CR-CH-056 chrome_style /
  defaults_solarized / defaults_legacy_soft / format_version / loader / serialiser
  tests); ff-theme-editor + integration binaries green.
- `cargo clippy -p ff-theme -p ff-theme-editor -- -D warnings` ->
  `tools/logs/p5-clippy.txt`: **Finished, no warnings**.
- `cargo fmt -p ff-theme -p ff-theme-editor --check` ->
  `tools/logs/p5-fmtcheck2.txt`: **empty (no diffs) = clean**.
- `cargo check -p ff-desktop` (compile-only, once) ->
  `tools/logs/p5-desktop-check.txt`: **Finished** -- the workspace still builds.

Per the Phase-3 lesson, the heavy `ff-desktop` / full-suite TEST run was NOT run
in-agent (only the compile-only check). No shipping `.rs` was changed this phase,
so the scoped TEST surface is unchanged from Phase 4.

## Files touched (Phase 5)

- `docs/specs/theme-and-appearance/theme-authoring.md` (NEW, egui-native guide).
- `docs/specs/theme-and-appearance/tasks.md` (Task 33 + 33.1/33.2/33.3 -> `[x]`
  with DONE notes).
- `docs/project-management/project-master/tasks.md` (TER.6 -> `[x]`,
  code-complete-pending-gate note).
- `docs/quality/TCR.md` (CR-CH-056 section -- Req 23/24/25 + reworded rows PASS;
  prior-attempt edits re-verified intact).
- `docs/status/change-log.md` (CR-CH-056 Status -> Phase 5 code-complete pending
  the owner's full gate).
- `.agents/tasks/theme-egui-rework/phase5-note.md` + `phase5-verification.md`
  (this note + the verification record).

No shipping source (`crates/**`) was modified in Phase 5.

## HAND-OFF -- for the owner's full gate

Kiro ran ONLY the scoped checks above. The full `ffwb-gate.ps1` is the OWNER's
manual step:

```
pwsh -ExecutionPolicy Bypass -File tools\ffwb-gate.ps1
```

Because Phase 5 changed no shipping `.rs`, the full-suite result should match
Phase 4 (9613 tests). CR-CH-056 becomes DONE only after the owner confirms the
full gate CLEAN.
