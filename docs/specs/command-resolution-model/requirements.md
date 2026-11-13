# Requirements Document -- Command-Resolution Model (CR-NR-121)

## Introduction

This sub-project defines the owner-approved END-STATE command-resolution model
for FileForgeWorkbench and the STAGED path that gets there. It captures, as
testable acceptance criteria, how the FIRST token of a command line is resolved
when the active Context Kind is a live Command Environment (CE), how a CE claims
as a FALLBACK (not a terminal hard-claim) onto the global FFCMD base, how the
seven editable Contexts become per-KIND CEs, how lifecycle-first command chains
are recognised, and how a per-row discriminator separates locale-translation
overlays from user-shorthand overlays so each resolves at its correct precedence.

It is a REQUIREMENTS + DESIGN gate only. No source (.rs) is authored. It is a
LOAD-BEARING ARCHITECTURE change: several criteria describe framework changes
(claim-as-fallback, the active-environment derivation change, FFEDIT re-landing,
reshaping the Environment_Registry / claim gate). Every such criterion is flagged
OWNER-GATED here and in `design.md` / `tasks.md` per the framework-conformance
steering rule, and the workflow PAUSES for owner approval after this gate.

### Evidence base (read-only investigation reports)

This model is grounded in three read-only investigation reports; the design cites
them by path and this document's criteria encode their confirmed findings:

- `.agents/tasks/cmdline-resolution-order/findings.md` -- first-token order
  (today: ALIAS pre-pass -> MENU OPTION -> ROOT, the inverse of the desired
  model; `=` reinitialises the active tab to the POM in place; `=0` == `=settings`;
  `=a` fails with a message).
- `.agents/tasks/ce-layered-resolution/findings.md` -- only FFCMD is a live CE;
  FFEDIT is parked (`is_ffedit_active` hard-false); the active-env claim is a DEAD
  terminal HARD-CLAIM with no FFCMD fallback; locale-seed and user-shorthand
  alias overlays are INDISTINGUISHABLE (both `is_overlay = true`, no provenance);
  chained segments re-resolve per-segment via front-door re-entry with
  synchronous navigation; `=end;Z` reinitialises to the POM before segment 1.
- `.agents/tasks/context-ce-identity/findings.md` -- the seven editable Contexts
  (Config / Theme / Menus / Keys / Kinds / Alias / Command Configurator) all
  collapse to FFCMD for resolution; the `EditableKind` lifecycle router in
  `shell/editable_context.rs` is the embryo per-KIND seam and already routes
  SAVE / END / RETURN / discard / reload per KindTag; END pops one nav level,
  RETURN unwinds to the Tab_Visual_Root; the natural CE attachment seam is
  `active_name()` / `builtin_entries()` PLUS the existing EditableKind router;
  the true cross-cutting blocker is the overlay seed-vs-user discriminator.

### Framework-conformance stance

This model BUILDS ON the existing single-dispatch framework (framework-conformance
mechanism 1). It adds NO second dispatcher and NO second navigation stack. FFCMD
remains `ff_command::resolve_target`. The change is to the ORDER and the CLAIM
SEMANTICS of the existing one front door (`dispatch_command_string`), to the
active-environment derivation (`EnvironmentRegistry::active_name`), and to the
alias-overlay row shape (a new discriminator). These are explicitly flagged as
OWNER-GATED framework changes; they are decisions to be approved, not re-litigated.

### Source references

- **[CR-NR-121]** = this change request (owner: the END-STATE layered
  command-resolution model and its staged path).
- **[CR-CH-053]** = Command Environments model (REXX ADDRESS-inspired); this
  model EXTENDS its phase-1 framework and router.
- **[CR-NR-119]** = Command Help model (referenced: FFCMD root commands and the
  `HELP <cmd>` / `--help` seam; resolution must not disturb it).
- **[CR-NR-120]** = the alias seed-set landing now with the current SS-surface
  menu display and a refreshed golden; CR-NR-121 will later flip that display to
  canonical and re-update the golden (recorded as a known downstream consequence).
- **[B080]** = the single command-dispatch front door (`dispatch_command_string`).
- **[CR-CH-116]** = ISPF END / RETURN / EXIT navigation model and the write-through
  alias seed.

### Cross-references

