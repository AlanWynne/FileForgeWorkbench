# FFWB Core Wiring Standard -- The Canonical Way to Wire a New Feature

This is the ONE repeatable pattern for wiring new functionality into the live
`ffwb` stack (`crates/ff-desktop` + its dependency-closure crates). It invents
nothing: it is grounded entirely in the existing framework -- `WorkspaceContext`
+ `render_workspace_context`, `resolve_target` / `CommandTarget` dispatch,
`menus/*.toml`, `WorkspaceDescriptor`, and the Workspace Kinds registry.

It complements the other steering rules: `framework-conformance.md` (build ON
the framework), `workspace-conformance.md` (the focus contract), `workflow.md`
(the requirements gate), `rust-standards.md` (400-line rule), and `testing.md`
(TDD + egui_kittest). When those conflict with this file on a specific point,
the more specific rule wins; this file is the integration recipe that ties them
together.

The model this standard is derived from is the Theme Editor Context (mirrored by
Menus / Keys / Kinds / Command Configurator / Macro Library / Event Log / Plugin
Manager / Search Results / Help). When in doubt, read how the Theme Editor
Context is wired and copy its shape.

## Known caveat -- Command Registration is temporarily weaker

Today, registering a command still means adding a verb to the
`shell/commands.rs::handle_command` ladder, because the typed command line
bypasses `resolve_target` and `builtin_workspace_target` is a stub (see the
Refactoring Roadmap, Phase 3 / S1). Until that lands, the Command Registration
step below is the one place the standard cannot yet be clean. Do NOT invent a
parallel dispatcher to work around this -- add the verb to the existing ladder,
and when the verb table lands, the ladder entry migrates into a table row with
no change to the rest of this standard.

## The eight integration concerns

### Feature Registration
- Keep the feature's pure logic in its OWN crate (model + egui render),
  depending only on narrow model crates, NOT on `ff-desktop`. Add the crate to
  `ff-desktop`'s `Cargo.toml`.
- If the feature is a new Workspace Context: add a `TabKind` variant
  (`tab_state.rs`) and, if it is navigable/persistable, a `WorkspaceKind` arm in
  `shell/nav_stack.rs` (`descriptor_for_current_context` + `reconstruct_custom`).

### Command Registration
- Register the verb so that the typed, menu, and key seams ALL resolve to the
  same handler. Post-verb-table (Phase 3): register a table entry and have
  `builtin_workspace_target` classify it. Until then: add the verb to the
  `handle_command` ladder (the documented temporary caveat above).
- Do NOT add a second dispatcher or a bespoke intercept outside the agreed seam
  (`framework-conformance.md`, mechanism 1).

### Menu Integration
- Add the option to the relevant `menus/*.toml` (content, not mechanism) with
  its command value. Built-in menus stay code-only (`DEFAULT_*_TOML`); NEVER
  write a default menu to disk.
- A clicked option routes through `resolve_target` -> `dispatch_command_target`
  -> `open_menu_by_name` (for Menu targets), exactly like the typed path.

### Toolbar Integration
- There is no separate toolbar dispatch. A toolbar button or any UI affordance
  MUST invoke the command (the same `dispatch_bound_command` seam as menu/key),
  never call the underlying handler directly.

### Event Subscription
- For in-process status/results, post to the shared
  `Arc<Mutex<NotificationQueue>>`.
- Only introduce a channel if a GENUINE background (cross-thread) producer
  exists. Do NOT pre-wire a channel speculatively.

### State Integration
- Hold the feature's UI state in its panel struct; store it on the shell inside
  the appropriate grouped sub-struct, not as a new flat field on the
  `WorkbenchShell` god-struct.
- Communicate state-changing intent from render by RETURNING an action enum that
  the shell applies in an `apply_<feature>_action` method. Do NOT mutate the
  shell from inside render. This is the `pending_action` pattern and it is THE
  Context-effect mechanism. Do NOT use the (currently unused) `ShellRequest`
  vocabulary.

### View Integration
- Implement `WorkspaceContext` for the panel state:
  `render(&mut self, ui, &mut ShellServices) -> InteriorFocus`.
- Give the FIRST interactive control a STABLE `egui::Id`; return
  `InteriorFocus::single(id)` (or `{ first, last }`), or `InteriorFocus::none()`
  for a documented no-interior Context.
- In the central-panel match arm, use the owned-panel swap: `mem::take` the
  panel -> `render_workspace_context(ctx, ui, &mut panel)` -> put it back ->
  apply the stashed action.
- Do NOT hand-write the focus latch. `render_workspace_context` (or
  `apply_interior_focus`) is the ONE latch path (`workspace-conformance.md`).

### Testing Requirements
- Full-shell `egui_kittest` first-Tab test: the first Tab from the command field
  MUST land EXACTLY on the reported first interior control (no phantom stop) --
  per `workspace-conformance.md`.
- Unit tests for the command handler and for the `apply_<feature>_action`
  effect.
- Keep every new/changed source file under 400 non-test lines; split tests
  before they exceed ~200 lines (`rust-standards.md`, `testing.md`).

## Implementation checklist (follow in order)

1. Create/extend the feature's pure crate (model + egui render returning an
   action enum). Add it to `ff-desktop/Cargo.toml`.
2. Add a `TabKind` variant and, if navigable, the `WorkspaceKind` descriptor
   arms in `shell/nav_stack.rs`.
3. Add the thin `impl WorkspaceContext` adapter in `ff-desktop` returning
   `InteriorFocus` with a stable first-control id.
4. Add the central-panel match arm using the `mem::take` ->
   `render_workspace_context` -> put-back -> `apply_<feature>_action` pattern.
5. Register the command (table entry / Command_Id, or ladder verb until the
   table lands) so typed, menu, and key seams all resolve to it; add the menu
   option in `menus/*.toml`.
6. Implement `apply_<feature>_action` for the rich shell-side effects.
7. Write the full-shell first-Tab `egui_kittest` test plus handler/effect unit
   tests; run the SCOPED `cargo check -p ff-desktop`, `cargo test -p ff-desktop`,
   `cargo clippy -p ff-desktop`, and `cargo fmt`.
8. Hand off for the owner's full `ffwb-gate.ps1` gate.

## Anti-patterns (do NOT do these)

- A bespoke `if upper == "..."` intercept outside the single dispatch seam.
- A second navigation stack or dispatcher.
- Mutating the shell from inside a Context's `render`.
- Hand-writing the focus latch per arm instead of the one `render_workspace_context`
  / `apply_interior_focus` path.
- Writing a built-in menu or theme file to disk.
- Adding a new flat field to `WorkbenchShell` when a grouped sub-struct fits.
- Pre-wiring a notification channel with no real cross-thread producer.

## Rationale

The decomposition is substantially complete; the remaining risk is divergence
inside the shell. A single documented wiring recipe keeps every new feature on
the same rails (one dispatch seam, one focus latch, one Context-effect
mechanism, one persistence model), which is what keeps the already-closed bug
classes (phantom Tab stops, divergent dispatch, lost tabs) closed as the product
grows.
