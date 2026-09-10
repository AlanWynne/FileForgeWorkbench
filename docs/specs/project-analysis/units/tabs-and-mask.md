# Analysis Record: tabs-and-mask (W2.7)

- **Wave**: 2 (Catalog and dataset -- the last Wave-2 analysis unit)
- **Backing crate**: `ff-tabmask` (spec says `ff-tabs-and-mask`; actual dir is
  `ff-tabmask` -- naming drift). NB: the separate `ff-tabs` crate is EDITOR TAB
  management (tab_bar/split_view/pinned/mru) belonging to multi-tab-editor
  (Wave 4), NOT this spec -- do not conflate.
- **Spec files**: requirements.md (456 lines, 18 requirements), tasks.md
  (152 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 2 pass

---

## 1. Split candidacy

NOT a split candidate.

- Requirement volume: 18 reqs, 456 lines. Above thresholds. (signal 1)
- Responsibilities: TWO closely-related features (TABS = tab-stop management +
  TABS_Line artifact; MASK = insert-template + MASK_Line artifact) sharing the
  Display_Artifact_Line + Session_State model. The spec bundles them precisely
  because they follow one pattern. Well-decomposed into 15 files
  (artifacts/mask/tab_stops/shift/state + commands/{tabs,mask,reset_tabs,line_commands}).
- Crates: single crate; minimal deps (ff-logging only -- integrations trait-injected).
- Cohesion: high (both are session-state display helpers).

1 signal (volume). The TABS/MASK pairing is a deliberate cohesive bundle, not two
separable concerns. No split.

### Source file sizes -- ALL WITHIN CAP

No file exceeds 300 non-test lines. No PA-STD size item. Well-sized.

---

## 2. Cross-unit consistency

### Display_Artifact_Line pattern -- REPLICATED, not shared (PA-WATCH-016)

The glossary defines `Display_Artifact_Line` as ONE unified concept spanning
COLS_Line, BNDS_Line (navigation-commands, W1.7), TABS_Line, MASK_Line
(ff-tabmask), and the cross-ref names navigation-commands as the "Pattern" source
("COLS/BNDS established the synthetic-line model; TABS/MASK extend it").

In code, however, there is NO shared `DisplayArtifactLine` trait/type (grep = 0
across all crates). ff-tabmask defines its OWN `ArtifactKind` enum + artifact
lifecycle (artifacts.rs); navigation-commands has its own COLS/BOUNDS handling
(BoundsManager, W1.7); exclude-show-filter has its own Placeholder_Line (W1.11).
So THREE+ subsystems each reimplement the synthetic-display-line concept the
glossary treats as unified. This is a pattern REPLICATION (consistency
observation), not a hard duplicate-type conflict -- each artifact is genuinely
different content. But it is a consolidation opportunity: a shared
`DisplayArtifactLine` abstraction (viewport-owned) would unify COLS/BNDS/TABS/MASK/
placeholder rendering. Recorded PA-WATCH-016 (LOW): consider a shared synthetic-
display-line abstraction (likely owned by viewport-and-scrolling or display-line-
mapping) at Wave 4; not blocking. Aligns with the Wave-2 fragmentation theme
(field model PA-CONFLICT-008; artifact-line pattern here).

### Minimal-dep clean seam -- consistent design

ff-tabmask's ONLY Cargo dep is ff-logging, yet the spec lists ff-command, ff-config,
ff-language-service, ff-edit-operations as dependencies. Resolution: traits.rs
defines `ConfigProvider` + `DocumentContext` injection traits, so those integrations
are CALLER-INJECTED (clean seam, same as sequence-numbers/auto-indent/ff-select).
Architecturally fine. But -- as with ff-select (W2.6) -- the spec's "depends on"
prose overstates the actual crate coupling; the integrations are trait seams. Note
consistency: this is the THIRD Wave-2 unit (after ff-select, and matching Wave-1
model crates) using minimal-deps + injection traits. Recorded as a CLEAN row (not
a defect), with a doc note that spec dependency lists should distinguish
"trait-injected integration" from "Cargo dependency".

### TABS vs auto-indentation Tab-key ownership -- pre-declared boundary

Cross-ref: auto-indentation owns Indent/Unindent (Tab/Shift+Tab on SELECTED lines,
W1.13); ff-tabmask owns single-cursor Tab-key advance-to-tab-stop. They coordinate
via the active Tab_Stop list in Session_State. Clean split of the Tab key by
context (selection vs single cursor). Consistent with W1.13. Confirm the shared
Tab_Stop session-state wiring at Wave 4 shell.

### RESET integration -- consistent

RESET (owned by command-semantics, Wave 3) clears TABS/MASK display artifacts
(Session_State). Same non-undoable session-state-cleared-by-RESET model as
exclude-show-filter (W1.11) and COLS/BNDS. Confirm at command-semantics (W3.1).

### Public types and ownership (non-conflicting)

- TABS_Line/MASK_Line artifacts, Tab_Stop list, Insert_Mask, `ArtifactKind`,
  shift logic, TABS/MASK/RESET-TABS commands, line commands -- sole-owned by
  ff-tabmask. No duplication of these.
- Config keys `editor.default_tab_stops` + per-language `default_tab_stops`/
  `default_mask` (language TOML) -- read via the ConfigProvider trait. Consistent.
- 0 fs calls -- session-state only, no I/O. Clean.

### Cross-reference integrity

All 8 declared cross-refs resolve as sub-projects; the dependency-vs-trait-seam
nuance is noted above. No dangling refs.

---

## 3. Completeness

Tracking: all 152 sub-tasks `[x]`. Implementation present across all 18 reqs
(TABS display/set/clear, tab-stop model, Tab-key advance, MASK display/edit/off,
mask application on I/In, per-language defaults, line commands, RESET clearing).
Tests in-file. No false-positive pattern. No PA-INCOMPLETE.

### TCR gap (PA-TCR-013)

TCR.md has 1 row for the crate against 18 reqs / ~110 criteria. Thin. Recorded
PA-TCR-013.

---

## 4. Logging audit

Scan of `crates/ff-tabmask/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT (sole dep) -- 0 uses. DEAD (same conspicuous
  pattern as ff-select W2.6).
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0

Pure session-state/display model -- zero-log is largely DEFENSIBLE (errors surface
via the error type). The error.rs variants (invalid tab stop, no active mask,
invalid config value) are returned as data. The one mild gap: config-value
validation (invalid `editor.default_tab_stops`) is a natural WARN site. Recorded
PA-LOG-017 (LOW): drop the dead ff-logging dep OR wire a config-coercion WARN +
dev-logging on TABS/MASK command dispatch (CR-NR-058). Lower priority than the
disk/DB catalog crates (PA-LOG-012/013).

---

## 5. Task revision proposals

- **PA-WATCH-016 (LOW)**: no shared `DisplayArtifactLine` abstraction exists; COLS/
  BNDS, TABS/MASK, and placeholder lines each reimplement the synthetic-display-line
  concept the glossary unifies. Consider a shared abstraction (viewport/display-line-
  mapping owned) at Wave 4. Consolidation, not blocking.
- **PA-STD-031 (ASCII, ACTIONABLE -- runtime strings)**: 33 non-ASCII bytes
  (3 non-comment). em-dashes inside runtime `#[error]` strings: error.rs (15/43/49).
  Non-ASCII in user-facing output is a genuine defect. Replace with `--`. REFACTOR,
  no gate.
- **PA-LOG-017 (LOW)**: drop the dead sole-dep ff-logging OR wire config-coercion
  WARN + dev-logging TABS/MASK dispatch.
- **PA-TCR-013**: enumerate per-requirement TCR rows (1 row for 18 reqs). No code.
- **PA-DOC (naming + dep-list)**: spec crate `ff-tabs-and-mask`; actual `ff-tabmask`.
  Also the spec lists trait-injected integrations as "dependencies" -- clarify.
  Fold into the naming reconciliation set.

No requirement CHANGE proposed; the spec is internally consistent and complete.

---

## Summary

tabs-and-mask (`ff-tabmask`, spec says `ff-tabs-and-mask`) is a cohesive,
well-sized session-state display-helper pair (TABS tab-stops + MASK insert-template,
both Display_Artifact_Line + non-undoable + RESET-cleared). Not a split candidate;
no size violation; 0 fs; clean minimal-dep + injection-trait design (ConfigProvider/
DocumentContext -- third Wave-2 unit using this seam). The notable consistency
observation is PA-WATCH-016: the Display_Artifact_Line concept is unified in the
glossary but REPLICATED in code across navigation-commands (COLS/BNDS),
exclude-show-filter (Placeholder), and ff-tabmask (TABS/MASK) with no shared
abstraction -- a Wave-4 consolidation opportunity. Minor items: dead sole-dep
ff-logging (PA-LOG-017, LOW), runtime-string ASCII (PA-STD-031), thin TCR
(PA-TCR-013), and the ff-tabmask/ff-tabs-and-mask naming + spec dependency-list drift.
NB: not to be confused with the separate `ff-tabs` crate (editor tab bar,
multi-tab-editor, Wave 4).