- `command-environments` (CR-CH-053) -- the Command_Environment / Environment_Registry
  framework this model extends; FFCMD base, Active_Environment, claim semantics.
- `command-semantics` -- the Command_Engine, the Primary_Command parser, and the
  first-token classification this model reorders.
- `command-framework` Req 2 (single dispatch), Req 8 (CommandTarget + ordered
  Target_Resolution chain and shadowing), Req 10 (Navigation_Stack).
- `localization` (Req 11.6) -- localisation IS locale-language aliases for root
  commands; locale surfaces MUST resolve at ROOT precedence. This is the correctness
  pivot for the discriminator requirement.
- `command-configurator` -- the alias editor surface that authors / displays user
  shorthands and must honour the discriminator.
- `navigation-commands` (Req 21), `menu-workspace` (Req 14) -- END / RETURN / EXIT
  landing behaviour this model pins as non-regressable.

## Glossary

| Term | Definition |
|------|-----------|
| **Active_CE** | The Command Environment selected by the currently focused Workspace Context Kind (editor -> FFEDIT, each editable Kind -> its per-KIND CE, a menu Kind -> its menu CE, otherwise FFCMD). |
| **Claim-as-fallback** | A CE gets FIRST crack at a token; if it does NOT claim, resolution FALLS THROUGH to the FFCMD base (and the other layers). The opposite of a terminal hard-claim. |
| **CE-ROOT** | A root verb owned by the active CE, including that CE's LOCALE surfaces (a locale alias is a root in another language, so it resolves at ROOT precedence). |
| **CE user-alias** | A user-authored shorthand scoped to the active CE. |
| **FFCMD-ROOT** | A root verb of the always-present FFCMD base, including FFCMD locale surfaces (root precedence). |
| **MENU OPTION** | An Option_Key of the active menu when the active Context is a menu Kind. |
| **FFCMD user-alias** | A user-authored FFCMD shorthand; resolves LAST so it cannot shadow a root or a menu option. |
| **Overlay Origin** | A per-row discriminator distinguishing a Seed / LocaleSeed overlay row from a UserOverride overlay row. |
| **Lifecycle lead verb** | A member of the CLOSED set {END, RETURN, CANCEL} (EXIT decision per Req 4.5) that, when leading a bare chain, makes the chain fastpath recognise and split it. |

## Functional Requirements

### Requirement 1 -- END-STATE layered first-token resolution order (Model part A)

The core ordering decision. When the active Context Kind is a live CE, the first
token resolves CE-first and falls back to FFCMD, with locale surfaces at ROOT
precedence and user shorthands LAST.

- **1.1** WHEN a command line is submitted and the active Context has a live CE
  THE resolver SHALL attempt resolution in this order and stop at the first match:
  (1) active-CE ROOT, (2) active-CE USER-ALIAS, (3) FFCMD ROOT, (4) MENU OPTION of
  the active menu (when the active Context is a menu Kind), (5) FFCMD USER-ALIAS.
- **1.2** WHEN the active-CE ROOT layer is evaluated THE resolver SHALL treat the
  active CE's LOCALE surfaces as ROOT-precedence entries (a locale alias for a
  root resolves at layer 1, not as a user alias).
- **1.3** WHEN the FFCMD ROOT layer is evaluated THE resolver SHALL treat FFCMD
  LOCALE surfaces as ROOT-precedence entries (layer 3).
- **1.4** WHEN the first token is both a valid root (CE or FFCMD) AND a user
  shorthand THE resolver SHALL resolve it as the ROOT, because the user-alias
  layers (2 and 5) are evaluated AFTER the root layers.
- **1.5** WHEN the first token is both a current-menu Option_Key AND an FFCMD
  user-alias THE resolver SHALL resolve it as the MENU OPTION (layer 4 precedes
  layer 5).
- **1.6** WHEN the active CE does not claim the token at any CE layer THE resolver
  SHALL fall through to the FFCMD layers (claim-as-fallback), NEVER terminating
  resolution on a non-claim.
- **1.7** WHEN the active Context is NOT a menu Kind THE resolver SHALL skip the
  MENU OPTION layer (layer 4) and evaluate the remaining layers in order.
