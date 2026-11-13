# Design Document -- Command-Resolution Model (CR-NR-121)

## Overview

This design captures the owner-approved END-STATE command-resolution model for
FileForgeWorkbench and a staged, prerequisite-ordered implementation path to reach
it. The end state makes every active Context Kind a live Command Environment (CE)
that gets FIRST crack at the first token of a command line and FALLS BACK to the
always-present FFCMD base (claim-as-fallback, not a terminal hard-claim). Locale
surfaces resolve at ROOT precedence because localisation IS locale-language aliases
for root commands; user shorthands resolve LAST so they cannot shadow a root or a
menu option. A per-row discriminator makes "locale at root, user shorthand last"
expressible, and the same discriminator drives the menu display rule (canonical
root, overridden only by a locale translation).

This is a LOAD-BEARING ARCHITECTURE design. It is grounded entirely in three
read-only investigation reports (cited by path throughout) and it BUILDS ON the
existing single-dispatch framework (framework-conformance mechanism 1): no second
dispatcher, no second navigation stack, `CommandTarget` and `WorkspaceContext`
unchanged. The framework changes it entails (claim-as-fallback, the `active_name`
derivation change, FFEDIT re-landing, the alias-row reshape) are flagged
OWNER-GATED and this gate PAUSES for owner approval; nothing is implemented here.

## Why this spec folder (home choice and justification)

The resolution model is fundamentally about Command Environments: the layered order
is CE-ROOT -> CE-alias -> FFCMD-ROOT -> MENU-OPTION -> FFCMD-alias, the pivot is
claim-as-fallback onto the FFCMD base, and the new CEs are the per-KIND config CEs,
the menu CEs, and the re-landed FFEDIT. That is the `command-environments`
(CR-CH-053) domain -- the authoritative home for the environment MODEL and ROUTER.

However, `command-environments` is deliberately PHASE-1 SCOPED (its own
`requirements.md` states it builds only the framework + FFCMD + FFEDIT-migration,
and does NOT implement CR-CH-052 or the layered order). Folding this END-STATE
model into that folder would blur its intentionally narrow phase-1 scope and mix two
CR ids in one requirements document.

Decision: author CR-NR-121 as a NEW dedicated spec folder
`docs/specs/command-resolution-model/`, which EXTENDS `command-environments`
(CR-CH-053) and cross-references `command-semantics` (first-token classification,
the Command_Engine), `command-framework` (single dispatch, CommandTarget chain),
`localization` (locale-aliases-as-roots, Req 11.6), and `command-configurator`
(the alias authoring/display surface). This keeps CR-CH-053's phase-1 scope intact,
gives the end-state model its own CR id and gate, and still lives in the correct
problem domain. The alternative (extend `command-environments` in place) was
rejected for the scope-blurring reason above; the alternative
`command-semantics` home was rejected because the model's centre of gravity is the
CE/registry mechanism, not the Primary_Command parser.

## End-state model (ASCII)

First-token resolution when the active Context has a live CE. Each layer is tried
in order; the FIRST match wins; a CE non-claim FALLS THROUGH (claim-as-fallback).

```
  command line (first token)
        |
        v
  [ = prefix? ] --yes--> reinitialise active tab to POM in place --> re-dispatch remainder
        | no                                     (Req 3.6; NOT END/EXIT/RETURN)
        v
  +-------------------------------------------------------------+
  | LAYER 1  active-CE ROOT                                     |
  |          (incl. active-CE LOCALE surfaces @ ROOT prec.)     |---claim--> dispatch
  +-------------------------------------------------------------+
        | no claim (fall through)
        v
  +-------------------------------------------------------------+
  | LAYER 2  active-CE USER-ALIAS                               |---claim--> dispatch
  +-------------------------------------------------------------+
        | no claim
        v
  +-------------------------------------------------------------+
  | LAYER 3  FFCMD ROOT                                         |
  |          (incl. FFCMD LOCALE surfaces @ ROOT prec.)         |---claim--> dispatch
  +-------------------------------------------------------------+
        | no claim
        v
  +-------------------------------------------------------------+
  | LAYER 4  MENU OPTION of active menu                         |
  |          (only when active Context is a menu Kind)          |---claim--> dispatch
  +-------------------------------------------------------------+
        | no claim
        v
  +-------------------------------------------------------------+
  | LAYER 5  FFCMD USER-ALIAS  (last; cannot shadow 1/3/4)      |---claim--> dispatch
  +-------------------------------------------------------------+
        | no claim
        v
     not-found message (CommandEngine terminal)
```

