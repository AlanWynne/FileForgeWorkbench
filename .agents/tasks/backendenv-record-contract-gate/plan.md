# Implementation Plan -- BRC.2: Record-aware `ff-vfs::BackendEnvironment` store contract (CR-CH-060, Shape 2)

Scope: BRC.2 ONLY -- the additive record-aware store entry (the
Record_Store_Contract, Shape 2) on `ff-vfs::BackendEnvironment`, its supporting
types, provided default, `record_capable()` advertisement, and ff-vfs unit tests.
Delivers command-environments Requirement 18.1, 18.2, 18.3, 18.4, and defines the
outcome type that 18.8 references.

OUT OF SCOPE (do NOT touch -- separate steps): the editor SAVE-walk byte-vs-record
selection (BRC.3, document-model Req 13 / Req 18.5-18.6 in ff-document-model /
ff-desktop); the mainframe CE in ff-idcams (BRC.4, Req 18.7); RC.B.8 Part 2
(b)-(e) registry / binding / provider wiring. If satisfying the contract appears
to require editing the editor SAVE walk, the host CE impls, the registry, or the
mainframe CE -- STOP; those are not BRC.2.

## Design decisions (approved Shape 2; names finalised here)

The design.md CR-CH-060 section APPROVES Shape 2 with illustrative names and
explicitly delegates final naming to implementation. These are the finalised
choices, each grounded in the current code in
`crates/ff-vfs/src/backend_environment.rs`:

- `save(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()>` is RETAINED
  byte-for-byte UNCHANGED (Req 18.1). No edit to its signature, body, or doc.
- NEW record-aware method: `save_records(&self, target: &StoreTarget, records:
  &mut dyn RecordSource, attrs: &RecordAttrs) -> RecordStoreOutcome`, with a
  PROVIDED default body that returns `RecordStoreOutcome::NotRecordCapable`
  (Req 18.2, 18.3). `&mut dyn RecordSource` is object-safe (a `&dyn`/`&mut dyn`
  trait object, not a generic), so the trait stays `dyn`-compatible (Req 18.4).
- NEW `fn record_capable(&self) -> bool { false }` default -- backends advertise
  record capability; the mainframe CE (later) overrides to `true` (Req 18.3).
- `StoreTarget` -- a plain struct carrying dataset identity, NOT a host path:
  `dsn: String`, `owning_env: String`, `catalog_id: Option<String>`
  (the optional catalog-identity token). ff-vfs-local; NO dependency on
  ff-dscatalog / ff-volume. The mainframe CE later maps `StoreTarget` ->
  `DatasetAccess` (Req 18.2a, 18.7).
- `RecordAttrs` -- a plain struct: `recfm: RecordFormatKind`, `lrecl: u32`,
  `encoding: String` (an opaque encoding label, e.g. "cp037"/"utf-8").
  ff-vfs-local minimal representation; do NOT reuse `ff-dscatalog::Recfm`
  (Req 18.2c).
- `RecordFormatKind` -- an ff-vfs-local enum: `Fixed`, `Variable`, `Undefined`.
  `#[non_exhaustive]` so it can gain variants without a breaking change. This is
  the attrs' RECFM; it is NOT the editor's `RecordFormat` (that lives in
  ff-document-model and is consumed by BRC.3, out of scope here).
- `RecordSource` -- an OBJECT-SAFE trait modelling the editor's framed records as
  a pull stream over raw record bytes, defined in ff-vfs so there is NO dependency
  on ff-document-model:
  ```
  pub trait RecordSource {
      /// Pull the next record's raw bytes, or None at end of stream.
      fn next_record(&mut self) -> Option<&[u8]>;
  }
  ```
  Object-safe: `&mut self`, no generics, no `Self`-returning methods, no
  associated types in the signature. (A borrowing `next_record(&mut self) ->
  Option<&[u8]>` keeps the per-record bytes owned by the source, avoiding an
  allocation per record and keeping the trait object-safe. The lifetime is the
  elided `&mut self` borrow, which is object-safe.)
