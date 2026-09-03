# Requirements Document

## Introduction

This feature specifies the Virtual File System (VFS) abstraction layer for FileForgeWorkbench — the `ff-vfs` crate. The VFS is the **overriding architectural principle** (FFW-ARCH-001) of the entire platform: all file and resource access throughout the workbench flows through this abstraction layer. No consuming crate ever calls `std::fs` directly or couples to a specific storage backend.

The VFS provides a **unified resource addressing scheme** (`vfs://provider/path`), a **provider registry** for runtime discovery of available storage backends, and a **provider trait** that all backends implement (local filesystem, dataset catalog, future remote connectors). This architecture ensures that providers are interchangeable without modifying consuming code, and that new providers can be added by implementing a single trait and registering with the registry.

The VFS defines **async method signatures** for all I/O operations (Tokio-based), **provider-agnostic file operations** (open, read, write, create, delete, rename, list, stat, exists), **directory/container operations**, **file watching** (capability-based), and **content search** (provider-scoped, async streaming).

**Source references:**
- **WB** = Workbench Platform Architecture Brief — FFW-ARCH-001 overriding connectivity principle
- **DSC** = Dataset Catalog Brief — VFS abstraction requirement for dataset catalog emulation
- **FFE** = FileForgeEditor specifications — file operation patterns adapted to VFS

## Glossary

- **VFS**: Virtual File System — the abstraction layer through which all file/resource access flows in the workbench. Implemented by the `ff-vfs` crate. [WB]
- **Resource_URI**: A unified resource identifier in the format `vfs://provider/path` that uniquely identifies any resource regardless of its backing store. [WB]
- **VfsProvider**: The core trait that all storage backends implement. Defines async methods for all I/O operations (open, read, write, create, delete, rename, list, stat, exists). [WB]
- **Provider_Registry**: The central registry that holds all registered VfsProvider instances, supports runtime discovery, and routes URI-based requests to the appropriate provider. [WB]
- **Provider_Scheme**: The unique string identifier (e.g., `local`, `catalog`, `ftp`) that a provider registers with, used as the authority component of Resource_URIs to route operations. [WB]
- **VfsError**: The unified error type that abstracts provider-specific errors into a common set of error variants, enabling consuming code to handle errors without knowledge of the underlying provider. [WB]
- **Capability**: A declared feature that a provider supports (e.g., read-only, read-write, watch, search, random-access). Consumers query capabilities before attempting operations. [WB]
- **Watch_Handle**: A handle returned when subscribing to file change notifications, used to cancel the watch subscription. [WB]
- **Watch_Event**: An event emitted when a watched resource changes — variants include created, modified, deleted, and renamed. [WB]
- **VfsStream**: An async stream of bytes or items returned by read operations and search results. [WB]
- **Default_Provider**: The provider used when a bare path (no URI scheme) is provided — defaults to the local filesystem provider. [WB]

## Requirements

### Requirement 1: VFS Abstraction Layer

**User Story:** As a workbench developer, I want all file and resource access to go through a single abstraction layer, so that my code never couples to a specific storage backend and providers can be swapped or added without modifying consuming crates.

**Source:** WB Architecture Brief — FFW-ARCH-001 overriding connectivity principle. [WB, DSC]

#### Acceptance Criteria

1. THE `ff-vfs` crate SHALL provide the sole public API through which all other `ff-*` crates access file and resource content — no consuming crate SHALL contain direct `std::fs`, `tokio::fs`, or other platform-specific file I/O calls.
2. THE VFS layer SHALL present a provider-agnostic interface: consuming code SHALL interact with resources using the VFS API without knowledge of which provider backs the resource.
3. ALL I/O methods exposed by the VFS public API SHALL be async (returning `Future`s compatible with the Tokio runtime), enabling non-blocking file operations throughout the workbench.
4. THE `ff-vfs` crate SHALL define a unified `VfsError` enum that abstracts provider-specific errors into common variants (not-found, permission-denied, already-exists, I/O error, unsupported-operation, invalid-URI, provider-unavailable, timeout), so that consuming code can handle errors without knowledge of the underlying provider.
5. EACH `VfsError` variant SHALL carry sufficient context to produce a diagnostic message conforming to the project error message standard: `[vfs] operation: description` with the resource URI included.
6. THE VFS layer SHALL NOT expose any provider-specific types, methods, or error variants through its public API — all provider details SHALL be encapsulated behind the `VfsProvider` trait.

---

### Requirement 2: Resource URI Scheme

**User Story:** As a workbench developer, I want a unified addressing scheme that uniquely identifies any resource regardless of its backing store, so that I can reference, persist, and restore resource locations using a single consistent format.