Collapse when active-CE == FFCMD (Req 1.8): when the active Context has NO
distinct per-KIND or editor CE -- the whole of today's reality, and any Context
before Stages D/E land -- the active CE IS the FFCMD base, so LAYER 1 (active-CE
ROOT) coincides with LAYER 3 (FFCMD ROOT) and LAYER 2 (active-CE USER-ALIAS)
coincides with LAYER 5 (FFCMD USER-ALIAS). The five-layer order then FOLDS to
exactly three distinct layers, evaluated once each:

```
  command line (first token, active-CE == FFCMD)
        |
        v
  LAYER 3  FFCMD ROOT (incl. FFCMD locale surfaces @ ROOT prec.) --claim--> dispatch
        | no claim
        v
  LAYER 4  MENU OPTION of active menu (only when a menu Kind)    --claim--> dispatch
        | no claim
        v
  LAYER 5  FFCMD USER-ALIAS (last)                               --claim--> dispatch
        | no claim
        v
     not-found message (CommandEngine terminal)
```

Layers 1/2 are NOT evaluated a second time in this degenerate case; this is the
baseline order that Stages A-C run in (Stage D deferred, Stage E owner-gated).

The end-state set of live CEs and their fallbacks:

```
  active Context Kind            active CE            falls back to
  ---------------------------    -----------------    -------------
  editor buffer (DEFERRED)       FFEDIT               FFCMD
  Config/Theme/Menus/Keys/       per-KIND config CE   FFCMD
    Kinds/Alias/CommandConfig     (reuse EditableKind
                                   lifecycle router)
  POM / Settings / named menu    menu CE              FFCMD
  anything else                  FFCMD (base)         (terminal)
```

Chained, lifecycle-first, late-bound (Req 3.4 / 4):

```
  "END;Z"  --recognise closed lead verb {END,RETURN,CANCEL}--> split [END][Z]
     |
     v  segment 1: END  -> nav_end pops ONE level -> Settings now active (synchronous)
     v  segment 2: Z    -> re-enter front door -> resolves against SETTINGS (late-bound)

  "RETURN;Z" -> segment 1 RETURN -> unwind to Tab_Visual_Root (POM default)
             -> segment 2 Z resolves against POM
```

## Current-vs-target gap analysis (citing the reports)

### Gap 1 -- First-token order is inverted (today ALIAS -> MENU OPTION -> ROOT)

Per `.agents/tasks/cmdline-resolution-order/findings.md`, the live order is an
ALIAS/locale verb-rewrite PRE-PASS (`rewrite_verb_to_root`, `shell/locale_resolver.rs`)
FIRST, then the current-menu Option_Key lookup (`try_current_menu_option`,
`shell/commands_menu.rs`) inside the prelude, then the ROOT classifier
(`ff_command::resolve_target`). That is the inverse of the end-state order for the
alias/root relationship, and MENU OPTION already precedes ROOT today. The report
also confirms `=` strips once and calls `reinitialise_active_tab_to_pom`
(`shell/nav_stack.rs`): clear nav_stack + set Home; `=0` == `=settings` (POM option
`0` -> Settings); `=a` fails via the CommandEngine terminal `open_error`.

Target (Req 1): demote the alias verb-rewrite from a universal pre-pass to the
LAST user-alias layer, keep locale surfaces at ROOT precedence, and sequence the
five layers CE-ROOT -> CE-alias -> FFCMD-ROOT -> MENU-OPTION -> FFCMD-alias.

### Gap 2 -- No live CE but FFCMD; the claim is a DEAD terminal hard-claim

