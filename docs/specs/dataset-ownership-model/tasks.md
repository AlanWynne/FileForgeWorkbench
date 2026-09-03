# Implementation Plan: Dataset Ownership Model (Governance Infrastructure)

## Overview

This task plan implements the architectural governance infrastructure for the Dataset Ownership Model. It does NOT produce a standalone crate — instead, it produces:

1. An architectural compliance test suite (`tests/architecture_compliance.rs`)
2. Trait interface definitions in owning crates (CatalogService, VsamService, AllocatorService)
3. Mock compilation tests proving trait-based coupling
4. Subsystem specification alignment updates
5. CI pipeline integration

**Location:** Workspace-level tests + trait definitions in owning crates
**Dependencies:** All dataset-related crates must exist (at least as stubs) for compliance tests to run
**Downstream impact:** All dataset-related crates must conform to the ownership rules

---

## Tasks

- [x] 1. Architectural compliance test infrastructure
  - [x] 1.1 Create `tests/architecture_compliance.rs` at workspace root — integration test file for dependency direction verification
  - [x] 1.2 Implement `parse_cargo_toml(path)` helper — read and parse a Cargo.toml file, extract `[dependencies]` and `[dev-dependencies]` sections
  - [x] 1.3 Implement `workspace_members()` helper — enumerate all workspace member crate paths from root Cargo.toml
  - [x] 1.4 Implement `DependencyRule` struct and `PROHIBITED_DEPENDENCIES` constant — static list of all prohibited dependency relationships from Requirement 7
  - [x] 1.5 Write test `vfs_has_no_domain_dependencies` — verify ff-vfs Cargo.toml has no dependency on ff-idcams, ff-dataset-catalog, ff-dataset-allocator, or ff-vsam-services
    - Validates: Requirement 2 AC 3; Requirement 7 AC 3
  - [x] 1.6 Write test `dataset_catalog_has_no_upstream_dependencies` — verify ff-dataset-catalog Cargo.toml has no dependency on ff-idcams or ff-dataset-allocator
    - Validates: Requirement 3 AC 3; Requirement 7 AC 3
  - [x] 1.7 Write test `vsam_services_has_no_upstream_dependencies` — verify ff-vsam-services Cargo.toml has no dependency on ff-idcams or ff-dataset-allocator
    - Validates: Requirement 5 AC 3; Requirement 7 AC 3
  - [x] 1.8 Write test `dataset_allocator_has_no_idcams_dependency` — verify ff-dataset-allocator Cargo.toml has no dependency on ff-idcams
    - Validates: Requirement 7 AC 5
  - [x] 1.9 Write test `idcams_has_no_storage_engine_dependencies` — verify ff-idcams has no transitive dependency on rusqlite, rocksdb, or lmdb (use `cargo tree` output or Cargo.lock parsing)
    - Validates: Requirement 6 AC 3; Requirement 7 AC 3
  - [x] 1.10 Implement violation reporting — clear error messages identifying the violating crate, the prohibited dependency, and the governance rule reference
    - Validates: Requirement 18 AC 1, AC 2, AC 5

- [x] 2. CatalogService trait interface definition
  - [x] 2.1 Define `CatalogService` trait in `ff-dataset-catalog/src/traits.rs` (or `src/service.rs`) with all methods from Requirement 15: create_dataset, delete_dataset, update_dataset, rename_dataset, resolve_dsn, dataset_exists, get_dataset_attributes, list_datasets, validate_dsn, create_gdg_base, create_generation, resolve_generation, list_generations, get_allocation_defaults
    - Validates: Requirement 15 AC 1–6
  - [x] 2.2 Define associated types on `CatalogService`: `DatasetAttributes`, `DatasetId`, `ResolutionResult`, `DatasetEntry`, `DatasetFilter`, `GenerationInfo`, `DsnValidationError`, `Dsorg`
    - Validates: Requirement 15 AC 2–5
  - [x] 2.3 Define `DynCatalogService` object-safe wrapper trait with concrete `CatalogError` type for dynamic dispatch
    - Validates: Requirement 15 AC 7
  - [x] 2.4 Implement blanket impl: `impl<T: CatalogService<Error=CatalogError> + Send + Sync> DynCatalogService for T`
    - Validates: Requirement 15 AC 7
  - [x] 2.5 Export `CatalogService` and `DynCatalogService` from `ff-dataset-catalog` crate root
    - Validates: Requirement 15 AC 1
  - [x] 2.6 Write compilation test in ff-dataset-catalog: verify `DynCatalogService` is object-safe (`Box<dyn DynCatalogService>` compiles)
    - Validates: Requirement 15 AC 7

