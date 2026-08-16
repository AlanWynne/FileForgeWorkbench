# Implementation Plan: Custom File Viewers (`ff-viewers`)

## Overview

This task plan implements the `ff-viewers` crate — the extensible file viewer framework for FileForgeWorkbench. The crate provides a `FileViewer` trait, a thread-safe Viewer_Registry, the `PREVIEW` command family, built-in viewer stubs (asa-report, hex, image, csv-table), plugin viewer bridge integration, a DockablePanel-based Viewer_Panel, content matching and selection logic, refresh/debounce handling, and viewer configuration.

**Crate location:** `crates/ff-viewers`
**Upstream dependencies:** `ff-core` (subsystem integration), `ff-layout` (DockablePanel), `ff-command` (Command_Registry), `ff-vfs` (content reads), `ff-plugin` (PluginContext)
**Downstream consumers:** `asa-report-preview`, `hex-display`, plugin-contributed viewers

---

## Tasks

- [ ] 1. Project scaffold and error types
  - [ ] 1.1 Create `crates/ff-viewers/Cargo.toml` with dependencies (egui, thiserror, toml, parking_lot or std sync, async-trait) and dev-dependencies (proptest, pretty_assertions, tempfile)
  - [ ] 1.2 Create `crates/ff-viewers/src/lib.rs` with crate-level doc comment and public module declarations (trait_def, registry, command, built_in, plugin_bridge, panel, selection, refresh, config)
  - [ ] 1.3 Implement `src/error.rs` — define `ViewerError` enum with variants: DuplicateKey, UnknownKey, InvalidKeyFormat, ViewerReadOnlyViolation, RenderError, ConfigError, PluginViewerUnavailable
  - [ ] 1.4 Write unit tests for `ViewerError` Display output format compliance (all variants produce descriptive messages including the offending key where applicable)
    - Validates: Requirement 1 AC 6, AC 8; Requirement 8 AC 4

- [ ] 2. FileViewer trait definition
  - [ ] 2.1 Implement `src/trait_def.rs` — define `FileViewer` trait with methods: `viewer_key`, `display_name`, `description`, `supported_extensions`, `supported_mime_types`, `can_render`, `render`, `on_content_changed`, and optional `configure` with default no-op
  - [ ] 2.2 Add object-safety compile-time assertion (`fn _assert_object_safe(_: &dyn FileViewer) {}`)
  - [ ] 2.3 Verify immutability constraints: `render` takes `&self` and `&[u8]`; only `on_content_changed` and `configure` take `&mut self`
  - [ ] 2.4 Write unit tests: trait object construction compiles, default `configure` is no-op, method signatures enforce read-only render
    - Validates: Requirement 2 AC 1–5; Requirement 8 AC 1

- [ ] 3. Viewer Registry
  - [ ] 3.1 Implement `src/registry.rs` — define `ViewerRegistry` struct with `Arc<RwLock<HashMap<String, Box<dyn FileViewer>>>>` storage
  - [ ] 3.2 Implement `register()` — validate Viewer_Key format (non-empty, lowercase ASCII letters/digits/hyphens only), check uniqueness, insert viewer; return `ViewerError::DuplicateKey` on conflict
  - [ ] 3.3 Implement `deregister()` — remove viewer by key, return error if not found
  - [ ] 3.4 Implement `get()` — read-lock lookup by key, return reference or None
  - [ ] 3.5 Implement `list_viewers()` — return Vec of (key, display_name, description, supported_extensions) tuples for all registered viewers
  - [ ] 3.6 Implement `contains()` and `viewer_count()` utility accessors
  - [ ] 3.7 Write unit tests for register/deregister/lookup lifecycle, duplicate key rejection, key format validation, thread-safety (spawn multiple threads)
    - Validates: Requirement 1 AC 1–7
  - [ ] 3.8 Write property test: Viewer_Key format validation (Property 1) — generate strings, assert only valid keys (lowercase ASCII + digits + hyphens, non-empty) are accepted
    - Validates: Requirement 1 AC 1
  - [ ] 3.9 Write property test: registry uniqueness (Property 2) — register N viewers with unique keys, then attempt duplicate registration, assert DuplicateKey error
    - Validates: Requirement 1 AC 6

