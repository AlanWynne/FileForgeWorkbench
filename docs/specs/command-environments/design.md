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

### An environment must have a Context that makes it active (no topical environments)

A Command Environment is a DISPATCH SCOPE tied to a focused Context, not a
topical grouping of related verbs. The dispatch contract is: the Active_Environment
(derived from the focused Context's kind) gets first crack; whatever it REJECTS
falls back to FFCMD, the always-present base (Requirement 5.1, and the E8 ordering
correction). Therefore an environment is only reachable through normal dispatch if
some focused Context declares it as its Active_Environment. An environment that no
kind ever declares could be reached ONLY by explicit addressing (`ADDRESS <name>
...`), never by the fallback chain -- so for everyday verbs it would be dead.

Consequence (owner decision, this CR): verbs that are issued FROM WITHIN whatever
Context the user is in -- and have no dedicated focused Context of their own --
belong in FFCMD, the base reachable from everywhere by fallback. This explicitly
RULES OUT standing up topical environments for them:

- Window / tab / session management -- SPLIT, DETACH, UNSPLIT, FOCUS, DOCK, SWAP,
  END, RETURN, WORKSPACE (OPEN/SAVE/CLOSE/ADD ROOT), CLOSE -- are FFCMD verbs.
  There is no "window Context" that is focused to make a hypothetical FFWIN
  active, so an FFWIN would never be invoked by fallback; these are FFCMD.
- Screen capture -- SNAPSHOT, CAPTURE -- are FFCMD verbs for the same reason: no
  "capture Context" is ever focused, so there is no FFSCRM; they are global
  commands reached from wherever the user is.
- Global/utility verbs -- HELP, PFSHOW, TIME, RETRIEVE, RESET BARE -- are FFCMD.
- Editor-numbering AUTONUM is FFEDIT (the editor Context makes FFEDIT active),
  consistent with the Step-4 editor-verb family.

A subsystem environment (e.g. a future FFJES, FFSQL) is justified ONLY when a
focused Context exists for it (a job-monitor Context, a Database Context) that
makes the environment active while the user is in it. Until that Context exists,
the subsystem's verbs reached from elsewhere go through FFCMD. This is a VISION
note here, not a phase-1 deliverable.

### Layered verbs: one surface name, per-environment front-ends delegating to a backend

Because the same verb NAME may be owned by several environments (the FIND example
above), a verb can present a context-appropriate FRONT END in each environment
while a single lower-layer SERVICE does the real work. Active-wins selects which
front end runs; each front end translates its context into a call to the shared
backend. This is the REXX ADDRESS model applied to a service with both an
implicit ("here") and an explicit ("there") form.

Worked example -- SUBMIT (VISION; the FFJES backend does not exist in phase 1):

- `FFJES.SUBMIT` -- the real implementation, owning the job submission, active
  while a focused job-monitor Context exists.
- `FFEDIT.SUBMIT` -- the editor's front end: submits the CURRENT buffer/member
  (implicit "here"), delegating to the FFJES backend. Active-wins means that in an
  editor Context, bare SUBMIT is this one.
- `FFCMD.SUBMIT <dsname>` -- the base front end: takes an EXPLICIT dataset
  argument (the "there" form), reachable from anywhere by fallback, delegating to
  the same FFJES backend.

Each front end is a thin environment-specific resolver+translator; none
re-implements the service. This keeps a verb's dialects consistent (one backend,
one observable effect per target) while letting each environment express the
natural local form. The alias/registry model already supports it: the same
surface form resolves to a canonical verb PER ENVIRONMENT (Requirement 6a), so
`SUBMIT` can canonicalise and dispatch differently in FFEDIT vs FFCMD without a
collision (collisions are rejected only WITHIN one environment, per the
alias-overlay model). Phase 1 builds none of the FFJES pieces; this records the
principle so FFCMD / FFEDIT are built with the delegation shape in mind and a
future FFJES slots in without reshaping the model.

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
- NO topical environments (no FFWIN, no FFSCRM): an environment must have a
  focused Context that makes it active, else it is unreachable by fallback.
  Window / session / capture / global verbs are FFCMD members (see "An environment
  must have a Context that makes it active"). Subsystem environments (FFJES,
  FFSQL) are justified only by a future focused Context and are vision-only; the
  layered-verb delegation principle (SUBMIT front-ends -> shared backend) is
  recorded as design guidance, not a phase-1 deliverable or a new criterion.

## Spec-ownership split (per the design proposal)

This sub-project is the authoritative home for the model (Requirements 1-5, 7-9).
Small cross-reference deltas belong in: `command-framework` (the front door now
consults the active environment before `resolve_target`), `command-semantics`
(FFEDIT vocabulary + manager delegation; FFLINE intake), `lua-macro-engine` (the
ADDRESS alias map on Req 11.11-11.14). None of those adds a parallel mechanism.

## Phase-1 maturation: built Environment_Registry, address-by-name, owning-environment binding, FS-CE family

This section APPENDS the design for Requirements 13-17 (the owner-approved
DESIGN-BRIEF Phase-1 core change: `.agents/tasks/mainframe-dataset-emulation/DESIGN-BRIEF.md`,
section 5). It does NOT contradict any decision above: the CommandEnvironment
trait, FFCMD == `resolve_target`, the kind-supplied active-env derivation, the one
front-door / shared-ladder router, and Req 9.4 "a new executable environment is a
PLUGIN capability" all stand. This is the generalisation the existing
"The Environment_Registry" section already describes (jobs 1-3) made REAL, plus
the minimal editor binding to use it.

### From the closed set to a built registry (Req 13)

Today the environment set is CLOSED: a fixed `EnvironmentKind {FfEdit, FfNav,
FfCmd}` enum, a hardcoded `environment_for_kind(BuiltinKind::from_tab_kind(..))`
match, a single hardcoded `== FfEdit` claim gate in the shared ladder path, no
registry of `dyn CommandEnvironment`, and FFEDIT implemented as `ffedit_claim`
methods on the `WorkbenchShell` god-struct (the documented E0 variance). The
maturation replaces that closed set with the BUILT registry the design above
already names:

- The registry becomes a shell-owned collection of `Box<dyn CommandEnvironment>`.
  Environments REGISTER into it: FFCMD (the implicit base = `resolve_target`),
  FFEDIT (now a real object, not shell methods), and the host FS environment
  (Req 16) at startup; future executable environments register as PLUGINS
  (Req 9.4).
- Active-env derivation (the "Active-environment derivation" section above) and
  the active-wins claim gate (Req 5.1) READ FROM the registry: the handler asks
  the focused kind for its environment NAME and the registry resolves that name to
  the registered environment, defaulting to FFCMD when the name is absent or
  unknown. The former hardcoded `environment_for_kind` match and `== FfEdit`
  literal are removed.
- FFEDIT-as-object: the verb bodies stay delegations to the existing managers
  (`nav_manager`, `exclude_manager`, `find_manager`, `edit_profile`,
  `scroll_amount`) and the dirty-aware SAVE path; the object is the trait wrapper
  the E0 design already anticipated as "the cleaner borrowed-manager-struct form
  as the eventual refactor". With only FFEDIT + FFCMD + the host FS env registered,
  behaviour is identical (Req 13.6).

This is additive and behaviour-preserving; it is NOT a second dispatcher (the
registry feeds the one shared path, Req 1.4 / 3.3) and changes neither
`CommandTarget` nor the per-tab Navigation_Stack.

### Address-by-name: dispatch_to_environment, with FFEDIT SAVE forwarding and the deferred macro ADDRESS as its two consumers (Req 14)

The registry gains ONE address-by-name entry point, `dispatch_to_environment(name,
raw)` -- the REXX ADDRESS pattern applied INTERNALLY. It is the SAME seam the
deferred macro ADDRESS (Req 8 / Task 10) needs; the two consumers are:

