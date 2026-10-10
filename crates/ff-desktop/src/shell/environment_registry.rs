//! # Built Environment_Registry (CR-CH-053 Task 17)
//!
//! The Environment_Registry is a BUILT, shell-owned collection of the command
//! environments that exist in the running shell. It REPLACES the former closed
//! `EnvironmentKind` enum, the hardcoded `environment_for_kind` match, and the
//! single hardcoded `== FfEdit` claim gate as the source of truth for which
//! environments exist (command-environments Requirement 13.1).
//!
//! Built-in environments register in code at startup via
//! `EnvironmentRegistry::with_builtins_and_mainframe` (Req 13.2): the FFCMD base (which IS
//! `resolve_target` and does not claim at the active-env step), FFEDIT (a real
//! [`CommandEnvironment`] object -- [`super::environment_ffedit::FfEditEnvironment`]),
//! and the host FS environment placeholder (Req 16, registered-but-non-claiming
//! in phase 1).
//!
//! The active-environment derivation (Req 2.1) and the active-wins claim gate
//! (Req 5.1) READ FROM this registry by the focused kind's command-environment
//! NAME (via [`EnvironmentRegistry::active_name`]), NOT the former hardcoded
//! match/`== FfEdit` literal (Req 13.3). An absent or unknown name degrades to
//! the FFCMD base (Req 13.4).
//!
//! The registry is a pure data/lookup structure feeding the ONE shared ladder
//! path -- it is NOT a second dispatcher and NOT a second navigation stack
//! (Req 13.7, 1.4, 3.3). `CommandTarget` and the per-tab Navigation_Stack are
//! unchanged.
//!
//! ## FFEDIT as a thin object over shell state
//!
//! The design (design.md "FFEDIT-as-object") PERMITS FFEDIT to be a thin object
//! that still reaches shell-entangled state (`self.tabs` + the managers), because
//! its verb bodies mutate `&mut WorkbenchShell`. Hence the
//! [`super::environment::CommandEnvironment`] trait method takes
//! `&mut WorkbenchShell`; the registered object
//! [`super::environment_ffedit::FfEditEnvironment`] is zero-sized and its `claim`
//! body is the former `WorkbenchShell::ffedit_claim`, moved verbatim so the
//! observable result is byte-identical (Req 13.5, 13.6, 4.2, 6.3). That object
//! lives in `environment_ffedit.rs` purely to keep each file under the 400-line
//! rule.
//!
//! See docs/specs/command-environments/{requirements,design}.md.

use crate::tab_state::KindTag;
use crate::workspace_kind::BuiltinKind;

/// The stable NAME of the FFCMD base environment (= `resolve_target`). The base
/// never claims at the active-env step; the front door reaches it via the
/// existing `resolve_target` call. Also the degrade-to-base name (Req 13.4).
pub(super) const FFCMD_NAME: &str = "FFCMD";
/// The stable NAME of the FFEDIT editor environment (the only phase-1 claiming
/// environment).
pub(super) const FFEDIT_NAME: &str = "FFEDIT";
/// The stable NAME of the file-navigator environment. NAMED only in phase 1 --
/// its verbs stay handled where they are today; it is not a registered claiming
/// environment, so resolving it degrades to the FFCMD base at the gate.
pub(super) const FFNAV_NAME: &str = "FFNAV";
/// The stable NAME of the host-FS environment placeholder (Req 16). Registered
/// so the registry membership is correct, but non-claiming in phase 1. This is
/// also the default Owning_Environment for a host-path tab (Task 19); the shared
/// constant lives on `TabState` so the tab default and the registry name cannot
/// drift.
pub(super) const HOST_FS_NAME: &str = crate::tab_state::DEFAULT_OWNING_ENVIRONMENT;
/// The stable NAME of the MAINFRAME backend Command Environment (RC.B.8 (b),
/// Req 16.4). It is an ADDRESSING/backend target only -- never an active
/// interactive environment -- so it is registered for membership + `backend_for`
/// resolution but does NOT change the active-env derivation (a mainframe-owned
/// editor tab still derives FFEDIT as its active env). The real record-capable
/// backend object (`ff_idcams::MainframeEnvironment`) is held behind
/// [`EnvironmentRegistry::mainframe`] when built; absent, a MAINFRAME lookup
/// falls back to the host backend so the save seam is always safe.
pub(super) const MAINFRAME_NAME: &str = "MAINFRAME";