- **1.8** WHEN the active Context's CE IS the FFCMD base itself (no distinct
  per-KIND or editor CE -- the whole of today's reality and any Context before
  Stages D/E land) THE resolver SHALL evaluate exactly THREE layers in order --
  FFCMD ROOT (incl. FFCMD locale surfaces) -> MENU OPTION (when a menu Kind) ->
  FFCMD USER-ALIAS -- because the active-CE ROOT / USER-ALIAS layers (1 and 2)
  COINCIDE with the FFCMD ROOT / USER-ALIAS layers (3 and 5) and SHALL NOT be
  evaluated twice.

### Requirement 2 -- The end-state set of Command Environments (Model part B)

- **2.1** WHEN the editor buffer Context is focused THE active CE SHALL be FFEDIT
  (the editor buffer CE), which owns the editor command-line verbs and falls back
  to FFCMD for anything it does not claim. (FFEDIT re-landing is DEFERRED for
  implementation; see Req 8.)
- **2.2** WHEN an editable config-like Context (Config, Theme, Menus, Keys, Kinds,
  and in practice Alias and Command Configurator) is focused THE active CE SHALL
  be that Context's per-KIND CE, which owns its editing verbs (SAVE / END / RETURN /
  field edits) and falls back to FFCMD.
- **2.3** WHEN a per-KIND config CE resolves its lifecycle / edit verbs THE CE
  SHALL REUSE the existing `EditableKind` lifecycle router (`shell/editable_context.rs`)
  for the verb bodies and SHALL NOT duplicate that router.
- **2.4** WHEN a menu Context (POM, Settings, a named menu) is focused THE active
  CE SHALL be that menu CE, which resolves the active menu's Option_Keys and falls
  back to FFCMD.
- **2.5** WHEN any CE reports a non-claim for a token THE FFCMD base SHALL resolve
  it as the always-present terminal fallback CE that IS `ff_command::resolve_target`,
  and SHALL NOT be bypassed by any CE's non-claim.
- **2.6** WHEN any CE is asked to resolve a token it does not own THE CE SHALL
  report a non-claim (so the fallback chain continues) rather than returning an
  error that ends resolution.

### Requirement 3 -- Navigation is not required to resolve; END / RETURN landing and late-bound chaining (Model part C)

- **3.1** WHEN the active menu or Context resolves the first token THE resolver
  SHALL resolve in place against the currently active Context WITHOUT performing a
  physical navigation as a precondition of resolution.
- **3.2** WHEN END is issued from a Context opened one nav level below a parent
  (for example Config opened under Settings) THE shell SHALL pop ONE navigation
  level and land on the parent Context (Settings).
