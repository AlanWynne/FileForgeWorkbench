# Implementation Plan -- BRC.3 (CR-CH-060): editor SAVE-walk byte-vs-record selection

Scope: BRC.3 ONLY. Wire + unit-test the SELECTION LOGIC in
`save_active_tab_via_backend` so the single CR-CH-053 Task 20/21 SAVE seam
chooses the BYTE store entry for Delimited/host documents and the RECORD-aware
store entry (`save_records`) for Fixed/Variable documents, driven by the
document's open-supplied `RecordFormat` AND `backend.record_capable()`, against
a record-capable TEST backend. Edits land DIRECTLY on `main`: no worktree, no
branch, no commit. OUT OF SCOPE (do NOT touch): BRC.4 (mainframe CE in
ff-idcams over `DatasetAccess`, Req 18.7) and RC.B.8 Part 2 (b)-(d) (open the
registry / bind `owning_env=MAINFRAME` / register the mainframe provider).

GUARDRAILS: ASCII-only `.rs`; 400-non-test-line rule; build ON the framework
(ONE seam, no second save fn / dispatcher); SCOPED checks only
(`-p ff-desktop`, `-p ff-vfs`, `-p ff-document-model`) -- NEVER `--workspace` or
the full gate.

---

## Design decisions (grounded in the code read)

The exploration confirmed every API this plan rides. Decisions, with rationale:

- D1. **Selection input (13.4).** The format is read from the stored value via
  `ff_document_model::Document::record_format() -> RecordFormat`
  (`crates/ff-document-model/src/document_records.rs`). It returns the value the
  owning CE set at open (`set_record_format`, Req 11.2); the save walk does NOT
  re-derive framing. `RecordFormat` has three variants: `Delimited { terminator }`,
  `Fixed { lrecl: u32 }`, `Variable { max_lrecl: u32, rdw: bool }`
  (`crates/ff-document-model/src/record_format.rs`).

- D2. **Selection rule (13.4 / 18.5).** Record path is taken ONLY when BOTH:
  the document format is `Fixed` or `Variable` (i.e. `!record_format.is_delimited()`),
  AND `backend.record_capable()` is `true`. In EVERY other case (Delimited, OR a
  non-record-capable backend even for Fixed/Variable) the existing BYTE path runs
  unchanged. This keeps native/host SAVE byte-identical and makes a
  not-record-capable backend fall back to bytes (see D6).

- D3. **Byte path unchanged (13.2).** The Delimited/native path keeps the EXACT
  current body: `doc.write().await.contiguous_view().to_vec()` then
  `backend.save(&path, &bytes)`. I deliberately do NOT switch it to
  `Document::save_image()` in this task -- `save_image()` is the F1-canonical
  byte-identical image and is semantically equivalent, but changing the byte
  source is an unrequested behaviour-adjacent change and risks a regression the
  task forbids. The byte path stays verbatim; byte-identicality is preserved by
  not touching it.

- D4. **RecordSource adapter lives in ff-desktop.** A new private adapter type in
  `tab_manager.rs` implements `ff_vfs::RecordSource`. The record query API
  (`total_records`, `record_start(RecordNumber)`, `record_byte_length(RecordNumber)`)
  is `&self` on `Document`; the full byte image comes from `save_image()` (also
  `&self`). The adapter OWNS a `Vec<u8>` image plus a `Vec<(usize,usize)>` of
  (start, len) record spans precomputed under the document lock, and yields
  `&self.image[start..start+len]` per `next_record()`. Owning the data makes the
  adapter `'static`-free of the document lock, object-safe (`&mut self`, no
  generics, returns `Option<&[u8]>`), and avoids holding the async lock across
  the backend call.

- D5. **StoreTarget identity from today's plumbing.** `StoreTarget` is built from
  what exists NOW: `dsn` = the tab path string (`TabState.path`), `owning_env` =
  `TabState.owning_environment` (defaults to `"HOSTFS"`, const
  `DEFAULT_OWNING_ENVIRONMENT` in `tab_state.rs`), `catalog_id = None`. This is a
  CLEARLY-MARKED PLACEHOLDER: real dataset identity (DSN / catalog) arrives when
  RC.B.8 (c) binds `owning_env = MAINFRAME` and the mainframe provider is
  registered (OUT OF SCOPE here). A doc comment on the construction states this.