/// A tag identifying WHICH registered environment a name resolves to.
///
/// With only three built-in members in phase 1 -- and FFEDIT's verb bodies
/// requiring `&mut WorkbenchShell` -- an enum tag is the minimal OPEN structure
/// that makes "which environments exist" data-driven without forcing
/// `self`-reentrancy gymnastics for zero behaviour benefit. Task 21's `ff-ce-*`
/// crates extend this (a new member / a boxed-plugin arm) without touching the
/// active-env derivation or the claim gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RegisteredEnv {
    /// The FFCMD base (= `resolve_target`); does not claim at the active-env step.
    FfCmdBase,
    /// The FFEDIT editor environment -- the real
    /// [`super::environment_ffedit::FfEditEnvironment`] object.
    FfEdit,
    /// The host-FS environment placeholder (Req 16); non-claiming in phase 1.
    HostFsPlaceholder,
    /// The MAINFRAME backend environment placeholder (RC.B.8 (b), Req 16.4):
    /// registered for membership + `backend_for` resolution, non-claiming (never
    /// an active interactive env). The real backend object is held in
    /// [`EnvironmentRegistry::mainframe`].
    MainframePlaceholder,
}

/// One registered environment: its stable NAME and the tag identifying it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EnvironmentEntry {
    /// The stable environment name used for derivation and (future) addressing.
    pub(super) name: &'static str,
    /// Which registered environment this entry is.
    pub(super) env: RegisteredEnv,
}

/// The BUILT, shell-owned collection of registered command environments
/// (command-environments Requirement 13.1, 13.2).
///
/// Holds an ordered set of [`EnvironmentEntry`]; owns name membership, lookup,
/// and the default-to-FFCMD rule. It does NOT own the mutable claim bodies -- the
/// shell performs a claim by calling the resolved environment object through the
/// [`super::environment::CommandEnvironment`] trait (which borrows
/// `&mut WorkbenchShell`), keeping the registry a pure data/lookup structure
/// (Req 13.7).
pub(super) struct EnvironmentRegistry {
    entries: Vec<EnvironmentEntry>,
    /// The resolved host-FS BACKEND Command Environment (CR-CH-053 Task 21):
    /// `ff-ce-host-fs` picks NTFS on Windows / POSIX elsewhere, registered here
    /// under `HOST_FS_NAME`. It is the real backend object FFEDIT's addressed SAVE
    /// delegates its physical write to (`host_fs_save`), while the dirty-aware
    /// orchestration stays shell-side (Req 14.7). Held as a boxed trait object so
    /// future backend CEs (mainframe, sqlite) register the same way without the
    /// registry knowing their concrete types.
    host_fs: Box<dyn ff_vfs::BackendEnvironment>,
    /// The resolved MAINFRAME backend Command Environment (RC.B.8 (b), Req
    /// 16.4): `ff_idcams::MainframeEnvironment` wrapping an owned
    /// `Arc<dyn ff_dscatalog::DatasetAccess>`, built by the shell at
    /// registry-build time and passed to [`Self::with_builtins_and_mainframe`].
    /// `None` for the test-only mainframe-less constructor (`with_builtins`); a
    /// `backend_for(MAINFRAME)` lookup then falls back to
    /// `host_fs`, so the save seam never panics. Held as a boxed trait object so
    /// the registry does not know the concrete type.
    mainframe: Option<Box<dyn ff_vfs::BackendEnvironment>>,
}

impl EnvironmentRegistry {
    /// Build the registry with the phase-1 built-in environments registered in
    /// code (Req 13.2): the FFCMD base, FFEDIT, the host-FS environment (its
    /// addressing identity plus the resolved backend object, Task 21). This is
    /// the single registration point.
    ///
    /// Validates: command-environments Requirement 13.1, 13.2, 16.2, 16.4
    ///
    /// Test-only: the production shell always builds the MAINFRAME backend via
    /// [`Self::with_builtins_and_mainframe`] (RC.B.8 (c)); this mainframe-less
    /// constructor remains for the registry unit tests that do not need it (a
    /// MAINFRAME lookup then safely falls back to the host backend).
    #[cfg(test)]
    pub(super) fn with_builtins() -> Self {
        Self {
            entries: Self::builtin_entries(),
            // The decider resolves the native-role backend for this host (Req
            // 16.2/16.3): Windows -> ff-ce-ntfs, Linux/macOS -> ff-ce-posix.
            host_fs: ff_ce_host_fs::native_backend_environment(),
            // No mainframe CE in the mainframe-less constructor; a MAINFRAME
            // lookup falls back to host_fs (Req 16.4).
            mainframe: None,
        }
    }

