# KindTag discriminant + payload-carrying TabKind (Task 8, Approach C)

Task 8 of the ff-desktop simplification splits the single `TabKind` enum into two
types so that per-kind STATE can live inside the kind variant rather than in a
parallel `Option<...>` field on `TabState`. The former `TabKind` (a cheap
`Copy` tag) is renamed `KindTag` and keeps its role as the equality/dispatch
discriminant; a new payload-carrying `TabKind` enum is introduced whose sole
non-unit variant is `MenuWorkspace(Option<MenuWorkspaceState>)`. The flat
`TabState::menu_workspace: Option<MenuWorkspaceState>` field is removed entirely,
and the ~55 comparison sites now go through `tab.kind.tag() == KindTag::X` while
the ~18 state-access sites go through `tab.kind.menu_workspace()` /
`menu_workspace_mut()`. Functions that classified by kind (`active_environment`,
`from_tab_kind`, `context_name_for_kind`) now take the `Copy` `KindTag`.

This is a behaviour-preserving internal refactor with no gate. The change set is
mechanical and consistent across 38 source/test files; the diff shows zero
`#[test]` lines added or removed (every test edit is an in-place type/accessor
rewrite), no `#[ignore]`, ASCII-clean `.rs`, and all non-test files under 400
lines.

Watch for: one forced-and-documented behavioural subtlety in SCRM auto-capture
(confirmed) — the capture hook moved from AFTER `reconstruct_context` to BEFORE
it, so auto-capture now snaps the DEPARTING context rather than the arrived one.
The single test covering it (POM -> FILES) passes under both orderings; the
difference only surfaces in menu->menu and panel->menu transitions, which no test
exercises. This is non-blocking (see below) but the owner should be aware.

**Verdict**: APPROVED

## High-level view

The two-type split is clean and the invariants hold. `KindTag` is the renamed
old enum — `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, no payload, variant
names unchanged. The new `TabKind` derives only `Debug` (not `Copy`, not
`PartialEq`/`Eq`), carries `MenuWorkspace(Option<MenuWorkspaceState>)` as its one
payload variant with all others unit, and exposes `tag()` plus
`menu_workspace()`/`menu_workspace_mut()` accessors that map correctly. The old
flat field is gone from `tab_state.rs` and from every constructor in
`tab_state_ctors.rs`.

Call-site rewiring is uniform. Every equality against a kind now routes through
`.tag()`; a grep for `kind == TabKind` / `kind != TabKind` returns nothing, so
the payload enum is never (and cannot be) compared directly. Pattern `match`es
on `tab.kind` that only need to discriminate unit variants are left as native
matches (legal without `PartialEq`), and the one place that destructures the
payload (`mod_helpers::title_line_text`) binds `TabKind::MenuWorkspace(ref mw)`
and then operates on the inner `Option`. Every `menu_workspace` state access goes
through the new accessor — no dangling read of the removed field remains.

Only the `menu_workspace` state was moved into the enum; no other kind-specific
state was invented or touched. Seeding/clearing sites that previously wrote
`tab.menu_workspace = Some(mw)` / `= None` now set the kind
(`TabKind::MenuWorkspace(Some(mw))` / `(None)`), and the `set_active_tab_context`
reset drops the inner state implicitly by replacing the kind — which is where the
one behavioural subtlety lives.

The SCRM auto-capture ordering change in `navigate_to` is the only non-mechanical
edit. It is forced by the refactor and the coder documented it in the code
comment and the resume note.

<details>
<summary>Issues (1)</summary>

1. **SCRM auto-capture now snaps the departing context (confirmed, non-blocking)**
   — the capture hook moved from after to before `reconstruct_context`, so a
   menu->menu transition now captures the departing menu instead of the arrived
   one, and a panel->menu transition captures nothing instead of the arrived
   menu. The old behaviour relied on a stale flat field and was itself
   inconsistent; the one covering test (POM->FILES) still passes. Flag for owner
   awareness; consider a follow-up test pinning the intended auto-capture content
   if the exact captured screen matters for SCRM evidence flows.

</details>

<details>
<summary>Details</summary>

## The two-type split and its invariants

The high-level view covers the derive traits and variant layout; here are the
specifics worth calling out.

`tag(&self) -> KindTag` maps all seventeen variants one-to-one; `MenuWorkspace(_)`
discards the payload and returns `KindTag::MenuWorkspace`. The accessors delegate
to `Option::as_ref()` / `as_mut()` on the inner `Option`:

```rust
pub fn menu_workspace(&self) -> Option<&MenuWorkspaceState> {
    match self { TabKind::MenuWorkspace(mw) => mw.as_ref(), _ => None }
}
```

All three constructor paths in `tab_state_ctors.rs` (`base_tab!`, the two
explicit `TabState` constructors) drop the `menu_workspace: None` initialiser.
`pom()` now builds `TabKind::MenuWorkspace(None)`; the menu-workspace constructor
builds `TabKind::MenuWorkspace(Some(mw_state))` inline, replacing the former
two-step `base_tab!` + `tab.menu_workspace = Some(...)`.

## Call-site rewiring: equality through tag(), state through accessors

A workspace grep for `kind == TabKind` / `kind != TabKind` returns no matches, so
no code compares the payload enum directly — consistent with it not deriving
`PartialEq`. Comparison sites across `tab_manager.rs`, `session_manager.rs`,
`commands*.rs`, `render*.rs`, `helpers.rs`, `titles.rs`, `help.rs`,
`update*.rs`, and the test modules now use `tab.kind.tag() == KindTag::X` (or
`matches!(tab.kind.tag(), KindTag::A | KindTag::B)`).

Pattern matches that only need to tell unit variants apart keep matching on
`tab.kind` directly (`kind_title` in `titles.rs`, `render_body` dispatch in
`render_body.rs` whose menu arm became `TabKind::MenuWorkspace(_)`). This is
legal without `PartialEq` and is the correct idiom. The single payload-binding
match is `mod_helpers::title_line_text`:

```rust
TabKind::MenuWorkspace(ref mw) => mw
    .as_ref()
    .and_then(|mw| mw.menu_title())
    .unwrap_or_else(|| tab.title.clone()),
