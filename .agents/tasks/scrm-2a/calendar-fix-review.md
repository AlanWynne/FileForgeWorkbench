# Snapshot calendar no longer truncates double-digit day numbers

The snapshot calendar builder `calendar_lines` marked "today" with `format!("*{day}").chars().take(2).collect()`, which kept only the first two characters. For any double-digit day (10-31) that meant the `*` plus a single digit, so the captured artifact shipped a wrong date (the 25th read as `*2`). The live on-screen calendar was unaffected; only the SCRM text capture carried the bug. The fix widens every day cell to a fixed three-character column so the leading marker sits in the spare column and never eats a digit: a plain day is right-aligned in three (`  3`, ` 25`), today is `*{day:>2}` (`* 3`, `*25`), and the weekday header is widened to match. Two TDD tests cover the double-digit and single-digit cases.

Watch for: the alignment assertion in both new tests only holds because October 2026 happens to fill complete 7-cell weeks; trailing cells of a partial last week are not padded, so the equal-width assertion would be a false pass risk for a month that ends mid-week (likely). No production impact.

**Verdict**: APPROVED

## High-level view

The fix is a clean, minimal column-width change confined to one function in the capture path. Day numbers are now always preserved because the marker occupies a dedicated column rather than overwriting a digit; the `take(2)` truncation is gone. The capture path reads nothing from egui and introduces no new mechanism, satisfying the SCRM constraints.

The new tests assert the right behavioral properties (full day number present, today marked, columns aligned) and the regression is pinned by an explicit check that no cell equals `*2`. The alignment check leans on a month whose days divide evenly into weeks, so it does not actually exercise the un-padded trailing-week case; that is a coverage gap in the test, not a defect in the shipped code, and does not block the fix.

File size and function length stay within the project's limits (non-test code ends before line 198; `calendar_lines` is about 25 lines), and the doc comment and all strings are ASCII.

<details>
<summary>Issues (2)</summary>

1. **Partial-week alignment untested** — both new tests use October 2026, which fills exactly five 7-cell weeks, so the `week.chars().count() == header.chars().count()` assertion never sees a short trailing week. Trailing cells are not padded, so a month ending mid-week would produce a shorter final row; add a case for such a month (or document that trailing padding is intentionally absent) to make the alignment claim meaningful. (likely)
2. **Double-digit regression check is narrow** — the loop asserts no cell equals the literal `*2`, which pins the exact old-bug symptom but would not catch a different truncation (e.g. a future change producing `*` alone). Low priority given the positive `contains("*25")` assertion already covers the main property. (possible)

</details>

<details>
<summary>Details</summary>

### Column-width fix in the capture path

The change is entirely inside `calendar_lines(year, month, today)`. Blank leading cells, plain days, and the today cell are now all three characters wide, joined by a single space, and the weekday header string was widened from `"Su Mo Tu We Th Fr Sa"` to `" Su  Mo  Tu  We  Th  Fr  Sa"` so the header and a full week row are both 27 characters. The today branch changed from the truncating `format!("*{day}").chars().take(2).collect()` to `format!("*{day:>2}")`, which right-aligns the day number in two columns and places the `*` in the third, preserving both digits for days 10-31 and yielding `* 3` for single digits. The non-today branch widened from `{day:>2}` to `{day:>3}`. This is the smallest coherent change that satisfies "every day 1-31 shows its correct full number, today stays distinguishable, columns stay aligned."

The function derives everything from `days_in_month`, `first_weekday_of_month`, and `month_name` in `primary_option_menu` — no egui read, no new dispatch or mechanism — so the SCRM constraints hold.

### Test coverage and the alignment assertion

`calendar_double_digit_today_keeps_full_day_and_aligns` asserts the joined text contains `*25`, that no space-split cell equals `*2` (the exact regression symptom), and that every week row matches the header width. `calendar_single_digit_today_renders_correctly_and_aligns` asserts `* 3` is present and re-checks row width. Both are annotated `// Validates: screen-snapshot-scrm Req 3.9-3.13` as required, and the pre-existing `full_snapshot_text_contains_calendar` (month header present) is untouched.

The alignment assertion compares each week row's char count to the header's. For October 2026, Oct 1 is a Thursday, so the grid is 4 blank cells + 31 days = 35 cells = exactly five full 7-cell weeks; every row is 27 chars and the assertion passes. Because the builder does not pad a short trailing week, a month that ends mid-week would produce a final row shorter than the header and the same assertion would fail — meaning the test only demonstrates alignment for the even-division case. This is a gap in what the tests prove, not a fault in the shipped behavior, and the task's required single/double-digit cases are both covered.

</details>

<details>
<summary>Files changed</summary>

- `crates/ff-desktop/src/screen_snapshot.rs` — widened `calendar_lines` day cells and weekday header to 3-char columns; replaced the truncating today marker with `*{day:>2}`; added two TDD tests for double- and single-digit today. (+67 / -9)

Full diff: `git -C .worktrees/scrm-2a-fullscreen-providers show 391acb5`

</details>
