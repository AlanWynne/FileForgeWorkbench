# Implementation Plan: Local Filesystem Connector (`ff-connector-local-fs`)

## Overview

Implement the local filesystem VFS provider for FileForgeWorkbench. This crate implements the `VfsProvider` trait from `ff-vfs`, providing full async I/O for local filesystem operations, cross-platform path handling, OS-native file watching, large file support (streaming and memory-mapped), and unified error mapping.

All tasks reference requirements from `.kiro/specs/connector-local-fs/requirements.md` and implement the architecture defined in `.kiro/specs/connector-local-fs/design.md`.

---

## Tasks

- [ ] 1. Project scaffolding and crate setup
  - [ ] 1.1 Create `crates/ff-connector-local-fs/Cargo.toml` with dependencies (tokio, notify, memmap2, thiserror, async-trait, ff-vfs, ff-logging) and dev-dependencies (proptest, tempfile, pretty_assertions)
  - [ ] 1.2 Create `crates/ff-connector-local-fs/src/lib.rs` with crate-level docs, module declarations, and public API re-exports
  - [ ] 1.3 Create module stub files: `provider.rs`, `path_resolver.rs`, `watcher.rs`, `streaming.rs`, `mmap.rs`, `metadata.rs`, `error.rs`, and `platform/mod.rs` with `windows.rs`, `unix.rs`, `macos.rs`
  - [ ] 1.4 Add `ff-connector-local-fs` to workspace `Cargo.toml` members list
  - [ ] 1.5 Verify `cargo check -p ff-connector-local-fs` compiles cleanly

- [ ] 2. Error handling and error mapping module
  - [ ] 2.1 Implement `ConnectorError` enum in `error.rs` with `HomeDirNotFound`, `WorkingDirFailed`, and `WatcherInitFailed` variants using `thiserror`
  - [ ] 2.2 Implement `map_io_error` function that maps `std::io::Error` to `VfsError` variants per the mapping table (PermissionDenied, NotFound, StorageFull, InvalidPath, ResourceBusy, DirectoryNotEmpty, IoError)
    - Validates: Requirement 7 AC 1–8
  - [ ] 2.3 Implement error message formatting conforming to `[connector-local-fs] operation: description` format with 200-char max
    - Validates: Requirement 7 AC 9
  - [ ] 2.4 Integrate WARN-level logging for all mapped errors via `ff-logging` before returning
    - Validates: Requirement 7 AC 10
  - [ ] 2.5 Write unit tests for error mapping covering all OS error kinds

- [ ] 3. Path resolver — core path resolution logic
  - [ ] 3.1 Implement `PathResolver::new()` caching home directory and current working directory
    - Validates: Requirement 4 AC 1, AC 2
  - [ ] 3.2 Implement `expand_tilde` method replacing `~/` or `~\` with user home directory
    - Validates: Requirement 4 AC 2
  - [ ] 3.3 Implement `expand_env_vars` supporting `$VAR`, `${VAR}` (Unix) and `%VAR%` (Windows) expansion with VfsError on undefined variables
    - Validates: Requirement 4 AC 3, AC 4, AC 5
  - [ ] 3.4 Implement `resolve` method combining relative path resolution, tilde expansion, env-var expansion, and `.`/`..` segment elimination
    - Validates: Requirement 4 AC 1, AC 6
  - [ ] 3.5 Implement `canonicalize` async method that resolves symlinks and produces true absolute path
    - Validates: Requirement 4 AC 7, AC 8
  - [ ] 3.6 Implement `normalise_separators` for platform-native separator conversion
    - Validates: Requirement 2 AC 3
  - [ ] 3.7 Implement `paths_equal` with case-insensitive comparison on Windows and case-sensitive on Unix
    - Validates: Requirement 2 AC 4, AC 5
  - [ ] 3.8 Implement `native_to_uri` and `uri_to_native` for bidirectional URI ↔ native path conversion
    - Validates: Requirement 2 AC 8, AC 9, AC 10, Requirement 4 AC 9
  - [ ] 3.9 Write unit tests for path resolution covering relative, tilde, env-var, and dotdot segments

- [ ] 4. Platform-specific path handling
  - [ ] 4.1 Implement `platform/windows.rs`: drive letter parsing, UNC path support, long path prefix (`\\?\`), case-insensitive comparison, hidden file detection via attributes
    - Validates: Requirement 2 AC 1, AC 4, AC 7
  - [ ] 4.2 Implement `platform/unix.rs`: Unix permissions mapping, symlink resolution, dot-prefix hidden file detection
    - Validates: Requirement 2 AC 2, AC 5, AC 6
  - [ ] 4.3 Implement `platform/macos.rs`: FSEvents specifics, case-insensitive-but-preserving comparison, UF_HIDDEN flag detection
  - [ ] 4.4 Implement `platform/mod.rs` with conditional compilation re-exports (`#[cfg(target_os = ...)]`)
  - [ ] 4.5 Write platform-conditional unit tests for Windows long paths, UNC paths, and Unix symlinks