**Source:** WB Architecture Brief — FFW-ARCH-001 unified addressing. [WB, DSC]

#### Acceptance Criteria

1. THE VFS layer SHALL define a Resource_URI format: `vfs://provider/path` where `provider` is the scheme identifier (authority component) and `path` is the provider-specific resource path.
2. A Resource_URI SHALL uniquely identify any resource across all registered providers — no two distinct resources SHALL share the same URI.
3. THE VFS layer SHALL provide a `ResourceUri` type with methods for parsing, validation, construction, and component extraction (scheme, provider, path, query parameters).
4. WHEN a URI string is parsed, THE `ResourceUri` parser SHALL validate that the scheme is `vfs`, the provider component is non-empty and contains only valid identifier characters (alphanumeric, hyphen, underscore), and the path component is non-empty.
5. IF a URI string fails validation, THEN THE parser SHALL return a `VfsError::InvalidUri` variant containing the original string and a description of the validation failure.
6. THE provider component SHALL be extracted from the URI authority segment (the portion between `://` and the first `/` after the authority).
7. THE path component SHALL be provider-specific: for the local filesystem provider it is an OS file path; for the dataset catalog provider it is a dataset name (e.g., `HLQ.QUALIFIER.MEMBER`); for other providers it may follow their own conventions.
8. THE Resource_URI format SHALL support optional query parameters (`?key=value&key2=value2`) for provider-specific options (e.g., `?encoding=ebcdic`, `?recfm=fb`).
9. THE `ResourceUri` type SHALL implement `Display` (producing the canonical URI string), `FromStr` (parsing), `Clone`, `Eq`, `Hash`, and `Debug`.
10. WHEN a bare path is provided (no `vfs://` scheme prefix), THE VFS layer SHALL interpret it as a path relative to the Default_Provider (local filesystem), constructing the full URI as `vfs://local/{path}`.

---

### Requirement 3: Provider Registry

**User Story:** As a workbench developer, I want providers to register themselves at runtime with unique scheme identifiers, so that new storage backends can be added without modifying existing code and the VFS can route requests to the correct provider.

**Source:** WB Architecture Brief — FFW-ARCH-001 provider extensibility. [WB, DSC]

#### Acceptance Criteria

1. THE VFS layer SHALL provide a Provider_Registry that holds references to all registered `VfsProvider` instances, indexed by their scheme identifier.
2. PROVIDERS SHALL register themselves with the Provider_Registry by providing a unique scheme identifier string and a `VfsProvider` trait implementation.
3. WHEN a provider registers with a scheme that is already registered, THE Provider_Registry SHALL return an error and log a WARN-level record — duplicate scheme registration is forbidden.
4. THE Provider_Registry SHALL support runtime discovery: consumers SHALL be able to query the list of all registered provider schemes and their capabilities without knowing at compile time which providers will be available.
5. WHEN a VFS operation is invoked with a Resource_URI, THE Provider_Registry SHALL extract the provider scheme from the URI and look up the corresponding `VfsProvider` instance to route the operation.
6. IF the provider scheme in a URI does not match any registered provider, THEN THE VFS SHALL return a `VfsError::ProviderUnavailable` error containing the scheme name.
7. THE Provider_Registry SHALL be thread-safe: registration and lookup operations SHALL be safe to call from any thread (including Tokio worker threads) without external synchronization.
8. THE Provider_Registry SHALL designate a default provider (scheme `local`) that is used when bare paths without a URI scheme are encountered.
9. IF no provider has been registered for the `local` scheme when the VFS is initialized, THEN THE VFS SHALL return a `VfsError::ProviderUnavailable` error for bare path operations rather than panicking.
10. THE Provider_Registry SHALL support provider deregistration (removal), enabling hot-unloading of providers (e.g., when a connector plugin is deactivated).

---

### Requirement 4: VfsProvider Trait

**User Story:** As a provider developer, I want a well-defined trait that I implement to integrate my storage backend with the VFS, so that my provider is automatically available to all workbench consumers without any per-consumer integration work.

**Source:** WB Architecture Brief — FFW-ARCH-001 provider interface. [WB, DSC]

#### Acceptance Criteria

