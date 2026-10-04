# Command Environments (CR-CH-053) -- Explainer and Analysis

STATUS: EXPLAINER. This document explains the Command Environment idea in plain
terms and then analyses it: what is strong, what is risky, and whether it could
be done better. It is a reading aid over the authoritative specs
(`requirements.md`, `design.md`, `tasks.md`, `environments-vision.md`) and the
read-only `design-proposal.md`. Plain ASCII only.

---

## 1. The idea in one paragraph

FileForgeWorkbench (FFWB) is a mainframe-style tool, and the mainframe world is
not one giant command set -- it is many small, named command environments (TSO,
ISPF Editor, SDSF, IDCAMS, DB2 SPUFI, ...), each owning the commands that make
sense for what you are doing. REXX captures this with `ADDRESS <env>`. CR-CH-053
adopts the same shape for FFWB: a **Command Environment** is a named owner of a
command vocabulary; the **active Workspace Context** picks which environment is
active; an always-present **base environment (FFCMD)** is the fallback; and a
macro can **address** a specific environment by name. Crucially, this is designed
as a thin generalisation of the command dispatch that already exists, not a new
engine.

---

## 2. Why this idea exists (the problem it solves)

The workbench already has one command "front door" (`dispatch_command_string`,
from the B080 work). Today that front door resolves a typed command in roughly
three stages:

1. A **prelude** -- menu option keys, the EXIT family, POM / chained fastpaths.
2. **`resolve_target`** -- the shell/workbench classifier that maps a verb to one
   of five `CommandTarget` variants (`Menu`, `CustomWorkspace`, `Function`,
   `Macro`, `External`). This is the thing CR-CH-053 renames **FFCMD**.
3. A **ladder** of hand-written `if` arms, ending in the semantics engine.

The problem: the editor's command-line verbs (`FIND`, `CHANGE`, `LOCATE`,
`EXCLUDE`, `SORT`, `CAPS`, `SCROLL`, ...) are stuck as hand-written arms in the
ladder (`commands_ladder_b2.rs`). They were left there deliberately because they
do not fit any of the five `CommandTarget` variants -- they act on the active
editor buffer through managers (`nav_manager`, `find_manager`, `exclude_manager`,
`edit_profile`), not on the workbench. There was no clean home for them.

The Command Environment model gives them a home: an **FFEDIT** environment that
owns exactly those verbs and runs them against the active editor. The verbs move
out of the shared ladder into a named environment selected by context.

---

## 3. The core concepts

| Concept | What it is | Phase-1 reality |
|---------|-----------|-----------------|
| **Command_Environment** | A named resolver that either CLAIMS a command string (handles it, returns a result + return code) or declines (falls through to the next). | A `trait` reusing the existing `TargetResolver` shape. |
| **FFCMD (Base_Environment)** | The always-present fallback. It IS the existing `resolve_target` / `ShellTargetResolver` chain -- no rewrite. | Already exists; just named. |
| **FFEDIT** | The editor command-line environment (LOCATE/FIND/CHANGE/EXCLUDE/SORT/profile/scroll verbs). | BUILT: verbs migrated off the shared ladder. |
| **FFLINE** | The prefix-area line commands (D/DD/M/C/CC/A/B...), a sibling of FFEDIT. | NAMED only -- intake stays prefix-gutter -> semantics engine; no code change. |
| **FFNAV** | The file-navigator environment, whose FIND locates a *file* (not buffer text). | NAMED only -- navigator verbs stay where they are handled today. |
| **Environment_Registry** | Shell-owned collection that derives the active environment, looks one up by name for addressing, and exposes FFCMD as base. | New, small. Not a dispatcher. |
| **Active_Environment** | The environment the focused Context selects. | Derived from `TabKind`. |
| **Address / Alias_Map** | Routing a command to a named env (REXX `ADDRESS`); mainframe names map onto FF* names (TSO -> FFCMD, ISREDIT -> FFEDIT). | Macros only in phase 1. |

### The key insight: the environment is the namespace

The same verb name can be owned by more than one environment, each with its own
implementation. `FIND` in **FFEDIT** searches the open file; `FIND` in **FFNAV**
locates a file. There is no single global definition of `FIND` -- the active
Context decides which runs. This formalises context-dependent behaviour that
already exists rather than inventing new behaviour.

---

## 4. How a typed command flows (the router)

The design makes exactly one insertion into the single existing front door:

```
dispatch_command_string(raw):
    if run_command_prelude(raw):  return          # UNCHANGED, FIRST, unshadowable
    # [reserved, inert in phase 1: parse_address_prefix]
    match active_env():                            # NEW: editor->FFEDIT, nav->FFNAV, else FFCMD
        FFEDIT => if ffedit.dispatch(raw): return  #   active env gets first crack
        FFNAV  => { }                              #   phase 1: named only, no claim
        FFCMD  => { }                              #   base: fall straight through
    if let Ok(t) = resolve_target(raw): dispatch(t); return   # FFCMD base, UNCHANGED
    run_command_ladder(raw)                        # shrinking fallback -> engine terminal
```