```

State-access rewiring is uniform: `tab.menu_workspace.as_ref()` ->
`tab.kind.menu_workspace()`, `tab.menu_workspace.as_mut()` ->
`tab.kind.menu_workspace_mut()` (e.g. the live menu render in
`render_body_arms.rs`). Seeding sites (`commands_fastpath::seed_pom`,
`nav_reconstruct::reconstruct_settings_menu`/`reconstruct_named_menu`) set
`tab.kind = TabKind::MenuWorkspace(Some(mw))`; clearing sites
(`set_active_tab_home`, `reset_bare` POM reset, the retired
`tab.menu_workspace = None` in `tab_manager::set_active_tab_context`) now set
`TabKind::MenuWorkspace(None)` or simply replace the kind. The classifier
signatures (`active_environment`, `BuiltinKind::from_tab_kind`,
`context_name_for_kind`) changed from `TabKind` to `KindTag`, and their callers
pass `tab.kind.tag()`; the match bodies inside them switched from `TabKind::` to
`KindTag::` arms with the same mapping.

## SCRM auto-capture: the one behavioural subtlety

`navigate_to` is the only non-mechanical edit. Previously the capture hook ran
AFTER `reconstruct_context`; it now runs BEFORE:

```rust
self.auto_capture_active_context();   // was below reconstruct_context
self.reconstruct_context(&descriptor);
```

`auto_capture_active_context` -> `active_screen_model` builds a model only for a
`MenuWorkspace` tab (`tab.kind.menu_workspace().map(...)`), returning `None` for
any non-menu context. `navigate_to` is the sole caller, and there is no second
hook inside `reconstruct_context`.

With the old flat field, `set_active_tab_context` set `tab.kind` but did NOT
clear `menu_workspace`, so a post-reconstruct capture read a STALE departing menu
when arriving at a non-menu panel, and the freshly-reconstructed menu when
arriving at a menu. That accidental, inconsistent behaviour cannot be reproduced
once the state lives inside the kind: after `reconstruct_context` the departing
menu is gone, so capturing after reconstruct would only ever see the arrived
state (or `None`). The coder's choice to capture before reconstruct makes the
hook deterministically snap the departing context.

Net effect on observable behaviour:
- menu -> non-menu panel (the tested POM->FILES path): old captured the stale
  departing menu; new captures the departing menu. Same result; the test
  `auto_capture_records_on_navigation` passes under both.
- menu -> menu: old captured the ARRIVED menu; new captures the DEPARTING menu.
  Different content. No test exercises this.
- panel -> menu: old captured the ARRIVED menu; new captures nothing. Different.
  No test exercises this.

This is a genuine observable change in the auto-capture feature, so it is noted
as a finding. It is non-blocking because the behaviour it replaces was accidental
(stale-field dependent) and self-inconsistent, the design intent ("capture the
screen transition") is still met, the move is forced by the refactor, the only
covering test still passes, and the coder documented the change in both the code
comment and the resume note. A follow-up test pinning the intended captured
screen would be the right place to lock down the desired semantics if the exact
content matters for evidence flows.

## Test integrity and hygiene

The full working-tree diff contains zero added or removed `#[test]` lines — every
test change is an in-place type/accessor rewrite (`.kind` -> `.kind.tag()`,
`TabKind::X` -> `KindTag::X`, `menu_workspace.as_ref()` ->
`kind.menu_workspace()`), including the renamed-but-equivalent assertions in
`tab_manager.rs`, `workspace_kind/tests.rs`, `tests_nav_ladder.rs`,
`tests_focus.rs`, `tests_menu_workspace.rs`, and `tests_common.rs`. No test was
weakened, deleted, or `#[ignore]`'d (grep confirms no `#[ignore]`). `check_line_limits.py`
reports no non-test `.rs` file over 400 lines; `tab_state.rs` is ASCII-clean.