1. THE `ff-vfs` crate SHALL define a `VfsProvider` trait that all storage backend implementations must implement.
2. THE `VfsProvider` trait SHALL define the following async methods (all returning `Result<T, VfsError>`):
   - `open(path, options) → VfsFile` — open a resource for reading and/or writing
   - `read(path) → Vec<u8>` — read entire resource content into memory
   - `read_stream(path) → impl AsyncRead` — read resource content as an async byte stream
   - `write(path, data) → ()` — write data to a resource (create or overwrite)
   - `create(path, options) → ()` — create a new resource or container
   - `delete(path, options) → ()` — delete a resource or container
   - `rename(old_path, new_path) → ()` — rename/move a resource within the same provider
   - `list(path) → Vec<VfsEntry>` — list directory/container contents
   - `stat(path) → VfsMetadata` — get resource metadata
   - `exists(path) → bool` — check if a resource exists
3. ALL `VfsProvider` trait methods SHALL be async (using `async_trait` or native async trait syntax), returning futures compatible with the Tokio runtime.
4. THE `VfsProvider` trait SHALL define a `capabilities() → VfsCapabilities` method that returns the set of capabilities the provider supports (read, write, watch, search, random-access, append, rename, delete, list).
5. WHEN a consumer invokes an operation that the provider does not support (as declared by capabilities), THE provider SHALL return a `VfsError::UnsupportedOperation` error containing the operation name and provider scheme.
6. THE `VfsProvider` trait SHALL define a `scheme() → &str` method that returns the provider's unique scheme identifier, used for registration with the Provider_Registry.
7. PROVIDER implementations SHALL map their internal/platform-specific errors to `VfsError` variants, ensuring consuming code never encounters provider-specific error types.
8. THE `VfsProvider` trait SHALL be object-safe, enabling dynamic dispatch through `dyn VfsProvider` trait objects stored in the Provider_Registry.
9. THE `VfsProvider` trait SHALL define an optional `watch(path, options) → WatchHandle` method with a default implementation that returns `VfsError::UnsupportedOperation` for providers that do not support file watching.
10. THE `VfsProvider` trait SHALL define an optional `search(path, query) → impl Stream<VfsSearchResult>` method with a default implementation that returns `VfsError::UnsupportedOperation` for providers that do not support native search.

---

### Requirement 5: File Operations (Provider-Agnostic)

**User Story:** As a workbench consumer, I want to open, read, write, save, delete, rename, and copy files through the VFS without knowing or caring which provider backs the resource, so that my code works identically regardless of whether the file is on local disk, in a dataset catalog, or on a future remote server.

**Source:** WB Architecture Brief — FFW-ARCH-001. [WB, DSC, FFE]

#### Acceptance Criteria

1. THE VFS SHALL support opening a resource for reading, returning an async reader (implementing `tokio::io::AsyncRead`) that streams the resource content without loading the entire resource into memory.
2. THE VFS SHALL support opening a resource for writing with the following modes: create (fail if exists), truncate (overwrite existing), append (add to end).
3. THE VFS SHALL support a save operation that writes content, flushes all buffers, and performs an fsync-equivalent operation to guarantee durability, returning only after the data is persisted.
4. THE VFS SHALL support deleting a resource by URI, removing it from the provider's storage.
5. THE VFS SHALL support renaming/moving a resource within the same provider, given old and new paths within that provider's namespace.
6. IF a rename/move is attempted across providers (source and destination URIs have different provider schemes), THEN THE VFS SHALL return a `VfsError::UnsupportedOperation` error with a message indicating cross-provider rename is not supported — consumers must use copy-then-delete.
7. THE VFS SHALL support copying between providers: reading from the source provider's resource and writing to the destination provider's resource, operating as an async streaming copy.
8. WHEN a file operation targets a resource that does not exist, THE VFS SHALL return a `VfsError::NotFound` error containing the Resource_URI.
9. WHEN a file operation is denied due to permissions (read-only provider, filesystem permissions), THE VFS SHALL return a `VfsError::PermissionDenied` error containing the Resource_URI and the attempted operation.
10. ALL file operations SHALL accept a `ResourceUri` as the resource identifier, extracting the provider scheme and routing to the appropriate provider automatically.

---

### Requirement 6: Directory and Container Operations

**User Story:** As a workbench consumer, I want to list, create, and delete directories (or their equivalent in non-filesystem providers) through the VFS, so that I can browse and manage resource hierarchies regardless of the backing store.

**Source:** WB Architecture Brief — FFW-ARCH-001. [WB, DSC]

#### Acceptance Criteria