- [ ] 5. Property-based tests for path resolution
  - [ ] 5.1 Write property test: URI ↔ Native Path Round-Trip Fidelity (Property 1)
    - **Validates: Requirements 2.8, 2.9, 2.10, 4.9**
    - Strategy: generate valid native paths with platform-appropriate characters
    - Assertion: `uri_to_native(native_to_uri(P)) == normalise(P)`
  - [ ] 5.2 Write property test: Path Resolution Determinism (Property 2)
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6**
    - Strategy: generate path strings containing ~, $VAR, relative segments, `..` components
    - Assertion: `resolve(path) == resolve(path)` and result is absolute
  - [ ] 5.3 Write property test: Path Normalisation Idempotence (Property 5)
    - **Validates: Requirements 2.3, 4.6**
    - Strategy: generate paths with mixed separators, redundant separators, `.` and `..`
    - Assertion: `normalise(normalise(P)) == normalise(P)`
  - [ ] 5.4 Write property test: Environment Variable Expansion Safety (Property 10)
    - **Validates: Requirements 4.3, 4.4, 4.5**
    - Strategy: generate path strings with variable references and partial env maps
    - Assertion: all vars defined → no unexpanded syntax; undefined var → `Err(InvalidPath)`
  - [ ] 5.5 Write property test: Platform Path Comparison Consistency (Property 8)
    - **Validates: Requirements 2.4, 2.5**
    - Strategy: generate pairs of path strings differing only in case
    - Assertion: Windows → equal; Unix → not equal when case differs

- [ ] 6. File metadata module
  - [ ] 6.1 Implement `FileMetadata` struct with size, timestamps, resource type, permissions, is_hidden, and symlink_target fields
    - Validates: Requirement 5 AC 1, AC 2, AC 3, AC 4, AC 9, AC 10
  - [ ] 6.2 Implement `ResourceType` enum (RegularFile, Directory, Symlink, Other) with `#[non_exhaustive]`
    - Validates: Requirement 5 AC 2
  - [ ] 6.3 Implement `FilePermissions` enum with Unix and Windows variants
    - Validates: Requirement 5 AC 3
  - [ ] 6.4 Implement `stat` helper that reads OS metadata via `tokio::fs::metadata` / `tokio::fs::symlink_metadata` and maps to `FileMetadata`
    - Validates: Requirement 5 AC 5, AC 6, AC 7, AC 8
  - [ ] 6.5 Write unit tests for metadata mapping including hidden file detection, symlink handling, and missing timestamp fields

- [ ] 7. Streaming I/O — reader and writer
  - [ ] 7.1 Implement `StreamingReader` struct with `tokio::io::AsyncRead` trait implementation and configurable chunk size
    - Validates: Requirement 6 AC 1, AC 2
  - [ ] 7.2 Implement progress callback support on `StreamingReader` (bytes_read / total_size reporting)
    - Validates: Requirement 6 AC 8
  - [ ] 7.3 Implement `StreamingWriter` with atomic write strategy (temp file + rename) and fallback to direct write
    - Validates: Requirement 1 AC 4, Requirement 6 AC 5
  - [ ] 7.4 Implement `MemoryMappedFile` struct using `memmap2` with fallback to streaming on mmap failure
    - Validates: Requirement 6 AC 3, AC 4, AC 7
  - [ ] 7.5 Ensure no artificial file size limits (Requirement 6 AC 6) — only OS/filesystem limits apply
    - Validates: Requirement 6 AC 6
  - [ ] 7.6 Write unit tests for streaming reader/writer using `tempfile::TempDir`