    /// Build the registry with the phase-1 built-ins PLUS the MAINFRAME backend
    /// Command Environment (RC.B.8 (b), Req 16.4). The shell constructs the
    /// mainframe CE (a `ff_idcams::MainframeEnvironment` owning an
    /// `Arc<dyn ff_dscatalog::DatasetAccess>`) and passes it here so the registry
    /// does not know its concrete type. `backend_for(MAINFRAME)` then resolves to
    /// this record-capable backend; HOSTFS and unknown names still resolve to the
    /// host backend.
    ///
    /// Validates: command-environments Requirement 13.1, 13.2, 16.2, 16.4
    pub(super) fn with_builtins_and_mainframe(
        mainframe: Box<dyn ff_vfs::BackendEnvironment>,
    ) -> Self {
        Self {
            entries: Self::builtin_entries(),
            host_fs: ff_ce_host_fs::native_backend_environment(),
            mainframe: Some(mainframe),
        }
    }

    /// The built-in environment entries (RC.B.8 (b) adds the MAINFRAME addressing
    /// entry). Shared by both constructors so membership is identical whether or
    /// not the mainframe backend object is built.
    fn builtin_entries() -> Vec<EnvironmentEntry> {
        vec![
            EnvironmentEntry {
                name: FFCMD_NAME,
                env: RegisteredEnv::FfCmdBase,
            },
            EnvironmentEntry {
                name: FFEDIT_NAME,
                env: RegisteredEnv::FfEdit,
            },
            EnvironmentEntry {
                name: HOST_FS_NAME,
                env: RegisteredEnv::HostFsPlaceholder,
            },
            EnvironmentEntry {
                name: MAINFRAME_NAME,
                env: RegisteredEnv::MainframePlaceholder,
            },
        ]
    }

    /// Resolve the BACKEND Command Environment that owns a tab's store, by its
    /// Owning_Environment NAME (RC.B.8 (b), Req 16.4). Case-INSENSITIVE on the
    /// name. `MAINFRAME` resolves to the record-capable mainframe CE when built;
    /// `HOSTFS` and ANY unknown name resolve to the host backend (the fallback
    /// preserves host SAVE behaviour, and keeps the save seam safe when no
    /// mainframe CE is built). This feeds the single SAVE-addressing seam
    /// (`host_fs_save` -> `save_active_tab_via_backend`); it is NOT a second
    /// dispatcher.
    ///
    /// Validates: command-environments Requirement 14.4, 14.5, 16.4
    pub(super) fn backend_for(&self, name: &str) -> &dyn ff_vfs::BackendEnvironment {
        if name.eq_ignore_ascii_case(MAINFRAME_NAME) {
            if let Some(mainframe) = self.mainframe.as_deref() {
                return mainframe;
            }
        }
        self.host_fs.as_ref()
    }

    /// Look up a registered environment by NAME (case-sensitive stable name).
    /// Returns `None` when the name is not registered.
    fn lookup(&self, name: &str) -> Option<RegisteredEnv> {
        self.entries.iter().find(|e| e.name == name).map(|e| e.env)
    }

    /// Resolve a registered environment by NAME for ADDRESS-by-name dispatch
    /// (CR-CH-053 Task 18, Req 14.1). Case-INSENSITIVE on the stable env name so
    /// `ADDRESS ffedit` and `ADDRESS FFEDIT` resolve alike. Returns `None` when
    /// no environment is registered under the name (the caller treats that as
    /// `NoSuchEnvironment`). This is a pure lookup -- it performs no dispatch
    /// itself (Req 13.7); the shell invokes the resolved environment's `claim`.
    ///
    /// Validates: command-environments Requirement 14.1
    pub(super) fn resolve(&self, name: &str) -> Option<RegisteredEnv> {
        self.entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
            .map(|e| e.env)
    }