1. THE VFS SHALL support listing the contents of a directory/container, returning a `Vec<VfsEntry>` where each entry contains the entry name, entry type (file, directory, symlink, or provider-specific type), and basic metadata.
2. THE VFS SHALL support creating a directory/container by URI, creating any intermediate containers as needed (equivalent to `mkdir -p` behaviour).
3. THE VFS SHALL support deleting a directory/container with an option for recursive deletion; if recursive is `false` and the container is non-empty, THE VFS SHALL return an error.
4. THE VFS SHALL support a stat operation that returns a `VfsMetadata` struct containing: resource size in bytes (if applicable), last modified time (as `SystemTime` or equivalent), resource type (file, directory, symlink, other), and provider-specific metadata as a `HashMap<String, String>`.
5. THE VFS SHALL support an exists check that returns `true` if the resource exists at the given URI and `false` otherwise, without raising an error for non-existent resources.
6. THE `VfsEntry` type SHALL include at minimum: name (`String`), entry type (enum: File, Directory, Symlink, Other), size (`Option<u64>`), and modified time (`Option<SystemTime>`).
7. WHEN a list operation targets a URI that does not refer to a container/directory, THE VFS SHALL return a `VfsError::NotADirectory` error.
8. WHEN a list operation targets a non-existent URI, THE VFS SHALL return a `VfsError::NotFound` error.

---

### Requirement 7: File Watching

**User Story:** As a workbench consumer, I want to watch resources for changes (creation, modification, deletion, rename), so that I can react to external modifications (e.g., reload a file that changed on disk) without polling.

**Source:** WB Architecture Brief — FFW-ARCH-001 file-watcher. [WB, FFE]

#### Acceptance Criteria

1. THE VFS SHALL support watching a resource or directory for changes, returning a `WatchHandle` that the consumer uses to receive events and cancel the watch.
2. THE VFS SHALL define the following watch event types: `Created` (new resource appeared), `Modified` (content changed), `Deleted` (resource removed), `Renamed { old_uri, new_uri }` (resource moved/renamed).
3. WATCH events SHALL be delivered as an async stream (via `tokio::sync::mpsc` channel or equivalent) attached to the `WatchHandle`, enabling consumers to `await` the next event.
4. THE `WatchHandle` SHALL provide a `cancel()` method that stops event delivery and releases all resources associated with the watch subscription.
5. THE VFS SHALL support debounce configuration: consumers SHALL be able to specify a minimum interval between consecutive events for the same resource (default: 100ms), collapsing rapid successive changes into a single event.
6. WHEN a watch is requested on a provider that does not support file watching (as declared by its capabilities), THE VFS SHALL return a `VfsError::UnsupportedOperation` error indicating that the provider does not support the `watch` capability.
7. IF a watched resource is deleted, THE VFS SHALL emit a `Deleted` event and automatically cancel the watch for that resource (no further events are emitted).
8. THE VFS SHALL support watching an entire directory/container for changes to any resource within it (recursive watch), where the provider supports it.

---

### Requirement 8: Search

**User Story:** As a workbench consumer, I want to search for files by name/pattern and search file contents within a provider scope, so that I can implement find-in-files and file-lookup features generically across all providers.

**Source:** WB Architecture Brief — FFW-ARCH-001. [WB, FFE]

#### Acceptance Criteria

1. THE VFS SHALL support content search within a provider scope: given a root URI, a search pattern (text or regex), and options, it SHALL return matching results as an async stream.
2. THE VFS SHALL support filename/pattern search within a provider scope: given a root URI and a glob or regex pattern, it SHALL return matching resource URIs as an async stream.
3. SEARCH results SHALL be delivered as an async `Stream<Item = VfsSearchResult>` where each result contains: the matching resource URI, the match location (line number, column, byte offset as applicable), and a context snippet (the matching line or surrounding text).
4. THE VFS SHALL support search cancellation: the consumer SHALL be able to drop or cancel the search stream at any time, causing the provider to stop searching and release resources.
5. WHEN a provider implements native search (e.g., indexed search, database full-text search), THE VFS SHALL delegate to the provider's native implementation for performance.
6. WHEN a provider does NOT implement native search, THE VFS SHALL provide a fallback implementation that enumerates resources (via `list`) and searches content (via `read_stream`) — this fallback SHALL be async and cancellable.
7. SEARCH operations SHALL accept options including: case sensitivity (bool), whole word (bool), regex mode (bool), max results (limit), file pattern filter (include/exclude globs), and max file size to search.
8. WHEN a search is requested on a provider scope that does not exist, THE VFS SHALL return a `VfsError::NotFound` error for the root URI.


---

## Requirements Added by CR-NR-016 — Mainframe Dataset Architecture

> **Source documents:** `docs/FileForgeWorkbench_Mainframe_Dataset_Architecture.md` and
> `docs/FileForgeWorkbench_Virtual_File_and_Dataset_Storage_Requirements.md`

