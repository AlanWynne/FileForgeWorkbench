# Implementation Plan: Connector Extensibility (`ff-connector-extensibility`)

## Overview

This plan implements the connector extensibility framework for FileForgeWorkbench — the plugin trait, registration protocol, capability advertisement, lifecycle state machine, authentication framework, and error types that all future VFS connectors must use. No concrete connector implementations are included (those are deferred); this crate defines the contracts and infrastructure.

**Crate location:** `crates/ff-connector-extensibility`
**Dependencies:** `ff-vfs` (VfsProvider), `ff-plugin` (FileForgePlugin), `ff-core` (EventBus), `ff-logging`

---

## Tasks

- [x] 1. Project scaffolding and crate setup
  - [x] 1.1 Create `crates/ff-connector-extensibility/Cargo.toml` with dependencies on ff-vfs, ff-plugin, ff-core, ff-logging, async-trait, thiserror, zeroize, tokio-util
  - [x] 1.2 Create `src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Verify crate compiles with `cargo check -p ff-connector-extensibility`

- [x] 2. ApiVersion type and compatibility checking
  - [x] 2.1 Create `src/api_version.rs` with `ApiVersion` struct (major, minor, patch), `CONNECTOR_API_VERSION` constant, and `is_compatible_with` method
  - [x] 2.2 Implement `Display`, `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash` for `ApiVersion`
  - [x] 2.3 Write unit tests for `is_compatible_with` — same major + minor ≤ current = compatible; different major or minor > current = incompatible
    - Validates: Requirement 1 AC 4, Requirement 2 AC 2c

- [x] 3. ConnectorCapability enum and validation
  - [x] 3.1 Create `src/capability.rs` with `ConnectorCapability` enum (`Read`, `Write`, `Watch`, `Search`, `Rename`, `Delete`, `CreateDirectory`, `Metadata`, `List`, `Copy`) marked `#[non_exhaustive]`
  - [x] 3.2 Define `REQUIRED_CAPABILITIES` constant (`Read`, `List`, `Metadata`)
  - [x] 3.3 Implement `validate_capabilities(capabilities: &[ConnectorCapability]) -> Result<(), ConnectorError>` function
  - [x] 3.4 Write unit tests for `validate_capabilities` — passes when all required present, fails when any missing
    - Validates: Requirement 3 AC 1, AC 2

- [x] 4. ConnectorError enum and error mapping
  - [x] 4.1 Create `src/error.rs` with full `ConnectorError` enum (NotConnected, AuthenticationFailed, PermissionDenied, ResourceNotFound, ResourceAlreadyExists, Timeout, NetworkError, UnsupportedOperation, RegistrationFailed, ProviderSpecific, Internal)
  - [x] 4.2 Implement `Display` following `[connector:{scheme}] {operation}: {description}` format
  - [x] 4.3 Implement `is_retryable(&self) -> bool` and `should_reconnect(&self) -> bool` classification methods
  - [x] 4.4 Implement `From<std::io::Error>` mapping (PermissionDenied, NotFound, TimedOut → corresponding variants)
  - [x] 4.5 Implement `std::error::Error` with `source()` chain for `ProviderSpecific` variant
  - [x] 4.6 Write unit tests for Display format, retryability classification, and From<io::Error> mapping
    - Validates: Requirement 7 AC 1–6

- [x] 5. ConnectorDescriptor metadata type
  - [x] 5.1 Create `src/descriptor.rs` with `ConnectorDescriptor` struct (scheme, display_name, description, icon, version)
  - [x] 5.2 Derive `Debug`, `Clone`, `PartialEq`, `Eq`
    - Validates: Requirement 1 AC 2

- [x] 6. ConnectorState lifecycle enum and transition validation
  - [x] 6.1 Create `src/state.rs` with `ConnectorState` enum (Registered, Connecting, Connected, Disconnecting, Disconnected, Error)
  - [x] 6.2 Implement `is_valid_transition(from: &ConnectorState, to: &ConnectorState) -> bool` enforcing the state machine rules
  - [x] 6.3 Implement helper methods: `is_connected()`, `is_operational()`, `can_connect()`, `can_disconnect()`
  - [x] 6.4 Write unit tests for valid and invalid transitions
    - Validates: Requirement 4 AC 1, AC 2

- [x] 7. RetryPolicy and ReconnectionManager
  - [x] 7.1 Create `src/reconnection.rs` with `RetryPolicy` struct (max_retries, initial_backoff, max_backoff, use_jitter) and `Default` impl
  - [x] 7.2 Implement `RetryPolicy::compute_backoff(attempt: u32) -> Duration` with exponential backoff capped at max_backoff
  - [x] 7.3 Implement `RetryPolicy::allows_retry(&self) -> bool`
  - [x] 7.4 Create `ReconnectionManager` struct with attempt tracking, cancellation token, and `next_backoff()` / `reset()` methods
  - [x] 7.5 Write unit tests for backoff computation, cap enforcement, and retry allowance
    - Validates: Requirement 4 AC 4, AC 5

