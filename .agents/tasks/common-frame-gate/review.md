# Design Review -- The Common Workspace Frame (CR-NR-115)

Gate: `common-frame-gate` (requirements + design, DOCS ONLY).
Reviewer: design-review subagent (fresh read).
Scope reviewed: `git diff main...HEAD` (8 files, 767 insertions, no `.rs`) plus
`gate-summary.md`. Key investigation claims were re-verified against the live
source, not taken on the design's word.

Verdict: APPROVED (0 HIGH, 0 MEDIUM, 3 NIT).

---

## How the gate was judged

The brief defines the blocking conditions. Each was checked:

- No `.rs` file touched -- CONFIRMED. `git diff --stat` shows only
  `docs/specs/{workspace-framework,screen-snapshot-scrm,menu-and-statusbar}/*` and
  `docs/status/change-log.md`. No framework type or resolver changed.
- Eight common-frame elements each as EARS criteria -- PRESENT
  (`workspace-framework` Req 7.1-7.10), in the correct top-to-bottom order, each
  element named shell-owned and rendered exactly once.
- NEW transient Message_Line (clears on next action, may show multiple, captured
  in snapshot, replaces RUNNING) -- PRESENT (7.11-7.15; reproduction 30.8-30.9).
- Status bar = common segments (`Last: HH:MM`, `Prof: <name>`, Ln/Col, UTF-8, line
  count) PLUS a per-Kind segment, no RUNNING -- PRESENT (7.16-7.21; 30.10).
- PF-key line driven by `pfshow` -- PRESENT (7.22; 30.11).
- Single-chrome stated normatively; D1/D2/D6 folded in; D5 and Alias-specific D2
  fix flagged as follow-up -- CONFIRMED (7.23/7.24 single-representation; the
  numbering note and change-log map D1->single-chrome, D2->single-representation
  with the Alias provider fix DEFERRED to the Alias Kind gate, D5/D6->per-Kind
  follow-up).
- Common-vs-unique boundary stated normatively as the backbone -- CONFIRMED
  (Req 7 intro + 7.23-7.25; design Section 11 boundary table).
