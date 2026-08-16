# Requirements Document

## Introduction

This feature specifies the local filesystem VFS provider for FileForgeWorkbench — the `ff-connector-local-fs` crate. This connector is the **primary VFS provider** for the initial release, implementing the `VfsProvider` trait defined by the `virtual-file-system` crate for native OS filesystem operations.

The local filesystem connector provides full read/write/create/delete/rename support for files and directories on the host operating system. It handles cross-platform path differences (Windows, Linux, macOS), OS-native file watching for real-time change detection, path resolution (relative paths, tilde expansion, environment variable expansion), and large file support via streaming I/O.

All filesystem I/O operations are performed asynchronously through Tokio to honour the workbench Async I/O Principle (Architecture Brief §9) — the GUI render thread is never blocked by file operations. The connector registers with the VFS provider registry under the `local` scheme, making local resources addressable as `vfs://local/path/to/resource`.

**Source references:**
- **WB** = Workbench Platform Architecture Brief (VFS principle FFW-ARCH-001, async I/O §9)
- **FFE** = FileForgeEditor file operations (adapted for VFS abstraction)

## Glossary

- **VfsProvider**: The trait defined by the `virtual-file-system` crate that all filesystem providers must implement. Defines async method signatures for open, read, write, create, delete, rename, stat, list, and watch operations. [WB]
- **Provider_Registry**: The component within the `virtual-file-system` crate where providers register themselves, keyed by URI scheme. [WB]
- **Resource_URI**: The unified resource identifier used by the VFS layer to address any resource: `vfs://scheme/path`. For the local filesystem provider, the scheme is `local` (e.g., `vfs://local/home/user/document.txt`). [WB]
- **Local_FS_Provider**: The `ff-connector-local-fs` crate's implementation of `VfsProvider`, providing access to the host operating system's native filesystem. [WB, FFE]
- **Native_Path**: An operating-system-specific filesystem path (e.g., `C:\Users\name\file.txt` on Windows, `/home/name/file.txt` on Unix). [FFE]
- **File_Watcher**: The component that monitors filesystem paths for changes using OS-native mechanisms (inotify on Linux, ReadDirectoryChangesW on Windows, FSEvents on macOS). [FFE]
- **Watch_Handle**: An opaque identifier returned when a watch is registered, used to remove or query the watch later. [FFE]
- **Watch_Event**: A notification emitted by the File_Watcher when a monitored path changes, containing the event type (created, modified, deleted, renamed), the affected path, and a timestamp. [FFE]
- **Debounce_Window**: A configurable time interval during which rapid successive events on the same path are coalesced into a single event to avoid overwhelming consumers. [FFE]
- **Path_Resolver**: The component responsible for converting relative paths, tilde paths, paths with environment variables, and paths with `..` segments into canonical absolute native paths. [WB]
- **Canonical_Path**: An absolute path with all symbolic links resolved, all `..` and `.` segments eliminated, and platform-appropriate normalisation applied. [WB]
- **VFS_Error**: The error type hierarchy defined by the `virtual-file-system` crate, providing provider-agnostic error categories (NotFound, PermissionDenied, StorageFull, InvalidPath, ResourceBusy). [WB]
- **Streaming_Reader**: An async reader that yields file content in chunks, suitable for files larger than available RAM. [WB]
- **Memory_Mapped_IO**: A mechanism that maps a file's contents directly into the process address space for random access without loading the entire file into heap memory. [FFE]
- **File_Metadata**: A struct containing file properties: size, modification time, creation time, access time, file type, permissions, and hidden status. [FFE]

## Requirements

### Requirement 1: Local Filesystem Provider

**User Story:** As a workbench developer, I want a VFS provider that maps the local filesystem to the VFS abstraction layer, so that all workbench components access local files through the unified VFS interface without direct `std::fs` calls.

**Source:** WB Architecture Brief FFW-ARCH-001 (VFS principle), FFE file operations (adapted). [WB, FFE]

#### Acceptance Criteria