    /// The command-environment NAME SUPPLIED BY a focused Context's kind,
    /// derived THROUGH the kind exactly as before (Req 2.1): the handler obtains
    /// the `BuiltinKind` via `BuiltinKind::from_tab_kind` (the kind's own
    /// mapping), then maps editor kinds -> FFEDIT, file-navigator kinds -> FFNAV,
    /// every other kind -> the FFCMD base. `is_home` only splits POM vs Menu for
    /// the kind mapping; neither is an editor/navigator kind.
    ///
    /// Validates: command-environments Requirement 2.1, 13.3
    pub(super) fn active_name(&self, kind: KindTag, is_home: bool) -> &'static str {
        match BuiltinKind::from_tab_kind(kind, is_home) {
            BuiltinKind::Editor => FFEDIT_NAME,
            BuiltinKind::Files | BuiltinKind::Catalogs => FFNAV_NAME,
            _ => FFCMD_NAME,
        }
    }

    /// Resolve the registered environment for a focused Context, degrading an
    /// absent/unknown name to the FFCMD base (Req 13.4). "FFNAV" is a derived name
    /// that is NOT a registered claiming environment in phase 1, so it degrades to
    /// the FFCMD base -- identical to the former `FfNav != FfEdit` making the gate
    /// false.
    ///
    /// Validates: command-environments Requirement 13.3, 13.4
    fn active_env(&self, kind: KindTag, is_home: bool) -> RegisteredEnv {
        let name = self.active_name(kind, is_home);
        self.lookup(name).unwrap_or(RegisteredEnv::FfCmdBase)
    }

    /// Whether the Active_Environment for a focused Context is FFEDIT (an editor
    /// Context), read FROM the registry (Req 13.3). This replaces the former
    /// `active_environment(..) == EnvironmentKind::FfEdit` literal; the predicate
    /// is unchanged in meaning.
    ///
    /// Validates: command-environments Requirement 5.1, 13.3
    pub(super) fn is_ffedit_active(&self, kind: KindTag, is_home: bool) -> bool {
        self.active_env(kind, is_home) == RegisteredEnv::FfEdit
    }
}