- [ ] 8. Property-based tests for streaming and errors
  - [ ] 8.1 Write property test: Streaming Reader Completeness (Property 6)
    - **Validates: Requirements 6.1, 6.2, 1.3**
    - Strategy: generate file content of sizes 0..10MB and chunk sizes 1KB–1MB
    - Assertion: `concat(all_chunks) == original_content` and `sum(chunk_sizes) == file_size`
  - [ ] 8.2 Write property test: Atomic Write Consistency (Property 7)
    - **Validates: Requirements 1.4**
    - Strategy: generate file content and simulate completion/failure scenarios
    - Assertion: successful → `read(path) == written_data`; failed → original preserved
  - [ ] 8.3 Write property test: Error Mapping Completeness (Property 3)
    - **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8**
    - Strategy: generate all `ErrorKind` variants with arbitrary OS codes, URIs, operations
    - Assertion: never panics, error message contains operation name, length ≤ 200 chars

- [ ] 9. File watcher implementation
  - [ ] 9.1 Implement `FileWatcher::new()` with configurable debounce window and background Tokio task for event coalescing
    - Validates: Requirement 3 AC 5, AC 6
  - [ ] 9.2 Implement `FileWatcher::watch()` using the `notify` crate for OS-native file watching, returning `WatchHandle`
    - Validates: Requirement 3 AC 1, AC 8
  - [ ] 9.3 Implement recursive directory watch option
    - Validates: Requirement 3 AC 4
  - [ ] 9.4 Implement `WatchEvent` and `WatchEventKind` types (Created, Modified, Deleted, Renamed) with Resource_URI and timestamp
    - Validates: Requirement 3 AC 2, AC 3, AC 11
  - [ ] 9.5 Implement debounce logic: coalesce events on same path within debounce window, preserve events on different paths
    - Validates: Requirement 3 AC 5
  - [ ] 9.6 Implement debounce configuration validation (50–5000ms range), clamping out-of-range values with WARN log
    - Validates: Requirement 3 AC 6, AC 7
  - [ ] 9.7 Implement `WatchHandle::cancel()` and `FileWatcher::unwatch()` for resource cleanup
    - Validates: Requirement 3 AC 9
  - [ ] 9.8 Implement auto-removal of watch on path deletion with final deletion event and INFO log
    - Validates: Requirement 3 AC 10
  - [ ] 9.9 Implement error handling for OS watch errors (too many watches, permission denied)
    - Validates: Requirement 3 AC 12
  - [ ] 9.10 Write integration tests for file watching using `tempfile::TempDir` with create/modify/delete scenarios

- [ ] 10. Property-based test for watch debounce
  - [ ] 10.1 Write property test: Watch Event Debounce Coalescing (Property 4)
    - **Validates: Requirements 3.5**
    - Strategy: generate sequences of (path, timestamp) events with repeating paths within debounce window
    - Assertion: at most one event per path per debounce period; distinct paths preserved independently

- [ ] 11. LocalFsProvider — VfsProvider trait implementation
  - [ ] 11.1 Implement `LocalFsProvider::new()` and `LocalFsProvider::with_config()` constructors
    - Validates: Requirement 1 AC 1
  - [ ] 11.2 Implement `LocalFsProvider::register()` to register with Provider_Registry under scheme `"local"`
    - Validates: Requirement 1 AC 2
  - [ ] 11.3 Implement `VfsProvider::scheme()` returning `"local"` and `VfsProvider::capabilities()` returning full capability set
  - [ ] 11.4 Implement `VfsProvider::read()` and `VfsProvider::read_stream()` using PathResolver and StreamingReader
    - Validates: Requirement 1 AC 3
  - [ ] 11.5 Implement `VfsProvider::write()` using StreamingWriter with atomic rename strategy
    - Validates: Requirement 1 AC 4
  - [ ] 11.6 Implement `VfsProvider::create()` for files and directories (with parent directory creation)
    - Validates: Requirement 1 AC 5, AC 6
  - [ ] 11.7 Implement `VfsProvider::delete()` with empty-directory guard and recursive option
    - Validates: Requirement 1 AC 7
  - [ ] 11.8 Implement `VfsProvider::rename()` using OS-native rename
    - Validates: Requirement 1 AC 8
  - [ ] 11.9 Implement `VfsProvider::list()` returning async stream of directory entries
    - Validates: Requirement 1 AC 9
  - [ ] 11.10 Implement `VfsProvider::stat()` and `VfsProvider::exists()` using metadata module
    - Validates: Requirement 5 AC 1–10
  - [ ] 11.11 Implement `VfsProvider::watch()` delegating to FileWatcher
    - Validates: Requirement 3 AC 1–12
  - [ ] 11.12 Implement `VfsProvider::search()` with async streaming results
  - [ ] 11.13 Ensure all I/O is fully async via Tokio — no blocking calls on executor thread
    - Validates: Requirement 1 AC 10
  - [ ] 11.14 Write VfsProvider trait contract integration tests (open, read, write, create, delete, rename, list, stat, exists round-trip)

