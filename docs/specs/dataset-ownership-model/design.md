# Design Document: Dataset Ownership Model (Governance Infrastructure)

## Overview

The Dataset Ownership Model is a **cross-cutting architectural governance specification** that establishes single-authority ownership boundaries, interface contracts, and dependency rules for all dataset-related subsystems in FileForgeWorkbench. Unlike implementation crates, this specification does not produce a standalone `ff-*` crate. Instead, it produces:

1. **Shared trait definitions** — Interface contracts (`CatalogService`, `VsamService`, `AllocatorService`) that live in the owning crate's public API and are consumed by dependent crates via trait bounds.
2. **Architectural fitness tests** — A `tests/architecture_compliance.rs` integration test suite that verifies dependency direction, ownership boundaries, and trait-based coupling at compile time and CI time.
3. **CI pipeline checks** — A `cargo`-based dependency direction checker that fails the build when prohibited dependencies are introduced.
4. **Specification alignment tracking** — A checklist tracking which subsystem specs have been updated to conform to this governance document.

### Position in Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│              Governance Layer (this document)                             │
│   Ownership rules, dependency direction, interface contracts             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────┐    ┌─────────────────────┐    ┌──────────────────┐   │
│  │  ff-idcams   │───▶│ ff-dataset-allocator │───▶│ ff-dataset-catalog│   │
│  │  (parsing +  │    │ (JCL allocation      │    │ (catalog CRUD,    │   │
│  │  orchestrate)│    │  workflow)            │    │  resolution, GDG) │   │
│  └──────┬───────┘    └──────────┬───────────┘    └────────┬──────────┘   │
│         │                       │                          │              │
│         ▼                       ▼                          ▼              │
│  ┌──────────────┐    ┌──────────────────────────────────────────────┐   │
│  │ff-vsam-svc   │    │              ff-vfs (abstraction only)        │   │
│  │(record-level)│    │    Universal infrastructure — all may depend   │   │
│  └──────────────┘    └──────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘

Arrows indicate permitted dependency direction (left → right).
Reverse arrows are PROHIBITED.
```

### Design Constraints

- **No new crate produced**: This governance document produces tests, CI scripts, and trait alignment — not a standalone binary or library.
- **Trait ownership stays with the owning crate**: `CatalogService` is defined in `ff-dataset-catalog`, `VsamService` in `ff-vsam-services`, `AllocatorService` in `ff-dataset-allocator`. Dependent crates import the trait from the owning crate.
- **Compile-time enforcement**: Prohibited dependencies are enforced by `Cargo.toml` declarations — if a crate doesn't list a prohibited dependency, Rust's module system prevents use. The fitness test verifies this declaratively.
- **Object-safe trait wrappers**: Each trait has both an ergonomic generic version and an object-safe `Dyn*` wrapper for dynamic dispatch and mocking.

---

## Architecture

### Dependency DAG

The permitted dependency graph forms a strict DAG (Directed Acyclic Graph):

```
ff-idcams
    ├── ff-dataset-allocator (via AllocatorService trait)
    ├── ff-dataset-catalog (via CatalogService trait)
    ├── ff-vsam-services (via VsamService trait)
    └── ff-vfs (via VfsProvider for content I/O)

ff-dataset-allocator
    ├── ff-dataset-catalog (via CatalogService trait)
    └── ff-vfs (for resource access abstraction)

ff-dataset-catalog
    ├── ff-vfs (implements VfsProvider trait)
    └── ff-vsam-services (for VSAM dataset initialization — optional)

ff-vsam-services
    ├── ff-vfs (implements VfsProvider under scheme "vsam")
    └── (no domain crate dependencies)

ff-vfs
    └── (no domain crate dependencies — pure abstraction)
```

### Prohibited Dependencies (Must Never Appear)

| Crate | SHALL NOT depend on |
|-------|---------------------|
| `ff-vfs` | `ff-idcams`, `ff-dataset-catalog`, `ff-dataset-allocator`, `ff-vsam-services` |
| `ff-dataset-catalog` | `ff-idcams`, `ff-dataset-allocator` |
| `ff-vsam-services` | `ff-idcams`, `ff-dataset-allocator` |
| `ff-dataset-allocator` | `ff-idcams` |

### Interface Contract Design

Each ownership boundary is expressed as a Rust trait. The pattern is:

```rust
// In the OWNING crate's public API (e.g., ff-dataset-catalog/src/traits.rs)

/// The primary interface for catalog operations.
/// Ergonomic generics for production code.
pub trait CatalogService {
    type Error: std::error::Error + Send + Sync + 'static;

