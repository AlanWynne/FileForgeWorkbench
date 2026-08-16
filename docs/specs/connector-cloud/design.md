# Design Document — `ff-connector-cloud`

> ⚠️ **STATUS: DEFERRED — Not in initial release.**
>
> This document captures the *planned* architecture for the cloud storage
> connector. No implementation tasks will be generated from this design until
> the connector moves to active development. It exists solely to document
> integration points and expected module structure so that upstream crates
> (particularly `ff-connector-extensibility`) can be designed with these
> consumers in mind.

---

## 1. Purpose

`ff-connector-cloud` will provide VFS-layer access to OAuth-protected cloud
storage services — initially Microsoft SharePoint Online and OneDrive (personal
and business) via the Microsoft Graph API. It plugs into the workbench via the
extensibility framework shipped in the initial release.

---

## 2. Integration with `ConnectorPlugin` Trait

The connector implements three traits defined in upstream crates:

| Trait | Source Crate | Role |
|-------|-------------|------|
| `VfsProvider` | `ff-vfs` | Read/write/list/stat/watch file operations |
| `ConnectorPlugin` | `ff-connector-extensibility` | Lifecycle, registration, capability advertisement, authentication |
| `FileForgePlugin` | `ff-plugin` | Plugin initialization and shutdown |

### Registration Flow

```text
1. ff-plugin discovers ff-connector-cloud via plugin manifest
2. FileForgePlugin::init() is called
3. Connector registers itself with ConnectorRegistry (from ff-connector-extensibility)
4. ConnectorRegistry advertises URI schemes to ff-vfs
5. VFS routes matching URIs to this connector's VfsProvider impl
```

---

## 3. VFS URI Schemes

Each cloud service uses a distinct URI scheme registered with the VFS layer:

| Service | URI Pattern | Example |
|---------|------------|---------|
| SharePoint | `vfs://sharepoint/<tenant>/<site>/<library>/<path>` | `vfs://sharepoint/contoso/marketing/documents/report.docx` |
| OneDrive | `vfs://onedrive/<account>/<path>` | `vfs://onedrive/user@contoso.com/Documents/notes.md` |
| OneDrive (personal) | `vfs://onedrive-personal/<path>` | `vfs://onedrive-personal/Pictures/photo.jpg` |

The connector advertises these schemes during registration. The VFS layer
delegates any path resolution under these schemes to the cloud connector's
`VfsProvider` implementation.

---

## 4. Upstream Crate Dependencies

| Crate | What This Connector Consumes |
|-------|------------------------------|
| `ff-vfs` | `VfsProvider` trait, `VfsEntry`, `VfsMetadata`, `VfsError` types |
| `ff-connector-extensibility` | `ConnectorPlugin` trait, `ConnectorRegistry`, `ConnectorCapability`, `ConnectorError`, `ConnectorState` |
| `ff-plugin` | `FileForgePlugin` trait, plugin manifest types |
| `ff-logging` | Structured logging macros and span context |

No changes to these upstream crates are required to add this connector — the
extensibility framework is designed to support new connectors without
modification.

---

## 5. Placeholder Module Structure

```text
crates/ff-connector-cloud/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Crate root, re-exports public API
│   ├── plugin.rs               # FileForgePlugin + ConnectorPlugin impl
│   ├── auth/
│   │   ├── mod.rs              # Re-exports
│   │   ├── oauth.rs            # OAuthFlow orchestration
│   │   ├── pkce.rs             # PKCE code challenge/verifier
│   │   ├── device_code.rs      # Device code flow (headless)
│   │   ├── client_creds.rs     # Client credentials flow (service)
│   │   └── token_store.rs      # Secure token persistence (OS keyring)
│   ├── graph/
│   │   ├── mod.rs              # Re-exports
│   │   ├── client.rs           # Graph API HTTP client wrapper
│   │   ├── pagination.rs       # nextLink / delta token handling
│   │   └── error_mapping.rs    # HTTP status → ConnectorError mapping
│   ├── sharepoint/
│   │   ├── mod.rs              # Re-exports
│   │   ├── provider.rs         # VfsProvider impl for SharePoint
│   │   ├── site.rs             # Site/library discovery
│   │   └── versioning.rs       # Version history, check-in/check-out
│   ├── onedrive/
│   │   ├── mod.rs              # Re-exports
│   │   ├── provider.rs         # VfsProvider impl for OneDrive
│   │   ├── delta_sync.rs       # Incremental change tracking
│   │   └── upload_session.rs   # Resumable chunked upload
│   └── cache/
│       ├── mod.rs              # Re-exports
│       ├── metadata_cache.rs   # LRU metadata cache
│       └── conflict.rs         # Sync conflict detection/resolution
└── tests/
    └── integration/
        ├── auth_flow_test.rs
        ├── sharepoint_vfs_test.rs
        └── onedrive_vfs_test.rs
```

