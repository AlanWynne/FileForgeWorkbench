# Implementation Plan: Virtual File System (`ff-vfs`)

## Overview

This task plan implements the `ff-vfs` crate — the Virtual File System abstraction layer for FileForgeWorkbench (FFW-ARCH-001). All file and resource access throughout the workbench flows through this crate. The implementation proceeds from foundational types (URI, errors) through the provider trait, registry, facade, and finally watch/search capabilities.

**Crate location:** `crates/ff-vfs`
**Upstream dependencies:** `ff-logging` (Wave 0), `ff-core` (Wave 2)
**Downstream consumers:** `connector-local-fs`, `connector-extensibility`, `dataset-catalog`, `document-model`, `file-operations`, `background-io`

---

## Tasks

- [x] 1. Project scaffold and error types
  - [x] 1.1 Create `crates/ff-vfs/Cargo.toml` with dependencies (tokio, async-trait, thiserror, tokio-util, pin-project-lite) and dev-dependencies (proptest, tokio-test, pretty_assertions, tempfile)
  - [x] 1.2 Create `crates/ff-vfs/src/lib.rs` with crate-level doc comment and public module declarations
  - [x] 1.3 Implement `src/error.rs` — define `VfsError` enum with all variants (NotFound, PermissionDenied, AlreadyExists, NotADirectory, UnsupportedOperation, InvalidUri, ProviderUnavailable, DuplicateScheme, Io, Timeout) including Display format `[vfs] operation: description`
  - [x] 1.4 Write unit tests for `VfsError` Display output format compliance (all variants start with `[vfs]`, include operation and URI/scheme context, ≤200 chars)
    - Validates: Requirement 1 AC 4, AC 5; Cross-cutting Req 8

- [x] 2. ResourceUri type and parsing
  - [x] 2.1 Implement `src/uri.rs` — define `ResourceUri` struct with `scheme`, `path`, `query` fields; derive Clone, Eq, Hash, Debug
  - [x] 2.2 Implement `ResourceUri::parse()` — validate `vfs://` prefix, non-empty provider (alphanumeric/hyphen/underscore only), non-empty path; return `VfsError::InvalidUri` on failure
  - [x] 2.3 Implement `ResourceUri::new()`, `ResourceUri::with_query()`, `ResourceUri::from_path()` constructors — `from_path` defaults to scheme `"local"`
  - [x] 2.4 Implement `Display` (canonical URI string), `FromStr` (delegates to parse), accessor methods (`scheme()`, `path()`, `query()`, `as_str()`)
  - [x] 2.5 Write unit tests for URI construction, parsing valid/invalid inputs, component extraction, and bare path default provider
    - Validates: Requirement 2 AC 1–10
  - [x] 2.6 Write property test: URI round-trip (Property 1) — serialize via Display, parse back via FromStr, assert equality
    - Validates: Requirements 2.3, 2.9
  - [x] 2.7 Write property test: URI validation rejects invalid inputs (Property 2) — generate invalid strings, assert VfsError::InvalidUri
    - Validates: Requirements 2.4, 2.5
  - [x] 2.8 Write property test: bare path default provider (Property 6) — generate paths without `vfs://` prefix, assert scheme == "local"
    - Validates: Requirements 2.10, 3.8

- [x] 3. Core data types
  - [x] 3.1 Implement `src/types.rs` — define `EntryType` enum (File, Directory, Symlink, Other with `#[non_exhaustive]`), `VfsMetadata` struct, `VfsEntry` struct, `WriteMode` enum, `OpenOptions`, `CreateOptions`, `DeleteOptions`
  - [x] 3.2 Implement `VfsCapabilities` struct with boolean fields (read, write, watch, search, random_access, append, rename, delete, list, create_directory)
  - [x] 3.3 Write unit tests for types: default values, Debug output, Clone/PartialEq behaviour
    - Validates: Requirement 4 AC 4; Requirement 6 AC 4, AC 6

- [x] 4. VfsProvider trait and VfsFile trait
  - [x] 4.1 Implement `src/provider.rs` — define `VfsProvider` trait with `#[async_trait]`, all required methods (`scheme`, `capabilities`, `open`, `read`, `read_stream`, `write`, `create`, `delete`, `rename`, `list`, `stat`, `exists`) and optional methods with default implementations (`watch`, `search`)
  - [x] 4.2 Define `VfsFile` trait with `#[async_trait]` — `read`, `write`, `flush`, `sync_all`, `close` methods
  - [x] 4.3 Verify object safety: confirm `dyn VfsProvider` and `dyn VfsFile` compile with Send + Sync bounds
  - [x] 4.4 Write unit tests verifying trait object construction compiles and default method implementations return UnsupportedOperation
    - Validates: Requirement 4 AC 1–3, AC 6, AC 8–10; Requirement 5 AC 1–3

