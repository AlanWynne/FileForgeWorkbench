# CR-NR-120 -- Owner decision: how to store the two-word CAPTURE seed aliases

Status: AWAITING OWNER DECISION (A or B). Gate docs authored with Option A active;
Option B recorded as the gated alternative.

## Shared, already-agreed facts (NOT in question)

- The bare/first-run + Reset-bare seed set grows from ONE alias (`X -> EXIT`) to
  FIVE: `X -> EXIT`, `SS -> SNAPSHOT`, `CS -> CAPTURE START`, `CT -> CAPTURE STOP`,
  `CR -> CAPTURE REPLAY`.
- Every seed row ships with an EMPTY stored description. The one-line description
  is INHERITED at DISPLAY time from the target command's `cmd:<ROOT>` help topic
  via the CR-NR-119 `short_description(cmd:<ROOT>)` lookup
  (command-configurator Req 9). NO literal description string is persisted on any
  seed row. (This is settled; it is NOT the A/B decision.)
- `X -> EXIT` and `SS -> SNAPSHOT` are unaffected by the A/B choice: `EXIT` and
  `SNAPSHOT` are both valid FFCMD roots and both have a `cmd:EXIT` / `cmd:SNAPSHOT`
  topic, so they pass validation and inherit cleanly under either option.

## What the A/B decision actually is

It is ONLY about how the three two-word CAPTURE shortcuts (`CS`, `CT`, `CR`) are
STORED as alias rows, because "CAPTURE START/STOP/REPLAY" is a root verb PLUS a
subcommand argument, not a single root verb.

### Why a decision is needed -- two verified facts about CURRENT main (HEAD 09757dd)

Verified by reading the code and the help tree on main (not the stale scratch note):

1. ROOT VALIDITY. `EnvironmentRegistry::contains_root("FFCMD", r)` is an EXACT,
   case-insensitive FULL-STRING match. The FFCMD root set
   (`builtin_ffcmd_roots()` / `BUILTIN_FFCMD_LADDER_VERB_ROOTS`) contains the
   single-word root `CAPTURE` -- it does NOT contain `CAPTURE START`. The alias
   load/save validator (`validate_user_alias_rows`) and the user-overlay load
   path (`alias_user_overlay.rs`) BOTH reject a row whose canonical is not a valid
   root. So a seed row stored as canonical `CAPTURE START` is REJECTED and skipped
   with a WARN -- the seed would not survive, which defeats the whole point of the
   change.

2. HELP TOPICS. `help/commands/scrm.help.md` on main defines exactly these TOPIC
   markers: `cmd:SCRM`, `cmd:SNAPSHOT`, `cmd:CAPTURE`, `cmd:RETRIEVE`. There is NO
   `cmd:CAPTURE START`, `cmd:CAPTURE STOP`, or `cmd:CAPTURE REPLAY` topic (grep of
   `help/**/*.md` for `TOPIC: cmd:CAPTURE ...` finds only the single `cmd:CAPTURE`).
   So `short_description("cmd:CAPTURE START")` returns None -> the inherited
   description for `CS/CT/CR` would be EMPTY. Inheritance only lands on a one-liner
   if the lookup key is `cmd:CAPTURE` (which exists).

   NOTE: an earlier relay stated CR-NR-119 authored `cmd:CAPTURE START/STOP/REPLAY`
   topics. That is NOT the case on current main; CR-NR-119 authored `cmd:SNAPSHOT`
   and a single `cmd:CAPTURE`. The options below are grounded on the verified tree.

## Option A -- RECOMMENDED (framework-conforming, NO framework change)

Store each CAPTURE shortcut as:
- canonical root = `CAPTURE` (a valid FFCMD root)
- parameter template (`command_template`) = `START` / `STOP` / `REPLAY`

At runtime the typed surface expands through the EXISTING `rewrite_verb_to_root`
pre-pass to `CAPTURE START` / `CAPTURE STOP` / `CAPTURE REPLAY`, then dispatches
through the single front door (the existing `CAPTURE` arm in
`commands_ladder_b.rs`, which already parses START/STOP/REPLAY from the rest).

Tradeoffs:
- PRO: passes `contains_root` -> seed rows survive first-run write and Reset bare.
- PRO: inherits its one-liner from the EXISTING `cmd:CAPTURE` topic (all three
  CAPTURE aliases share the single CAPTURE one-liner).
- PRO: NO framework change, NO new help topics, NO new dispatch path. Reuses the
  already-merged `command_template` expansion. Lowest risk; implementable now.
- CON: the alias's canonical column shows `CAPTURE` (with the subcommand in the
  template column), not the literal `CAPTURE START`, in the Alias Editor.
- CON: `CS`, `CT`, `CR` all show the SAME inherited description (the CAPTURE
  one-liner), because there is one CAPTURE topic -- the display does not
  distinguish start/stop/replay.

## Option B -- store the literal two-word canonical `CAPTURE START` (as the brief said)

Store each CAPTURE shortcut as canonical = `CAPTURE START` / `CAPTURE STOP` /
`CAPTURE REPLAY` (no template).

This REQUIRES, before it can work:
1. A FRAMEWORK CHANGE to root validity: relax `contains_root` (and the FFCMD root
   set) to accept compound / first-word-matched roots so `CAPTURE START` validates.
   This ripples into the no-shadow rule and the root-validity rule used by BOTH
   the save validator and the load overlay. Per framework-conformance this needs
   EXPRESS owner confirmation -- it is not an implementation side effect.
2. Authoring THREE new help topics `cmd:CAPTURE START`, `cmd:CAPTURE STOP`,
   `cmd:CAPTURE REPLAY` under CR-NR-119 (a separate help-model task) so each
   CAPTURE alias inherits its OWN distinct one-liner.

Tradeoffs:
- PRO: the Alias Editor shows the literal `CAPTURE START` as the canonical, and
  each of `CS/CT/CR` can inherit a DISTINCT one-liner (start vs stop vs replay).
- CON: a genuine FRAMEWORK CHANGE to a load-bearing validation rule
  (`contains_root` + no-shadow + root-validity), owner-gated, higher blast radius
  and risk.
- CON: requires new help-topic authoring work (3 topics) that does not exist yet.
- CON: larger change for a cosmetic gain (distinct descriptions + literal canonical
  display) over Option A.

## Recommendation

Option A. It fixes the reported defect (SCRM shortcuts survive Reset bare) with no
framework change, no new help topics, and reuses merged mechanisms. Choose Option B
only if the owner wants each CAPTURE shortcut to show a distinct one-liner and the
literal two-word canonical in the editor, and accepts the gated framework change +
new help-topic authoring that entails.