- [ ] 4. Built-in viewer stubs
  - [ ] 4.1 Implement `src/built_in/mod.rs` — module declarations and `register_built_in_viewers()` function that registers all built-ins into a ViewerRegistry
  - [ ] 4.2 Implement `src/built_in/asa_report.rs` — `AsaReportViewer` struct implementing `FileViewer` with key `"asa-report"`, extensions `["lst", "rpt", "spool"]`, stub `render` method (placeholder rendering)
  - [ ] 4.3 Implement `src/built_in/hex.rs` — `HexViewer` struct implementing `FileViewer` with key `"hex"`, empty extensions (activated explicitly), stub `render` method (offset + hex bytes + ASCII decode)
  - [ ] 4.4 Implement `src/built_in/image.rs` — `ImageViewer` struct implementing `FileViewer` with key `"image"`, extensions `["png", "jpg", "jpeg", "gif", "bmp", "webp"]`, stub `render` with placeholder/error display
  - [ ] 4.5 Implement `src/built_in/csv_table.rs` — `CsvTableViewer` struct implementing `FileViewer` with key `"csv-table"`, extensions `["csv", "tsv"]`, MIME `["text/csv"]`, stub `render` with grid layout
  - [ ] 4.6 Write unit tests: all built-in viewers implement FileViewer correctly, `register_built_in_viewers` populates registry with 4 entries, each viewer returns correct key/name/extensions
    - Validates: Requirement 4 AC 1–5
  - [ ] 4.7 Write property test: built-in viewer keys are stable and unique (Property 3) — assert all 4 built-in keys are distinct, non-empty, and format-compliant
    - Validates: Requirement 4 AC 5; Requirement 1 AC 1

- [ ] 5. PREVIEW command handler
  - [ ] 5.1 Implement `src/command.rs` — define `PreviewCommand` struct and register command with ID `"viewer.preview"` accepting optional `action` parameter
  - [ ] 5.2 Implement toggle logic (no argument): activate default viewer if none active, deactivate if one is showing
  - [ ] 5.3 Implement `PREVIEW ON` — activate default viewer for current resource's content type; show available viewers message if no default found
  - [ ] 5.4 Implement `PREVIEW <viewer-key>` — activate named viewer regardless of language profile default
  - [ ] 5.5 Implement `PREVIEW OFF` — deactivate active viewer, hide Viewer_Panel
  - [ ] 5.6 Implement `PREVIEW LIST` — display all registered viewers with key, display name, and description
  - [ ] 5.7 Write unit tests: command ID registration, toggle on/off, explicit key activation, OFF hides panel, LIST returns all viewers, invalid key returns warning
    - Validates: Requirement 3 AC 1–9
  - [ ] 5.8 Write property test: PREVIEW command never produces an Undo_Record (Property 4) — issue various PREVIEW commands, assert no undo state is generated
    - Validates: Requirement 3 AC 9

- [ ] 6. Plugin viewer bridge
  - [ ] 6.1 Implement `src/plugin_bridge.rs` — define `register_viewer` function callable from PluginContext, delegating to ViewerRegistry with validation
  - [ ] 6.2 Implement `deregister_viewer` function — remove plugin viewer by key, close any active Viewer_Panel using that viewer
  - [ ] 6.3 Implement plugin shutdown hook — auto-deregister all viewers contributed by the shutting-down plugin, close affected panels gracefully
  - [ ] 6.4 Write unit tests: plugin registration succeeds, duplicate key from plugin rejected, deregistration closes panel, shutdown auto-deregisters all plugin viewers
    - Validates: Requirement 5 AC 1–6
  - [ ] 6.5 Write property test: plugin viewer lifecycle (Property 5) — register then deregister plugin viewers in random order, assert registry consistency (no dangling keys, count correct)
    - Validates: Requirement 5 AC 2, AC 3

