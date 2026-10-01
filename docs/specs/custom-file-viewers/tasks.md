# Implementation Plan: Custom File Viewers (`ff-viewers`)

## Overview

This task plan implements the `ff-viewers` crate -- the extensible file viewer framework for FileForgeWorkbench. The crate provides a `FileViewer` trait, a thread-safe Viewer_Registry, the `PREVIEW` command family, built-in viewer stubs (asa-report, hex, image, csv-table), plugin viewer bridge integration, a DockablePanel-based Viewer_Panel, content matching and selection logic, refresh/debounce handling, and viewer configuration.

**Crate location:** `crates/ff-viewers`
**Upstream dependencies:** `ff-core` (subsystem integration), `ff-layout` (DockablePanel), `ff-command` (Command_Registry), `ff-vfs` (content reads), `ff-plugin` (PluginContext)
**Downstream consumers:** `asa-report-preview`, `hex-display`, plugin-contributed viewers

---

## Tasks

- [x] 1. Project scaffold and error types
  - [x] 1.1 Create `crates/ff-viewers/Cargo.toml` with dependencies (egui, thiserror, toml, parking_lot or std sync, async-trait) and dev-dependencies (proptest, pretty_assertions, tempfile)
  - [x] 1.2 Create `crates/ff-viewers/src/lib.rs` with crate-level doc comment and public module declarations (trait_def, registry, command, built_in, plugin_bridge, panel, selection, refresh, config)
  - [x] 1.3 Implement `src/error.rs` -- define `ViewerError` enum with variants: DuplicateKey, UnknownKey, InvalidKeyFormat, ViewerReadOnlyViolation, RenderError, ConfigError, PluginViewerUnavailable
  - [x] 1.4 Write unit tests for `ViewerError` Display output format compliance (all variants produce descriptive messages including the offending key where applicable)
    - Validates: Requirement 1 AC 6, AC 8; Requirement 8 AC 4

- [x] 2. FileViewer trait definition
  - [x] 2.1 Implement `src/trait_def.rs` -- define `FileViewer` trait with methods: `viewer_key`, `display_name`, `description`, `supported_extensions`, `supported_mime_types`, `can_render`, `render`, `on_content_changed`, and optional `configure` with default no-op
  - [x] 2.2 Add object-safety compile-time assertion (`fn _assert_object_safe(_: &dyn FileViewer) {}`)
  - [x] 2.3 Verify immutability constraints: `render` takes `&self` and `&[u8]`; only `on_content_changed` and `configure` take `&mut self`
  - [x] 2.4 Write unit tests: trait object construction compiles, default `configure` is no-op, method signatures enforce read-only render
    - Validates: Requirement 2 AC 1-5; Requirement 8 AC 1

- [x] 3. Viewer Registry
  - [x] 3.1 Implement `src/registry.rs` -- define `ViewerRegistry` struct with `Arc<RwLock<HashMap<String, Box<dyn FileViewer>>>>` storage
  - [x] 3.2 Implement `register()` -- validate Viewer_Key format (non-empty, lowercase ASCII letters/digits/hyphens only), check uniqueness, insert viewer; return `ViewerError::DuplicateKey` on conflict
  - [x] 3.3 Implement `deregister()` -- remove viewer by key, return error if not found
  - [x] 3.4 Implement `get()` -- read-lock lookup by key, return reference or None
  - [x] 3.5 Implement `list_viewers()` -- return Vec of (key, display_name, description, supported_extensions) tuples for all registered viewers
  - [x] 3.6 Implement `contains()` and `viewer_count()` utility accessors
  - [x] 3.7 Write unit tests for register/deregister/lookup lifecycle, duplicate key rejection, key format validation, thread-safety (spawn multiple threads)
    - Validates: Requirement 1 AC 1-7
  - [x] 3.8 Write property test: Viewer_Key format validation (Property 1) -- generate strings, assert only valid keys (lowercase ASCII + digits + hyphens, non-empty) are accepted
    - Validates: Requirement 1 AC 1
  - [x] 3.9 Write property test: registry uniqueness (Property 2) -- register N viewers with unique keys, then attempt duplicate registration, assert DuplicateKey error
    - Validates: Requirement 1 AC 6

