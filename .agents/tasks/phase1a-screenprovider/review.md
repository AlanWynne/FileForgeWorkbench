# Per-Context ScreenProvider dispatch seam + conformance harness (Phase 1A)

Phase 1A of the master-screen-directive / SCRM program replaces the hard-coded
menu-only `active_screen_model()` with a per-Context dispatch seam
(`shell/screen_provider.rs`) keyed on the active Context's `KindTag`. The seam
returns a `ScreenCapture` enum (`Model | OptOut`) that makes the two legitimate
outcomes explicit and symmetric with the focus contract's
`InteriorFocus::none()`. Menu Contexts keep the existing
`menu_workspace_screen_model` provider so POM/Settings/menu snapshots are
byte-for-byte unchanged; every non-menu Context flows through a documented
opt-out carrying a `PENDING_PROVIDER_REASON` string until Stream SCRM C.2 adds
real providers. A reusable, parameterised full-shell `egui_kittest` harness
(`shell/tests_screen_directive.rs`) asserts the snapshot/capture contract per
Context at both the seam and the user-visible SNAPSHOT layers.

Watch for: nothing blocking. The refactor is behaviour-preserving for menu
Contexts (confirmed against the `main` baseline), the opt-out is explicit and
documented, the not-capturable path now flows through the new seam, and scope is
held tightly to 1A.1 + 1A.2. A couple of minor observations are noted below, none
of which gate the merge.

**Verdict**: APPROVED

## High-level view

The seam refactor is faithful. On `main`, `active_screen_model()` was exactly
`tab.kind.menu_workspace().map(|mw| menu_workspace_screen_model(mw, &self.command_text))`.
The new code produces the identical value for every Context: a seeded
`MenuWorkspace` yields `Model(menu_workspace_screen_model(...))`, and everything
else (including an unseeded `MenuWorkspace`, where `menu_workspace()` was `None`)
collapses to `None` through `into_model()`. No menu snapshot can change, because
the model-building call and its single argument are untouched and
`screen_snapshot.rs` is not in the diff.

The opt-out is a real, documented mechanism rather than a silent gap. The
`ScreenCapture::OptOut(&'static str)` variant carries a reason; non-menu Contexts
use the shared `PENDING_PROVIDER_REASON` constant that points at the Req 5
retrofit matrix, and an unseeded menu reports its own distinct reason. The
user-visible SNAPSHOT error now names the reason, so "cannot be captured yet" is
a declared exception with provenance, not a bare failure.

Framework-seam conformance holds. SNAPSHOT and CAPTURE still enter through the
single front door; the seam only reads the logical model via the existing
`ScreenProvider`/`menu_workspace_screen_model` path and never touches egui
widgets. Navigation in the opt-out test goes through the `nav_to_kind` ->
`navigate_to` choke point rather than mutating tab state directly.

The harness is genuinely reusable. `assert_capture_contract(command, kind,
expectation)` is parameterised by a `CaptureExpectation` enum and asserts at both
the `active_screen_capture()` seam and the `snapshot_text_for_active` result, so
Stream SCRM C.2 can call it per Context and flip each from `OptOutPending` to
`Capturable` as providers land. It is exercised for the POM (capturable) and the
Event Log (documented opt-out) as the task requires.

Scope is clean: four files, all seam-related. No Context retrofits, no
save-convention work, no new SCRM features, no merge or push.

<details>
<summary>Issues (2)</summary>

1. **`into_model` discards the opt-out reason at two call sites** (possible) — `active_screen_model()` -> `into_model()` is still used by `CAPTURE SCREEN` and `auto_capture_active_context`, which report a generic not-capturable message and lose the reason the seam now carries. Acceptable for Phase 1A (the task only required the path to flow through the seam, which it does), but worth a follow-up so CAPTURE SCREEN names the reason like SNAPSHOT does.
2. **`#![allow(unused_imports)]` in the harness file** (confirmed) — `tests_screen_directive.rs` blanket-allows unused imports at module scope. Minor; a blanket allow can mask a genuinely dead import later. Consider dropping it or scoping it once the import set is final.

</details>

<details>
<summary>Details</summary>

### Seam refactor correctness and the menu-snapshot no-change guarantee

The `main` baseline for `active_screen_model()` was confirmed by reading
`main:crates/ff-desktop/src/shell/commands_scrm.rs`:

```rust
let tab = self.tabs.active_tab();
tab.kind
    .menu_workspace()
    .map(|mw| crate::screen_snapshot::menu_workspace_screen_model(mw, &self.command_text))
```