1. THE Local_FS_Provider SHALL implement the `VfsProvider` trait defined by the `virtual-file-system` crate, providing async implementations for all required trait methods.
2. THE Local_FS_Provider SHALL register itself with the Provider_Registry under the URI scheme `"local"`, making local resources addressable as `vfs://local/{path}`.
3. WHEN the Local_FS_Provider receives a read request for a Resource_URI, THE provider SHALL open the corresponding Native_Path using Tokio async filesystem operations and return the file contents as an async byte stream.
4. WHEN the Local_FS_Provider receives a write request, THE provider SHALL write the provided content to the corresponding Native_Path atomically where possible (write to temporary file, then rename), falling back to direct write if atomic write is not supported on the target filesystem.
5. WHEN the Local_FS_Provider receives a create request for a file, THE provider SHALL create the file (and any missing parent directories) at the specified Native_Path, returning an error if the file already exists and overwrite is not specified.
6. WHEN the Local_FS_Provider receives a create request for a directory, THE provider SHALL create the directory (and any missing parent directories) at the specified Native_Path.
7. WHEN the Local_FS_Provider receives a delete request, THE provider SHALL remove the file or empty directory at the specified Native_Path; IF the target is a non-empty directory and recursive deletion is not explicitly requested, THEN THE provider SHALL return a VFS_Error indicating the directory is not empty.
8. WHEN the Local_FS_Provider receives a rename request, THE provider SHALL rename (move) the resource from the source path to the destination path using the native OS rename operation.
9. WHEN the Local_FS_Provider receives a list request for a directory, THE provider SHALL return an async stream of directory entries, each containing the entry name, resource type (file, directory, symlink), and basic metadata.
10. THE Local_FS_Provider SHALL perform all I/O operations asynchronously via Tokio, ensuring that no operation blocks the calling task's executor thread for more than 1 millisecond.

---

### Requirement 2: Cross-Platform Path Handling

**User Story:** As a user on Windows, Linux, or macOS, I want the local filesystem connector to correctly handle my platform's path conventions, so that I can access files using native path syntax regardless of which operating system I am running.

**Source:** WB Architecture Brief (multi-platform support). [WB, FFE]

#### Acceptance Criteria