- `RecordStoreOutcome` -- a SIBLING to the existing `BackendOutcome` (not a reuse),
  because the record store has a third state the byte path does not need
  (not-record-capable, distinct from a handled-with-nonzero-rc soft failure):
  ```
  #[derive(Debug)]
  pub enum RecordStoreOutcome {
      /// The record-aware backend stored the records. rc == 0 on success; a
      /// non-zero rc is a backend-reported soft failure (e.g. an x37 space-full
      /// abend the mainframe DatasetAccess surfaces -- Req 18.8).
      Stored { rc: i32 },
      /// This backend is not record-capable (the provided default, or a backend
      /// that declines). Distinct from Stored{rc!=0}: nothing was attempted.
      NotRecordCapable,
  }
  ```
  Rationale (document this choice in the doc comment per the task): reusing
  `BackendOutcome` would conflate "declined because not record-capable" with
  "handled the verb but did not store" (`NotHandled`), which carry different
  caller semantics; a dedicated outcome keeps the record API's rc-carrying
  success and the not-capable decline unambiguous while still mirroring
  `BackendOutcome`'s rc convention (Req 18.8).

Object-safety is a construction fact here: every new item takes `&self`/`&mut
self`, uses only `&dyn`/`&mut dyn` trait-object or concrete-struct parameters, and
returns a concrete enum -- no generics, no `impl Trait`, no `Self` return. Verified
by a `Box<dyn BackendEnvironment>` test (Req 18.4).

## File layout (400-non-test-line rule)

`backend_environment.rs` is currently ~95 non-test lines. Adding four types, two
trait methods with doc comments, and keeping it readable stays WELL under 400
non-test lines (the `#[cfg(test)]` module does NOT count toward the 400 limit per
rust-standards.md), so NO production-code split is required.

CONVENTION: every other ff-vfs source file uses an INLINE
`#[cfg(test)] mod tests { use super::*; ... }` block at the bottom of the file
(confirmed: posix_provider.rs, error.rs, workspace.rs, transaction.rs, etc.; NO
sibling `#[path]` test files exist in ff-vfs). Match that convention: put the
tests in an inline `#[cfg(test)] mod tests` block in `backend_environment.rs`.
rust-standards.md's "split tests before ~200 lines" guidance is a SOFT ceiling --
these five tests with two small stub backends and a `SliceRecordSource` helper
should land near or just under 200 test lines. ONLY IF the inline test module
exceeds ~200 lines, split it into a sibling `backend_environment_tests.rs`
included via `#[cfg(test)] #[path = "backend_environment_tests.rs"] mod tests;`.
Default to inline (the crate convention); treat the sibling file as the fallback.

- Production types + trait: `crates/ff-vfs/src/backend_environment.rs` (one file;
  confirm it remains < 400 NON-TEST lines after the edit -- tests excluded).
- Tests: inline `#[cfg(test)] mod tests` in `backend_environment.rs` (crate
  convention), or a sibling `backend_environment_tests.rs` ONLY if the module
  exceeds ~200 lines (see above).
- Re-exports: extend the existing `pub use backend_environment::{...}` line in
  `crates/ff-vfs/src/lib.rs` to also export `StoreTarget`, `RecordAttrs`,
  `RecordFormatKind`, `RecordSource`, `RecordStoreOutcome`.

## `impl BackendEnvironment` sites that inherit the new defaults (full-gate risk audit)

`grep` for `impl BackendEnvironment` / `BackendEnvironment for` across the
workspace found exactly THREE impl sites, and all three implement ONLY
`name`/`is_case_sensitive`/`save` -- so all three inherit the new `save_records`
and `record_capable` provided defaults with ZERO edits:

