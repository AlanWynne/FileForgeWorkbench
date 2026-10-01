# Markdown Viewer / Explorer -- Compliance Audit

Read-only audit. No source was modified. All findings are grounded in files read
on disk and greps run against the workspace. Judged literally against the
authoritative steering rules under `.kiro/steering/`.

Scope audited (workspace root `c:\workspace\VSC\FileForgeWorkbench`):

- `crates/ff-mdx-app` -- standalone Markdown Explorer binary (`ffmdx.exe`)
- `crates/ff-mdx-plugin` -- workbench plugin exposing a "Markdown Viewer"
- `crates/ff-md-viewer` -- markdown scan/render/watch library
- `crates/ff-mdx-installer` -- self-extracting installer (`payload.zip` NOT opened)

---

## Executive summary

The user's instinct is correct: this code was **not** built with TDD and it does
**not** meet several of the project's mandatory standards.

The single biggest finding is the user's stated concern, confirmed literally:
**there are zero tests across all four crates.** A grep for
`#[test]` / `#[cfg(test)]` / `egui_kittest` / `mod tests` over
`crates/ff-md*/**/*.rs` returns **no matches**. Every behaviour -- markdown
scanning, HTML rendering, file watching/debounce, file-tree selection, the
viewer load/reload, the filter, keyboard shortcuts, drag-and-drop, the installer
unzip and PATH edit -- ships with no automated coverage. For a workspace whose
testing rule opens with "TDD is non-negotiable" and mandates `egui_kittest`
harness tests for all rendered-widget behaviour, this is a wholesale miss.

The second systemic finding is a **process-compliance gap**: there is no
requirements gate behind this code. There is no `docs/specs/<sub-project>` folder
for a markdown viewer, and `docs/quality/TCR.md` has **no rows** for `ff-md-viewer`,
`ff-mdx-app`, `ff-mdx-plugin`, or `ff-mdx-installer` (the only `markdown` hits in
TCR belong to the unrelated `ff-screen-model` / `ff-scrm` renderers). Per
`workflow.md` the gate (requirements -> design -> tasks -> master -> TCR ->
approval) is mandatory before any source is written; none of that artefact trail
exists for these crates.

The third systemic finding is **framework non-conformance of the integration
story**: `ff-mdx-plugin` is a dead bolt-on. Grepping all of `crates/**/*.rs` for
`MdxPlugin` / `MdxFileViewer` finds references **only inside `ff-mdx-plugin`
itself**. Nothing in `ff-desktop` references the plugin, the viewer, or even the
`FileForgePlugin` / `FileViewer` traits (grep for
`register_plugin|FileForgePlugin|register_viewer|FileViewer` over
`crates/ff-desktop/**/*.rs` = no matches). So as integrated into the workbench
today, the markdown viewer participates in **neither** the single command
dispatch **nor** the `WorkspaceContext` / `InteriorFocus` focus model -- because
it is not wired into the shell at all.

Non-ASCII characters (em dash, ellipsis, emoji) are present in `.rs` source in
all of `ff-mdx-app`, `ff-mdx-plugin`, and `ff-mdx-installer`, violating the
plain-ASCII rule for Rust source.

On the positive side: all `.rs` files are well under the 400-line limit; there
are **no** bare `unwrap()` / `expect()` / `panic!` anywhere in the four crates
(error paths use `unwrap_or*` and `anyhow`); the crates ARE registered as
workspace members; and the pure functions (`render_to_html`, `Scanner::scan`,
the debounce) are small and eminently testable -- the gap is missing tests, not
untestable code.

Important distinction (per the brief): the **standalone `ff-mdx-app` binary is a
separate `ffmdx.exe`** and is **not** bound by the in-shell framework rules
(command dispatch, WorkspaceContext). It IS still bound by `rust-standards.md`,
`testing.md`, and `documentation.md`. The framework-conformance findings below
apply to the **in-workbench integration** (`ff-mdx-plugin` and any `ff-desktop`
wiring), not to the standalone app.

### Severity tally