---

### Requirement 9: StorageProvider Interface

**User Story:** As a platform developer, I want a StorageProvider interface below the VfsProvider layer so that physical storage concerns are separated from VFS routing and new backends can be added without touching the VFS API.

**Source:** [VFS-REQ] §8 FFW-VFS-SPI-001 to SPI-004.

#### Acceptance Criteria

9.1 THE `ff-vfs` crate (or a new `ff-storage-provider` crate) SHALL define a `StorageProvider` trait that all physical storage backends implement, separate from the `VfsProvider` trait.

9.2 THE `StorageProvider` trait SHALL expose at minimum: `allocate`, `open`, `stat`, `rename`, `delete`, `list`, and `reconcile` operations.

9.3 PROVIDERS SHALL declare capabilities rather than requiring callers to infer them from dataset type; declared capabilities SHALL include at minimum: stream-read, stream-write, record-read, record-write, keyed-access, relative-access, append-only, member-operations, atomic-rename, locking, snapshotting, watch-notifications.

9.4 THE native-filesystem provider and the SQLite record provider SHALL implement a common error taxonomy that maps to `VfsError` variants.

9.5 Provider-specific locators SHALL be opaque outside the provider and catalogue services; no user-interface or editor code SHALL construct or parse raw provider paths directly.

---

### Requirement 10: POSIX File Constraints

**User Story:** As a platform developer, I want POSIX files to remain native host filesystem objects so that external tools, editors, Git, and backup utilities can access them without workbench-specific extraction.

**Source:** [VFS-REQ] §7.9 FFW-VFS-POSIX-001 to POSIX-007; [ARCH] §9 POSIX Files.

#### Acceptance Criteria

10.1 POSIX files and directories SHALL remain native host filesystem objects by default; the system SHALL NOT copy POSIX file contents into SQLite.

10.2 THE catalogue MAY register a POSIX root, file, or directory using a provider locator and optional metadata, but registration SHALL NOT move or copy the content.

10.3 External changes to registered POSIX files SHALL be detected through refresh, filesystem notifications where supported, or reconciliation — the system SHALL NOT overwrite external changes silently.

10.4 Symlink handling SHALL be configurable, with loop detection and prevention of traversal beyond authorised roots.

10.5 Host permissions, file locking, case sensitivity, and path semantics SHALL be surfaced accurately and SHALL NOT be silently normalised into mainframe semantics.

10.6 WHEN a POSIX catalog is configured as read-only, THE provider SHALL return `VfsError::PermissionDenied` for any write, create, delete, or rename operation.

---

### Requirement 11: Cross-Resource Consistency

**User Story:** As a platform developer, I want operations that span SQLite and the filesystem to use a staged protocol so that interrupted operations leave the system in a recoverable state.

**Source:** [VFS-REQ] §11 FFW-VFS-TXN-001 to TXN-006.

#### Acceptance Criteria

11.1 WHEN a VFS create operation affects both catalogue state and physical content, THE system SHALL use a staged protocol: stage physical content, reserve catalogue state, publish physical object, mark catalogue entry active.

11.2 WHEN a VFS delete operation affects both catalogue state and physical content, THE system SHALL first mark the entry pending-deletion, then move or tombstone physical content, then finalise catalogue state.

11.3 Interrupted operations SHALL be discoverable through operation journals or transitional catalogue states.

11.4 On startup, THE system SHALL detect and offer deterministic recovery for incomplete operations.

11.5 THE system SHALL NOT report a VFS operation as successful until both catalogue and provider state satisfy the operation's postconditions.

---

### Requirement 12: Workspace Backup and Restore via VFS

**User Story:** As a workbench user, I want workspace backup and restore accessible through the VFS command layer so that I can protect and migrate my complete dataset environment.

**Source:** [VFS-REQ] §12 FFW-VFS-INT-001 to INT-006.

#### Acceptance Criteria

12.1 THE VFS layer SHALL expose a `workspace.backup` command that captures: the catalogue database, all SQLite record stores, all native dataset files, all library directories, and operation journals as one recoverable unit.

12.2 A backup SHALL include a manifest containing: schema version, provider configuration, object inventory, and integrity information.

12.3 THE VFS layer SHALL expose a `workspace.restore` command that supports restoration to the original workspace or remapping to a different root without changing logical dataset names.

12.4 THE system SHALL provide a `workspace.reconcile` command that compares catalogue state with provider state and reports discrepancies without automatically changing data.

12.5 THE system SHALL provide a `workspace.diagnose` command that reports orphaned physical objects and dangling catalogue entries.
