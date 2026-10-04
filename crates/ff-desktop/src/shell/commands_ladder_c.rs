//! # Shell Command Dispatch -- detach / split / dataset-stub ladder
//!
//! An extracted contiguous segment of the `handle_command` dispatch ladder
//! (TASK 2.2, pure code movement -- no behaviour change). `try_commands_c`
//! (DETACH..WORKSPACE) returns `true` when it handled the command. Branch
//! order is identical to the original ladder.

use ff_command::{CommandParams, CommandResult};

use super::helpers::*;
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Extracted ladder segment of `handle_command` (TASK 2.2, pure code
    /// movement -- no behaviour change). Returns `true` when a branch
    /// handled the command (the caller then returns), `false` to fall
    /// through to the next segment. Branch order is preserved exactly.
    pub(super) fn try_commands_c(&mut self, cmd: &str, upper: &str) -> bool {
        // ── DETACH / SPLIT DETACH -- Validates: menu-and-statusbar Req 18.14 ──
        // CR-CH-040 (B046 Slice 1): DETACH detaches the current Workspace into a
        // Detached_Workspace (OS window). `SPLIT DETACH` is a DEPRECATED ALIAS of
        // DETACH. The former editor-tab `SPLIT`-splits-screen behaviour (an inert,
        // never-rendered `SplitScreenState`) is RETIRED; the bare verb `SPLIT` is
        // reserved for a future real in-window split (Slice 2) and is no longer
        // handled here (falls through to the normal command-resolution chain).
        if upper == "DETACH" || upper == "SPLIT DETACH" {
            let idx = self.tabs.active_index();
            // CR-CH-035: the 16-window limit counts recorded FloatingTabs, the
            // single source of truth shared with the "Move to Other View" context
            // item (was `is_floating` count here -- an inconsistency, B045).
            if self.detach_split.floating_tabs.len() >= 16 {
                self.open_error =
                    Some("Maximum number of detached Workspaces (16) reached.".to_string());
            } else {
                self.detach_split.detach_pending = Some(idx);
                self.open_error = None;
            }
            return true;
        }

        // === SPLIT / UNSPLIT / FOCUS (CR-NR-092, B046 Slice 2b) ===
        // In-window split of the Workspace area. Every user action is a command
        // (architecture Principle 2): the menu/keys route here too. `SPLIT
        // DETACH` was already handled above, so a bare `SPLIT` (or `SPLIT RIGHT`
        // / `SPLIT DOWN`) reaches this branch. CR-NR-093 (Slice 2c.1): SPLIT now
        // NESTS -- splitting an already-split focused group to arbitrary depth
        // (the Slice 2b "one split only" rejection is removed).
        //
        // Validates: layout-and-docking Requirement 14.1, 14.2, 14.3, 14.5
        if upper == "SPLIT" || upper == "SPLIT RIGHT" || upper == "SPLIT VERTICAL DIVIDER" {
            // Side-by-side (left/right). Bare SPLIT defaults to horizontal.
            self.tabs
                .split_focused(ff_layout::SplitDirection::Horizontal, &self.runtime);
            self.open_error = None;
            return true;
        }
        if upper == "SPLIT DOWN" || upper == "SPLIT HORIZONTAL DIVIDER" {
            // Stacked (top/bottom).
            self.tabs
                .split_focused(ff_layout::SplitDirection::Vertical, &self.runtime);
            self.open_error = None;
            return true;
        }
        if upper == "UNSPLIT" {
            // Req 14.3: collapse the split around the focused leaf. No-op /
            // status when not split.
            if self.tabs.is_split() {
                self.tabs.unsplit();
                self.open_error = None;
            } else {
                self.open_error = Some("UNSPLIT: the Workspace is not split.".to_string());
            }
            return true;
        }
        if upper == "FOCUS" || upper == "FOCUS OTHER" {
            // Req 14.5: move focus to the NEXT Tab_Group leaf (cycles). No-op /
            // status when not split. No forced key binding this slice.
            if self.tabs.is_split() {
                self.tabs.focus_other_group();
                self.open_error = None;
            } else {
                self.open_error = Some("FOCUS: the Workspace is not split.".to_string());
            }
            return true;
        }

        if upper == "SWAP" || upper.starts_with("SWAP ") {
            // SWAP is the tab/workspace switcher (multi-tab-editor Req 18). Parse
            // the argument to decide. (CR-CH-040: the former split-focus-swap
            // branch for a bare SWAP is retired with the inert split model.)
            let arg = cmd.trim().get(4..).unwrap_or("").trim().to_string();
            let arg_upper = arg.to_uppercase();

            if arg.is_empty() {
                // Bare SWAP: toggle to the previously active workspace (Req 18.7,
                // CR-CH-031), falling back to the tab picker when there is no
                // distinct previous tab (Req 18.10).
                if let Some(prev) = self.tabs.previous_active_index() {
                    self.tabs.set_active(prev);
                    self.open_error = None;
                } else {
                    self.show_swap_list = Some(());
                    self.open_error = None;
                }
            } else if arg_upper == "LIST" {
                // SWAP LIST: open the tab picker (Req 18.3).
                self.show_swap_list = Some(());
                self.open_error = None;
            } else if let Ok(n) = arg.parse::<usize>() {
                // SWAP n: activate the n-th tab, 1-based (Req 18.1, 18.2).
                let count = self.tabs.len();
                if n >= 1 && n <= count {
                    self.tabs.set_active(n - 1);
                    self.open_error = None;
                } else {
                    self.open_error = Some(format!(
                        "SWAP: tab number {n} out of range (valid: 1 to {count})"
                    ));
                }
            } else {
                // Non-numeric, non-LIST argument (Req 18.2).
                self.open_error = Some(format!(
                    "SWAP: invalid argument '{arg}'. Usage: SWAP <n>, SWAP LIST"
                ));
            }
            return true;
        }

        // ── AUTONUM / NUM aliases — Validates: Requirement 16.10, 16.11 ──────
        if upper == "AUTONUM ON" || upper == "AUTONUM OFF" {
            let rest = &cmd.trim()[7..];
            let redirected = format!("NUMBER{rest}");
            self.handle_command(&redirected);
            return true;
        }
        if let Some(rest) = verb_arg(cmd, "NUM") {
            let redirected = if rest.is_empty() {
                "NUMBER".to_string()
            } else {
                format!("NUMBER {rest}")
            };
            self.handle_command(&redirected);
            return true;
        }

        // ── SUBMIT — Validates: Requirement 17.1 ─────────────────────────────
        if upper == "SUBMIT" {
            // Stub: JES subsystem dispatch deferred to Phase CC/CD.
            self.open_error = Some("SUBMIT: JES subsystem not yet available".to_string());
            return true;
        }

        // ── TIME — Validates: Requirement 20.4 ───────────────────────────────
        if upper == "TIME" {
            let now = chrono::Local::now();
            let msg = format!(
                "Date: {}  Time: {}  Day: {}",
                now.format("%Y-%m-%d"),
                now.format("%H:%M:%S"),
                now.format("%j")
            );
            self.open_error = Some(msg);
            return true;
        }

        // ── STATUS — Validates: Requirement 20.5, 20.6 ───────────────────────
        if let Some(arg) = verb_arg(cmd, "STATUS") {
            let jobname = if arg.is_empty() {
                None
            } else {
                Some(arg.to_string())
            };
            let msg = match jobname {
                Some(ref j) => format!("STATUS: routing to JES panel (filter: {})", j),
                None => "STATUS: routing to JES job status panel".to_string(),
            };
            self.open_error = Some(msg);
            return true;
        }

        // ── CREATE — Validates: Requirement 17.2 ─────────────────────────────
        if let Some(dsn) = verb_arg(cmd, "CREATE").filter(|a| !a.is_empty()) {
            // Stub: dataset creation deferred to Phase BU/CB.
            self.open_error = Some(format!("CREATE {dsn}: dataset creation not yet available"));
            return true;
        }

        // ── REPLACE — Validates: Requirement 17.3 ────────────────────────────
        if let Some(dsn) = verb_arg(cmd, "REPLACE").filter(|a| !a.is_empty()) {
            self.open_error = Some(format!("REPLACE {dsn}: dataset replace not yet available"));
            return true;
        }

        // ── BROWSE — Validates: Requirement 17.5 ─────────────────────────────
        // DEFERRED from the Step 3 Function family (B080): BROWSE opens via
        // `dispatch.execute_command("file.open", { path })`, carrying the path
        // param. The Function arm's `handle_command(command_id)` carries no
        // params, so a Function target would drop the path (Req 8.4 regression).
        // Stays on the ladder (same analysis as EDIT in commands_ladder_a.rs).
        if let Some(dsn) = verb_arg(cmd, "BROWSE").filter(|a| !a.is_empty()) {
            // Open as read-only editor tab (full browse mode deferred).
            let mut p = CommandParams::new();
            p.insert("path", dsn);
            let result = self.dispatch.execute_command("file.open", p);
            if let CommandResult::Err(e) = result {
                self.open_error = Some(e.to_string());
            } else {
                self.open_error = None;
            }
            return true;
        }

        // ── VIEW — Validates: Requirement 17.6 ───────────────────────────────
        // DEFERRED from the Step 3 Function family (B080): same analysis as
        // BROWSE/EDIT -- opens via `dispatch.execute_command("file.open",
        // { path })` and the Function arm carries no params, so it stays on the
        // ladder to preserve the observable file-open behaviour (Req 8.4).
        if let Some(dsn) = verb_arg(cmd, "VIEW").filter(|a| !a.is_empty()) {
            let mut p = CommandParams::new();
            p.insert("path", dsn);
            let result = self.dispatch.execute_command("file.open", p);
            if let CommandResult::Err(e) = result {
                self.open_error = Some(e.to_string());
            } else {
                self.open_error = None;
            }
            return true;
        }

        // ── COMPARE — Validates: Requirement 17.7 ────────────────────────────
        if let Some(dsn) = verb_arg(cmd, "COMPARE").filter(|a| !a.is_empty()) {
            // Stub: compare view deferred to Phase BX/ff-compare.
            self.open_error = Some(format!("COMPARE {dsn}: compare view not yet available"));
            return true;
        }

        // ── WORKSPACE commands -- Validates: workspace-model Requirement 2.1-2.4 ──
        if upper.starts_with("WORKSPACE ") || upper == "WORKSPACE" {
            let rest = cmd.trim().get(9..).unwrap_or("").trim();
            let rest_upper = rest.to_uppercase();
            if rest_upper.starts_with("OPEN ") {
                let path = rest.get(5..).unwrap_or("").trim();
                if path.is_empty() {
                    self.open_error = Some("WORKSPACE OPEN requires a path".to_string());
                } else {
                    self.open_workspace(std::path::Path::new(path));
                }
            } else if rest_upper == "SAVE" {
                self.save_workspace_to(None);
            } else if rest_upper.starts_with("SAVE AS ") {
                let path = rest.get(8..).unwrap_or("").trim();
                if path.is_empty() {
                    self.open_error = Some("WORKSPACE SAVE AS requires a path".to_string());
                } else {
                    self.save_workspace_to(Some(std::path::Path::new(path)));
                }
            } else if rest_upper == "CLOSE" {
                self.close_workspace();
            } else if rest_upper.starts_with("ADD ROOT ") {
                let path = rest.get(9..).unwrap_or("").trim();
                if path.is_empty() {
                    self.open_error = Some("WORKSPACE ADD ROOT requires a path".to_string());
                } else if let Some(ws) = self.active_workspace.as_mut() {
                    let p = std::path::PathBuf::from(path);
                    if !ws.roots.contains(&p) {
                        ws.roots.push(p.clone());
                        ws.is_modified = true;
                    }
                    let cat = crate::catalog_registry::VirtualCatalog {
                        name: p
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| p.to_string_lossy().into_owned()),
                        catalog_type: crate::catalog_registry::CatalogType::Native,
                        path: p.to_string_lossy().into_owned(),
                        description: Some("Workspace root".to_string()),
                        auto_mount: true,
                        default_hlq: None,
                        mount_point: None,
                        read_only: false,
                    };
                    let _ = self.files_panel.registry.register(cat);
                    self.open_error = None;
                } else {
                    self.open_error =
                        Some("No active workspace -- use WORKSPACE OPEN first".to_string());
                }
            } else if rest_upper.starts_with("REMOVE ROOT ") {
                let path = rest.get(12..).unwrap_or("").trim();
                if path.is_empty() {
                    self.open_error = Some("WORKSPACE REMOVE ROOT requires a path".to_string());
                } else if let Some(ws) = self.active_workspace.as_mut() {
                    let p = std::path::PathBuf::from(path);
                    ws.roots.retain(|r| r != &p);
                    ws.is_modified = true;
                    let name = p
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| p.to_string_lossy().into_owned());
                    let _ = self.files_panel.registry.remove(&name);
                    self.open_error = None;
                } else {
                    self.open_error = Some("No active workspace".to_string());
                }
            } else {
                self.open_error = Some(
                    "WORKSPACE: unknown subcommand. Use OPEN/SAVE/SAVE AS/CLOSE/ADD ROOT/REMOVE ROOT"
                        .to_string(),
                );
            }
            return true;
        }
        false
    }
}