- D6. **Outcome mapping + NotRecordCapable decision.**
  `RecordStoreOutcome -> Result<(), String>`:
  - `Stored { rc: 0 }` -> `Ok(())`.
  - `Stored { rc }` with `rc != 0` -> `Err(format!("Save failed: backend returned rc {rc}"))`
    (the rc appears in the message so the caller/status surfaces it).
  - `NotRecordCapable` -> **fall back to the BYTE path** (perform the existing
    `backend.save(&path, &bytes)` flatten-and-save), NOT an error. Justification:
    D2 already gates the record path on `record_capable()`, so reaching
    `NotRecordCapable` means a backend declined despite advertising capability --
    a defensive edge. Falling back to bytes keeps SAVE functional and
    byte-identical for host resources rather than failing a save the byte path
    could complete; it also means a mis-advertising backend degrades gracefully.
    This decision is documented in the code comment and in the DESIGN note (step 8).

- D7. **RecordAttrs mapping (13.6, GUI-independent plain data).** Map the document
  `RecordFormat` to `ff_vfs::RecordAttrs { recfm, lrecl, encoding }`:
  - `Fixed { lrecl }` -> `recfm = RecordFormatKind::Fixed`, `lrecl`, `encoding = "utf-8"`.
  - `Variable { max_lrecl, .. }` -> `recfm = RecordFormatKind::Variable`,
    `lrecl = max_lrecl`, `encoding = "utf-8"`.
  - `Delimited { .. }` never reaches this mapping (D2). A small helper returns
    `Option<(RecordFormatKind, u32)>`; `None` for Delimited routes to the byte path.
  Encoding is a plain string placeholder (`"utf-8"`); the real codepage (e.g.
  cp037) is a mainframe-CE concern bound later. No editor/shell type crosses the
  boundary -- only `RecordFormatKind` + `u32` + `String`.

- D8. **Single seam, in place (13.5).** Only `save_active_tab_via_backend` in
  `crates/ff-desktop/src/tab_manager.rs` changes. `host_fs_save`
  (`shell/dispatch_ffedit.rs`) stays the sole caller and is UNCHANGED. No second
  save function, no parallel dispatcher, no change to command dispatch / nav /
  focus / persistence seams. The dirty-flag-clear + `set_save_point()`
  orchestration runs identically on success for BOTH paths.

- D9. **ff-desktop already depends on ff-vfs and ff-document-model** (both are
  `use`d in `tab_manager.rs` today: `ff_vfs::BackendEnvironment`,
  `ff_document_model::...`). No `Cargo.toml` change is needed. `StoreTarget`,
  `RecordAttrs`, `RecordFormatKind`, `RecordSource`, `RecordStoreOutcome` are all
  public in `ff_vfs` (`crates/ff-vfs/src/backend_environment.rs`).

---

## Current state facts (verified)

- `crates/ff-desktop/src/tab_manager.rs` is **2349 lines total**; the
  `#[cfg(test)] mod tests` starts at **line 1397**, so **non-test code is ~1396
  lines** -- ALREADY far over the 400-line rule (pre-existing condition, not
  introduced by this task). See step 9 for the handling decision.
- `save_active_tab_via_backend(&mut self, backend: &dyn ff_vfs::BackendEnvironment,
  runtime: &Runtime) -> Result<(), String>` current body: resolve `path` from
  `tab.path` (Err "untitled" if none) -> `runtime.block_on` read bytes via
  `doc.write().await.contiguous_view().to_vec()` -> `backend.save(&path,&bytes)`
  mapped to `Err("Save failed: {e}")` -> `tab.is_modified = false` ->
  `tab.document.write().await.set_save_point()` -> `Ok(())`.
- Existing in-test backend is `struct TestBackend` (byte-only `save` via
  `std::fs::write`) inside the `tests` module. The record-capable model to copy
  is `RecordCapableBackend` in `crates/ff-vfs/src/backend_environment.rs` tests
  (captures records + last_target + last_attrs in `Mutex`, configurable `rc`).
- `host_fs_save` resolves the backend via `environments.host_fs_backend()`
  (`shell/environment_registry.rs`, returns `&dyn ff_vfs::BackendEnvironment`)
  and calls `tabs.save_active_tab_via_backend(backend, runtime)` with disjoint
  field borrows. Tab identity plumbing available: `TabState.path: Option<String>`
  and `TabState.owning_environment: String`.

---

## Plan

- [ ] 1. Add private record-save helpers to `tab_manager.rs` (outside the impl's
      existing body, same module): (a) a `RecordImageSource` struct holding
      `image: Vec<u8>` and `spans: Vec<(usize, usize)>` and `cursor: usize`,
      implementing `ff_vfs::RecordSource` (`next_record` yields
      `self.image.get(start..start+len)` and advances); (b) a free fn
      `record_attrs_for(format: ff_document_model::RecordFormat) ->
      Option<(ff_vfs::RecordFormatKind, u32)>` returning `Some((Fixed, lrecl))` /
      `Some((Variable, max_lrecl))` / `None` for Delimited (per D7). Keep all
      new code ASCII and under the function-size guidance (split logic into these
      small helpers rather than inflating the save fn).
      Files: crates/ff-desktop/src/tab_manager.rs
      Verify: `cargo check -p ff-desktop` compiles (helpers unused-until-step-2
      may warn; step 2 consumes them so defer the clippy run to step 3).