- [x] 4. Built-in viewer stubs
  - [x] 4.1 Implement `src/built_in/mod.rs` -- module declarations and `register_built_in_viewers()` function that registers all built-ins into a ViewerRegistry
  - [x] 4.2 Implement `src/built_in/asa_report.rs` -- `AsaReportViewer` struct implementing `FileViewer` with key `"asa-report"`, extensions `["lst", "rpt", "spool"]`, stub `render` method (placeholder rendering)
  - [x] 4.3 Implement `src/built_in/hex.rs` -- `HexViewer` struct implementing `FileViewer` with key `"hex"`, empty extensions (activated explicitly), stub `render` method (offset + hex bytes + ASCII decode)
  - [x] 4.4 Implement `src/built_in/image.rs` -- `ImageViewer` struct implementing `FileViewer` with key `"image"`, extensions `["png", "jpg", "jpeg", "gif", "bmp", "webp"]`, stub `render` with placeholder/error display
  - [x] 4.5 Implement `src/built_in/csv_table.rs` -- `CsvTableViewer` struct implementing `FileViewer` with key `"csv-table"`, extensions `["csv", "tsv"]`, MIME `["text/csv"]`, stub `render` with grid layout
  - [x] 4.6 Write unit tests: all built-in viewers implement FileViewer correctly, `register_built_in_viewers` populates registry with 4 entries, each viewer returns correct key/name/extensions
    - Validates: Requirement 4 AC 1-5
  - [x] 4.7 Write property test: built-in viewer keys are stable and unique (Property 3) -- assert all 4 built-in keys are distinct, non-empty, and format-compliant
    - Validates: Requirement 4 AC 5; Requirement 1 AC 1

- [x] 5. PREVIEW command handler
  - [x] 5.1 Implement `src/command.rs` -- define `PreviewCommand` struct and register command with ID `"viewer.preview"` accepting optional `action` parameter
  - [x] 5.2 Implement toggle logic (no argument): activate default viewer if none active, deactivate if one is showing
  - [x] 5.3 Implement `PREVIEW ON` -- activate default viewer for current resource's content type; show available viewers message if no default found
  - [x] 5.4 Implement `PREVIEW <viewer-key>` -- activate named viewer regardless of language profile default
  - [x] 5.5 Implement `PREVIEW OFF` -- deactivate active viewer, hide Viewer_Panel
  - [x] 5.6 Implement `PREVIEW LIST` -- display all registered viewers with key, display name, and description
  - [x] 5.7 Write unit tests: command ID registration, toggle on/off, explicit key activation, OFF hides panel, LIST returns all viewers, invalid key returns warning
    - Validates: Requirement 3 AC 1-9
  - [x] 5.8 Write property test: PREVIEW command never produces an Undo_Record (Property 4) -- issue various PREVIEW commands, assert no undo state is generated
    - Validates: Requirement 3 AC 9

- [x] 6. Plugin viewer bridge
  - [x] 6.1 Implement `src/plugin_bridge.rs` -- define `register_viewer` function callable from PluginContext, delegating to ViewerRegistry with validation
  - [x] 6.2 Implement `deregister_viewer` function -- remove plugin viewer by key, close any active Viewer_Panel using that viewer
  - [x] 6.3 Implement plugin shutdown hook -- auto-deregister all viewers contributed by the shutting-down plugin, close affected panels gracefully
  - [x] 6.4 Write unit tests: plugin registration succeeds, duplicate key from plugin rejected, deregistration closes panel, shutdown auto-deregisters all plugin viewers
    - Validates: Requirement 5 AC 1-6
  - [x] 6.5 Write property test: plugin viewer lifecycle (Property 5) -- register then deregister plugin viewers in random order, assert registry consistency (no dangling keys, count correct)
    - Validates: Requirement 5 AC 2, AC 3

- [x] 7. Viewer selection and content matching
  - [x] 7.1 Implement `src/selection.rs` -- define `select_viewer_by_extension()` that matches resource extension against all registered viewers' `supported_extensions`
  - [x] 7.2 Implement `select_viewer_by_language_profile()` -- check active language profile `default_viewer` key, return matching viewer if it exists in registry
  - [x] 7.3 Implement `select_viewer_by_content_sniff()` -- invoke `can_render` on all registered viewers with URI + content sample, return first match
  - [x] 7.4 Implement selection priority: language profile > extension match > content sniff > none
  - [x] 7.5 Implement notification suppression tracking -- record dismissed viewer offers per resource per session
  - [x] 7.6 Write unit tests: extension match works, language profile overrides extension, content sniff fallback, no match returns None, dismissed notification not re-shown
    - Validates: Requirement 6 AC 1-6
  - [x] 7.7 Write property test: selection priority ordering (Property 6) -- when language profile defines a default, it always wins over extension match; when no profile, extension wins over content sniff
    - Validates: Requirement 6 AC 1, AC 2

