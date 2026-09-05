# Automated Dialog Testing Framework (FFTest) -- Design

## 1. Overview

The FFTest framework is a native testing subsystem built into FileForgeWorkbench.
It is implemented as a new crate `ff-fftest` and a thin integration layer in
`ff-desktop`.

The design follows the existing command-driven architecture principle: the UI
never directly performs work. This means the majority of testing (Layer 1) is
already achievable via `cargo test` against the command and business logic layers.
FFTest (Layers 2-4) adds the ability to drive the egui shell itself.

---

## 2. Architecture

### 2.1 Testing Layers

```
Layer 1: cargo test (existing)
    -- Unit tests, integration tests, property tests
    -- No GUI required
    -- Target: >= 90% of business logic

Layer 2: FFTest Dialog Automation
    -- FFTest scripts (.fftest) drive the egui shell
    -- Automation IDs identify controls
    -- Assertions verify state

Layer 3: End-to-End Workflow Tests
    -- Multi-step FFTest scripts
    -- Checkpoint validation at each step

Layer 4: Visual Regression
    -- Screenshot capture at checkpoints
    -- Pixel-level comparison against baselines
```

### 2.2 Component Diagram

```
FFTest Script (.fftest)
        |
        v
FFTest Parser (ff-fftest::parser)
        |
        v
FFTest Runner (ff-fftest::runner)
        |
        +---> Automation Layer (ff-fftest::automation)
        |           |
        |           v
        |     egui Widget State (ff-desktop)
        |
        +---> Assertion Engine (ff-fftest::assertions)
        |
        +---> Screenshot Capture (ff-fftest::capture)
        |
        v
Report Generator (ff-fftest::report)
        |
        +---> reports/<run>.json
        +---> reports/<run>.html
```

### 2.3 Crate Structure

A new crate `ff-fftest` is added to the workspace under `crates/ff-fftest/`.

```
crates/ff-fftest/
    src/
        lib.rs          -- public API
        parser.rs       -- FFTest script lexer and parser
        runner.rs       -- sequential command executor
        automation.rs   -- Automation ID registry and control query
        assertions.rs   -- assertion evaluation engine
        capture.rs      -- screenshot capture (optional feature)
        report.rs       -- JSON and HTML report generation
        headless.rs     -- headless execution support
    tests/
        parser_tests.rs
        runner_tests.rs
        assertion_tests.rs
        report_tests.rs
```

`ff-desktop` gains a thin integration layer:

```
crates/ff-desktop/src/
    automation/
        mod.rs          -- AutomationRegistry, register_control()
        ids.rs          -- all Automation ID constants
```

---

## 3. Automation ID System

### 3.1 Design Principle

Automation IDs are stable string constants defined in `automation/ids.rs`.
Every egui widget that needs to be automatable is rendered with its ID registered
in the `AutomationRegistry`.

The registry maps `AutomationId -> ControlState` where `ControlState` captures:
- `visible: bool`
- `enabled: bool`
- `value: Option<String>` (text content or selected item)
- `label: Option<String>`

### 3.2 ID Naming Convention

```
<panel>.<group>.<control>

Examples:
  menu.file.open
  menu.file.save
  button.save
  button.cancel
  textbox.filename
  textbox.command_field
  grid.files_panel.records
  dialog.catalog_manager.name_field
  pom.option.1
  pom.option.exit
  statusbar.message
  statusbar.line_col
  tab.editor.<index>
  tab.pom
  tab.files_panel
  tab.settings
```

### 3.3 Registration Pattern

During each egui frame, the shell calls `automation_registry.begin_frame()` to
clear stale state, then each rendered widget calls `automation_registry.register(id, state)`.
The runner queries the registry between frames.

This is a pull model: the runner does not inject into the render loop; it reads
the registry snapshot after each frame.

---

## 4. FFTest Script Language

### 4.1 Grammar (informal)

```
script      ::= line*
line        ::= comment | command
comment     ::= '#' <text>
command     ::= keyword args
keyword     ::= OPEN | WAIT | CLICK | SELECT | TYPE | PRESS | ASSERT
              | CHECKPOINT | CLOSE | LOAD | VARIABLE
args        ::= <quoted-string>*
```

### 4.2 Command Reference

