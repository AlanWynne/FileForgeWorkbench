//! # Shell Command Dispatch
//!
//! `handle_command()` — parses and routes every primary command entered in the
//! Command ===> field or dispatched programmatically.

use ff_command::{CommandParams, CommandResult};
use ff_command_semantics::StatusKind;
use ff_edit_operations::ProfileError;
use ff_help::{ContextDetector, EditorContext, EditorMode, HelpTopicRegistry};
use ff_keys::RetrieveResult;

use crate::tab_state::TabKind;

use super::helpers::*;

impl WorkbenchShell {
    pub(super) fn handle_command(&mut self, cmd: &str) {
        let upper = cmd.trim().to_uppercase();

        // ── Shell-level intercepts ───────────────────────────────────────
        if upper == "EXIT" || upper == "QUIT" || upper == "=X" || upper == "X" {
            let result = self
                .dispatch
                .execute_command("file.exit", CommandParams::new());
            if let CommandResult::Err(e) = result {
                self.open_error = Some(e.to_string());
            }
            return;
        }

        if upper.starts_with("EDIT") && (upper == "EDIT" || upper.starts_with("EDIT ")) {
            let rest = cmd.trim().split_once(' ').map(|x| x.1.trim()).unwrap_or("");
            if rest.is_empty() {
                self.open_error = Some("EDIT requires a file path".to_string());
            } else {
                let mut p = CommandParams::new();
                p.insert("path", rest);
                let result = self.dispatch.execute_command("file.open", p);
                if let CommandResult::Err(e) = result {
                    self.open_error = Some(e.to_string());
                } else {
                    self.open_error = None;
                }
            }
            return;
        }

        if upper == "START" || upper == "POM" {
            // Validates: Requirement 14.10, 14.14 — START/POM opens a new POM tab
            self.tabs.insert_pom_tab(&self.runtime);
            self.open_error = None;
            return;
        }

        if upper == "CLOSE" {
            // Validates: Requirement 14.11 — CLOSE closes the current tab
            let idx = self.tabs.active_index();
            self.tabs.close_tab(idx);
            self.open_error = None;
            return;
        }

        // ── HELP / F1 fallback — Validates: Requirement 18.1, 18.2 ————————
        if upper == "HELP" {
            let registry = HelpTopicRegistry::new(); // empty registry — no topics loaded yet
            let ctx = EditorContext {
                command_line_text: self.command_text.clone(),
                command_line_has_focus: true,
                prefix_area_text: None,
                prefix_area_has_focus: false,
                active_mode: EditorMode::Edit,
                help_panel_open: false,
                current_help_topic: None,
            };
            if let Err(msg) = ContextDetector::resolve_with_fallback(&ctx, &registry) {
                self.open_error = Some(msg);
            }
            return;
        }

        // ── KEYS — Validates: Requirement 20.1 ————————————————————————————
        if upper == "KEYS" {
            self.key_config_dialog.open = true;
            self.open_error = None;
            return;
        }

        // ── PFSHOW — Validates: Requirement 12.1–12.3 ——————————————————————
        if upper == "PFSHOW" {
            self.key_bar_visible = !self.key_bar_visible;
            self.open_error = None;
            return;
        }
        if upper == "PFSHOW ON" {
            self.key_bar_visible = true;
            self.open_error = None;
            return;
        }
        if upper == "PFSHOW OFF" {
            self.key_bar_visible = false;
            self.open_error = None;
            return;
        }

        // ── END — Validates: Requirement 17.1, 17.2 ———————————————————————
        if upper == "END" {
            let kind = self.tabs.active_tab().kind;
            if kind == TabKind::PrimaryOptionMenu {
                // Validates: Requirement 17.2 — END from POM exits
                let result = self
                    .dispatch
                    .execute_command("file.exit", CommandParams::new());
                if let CommandResult::Err(e) = result {
                    self.open_error = Some(e.to_string());
                }
            } else if kind == TabKind::FileExplorerPanel {
                // Validates: Requirement 19.10 — END from FileExplorerPanel returns to POM
                self.pending_return_to_pom = true;
            } else {
                // Validates: Requirement 17.1 — close current tab, go to previous
                let current = self.tabs.active_index();
                self.tabs.close_tab(current);
                if let Some(prev) = self.tab_history.pop() {
                    let clamped = prev.min(self.tabs.len().saturating_sub(1));
                    self.tabs.set_active(clamped);
                }
            }
            self.open_error = None;
            return;
        }

        // ── RETURN — Validates: Requirement 17.3, 17.4 ————————————————————
        if upper == "RETURN" {
            let is_pom = self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu;
            if is_pom {
                // Validates: Requirement 17.4 — RETURN from POM exits
                let result = self
                    .dispatch
                    .execute_command("file.exit", CommandParams::new());
                if let CommandResult::Err(e) = result {
                    self.open_error = Some(e.to_string());
                }
            } else {
                // Validates: Requirement 17.3 — navigate to POM tab
                if let Some(pom_idx) = self
                    .tabs
                    .tabs()
                    .iter()
                    .position(|t| t.kind == TabKind::PrimaryOptionMenu)
                {
                    self.tabs.set_active(pom_idx);
                } else {
                    self.tabs.insert_pom_tab(&self.runtime);
                }
            }
            self.open_error = None;
            return;
        }

        if upper == "0" || upper == "SETTINGS" || upper == "=0" {
            // Validates: Requirement 15.1 — option 0 / SETTINGS / =0 opens Settings panel
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::SettingsPanel, "[SETTINGS]");
            } else {
                self.tabs.open_settings_panel_tab(&self.runtime);
            }
            self.open_error = None;
            return;
        }

        if upper == "2" || upper == "=2" || upper == "=FILES" {
            // Validates: Requirement 19.1, 19.2, 19.4
            // =2 and =FILES transform current tab in-place; bare 2 on POM also transforms.
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::FileExplorerPanel, "[FILES]");
            } else {
                self.tabs.open_file_explorer_panel_tab(&self.runtime);
            }
            self.open_error = None;
            return;
        }

        if upper == "FILES" {
            // Validates: Requirement 19.3 — FILES (no =) always opens a NEW tab
            self.tabs.open_file_explorer_panel_tab(&self.runtime);
            self.open_error = None;
            return;
        }

        if upper == "1" || upper == "=1" || upper == "FILE CATALOGS" {
            // Validates: Requirement 1.1, 14.6 — option 1 opens the Files Panel
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::FilesPanel, "[FILES]");
            } else {
                self.tabs.open_files_panel_tab(&self.runtime);
            }
            self.open_error = None;
            return;
        }

        if upper == "3" || upper == "UTILITIES" {
            // Req 14.6 — option 3 opens Utilities (stub)
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::Untitled, "Utilities");
            }
            self.open_error = None;
            return;
        }

        if upper == "4" || upper == "COMPILERS" {
            // Req 14.6 — option 4 opens the Toolchain Panel
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.show_toolchain_panel = true;
                self.tabs
                    .transform_active_pom_tab(TabKind::Untitled, "Compilers");
            } else {
                self.show_toolchain_panel = true;
            }
            self.open_error = None;
            return;
        }

        if upper == "7" || upper == "DATABASES" {
            // Req 14.6 — option 7 opens the Databases panel (stub)
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::Untitled, "Databases");
            }
            self.open_error = None;
            return;
        }

        if upper == "8" || upper == "PLUGINS" {
            // Req 14.6 — option 8 opens the Plugins panel (stub)
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::Untitled, "Plugins");
            }
            self.open_error = None;
            return;
        }

        if upper == "RETRIEVE" {
            let cmd_text = self.command_text.clone();
            match self.retrieve_state.retrieve(&self.cmd_history, &cmd_text) {
                RetrieveResult::Recalled { command } => {
                    self.command_text = command;
                }
                RetrieveResult::ShowList { entries } => {
                    // Validates: Requirement 19.1 — show history list overlay
                    self.show_history_list = Some(entries);
                    self.command_text.clear();
                }
                RetrieveResult::HistoryEmpty | RetrieveResult::NoOlderHistory => {}
            }
            return;
        }

        // ── LOCATE / SORT / UP / DOWN / LEFT / RIGHT / TOP / BOTTOM ────────
        if upper.starts_with("LOCATE ") {
            let arg = cmd.trim()[7..].trim();
            let status = self.nav_manager.locate(arg, &mut self.tabs);
            self.open_error = if status.is_empty() {
                None
            } else {
                Some(status)
            };
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "TOP" {
            self.nav_manager.top(&mut self.tabs);
            self.open_error = None;
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "BOTTOM" {
            self.nav_manager.bottom(&mut self.tabs);
            self.open_error = None;
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "UP" || upper.starts_with("UP ") {
            let n = parse_optional_u64(cmd.trim().get(2..).unwrap_or("").trim());
            self.nav_manager.up(n, &mut self.tabs);
            self.open_error = None;
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "DOWN" || upper.starts_with("DOWN ") {
            let n = parse_optional_u64(cmd.trim().get(4..).unwrap_or("").trim());
            self.nav_manager.down(n, &mut self.tabs);
            self.open_error = None;
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "LEFT" || upper.starts_with("LEFT ") {
            let n = parse_optional_u64(cmd.trim().get(4..).unwrap_or("").trim());
            self.nav_manager.left(n, &mut self.tabs);
            self.open_error = None;
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "RIGHT" || upper.starts_with("RIGHT ") {
            let n = parse_optional_u64(cmd.trim().get(5..).unwrap_or("").trim());
            self.nav_manager.right(n, &mut self.tabs);
            self.open_error = None;
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "SORT" || upper.starts_with("SORT ") {
            let rest = cmd.trim().get(4..).unwrap_or("").trim();
            let args: Vec<&str> = rest.split_whitespace().collect();
            let status = self.nav_manager.sort(&args, &mut self.tabs, &self.runtime);
            self.open_error = if status.is_empty() {
                None
            } else {
                Some(status)
            };
            self.cmd_history.add(cmd);
            return;
        }

        // ── EXCLUDE / SHOW / RESET ────────────────────────────────────────────
        if upper == "EXCLUDE ALL" || upper == "X ALL" {
            let msg = self
                .exclude_manager
                .exclude_all(&mut self.tabs, &self.runtime);
            self.open_error = info_or_error(&msg);
            self.cmd_history.add(cmd);
            return;
        }

        if upper.starts_with("EXCLUDE ") || upper.starts_with("X ") {
            // EXCLUDE 'text' [ALL]  or  X 'text' [ALL]
            let rest = if upper.starts_with("EXCLUDE ") {
                cmd.trim()[8..].trim()
            } else {
                cmd.trim()[2..].trim()
            };
            let (text, all_flag) = strip_all_suffix(rest);
            let msg = if all_flag {
                self.exclude_manager
                    .exclude_text_all(text, &mut self.tabs, &self.runtime)
            } else {
                self.exclude_manager
                    .exclude_text(text, &mut self.tabs, &self.runtime)
            };
            self.open_error = info_or_error(&msg);
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "SHOW ALL" || upper == "INCLUDE ALL" {
            let msg = self.exclude_manager.show_all(&mut self.tabs, &self.runtime);
            self.open_error = info_or_error(&msg);
            self.cmd_history.add(cmd);
            return;
        }

        if upper.starts_with("SHOW ") || upper.starts_with("INCLUDE ") {
            let rest = if upper.starts_with("SHOW ") {
                cmd.trim()[5..].trim()
            } else {
                cmd.trim()[8..].trim()
            };
            let msg = self
                .exclude_manager
                .show_text(rest, &mut self.tabs, &self.runtime);
            self.open_error = info_or_error(&msg);
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "RESET" || upper == "RESET EXCLUDED" || upper == "RESET ALL" {
            use ff_exclude_show_filter::ResetVariant;
            let variant = if upper == "RESET ALL" {
                ResetVariant::All
            } else if upper == "RESET EXCLUDED" {
                ResetVariant::Excluded
            } else {
                ResetVariant::Default
            };
            let msg = self
                .exclude_manager
                .reset(variant, &mut self.tabs, &self.runtime);
            self.open_error = info_or_error(&msg);
            self.cmd_history.add(cmd);
            return;
        }

        // ── FIND / RFIND / CHANGE / RCHANGE ─────────────────────────────────
        if upper == "RFIND" {
            let status = self.find_manager.rfind(&mut self.tabs, &self.runtime);
            self.open_error = if status.contains("NOT FOUND") || status.contains("error") {
                Some(status)
            } else {
                self.open_error = None;
                None
            };
            self.cmd_history.add(cmd);
            return;
        }

        if upper == "RCHANGE" {
            let status = self.find_manager.rchange(&mut self.tabs, &self.runtime);
            self.open_error = if status.contains("NOT FOUND") || status.contains("error") {
                Some(status)
            } else {
                None
            };
            self.cmd_history.add(cmd);
            return;
        }

        if upper.starts_with("FIND ") {
            let term = cmd.trim()[5..].trim();
            let status = self.find_manager.find(term, &mut self.tabs, &self.runtime);
            self.open_error = if status.contains("NOT FOUND") || status.contains("error") {
                Some(status)
            } else {
                None
            };
            self.cmd_history.add(cmd);
            return;
        }

        if upper.starts_with("CHANGE ") {
            // Parse: CHANGE 'old' 'new'  (single-quoted or bare words)
            let rest = cmd.trim()[7..].trim();
            if let Some((old, new)) = parse_two_args(rest) {
                let status = self
                    .find_manager
                    .change(&old, &new, &mut self.tabs, &self.runtime);
                self.open_error = if status.contains("NOT FOUND") || status.contains("error") {
                    Some(status)
                } else {
                    None
                };
            } else {
                self.open_error =
                    Some("CHANGE requires two arguments: CHANGE 'old' 'new'".to_string());
            }
            self.cmd_history.add(cmd);
            return;
        }

        // ── CAPS — Validates: Requirement 16.1, 16.2 ────────────────────────
        if upper == "CAPS ON" {
            self.tabs.active_tab_mut().edit_profile.caps = ff_edit_operations::CapsMode::On;
            self.open_error = None;
            return;
        }
        if upper == "CAPS OFF" {
            self.tabs.active_tab_mut().edit_profile.caps = ff_edit_operations::CapsMode::Off;
            self.open_error = None;
            return;
        }
        if upper == "CAPS" {
            let tab = self.tabs.active_tab_mut();
            tab.edit_profile.caps = tab.edit_profile.caps.toggle();
            self.open_error = None;
            return;
        }

        // ── NULLS — Validates: Requirement 16.4 ──────────────────────────────
        if upper == "NULLS ON" {
            self.tabs.active_tab_mut().edit_profile.nulls = ff_edit_operations::NullsMode::On;
            self.open_error = None;
            return;
        }
        if upper == "NULLS OFF" {
            self.tabs.active_tab_mut().edit_profile.nulls = ff_edit_operations::NullsMode::Off;
            self.open_error = None;
            return;
        }

        // ── STATS — Validates: Requirement 16.7 ──────────────────────────────
        if upper == "STATS ON" {
            self.tabs.active_tab_mut().edit_profile.stats = ff_edit_operations::StatsMode::On;
            self.open_error = None;
            return;
        }
        if upper == "STATS OFF" {
            self.tabs.active_tab_mut().edit_profile.stats = ff_edit_operations::StatsMode::Off;
            self.open_error = None;
            return;
        }

        // ── LOCK — Validates: Requirement 16.8 ───────────────────────────────
        if upper == "LOCK ON" {
            self.tabs.active_tab_mut().edit_profile.lock = ff_edit_operations::ProfileLock::On;
            self.open_error = None;
            return;
        }
        if upper == "LOCK OFF" {
            self.tabs.active_tab_mut().edit_profile.lock = ff_edit_operations::ProfileLock::Off;
            self.open_error = None;
            return;
        }

        // ── PROFILE — Validates: Requirement 16.5, 16.6 ──────────────────────
        if upper == "PROFILE" {
            let summary = self.tabs.active_tab().edit_profile.display_summary();
            self.open_error = Some(summary);
            return;
        }
        if upper.starts_with("PROFILE ") {
            let rest = cmd.trim()[8..].trim();
            let mut parts = rest.splitn(2, ' ');
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("").trim();
            let result = self
                .tabs
                .active_tab_mut()
                .edit_profile
                .apply_keyword(key, val);
            match result {
                Ok(()) => self.open_error = None,
                Err(ProfileError::Locked) => {
                    self.open_error =
                        Some("Profile is locked -- use LOCK OFF to unlock".to_string());
                }
                Err(e) => self.open_error = Some(e.to_string()),
            }
            return;
        }

        // ── HILITE — Validates: Requirement 16.12 ────────────────────────────
        if upper == "HILITE" || upper.starts_with("HILITE ") {
            let keyword = cmd.trim().get(6..).unwrap_or("").trim();
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
            return;
        }

        // ── AUTONUM / NUM aliases — Validates: Requirement 16.10, 16.11 ──────
        if upper == "AUTONUM ON" || upper == "AUTONUM OFF" {
            let rest = &cmd.trim()[7..];
            let redirected = format!("NUMBER{rest}");
            self.handle_command(&redirected);
            return;
        }
        if upper == "NUM" || upper.starts_with("NUM ") {
            let rest = cmd.trim().get(3..).unwrap_or("").trim();
            let redirected = if rest.is_empty() {
                "NUMBER".to_string()
            } else {
                format!("NUMBER {rest}")
            };
            self.handle_command(&redirected);
            return;
        }

        // ── SUBMIT — Validates: Requirement 17.1 ─────────────────────────────
        if upper == "SUBMIT" {
            // Stub: JES subsystem dispatch deferred to Phase CC/CD.
            self.open_error = Some("SUBMIT: JES subsystem not yet available".to_string());
            return;
        }

        // ── CREATE — Validates: Requirement 17.2 ─────────────────────────────
        if upper.starts_with("CREATE ") {
            let dsn = cmd.trim()[7..].trim();
            if dsn.is_empty() {
                self.open_error = Some("CREATE requires a dataset name argument".to_string());
            } else {
                // Stub: dataset creation deferred to Phase BU/CB.
                self.open_error = Some(format!("CREATE {dsn}: dataset creation not yet available"));
            }
            return;
        }

        // ── REPLACE — Validates: Requirement 17.3 ────────────────────────────
        if upper.starts_with("REPLACE ") {
            let dsn = cmd.trim()[8..].trim();
            if dsn.is_empty() {
                self.open_error = Some("REPLACE requires a dataset name argument".to_string());
            } else {
                self.open_error = Some(format!("REPLACE {dsn}: dataset replace not yet available"));
            }
            return;
        }

        // ── BROWSE — Validates: Requirement 17.5 ─────────────────────────────
        if upper.starts_with("BROWSE ") {
            let dsn = cmd.trim()[7..].trim();
            if dsn.is_empty() {
                self.open_error = Some("BROWSE requires a dataset name argument".to_string());
            } else {
                // Open as read-only editor tab (full browse mode deferred).
                let mut p = CommandParams::new();
                p.insert("path", dsn);
                let result = self.dispatch.execute_command("file.open", p);
                if let CommandResult::Err(e) = result {
                    self.open_error = Some(e.to_string());
                } else {
                    self.open_error = None;
                }
            }
            return;
        }

        // ── VIEW — Validates: Requirement 17.6 ───────────────────────────────
        if upper.starts_with("VIEW ") {
            let dsn = cmd.trim()[5..].trim();
            if dsn.is_empty() {
                self.open_error = Some("VIEW requires a dataset name argument".to_string());
            } else {
                let mut p = CommandParams::new();
                p.insert("path", dsn);
                let result = self.dispatch.execute_command("file.open", p);
                if let CommandResult::Err(e) = result {
                    self.open_error = Some(e.to_string());
                } else {
                    self.open_error = None;
                }
            }
            return;
        }

        // ── COMPARE — Validates: Requirement 17.7 ────────────────────────────
        if upper.starts_with("COMPARE ") {
            let dsn = cmd.trim()[8..].trim();
            if dsn.is_empty() {
                self.open_error = Some("COMPARE requires a dataset name argument".to_string());
            } else {
                // Stub: compare view deferred to Phase BX/ff-compare.
                self.open_error = Some(format!("COMPARE {dsn}: compare view not yet available"));
            }
            return;
        }

        // ── Route through CommandEngine ──────────────────────────────────
        self.retrieve_state.reset();
        let status = self.cmd_engine.execute_command_line(cmd);
        match status.kind {
            StatusKind::Info => {
                self.open_error = None;
            }
            StatusKind::SyntaxError | StatusKind::StructureError | StatusKind::RuntimeError => {
                self.open_error = Some(status.text.clone());
            }
        }
        // Record in history (skip empty / error-only inputs)
        if !cmd.trim().is_empty() {
            self.cmd_history.add(cmd);
        }
    }
}
use super::WorkbenchShell;