- [x] 8. Viewer Panel (DockablePanel implementation)
  - [x] 8.1 Implement `src/panel.rs` -- define `ViewerPanel` struct implementing `DockablePanel` trait with panel_id `"viewer"`, default dock zone Center
  - [x] 8.2 Implement `panel_title()` -- return dynamic title including active Viewer_Key (e.g., `"Preview: asa-report"`)
  - [x] 8.3 Implement panel visibility lifecycle -- show on PREVIEW activation, hide on PREVIEW OFF while preserving dock position
  - [x] 8.4 Implement `render_content()` -- delegate to active FileViewer's `render` method, passing content as `&[u8]` and egui Ui reference
  - [x] 8.5 Implement read-only input filtering -- reject keyboard/mouse input that would modify document, allow clipboard copy
  - [x] 8.6 Write unit tests: panel_id is "viewer", default zone is Center, title includes viewer key, visibility toggle preserves position, no editing affordances exposed
    - Validates: Requirement 7 AC 1-7; Requirement 8 AC 2, AC 3
  - [x] 8.7 Write property test: Viewer_Panel never exposes mutable content (Property 7) -- render with various content inputs, assert no mutation path exists on the byte slice
    - Validates: Requirement 8 AC 1-3

- [x] 9. Viewer refresh and debounce logic
  - [x] 9.1 Implement `src/refresh.rs` -- define `RefreshController` struct with configurable debounce interval (default 300ms)
  - [x] 9.2 Implement debounce logic -- on document change event, reset timer; after quiet period elapses, invoke active viewer's `on_content_changed`
  - [x] 9.3 Implement external change detection -- on VFS file-watcher event, reload content and invoke `on_content_changed`
  - [x] 9.4 Implement error resilience -- catch panics/errors from `on_content_changed`, log warning, display stale-content indicator in panel
  - [x] 9.5 Implement background refresh -- ensure `on_content_changed` runs off the UI thread, never blocking editor input
  - [x] 9.6 Write unit tests: debounce groups rapid changes, single refresh after quiet period, external change triggers refresh, error in viewer shows stale indicator, refresh does not block UI thread
    - Validates: Requirement 9 AC 1-6
  - [x] 9.7 Write property test: debounce coalesces rapid edits (Property 8) -- generate sequences of N edits within debounce window, assert only 1 refresh call occurs per quiet period
    - Validates: Requirement 9 AC 2, AC 3

- [x] 10. Viewer configuration
  - [x] 10.1 Implement `src/config.rs` -- define `ViewerConfig` struct with fields: `auto_offer` (bool, default true), `default_position` (enum, default "split-right"), `split_ratio` (f32, 0.1-0.9, default 0.5), `refresh_debounce_ms` (u32, default 300)
  - [x] 10.2 Implement TOML parsing for `[viewers]` section -- validate values, emit warning and apply defaults for invalid entries
  - [x] 10.3 Implement hot-reload support -- detect config file changes, apply new values to next viewer activation without restart
  - [x] 10.4 Implement per-viewer config sub-sections -- parse `[viewers.<viewer-key>]` and pass `toml::Value` to viewer's `configure()` method
  - [x] 10.5 Write unit tests: default config values, valid TOML parsing, invalid values fall back to defaults with warning, hot-reload picks up changes, per-viewer config passed to viewer
    - Validates: Requirement 10 AC 1-4
  - [x] 10.6 Write property test: configuration validation bounds (Property 9) -- generate split_ratio values, assert only 0.1-0.9 accepted; generate debounce_ms values, assert only positive integers accepted
    - Validates: Requirement 10 AC 1, AC 2