Per `.agents/tasks/ce-layered-resolution/findings.md`, `builtin_entries()` registers
only FFCMD; `is_ffedit_active(..)` is hard-coded `false`; `dispatch_to_environment("FFEDIT", ..)`
resolves to `None` -> `NoSuchEnvironment`. Even structurally, a `Claimed` outcome
returns immediately with NO subsequent FFCMD fallback -- it is a HARD-CLAIM shape,
not a fallback, and it is unreachable today. So there is no genuine CE-ROOT ->
FFCMD-ROOT fallback because there is no second live CE.

Target (Req 1.6, 2): re-land FFEDIT and add per-KIND config CEs, AND change the
claim from terminal hard-claim to "CE first, else FFCMD" (the pivot; Stage C).

### Gap 3 -- Locale and user overlays are indistinguishable

Per `.agents/tasks/ce-layered-resolution/findings.md` and
`.agents/tasks/cmdline-resolution-order/findings.md`, both a locale-seed row and a
user-shorthand row flow through the same `AliasTable::apply_locale_overlay_keyed`
and carry identical metadata (`is_overlay = true`, no provenance). `AliasEntry`
(`shell/environment.rs`) has only `surface` / `canonical` / `command_template` /
`is_overlay`; the TOML row (`UserAliasRowToml`, `shell/user_aliases.rs`) has no
locale-vs-shorthand field. "Locale at root precedence, user shorthand last" is
therefore NOT expressible until a discriminator exists. This is the true
cross-cutting blocker (all three reports converge on it).

Target (Req 5): add a per-row discriminator (Stage B), plumbed through the overlay
apply path and the display/precedence consumers.

### Gap 4 -- Editable Contexts have no resolution CE identity (but a lifecycle seam exists)

Per `.agents/tasks/context-ce-identity/findings.md`, all seven editable Contexts
collapse to FFCMD in `EnvironmentRegistry::active_name` (the blanket `_ => FFCMD_NAME`
arm). BUT the `EditableKind` router (`shell/editable_context.rs`) already enumerates
all seven and routes `editable_persist` / `editable_discard` / `editable_reload` per
`KindTag`. SAVE / END / RETURN / CANCEL / REFRESH are FFCMD roots in `try_commands_a`
(`shell/commands_ladder_a.rs`) that route to this seam. The natural CE attachment seam
is `active_name()` + `builtin_entries()` PLUS this existing router.

Target (Req 2.2 / 2.3): map each editable Kind to its own CE in `active_name`,
register it in `builtin_entries`, and have each claim body REUSE the `EditableKind`
router (Stage E).

### Gap 5 -- Lifecycle-first bare chains are not recognised

Per `.agents/tasks/ce-layered-resolution/findings.md` and
`.agents/tasks/context-ce-identity/findings.md`, `try_chained_fastpath`
(`shell/commands_fastpath.rs`) only accepts a bare chain whose first segment is a
single digit (or an `=`-origin chain). A bare `END;Z` is therefore NOT split and
errors as one token. The downstream per-segment dispatch loop already does the
correct late-bound thing; the landing behaviour (END -> Settings, RETURN ->
Tab_Visual_Root) is already correct (`nav_end` pops one level, `nav_return` collapses
to the visual root, both synchronous). Only RECOGNITION is missing.

Target (Req 4): widen the recognition guard to a CLOSED lifecycle lead-verb set
(Stage A); the dispatch loop and landings are unchanged.

### Gap summary

| Gap | Today | Target | Stage | Kind |
|-----|-------|--------|-------|------|
| 1 order inverted | ALIAS->OPTION->ROOT | CE-ROOT->CE-alias->FFCMD-ROOT->OPTION->FFCMD-alias | F | FRAMEWORK+MODEL |
| 2 dead hard-claim | no live CE but FFCMD, terminal claim | CE-first, else FFCMD | C | FRAMEWORK |
| 3 overlays indistinguishable | both is_overlay=true | per-row discriminator | B | MODEL |
| 4 no CE identity | all -> FFCMD | per-KIND CEs reuse EditableKind | E | FRAMEWORK |
| 5 chain not recognised | digit-first / =-origin only | closed lifecycle lead set | A | ADDITIVE |