| Severity | Count | Headline |
|----------|-------|----------|
| Critical | 2 | Zero tests (TDD rule); no requirements gate / spec / TCR |
| High | 3 | Plugin not wired into shell; no command parity; non-ASCII in `.rs` |
| Medium | 4 | Library uses `anyhow` not `thiserror`; `pub` field on `FileWatcher`; GUI behaviours untested; missing `///` docs on public API |
| Low | 3 | Hardcoded version strings; `set_files` dead-ish API; Cargo.toml em dash |

---

## Per-crate findings

### 1. `ff-md-viewer` (library: scan / render / watch)

Files read: `src/lib.rs`, `src/renderer.rs`, `src/scanner.rs`, `src/watcher.rs`,
`Cargo.toml`.

- **Tests: none.** No `#[cfg(test)]` module in any file. This is a pure,
  deterministic library (string in -> HTML out; path in -> sorted `Vec<FileEntry>`
  out; debounce logic) -- it is the *easiest* code in the whole scope to unit-test,
  and the most glaring TDD omission. Untested behaviours:
  - `render_to_html` (`renderer.rs:4`): the enabled option set (tables,
    footnotes, strikethrough, tasklists, smart punctuation) and the HTML output.
  - `Scanner::scan` (`scanner.rs:28`): recursion, the `EXCLUDED` directory list
    (`scanner.rs:3`), the `.md` extension filter, relative-path normalisation
    (`\\` -> `/`, `scanner.rs:52`), and the final sort (`scanner.rs:31`).
  - `FileWatcher` debounce (`watcher.rs:40-47`): the ~400 ms coalescing window
    and the `.md` filter -- genuinely harder to test deterministically, but the
    path-filter predicate can be extracted and tested.
- **Error-handling style (Medium).** `rust-standards.md` says *library* code uses
  `thiserror` with one `Error` enum per crate; application code uses `anyhow`.
  `FileWatcher::new` returns `anyhow::Result<Self>` (`watcher.rs:14`) and the
  crate depends on `anyhow` (`Cargo.toml`). A library surfacing `anyhow` to its
  callers is the documented wrong tool; a `thiserror` `MdViewerError` enum is the
  standard.
- **`pub` field (Medium).** `FileWatcher.rx: Receiver<PathBuf>` is a public field
  (`watcher.rs:9`). `rust-standards.md`: "Avoid `pub` fields unless a plain data
  container; prefer accessors." `FileWatcher` is not a plain data container (it
  owns a live watcher thread), so `rx` should be an accessor (`fn changes(&self)
  -> &Receiver<PathBuf>`). `FileEntry` (`scanner.rs:19`) with two `pub` data
  fields is a defensible plain-data container, so that one is borderline-OK.
- **Missing `///` docs (Medium).** `rust-standards.md`: "Every public item has a
  `///` comment." `FileEntry` and its fields, `Scanner`, and `FileWatcher.rx`
  carry no doc comments. `render_to_html` and `FileWatcher::new` are documented.
- **Positives.** No `unwrap()/expect()/panic!`; files tiny (lib 8, renderer 15,
  scanner ~60, watcher ~55 lines); ASCII-clean; sensible `default-features =
  false` on `pulldown-cmark` and `notify`.

### 2. `ff-mdx-app` (standalone `ffmdx.exe`)

Files read: `src/main.rs`, `src/app.rs`, `src/file_tree.rs`, `src/viewer.rs`,
`Cargo.toml`.

