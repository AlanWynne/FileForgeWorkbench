//! # Shell Command Dispatch -- POM fastpath and chained navigation
//!
//! The POM menu lazy-load guarantee, the chained Option_Key fastpath
//! (`=0.K` / `3.1`), and the config-driven POM Option_Key resolver. Split out
//! of `commands.rs` / `commands_menu.rs` (TASK 2.2, pure code movement, no
//! behaviour change). All methods are on `WorkbenchShell` and keep their exact
//! signatures and visibility.

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Ensure the active Home tab carries a loaded `MenuWorkspaceState` backed
    /// by `menus/pom.toml`, loading it lazily on first render. After CR-CH-042 the
    /// Home Context's Title_Line and Tab_Header are DERIVED like any other Menu
    /// Workspace -- the raw Menu_Title on the Title_Line and the uppercased
    /// Menu_Name (`POM`) on the tab -- from this loaded state; only the load-time
    /// "always have a POM" guarantee is POM-specific. Idempotent: does nothing if
    /// the state is already present or the active tab is not the Home Context.
    ///
    /// Validates: menu-workspace Requirement 2.1c, 2.1d, 18.2, 18.4
    pub(super) fn ensure_pom_menu_loaded(&mut self) {
        if !self.tabs.active_tab().is_home {
            return;
        }
        if self.tabs.active_tab().kind.menu_workspace().is_some() {
            return;
        }
        let pom_path = self.menus_dir().join("pom.toml");
        let limits = crate::menu_workspace::loader::option_limits_from_config(&self.config_handle);
        let mut state =
            crate::menu_workspace::MenuWorkspaceState::load_with_limits(&pom_path, limits);
        // Code-only fallback (menu-workspace Req 12.4/12.5, CR-CH-021): if no
        // valid user pom.toml produced a menu, use the compiled Recovery_Baseline
        // so the Home Context always has its options. A file that EXISTED but
        // failed to PARSE gets a non-blocking notice; a merely absent file is
        // silent. Keeps the POM deterministic in tests with no menus dir.
        if state.menu.is_none() {
            let parse_error = state
                .load_error
                .as_deref()
                .filter(|e| e.starts_with("Menu file error"))
                .map(str::to_string);
            state.menu = Some(crate::menu_workspace::defaults::recovery_pom_menu());
            state.load_error = None;
            if let Some(err) = parse_error {
                self.notify_menu_fallback("pom.toml", &err);
            }
        }
        let idx = self.tabs.active_index();
        if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
            tab.kind = crate::tab_state::TabKind::MenuWorkspace(Some(state));
        }
    }

    /// Handle a chained fastpath navigation path (menu-workspace Requirement 5).
    ///
    /// A chained path walks a menu Option_Key chain across one or more levels,
    /// e.g. `=0.K` (POM option 0 -> Settings, then option K -> KEYS), `3.1`, or
    /// `=0;E.T` (mixed separators). Segments are separated by `.` (collapse /
    /// STOP) or `;` (push / PUSH); this method splits on BOTH.
    ///
    /// Semantics:
    /// - A leading `=` is the Navigation_Origin (Req 5.7): the shell pops to the
    ///   POM before resolving the first segment, so `=<k>...` always resolves
    ///   `<k>` against the POM regardless of the active Workspace.
    /// - A bare dotted path (no `=`) resolves its first segment against the
    ///   CURRENT menu (the legacy `3.1` behaviour, Requirement 19.4). To avoid
    ///   hijacking ordinary dotted input (dataset names, `abc.def`), a bare path
    ///   is only treated as a fastpath when its first segment is a single digit.
    /// - Each segment is dispatched as an Option_Key through `handle_command`,
    ///   which opens the target sub-menu and activates the option (the same code
    ///   path `MENU <name> <key>` and the single-segment fastpath use).
    /// - Depth is capped at 4 segments (Req 5.2); deeper paths report an error.
    ///
    /// Returns `Some(true)` when the input was a chained path and was handled
    /// (the caller must return), `Some(false)`/`None` when the input is NOT a
    /// chained path and the caller should fall through to the rest of the chain.
    ///
    /// Validates: menu-workspace Requirement 5.1, 5.2, 5.4, 5.7; Requirement 19.4
    pub(super) fn try_chained_fastpath(&mut self, upper: &str) -> Option<bool> {
        let has_sep = upper.contains('.') || upper.contains(';');
        let is_origin = upper.starts_with('=');
        // Not a chained path unless it has a separator (or is a bare `=<key>`,
        // which the single-segment fastpath below already handles -- so require
        // a separator here). A leading `.`/`;` is not a path.
        if !has_sep || upper.starts_with('.') || upper.starts_with(';') {
            return None;
        }

        // Strip the Navigation_Origin marker; split into segments on either
        // separator, preserving order. splitn-style cap at 5 so >4 is an error.
        let body = upper.strip_prefix('=').unwrap_or(upper);
        let segments: Vec<&str> = body
            .split(['.', ';'])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if segments.is_empty() {
            return None;
        }
        if segments.len() > 4 {
            self.open_error = Some("Chained path exceeds maximum depth of 4 segments.".to_string());
            return Some(true);
        }

        // A bare dotted path (no `=`) is only a fastpath when the first segment
        // is a single digit -- otherwise ordinary dotted text (`abc.def`, DSNs)
        // would be hijacked. An `=`-origin path is always a fastpath.
        let first = segments[0];
        if !is_origin && !(first.len() == 1 && first.chars().all(|c| c.is_ascii_digit())) {
            return None;
        }

        // CR-CH-052: the Navigation_Origin pop is REMOVED here. The single
        // front-door `=` step (`dispatch_command_string`) already reinitialised
        // the active tab's Navigation_Stack to the POM before any `=`-prefixed
        // input reaches this method, so a second pop here would be redundant (and
        // the three nav callers now route through the front door, so no bare `=`
        // segment arrives). The `strip_prefix('=')` on `body` above is retained
        // as a harmless no-op for defensive direct callers; `is_origin` is still
        // used above only to classify a bare `=<key>` path as a fastpath.

        // Dispatch each segment as an Option_Key. CR-CH-052 (B080 reroute):
        // every segment re-enters through the single front door
        // `dispatch_command_string` (not `handle_command`), so a segment that is
        // itself an in-scope verb reaches `resolve_target` and the CustomWorkspace
        // dispatch arm (the same shell open method as the deleted ladder arm);
        // the sub-menu / option opens in place (D8). The front door's `=` step is
        // never triggered here because segments are already `=`-stripped.
        for segment in segments {
            self.dispatch_command_string(segment);
        }
        Some(true)
    }

    /// Resolve a POM fastpath key to its Option_Command using the loaded
    /// `pom.toml` option list. Accepts a bare Option_Key (e.g. `1`, `S`) or the
    /// `=<key>` fastpath form (e.g. `=1`). Returns the option's `command` when a
    /// matching, enabled option exists in the POM menu; otherwise `None`.
    ///
    /// This is how `=<key>` and bare keys become config-driven (menu-workspace
    /// Req 2.1e, 2.1i): the key selects a row, and that row's command drives the
    /// behaviour -- no digit is coupled to a destination in code.
    ///
    /// The lookup uses the POM tab's menu regardless of which Workspace is
    /// active, so `=1` from any context resolves against the POM (Navigation
    /// Origin, menu-workspace Req 5.7).
    ///
    /// Validates: menu-workspace Requirement 2.1e, 2.1i
    pub(super) fn resolve_pom_option_key(&mut self, upper: &str) -> Option<String> {
        // Normalise: strip a single leading '=' for the fastpath form. CR-CH-052:
        // the front-door `=` step (`dispatch_command_string`) already strips `=`
        // before this is reached on the typed/front-door path, so this strip is
        // now a defensive no-op there; it is kept only so a direct `handle_command`
        // caller that passes a stale `=key` still resolves.
        let key = upper.strip_prefix('=').unwrap_or(upper);
        // Only single short keys are POM option keys (1-4 chars, no spaces).
        if key.is_empty() || key.len() > 4 || key.contains(' ') {
            return None;
        }
        // Ensure the active POM tab's menu is loaded so a fastpath resolves even
        // before the POM has rendered.
        if self.tabs.active_tab().is_home {
            self.ensure_pom_menu_loaded();
        }
        // Prefer a loaded POM tab's menu (respects user edits / hot-reload);
        // otherwise consult the on-disk pom.toml, then the built-in default, so
        // the fastpath is config-driven from any Workspace (Navigation Origin =
        // POM, menu-workspace Req 5.7). The default fallback also keeps the
        // resolver deterministic when no POM tab exists yet.
        let menu = self
            .tabs
            .tabs()
            .iter()
            .find(|t| t.is_home)
            .and_then(|t| t.kind.menu_workspace())
            .and_then(|mw| mw.menu.clone())
            .or_else(|| {
                let pom_path = self.menus_dir().join("pom.toml");
                crate::menu_workspace::loader::load_menu_file(&pom_path).ok()
            })
            .or_else(|| {
                crate::menu_workspace::loader::parse_menu_str(
                    crate::menu_workspace::defaults::DEFAULT_POM_TOML,
                )
                .ok()
            })?;
        let option = menu
            .options
            .iter()
            .find(|o| o.key.eq_ignore_ascii_case(key) && o.enabled)?;
        Some(option.command.clone())
    }
}