1. `crates/ff-ce-posix/src/lib.rs` -- `PosixEnvironment` (production host CE).
2. `crates/ff-ce-ntfs/src/lib.rs` -- `NtfsEnvironment` (production host CE).
3. `crates/ff-desktop/src/tab_manager.rs` -- `tests::TestBackend` (test-only
   backend in ff-desktop's save-orchestration tests).

NOTE on "ff-ce-host-fs": the task brief names three host CEs including
`ff-ce-host-fs`, but there is NO `ff-ce-host-fs` crate with an `impl
BackendEnvironment` -- it is the host-fs DECIDER that resolves "native" to the
ntfs/posix environments. The two real production host impls are ntfs + posix.

FULL-GATE RISK: site #3 (`ff-desktop::tab_manager::tests::TestBackend`) is in a
crate BRC.2 does not scope-check. It implements only the three existing methods,
so it inherits the defaults and MUST compile unchanged -- but ff-desktop is only
exercised by the owner's full `cargo gate --build`, not by BRC.2's scoped checks.
Flag this in the hand-off: the owner's full gate must confirm ff-desktop still
compiles (expected to, since the change is purely additive with provided
defaults). Do NOT edit `TestBackend`.

## Implementation steps (TDD -- tests first, red before green)

- [ ] 1. Write the failing ff-vfs unit tests for the record-aware contract.
      Add an inline `#[cfg(test)] mod tests { use super::*; ... }` block at the
      bottom of `backend_environment.rs` (the ff-vfs convention; split to a
      sibling `backend_environment_tests.rs` only if the block exceeds ~200
      lines). Add these tests (each with
      `// Validates: command-environments Requirement 18.x`),
      using two in-file stub backends -- `ByteOnlyBackend` (implements only
      `save`, mirroring a host CE) and `RecordCapableBackend` (overrides
      `save_records` + `record_capable`, writing records into an in-memory
      `Vec<Vec<u8>>` sink) -- plus a `SliceRecordSource` test helper implementing
      `RecordSource` over a `&[&[u8]]`:
        - 18.1 `byte_save_entry_is_retained_and_writes_bytes`: `ByteOnlyBackend`
          (implements only `save`) compiles and its `save(path, bytes)` writes the
          exact bytes to a `tempfile::TempDir` path (byte-identical).
        - 18.3 `default_backend_is_not_record_capable`:
          `ByteOnlyBackend.record_capable() == false`.
        - 18.3 `default_save_records_declines`: calling `save_records` on
          `ByteOnlyBackend` (inheriting the default) returns
          `RecordStoreOutcome::NotRecordCapable`.
        - 18.2 `record_capable_backend_stores_records_with_target_and_attrs`:
          `RecordCapableBackend.record_capable() == true`; calling `save_records`
          with a `StoreTarget` (dsn/owning_env/catalog_id) + a `SliceRecordSource`
          of 3 records + `RecordAttrs { recfm: Fixed, lrecl: 80, encoding }`
          returns `RecordStoreOutcome::Stored { rc: 0 }` and the sink captured all
          3 records' bytes in order (and the test asserts the target's dsn/attrs
          were threaded through, e.g. stored alongside the records in the stub).
        - 18.4 `backend_environment_is_object_safe_for_both_entries`: build a
          `Box<dyn BackendEnvironment>` from `RecordCapableBackend`, then call BOTH
          `save` (byte) and `save_records` (record) THROUGH the `dyn` and assert
          each works. (Compiling this test IS the object-safety proof.)
        - 18.8 `record_store_outcome_carries_rc_and_distinguishes_decline`:
          `Stored { rc: 0 }` is success; a stub returning `Stored { rc: 8 }` is a
          soft failure; `NotRecordCapable` is distinguishable from both (assert via
          match arms / `matches!`).
      Files: crates/ff-vfs/src/backend_environment.rs (inline test module; or a
      new crates/ff-vfs/src/backend_environment_tests.rs ONLY if split per the
      ~200-line rule).
      Verify: `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command
      "cargo test -p ff-vfs"` -- the new tests FAIL to COMPILE (types/methods do
      not exist yet). This is the expected red; a compile failure is the red state
      for new API.