1. FFEDIT SAVE forwarding (built in this phase, the first real consumer). FFEDIT
   splits its verbs into IN-BUFFER (handled directly: LOCATE / FIND /
   CHANGE-in-buffer / CAPS / SORT / EXCLUDE / NUMBER / UNNUM / BNDS / COLS and the
   rest of Req 6.1) and STORE-AFFECTING (SAVE today; future CREATE / REPLACE member
   / save-time validation). For a store verb, FFEDIT calls
   `dispatch_to_environment(owning_env, raw)` instead of writing the store itself.
2. The macro `ADDRESS <env>` / Lua `address(<env>)` binding (Task 10, still
   deferred) routes through the SAME entry point via the Alias_Map (TSO -> FFCMD,
   ISREDIT -> FFEDIT). This retires the review's worry that the address seam would
   "rot": FFEDIT SAVE forwarding exercises it now; the macro path reuses it later.

The dispatch outcome carries a return code (Req 8.3) convertible to macro `RC`.

### SAVE routing as a behaviour change to Requirement 10.1 (native default preserving today)

This is the one place behaviour changes, and it is flagged explicitly. Today
FFEDIT SAVE calls `tab_manager::save_active_tab` which byte-writes via the
`LocalFsProvider`. After this change, FFEDIT ADDRESSes SAVE to the tab's
Owning_Environment, whose SAVE performs the write. The dirty-awareness contract of
Req 10.1 (clean no-op; dirty write + stay + clear flag + save point; write-fail
stay + error; not Confirmable) is PRESERVED verbatim -- only the EXECUTOR moves
from FFEDIT to the Owning_Environment. Because the DEFAULT Owning_Environment is
the host FS environment (Req 16) whose SAVE is byte-identical to today's
`save_active_tab` logic (that logic MOVES into the host FS CE), a native file's
SAVE is on-disk-identical and dirty/save-point-identical to today. The RECFM /
LRECL / record knowledge therefore lives ONCE, in each FS CE, never duplicated
into an editor-side profile (DESIGN-BRIEF D2).

### The tab -> owning-environment binding and open-time capture (Req 15)

A `TabState` gains an `owning_environment` (an environment name / reference). It is
CAPTURED AT OPEN from the originating catalog / provider -- the `CatalogType` the
navigator already knows at open and discards today -- threaded through the open
command via a new `file.open` `CommandParams` entry. When no origin is supplied (a
plain host-path open), it DEFAULTS to the host FS environment, so every existing
open is behaviour-preserving. FFEDIT reads this field to choose the SAVE target
(Req 14.4). Persistence reuses the existing `WorkspaceDescriptor` model: a tab
reopened from its descriptor recaptures its Owning_Environment from its origin; no
new persistence format is added (framework-conformance mechanism 6).

### The ff-ce-* family + ff-ce-host-fs decider + the native ROLE (Req 16)

Each file system is its OWN Command Environment named by the `ff-ce-*` convention
(`ff-ce-ntfs`, `ff-ce-posix`, future `ff-ce-apfs`; the mainframe CE housed in
`ff-idcams`), owning its store semantics. `ff-ce-host-fs` is a DECIDER (not a file
system): at startup it detects the host and resolves the "native" ROLE to the
matching concrete FS CE (Windows -> ntfs; Linux / macOS -> posix / future apfs).
Whatever it resolves to IS the Host_FS_Environment and the default
Owning_Environment. "Native" is a ROLE, not a CE; any FS CE is emulatable in a
non-native context because the CE owns the semantics, not the platform. Initial
build: the host-fs decider + light ntfs + light posix (OS-backed controls/attrs,
byte-write SAVE == today, cheap FS defaults) + the mainframe CE in ff-idcams (the
first genuinely diverging CE: records, RECFM / LRECL, DEFINE / REPRO / LISTCAT).
Deep NTFS / POSIX / APFS semantic emulation and cross-emulation are DEFERRED
(enabled by the abstraction, not built now). This reconciles with Req 9.4: a new
executable FS CE is a plugin capability under the plugin permission model.

#### Interactive vs backend Command Environments (Task 21 implementation decision)

The `ff-ce-*` family revealed that "Command Environment" names two different
roles that must not share one trait:

- An INTERACTIVE environment (FFCMD, FFEDIT, future FFNAV / FFLINE) claims a typed
  command against shell-entangled state. Its verbs mutate the editor buffer, the
  navigation managers, the exclude / find managers -- things only the shell owns.
  This is the existing `CommandEnvironment` trait, whose FFEDIT implementor claims
  via `WorkbenchShell::ffedit_claim` (the documented variance). It needs the whole
  shell.

- A BACKEND environment (`ff-ce-ntfs`, `ff-ce-posix`, the resolved host-fs pick,
  the future mainframe CE) is ADDRESSING-ONLY: it owns a file system's STORE
  semantics (write-back, case rules) and is ADDRESSed by name for an effect such
  as SAVE. It must NOT depend on `WorkbenchShell` -- it is lighter from FFWB's
  side (no shell coupling) yet potentially richer on the backend side (records,
  RECFM, catalog). Forcing it through `CommandEnvironment` would drag the shell
  into a crate that has no business knowing about it.

So the backend role gets its OWN narrow trait, `ff_vfs::BackendEnvironment`
(`name`, `is_case_sensitive`, `save(path, bytes) -> io::Result`), living in
`ff-vfs` rather than a new `ff-ce-core` hub crate (a hub was rejected -- it would
need per-backend maintenance and nobody needs to hold a list of all backends).
Backends register BY NAME into the open `Environment_Registry`; the registry
holds the one resolved host-fs backend as `Box<dyn BackendEnvironment>`. FFEDIT's
SAVE reroute (Req 14 / Task 20) delegates ONLY the physical write to the owning
backend's `save`; the dirty-aware orchestration (read buffer, clear flag, save
point) stays shell-side, which is why native SAVE remains byte-identical. The
light ntfs / posix backends differ only in the cheap case default; no deep
per-FS semantics ship, and the decider resolves only the NATIVE role (no
cross-emulation wired).

### Live-provider-registry prerequisite (Req 17)

The `ff-vfs` `ProviderRegistry` is built and tested but NOT registered live in the
running shell today. Registering it at startup is ADDITIVE shell wiring (no type
reshape) and is the single most important prerequisite for the non-host parts of
Req 14-16: without it, a plugin-provided `VfsProvider` (e.g. the mainframe VFS
provider) has nowhere to land, so a non-host Owning_Environment cannot reach its
store. The host-FS default path does NOT depend on it (native editing works
regardless). This is called out as its own requirement so the dependency is
explicit, not silent.

### Vertical-slice phasing (keeps FFWB building between tasks)

The maturation lands as a vertical slice that de-risks the whole model while
keeping the app building and native behaviour unchanged at every step:

1. Build the Environment_Registry + make FFEDIT an object (behaviour-preserving;
   only FFEDIT + FFCMD registered).
2. Add `dispatch_to_environment` (address-by-name) with no consumer behaviour
   change yet.
3. Add the `TabState` owning-environment field + capture at open + thread
   `file.open` (default host FS env) -- still native-identical.
4. Redirect FFEDIT SAVE to address the owning env (host-FS SAVE == today).
5. Register the host-fs decider + light ntfs + posix CEs.
6. Register the live `ProviderRegistry` (prerequisite for later non-host phases).

Phases 2-5 of the DESIGN-BRIEF (mainframe CE, deferred navigator arms, universal
seq-number/bounds wiring, concrete VSAM service) build ON this slice and are NOT
part of this requirements-gate extension.

### Framework-conformance stance (restated for Req 13-17)