- [ ] 12. Property-based test for metadata timestamps
  - [ ] 12.1 Write property test: FileMetadata Timestamp Validity (Property 9)
    - **Validates: Requirements 5.1, 5.9, 5.10**
    - Strategy: create files with various operations, then stat them
    - Assertion: `created <= modified` (when both available); all timestamps ≤ `SystemTime::now()`

- [ ] 13. End-to-end integration tests
  - [ ] 13.1 Write integration test: full VFS round-trip — register provider, write file via URI, read back, verify content matches
  - [ ] 13.2 Write integration test: directory operations — create directory tree, list contents, delete recursively
  - [ ] 13.3 Write integration test: file watching round-trip — register watch, create/modify/delete file, verify events received in order
  - [ ] 13.4 Write integration test: path resolution with tilde and env vars in real filesystem context
  - [ ] 13.5 Write integration test: large file streaming — write 10MB file in chunks, read back in different chunk size, verify byte-equality
  - [ ] 13.6 Write integration test: error scenarios — permission denied, not found, directory not empty
  - [ ] 13.7 Verify `cargo test -p ff-connector-local-fs` passes cleanly with no warnings

---

## Acceptance Criteria Coverage Map

| Requirement | Acceptance Criteria | Covered by Task(s) |
|-------------|--------------------|--------------------|
| Req 1: Local Filesystem Provider | AC 1 (VfsProvider impl) | 11.1, 11.3 |
| | AC 2 (register under "local") | 11.2 |
| | AC 3 (async read via Tokio) | 11.4 |
| | AC 4 (atomic write) | 7.3, 11.5 |
| | AC 5 (create file + parents) | 11.6 |
| | AC 6 (create directory + parents) | 11.6 |
| | AC 7 (delete with empty-dir guard) | 11.7 |
| | AC 8 (rename) | 11.8 |
| | AC 9 (list directory) | 11.9 |
| | AC 10 (async, non-blocking) | 11.13 |
| Req 2: Cross-Platform Path Handling | AC 1 (Windows paths) | 4.1 |
| | AC 2 (Unix paths) | 4.2 |
| | AC 3 (normalise separators) | 3.6 |
| | AC 4 (case-insensitive Windows) | 3.7, 4.1 |
| | AC 5 (case-sensitive Unix) | 3.7, 4.2 |
| | AC 6 (symlink resolution) | 4.2 |
| | AC 7 (long path prefix Windows) | 4.1 |
| | AC 8 (URI ↔ native fidelity) | 3.8 |
| | AC 9 (URI → native Windows) | 3.8, 4.1 |
| | AC 10 (native → URI encoding) | 3.8 |
| Req 3: File Watching | AC 1 (OS-native mechanisms) | 9.2 |
| | AC 2 (file events: modified, deleted, renamed) | 9.4 |
| | AC 3 (directory events) | 9.4 |
| | AC 4 (recursive watch) | 9.3 |
| | AC 5 (debounce) | 9.5 |
| | AC 6 (configurable debounce) | 9.6 |
| | AC 7 (debounce range clamping) | 9.6 |
| | AC 8 (WatchHandle returned) | 9.2 |
| | AC 9 (watch removal + cleanup) | 9.7 |
| | AC 10 (auto-remove on deletion) | 9.8 |
| | AC 11 (WatchEvent structure) | 9.4 |
| | AC 12 (OS error handling) | 9.9 |
| Req 4: Path Resolution | AC 1 (relative path resolution) | 3.4 |
| | AC 2 (tilde expansion) | 3.2 |
| | AC 3 (Unix env-var expansion) | 3.3 |
| | AC 4 (Windows env-var expansion) | 3.3 |
| | AC 5 (undefined var → error) | 3.3 |
| | AC 6 (dot/dotdot elimination) | 3.4 |
| | AC 7 (canonicalize) | 3.5 |
| | AC 8 (canonicalize not-found error) | 3.5 |
| | AC 9 (bidirectional URI ↔ native) | 3.8 |
| | AC 10 (Unicode + special chars) | 3.8, 4.1, 4.2 |
| Req 5: Metadata and Stat | AC 1 (size, timestamps) | 6.1, 6.4 |
| | AC 2 (resource type enum) | 6.2 |
| | AC 3 (platform permissions) | 6.3 |
| | AC 4 (is_hidden) | 6.1 |
| | AC 5 (follow_links=true) | 6.4 |
| | AC 6 (follow_links=false) | 6.4 |
| | AC 7 (not-found error) | 6.4 |
| | AC 8 (permission-denied error) | 6.4 |
| | AC 9 (timestamp precision) | 6.1 |
| | AC 10 (None for unsupported timestamps) | 6.1 |
| Req 6: Large File Support | AC 1 (streaming chunks) | 7.1 |
| | AC 2 (AsyncRead trait) | 7.1 |
| | AC 3 (memory-mapped I/O) | 7.4 |
| | AC 4 (large file mmap support) | 7.4 |
| | AC 5 (chunked writing) | 7.3 |
| | AC 6 (no artificial limits) | 7.5 |
| | AC 7 (mmap fallback) | 7.4 |
| | AC 8 (progress reporting) | 7.2 |
| Req 7: Error Handling | AC 1 (PermissionDenied) | 2.2 |
| | AC 2 (NotFound) | 2.2 |
| | AC 3 (StorageFull) | 2.2 |
| | AC 4 (InvalidPath) | 2.2 |
| | AC 5 (ResourceBusy) | 2.2 |
| | AC 6 (DirectoryNotEmpty) | 2.2 |
| | AC 7 (read-only FS) | 2.2 |
| | AC 8 (unmapped → IoError) | 2.2 |
| | AC 9 (error format) | 2.3 |
| | AC 10 (WARN logging) | 2.4 |