## Staged implementation path (A-F)

Each stage is labelled ADDITIVE or FRAMEWORK per framework-conformance.md, with
prerequisites. FRAMEWORK stages are OWNER-GATED: they touch the single-dispatch /
active-env mechanism and must not proceed without express owner confirmation.

### Stage A -- Lifecycle-first chain recognition  [ADDITIVE]

Widen the recognition guard in `try_chained_fastpath` to also accept a bare chain
whose first segment is in the CLOSED lifecycle lead set {END, RETURN, CANCEL}. The
set is short and closed so ordinary dotted/semicolon text (DSNs like `abc.def`) is
NOT hijacked. The downstream per-segment `dispatch_command_string` loop and the
`nav_end` / `nav_return` landings are UNCHANGED. Localised to one recognition branch.

- Prerequisites: none (independent; immediately useful for `END;Z`).
- Framework stance: ADDITIVE (a new recognition branch, no dispatcher/nav change).
- Satisfies: Req 4.1-4.4.

EXIT decision (Req 4.5): RECOMMEND EXCLUDING EXIT from the lead set. Nothing can
follow an application-terminating verb, so `EXIT;...` has no meaningful trailing
segment; keeping the set {END, RETURN, CANCEL} keeps it minimal and avoids a
split whose tail never runs. If the owner prefers consistency, including EXIT is
safe (the tail simply never executes), but the recommendation is to omit it.

### Stage B -- Overlay seed-vs-user discriminator  [MODEL]

Add a per-row discriminator to `AliasEntry` and plumb it through
`apply_locale_overlay_keyed` and the TOML row shapes, then update the
display/precedence consumers (`display_surface_for_canonical`, `display_rows`,
`menu_option_command_surfaces`, and the future ordering logic). This is the
foundational prerequisite for the ordering rule (locale at root, user last) and for
the menu-display rule. It is a MODEL change (a new field + plumbing), not a
load-bearing-type reshape by itself.

- Prerequisites: none (independent of CE re-landing; gates ordering + display).
- Framework stance: MODEL (new field + plumbing). Not a framework mechanism change,
  but it reshapes a persisted row shape, so the TOML default must keep existing
  files valid.
- Satisfies: Req 5.1-5.5.

### Stage C -- Claim-as-fallback  [FRAMEWORK, OWNER-GATED]

Change the active-environment claim at the top of `run_command_ladder` from a
TERMINAL hard-claim (a `Claimed` outcome returns with no FFCMD fallback) to
"CE first, else FFCMD" (a non-claim falls through to the FFCMD layers). This is
the PIVOT of the whole model -- without it, any registered CE would shadow FFCMD
roots and menu options entirely.

- Prerequisites: owner confirmation.
- Framework stance: FRAMEWORK (reshapes the active-wins claim gate,
  framework-conformance mechanism 1). OWNER-GATED.
- Satisfies: Req 1.6, 2.5, 2.6.

### Stage D -- Re-land / register FFEDIT  [FRAMEWORK, OWNER-GATED, DEFERRED]

Un-park the editor buffer CE, make `is_ffedit_active` real, and register FFEDIT in
`builtin_entries()` so the editor Context resolves CE-first then falls back to FFCMD.
IN SCOPE of this design (it is part of the end state), but EXPLICITLY DEFERRED for
implementation until editor work resumes -- it is NOT immediate.

- Prerequisites: Stage C (so FFEDIT falls back, not hard-claims); the parked editor
  domain resuming.
- Framework stance: FRAMEWORK (single-dispatch + registry). OWNER-GATED and DEFERRED.
- Satisfies: Req 2.1, 8.3.

### Stage E -- Per-KIND config CEs  [FRAMEWORK, OWNER-GATED]

Map each editable Kind (Config, Theme, Menus, Keys, Kinds, Alias, Command
Configurator) to its own environment in `EnvironmentRegistry::active_name` (replacing
the blanket `_ => FFCMD_NAME`), register those environments in `builtin_entries()`,
and have each claim body REUSE the existing `EditableKind` lifecycle router for its
verb bodies, then FALL BACK to FFCMD for anything it does not claim. Registry
MEMBERSHIP is near-additive; the `active_name` derivation change is the framework part.