- [ ] 7. Viewer selection and content matching
  - [ ] 7.1 Implement `src/selection.rs` — define `select_viewer_by_extension()` that matches resource extension against all registered viewers' `supported_extensions`
  - [ ] 7.2 Implement `select_viewer_by_language_profile()` — check active language profile `default_viewer` key, return matching viewer if it exists in registry
  - [ ] 7.3 Implement `select_viewer_by_content_sniff()` — invoke `can_render` on all registered viewers with URI + content sample, return first match
  - [ ] 7.4 Implement selection priority: language profile > extension match > content sniff > none
  - [ ] 7.5 Implement notification suppression tracking — record dismissed viewer offers per resource per session
  - [ ] 7.6 Write unit tests: extension match works, language profile overrides extension, content sniff fallback, no match returns None, dismissed notification not re-shown
    - Validates: Requirement 6 AC 1–6
  - [ ] 7.7 Write property test: selection priority ordering (Property 6) — when language profile defines a default, it always wins over extension match; when no profile, extension wins over content sniff
    - Validates: Requirement 6 AC 1, AC 2

- [ ] 8. Viewer Panel (DockablePanel implementation)
  - [ ] 8.1 Implement `src/panel.rs` — define `ViewerPanel` struct implementing `DockablePanel` trait with panel_id `"viewer"`, default dock zone Center
  - [ ] 8.2 Implement `panel_title()` — return dynamic title including active Viewer_Key (e.g., `"Preview: asa-report"`)
  - [ ] 8.3 Implement panel visibility lifecycle — show on PREVIEW activation, hide on PREVIEW OFF while preserving dock position
  - [ ] 8.4 Implement `render_content()` — delegate to active FileViewer's `render` method, passing content as `&[u8]` and egui Ui reference
  - [ ] 8.5 Implement read-only input filtering — reject keyboard/mouse input that would modify document, allow clipboard copy
  - [ ] 8.6 Write unit tests: panel_id is "viewer", default zone is Center, title includes viewer key, visibility toggle preserves position, no editing affordances exposed
    - Validates: Requirement 7 AC 1–7; Requirement 8 AC 2, AC 3
  - [ ] 8.7 Write property test: Viewer_Panel never exposes mutable content (Property 7) — render with various content inputs, assert no mutation path exists on the byte slice
    - Validates: Requirement 8 AC 1–3

- [ ] 9. Viewer refresh and debounce logic
  - [ ] 9.1 Implement `src/refresh.rs` — define `RefreshController` struct with configurable debounce interval (default 300ms)
  - [ ] 9.2 Implement debounce logic — on document change event, reset timer; after quiet period elapses, invoke active viewer's `on_content_changed`
  - [ ] 9.3 Implement external change detection — on VFS file-watcher event, reload content and invoke `on_content_changed`
  - [ ] 9.4 Implement error resilience — catch panics/errors from `on_content_changed`, log warning, display stale-content indicator in panel
  - [ ] 9.5 Implement background refresh — ensure `on_content_changed` runs off the UI thread, never blocking editor input
  - [ ] 9.6 Write unit tests: debounce groups rapid changes, single refresh after quiet period, external change triggers refresh, error in viewer shows stale indicator, refresh does not block UI thread
    - Validates: Requirement 9 AC 1–6
  - [ ] 9.7 Write property test: debounce coalesces rapid edits (Property 8) — generate sequences of N edits within debounce window, assert only 1 refresh call occurs per quiet period
    - Validates: Requirement 9 AC 2, AC 3

