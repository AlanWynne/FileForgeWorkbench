# Analysis Record: theme-and-appearance (W4.4)

- **Wave**: 4 (UI, panels, layout)
- **Backing crate**: `ff-theme` (central visual-identity layer; GUI-independent --
  no egui dep, produces abstract colour/attribute values the renderer applies)
- **Spec files**: requirements.md (324 lines, 15 requirements -- numbered 1-11,
  13, 14, 16, 15), tasks.md (172 sub-tasks, **171 done / 1 OPEN**), design.md
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

NOT a split candidate. 15 reqs, 324 lines. One cohesive concern (theme config +
palette + style slots + fonts + visual modes + design tokens + serialization +
plugin/user themes). 19 files. No split.

### Source file size violation (PA-STD-046)

`defaults.rs` = 661 non-test lines, over the 400 cap (built-in dark/light/
high-contrast default palettes + all colour groups: Chrome/Decoration/Editor/
FileTree/Indicator/Syntax/TabBar/Ui + StyleSlotTable defaults). Split by palette/
colour-group. REFACTOR, no gate. Recorded PA-STD-046. (Only file over cap.)

---

## 2. Cross-unit consistency -- CLEAN (sole owner of the visual identity)

theme-and-appearance is the SINGLE OWNER of the style-slot table + colour groups
that the editor-core units resolve against -- confirming the W1.10/W1.15 layering.

- `StyleSlot` / `StyleSlotTable` -- sole-owned by ff-theme (grep: 0 StyleSlotTable
  outside ff-theme). `api.rs::style_slot(index) -> StyleSlot` is the resolution
  entry. This is the target for:
  - syntax-highlighting Req 12 (W1.10): lexer emits `StyleSlotIndex` (u8); ff-theme
    resolves to colour/bold/italic. CONFIRMED clean single-owner.
  - text-decorations Req 15 (W1.15): indicator/marker colours resolved from the
    `DecorationColours` / `IndicatorColours` groups. CONFIRMED.
  - whitespace-and-guides (W1.12): whitespace/guide/edge colours from theme tokens.
- Colour GROUPS (`SyntaxColours`, `DecorationColours`, `IndicatorColours`,
  `EditorColours`, `ChromeColours`, `FileTreeColours`, `TabBarColours`, `UiColours`)
  -- sole-owned; each consumer reads its group. No duplicate colour model anywhere.
  This is the RIGHT counter-example to the Wave-2 domain-type fragmentation: one
  authoritative visual model, many read-only consumers.
- Req 8 (Replacing Hardcoded Colours): all rendering obtains colours via ff-theme,
  not hardcoded. Consistent with the "no colour values in the engine" principles of
  syntax-highlighting Req 12.1 (W1.10) + text-decorations Req 15.1 (W1.15).
- GUI-independent (no egui dep) -- shell/renderer applies the resolved attributes.
  Correct layering (like ff-layout/ff-menu/ff-completion). Deps: ff-config +
  ff-logging.

### High-contrast / accessibility (Req 5) -- ties to W4.9 accessibility

11 high-contrast/WCAG refs. Req 5 (Visual Modes: Dark/Light/High-Contrast) + the
high-contrast defaults are the theme side of accessibility. Recorded a note to
confirm at accessibility (W4.9): the high-contrast theme is the visual mechanism;
accessibility (W4.9) likely owns the WCAG contrast-ratio verification. Consistent
producer(theme)/verifier(a11y) split expected.

### Theme TOML store -- raw-fs (PA-WATCH-019 adjacent)

9 fs calls for theme TOML loading (Req 1/7). ff-theme owns theme files -- legitimate.
Local-store family (PA-WATCH-019 adjacent), but theme files are the theme owner's data.

### Public types and ownership

- ThemePalette, StyleSlot/StyleSlotTable, colour groups, Font_Stack, design tokens,
  visual modes -- sole-owned by ff-theme. No duplication.

### Cross-reference integrity

Cross-refs resolve. No dangling refs.

---

## 3. Completeness -- PA-INCOMPLETE-010 (Req 16 OS mode-follow, 1 task)

