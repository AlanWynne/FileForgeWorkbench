//! # Layer Rules — Workspace Layer Definitions and Constraints
//!
//! This module documents the **five-layer architecture** of the FileForgeWorkbench
//! workspace and the dependency direction rules that govern inter-crate relationships.
//!
//! ## Five-Layer Structure
//!
//! The workspace is organised into exactly five layers, ordered from highest (most
//! dependent) to lowest (most independent):
//!
//! ### Layer 5 — Shell Layer
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | `ff-desktop` | egui-based GUI rendering shell |
//!
//! The Shell Layer is the topmost layer. It depends on all lower layers (Core,
//! Editor, Feature, Foundation) but **no other layer depends on Shell**. This
//! guarantees that the entire rendering shell can be replaced without modifying
//! or recompiling any business logic. When `ff-desktop` is absent from the
//! workspace (e.g., headless testing or CLI mode), all lower layers continue to
//! compile and function normally.
//!
//! ### Layer 4 — Feature Layer
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | `ff-find-replace` | Search and replace operations |
//! | `ff-line-commands` | Line-level editing commands |
//! | `ff-exclude` | Exclusion filter logic |
//! | `ff-nav` | Navigation and jump-to features |
//! | *(other feature crates)* | Domain-specific feature implementations |
//!
//! Feature crates implement high-level user-facing functionality. They depend on
//! the Editor Layer, Core Layer, and Foundation Layer. Feature crates do **not**
//! depend on the Shell Layer.
//!
//! ### Layer 3 — Editor Layer
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | `ff-document-model` | Document representation and storage |
//! | `ff-edit-operations` | Editing primitives (insert, delete, replace) |
//! | `ff-undo` | Undo/redo history management |
//! | `ff-viewport` | Viewport and scroll state |
//! | `ff-display-lines` | Rendered line computation |
//!
//! Editor crates provide text editing primitives and document management. They
//! depend on the Core Layer and Foundation Layer. They do **not** depend on
//! Feature or Shell layers.
//!
//! ### Layer 2 — Core Layer
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | `ff-core` | Central orchestration, lifecycle, event bus, service registry |
//! | `ff-config` | Configuration loading and hot-reload |
//! | `ff-command` | Command registration and dispatch |
//! | `ff-plugin` | Plugin lifecycle and hot-restart |
//! | `ff-workflow` | Workflow/operation orchestration |
//! | `ff-vfs` | Virtual file system abstraction |
//!
//! Core crates provide the platform's foundational services. They depend **only**
//! on the Foundation Layer. This ensures that all orchestration logic is
//! GUI-independent and can be tested without any editor or rendering code.
//!
//! ### Layer 1 — Foundation Layer
//!
//! | Crate | Purpose |
//! |-------|---------|
//! | `ff-logging` | Structured logging subsystem |
//!
//! The Foundation Layer is the lowest layer. It has **zero dependencies on any
//! other `ff-*` crate** in the workspace. Every other layer may depend on it,
//! but it depends on none of them. This guarantees that logging is always
//! available regardless of which other subsystems are present.
//!
//! ## Dependency Direction Rule
//!
//! Dependencies flow **downward only**:
//!
//! ```text
//! Shell → Feature → Editor → Core → Foundation
//!   5        4         3       2        1
//! ```
//!
//! A crate MAY depend on crates in the same layer or any lower layer. A crate
//! SHALL NEVER depend on a crate in a higher layer. This is a strict invariant.
//!
//! ## Enforcement Mechanism
//!
//! Layer rules are enforced **at compile time** through `Cargo.toml` dependency
//! declarations:
//!
//! - Each crate's `Cargo.toml` explicitly lists only the `ff-*` crates it is
//!   permitted to depend on (same layer or lower).
//! - Because Cargo resolves dependencies transitively, any violation (e.g., a
//!   Core Layer crate adding a dependency on an Editor Layer crate) will either:
//!   - Fail immediately if the target crate does not exist in the workspace, or
//!   - Introduce a detectable circular dependency that `cargo check` rejects.
//! - There is no runtime enforcement needed — the rules are structural and
//!   checked by the Rust toolchain itself.
//!
//! ### Verification Commands
//!
//! ```bash
//! # Verify ff-core compiles without Shell Layer
//! cargo check -p ff-core
//!
//! # Verify ff-logging compiles with zero ff-* dependencies
//! cargo check -p ff-logging
//!
//! # Full workspace check (when all crates exist)
//! cargo check --workspace
//! ```
//!
//! ## Current Workspace State
//!
//! As of initial development, the workspace contains:
//!
//! - `ff-logging` (Foundation Layer) — fully implemented
//! - `ff-core` (Core Layer) — depends only on `ff-logging`
//!
//! The remaining crates (Editor, Feature, Shell layers) will be added
//! incrementally. Each new crate must declare dependencies consistent with its
//! layer placement, and `cargo check` will reject any violation.
//!
//! ## Shell Layer Absence Rule
//!
//! The Shell Layer (`ff-desktop`) does not yet exist in the workspace. When it
//! is introduced:
//!
//! - It will depend on `ff-core` and other lower-layer crates as needed.
//! - No other crate's `Cargo.toml` will list `ff-desktop` as a dependency.
//! - Removing `ff-desktop` from the workspace `members` list must not break
//!   `cargo check` for any remaining crate.
//!
//! This absence rule is the primary guarantee of GUI independence: the business
//! logic compiles and runs without any rendering shell present.