- Prerequisites: Stage C (claim-as-fallback) and Stage B (so each CE's locale vs user
  aliases order correctly). The `EditableKind` router already exists and is reused.
- Framework stance: FRAMEWORK (changes active-env derivation) + near-additive registry
  membership. OWNER-GATED.
- Satisfies: Req 2.2, 2.3.

### Stage F -- Full layered order  [FRAMEWORK + MODEL, OWNER-GATED]

Implement the full end-state order across the live CEs: the ROOT-before-OPTION reorder
and user-alias-last, with CE layers ahead of FFCMD layers, locale surfaces at ROOT
precedence. This is the convergence of all prior stages.

- Prerequisites: Stages B, C, D (for the FFEDIT path; may land with FFEDIT deferred as
  long as the order is correct for the live CEs), E. Also the two constraints the
  reports named: the AliasTable identity rows double as the root table (localization
  Req 11.6) -- preserved so a bare root still self-resolves (Req 6.3); and
  parameterised aliases (CR-NR-112) must still expand templates, just later (Req 6.2).
- Framework stance: FRAMEWORK + MODEL. OWNER-GATED.
- Satisfies: Req 1.1-1.7, 6.1-6.3.

Dependency graph:

```
  A (independent)
  B (independent) ----+-----------------+
                      |                 |
  C (owner-gated) ----+--> E (B,C) -----+--> F (B,C,D,E)
                      |                 |
                      +--> D (C) -------+   (D deferred; F may land with D deferred)
```

## Discriminator model options (both presented; recommendation argued)

Requirement 5.5 demands the model choose how the discriminator is represented. Two
options:

### Option 1 -- ORIGIN enum derived at load time (RECOMMENDED)

Add an `origin` field to `AliasEntry` and the row shape, an enum such as
`Origin { Seed, LocaleSeed, UserOverride }`, DERIVED at load time from WHICH overlay
layer produced the row: the shipped locale seed (`i18n/<locale>/aliases/<ENV>.toml`)
yields `LocaleSeed`; the user override store (`<User_Data_Dir>/.../aliases/<locale>/<ENV>.toml`)
yields `UserOverride`; synthesised identity rows yield `Seed`. The overlay apply path
(`apply_locale_overlay_keyed`) already KNOWS which layer it is applying when it runs,
so the origin can be stamped there with no authored field.

- Pros: cheap (the loader already knows the layer); no new authored TOML key needed;
  no migration of existing user files (origin is derived, not read); keeps the TOML
  shape minimal.
- Cons: "locale-seed == translation" is an approximation -- a user could in principle
  author a translation in their override store, and the shipped seed IS locale data.
  For the SS-vs-SNAPSHOT and locale-at-root rules this approximation is exactly right
  (seed = translation, user store = shorthand), so the approximation is acceptable for
  the decided rules.

### Option 2 -- Authored locale-vs-shorthand CATEGORY

Add an authored `category` key to the TOML row, such as
`category = "locale" | "shorthand"`, persisted on both the seed and user row shapes,
with a default (for example `shorthand` for a user row, `locale` for a seed row) that
keeps existing files valid. Consumers read the authored category directly.

- Pros: semantically honest -- a user CAN author a translation and mark it `locale`;
  does not conflate "which store" with "what kind".
- Cons: requires a new persisted key and a default-migration story; more authoring
  burden; more plumbing through the TOML readers/writers and the editor UI.

### Recommendation

RECOMMEND Option 1 (origin-derived). Argument: the decided rules (Req 5.2-5.4) map
cleanly onto the store layer -- the shipped locale seed IS the translation set and
the user override store IS the shorthand set -- so deriving origin from the layer
captures exactly the distinction the rules need, at the lowest cost (no authored key,
no file migration, the loader already knows the layer). Option 2's extra honesty
(user-authored translations) is not required by any decided rule and can be added
LATER as an additive authored override of the derived origin if a future requirement
needs it. The design therefore specifies origin-derived as the implementation model,
with the authored category retained as a documented future extension.

