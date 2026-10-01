# Task 21 / Requirement 14 -- Markdown Viewer Shell Integration (CR-CH-049)

The implementer investigated whether the markdown viewer could be wired into the
ff-desktop shell to satisfy custom-file-viewers Requirement 14, and concluded
that doing so requires altering a core framework seam that does not exist today.
Per `framework-conformance.md` that is an owner-confirmation gate, so Task 21 was
DEFERRED: no source was written, the blocker was surfaced to the owner with
options and a recommendation, and the spec/TCR state was left honest (Task 21
`[ ]`, Req 14 rows NOT COVERED). This is one of the two explicitly valid
outcomes for this review.

Watch for: nothing blocking. The deferral rests on three factual claims about
the shell -- no `ff-viewers`/`ff-mdx-plugin` dependency, no shell-owned viewer
registry, and no `viewer.preview` on the single-dispatch path -- all three of
which I verified directly against the code this pass (confirmed).

**Verdict**: APPROVED

## High-level view

Requirement 14 is tagged `[framework]` and demands the markdown viewer be
registered in the shell's `ff-viewers` Viewer_Registry (14.1) and reachable
ONLY through a registered Command_Id resolved by
`ff_command::resolve_target -> dispatch_command_target` (14.2), with menu/
shortcut parity (14.3). The implementer's investigation establishes that none of
the enabling seams exist in `ff-desktop`: the crate does not depend on
`ff-viewers`, there is no `ViewerRegistry` constructed or owned by the shell, and
there is no `viewer.preview` command on the single dispatch path. The
`ff-viewers` `PreviewCommand` is a self-contained handler that operates on a
registry/panel handed to it directly; it is not an `ff_command::CommandHandler`
and is not registered into the shell's `CommandRegistry`.

Because satisfying Req 14 would require creating a new viewer-to-shell dispatch
seam (a shell-owned registry plus bridging `viewer.preview` onto the single
dispatch path), `framework-conformance.md`'s "new command-dispatch path"
clause applies -- an owner-confirmed framework change, not an additive wiring
step. Deferring and surfacing the decision is the conformant action, not a
shortfall. The plan documents three options (ViewerPanel dock, WorkspaceContext,
defer) with tradeoffs and a recommendation, which is exactly what the gate asks
for.

The spec state is consistent with a deferral: Task 21 remains `[ ]` with a
BLOCKED sub-note, its subtasks 21.1-21.4 are unchecked, and the custom-file-
viewers Req 14 TCR rows are NOT COVERED (the Req 14 PASS rows that exist in TCR
belong to unrelated specs -- startup-and-session POM and theme-and-appearance --
not to this spec). Tasks 13-20 are genuinely complete and the two MANUAL rows
(16.6 rfd dialogs, 17.4 registry write) carry the correct testing.md exception
reasons.

<details>
<summary>Issues (0)</summary>

No blocking or non-blocking findings. One pre-existing, out-of-scope observation
is noted in the details (ff-viewers `.rs` ASCII), which the plan already flagged
and correctly excluded from Task 21.

</details>

<details>
<summary>Details</summary>

### The deferral is the correct framework-conformance outcome

Req 14 is explicitly a `[framework]` requirement whose criterion 14.2 pins the
viewer to the single command-dispatch path (`resolve_target ->
dispatch_command_target`) and forbids a bespoke intercept or parallel
dispatcher. framework-conformance.md mechanism 1 treats "a new command-dispatch
path or intercept" as a change that needs express owner confirmation. The
implementer correctly reasoned that there is no existing `viewer.preview` seam to
register against, so wiring one in is the creation of that seam, not an additive
registration into an already-wired path. The plan's line-1 token
`TASK21_FRAMEWORK_CHANGE` and the owner-decision section (options A/B/C with
tradeoffs and a recommendation of A, fallback C) match the brief's Outcome 1
precisely.

### Claim verification (the deferral's load-bearing facts)

