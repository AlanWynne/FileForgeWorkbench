# Behaviour-preserving indirection for the unified command front door (B080, Steps 0-1)

This change introduces `WorkbenchShell::dispatch_command_string`, the single
shell-side front door that Steps 1-7 of the dispatch-unify migration will grow
into the one ordered Target_Resolution chain. In Steps 0 and 1 it is pure
indirection plus a computed-but-not-wired verb/arg split: the two outermost
command-string submit sites (`run_command_line` and the `dispatch_bound_command`
FallThrough arm) now call the front door instead of `handle_command` directly,
and the front door delegates straight back to `handle_command` with the original
raw string. A sibling split helper (`split_verb_arg`) and a Function-target
builder (`function_target_with_arg`) are added but exercised only by unit tests.
The bar for this change is that nothing observable changes yet, and the diff
meets it.

Watch for: nothing blocking. The front door computes `(_verb, _arg)` it does not
use (confirmed intentional and clippy-clean via the underscore prefix), and
`function_target_with_arg` carries an `#[allow(dead_code)]` until its Step 2
caller lands. Both are documented dead-until-Step-2 seams, not loose ends.

**Verdict**: APPROVED

## High-level view

The reroute is genuinely a two-site indirection. `dispatch_command_string` has
exactly three references in the crate: the two live submit sites and one
regression test. Every inner re-dispatch the plan said to leave alone -- the
AUTONUM->NUMBER redirect in `commands_ladder_c.rs`, the chained-fastpath segment
loop in `commands_fastpath.rs` -- still calls `self.handle_command(...)`
directly, so the single `begin_command_line`/`finish_command_line` wrap stays at
the outermost submit and inner re-dispatches stay unwrapped (D4). The front door
delegates with the ORIGINAL `raw` string, so the ladder's per-arm parsing is
untouched and the result is byte-identical.

The Step-1 split reuses the existing `verb_arg` rule rather than reimplementing
it: `split_verb_arg` is literally the same `split_once(char::is_whitespace)` +
`trim` body as `verb_arg`, differing only in that it extracts the verb token
instead of taking a known verb. B062 case semantics hold -- the verb is matched
case-insensitively by callers (`eq_ignore_ascii_case` in the tests) and the
argument remainder is returned with case preserved. Critically, the split result
is bound to `_verb`/`_arg` and discarded; the live path still delegates on `raw`,
so Step 1 adds no behavioural effect.

Step 2 is correctly absent: `builtin_workspace_target` remains a `None` stub in
both `command_config/mod.rs` locations, no ladder arm was removed, and
`function_target_with_arg` has no live caller. The notification channel and
`ShellServices` surfaces are not touched by any file in the diff.

The verification evidence is present and specific (serial `cargo test -p
ff-desktop`: 936 passed, 0 failed, including the 454 shell backstop tests and the
3 new tests, each named in the impl note). dispatch.rs is ASCII-only and ~70
non-test lines, well under the 400-line rule.

<details>
<summary>Issues (0)</summary>

No blocking or non-blocking issues. The two dead-until-Step-2 seams
(`_verb`/`_arg` discard, `#[allow(dead_code)]` on `function_target_with_arg`) are
documented and expected for this slice.

</details>

<details>
<summary>Details</summary>

### Step 0 is pure indirection, two sites only

`dispatch_command_string`'s Step-0 body is `self.handle_command(raw)` with the
original string, preceded only by the discarded split. Behaviourally this is
identical to the prior direct `handle_command(&original)` / `handle_command(command)`
calls: the command history `record` at the top of `handle_command`, the stage
ordering, the per-arm parsing, and the engine fallback all run exactly as before.

A crate-wide search for `dispatch_command_string` returns three hits:

```
commands.rs:30         self.dispatch_command_string(&original);   // run_command_line
target_dispatch.rs:92  ResolveOutcome::FallThrough => self.dispatch_command_string(command)
tests_command.rs:704   typed_submit_routes_through_dispatch_command_string
```

The two live hits are precisely the outermost submit sites named in the plan.
`run_command_line` keeps its `begin_command_line()` ... `finish_command_line(&original)`
wrap around the front-door call; the FallThrough arm sits inside
`dispatch_bound_command`, which is called from `dispatch_key_command` inside that
method's own single wrap. No new wrap was introduced.

Inner re-dispatches remain direct `handle_command` calls, as required by D4/D8:

```
commands_ladder_c.rs:133/142  self.handle_command(&redirected);  // AUTONUM -> NUMBER
commands_fastpath.rs:124      self.handle_command(segment);      // chained segment
```

