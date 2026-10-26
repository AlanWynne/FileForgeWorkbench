# Gate Policy (FileForgeWorkbench) -- scoped-per-change, full gate daily, gate-before-merge

Workspace-specific gate cadence for this repository. It REFINES the global
`testing.md` "execution split" (agents compile-verify; the owner runs the heavy
suite) with a concrete per-change vs daily cadence agreed for this project. Where
this file and the global `testing.md` overlap, this file governs the CADENCE for
FileForgeWorkbench; the global file still governs everything else (TDD, test
quality, the compile-verify discipline, no silencing failing tests).

## Why

The full `cargo gate` / whole-workspace suite on this machine takes many minutes
and has repeatedly wedged the terminal. Running it on every change throttles the
develop loop. This policy keeps a fast per-change safety net while deferring the
slow full run -- WITHOUT letting logic regressions pile onto `main`.

## The rule

### On EVERY change (fast loop, seconds)
Run the SCOPED trio on the touched crate(s) and READ the logs back (verify by
effect, per `terminal-discipline.md`):

- `cargo check -p <crate>`
- `cargo clippy -p <crate> -- -D warnings`
- `cargo fmt -p <crate> -- --check`

AND run the QUICK scoped tests for the touched crate where they are fast
(`cargo test -p <crate> <focused-name>`), near the change -- this catches logic
regressions close to where they are introduced, not a day later. If a scoped test
risks a long/quiet run, write it, compile-verify, and leave it to the daily full
gate. This is unchanged from the global `testing.md` execution split; this file
only adds the cadence around it.

### DEFER to once a day (slow full run)
The full `cargo gate` / `cargo nextest` whole-workspace suite is run ONCE A DAY
(the owner's authoritative run) for in-progress BRANCH work. Between daily runs,
branch development relies on the scoped trio + quick scoped tests above. The
tradeoff, accepted for this repo: a test (not compile) failure on a work branch
could go unseen until the next daily full gate.

### MANDATORY: full gate GREEN before merging to `main`
The daily-deferral applies ONLY to in-progress branch work. A branch MUST pass a
full `cargo gate` (fmt + clippy + build + whole-suite CLEAN) BEFORE it is merged
to `main`. Merging is the protected checkpoint: logic regressions must not land on
the mainline on a deferred-gate basis. So the merge sequence stays:

1. scoped trio + quick scoped tests during development (every change);
2. a full `cargo gate` on the branch, CLEAN, immediately before merge;
3. merge to `main` only on that green full gate.

If a full gate surfaces only a known-flaky, out-of-scope failure (e.g. a
GUI-harness timing test), re-run that one test in isolation to classify it;
a confirmed flake does not block the merge, but a reproducible failure does.

## Summary

- Every change: `check` + `clippy -D warnings` + `fmt --check` + quick scoped
  tests, SCOPED to the touched crate, logs read back.
- Daily: one full `cargo gate` for branch work.
- Before any merge to `main`: a full `cargo gate` must be CLEAN -- no exceptions
  beyond a classified, out-of-scope flake re-run green in isolation.