- **Tests: none.** No test module anywhere. GUI behaviours with no coverage
  (these are exactly the `egui_kittest`-mandated kind under `testing.md` "GUI
  Behaviour Testing"):
  - file-tree selection returning the clicked path (`file_tree.rs:show`),
  - root-vs-folder partitioning of entries (`file_tree.rs:21-33`),
  - the filter in the side panel (`app.rs:160-175`, case-insensitive `contains`),
  - viewer load / reload / clear and the empty-state placeholder
    (`viewer.rs:load/reload/clear/show`),
  - the `Ctrl+O` / `F5` shortcuts (`app.rs:140-151`),
  - drag-and-drop of a folder or `.md` file (`app.rs:handle_drops`),
  - watcher-driven auto-reload of the open file (`app.rs:poll_watcher`).
  Pure, non-egui helpers (the filter predicate, the drop-path classification, the
  `strip_prefix` title logic) can be extracted and unit-tested without a harness;
  the focus/selection behaviours want `egui_kittest`.
- **Non-ASCII in source (High).** `documentation.md` "Rust Source Files: Plain
  ASCII only (0x00-0x7F)". Confirmed offenders (code points verified byte-wise):
  - em dash U+2014: `app.rs:28` (`"Ready — open a folder to get started"`).
  - ellipsis U+2026: `app.rs:105` (`"Export as HTML…"`).
  - emoji: `app.rs:87` folder U+1F4C1, `app.rs:92` U+1F504, `app.rs:104` U+1F4BE,
    `app.rs:168` U+1F50D; `file_tree.rs:39,62` U+1F4C4, `file_tree.rs:49` U+1F4C1;
    `viewer.rs:49,58` U+1F4C4.
    (Emoji in UI button labels is a UX choice, but the rule is literal: `.rs`
    files are ASCII-only. At minimum the em dash and ellipsis must go; the emoji
    are also violations as written.)
- **Error handling: acceptable.** No bare `unwrap()/expect()`. `main.rs:11` uses
  `unwrap_or_default()`; `viewer.rs` uses `unwrap_or_else(|e| format!(...))` for
  read failures; `app.rs` uses `unwrap_or_default()`. `main` returns
  `anyhow::Result` (correct for a binary).
- **File size: OK.** `app.rs` ~210 lines, others smaller -- all under 400.
- **Function length (Low/Medium).** `MdxApp::toolbar` (`app.rs:84-152`) and the
  `eframe::App::update` body (`app.rs:155-200`) each run well past the ~40-line
  guideline and mix concerns (toolbar + export + shortcut handling in one;
  panel layout + filtering in the other). Candidates for extraction.
- **Low.** Hardcoded `"ffmdx v0.1"` label (`app.rs:134`) duplicates the crate
  version rather than reading `env!("CARGO_PKG_VERSION")`.

### 3. `ff-mdx-plugin` (in-workbench plugin)

Files read: `src/lib.rs`, `src/viewer.rs`, `Cargo.toml`.

- **Not wired into the shell (High) -- framework-conformance.** `MdxPlugin` and
  `MdxFileViewer` are referenced nowhere outside this crate (grep over
  `crates/**/*.rs`). `ff-desktop` references neither this plugin nor the
  `FileForgePlugin` / `FileViewer` traits at all. Consequences measured against
  `framework-conformance.md` and `workspace-conformance.md`:
  - **Command dispatch (High).** `framework-conformance.md` mechanism 1 +
    `workflow.md` 1b "command parity": every user action must resolve through
    `ff_command::resolve_target` -> `target_dispatch`. There is no command
    (`PREVIEW`/`VIEW`/open-markdown) registered for this viewer and no menu/button
    invoking one. The viewer cannot be invoked by the user through the single
    dispatch path because it is not registered anywhere.
  - **WorkspaceContext / InteriorFocus (High if/when rendered in-shell).**
    `workspace-conformance.md` requires every shell-rendered Context to report
    interior focus via `WorkspaceContext::render -> InteriorFocus` (or documented
    `InteriorFocus::none()`), plus a full-shell first-Tab `egui_kittest` test.
    `MdxFileViewer::render` returns a `String` of HTML via the `ff-viewers`
    `FileViewer` trait; it is not a `WorkspaceContext` and has no `InteriorFocus`
    contract or Tab-order test. If this viewer is ever surfaced as a shell
    Context, it must go through that framework, not a bespoke arm.
  - Net: today it is an **orphan** -- compiled as a workspace member but never
    constructed by the app. That is a real integration gap, and it is also why no
    in-shell framework rule is currently satisfied.
- **Tests: none.** No coverage of `MdxFileViewer`'s `can_render` (extension
  match, `viewer.rs:30`), `render` (UTF-8-lossy decode + `render_to_html`,
  `viewer.rs:34`), or the capability/metadata wiring in `lib.rs`. Contrast:
  the trait it implements, `ff-viewers::trait_def::FileViewer`, DOES ship a
  `#[cfg(test)]` module with `// Validates: Requirement X.Y` annotations -- the
  house style this crate ignores.
