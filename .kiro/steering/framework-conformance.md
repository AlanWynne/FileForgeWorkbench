# Framework Conformance -- Build ON the Framework, Do Not Change It

Every new requirement, change request, and bug fix MUST be evaluated against the
CURRENT framework and MUST build ON it. A requirement MUST NOT alter, replace, or
work around a core framework mechanism UNLESS the owner has EXPRESSLY requested
and CONFIRMED that framework change.

This rule exists because several older requirements were written against an
earlier architecture; implementing them verbatim would duplicate or fight the
framework that has since landed (e.g. proposing a second navigation stack when
the per-tab Navigation_Stack already exists). Re-phrase such requirements to use
the framework before implementing them.

## The core framework (the load-bearing mechanisms)

Treat these as fixed unless a framework change is expressly confirmed:

1. **Single command dispatch (CommandTarget).** Every user action is a command
   that resolves through the ONE classifier `ff_command::resolve_target` into a
   `CommandTarget` (`Menu` / `CustomWorkspace` / `Function` / `Macro` /
   `External`) and dispatches through `shell/target_dispatch.rs`
   (`resolve_and_dispatch_command` / `dispatch_bound_command` /
   `dispatch_command_target`). The typed command line, menu-option clicks, and
   keyboard shortcuts ALL route through this single path (CR-CH-043/044).
   - DO: add a new command as a registered Command_Id (a `Function` target) or a
     new `menus/<name>.toml`, resolved by the existing chain.
   - DO NOT: add a bespoke `if upper == "..."` intercept in `handle_command`, or a
     parallel dispatcher.

2. **Menu-name resolution, no privileged verbs (CR-CH-025).** `POM`, `SETTINGS`,
   and any user menu open by keyword-less Menu_Name resolution through
   `try_menu_name_dispatch` -> `open_menu_by_name` (which OWNS in-place vs
   new-tab placement). No menu is a special-cased verb.
   - DO: rely on `open_menu_by_name` for any menu-open effect.
   - DO NOT: reintroduce a hardcoded menu verb.

3. **Per-tab Navigation_Stack, transform-in-place (CR-CH-022).** Navigating from
   one Context to another transforms the CURRENT tab in place and pushes the
   previous Context via `navigate_to(descriptor, push)` (`shell/nav_stack.rs`).
   END/F3 pops one level; only `START` creates a new tab.
   - DO: route navigation through `navigate_to`; use its `push` flag for
     STOP (`push=false`) vs PUSH (`push=true`) semantics.
   - DO NOT: introduce a second navigation stack, or spawn a tab for navigation.

4. **Code-only built-in menus (CR-CH-021).** Built-in menus (`pom`, `settings`)
   live as compiled `DEFAULT_*_TOML` Recovery_Baseline constants and are NEVER
   written to disk. A user `menus/<name>.toml` OVERRIDES the compiled default; an
   absent/invalid user file falls back to the compiled baseline (or the
   load-error state for a named menu).
   - DO NOT: write a default menu file on first launch.

5. **WorkspaceContext single focus-latch path (CR-NR-078, CR-CH-023).** Every
   Workspace Context reports its interior focus through ONE path -- the
   `WorkspaceContext::render -> InteriorFocus` trait dispatched by
   `render_workspace_context`, or the shared `apply_interior_focus` helper for the
   documented variances. See `workspace-conformance.md`.
   - DO NOT: hand-wire a per-arm focus ring or leave a render arm's focus anchors
     silently unset.

6. **Descriptor-based session persistence (CR-CH-012).** Visible Workspaces
   persist as `WorkspaceDescriptor` (`Menu { name }` or `CustomWorkspace { kind,
   params }`) and restore by re-opening the descriptor. Started Tasks and
   transient output are never persisted.

7. **WorkspaceContext / Workspace Kinds title + chrome model (CR-NR-090,
   CR-CH-045).** Kind title/menu-bar/key-list/profile come from the Kind
   registry; the Title_Line uses the descriptive display title; the `[XXX]`
   Tab_Header tag is separate. See `workspace-conformance.md`.

## The rule for every new requirement / change request

During the requirements gate (see `workflow.md`), BEFORE writing criteria:

1. **Evaluate against the framework.** Identify which of the mechanisms above the
   requirement touches. State, in the requirement or its design delta, HOW it
   uses each mechanism.
2. **Prefer the framework seam.** Express the requirement in terms of the
   existing seams (a new Command_Id, a `menus/*.toml`, a `navigate_to` call, a
   `WorkspaceContext` impl, a `CommandParams` argument, a `WorkspaceDescriptor`),
   not a new parallel mechanism.
3. **Flag any framework change EXPLICITLY.** If the requirement genuinely cannot
   be satisfied without altering a core mechanism, STOP and surface it to the
   owner as a FRAMEWORK CHANGE with the tradeoffs. Do not proceed until the owner
   expressly confirms the framework change. A confirmed framework change is
   itself a gated change request that updates the relevant steering rule.
4. **Re-phrase legacy requirements.** A pre-existing requirement written against
   an older architecture MUST be re-phrased to use the current framework before
   implementation. Record the re-phrasing (and what it now builds on) in the
   change-log entry. Do not implement the stale wording.

## What counts as a framework change (needs express confirmation)

- A new command-dispatch path or intercept that bypasses `resolve_target` /
  `dispatch_command_target`.
- A second navigation stack, or navigation that is not `navigate_to`.
- Writing built-in menu/theme files to disk.
- A per-Context focus mechanism outside the `InteriorFocus` single latch path.
- A new session-persistence format outside the `WorkspaceDescriptor` model.
- Removing or reshaping a public framework type (`CommandTarget`,
  `WorkspaceContext`, `InteriorFocus`, `WorkspaceDescriptor`, `KindConfig`).

## What is NOT a framework change (proceed normally)

- Adding a registered Command_Id / `Function` handler.
- Adding or editing a `menus/*.toml` (content, not mechanism).
- A new `WorkspaceContext` implementor dispatched via `render_workspace_context`.
- Forwarding arguments through the existing `CommandParams` on `execute_command`.
- Additive logging/instrumentation at an existing seam.
- A new `CustomWorkspace` Kind in the registry.

## Rationale

The framework is the product's spine: one command dispatch, one navigation
model, one focus contract, one persistence model, code-only built-ins. Each was
built to kill a recurring class of bug (phantom Tab stops, divergent dispatch,
lost tabs, doubled titles). A requirement that quietly re-implements one of these
re-opens that bug class. Building ON the framework keeps the bug classes closed;
changing it is a deliberate, owner-confirmed decision, never an implementation
side effect.
