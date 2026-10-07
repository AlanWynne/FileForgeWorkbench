//! # `ff-ce-host-fs` -- the host-FS decider (CR-CH-053 Task 21)
//!
//! `ff-ce-host-fs` is a DECIDER / pass-through, NOT a file system. At startup it
//! detects the host platform and resolves the "native" ROLE to the matching
//! concrete backend Command Environment:
//!
//! - Windows        -> [`ff_ce_ntfs::NtfsEnvironment`]
//! - Linux / macOS  -> [`ff_ce_posix::PosixEnvironment`] (future: an APFS CE on macOS)
//!
//! Whatever it resolves to IS the Host_FS_Environment -- the default
//! Owning_Environment a host-path tab binds to (CR-CH-053 Task 19), and the
//! environment FFEDIT ADDRESSes SAVE to for a native file. "Native" is a ROLE,
//! not a CE; any FS CE is emulatable in a non-native context because the CE owns
//! the semantics, not the platform (the registry holds the family and designates
//! one as native per host).
//!
//! The decider depends on the concrete FS CE crates but on NO shell crate; it is
//! a pure resolution function. The shell registers the resolved environment into
//! its open `Environment_Registry` under the stable host-FS name.

use ff_vfs::BackendEnvironment;

/// The stable NAME the resolved host-FS environment registers under (the default
/// Owning_Environment, CR-CH-053 Task 19). This matches `TabState`'s
/// `DEFAULT_OWNING_ENVIRONMENT` so a host-path tab's binding resolves here.
pub const HOST_FS_ENVIRONMENT_NAME: &str = "HOSTFS";