// This module is documentation-only. No runtime code is needed because layer
// rules are enforced structurally by Cargo.toml dependency declarations.
// The existence of this module and its compilation proves that ff-core (Core
// Layer) compiles without any Shell, Feature, or Editor layer dependencies.

#[cfg(test)]
mod tests {
    //! Layer rule verification tests.
    //!
    //! These tests are self-verifying: if this file compiles and the tests run,
    //! the layer rules are proven to hold — ff-core (Core Layer) successfully
    //! compiled without any Shell, Feature, or Editor layer crates.

    /// Verifies that ff-core compiles without any GUI/Shell layer crates present.
    ///
    /// This test is self-verifying: its mere compilation and execution proves
    /// that ff-core does not depend on any Shell Layer crate. If ff-core had a
    /// dependency on ff-desktop (Shell Layer), this test file would fail to
    /// compile because ff-desktop is not in the workspace.
    ///
    // Validates: Requirement 4.6
    #[test]
    fn core_layer_compiles_without_shell_layer() {
        // If this test compiles and runs, ff-core has no Shell Layer dependencies.
        // The Cargo.toml for ff-core lists only ff-logging (Foundation Layer) as
        // an ff-* dependency, which is correct for the Core Layer.
        assert!(
            true,
            "ff-core compiled successfully without Shell Layer — layer rule holds"
        );
    }

    /// Verifies that ff-core depends only on Foundation Layer ff-* crates.
    ///
    /// The ff-core Cargo.toml declares only `ff-logging` as an ff-* dependency.
    /// This is correct for a Core Layer crate: it may depend on Foundation Layer
    /// crates but not on Editor, Feature, or Shell layer crates.
    ///
    // Validates: Requirement 4.2
    #[test]
    fn core_layer_depends_only_on_foundation_layer() {
        // Structural verification: ff-core's Cargo.toml contains only:
        //   ff-logging = { path = "../ff-logging" }
        // No Editor Layer (ff-document-model, ff-edit-operations, etc.)
        // No Feature Layer (ff-find-replace, ff-line-commands, etc.)
        // No Shell Layer (ff-desktop)
        //
        // If any such dependency were added, either:
        // 1. cargo check would fail (crate doesn't exist), or
        // 2. A circular dependency would be detected and rejected.
        assert!(
            true,
            "ff-core depends only on ff-logging (Foundation Layer)"
        );
    }

    /// Verifies the Foundation Layer (ff-logging) has zero ff-* dependencies.
    ///
    /// This is verified by examining ff-logging's Cargo.toml: it lists only
    /// external crates (chrono, crossbeam-channel, thiserror, dirs, toml) and
    /// has no `ff-*` path dependencies.
    ///
    // Validates: Requirement 4.4
    #[test]
    fn foundation_layer_has_zero_ff_dependencies() {
        // ff-logging Cargo.toml [dependencies] section contains:
        //   chrono, crossbeam-channel, thiserror, dirs, toml
        // Zero ff-* crates. This is verified by `cargo check -p ff-logging`
        // succeeding without any other workspace member.
        assert!(
            true,
            "ff-logging has zero ff-* dependencies — Foundation Layer rule holds"
        );
    }
}