- [ ] 2. Add the supporting types to `backend_environment.rs`.
      Define `StoreTarget` (struct: `dsn: String`, `owning_env: String`,
      `catalog_id: Option<String>`), `RecordFormatKind` (`#[non_exhaustive]` enum:
      `Fixed`/`Variable`/`Undefined`), `RecordAttrs` (struct: `recfm:
      RecordFormatKind`, `lrecl: u32`, `encoding: String`), the `RecordSource`
      object-safe trait (`fn next_record(&mut self) -> Option<&[u8]>`), and the
      `RecordStoreOutcome` enum (`Stored { rc: i32 }`, `NotRecordCapable`). Derive
      `Debug` on all; `Clone`/`PartialEq`/`Eq` where meaningful (the plain-data
      structs and the outcome/kind enums). Rich `///` doc comments (what + why)
      each carrying `Validates: command-environments Requirement 18.x`
      (18.2 for StoreTarget/RecordAttrs/RecordSource, 18.8 for RecordStoreOutcome).
      ASCII only.
      Files: crates/ff-vfs/src/backend_environment.rs
      Verify: `cargo check -p ff-vfs` (via the pwsh7 wrapper) -- compiles; tests
      still fail only on the missing trait methods.

- [ ] 3. Add the two trait methods with provided defaults to `BackendEnvironment`.
      Add `fn save_records(&self, target: &StoreTarget, records: &mut dyn
      RecordSource, attrs: &RecordAttrs) -> RecordStoreOutcome { let _ = (target,
      records, attrs); RecordStoreOutcome::NotRecordCapable }` and `fn
      record_capable(&self) -> bool { false }`. Do NOT alter `name`,
      `is_case_sensitive`, or `save`. Add rich `///` docs: `save_records` doc must
      state it is the Record_Store_Contract, additive alongside the retained byte
      `save`, that the default declines so host CEs inherit it unchanged, and must
      document the `RecordStoreOutcome` choice (why a sibling to `BackendOutcome`).
      Carry `Validates: command-environments Requirement 18.2, 18.3` on
      `save_records` and `18.3` on `record_capable`. Update the trait-level doc to
      add `18.1, 18.2, 18.3, 18.4` to its `Validates:` line. ASCII only.
      Files: crates/ff-vfs/src/backend_environment.rs
      Verify: `cargo test -p ff-vfs` (pwsh7 wrapper) -- the new tests now PASS
      (green). Confirm no test regressions in ff-vfs.

- [ ] 4. Re-export the new public types from the crate root.
      Extend `pub use backend_environment::{BackendEnvironment, BackendOutcome};`
      in `crates/ff-vfs/src/lib.rs` to also re-export `StoreTarget`, `RecordAttrs`,
      `RecordFormatKind`, `RecordSource`, `RecordStoreOutcome`.
      Files: crates/ff-vfs/src/lib.rs
      Verify: `cargo check -p ff-vfs` (pwsh7 wrapper) -- compiles cleanly, no
      unused-import or missing-export warnings.

- [ ] 5. Verify host CEs still compile unchanged (inherit the defaults).
      Make NO edits to ff-ce-posix / ff-ce-ntfs. Confirm they compile and their
      existing byte-`save` tests still pass, proving the additive change left host
      CEs untouched (Req 18.1, 18.3).
      Files: (none -- verification only)
      Verify: `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command
      "cargo test -p ff-ce-ntfs -p ff-ce-posix"` -- all existing tests pass with
      zero source edits to those crates.

- [ ] 6. Lint and format the ff-vfs change.
      Files: crates/ff-vfs/src/backend_environment.rs, crates/ff-vfs/src/lib.rs
      (plus crates/ff-vfs/src/backend_environment_tests.rs if the tests were split)
      Verify (run each via the pwsh7 non-interactive wrapper):
        - `cargo clippy -p ff-vfs -- -D warnings` -- clean (no new lints).
        - `cargo fmt` then `cargo fmt -- --check` -- no formatting diff.
        - `cargo test -p ff-vfs` -- green.
      Also confirm ASCII-only (no prohibited chars) in the touched `.rs` files and
      that `backend_environment.rs` is still under 400 non-test lines.

