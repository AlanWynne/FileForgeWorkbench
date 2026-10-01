# Implementation Tasks

Tasks for CR-NR-102 (ffmdx-app). TDD-first; every test carries `// Validates: Requirement X.Y`. Scoped checks only (`cargo test -p ff-app-bootstrap`, `cargo test -p ff-mdx-app`, `cargo test -p ff-desktop`); the full gate is the owner's manual step. Depends on CR-NR-101 (`ff-md-style`) for Task 6.

- [ ] 1. Scaffold the `ff-app-bootstrap` crate (Req 1.1, 1.5, 5)
  - [ ] 1.1 Create `crates/ff-app-bootstrap` with Cargo.toml (deps: ff-logging, ff-config, ff-theme, egui, eframe, anyhow; NO editor/command/vfs/shell deps) and add to workspace members
  - [ ] 1.2 Write `options.rs` (BootstrapOptions, BootstrapResult, ConfigHome) and `lib.rs` coordinator
  - [ ] 1.3 Test: the crate's dependency set excludes editor/command/vfs/toolchain/plugin/shell crates (Req 1.5, 5) -- a compile-level/dependency assertion

- [ ] 2. Bootstrap sequence (Req 1.1, 1.4)
  - [ ] 2.1 Write `run.rs` AppBootstrap::run: init ff-logging -> load config (consumer schema closure) -> resolve ff-theme palette, in order (Req 1.1)
  - [ ] 2.2 BootstrapOptions carries app identity, config home, title, default size, profile (Req 1.4)
  - [ ] 2.3 Test (tempdir config home): run returns a ConfigHandle, LoggingStatus, and resolved palette

- [ ] 3. Apply fonts + palette to egui (Req 3.2)
  - [ ] 3.1 Write `egui_apply.rs` apply_to_egui: build egui::Visuals from palette tokens (reuse the ff-desktop apply_theme pattern) + set monospace/proportional fonts from FontConfig
  - [ ] 3.2 egui_kittest test: apply_to_egui sets expected Visuals (panel fill / text colour) and font sizes

- [ ] 4. Refactor ffwb onto the bootstrap (Req 1.3 -- behaviour-preserving)
  - [ ] 4.1 Replace the inline logging+config+palette block in ff-desktop/main.rs with AppBootstrap::run, passing ffwb identity + a ffwb schema-registration closure (keep register_builtin_schema contents)
  - [ ] 4.2 Keep ffwb-only steps (tokio, WorkbenchApp, CLI/batch/fftest, WorkbenchShell) in ff-desktop
  - [ ] 4.3 Confirm existing ff-desktop startup tests still pass; add a test asserting the bootstrap path yields the same effective logging/theme config as before (Req 1.3)

- [ ] 5. ffmdx config environment (Req 2.1-2.6)
  - [ ] 5.1 Write `config.rs` in ff-mdx-app: resolve ffmdx's own config home via ff-config; ffmdx schema keys (theme.active, theme.active_name, markdown.style_file, logging.level + dir/rotation) with defaults (Req 2.5)
  - [ ] 5.2 Test: ffmdx config home is distinct from ffwb's; absent file/key falls back with DEBUG log (Req 2.3, 2.6); invalid TOML retains defaults + warning (Req 2.4)

- [ ] 6. Markdown rendering via ff-md-style (Req 6.1-6.4) -- depends on CR-NR-101
  - [ ] 6.1 ffmdx holds a MarkdownStyleHandle loaded from markdown.style_file; viewer.rs applies EguiStyleMapping + configure_viewer before CommonMarkViewer::show (Req 6.1)
  - [ ] 6.2 HTML export uses build_standalone_html_with_css + emit_css (Req 6.2)
  - [ ] 6.3 Test: viewer renders with the mapping applied (model-level); export output carries the emitted CSS
  - [ ] 6.4 Re-render on markdown-style/VisualMode change without restart (Req 6.3); no independent styling in ffmdx (Req 6.4)

- [ ] 7. Theme inheritance in ffmdx chrome (Req 3.1-3.6)
  - [ ] 7.1 main.rs builds BootstrapOptions for ffmdx and calls apply_to_egui in the eframe creation closure (Req 3.2)
  - [ ] 7.2 Replace hardcoded RichText colours in app.rs chrome (toolbar/tree/status) with ambient theme Visuals / palette tokens (Req 3.3)
  - [ ] 7.3 Visual_Mode selector writes theme.active via ff-config and re-applies palette next frame (Req 3.4, 3.5)
  - [ ] 7.4 Theme discovery over ffmdx themes dir (Req 3.6)
  - [ ] 7.5 egui_kittest test: a known chrome widget uses a palette-derived colour (Req 3.3)

- [ ] 8. Logging in ffmdx (Req 4.1-4.5)
  - [ ] 8.1 Bootstrap inits ff-logging (two-phase level/dir apply); honour dev-logging gate (Req 4.1, 4.2)
  - [ ] 8.2 Replace status-string-only feedback with log_info/warn/error at folder-open/view/export/error seams, keeping the status line (Req 4.3)
  - [ ] 8.3 Logging level is config-driven (Req 4.4); init failure degrades to disabled with a non-fatal notice (Req 4.5)

- [ ] 9. Session persistence (Req 5.1-5.6)
  - [ ] 9.1 Write `session.rs`: persist/restore WindowGeometryState + last folder + last file via ff-session (Req 5.1)
  - [ ] 9.2 On start with no CLI folder, restore last folder if it exists; re-open last file if present (Req 5.2, 5.3); missing path -> empty state + DEBUG (Req 5.4)
  - [ ] 9.3 CLI folder arg precedence over persisted folder (Req 5.5)
  - [ ] 9.4 Update session on folder/file/geometry change; save on clean exit (Req 5.6)
  - [ ] 9.5 egui_kittest/tempdir tests for restore + CLI precedence

- [ ] 10. Help / About surface (Req 7.1-7.4)
  - [ ] 10.1 Add About/Help (app name+version, usage, active shortcuts) reachable from the toolbar; reuse ff-help primitives where it fits the cut-down scope (Req 7.1, 7.2)
  - [ ] 10.2 Keep Ctrl+O / F5 hardcoded; record ff-keys deferral in code comment + requirements (Req 7.3, 7.4)
  - [ ] 10.3 egui_kittest test: About/Help is reachable and shows version

- [ ] 11. Cut-down surface boundary (Req 8.1-8.4)
  - [ ] 11.1 Enforce the ff-mdx-app dependency allow-list (Req 8.1); test asserts excluded crates are not dependencies
  - [ ] 11.2 Confirm existing behaviours preserved (folder scan, F5 rescan, drag-drop open, filter, HTML export) via the existing + extended tests (Req 8.4)

- [ ] 12. Update TCR rows for every ffmdx-app criterion to PASS as covered; document any MANUAL with a reason
