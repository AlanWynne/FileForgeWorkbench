# Design Document -- Command Environments (CR-CH-053)

## Overview

The Command Environment model generalises the single B080 command front door
(`dispatch_command_string`) to N named environments, modelled on REXX/ISPF
ADDRESS. It BUILDS ON the existing framework: FFCMD is the existing
`resolve_target` chain, the router inserts one active-environment step into the
one front door, and no second dispatcher or navigation stack is added. Phase 1
builds the framework + FFCMD (existing) + FFEDIT (editor command-line verbs
migrated off the shared ladder); FFLINE is named-only; all other FF*
environments are vision-only.

See `requirements.md` for the criteria and `environments-vision.md` for the full
FF* catalogue. This document describes the architecture.

## The CommandEnvironment trait

Reuse the shape that already works (the `TargetResolver` pattern). An environment
is a named claim-or-fall-through resolver:

```
pub trait CommandEnvironment {
    /// Stable name for addressing and diagnostics ("FFCMD", "FFEDIT", "FFLINE",
    /// future "FFSQL"...). Case-insensitive match; the Alias_Map resolves
    /// mainframe names (TSO -> FFCMD, ISREDIT -> FFEDIT) onto these.
    fn name(&self) -> &'static str;

    /// Try to CLAIM a raw command string. Internally the environment first
    /// resolves the typed VERB (surface form / alias) to its CANONICAL verb via
    /// its per-environment alias table (Requirement 6a), case-insensitively,
    /// THEN dispatches the canonical verb's handler. Some(outcome) = this
    /// environment owns the (canonical) verb and handled it (outcome carries a
    /// return code for macro RC and the open_error/status to surface); None =
    /// fall through to the next step. Mirrors the ladder's
    /// `try_commands_* -> bool`, typed. The canonical verb (not the alias) is
    /// what is recorded/persisted (Requirement 6a.2).
    fn dispatch(&mut self, raw: &str, cx: &mut EnvContext) -> Option<EnvOutcome>;
}
```

The per-environment ALIAS TABLE (surface form -> canonical verb) is the point
where localization extends to verbs: phase 1 seeds it with the existing English
aliases (incl. EXCLUDE/X, SHOW/INCLUDE); CR-NR-103 loads per-locale alias sets
into the same table. Collisions are rejected at load; the prelude/shadowing
precedence is respected.

`EnvContext` / `EnvOutcome` are small shims local to the router: the outcome is
converted into the existing `open_error` / Command_Line_Outcome at the front-door
boundary, so command-framework Req 13.1 stays the one decision point. The outcome
return code feeds the macro `RC` (Requirement 8.3).

### FFCMD is NOT a new type

FFCMD == the existing `resolve_target(raw, &ShellTargetResolver)` ->
`dispatch_command_target(target)` pair
(`crates/ff-command/src/command_target.rs:267`,
`crates/ff-desktop/src/shell/target_dispatch.rs`). The router treats FFCMD as the
base fallback and runs that exact path. This is the single most important
reconciliation: there is one shell resolver, not two.

### FFEDIT is a resolver+executor over the active editor

FFEDIT owns the verbs now in `try_commands_b2`
(`crates/ff-desktop/src/shell/commands_ladder_b2.rs`), each delegating to the
existing manager (`nav_manager`, `exclude_manager`, `find_manager`,
`edit_profile`, `scroll_amount`). Because these mutate shell-entangled state
(`self.tabs` + the managers), FFEDIT is implemented first as methods ON
`WorkbenchShell` behind an "editor env" marker that the router calls (least
churn, mirrors how the Files Panel keeps its own redirect), with the cleaner
borrowed-manager-struct form as the eventual refactor once the managers are
grouped (rust-standards god-struct guidance).

FFEDIT claims its verbs at the active-environment step -- BEFORE the FFCMD base
and BEFORE the Command_Engine terminal fallback -- preserving today's precedence
(the shell-ladder arm handled these verbs ahead of the engine). The
Command_Engine stays the final fallback for input FFEDIT does not claim (the
prefix-area/line-command path and any engine-only primary command). FFEDIT and
the Command_Engine are therefore not two competing executors of the same
command-line verb (Requirement 6.2a; resolves the B080-Step-4-era boundary
fuzziness).

