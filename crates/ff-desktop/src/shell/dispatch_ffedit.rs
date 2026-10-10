//! # FFEDIT Command_Environment family handlers (CR-CH-053 E2-E5).
//!
//! The verb-body implementations the FFEDIT environment claims, split out of
//! `shell/dispatch.rs` (which keeps the router: `dispatch_command_string` +
//! `ffedit_claim`) to hold both files under the 400-line rule
//! (`rust-standards.md`). Pure code movement -- no behaviour change.
//!
//! Each handler is the former `try_commands_b2` ladder arm body, moved VERBATIM
//! so the observable result is identical (command-environments Req 6.3). `raw` /
//! `upper` carry the surface form the user typed; the canonical verb was already
//! matched by `ffedit_claim` before delegating here. These are `pub(super)` so
//! the router in the sibling `dispatch` module can call them.

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Map a find/change status string to an `open_error` (shared by the FIND
    /// family). Mirrors the former ladder arms: a NOT FOUND / error status is
    /// surfaced; anything else clears the error.
    pub(super) fn find_status_to_error(status: String) -> Option<String> {
        if status.contains("NOT FOUND") || status.contains("error") {
            Some(status)
        } else {
            None
        }
    }

    /// The host-FS Command Environment's SAVE (CR-CH-053 Task 20, Req
    /// 14.4/14.5/14.6; the dirty-aware contract of Req 10.1). This is the SAVE
    /// that FFEDIT ADDRESSes to the owning environment for a host-path resource
    /// (`dispatch_to_environment` -> the `HostFsPlaceholder` arm), rather than
    /// FFEDIT executing the write itself. The body is UNCHANGED from the former
    /// `ffedit_save`, so a native file's SAVE is byte-identical -- only the
    /// EXECUTOR moved from FFEDIT to the host FS environment.
    ///
    /// Dirty-awareness (Req 10.1, preserved verbatim): clean buffer
    /// (`!is_modified`) -> no-op (no write, clear any stale error). Dirty ->
    /// delegate to `save_active_tab` (write + clear flag + save point); a write
    /// failure (incl. an untitled buffer with no path) STAYS and surfaces the
    /// error (the dirty flag is left set because nothing was written). SAVE is
    /// NOT a Confirmable_Command and never leaves the editor.
    ///
    /// Task 21 moves this body into the dedicated `ff-ce-*` host FS crate; the
    /// routing (FFEDIT -> owning env) is already in place here.
    ///
    /// RC.B.8 (b): the backend is now resolved by the active tab's
    /// Owning_Environment via `EnvironmentRegistry::backend_for` (MAINFRAME tabs
    /// reach the record-capable mainframe CE; HOSTFS/unknown fall back to the
    /// host backend, byte-identical). This stays the SINGLE SAVE-addressing seam
    /// -- only the backend SELECTION became owning-env-aware.
    pub(super) fn host_fs_save(&mut self) {
        if !self.tabs.active_tab().is_modified {
            // Clean: nothing changed since the last save -> no-op.
            self.open_error = None;
            return;
        }
        // CR-CH-053 Task 21: delegate the physical WRITE to the resolved host-FS
        // backend Command Environment (ff-ce-ntfs / ff-ce-posix via the decider),
        // keeping the dirty-aware orchestration in `save_active_tab_via_backend`.
        // Borrow `environments`, `tabs`, `runtime` as disjoint fields so the
        // immutable backend ref and the mutable tab-manager call do not conflict;
        // the borrows end before `self.open_error` is written.
        // RC.B.8 (b): resolve the backend by the ACTIVE TAB's Owning_Environment
        // (Req 14.4) through the single `backend_for` seam -- MAINFRAME tabs reach
        // the record-capable mainframe CE, HOSTFS/unknown fall back to the host
        // backend (byte-identical native SAVE). This is still the SINGLE save
        // seam; only the backend selection became owning-env-aware.
        let result = {
            let Self {
                environments,
                tabs,
                runtime,
                ..
            } = self;
            let owning = tabs.active_tab().owning_environment.clone();
            let backend = environments.backend_for(&owning);
            tabs.save_active_tab_via_backend(backend, runtime)
        };
        self.open_error = result.err();
    }

    /// EXCLUDE / X [text] [ALL] (E2). Delegates to `exclude_manager` with the
    /// same snapshot closure as the former ladder arm.
    pub(super) fn ffedit_exclude(&mut self, cmd: &str) {
        use super::helpers::{info_or_error, strip_all_suffix};
        let upper = cmd.trim().to_uppercase();
        if upper == "EXCLUDE ALL" || upper == "X ALL" {
            let tab = self.tabs.active_tab();
            let (tab_id, line_count) = (tab.id.0, tab.line_count as usize);
            let msg = self.exclude_manager.exclude_all(tab_id, line_count, || {
                crate::exclude_manager::snapshot_lines(tab, &self.runtime)
            });
            self.open_error = info_or_error(&msg);
            return;
        }
        // EXCLUDE 'text' [ALL]  or  X 'text' [ALL]: strip the surface verb to get
        // the argument text (case preserved).
        let rest = if upper.starts_with("EXCLUDE ") {
            cmd.trim()[8..].trim()
        } else {
            cmd.trim()[2..].trim()
        };
        let (text, all_flag) = strip_all_suffix(rest);
        let tab = self.tabs.active_tab();
        let (tab_id, line_count) = (tab.id.0, tab.line_count as usize);
        let msg = if all_flag {
            self.exclude_manager
                .exclude_text_all(text, tab_id, line_count, || {
                    crate::exclude_manager::snapshot_lines(tab, &self.runtime)
                })
        } else {
            self.exclude_manager
                .exclude_text(text, tab_id, line_count, || {
                    crate::exclude_manager::snapshot_lines(tab, &self.runtime)
                })
        };
        self.open_error = info_or_error(&msg);
    }

    /// SHOW / INCLUDE [text] [ALL] (E2).
    pub(super) fn ffedit_show(&mut self, cmd: &str) {
        use super::helpers::info_or_error;
        let upper = cmd.trim().to_uppercase();
        if upper == "SHOW ALL" || upper == "INCLUDE ALL" {
            let tab = self.tabs.active_tab();
            let (tab_id, line_count) = (tab.id.0, tab.line_count as usize);
            let msg = self.exclude_manager.show_all(tab_id, line_count, || {
                crate::exclude_manager::snapshot_lines(tab, &self.runtime)
            });
            self.open_error = info_or_error(&msg);
            return;
        }
        let rest = if upper.starts_with("SHOW ") {
            cmd.trim()[5..].trim()
        } else {
            cmd.trim()[8..].trim()
        };
        let tab = self.tabs.active_tab();
        let (tab_id, line_count) = (tab.id.0, tab.line_count as usize);
        let msg = self
            .exclude_manager
            .show_text(rest, tab_id, line_count, || {
                crate::exclude_manager::snapshot_lines(tab, &self.runtime)
            });
        self.open_error = info_or_error(&msg);
    }

    /// RESET [EXCLUDED|ALL] (E2).
    pub(super) fn ffedit_reset(&mut self, cmd: &str) {
        use super::helpers::info_or_error;
        use ff_exclude_show_filter::ResetVariant;
        let upper = cmd.trim().to_uppercase();
        let variant = if upper == "RESET ALL" {
            ResetVariant::All
        } else if upper == "RESET EXCLUDED" {
            ResetVariant::Excluded
        } else {
            ResetVariant::Default
        };
        let tab = self.tabs.active_tab();
        let (tab_id, line_count) = (tab.id.0, tab.line_count as usize);
        let msg = self.exclude_manager.reset(variant, tab_id, line_count, || {
            crate::exclude_manager::snapshot_lines(tab, &self.runtime)
        });
        self.open_error = info_or_error(&msg);
    }

    /// CAPS [ON|OFF] | bare CAPS toggles (E4).
    pub(super) fn ffedit_caps(&mut self, upper: &str) {
        if upper == "CAPS ON" {
            self.tabs.active_tab_mut().edit_profile.caps = ff_edit_operations::CapsMode::On;
            self.open_error = None;
        } else if upper == "CAPS OFF" {
            self.tabs.active_tab_mut().edit_profile.caps = ff_edit_operations::CapsMode::Off;
            self.open_error = None;
        } else {
            // bare CAPS toggles.
            let tab = self.tabs.active_tab_mut();
            tab.edit_profile.caps = tab.edit_profile.caps.toggle();
            self.open_error = None;
        }
    }

    /// NULLS ON|OFF (E4).
    pub(super) fn ffedit_nulls(&mut self, upper: &str) {
        if upper == "NULLS ON" {
            self.tabs.active_tab_mut().edit_profile.nulls = ff_edit_operations::NullsMode::On;
        } else if upper == "NULLS OFF" {
            self.tabs.active_tab_mut().edit_profile.nulls = ff_edit_operations::NullsMode::Off;
        }
        self.open_error = None;
    }

    /// STATS ON|OFF (E4).
    pub(super) fn ffedit_stats(&mut self, upper: &str) {
        if upper == "STATS ON" {
            self.tabs.active_tab_mut().edit_profile.stats = ff_edit_operations::StatsMode::On;
        } else if upper == "STATS OFF" {
            self.tabs.active_tab_mut().edit_profile.stats = ff_edit_operations::StatsMode::Off;
        }
        self.open_error = None;
    }

    /// LOCK ON|OFF (E4).
    pub(super) fn ffedit_lock(&mut self, upper: &str) {
        if upper == "LOCK ON" {
            self.tabs.active_tab_mut().edit_profile.lock = ff_edit_operations::ProfileLock::On;
        } else if upper == "LOCK OFF" {
            self.tabs.active_tab_mut().edit_profile.lock = ff_edit_operations::ProfileLock::Off;
        }
        self.open_error = None;
    }

    /// PROFILE | PROFILE <keyword> [value] (E4).
    pub(super) fn ffedit_profile(&mut self, cmd: &str) {
        use super::helpers::verb_arg;
        use ff_edit_operations::ProfileError;
        if cmd.trim().eq_ignore_ascii_case("PROFILE") {
            let summary = self.tabs.active_tab().edit_profile.display_summary();
            self.open_error = Some(summary);
            return;
        }
        let Some(rest) = verb_arg(cmd, "PROFILE").filter(|a| !a.is_empty()) else {
            return;
        };
        let mut parts = rest.splitn(2, ' ');
        let key = parts.next().unwrap_or("");
        let val = parts.next().unwrap_or("").trim();
        match self
            .tabs
            .active_tab_mut()
            .edit_profile
            .apply_keyword(key, val)
        {
            Ok(()) => self.open_error = None,
            Err(ProfileError::Locked) => {
                self.open_error = Some("Profile is locked -- use LOCK OFF to unlock".to_string());
            }
            Err(e) => self.open_error = Some(e.to_string()),
        }
    }

    /// HILITE [mode] (E4).
    pub(super) fn ffedit_hilite(&mut self, cmd: &str) {
        use super::helpers::verb_arg;
        let Some(keyword) = verb_arg(cmd, "HILITE") else {
            return;
        };
        let mode = if keyword.is_empty() {
            Some(ff_edit_operations::HiliteMode::On)
        } else {
            ff_edit_operations::HiliteMode::from_keyword(keyword)
        };
        match mode {
            Some(m) => {
                self.tabs.active_tab_mut().edit_profile.hilite = m;
                self.open_error = None;
            }
            None => {
                self.open_error = Some(format!("HILITE: unknown mode '{keyword}'"));
            }
        }
    }
}