- [x] 5. Provider Registry
  - [x] 5.1 Implement `src/registry.rs` — define `ProviderRegistry` struct with `Arc<RwLock<HashMap<String, Arc<dyn VfsProvider>>>>` and `default_scheme` field
  - [x] 5.2 Implement `register()` — validate scheme uniqueness, insert provider, log at INFO level; return `VfsError::DuplicateScheme` on conflict
  - [x] 5.3 Implement `deregister()` — remove provider by scheme, return error if not found
  - [x] 5.4 Implement `get()` — read-lock lookup by scheme, return `Option<Arc<dyn VfsProvider>>`
  - [x] 5.5 Implement `list_schemes()`, `list_providers()`, `default_scheme()`, `has_default_provider()`
  - [x] 5.6 Write unit tests for register/deregister/lookup lifecycle, duplicate detection, thread-safety (spawn multiple tokio tasks)
    - Validates: Requirement 3 AC 1–10
  - [x] 5.7 Write property test: registry uniqueness (Property 3) — register N providers with unique schemes, then attempt duplicate
    - Validates: Requirements 3.2, 3.3
  - [x] 5.8 Write property test: routing correctness (Property 4) — registered schemes route correctly, unregistered return ProviderUnavailable
    - Validates: Requirements 3.5, 3.6
  - [x] 5.9 Write property test: deregistration completeness (Property 10) — after deregister, get() returns None
    - Validates: Requirement 3.10

- [x] 6. Vfs facade — file operations
  - [x] 6.1 Implement `src/vfs.rs` — define `Vfs` struct wrapping `ProviderRegistry`, implement `new()` and `registry()` accessor
  - [x] 6.2 Implement routing logic — extract scheme from URI, lookup provider, delegate to provider method; return ProviderUnavailable on missing scheme
  - [x] 6.3 Implement `open()`, `read()`, `read_stream()`, `write()`, `delete()` — delegate to provider after routing
  - [x] 6.4 Implement `rename()` — validate same-provider constraint, return UnsupportedOperation for cross-provider rename
  - [x] 6.5 Implement `copy()` — cross-provider streaming copy: read_stream from source, write to destination
  - [x] 6.6 Write unit tests with a mock provider: read/write round-trip, delete, rename same-provider, rename cross-provider rejection, copy
    - Validates: Requirement 5 AC 1–10
  - [x] 6.7 Write property test: cross-provider rename rejection (Property 7) — generate URI pairs with different schemes, assert UnsupportedOperation
    - Validates: Requirement 5.6

- [x] 7. Vfs facade — directory and container operations
  - [x] 7.1 Implement `list()`, `create_dir()`, `stat()`, `exists()` on `Vfs` — delegate to provider after routing
  - [x] 7.2 Write unit tests with mock provider: list returns VfsEntry vec, create_dir with parents, stat returns VfsMetadata, exists true/false, list on non-directory returns NotADirectory, list on missing returns NotFound
    - Validates: Requirement 6 AC 1–8

- [x] 8. File watching
  - [x] 8.1 Implement `src/watch.rs` — define `WatchHandle` (mpsc receiver + CancellationToken + URI), `WatchEvent` enum, `WatchOptions` struct
  - [x] 8.2 Implement `WatchHandle::recv()` — async receive from mpsc channel
  - [x] 8.3 Implement `WatchHandle::cancel()` — trigger CancellationToken, drop resources
  - [x] 8.4 Implement watch support on `Vfs` facade — delegate to provider, return UnsupportedOperation if provider lacks watch capability
  - [x] 8.5 Write unit tests: event delivery via channel, cancel stops delivery, unsupported provider returns error, debounce configuration accepted
    - Validates: Requirement 7 AC 1–8
  - [x] 8.6 Write property test: watch event types are exhaustive (Property 8) — generate WatchEvents, assert all carry valid ResourceUri
    - Validates: Requirement 7.2

- [x] 9. Content and filename search
  - [x] 9.1 Implement `src/search.rs` — define `SearchQuery` enum, `SearchOptions` struct, `VfsSearchResult` struct
  - [x] 9.2 Implement search fallback logic — enumerate via `list`, read via `read_stream`, match content line-by-line, emit results as async stream; respect CancellationToken
  - [x] 9.3 Implement `Vfs::search()` — delegate to provider native search if supported, otherwise use fallback
  - [x] 9.4 Write unit tests with mock provider: native search delegation, fallback enumeration search, cancellation stops results, options (case sensitivity, whole word, max results) applied
    - Validates: Requirement 8 AC 1–8

- [x] 10. Subsystem integration with ff-core
  - [x] 10.1 Implement `src/subsystem.rs` — define `VfsSubsystem` struct implementing ff-core `Subsystem` trait
  - [x] 10.2 Implement `descriptor()` — return name "vfs", criticality Critical, order Vfs (third in startup)
  - [x] 10.3 Implement `initialize()` — create ProviderRegistry, instantiate Vfs, register with ServiceRegistry
  - [x] 10.4 Implement `shutdown()` — cancel all active watches, deregister all providers, log shutdown
  - [x] 10.5 Write integration test: VfsSubsystem initializes, registers Vfs with ServiceRegistry, shuts down cleanly
    - Validates: Cross-cutting Req 6 AC 1; Integration with ff-core

