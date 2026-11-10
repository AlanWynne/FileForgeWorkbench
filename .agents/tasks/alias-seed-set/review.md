# Expand the FFCMD alias seed set to five rows (X, SS, CS, CT, CR)

The single Rust alias Recovery_Source (`config_bootstrap::write_alias_seed`, shared by first-run write and Reset bare) grows from one row to five via a new `FFCMD_ALIAS_SEED` const: `X -> EXIT`, `SS -> SNAPSHOT`, and the three CAPTURE shortcuts `CS`/`CT`/`CR`. The two-word CAPTURE targets are stored the owner-approved **Option A** way -- canonical root `CAPTURE` plus a parameter template (`START`/`STOP`/`REPLAY`) -- not as a literal `CAPTURE START` canonical, so each row passes `contains_root` validation and inherits the single `cmd:CAPTURE` one-liner. Every row's stored description is empty; the displayed one-liner is inherited at render time from the help topic, unchanged by this diff. The change touches only two files and introduces no new mechanism.

Watch for: the literal brief wording ("CS -> CAPTURE START") is intentionally NOT how the canonical is stored (confirmed); the diff implements the requirement-documented Option A instead, and that deviation is the correct call. A minor test-coverage gap exists around end-to-end resolution of a seed CAPTURE row (likely acceptable).

**Verdict**: APPROVED

## High-level view

The seed definition is now a single 5-row `const FFCMD_ALIAS_SEED: [(&str,&str,&str); 5]`, and `write_alias_seed` maps it into `UserAliasRowToml` rows. Because that one function is the only emitter and both the first-run write and Reset-bare rebuild call it, the two paths cannot diverge (criterion 24.2). No second seed definition exists. This stays squarely inside the CR-CH-116 write-through model and framework-conformance mechanisms 1 and 4.

The brief carried the CAPTURE targets as two-word strings (`CAPTURE START`). Stored literally, those would be rejected by `EnvironmentRegistry::contains_root("FFCMD", ...)` (an exact match where only single-word `CAPTURE` is a root) and would have no matching `cmd:CAPTURE START` help topic, so the inherited description would be empty. The requirement surfaced this at the gate and the owner chose Option A: canonical `CAPTURE` + template subcommand. The implementation follows Option A exactly. `EXIT`, `SNAPSHOT`, and `CAPTURE` are all present in `BUILTIN_WORKSPACE_VERB_ROOTS`, so every seed row's canonical is a valid root.

Descriptions stay empty on disk; display-time inheritance (`alias_description.rs`, CR-NR-119) is not re-implemented and not in the diff. The expansion of `CAPTURE` + template to `CAPTURE START/STOP/REPLAY` rides the existing `rewrite_verb_to_root` pre-pass, which already has its own test suite, so no new resolution path is added.

The TDD test replaces the old single-row assertion with `alias_seed_writes_exactly_the_five_ffcmd_seed_rows`, asserting exactly five rows with canonical uppercase values, Option A templates, and empty descriptions. The alias-editor golden is deliberately untouched because it builds explicit working-copy rows rather than reading the seed, so its `Aliases: 3` band is not in scope for this change.

<details>
<summary>Issues (2)</summary>

1. **Canonical stored as root + template, not literal two-word string** (confirmed) -- intentional and correct per owner-approved Option A (requirement 24.5); the brief's literal wording is superseded by the documented decision. No action.
2. **No end-to-end resolution test for a seed CAPTURE row** (likely) -- the new test asserts the written TOML row shape only; acceptance by `contains_root` and expansion via `rewrite_verb_to_root` rest on pre-existing generic tests rather than a seed-specific one. Optional: add one assertion that a seeded `CS` resolves/expands to `CAPTURE START`. Not a blocker.

</details>

<details>
<summary>Details</summary>

## Single Recovery_Source, no divergence

`write_alias_seed` now builds its rows from the new module-level const rather than an inline `vec![...]` of one entry:

```rust
const FFCMD_ALIAS_SEED: [(&str, &str, &str); 5] = [
    ("X", "EXIT", ""),
    ("SS", "SNAPSHOT", ""),
    ("CS", "CAPTURE", "START"),
    ("CT", "CAPTURE", "STOP"),
    ("CR", "CAPTURE", "REPLAY"),
];
```

