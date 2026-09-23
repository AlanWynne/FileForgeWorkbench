---
inclusion: always
---

# Workspace Tab-Order Conformance (CR-CH-023)

Every Workspace Context rendered by the shell MUST participate in the unified
tab-order model (menu-and-statusbar Requirement 16, CR-CH-023). Three bugs of the
SAME class -- B056 (POM), B057 (Theme Editor), B058 (Config Panel) -- were all
caused by a render arm that drew its panel but forgot to wire the focus contract,
leaving a phantom "invisible" Tab stop between the command line and the first
real control. This rule exists so that class of bug cannot recur.

## The rule -- every workspace render arm

In `crates/ff-desktop/src/shell/render.rs`, the central-panel `match
self.tabs.active_tab().kind { ... }` resets `self.first_interior_id` and
`self.last_interior_id` to `None` at the top of each frame. EVERY arm therefore
MUST, after rendering its panel, do ONE of:

1. **Report its interior focus stops and honour the latch** (the normal case):
   ```rust
   self.first_interior_id = Some(<first focusable control id>);
   self.last_interior_id  = Some(<last focusable control id>);
   self.honour_interior_focus_latch(ctx, self.first_interior_id, self.last_interior_id);
   ```
   - The FIRST interior control MUST have a STABLE `egui::Id` (assigned via
     `.id(egui::Id::new("<panel>_<field>"))` on the widget, or captured from a
     combo's `response.id` each frame and stored on the panel state). A stale or
     guessed id does not round-trip through egui focus (B056) -- the id reported
     MUST be the fresh same-frame id of a real widget.
   - The LAST interior control anchors the Shift+Tab reverse boundary. When the
     panel's content is variable, a guaranteed-present control (e.g. the filter
     field) MAY serve as both first and last; egui-native Tab walks the controls
     in between in visual order.

2. **Explicitly document that the workspace has NO interior focus stops** -- leave
   both ids `None` with a comment stating why (e.g. a purely non-interactive
   display). This is the rare exception, not the default.

A render arm that renders interactive widgets but leaves both ids `None` without
the documented exception is a BUG (the phantom-stop class), not a style nit.

## The test -- every workspace MUST prove it

Each workspace that reports interior focus stops MUST ship a full-shell
`egui_kittest` `build_eframe` test asserting that the FIRST Tab from the command
field lands EXACTLY on the reported first interior control (no phantom stop).
Model it on `full_shell_config_first_tab_focuses_filter_field` /
`full_shell_theme_editor_first_tab_focuses_theme_selector`:

```rust
let mut harness = harness_shell();
harness.state_mut().handle_command("<open the workspace>");
for _ in 0..4 { harness.run(); }
let expected = harness.state().first_interior_id;
assert!(expected.is_some(), "<workspace> must report a first interior control");
assert_eq!(harness.ctx.memory(|m| m.focused()), Some(cmd_field_id())); // entry focus
harness.press_key(egui::Key::Tab);
harness.run();
assert_eq!(harness.ctx.memory(|m| m.focused()), expected,
    "first Tab must focus the reported first interior, not a phantom stop");
```

This is a specialisation of the "GUI Behaviour Testing" rule in `testing.md`
(rendered-widget focus criteria MUST have an `egui_kittest` test; MANUAL is a
justified exception only for pixel-exact appearance, OS-native dialogs, real
OS windows/viewports, or screen-reader output).

## The `WorkspaceContext` framework (CR-NR-078) -- now the enforcement mechanism

Phase 1 of the framework has landed (`crates/ff-desktop/src/shell/workspace_context.rs`).
For a MIGRATED Context, the contract is COMPILER-ENFORCED: it implements
`WorkspaceContext`, whose `render(&mut self, ui, &mut ShellServices) ->
InteriorFocus` RETURNS the focus contract, and the shell dispatches it through
`WorkbenchShell::render_workspace_context` (owned-panel swap) which applies the
latch on one code path. An implementor CANNOT compile without returning
`InteriorFocus` (use `InteriorFocus::none()` for the deliberate no-interior case).

Migration COMPLETE (phase 1 + WF.6, CR-NR-078). `impl WorkspaceContext` +
`render_workspace_context` dispatch: Config panel, Theme Editor, Menus Editor,
Keys Editor, Kinds Editor, Plugin Manager, Event Log, Macro Library, Command
Configurator, Search Results. Routed through the shared
`WorkbenchShell::apply_interior_focus` helper (SAME `InteriorFocus` type + SAME
single latch path) because their render carries inputs the `ShellServices`-only
trait cannot express: the MenuWorkspace (POM + Settings + user menus; menu-specific
calendar io), the Editor Context (renders the active `TabState` with
shell-entangled inputs), and the Files Panel (own command field + bespoke
`files_panel_cmd` -> tree Tab redirect). The Editor and Files Panel are the two
documented `InteriorFocus::none()` no-interior cases (exception 2 below).

There are therefore NO unmigrated arms: EVERY central-panel arm either implements
`WorkspaceContext` (dispatched via `render_workspace_context`) or calls
`apply_interior_focus` with an explicit `InteriorFocus`. The "Legacy manual rule"
section below is retained only as guidance for any future arm added before it is
converted -- it should not be needed.

## When adding a NEW workspace

1. Implement `WorkspaceContext` for it: `render` returns an `InteriorFocus`
   (give the first interactive control a stable `egui::Id`; `InteriorFocus::none()`
   only for the deliberate no-interior case). Route state-changing intent via
   `ShellRequest`s / `ShellServices` (or a stashed action for rich shell-side
   effects, as Theme Editor / Menus Editor do).
2. Dispatch it via `render_workspace_context` (owned-panel swap) -- do NOT hand
   -write the focus-latch ritual.
3. Add the full-shell first-Tab egui_kittest test.
4. Do NOT add a new bespoke `match` arm that renders a panel and manually
   reports focus ids -- that is the pre-framework pattern the framework replaces.

## Legacy manual rule (unmigrated arms only)

Until an unmigrated Context is converted to `WorkspaceContext`, its `render.rs`
arm MUST still, after rendering, report `self.first_interior_id`/`self.last_interior_id`
and call `honour_interior_focus_latch` (or explicitly document `None` with a
reason). This is the interim guardrail for the not-yet-migrated arms listed above.

## The test -- every workspace MUST prove it (unchanged)

Each workspace (migrated or not) MUST ship a full-shell `egui_kittest`
`build_eframe` test asserting the FIRST Tab from the command field lands EXACTLY
on the reported first interior control (no phantom stop). See the "The test"
section below.

## Rationale

The correct behaviour was opt-in per arm and enforced only by convention, which
is why the bug recurred four times (B056-B059). The `WorkspaceContext` framework
makes the contract a mandatory return value the compiler demands; for migrated
Contexts the bug is now unrepresentable, and the full-shell test remains the
behaviour safety net.
