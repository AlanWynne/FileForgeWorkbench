# Design Review (Revision 1) -- Root-Command Help Model (CR-NR-119)

Scope: the requirements + design docs-only gate in worktree
`.worktrees/command-help-gate`. This is a REVISION review: the prior review
(`review.json`, verdict CHANGES_REQUESTED) raised 2 MEDIUM findings. I re-checked
both against the revised design/requirements and the live source, then reviewed
what changed.

Reviewed: gate-summary.md, the diff `main...HEAD`, context-help requirements.md
(Req 21-23), command-semantics Req 15, command-configurator Req 9,
command-framework Req 3.8/3.9, context-help design.md (CR-NR-119 section),
tasks.md (tasks 24-29 + coverage mapping), `.kiro/steering/command-help-guideline.md`,
and the CR-NR-119 change-log entry. Source claims were verified against the live
tree (`crates/ff-help/src/commands.rs`, `crates/ff-help/src/topic.rs`,
`crates/ff-desktop/src/shell/helpers.rs`, `crates/ff-desktop/src/shell/alias_reader.rs`,
`crates/ff-desktop/src/shell/environment_registry.rs`).

Verdict basis: 0 HIGH, 0 MEDIUM (2 NIT). HIGH+MEDIUM == 0 => APPROVED.

---

## Prior findings re-check

### Prior Finding 1 (MEDIUM) -- "No routing change needed" false for KEYS/CONFIG -- CLOSED

Verified the root cause still exists in source: `resolve_help_argument`
(`crates/ff-help/src/commands.rs`) matches `"KEYS" => feature("function_keys")`
and `"CONFIG" | "CONFIGURATION" => feature("configuration")` BEFORE the
`cmd:<NAME>` fall-through, and both KEYS and CONFIG are enforced FFCMD roots.

The revision ADDRESSES this correctly:
- The unqualified "no routing change is needed" claim is DELETED from design.md.
- The "HELP <cmd> routing for FFCMD roots" section now explicitly documents the
  divergence and chooses recommended option (a): enforced-root `cmd:<ROOT>`
  resolution takes PRECEDENCE over the `feature:` keyword special-cases, so
  `HELP KEYS`/`KEYS --help` open `cmd:KEYS` and `HELP CONFIG`/`CONFIG --help`
  open `cmd:CONFIG`; the `feature:*` topics stay reachable via Index/menus.
- New criterion context-help Req 22.8 encodes this precisely (names the resolver,
  the two roots, the precedence, and preservation of the feature topics).
- The design `--help` pseudocode now routes through an enforced-root-aware
  `open_root_help` rather than a bare `open_help`.
- Task 25.4 adds the RED-first test for the precedence; the coverage mapping
  maps Req 22.8 to 25.4.

Closed.

### Prior Finding 2 (MEDIUM) -- `--help` cannot serve two-word roots -- CLOSED

Verified the root cause: `split_verb_arg` (`crates/ff-desktop/src/shell/helpers.rs`)
is `split_once(char::is_whitespace)` -- first whitespace only. So
`FILE CATALOGS --help` yields verb=`FILE`, arg=`CATALOGS --help`, first arg token
is not a help flag.

The revision ADDRESSES this correctly:
- New "Known limitation -- two-word roots and the `--help` token split"
  subsection in design.md states the problem and chooses recommended option (a):
  `--help` applies to single-token verbs this version; `FILE CATALOGS` /
  `RESET BARE` POSIX `--help` is DEFERRED, both still reachable via `HELP <ROOT>`.
- New criterion command-semantics Req 15.9 encodes the single-token scope, names
  the `split_verb_arg` reason, defers the two-word roots, preserves the HELP
  reach-path, and marks a later trailing-token extension as a non-framework
  additive follow-up.