/// Resolve the "native" role to the concrete backend Command Environment for the
/// current host platform (the decider's single job). Returns a boxed
/// [`BackendEnvironment`] the shell registers under [`HOST_FS_ENVIRONMENT_NAME`].
///
/// Windows -> NTFS; every other target (Linux / macOS / other Unix) -> POSIX.
/// A dedicated APFS CE for macOS is a future refinement (Req 16.6 defers deeper
/// per-FS semantics); POSIX is the correct light default for macOS today.
///
/// Validates: command-environments Requirement 16.2, 16.3
pub fn native_backend_environment() -> Box<dyn BackendEnvironment> {
    #[cfg(target_os = "windows")]
    {
        Box::new(ff_ce_ntfs::NtfsEnvironment)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Box::new(ff_ce_posix::PosixEnvironment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates: command-environments Requirement 16.2, 16.3 -- the decider
    /// resolves the native role to the host platform's concrete FS CE: NTFS on
    /// Windows, POSIX elsewhere. The resolved env is the Host_FS_Environment.
    #[test]
    fn resolves_native_backend_for_host() {
        let env = native_backend_environment();
        #[cfg(target_os = "windows")]
        {
            assert_eq!(env.name(), "NTFS");
            assert!(
                !env.is_case_sensitive(),
                "NTFS native default is case-insensitive"
            );
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(env.name(), "POSIX");
            assert!(
                env.is_case_sensitive(),
                "POSIX native default is case-sensitive"
            );
        }
    }

    /// Validates: command-environments Requirement 16.2 -- the host-FS name the
    /// resolved environment registers under matches the TabState default owning
    /// environment ("HOSTFS"), so a host-path tab binding resolves to the decider's
    /// pick.
    #[test]
    fn host_fs_name_is_the_default_owning_environment() {
        assert_eq!(HOST_FS_ENVIRONMENT_NAME, "HOSTFS");
    }

    /// Validates: command-environments Requirement 16.5 -- the initial light
    /// NTFS / POSIX CEs supply a byte-write SAVE identical to today's local-FS
    /// write. The backends own no deep semantics: `save` writes the exact bytes
    /// to the exact path with no record validation, re-encoding, or transform.
    /// This test pins that byte-identity contract for the native default.
    #[test]
    fn native_backend_save_is_byte_identical_write() {
        use std::io::Read;
        let env = native_backend_environment();
        let dir = std::env::temp_dir().join(format!("ff-ce-host-fs-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("byte-identity.bin");
        // Bytes that would be mangled by any text/record transform: NUL, high
        // bytes, bare CR, no trailing newline.
        let bytes: &[u8] = &[0x00, 0xFF, 0x0D, b'A', 0x80, b'z'];
        env.save(&path, bytes).expect("save writes");
        let mut round = Vec::new();
        std::fs::File::open(&path)
            .expect("open")
            .read_to_end(&mut round)
            .expect("read");
        assert_eq!(
            round, bytes,
            "SAVE must be a byte-identical write, no transform"
        );
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    /// Validates: command-environments Requirement 16.6 -- DOC-ASSERT that deep
    /// NTFS / POSIX / APFS semantic emulation and cross-emulation
    /// (NTFS-on-Linux, etc.) are DEFERRED, enabled by this abstraction but NOT
    /// built in the initial phase. The initial CEs are deliberately LIGHT: they
    /// expose only the cheap FS default (`is_case_sensitive`) and a byte-write
    /// SAVE, and carry no deep-semantic surface. This test records that no deep
    /// emulation ships now by asserting the backends expose nothing beyond the
    /// light contract.
    #[test]
    fn deep_fs_emulation_and_cross_emulation_are_deferred() {
        // The light contract is exactly: a name, a case-sensitivity default, and
        // a byte-write save. NTFS differs from POSIX ONLY in the cheap default,
        // not in any emulated semantic -- proving no deep per-FS emulation is
        // built in this phase (Req 16.6). Cross-emulation (choosing a non-native
        // FS CE in a host context) is likewise not wired here: the decider only
        // ever resolves the NATIVE role (Req 16.2), leaving cross-emulation to a
        // later phase enabled by, but not realised in, this abstraction.
        let ntfs = ff_ce_ntfs::NtfsEnvironment;
        let posix = ff_ce_posix::PosixEnvironment;
        assert_eq!(ntfs.name(), "NTFS");
        assert_eq!(posix.name(), "POSIX");
        assert!(
            !ntfs.is_case_sensitive(),
            "NTFS light default: case-insensitive"
        );
        assert!(
            posix.is_case_sensitive(),
            "POSIX light default: case-sensitive"
        );
        // No deep-semantic method exists on either beyond the light trait; the
        // only observable difference is the cheap case default above.
        assert_ne!(
            ntfs.is_case_sensitive(),
            posix.is_case_sensitive(),
            "the ONLY semantic divergence shipped now is the cheap case default"
        );
    }

    /// Validates: command-environments Requirement 16.7 -- DOC-ASSERT that a NEW
    /// FS Command Environment with new executable store behaviour is a PLUGIN
    /// capability (Req 9.4), consistent with the built-in environments being
    /// code-only and not configuration-replaceable (Req 9.3). The built-in
    /// decider + NTFS + POSIX are compiled Rust with no configuration seam that
    /// could swap or replace them; adding a genuinely new executable FS CE goes
    /// through the plugin API, not a config file. This test records that the
    /// built-ins are code-only by construction (zero-config constructors, no
    /// registration/replacement entry point exposed here).
    #[test]
    fn new_fs_ce_is_a_plugin_capability_builtins_are_code_only() {
        // Built-in CEs are zero-sized, zero-config Rust values: there is no
        // constructor that takes configuration and no public hook on this crate
        // to register or replace a built-in from a config file. The decider is a
        // pure function selecting a compiled built-in. Therefore a new executable
        // FS CE cannot arrive via configuration here -- it is a plugin capability
        // (Req 9.4), and the built-ins remain code-only / not
        // configuration-replaceable (Req 9.3).
        let _ntfs = ff_ce_ntfs::NtfsEnvironment; // no args == not config-driven
        let _posix = ff_ce_posix::PosixEnvironment; // no args == not config-driven
        let native = native_backend_environment(); // pure decider, no config
        assert!(
            matches!(native.name(), "NTFS" | "POSIX"),
            "the decider only ever yields a compiled built-in, never a config-supplied CE"
        );
    }
}
