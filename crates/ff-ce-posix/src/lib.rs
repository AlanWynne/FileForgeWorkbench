//! # `ff-ce-posix` -- the POSIX backend Command Environment (CR-CH-053 Task 21)
//!
//! A LIGHT, OS-backed backend Command Environment for POSIX file systems
//! (Linux / macOS hosts). It implements the shell-free
//! [`ff_vfs::BackendEnvironment`] contract and nothing more: no Workspace
//! Context, no `ff-desktop` dependency, no egui. The host-fs decider
//! (`ff-ce-host-fs`) resolves the "native" role to this environment on
//! Linux/macOS; FFEDIT ADDRESSes store verbs (SAVE) to it by name through the
//! shell's open `Environment_Registry`.
//!
//! "Light" means it relies on the OS for controls/attributes and performs a plain
//! byte write (byte-identical to the former direct local-FS save). The only
//! FS-specific default it exposes is case-SENSITIVITY (POSIX default). Deeper
//! POSIX/APFS semantic emulation and cross-emulation on a non-POSIX host are
//! DEFERRED (Req 16.6) and would be a richer future build or a plugin.

use std::path::Path;

use ff_vfs::BackendEnvironment;

/// The stable addressing NAME for the POSIX backend environment.
pub const POSIX_ENVIRONMENT_NAME: &str = "POSIX";

/// The POSIX backend Command Environment (light, OS-backed).
///
/// Zero-sized: it holds no state; the resource it operates on is passed to each
/// call. Register one instance by name into the shell's `Environment_Registry`.
#[derive(Debug, Default, Clone, Copy)]
pub struct PosixEnvironment;

impl BackendEnvironment for PosixEnvironment {
    fn name(&self) -> &str {
        POSIX_ENVIRONMENT_NAME
    }

    /// POSIX file systems are case-SENSITIVE by default.
    fn is_case_sensitive(&self) -> bool {
        true
    }

    /// Plain OS-backed byte write -- the physical store write for a host POSIX
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

    /// Validates: command-environments Requirement 16.1 -- the POSIX backend CE
    /// reports its stable addressing name.
    #[test]
    fn reports_posix_name() {
        assert_eq!(PosixEnvironment.name(), "POSIX");
    }

    /// Validates: command-environments Requirement 16.4 -- POSIX default is
    /// case-sensitive.
    #[test]
    fn posix_is_case_sensitive() {
        assert!(PosixEnvironment.is_case_sensitive());
    }

    /// Validates: command-environments Requirement 16.5 -- the POSIX SAVE performs
    /// a plain byte write (OS-backed), identical to a local-FS write.
    #[test]
    fn save_writes_bytes_to_disk() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let file = dir.path().join("out.txt");
        PosixEnvironment.save(&file, b"posix bytes").expect("write");
        assert_eq!(std::fs::read(&file).expect("read"), b"posix bytes");
    }
}
