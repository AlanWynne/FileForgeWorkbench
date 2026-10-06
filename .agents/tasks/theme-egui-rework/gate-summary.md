# CR-CH-056 Gate Summary -- egui-Native Theme Model Rework

This is the specification-step output of the requirements gate for **CR-CH-056**
(CORE, owner-confirmed framework change). It is DOCUMENTATION ONLY -- no source
file was changed. Present this to the owner for approval before any code.

Confirmed scope (from CR-CH-056 in `docs/status/change-log.md`): an EGUI-NATIVE
HYBRID theme model -- a CHROME layer configuring the full `egui::Style`/`Visuals`
(via egui's OWN serde) plus RETAINED domain groups egui does not model; a single
`apply_to_egui` seam; Solarized Dark/Light defaults (replacing Catppuccin);
Default Legacy retrofitted with its primary option-menu band toned `#0000AA` ->
`#000060`; a new Legacy Soft built-in (body green `#33FF66`); FFWB-native-only
import/export; versioned TOML with embedded `Style` + v1->v2 load + resolved
`base`; and a reworked Theme Editor that edits the egui surface + domain groups,
drives import/export, and folds in bug B081. Supersedes CR-CH-055 and CR-NR-104.

---

## 1. Requirements added, reworded, or superseded (`docs/specs/theme-and-appearance/requirements.md`)

### New requirements
- **Req 23 -- egui-Native Theme Model.** Chrome layer configures the full
  `egui::Style`/`Visuals`/`WidgetVisuals` surface (23.1), serialised via egui's
  own serde (23.2); domain groups retained (syntax, gutter [renamed from chrome],
  file_tree, decorations, indicators, style_slots, elements) (23.3); Theme is the
  user concept producing a `Style` + domain groups (23.4); single wholesale
  `apply_to_egui` seam removing the hardcoded Legacy slider smell (23.5);
  DesignTokens wired to egui spacing/rounding/shadow (23.6); the old Req 21
  three-level hierarchy / accent focus ring / accent active tab / accent
  primary-menu band contract restated in egui-Visuals terms so Solarized
  Dark/Light satisfy it (23.7-23.10); WCAG AA advisory preserved, High Contrast
  AAA unchanged (23.11).
- **Req 24 -- Theme Import/Export.** Export the active/selected theme to a native
  FFWB file (24.1); import a native FFWB file as a selectable user theme (24.2);
  invalid/foreign files rejected with a clear message, no corruption (24.3);
  external formats explicitly OUT of scope / future (24.4); command parity for
  import/export (24.5).
- **Req 25 -- Versioned Theme File Format and base Resolution.** Top-level
  `version` field + recorded egui version (25.1); backward-compatible v1 load via
  default-fill (25.2); version-tolerant embedded `Style` deserialise -- missing
  default, extra ignored (25.3); `base` inheritance RESOLVED, not discarded (25.4);
  unresolvable base WARNs + falls back (25.5); versioned TOML with embedded
  `Style` sub-table, TOML-over-JSON decision flaggable (25.6); new-model
  round-trip (25.7).
- **Glossary addendum** distinguishing Theme (user concept) from egui `Style` /
  `Visuals` / `WidgetVisuals`, the chrome layer, the domain groups, and the
  renamed `gutter` group.

### Superseded
- **Req 2 (Theme Palette Structure)** -- CHROME portions (`tab_bar`, `ui`,
  chrome-adjacent `editor` fields) superseded by Req 23's egui-native chrome
  layer; DOMAIN portions retained. Marked SUPERSEDED IN PART with a pointer to
  Req 23 (criteria kept for history).
- **Req 21 (Non-Monochrome Chrome)** -- fully superseded by Req 23; its contract
  re-expressed in egui `Visuals` terms (23.7-23.11). Marked SUPERSEDED with a
  pointer (criteria kept for history).

### Reworded (notes added; criteria not deleted)
- **Req 1** -- theme file is now versioned TOML with an embedded `Style`
  sub-table; partial-definition/default-fill preserved (points to Req 25).
- **Req 5** -- VisualModes retained; a Theme produces a `Style`; Dark/Light become
  Solarized; AA advisory re-expressed against egui Visuals (23.11).
- **Req 6** -- DesignTokens now WIRED onto the egui `Style` at the seam (23.6).
- **Req 8** -- no-hardcoded-colours preserved; chrome sourcing (8.4/8.5) now via
  the applied `Style`/`Visuals`; Legacy slider injection removed; domain sourcing
  unchanged.
- **Req 13 (Legacy semantics)** -- RETAINED but reframed as the Legacy INSTANCE of
  the general model; **13.2 amended**: primary option-menu background toned
  `#0000AA` -> `#000060`; all else byte-identical.
- **Req 14** -- "every token overridable" now covers the embedded `Style`; `base`
  must be resolved (points to Req 25); built-in count 4 -> 5.
- **Req 17** -- shorthand list extended with `legacy soft` / `legacy-soft` /
  `legacy_soft` -> `Legacy Soft` (17.2b).
- **Req 18 (built-ins)** -- set grows to FIVE; **18.3 amended** to enumerate
  Default Dark[Solarized], Default Light[Solarized], Default High Contrast,
  Default Legacy[toned], Legacy Soft; code-only/read-only/fallback unchanged.
- **Req 20 (Theme Editor)** -- reworked to edit the egui surface + domain groups;
  **20.3/20.5 amended**; **new 20.11** (derived editable surface), **20.12**
  (import/export), **20.13** (B081 Save fix + full-shell type-a-name-and-Save
  test).
- **Req 22** -- unchanged in substance; a note records that CR-CH-056 tones the
  Legacy background per 13.2 (white-on-`#000060` remains a legible background
  pairing).

---

## 2. Key design decisions (`docs/specs/theme-and-appearance/design.md`, section "CR-CH-056")

- **Hybrid model**: a chrome layer (new type, e.g. `ChromeStyle`, backed by
  `egui::Style` via egui's own serde) + retained domain groups; the `chrome`
  domain group is RENAMED to `gutter` to avoid clashing with the new chrome layer.
- **Adopt egui's serde**: enable egui's `serde` feature (egui 0.33) so the theme
  file embeds egui's native `Style` representation rather than a hand-written
  mirror -- this is the "does egui already have a method?" answer.
- **Single seam**: `WorkbenchShell::apply_theme` becomes a wholesale
  `apply_to_egui(&mut Style)`; the hardcoded Legacy slider colours leave the seam
  and become part of the Legacy instance's `Style`; DesignTokens wired here;
  dark_mode set to match VisualMode.
- **~33 chrome read sites** in `shell/render_*.rs` migrate to read the applied
  `Style`/`Visuals`; domain read sites unchanged.
- **Format**: versioned TOML with the egui `Style` embedded as a sub-table;
  TOML-over-JSON recorded as the chosen decision, FLAGGABLE at review; `version`
  branching for v1->v2 load; `base` resolved in the loader.
- **Built-in instance values** recorded: Solarized hex (base03..base3 + accents),
  Legacy primary-menu `#000060`, Legacy Soft body `#33FF66`.
- **Theme Editor rebuild**: derived editable surface, import/export actions
  (`ThemeEditorAction::Export/Import`), B081 Save-on-built-in prompt/create fix.
- **5-phase plan**: (1) model + egui mapping; (2) built-in instances; (3) editor
  rebuild incl. B081; (4) version + migration + base; (5) docs + test migration.
- **Framework conformance**: single-command-dispatch, nav stack, WorkspaceContext,
  code-only built-ins all UNCHANGED; the only framework changes are the
  owner-confirmed `ThemePalette` shape and the Theme_File format.

---

## 3. Task numbers added (`docs/specs/theme-and-appearance/tasks.md`, Phase (theme-egui-rework))

- **Task 29** (Phase 1) -- enable egui serde; chrome layer type; `apply_to_egui`
  seam; rename chrome -> gutter; migrate ~33 read sites. (29.1-29.7)
- **Task 30** (Phase 2) -- Solarized Dark/Light; Legacy retrofit `#000060`; Legacy
  Soft `#33FF66`; built-in set 4 -> 5; `legacy soft` shorthand. (30.1-30.6)
- **Task 31** (Phase 3) -- derived editable surface; Import/Export; reject foreign;
  B081 Save fix; full-shell type-a-name-and-Save test. (31.1-31.5)
- **Task 32** (Phase 4) -- `version` field + embedded `Style`; v1->v2 load;
  version-tolerant deserialise; resolve `base`. (32.1-32.5)
- **Task 33** (Phase 5) -- egui-native docs rewrite; migrate ~25-30 chrome-field
  tests; TCR + scoped checks + hand-off. (33.1-33.3)

All `[ ]` (pending). `docs/project-management/project-master/tasks.md` gains a
**Phase (theme-egui-rework)** CORE section with deliverables TER.1-TER.6 (one per
gate + the 5 phases) and a status row, all `[ ]`.

---

## 4. TCR rows added (`docs/quality/TCR.md`, section "CR-CH-056")

One NOT COVERED (red-circle) row per new/amended criterion, in the correct crate
section:
- `ff-theme`: Req 23.1, 23.2, 23.3, 23.4, 23.6, 23.7, 23.8, 23.9, 23.10, 23.11;
  Req 24.4; Req 25.1-25.7; Req 13.2 (amended); Req 18.3 (amended to five).
- `ff-theme-editor`: Req 24.3; Req 20.11; Req 20.12.
- `ff-desktop`: Req 23.5; Req 24.1, 24.2, 24.5; Req 20.13 (B081); Req 20.5
  (amended).

---

## 5. Acceptance-plan rows added/changed (`docs/project-management/core-acceptance-test-plan.md`, Group 8)

- **8.5 CHANGED** -- FOUR -> FIVE built-ins; now lists Legacy Soft; marker `[ ]`
  -> `[B]`; backing CR-CH-056.
- **8.5a NEW** `[B]` -- select Solarized Dark/Light; confirm egui-Visuals chrome
  contract + AA advisory (Req 18.3, 23.7-23.11).
- **8.5b NEW** `[B]` -- select Default Legacy (toned `#000060`) and Legacy Soft
  (`#33FF66`) (Req 13.2 amended, 18.3).
- **8.6 REFRAMED** -- reworked Theme Editor create+Save; backing CR-CH-056
  (20.5 amended, 20.13) + B081; marker kept `[F]` (B081 still open).
- **8.6a NEW** `[B]` -- edit an egui `Style` chrome field in the reworked editor
  (Req 20.11, 23.1).
- **8.6b NEW** `[B]` -- export then import a FFWB-exported theme; foreign file
  rejected (Req 24.1-24.3).
- **Regression Traceability** -- new entry mapping CR-CH-056 (PENDING GATE) ->
  rows 8.5, 8.5a, 8.5b, 8.6, 8.6a, 8.6b.

---

## 6. Open decision for the owner at approval

- **Theme file format**: TOML (chosen, for consistency with `menus/*.toml` and
  config) vs JSON (serialises egui's nested `Style` more naturally). Recorded as
  the one flaggable decision in Req 25.6 and design C56.5.

No source code was written; no task is marked `[x]`. Awaiting owner approval of
this gate before implementation.
