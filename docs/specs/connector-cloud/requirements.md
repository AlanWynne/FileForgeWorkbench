# Requirements Document — DEFERRED

> ⚠️ **STATUS: DEFERRED — Not in initial release.**
>
> This specification documents the *future* Cloud connector for
> FileForgeWorkbench — SharePoint Online, OneDrive, and OAuth 2.0
> authentication. It is NOT scheduled for the initial release. The
> `connector-extensibility` trait (defined in `ff-connector-extensibility`)
> provides the architectural hook that this connector will use when implemented.

## Introduction

The `ff-connector-cloud` crate will provide VFS connectors for cloud storage
services accessed via the Microsoft Graph API — initially SharePoint Online
document libraries and OneDrive (personal and business). It will implement
the `ConnectorPlugin` trait from `ff-connector-extensibility`, which combines
`VfsProvider` (from `ff-vfs`) with connector lifecycle, authentication, and
capability advertisement.

### What This Connector Will Provide

- **SharePoint Online** — document library access via Microsoft Graph API,
  including site/library/folder/file navigation, metadata retrieval, and
  versioning support.
- **OneDrive** — personal and business file access via Microsoft Graph API,
  including delta sync for efficient change tracking and sharing link support.
- **OAuth 2.0 authentication flow** — PKCE authorization code flow for
  desktop apps, device code flow for headless environments, and client
  credentials flow for service scenarios. Includes token refresh, consent
  management, and secure token storage.
- **Potential future expansion** — Google Drive, Dropbox, AWS S3, Azure Blob
  Storage. The connector architecture is designed to accommodate additional
  cloud providers via the same extensibility framework.

### Architectural Integration Point

This connector implements:
- `VfsProvider` trait from `ff-vfs` (for read/write/list/stat/watch operations)
- `ConnectorPlugin` trait from `ff-connector-extensibility` (for lifecycle,
  registration, capability advertisement, and authentication)
- `FileForgePlugin` trait from `ff-plugin` (for plugin initialization/shutdown)

Registration occurs at plugin initialization time via the `ConnectorRegistry`.
The connector advertises its URI schemes (e.g., `sharepoint://`, `onedrive://`)
and its supported capabilities per cloud service.

### Extensibility Hook (Initial Release)

The `connector-extensibility` crate ships in the initial release and defines all
the traits, error types, and registry infrastructure that this connector will
consume. No code changes to VFS core or the workbench platform will be required
to add this connector — it plugs in via the existing extensibility framework.

---

## Placeholder Requirements (Future Scope)

The following outline documents the eventual scope. Full EARS-format acceptance
criteria will be written when this connector moves to active development.

### Requirement 1: OAuth 2.0 Authentication

- Authorization Code flow with PKCE for desktop application scenarios
- Device Code flow for headless/terminal environments
- Client Credentials flow for service-to-service (unattended) access
- Secure token storage via OS credential manager (Windows Credential Vault,
  macOS Keychain, Linux Secret Service)
- Automatic token refresh before expiry with configurable margin
- Consent and scope management (Files.Read, Files.ReadWrite, Sites.Read.All)
- Multi-tenant and single-tenant Azure AD app registration support
- Emit `ConnectorStateChanged` events on authentication success/failure/expiry

### Requirement 2: SharePoint Online Access

- Site discovery — enumerate sites accessible to the authenticated user
- Document library browsing — list libraries within a site
- Folder/file navigation — hierarchical traversal of library contents
- File metadata retrieval — size, created/modified dates, author, content type
- Version history access — list versions, download specific version, restore
- Check-out/check-in support for collaborative editing workflows
- Search within a site or library via Microsoft Search API

### Requirement 3: OneDrive Access

- Personal OneDrive and OneDrive for Business access via same Graph endpoint
- Delta sync — efficient incremental change tracking using delta tokens
- Sharing — resolve shared links, access shared-with-me items
- Special folders (Documents, Pictures, App Root) as VFS mount points
- Large file upload via upload sessions (resumable chunked upload)
- Thumbnail retrieval for image/document preview

### Requirement 4: File Operations via Graph API

- Read — download file content with range support for partial reads
- Write — upload file content (small files direct, large files via session)
- List — directory enumeration with pagination (nextLink handling)
- Search — full-text and metadata search via Graph `/search` endpoint
- Create — create folders and empty files
- Delete — move to recycle bin (soft delete) with permanent delete option
- Rename/Move — item rename and cross-folder move within same drive

### Requirement 5: Offline Cache and Sync Conflict Resolution

- Local cache for recently accessed file metadata and content
- Conflict detection when local edits collide with remote changes
- Conflict resolution strategies: last-writer-wins, prompt user, fork copy
- Cache invalidation via delta sync polling or webhook notification
- Configurable cache size limit with LRU eviction

### Requirement 6: Error Mapping

- Map HTTP 4xx errors (401 Unauthorized, 403 Forbidden, 404 Not Found) to
  `ConnectorError` variants with actionable context
- Map HTTP 5xx errors (502 Bad Gateway, 503 Service Unavailable) as retryable
- Handle 429 Too Many Requests with Retry-After header respect
- Token expiry detection and automatic re-authentication attempt
- Classify throttling, quota exceeded, and item-locked as distinct error kinds
- Include Graph API error code and message in error context

---

## Dependencies

| Crate | Relationship |
|-------|-------------|
| `ff-vfs` | Implements `VfsProvider` trait |
| `ff-connector-extensibility` | Implements `ConnectorPlugin` trait |
| `ff-plugin` | Implements `FileForgePlugin` lifecycle |
| `ff-logging` | Structured logging |

## References

- **WB**: Workbench Architecture Brief — VFS extensibility, FFW-ARCH-001
- **FFW**: FileForgeWorkbench cross-cutting requirements (VFS Principle, Plugin Architecture)
- Connector-extensibility requirements (Requirement 6: Future Connector Hooks)
- [Microsoft Graph API documentation](https://learn.microsoft.com/en-us/graph/overview)
- [OAuth 2.0 Authorization Code with PKCE](https://datatracker.ietf.org/doc/html/rfc7636)
