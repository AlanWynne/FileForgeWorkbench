# ISPF Table Editor -- Requirements Gate Summary (CR-CH-076 / 077 / 078)

Docs-only gate for three owner-confirmed (2026-11-01) UI-behaviour change
requests against the shared ISPF table editor. NO `.rs` changed; no resolver or
framework type touched; no cargo gate run. STOP before merge for owner review.

Worktree: `c:/workspace/apps/FileForgeWorkbench/.worktrees/ispf-editor-gate`
(branch `ispf-editor-gate`).

## CR ids assigned

Next free ids in the change-log CR-CH series (highest existing was CR-CH-075):

- **CR-CH-076** -- Block line commands (RR / DD / CC / MM range pairs).
- **CR-CH-077** -- Single-chrome layout (live panel and snapshot agree).
- **CR-CH-078** -- Full arrow-key focus navigation (Up/Down rows, Left/Right cols).

All three logged under `docs/status/change-log.md` -> Change Requests.

## New acceptance-criteria ids

### `docs/specs/ispf-table-editor/requirements.md` (edited in place)

- **Requirement 9 (CR-CH-076) -- Block line commands.** Criteria 9.1-9.6:
  - 9.1 recognise doubled RR/DD/CC/MM as an inclusive, order-independent range;
    block set exactly RR/DD/CC/MM (no II).
  - 9.2 single R/Rn vs block RR distinction preserved.
  - 9.3 CC/MM blocks still require an A/B destination.
  - 9.4 unterminated block -> non-blocking "block not closed" notice, NO
    structural change, marker retained, rest of batch still applies.
  - 9.5 block set advertised per-host via LineCommandSet (opt-in).
  - 9.6 reuse the `ff-line-commands` engine block parsing/pairing; no second
    parser, no engine-API change.
- **Requirement 10 (CR-CH-077) -- Single-chrome layout.** Criteria 10.1-10.4:
  one Command line + one header owned by the component; host draws no duplicate;
  snapshot includes the Command line + header so live and snapshot agree; no new
  chrome mechanism / no dispatch-nav-save change.
- **Requirement 11 (CR-CH-078) -- Arrow navigation.** Criteria 11.1-11.6:
  Up/Down same column adjacent row (clamped); Left/Right adjacent column
  (clamped); component owns the walk and consumes keys before egui; Tab/Shift+Tab
  + stable ids unchanged; Command-line history gesture preserved; reusable by all
  hosts.

### `docs/specs/command-configurator/requirements.md` Req 7 (Alias, edited in place)

- **7.11** Alias opts IN to the block set (DD/CC/RR enabled; MM stays disabled
  because Move is not enabled for an unordered alias list); uniform-pattern
  rationale recorded.
- **7.12** Alias Context single-chrome: remove the duplicated heading + command
  line; the Alias `ScreenProvider` snapshot renders the same single-chrome layout
  (including the currently-missing Command line + header).
- **7.13** Alias inherits the shared component's arrow navigation; first-Tab test
  not weakened.

## ff-line-commands investigation verdict (REQUIRED by the brief)

**BUILD ON -- NO engine public-API change needed.** The `crates/ff-line-commands`
engine was read end to end (`parser.rs`, `command.rs`, `block_pair.rs`,
`pending.rs`). It ALREADY fully models the doubled block form and paired-range
semantics:

- `LineCommandParser::parse` matches the doubled tokens FIRST, case-insensitively:
  `DD -> DeleteBlock`, `RR -> RepeatBlock`, `CC -> CopyBlock`, `MM -> MoveBlock`
  (existing tests `parse_dd_delete_block` / `parse_rr_repeat_block` /
  `parse_cc_copy_block` / `parse_mm_move_block` / `parse_mixed_case_works`).
- `command::classify` returns `LineCommandCategory::Block` for the `*Block` kinds;
  `is_block_marker` / `to_block_kind` map them; single `R`/`D` stay `Immediate`,
  `C`/`M` stay `Source` -- so the R-vs-RR distinction is type-level, not heuristic.
- `BlockPairValidator::validate_pair` forms an inclusive, order-normalised
  `BlockPair { kind, start_line, end_line }` (via `normalize`); `check_overlaps`
  rejects overlapping ranges.
- An unterminated block (0/1 marker) returns `LineCommandError::AwaitingPair`;
  > 2 markers returns `TooManyMarkers` -- exactly the "block not closed" /
  malformed signal Requirement 9.4 needs. `PendingCommandStore` retains a lone
  marker across a cycle.

Consequence: CR-CH-076 is NOT a framework-adjacent engine change. The only code
that changes is the shared component's apply adapter
(`crates/ff-desktop/src/shell/ispf_table_editor/line_commands.rs`, which today
classifies every `Block` kind as "not enabled" and clears it) plus a block
opt-in flag on the component's own `LineCommandSet`. The brief's "flag an
ff-line-commands public-API change for owner confirmation IF the parser does not
already do blocks" condition does NOT fire -- the parser already does blocks.

## Open owner decisions: NONE

Both points raised at the first report are now RESOLVED by the owner (relayed via
the orchestrator); recorded here for the record:

1. **Alias `description` field (CR-CH-075 carryover) -- SETTLED = option (a),
   implemented.** The owner chose the additive optional
   `#[serde(default, skip_serializing_if)] description` field earlier this
   session; the Description column SHIPPED in the Group B merge (commit 1532a5f)
   and is live on main. command-configurator Req 7.2 / 7.3 are recorded as
   SETTLED on option (a); option (b) is not carried forward. NOT an open decision.

2. **MM disabled for the Alias Editor -- CONFIRMED.** The Alias block set is
   DD/CC/RR; MM stays disabled because the alias list is unordered (resolver keys
   by surface, auto-sort-on-save), so Move has no meaning there -- consistent with
   the earlier decision that Alias line commands are I/D/C + Sort, no Move. The
   shared component still supports MM for ordered hosts (Menu, Kinds). Req 7.11 is
   kept as written.

No new framework or engine-API decision is required by CR-CH-076 / 077 / 078:
the block engine already exists (BUILD-ON, no `ff-line-commands` API change),
and arrow navigation and single-chrome are additive on existing seams. The owner
is asked only to APPROVE the three change requests and their criteria so the
orchestrator can merge.

## Files changed (docs only)

- `docs/specs/ispf-table-editor/requirements.md` -- NEW Req 9, 10, 11; source
  refs + Out-of-Scope updated.
- `docs/specs/ispf-table-editor/design.md` -- NEW CR-CH-076 investigation verdict
  + adapter plan, CR-CH-078 arrow-walk design, CR-CH-077 single-chrome design,
  reusability principle; framework-conformance statement extended.
- `docs/specs/ispf-table-editor/tasks.md` -- NEW task groups 9, 10, 11 (TDD-first:
  unit tests for block apply + unterminated-block notice + matrix-walk clamp;
  full-shell egui_kittest for in-cell arrow focus and block commands); coverage
  table extended.
- `docs/specs/command-configurator/requirements.md` -- Req 7 heading/source +
  NEW criteria 7.11 / 7.12 / 7.13.
- `docs/status/change-log.md` -- three CR entries (CR-CH-076 / 077 / 078).
- `.agents/tasks/ispf-editor-gate/gate-summary.md` -- this summary.

## Next step

STOP for owner approval. On approval the orchestrator merges/cleans up and the
implementation proceeds TDD-first per the task groups (compile-verify scoped per
`.kiro/steering/gate-policy.md`; owner runs the full gate).