1. WHEN the Local_FS_Provider receives a path on Windows, THE provider SHALL correctly handle Windows path formats including drive letter paths (`C:\Users\...`), UNC paths (`\\server\share\...`), and paths using either forward slash (`/`) or backslash (`\`) as separator.
2. WHEN the Local_FS_Provider receives a path on Unix (Linux or macOS), THE provider SHALL correctly handle Unix paths starting with `/` and using forward slash as the separator.
3. THE Local_FS_Provider SHALL normalise path separators internally to the platform-native separator (backslash on Windows, forward slash on Unix) before performing filesystem operations.
4. WHEN comparing paths on Windows, THE Local_FS_Provider SHALL perform case-insensitive comparison for path components (since NTFS is case-insensitive by default), using Unicode case folding rules.
5. WHEN comparing paths on Unix (Linux or macOS), THE Local_FS_Provider SHALL perform case-sensitive comparison for path components (since ext4/APFS are case-sensitive by default).
6. WHEN the Local_FS_Provider encounters a symbolic link, THE provider SHALL resolve the link to its target path before performing the requested operation, unless the operation explicitly requests link metadata (stat on the link itself).
7. WHEN the Local_FS_Provider receives a Windows path that exceeds 260 characters (MAX_PATH), THE provider SHALL automatically apply the extended-length path prefix (`\\?\`) to enable long path support on Windows.
8. THE Local_FS_Provider SHALL convert between Resource_URI format (`vfs://local/path`) and Native_Path format in both directions without data loss, preserving Unicode characters and platform-specific path features.
9. WHEN converting a Resource_URI to a Native_Path on Windows, THE provider SHALL decode the URI path component and map it to the appropriate drive letter or UNC form (e.g., `vfs://local/C:/Users/name` → `C:\Users\name`).
10. WHEN converting a Native_Path to a Resource_URI, THE provider SHALL encode special characters (spaces, Unicode) according to URI encoding rules and produce a valid `vfs://local/...` URI.

---

### Requirement 3: File Watching

**User Story:** As a workbench component, I want to be notified when files or directories change on disk, so that I can react to external modifications (e.g., reload a file edited externally, refresh a directory listing).

**Source:** FFE external-modification detection (adapted for VFS watcher interface). [FFE, WB]

#### Acceptance Criteria

1. THE File_Watcher SHALL use OS-native file watching mechanisms: inotify on Linux, ReadDirectoryChangesW on Windows, and FSEvents on macOS.
2. WHEN a watch is registered on an individual file, THE File_Watcher SHALL emit Watch_Events for the following changes to that file: content modified, file deleted, file renamed (moved away).
3. WHEN a watch is registered on a directory, THE File_Watcher SHALL emit Watch_Events for the following changes within that directory: file or subdirectory created, file or subdirectory deleted, file modified, file or subdirectory renamed.
4. THE File_Watcher SHALL support a recursive directory watch option that monitors all subdirectories within the watched directory to any depth.
5. THE File_Watcher SHALL debounce rapid successive events on the same path, coalescing events that occur within the Debounce_Window into a single event; THE default Debounce_Window SHALL be 500 milliseconds.
6. THE Debounce_Window SHALL be configurable via the workbench configuration system (`vfs.local.debounce_ms`), accepting values from 50 to 5000 milliseconds.
7. IF the configured debounce value is outside the valid range (50–5000), THEN THE File_Watcher SHALL clamp the value to the nearest bound and write a WARN-level log record indicating the adjustment.
8. WHEN a watch is registered, THE File_Watcher SHALL return a Watch_Handle that uniquely identifies the watch registration; THE caller SHALL use this handle to remove the watch later.
9. WHEN a watch is removed via its Watch_Handle, THE File_Watcher SHALL stop emitting events for that path and release all OS resources associated with the watch.
10. IF the watched path is deleted or becomes inaccessible, THEN THE File_Watcher SHALL emit a final deletion event and automatically remove the watch, logging an INFO-level record.
11. THE File_Watcher SHALL emit Watch_Events containing: the event type (Created, Modified, Deleted, Renamed), the affected Resource_URI, the new Resource_URI (for rename events), and a timestamp.
12. IF the OS-native watch mechanism encounters an error (too many watches, permission denied), THEN THE File_Watcher SHALL log a WARN-level record and return a VFS_Error to the caller without crashing the application.

---

### Requirement 4: Path Resolution

**User Story:** As a user, I want to use convenient path shorthand (relative paths, `~/`, environment variables) when specifying files, so that I don't have to type fully qualified absolute paths for common locations.

**Source:** WB Architecture Brief (developer convenience, cross-platform usability). [WB, FFE]

#### Acceptance Criteria

1. WHEN the Path_Resolver receives a relative path (not starting with `/`, `~`, or a drive letter), THE resolver SHALL resolve it against the current working directory of the application process.
2. WHEN the Path_Resolver receives a path starting with `~/` or `~\`, THE resolver SHALL expand the tilde to the current user's home directory (`$HOME` on Unix, `%USERPROFILE%` on Windows).
3. WHEN the Path_Resolver receives a path containing Unix-style environment variables (`$VARNAME` or `${VARNAME}`), THE resolver SHALL expand each variable to its current value from the process environment.
4. WHEN the Path_Resolver receives a path containing Windows-style environment variables (`%VARNAME%`), THE resolver SHALL expand each variable to its current value from the process environment.
5. IF an environment variable referenced in a path is not defined, THEN THE Path_Resolver SHALL return a VFS_Error of type InvalidPath indicating the undefined variable name.
6. WHEN the Path_Resolver receives a path containing `.` or `..` segments, THE resolver SHALL resolve these segments logically (without filesystem access) to produce an equivalent absolute path, except when symlinks are involved — in that case, the resolver SHALL perform filesystem-aware canonical resolution.
7. THE Path_Resolver SHALL provide a `canonicalize` method that resolves all symbolic links, eliminates all `.` and `..` segments, and returns the true absolute path as reported by the operating system.
8. IF the target of a `canonicalize` call does not exist, THEN THE Path_Resolver SHALL return a VFS_Error of type NotFound.
9. THE Path_Resolver SHALL provide bidirectional conversion between Resource_URI (`vfs://local/...`) and Native_Path, with round-trip fidelity — converting to URI and back SHALL produce a path equivalent to the original.
10. THE Path_Resolver SHALL handle paths containing Unicode characters, spaces, and special characters correctly on all supported platforms.

---

### Requirement 5: Metadata and Stat

**User Story:** As a workbench component, I want to query file metadata (size, timestamps, type, permissions) through the VFS interface, so that I can make decisions based on file properties without platform-specific code.

**Source:** FFE file operations (adapted for VFS metadata interface). [FFE, WB]

#### Acceptance Criteria

1. WHEN the Local_FS_Provider receives a stat request for a Resource_URI, THE provider SHALL return a File_Metadata struct containing: file size in bytes, last modification time, creation time (where available), and last access time.
2. THE File_Metadata SHALL include the resource type as an enum: RegularFile, Directory, Symlink, or Other (for device files, pipes, sockets, etc.).
3. THE File_Metadata SHALL include platform-appropriate permission information: on Unix, the read/write/execute bits for owner, group, and others; on Windows, the read-only attribute and effective access permissions.
4. THE File_Metadata SHALL include a `is_hidden` field that reports whether the file is hidden: on Unix, files whose name starts with `.` (dot); on Windows, files with the hidden attribute set.
5. WHEN the stat request targets a symbolic link and the `follow_links` option is true (the default), THE provider SHALL return metadata for the symlink's target.
6. WHEN the stat request targets a symbolic link and the `follow_links` option is false, THE provider SHALL return metadata for the symbolic link itself, including the link target path.
7. IF the stat request targets a path that does not exist, THEN THE provider SHALL return a VFS_Error of type NotFound.
8. IF the stat request targets a path for which the process lacks read permission, THEN THE provider SHALL return a VFS_Error of type PermissionDenied.
9. THE timestamp fields in File_Metadata SHALL use a platform-independent representation (e.g., `SystemTime` or equivalent) with at least second-level precision; sub-second precision SHALL be provided where the underlying filesystem supports it.
10. IF the underlying filesystem does not support a particular timestamp (e.g., creation time on older Linux filesystems), THEN THE corresponding field in File_Metadata SHALL be `None` rather than a fabricated or zero value.

---

### Requirement 6: Large File Support

**User Story:** As a user working with large data files, I want the local filesystem connector to handle files larger than available RAM without crashing or excessive memory usage, so that I can open and process large files reliably.

**Source:** FFE large-file support, WB Architecture Brief §9 (async I/O). [FFE, WB]

#### Acceptance Criteria

1. WHEN reading a file, THE Local_FS_Provider SHALL support streaming reads that yield the file content in configurable chunks (default chunk size: 64 KB), enabling the consumer to process data incrementally without loading the entire file into memory.
2. THE Streaming_Reader SHALL implement the standard async `Stream` or `AsyncRead` trait, allowing consumers to read data at their own pace with backpressure support.
3. THE Local_FS_Provider SHALL support memory-mapped I/O for files that require random access, mapping the file into the process address space without copying data into heap memory.
4. WHEN memory-mapped I/O is requested, THE provider SHALL handle files up to the platform maximum (2^63 bytes on 64-bit systems) without artificial size limits.
5. WHEN writing a file, THE Local_FS_Provider SHALL support chunked writing, accepting data in incremental chunks without requiring the entire content to be assembled in memory first.
6. THE Local_FS_Provider SHALL NOT impose any artificial file size limit beyond what the host operating system and filesystem support (e.g., no hard-coded maximum file size constant).
7. IF memory-mapped I/O is requested for a file that cannot be mapped (e.g., due to OS resource limits), THEN THE provider SHALL fall back to streaming read and write a DEBUG-level log record indicating the fallback.
8. WHEN reading a large file via streaming, THE provider SHALL report progress (bytes read so far / total file size) to enable progress bar display through the VFS progress reporting interface.

---

### Requirement 7: Error Handling

**User Story:** As a workbench developer, I want all OS-specific filesystem errors to be mapped to consistent VFS error types, so that consuming code handles errors uniformly regardless of the underlying platform.

**Source:** WB Architecture Brief (VFS error abstraction). [WB, FFE]

#### Acceptance Criteria

1. WHEN the operating system returns a "permission denied" error (EACCES on Unix, ERROR_ACCESS_DENIED on Windows), THE Local_FS_Provider SHALL return a VFS_Error of type PermissionDenied, including the Resource_URI that was accessed and the operation that was attempted.
2. WHEN the operating system returns a "file not found" or "path not found" error (ENOENT on Unix, ERROR_FILE_NOT_FOUND or ERROR_PATH_NOT_FOUND on Windows), THE Local_FS_Provider SHALL return a VFS_Error of type NotFound, including the Resource_URI.
3. WHEN the operating system returns a "disk full" or "no space left on device" error (ENOSPC on Unix, ERROR_DISK_FULL on Windows), THE Local_FS_Provider SHALL return a VFS_Error of type StorageFull, including the Resource_URI and the operation that was attempted.
4. WHEN the operating system returns a "name too long" error (ENAMETOOLONG on Unix, ERROR_FILENAME_EXCED_RANGE on Windows), THE Local_FS_Provider SHALL return a VFS_Error of type InvalidPath, including the offending path.
5. WHEN the operating system returns a "file is locked" or "sharing violation" error (ETXTBSY/EBUSY on Unix, ERROR_SHARING_VIOLATION or ERROR_LOCK_VIOLATION on Windows), THE Local_FS_Provider SHALL return a VFS_Error of type ResourceBusy, including the Resource_URI and a description of the lock conflict.
6. WHEN the operating system returns a "directory not empty" error (ENOTEMPTY on Unix, ERROR_DIR_NOT_EMPTY on Windows), THE Local_FS_Provider SHALL return a VFS_Error of type DirectoryNotEmpty, including the Resource_URI.
7. WHEN the operating system returns a "read-only filesystem" error (EROFS on Unix), THE Local_FS_Provider SHALL return a VFS_Error of type PermissionDenied with an error message indicating the filesystem is read-only.
8. FOR any OS error code not explicitly mapped above, THE Local_FS_Provider SHALL return a VFS_Error of type IoError containing the raw OS error code, the error description, the Resource_URI, and the operation that was attempted.
9. ALL VFS_Error instances returned by the Local_FS_Provider SHALL include a human-readable message that follows the workbench error format: `[connector-local-fs] operation: description` with a maximum length of 200 characters.
10. THE Local_FS_Provider SHALL log all errors at WARN level or above via the logging subsystem before returning them to the caller, including the full OS error code and path for diagnostic purposes.