### FFLINE and FFNAV are named only (phase 1)

FFLINE (prefix-area line commands) is a sibling of FFEDIT; its intake is the
prefix gutter -> `ff-command-semantics` Command_Engine and is UNCHANGED in phase
1 (not routed through the command-line front door). FFNAV (file-navigator
Contexts) owns the navigator verbs -- including its OWN FIND (locate a file, not
search a buffer) -- which stay handled wherever they are today; FFNAV is not
built as a first-class resolver in phase 1. Both are NAMED so the active-env
derivation and the per-environment verb model (Requirement 2a) are correct and
so future `ADDRESS FFLINE` / `ADDRESS FFNAV` have defined targets. Only FFEDIT is
migrated in phase 1.

## The Environment_Registry

A small shell-owned registry: the FFCMD base (implicit = resolve_target) plus the
Context environments (phase 1: FFEDIT). Its jobs: (1) given the focused Context,
return the Active_Environment; (2) given a name (for addressing), return that
environment; (3) expose FFCMD as the base. It is not a dispatcher; it feeds the
one front door.

## Active-environment derivation (kind-supplied attribute)

The active environment is SUPPLIED BY the focused Context's KIND as a kind
ATTRIBUTE (a named reference), NOT a central hardcoded match in the handler. The
handler asks the focused kind "what is your command environment?" and dispatches
to whatever name comes back, defaulting to FFCMD when a kind supplies none. This
INVERTS the dependency: the handler depends on the abstraction (a kind supplies
an environment name), not on the concrete list of kinds, so a new kind plugs in
with NO handler change (Requirement 9).

The command-environment name is one more Workspace Kind attribute, alongside the
existing title / menu-bar / key-list / profile attributes (CR-NR-090 Kinds
registry). It is DATA (a name), never executable code: a kind (or a user
reconfiguring it) may POINT at a different existing environment, but cannot supply
an environment's executable implementations. Built-in environments (FFCMD,
FFEDIT) are code-only and not replaceable by configuration (like code-only
built-in menus/themes, CR-CH-021); a genuinely new executable environment is a
PLUGIN (registered through the plugin API under the plugin permission model --
"copy the Editor Plugin to build your own"), and per-verb customization is via a
shadowing layered environment (Requirement 5), not wholesale replacement.

The same verb NAME (e.g. FIND) is owned by multiple environments, each with its
own canonical verb + implementation; the active Context selects which runs
(FFEDIT.FIND searches the open file; FFNAV.FIND locates a file). The environment
is the namespace (Requirement 2a).

Phase-1 kinds declare their environment as:

| Focused Context kind declares | Active_Environment | Phase-1 status |
|-------------------------------|--------------------|----------------|
| FileEditor, Untitled | FFEDIT | BUILT (verbs migrated off the ladder) |
| FilesPanel, FileExplorerPanel, catalog | FFNAV | NAMED only (navigator verbs incl. its own FIND stay where handled today; not migrated) |
| every other kind (declares nothing) | FFCMD | base default (= resolve_target) |
| future Database Context | FFSQL (the kind declares it; registers its own env) | vision-only |

These are the KINDS declaring their environment, not a handler lookup table. The
handler reads the attribute; it does not enumerate kinds. FFNAV is named now so
the model and derivation are correct; only FFEDIT is built as a first-class
environment in phase 1.

### Verb aliases resolve to a canonical verb (Requirement 6a)

An environment matches a typed surface form to a CANONICAL verb via a
per-environment alias table BEFORE dispatch; the handler only ever sees the
canonical verb, so aliases are behaviour-neutral. The existing one-behaviour
aliases (EXCLUDE/X, SHOW/INCLUDE) ARE this mechanism. The CANONICAL verb is what
is recorded/persisted/used internally (history, macro + menu `command=` values,
keybindings, session descriptors); an alias is command-line input only (type
`CHERCHER`, store `FIND`), so persisted data and macros do not break across
locales. The table is matched case-insensitively (B062), rejects collisions at
load (Shortcut_Registry-style conflict detection), and respects the
prelude/shadowing precedence. Phase 1 builds this alias-resolution MECHANISM with
the existing English aliases; per-locale alias DATA loads into the SAME tables
under CR-NR-103 (localization), adding no new mechanism -- localization extends to
verbs with no behaviour impact.