`resolve_pom_option_key` recursion is likewise untouched (it was not rerouted and
does not appear among the `dispatch_command_string` callers). This preserves the
single-wrap invariant: exactly one `begin/finish` wrap at the outermost submit,
inner re-dispatch unwrapped.

### Step 1 split reuses the existing rule and preserves B062

`split_verb_arg` in helpers.rs is the extracting sibling of `verb_arg`, sharing
the identical body:

```rust
pub(super) fn split_verb_arg(cmd: &str) -> (&str, &str) {
    let c = cmd.trim();
    let (head, rest) = c.split_once(char::is_whitespace).unwrap_or((c, ""));
    (head, rest.trim())
}
```

Compare `verb_arg` (same `split_once(char::is_whitespace)` + `trim`), which only
adds the `head.eq_ignore_ascii_case(verb)` gate. This is reuse of the rule, not a
divergent reimplementation (D5). The front door binds the result to
`(_verb, _arg)` with underscore prefixes and does not consume it; the live
delegation remains `self.handle_command(raw)` on the ORIGINAL string, so the
ladder parses as before and nothing observable changes. The split only becomes
live in Step 2.

The two helper unit tests assert what they claim: `single_verb_arg_split_populates_arg_param`
splits `"DOWN 8"`, asserts the verb matches `DOWN` case-insensitively and arg is
`"8"`, then builds `function_target_with_arg("nav.down", "8")` and asserts the
Function target's `params["arg"] == TargetValue::String("8")` (Req 9.2 fold).
`verb_arg_split_preserves_argument_case` splits `"LOCATE Foo"` and asserts the
arg is `"Foo"` with case preserved (B062). These exercise the arg-param helper
only through the test module, never the live path -- matching the Step-1 contract.

### Step 2 correctly not started

`builtin_workspace_target` is still a `None` stub in both impls in
`command_config/mod.rs` (lines 90-92 and 167-169), with the preserved comment
explaining built-in verbs keep their fall-through handling. No ladder arm was
removed. `function_target_with_arg` carries `#[allow(dead_code)] // wired into
the live path in Step 2.` and has no runtime caller. This is the correct state
for a Steps-0-1-only slice.

### Notification channel and ShellServices intact

None of the four edited files (dispatch.rs, mod.rs, commands.rs, target_dispatch.rs)
nor helpers.rs touch the notification queue or the `ShellServices` surfaces. The
mod.rs change is a single `mod dispatch;` line placed alphabetically between
`mod construct;` and `mod external_adapter;`.

### Verification evidence

The impl note records a serial scoped run (`cargo test -p ff-desktop --
--test-threads=1`): 936 passed, 0 failed, 0 ignored, including all 454
`shell::tests_*` backstop tests plus the three new tests, each named explicitly:

- `shell::tests_command::typed_submit_routes_through_dispatch_command_string`
- `shell::dispatch::tests::single_verb_arg_split_populates_arg_param`
- `shell::dispatch::tests::verb_arg_split_preserves_argument_case`

`cargo fmt` clean, `cargo check -p ff-desktop` clean, `cargo clippy -p ff-desktop
--tests` with no warnings. Serial threads were used to avoid the known B048
`FFWB_HISTORY_PATH`/`FFWB_USER_CONFIG_PATH` env-var races, which is the only
acceptable multithreaded noise. dispatch.rs is ASCII-only (grep for non-ASCII
returns no matches) and ~70 non-test lines, under the 400-line rule.

The evidence is specific and complete; no re-run or spot-check was warranted.

</details>

<details>
<summary>File map</summary>

- `crates/ff-desktop/src/shell/dispatch.rs` (new) -- the front door
  `dispatch_command_string` (pure indirection + discarded split), the
  dead-until-Step-2 `function_target_with_arg` helper, and the two in-file unit
  tests.
- `crates/ff-desktop/src/shell/mod.rs` -- adds `mod dispatch;` alphabetically.
- `crates/ff-desktop/src/shell/helpers.rs` -- adds `split_verb_arg`, the
  extracting sibling of `verb_arg` sharing the same split/trim rule.
- `crates/ff-desktop/src/shell/commands.rs` -- `run_command_line` routes through
  the front door inside its existing outcome wrap.
- `crates/ff-desktop/src/shell/target_dispatch.rs` -- the `FallThrough` arm of
  `dispatch_bound_command` routes through the front door.
- `crates/ff-desktop/src/shell/tests_command.rs` -- adds the Step-0 regression
  test `typed_submit_routes_through_dispatch_command_string`.

</details>
