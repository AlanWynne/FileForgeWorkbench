# Automated Dialog Testing Framework (FFTest) -- Requirements

## Introduction

This specification defines the requirements for the FFTest Automated Dialog Testing
Framework, a native testing subsystem built into FileForgeWorkbench.

The framework enables repeatable, automated validation of all user interface
interactions, dialogs, screens, workflows, commands, plugins, and business functions
without requiring continuous manual user intervention.

The framework is designed around four testing layers:

- Layer 1: Unit/command-layer tests (no GUI required) -- existing `cargo test`
- Layer 2: Dialog automation via FFTest scripts
- Layer 3: End-to-end workflow tests
- Layer 4: Visual regression screenshot comparison

This specification covers Layers 2-4 and the infrastructure that supports them.
Layer 1 is already covered by the existing TDD workflow and `cargo test`.

## Glossary

| Term | Definition |
|------|-----------|
| FFTest Script | A human-readable plain-text file (`.fftest`) containing automation commands |
| FFTest Runner | The engine that parses and executes FFTest scripts |
| Automation ID | A stable string identifier assigned to every UI control |
| Assertion | A script command that verifies an expected application state |
| Baseline | A reference screenshot used for visual regression comparison |
| Headless | Execution without an attached display device |
| Checkpoint | A named point in a script where a screenshot is captured |
| Dialog Automation Layer | The subsystem that maps Automation IDs to egui widget state |

---

## Requirement 1 -- Framework Foundation

**User Story:** As a QA engineer, I want a native automated testing framework built
into FileForgeWorkbench, so that I can validate all dialogs and workflows without
manual intervention.

### Acceptance Criteria

1.1 THE FileForgeWorkbench platform SHALL provide an integrated automated dialog
    testing framework named FFTest.

1.2 THE FFTest framework SHALL support execution without user interaction.

1.3 THE FFTest framework SHALL be cross-platform, executing on Windows, Linux,
    and macOS without platform-specific test scripts.

1.4 THE FFTest framework SHALL be designed such that not less than 90% of business
    logic testing can be performed without requiring the graphical user interface
    to be loaded (command-layer testing via `cargo test`).

1.5 THE FFTest framework SHALL maintain a version-controlled repository of test
    scripts, baselines, test data, and execution reports under `tests/` and
    `reports/` at the workspace root.

---

## Requirement 2 -- Automation Identifiers

**User Story:** As a test author, I want every UI control to have a stable identifier,
so that my tests survive UI redesigns and do not rely on screen coordinates.

### Acceptance Criteria

2.1 EACH user interface component in `ff-desktop` SHALL expose a stable automation
    identifier string (Automation ID).

2.2 Automation IDs SHALL follow a dot-separated hierarchical naming convention:
    `<panel>.<group>.<control>` (e.g., `menu.file.open`, `button.save`,
    `textbox.filename`, `grid.records`).

2.3 THE framework SHALL NEVER rely solely on screen coordinates, mouse locations,
    or window positions to identify controls.

2.4 THE following control types SHALL be automatable: menus, toolbars, buttons,
    text boxes, tables/grids, tree controls, dialog windows, tabs, console windows,
    ISPF panels, dataset browsers, and plugin-provided dialogs.

2.5 WHEN a UI control is rendered by egui, THE automation subsystem SHALL be able
    to query its current state (visible, enabled, value, label) by Automation ID
    without requiring a display device.

---

## Requirement 3 -- FFTest Scripting Language

**User Story:** As a test author, I want a human-readable scripting language for
defining automated tests, so that non-developers can write and maintain test scripts.

### Acceptance Criteria

3.1 THE FFTest scripting language SHALL provide the following command categories:
    navigation (OPEN, CLOSE, WAIT), interaction (CLICK, TYPE, PRESS, SELECT),
    assertion (ASSERT), and control flow (CHECKPOINT, COMMENT).

3.2 THE following script commands SHALL be supported:

    OPEN FILE "<path>"
    WAIT WINDOW "<title>"
    CLICK MENU "<path>"
    CLICK BUTTON "<automation-id>"
    SELECT MENUITEM "<label>"
    TYPE TEXT "<value>"
    PRESS KEY <keyname>
    ASSERT WINDOW EXISTS "<title>"
    ASSERT TEXT EXISTS "<text>"
    ASSERT STATUSBAR CONTAINS "<text>"
    ASSERT FILE OPEN
    CHECKPOINT "<name>"
    CLOSE WINDOW

3.3 THE FFTest script parser SHALL be case-insensitive for command keywords.

3.4 THE FFTest script parser SHALL treat lines beginning with `#` as comments.

3.5 WHEN a script command references an Automation ID that does not exist, THE
    runner SHALL record a diagnostic failure with the script file name, line
    number, and the unresolved ID.

3.6 THE FFTest scripting language SHALL support parameterised scripts via
    variable substitution using `${VARIABLE_NAME}` syntax.

---

## Requirement 4 -- Script Execution

**User Story:** As a QA engineer, I want to execute FFTest scripts automatically,
so that I can run regression tests without manual steps.

### Acceptance Criteria

4.1 WHEN a dialog script is executed, THE FFTest runner SHALL validate all
    assertions contained within the script.