For split/detached, the focused region/window's Context is the source of truth
(`tab_manager.rs` `focused_group` / `active_tab`; detached via the
`update_keys.rs` path), per Requirement 2.2.

## The router -- one front door generalised

### Today (B080)

`dispatch_command_string` (`crates/ff-desktop/src/shell/dispatch.rs`): prelude
(`run_command_prelude`) -> `resolve_target` (FFCMD) -> `run_command_ladder` (which
TODAY still contains `try_commands_b2`, the editor verbs) -> engine terminal.

### Proposed (minimal insertion)

```
dispatch_command_string(raw):
    upper = raw.trim().to_uppercase()
    if run_command_prelude(raw, upper): return      // UNCHANGED: stage1, EXIT, POM, chained -- FIRST, unshadowable
    // [deferred address-prefix seam -- parse_address_prefix -- NOT implemented in phase 1]
    // NOTE (E1 correction): the active-environment claim is NOT here in the
    // front door -- it lives in the SHARED ladder path (`run_command_ladder`,
    // called by `handle_command`), at the precedence point the migrated editor
    // arms occupied. See "Placement correction" below.
    if let Ok(t) = resolve_target(raw, &ShellTargetResolver):  // FFCMD base (UNCHANGED)
        dispatch_command_target(t); return
    run_command_ladder(raw, upper)                    // shrinking fallback -> engine terminal
```

Properties (each a reconciliation): ONE front door (the active-env step is a
resolver stage inserted into the same chain, exactly as `builtin_workspace_target`
was in B080 Step 2); FFCMD is still `resolve_target`; the prelude keeps its
precedence FIRST; the ladder is the transitional fallback that shrinks as FFEDIT
verbs migrate out.

The ONLY new ordering decision is "Active_Environment before FFCMD" -- the ISPF/
REXX active-wins rule (Requirement 5.1).

### Placement correction (E1): the active-env claim lives in the SHARED ladder path

An earlier draft placed the active-environment step in the FRONT DOOR
(`dispatch_command_string`) between the prelude and `resolve_target`. E1
implementation proved that WRONG: `handle_command` is called DIRECTLY by many
internal callers (chained-fastpath segments, the AUTONUM->NUMBER redirect, menu
option dispatch in `commands_menu.rs`, `nav_stack`, menu buttons in
`render_chrome`, and dozens of tests), all of which BYPASS the front door. The
migrated editor verbs previously lived in the ladder (which `handle_command`
runs), so putting FFEDIT only in the front door made them UNREACHABLE from those
callers -- a real regression (3 tests failed).

CORRECT placement: the active-environment claim lives in the SHARED ladder path
(`run_command_ladder`, invoked by `handle_command`), at the SAME precedence point
the migrated arms occupied (the start of the former `try_commands_b2`), gated by
the active environment being FFEDIT. Every caller -- the front door AND every
direct `handle_command` caller -- reaches FFEDIT identically to how it reached the
former ladder arms, preserving reachability and precedence exactly (Req 4.1,
6.2a). This is why FFEDIT, like `resolve_target`/FFCMD, is a stage of the ONE
shared dispatch path, not a front-door-only step.

### Ordering correction (E8): no "exit family"; environment-before-FFCMD; `=` is universal

A dispatch-ordering defect was found after E1-E7 (owner report: "when in the
editor X must work as exclude not as return or exit; the relevant command
environment must be checked before FFCMD takes it ... the command handler should
take the command line and pass it first to the relevant Command Environment, if
rejected then to FFCMD"). The handler was special-casing a set of verbs (`EXIT` /
`QUIT` / `LOGOFF` / `X` / `=X`) in `run_command_prelude` and as the first arm of
`try_commands_a`, BOTH running BEFORE the active-environment claim. So bare `X` on
an editor Context closed/exited before FFEDIT could map it to EXCLUDE.