Properties the design is careful to preserve:

- **One front door.** The active-env step is a resolver stage inside the same
  chain, not a parallel dispatcher.
- **FFCMD is still `resolve_target`.** No second classifier.
- **The prelude stays first and unshadowable.** EXIT, menu option keys, and POM
  fastpaths keep their precedence.
- **Active-wins shadowing.** When a verb exists in both the active env and FFCMD,
  the active env claims it; the FFCMD version is reachable by explicit address.
  This is the one genuinely new ordering rule, and it is the ISPF/REXX rule.
- **The ladder shrinks.** As FFEDIT families migrate out, `try_commands_b2`
  empties and is eventually deleted.

---

## 5. What phase 1 actually builds

Deliberately small. Only the framework plus the two environments that already
exist in the running app:

- **FFCMD** -- named, not rewritten.
- **FFEDIT** -- editor verbs migrated off the ladder in behaviour-preserving
  families (navigation, exclude/show, find, profile, scroll), each with a
  red-before-green test and per-step rollback discipline.
- **FFLINE** and **FFNAV** -- named/modeled only, so the active-env derivation and
  the per-environment verb model are correct and future `ADDRESS` targets exist.
- **Macro addressing** -- reconciled with the already-specified lua-macro-engine
  `ADDRESS` host environments via the alias map; dispatch returns a code that maps
  to macro `RC`.

Everything else in the catalogue (FFBROWSE, FFAMS, FFJES, FFJOB, FFSQL, FFCICS,
FFVFS, FFADMIN, FFDEBUG, FFMON, FFLIB) is **vision-only**: each registers when its
consuming subsystem is built. The template for adding one is deliberately tiny:
add the Context's `TabKind`, map that kind to its environment in the active-env
derivation, register the environment's resolver -- no front-door change.

---

## 6. Analysis -- is this a good design?

### 6.1 What is strong

1. **It builds on the existing framework instead of fighting it.** FFCMD *is*
   `resolve_target`; the router adds one step; `CommandTarget` is untouched; there
   is no second dispatcher and no second navigation stack. This is the single most
   important property and it is honoured consistently across all four spec docs.

2. **It gives homeless verbs a principled home.** The editor verbs never fit the
   five `CommandTarget` variants. Rather than contorting them into fake `Function`
   targets, the model recognises them as a different *kind* of command. That is
   the correct diagnosis.

3. **The namespace insight is genuinely good.** "The environment is the
   namespace" cleanly explains why `FIND` can mean two things, and it scales:
   every future subsystem gets its own verb namespace without global collisions.

4. **Scope discipline is excellent.** Phase 1 builds 2 real environments and names
   2 more, with everything else documented but not built. The migration is
   family-by-family, behaviour-preserving, test-first, with rollback. This is low
   risk and reviewable.

5. **Addressing is reconciled, not reinvented.** The lua-macro-engine spec already
   had `ADDRESS TSO/ISREDIT/ISPEXEC`; the alias map (TSO->FFCMD, ISREDIT->FFEDIT)
   avoids two competing vocabularies. Catching that collision early is good spec
   hygiene.

### 6.2 Where it is weak or risky

1. **FFEDIT starts as methods on the god-struct.** Because the editor verbs mutate
   shell-entangled state (`self.tabs` + managers), the design implements FFEDIT
   first as methods *on* `WorkbenchShell` behind a marker, with the clean
   borrowed-manager form deferred "once the managers are grouped." That deferral is
   where good intentions usually die. The `CommandEnvironment` trait exists but
   FFEDIT does not really implement it as an independent object yet -- it is a trait
   with one honest implementor (future environments) and one special case (FFEDIT)
   that leans on the shell. The abstraction is real for FFCMD and future envs, but
   partly aspirational for the one env being built now.

2. **FFEDIT vs the Command_Engine boundary is still fuzzy.** The semantics engine
   (`ff-command-semantics`) is itself an editor-command executor for the
   prefix-area/scope path. The design says FFEDIT claims command-line verbs *before*
   the engine and the engine remains the fallback, so they "are not two competing
   executors." That is true for the verbs FFEDIT explicitly claims, but the line
   between "FFEDIT verb" and "engine primary command" is drawn by hand, verb by
   verb. Open question Q5 (does FFEDIT eventually subsume the engine's command-line
   role?) is deferred, which means this boundary has to be re-litigated later.

3. **FFLINE and FFNAV are named but not real.** This is honest and defensible, but
   it means the model's headline feature -- per-environment verb ownership
   (FFEDIT.FIND vs FFNAV.FIND) -- is only *asserted* in phase 1, not *exercised*.
   The active-env derivation maps nav Contexts to FFNAV, but FFNAV claims nothing,
   so navigator FIND still flows through whatever handles it today. The two-FIND
   story is documentation until FFNAV is actually built.