---

## Task Dependency Graph

```json
{
  "taskGroups": [
    {
      "id": "1",
      "label": "Project scaffolding and crate setup",
      "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5"],
      "dependsOn": []
    },
    {
      "id": "2",
      "label": "Error handling and error mapping",
      "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5"],
      "dependsOn": ["1"]
    },
    {
      "id": "3",
      "label": "Path resolver core logic",
      "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7", "3.8", "3.9"],
      "dependsOn": ["1", "2"]
    },
    {
      "id": "4",
      "label": "Platform-specific path handling",
      "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5"],
      "dependsOn": ["3"]
    },
    {
      "id": "5",
      "label": "Property-based tests for path resolution",
      "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5"],
      "dependsOn": ["3", "4"]
    },
    {
      "id": "6",
      "label": "File metadata module",
      "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5"],
      "dependsOn": ["2", "4"]
    },
    {
      "id": "7",
      "label": "Streaming I/O — reader and writer",
      "tasks": ["7.1", "7.2", "7.3", "7.4", "7.5", "7.6"],
      "dependsOn": ["2"]
    },
    {
      "id": "8",
      "label": "Property-based tests for streaming and errors",
      "tasks": ["8.1", "8.2", "8.3"],
      "dependsOn": ["7", "2"]
    },
    {
      "id": "9",
      "label": "File watcher implementation",
      "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "9.8", "9.9", "9.10"],
      "dependsOn": ["2", "3"]
    },
    {
      "id": "10",
      "label": "Property-based test for watch debounce",
      "tasks": ["10.1"],
      "dependsOn": ["9"]
    },
    {
      "id": "11",
      "label": "LocalFsProvider VfsProvider trait implementation",
      "tasks": ["11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "11.7", "11.8", "11.9", "11.10", "11.11", "11.12", "11.13", "11.14"],
      "dependsOn": ["3", "4", "6", "7", "9"]
    },
    {
      "id": "12",
      "label": "Property-based test for metadata timestamps",
      "tasks": ["12.1"],
      "dependsOn": ["6", "11"]
    },
    {
      "id": "13",
      "label": "End-to-end integration tests",
      "tasks": ["13.1", "13.2", "13.3", "13.4", "13.5", "13.6", "13.7"],
      "dependsOn": ["11"]
    }
  ]
}
```