| Command | Syntax | Description |
|---------|--------|-------------|
| OPEN FILE | `OPEN FILE "<path>"` | Open a file in the editor |
| WAIT WINDOW | `WAIT WINDOW "<title>"` | Wait until window with title is visible |
| CLICK MENU | `CLICK MENU "<path>"` | Click a menu item by dot-path |
| CLICK BUTTON | `CLICK BUTTON "<automation-id>"` | Click a button by Automation ID |
| SELECT MENUITEM | `SELECT MENUITEM "<label>"` | Select a menu item by label |
| TYPE TEXT | `TYPE TEXT "<value>"` | Type text into the focused control |
| PRESS KEY | `PRESS KEY <keyname>` | Press a named key (ENTER, ESCAPE, F3, etc.) |
| ASSERT WINDOW EXISTS | `ASSERT WINDOW EXISTS "<title>"` | Assert window is visible |
| ASSERT TEXT EXISTS | `ASSERT TEXT EXISTS "<text>"` | Assert text appears in active panel |
| ASSERT STATUSBAR CONTAINS | `ASSERT STATUSBAR CONTAINS "<text>"` | Assert status bar text |
| ASSERT FILE OPEN | `ASSERT FILE OPEN` | Assert a file is open in the editor |
| ASSERT CONTROL VALUE | `ASSERT CONTROL VALUE "<id>" "<expected>"` | Assert control value |
| CHECKPOINT | `CHECKPOINT "<name>"` | Capture screenshot at this point |
| CLOSE WINDOW | `CLOSE WINDOW` | Close the active window |
| LOAD PLUGIN | `LOAD PLUGIN "<name>"` | Activate a named plugin |
| VARIABLE | `VARIABLE <name> "<value>"` | Define a script variable |

### 4.3 Variable Substitution

Variables are referenced as `${NAME}` in any argument string:

```
VARIABLE TESTFILE "C:\workspace\test.txt"
OPEN FILE "${TESTFILE}"
```

---

## 5. Headless Execution

### 5.1 CLI Integration

`ff-desktop` gains two new CLI flags:

```
ffwb --run-tests                          # run all tests/dialog/**/*.fftest
ffwb --run-script <path>                  # run a single .fftest file
ffwb --update-baselines                   # update visual regression baselines
```

### 5.2 Headless Mode

When `--run-tests` or `--run-script` is passed, `ff-desktop` starts in headless
mode: the egui context is created with a virtual framebuffer (using `eframe`'s
offscreen rendering support). The runner drives the application through the
automation layer without a visible window.

### 5.3 Exit Codes

| Code | Meaning |
|------|---------|
| 0 | All tests passed |
| 1 | One or more test failures |
| 2 | Script parse error |
| 3 | Runner initialisation failure |

---

## 6. Reporting

### 6.1 JSON Report Schema

```json
{
  "suite": "dialog/open-file",
  "timestamp": "2025-01-01T00:00:00Z",
  "duration_ms": 1234,
  "total": 10,
  "passed": 9,
  "failed": 1,
  "tests": [
    {
      "name": "open_file_test",
      "status": "PASS",
      "duration_ms": 123,
      "assertions": [...]
    }
  ]
}
```

### 6.2 HTML Report

The HTML report is a self-contained single file with:
- Summary table at the top
- Per-test expandable sections
- Embedded base64 screenshots at checkpoints
- Failure details with expected vs actual values

---

## 7. Visual Regression

Screenshots are captured using `eframe`'s texture readback API. Comparison uses
pixel-level diff with a configurable tolerance (default: 0 pixels differ by more
than 5 intensity units).

Baselines are stored as PNG files under `tests/baselines/<checkpoint-name>.png`.

---

## 8. Dependencies

New crate `ff-fftest` depends on:
- `ff-core` (service registry)
- `ff-command` (command dispatch for command-layer testing)
- `serde`, `serde_json` (report serialisation)
- `image` (screenshot comparison, optional feature `visual-regression`)
- `base64` (HTML report screenshot embedding)

`ff-desktop` gains a dependency on `ff-fftest` (dev-dependency only for test builds;
feature-gated for release builds).

---

## 9. Phased Delivery

The implementation is split into 4 phases to keep each task independently
completable and testable:

| Phase | Deliverable |
|-------|------------|
| CK-1 | Requirements gate (this document) -- no code |
| CK-2 | Automation ID infrastructure in ff-desktop |
| CK-3 | FFTest parser, runner, assertion engine in ff-fftest |
| CK-4 | Headless runner, reporting, visual regression |

---

## 10. Relationship to Existing Architecture

### 10.1 Command-Driven Architecture (Req 4 project-master)

FFTest validates the command-driven principle: the majority of tests (Layer 1)
call commands directly without the GUI. FFTest Layers 2-4 only verify that the
correct command is invoked when a button is clicked -- not that the command works
(that is Layer 1's job).

### 10.2 Plugin Architecture (Req 3 project-master)

Plugin dialogs are testable via the same Automation ID mechanism. The `LOAD PLUGIN`
command activates a plugin before test steps run.

### 10.3 GUI Independence (Req 2 project-master)

The headless runner validates the GUI independence principle: the application
must be driveable without a display device.

### 10.4 No New Crate Dependencies on egui in ff-fftest

`ff-fftest` does NOT depend on egui directly. It communicates with `ff-desktop`
through the `AutomationRegistry` interface, which is defined in `ff-core` as a
trait. This preserves the GUI-independence principle.