- Req 15's design-decision note, the `--help` pseudocode comment (`// single-token
  verb (Req 15.9)`), and tasks 25.2/25.5 reflect the scope and the limitation.

Closed.

---

## Findings (this revision)

### 1. NIT -- `open_root_help` is referenced as both a concrete name and "equivalent"

Where: design.md `--help` pseudocode calls `self.open_root_help(verb)` and task
25.4 says "an `open_root_help`/equivalent". The name is introduced in the design
as the enforced-root-aware help open but is not a committed public seam name.

Why NIT: it does not affect correctness or the criteria (the criteria speak in
behaviour, "resolve `cmd:<ROOT>` directly"); naming is an implementation choice.
Fix (optional): add one sentence noting `open_root_help` is an illustrative name
for the enforced-root branch and the implementer may inline it into the existing
`open_help` seam, as long as there remains ONE resolution path (Req 22.1/22.8).

### 2. NIT -- Req 21.4 ties the 200-char cap to "command-semantics Status_Message length" without a cross-ref id

Where: context-help Req 21.4 says "<= 200 characters (consistent with
command-semantics Status_Message length)". command-semantics does define
Status_Message as "short (<=200 characters)" (verified, Req 1/definitions), but
21.4 does not cite the specific requirement id.

Why NIT: the number is unambiguous and self-contained; the parenthetical is only
rationale. Fix (optional): cite the command-semantics Status_Message definition
id for traceability.

---

## Verified Assumptions

- `resolve_help_argument` (`crates/ff-help/src/commands.rs`) special-cases
  `KEYS` -> `feature:function_keys` and `CONFIG`/`CONFIGURATION` ->
  `feature:configuration` before the `cmd:<NAME>` fall-through (Finding 1 root
  cause). The design's Req 22.8 description of this is accurate.
- `split_verb_arg` (`crates/ff-desktop/src/shell/helpers.rs`) splits on the first
  whitespace only (`split_once(char::is_whitespace)`), confirming the two-word
  root problem and the Req 15.9 rationale.
- `builtin_ffcmd_roots()` exists (`crates/ff-desktop/src/shell/alias_reader.rs`)
  and unions the recovery POM + recovery SETTINGS menu-option roots +
  `BUILTIN_WORKSPACE_VERB_ROOTS` + `BUILTIN_FFCMD_LADDER_VERB_ROOTS`, matching the
  gate's enumeration and the "built-in menu option roots (HELP, X, RESET BARE,
  SETTINGS)" claim.
- `EnvironmentRegistry::root_commands("FFCMD")`
  (`crates/ff-desktop/src/shell/environment_registry.rs`) exists and returns the
  FFCMD root set; the enforcement test target is real.
- `HelpTopic::body()` exists (`crates/ff-help/src/topic.rs`), so
  `extract_one_line_description(body)` and `short_description` are feasible as
  designed (Req 21.5 / 22.2).
- The single front-door order is as the design states (`=` strip ->
  `rewrite_verb_to_root` -> `run_command_prelude` -> `resolve_target` ->
  `run_command_ladder`). A prelude-level `--help` short-circuit after the rewrite
  is framework-conforming (mechanism 1) -- no second dispatcher.
- Alias canonicalisation `SS --help` -> `cmd:SNAPSHOT` is sound: the rewrite runs
  before the short-circuit (Req 15.3).
- CR id `CR-NR-119` is the next free, unique id: the change-log's highest prior
  New-Requirement id is CR-NR-118; 119 is distinct from CR-CH-116; no reuse.
- EARS form throughout (WHEN ... THE ... SHALL ...); every new criterion carries
  the `[CR-NR-119]` tag; cross-references resolve.
- The CR-NR-119 ADDED doc lines are ASCII-only (`--`, `->`, straight quotes).
  Non-ASCII hits in the files (a `<=200` glyph in the pre-existing Status_Message
  rows of command-semantics, box-drawing in design.md's pre-existing Overview
  code block) are UNCHANGED lines outside this diff; box-drawing in a fenced code
  block is allowed by documentation-charset.md.
- Tiered enforcement is correctly scoped: enforcement lives in an `ff-desktop`
  TEST over `root_commands("FFCMD")` (Req 23.2/23.3); FFEDIT deferred (Req 23.4);
  extensions never asserted and degrade gracefully (Req 23.5, steering guideline)
  -- no registration-time block, matching owner message 12.
- Description-source unification is genuine: one `short_description(cmd:<ROOT>)`
  lookup backs the alias editor, palette, and status line; the 4 CommandRegistry
  descriptions and the ff-file-ops table stay for their dotted Command_Ids but are
  explicitly NOT the root-verb source. No new duplicate literal is introduced
  (Req 22.2/22.3, command-framework Req 3.8).
- Help-file structure decision is sound: Option (c) (formalise the existing
  `cmd:<NAME>` topic, extract the first `## Description` paragraph) is presented
  against (a) YAML frontmatter and (b) heading convention with a clear
  recommendation that prefers extending the shipped structure. Deterministic
  extraction (first non-empty Description paragraph, inline markup stripped,
  whitespace collapsed, non-empty, <= 200 chars). No new markup (Req 21.7).
- Tasks use only `[ ]` markers, each has a title, each test carries a
  `// Validates:` line, and the coverage mapping covers 21.1-21.7, 22.1-22.8,
  23.1-23.6, 15.1-15.9, 9.1-9.5, and 3.8/3.9 (including the revision's 22.8 ->
  25.4 and 15.9 -> 25.2/25.5).

## Unverified / Wrong Assumptions

- UNVERIFIED (design-acknowledged, acceptable, not blocking): decision point 5 --
  whether `TopicKey` round-trips an embedded space in `cmd:FILE CATALOGS` /
  `cmd:RESET BARE`. The design flags this as "confirm in implementation";
  `resolve_help_argument` uppercases and preserves the space, so the HELP
  reach-path is plausible. Left to implementation; not a gate blocker.
- No wrong assumptions found in this revision. Both prior WRONG items (Finding 1
  and Finding 2) are now corrected in the docs.