- [ ] 2. Rewrite the body of `save_active_tab_via_backend` IN PLACE to branch on
      the selection rule (D2). After resolving `path`: read the document's
      `RecordFormat` via `doc.record_format()` and compute, under one
      `runtime.block_on` read section, both the byte image (for the byte path)
      and -- only when the record path is selected -- the image + per-record spans
      from `total_records()` / `record_start(RecordNumber(k))` /
      `record_byte_length(RecordNumber(k))` over `save_image()`. Selection:
      if `record_attrs_for(format).is_some() && backend.record_capable()` -> build
      `StoreTarget` (D5), `RecordAttrs` (D7), a `RecordImageSource`, call
      `backend.save_records(&target, &mut source, &attrs)` and map the outcome
      per D6 (NotRecordCapable -> run the byte path); ELSE run the existing byte
      path verbatim (`contiguous_view().to_vec()` + `backend.save`). On success of
      EITHER path, keep the UNCHANGED orchestration: `tab.is_modified = false`
      then `set_save_point()`. Add a doc comment noting the CR-CH-060 selection,
      the placeholder StoreTarget identity (RC.B.8 (c) out of scope), and the
      NotRecordCapable fallback. Mind the borrow discipline already in the file
      (disjoint field borrows / lock scopes end before mutating `tab`).
      Files: crates/ff-desktop/src/tab_manager.rs
      Verify: `cargo check -p ff-desktop` compiles clean.

- [ ] 3. Run scoped lint/format on the changed crate and confirm no new warnings.
      Files: (none -- validation only)
      Verify: `cargo clippy -p ff-desktop` is clean and `cargo fmt -- --check`
      passes (run `cargo fmt` first if needed).

- [ ] 4. Add a record-capable test backend to the `tab_manager.rs` tests module
      (or the split sibling from step 8), modelled on `ByteOnlyBackend` /
      `RecordCapableBackend` in `ff-vfs`: a `CapturingRecordBackend` that
      overrides `record_capable() -> true` and `save_records(...)` to CAPTURE the
      received records (`Vec<Vec<u8>>`), `last_target: Option<StoreTarget>`,
      `last_attrs: Option<RecordAttrs>` in `Mutex`es, with a configurable `rc`,
      and whose byte `save` still writes to disk (so a byte-path assertion can
      read it back). Keep the existing `TestBackend` for the byte-path tests.
      Files: crates/ff-desktop/src/tab_manager.rs (tests)
      Verify: `cargo test -p ff-desktop save_` builds the test module (tests may
      be added in step 5 before this passes meaningfully).

- [ ] 5. Write failing-first unit tests (TDD red before green already satisfied
      because step 2 is implemented; add tests and confirm they PASS, and that an
      inverted assertion would fail). Each carries a `// Validates: Requirement
      X.Y` line. Cover:
      (13.2) a Delimited document saved via the capturing backend writes
      BYTE-identical content to disk and `save_records` was NOT called (captured
      records empty);
      (13.3) a `Fixed { lrecl }` document on the record-capable backend calls
      `save_records` with the expected re-framed records (each record's raw bytes
      per `record_byte_length`), a `StoreTarget` whose `owning_env` matches
      `TabState.owning_environment` and `dsn` matches the path, and `RecordAttrs {
      recfm: Fixed, lrecl, .. }`;
      (13.4) selection follows the STORED format: a document with
      `set_record_format(Fixed{..})` takes the record path while an identical
      byte-content Delimited document takes the byte path (no re-derivation);
      a Fixed document on a NON-record-capable backend (`TestBackend`) takes the
      BYTE path;
      (D6) outcome mapping: `rc: 0` -> `Ok`; a backend configured with `rc: 8` ->
      `Err` whose message contains `8`; a `NotRecordCapable`-returning backend
      falls back to the byte path and returns `Ok` with bytes on disk.
      Construct the Fixed/Variable document in-test via the REAL F1 API:
      build a file-backed tab, then
      `runtime.block_on(async { doc.write().await.set_record_format(
      ff_document_model::RecordFormat::Fixed { lrecl: 4 }) })` before saving.
      Files: crates/ff-desktop/src/tab_manager.rs (tests) [or sibling from step 8]
      Verify: `cargo test -p ff-desktop save` -- all new + existing save tests
      pass (`save_writes_document_content_to_file`, `save_clears_modified_flag`,
      `save_on_untitled_tab_is_noop` must STILL pass -> byte path not regressed).