- [x] 11. Read-only enforcement integration
  - [x] 11.1 Implement Command_Dispatch guard -- when Viewer_Mode is active, intercept document-mutating commands and reject with `ViewerReadOnlyViolation` error
  - [x] 11.2 Implement performance warning -- log warning if `on_content_changed` exceeds 100ms execution time
  - [x] 11.3 Write unit tests: mutating command rejected during Viewer_Mode, ViewerReadOnlyViolation error raised, 100ms warning logged for slow viewers
    - Validates: Requirement 8 AC 4, AC 5
  - [x] 11.4 Write property test: read-only invariant under Viewer_Mode (Property 10) -- generate random command sequences during active viewer, assert no document mutation occurs
    - Validates: Requirement 8 AC 3, AC 4

---

## Markdown Viewer Family tasks (CR-CH-049)

These tasks bring the Markdown Viewer crates (`ff-md-viewer`, `ff-mdx-plugin`,
`ff-mdx-app`, `ff-mdx-installer`) into compliance, back-filling the gate for
Requirements 11-17. They are independently completable and reference the
criteria they satisfy. TDD applies: write the failing test first (red), then the
minimum implementation (green). None are pre-checked.

- [x] 12. ASCII cleanup across the markdown crates (done in Part A, pending gate)
  - [x] 12.1 Replace em dash / en dash / ellipsis / emoji in `.rs` and `Cargo.toml` of `ff-mdx-app`, `ff-mdx-plugin`, `ff-mdx-installer` per documentation.md
    - DONE IN PART A of this run (pre-gate mechanical fix): em dash -> `--`, ellipsis -> `...`, emoji removed/replaced across app.rs, file_tree.rs, viewer.rs, plugin viewer.rs, installer main.rs, and ff-mdx-app Cargo.toml. Non-ASCII scan returned zero matches.
    - Validates: documentation.md (Rust source ASCII-only); audit F05, F06, F07, F14

- [x] 13. Unit tests for ff-md-viewer render_to_html
  - [x] 13.1 Unit tests assert the enabled extension output: table -> `<table>`, footnote reference -> footnote anchor, strikethrough -> `<del>`, task-list item -> checkbox list item, straight quotes -> smart-punctuation; empty input -> empty fragment
    - Validates: Requirement 11.1, 11.2, 11.3, 11.4

- [x] 14. Unit tests for ff-md-viewer Scanner::scan
  - [x] 14.1 Unit tests over `tempfile::TempDir` fixtures: only `.md` included; excluded directories not descended; `relative_path` normalised backslash->forward-slash + absolute `full_path`; results sorted lexicographically; missing/non-directory root returns an empty vector
    - Validates: Requirement 12.1, 12.2, 12.3, 12.4, 12.5, 12.6

- [x] 15. Unit tests for ff-md-viewer FileWatcher predicate and debounce
  - [x] 15.1 Extracted the `md`-extension predicate (`is_markdown_path`) into a pure testable function; test rejects non-`md` paths
    - Validates: Requirement 15.2
  - [x] 15.2 Extracted the burst coalescing (`coalesce_burst`) and tested deterministically: a burst yields the single most-recent path; empty burst yields None
    - Validates: Requirement 15.1, 15.3

- [x] 16. Unit tests for ff-mdx-plugin MdxFileViewer
  - [x] 16.1 Unit tests: `viewer_key == "mdx-markdown"`; declared extensions/MIME; `can_render` true for `.md`/`.markdown` and false otherwise; `render` on non-UTF-8 bytes does not panic and returns HTML; `MdxPlugin` capability metadata matches the viewer's MIME types/display name
    - Validates: Requirement 13.1, 13.2, 13.3, 13.4, 13.5

- [x] 17. Unit tests for ff-mdx-installer PATH de-duplication
  - [x] 17.1 Factored the PATH-dedup predicate (`path_already_contains`) and `append_to_path` into pure functions; tests assert existing case-insensitive/trimmed match leaves PATH unchanged; new entry appended; `;` join only when PATH is non-empty
    - Validates: Requirement 17.1, 17.2, 17.3, 17.4

- [x] 18. Extract and unit-test ff-mdx-app pure helpers
  - [x] 18.1 Extracted the filter predicate (`filter_matches`), drop-path classification (`classify_drop`), viewer-title logic (`relative_title`), and reload decision (`should_reload`) into pure functions in `helpers.rs`; unit-tested each
    - Validates: Requirement 16.2, 16.6