- **3.3** WHEN RETURN is issued from such a Context THE shell SHALL unwind the
  navigation stack to the Tab_Visual_Root (the POM in the default-start case, else
  the tab's START root).
- **3.4** WHEN a chained follow-on token (for example the `Z` in `END;Z` or
  `RETURN;Z`) is dispatched THE resolver SHALL re-resolve that token against
  WHICHEVER Context is active AFTER the preceding lifecycle verb has taken effect
  (late-bound per-segment re-resolution).
- **3.5** WHEN `END;Z` is issued from Config-under-Settings THE `Z` SHALL resolve
  against the SETTINGS menu; WHEN `RETURN;Z` is issued from the same THE `Z` SHALL
  resolve against the Tab_Visual_Root (the POM in the default case).
- **3.6** WHEN a command line begins with `=` THE shell SHALL reinitialise the
  active tab to the POM IN PLACE (clear the tab's Navigation_Stack and set the Home
  Context) -- this is NOT END / EXIT / RETURN semantics -- and SHALL then re-dispatch
  the remainder against the POM.
- **3.7** WHEN `=0` is submitted THE shell SHALL behave identically to `=settings`
  (POM Option_Key `0` resolves to the Settings command).
- **3.8** WHEN `=a` is submitted and `a` is neither an FFCMD root, nor a POM
  Option_Key, nor a defined alias THE shell SHALL fail with a not-found message.
- **3.9** WHEN a user has defined an alias `a` THE shell SHALL, on `=a`, perform
  whatever that alias resolves to (subject to the Requirement 1 precedence once the
  end-state order lands).

### Requirement 4 -- Lifecycle-first chain recognition (Model part D)

- **4.1** WHEN a bare (non-`=`) command line is a chain (separated by `.` or `;`)
  whose FIRST segment is a lifecycle lead verb THE chain fastpath SHALL recognise
  and split it into segments.
- **4.2** WHEN the chain fastpath tests a bare chain's first segment for lifecycle
  recognition THE trigger set SHALL be a CLOSED set containing {END, RETURN,
  CANCEL}; EXIT membership is decided in Req 4.5.
- **4.3** WHEN a bare chain's first segment is ordinary dotted or semicolon text
  that is NOT a lifecycle lead verb (for example a DSN like `abc.def`) THE chain
  fastpath SHALL NOT hijack it as a lifecycle chain.
- **4.4** WHEN a lifecycle-led chain is split THE shell SHALL dispatch each segment
  sequentially through the single front door so that per-segment late-bound
  re-resolution (Req 3.4) applies unchanged downstream.
- **4.5** THE design SHALL decide and record whether EXIT is a member of the closed
  lead-verb set; WHEN EXIT is excluded THE rationale (nothing can follow an app
  termination) SHALL be recorded, and WHEN included THE split SHALL remain safe
  because trailing segments after EXIT never run.

### Requirement 5 -- Overlay seed-vs-user (locale-vs-shorthand) discriminator (Model part E)

- **5.1** WHEN an alias overlay row is loaded into the in-memory table
  (`AliasEntry` / the alias table) or persisted to a TOML row THE row SHALL carry a
  per-row DISCRIMINATOR that distinguishes a locale-translation overlay from a
  user-shorthand overlay (today both are `is_overlay = true` and indistinguishable).
- **5.2** WHEN the discriminator marks a row as a LOCALE surface THE resolver SHALL
  resolve that row at ROOT precedence (layer 1 for a CE locale surface, layer 3 for
  an FFCMD locale surface, per Req 1.2 / 1.3).
- **5.3** WHEN the discriminator marks a row as a USER shorthand THE resolver SHALL
  resolve that row at the user-alias precedence (layer 2 for a CE user alias, layer
  5 for an FFCMD user alias, per Req 1).
- **5.4** WHEN the menu DISPLAY SURFACE is computed for an option's canonical root
  THE display SHALL show the canonical root overridden ONLY by a LOCALE translation
  and NEVER by a USER shorthand.
- **5.5** WHEN the discriminator is derived rather than authored THE model SHALL
  derive it at load time from the overlay layer (the shipped locale seed vs the user
  override store); WHEN it is authored THE model SHALL persist a category key on the
  TOML row with a default that keeps existing files valid. (The design recommends
  and argues one of these; see `design.md`.)
- **5.6** WHEN CR-NR-121's menu-display rule (5.4) lands THE SCRM menu display SHALL
  flip from the current user-shorthand surface (`SS`) back to the canonical
  `SNAPSHOT`, and the CR-NR-120 golden SHALL be re-updated accordingly (a known
  downstream consequence recorded here, NOT delivered by this gate).

### Requirement 6 -- Non-regression of confirmed current behaviour

- **6.1** WHEN the end-state order lands THE existing confirmed landings of Req
  3.2-3.9 SHALL continue to hold (they are correct today and must not regress).
- **6.2** WHEN a parameterised alias (template) is submitted THE resolver SHALL
  still expand the template as part of resolution even though the user-alias layer
  is demoted below roots and menu options (the pre-pass template expansion is
  preserved, just re-sequenced).
- **6.3** WHEN a bare root command is submitted THE resolver SHALL still classify it
  as that root even after the alias verb-rewrite is demoted from a universal pre-pass
  to a fallback layer (the identity-row self-resolution mechanism is preserved by
  some means the design specifies).

## Non-Functional Requirements

### Requirement 7 -- Framework conformance and single dispatch

- **7.1** WHEN any seam (typed command line, menu-option click, keyboard shortcut,
  chained segment) dispatches a command THE model SHALL route it through the single
  front door `dispatch_command_string` and SHALL NOT introduce a second dispatcher
  or a second navigation stack.
- **7.2** WHEN any stage of the model is implemented THE model SHALL preserve the
  `CommandTarget` enum and the `WorkspaceContext` / `InteriorFocus` contracts, and
  any reshape of `EnvironmentRegistry` / the active-env claim gate SHALL be an
  OWNER-GATED framework change called out in `design.md` and `tasks.md`.
- **7.3** WHEN the CR-NR-121 model documents are authored or edited THE documents
  SHALL be ASCII-only per the documentation-charset steering rule.

### Requirement 8 -- Staged, owner-gated delivery

- **8.1** WHEN the model is implemented THE implementation SHALL follow the staged
  path A-F defined in `design.md` / `tasks.md`, each stage labelled ADDITIVE or
  FRAMEWORK with its prerequisites.
- **8.2** WHEN a stage is a FRAMEWORK change (claim-as-fallback, the `active_name`
  derivation change, FFEDIT re-landing, any reshape of the Environment_Registry or
  the claim gate) THE stage SHALL be explicitly OWNER-GATED and SHALL NOT proceed
  without express owner confirmation.
- **8.3** WHEN the end state is defined THE FFEDIT re-landing SHALL be IN SCOPE of
  this design (documented as part of the end state) but EXPLICITLY DEFERRED for
  implementation until editor work resumes, and SHALL NOT be treated as immediate.

## Acceptance Criteria (testable summary)

Each criterion is numbered for traceability from `tasks.md`.

1. AC-1 (Req 1.1-1.8): A resolution-order table/test fixture demonstrates the five
   layers resolve in order active-CE ROOT -> active-CE USER-ALIAS -> FFCMD ROOT ->
   MENU OPTION -> FFCMD USER-ALIAS, with CE non-claim falling through to FFCMD, and
   the three-layer collapse when active-CE == FFCMD (Req 1.8).
2. AC-2 (Req 1.2 / 1.3 / 5.2): A locale surface for a root resolves at ROOT
   precedence (ahead of any user shorthand) for both the active CE and FFCMD.
3. AC-3 (Req 1.4 / 1.5): A user shorthand does NOT shadow a same-named root or a
   same-named current-menu option.
4. AC-4 (Req 2.1-2.6): The end-state CE set (FFEDIT, per-KIND config CEs, menu CEs,
   FFCMD base) is enumerated; each claims its own verbs and falls back to FFCMD; the
   per-KIND config CEs reuse the EditableKind router.
5. AC-5 (Req 3.1-3.9): Resolution requires no navigation; `END;Z` lands `Z` on
   Settings; `RETURN;Z` lands `Z` on the Tab_Visual_Root; `=` reinitialises to POM;
   `=0` == `=settings`; `=a` fails with a message.
6. AC-6 (Req 4.1-4.5): A bare `END;Z` (and RETURN/CANCEL-led chains) is recognised
   and split; a DSN-like `abc.def` is NOT hijacked; the EXIT decision is recorded.
7. AC-7 (Req 5.1-5.6): A per-row discriminator separates locale overlays from user
   shorthands; the menu display shows canonical overridden only by a locale
   translation; the SCRM `SS` -> `SNAPSHOT` display flip and golden re-update are
   recorded as a downstream consequence.
8. AC-8 (Req 6.1-6.3, 7, 8): No confirmed current behaviour regresses; template
   expansion and bare-root self-resolution are preserved; every framework change is
   OWNER-GATED; FFEDIT is in scope but DEFERRED.

## Assumptions

- The three investigation reports accurately reflect the code on `main @ c72ff45`
  and the `cmd-resolution-model` worktree (the reports state the load-bearing files
  are byte-identical across the relevant checkouts).
- The owner's model (parts A-E) is a DECISION to be specified, not re-litigated;
  these requirements encode it verbatim.
- FFNAV (file-navigator) is out of scope here except that the active-env derivation
  must account for it; it is named/modeled only, per CR-CH-053.

## Out of Scope

- Any .rs source change or crate edit (this is a docs-only gate).
- Building FFNAV / FFLINE / any vision-only environment from the CR-CH-053 catalogue.
- The CR-NR-120 golden re-update itself (recorded as a downstream consequence only).
- The CR-CH-052 `=` / X / =X navigation-ladder redesign beyond pinning the confirmed
  `=` reinitialise behaviour (Req 3.6-3.8).
- Macro addressing (`ADDRESS <env>`) semantics beyond noting the model reconciles
  with them via CR-CH-053.