This completes CR-CH-053's deferred design: registry jobs 1-3 (built collection +
address-by-name + FFCMD base), the Req 8 addressing seam, and the Req 9.4 plugin
environments. It adds NO second dispatcher and NO second navigation stack, leaves
`CommandTarget` / `WorkspaceContext` / `InteriorFocus` / `WorkspaceDescriptor`
unchanged, and is the single owner-APPROVED framework extension of this stream.
The mainframe-dataset-emulation (`V`) plugin stream builds its FS CEs and VFS
providers ON this mainline core change.

### Spec-ownership split for the maturation

This sub-project remains the authoritative home for Req 13-17. Cross-reference
deltas that other sub-projects will carry (NOT authored here; recorded for the
later gate steps): `virtual-file-system` (live provider registration + the
owning-CE store seam), `multi-tab-editor` / `edit-operations` (the TabState
owning-environment field + FFEDIT SAVE routing), `dataset-catalog` (the mainframe
CE SAVE semantics). None adds a parallel mechanism.

## Record-aware BackendEnvironment store contract (CR-CH-060)

This section designs the record-aware `ff-vfs::BackendEnvironment` store contract
(Requirement 18). It is an OWNER-DIRECTED FRAMEWORK CHANGE to a load-bearing core
type, authored ONCE to serve BOTH consumers that need the same reshape:

- **CR-CH-058** (universal windowed record-oriented document model): its SAVE
  walk re-frames the Piece_List per the owning CE's `RecordFormat` and must hand
  the owning CE RECORDS (not a flat byte buffer) for Fixed/Variable documents.
- **CR-CH-059 RC.B.8 Part 2** (record-aware MAINFRAME editor SAVE): its
  prerequisite (a) is exactly this contract reshape so the mainframe CE can store
  through `ff_dscatalog::DatasetAccess::put` instead of a `(path, bytes)` write.

Designing it once avoids reshaping the SAVE seam twice and avoids a separate
mainframe "records -> store" packer duplicating CR-CH-058's piece-list re-framing.

### Today

```rust
trait BackendEnvironment: Send + Sync {
    fn name(&self) -> &str;
    fn is_case_sensitive(&self) -> bool;
    fn save(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()>;
}
```

`(path, bytes)` carries no DSN, no RECFM/LRECL/catalog identity, and no record
boundaries, so a mainframe backend cannot pack records per RECFM/LRECL through it
(the trait's own doc comment already acknowledges this limitation).

### The two shapes (OWNER DECISION: SHAPE 2 APPROVED)

> OWNER DECISION (gate review): **SHAPE 2 is APPROVED** as the owner-directed
> framework change -- add a record-aware store method ALONGSIDE the byte `save`,
> with a provided default that declines (not-record-capable); only record-aware
> CEs override. Shape 1 is REJECTED for this contract but retained below as the
> considered alternative. Illustrative names may be finalised at design discretion
> during implementation, keeping the approved semantics.

**Shape 1 -- change `save` to a single record-aware form.** One store method
whose parameter object expresses EITHER bytes (host) OR records (mainframe) plus
the target and attributes.
- Object-safety: achievable (still `&self` + concrete params), but the one method
  must branch internally for the bytes-only host case.
- Native byte-identical: AT RISK -- every host write now travels through the
  record-aware parameter shape, so the "nothing changed for native" guarantee is
  a re-proof rather than a construction fact, and the regression surface widens.
- Migration cost: HIGH -- all three existing impls (`ff-ce-host-fs`, `ff-ce-ntfs`,
  `ff-ce-posix`) and the single call site (`host_fs_save` ->
  `save_active_tab_via_backend`) change together.
- Consumption: CR-CH-058's Delimited path and the mainframe path share one method
  but branch inside it.

**Shape 2 -- add a record-aware method ALONGSIDE the byte `save` (OWNER-APPROVED).**
```rust
trait BackendEnvironment: Send + Sync {
    fn name(&self) -> &str;
    fn is_case_sensitive(&self) -> bool;
    fn save(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()>;      // UNCHANGED
    fn save_records(&self, target: &StoreTarget, records: &dyn RecordSource,
                    attrs: &RecordStoreAttrs) -> BackendStoreResult {        // NEW
        BackendStoreResult::NotRecordCapable                                  // provided default
    }
    fn record_capable(&self) -> bool { false }                               // mainframe CE overrides true
}
```
(Names ILLUSTRATIVE; finalised at owner approval.)
- `StoreTarget` carries dataset identity (DSN / catalog identity / owning-env),
  NOT a host path.
- `RecordSource` is an object-safe (`&dyn`) record stream the editor SAVE walk
  fills from the re-framed Piece_List.
- `RecordStoreAttrs` carries RECFM / LRECL / encoding -- sourced from the catalog
  attributes, the SAME values the CE used to build the `RecordFormat` at open.
- `BackendStoreResult` mirrors `BackendOutcome` (rc-carrying) so the addressing
  caller maps it to a status / macro RC.

Shape 2 is the OWNER-APPROVED choice because:
- **Object-safety is trivially preserved** (Req 18.4): both methods take `&self`
  and non-generic, non-`Self`-returning parameters, so `Box<dyn BackendEnvironment>`
  stays valid. The `provided` default keeps `dyn` dispatch and lets host impls
  compile untouched.
- **Native byte-identical is a construction guarantee** (Req 18.1): `save(path,
  bytes)` is literally unchanged, so native/Delimited SAVE is the SAME code path
  and the SAME bytes (Req 16.5, document-model Req 12.12). No re-proof needed.
- **Migration cost is low** (Req 18.3): host CEs inherit the default and need NO
  change; only the mainframe CE (ff-idcams) and the editor SAVE walk opt in.
- **Each consumer opts in cleanly** (Req 18.5): the SAVE walk picks the method by
  the owning CE's advertised `RecordFormat`.

The overlap report (`.agents/tasks/crch058-rcb8-overlap/report.md` section E)
also leans to "add a record-aware store entry alongside the byte save"; this
design validates that lean. The owner APPROVED Shape 2 at gate review; Shape 1 is
REJECTED for this contract but remains documented above as the considered
alternative with its tradeoffs.

### The records -> store dataflow (one seam, both consumers)

```
editor SAVE (FFEDIT verb) -- CR-CH-053 Task 20/21 owning-CE SAVE-addressing seam
  -> CR-CH-058 SAVE walk: walk the Piece_List, re-frame each piece per the owning
     CE's RecordFormat (Delimited -> bytes; Fixed/Variable -> records)
  -> owning CE advertises Delimited/host : BackendEnvironment::save(path, bytes)   [byte-identical]
     owning CE advertises Fixed/Variable : BackendEnvironment::save_records(target, records, attrs)
  -> mainframe CE (ff-idcams) implements save_records over
       ff_dscatalog::DatasetAccess::open -> put(record)* -> close
     (FixedCodec / VariableCodec / BinaryCodec frames the bytes on close;
      DatasetAccess surfaces x37 space-full abends as the store RC)
```

Single source of truth (Req 18.5 / document-model Req 11.2): the owning CE
supplies the `RecordFormat` at OPEN; the SAVE path never re-derives framing. The
mainframe CE converts the editor's framed records into `ff_dscatalog::Record`s
and drives `DatasetAccess::put`; it does not re-invent CR-CH-058's piece-list
re-framing.

### Registry opening is prereq wiring, not a new mechanism

Reaching the mainframe CE's `save_records` requires the closed
`EnvironmentRegistry` `enum RegisteredEnv { FfCmdBase, FfEdit, HostFsPlaceholder }`
+ single `host_fs` backend to open into a named-backend map, plus binding a
mainframe tab's `owning_env` and registering the mainframe VFS provider live
(Req 17.4). Those are RC.B.8 Part 2 prerequisites (b), (c), (d) -- shell/registry/
provider WIRING on the EXISTING `dispatch_to_environment` seam, NOT a new
framework mechanism and NOT part of CR-CH-060's contract criteria. CR-CH-060
owns ONLY the trait reshape (Req 18); the registry/binding/provider wiring and
the mainframe CE implementation are downstream tasks (see tasks.md).

### Designed ONCE; DAG unchanged

The record-aware trait stays in `ff-vfs` (where `BackendEnvironment` already
lives); the mainframe CE in `ff-idcams` implements it over `DatasetAccess`. The
dependency DAG is UNCHANGED and acyclic: `ff-idcams -> ff-dscatalog -> ff-volume
-> ff-vfs`. No new hub crate, no second dispatcher, no second navigation stack;
`CommandTarget` / `WorkspaceContext` / `InteriorFocus` / `WorkspaceDescriptor`
are untouched. This is the SINGLE owner-approved framework reshape serving
CR-CH-058 and CR-CH-059 RC.B.8 Part 2 together.

## MAINFRAME CE same-session store visibility (CR-CH-061) -- Requirement 19

Design delta for CR-CH-061. The record-aware store path (Req 18) is in place and
rides the single seam, but the mainframe CE was constructed with an in-memory
`CatalogDatasetAccess` that does not read the on-disk SQLite catalog the Files
Panel writes, so a same-session SAVE of a Files-Panel-allocated dataset resolves
a FALSE NotFound (rc 12). Root cause and option analysis:
`.agents/tasks/crch061-live-handles/findings.md` (Option c).

### The only change: which concrete `DatasetAccess` the shell builds

This CR introduces NO new command-environments mechanism. The mainframe CE
(`MainframeEnvironment` in `ff-idcams`) already owns `Arc<dyn DatasetAccess>` and
its `save_records` already does resolve -> open(Write) -> put -> close (Req 18.7).
The ONLY change at this layer is that the shell constructs the CE with the
catalog-backed `DatasetAccess` added under dataset-catalog Req 36 (which resolves
against the on-disk catalog of record) instead of the fresh in-memory fixture in
`build_mainframe_backend`. `MainframeEnvironment` is unchanged -- it is agnostic
to which concrete access it wraps; a catalog-backed access simply makes `resolve`
SUCCEED for a Files-Panel dataset, turning the rc-12 miss into rc 0.

### Catalog identity from the existing tab binding

The catalog repository path reaches the CE from the tab's EXISTING `(dsn, catalog)`
store identity (Req 15 owning-environment binding; `PendingOpenReq.identity` /
the tab `StoreTarget`), resolved to a repository path via the existing
`ff_catalog_registry::CatalogRegistry::get_by_name(catalog).path` lookup. No new
persistence field, no second identity channel (Req 19.3). Whether the catalog
path is fixed at CE build time or resolved per save is a dataset-catalog design
choice (see dataset-catalog design.md, Req 36) -- the per-save variant is
preferred so multiple mainframe catalogs are each served.

### Framework conformance

- Single SAVE seam preserved (Req 18.6 / 19.4): FFEDIT -> owning CE ->
  `save_records` -> `DatasetAccess`. No parallel save path, no second dispatcher.
- Native/host byte SAVE unchanged and byte-identical (Req 18.1 / 19.5); only a
  MAINFRAME-owned tab reaches the catalog-backed access.
- DAG unchanged and acyclic (`ff-idcams -> ff-dscatalog -> ff-volume -> ff-vfs`);
  the shell (ff-desktop) sits on top and threads the catalog path. No new edge.
- `CommandTarget` / `WorkspaceContext` / `InteriorFocus` / `WorkspaceDescriptor`
  are untouched.

## Environment_Registry root-command enumeration + user-alias-root load validation (CR-CH-067) -- Requirement 20

Design delta for CR-CH-067 (Gate 1: the FOUNDATION for the Gate 2 Alias Editor
redesign). Grounded in `.agents/tasks/command-env-enumeration/findings.md`. This
is an ADDITIVE, read-only query on the built `EnvironmentRegistry` (Req 13) plus a
load-time rejection branch mirroring the existing skip-with-WARN discipline. NO
new mechanism: it exposes data that already exists and reuses guards that already
run.

### Why a name-keyed REGISTRY query, not a trait method

The two phase-1 root sets already exist as enumerable data, in two DIFFERENT
shapes, in two places used today:

- **FFEDIT** -- `AliasTable::ffedit_english().canonical_verbs_owned()`
  (`shell/environment.rs`) yields the distinct canonical editor verbs (LOCATE,
  TOP, ..., FIND, CHANGE, ..., SCROLL, SAVE).
- **FFCMD** -- `builtin_ffcmd_roots()` (`shell/alias_reader.rs`) unions the two
  code-only built-in menus' option roots (`recovery_pom_menu` +
  `recovery_settings_menu`) with `BUILTIN_WORKSPACE_VERB_ROOTS`
  (`command_config/mod.rs`, pinned against the classifier by a drift-guard test).