4.2 THE FFTest runner SHALL process script commands sequentially in file order.

4.3 WHEN an assertion fails, THE runner SHALL record diagnostic information
    including: script file, line number, assertion text, expected value,
    and actual value.

4.4 WHEN a test execution completes, THE runner SHALL generate a pass/fail
    summary including total assertions, passed count, failed count, and
    execution duration.

4.5 WHILE executing automated tests, THE runner SHALL continue processing
    user interface events and background tasks to ensure realistic application
    behaviour.

4.6 WHILE executing end-to-end workflow tests, THE runner SHALL validate
    expected outcomes at each workflow checkpoint.

---

## Requirement 5 -- Recording and Playback

**User Story:** As a test author, I want to record my interactions and replay them,
so that I can create regression tests from real usage without writing scripts manually.

### Acceptance Criteria

5.1 WHEN a user starts test recording, THE framework SHALL capture all supported
    user interactions (clicks, keystrokes, menu selections, text input).

5.2 WHEN test recording ends, THE framework SHALL generate an executable FFTest
    script from the recorded interactions.

5.3 Recorded scripts SHALL be executable without modification.

5.4 THE recording subsystem SHALL emit Automation IDs in generated scripts, not
    screen coordinates.

---

## Requirement 6 -- Headless Execution

**User Story:** As a CI/CD engineer, I want to run FFTest scripts on headless
machines, so that automated tests can run in pipelines without a display.

### Acceptance Criteria

6.1 WHILE executing headless tests, THE framework SHALL support operation without
    an attached display device.

6.2 THE FFTest runner SHALL be invocable from the command line:

    ffwb --run-tests
    ffwb --run-script tests/dialog/open-file.fftest

6.3 WHEN CI/CD integration is configured, THE framework SHALL return process exit
    codes suitable for pipeline execution: 0 for all tests passed, non-zero for
    any failure.

6.4 THE headless runner SHALL support the following CI/CD environments:
    GitHub Actions, GitLab CI, Azure DevOps, Jenkins, and local build pipelines.

---

## Requirement 7 -- Reporting

**User Story:** As a QA engineer, I want test execution reports in standard formats,
so that I can review results in browsers and integrate with CI dashboards.

### Acceptance Criteria

7.1 THE framework SHALL generate machine-readable test results in JSON format
    after every test run.

7.2 THE framework SHALL generate human-readable test reports in HTML format
    after every test run.

7.3 THE JSON report SHALL include: test suite name, execution timestamp, total
    duration, per-test pass/fail status, assertion details, and error messages.

7.4 THE HTML report SHALL include: summary table, per-test expandable sections,
    embedded screenshots at checkpoints, and stack traces for failures.

7.5 WHERE screenshot capture is enabled, THE framework SHALL record screenshots
    at configured checkpoints and embed them in the HTML report.

7.6 THE framework SHALL write reports to `reports/` at the workspace root.

---

## Requirement 8 -- Visual Regression Testing

**User Story:** As a QA engineer, I want to compare screenshots against baselines,
so that rendering regressions are detected automatically.

### Acceptance Criteria

8.1 WHERE visual regression testing is enabled, THE framework SHALL compare
    screenshots against baseline images stored in `tests/baselines/`.

8.2 WHEN a screenshot differs from its baseline by more than the configured
    tolerance threshold, THE framework SHALL record a visual regression failure.

8.3 THE visual regression subsystem SHALL support the following areas:
    ISPF emulation panels, dataset browsers, hex editors, compare windows,
    tree structures, and editor windows.

8.4 THE framework SHALL provide a command to update baselines from current
    screenshots:

    ffwb --update-baselines

8.5 WHEN a baseline does not exist for a checkpoint, THE framework SHALL create
    it automatically on first run and report the checkpoint as BASELINE_CREATED
    (not a failure).

---

## Requirement 9 -- Plugin Testing

**User Story:** As a plugin developer, I want to test my plugin's dialogs using
FFTest, so that plugin quality is validated with the same tooling as core features.

### Acceptance Criteria

9.1 WHEN a plugin is loaded for testing, THE framework SHALL expose plugin user
    interfaces through the automation subsystem using the same Automation ID
    mechanism as core controls.

9.2 THE FFTest scripting language SHALL support a LOAD PLUGIN "<name>" command
    that activates a named plugin before executing test steps.

9.3 Plugin test scripts SHALL be stored under `tests/plugins/<plugin-name>/`.

---

## Requirement 10 -- Test Repository Structure

**User Story:** As a project maintainer, I want a defined test repository layout,
so that all contributors know where to place test artefacts.

### Acceptance Criteria

10.1 THE test repository SHALL follow this structure:

    tests/
        unit/           -- cargo test unit tests (existing)
        dialog/         -- FFTest dialog automation scripts (.fftest)
        workflow/       -- end-to-end workflow scripts (.fftest)
        visual/         -- visual regression scripts (.fftest)
        plugins/        -- plugin-specific test scripts
        fixtures/       -- test data files
        baselines/      -- reference screenshots for visual regression
    reports/            -- generated HTML and JSON reports (gitignored)

10.2 THE `reports/` directory SHALL be listed in `.gitignore`.

10.3 THE `tests/baselines/` directory SHALL be version-controlled so baseline
    images are shared across the team.