- [x] 3. VsamService trait interface definition
  - [x] 3.1 Define `VsamService` trait in `ff-vsam-services/src/traits.rs` with all methods from Requirement 16: create_ksds, create_esds, create_rrds, create_lds, destroy_dataset, initialize_dataset, open, get, put, delete, close, start_browse, next_record, end_browse, define_aix, build_index
    - Validates: Requirement 16 AC 1–5
  - [x] 3.2 Define associated types: `VsamHandle`, `BrowseHandle`, `Record`, `VsamType`, `VsamParams`, `AccessMode`, `BrowseDirection`, `KeyField`, `VsamError`
    - Validates: Requirement 16 AC 2–5
  - [x] 3.3 Ensure `VsamService` trait is object-safe (all methods use `&self`, no associated types in return position that prevent object safety)
    - Validates: Requirement 16 AC 6
  - [x] 3.4 Implement a no-op stub `StubVsamService` that returns `VsamError::NotImplemented` for all methods — enables dependent crates to compile before full VSAM implementation
    - Validates: Requirement 16 AC 7
  - [x] 3.5 Export `VsamService` and `StubVsamService` from `ff-vsam-services` crate root
    - Validates: Requirement 16 AC 1, AC 7
  - [x] 3.6 Write compilation test: verify `Box<dyn VsamService>` compiles (object safety)
    - Validates: Requirement 16 AC 6

- [x] 4. AllocatorService trait interface definition
  - [x] 4.1 Define `AllocatorService` trait in `ff-dsalloc/src/traits.rs` with methods from Requirement 17: resolve_job, resolve_dsn, substitute_symbols
    - Validates: Requirement 17 AC 1–5
  - [x] 4.2 Use existing types: `ResolveMode`, `ResolveOutput`, `SymbolTable`, `JclResolverError` (all defined in owning modules)
    - Validates: Requirement 17 AC 2–5
  - [x] 4.3 Ensure `AllocatorService` trait is object-safe (all methods use `&self`, no generics in return position)
    - Validates: Requirement 17 AC 7
  - [x] 4.4 Export `AllocatorService` from `ff-dsalloc` crate root
    - Validates: Requirement 17 AC 1
  - [x] 4.5 Write compilation test: verify `Box<dyn AllocatorService>` compiles (object safety)
    - Validates: Requirement 17 AC 7

- [x] 5. Mock compilation tests — trait-based coupling verification
  - [x] 5.1 Write test in `ff-governance-tests/tests/mock_compilation.rs`: construct a mock `CatalogService` implementation, exercise all trait methods, verify compilation succeeds
    - Validates: Requirement 4 AC 7; Requirement 18 AC 4
  - [x] 5.2 Write test in `ff-governance-tests/tests/mock_compilation.rs`: construct mock implementations of `CatalogService` and `VsamService`, simulate DEFINE/DELETE workflows, verify compilation succeeds
    - Validates: Requirement 6 AC 9; Requirement 18 AC 4
  - [x] 5.3 Write test `dataset_allocator_source_has_no_rusqlite_imports` — scan ff-dsalloc source files for `use rusqlite`
    - Validates: Requirement 12 AC 3
  - [x] 5.4 Write test `idcams_source_has_no_storage_imports` — scan ff-idcams source files for storage engine imports
    - Validates: Requirement 6 AC 3; Requirement 21 AC 1