The query lives on `EnvironmentRegistry` (`shell/environment_registry.rs`), keyed
by environment NAME, NOT on the `CommandEnvironment` trait, because:

1. the two root sets live in two different shapes the registry already bridges by
   name;
2. FFCMD is NOT a `CommandEnvironment` trait object (FFCMD == `resolve_target`,
   Req 1.2), so a trait-only method cannot cover it without a shim; the registry
   query covers all names uniformly;
3. the consumer (the Alias Editor) already works by NAME
   (`AliasEnvironment::env_name()` -> "FFCMD" / "FFEDIT"), so a name-keyed query is
   the natural, shell-decoupled seam.

### API shape

```rust
// crates/ff-desktop/src/shell/environment_registry.rs
impl EnvironmentRegistry {
    /// The canonical ROOT commands owned by the named environment, in a stable,
    /// de-duplicated order. Canonical roots ONLY -- NOT localised alias surfaces.
    /// Unknown / non-enumerable environment -> empty Vec (Req 20.1-20.3, 20.5).
    pub(crate) fn root_commands(&self, env_name: &str) -> Vec<String>;

    /// True iff `root` (case-insensitive, trimmed) is a canonical root of the
    /// named environment. The validation primitive (Req 20.4-20.5).
    pub(crate) fn contains_root(&self, env_name: &str, root: &str) -> bool {
        let r = root.trim();
        self.root_commands(env_name)
            .iter()
            .any(|c| c.eq_ignore_ascii_case(r))
    }
}
```

Delegation by name:
- `FFCMD` -> `crate::shell::alias_reader::builtin_ffcmd_roots()`.
- `FFEDIT` -> `AliasTable::ffedit_english().canonical_verbs_owned()` (the
  canonical roots are identical whether sourced from `ffedit_english()` or the
  live `LocaleAliasStore` table, because the locale overlay only adds SURFACES,
  never roots -- so the cheaper fresh-seed source is used).
- any other name (FFNAV / FFLINE / HOSTFS / MAINFRAME) -> `Vec::new()`.

`root_commands` returns a DETERMINISTIC, case-insensitively de-duplicated order
(Req 20.2); `builtin_ffcmd_roots` and `canonical_verbs_owned` already de-duplicate
case-insensitively, so this is either their existing stable order or a sort
applied in the query (an implementation choice confirmed at build time).

### FFCMD completeness posture

`root_commands("FFCMD")` is the COMPLETE BUILT-IN canonical root set. It unions
THREE co-located, drift-guarded sources inside `builtin_ffcmd_roots()`
(`shell/alias_reader.rs`):

1. the two code-only built-in menus' option roots (`recovery_pom_menu` +
   `recovery_settings_menu`): SETTINGS / CATALOGS / FILES / HELP / X / CONFIG /
   ALIASES / THEME / MENUS / KEYS / KINDS / RESET BARE;
