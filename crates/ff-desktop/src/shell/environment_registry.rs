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
pub(crate) struct EnvironmentRegistry {
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

    // === Root-command enumeration (CR-CH-067, Req 20) =======================

    /// The canonical ROOT commands owned by the named environment, in a stable
    /// (sorted), case-insensitively de-duplicated order. Canonical roots ONLY --
    /// NOT localised alias SURFACES (Req 20.3): aliases are what a user AUTHORS
    /// against a root (CR-NR-112 / CR-NR-113), so offering surfaces as root
    /// choices would be circular. An unknown / non-enumerable environment name
    /// (anything other than FFCMD or FFEDIT in phase 1 -- FFNAV / FFLINE / HOSTFS
    /// / MAINFRAME) yields an EMPTY list, never a panic (Req 20.5).
    ///
    /// This is an ASSOCIATED function keyed by environment NAME rather than an
    /// instance method: the two phase-1 root sets are STATIC, derivable data
    /// (FFCMD from [`builtin_ffcmd_roots`], FFEDIT from
    /// [`AliasTable::canonical_verbs_owned`] over [`AliasTable::ffedit_english`]),
    /// so no registry instance state is needed, and the user-alias LOAD path
    /// (`apply_user_rows`, which has no `EnvironmentRegistry` handle) can reach it
    /// uniformly. It is name-keyed ON the registry type exactly as Req 20.1
    /// requires -- NOT a method on a `CommandEnvironment` trait object, because the
    /// FFCMD base IS `resolve_target` and is not such an object (Req 1.2) so a
    /// trait-only method could not cover FFCMD without a shim.
    ///
    /// Validates: command-environments Requirement 20.1, 20.2, 20.3, 20.5
    pub(crate) fn root_commands(env_name: &str) -> Vec<String> {
        let mut roots: Vec<String> = if env_name.eq_ignore_ascii_case(FFCMD_NAME) {
            // FFCMD: the built-in canonical roots (the two code-only built-in
            // menus' option roots unioned with BUILTIN_WORKSPACE_VERB_ROOTS).
            crate::shell::alias_reader::builtin_ffcmd_roots()
        } else if env_name.eq_ignore_ascii_case(FFEDIT_NAME) {
            // FFEDIT: the canonical editor verbs. The canonical roots are
            // identical whether sourced from `ffedit_english()` or the live
            // overlaid store (the locale overlay only adds SURFACES, never roots),
            // so the cheaper fresh seed is used.
            super::environment::AliasTable::ffedit_english().canonical_verbs_owned()
        } else {
            // Unknown / non-enumerable environment -> empty (Req 20.5).
            Vec::new()
        };
        // Deterministic, case-insensitively de-duplicated order (Req 20.2). Both
        // source collectors already de-duplicate case-insensitively; sorting makes
        // the order unambiguously stable for repeated calls and the Gate 2
        // dropdown, independent of either source's internal first-seen order.
        roots.sort();
        roots
    }