The new seam is behaviourally identical for the `Option`-shaped callers. For a
`MenuWorkspace` tag it calls `menu_workspace()`; when `Some(mw)` it builds
`menu_workspace_screen_model(mw, &self.command_text)` — same function, same
single `command_text` argument — and when `None` it opts out, which `into_model()`
turns back into `None`. For any other tag the old code's `menu_workspace()` was
`None` and so is the new opt-out. `screen_snapshot.rs` is not in the diff, so the
model-building and rendering code is untouched. The commit records 33 passing
`scrm` tests (snapshot/markdown/unknown-format and auto-capture/capture-screen)
and 12 passing `screen` tests, consistent with a no-behaviour-change refactor.

The one representational shift is `ScreenCapture::Model(Box<ScreenModel>)`. Boxing
avoids a large enum (one variant holds a full `ScreenModel`, the other a thin
`&'static str`) and is unwrapped at the single consumer; it has no effect on the
rendered output.

### The opt-out is explicit, documented, and the not-capturable path flows through it

The directive's Req 4.3/4.4 wanted an explicit opt-out analogous to
`InteriorFocus::none()`. `ScreenCapture::OptOut(&'static str)` delivers that: the
variant cannot be produced without a reason string. Non-menu Contexts share
`PENDING_PROVIDER_REASON`, which explicitly cites the Req 5 retrofit matrix, and
an unseeded `MenuWorkspace` carries its own `"Menu_Workspace not yet seeded"`
reason instead of silently returning nothing. `snapshot_text_for_active` now
formats that reason into the user-facing error, so the not-capturable response is
sourced from the seam rather than a stray hard-coded string. The module
documentation states the Phase 1A scope and that Stream SCRM C.2 flips each
default opt-out to a `Model` — the seam is designed so later streams only edit
the dispatch arms, not the callers.

### Framework-seam conformance

SNAPSHOT and CAPTURE continue to resolve through the single front door; this
change sits below dispatch, in how the active Context yields its logical model. It
reads only the logical `ScreenModel` (via the existing provider) and never queries
egui widgets, satisfying screen-snapshot-scrm Req 2.1/2.2. The opt-out test drives
the Context transition through `nav_to_kind` -> `navigate_to`, the sanctioned
navigation choke point, rather than poking tab state.

### The harness is reusable and parameterised

`assert_capture_contract(command, expect_kind, expect)` opens a Context through
`dispatch_command_string` (the single front door), settles frames, asserts the
opened Context's `KindTag`, then checks both layers: the `active_screen_capture()`
seam outcome and the `snapshot_text_for_active("")` result. Driving both layers is
what proves the not-capturable message comes from the opt-out seam and not a
leftover hard check. The `CaptureExpectation` enum (`Capturable` /
`OptOutPending`) is the flip point Stream SCRM uses per Context. It is exercised
for the POM (`conformance_pom_is_capturable`) and the Event Log
(`conformance_non_menu_context_reports_documented_opt_out`), matching the task's
"one representative non-menu Context" requirement. The unit tests in
`screen_provider.rs` additionally cover the seeded-POM provider, the non-menu
opt-out via `nav_to_kind`, and the unseeded-`MenuWorkspace` opt-out. All tests
carry `// Validates:` annotations citing master-screen-directive Req 4 and
screen-snapshot-scrm Req 2.

### Standards and scope

Both new files are within limits: `screen_provider.rs` is 179 lines (56 of them a
`#[cfg(test)]` module excluded from the 400-line rule) and
`tests_screen_directive.rs` is 124 lines (under the ~200-line test-file guide).
The diff touches four files, all part of the seam and its registration
(`mod.rs`). No Context retrofits, no save-convention changes, no SCRM features, and
the commit explicitly records no merge/push. The coder's recorded scoped
verification (check / clippy --tests -D warnings / fmt --check all clean; scoped
`screen` and `scrm` test runs green) is consistent with the diff; per the gate
policy the authoritative full-suite run is the owner's.

</details>

<details>
<summary>File map</summary>

- `crates/ff-desktop/src/shell/screen_provider.rs` (new, 179 lines) — the
  `ScreenCapture` enum, `PENDING_PROVIDER_REASON`, `active_screen_capture()`
  dispatch keyed on `KindTag`, and three unit tests.
- `crates/ff-desktop/src/shell/commands_scrm.rs` — `snapshot_text_for_active` and
  `active_screen_model` now delegate to the seam; opt-out reason surfaced in the
  SNAPSHOT error; old menu-only check removed.
- `crates/ff-desktop/src/shell/tests_screen_directive.rs` (new, 124 lines) — the
  reusable `assert_capture_contract` harness plus the POM and Event Log
  conformance tests.
- `crates/ff-desktop/src/shell/mod.rs` — registers `screen_provider` and the
  `tests_screen_directive` test module.

Full diff: `git -C .worktrees/phase1a-screenprovider diff main...HEAD`

</details>