I did not re-run suites; I spot-checked the three facts the deferral depends on.

Dependency absence (confirmed): `crates/ff-desktop/Cargo.toml` lists ~45 path
dependencies; none is `ff-viewers`, `ff-mdx-plugin`, or `ff-md-viewer`. It
depends on `ff-plugin`, but only as the Plugin Manager display enum per the plan.

No shell viewer seam (confirmed): a grep across `crates/ff-desktop/**/*.rs` for
`viewer.preview`, `ViewerRegistry`, `ff-viewers`, `ff_viewers`, `MdxPlugin`, and
`register_viewer` returned zero matches. There is no shell-owned registry and no
viewer command registration.

PreviewCommand is not on the single dispatch path (confirmed):
`crates/ff-viewers/src/command.rs` defines `PreviewCommand<'a>` holding
`&ViewerRegistry` + `&mut ViewerPanel` + selector, with `PREVIEW_COMMAND_ID =
"viewer.preview"`. It is a standalone handler invoked directly, not an
`ff_command::CommandHandler`, and nothing registers it into the shell's
`CommandRegistry`. So even inside `ff-viewers`, PREVIEW is not plumbed into the
shell dispatch chain -- the bridge is exactly the missing seam.

### Spec and TCR state is honest

`docs/specs/custom-file-viewers/tasks.md` Task 21 is `[ ]` with a BLOCKED
sub-note explaining the missing dependency and seam, and subtasks 21.1-21.4
remain unchecked -- no premature `[x]`. The Req 14 rows for custom-file-viewers
are NOT COVERED; the `Req 14.*` PASS rows surfaced by a TCR grep all belong to
other specs (startup-and-session POM options, theme-and-appearance tokens), so no
Req 14 row for this spec was falsely flipped to PASS. Tasks 13-20 are `[x]` and
the two MANUAL rows (16.6 OS-native rfd dialogs; 17.4 HKCU registry write) carry
reasons that match the testing.md exception list.

### No source written

The plan states no source file was modified, and the absence of any ff-viewers
wiring in ff-desktop (grep + Cargo.toml) is consistent with that. There is
therefore no new `.rs` to measure against the 400-line / thiserror / no-unwrap /
ASCII rules for this task. The only ASCII observation -- em dashes in
`ff-viewers` source and in the ff-desktop package `description` -- is pre-existing
and outside the four markdown crates and Task 21's scope; the plan already flagged
the ff-viewers case and correctly declined to fix it under this task.

### Why not demand an implementation

The brief is explicit that a correct deferral is APPROVE-able and that the
reviewer must not demand an implementation requiring unconfirmed framework
changes. Options A and B in the plan would both introduce the viewer-to-shell
dispatch seam (and B additionally the WorkspaceContext/InteriorFocus + full-shell
first-Tab egui_kittest obligations). Forcing either without owner sign-off would
itself violate framework-conformance.md. Deferral is the conformant choice.

</details>

<details>
<summary>File map</summary>

No source files changed under Task 21 (deferred). Documents/inputs examined:

- `.agents/tasks/md-viewer-task21/task21-plan.md` -- investigation + owner-decision plan (line 1: `TASK21_FRAMEWORK_CHANGE`).
- `docs/specs/custom-file-viewers/requirements.md` -- Req 13-16 read; Req 14 is `[framework]`.
- `docs/specs/custom-file-viewers/tasks.md` -- Task 21 `[ ]` BLOCKED; Tasks 13-20 `[x]`.
- `docs/quality/TCR.md` -- confirmed no custom-file-viewers Req 14 row flipped to PASS.
- `crates/ff-desktop/Cargo.toml` -- confirmed no ff-viewers/ff-mdx-plugin/ff-md-viewer dep.
- `crates/ff-desktop/**/*.rs` (grep) -- confirmed no viewer registry / viewer.preview / register_viewer.
- `crates/ff-viewers/src/command.rs` -- confirmed PreviewCommand is self-contained, off the single dispatch path.

</details>