CORRECTED MODEL (requirements revision, Req 2a.4/2a.5, 3.1/3.2/3.2a, 4.1, 5.1-5.3).
Two principles, both from the owner:

1. THE HANDLER DOES NOT SPECIAL-CASE VERBS. There is NO "exit family". The handler
   offers the command string to resolvers in a fixed order; each CLAIMS or REJECTS.
   `X` / `RETURN` / `EXIT` / `QUIT` / `LOGOFF` are ordinary FFCMD verbs that FFCMD
   RECEIVES when the active environment rejects them. The close-the-application
   effect of `EXIT` may be performed by an UPPER layer that FFCMD calls, but FFCMD
   is the resolver that receives the string -- not a privileged handler branch.
   (So FFCMD still "gets" `x` / `return` / `exit`; it simply gets them AFTER the
   active environment has declined.)

2. `=` IS A SINGLE UNIVERSAL RULE, handled in ONE place BEFORE the environment.
   When `=` is the first character, the handler drops the Navigation_Stack (return
   to POM top) and dispatches the REMAINDER as an FFCMD/base command from that top.
   The environment NEVER receives a `=`-prefixed string. So `=X` in the editor is
   RETURN/EXIT (drop to POM, run `X` as the base verb), by the `=` rule -- NOT by
   any accident of FFEDIT's token matching. Only the bare `X` is environment-
   sensitive.

Dispatch order (one front door, one shared ladder path, no second dispatcher):

```
dispatch(raw):
  record history                                   # universal
  if raw starts with '=':                           # universal = rule (ONE place)
      drop nav ladder; dispatch(rest-from-POM-top); return
  if menu-context fastpath claims raw: return       # inert off a menu Context
  if active_env != FFCMD and active_env.claim(raw): return   # ENVIRONMENT FIRST
  # FFCMD base (receives whatever the environment rejected):
  if try_commands_a(raw): return                    # X/RETURN/EXIT/QUIT/LOGOFF live here
  if try_commands_b1(raw): return                   # CONFIG/FILES/SEARCH/... workbench verbs
  if THEME / chained / NAME / menu-name claims raw: return
  if resolve_target(raw): dispatch_target; return
  engine terminal
```