- [x] 6. Subsystem specification alignment — dataset-catalog
  - [x] 6.1 Add clarification note to `dataset-catalog/requirements.md` Requirement 7 stating that JCL-driven allocation workflows (DD parsing, DISP interpretation, symbolic substitution) are owned by ff-dataset-allocator; this requirement defines only the low-level catalog CRUD API
    - Validates: Requirement 19 AC 2a
  - [x] 6.2 Add clarification note to `dataset-catalog/requirements.md` Requirement 13 stating that `catalog.listcat` and `catalog.listds` are workbench-native commands distinct from the IDCAMS `LISTCAT` command (which is owned by ff-idcams)
    - Validates: Requirement 19 AC 2b
  - [x] 6.3 Add cross-reference to dataset-ownership-model governance document at the top of `dataset-catalog/requirements.md`
    - Validates: Requirement 19 AC 5

- [x] 7. Subsystem specification alignment — dataset-allocator
  - [x] 7.1 Verify that `dataset-allocator/requirements.md` Requirement 2 AC 8 explicitly names the `CatalogService` trait interface — confirmed present
    - Validates: Requirement 19 AC 3a
  - [x] 7.2 `dataset-allocator/requirements.md` Requirement 14 AC 3 already clarifies that allocation defaults flow through `CatalogService::get_allocation_defaults()` — confirmed present
    - Validates: Requirement 19 AC 3c
  - [x] 7.3 Cross-reference to dataset-ownership-model governance document is at the top of `dataset-allocator/requirements.md` — confirmed present
    - Validates: Requirement 19 AC 5

- [x] 8. Subsystem specification alignment — idcams-emulator
  - [x] 8.1 Verify that `idcams-emulator/requirements.md` Requirements 2–14 use delegation language (invoke `CatalogService`, `VsamService`) — confirmed present with full delegation model table
    - Validates: Requirement 19 AC 1a, AC 1b, AC 1c
  - [x] 8.2 Verify that `idcams-emulator/requirements.md` Requirement 21 (Ownership Boundary Enforcement) is present and complete — confirmed present
    - Validates: Requirement 19 AC 1
  - [x] 8.3 Cross-reference to dataset-ownership-model governance document is in the Introduction of `idcams-emulator/requirements.md` — confirmed present
    - Validates: Requirement 19 AC 5

- [x] 9. Dataset lifecycle ownership documentation
  - [x] 9.1 Create `docs/dataset-lifecycle.md` documenting the create→use→modify→delete lifecycle with ownership at each stage, referencing Requirements 13 and 14
    - Validates: Requirement 13 AC 1–6; Requirement 14 AC 1–5
  - [x] 9.2 Include sequence diagrams for: dataset creation (via IDCAMS and via JCL allocator), dataset access (DSN resolution → VFS I/O), dataset modification (ALTER), dataset deletion, GDG generation lifecycle
    - Validates: Requirement 13 AC 1–5
  - [x] 9.3 Include the resolution lifecycle sequence: DD parse → symbolic substitution → referback → catalog resolve_dsn → physical path return
    - Validates: Requirement 14 AC 1–5

- [x] 10. CI pipeline integration
  - [x] 10.1 Add `cargo test -p ff-governance-tests` step to CI pipeline configuration
    - Validates: Requirement 18 AC 5
  - [x] 10.2 Verify the step fails the build (non-zero exit code) when a prohibited dependency is introduced
    - Validates: Requirement 18 AC 5
  - [x] 10.3 Document the CI step in the project's contributing guide, explaining how to update the fitness function when adding a new dataset-related crate
    - Validates: Requirement 20 AC 4