2. `command_config::BUILTIN_WORKSPACE_VERB_ROOTS` -- the `resolve_target`
   CustomWorkspace verb roots (COMMANDS / LOG / PLUGINS / MACROS / SEARCH /
   GSEARCH / etc.), pinned to the classifier by an existing drift-guard test;
3. `command_config::BUILTIN_FFCMD_LADDER_VERB_ROOTS` (ADDED for CR-CH-067) -- the
   FFCMD root verbs handled by the `handle_command` LADDER rather than the
   classifier, MOST IMPORTANTLY the file-open family EDIT / VIEW / BROWSE, plus
   the lifecycle/navigation/display verbs (EXIT, START, MENU, END, RETURN,
   DETACH, SPLIT, SWAP, WORKSPACE, PFSHOW, COMMAND, SNAPSHOT, CAPTURE, RETRIEVE,
   SUBMIT, TIME, STATUS, ...). A drift-guard test asserts this list is DISJOINT
   from the classifier list (these verbs are NOT claimed by
   `builtin_workspace_target_for`) and carries EDIT/VIEW/BROWSE.

Source 3 is why `ECOB -> EDIT` is a valid FFCMD alias: EDIT is an FFCMD file-open
verb (owner-confirmed), NOT an FFEDIT in-buffer verb, so it belongs in the FFCMD
root set. Without the union a user alias to EDIT in FFCMD would be incorrectly
rejected by the CR-CH-067 load/save validation.

A root reachable ONLY via a user `menus/*.toml` override, or a dynamic
menu/command-definition name, is still intentionally NOT included (that open-ended
set shifts with user files). For the Gate 2 Alias Editor dropdown this is the
correct, stable canonical set, and load/save validation uses the SAME set so
author and validator agree (Req 20.1). This is also the foundation for the future
"list the supported FFCMD commands" API the owner noted.

### Reject-with-WARNING-at-load (Req 20.7) -- mirrors the existing discipline

`apply_user_rows` (`shell/alias_reader/alias_user_overlay.rs`) already SKIPS a row
with a WARN in two cases: a surface that shadows a reserved built-in root, and a
cross-layer different-root collision. CR-CH-067 ADDS one more skip-with-WARN
branch, BEFORE the row is applied, using `contains_root`:

```text
for each user alias row:
    (existing) skip malformed (empty surface/canonical)
    (existing) skip + WARN if surface shadows a reserved built-in root
    (NEW, Req 20.7) if !registry.contains_root(row.environment.env_name(), &row.canonical):
        log_warn!("[aliases] user alias '{surface}' names root '{canonical}'
                   which is not a valid root of environment '{env}' -- skipped")
        continue   // skip this row only; keep loading the rest
    (existing) route into the environment table; apply_locale_overlay_keyed,
               WARN on a cross-layer collision
```

The branch is a SKIP, never an abort: a single invalid row cannot block startup or
drop any other valid row (Req 20.7). The existing guards are retained unchanged.

### Save-side hook (Req 20.8)

`validate_user_alias_rows` (`shell/alias_editor.rs`) already rejects an empty
surface/canonical, a within-environment same-surface-two-roots collision, and a
surface equal to a reserved built-in root. CR-CH-067 adds the SAME
`contains_root(row.environment, row.canonical)` check so the validator can reject
an invalid `(environment, root)` pair at save. Gate 1 only ADDS the check as a
hook; the interactive save-side rejection UX (surfacing the rejection in the Alias
Editor) is Gate 2.

### Framework conformance

- ADDITIVE and read-only: a new registry query + a new load-time skip branch + a
  new save-side validation branch. No second dispatcher, no second navigation
  stack, no new persistence format (framework-conformance mechanisms 1, 3, 4, 6).
- Reshapes NO public framework type. `EnvironmentRegistry` gains methods; its
  shape and `CommandEnvironment` / `CommandTarget` / `WorkspaceContext` are
  unchanged.
- Builds ON existing data (`canonical_verbs_owned`, `builtin_ffcmd_roots`) and the
  existing skip-with-WARN overlay discipline.

## Correct editor command-environment placement (CR-CH-069, Requirement 21)

The command-environment placement audit
(`.agents/tasks/command-placement-audit/command-environment-audit.md`, sections 5
and 8) applied ONE semantic test to every registered verb:

> Does the command act on the WORKBENCH / a target / navigation (FFCMD), or does
> it act on the CONTENT of the buffer currently open in the editor (FFEDIT)?

That test produced exactly three corrections, which this design records. No verb
other than the two named below is moved; SUBMIT / TIME / STATUS are AMBIGUOUS and
deliberately LEFT on FFCMD for a later FFJES decision.

### (A) AUTONUM / NUM move FFCMD -> FFEDIT

AUTONUM toggles AUTOMATIC line renumbering (it edits the buffer's sequence
columns), and NUM is an alias of NUMBER (generate/insert sequence numbers into the
buffer). Both mutate the CONTENT of the open editor buffer and are meaningless
with no buffer open, so by the ownership principle (Requirement 2a.4 / Requirement
10) they are FFEDIT verbs.

Today they live on the FFCMD `handle_command` ladder (`commands_ladder_c.rs`
AUTONUM / NUM arms) and are listed in `BUILTIN_FFCMD_LADDER_VERB_ROOTS`
(`command_config/mod.rs`). The correction:

- REMOVE AUTONUM and NUM from `BUILTIN_FFCMD_LADDER_VERB_ROOTS` so
  `root_commands("FFCMD")` no longer contains them.
- ADD AUTONUM and NUM as canonical roots of `AliasTable::ffedit_english()`
  (`shell/environment.rs`) so `root_commands("FFEDIT")` contains them and the
  FFEDIT active-wins claim path (Requirement 5.1) handles them when an editor
  Context is focused.

This is a PURE CLASSIFICATION MOVE. Their redirect target NUMBER has NO ladder arm
and NO `CommandEngine` implementation on main -- it falls through to the engine's
"unrecognised command" result today. So the NUMBER / UNNUM / NUMBER-SHOW engine is
NOT built here (the deferred `ff-sequence-numbers` effort). The one observable
change is WHERE AUTONUM/NUM are claimed: editor-active (FFEDIT) instead of
anywhere (the global FFCMD ladder). Because NUMBER is unimplemented, the practical
effect today is already "nothing useful happens", so the behaviour risk is low --
but the classification change is gated because it changes the (environment, root)
sets the alias-anchoring model reads.

### (B) Relocate the mis-placed af SOEK -> FIND alias

`i18n/af/aliases/FFCMD.toml` maps SOEK -> FIND under FFCMD, but FIND is an FFEDIT
root (command-semantics Requirement 12.1; `ffedit_english` owns FIND), NOT an
FFCMD root. The row shipped only because the per-locale reader path uses the
3-field `parse_alias_toml` (`alias_reader.rs`) which runs NO `contains_root` check
(that check runs only on the user-store path). The correction moves the row into
`i18n/af/aliases/FFEDIT.toml` (the per-locale FFEDIT catalogue) and removes it from
the af FFCMD.toml, and moves the af POC load test's assertion from the FFCMD
catalogue to the FFEDIT catalogue. These are per-locale DATA files (out of ASCII
scope).

### (C) Fix the stale EDIT doc-comment

