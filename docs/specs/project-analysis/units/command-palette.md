# Analysis Record: command-palette (W3.3)

- **Wave**: 3 (Shell, commands, menus, session)
- **Backing code**: NO dedicated crate -- implemented in `ff-desktop` as a modal
  overlay module `ff-desktop/src/command_palette/` (mod.rs, render.rs 350,
  state.rs 95, fuzzy.rs 90). Reads the `ff-command` Command_Registry; the spec
  explicitly says it "requires no new library crates".
- **Spec files**: requirements.md (167 lines, 5 requirements), tasks.md
  (20 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 3 pass

---

## 1. Split candidacy

N/A / NOT a split candidate. Small, focused shell feature: 5 reqs, 167 lines,
20 tasks. One cohesive concern (Ctrl+Shift+P fuzzy-search command overlay:
activation, fuzzy search, entry display, execution, recent commands). No file
near the cap (render.rs 350 largest). No split.

---

## 2. Cross-unit consistency

### PA-CONFLICT-009 (NEW) -- duplicate fuzzy matcher vs command-completion

Req 2 ("Fuzzy Search Over Commands") is implemented by command-palette's OWN
`fuzzy.rs` (`ff-desktop/src/command_palette/fuzzy.rs`, 90 lines): `fuzzy_match(query,
target) -> bool` + `fuzzy_score(query, target) -> i32` (subsequence match + scoring
by contiguity/word-boundary).

command-completion (W3.2, `ff-completion/src/matching/fuzzy.rs`) ALSO implements a
fuzzy subsequence matcher: `fuzzy_match(query, candidate, case_sensitive) ->
Option<FuzzyMatchResult>` + `highlight_positions(...)`.

Two fuzzy subsequence matchers doing the SAME job -- both "fuzzy search over
registered commands" for the two closely-related ISPF-discoverability features
(palette Ctrl+Shift+P; completion autocomplete). ff-desktop has NO dep on
ff-completion (grep empty), so the palette CANNOT reuse the completion crate's
matcher -- it rolls its own, and the two have already DIVERGED in signature
(bool/i32 score vs Option<result> + highlight positions). Same duplicate-logic
anti-pattern as PA-CONFLICT-003/004/006/008.

Recorded PA-CONFLICT-009 (owner-gated, LOW-MED): extract a single fuzzy-match
engine (candidate: a small `ff-fuzzy` crate, or expose ff-completion's matcher as
the shared one) consumed by BOTH command-completion and command-palette. Unifies
scoring/highlight behaviour so the palette and autocomplete rank identically.
Lower urgency than the Wave-2 domain-type conflicts (both impls are small and
self-contained), but it is a real duplication of the same algorithm.

### CommandRegistry consumption -- CLEAN

The palette reads command names/metadata from the `ff-command` Command_Registry
(Req 2/3), correctly consuming the single registry (like command-completion's
command_name provider, W3.2). No registry duplication. Recent-commands (Req 5) is
palette-local session state. Command execution (Req 4) dispatches through the
command framework. Consistent.

### Public types and ownership

- CommandPaletteState, PaletteEntry, the palette fuzzy matcher, render/activation
  logic -- sole-owned by the ff-desktop command_palette module. Only the fuzzy
  matcher is a duplication (PA-CONFLICT-009).
- No config namespace of its own beyond activation (Ctrl+Shift+P). No fs, no VFS.

### Cross-reference integrity

Cross-refs (command-framework registry) resolve. No dangling refs.

---

## 3. Completeness

Tracking: all 20 sub-tasks `[x]`. Implementation present for all 5 reqs
(activation/dismiss, fuzzy search, entry display, execution, recent commands).
Tests in-file (fuzzy.rs has match/score tests). No PA-INCOMPLETE. This is a small,
complete feature.

### TCR gap (PA-TCR-015)

TCR.md has ZERO rows for command-palette (grep = 0) across 5 reqs. Total absence,
though the feature is small. RECORDING gap. Recorded PA-TCR-015 (LOW -- small
surface).

---

## 4. Logging audit

Scan of `ff-desktop/src/command_palette`:

- `ff_logging` / `log_*!`: 0
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0
- non-ASCII: 0 (clean)

No mandated logging in the 5 reqs (a pure UI overlay). Zero-log is defensible.
Command execution from the palette flows through the command framework, so the
CR-NR-058 per-command instrumentation (PA-W0.2) will cover the invocations
automatically -- dev-logging on palette open/select is a nice-to-have only.
Recorded PA-LOG-020 (LOW): optional dev-logging on palette activation/selection
under the `dev-logging` gate; not required.

---

## 5. Task revision proposals

- **PA-CONFLICT-009 (owner-gated, LOW-MED)**: unify the fuzzy-match engine between
  command-palette (ff-desktop) and command-completion (ff-completion) -- extract a
  shared `ff-fuzzy` (or expose ff-completion's matcher) consumed by both. Both
  impls are small/self-contained so low risk, but it is a real duplicate of the
  same subsequence-match algorithm and the two have already diverged.
- **PA-TCR-015 (LOW)**: add TCR rows for the 5 palette reqs (0 today). Small surface.
- **PA-LOG-020 (LOW)**: optional dev-logging on palette open/select; command
  invocations covered by PA-W0.2 instrumentation.

No PA-STD (no file over cap, 0 non-ASCII). No requirement CHANGE proposed; the
spec is small, internally consistent, and complete.

---

## Summary

command-palette is a small, complete shell feature (Ctrl+Shift+P fuzzy command
overlay) implemented in `ff-desktop/src/command_palette/` -- no dedicated crate,
reads the ff-command registry, clean on size/ASCII/fs/logging. The one finding is
PA-CONFLICT-009 (NEW, LOW-MED): it rolls its OWN fuzzy subsequence matcher
(`fuzzy.rs`) that duplicates command-completion's `matching/fuzzy.rs` -- two fuzzy
engines for the two closely-related discoverability features (palette + autocomplete),
already diverged in signature, with no shared crate (ff-desktop has no ff-completion
dep). Extract a shared fuzzy engine. Minor: TCR total absence (PA-TCR-015, small
surface) and optional palette dev-logging (PA-LOG-020). The command execution path
is clean (dispatches through the framework; covered by the CR-NR-058 PA-W0.2
instrumentation).