- **Non-ASCII (High).** `viewer.rs:36` comment contains em dash U+2014
  ("Stateless renderer — nothing to invalidate").
- **Capability/description duplication / drift (Low).** `lib.rs` advertises only
  `mime_types: ["text/markdown"]` while `viewer.rs` lists both `text/markdown`
  and `text/x-markdown` and extensions `md`/`markdown`; the two descriptions also
  differ ("Renders .md files via pulldown-cmark" vs "...to HTML using
  pulldown-cmark"). Not a rule violation per se, but it is the kind of
  inconsistency a spec + tests would have caught.
- **Positives.** No `unwrap/expect/panic`; both files tiny; trait impl is clean.

### 4. `ff-mdx-installer`

Files read: `src/main.rs`, `Cargo.toml`. `src/payload.zip` deliberately not
opened (binary).

- **Tests: none.** `install()` (`main.rs:unzip loop`) and `add_to_user_path()`
  (`main.rs`, Windows registry edit) have no tests. The PATH-dedup predicate
  (`split(';')` + `eq_ignore_ascii_case`) and the unzip path-join logic are
  unit-testable without touching the real registry if factored out. The registry
  write itself is a reasonable `MANUAL` candidate, but the rule requires the
  reason to be recorded (there is no TCR row at all).
- **Non-ASCII (High).** em dash U+2014 at `main.rs:8` (`"ffmdx — Installer"`);
  emoji U+2705 at `main.rs:68` and U+274C at `main.rs:79`.
- **Security note (informational, not a steering violation).** `add_to_user_path`
  edits `HKCU\Environment` `Path`. It is read-modify-write with a dedup check and
  only touches the per-user hive (not system PATH), which is the safe choice.
  Worth a test around the dedup so a future change cannot silently duplicate or
  clobber the user's PATH.
- **Positives.** No bare `unwrap/expect`; binary `main` returns
  `anyhow::Result`; `install` propagates errors with `?`; file under 400 lines.

### Workspace membership (verified)

Root `Cargo.toml` lists all four as members (lines 87-92: `ff-md-viewer`,
`ff-mdx-app`, `ff-mdx-installer`, `ff-mdx-plugin`), under a "Viewer / Exporter
family" comment. So they build in the workspace. But membership is the *only*
link to the app for the plugin: `ff-desktop` never constructs `MdxPlugin`, so at
runtime the workbench has no markdown viewer. `ff-mdx-plugin` depends on
`ff-plugin`, `ff-viewers`, `ff-html-export`, `ff-logging`, `ff-md-viewer`
(Cargo.toml), yet imports `ff-logging` without any log call sites visible in its
two source files (dependency declared, unused -- a minor smell the logging
inventory task 23.5 in `docs/specs/logging-subsystem` would flag).

---

## Findings table

| ID | Severity | Crate | File:line | Standard violated | Description | Recommendation |
|----|----------|-------|-----------|-------------------|-------------|----------------|
| F01 | Critical | all four | (whole crates) | `testing.md` -- "TDD is non-negotiable"; GUI behaviour MUST have `egui_kittest` tests | Zero tests in any of the four crates (`#[test]`/`#[cfg(test)]`/`egui_kittest` grep = 0 matches) | Backfill unit tests for pure logic (render, scan, debounce, can_render, PATH dedup) and `egui_kittest` tests for file-tree selection, filter, viewer load, shortcuts |
| F02 | Critical | all four | n/a | `workflow.md` section 2 (requirements gate); `specs.md` | No `docs/specs/<sub-project>` for the markdown viewer; no requirements/design/tasks; code written with no gate | Create a `markdown-viewer` (or `custom-file-viewers`-aligned) spec with EARS criteria, then back-fill TCR rows and tests to the criteria |
| F03 | High | ff-mdx-plugin | `lib.rs`, `viewer.rs` (whole) | `framework-conformance.md` mech. 1; `workflow.md` 1b command parity | Plugin/viewer referenced only within its own crate; `ff-desktop` never constructs it; no command registered to invoke it | Wire the plugin into the shell's plugin/viewer registry and expose it via a registered Command_ID (e.g. `PREVIEW`/`VIEW`), menu invoking that command |
| F04 | High | ff-mdx-plugin | `viewer.rs:34` | `workspace-conformance.md`; `framework-conformance.md` mech. 5 | If surfaced as a shell Context, viewer has no `WorkspaceContext`/`InteriorFocus` contract and no first-Tab test | When integrating, render via `WorkspaceContext::render -> InteriorFocus` (or documented `none()`) + add full-shell first-Tab `egui_kittest` test |
| F05 | High | ff-mdx-app | app.rs:28,105,87,92,104,168; file_tree.rs:39,49,62; viewer.rs:49,58 | `documentation.md` -- Rust source ASCII-only | em dash U+2014, ellipsis U+2026, and multiple emoji in `.rs` | Replace em dash with `--`/`-`, ellipsis with `...`; remove emoji from source string literals (or move labels to ASCII) |
| F06 | High | ff-mdx-plugin | viewer.rs:36 | `documentation.md` -- Rust source ASCII-only | em dash U+2014 in a comment | Replace with `--` |
| F07 | High | ff-mdx-installer | main.rs:8,68,79 | `documentation.md` -- Rust source ASCII-only | em dash U+2014 and emoji U+2705/U+274C in `.rs` | Replace em dash with `--`; replace/remove emoji in status labels |
| F08 | Medium | ff-md-viewer | watcher.rs:14; Cargo.toml | `rust-standards.md` -- libraries use `thiserror`, not `anyhow` | Library surfaces `anyhow::Result` to callers | Define a `thiserror` error enum for the crate; return it from `FileWatcher::new` |
| F09 | Medium | ff-md-viewer | watcher.rs:9 | `rust-standards.md` -- avoid `pub` fields; prefer accessors | `FileWatcher.rx` is a public field on a non-data type that owns a thread | Make `rx` private; expose `fn changes(&self) -> &Receiver<PathBuf>` |
| F10 | Medium | all four | public items in each crate | `rust-standards.md` -- every public item has a `///` comment | `FileEntry`/fields, `Scanner`, `MdxApp`, `MdxPlugin`, `MdxFileViewer`, installer types lack doc comments | Add `///` docs to all public items |
| F11 | Medium | ff-mdx-app | app.rs:84-152, 155-200 | `rust-standards.md` -- functions ~40 lines, one responsibility | `toolbar()` and `update()` are long and mix concerns | Extract export, shortcut-handling, and side-panel filtering into helpers |
| F12 | Low | ff-mdx-app | app.rs:134 | `rust-standards.md` (maintainability) | Hardcoded `"ffmdx v0.1"` duplicates crate version | Use `env!("CARGO_PKG_VERSION")` |
| F13 | Low | ff-md-viewer | file_tree.rs:11 (app) / scanner API | `rust-standards.md` -- dead/empty API | `FileTree::set_files` only clears selection and ignores its `_files` arg | Remove or implement meaningfully; document intent |
| F14 | Low | ff-mdx-app / ff-mdx-installer | app Cargo.toml:7; (installer desc OK) | `documentation.md` (spirit; `.toml` under `crates/`) | em dash U+2014 in `ff-mdx-app` Cargo.toml `description` | Replace with `--` for consistency with the ASCII rule |
| F15 | Low | ff-mdx-plugin | Cargo.toml; lib.rs | code smell | `ff-logging` dependency declared but no log call sites in source | Add logging at plugin lifecycle seams or drop the dependency |

Line numbers are from the files as read in this session. Multi-site findings
(F05, F07) list every confirmed offending line from the byte-level scan.

---

## TDD / test-coverage gap (the user's primary concern)

**Finding: no automated tests exist in any of the four crates.** Evidence: a
grep for `#[test]|#[cfg(test)]|egui_kittest|mod tests` across
`crates/ff-md*/**/*.rs` returns zero matches. There are also no `tests/`
integration directories (the directory trees show only `src/` + `Cargo.toml`,
plus `payload.zip` for the installer).

This directly violates `testing.md`:
- "TDD is non-negotiable. Every piece of implementation code must be preceded by
  a failing test." -- No implementation here was test-first.
- "WHENEVER an acceptance criterion is about RENDERED WIDGET BEHAVIOUR, it MUST
  have an `egui_kittest` harness test." -- All GUI behaviour is untested.
- "Every test links to its criterion" via `// Validates: Requirement X.Y`. -- No
  such annotations can exist because there are neither tests nor requirements.

### Untested behaviours, grouped

Pure / headless-testable (unit tests, no harness needed):

- `render_to_html` option set and output (`ff-md-viewer/src/renderer.rs:4`).
- `Scanner::scan`: recursion, `EXCLUDED` dirs, `.md` filter, `\\`->`/`
  normalisation, sort (`ff-md-viewer/src/scanner.rs`).
- `FileWatcher` `.md` path predicate and the ~400 ms debounce coalescing
  (`ff-md-viewer/src/watcher.rs:40-47`) -- debounce needs care to keep
  deterministic (`testing.md`: no timing flakiness); extract the predicate.
- `MdxFileViewer::can_render` extension matching and `render` UTF-8-lossy path
  (`ff-mdx-plugin/src/viewer.rs`).
- Installer PATH dedup predicate and unzip path-join (`ff-mdx-installer/src/main.rs`).
- Pure helpers extractable from `ff-mdx-app`: filter predicate (`app.rs:163-172`),
  drop-path classification (`app.rs:handle_drops`), viewer title `strip_prefix`
  logic (`viewer.rs:load`).

GUI / rendered-widget behaviour (`egui_kittest` MANDATORY per `testing.md`):

- File-tree click returns the selected path and marks selection
  (`ff-mdx-app/src/file_tree.rs:show`).
- Side-panel filter narrows the list (`ff-mdx-app/src/app.rs:160-175`).
- Viewer empty-state placeholder vs loaded document (`ff-mdx-app/src/viewer.rs:show`).
- `Ctrl+O` opens picker, `F5` refreshes (`ff-mdx-app/src/app.rs:140-151`).
- Watcher-driven reload updates the open document (`app.rs:poll_watcher`).
- (In-shell) first-Tab focus lands on the viewer's first interior control, no
  phantom stop -- required by `workspace-conformance.md` IF the viewer is ever a
  shell Context.

Justified `MANUAL` candidates (must still be recorded with a reason in TCR):

- `rfd` native folder/save dialogs (`ff-mdx-app/src/app.rs:pick_folder/save_file`)
  -- OS-native dialog, on the `testing.md` exception list.
- Installer registry write to `HKCU\Environment` -- real OS side effect; test the
  pure dedup logic, mark the actual write MANUAL with the reason stated.

---

## Process compliance (gate / specs / TCR)

- **Requirements gate: not run.** `workflow.md` section 2 requires
  requirements -> design -> tasks -> master tasks -> TCR -> approval before any
  source. No evidence of any of these artefacts for the markdown viewer.
- **Spec folder: absent.** No `docs/specs/markdown-viewer` (or similar). The
  closest existing spec is `docs/specs/custom-file-viewers`, which is the natural
  home for a `FileViewer`-based markdown viewer, but it contains no markdown-viewer
  criteria. (The `markdown` hits elsewhere in `docs/specs/**` are the unrelated
  `screen-snapshot-scrm` / `ff-screen-model` Markdown *renderer*, and
  database/export features -- none cover these crates.)
- **TCR: no rows.** `docs/quality/TCR.md` has no rows for `ff-md-viewer`,
  `ff-mdx-app`, `ff-mdx-plugin`, or `ff-mdx-installer`. The `markdown` matches in
  TCR all belong to `ff-screen-model`/`ff-scrm`. Per `testing.md` the TCR is the
  authoritative coverage record; these crates are invisible to it.
- **Command parity (`workflow.md` 1b): not met for the plugin.** No command is
  registered to invoke the markdown viewer, and no menu/shortcut routes to one.
- **Standalone-app caveat.** The standalone `ff-mdx-app` binary is legitimately
  outside the in-shell framework (its own `eframe::App`, own window, own
  `rfd`-based menus). It is NOT exempt from `testing.md`, `rust-standards.md`, or
  `documentation.md`. Those still apply and are the basis for F01, F05, F11-F13.

---

## Prioritised remediation plan

Order chosen so the gate exists before tests reference it, pure tests precede
GUI tests, and integration lands only after the viewer is covered.

1. **Run the requirements gate (unblocks everything).** Decide the home spec --
   recommended `docs/specs/custom-file-viewers` for the in-shell viewer, with the
   standalone `ffmdx` app and installer noted as out-of-shell deliverables. Write
   EARS criteria for: markdown-to-HTML rendering, folder scan + exclusions, live
   reload/debounce, viewer `can_render`/`render`, file-tree selection, filter,
   shortcuts, and (for integration) command invocation + WorkspaceContext focus.
   Add `tasks.md`, master-task lines, and NOT-COVERED TCR rows per criterion.
   (Addresses F02.)

2. **Fix the documentation (ASCII) violations now -- cheap, mechanical.** Replace
   em dashes (`--`), ellipsis (`...`), and emoji in all `.rs` files across
   `ff-mdx-app`, `ff-mdx-plugin`, `ff-mdx-installer`, and the `ff-mdx-app`
   Cargo.toml description. (Addresses F05, F06, F07, F14.)

3. **Back-fill unit tests for pure logic, test-first against the new criteria.**
   `render_to_html`, `Scanner::scan` (+exclusions/sort/normalisation), the
   watcher `.md` predicate, `MdxFileViewer::can_render`/`render`, the installer
   PATH-dedup, and the extracted `ff-mdx-app` helpers. Annotate each
   `// Validates: Requirement X.Y`; flip TCR rows to PASS. (Addresses F01 pure
   half.)

4. **Add `egui_kittest` tests for the GUI behaviours** of `ff-mdx-app`
   (file-tree selection, filter, viewer load, shortcuts, watcher reload). Mark the
   `rfd` dialogs MANUAL with a stated reason. (Addresses F01 GUI half.)

5. **Harden `rust-standards` issues.** Introduce a `thiserror` error enum in
   `ff-md-viewer` and stop leaking `anyhow` from the library (F08); make
   `FileWatcher.rx` private with an accessor (F09); add `///` docs to public items
   (F10); split `MdxApp::toolbar`/`update` into helpers (F11); use
   `CARGO_PKG_VERSION` (F12); resolve `FileTree::set_files` (F13); add logging or
   drop `ff-logging` from the plugin (F15).

6. **Integrate the plugin into the shell, on the framework.** Register
   `MdxPlugin`/`MdxFileViewer` with the shell's plugin/viewer registry; expose it
   through a registered Command_ID so a menu/shortcut and the typed command share
   one dispatch path (F03); if it renders a shell Context, do so via
   `WorkspaceContext::render -> InteriorFocus` and add the full-shell first-Tab
   `egui_kittest` test (F04). This step is itself gated by step 1's criteria.

### What I could not verify

- I did not run `cargo build`/`clippy`/tests (owner's manual gate); line counts
  and the no-test / no-wiring findings come from reading files and greps, which
  is sufficient for those claims.
- `src/payload.zip` was not opened (binary, out of scope), so the installer's
  shipped payload contents are unverified.
- I confirmed the plugin is unreferenced in `ff-desktop` and across `crates/**`;
  I did not exhaustively trace every possible dynamic plugin-loading path (e.g. a
  manifest-driven loader), but no `FileForgePlugin`/`register_*` references exist
  in `ff-desktop`, so no static wiring exists.
- The `ff-viewers::FileViewer` trait (the plugin's integration seam) was read and
  does ship its own tests + `// Validates:` annotations -- cited as the house
  style the markdown crates diverge from.