// The real registered FFEDIT (editor) `CommandEnvironment` object --
// `FfEditEnvironment` -- lives in `environment_ffedit.rs` (split out purely to
// keep each file under the 400-line rule). The registry resolves the name
// "FFEDIT" to it; the shell constructs it at the claim gate (it is zero-sized).

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates: command-environments Requirement 13.3 -- the active-environment
    /// NAME is SUPPLIED BY the focused Context's kind (editor -> FFEDIT,
    /// file-navigator -> FFNAV, every other kind -> FFCMD), read through the
    /// kind's own `from_tab_kind` mapping, not a handler-owned TabKind match.
    #[test]
    fn active_name_is_supplied_by_kind() {
        let reg = EnvironmentRegistry::with_builtins();
        assert_eq!(reg.active_name(KindTag::FileEditor, false), FFEDIT_NAME);
        assert_eq!(reg.active_name(KindTag::Untitled, false), FFEDIT_NAME);
        assert_eq!(
            reg.active_name(KindTag::FileExplorerPanel, false),
            FFNAV_NAME
        );
        assert_eq!(reg.active_name(KindTag::FilesPanel, false), FFNAV_NAME);
        assert_eq!(reg.active_name(KindTag::MenuWorkspace, true), FFCMD_NAME);
        assert_eq!(reg.active_name(KindTag::ConfigPanel, false), FFCMD_NAME);
    }

    /// Validates: command-environments Requirement 13.4 -- an absent or unknown
    /// environment name degrades to the FFCMD base rather than erroring. FFNAV is
    /// a derived name that is NOT a registered claiming environment in phase 1, so
    /// resolving a navigator Context degrades to the base (the former
    /// `FfNav != FfEdit` making the claim gate false).
    #[test]
    fn absent_or_unknown_name_degrades_to_ffcmd_base() {
        let reg = EnvironmentRegistry::with_builtins();
        // FFNAV is derived for a navigator Context but is NOT a registered
        // claiming environment -> degrades to the FFCMD base, so is_ffedit_active
        // is false and the gate falls through exactly as before.
        assert_eq!(
            reg.active_env(KindTag::FileExplorerPanel, false),
            RegisteredEnv::FfCmdBase
        );
        assert_eq!(
            reg.active_env(KindTag::FilesPanel, false),
            RegisteredEnv::FfCmdBase
        );
        // A completely unknown name is not registered -> base.
        assert_eq!(reg.lookup("NOTANENV"), None);
    }

    /// Validates: command-environments Requirement 13.1, 13.2 -- the built
    /// registry registers the phase-1 built-in environments (FFCMD base, FFEDIT,
    /// host-FS placeholder) by name at startup.
    #[test]
    fn registry_registers_the_builtin_environments() {
        let reg = EnvironmentRegistry::with_builtins();
        assert_eq!(reg.lookup(FFCMD_NAME), Some(RegisteredEnv::FfCmdBase));
        assert_eq!(reg.lookup(FFEDIT_NAME), Some(RegisteredEnv::FfEdit));
        assert_eq!(
            reg.lookup(HOST_FS_NAME),
            Some(RegisteredEnv::HostFsPlaceholder)
        );
    }

    /// A record-capable test backend (mirrors the mainframe CE's advertised
    /// capability) so the `backend_for` mapping test does not depend on
    /// `ff-idcams`.
    struct RecordCapableTestBackend;
    impl ff_vfs::BackendEnvironment for RecordCapableTestBackend {
        fn name(&self) -> &str {
            MAINFRAME_NAME
        }
        fn is_case_sensitive(&self) -> bool {
            false
        }
        fn save(&self, _path: &std::path::Path, _bytes: &[u8]) -> std::io::Result<()> {
            Err(std::io::Error::other("record-only"))
        }
        fn record_capable(&self) -> bool {
            true
        }
    }

    /// Validates: command-environments Requirement 14.1, 16.4 -- a registry built
    /// with a mainframe (record-capable) backend resolves `backend_for` by name
    /// case-insensitively: `MAINFRAME` -> the record-capable backend; `HOSTFS`
    /// and any unknown name -> the host_fs (non-record-capable) backend.
    #[test]
    fn backend_for_resolves_mainframe_to_record_capable_backend() {
        let reg =
            EnvironmentRegistry::with_builtins_and_mainframe(Box::new(RecordCapableTestBackend));
        assert!(
            reg.backend_for(MAINFRAME_NAME).record_capable(),
            "MAINFRAME must resolve to a record-capable backend"
        );
        assert!(
            reg.backend_for("mainframe").record_capable(),
            "backend_for must be case-insensitive"
        );
        assert!(
            !reg.backend_for(HOST_FS_NAME).record_capable(),
            "HOSTFS must resolve to the non-record-capable host backend"
        );
        assert!(
            !reg.backend_for("NOTANENV").record_capable(),
            "an unknown name must fall back to the host backend"
        );
    }

    /// Validates: command-environments Requirement 16.4 -- `with_builtins`
    /// (the mainframe-less constructor used by most tests) still answers
    /// `backend_for('MAINFRAME')` safely: it falls back to the host backend so
    /// the save seam never panics on a MAINFRAME lookup.
    #[test]
    fn with_builtins_mainframe_lookup_falls_back_to_host() {
        let reg = EnvironmentRegistry::with_builtins();
        // Fallback to host_fs (non-record-capable) when no mainframe CE is built.
        assert!(!reg.backend_for(MAINFRAME_NAME).record_capable());
        assert!(!reg.backend_for(HOST_FS_NAME).record_capable());
    }

    /// Validates: command-environments Requirement 13.3, 5.1 -- the active-wins
    /// claim gate reads is_ffedit_active FROM the registry: true for editor
    /// Contexts, false for every non-editor Context (menu / navigator / config).
    #[test]
    fn is_ffedit_active_matches_editor_contexts_only() {
        let reg = EnvironmentRegistry::with_builtins();
        assert!(reg.is_ffedit_active(KindTag::FileEditor, false));
        assert!(reg.is_ffedit_active(KindTag::Untitled, false));
        assert!(!reg.is_ffedit_active(KindTag::MenuWorkspace, true));
        assert!(!reg.is_ffedit_active(KindTag::ConfigPanel, false));
        assert!(!reg.is_ffedit_active(KindTag::FileExplorerPanel, false));
        assert!(!reg.is_ffedit_active(KindTag::FilesPanel, false));
    }
}