- [x] 11. Error format compliance property test
  - [x] 11.1 Write property test: error format compliance (Property 9) — generate all VfsError variants, assert Display starts with `[vfs]` and length ≤ 200
    - Validates: Requirements 1.4, 1.5; Cross-cutting Req 8

- [x] 12. Capability-gated operations property test
  - [x] 12.1 Write property test: capability-gated operations (Property 5) — create mock providers with various capability sets, invoke unsupported operations, assert UnsupportedOperation
    - Validates: Requirements 4.4, 4.5

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: VFS Abstraction Layer | AC 1 (sole API) | Architecture enforcement (design constraint) |
| Req 1: VFS Abstraction Layer | AC 2 (provider-agnostic) | 6.1–6.6, 7.1 |
| Req 1: VFS Abstraction Layer | AC 3 (async methods) | 4.1, 6.1–6.5 |
| Req 1: VFS Abstraction Layer | AC 4 (VfsError enum) | 1.3, 1.4 |
| Req 1: VFS Abstraction Layer | AC 5 (error context) | 1.3, 1.4, 11.1 |
| Req 1: VFS Abstraction Layer | AC 6 (no provider types exposed) | Architecture enforcement (pub API review) |
| Req 2: Resource URI Scheme | AC 1–10 | 2.1–2.8 |
| Req 3: Provider Registry | AC 1–10 | 5.1–5.9 |
| Req 4: VfsProvider Trait | AC 1–10 | 4.1–4.4, 12.1 |
| Req 5: File Operations | AC 1–10 | 6.1–6.7 |
| Req 6: Directory Operations | AC 1–8 | 3.1, 7.1–7.2 |
| Req 7: File Watching | AC 1–8 | 8.1–8.6 |
| Req 8: Search | AC 1–8 | 9.1–9.4 |

---

## Property-Based Test Summary

| Property | Statement | Task | Validates |
|----------|-----------|------|-----------|
| P1 | URI round-trip: Display → FromStr → equal | 2.6 | Req 2.3, 2.9 |
| P2 | URI validation rejects invalid inputs | 2.7 | Req 2.4, 2.5 |
| P3 | Registry uniqueness: duplicate scheme rejected | 5.7 | Req 3.2, 3.3 |
| P4 | Registry routing: registered → routes; unregistered → error | 5.8 | Req 3.5, 3.6 |
| P5 | Capability-gated: unsupported op → UnsupportedOperation | 12.1 | Req 4.4, 4.5 |
| P6 | Bare path default provider: no prefix → scheme "local" | 2.8 | Req 2.10, 3.8 |
| P7 | Cross-provider rename rejected | 6.7 | Req 5.6 |
| P8 | Watch events carry valid URIs | 8.6 | Req 7.2 |
| P9 | Error format: starts with `[vfs]`, ≤200 chars | 11.1 | Req 1.4, 1.5 |
| P10 | Deregistration: after removal, get() → None | 5.9 | Req 3.10 |

---

## Notes

- Tasks 2 and 3 can be implemented in parallel since they are independent (both depend only on task 1)
- Task 11 (error format property test) can run as soon as task 1 is complete — it validates only the error module
- Task 12 (capability-gated property test) requires the provider trait and a mock provider
- All property tests use the `proptest` crate with a minimum of 100 iterations
- The mock provider used in tests should be defined in a `tests/mock_provider.rs` helper shared across integration test files
- `ff-core` types (`Subsystem`, `ServiceRegistry`) are consumed via trait bounds — task 10 is the only task tightly coupled to ff-core internals
- All async tests use `#[tokio::test]` with multi-threaded runtime flavour

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Project scaffold and error types", "tasks": ["1.1", "1.2", "1.3", "1.4"] },
    { "id": 1, "label": "ResourceUri and core data types", "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7", "2.8", "3.1", "3.2", "3.3"], "dependsOn": [0] },
    { "id": 2, "label": "VfsProvider trait and VfsFile trait", "tasks": ["4.1", "4.2", "4.3", "4.4"], "dependsOn": [1] },
    { "id": 3, "label": "Provider Registry", "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8", "5.9"], "dependsOn": [2] },
    { "id": 4, "label": "Vfs facade — file and directory operations", "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "6.7", "7.1", "7.2"], "dependsOn": [3] },
    { "id": 5, "label": "File watching and search", "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "9.1", "9.2", "9.3", "9.4"], "dependsOn": [4] },
    { "id": 6, "label": "Subsystem integration and standalone property tests", "tasks": ["10.1", "10.2", "10.3", "10.4", "10.5", "11.1", "12.1"], "dependsOn": [5] }
  ]
}
```