- [x] 19. egui_kittest GUI tests for ff-mdx-app
  - [x] 19.1 Harness test: clicking a file-tree entry marks it selected and returns its `full_path`
    - Validates: Requirement 16.1
  - [x] 19.2 Harness test: the side-panel filter narrows the file list case-insensitively; empty filter shows all
    - Validates: Requirement 16.2
  - [x] 19.3 Harness test: empty-state placeholder when no file selected; rendered Markdown with relative-path title when selected
    - Validates: Requirement 16.3
  - [x] 19.4 Harness test (build_eframe): F5 rescans the current folder. Ctrl+O opens the rfd native folder picker and is recorded MANUAL (OS-native dialog)
    - Validates: Requirement 16.4
  - [x] 19.5 Watcher reload decision tested via the pure `should_reload` helper: change to the open file reloads; change to another file does not
    - Validates: Requirement 16.5, 15.4
  - [x] 19.6 Recorded the `rfd` folder/save dialogs as MANUAL in TCR with the OS-native-dialog reason
    - Validates: Requirement 16.6

- [x] 20. rust-standards hardening of ff-md-viewer and the markdown crates
  - [x] 20.1 Introduced the `thiserror` `MdViewerError` enum in `ff-md-viewer` (`error.rs`); `FileWatcher::new` returns it; removed the `anyhow` dependency from the library
    - Validates: rust-standards.md (libraries use thiserror); audit F08
  - [x] 20.2 Made `FileWatcher.rx` private with a `changes(&self) -> &Receiver<PathBuf>` accessor; added `///` docs to the public items (`FileEntry`, `Scanner`, `FileWatcher`, `MdViewerError`, `MdxApp`, `MdxPlugin`, `MdxFileViewer`, installer types)
    - Validates: rust-standards.md (accessors over pub fields; doc every public item); audit F09, F10
  - [x] 20.3 Split `MdxApp::toolbar`/`update` into focused helpers (`open_folder_dialog`, `refresh_current_folder`, `export_current_as_html`, `handle_shortcuts`); used `env!("CARGO_PKG_VERSION")` for the version label; resolved `FileTree::set_files` to reconcile selection against the new file set (documented)
    - Validates: rust-standards.md (function length); audit F11, F12, F13
  - [x] 20.4 Added `ff-logging` log calls at the `MdxPlugin` lifecycle seams (initialize/activate/deactivate/shutdown), keeping the dependency justified
    - Validates: audit F15

- [ ] 21. Shell integration of the markdown viewer on the framework
  - BLOCKED (surfaced to owner): `ff-desktop` does NOT currently depend on `ff-viewers` or `ff-mdx-plugin`, and there is no viewer-to-shell seam in the shell (no ViewerRegistry construction, no PREVIEW command wired into `resolve_target`/`dispatch_command_target`). The `ff-viewers` framework exists and is tested only in isolation. Wiring it in is a framework-integration change touching the ff-desktop command-dispatch seam -- per the owner's instruction, stopped and surfaced rather than improvised. Req 14 TCR rows remain NOT COVERED.
  - [ ] 21.1 Register `MdxPlugin`/`MdxFileViewer` with the `ff-viewers` Viewer_Registry so it appears in `PREVIEW LIST` and is activatable by Viewer_Key
    - Validates: Requirement 14.1
  - [ ] 21.2 Ensure the viewer is reached only through the single command-dispatch path (the existing `PREVIEW` command); no bespoke intercept or parallel dispatcher
    - Validates: Requirement 14.2
  - [ ] 21.3 Ensure any menu/toolbar/shortcut affordance invokes the same `PREVIEW` command (command parity)
    - Validates: Requirement 14.3
  - [ ] 21.4 IF the viewer is surfaced as its own shell Context, implement `WorkspaceContext::render -> InteriorFocus` (stable first-control `egui::Id`) dispatched via `render_workspace_context`, and add the mandatory full-shell first-Tab `egui_kittest` test
    - Validates: Requirement 14 (framework-conformance mechanism 5; workspace-conformance)

---

## Markdown Viewer Link Navigation tasks (CR-CH-050)

These tasks implement Requirement 18 (external/web links, `.md`-file links,
same-file and cross-file HTML-style anchors). The pure helpers and the STANDALONE
`ff-mdx-app` behaviour are NOT blocked by Task 21; the IN-SHELL link activation
IS `[Depends on Req 14 / Task 21]`. TDD applies: failing test first. All queued
behind the ff-desktop decoupling per the owner's finish-decoupling-first priority.
`[ ]` only.