The function maps each tuple to a `UserAliasRowToml` with `environment: AliasEnvironment::Ffcmd` and `description: String::new()`, then serialises through the existing `user_aliases::serialise_user_aliases`. This is the only place the seed is defined; `construct.rs` only updates its comments and still calls `first_run_write_builtin_config()` followed by `refresh_alias_store_in_place()` so the seed resolves on the first launch too. Both first-run and Reset-bare go through `write_builtin_defaults` -> `write_alias_seed`, satisfying criterion 24.2 (the two emitters share one definition). No new dispatcher, navigation stack, or persistence format appears -- the on-disk row shape and the User_Alias_Store overlay load path are unchanged (criteria 24.3, 24.7).

## Option A CAPTURE representation

The brief's confirmed seed set names `CS -> CAPTURE START` etc., but storing `CAPTURE START` as the canonical would fail the existing root-validity rule: `contains_root` is an exact full-string match and the FFCMD root set contains single-word `CAPTURE` only (verified in `command_config/mod.rs`, where `EXIT`, `SNAPSHOT`, and `CAPTURE` all appear in the builtin root list). Requirement 24.5 and its OWNER DECISION note record this and mandate Option A: canonical `CAPTURE`, subcommand in `command_template`. The implementation matches that letter for letter. The accepted tradeoff -- `CS`/`CT`/`CR` all inherit the one `cmd:CAPTURE` one-liner -- is explicitly documented in both the requirement and the source doc-comment.

The expansion of a `CAPTURE` + `START` row to the typed `CAPTURE START` form is handled by the pre-existing `rewrite_verb_to_root` pre-pass (`locale_resolver.rs`), which carries its own tests for parameterised-alias expansion (`rewrite_verb_to_root_expands_parameterised_alias`, `..._templateless_alias_is_byte_identical`, `..._is_single_pass`). This change reuses that mechanism rather than adding one.

## Test and golden scope

The replaced test `alias_seed_writes_exactly_the_five_ffcmd_seed_rows` is scenario-named, deterministic (writes to `TempDir`, parses the emitted TOML), and annotated `// Validates: ... configuration-system Requirement 24.1, 24.2, 24.3, 24.5`. It asserts exactly five rows, each surface/canonical/template, and an empty stored description per row -- directly covering the "bare/reset seed yields exactly the five aliases with correct values and empty descriptions" requirement.

The alias-editor golden (`tests_alias_golden.rs`) builds its rows explicitly via `three_rows()` and asserts `Aliases: 3`; it does not read the seed, so it is correctly left unchanged (the commit notes this). The brief's "any golden/snapshot that shows ... Aliases: 5" is conditional, and no golden exercises the seed count, so there is nothing to update. The one item the brief lists that is not directly asserted by a new test is end-to-end resolution/CE-acceptance of a seed CAPTURE row; it is covered transitively by the established `rewrite_verb_to_root` and root-set tests rather than a seed-specific case. Minor, non-blocking.

## Standards and hygiene

The diff is ASCII-only (documentation-charset). The const is 7 lines and `write_alias_seed` stays well under the 40-line function limit. `config_bootstrap.rs` is 418 physical lines but the majority sits inside `#[cfg(test)] mod tests`, which the 400-line rule excludes; non-test code is well within budget. The single commit message contains no semicolons and records the scoped verification the coder ran: `cargo check -p ff-desktop` (ok), `cargo test -p ff-desktop config_bootstrap` (7 passed, 0 failed including the new test), `cargo clippy -p ff-desktop -- -D warnings` (clean), `cargo fmt -p ff-desktop -- --check` (no diffs). The working tree is clean and `.agents/` is not committed.

</details>

<details>
<summary>File map</summary>

- `crates/ff-desktop/src/shell/config_bootstrap.rs` -- new `FFCMD_ALIAS_SEED` const (5 rows); `write_alias_seed` maps it instead of a single inline row; doc-comments updated; the seed test replaced with the five-row assertion.
- `crates/ff-desktop/src/shell/construct.rs` -- comment-only updates on the first-run write hook and the in-place alias-store refresh to reflect five seed aliases.

Full diff: `git -C .worktrees/alias-seed-set diff 09757dd...HEAD`

</details>
