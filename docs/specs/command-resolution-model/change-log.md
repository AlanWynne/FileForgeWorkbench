# Change Log -- Command-Resolution Model

## CR-NR-121 -- END-STATE command-resolution model + staged path (design gate)

- **Id:** CR-NR-121 (next free CR-NR after the highest existing id CR-NR-120;
  highest CR-CH is CR-CH-116).
- **Type:** Requirements + design gate (docs only; no .rs source). LOAD-BEARING
  ARCHITECTURE. The workflow PAUSES for owner approval after this gate.
- **Status:** Authored, awaiting owner approval.
- **Supersedes / extends:** Supersedes and extends NOTHING. REFERENCES CR-NR-119
  (Command Help model -- FFCMD roots and the `HELP <cmd>` / `--help` seam must not be
  disturbed by resolution) and CR-NR-120 (the alias seed-set landing now with the
  current `SS`-surface menu display and a refreshed golden; CR-NR-121 will later flip
  that display to canonical `SNAPSHOT` and re-update the golden -- recorded as a known
  downstream consequence). EXTENDS the CR-CH-053 Command Environments framework and
  router (its phase-1 scope is left intact).

### Summary

Captures the owner-approved END-STATE command-resolution model: when the active
Context Kind is a live Command Environment (CE), the first token resolves CE-first
and falls back to the FFCMD base (claim-as-fallback, not a terminal hard-claim), in
the order active-CE ROOT -> active-CE USER-ALIAS -> FFCMD ROOT -> MENU OPTION ->
FFCMD USER-ALIAS. Locale surfaces resolve at ROOT precedence (localisation IS
locale-language aliases for root commands); user shorthands resolve LAST. A per-row
discriminator (origin-derived enum, recommended; authored category, the documented
alternative) separates locale-translation overlays from user-shorthand overlays so
each resolves at its correct precedence and so the menu display shows the canonical
root overridden only by a locale translation. The confirmed END / RETURN landing and
late-bound per-segment chaining behaviour is pinned as non-regressable, and
lifecycle-first bare chains ({END, RETURN, CANCEL}) become recognisable.

### Evidence base

Grounded in three read-only investigation reports, cited by path in design.md:
`.agents/tasks/cmdline-resolution-order/findings.md`,
`.agents/tasks/ce-layered-resolution/findings.md`,
`.agents/tasks/context-ce-identity/findings.md`.

### Staged path (A-F)

- Stage A -- lifecycle-first chain recognition (ADDITIVE; independent).
- Stage B -- overlay seed-vs-user discriminator (MODEL; foundational prerequisite).
- Stage C -- claim-as-fallback (FRAMEWORK, OWNER-GATED; the pivot).
- Stage D -- re-land / register FFEDIT (FRAMEWORK, OWNER-GATED, DEFERRED; in scope of
  the end state but deferred for implementation until editor work resumes).
- Stage E -- per-KIND config CEs reusing the EditableKind router (FRAMEWORK, OWNER-GATED).
- Stage F -- full layered order incl. ROOT-before-OPTION and user-alias-last
  (FRAMEWORK + MODEL, OWNER-GATED).

### Framework changes flagged OWNER-GATED (framework-conformance.md)

Claim-as-fallback (C); the `EnvironmentRegistry::active_name` derivation change (E);
FFEDIT re-landing / registration (D, deferred); any reshape of the Environment_Registry
/ claim gate / `dispatch_to_environment` claim outcome (C/E); the full layered-order
reorder (F). Stages A and B are not framework-mechanism changes (A additive
recognition; B a row-shape/model change with a loadable default).

### Home choice

Authored as the new spec folder `docs/specs/command-resolution-model/` to keep the
phase-1 scope of `command-environments` (CR-CH-053) intact while giving the end-state
model its own CR id and gate. Cross-references command-environments, command-semantics,
command-framework, localization, command-configurator, navigation-commands, and
menu-workspace. See design.md "Why this spec folder".

### Deliverables

- `docs/specs/command-resolution-model/requirements.md`
- `docs/specs/command-resolution-model/design.md`
- `docs/specs/command-resolution-model/tasks.md`
- `docs/specs/command-resolution-model/change-log.md` (this file)

### Review-loop revision (design-review findings addressed)

The first design review (`.agents/tasks/cmd-resolution-model/review.md`,
verdict CHANGES_REQUESTED) raised two MEDIUM and one NIT finding; all three are
addressed in this revision:

- Finding 1 (MEDIUM) -- resolution order undefined when active-CE == FFCMD: added
  Req 1.8 specifying the three-layer collapse (FFCMD ROOT -> MENU OPTION -> FFCMD
  USER-ALIAS) when the active CE IS the FFCMD base, and mirrored the collapse in the
  design.md end-state ASCII diagram. AC-1 now cites Req 1.1-1.8.
- Finding 2 (MEDIUM) -- several criteria not in WHEN...THE...SHALL form: reworded
  Req 2.5, 4.2, 5.1, 7.1, 7.2, 7.3, 8.1, 8.3 to carry an explicit WHEN trigger.
- Finding 3 (NIT) -- Req 4.2 pre-committed the lead-verb set while Req 4.5 framed the
  EXIT decision as open: reworded Req 4.2 to defer EXIT membership to Req 4.5.