- [ ] 10. Viewer configuration
  - [ ] 10.1 Implement `src/config.rs` — define `ViewerConfig` struct with fields: `auto_offer` (bool, default true), `default_position` (enum, default "split-right"), `split_ratio` (f32, 0.1–0.9, default 0.5), `refresh_debounce_ms` (u32, default 300)
  - [ ] 10.2 Implement TOML parsing for `[viewers]` section — validate values, emit warning and apply defaults for invalid entries
  - [ ] 10.3 Implement hot-reload support — detect config file changes, apply new values to next viewer activation without restart
  - [ ] 10.4 Implement per-viewer config sub-sections — parse `[viewers.<viewer-key>]` and pass `toml::Value` to viewer's `configure()` method
  - [ ] 10.5 Write unit tests: default config values, valid TOML parsing, invalid values fall back to defaults with warning, hot-reload picks up changes, per-viewer config passed to viewer
    - Validates: Requirement 10 AC 1–4
  - [ ] 10.6 Write property test: configuration validation bounds (Property 9) — generate split_ratio values, assert only 0.1–0.9 accepted; generate debounce_ms values, assert only positive integers accepted
    - Validates: Requirement 10 AC 1, AC 2

- [ ] 11. Read-only enforcement integration
  - [ ] 11.1 Implement Command_Dispatch guard — when Viewer_Mode is active, intercept document-mutating commands and reject with `ViewerReadOnlyViolation` error
  - [ ] 11.2 Implement performance warning — log warning if `on_content_changed` exceeds 100ms execution time
  - [ ] 11.3 Write unit tests: mutating command rejected during Viewer_Mode, ViewerReadOnlyViolation error raised, 100ms warning logged for slow viewers
    - Validates: Requirement 8 AC 4, AC 5
  - [ ] 11.4 Write property test: read-only invariant under Viewer_Mode (Property 10) — generate random command sequences during active viewer, assert no document mutation occurs
    - Validates: Requirement 8 AC 3, AC 4

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Viewer Registry | AC 1 (Viewer_Key → Box\<dyn FileViewer\>) | 3.1–3.2, 3.8 |
| Req 1: Viewer Registry | AC 2 (thread-safe) | 3.1, 3.7 |
| Req 1: Viewer Registry | AC 3 (built-ins before plugins) | 4.1, 4.6 |
| Req 1: Viewer Registry | AC 4 (runtime plugin registration) | 6.1 |
| Req 1: Viewer Registry | AC 5 (deregistration on shutdown) | 6.2, 6.3 |
| Req 1: Viewer Registry | AC 6 (duplicate key rejection) | 3.2, 3.7, 3.9 |
| Req 1: Viewer Registry | AC 7 (runtime discovery) | 3.5, 5.6 |
| Req 1: Viewer Registry | AC 8 (unknown key warning + fallback) | 1.3, 5.7 |
| Req 2: FileViewer Trait | AC 1 (trait methods) | 2.1 |
| Req 2: FileViewer Trait | AC 2 (object-safe) | 2.2 |
| Req 2: FileViewer Trait | AC 3 (non-mutating except on_content_changed) | 2.3, 2.4 |
| Req 2: FileViewer Trait | AC 4 (read-only render) | 2.3, 8.5 |
| Req 2: FileViewer Trait | AC 5 (no panel lifecycle management) | 2.1, 8.1 |
| Req 3: PREVIEW Command | AC 1 (command registration) | 5.1 |
| Req 3: PREVIEW Command | AC 2 (toggle no-arg) | 5.2, 5.7 |
| Req 3: PREVIEW Command | AC 3 (PREVIEW ON default) | 5.3, 5.7 |
| Req 3: PREVIEW Command | AC 4 (PREVIEW \<key\>) | 5.4, 5.7 |
| Req 3: PREVIEW Command | AC 5 (PREVIEW OFF) | 5.5, 5.7 |
| Req 3: PREVIEW Command | AC 6 (PREVIEW LIST) | 5.6, 5.7 |
| Req 3: PREVIEW Command | AC 7 (status bar display) | 5.1 |
| Req 3: PREVIEW Command | AC 8 (browse/edit mode agnostic) | 5.7 |
| Req 3: PREVIEW Command | AC 9 (no Undo_Record) | 5.8 |
| Req 4: Built-In Viewers | AC 1 (asa-report) | 4.2, 4.6 |
| Req 4: Built-In Viewers | AC 2 (hex) | 4.3, 4.6 |
| Req 4: Built-In Viewers | AC 3 (image) | 4.4, 4.6 |
| Req 4: Built-In Viewers | AC 4 (csv-table) | 4.5, 4.6 |
| Req 4: Built-In Viewers | AC 5 (registered before plugins) | 4.1, 4.6, 4.7 |
| Req 5: Plugin-Provided Viewers | AC 1 (register_viewer on PluginContext) | 6.1, 6.4 |
| Req 5: Plugin-Provided Viewers | AC 2 (key validation) | 6.1, 6.4, 6.5 |
| Req 5: Plugin-Provided Viewers | AC 3 (auto-deregister on shutdown) | 6.3, 6.4, 6.5 |
| Req 5: Plugin-Provided Viewers | AC 4 (close panel on plugin shutdown) | 6.2, 6.3, 6.4 |
| Req 5: Plugin-Provided Viewers | AC 5 (deregister_viewer method) | 6.2, 6.4 |
| Req 5: Plugin-Provided Viewers | AC 6 (same capabilities as built-in) | 6.1, 6.4 |
| Req 6: Viewer Selection | AC 1 (extension match) | 7.1, 7.6, 7.7 |
| Req 6: Viewer Selection | AC 2 (language profile precedence) | 7.2, 7.4, 7.6, 7.7 |
| Req 6: Viewer Selection | AC 3 (status bar notification) | 7.1, 7.6 |
| Req 6: Viewer Selection | AC 4 (can_render fallback) | 7.3, 7.6 |
| Req 6: Viewer Selection | AC 5 (manual override) | 5.4, 7.6 |
| Req 6: Viewer Selection | AC 6 (dismiss suppression) | 7.5, 7.6 |
| Req 7: Viewer Panel (DockablePanel) | AC 1 (DockablePanel impl) | 8.1, 8.6 |
| Req 7: Viewer Panel (DockablePanel) | AC 2 (Panel_Registry) | 8.1 |
| Req 7: Viewer Panel (DockablePanel) | AC 3 (visible on PREVIEW) | 8.3, 8.6 |
| Req 7: Viewer Panel (DockablePanel) | AC 4 (hidden on OFF, position preserved) | 8.3, 8.6 |
| Req 7: Viewer Panel (DockablePanel) | AC 5 (tab group / split view) | 8.1, 8.6 |
| Req 7: Viewer Panel (DockablePanel) | AC 6 (floating) | 8.1, 8.6 |
| Req 7: Viewer Panel (DockablePanel) | AC 7 (persona serialization) | 8.1, 8.6 |
| Req 8: Read-Only Constraint | AC 1 (immutable byte slice) | 2.3, 8.4, 8.7 |
| Req 8: Read-Only Constraint | AC 2 (no editing affordances) | 8.5, 8.6 |
| Req 8: Read-Only Constraint | AC 3 (no Undo_Records from viewer input) | 8.5, 8.6, 11.4 |
| Req 8: Read-Only Constraint | AC 4 (ViewerReadOnlyViolation on mutating command) | 1.3, 11.1, 11.3, 11.4 |
| Req 8: Read-Only Constraint | AC 5 (100ms warning for slow on_content_changed) | 11.2, 11.3 |
| Req 9: Viewer Refresh | AC 1 (notify on document change) | 9.1, 9.2, 9.6 |
| Req 9: Viewer Refresh | AC 2 (debounce) | 9.2, 9.6, 9.7 |
| Req 9: Viewer Refresh | AC 3 (configurable debounce_ms) | 9.1, 10.1, 9.7 |
| Req 9: Viewer Refresh | AC 4 (external change via VFS watcher) | 9.3, 9.6 |
| Req 9: Viewer Refresh | AC 5 (error resilience / stale indicator) | 9.4, 9.6 |
| Req 9: Viewer Refresh | AC 6 (background thread, no UI block) | 9.5, 9.6 |
| Req 10: Viewer Configuration | AC 1 (TOML [viewers] keys) | 10.1, 10.2, 10.5, 10.6 |
| Req 10: Viewer Configuration | AC 2 (invalid value fallback) | 10.2, 10.5, 10.6 |
| Req 10: Viewer Configuration | AC 3 (hot-reload) | 10.3, 10.5 |
| Req 10: Viewer Configuration | AC 4 (per-viewer sub-sections) | 10.4, 10.5 |

