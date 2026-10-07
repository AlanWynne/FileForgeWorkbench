//! # Backend Command Environment contract (CR-CH-053 Task 21)
//!
//! A [`BackendEnvironment`] is the contract a BACKEND / resource Command
//! Environment implements -- the file-system (and later dataset / database)
//! environments that OWN a resource's store semantics. It is deliberately
//! distinct from the shell-side interactive `CommandEnvironment` (FFEDIT / FFNAV
//! / FFCMD), which carries a Workspace Context the user focuses and mutates
//! shell-entangled state.
//!
//! ## Interactive vs backend environments (the two-category model)
//!
//! The CR-CH-053 model has TWO kinds of Command Environment:
//!
//! - **Interactive / context environments** (FFEDIT, FFNAV, FFCMD): the user
//!   FOCUSES a Workspace Context that makes the environment active; its verbs are
//!   UI-facing and operate on shell state. These implement the shell-side
//!   `CommandEnvironment` trait (which takes `&mut WorkbenchShell`) and can be
//!   reached by the active-env gate AND by addressing.
//!
//! - **Backend / resource environments** (`ff-ce-ntfs`, `ff-ce-posix`, the
//!   mainframe/VSAM CE, a future `ff-ce-sqlite`): NO user ever focuses them --
//!   they are reached ONLY by ADDRESSING (`dispatch_to_environment(name, cmd)`)
//!   from another environment (e.g. FFEDIT addresses SAVE to the owning backend;
//!   a macro `ADDRESS <name>`; the mainframe CE addressing `ff-ce-sqlite`). They
//!   are LIGHT from FFWB's perspective (no Context, no shell dependency) but may
//!   be RICH in backend command support. They implement THIS trait.
//!
//! ## No central hub
//!
//! This trait is a pure CONTRACT. It knows about NO concrete backend. Backend CEs
//! REGISTER THEMSELVES by name into the shell's open `Environment_Registry`
//! (CR-CH-053 Task 13); anyone addresses them BY NAME through that registry. No
//! crate holds a list of backends, so adding a new backend (e.g. `ff-ce-sqlite`)
//! is a new crate that registers itself -- zero edits elsewhere. The trait lives
//! in `ff-vfs` (the storage-abstraction crate every backend already depends on),
//! NOT a dedicated hub crate that would need maintenance per backend.
//!
//! A future configurable "addressable environments" Context (owner idea, logged
//! as a pending CR) would READ/manage that same open registry; this contract does
//! not foreclose it.

use std::path::Path;

/// The outcome of a backend store operation, carrying a return code so the
/// addressing caller (FFEDIT's SAVE forwarding, a macro `ADDRESS`) can map it to
/// a status / macro `RC`. Mirrors the shell-side dispatch outcome's intent at the
/// backend boundary.
#[derive(Debug)]
pub enum BackendOutcome {
    /// The backend handled the operation. `rc` is 0 on success (the macro-RC
    /// convention); a non-zero code is a backend-reported soft failure.
    Handled { rc: i32 },
    /// The backend does not own / recognise the requested verb (the caller may
    /// fall through or report unresolved).
    NotHandled,
}

/// A backend / resource Command Environment: owns the store semantics for one
/// file system (or dataset / database) kind. Implemented by the `ff-ce-*` crates
/// and the mainframe/VSAM CE; shell-free (no `WorkbenchShell`, no egui), so these
/// crates are pure leaves depending only on `ff-vfs` + std.
///
/// Validates: command-environments Requirement 16.1, 16.4, 16.7
pub trait BackendEnvironment: Send + Sync {
    /// The stable NAME this backend registers under and is addressed by (e.g.
    /// "NTFS", "POSIX", "HOSTFS", later "MAINFRAME" / "SQLITE"). Case-insensitive
    /// at the addressing boundary.
    fn name(&self) -> &str;

    /// Whether this file system treats paths case-sensitively (a cheap FS default
    /// the light host CEs expose: NTFS = false, POSIX = true). Advisory metadata;
    /// richer attributes arrive with the heavier backends.
    fn is_case_sensitive(&self) -> bool;

    /// Perform the backend's store WRITE for a resource: write `bytes` to the
    /// resource identified by `path`. For a light host FS CE this is a plain
    /// OS-backed byte write (identical to the former direct local-FS save); a
    /// mainframe backend would instead pack records per RECFM/LRECL. The caller
    /// (FFEDIT via `dispatch_to_environment`) owns the dirty-aware orchestration
    /// (read the buffer, clear the modified flag, set the save point); this method
    /// owns ONLY the physical write, so the ownership boundary (Req 14.7) is clean.
    ///
    /// Validates: command-environments Requirement 14.5, 14.6, 16.5
    fn save(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()>;
}