- [ ] 22. Pure link classification and anchor resolution in ff-md-viewer (not Task-21-blocked)
  - [ ] 22.1 Implement `classify_link(dest) -> MdLinkTarget` (External / MarkdownFile{path,fragment} / SameFileAnchor / Other); unit-tested first (red)
    - Validates: Requirement 18.1, 18.2, 18.6, 18.7
  - [ ] 22.2 Implement `resolve_markdown_link(current_doc_dir, target, root)` (relative-to-current-doc join, absolute as-is, reject root-escape when a root is given); unit-tested
    - Validates: Requirement 18.2, 18.3, 18.7
  - [ ] 22.3 Implement `heading_slug(heading)` (GitHub-style slug) and `resolve_anchor(doc_markdown, anchor)` (explicit anchor id if present else heading slug else None->top); unit-tested. Record the optional non-standard explicit-anchor-marker extension as design-gated, default = heading slug
    - Validates: Requirement 18.8, 18.9
- [ ] 23. Standalone ff-mdx-app link navigation + GUI tests (out-of-shell, not Task-21-blocked)
  - [ ] 23.1 On a `MarkdownFile` (no fragment) activation: resolve path (22.2), load file, update viewer to top; optionally update tree selection. On missing/unreadable target: status message, keep current doc. Unit-test the decision helper
    - Validates: Requirement 18.2, 18.3, 18.4
  - [ ] 23.2 On a `SameFileAnchor`: scroll to `resolve_anchor` within the current doc. On a cross-file `MarkdownFile{fragment}`: load target then position at `resolve_anchor`; missing anchor -> top. Unit-test the decision helpers
    - Validates: Requirement 18.6, 18.7, 18.9
  - [ ] 23.3 On an `External` link: open the OS default browser (classification unit-tested; the browser launch itself is MANUAL -- OS side effect, justified exception). `Other` targets ignored
    - Validates: Requirement 18.1
  - [ ] 23.4 egui_kittest tests: clicking a `.md` link loads the target; clicking a SAME-FILE anchor scrolls to it; clicking a CROSS-FILE anchor loads then positions at the anchor
    - Validates: Requirement 18.2, 18.6, 18.7
- [ ] 24. In-shell markdown viewer link navigation ON the framework (BLOCKED: Req 14 / Task 21)
  - BLOCKED on the viewer-to-shell seam (same dependency as Task 21). Do NOT implement ahead of it.
  - [ ] 24.1 When the markdown viewer is surfaced in-shell, reach link activation through the single command-dispatch path / existing PREVIEW integration (Req 14); reuse the Task 22 pure helpers; file/anchor/external behaviour matches criteria 1-9; no bespoke dispatcher
    - Validates: Requirement 18.5 (and 18.1-18.3, 18.6-18.9 in-shell)

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Viewer Registry | AC 1 (Viewer_Key -> Box\<dyn FileViewer\>) | 3.1-3.2, 3.8 |
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
| P7 | Viewer_Panel never exposes mutable content reference | 8.7 | Req 8 AC 1-3 |
| P8 | Debounce coalesces rapid edits: N edits within window -> 1 refresh call | 9.7 | Req 9 AC 2, AC 3 |
| P9 | Configuration validation: split_ratio 0.1-0.9 only, debounce_ms positive only | 10.6 | Req 10 AC 1, AC 2 |
| P10 | Read-only invariant: no document mutation during Viewer_Mode regardless of input | 11.4 | Req 8 AC 3, AC 4 |

---

## Notes

- Tasks 2 and 3 can be implemented in parallel (trait definition and registry are independent after scaffold)
- Task 4 (built-in viewers) depends on both the trait (task 2) and registry (task 3)
- Task 5 (PREVIEW command) depends on the registry (task 3) and panel (task 8) -- panel can be a stub initially
- Tasks 7, 8, 9, and 10 depend on the trait and registry being complete
- Task 11 (read-only enforcement) is an integration-level concern that spans command dispatch and the panel
- All property tests use the `proptest` crate with a minimum of 100 iterations
- Built-in viewers are stubs -- full rendering logic lives in separate specs (`asa-report-preview`, `hex-display`)
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