## Scoped verification commands BRC.2 must pass (all via the pwsh7 wrapper)

Run each as `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command
"<cmd>"`:

- `cargo check -p ff-vfs`
- `cargo test -p ff-vfs`
- `cargo clippy -p ff-vfs -- -D warnings`
- `cargo test -p ff-ce-ntfs -p ff-ce-posix` (host CEs unchanged, byte-identical)
- `cargo fmt` and `cargo fmt -- --check`

Do NOT run `--workspace` or `cargo gate` -- those are the owner's manual full gate.
Hand off noting the ff-desktop `TestBackend` full-gate risk (expected to compile
unchanged; only the owner's full gate exercises ff-desktop).

## Doc edits BRC.2 must make

- [ ] TCR (`docs/quality/TCR.md`, Phase (backendenv-record-contract) section):
      flip the four `ff-vfs` rows from `NOT COVERED` (red circle) to `PASS`
      (checkmark) with the test-name citations (cite the actual location:
      `backend_environment.rs` tests module, or `backend_environment_tests.rs` if
      split):
        - Req 18.1 -> `byte_save_entry_is_retained_and_writes_bytes`
        - Req 18.2 -> `record_capable_backend_stores_records_with_target_and_attrs`
        - Req 18.3 -> `default_backend_is_not_record_capable` /
          `default_save_records_declines`
        - Req 18.4 -> `backend_environment_is_object_safe_for_both_entries`
      Also add a NEW PASS row for the outcome type that Req 18.8 references, cited
      to `record_store_outcome_carries_rc_and_distinguishes_decline`, noting it is
      the ff-vfs outcome TYPE only (the mainframe-CE x37 behaviour of 18.8 stays
      red / BRC.4). Leave the `ff-document-model` (18.5/18.6, 13.x) and `ff-idcams`
      (18.7/18.8 mainframe) rows RED -- out of scope.
- [ ] command-environments tasks (`docs/specs/command-environments/tasks.md`):
      mark task 23.1 and 23.2 `[x]` (the contract reshape + host-CE-unchanged
      confirmation). Leave tasks 24 and 25 `[ ]`.
- [ ] Master tasks (`docs/project-management/project-master/tasks.md`, Phase
      (backendenv-record-contract)): mark BRC.2 `[x]` (code-complete pending the
      owner's full gate). Leave BRC.3 / BRC.4 `[ ]`. Do NOT alter the Summary
      verdict beyond reflecting BRC.2.
- [ ] RESUME doc (`docs/status/RESUME-dataset-rationalisation.md`, SEQUENCED PLAN
      section): mark the BRC.2 bullet as code-complete pending the owner's full
      gate (scoped ff-vfs checks clean), noting the new types + methods landed and
      BRC.3 (editor SAVE-walk selection) is the next sequenced step.
- [ ] Do NOT create a change-log entry (CR-CH-060 is already logged and APPROVED;
      BRC.2 is approved implementation work, not a new gate).

## Guardrails (encoded above; restate for the implementer)

- ACYCLIC DAG: ff-vfs gains NO dependency on ff-document-model / ff-dscatalog /
  ff-volume / ff-idcams. All new types are ff-vfs-local. Do NOT add any `path`
  dep to `crates/ff-vfs/Cargo.toml` (dev-deps already have tempfile/
  pretty_assertions/proptest -- sufficient).
- Byte `save` RETAINED UNCHANGED (18.1); do NOT touch host CE impls or the
  ff-desktop `TestBackend`.
- Object-safe (18.4): proven by the `Box<dyn BackendEnvironment>` test.
- Single SAVE seam (18.6): add ONLY the trait entry; no dispatcher, no second save
  path, no registry edits.
- ASCII-only `.rs`; `backend_environment.rs` stays under 400 non-test lines (tests
  live in the sibling file).