`shell/user_aliases.rs` (~lines 26-28) wrongly says the AliasEnvironment choice for
an editor-verb root such as EDIT is FFEDIT (e.g. "ECOB -> EDIT is an FFEDIT
alias"). The code and tests are authoritative and treat ECOB -> EDIT as an FFCMD
alias: EDIT is an FFCMD file-open root (file-open = workbench, owner-confirmed; see
Requirement 20.1 and the `builtin_ffcmd_roots` union of the file-open family). The
comment is corrected to say FFCMD. Documentation-only, no behaviour change.

### Interaction with CR-CH-067 (the validation safety net)

Moving AUTONUM/NUM directly changes `root_commands`/`contains_root` for both
environments, so the correction updates `builtin_ffcmd_roots()` inputs and
`ffedit_english()` together and re-baselines the `environment_registry.rs`
enumeration tests. After the move, a user alias anchoring AUTONUM/NUM (or FIND) to
FFCMD is correctly invalid-in-environment and is rejected by the CR-CH-067
load/save guards (Requirement 20.7 / 20.8) -- intended, not a regression.

### Sequencing

This correction lands BEFORE the Afrikaans FFCMD alias-data authoring and BEFORE
the `=FILES` root-token fix (audit item D, a separate change request), because the
corrected (environment, root) sets are the anchor both of those tasks build on.
Authoring Afrikaans aliases against the current (wrong) root sets would bake the
misplacement into shipped localisation data.

### Framework conformance

CLASSIFICATION/DATA/DOC edits only. No second dispatcher, no second navigation
stack, no new persistence format, no reshape of a public framework type. The move
edits the existing root SOURCES (`BUILTIN_FFCMD_LADDER_VERB_ROOTS` /
`ffedit_english`) and routes AUTONUM/NUM through the existing FFEDIT active-wins
claim path; it introduces nothing new.

## ISPF Save/End/Cancel/Return/Refresh for editable Contexts (CR-CH-071) -- Requirement 20a

Design delta for CR-CH-071. Grounded in
`.agents/tasks/alias-editor-save-semantics/findings.md` (sections B and C). This
gives every EDITABLE Workspace Context the ISPF Save/End/Cancel/Return/Refresh
verb set through the ONE command-dispatch path, retires the Alias Editor's private
`Command ===>` SAVE/CANCEL/END interpreter, and makes SAVE/CANCEL/REFRESH FFCMD
canonical roots handled per-focused-editable-Context, with FFEDIT keeping its
buffer SAVE as the active-wins override. It is ADDITIVE (CR-CH-069 class) plus a
gated behaviour change; it reshapes no public framework type and adds no second
dispatcher or navigation stack.

### The problem it fixes (from the findings)

Today the Alias Editor has TWO parallel command paths. (1) The panel's own
`Command ===>` field is interpreted by `WorkbenchShell::apply_alias_command_line`
(`shell/alias_editor_line_commands.rs`), where SAVE persists+stays (correct) but
CANCEL and END BOTH just `nav_end()` -- leave with no save and no explicit
discard, no distinction between them. (2) F3 / a globally-typed END never reaches
that interpreter; it goes through the FFCMD ladder (`try_commands_a -> nav_end`),
which also does not save. So END never saves (wrong vs ISPF), CANCEL does not
explicitly discard, and SAVE is reachable only from the panel field, not as a
workbench verb. There is also no CANCEL verb and no REFRESH/REVERT verb anywhere
dispatchable today (CANCEL exists only in the panel interpreter + a help topic
list + the ff-navigation-commands delegation table). This CR unifies the two paths
and adds the missing verbs and semantics.

### The editable-Context verb set (ISPF)

| Verb | Effect on a focused editable Context |
|------|--------------------------------------|
| SAVE | persist the working copy, STAY |
| END (F3) | SAVE the working copy, then leave (`nav_end`) -- ISPF END commits |
| CANCEL | DISCARD the working copy, then leave (`nav_end`), no save |
| RETURN | DISCARD the working copy, then leave to the tab visual root (`nav_return`), no save |
| REFRESH | DISCARD the working copy, RELOAD last-saved state, STAY |

EXIT / QUIT / LOGOFF remain the application-exit family UNCHANGED (Req 20a.13): the
save-then-leave verb is END, not EXIT. This settles the findings' open point -- EXIT
is NOT repurposed, so app-exit is not broken.

### The focused-editable-Context SEAM (the one new abstraction)

The FFCMD base needs a way to ask "the Context I am focused on: persist / discard /
reload your working copy". This is a small trait-shaped seam the shell implements
for each editable Context, mirroring the `pending_action` / `apply_<editor>_action`
pattern the config editors already use (command-environments Req 11 E10 option a;
wiring-standard.md). Illustrative shape (names finalised at implementation):

```rust
/// A focused Workspace Context that owns an editable WORKING COPY (an alias set,
/// a theme, a menu, a key map, a kind config, a configuration). The FFCMD base
/// calls these when SAVE / CANCEL / REFRESH / END / RETURN are dispatched and an
/// editor BUFFER is not the active-wins claimant. Returns an outcome the caller
/// surfaces (error -> stay + show; ok -> leave for END/CANCEL/RETURN, stay for
/// SAVE/REFRESH).
trait EditableContext {
    /// Persist the working copy. Ok => stay (SAVE) or proceed to leave (END).
    /// Err(msg) => leave the backing store unchanged, surface msg, STAY.
    fn persist(&mut self) -> Result<(), String>;
    /// Drop the working copy's unsaved changes (explicit, immediate). Used by
    /// CANCEL / RETURN before leaving, and by REFRESH before reload.
    fn discard(&mut self);
    /// Reload the last-saved state from the backing store into the working copy
    /// (REFRESH). For the Alias Editor: re-read the user alias store from disk.
    fn reload(&mut self);
}
```

Because the editable Contexts are render-entangled with the shell (they are panels
on `WorkbenchShell`, not standalone objects), the seam is realised the SAME way
FFEDIT is (design "FFEDIT is a resolver+executor over the active editor"): as a
small dispatch helper on the shell that, given the focused Context kind, routes to
that Context's existing persist/discard/reload methods. The handler does NOT
enumerate kinds in a hardcoded match beyond asking "is the focused Context an
editable non-buffer Context, and if so which". For the Alias Editor the three
operations map to:

- `persist` -> the EXISTING `save_user_aliases` (validate + write `user.toml` +
  refresh the live `ACTIVE_ALIAS_STORE` in place), UNCHANGED.
- `discard` -> drop the in-memory `alias_editor_panel.rows` working copy (clear
  the pending edits so they cannot linger to the next open -- the findings note
  today's discard is only accidental, via reload-on-next-open).
- `reload` -> re-run the open-time `load_user_alias_rows` + `load_rows(rows)` to
  repopulate the panel from the last-saved `user.toml`.

### FFCMD-base handler + the active-wins override (how it dispatches)

SAVE / CANCEL / REFRESH become FFCMD-base ladder verbs, reached through the ONE
shared dispatch path exactly like the existing FFCMD verbs. On the shared ladder:

```text
dispatch(raw):                                   # (the existing order, unchanged)
  prelude / = universal / menu fastpath ...
  if active_env == FFEDIT and FFEDIT.claim(raw): return   # active-wins: buffer SAVE/END/CANCEL/RETURN
  # FFCMD base:
  if try_commands_* handles SAVE/CANCEL/REFRESH/END/RETURN:
      if focused Context is an editable non-buffer Context (EditableContext seam):
          SAVE    -> persist(); stay
          REFRESH -> discard(); reload(); stay
          CANCEL  -> discard(); nav_end()
          RETURN  -> discard(); nav_return()
          END     -> persist(); if ok nav_end() else stay+error
          return
      else:  # non-editable Context
          SAVE/REFRESH -> unresolved / no-op (as today, Req 2a.5)
          END/RETURN   -> plain FFCMD/universal navigation (as today)
  ...
```

The ACTIVE-WINS rule (Req 5.1) is the override mechanism and needs NO new
machinery: when an editor BUFFER is focused, FFEDIT is the active environment and
its `claim` runs BEFORE the FFCMD base, so FFEDIT's buffer SAVE (Req 10.1, dirty-
aware, `dispatch_to_environment`-addressed) and its buffer END/CANCEL/RETURN
(Req 10.2-10.4) are reached FIRST and are BYTE-IDENTICAL to today. On an editable
NON-buffer Context (Alias Editor + config editors) FFEDIT is not active, declines,
and the FFCMD base's editable-Context handling runs. REFRESH is NOT claimed by
FFEDIT in v1 (Req 20a.8), so on an editor buffer REFRESH falls to the FFCMD base,
which is a no-op there (a buffer is not the `EditableContext` seam); an FFEDIT
buffer-level revert is a later additive task.

This means the owner's "FFCMD should have its own save command separate from the
FFEDIT save" is realised as: SAVE is an FFCMD-base verb that DELEGATES to the
focused Context's seam, and FFEDIT's buffer SAVE OVERRIDES it via active-wins when
an editor is focused -- precisely the "reuse or overwrite" the owner anticipated.

### Registering SAVE / CANCEL / REFRESH as FFCMD roots (CR-CH-067 enumeration)

SAVE, CANCEL, and REFRESH are added to the FFCMD root SOURCE
`command_config::BUILTIN_FFCMD_LADDER_VERB_ROOTS` (the list `builtin_ffcmd_roots()`
unions, which feeds `EnvironmentRegistry::root_commands("FFCMD")` /
`contains_root`, CR-CH-067 Req 20.1). END and RETURN already live there and are
UNCHANGED. Consequences, all intended:

- `root_commands("FFCMD")` now lists SAVE / CANCEL / REFRESH; the Alias Editor's
  Root dropdown (for a FFCMD-environment alias) will offer them -- harmless, they
  are legitimate FFCMD roots.
- The CR-NR-113 no-shadow invariant (command-configurator Req 5.4 (d) / 5.7,
  grounded on `builtin_ffcmd_roots()`) now REJECTS a user alias named SAVE /
  CANCEL / REFRESH at save and SKIPS it with a WARN at load (Req 20a.12) -- the
  desired "a user alias may not be named SAVE/CANCEL/REFRESH" consequence.
- FFEDIT keeps SAVE in `ffedit_english` (its canonical root set is UNCHANGED).
  Both environments listing SAVE is consistent: `contains_root("FFEDIT","SAVE")`
  and `contains_root("FFCMD","SAVE")` are both TRUE, so an alias anchoring SAVE in
  either environment validates, and active-wins decides which runs. CANCEL /
  REFRESH are FFCMD roots only in v1.

### Retiring the Alias Editor's private interpreter (one dispatch path)

`apply_alias_command_line` (`shell/alias_editor_line_commands.rs`) currently has
arms `SAVE -> save_user_aliases`, `CANCEL | END -> nav_end`. Those three verb arms
are REMOVED: SAVE / CANCEL / END / RETURN / REFRESH now flow through the shared
dispatch path (as FFCMD-base verbs resolved per-focused-Context). The panel-LOCAL
verbs FILTER and RESET (which set/clear the filter row, command-configurator
Req 5.15) are RETAINED in the panel interpreter -- they are a Context-local display
concern, not a cross-Context command. The line-command apply pass (I / R / D,
Req 5.16) still runs FIRST on Enter (Req 5.22); after retirement the sequence is:
apply pending line commands -> if the primary verb is FILTER/RESET handle locally,
otherwise hand the typed line to the shared dispatch path (so SAVE/END/CANCEL/
REFRESH reach the FFCMD-base editable-Context handler, identical to F3 / a typed
END). This closes the two-parallel-paths divergence: F3 and the typed line reach
the SAME handler.

### Per-editor wiring plan (Alias Editor first, siblings as fast-follow)

The seam is designed UNIFORMLY now; implementation is phased (Req 20a.11):

1. ALIAS EDITOR (first implementor): wire the `EditableContext` seam to
   `save_user_aliases` (persist), a working-copy clear (discard), and the open-time
   row reload (reload); retire the three private-interpreter arms; add the FFCMD-
   base editable-Context handler + the root registration. TDD each verb.
2. SIBLING CONFIG EDITORS (Settings / Menus / Theme / Keys / Kinds) as fast-follow
   tasks: each already has a Save action applied by `apply_<editor>_action` (the
   E10 pattern); wire each to the SAME seam (persist = the existing save action;
   discard/reload = drop + re-open its working copy). No new per-editor verb logic
   -- they adopt the shared handler. The semantics are NOT re-implemented per
   editor.

### REFRESH vs UNDO naming (collision rationale)

The context-revert verb is REFRESH, NOT UNDO. UNDO is the FFEDIT buffer-level
single-step undo (command-environments Req 10.6 defers it as FFEDIT buffer scope;
CR-CH-054 Req 12.8 builds the UNDO verb over the buffer transaction manager;
undo-redo-transactions owns the model). A whole-Context discard-and-reload is a
DIFFERENT concept (it drops ALL unsaved changes and reloads from disk, not one
edit step). Reusing "UNDO" for it would put two incompatible meanings on one verb
name and collide on an editor Context (where UNDO already means buffer single-step
undo). REFRESH is unambiguous and is the RECOMMENDED canonical verb. REVERT was
considered as an alternative surface; the recommendation is REFRESH as the
canonical verb, and IF a future decision wants REVERT it SHALL be an ALIAS surface
of canonical REFRESH (the per-environment alias mechanism, Req 6a), not a second
verb. (OPEN DECISION, recorded for the owner CONFIRM step; recommended default =
REFRESH.)

### Open decisions (recorded for the owner CONFIRM step)

1. REFRESH vs REVERT verb name. RECOMMENDED: REFRESH (collision rationale above).
2. CANCEL / REFRESH FFCMD-only in v1 vs FFEDIT also claims a buffer-revert now.
   RECOMMENDED: FFCMD-base-only in v1 (FFEDIT declines REFRESH, keeps its buffer
   CANCEL/RETURN per Req 10); an FFEDIT buffer-revert is a later additive task.
3. EXIT on an editable Context. RESOLVED in this design: END (not EXIT) is the
   save-then-leave verb; EXIT stays the app-exit family (Req 20a.13), so app-exit
   is not broken. Recorded as a decision, not left open.
4. Registration shape: FFCMD-base-delegating handler (this design) vs per-Context-
   environment SAVE (Req 11 E10 option a). RECOMMENDED: the FFCMD-base-delegating
   handler, because it gives ONE handler + ONE seam for all editable Contexts and
   matches the owner's "FFCMD's own save" phrasing; the OBSERVABLE behaviour is
   identical to the per-Context-environment shape, which is why this reconciles
   with Req 2a.5 / 11.1 (the base owns no working copy; it delegates).
5. Scope of sibling editors in v1 vs follow-up. RECOMMENDED: design the uniform
   seam now; implement the Alias Editor first, siblings as fast-follow tasks.

### What is explicitly NOT changing

- FFEDIT's buffer SAVE root (`ffedit_english`) and its `dispatch_to_environment`
  store-addressing claim (Req 14) -- BYTE-IDENTICAL native write.
- FFEDIT's buffer END / CANCEL / RETURN dirty-aware behaviour (Req 10.2-10.4).
- EXIT / QUIT / LOGOFF app-exit handling.
- The command-dispatch front door, the per-tab Navigation_Stack, the
  `WorkspaceContext` focus latch, session persistence, and `CommandTarget` -- no
  second dispatcher, no second navigation stack, no new persistence format, no
  public framework type reshaped.

### Framework conformance

ADDITIVE ownership/placement (CR-CH-069 class) plus a gated behaviour change. The
new verbs register in the EXISTING FFCMD root source and dispatch through the
EXISTING ladder + active-wins; the seam reuses the EXISTING `pending_action`
pattern; FFEDIT is untouched. No core framework mechanism is altered, so this is
NOT a framework change by the framework-conformance definition; the behaviour /
criteria change runs the normal gate and needs owner CONFIRM.

## Remove the malformed `=FILES` literal root token (CR-CH-070, Requirement 22)

Design delta for CR-CH-070, the `=FILES` root-token fix that CR-CH-069 explicitly
deferred as a separate change request (audit item D). `=FILES` was registered as a
LITERAL root token, but `=` is the ISPF JUMP PREFIX and the real root command is
FILES. The literal is redundant with the generic `=` handling AND pollutes the
FFCMD root set the alias-anchoring model reads, so it is removed.

### The bug

Two co-located source sites carried the malformed token
(`crates/ff-desktop/src/command_config/mod.rs`):

1. `BUILTIN_WORKSPACE_VERB_ROOTS` listed `"=FILES"` as its first entry (described
   in the doc-comment as "the fastpath single token").
2. `builtin_workspace_target_for` had a dedicated arm:
   `if is("=FILES") { return custom("file_explorer"); }`, placed before the
   two-word `FILE CATALOGS` arm and the bare `FILES` arm.

Because `BUILTIN_WORKSPACE_VERB_ROOTS` is one of the three sources
`builtin_ffcmd_roots()` unions into the FFCMD root set (Requirement 20.1), the
malformed `=FILES` flowed straight into `EnvironmentRegistry::root_commands("FFCMD")`
and was therefore accepted by `contains_root("FFCMD", "=FILES")`. The Gate 2 Alias
Editor Root dropdown (populated by `root_commands`) and the CR-CH-067 load/save
alias-anchoring validation (keyed on `contains_root`) would then OFFER and ACCEPT
`=FILES` as if it were a legitimate anchorable root -- it is not.

### Why removing the literal is SAFE (the `=` jump already covers it)

The `=` prefix is handled EXACTLY ONCE at the single front door
(`shell/dispatch.rs::dispatch_command_string`, CR-CH-052, command-framework
Requirement 10 criterion 10.2 / criterion 14). That step runs BEFORE the prelude
and `resolve_target`:

```
if let Some(rest) = raw.trim_start().strip_prefix('=') {
    let rest = rest.trim().to_string();
    self.reinitialise_active_tab_to_pom();   // jump to the POM (FFCMD_Root)
    self.dispatch_command_string(&rest);     // re-dispatch the remainder
    return;
}
```

So typing `=FILES` strips the `=`, reinitialises the active tab to the POM, and
RECURSIVELY dispatches `FILES` through the normal front door, where the generic
bare arm `_ if is("FILES") => custom("file_explorer")` classifies it. The literal
`=FILES` classifier arm is therefore NEVER the thing that makes `=FILES` work --
the generic `=` step already covers it. Removing the literal preserves the
`=FILES` jump path verbatim (Requirement 22.3).

### The correction

- REMOVE `"=FILES"` from `BUILTIN_WORKSPACE_VERB_ROOTS` so
  `root_commands("FFCMD")` returns FILES and NO LONGER returns `=FILES`
  (Requirement 22.1).
- REMOVE the dedicated `is("=FILES")` arm from `builtin_workspace_target_for`. The
  bare `is("FILES") => custom("file_explorer")` arm is retained unchanged and
  continues to open the File Explorer workspace (Requirement 22.2).
- Update the `BUILTIN_WORKSPACE_VERB_ROOTS` doc-comment to drop the "`=FILES` is
  the fastpath single token" sentence.

### FILE CATALOGS finding (verified, LEFT UNCHANGED)

The audit flagged `FILE CATALOGS` as a two-word classifier entry to check.
Verification at this gate confirms it is a LEGITIMATE two-word verb:
`builtin_workspace_target_for` has `if is("FILE") && arg.eq_ignore_ascii_case("CATALOGS")
{ return custom("files"); }`, which opens the `files` workspace. It is a genuine
command surface, not a jump-prefix artefact, and it is NOT polluting the root set
in the malformed way `=FILES` was. It is therefore LEFT UNCHANGED (Requirement
22.4); only `=FILES` is fixed.

### Interaction with CR-CH-067 (the validation safety net)

Removing `=FILES` directly changes what `root_commands("FFCMD")` and
`contains_root` return: `root_commands("FFCMD")` loses `=FILES` and keeps FILES, so
`contains_root("FFCMD", "=FILES")` becomes FALSE (Requirement 22.1). This cleans the
FFCMD root set the CR-CH-067 Gate 1 validation and the future Gate 2 Alias Editor
dropdown consume, ahead of the Afrikaans alias-data authoring (which anchors against
this root set).

### Sequencing

Lands AFTER CR-CH-069 (the AUTONUM/NUM + af SOEK->FIND placement correction) and
BEFORE the Afrikaans FFCMD alias-data authoring, consistent with the audit's
"correct the (environment, root) sets before authoring localisation data against
them" ordering.

### Framework conformance

Builds ON the existing single `=` jump-prefix front-door step (command-framework
Requirement 10 criterion 10.2 / criterion 14, CR-CH-052) and the existing bare
`FILES` classifier arm. It REMOVES a redundant, malformed literal; it adds no
dispatcher, no navigation stack, no persistence format, and reshapes no public
framework type. NOT a framework change.

---

## CR-CH-072 -- Two-level resolution over the in-row surface (Requirement 2b)

### Overview

CR-CH-072 records the owner-confirmed TWO-LEVEL resolution model and points it at
the simplified single-layer alias surface. The alias DATA model, the per-locale
store collapse, the menu-option-references-an-alias change, and the Alias Editor's
active-locale view are designed in `docs/specs/localization/design.md`
("CR-CH-072 -- Single-layer surface-in-row alias + ONE locale-aware store"); this
section covers the RESOLUTION PATH that lives in the command-environments spec and
cross-references the localization design for the data/store/editor concerns.

### The resolution path (no new dispatcher)

The two-level contract (Requirement 2b) IS the existing `rewrite_verb_to_root`
pre-pass (`shell/locale_resolver.rs`) running before `ff_command::resolve_target`
at the two seams it already serves (`shell/dispatch.rs` typed front door,
`shell/commands_menu.rs` menu selection). FFCMD remains `resolve_target`; the
active-environment step is the already-present pre-`resolve_target` stage (the
B080 front door, Requirement 3). The algorithm:

```
# Level 1 -- ACTIVE Command Environment
1a. V matches an In_Row_Surface in ACTIVE.alias_table[locale] -> root; execute(root)
1b. else V is a root of ACTIVE                                -> execute(V)
1c. else REJECT and pass V back
# Level 2 -- FFCMD base (receives any string ACTIVE rejected)
2a. V matches an In_Row_Surface in FFCMD.alias_table[locale]  -> root; execute(root)
2b. else V is a root of FFCMD                                 -> execute(V)
2c. else                                                      -> unresolved-command result
```

Each surface comparison uses the diacritic-insensitive fold (localization
Req 11.11); an alias carrying a `command_template` expands it (CR-NR-112) before
`resolve_target`. The resolved ROOT is recorded/persisted (Req 6a.2); the
In_Row_Surface is input/display only. EXACTLY two levels; the SAME path for typed
and menu-selected commands (no render-time reverse-derivation).

### What is NOT changing

`CommandTarget` and `resolve_target` are unchanged; FFCMD = `resolve_target`; no
second dispatcher, no second navigation stack, no new persistence format. The
FFCMD/FFEDIT root sets (CR-CH-069/070), active-wins (Req 5), the prelude/shadowing
precedence (Req 6a.4), and the WorkspaceContext focus contract are unchanged. This
is a re-statement of the existing pre-pass contract plus the alias-surface source
collapse; NOT a framework change.