- FULL tab bar, all `[XXX]` tags + close affordance, NOT active-tag-only --
  CONFIRMED (7.4; 30.4 explicitly REVISES 29.2's active-tag-only rule).
- Box-drawing grid convention specified -- CONFIRMED (30.12-30.14; design Section
  13 fixture; runtime box-drawing exempt from the ASCII doc rule, doc fixture
  uses ASCII `+ - |` for safe diffing).
- Spec homes correct and cross-referenced -- CONFIRMED (contract + boundary in
  workspace-framework Req 7 / design Sec 11 / tasks Phase CF; snapshot
  reproduction revising/extending Req 29 in screen-snapshot-scrm Req 30 / design
  Sec 13 / tasks Phase 2H; status/menu/PF-key mechanics cross-referenced in
  menu-and-statusbar Req 5).
- design.md records investigation findings rather than assuming, with a framework
  verdict -- CONFIRMED and independently re-verified (see Verified Assumptions).
- tasks.md TDD-first, titled, `[ ]`/`[x]` only, cross-references criteria, scoped
  to the COMMON frame + headless snapshot reproduction, per-Kind deferred --
  CONFIRMED (Phase CF, Phase 2H; every impl task preceded by a failing-test task
  with a `Validates:`/`Covers:` criterion reference).
- Change-log CR entry with a correctly incremented id -- CONFIRMED (CR-NR-115 is
  the next free NR id; prior highest were CR-CH-115 and CR-NR-114, both verified
  present; no pre-existing CR-NR-115).
- Gate summary ends with `GATE_READY_FOR_APPROVAL` -- CONFIRMED.

No blocking condition is triggered.

---

## Findings

1. NIT -- Window-title-bar leading marker is inconsistent across the three
   places it appears. `workspace-framework` Req 7.2 and `screen-snapshot-scrm`
   Req 30.2 render the line as brand token + `_ [] X` with NO leading marker;
   Req 29.1 shows a `# |` marker and the design Section 13 fixture shows `| #  `.
   All three are explicitly "SETTLED-BUT-TUNABLE at implementation" (29.1), so
   this is presentation, not a contract conflict. Fix (optional): add a one-line
   note in Req 30.2 that the exact leading glyph follows 29.1's tunable marker,
   so an implementer does not read 7.2/30.2 as forbidding the `# |` marker 29.1
   permits.

2. NIT -- The design Section 13 fixture status line
   (`... | 1 lines | Aliases: 1 | Rows 1`) truncates the Alias Kind example
   segment mid-phrase at the frame edge. Since 7.21/30.10 defer the per-Kind
   segment CONTENT, this is only illustrative; but the clipped `Rows 1` could be
   misread as the literal required text. Fix (optional): annotate the fixture
   that the Alias segment text is illustrative and defined by the Alias follow-up
   Kind gate (the surrounding prose already says so; a note on the fixture line
   itself would remove any doubt).

3. NIT -- `workspace-framework` tasks Phase 2H dependency is stated in
   screen-snapshot-scrm tasks ("depends on workspace-framework Phase CF for the
   Message_Line and `last_action_at` shell state; where that state is absent the
   reproduction gracefully omits the message line"), which is correct and
   graceful. The reverse pointer (Phase CF does not mention Phase 2H consumes its
   state) is absent. Fix (optional): a one-line cross-reference in Phase CF noting
   screen-snapshot-scrm Phase 2H reproduces the Message_Line it introduces. Pure
   navigability; no correctness impact.

No HIGH or MEDIUM findings. The three NITs do not affect the verdict.

---

## Verified Assumptions (checked against live source, not taken on the design's word)

- Window title constant: design claims `ViewportBuilder::with_title("FileForge
  Workbench")` + `run_native("FileForge Workbench", ...)` in `main.rs`. VERIFIED
  -- `crates/ff-desktop/src/main.rs` lines 199 and 210 match exactly.
- Status-bar segments: design claims `render_status_bar` emits a phase label
  (`Running`), `format_session_start()` (Started), and `active_profile_label()`
  (Profile), followed by Ln/Col, encoding, line count. VERIFIED --
  `crates/ff-desktop/src/shell/render_status.rs` shows `LifecyclePhase::Running`
  phase label, `self.format_session_start()`, and `self.active_profile_label()`
  in the status row. The design's claim that RUNNING/Started/Profile are the
  segments to change is accurate.
- Workspace title source: design claims the title flows from
  `mod_helpers::title_line_text` -> `active_display_title`. VERIFIED --
  `title_line_text` exists in `crates/ff-desktop/src/shell/mod_helpers.rs`.
- PF-key bar visibility: design claims `render_key_label_bar` returns early when
  `!self.key_bar_visible`, driven by `pfshow`. VERIFIED --
  `render_key_label_bar` in `render_status.rs` begins `if !self.key_bar_visible {
  ... return; }`.
- CR id increment: design/summary claim CR-NR-115 is the next free NR id (prior
  highest CR-CH-115, CR-NR-114). VERIFIED -- change-log contains CR-CH-115 and
  CR-NR-114 and no pre-existing CR-NR-115; the new entry is at the top of the New
  Requirements section.
- "Seven of eight elements already shell-owned; Message_Line is the one new
  element": consistent with the verified sources above (title bar, menu bar, tab
  bar, title, command field, status bar, key bar all have existing render sites;
  the design correctly identifies `open_error` and `NotificationQueue` as the
  near-but-wrong existing mechanisms, so the Message_Line is genuinely additive
  shell state).

## Unverified / Wrong Assumptions

- None wrong. The following design claims were NOT independently opened but are
  low-risk and internally consistent with the verified set; an implementer should
  confirm them when coding (not blocking for a docs gate):
  - The exact `ScreenModel` slot set (`header_lines`/`body_lines`/`footer_lines`/
    `tables`/`command_line`/`status_bar`/`dimensions`) used by Req 30.17's
    "compose into existing slots, no reshape" claim. The design in
    screen-snapshot-scrm already cites these from prior CR-CH-115 work; this gate
    adds no reshape, so the risk is contained.
  - `BuiltinKind::default_title()` as the `[XXX]` tag source for the full tab bar
    (30.4). Cited consistently with CR-NR-090/CR-CH-045; not opened here.
  - The menu-bar line being a runtime echo of the menu TOML (CR-NR-114). Inherited
    claim from a prior gate; not opened here.
