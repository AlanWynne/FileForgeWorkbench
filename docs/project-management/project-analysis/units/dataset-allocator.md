# Analysis Record: dataset-allocator (W2.3)

- **Wave**: 2 (Catalog and dataset)
- **Backing crate**: `ff-dsalloc` (spec + governance call it `ff-dataset-allocator`;
  actual dir is `ff-dsalloc` -- naming drift, and it delegates to what the spec
  calls `ff-dataset-catalog` = actual `ff-dscatalog`)
- **Spec files**: requirements.md (378 lines, 16 requirements; governed by ADR-001
  dataset-ownership-model), tasks.md (181 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 2 pass

---

## 1. Split candidacy

NOT a split candidate.

Assessment against the 2-of-4 rule:

- Requirement volume: 16 reqs, 378 lines. Above the 12-req line, marginally above
  350 lines. (1 weak signal)
- Responsibilities: ONE cohesive domain -- JCL-driven dataset allocation
  (DD parsing, DISP, symbolic substitution, referback, GDG relative refs, RESOLVE
  command). ADR-001 Req 4 explicitly scopes it to exactly this. Well-decomposed
  into 21 files (parser/dd_statement/operands/symbols/referback/gdg_resolver/
  allocation/pipeline/catalog_bridge/...).
- Crates: single crate; minimal deps (thiserror/serde/toml/chrono only -- catalog
  access is trait-injected, see below).
- Cohesion: high.

1 weak signal. No split.

### Source file size violation (PA-STD-027)

`pipeline.rs` = 412 non-test lines, just OVER the 400 cap (parser.rs 392,
allocation.rs 359, operands.rs 317 are near/under). Split the pipeline
orchestration (parse -> substitute -> resolve -> validate) by stage. REFACTOR,
no gate. Recorded PA-STD-027.

---

## 2. Cross-unit consistency

### PA-WATCH-011 (catalog CRUD delegation) -- RESOLVED CLEAN

The W2.1 watch (does ff-dsalloc invoke rather than duplicate catalog CRUD?)
resolves CLEAN for CRUD/resolution/GDG:

- ff-dsalloc has NO Cargo dep on ff-dscatalog (deps = thiserror/serde/toml/chrono).
- `catalog_bridge.rs` defines a `CatalogProvider` trait (`lookup_dsn`,
  `verify_member`, `query_gdg`, `allocate_dataset`, `dataset_exists`) and delegates
  ALL catalog access through it. The spec: "delegates all catalog operations to
  ff-dataset-catalog via the CatalogService trait interface"; the code comment:
  "Production implementation delegates to ff-dataset-catalog; tests use MockCatalog."
- Only `MockCatalog` lives here; the production impl is wired by the shell at
  runtime (clean-seam pattern, same as Wave-1 sequence-numbers/auto-indent). So it
  CANNOT duplicate catalog CRUD -- it has no catalog storage code at all.
- Matches ADR-001 Req 4.3-4.7 exactly (obtain metadata exclusively from catalog
  services; depend via a trait for mock testing). Architecturally correct.

PA-WATCH-011 (the CRUD half) is CLEARED. The ff-idcams/listcat half remains for
Wave 5.

### PA-CONFLICT-006 (NEW) -- duplicate DSN naming validation vs ADR-001

ONE governance conflict: ff-dsalloc `dsn.rs` defines its OWN `DatasetName` with
`parse()` "which enforces z/OS DSN syntax rules" (qualifier rules, uppercase,
length). But ADR-001 Req 3.1 states ff-dataset-catalog "SHALL own ... dataset
naming validation (DSN syntax, qualifier rules, HLQ management)" and exposes
`validate_dsn(dsn)` (Req 3.4). ADR-001 Req 4.3: the allocator "SHALL obtain all
dataset metadata exclusively from ff-dataset-catalog services."

So ff-dsalloc having its own DSN syntax validator DUPLICATES the naming-validation
the catalog is the SINGLE AUTHORITY for -- a governance conflict (same
duplicate-type anti-pattern as PA-CONFLICT-003/004/005). Nuance: DD-STATEMENT
parsing (the JCL syntax around `DSN=...`) is legitimately the allocator's (ADR-001
Req 4.1); but the DSN-string SYNTAX VALIDATION is the catalog's. Recorded
PA-CONFLICT-006 (owner-gated): route DSN validation through
`ff-dscatalog::validate_dsn` (via the CatalogProvider trait -- add a `validate_dsn`
method), delete ff-dsalloc's own validator OR document it as a permitted
pre-parse convenience that MUST agree with the catalog's rules. Also note
ff-dscatalog ALSO has its own dsn.rs (454 lines, W2.1) -- there are now THREE DSN
parsers in the codebase if find-and-replace-style duplication spreads; consolidate
under the catalog as the single owner.

### ADR-001 also names ff-vsam-services (reinforces PA-SPLIT-008)

ADR-001 Req 5 defines a separate `ff-vsam-services` crate owning VSAM record ops
(KSDS/ESDS/RRDS/LDS). This CONFIRMS the W2.1 PA-SPLIT-008 recommendation to extract
VSAM/storage out of ff-dscatalog -- the governance model already anticipates a
dedicated VSAM crate. Cross-linked: PA-SPLIT-008's `ff-record-store`/`ff-vsam`
target = ADR-001's `ff-vsam-services`.

### Public types and ownership

- DD_Statement/operands/DISP/DCB/SPACE parsing, symbol table, referback resolver,
  GDG relative-ref resolver, RESOLVE command, `CatalogProvider` trait,
  `LintDiagnostic` -- sole-owned by `ff-dsalloc`. No duplication EXCEPT DatasetName
  (PA-CONFLICT-006).