    fn create_dataset(&self, dsn: &str, attrs: DatasetAttributes) -> Result<DatasetId, Self::Error>;
    fn delete_dataset(&self, dsn: &str) -> Result<(), Self::Error>;
    fn update_dataset(&self, dsn: &str, attrs: DatasetAttributes) -> Result<(), Self::Error>;
    fn rename_dataset(&self, old_dsn: &str, new_dsn: &str) -> Result<(), Self::Error>;
    fn resolve_dsn(&self, dsn: &str) -> Result<ResolutionResult, Self::Error>;
    fn dataset_exists(&self, dsn: &str) -> Result<bool, Self::Error>;
    fn get_dataset_attributes(&self, dsn: &str) -> Result<DatasetAttributes, Self::Error>;
    fn list_datasets(&self, filter: &DatasetFilter) -> Result<Vec<DatasetEntry>, Self::Error>;
    fn validate_dsn(&self, dsn: &str) -> Result<(), DsnValidationError>;
    fn create_gdg_base(&self, dsn: &str, limit: u8, scratch: bool) -> Result<(), Self::Error>;
    fn create_generation(&self, base_dsn: &str, attrs: DatasetAttributes) -> Result<GenerationInfo, Self::Error>;
    fn resolve_generation(&self, base_dsn: &str, offset: i32) -> Result<GenerationInfo, Self::Error>;
    fn list_generations(&self, base_dsn: &str) -> Result<Vec<GenerationInfo>, Self::Error>;
    fn get_allocation_defaults(&self, dsorg: Dsorg) -> DatasetAttributes;
}

/// Object-safe wrapper for dynamic dispatch and mock injection.
pub trait DynCatalogService: Send + Sync {
    fn create_dataset(&self, dsn: &str, attrs: DatasetAttributes) -> Result<DatasetId, CatalogError>;
    fn delete_dataset(&self, dsn: &str) -> Result<(), CatalogError>;
    // ... (concrete error type for object safety)
}

// Blanket impl: any CatalogService with Error=CatalogError auto-implements DynCatalogService
impl<T: CatalogService<Error = CatalogError> + Send + Sync> DynCatalogService for T { ... }
```

### Architectural Fitness Function Design

The fitness function is an integration test that:

1. **Parses `Cargo.toml` files** — Reads all workspace member `Cargo.toml` files to extract `[dependencies]` sections.
2. **Builds a dependency matrix** — For each dataset-related crate, records what it depends on.
3. **Checks against prohibition rules** — Verifies that no prohibited dependency exists.
4. **Checks trait-based coupling** — Verifies that dependent crates compile with mock trait implementations (no concrete-type coupling).
5. **Reports violations** — Produces a clear error message identifying the violating crate and the prohibited dependency.

```rust
// tests/architecture_compliance.rs (in workspace root or a dedicated test crate)

#[test]
fn vfs_has_no_domain_dependencies() {
    let cargo_toml = parse_cargo_toml("crates/ff-vfs/Cargo.toml");
    let deps = cargo_toml.dependencies();
    assert!(!deps.contains_key("ff-idcams"), "ff-vfs must not depend on ff-idcams");
    assert!(!deps.contains_key("ff-dataset-catalog"), "ff-vfs must not depend on ff-dataset-catalog");
    assert!(!deps.contains_key("ff-dataset-allocator"), "ff-vfs must not depend on ff-dataset-allocator");
    assert!(!deps.contains_key("ff-vsam-services"), "ff-vfs must not depend on ff-vsam-services");
}

#[test]
fn dataset_catalog_has_no_upstream_dependencies() {
    let cargo_toml = parse_cargo_toml("crates/ff-dataset-catalog/Cargo.toml");
    let deps = cargo_toml.dependencies();
    assert!(!deps.contains_key("ff-idcams"), "ff-dataset-catalog must not depend on ff-idcams");
    assert!(!deps.contains_key("ff-dataset-allocator"), "ff-dataset-catalog must not depend on ff-dataset-allocator");
}

#[test]
fn vsam_services_has_no_upstream_dependencies() {
    let cargo_toml = parse_cargo_toml("crates/ff-vsam-services/Cargo.toml");
    let deps = cargo_toml.dependencies();
    assert!(!deps.contains_key("ff-idcams"), "ff-vsam-services must not depend on ff-idcams");
    assert!(!deps.contains_key("ff-dataset-allocator"), "ff-vsam-services must not depend on ff-dataset-allocator");
}

#[test]
fn dataset_allocator_has_no_idcams_dependency() {
    let cargo_toml = parse_cargo_toml("crates/ff-dataset-allocator/Cargo.toml");
    let deps = cargo_toml.dependencies();
    assert!(!deps.contains_key("ff-idcams"), "ff-dataset-allocator must not depend on ff-idcams");
}