CODE MOVE (behaviour-preserving except the intended editor-`X` correction):
  1. Add the universal `=` step at the very top of the shared path (strip + redispatch
     the remainder from the POM base), BEFORE the active-environment claim. (This
     generalises today's `=`-origin handling, which is currently entangled in the
     prelude's chained/EXIT handling, into one explicit universal step.)
  2. Move the active-environment (FFEDIT) claim so it runs BEFORE `try_commands_a`
     (today the FFEDIT gate sits after `try_commands_a`/`try_commands_b1`; it moves
     ahead of them so the environment gets first crack -- see "FFEDIT claim position"
     below).
  3. Remove the verb special-casing from `run_command_prelude`; let `X`/`EXIT`/etc.
     be received by `try_commands_a` as ordinary FFCMD verbs AFTER the env claim.

FFEDIT claim position: by the E1 correction the claim must be in the SHARED ladder
path (reached by every `handle_command` caller). To honour "environment before
FFCMD" it moves to the START of `run_command_ladder`, BEFORE `try_commands_a`
(gated by the active env being FFEDIT). PRECEDENCE SAFETY (non-editor): on a
non-editor Context the gate is false, so `try_commands_a`/`try_commands_b1` run in
their current order -- identical to today. On an editor Context the FFEDIT verbs
(incl. bare `X` -> EXCLUDE) are claimed before `try_commands_a`'s `X` is reached.

Result: editor bare `X` -> FFEDIT EXCLUDE; editor `=X` -> `=` rule -> FFCMD `X`
(return/exit); non-editor bare `X` and `=X` -> FFCMD, unchanged; POM `X` -> the
menu Option_Key fastpath still claims it first (universal, inert off a menu).

INTERACTION with the existing `=`-prefixed menu fastpaths (implementer caution).
Today `=` is NOT handled in one place: the prelude's POM fastpath
(`resolve_pom_option_key`) and chained fastpath (`try_chained_fastpath`) already
interpret `=0`, `=0.K`, `=0;E.T`, and `try_commands_b1` has `=FILES`. The new
universal `=` step MUST be reconciled with these, NOT layered on top of them, or
`=` would be processed twice. The correct shape: the universal `=` step IS the
single owner of "leading `=` means drop-ladder-then-run-the-rest-from-POM"; the
existing menu/chained fastpaths become consumers of the already-stripped,
already-at-POM remainder (e.g. `=0.K` -> drop ladder -> run `0.K` as a chained
menu path from POM). This is a REFACTOR of where `=` is interpreted, folded into
E8, and it must preserve every existing `=0*` / `=FILES` behaviour (covered by the
existing menu-workspace / fastpath tests). If that reconciliation proves larger
than E8 should carry, split it: keep E8 to "environment-before-FFCMD + bare-`X`
correction" and gate the `=`-unification as its own slice (E8b), with E8 using the
narrow rule "a `=`-prefixed string is not offered to the active environment"
(enough to make `=X` reach FFCMD) while leaving today's `=`-fastpath code where it
is. DECISION for this gate: implement the NARROW rule in E8 (environment skips
`=`-prefixed strings) and record the full `=`-unification as E8b, so E8 stays
behaviour-preserving and small; the owner's "`=` handled in one place" ideal is
E8b.

### Editor-buffer verb ownership (E9) -- Requirement 10

By the ownership principle (Req 2a.4), the buffer verbs SAVE / CANCEL / UNDO /
REDO are FFEDIT's, not FFCMD's: they act on the active editor buffer, which has no
meaning in FFCMD's workbench context. E1-E5 migrated the search/filter/profile
families but LEFT these in FFCMD (SAVE resolves via `resolve_target` to a Function
target `file.save`; UNDO/REDO/CANCEL via the shell ladder / Function targets). E9
migrates them into FFEDIT, behaviour-preserving:

- Add SAVE / CANCEL / UNDO / REDO (identity entries) to the FFEDIT alias table and
  add their arms to `ffedit_claim` (/ `dispatch_ffedit.rs`), delegating to the SAME
  underlying operations the shell path calls today (the file-save, the undo/redo
  transaction manager, the discard-changes op) so the observable effect is
  unchanged -- only the ownership/routing moves ahead of FFCMD.
- Because the FFEDIT claim now runs BEFORE `try_commands_a`/`resolve_target` (E8),
  SAVE on an editor Context is claimed by FFEDIT; on a non-editor Context FFEDIT
  rejects and FFCMD handles it as today (and FFCMD has no buffer SAVE, Req 2a.5 --
  so a non-editor SAVE resolves exactly as it does now, which for most non-editing
  Contexts is unresolved/no-op).
- END / RETURN are NOT migrated: they are navigation (pop the Navigation_Stack),
  meaningful everywhere, and stay in the universal/FFCMD navigation handling.
- Red-before-green per verb: a test that on an editor Context the FFEDIT verb's
  result equals today's (file written / undo applied / changes discarded).

### Per-Context SAVE + chaining (E10) -- Requirement 11

Per-Context SAVE: each editing Context (Theme / Menus / Keys / Kinds / Config)
owns a SAVE in ITS environment that persists its working copy. Today these
Contexts have Save ACTIONS wired to buttons (e.g. `ThemeEditorAction::Save`,
`MenusEditorAction`, the kinds/keys/config equivalents) applied by the shell's
`apply_<panel>_action`. E10 gives each a command-line SAVE that invokes the SAME
action (command parity, Req 11.2) -- not a second save path. Mechanically this is
either (a) a small per-Context environment that claims SAVE and stashes the
Context's save action for the shell to apply (the `pending_action` pattern,
wiring-standard.md), or (b) the active-env derivation maps each editing kind to a
"<Context>Env" whose `claim` handles SAVE. Phase chooses (a) -- minimal, reuses the
existing action+apply pattern; (b) is the fuller "make each environment real"
option deferred with FFNAV. No global SAVE, no FFCMD SAVE (Req 2a.5).

