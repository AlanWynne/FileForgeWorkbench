# Analysis Record: compiler-toolchain-integration (W5.5)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crates**: `ff-toolchain-api` (shared ToolchainPlugin trait + types),
  `ff-gcc-toolchain` (GCC plugin), `ff-rust-toolchain` (Rust plugin)
- **Spec files**: requirements.md (263 lines, 5 requirements), tasks.md
  (42 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy + cap violations (PA-SPLIT-014 / PA-STD-059)

The three-crate structure is correct (shared api + one plugin per toolchain -- a clean
extension-point design, Req 5). No CRATE split needed. But the two plugin crates are
single oversized files:

- `ff-gcc-toolchain/src/lib.rs` = 421 non-test lines (over the 400 cap).
- `ff-rust-toolchain/src/lib.rs` = 462 non-test lines (over the 400 cap).

Each plugin does detection + installation + build-invocation + diagnostic-parsing in one
file. Recorded PA-SPLIT-014 / PA-STD-059 (MEDIUM -- cap): split each plugin lib.rs by
concern (detection.rs / install.rs / build.rs / diagnostics.rs, with lib.rs the plugin
impl + re-exports). REFACTOR, no behaviour change. `ff-toolchain-api` (1 file, under
cap) is fine.

---

## 2. Cross-unit consistency -- CLEAN (well-wired, positive)

### Fully wired (NOT orphans) -- positive exemplar

`ff-toolchain-api` is used by ff-desktop (99 toolchain refs in the shell) + both
plugins; the plugins depend on the api. This is the properly-integrated extension-point
model -- a POSITIVE COUNTER-EXAMPLE to the Wave-4 orphans and the idcams orphan (W5.2):
shared trait crate + plugin crates + shell consumer, all connected. `MockToolchain`
(in the api) gives the plugins a testable seam (the source of the strong TCR).

### VFS discipline -- CLEAN (0 production raw fs)

ff-rust-toolchain shows 8 std::fs hits, but ALL are inside `#[cfg(test)]` (lines 543-589
-- building tempfile Cargo fixtures). Non-test lib code has 0 raw fs. Toolchains are
OS-level executables OUTSIDE the workbench VFS, located via `which` + run via Command
spawn -- so touching the real filesystem for detection/build is correct and NOT a VFS
bypass (unlike JES queue persistence PA-CONFLICT-015, which stores WORKBENCH data raw).
Clean.

### Provider/plugin abstraction consistent

`ToolchainPlugin` trait + `ToolchainState` / `Diagnostic` / `DiagnosticSeverity` /
`BuildProfile` / `BuildEvent` shared types (ff-toolchain-api) mirror the JES
`JobProvider` and connector provider patterns -- consistent extension-point design across
the plugin family.

### Public types and ownership

- ff-toolchain-api sole-owns the trait + shared diagnostic/build types; each plugin
  sole-owns its detection/install/build/parse. No duplication.

### Cross-reference integrity

Cross-refs (ff-toolchain-api <-> plugins <-> ff-desktop, `which`) all resolve and are
used. No dangling refs.

---

## 3. Completeness

Tracking: all 42 sub-tasks `[x]`. REAL implementation (not a skeleton -- contrast
database-tool W5.4): `which`-based detection, platform install (winget/apt/brew per the
master tasks), Command-spawn build invocation, diagnostic parsing, MockToolchain testing.
TCR = 38 rows across the three crates (api 10, gcc 14, rust 14) -- strong coverage. No
PA-INCOMPLETE. Genuinely complete.

---

## 4. Logging audit

Scan across the three crates:

- `ff_logging` / `log_*!`: 0 (all three); `ff-logging` dep: ABSENT (not dead).
- `println!` / `eprintln!`: 0.
- production `std::fs`: 0 (test-only fs).

A toolchain detect/install/build system is a meaningful trace site: detection probes
(which found what, versions), INSTALL steps (winget/apt/brew invocation + outcomes --
these can fail in many environment-specific ways), build invocation (command line, exit
code), diagnostic-parse results. Zero logging. Recorded PA-LOG-044 (MEDIUM): add
ff-logging + dev-logging on detection / install steps / build invocation / diagnostic
parsing under the `dev-logging` gate. Install-step logging is especially valuable (env-
specific failures). Pairs with the emulator logging gaps (PA-LOG-041/042).

---

## 5. Task revision proposals

- **PA-SPLIT-014 / PA-STD-059 (MEDIUM -- cap)**: split ff-gcc-toolchain/lib.rs (421) and
  ff-rust-toolchain/lib.rs (462) by concern (detection / install / build / diagnostics).
  REFACTOR.
- **PA-STD-060 (ASCII, comment separators + mojibake)**: the three crates have non-ASCII
  in comments -- box-drawing separators in .rs (NOT allowed in Rust source per
  documentation.md; box-drawing is .md-only) plus some corrupted-encoding artifacts and
  em-dashes (api 22, gcc 35, rust 32 bytes). Replace box-drawing separators with the
  ASCII `// === ... ===` form and em-dashes with `--`. REFACTOR, no gate.
- **PA-LOG-044 (MEDIUM)**: add detection / install / build / diagnostic dev-logging.

No PA-INCOMPLETE (genuinely complete). No orphan (well-wired). No production raw-fs
(clean). No TCR gap (38 rows).

---

## Summary

compiler-toolchain-integration (`ff-toolchain-api` + `ff-gcc-toolchain` +
`ff-rust-toolchain`) is a GENUINELY COMPLETE, well-integrated, well-tested unit -- a
POSITIVE exemplar after the Wave-4/W5.2 orphans and the W5.4 database-tool skeleton. The
three-crate extension-point design (shared trait api + one plugin per toolchain + shell
consumer, 99 toolchain refs in ff-desktop) is clean and fully wired; TCR is strong (38
rows); the implementation is real (`which` detection, platform install, Command-spawn
build, diagnostic parsing, MockToolchain). VFS discipline is CLEAN -- the only std::fs is
test-only, and locating/running OS compilers correctly sits outside the workbench VFS
(not a bypass like JES). Findings are hygiene: two plugin lib.rs files over the 400 cap
(gcc 421, rust 462 -- PA-SPLIT-014 / PA-STD-059, split by concern); comment-level ASCII
violations incl. box-drawing separators in .rs + mojibake (PA-STD-060); and 0 logging on
a detect/install/build system (PA-LOG-044, MEDIUM -- install-step logging especially
valuable for env-specific failures). No incompleteness, no orphan, no raw-fs.