    /// True iff `root` (trimmed, case-insensitive) is a canonical root of the
    /// named environment, using the SAME `eq_ignore_ascii_case` comparison the
    /// existing verb resolution uses. False for an unknown environment name or an
    /// unknown root, never panicking (Req 20.4, 20.5). This is the validation
    /// primitive the user-alias LOAD skip-with-WARN (Req 20.7) and the save-side
    /// hook (Req 20.8) share.
    ///
    /// Validates: command-environments Requirement 20.4, 20.5
    pub(crate) fn contains_root(env_name: &str, root: &str) -> bool {
        let r = root.trim();
        Self::root_commands(env_name)
            .iter()
            .any(|c| c.eq_ignore_ascii_case(r))
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

    // === Root-command enumeration tests (CR-CH-067, Req 20) ================

    /// Validates: command-environments Requirement 20.1, 20.3, and Requirement
    /// 22.1 -- `root_commands` for FFCMD returns the COMPLETE built-in canonical
    /// root set: the built-in menu-option roots (CONFIG, THEME, FILES, CATALOGS,
    /// ...) unioned with BUILTIN_WORKSPACE_VERB_ROOTS (COMMANDS, LOG, PLUGINS,
    /// MACROS, SEARCH, GSEARCH, KINDS, MENUS, ALIASES, KEYS). A user-menu-only
    /// root (never in the two code-only built-in menus nor the verb list) is
    /// intentionally EXCLUDED. CR-CH-070 update: FILES is a root and the malformed
    /// `=FILES` literal is NOT -- the latter was removed from the FFCMD root set.
    #[test]
    fn root_commands_ffcmd_returns_complete_builtin_root_set() {
        let roots = EnvironmentRegistry::root_commands(FFCMD_NAME);
        for expected in [
            // Menu-option + CustomWorkspace verb roots.
            "CONFIG", "THEME", "FILES", "CATALOGS", "COMMANDS", "LOG", "PLUGINS", "MACROS",
            "SEARCH", "GSEARCH", "KINDS", "MENUS", "ALIASES",
            "KEYS", // Ladder-handled FFCMD roots (CR-CH-067), incl. the file-open family.
            "EDIT", "VIEW", "BROWSE", "START", "RETURN", "SWAP",
        ] {
            assert!(
                roots.iter().any(|r| r == expected),
                "FFCMD roots must contain built-in root {expected}, got {roots:?}"
            );
        }
        // CR-CH-070 Req 22.1: the malformed `=FILES` literal is NO LONGER a
        // standalone FFCMD root -- `=` is the jump prefix, FILES is the root.
        assert!(
            !roots.iter().any(|r| r.eq_ignore_ascii_case("=FILES")),
            "the malformed =FILES literal must NOT be an FFCMD root, got {roots:?}"
        );
        // A root reachable ONLY via a user menus/*.toml override is NOT a built-in
        // canonical root and must not appear (the documented completeness posture).
        assert!(
            !roots.iter().any(|r| r.eq_ignore_ascii_case("USERONLYROOT")),
            "a user-menu-only root must not be in the built-in canonical set"
        );
    }

    /// Validates: command-environments Requirement 22.1 -- `contains_root` agrees
    /// with the cleaned FFCMD root set: FILES is a root, the malformed `=FILES`
    /// literal is NOT.
    #[test]
    fn contains_root_ffcmd_files_true_equals_files_false() {
        assert!(
            EnvironmentRegistry::contains_root(FFCMD_NAME, "FILES"),
            "FILES is a legitimate FFCMD root (CR-CH-070 Req 22.1/22.2)"
        );
        assert!(
            !EnvironmentRegistry::contains_root(FFCMD_NAME, "=FILES"),
            "=FILES must NOT be an FFCMD root after CR-CH-070 (Req 22.1)"
        );
    }

    /// Validates: command-environments Requirement 20.1 -- `root_commands` for
    /// FFEDIT returns the canonical editor verbs sourced from
    /// `AliasTable::ffedit_english().canonical_verbs_owned()`.
    #[test]
    fn root_commands_ffedit_returns_canonical_editor_verbs() {
        let roots = EnvironmentRegistry::root_commands(FFEDIT_NAME);
        // AUTONUM / NUM added by CR-CH-069 (the in-buffer line-renumbering verbs
        // moved out of the FFCMD ladder into ffedit_english).
        for expected in [
            "FIND", "CHANGE", "LOCATE", "SAVE", "SORT", "EXCLUDE", "SHOW", "AUTONUM", "NUM",
        ] {
            assert!(
                roots.iter().any(|r| r == expected),
                "FFEDIT roots must contain editor verb {expected}, got {roots:?}"
            );
        }
    }

    /// Validates: command-environments Requirement 20.3 -- `root_commands` returns
    /// CANONICAL roots ONLY, not localised alias SURFACES. X and INCLUDE are
    /// FFEDIT alias surfaces whose canonicals are EXCLUDE and SHOW; they must NOT
    /// appear as roots (offering a surface as a root choice would be circular).
    #[test]
    fn root_commands_ffedit_excludes_alias_surfaces() {
        let roots = EnvironmentRegistry::root_commands(FFEDIT_NAME);
        assert!(
            roots.iter().any(|r| r == "EXCLUDE"),
            "the canonical EXCLUDE is a root"
        );
        assert!(
            roots.iter().any(|r| r == "SHOW"),
            "the canonical SHOW is a root"
        );
        assert!(
            !roots.iter().any(|r| r == "X"),
            "the alias SURFACE X must NOT be a root, got {roots:?}"
        );
        assert!(
            !roots.iter().any(|r| r == "INCLUDE"),
            "the alias SURFACE INCLUDE must NOT be a root, got {roots:?}"
        );
    }

    /// Validates: command-environments Requirement 20.2 -- `root_commands` is
    /// DETERMINISTIC (stable sorted order) across repeated calls, and is
    /// case-insensitively de-duplicated (no two entries differ only by case).
    #[test]
    fn root_commands_is_deterministic_and_deduplicated() {
        let a = EnvironmentRegistry::root_commands(FFCMD_NAME);
        let b = EnvironmentRegistry::root_commands(FFCMD_NAME);
        assert_eq!(a, b, "repeated calls must return the same order");
        let mut sorted = a.clone();
        sorted.sort();
        assert_eq!(a, sorted, "order must be the stable sorted order");
        for i in 0..a.len() {
            for j in (i + 1)..a.len() {
                assert!(
                    !a[i].eq_ignore_ascii_case(&a[j]),
                    "roots must be case-insensitively de-duplicated: {} vs {}",
                    a[i],
                    a[j]
                );
            }
        }
    }

    /// Validates: command-environments Requirement 20.5 -- an unknown /
    /// non-enumerable environment name (FFNAV / FFLINE / HOSTFS / MAINFRAME or any
    /// other) yields an EMPTY root list, never a panic.
    #[test]
    fn root_commands_unknown_environment_is_empty() {
        assert!(EnvironmentRegistry::root_commands(FFNAV_NAME).is_empty());
        assert!(EnvironmentRegistry::root_commands(HOST_FS_NAME).is_empty());
        assert!(EnvironmentRegistry::root_commands(MAINFRAME_NAME).is_empty());
        assert!(EnvironmentRegistry::root_commands("FFLINE").is_empty());
        assert!(EnvironmentRegistry::root_commands("NOTANENV").is_empty());
        assert!(EnvironmentRegistry::root_commands("").is_empty());
    }

    /// Validates: command-environments Requirement 20.4 -- `contains_root` is
    /// case-insensitive and whitespace-trimming for a real root of a known
    /// environment, in both source-case directions.
    #[test]
    fn contains_root_is_case_insensitive_and_trimmed() {
        assert!(EnvironmentRegistry::contains_root(FFCMD_NAME, "CONFIG"));
        assert!(EnvironmentRegistry::contains_root(FFCMD_NAME, "config"));
        assert!(EnvironmentRegistry::contains_root(FFCMD_NAME, "  Config  "));
        assert!(EnvironmentRegistry::contains_root(FFEDIT_NAME, "find"));
        assert!(EnvironmentRegistry::contains_root(FFEDIT_NAME, "FIND"));
    }

    /// Validates: command-environments Requirement 20.4, 20.5 -- `contains_root`
    /// is FALSE for a bogus root, for a known root in the WRONG environment, and
    /// for any root of an unknown environment, never panicking.
    #[test]
    fn contains_root_false_for_bogus_or_unknown() {
        assert!(!EnvironmentRegistry::contains_root(FFCMD_NAME, "NOTAROOT"));
        // FIND is an FFEDIT root, not an FFCMD root.
        assert!(!EnvironmentRegistry::contains_root(FFCMD_NAME, "FIND"));
        // CONFIG is an FFCMD root, not an FFEDIT root.
        assert!(!EnvironmentRegistry::contains_root(FFEDIT_NAME, "CONFIG"));
        // X is an FFEDIT alias SURFACE, not a canonical root.
        assert!(!EnvironmentRegistry::contains_root(FFEDIT_NAME, "X"));
        // Unknown environment -> always false.
        assert!(!EnvironmentRegistry::contains_root("NOTANENV", "CONFIG"));
        assert!(!EnvironmentRegistry::contains_root(MAINFRAME_NAME, "FIND"));
    }

    /// Validates: command-environments Requirement 21.1, 21.4 (CR-CH-069) --
    /// AUTONUM and NUM are FFEDIT roots, NOT FFCMD roots, after the placement
    /// correction: `root_commands("FFEDIT")` CONTAINS them and
    /// `root_commands("FFCMD")` does NOT. They were moved out of
    /// `BUILTIN_FFCMD_LADDER_VERB_ROOTS` and into `ffedit_english` (in-buffer
    /// line-renumbering verbs, CR-CH-053 Req 2a.4 / Req 10), so the enumeration
    /// the alias-anchoring model reads reflects the corrected ownership.
    #[test]
    fn autonum_and_num_are_ffedit_roots_not_ffcmd_roots() {
        let ffedit = EnvironmentRegistry::root_commands(FFEDIT_NAME);
        let ffcmd = EnvironmentRegistry::root_commands(FFCMD_NAME);
        for verb in ["AUTONUM", "NUM"] {
            assert!(
                ffedit.iter().any(|r| r == verb),
                "FFEDIT roots must contain {verb} after CR-CH-069, got {ffedit:?}"
            );
            assert!(
                !ffcmd.iter().any(|r| r == verb),
                "FFCMD roots must NOT contain {verb} after CR-CH-069, got {ffcmd:?}"
            );
        }
    }

    /// Validates: command-environments Requirement 21.4 (CR-CH-069) --
    /// `contains_root` reflects the corrected placement: FFEDIT owns AUTONUM/NUM
    /// (and FIND); FFCMD owns none of them. An FFCMD-anchored row for these is
    /// therefore invalid-in-environment (consistent with CR-CH-067 Req 20.4).
    #[test]
    fn contains_root_reflects_autonum_num_find_moved_to_ffedit() {
        for verb in ["AUTONUM", "NUM", "FIND"] {
            assert!(
                EnvironmentRegistry::contains_root(FFEDIT_NAME, verb),
                "FFEDIT must contain root {verb} after CR-CH-069"
            );
            assert!(
                !EnvironmentRegistry::contains_root(FFCMD_NAME, verb),
                "FFCMD must NOT contain root {verb} after CR-CH-069"
            );
        }
        // Case-insensitive on the corrected roots too.
        assert!(EnvironmentRegistry::contains_root(FFEDIT_NAME, "autonum"));
        assert!(EnvironmentRegistry::contains_root(FFEDIT_NAME, "num"));
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

    // === CR-CH-071: SAVE/CANCEL/REFRESH as FFCMD roots =====================

    /// Validates: command-environments Requirement 20a.5 -- SAVE, CANCEL, and
    /// REFRESH are FFCMD-base canonical ROOT commands: `root_commands("FFCMD")`
    /// contains all three and `contains_root("FFCMD", v)` is TRUE for each.
    #[test]
    fn ffcmd_roots_contain_save_cancel_refresh() {
        let roots = EnvironmentRegistry::root_commands(FFCMD_NAME);
        for expected in ["SAVE", "CANCEL", "REFRESH"] {
            assert!(
                roots.iter().any(|r| r == expected),
                "FFCMD roots must contain the CR-CH-071 editable-Context verb \
                 {expected}, got {roots:?}"
            );
            assert!(
                EnvironmentRegistry::contains_root(FFCMD_NAME, expected),
                "contains_root(FFCMD, {expected}) must be true (Req 20a.5)"
            );
        }
        // Case-insensitivity on the new roots too.
        assert!(EnvironmentRegistry::contains_root(FFCMD_NAME, "save"));
        assert!(EnvironmentRegistry::contains_root(FFCMD_NAME, "refresh"));
    }
}