- [x] 8. Credential types and CredentialStore trait
  - [x] 8.1 Create `src/credential.rs` with `SecureString` and `SecureBytes` types wrapping `zeroize::Zeroizing`
  - [x] 8.2 Implement `Credential` enum (Password, KeyBased, OAuth, Token) with `#[non_exhaustive]`
  - [x] 8.3 Define `CredentialStore` trait with `store`, `retrieve`, `delete`, `exists`, `refresh_credential` methods
  - [x] 8.4 Implement `Debug` for `SecureString`/`SecureBytes` that masks values (never prints plaintext)
  - [x] 8.5 Write unit tests for SecureString zeroize-on-drop and masked Debug output
    - Validates: Requirement 5 AC 1, AC 2, AC 5, AC 6

- [x] 9. ConnectorPlugin trait definition
  - [x] 9.1 Create `src/traits.rs` with the full `ConnectorPlugin` trait combining `VfsProvider + FileForgePlugin` with connector-specific methods
  - [x] 9.2 Add async methods: `connect`, `disconnect`, `authenticate`
  - [x] 9.3 Add query methods: `descriptor`, `connector_capabilities`, `api_version`, `state`, `retry_policy`
  - [x] 9.4 Add `map_error` method and `custom_operation` with default UnsupportedOperation impl
  - [x] 9.5 Verify object-safety — write a compile-time test that `Box<dyn ConnectorPlugin>` compiles
  - [x] 9.6 Add comprehensive doc comments documenting FTP/SFTP, z/OS, and cloud mapping guidance
    - Validates: Requirement 1 AC 1, AC 3–6, Requirement 6 AC 1–6

- [x] 10. Custom operation types
  - [x] 10.1 Create `src/custom_op.rs` with `CustomOperationRequest` and `CustomOperationResponse` helper types
  - [x] 10.2 Document the z/OS-specific custom operation patterns (JES spool, job submission)
    - Validates: Requirement 6 AC 3, AC 6

- [x] 11. Platform events
  - [x] 11.1 Create `src/event.rs` with `ConnectorRegisteredEvent`, `ConnectorStateChangedEvent`, and `ConnectorCapabilityChangedEvent` structs
  - [x] 11.2 Implement `Debug`, `Clone` on all event types
    - Validates: Requirement 2 AC 6, Requirement 3 AC 6, Requirement 4 AC 3

- [x] 12. ConnectorRegistry implementation
  - [x] 12.1 Create `src/registry.rs` with `ConnectorRegistry` struct (connectors map, vfs_registry, event_bus, reconnection_managers)
  - [x] 12.2 Implement `register()` — validate scheme uniqueness, required capabilities, API version compatibility; register with VFS ProviderRegistry; emit event
  - [x] 12.3 Implement `deregister()` — disconnect if connected, remove from VFS registry, emit event
  - [x] 12.4 Implement `hot_swap()` — deactivate old connector, register new version, preserve URI resolution
  - [x] 12.5 Implement `get_connector()`, `supports()`, `capabilities_for()`, `refresh_capabilities()`
  - [x] 12.6 Implement `connect()`, `disconnect()`, `shutdown_all()` lifecycle operations
  - [x] 12.7 Implement `list_connectors()` returning all schemes with their states
  - [x] 12.8 Write unit tests for registration validation (scheme uniqueness, capability check, version check)
    - Validates: Requirement 2 AC 1–7, Requirement 3 AC 3–6, Requirement 4 AC 6–8

- [x] 13. Integration tests
  - [x] 13.1 Create `tests/integration.rs` with a mock connector implementing `ConnectorPlugin`
  - [x] 13.2 Test end-to-end flow: register → connect → operations → disconnect → deregister
  - [x] 13.3 Test error cases: duplicate registration, missing capabilities, incompatible version
  - [x] 13.4 Test hot-swap flow: register v1 → hot_swap v2 → verify new connector active
    - Validates: Requirement 1 AC 1–6, Requirement 2 AC 1–7