---

## 6. Key Types

```rust
/// Represents an authenticated connection to a cloud service.
pub struct CloudConnection {
    pub service: CloudService,
    pub account: String,
    pub token: AccessToken,
    pub state: ConnectorState,
}

/// Identifies which cloud service a connection targets.
pub enum CloudService {
    SharePointOnline { tenant: String, site: String },
    OneDriveBusiness { account: String },
    OneDrivePersonal,
}

/// Orchestrates OAuth 2.0 authentication across supported flows.
pub struct OAuthFlow {
    pub client_id: String,
    pub tenant_id: String,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
    pub flow_type: OAuthFlowType,
}

/// Supported OAuth flow variants.
pub enum OAuthFlowType {
    AuthorizationCodePkce,
    DeviceCode,
    ClientCredentials { client_secret: String },
}

/// An access token with metadata for refresh management.
pub struct AccessToken {
    pub token: String,
    pub refresh_token: Option<String>,
    pub expires_at: std::time::SystemTime,
    pub scopes: Vec<String>,
}

/// Represents a file or folder item retrieved from a cloud service.
pub struct CloudItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub item_type: CloudItemType,
    pub size: u64,
    pub created: std::time::SystemTime,
    pub modified: std::time::SystemTime,
    pub etag: Option<String>,
}

pub enum CloudItemType {
    File { mime_type: String },
    Folder,
    Site,
    Library,
}
```

---

## 7. OAuth Authentication Flow

### Authorization Code with PKCE (Desktop)

```text
1. Generate code_verifier (random 43–128 chars) and code_challenge (S256 hash)
2. Open system browser → Azure AD /authorize endpoint with:
   - client_id, redirect_uri (http://localhost:<port>)
   - code_challenge, code_challenge_method=S256
   - scope: Files.ReadWrite.All Sites.Read.All offline_access
3. User authenticates and consents in browser
4. Azure AD redirects to localhost with authorization_code
5. Exchange code + code_verifier → POST /token endpoint
6. Receive access_token + refresh_token
7. Store tokens securely in OS credential manager
8. Emit ConnectorStateChanged::Authenticated event
```

### Token Refresh Mechanism

```text
1. Before each Graph API call, check token.expires_at
2. If expires_at - now < refresh_margin (default: 5 minutes):
   a. POST /token with grant_type=refresh_token
   b. Update stored access_token and new refresh_token
   c. Reset expires_at
   d. If refresh fails (revoked, expired refresh token):
      - Emit ConnectorStateChanged::AuthenticationRequired
      - Queue pending operations for retry after re-auth
3. If token is already expired and refresh unavailable:
   - Return ConnectorError::AuthenticationRequired
   - Connector enters Disconnected state
```

---

## 8. Error Mapping Strategy

| HTTP Status | ConnectorError Variant | Retryable |
|-------------|----------------------|-----------|
| 401 Unauthorized | `AuthenticationRequired` | Yes (after re-auth) |
| 403 Forbidden | `AccessDenied` | No |
| 404 Not Found | `NotFound` | No |
| 409 Conflict | `Conflict` | No (requires resolution) |
| 423 Locked | `ItemLocked` | Yes (after delay) |
| 429 Too Many Requests | `Throttled` | Yes (Retry-After) |
| 502/503/504 | `ServiceUnavailable` | Yes (exponential backoff) |
| 507 Insufficient Storage | `QuotaExceeded` | No |

---

## 9. Non-Goals for Initial Design

- Google Drive, Dropbox, AWS S3, Azure Blob — future expansion, not designed here
- Real-time co-authoring via WebSocket (Graph subscriptions may be added later)
- Offline-first mode with full local replica — only a metadata/content cache is planned

---

## 10. Open Questions (To Be Resolved Before Implementation)

1. Should the connector support multiple simultaneous authenticated accounts?
2. What is the maximum cache size default — 500 MB? 1 GB? User-configurable?
3. Should delta sync polling interval be configurable per-connection or global?
4. How should the connector handle Azure AD Conditional Access policies that
   require device compliance or MFA step-up?
5. Should VFS watch (file change notifications) be implemented via Graph
   webhooks or polling-based delta queries?