Verification evidence was taken from `docs/status/RESUME-ffdesktop-simplification.md`
(coder ran `cargo fmt`, `cargo check -p ff-desktop`, `cargo clippy -p ff-desktop`,
`cargo test -p ff-desktop`; all green, with the known B048 env-var flakes passing
in isolation under nextest). The test suite was not re-run per the step brief; the
only spot-checks performed were read-only greps and the line-limit script.

</details>

<details>
<summary>File map</summary>

- `tab_state.rs` — defines `KindTag` (renamed old enum) + new payload `TabKind`,
  `tag()`/`menu_workspace()`/`menu_workspace_mut()`; removes the flat field.
- `tab_state_ctors.rs` — drops `menu_workspace: None` from all constructors;
  `pom()`/menu constructor build the payload variant inline.
- `tab_manager.rs` — `.tag()` comparisons in the open-or-focus helpers; menu
  dedup via `kind.menu_workspace()`; retired `tab.menu_workspace = None`.
- `session_manager.rs` — descriptor/active-tab classification via `.tag()`;
  menu name via `kind.menu_workspace()`.
- `shell/nav_stack.rs` — descriptor derivation via `.tag()`; auto-capture moved
  before `reconstruct_context`.
- `shell/nav_reconstruct.rs`, `shell/reset_bare.rs`, `shell/commands_fastpath.rs`
  — seed/clear sites set `TabKind::MenuWorkspace(Some|None)`.
- `shell/environment.rs`, `shell/helpers.rs`, `workspace_kind/mod.rs` —
  classifier signatures take `KindTag`.
- `shell/commands*.rs`, `shell/render*.rs`, `shell/titles.rs`,
  `shell/mod_helpers.rs`, `shell/help.rs`, `shell/kinds_editor.rs`,
  `shell/update*.rs` — comparison/access rewiring.
- test modules (`tests_*.rs`, `workspace_kind/tests.rs`) — in-place
  type/accessor rewrites, no test count change.
- `docs/status/RESUME-ffdesktop-simplification.md` — task 8 marked done with
  evidence.

Full diff: `git diff` in the workspace root (37 code files + 1 doc).

</details>