- [x] 14. Property-based tests
  - [x] 14.1 Write property test: Registration Uniqueness — no duplicate schemes in registry after arbitrary register/deregister sequences
    - Validates: Requirement 2 AC 2a
    - **Property 1 from design.md**
  - [x] 14.2 Write property test: Required Capabilities Enforcement — registration succeeds iff Read ∈ C ∧ List ∈ C ∧ Metadata ∈ C
    - Validates: Requirement 3 AC 2
    - **Property 2 from design.md**
  - [x] 14.3 Write property test: API Version Compatibility — compatible iff same major and minor ≤ current
    - Validates: Requirement 1 AC 4, Requirement 2 AC 2c
    - **Property 3 from design.md**
  - [x] 14.4 Write property test: State Machine Validity — only valid transitions succeed, invalid transitions rejected
    - Validates: Requirement 4 AC 1, AC 2
    - **Property 4 from design.md**
  - [x] 14.5 Write property test: Exponential Backoff Monotonicity — backoff values non-decreasing until cap reached
    - Validates: Requirement 4 AC 4, AC 5
    - **Property 5 from design.md**
  - [x] 14.6 Write property test: Capability Query Consistency — supports() matches set membership, capabilities_for() returns exact set
    - Validates: Requirement 3 AC 3, AC 5
    - **Property 6 from design.md**
  - [x] 14.7 Write property test: Error Retryability Classification — is_retryable() and should_reconnect() match specification for all variants
    - Validates: Requirement 7 AC 2
    - **Property 7 from design.md**
  - [x] 14.8 Write property test: Credential Scoping Isolation — credentials retrievable only with exact key, no cross-scope leakage
    - Validates: Requirement 5 AC 7
    - **Property 8 from design.md**
  - [x] 14.9 Write property test: Disconnected Connector Operation Rejection — all VFS ops return NotConnected when connector in Disconnected/Error state
    - Validates: Requirement 4 AC 7
    - **Property 9 from design.md**
  - [x] 14.10 Write property test: ConnectorError Display Format Compliance — Display matches regex pattern and length ≤ 200 chars
    - Validates: Requirement 7 AC 6, cross-cutting Req 8
    - **Property 10 from design.md**

- [x] 15. Documentation and final validation
  - [x] 15.1 Add crate-level documentation in `src/lib.rs` describing purpose, architecture position, and usage for future connector authors
  - [x] 15.2 Add `README.md` in crate directory with quick-start guide for implementing a new connector
  - [x] 15.3 Verify all public items have `///` doc comments
  - [x] 15.4 Run `cargo clippy -p ff-connector-extensibility -- -D warnings` clean
  - [x] 15.5 Run `cargo test -p ff-connector-extensibility` with all tests passing

---

## Acceptance Criteria Coverage Map

| Requirement | AC | Covered By Task(s) |
|---|---|---|
| Req 1: Connector Plugin Trait | AC 1 | 9.1, 13.1 |
| Req 1: Connector Plugin Trait | AC 2 | 5.1, 9.3 |
| Req 1: Connector Plugin Trait | AC 3 | 9.3, 14.6 |
| Req 1: Connector Plugin Trait | AC 4 | 2.1, 2.3, 14.3 |
| Req 1: Connector Plugin Trait | AC 5 | 9.5 |
| Req 1: Connector Plugin Trait | AC 6 | 9.2, 13.2 |
| Req 2: Provider Registration | AC 1 | 12.2, 13.2 |
| Req 2: Provider Registration | AC 2a | 12.2, 12.8, 14.1 |
| Req 2: Provider Registration | AC 2b | 12.2, 12.8, 14.2 |
| Req 2: Provider Registration | AC 2c | 12.2, 12.8, 14.3 |
| Req 2: Provider Registration | AC 3 | 12.2, 13.3 |
| Req 2: Provider Registration | AC 4 | 12.3, 13.2 |
| Req 2: Provider Registration | AC 5 | 12.4, 13.4 |
| Req 2: Provider Registration | AC 6 | 11.1, 12.2, 12.3 |
| Req 2: Provider Registration | AC 7 | 12.5 |
| Req 3: Capability Advertisement | AC 1 | 3.1 |
| Req 3: Capability Advertisement | AC 2 | 3.2, 3.3, 3.4, 14.2 |
| Req 3: Capability Advertisement | AC 3 | 12.5, 14.6 |
| Req 3: Capability Advertisement | AC 4 | 4.1 (UnsupportedOperation variant) |
| Req 3: Capability Advertisement | AC 5 | 12.5, 14.6 |
| Req 3: Capability Advertisement | AC 6 | 11.1, 12.5 |
| Req 4: Provider Lifecycle | AC 1 | 6.1 |
| Req 4: Provider Lifecycle | AC 2 | 6.3, 9.3, 14.4 |
| Req 4: Provider Lifecycle | AC 3 | 11.1, 12.6 |
| Req 4: Provider Lifecycle | AC 4 | 7.1, 7.2, 7.3, 14.5 |
| Req 4: Provider Lifecycle | AC 5 | 7.4, 14.5 |
| Req 4: Provider Lifecycle | AC 6 | 12.6 |
| Req 4: Provider Lifecycle | AC 7 | 14.9 |
| Req 4: Provider Lifecycle | AC 8 | 12.6 |
| Req 5: Authentication Framework | AC 1 | 8.3 |
| Req 5: Authentication Framework | AC 2 | 8.2 |
| Req 5: Authentication Framework | AC 3 | 9.2 |
| Req 5: Authentication Framework | AC 4 | 8.3 (refresh_credential method) |
| Req 5: Authentication Framework | AC 5 | 8.4 |
| Req 5: Authentication Framework | AC 6 | 8.1, 8.5 |
| Req 5: Authentication Framework | AC 7 | 14.8 |
| Req 6: Future Connector Hooks | AC 1 | 9.1 |
| Req 6: Future Connector Hooks | AC 2 | 9.6 |
| Req 6: Future Connector Hooks | AC 3 | 9.4, 10.1, 10.2 |
| Req 6: Future Connector Hooks | AC 4 | 9.6 |
| Req 6: Future Connector Hooks | AC 5 | 9.6 |
| Req 6: Future Connector Hooks | AC 6 | 9.4, 10.1 |
| Req 7: Error Mapping | AC 1 | 4.1 |
| Req 7: Error Mapping | AC 2 | 4.3, 4.6, 14.7 |
| Req 7: Error Mapping | AC 3 | 4.4, 4.6 |
| Req 7: Error Mapping | AC 4 | 4.1 (context fields in error variants) |
| Req 7: Error Mapping | AC 5 | 4.5 |
| Req 7: Error Mapping | AC 6 | 4.2, 4.6, 14.10 |
| Req 7: Error Mapping | AC 7 | 9.4 |