---

## Property-Based Test Summary

| Property | Statement | Task | Validates |
|----------|-----------|------|-----------|
| P1 | Viewer_Key format: only non-empty lowercase ASCII + digits + hyphens accepted | 3.8 | Req 1 AC 1 |
| P2 | Registry uniqueness: duplicate key always rejected with DuplicateKey error | 3.9 | Req 1 AC 6 |
| P3 | Built-in viewer keys: all 4 are distinct, non-empty, and format-compliant | 4.7 | Req 4 AC 5; Req 1 AC 1 |
| P4 | PREVIEW command never produces an Undo_Record | 5.8 | Req 3 AC 9 |
| P5 | Plugin viewer lifecycle: register/deregister in random order maintains registry consistency | 6.5 | Req 5 AC 2, AC 3 |
| P6 | Selection priority: language profile > extension > content sniff | 7.7 | Req 6 AC 1, AC 2 |
| P7 | Viewer_Panel never exposes mutable content reference | 8.7 | Req 8 AC 1–3 |
| P8 | Debounce coalesces rapid edits: N edits within window → 1 refresh call | 9.7 | Req 9 AC 2, AC 3 |
| P9 | Configuration validation: split_ratio 0.1–0.9 only, debounce_ms positive only | 10.6 | Req 10 AC 1, AC 2 |
| P10 | Read-only invariant: no document mutation during Viewer_Mode regardless of input | 11.4 | Req 8 AC 3, AC 4 |