Chaining (Req 11.3-11.8): a quote-aware splitter divides the submitted string on
top-level `;` (ignoring `;` inside quotes), then dispatches each segment through
the ONE existing front door in order. It is a PRE-SPLIT feeding the single front
door once per segment -- NOT a new dispatcher. The active environment is
re-derived per segment (so `...; =0` returning to POM makes a later segment resolve
against FFCMD). `=` scopes to its own segment (E8b's universal `=` runs per
re-dispatched segment). Error policy: continue-on-error (best-effort), each
segment's status surfaced; no stop modifier in this slice. NOTE: today's
`try_chained_fastpath` chains ONLY menu-option paths (`=0;E.T`); E10 generalises
chaining to ALL command strings. The two must be reconciled so a chain is split
ONCE at the top (not re-split by the menu fastpath); the menu fastpath becomes a
per-segment consumer, mirroring the `=`-unification shape in E8b.

### FFEDIT CUA verbs + keyboard->command (CR-CH-054) -- Requirement 12 (+ command-framework Req 17)

Two coupled pieces: (A) keyboard chords resolve to command strings through the
one front door; (B) FFEDIT owns the CUA editing verbs those keys (and typing)
dispatch to.

(A) KEYBOARD -> COMMAND (command-framework Req 17). Today the Ctrl/Alt editing
keys are scattered inline `ctx.input` checks (editor Ctrl+Z in
`editor_panel/input.rs`, editor Ctrl+C in `editor_panel/paint.rs`, shell Ctrl+S /
Ctrl+Shift+P / Ctrl+Shift+F in `shell/update.rs`) that act directly, bypassing
the Shortcut_Registry. The design routes a bound chord -> its command string ->
`dispatch_command_string` -> the active environment, exactly like typing. The
Reserved_CUA_Set (Ctrl+Z/Y/C/X/V/A/S) is bound to UNDO/REDO/COPY/CUT/PASTE/
SELECT ALL/SAVE and is not user-overridable (reserved, Req 5.3); non-reserved
Ctrl/Alt chords are configurable via the existing keys editor / TOML (Req 5.6;
the keys-editor Ctrl/Alt columns already exist). egui's own `TextEdit` keeps its
native editing inside text widgets (the Command Field); the Reserved_CUA_Set
targets the FFEDIT BUFFER only when the custom editor buffer is focused.
The Ctrl+S point-fix (dispatch SAVE instead of direct `save_active_tab`) landed
ahead of the full slice to close the key/verb divergence from E9 13.1.

(B) FFEDIT CUA VERBS (Requirement 12). COPY/CUT/PASTE/SELECT-ALL/UNDO/REDO are
FFEDIT verbs reading context inputs, NOT explicit area params:
- COPY/CUT SOURCE: Cursor_Context selection (command-framework Req 12) first;
  else the pending FFLINE `C`/`CC` line block (whole lines); else no-op. CUT also
  deletes (dirty + undo). The `C`/`CC` block is the CLIPBOARD source (distinct
  from ISPF in-document `C...A/B`); markers cleared after.
- PASTE DESTINATION: cursor-in-editing-space (insert at cursor) first; else the
  pending FFLINE `A`/`B` marker (insert clipboard as WHOLE LINES after/before,
  always line-granular); cursor wins the tie-break; `A`/`B` cleared after, clipboard
  retained.
- CLIPBOARD backend: the `ff-clipboard` crate (wires in a currently-orphan crate).
- SELECT ALL sets the whole-buffer Cursor_Context selection.
- UNDO extracts the existing inline Ctrl+Z logic into a verb; REDO is NEW (no redo
  stack today) -- built in slice 2, coordinating with undo-redo-transactions.

FFLINE -> FFEDIT coupling: the line-command layer must EXPOSE accessors for the
pending `C`/`CC` source block and the pending `A`/`B` destination marker so the
FFEDIT verbs can read them. The markers are context inputs (same category as the
cursor selection), so this is not a layering violation -- it is one small, named
read-only coupling, documented here.