- RESOLVE command (`dataset.resolve`) registered via command-framework. Consistent.
- Config (default HLQ, symbol tables, resolution prefs) read from ff-config.
  Consistent.
- 0 fs calls -- CLEAN (all resource access via the catalog trait / VFS at the
  caller, per ADR-001 Req 4.2). No FFW-ARCH-001 concern.

### Cross-reference integrity

All declared cross-refs (dataset-catalog, virtual-file-system, command-framework,
language-service, configuration-system, logging) resolve. No dangling refs.

---

## 3. Completeness

Tracking: all 181 sub-tasks `[x]`. Implementation present across all 16 reqs
(DD parsing, DISP, DCB, SPACE, continuation, symbolic substitution, referback,
GDG resolution, concatenation, temp datasets, RESOLVE command, lint, panel). Tests
in-file with `// Validates:` annotations (catalog_bridge tests verify the mock).
No PA-INCOMPLETE raised.

### TCR gap (PA-TCR-010)

TCR.md has 1 row for ff-dsalloc against 16 reqs / ~120 criteria. Thin (like the
Wave-1 thin cluster). Contrast the sibling ff-dscatalog (99 rows). Recorded
PA-TCR-010: enumerate per-requirement rows citing the existing in-file tests.

---

## 4. Logging audit

Scan of `crates/ff-dsalloc/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: ABSENT -- BUT the spec Introduction lists
  "`ff-logging` -- for structured diagnostics" as a declared dependency. So there
  is SPEC-VS-IMPL drift: the spec claims an ff-logging dependency the crate does
  not have.
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0

The allocator's diagnostics are returned as `LintDiagnostic` DATA (Req 1.11, 16 --
ERROR/WARN severities) rather than logged. That is a reasonable return-data design
for a pure parser/resolver (the RESOLVE panel + lint UI consume the diagnostics).
But the spec claiming ff-logging means either the dep should be added and used, or
the spec should drop the claim. Recorded PA-LOG-014 (LOW): reconcile -- add
ff-logging + dev-logging on RESOLVE/allocation-pipeline stages (CR-NR-058) OR
correct the spec's dependency list. Given the crate is a pure resolver returning
diagnostics, the dev-logging trace on the parse->resolve pipeline is the useful
addition; hard operational logging is less critical than for ff-dscatalog
(PA-LOG-012).

---

## 5. Task revision proposals

- **PA-CONFLICT-006 (owner-gated, MEDIUM)**: ff-dsalloc `DatasetName::parse()`
  duplicates DSN naming validation that ADR-001 assigns exclusively to
  ff-dscatalog (`validate_dsn`). Route validation through the CatalogProvider trait
  (add `validate_dsn`), delete the local validator OR document it as a permitted
  pre-parse convenience that MUST match the catalog rules. Consolidate the 2-3 DSN
  parsers under ff-dscatalog. Owner decision + code.
- **PA-STD-027 (REFACTOR)**: split `pipeline.rs` (412 non-test, over cap) by stage.
  No gate.
- **PA-STD-028 (ASCII, ACTIONABLE -- runtime strings)**: 79 non-ASCII bytes
  (7 non-comment). em-dashes inside runtime `#[error]`/message strings in
  error.rs (57/104), gdg_resolver.rs (148/170/187), allocation.rs (537 comment),
  and a `->` arrow in panel.rs:175 runtime format string ("{} -> {}"). Non-ASCII in
  user-facing output is a genuine defect. Replace with `--`/`-`/`->`. REFACTOR,
  no gate.
- **PA-LOG-014 (LOW)**: reconcile the ff-logging spec-vs-impl drift (spec lists it
  as a dep; crate lacks it). Add dev-logging on the RESOLVE/allocation pipeline
  (CR-NR-058) OR correct the spec dependency list.
- **PA-TCR-010**: enumerate per-requirement TCR rows (1 row for 16 reqs). No code.
- **PA-DOC (naming)**: spec/governance say `ff-dataset-allocator` +
  `ff-dataset-catalog`; actual crates are `ff-dsalloc` + `ff-dscatalog`. Fold into
  the PA-W1.12-style naming reconciliation. Doc-only.

No requirement CHANGE proposed; the spec is internally consistent (governed by
ADR-001). PA-CONFLICT-006 is a code/ownership fix, not a spec correction.

---

## Summary

dataset-allocator (`ff-dsalloc`) is a clean, well-decomposed JCL-driven allocation
engine that RESOLVES the W2.1 PA-WATCH-011 delegation question in the affirmative:
it defines a `CatalogProvider` trait and delegates ALL catalog CRUD/resolution/GDG
through it (no ff-dscatalog dep, no catalog storage code, MockCatalog for tests) --
exactly matching ADR-001 Req 4. Not a split candidate; one small cap violation
(pipeline.rs 412, PA-STD-027). The one governance conflict is PA-CONFLICT-006:
ff-dsalloc defines its OWN DSN syntax validator (`DatasetName::parse`), duplicating
the naming validation ADR-001 assigns exclusively to ff-dscatalog (`validate_dsn`)
-- route it through the catalog trait. ADR-001 Req 5 (ff-vsam-services) reinforces
the W2.1 PA-SPLIT-008 VSAM extraction. Minor items: ff-logging spec-vs-impl drift
(PA-LOG-014, LOW), runtime-string ASCII (PA-STD-028), thin TCR (PA-TCR-010, vs the
sibling catalog's 99), and crate-name doc drift.