Tracking: 171/172 tasks done, **1 OPEN** = Task 20 "OS dark/light mode follow
(Phase CR)" = Requirement 16 (OS Dark/Light Mode Follow -- switch the theme when the
OS switches dark/light). A genuine small feature, unbuilt, honestly tracked
(NOT a false-positive). Recorded PA-INCOMPLETE-010 (LOW): implement Req 16 OS
theme-follow (query the OS appearance + switch visual mode). Small, self-contained;
lower priority than the Wave-3 CR-NR-057 bundle.

Reqs 1-11, 13-15 are implemented (config, palette, style slots, fonts, modes,
tokens, loading, hardcoded-colour replacement, serialization, element colours,
plugin themes, legacy semantics, user themes, extensibility).

### TCR (adequate)

21 TCR rows for 15 reqs -- adequate. No PA-TCR raised.

---

## 4. Logging audit

Scan of `crates/ff-theme/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs`: 9 (theme TOML loading -- legitimate)
- non-ASCII: 108 total, ALL comment-only (0 runtime -- clean)

MANDATED logging unimplemented: Req 7.4 ("theme file contains invalid TOML -> LOG A
WARNING identifying...") + Req 7.5 (individual invalid values -> fall back +
warn). Also text-decorations Req 15.8 (W1.15) DELEGATES its theme-value-validation
WARN to the theme system -- i.e. the WARN belongs HERE. Currently 0 log calls.
Recorded PA-LOG-030 (LOW-MED): wire ff-logging for the Req 7.4/7.5 theme-load +
invalid-value WARNs (this also satisfies the text-decorations PA-LOG-011 theme-side
delegation); dev-logging on theme switch / hot-reload. The invalid-theme WARN is
mildly important (a user's broken theme silently falling back is confusing).

---

## 5. Task revision proposals

- **PA-INCOMPLETE-010 (LOW)**: implement Req 16 OS dark/light mode follow (Task 20,
  Phase CR). Small self-contained feature; honest `[ ]`.
- **PA-STD-046 (REFACTOR)**: split `defaults.rs` (661) by palette/colour-group.
  No gate.
- **PA-LOG-030 (LOW-MED)**: wire ff-logging for Req 7.4/7.5 theme-load + invalid-
  value WARNs (also satisfies text-decorations Req 15.8 delegation, PA-LOG-011
  theme-side); dev-logging on theme switch. Resolve dead dep.
- **PA-TCR (none)**: 21 rows adequate. Task 20/Req 16 needs a row once built.
- **PA-WATCH-025 (LOW)**: confirm the high-contrast theme (Req 5) vs accessibility
  WCAG contrast-ratio verification boundary at accessibility (W4.9) --
  producer(theme)/verifier(a11y).

No PA-STD ASCII item (108 non-ASCII but ALL comment-only -- still a doc.md
violation, folds into the project-wide ASCII sweep; noted, not a runtime defect).
No requirement CHANGE; Req 16 is a correctly-gated pending feature.

---

## Summary

theme-and-appearance (`ff-theme`) is the GUI-independent central visual-identity
layer and a CLEAN single-owner exemplar: it sole-owns the `StyleSlotTable` + all
colour groups (Syntax/Decoration/Indicator/Editor/Chrome/FileTree/TabBar/Ui, 0
duplicates anywhere), confirming the W1.10 (syntax-highlighting Req 12) + W1.15
(text-decorations Req 15) + W1.12 (whitespace-guides) resolution layering -- the
right counter-example to Wave-2 domain-type fragmentation. Complete except one
honestly-tracked open task: PA-INCOMPLETE-010 (Req 16 OS dark/light mode follow,
LOW). Findings are mechanical: defaults.rs over the cap (PA-STD-046), a dead
ff-logging dep whose mandated Req 7.4/7.5 theme-validation WARNs are unimplemented
(PA-LOG-030 -- also satisfies the text-decorations Req 15.8 theme-side delegation),
adequate TCR (21), comment-only ASCII, and a high-contrast/WCAG boundary watch vs
accessibility (PA-WATCH-025, W4.9). No conflict, no split.
