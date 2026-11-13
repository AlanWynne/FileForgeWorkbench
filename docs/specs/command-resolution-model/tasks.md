# Implementation Tasks -- Command-Resolution Model (CR-NR-121)

Staged in dependency order. Each top-level task is labelled ADDITIVE or FRAMEWORK
(and DEFERRED where applicable) per framework-conformance.md. FRAMEWORK tasks are
OWNER-GATED: they must not proceed without express owner confirmation. This gate is
docs-only; these tasks describe the FUTURE implementation and are NOT started here.

Markers: `[ ]` pending, `[x]` complete only. Status notes live on indented
sub-bullets, never inside the brackets.

- [ ] 1. Stage A -- Lifecycle-first chain recognition (ADDITIVE) (Req 4.1-4.5)
  - Independent; immediately useful for `END;Z`. No prerequisites.
  - [ ] 1.1 Define the closed lifecycle lead-verb set {END, RETURN, CANCEL} (Req 4.2)
    - Record the EXIT decision: recommend EXCLUDING EXIT (nothing follows an app
      termination); including it is safe but not recommended (Req 4.5).
  - [ ] 1.2 Widen the `try_chained_fastpath` recognition guard to accept a bare chain
        whose first segment is in the closed lead set (Req 4.1)
    - Keep the set closed so DSN-like `abc.def` is NOT hijacked (Req 4.3).
    - Downstream per-segment dispatch loop and nav landings UNCHANGED (Req 4.4).
  - [ ] 1.3 Add a chain-recognition test: `END;Z` splits and `Z` lands on the revealed
        menu; `abc.def` is not split (Req 4.1, 4.3, 4.4; AC-6)

- [ ] 2. Stage B -- Overlay seed-vs-user discriminator (MODEL) (Req 5.1-5.5)
  - Foundational prerequisite for ordering (Stage F) and the menu-display rule. No
    prerequisites of its own.
  - RECOMMENDED model: origin-derived enum `Origin { Seed, LocaleSeed, UserOverride }`
    stamped at load time from the overlay layer (see design.md; authored CATEGORY is
    the documented future alternative).
  - [ ] 2.1 Add the per-row discriminator field to the in-memory alias entry and the
        persisted TOML row shapes, with a loadable default for existing files (Req 5.1, 5.5)
  - [ ] 2.2 Plumb the discriminator through the overlay apply path so the locale seed
        and the user override store stamp different origins (Req 5.1, 5.5)
  - [ ] 2.3 Update the precedence consumers so locale rows resolve at ROOT precedence
        and user rows resolve at the user-alias precedence (Req 5.2, 5.3)
  - [ ] 2.4 Apply the menu-display rule: canonical root overridden ONLY by a locale
        translation, never by a user shorthand (Req 5.4)
    - Records the downstream SCRM `SS` -> `SNAPSHOT` display flip and CR-NR-120 golden
      re-update as a consequence NOT delivered by this task (Req 5.6).
  - [ ] 2.5 Add discriminator tests: a LocaleSeed overlay wins display and resolves at
        root; a UserOverride overlay does not win display and resolves last (AC-2, AC-7)

- [ ] 3. Stage C -- Claim-as-fallback (FRAMEWORK, OWNER-GATED) (Req 1.6, 2.5, 2.6)
  - Prerequisite: owner confirmation. The PIVOT of the whole model.
  - FRAMEWORK: reshapes the active-wins claim gate (framework-conformance mechanism 1).
  - [ ] 3.1 Change the active-environment claim from terminal hard-claim to
        "CE first, else FFCMD" so a non-claim falls through (Req 1.6, 2.6)
  - [ ] 3.2 Ensure no CE non-claim bypasses the FFCMD terminal fallback (Req 2.5)
  - [ ] 3.3 Add a fallback test: a CE that does not claim a token falls through to the
        FFCMD layers (AC-1, AC-4)

- [ ] 4. Stage D -- Re-land / register FFEDIT (FRAMEWORK, OWNER-GATED, DEFERRED) (Req 2.1, 8.3)
  - Prerequisite: Stage C. IN SCOPE of the end state but DEFERRED for implementation
    until editor work resumes; do NOT treat as immediate.
    - DEFERRED: not started until the parked editor domain resumes (Req 8.3).
  - FRAMEWORK: single-dispatch + registry (framework-conformance mechanism 1).
  - [ ] 4.1 Un-park the editor buffer CE and make `is_ffedit_active` real (Req 2.1)
  - [ ] 4.2 Register FFEDIT in `builtin_entries()` so the editor Context resolves
        CE-first then falls back to FFCMD (Req 2.1)
  - [ ] 4.3 Add an FFEDIT fallback test once editor work resumes (AC-4)

- [ ] 5. Stage E -- Per-KIND config CEs (FRAMEWORK, OWNER-GATED) (Req 2.2, 2.3)
  - Prerequisites: Stage C (claim-as-fallback) and Stage B (so locale vs user aliases
    order correctly). Reuses the existing `EditableKind` router.
  - FRAMEWORK: changes active-env derivation; registry membership is near-additive.
  - [ ] 5.1 Map each editable Kind (Config, Theme, Menus, Keys, Kinds, Alias, Command
        Configurator) to its own environment in `active_name` (Req 2.2)
  - [ ] 5.2 Register those environments in `builtin_entries()` (Req 2.2)
  - [ ] 5.3 Each CE claim body REUSES the `EditableKind` lifecycle router for its verb
        bodies, then falls back to FFCMD (Req 2.3)
    - Do NOT duplicate the EditableKind router.
  - [ ] 5.4 Add per-KIND CE tests: each claims its own verbs and falls back to FFCMD (AC-4)

- [ ] 6. Stage F -- Full layered order (FRAMEWORK + MODEL, OWNER-GATED) (Req 1.1-1.7, 6.1-6.3)
  - Prerequisites: Stages B, C, D (FFEDIT path; F may land with D deferred as long as
    the order is correct for the live CEs), E.
  - FRAMEWORK + MODEL: single-dispatch ordering (framework-conformance mechanism 1).
  - [ ] 6.1 Sequence the five layers: active-CE ROOT -> active-CE USER-ALIAS ->
        FFCMD ROOT -> MENU OPTION -> FFCMD USER-ALIAS (Req 1.1, 1.7)
  - [ ] 6.2 Keep locale surfaces at ROOT precedence for both the CE and FFCMD (Req 1.2, 1.3)
  - [ ] 6.3 Demote the alias verb-rewrite from a universal pre-pass to the user-alias
        layer while preserving bare-root self-resolution via the identity rows (Req 1.4, 6.3)
  - [ ] 6.4 Preserve parameterised-alias (template) expansion in the demoted layer (Req 6.2)
  - [ ] 6.5 Verify no confirmed landing regresses: `END;Z` -> Settings, `RETURN;Z` ->
        Tab_Visual_Root, `=` -> POM, `=0` == `=settings`, `=a` fails (Req 3.2-3.9, 6.1)
  - [ ] 6.6 Add the full resolution-order fixture test demonstrating the five-layer
        precedence and all AC rows (AC-1, AC-2, AC-3, AC-5, AC-8)

- [ ] 7. Gate close-out (docs) (Req 7, 8)
  - [ ] 7.1 Confirm every framework change is flagged OWNER-GATED in design.md and
        tasks.md and that the workflow paused for owner approval (Req 7.2, 8.2)
  - [ ] 7.2 Confirm ASCII-only across all CR-NR-121 docs (Req 7.3)