Slices: (1) chord->command dispatch wiring + SAVE (done) + UNDO-extract; (2) the
new buffer commands (CUT/PASTE/SELECT-ALL + COPY unification) + REDO + ff-clipboard
+ the FFLINE marker accessors. No second dispatcher; chord -> command string ->
the one front door; `CommandTarget`/`resolve_target`/nav stack unchanged.

## Addressing

### Macros (phase 1)

Reconcile with lua-macro-engine Req 11.11-11.14 (which ALREADY specify
`ADDRESS <env>` + TSO/ISPEXEC/ISREDIT). The Lua `address(<env>)` binding routes
through the Scripting_Bridge (`crates/ff-command/src/scripting.rs`,
command-framework Req 6) into the named FFWB environment via the Alias_Map:
`ADDRESS TSO` -> FFCMD, `ADDRESS ISREDIT` -> FFEDIT, `ADDRESS FFCMD`/`ADDRESS
FFEDIT` the native names. One environment set, documented alias map, no second
vocabulary. The dispatch outcome's return code maps to `RC` (Req 11.15).

### Interactive prefix (deferred)

NOT built in phase 1 (Requirement 8.4). The `parse_address_prefix` seam is marked
in the router for a later additive change (recommended future syntax `ENV: cmd`),
but phase 1 writes no interactive-prefix code and no criterion requires it.

## Migration shape (E0-E7, design level)

- **E0 scaffold** -- add the `CommandEnvironment` trait, the Environment_Registry,
  the active-env derivation, and the front-door insertion as PURE INDIRECTION
  (empty FFEDIT `dispatch` returning None; no address prefix). Behaviour
  unchanged; prove no shell test regresses.
- **E1 navigation** -- LOCATE/TOP/BOTTOM/UP/DOWN/LEFT/RIGHT/SORT into FFEDIT via
  `nav_manager` (incl. `up_by_amount`/`down_by_amount` vs `scroll_amount` for bare
  UP/DOWN, CR-NR-087); delete those arms from `try_commands_b2`.
- **E2 exclude/show** -- EXCLUDE/X [ALL], SHOW/INCLUDE [ALL], RESET variants via
  `exclude_manager` with the same snapshot closure; the ALL/text sub-parse moves
  inside the one env entry.
- **E3 find** -- FIND/RFIND/CHANGE/RCHANGE via `find_manager`; CHANGE's
  `parse_two_args` quoting and the B062 case-preserved term carried verbatim.
- **E4 profile** -- CAPS/NULLS/STATS/LOCK/PROFILE/HILITE via `edit_profile`.
- **E5 scroll** -- SCROLL <amt> via `scroll_amount`.
- **E6 FFLINE** -- naming/model only; NO code (intake stays prefix-area ->
  Command_Engine).
- **E7 retire** -- once `try_commands_b2` is emptied, delete it and drop the call
  from `run_command_ladder`. Composes with (does not block) B080 Step 7.

Each code step is behaviour-preserving, scoped-test-green, with a full-shell/unit
test per criterion (red before green), and keeps both the env entry and the
ladder arm reachable until the env entry is test-proven (B080's per-step
rollback discipline).

## Non-goals

- NO second navigation stack (per-tab Navigation_Stack, command-framework Req 10,
  untouched; environments route COMMANDS, they do not navigate).
- NO parallel dispatcher (the active-env step is inside the one front door).
- NO change to `CommandTarget` (the editor verbs do NOT become new variants; that
  is exactly why B080 could not migrate them).
- Does NOT implement CR-CH-052 (`=` / X / =X ladder semantics); that composes on
  the same front door independently.
- Does NOT design the database tool or any future environment; FFLINE is
  named-only; the interactive address prefix is deferred.

## Spec-ownership split (per the design proposal)

This sub-project is the authoritative home for the model (Requirements 1-5, 7-9).
Small cross-reference deltas belong in: `command-framework` (the front door now
consults the active environment before `resolve_target`), `command-semantics`
(FFEDIT vocabulary + manager delegation; FFLINE intake), `lua-macro-engine` (the
ADDRESS alias map on Req 11.11-11.14). None of those adds a parallel mechanism.
