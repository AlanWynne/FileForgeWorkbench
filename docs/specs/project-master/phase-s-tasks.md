# Phase S — Binary Polish

Phase S closes the known gaps in the `ff-desktop` binary identified after Phase R completion.
All tasks follow the mandatory TDD workflow: failing test → implementation → green → clippy.

## Tasks

- [ ] S.0 Fix `ff-dsalloc` property test compile failure
  - [ ] S.0.1 Replace `'A'..='Z'` and `'0'..='9'` char range literals in `valid_dsn_strategy()` with `prop::char::ranges(vec!['A'..='Z'].into())` equivalents so `cargo test --workspace` exits 0
  - [ ] S.0.2 Fix moved-value borrow of `expected` at line 157 — change `prop_assert_eq!(result.text, expected, ...)` to `prop_assert_eq!(result.text, expected.clone(), ...)`
  - [ ] S.0.3 Confirm `cargo test -p ff-dsalloc` passes and `docs/TCR.md` row for `ff-dsalloc` updated to ✅

- [ ] S.1 Native file-open dialog (`File > Open…`)
  - [ ] S.1.1 Add `rfd` crate to `ff-desktop` dependencies (`rfd = { version = "0.15", features = ["async-std"] }`)
  - [ ] S.1.2 Write failing test: `file_open_dialog_pending_open_is_set_when_path_returned` — verifies that when `rfd::FileDialog` returns a path, `pending_open` is set to that path string
  - [ ] S.1.3 Add `open_file_dialog()` async helper to `shell.rs` that calls `rfd::AsyncFileDialog::new().pick_file().await` and sets `pending_open` if a file is chosen
  - [ ] S.1.4 Wire `File > Open…` menu button to spawn `open_file_dialog()` on the Tokio runtime via `runtime.spawn()`; remove the current stub that sets `pending_open` to empty string
  - [ ] S.1.5 Confirm `cargo clippy -p ff-desktop -- -D warnings` clean

- [ ] S.2 Keyboard text input (make the editor writable)
  - [ ] S.2.1 Read `docs/specs/edit-operations/requirements.md` Requirements 1 and 4 before writing any code
  - [ ] S.2.2 Write failing test: `typed_character_inserts_into_document` — creates a tab with known content, calls the insert handler with a char, asserts document line content changed (Validates: edit-operations Req 1.1)
  - [ ] S.2.3 Write failing test: `backspace_deletes_character_before_cursor` — asserts document shrinks by one char (Validates: edit-operations Req 4.1)
  - [ ] S.2.4 Write failing test: `enter_key_splits_line_in_insert_mode` — asserts line count increases by 1 (Validates: edit-operations Req 2.1)
  - [ ] S.2.5 In `editor_panel.rs`, handle `egui::Event::Text(s)` — for each char in `s`, call `doc.insert(byte_pos, char.as_bytes())` on the active tab's document; advance cursor column; mark tab modified
  - [ ] S.2.6 Handle `egui::Key::Backspace` — call `doc.delete(range_before_cursor)` if cursor is not at document start; update cursor; mark tab modified
  - [ ] S.2.7 Handle `egui::Key::Enter` — insert the document's line-ending bytes at cursor position; advance cursor to column 1 of next line; mark tab modified; update `line_count` on tab
  - [ ] S.2.8 After any edit, refresh `tab.line_count` from `doc.line_count()` so the status bar stays accurate
  - [ ] S.2.9 Confirm all 3 new tests pass and `cargo clippy -p ff-desktop -- -D warnings` clean

- [ ] S.3 Save to disk (`File > Save`)
  - [ ] S.3.1 Read `docs/specs/file-operations/requirements.md` Requirements 1 and 7 before writing any code
  - [ ] S.3.2 Write failing test: `save_writes_document_content_to_file` — opens a temp file, modifies the document, calls `save_active_tab()`, reads the file back, asserts content matches (Validates: file-operations Req 1.1)
  - [ ] S.3.3 Write failing test: `save_clears_modified_flag` — after a successful save, `tab.is_modified` is `false` (Validates: file-operations Req 1.2 / edit-operations Req 11.7)
  - [ ] S.3.4 Write failing test: `save_on_untitled_tab_is_noop` — calling save on a tab with no path does not panic and returns an error (Validates: file-operations Req 1.4)
  - [ ] S.3.5 Add `save_active_tab(tabs: &mut TabManager, runtime: &Runtime) -> Result<(), String>` to `tab_manager.rs` — extracts document bytes, writes via `LocalFsProvider`, clears `is_modified`
  - [ ] S.3.6 Wire `File > Save` menu button and `Ctrl+S` key event in `shell.rs` to call `save_active_tab()`; display any error in `open_error`
  - [ ] S.3.7 Confirm all 3 new tests pass and `cargo clippy -p ff-desktop -- -D warnings` clean