#[test]
fn idcams_has_no_storage_engine_dependencies() {
    let cargo_toml = parse_cargo_toml("crates/ff-idcams/Cargo.toml");
    let all_deps = cargo_tree_transitive("ff-idcams");
    assert!(!all_deps.contains("rusqlite"), "ff-idcams must not transitively depend on rusqlite");
    assert!(!all_deps.contains("rocksdb"), "ff-idcams must not transitively depend on rocksdb");
    assert!(!all_deps.contains("lmdb"), "ff-idcams must not transitively depend on lmdb");
}
```

### Mock Trait Compilation Tests

These tests verify that dependent crates can compile against mock implementations, proving no concrete-type coupling:

```rust
// In ff-dataset-allocator's test suite
#[test]
fn allocator_compiles_with_mock_catalog_service() {
    struct MockCatalog;
    impl CatalogService for MockCatalog {
        type Error = CatalogError;
        fn resolve_dsn(&self, _dsn: &str) -> Result<ResolutionResult, CatalogError> {
            Ok(ResolutionResult::default())
        }
        // ... all methods return default/stub values
    }
    
    let config = ResolverConfig::default();
    let resolver = Resolver::new(&MockCatalog, &config);
    // If this compiles, trait-based coupling is verified
}
```

### Specification Alignment Tracking

A tracking matrix records which subsystem specs have been updated:

| Subsystem Spec | Req 19 Alignment Status | Notes |
|----------------|------------------------|-------|
| `ff-idcams` (idcams-emulator) | ✅ Aligned | Delegation model explicit in Reqs 2-14, Ownership boundary in Req 21 |
| `ff-dataset-catalog` (dataset-catalog) | ⚠️ Needs clarification note | Req 7 needs note that JCL allocation workflows are owned by allocator |
| `ff-dataset-allocator` (dataset-allocator) | ✅ Aligned | Req 2 AC 8 already states catalog-only API access |
| `ff-vfs` (virtual-file-system) | ✅ No changes needed | Correctly owns only abstraction layer |
| `ff-vsam-services` | 🔴 Not yet created | Future crate — trait interface defined in this governance doc |

---

## Components

### Architectural Compliance Test Suite

**Location:** `tests/architecture_compliance.rs` (workspace-level integration test)

**Responsibilities:**
- Parse all workspace `Cargo.toml` files
- Extract dependency lists (direct and transitive via `cargo tree`)
- Verify prohibition rules from the dependency matrix
- Verify no storage-engine transitive dependencies in ff-idcams
- Report clear violation messages with fix instructions

### Trait Interface Definitions

Each owning crate defines its public trait. The governance document specifies the method signatures and error types that each trait MUST provide:

| Trait | Owning Crate | Methods |
|-------|-------------|---------|
| `CatalogService` | `ff-dataset-catalog` | create_dataset, delete_dataset, update_dataset, rename_dataset, resolve_dsn, dataset_exists, get_dataset_attributes, list_datasets, validate_dsn, create_gdg_base, create_generation, resolve_generation, list_generations, get_allocation_defaults |
| `VsamService` | `ff-vsam-services` | initialize_dataset, destroy_dataset, define_aix, define_path, delete_path, verify_integrity, build_index, open, start_browse, next_record, put, delete, close |
| `AllocatorService` | `ff-dataset-allocator` | resolve_dd, resolve_job, resolve_dsn, substitute_symbols |

### CI Pipeline Integration

**Script:** A CI step that runs `cargo test --test architecture_compliance` as part of the standard PR checks. Any violation causes build failure with a clear message pointing to the offending `Cargo.toml` and the governance rule being violated.

---

## Data Models

### DependencyRule

```rust
/// A single dependency direction rule.
pub struct DependencyRule {
    /// The crate that must NOT depend on the target
    pub crate_name: String,
    /// The crate that must NOT appear as a dependency
    pub prohibited_dependency: String,
    /// Human-readable reason
    pub reason: String,
    /// Which governance requirement mandates this rule
    pub requirement_ref: String,
}
```

### ComplianceResult

```rust
/// Result of running the architectural fitness function.
pub struct ComplianceResult {
    pub passed: bool,
    pub violations: Vec<ComplianceViolation>,
    pub crates_checked: usize,
    pub rules_checked: usize,
}

pub struct ComplianceViolation {
    pub rule: DependencyRule,
    pub found_in: String, // Cargo.toml path
    pub fix_instruction: String,
}
```

---

## Cross-Cutting Concerns

### Enforcement Timing

- **Compile-time**: Rust's module system prevents use of types from crates not listed in `Cargo.toml`. This is the primary enforcement.
- **CI-time**: The architectural fitness test runs in CI, catching any newly introduced prohibited dependencies before merge.
- **Documentation-time**: This governance document serves as the authoritative reference. Any dispute about ownership is resolved by consulting this document.

### Extensibility

When a new dataset-related crate is proposed:
1. An ADR amendment defines its ownership, prohibited dependencies, and trait interfaces
2. The fitness function test file is extended with new prohibition rules
3. The trait interface section of this design is updated
4. The tracking matrix gets a new row

### Testing Strategy

- The fitness function itself is tested by verifying it catches known-bad `Cargo.toml` configurations (using test fixture files)
- Mock compilation tests prove trait-based coupling for each dependent crate
- No property-based tests needed — governance rules are deterministic checks
