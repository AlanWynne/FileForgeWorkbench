# Review: frame-content-rule

Verdict: APPROVED (no blockers). One non-blocking docs nit below.

## Evidence

Commit messages record: ff-screen-model 65 passed; ff-desktop filtered suites (common_frame, content_rule, golden, snapshot, capture, tests_focus, menu) 515 passed 0 failed, including the first-Tab tests; clippy -D warnings clean on both crates; fmt clean. Docs commit says no build run (docs only). I did not re-run the suites.

1. Goldens. I read all 7 files in crates/ff-desktop/src/shell/golden (alias x3 incl. ASCII, config x2, menu_pom, menu_settings). Each has exactly one rule between `Command ===>` and the first content row, and no two consecutive rules anywhere. The Alias table is self-boxed (it has its own corners), so it is correctly left unmerged and the frame rule appears once. The bottom rule before the message/status band is single in every file.
2. Merge tests (framed_rule_tests.rs). The top and bottom merges, unicode and ASCII, plus the message-band variant and the both-ends variant, assert exact rule lines (`got[c+1] == rule(..)` with tee junctions) and call `assert_no_consecutive_rules`. If the renderer emitted two rules, the exact-index assertions and the consecutive-rule scan would both fail. A self-boxed ASCII grid test and an empty-content test cover the false-merge and doubling edges. framed.rs guards the doubling in code too: `!rows.last().is_some_and(is_rule_row)` at both boundaries.
3. Live separator. tests_content_rule.rs asserts the rule's rect is below the command field and that the central area starts at or below it. It also asserts the rule is identical for POM and CONFIG, and that Tab never focuses it. render_content_rule.rs uses `Sense::hover()` only, so it cannot be a focus stop. The first-Tab test files are not in the diff, so they are unchanged. It is wired after the command panel in update.rs (not split) and in render_command_line.rs (detached).
4. Specs. workspace-framework Req 7.30-7.32 are new, with a note on 7.7. screen-snapshot-scrm 30.18 is revised in place, and 30.29 and 30.30 are new. Design and tasks are updated. The added prose is ASCII; the only added non-ASCII is a box-drawing line inside the existing fenced diagram, which is allowed. Tests carry `// Validates:` annotations with the matching requirement numbers.
5. Constraints. state.rs and shell/environment.rs are not in the diff. There is no public ScreenModel, Table or WorkspaceContext change: the new `Row` variants and helpers are private to framed.rs. The clipboard and PNG paths are untouched. framed.rs has 371 non-test lines (the test module starts at line 372), under 400. The new files are small. The commit log shows no merge or push, and no parked branches are touched.

## Non-blocking note

docs/status/change-log.md line 40: the new CR-NR-115 entry is glued onto the end of the previous bullet (`...through reported_text.- **Revision (...Content_Area top-boundary rule...`) with no line break. The content is present but renders as one bullet. Suggest a newline before `- **Revision`. Cosmetic only.