---

## Property-Based Test Summary

| # | Property | Strategy | Validates |
|---|----------|----------|-----------|
| 1 | Registration Uniqueness | Random register/deregister sequences, verify no duplicate schemes | Req 2 AC 2a |
| 2 | Required Capabilities Enforcement | All subsets of ConnectorCapability, assert registration iff required present | Req 3 AC 2 |
| 3 | API Version Compatibility | Random (connector, current) version pairs, verify compatibility formula | Req 1 AC 4, Req 2 AC 2c |
| 4 | State Machine Validity | Random transition sequences, verify only valid transitions succeed | Req 4 AC 1, AC 2 |
| 5 | Exponential Backoff Monotonicity | Random RetryPolicy configs, verify monotonic non-decreasing capped at max | Req 4 AC 4, AC 5 |
| 6 | Capability Query Consistency | Random capability sets, verify supports() matches membership | Req 3 AC 3, AC 5 |
| 7 | Error Retryability Classification | All ConnectorError variants, verify is_retryable/should_reconnect match spec | Req 7 AC 2 |
| 8 | Credential Scoping Isolation | Random (scheme, connection) keys, verify no cross-scope access | Req 5 AC 7 |
| 9 | Disconnected Connector Operation Rejection | Arbitrary VFS ops on Disconnected/Error connectors return NotConnected | Req 4 AC 7 |
| 10 | ConnectorError Display Format Compliance | Random error instances, verify regex match and length ≤ 200 | Req 7 AC 6, Req 8 |

---

## Task Dependency Graph

```json
{
  "tasks": [
    { "id": "1", "label": "Project scaffolding and crate setup", "dependsOn": [] },
    { "id": "2", "label": "ApiVersion type and compatibility", "dependsOn": ["1"] },
    { "id": "3", "label": "ConnectorCapability enum and validation", "dependsOn": ["1"] },
    { "id": "4", "label": "ConnectorError enum and error mapping", "dependsOn": ["1"] },
    { "id": "5", "label": "ConnectorDescriptor metadata type", "dependsOn": ["1"] },
    { "id": "6", "label": "ConnectorState lifecycle enum", "dependsOn": ["4"] },
    { "id": "7", "label": "RetryPolicy and ReconnectionManager", "dependsOn": ["4"] },
    { "id": "8", "label": "Credential types and CredentialStore trait", "dependsOn": ["4"] },
    { "id": "9", "label": "ConnectorPlugin trait definition", "dependsOn": ["2", "3", "4", "5", "6", "7", "8"] },
    { "id": "10", "label": "Custom operation types", "dependsOn": ["4", "9"] },
    { "id": "11", "label": "Platform events", "dependsOn": ["6"] },
    { "id": "12", "label": "ConnectorRegistry implementation", "dependsOn": ["9", "11"] },
    { "id": "13", "label": "Integration tests", "dependsOn": ["12"] },
    { "id": "14", "label": "Property-based tests", "dependsOn": ["12", "13"] },
    { "id": "15", "label": "Documentation and final validation", "dependsOn": ["14"] }
  ]
}
```
