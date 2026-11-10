# Design Review -- Group B Requirements Gate (CR-CH-075)

Reviewer: design-review subagent (fresh read, no authoring context).
Scope reviewed: the Group B docs-only requirements gate in the worktree
`c:/workspace/apps/FileForgeWorkbench/.worktrees/group-b-reqs`:

- `docs/specs/ispf-table-editor/{requirements,design,tasks}.md` (NEW)
- `docs/specs/command-configurator/requirements.md` (NEW Req 7)
- `docs/specs/menu-workspace/requirements.md` (NEW Req 13 criteria 14-22)
- `docs/specs/workspace-kinds/requirements.md` (NEW Req 10)
- `docs/status/change-log.md` (CR-CH-075 entry)
- `.kiro/steering/specs.md` (folder registration)

Grounded against `.agents/tasks/ispf-editor-redesign-investigation/findings.md`,
`.kiro/steering/framework-conformance.md`, `wiring-standard.md`,
`documentation-charset.md`, `workspace-conformance.md`, `save-convention.md`,
and the `ff-line-commands` crate source.

Verdict: APPROVED (0 HIGH, 0 MEDIUM, 3 NIT).

---

## Verified Assumptions

1. **Menu "Command/Alias" column is the existing `command` field relabelled, NO
   schema change.** menu-workspace criterion 15 and its OWNER-CONFIRMED note state
   the column maps to the existing `MenuOption.command` (a root command OR an
   alias; the string already carries either) with no new field and no TOML schema
   change. This matches findings section 2 (the `command` field exists; "Alias"
   had no backing field) and resolves the investigation's open question the
   correct way. CONFIRMED.

2. **Per-editor line-command sets match the brief.**
   - Alias: `I` / `D` / `C` + SORT, Move NOT enabled (command-configurator 7.4).
     Matches the brief (Alias = I/D/C + Sort, no move) and the rationale (an
     alias set is keyed by surface, not position). CONFIRMED.
   - Menu: `I` / `D` / `C` / `M` (menu-workspace criterion 16). CONFIRMED.
   - Kinds: `I` / `D` / `C` / `M` (workspace-kinds criterion 10.4). CONFIRMED.

3. **Config editor redesign recorded as out-of-scope follow-up, not specified.**
   ispf-table-editor Requirement 8.2 and the Out of Scope section, plus the tasks
   Notes, all state the Config editor ISPF redesign is OUT OF SCOPE and recorded
   as a follow-up. No Config criteria are authored. CONFIRMED (matches findings
   Group B item 4).

4. **Snapshot/capture column alignment with Group A.** ispf-table-editor
   Requirement 7.3 makes the displayed `ColumnSpec` order the authoritative set
   each host `ScreenProvider` populates into a `ff-screen-model::Table`; the three
   per-editor specs restate it (command-configurator 7.9, menu-workspace 21,
   workspace-kinds 9). Consistent with findings section 6 (renderer exists,
   provider change only). CONFIRMED.

5. **Framework conformance -- builds ON the framework, no framework change.** The
   design's "Framework-conformance statement" and each spec's non-change clause
   (command-configurator 7.10, menu-workspace 22, workspace-kinds 10) assert: no
   new dispatcher, no second navigation stack, no new session-persistence format,
   no new line-command engine, no new save path, no reshape of a core type, no
   `ff-line-commands` public-API change. The component is a `ff-desktop` panel
   helper the three `WorkspaceContext` panels call; dispatch/nav/save/focus route
   through the existing seams (CR-NR-078, CR-CH-071, the `ff-line-commands`
   engine). This is consistent with `framework-conformance.md` and
   `wiring-standard.md`. CONFIRMED.

6. **The ONE schema change is FLAGGED, not silently specified.** The Alias
   Description column (replacing the retired L18N column) requires an additive
   optional `#[serde(default)]` `description` field on the alias row. Both the
   change-log entry and command-configurator Req 7 (OWNER-CONFIRM FLAG, options
   (a) persisted / (b) display-only) raise it explicitly for owner confirmation at
   CONFIRM, and correctly classify it as additive (NOT a core-type reshape, so not
   a framework change) while still a persisted-shape change. This satisfies the
   "flag, do not silently specify" requirement. CONFIRMED.

7. **ASCII-only in the new content.** A `[^\x00-\x7F\u2500-\u257F]` scan of all
   six touched files reports non-ASCII ONLY on three PRE-EXISTING change-log rows
   (lines 189/367/911/919/927) that use the established status-emoji convention
   (red/green/blue circles) in unrelated earlier entries. The new CR-CH-075
   content (ispf-table-editor specs, the editor-spec additions, specs.md line, the
   CR-CH-075 change-log entry) is clean ASCII. CONFIRMED.

8. **specs.md registration + change-log entry both present.** specs.md adds the
   `ispf-table-editor` line with a correct description; change-log.md adds the
   CR-CH-075 entry with status GATE AUTHORED / PENDING OWNER APPROVAL and the loop
   closing on the review verdict. CONFIRMED.