---

## Notes

- Tasks 2 and 3 can be implemented in parallel (trait definition and registry are independent after scaffold)
- Task 4 (built-in viewers) depends on both the trait (task 2) and registry (task 3)
- Task 5 (PREVIEW command) depends on the registry (task 3) and panel (task 8) — panel can be a stub initially
- Tasks 7, 8, 9, and 10 depend on the trait and registry being complete
- Task 11 (read-only enforcement) is an integration-level concern that spans command dispatch and the panel
- All property tests use the `proptest` crate with a minimum of 100 iterations
- Built-in viewers are stubs — full rendering logic lives in separate specs (`asa-report-preview`, `hex-display`)
- The `egui::Ui` parameter in `render` is used for layout; actual rendering tests may use `egui::__run_test_ui` or mock Ui contexts
- Thread-safety tests should spawn multiple threads accessing the registry concurrently

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Project scaffold and error types", "tasks": ["1.1", "1.2", "1.3", "1.4"] },
    { "id": 1, "label": "FileViewer trait and Viewer Registry", "tasks": ["2.1", "2.2", "2.3", "2.4", "3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7", "3.8", "3.9"], "dependsOn": [0] },
    { "id": 2, "label": "Built-in viewer stubs and configuration", "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7", "10.1", "10.2", "10.3", "10.4", "10.5", "10.6"], "dependsOn": [1] },
    { "id": 3, "label": "PREVIEW command and viewer selection", "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8", "7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7"], "dependsOn": [2] },
    { "id": 4, "label": "Viewer Panel and plugin bridge", "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "6.1", "6.2", "6.3", "6.4", "6.5"], "dependsOn": [3] },
    { "id": 5, "label": "Refresh/debounce and read-only enforcement", "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "11.1", "11.2", "11.3", "11.4"], "dependsOn": [4] }
  ]
}
```