- [ ] 6. Confirm the three EXISTING save tests remain green (byte-identical,
      modified-flag-clear, untitled-error) to prove 13.2 / 13.5 no-regression.
      Files: (none -- validation only)
      Verify: `cargo test -p ff-desktop save_writes_document_content_to_file
      save_clears_modified_flag save_on_untitled_tab_is_noop` all pass.

- [ ] 7. Re-run the ff-vfs and ff-document-model scoped tests to confirm the
      consumed contract APIs are unaffected (no changes expected to those crates,
      but the selection logic depends on their behaviour).
      Files: (none -- validation only)
      Verify: `cargo test -p ff-vfs` and `cargo test -p ff-document-model` pass.

- [ ] 8. 400-line handling for `tab_manager.rs`. The file's non-test code is
      ALREADY ~1396 lines (pre-existing over-limit, not caused by this change);
      the task is a REFACTOR-free behaviour addition to one existing fn plus two
      small private helpers. Per `rust-standards.md` the split-by-concern is a
      separate refactor; do NOT bundle a large file split into this gated change.
      DECISION: if the `#[cfg(test)] mod tests` block (currently starting ~line
      1397) grows past ~200 lines of NEW test code when steps 4-5 land, move the
      tests module to a sibling `#[path = "tab_manager_tests.rs"] mod tests;`
      (mirroring the `#[path = "..._tests.rs"]` pattern already used in
      ff-document-model) so the test file is independently sized. The non-test
      coordinator stays as-is; a full `tab_manager.rs` concern-split (_state /
      _render-free here, so _save helpers could move to a `tab_save.rs`) is NOTED
      as a follow-up refactor but is NOT required by BRC.3 and is out of this
      task's scope. Record this decision in the task summary and (if the owner
      wants it tracked) as a follow-up.
      Files: crates/ff-desktop/src/tab_manager.rs (+ optional
      crates/ff-desktop/src/tab_manager_tests.rs if the test block is split)
      Verify: `cargo test -p ff-desktop` still discovers and passes the moved
      tests if split; `cargo fmt -- --check` clean.

- [ ] 9. Write a short DESIGN note capturing the two documented decisions for the
      record walk: (a) the NotRecordCapable -> byte-fallback choice (D6) and its
      justification, and (b) the placeholder `StoreTarget` identity pending
      RC.B.8 (c) (D5). Place it as a code comment on `save_active_tab_via_backend`
      AND, if the sub-project design file exists, a one-paragraph addition to
      `docs/specs/document-model/design.md` (or `command-environments/design.md`)
      -- documentation edits only, no gate needed per workflow.md.
      Files: crates/ff-desktop/src/tab_manager.rs (comment);
      docs/specs/document-model/design.md (if present)
      Verify: `rg "[^\x00-\x7F]" crates/ff-desktop/src/tab_manager.rs` returns no
      matches (ASCII-only); the design note reads correctly.

---

## Anticipated out-of-scope full-gate risks (owner's manual `cargo gate --build`)

- `tab_manager.rs` non-test code is already ~1396 lines; a workspace-wide
  line-count lint (if any) will flag it, but that is PRE-EXISTING, not introduced
  here. The added helpers increase the count slightly; the test-module split
  (step 8) keeps the test side sized but does not reduce the non-test total.
- `RecordFormat::Variable` framing returns an EMPTY boundary set in F1
  (`frame_records` -> `Variable => Vec::new()`), and `build_original_index` is the
  non-Delimited record source. A `Variable` save in-test may therefore yield zero
  records against the resident image; the tests should center on `Fixed` (which
  frames arithmetically) and treat `Variable` as mapped-but-minimal (document
  this so a reviewer does not read empty-Variable as a bug).
- The byte path is left verbatim, so the three existing save tests and any
  full-shell SAVE tests should be unaffected; if a full-shell test asserts
  `save_records` is never called for host saves, the selection gate on
  `record_capable()` (host backend returns the default `false`) keeps that true.
- No `Cargo.toml` / public-API changes, so no cross-crate rebuild surprises are
  expected beyond `ff-desktop` itself.

---

## Hand-off

After steps 1-9 with scoped `cargo check/test/clippy -p ff-desktop`,
`-p ff-vfs`, `-p ff-document-model` and `cargo fmt` clean, STOP and hand off:
the owner runs the full `cargo gate --build` manually and reports back. Do NOT
run `--workspace` or `cargo gate` from this session.