- [x] 11. Future extensibility documentation
  - [x] 11.1 `docs/dataset-lifecycle.md` includes a Future Extensibility section describing the ADR amendment → trait interface → fitness function update process
    - Validates: Requirement 20 AC 1–5
  - [x] 11.2 `docs/adr/template-dataset-subsystem.md` created with Ownership, Prohibited Dependencies, Trait Interface, Integration Pattern, and Fitness Function Updates sections
    - Validates: Requirement 20 AC 1

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Single Authority | AC 1 (one owner per capability) | 2.1, 3.1, 4.1 |
| Req 1: Single Authority | AC 2 (invoke via API) | 5.1, 5.2 |
| Req 1: Single Authority | AC 3 (compile-time enforcement) | 1.5–1.9 |
| Req 1: Single Authority | AC 4 (new capability ADR) | 11.1, 11.2 |
| Req 1: Single Authority | AC 5 (precedence) | 6.1–6.3, 7.1–7.3, 8.1–8.3 |
| Req 2: ff-vfs Boundary | AC 1 (VFS owns) | Design doc |
| Req 2: ff-vfs Boundary | AC 2 (VFS shall not) | 1.5 |
| Req 2: ff-vfs Boundary | AC 3 (no imports) | 1.5 |
| Req 2: ff-vfs Boundary | AC 4 (VfsProvider trait) | Design doc |
| Req 2: ff-vfs Boundary | AC 5 (registration direction) | Design doc |
| Req 3: Catalog Boundary | AC 1–2 (catalog owns/does not) | Design doc, 2.1 |
| Req 3: Catalog Boundary | AC 3 (no upstream imports) | 1.6 |
| Req 3: Catalog Boundary | AC 4 (public API) | 2.1–2.5 |
| Req 3: Catalog Boundary | AC 5 (exclusive writes) | 5.3, 5.4 |
| Req 3: Catalog Boundary | AC 6 (LISTCAT ownership) | 6.2 |
| Req 4: Allocator Boundary | AC 1–2 (allocator owns/does not) | Design doc |
| Req 4: Allocator Boundary | AC 3 (catalog-only access) | 5.1, 5.3 |
| Req 4: Allocator Boundary | AC 4–6 (delegation rules) | 7.1, 7.2 |
| Req 4: Allocator Boundary | AC 7 (trait interface) | 5.1 |
| Req 5: VSAM Boundary | AC 1–3 (VSAM owns/does not) | Design doc, 1.7 |
| Req 5: VSAM Boundary | AC 4–5 (exclusive operations) | 3.1–3.5 |
| Req 5: VSAM Boundary | AC 6 (catalog metadata) | Design doc |
| Req 5: VSAM Boundary | AC 7 (VFS provider) | Design doc |
| Req 6: IDCAMS Boundary | AC 1–2 (owns/does not) | Design doc, 8.1 |
| Req 6: IDCAMS Boundary | AC 3 (no catalog DB) | 1.9, 5.4 |
| Req 6: IDCAMS Boundary | AC 4–8 (delegation rules) | 8.1, 8.2 |
| Req 6: IDCAMS Boundary | AC 9 (trait interfaces) | 5.2 |
| Req 7: Dependency Direction | AC 1 (permitted direction) | Design doc |
| Req 7: Dependency Direction | AC 2 (VFS universal) | Design doc |
| Req 7: Dependency Direction | AC 3 (prohibited list) | 1.5–1.9 |
| Req 7: Dependency Direction | AC 4 (CI detection) | 10.1, 10.2 |
| Req 7: Dependency Direction | AC 5 (allocator not depends idcams) | 1.8 |
| Req 7: Dependency Direction | AC 6 (trait indirection) | 2.1–2.6, 3.1–3.6, 4.1–4.5 |
| Req 8: IDCAMS Catalog Conflict | AC 1–6 (delegation rewrite) | 8.1, 8.2 |
| Req 9: IDCAMS VSAM Conflict | AC 1–7 (orchestration rewrite) | 8.1, 8.2 |
| Req 10: Catalog Allocation Conflict | AC 1–6 (boundary clarification) | 6.1, 7.1, 7.2 |
| Req 11: LISTCAT Ownership | AC 1–6 (dual command clarification) | 6.2 |
| Req 12: Allocator Catalog Refs | AC 1–5 (API-only access) | 7.1, 7.2, 5.1, 5.3 |
| Req 13: Dataset Lifecycle | AC 1–6 (ownership per stage) | 9.1, 9.2 |
| Req 14: Resolution Lifecycle | AC 1–5 (ownership per stage) | 9.3 |
| Req 15: Catalog API Contract | AC 1–7 (trait definition) | 2.1–2.6 |
| Req 16: VSAM API Contract | AC 1–7 (trait definition) | 3.1–3.6 |
| Req 17: Allocator API Contract | AC 1–7 (trait definition) | 4.1–4.5 |
| Req 18: Compliance Verification | AC 1 (CI dependency check) | 1.1–1.10, 10.1 |
| Req 18: Compliance Verification | AC 2 (fitness function) | 1.5–1.9 |
| Req 18: Compliance Verification | AC 3 (new crate update) | 11.1 |
| Req 18: Compliance Verification | AC 4 (trait mock compilation) | 5.1–5.4 |
| Req 18: Compliance Verification | AC 5 (cargo test invocation) | 10.1, 10.2 |
| Req 19: Spec Alignment | AC 1 (IDCAMS spec update) | 8.1–8.3 |
| Req 19: Spec Alignment | AC 2 (catalog spec update) | 6.1–6.3 |
| Req 19: Spec Alignment | AC 3 (allocator spec update) | 7.1–7.3 |
| Req 19: Spec Alignment | AC 4 (VFS no change) | Design doc |
| Req 19: Spec Alignment | AC 5 (cross-references) | 6.3, 7.3, 8.3 |
| Req 19: Spec Alignment | AC 6 (incremental updates) | All tasks independent |
| Req 20: Future Extensibility | AC 1 (ADR amendment process) | 11.1, 11.2 |
| Req 20: Future Extensibility | AC 2 (trait integration) | Design doc |
| Req 20: Future Extensibility | AC 3 (DAG preservation) | Design doc |
| Req 20: Future Extensibility | AC 4 (fitness function extension) | 10.3, 11.1 |
| Req 20: Future Extensibility | AC 5 (API extension process) | 11.1 |

