# Automated Dialog Testing Framework (FFTest) -- Tasks

## Phase CK-1 -- Requirements Gate (COMPLETE -- no code)

- [x] 1. Write docs/specs/automated-dialog-testing/requirements.md (Req 1-10)
- [x] 2. Write docs/specs/automated-dialog-testing/design.md
- [x] 3. Write docs/specs/automated-dialog-testing/tasks.md (this file)
- [x] 4. Add Phase CK to docs/specs/project-master/tasks.md
- [x] 5. Add NOT COVERED rows to docs/quality/TCR.md for all new criteria
- [x] 6. Log CR-NR-033 to docs/status/change-log.md
- [x] 7. Add automated-dialog-testing to .amazonq/rules/specs.md sub-project list

## Phase CK-2 -- Automation ID Infrastructure

- [x] 8. Create crates/ff-fftest/ crate scaffold: Cargo.toml, src/lib.rs
  - Satisfies: Req 1.1
- [x] 9. Define AutomationId newtype and ControlState struct in ff-fftest::automation
  - Satisfies: Req 2.1, 2.2
- [x] 10. Define AutomationRegistry trait in ff-fftest (no egui dependency)
  - Satisfies: Req 2.5, design section 10.4
- [x] 11. Implement ShellAutomationRegistry in ff-desktop/src/automation/mod.rs
  - Satisfies: Req 2.1, 2.5
- [x] 12. Define all Automation ID constants in ff-desktop/src/automation/ids.rs
    covering: menu, button, textbox, grid, dialog, pom, statusbar, tab namespaces
  - Satisfies: Req 2.2, 2.4
- [x] 13. Register begin_frame() call in ff-desktop shell render loop
  - Satisfies: Req 2.5
- [x] 14. Register key controls in ff-desktop render functions:
    command field and status bar message registered each frame
  - Satisfies: Req 2.4
- [x] 15. Write unit tests for AutomationRegistry: register, query, begin_frame clears stale
  - Satisfies: Req 2.1, 2.5

## Phase CK-3 -- FFTest Parser, Runner, and Assertion Engine

- [x] 16. Implement `ff-fftest::parser` -- lexer and parser for FFTest script language
  - Satisfies: Req 3.1, 3.2, 3.3, 3.4
- [x] 17. Write parser unit tests: all command keywords, comments, variable substitution,
    unknown command error, missing argument error
  - Satisfies: Req 3.3, 3.4, 3.5, 3.6
- [x] 18. Implement `ff-fftest::runner` -- sequential command executor
  - Satisfies: Req 4.1, 4.2, 4.5
- [x] 19. Implement `ff-fftest::assertions` -- assertion evaluation engine
  - Satisfies: Req 4.3, 4.4
- [x] 20. Wire OPEN FILE, WAIT WINDOW, CLICK BUTTON, TYPE TEXT, PRESS KEY commands
    to `AutomationRegistry` queries
  - Satisfies: Req 3.2, 4.2
- [x] 21. Wire ASSERT commands to `AutomationRegistry` state queries
  - Satisfies: Req 3.2, 4.3
- [x] 22. Implement CHECKPOINT command (screenshot capture stub -- full impl in CK-4)
  - Satisfies: Req 3.2, 4.6
- [x] 23. Implement VARIABLE command and `${NAME}` substitution in all argument strings
  - Satisfies: Req 3.6
- [x] 24. Write runner unit tests: sequential execution, assertion pass/fail, diagnostic
    output on failure, pass/fail summary
  - Satisfies: Req 4.3, 4.4

## Phase CK-4 -- Headless Runner, Reporting, and Visual Regression

- [x] 25. Add `--run-tests` and `--run-script <path>` CLI flags to `ff-desktop`
  - Satisfies: Req 6.2
- [x] 26. Implement headless execution mode in `ff-desktop` (virtual framebuffer)
  - Satisfies: Req 6.1
- [x] 27. Implement exit code logic: 0=pass, 1=failure, 2=parse error, 3=init failure
  - Satisfies: Req 6.3
- [x] 28. Implement `ff-fftest::report` -- JSON report serialisation
  - Satisfies: Req 7.1, 7.3
- [x] 29. Implement HTML report generation with summary table and per-test sections
  - Satisfies: Req 7.2, 7.4
- [x] 30. Implement screenshot capture in `ff-fftest::capture` (eframe texture readback)
  - Satisfies: Req 7.5, 8.1
- [x] 31. Implement visual regression comparison with configurable tolerance
  - Satisfies: Req 8.2
- [x] 32. Implement `--update-baselines` CLI flag
  - Satisfies: Req 8.4
- [x] 33. Implement auto-create baseline on first run (BASELINE_CREATED status)
  - Satisfies: Req 8.5
- [x] 34. Implement LOAD PLUGIN command in runner
  - Satisfies: Req 9.1, 9.2
- [x] 35. Create `tests/` directory structure with `.gitkeep` files and update `.gitignore`
  - Satisfies: Req 10.1, 10.2
- [x] 36. Write integration test: run a minimal .fftest script end-to-end in headless mode
  - Satisfies: Req 6.1, 6.2, 4.1
- [x] 37. Write report unit tests: JSON schema, HTML structure, screenshot embedding
  - Satisfies: Req 7.1, 7.2, 7.3, 7.4
- [x] 38. Update TCR.md to mark implemented criteria as PASS
  - Satisfies: all criteria above
