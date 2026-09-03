# Test-Driven Development — MANDATORY

TDD is non-negotiable. Every piece of implementation code must be preceded by a failing test.

## TDD Cycle — follow for every acceptance criterion

```
1. READ   the acceptance criterion from requirements.md
2. WRITE  the test that directly exercises that criterion
3. RUN    cargo test — confirm the test FAILS (red)
           → If it passes before implementation, the test is wrong — fix it
4. WRITE  the minimum implementation to make the test pass
5. RUN    cargo test — confirm the test PASSES (green)
6. REFACTOR if needed, keeping all tests green
7. REPEAT for the next criterion
```

You may not write a single line of implementation code before steps 2 and 3 are complete.

## Task Completion Gate

A task is only done when:
- Every acceptance criterion has at least one test
- All tests pass (`cargo test` exits 0)
- `docs/quality/TCR.md` is updated

## Test Organisation

- Unit tests: `#[cfg(test)] mod tests { ... }` at the bottom of the source file
- Integration tests: `tests/` directory at crate root, one file per feature area
- Property tests: use `proptest` crate, minimum 100 iterations, with comment:
  ```rust
  // Feature: <sub-project>, Property N: <property statement>
  ```

## Test Naming

Names must describe the scenario and expected outcome as a sentence:

```rust
// Good
fn scroll_past_last_line_clamps_top_line_to_last_page() { ... }

// Bad
fn test_scroll() { ... }
```

## Test Quality Rules

- Tests must be deterministic — no `HashMap` iteration order, system time, or external files outside `tests/fixtures/`
- Each test asserts one primary behaviour
- Use `pretty_assertions::assert_eq!` for diff-friendly output
- Use `tempfile::TempDir` for any test that writes to disk
- Do not `#[ignore]` a failing test — fix it or delete it

## Requirement Coverage Annotation

Every test must link to its acceptance criterion:

```rust
#[test]
fn scroll_down_clamps_at_last_line() {
    // Validates: Requirement 2.4 — Down Arrow advances top_line by 1, clamped
    ...
}
```

## Test Coverage Report (TCR)

The TCR lives at `docs/quality/TCR.md` and is the authoritative record of test status.

| Status | Symbol | Meaning |
|--------|--------|---------|
| PASS | ✅ | Automated test exists and passes |
| FAIL | ❌ | Automated test exists but fails |
| MANUAL | 🔲 | Requires manual/UI verification |
| NOT COVERED | 🔴 | No test exists yet |

TCR is append-only for criteria rows — never remove a row, only update status in-place.

## Test Checklist Verification

Before claiming any task complete:

1. Read `.kiro/test-checklist.json`
2. Check `status` of every `TestEntry` linked to the task's requirements
3. All entries must be **PASS** or **MANUAL_PASS**
4. If any entry is **FAIL**, **NEEDS_RETEST**, or **NOT_COVERED** — report it before claiming completion
