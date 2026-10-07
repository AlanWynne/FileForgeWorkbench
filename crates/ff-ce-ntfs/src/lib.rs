//! # `ff-ce-ntfs` -- the NTFS backend Command Environment (CR-CH-053 Task 21)
//!
//! A LIGHT, OS-backed backend Command Environment for the Windows NTFS file
//! system. It implements the shell-free [`ff_vfs::BackendEnvironment`] contract
//! and nothing more: no Workspace Context, no `ff-desktop` dependency, no egui.
//! The host-fs decider (`ff-ce-host-fs`) resolves the "native" role to this
//! environment on Windows; FFEDIT then ADDRESSes store verbs (SAVE) to it by name
//! through the shell's open `Environment_Registry`.
//!
//! "Light" means it relies on the OS for controls/attributes and performs a plain
//! byte write (byte-identical to the former direct local-FS save). The only
//! FS-specific default it exposes is case-INSENSITIVITY (NTFS default). Deeper
//! NTFS semantic emulation (alternate data streams, ACLs, cross-emulation on a
//! non-Windows host) is DEFERRED (Req 16.6) and would be a richer future build or
//! a plugin.

use std::path::Path;

use ff_vfs::BackendEnvironment;

/// The stable addressing NAME for the NTFS backend environment.
pub const NTFS_ENVIRONMENT_NAME: &str = "NTFS";

/// The NTFS backend Command Environment (light, OS-backed).
///
/// Zero-sized: it holds no state; the resource it operates on is passed to each
/// call. Register one instance by name into the shell's `Environment_Registry`.
#[derive(Debug, Default, Clone, Copy)]
pub struct NtfsEnvironment;

impl BackendEnvironment for NtfsEnvironment {
    fn name(&self) -> &str {
        NTFS_ENVIRONMENT_NAME
    }

    /// NTFS is case-INSENSITIVE (preserving) by default.
    fn is_case_sensitive(&self) -> bool {
        false
    }

    /// Plain OS-backed byte write -- the physical store write for a host NTFS
    /// resource. Byte-identical to the former local-FS save; the dirty-aware
    /// orchestration stays with the addressing caller (Req 14.7).
    ///
    /// Validates: command-environments Requirement 16.5
    fn save(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        std::fs::write(path, bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates: command-environments Requirement 16.1 -- the NTFS backend CE
    /// reports its stable addressing name.
    #[test]
    fn reports_ntfs_name() {
        assert_eq!(NtfsEnvironment.name(), "NTFS");
    }

    /// Validates: command-environments Requirement 16.4 -- NTFS default is
    /// case-insensitive.
    #[test]
    fn ntfs_is_case_insensitive() {
        assert!(!NtfsEnvironment.is_case_sensitive());
    }

    /// Validates: command-environments Requirement 16.5 -- the NTFS SAVE performs
    /// a plain byte write (OS-backed), identical to a local-FS write.
    #[test]
    fn save_writes_bytes_to_disk() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let file = dir.path().join("out.txt");
        NtfsEnvironment.save(&file, b"ntfs bytes").expect("write");
        assert_eq!(std::fs::read(&file).expect("read"), b"ntfs bytes");
    }
}