4. **Active-wins shadowing adds a new precedence rule to a hot path.** It is the
   right rule, but it is a new ordering decision in the one front door every typed
   command passes through. The backward-compatibility criterion (Req 4) is the
   safety net, but it leans heavily on test coverage of verb collisions. The
   proposal itself flags (Q4) that `X` is claimed by the prelude's EXIT family
   before either environment -- exactly the kind of subtle collision that needs
   exhaustive tests, not just the representative ones the tasks list.

5. **The interactive address prefix is deferred but seam-reserved.** Reserving an
   inert `parse_address_prefix` seam is reasonable, but "reserved but inert" code
   tends to rot. If interactive addressing never ships, the seam is dead weight; if
   it does, the deferred syntax decision (Q1: `ENV: cmd` vs `ADDRESS ENV cmd` vs
   `ENV cmd`) re-opens the ambiguity the phase-1 deferral postponed.

### 6.3 Could it be done better?

The design is sound and conservative, and for a phase-1 that must preserve
behaviour, conservative is correct. The improvements below are mostly about not
letting the deferred-cleanup items quietly become permanent.

1. **Make FFEDIT a real object sooner, even if small.** The biggest smell is that
   the one environment actually being built is the one that does not cleanly
   implement the trait. Consider grouping the editor managers into a borrowable
   sub-struct (`EditorEnv<'a>` holding `&mut nav_manager`, `&mut find_manager`,
   etc.) *as part of this gate* rather than "eventually." It is more churn now, but
   it is the difference between a trait with a genuine implementor and a trait with
   a special case. The rust-standards god-struct guidance already pushes this
   direction; CR-CH-053 is the natural moment to pay it.

2. **Define the FFEDIT/engine boundary declaratively, not verb-by-verb.** Instead
   of hand-partitioning verbs between FFEDIT and the Command_Engine, give FFEDIT an
   explicit, data-driven verb table (the set of names it owns). Then "FFEDIT claims
   it, else engine" is a lookup, not a maintenance hazard, and Q5 becomes a
   question of *what is in the table* rather than *which code arm runs first*. This
   also makes the per-environment namespace testable: you can assert the exact verb
   set each environment owns.

3. **Exercise the namespace with at least one real second environment.** The whole
   model's distinctive claim is per-context verb ownership. Phase 1 proves it with
   zero collisions actually resolved at runtime (FFNAV claims nothing). If FFNAV is
   too big, consider making even a *tiny* real FFNAV claim (just its `FIND`) so the
   FFEDIT.FIND vs FFNAV.FIND behaviour is demonstrated end-to-end by a test, not
   asserted on paper. That would validate the architecture's central idea instead
   of deferring the validation.

4. **Decide the interactive-prefix question now, even if the answer is "never."**
   Either commit to the `ENV: cmd` syntax and ship a minimal version in phase 1, or
   decide macros-only addressing is the permanent model and *remove* the reserved
   seam. A reserved-but-inert seam is the worst of both: it carries design debt
   without delivering the feature.

5. **Treat the shadowing precedence as a tested invariant, not a convention.** Add
   a focused test matrix of every verb name that appears in more than one place
   (prelude vs FFEDIT vs FFCMD), asserting exactly which layer wins. The `X` /
   EXIT / EXCLUDE collision shows these are not hypothetical. This is cheap
   insurance for the one hot path all commands traverse.

### 6.4 Bottom line

The idea is well-chosen and the design is faithful to the project's framework
principles: one dispatch path, one navigation model, additive and
behaviour-preserving. The mainframe `ADDRESS` analogy is not decoration -- it maps
almost exactly onto what the codebase already has, and the model's "environment is
the namespace" framing is a real improvement over the current hand-written ladder.

The honest weaknesses are all in the gap between the *model* and *phase 1's
realisation of it*: the one environment being built (FFEDIT) is the one that does
not cleanly implement the abstraction, the distinctive multi-FIND feature is named
but never exercised at runtime, and two cleanups (the manager sub-struct and the
address-prefix seam) are deferred in ways that tend to become permanent. None of
these are reasons to change course; they are reasons to pull a little more of the
"eventual" work into this gate so the abstraction is proven, not just promised.

---

## 7. Source documents

- `docs/specs/command-environments/requirements.md` -- authoritative model + router.
- `docs/specs/command-environments/design.md` -- architecture.
- `docs/specs/command-environments/tasks.md` -- phase-1 implementation tasks (E0-E7).
- `docs/specs/command-environments/environments-vision.md` -- full FF* catalogue.
- `.agents/tasks/command-environments/design-proposal.md` -- read-only investigation
  and the open questions (Q1-Q7) this analysis draws on.