---

## Task Dependency Graph

```json
{
  "waves": [
    {
      "id": 0,
      "label": "Architectural compliance test infrastructure",
      "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7", "1.8", "1.9", "1.10"]
    },
    {
      "id": 1,
      "label": "Trait interface definitions (CatalogService, VsamService, AllocatorService)",
      "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "4.1", "4.2", "4.3", "4.4", "4.5"],
      "dependsOn": [0]
    },
    {
      "id": 2,
      "label": "Mock compilation tests",
      "tasks": ["5.1", "5.2", "5.3", "5.4"],
      "dependsOn": [1]
    },
    {
      "id": 3,
      "label": "Subsystem specification alignment",
      "tasks": ["6.1", "6.2", "6.3", "7.1", "7.2", "7.3", "8.1", "8.2", "8.3"],
      "dependsOn": [1]
    },
    {
      "id": 4,
      "label": "Lifecycle documentation",
      "tasks": ["9.1", "9.2", "9.3"],
      "dependsOn": [3]
    },
    {
      "id": 5,
      "label": "CI pipeline integration and extensibility docs",
      "tasks": ["10.1", "10.2", "10.3", "11.1", "11.2"],
      "dependsOn": [2]
    }
  ]
}
```

---

## Notes

- Tasks in Wave 0 (compliance tests) can proceed immediately — they only need to parse Cargo.toml files, which exist for all current crates
- Tasks in Wave 1 (trait definitions) require the owning crates to exist as stubs — `ff-dataset-catalog` and `ff-dataset-allocator` already exist; `ff-vsam-services` may need to be created as a stub crate first
- Wave 2 and 3 are independent and can proceed in parallel once Wave 1 is complete
- Specification alignment (Wave 3) is documentation-only work — no code changes to existing implementations
- The architectural fitness function (Wave 0) should be run in CI from the start, even before all crates exist — tests for non-existent crates can be `#[ignore]`-ed with a note