## Menu-display-surface rule

Today `display_surface_for_canonical` (`shell/environment.rs`) returns the FIRST
AliasTable row whose canonical matches, and `apply_locale_overlay_keyed` inserts an
overlay row at the FRONT, so a user shorthand (`SS -> SNAPSHOT`) is returned ahead of
the identity row and the SCRM menu shows `SS`
(`.agents/tasks/cmdline-resolution-order/findings.md`, section e).

End-state rule (Req 5.4): the menu display surface for an option's canonical root
SHALL show the canonical root overridden ONLY by a LOCALE translation, NEVER by a
USER shorthand. With the Stage B discriminator in place,
`display_surface_for_canonical` / `menu_option_command_surfaces` select the canonical
unless there is a LocaleSeed-origin overlay for it; a UserOverride-origin overlay is
ignored for display (it remains typeable, but it does not replace the canonical in the
menu command column). This is the rule that will later flip the SCRM menu from `SS`
back to `SNAPSHOT`.

Known downstream consequence (Req 5.6): CR-NR-120 is landing NOW with the current
`SS`-surface display and a refreshed golden. When CR-NR-121's menu-display rule lands,
that display flips to the canonical `SNAPSHOT` and the CR-NR-120 golden is re-updated.
This gate RECORDS that consequence; it does NOT deliver the golden re-update.

## Error handling

- A CE that does not own a token reports a NON-CLAIM (not an error), so the fallback
  chain continues to the next layer; only the FFCMD CommandEngine terminal produces a
  not-found message (`open_error`), as today (Req 2.6, 3.8).
- A SAVE that fails inside an editable-context END keeps the Context open and surfaces
  the error (existing `editable_persist` behaviour; the END-implies-SAVE path stays).
- A malformed TOML alias row under the discriminator must default its origin so the
  file remains loadable (Stage B default-migration, Req 5.5); a missing/corrupt locale
  seed falls back to identity (existing graceful degradation).
- A lifecycle-led chain whose first segment is a lifecycle verb that is NOT a live
  root would split and then error on segment 1; the closed lead set is therefore
  restricted to verbs that exist as roots (END/RETURN/CANCEL).

## Testability

- Resolution-ORDER: unit-testable with a fixture table asserting the five-layer
  precedence and claim-as-fallback (AC-1, AC-2, AC-3) without a running GUI.
- Discriminator: unit-testable at the overlay apply path and the display-surface
  function (AC-7); a focused test asserts a LocaleSeed overlay wins display and a
  UserOverride overlay does not.
- Chain recognition: unit-testable on `try_chained_fastpath` (AC-6) -- `END;Z` splits,
  `abc.def` does not.
- Landing / late-bound chaining: integration-testable via the front door
  (`END;Z` -> Settings, `RETURN;Z` -> Tab_Visual_Root), consistent with the existing
  nav tests (AC-5).
- Per-KIND CE + FFEDIT: integration-testable once Stages C/E (and D, deferred) land;
  until then the end state is documented and the CE set is asserted structurally (AC-4).

## Framework changes (OWNER-GATED -- explicit list)

Per framework-conformance.md, these entail changes to a core mechanism and are
OWNER-GATED; they must not proceed without express owner confirmation:

1. Claim-as-fallback (Stage C): reshapes the active-wins claim gate (mechanism 1).
2. The `EnvironmentRegistry::active_name` derivation change (Stage E): changes the
   active-environment derivation (mechanism 1).
3. FFEDIT re-landing / registration (Stage D): single-dispatch + registry
   (mechanism 1). IN SCOPE but DEFERRED.
4. Any reshape of `EnvironmentRegistry` / the claim gate / the `dispatch_to_environment`
   claim outcome to support fallback (Stages C/E).
5. The full layered-order reorder incl. ROOT-before-OPTION and user-alias-last
   (Stage F): single-dispatch ordering (mechanism 1) + model.

Stages A (chain recognition) and B (discriminator) are NOT framework-mechanism changes
(A is additive recognition; B is a row-shape/model change), though B reshapes a
persisted TOML row and so needs a loadable default.
