# Analysis Record: record-selection-criteria (W2.6)

- **Wave**: 2 (Catalog and dataset)
- **Backing crate**: `ff-select` (spec says `ff-criteria`; actual dir is
  `ff-select` -- naming drift). Content clearly matches: comparison/evaluator/
  logical/filter_state/wildcard/scope/persistence.
- **Spec files**: requirements.md (334 lines, 14 requirements), tasks.md
  (158 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 2 pass

---

## 1. Split candidacy

NOT a split candidate.

- Requirement volume: 14 reqs, 334 lines. Above the 12-req line, just under 350
  lines. (1 weak signal)
- Responsibilities: SEVEN capabilities (criteria-set def, comparison operators,
  logical AND/OR grouping, CRITERIA command, grid-display filtering, FIND/CHANGE
  scope, persistence) + UI panel -- all one cohesive concern (field-level record
  filtering). Well-decomposed into 16 files.
- Crates: single crate.
- Cohesion: high.

1 weak signal. No split.

### Source file sizes -- ALL WELL WITHIN CAP

No file exceeds 300 non-test lines. No PA-STD size item. (Best-sized Wave-2 unit.)

---

## 2. Cross-unit consistency

### PA-CONFLICT-008 EXTENSION -- third parallel field-type model, no ff-forge dep

The spec is explicit: "The `file_forge` crate continues to own all record parsing
and field extraction"; Packed_Decimal_Comparison "decoded to numeric value via the
`file_forge` crate". Cross-refs name fileforge-integration + structure-catalog for
Structure_Definition/Record_Structure/field extraction. But ff-select:

- Defines its OWN `FieldDataType` enum (types.rs: Int/Float/Packed/Str/Bool/Ebcdic)
  and `FieldTypes = HashMap<String, FieldDataType>`.
- Has NO dependency on ff-forge (fileforge-integration) OR ff-structure-catalog --
  its ONLY dep is ff-logging.

So this is a THIRD parallel field-type enum in the codebase (after ff-forge
`DataType` and ff-structure-catalog `FieldType`, PA-CONFLICT-008), and the crate
CANNOT actually delegate COMP-3 decoding to file_forge as the spec requires (no
dep). `comparison.rs` maps `FieldDataType::Packed -> PackedDecimal` mode but
sources the type from its own enum, not ff-forge. Recorded as a PA-CONFLICT-008
EXTENSION (not a new number): the record-structure/field-type model must unify
under ff-forge; ff-select should consume ff-forge's field-type + COMP-3 decode
rather than redefine. This is the third data point that the FileForge/record
cluster (ff-forge, structure-catalog, ff-select) has fragmented the field model
across crates that all should share ff-forge.

### FFW-ARCH-001 / config-mediation -- PA-WATCH-015 (raw std::fs where spec says config-managed)

Req 9.1 states the Criteria_Store is "managed through the configuration system's
user layer" (`~/.config/ffworkbench/criteria_store.toml`), and the cross-ref lists
configuration-system for "Criteria_Catalog path configuration". But ff-select:

- Has NO ff-config dep (only ff-logging), and
- Does DIRECT `std::fs` in `location.rs` and `persistence.rs` for the
  criteria_store.toml + `.criteria.json` files.

So the config-system mediation the spec mandates is BYPASSED -- the crate reads/
writes its store directly rather than through ff-config. This is a spec-vs-impl
architecture gap (analogous to workflow-engine PA-WATCH-004: a non-provider doing
raw file I/O; here also bypassing the mandated config layer). Recorded
PA-WATCH-015 (MEDIUM): route the Criteria_Store through ff-config's user layer
(add the ff-config dep) OR document why criteria files are managed outside config;
if raw file access is intended for the `.criteria.json` catalog, route it through
ff-vfs per FFW-ARCH-001. Only 2 fs call sites, so the fix is contained.

### find-and-replace scope integration -- consistent (Criteria_Scope)

Req: Criteria_Scope is a SearchScope modifier restricting FIND/CHANGE to
criteria-matching records (Glossary + capability 6). This mirrors the
exclude-show-filter EXCLUDED/VISIBLE scope (W1.11) and find-and-replace scope
readers (W1.8). ff-select EXPOSES the criteria filter; find-and-replace CONSUMES
it. Single-owner-many-readers -- consistent. Confirm the wiring at Wave 5
find-and-replace/scope integration.

### Public types and ownership (non-conflicting)

- Criteria_Set/Criterion/Criteria_Operator/Criteria_Connector, evaluator, logical
  grouping, wildcard matcher, CRITERIA command, Criteria_Store/`.criteria.json`
  persistence, panel model -- sole-owned by ff-select. No duplication of these.
- CRITERIA command (SET/CLEAR/SHOW) -- registered via command-framework. BUT
  ff-select has NO ff-command dep (only ff-logging), so command registration must
  be caller-mediated (trait/return-data) -- confirm it is not a broken cross-ref.
  Folded into PA-WATCH-015 (the crate's dep set is suspiciously minimal: ff-logging
  only, yet spec cross-refs command-framework + configuration-system).

### Cross-reference integrity

The 6 declared cross-refs resolve as sub-projects, BUT two (configuration-system,
command-framework) are NOT reflected as Cargo deps -- the crate depends only on
ff-logging. Either those integrations are caller-injected (clean seam) or the
cross-refs overstate the coupling. Recorded within PA-WATCH-015.

---

## 3. Completeness

Tracking: all 158 sub-tasks `[x]`. Implementation present across all 14 reqs
(criteria model, comparison engine, logical evaluator, wildcard, scope, CRITERIA
command, persistence, validator, association). Tests in-file. No false-positive
pattern. No PA-INCOMPLETE for functional behaviour.

### TCR gap (PA-TCR-012)

TCR.md has 1 row for the crate against 14 reqs / ~90 criteria. Thin. Recorded
PA-TCR-012.

---

## 4. Logging audit

Scan of `crates/ff-select/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT and is the ONLY dependency -- yet 0 uses. DEAD
  (and conspicuously so, being the sole dep).
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 2 (location.rs, persistence.rs -- see PA-WATCH-015)

Mandated logging unimplemented: Req 9.9 ("Criteria_Store corrupt -> initialise with
defaults, EMIT A WARNING"). The config.rs coercion messages (49-85: "must be a
string -- using default") and error.rs parse/io/corrupt errors are built as
strings/error types but never logged. With ff-logging as the sole dep and file I/O
present, this crate is a clear dev-logging + operational-WARN candidate. Recorded
PA-LOG-016 (MEDIUM): wire ff-logging for the Req 9.9 WARN + config-coercion
warnings + `.criteria.json` load failures (Req 9.7) + dev-logging on CRITERIA
command dispatch.

---

## 5. Task revision proposals

- **PA-CONFLICT-008 (EXTENSION, owner-gated)**: ff-select defines a third
  `FieldDataType` enum and cannot delegate COMP-3 to file_forge (no dep), despite
  the spec assigning field extraction/decode to file_forge. Unify the field-type
  model under ff-forge across ff-forge / structure-catalog / ff-select; ff-select
  consumes it. Handled with the PA-CONFLICT-008 resolution at fileforge-integration
  (Wave 5).
- **PA-WATCH-015 (MEDIUM)**: the Criteria_Store is spec'd as config-system-managed
  (Req 9.1) but implemented with raw `std::fs` and no ff-config dep; command +
  config cross-refs are not Cargo deps. Route the store through ff-config (and
  `.criteria.json` through ff-vfs per FFW-ARCH-001) OR document the caller-mediated
  design. Contained (2 fs sites).
- **PA-LOG-016 (MEDIUM)**: wire ff-logging (dead sole-dep) for the Req 9.9 WARN,
  config-coercion warnings, and `.criteria.json` load failures + dev-logging on
  CRITERIA dispatch.
- **PA-STD-030 (ASCII, ACTIONABLE -- runtime strings)**: 54 non-ASCII bytes
  (13 non-comment). em-dashes inside runtime config-warning + `#[error]` strings:
  config.rs (49/59/69/85), error.rs (40/58/67/78). Non-ASCII in user-facing output
  is a genuine defect. Replace with `--`/`-`. REFACTOR, no gate.
- **PA-TCR-012**: enumerate per-requirement TCR rows (1 row for 14 reqs). No code.
- **PA-DOC (naming)**: spec crate `ff-criteria`; actual `ff-select`. Fold into the
  naming reconciliation set.

No requirement CHANGE proposed; the spec is internally consistent and complete.

---

## Summary

record-selection-criteria (`ff-select`, spec says `ff-criteria`) is a cohesive,
best-sized Wave-2 unit (no file near the cap) implementing FileForge-mode record
filtering (criteria sets, AND/OR grouping, comparison operators, wildcard, grid/
find scope, `.criteria.json` persistence). Two architecture findings: (1) it
EXTENDS the PA-CONFLICT-008 pattern -- a THIRD parallel field-type enum
(`FieldDataType`) with no ff-forge dep, so it cannot delegate COMP-3 decode to
file_forge as the spec requires; the field-type model must unify under ff-forge.
(2) PA-WATCH-015 -- the Criteria_Store is spec'd as configuration-system-managed
but implemented with raw `std::fs` and no ff-config dep (config mediation bypassed;
also .criteria.json file access outside VFS). Plus the recurring dead-ff-logging
dep with a mandated Req 9.9 WARN absent (PA-LOG-016), runtime-string ASCII
(PA-STD-030), thin TCR (PA-TCR-012), and the ff-criteria/ff-select naming drift.