9. **Alias redesign correctly supersedes the existing I/R/D mini-flow.** findings
   section 5a states the Alias editor today has a bespoke I/R/D flow in
   `alias_editor_line_commands.rs`; command-configurator 7.6 replaces it with the
   shared engine-backed I/D/C flow and retains `R` only as an optional single-row
   Copy alias. Consistent. CONFIRMED.

10. **TDD + compile-verify split honoured in tasks.** ispf-table-editor tasks.md
    writes failing unit tests first for the pure apply/sort/filter adapter, scopes
    agent verification to `cargo check/clippy/fmt -p ff-desktop` + quick scoped
    tests, defers the full gate to the owner, and cites the 400-line rule. Matches
    `testing.md` and `gate-policy.md`. CONFIRMED.

## Unverified / Wrong Assumptions

1. **WRONG (minor, in the change-log only): "ff-line-commands ... already does
   D/I/R/C/M/sort for the text editor".** The `ff-line-commands` crate docstring
   (`crates/ff-line-commands/src/lib.rs`) lists Delete / Insert / Repeat / Copy /
   Move / Exclude / Tag-Untag / Shift -- there is NO Sort command in the engine.
   The DESIGN is correct and does NOT depend on the engine for Sort: design.md
   implements Sort at the component level (`slice::sort_by_key`, stable) and
   ispf-table-editor Requirement 5 / tasks group 3 keep Sort separate from the
   engine. So the only inaccuracy is the change-log's parenthetical "/sort"
   attribution to the engine. See NIT 1.

2. **Not independently re-verified (accepted from findings, low risk):** that the
   `EditProfile { caps, nulls, stats, lock, hilite }` fields and the full
   `Kind_Config` / `Kind_Profile` field set already exist (workspace-kinds 10.2/
   10.3 claim "no new field"). findings section 5b asserts these all exist; I did
   not re-open `profile.rs` / `workspace_kind/mod.rs`. Since this gate is docs-only
   and the claim is the investigation's, it is low risk; the implementer must
   confirm at code time.

---

## Findings

### NIT 1 -- Change-log falsely attributes Sort to the `ff-line-commands` engine
Location: `docs/status/change-log.md`, CR-CH-075 entry, Description item (1):
"...the existing `ff-line-commands` engine (`crates/ff-line-commands`, already
does D/I/R/C/M/sort for the text editor)...". The engine does D/I/R/C/M/X/T/U/
shift but NOT sort (verified in `lib.rs`). The design correctly implements Sort at
the component level, so this is a wording inaccuracy with no design impact.
Concrete fix: change the parenthetical to "already does D/I/R/C/M for the text
editor; Sort is a component-level reorder, not an engine command" (or drop
"/sort" from the engine list).

### NIT 2 -- Req 4.3 "independent of the order the user typed" reads as over-broad for paired C/M
Location: `docs/specs/ispf-table-editor/requirements.md` Requirement 4.3: "the
result SHALL be independent of the order in which the user typed the letters
within one batch." For `C`/`M` with `A`/`B` target markers the outcome is
inherently position/marker dependent, so a literal reading conflicts with the
pairing semantics of 4.4. The intent (deterministic apply via descending-index
processing for immediate I/D) is clear from context, but the blanket wording is
ambiguous. Concrete fix: scope 4.3 to the immediate commands, e.g. "FOR the
immediate commands (I/D), the batch result SHALL be independent of the order the
letters were typed (achieved by descending-index processing); paired C/M results
are determined by their source/target markers per 4.4."

### NIT 3 -- "ff-line-commands already a ff-desktop dependency transitively" is an unverified aside
Location: `docs/specs/ispf-table-editor/design.md`, Technology-stack paragraph:
"the existing `ff-line-commands` crate (already a `ff-desktop` dependency
transitively via the editor stack)". findings section 5a says the engine is wired
to the TEXT editor, which implies the dependency path exists, but the "transitive
`ff-desktop` dependency" is stated as fact without a reference. If it is only a
`ff-editor`-level dependency, the implementer may need to add `ff-line-commands`
to `ff-desktop`'s `Cargo.toml` (an additive, non-framework change). Concrete fix:
soften to "reused via the editor stack; add `ff-line-commands` to `ff-desktop`'s
`Cargo.toml` if it is not already a direct dependency" so the task list captures
the possible Cargo edit.

---

## Verdict rationale

The gate is thorough, well grounded in the investigation findings, and framework
conformant. Every load-bearing mechanism is explicitly preserved, the single
persisted-shape change (Alias Description) is correctly flagged for owner
confirmation rather than silently specified, the Menu Command/Alias relabel is
recorded as no-schema-change, the per-editor line-command sets match the brief,
the Config redesign is deferred, and snapshot-column alignment with Group A is
specified. EARS criteria are in WHEN/THE/SHALL form and testable; the tasks
follow TDD and the compile-verify/gate split. The three NITs are wording/accuracy
clarifications with no design impact.

Count: 0 HIGH, 0 MEDIUM, 3 NIT -> APPROVED.
