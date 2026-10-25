//! Shell-side wiring for the Alias Editor Context (command-configurator Req 5,
//! CR-NR-113).
//!
//! The panel render is pure (`alias_editor_panel::render`); all side effects
//! (load, validation, serialisation, file writes, live-resolver refresh) live
//! here in the command layer, mirroring the Menus Editor
//! (`shell/menus_editor.rs`).

use super::WorkbenchShell;
use crate::alias_editor_panel::{AliasEditorAction, AliasRowEdit};
use crate::shell::user_aliases::{
    self, serialise_user_aliases, AliasEnvironment, UserAliasRowToml,
};

impl WorkbenchShell {
    /// Resolve the User_Alias_Store file path (Req 5.1): the shell's test
    /// override when set, else the user-data-dir default
    /// (`<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`).
    pub(super) fn user_aliases_path(&self) -> Option<std::path::PathBuf> {
        if let Some(path) = &self.dir_overrides.user_aliases {
            return Some(path.clone());
        }
        user_aliases::user_aliases_file()
    }

    /// Open the Alias Editor Context: load the User_Alias_Store into the working
    /// rows, then navigate the current tab to a `CommandConfigurator`
    /// `editor = "aliases"` descriptor IN PLACE (push onto the Navigation_Stack),
    /// so END/RETURN returns to whatever the editor was opened from. Mirrors
    /// `open_menus_editor` exactly.
    ///
    /// Validates: command-configurator Requirement 5.2, 5.9
    pub(super) fn open_alias_editor(&mut self) {
        let rows = self.load_user_alias_rows();
        self.alias_editor_panel.load_rows(rows);

        self.navigate_to(
            ff_session::session_state::WorkspaceDescriptor::CustomWorkspace {
                workspace_kind: ff_session::session_state::WorkspaceKind::CommandConfigurator,
                params: {
                    let mut p = ff_session::session_state::DescriptorParams::new();
                    p.insert(
                        "editor".to_string(),
                        ff_session::session_state::DescriptorValue::from("aliases"),
                    );
                    p
                },
            },
            true,
        );
    }

    /// Read the User_Alias_Store file into editable rows. An absent/unreadable or
    /// malformed file yields an empty working copy (the editor starts blank and
    /// the file is created on first save, Req 5.1).
    fn load_user_alias_rows(&self) -> Vec<AliasRowEdit> {
        let Some(path) = self.user_aliases_path() else {
            return Vec::new();
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Vec::new();
        };
        match user_aliases::parse_user_alias_toml(&text) {
            Ok(rows) => rows
                .into_iter()
                .map(|r| AliasRowEdit {
                    surface: r.surface,
                    canonical: r.canonical,
                    environment: r.environment,
                    command_template: r.command_template,
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// Apply an [`AliasEditorAction`] produced by the Alias Editor render.
    ///
    /// Validates: command-configurator Requirement 5.3, 5.5
    pub(super) fn apply_alias_editor_action(&mut self, action: AliasEditorAction) {
        use AliasEditorAction as A;
        match action {
            A::None => {}
            // Note: per-field edits (surface/root/environment/template) are
            // applied DIRECTLY by the render on the mutable working rows, so they
            // do not appear as actions here -- only structural changes do.
            A::AddRow => {
                self.alias_editor_panel.rows.push(AliasRowEdit::default());
            }
            A::DeleteRow(index) => {
                if index < self.alias_editor_panel.rows.len() {
                    self.alias_editor_panel.rows.remove(index);
                }
            }
            A::Save => self.save_user_aliases(),
        }
    }

    /// Validate -> serialise -> write -> refresh the User_Alias_Store (Req 5.4,
    /// 5.6). On ANY validation or write failure the on-disk file is left
    /// UNCHANGED and the error is surfaced inline (Req 5.4). On success the live
    /// `LocaleAliasStore` is refreshed IN PLACE so the alias resolves immediately
    /// without a restart (Req 5.6).
    ///
    /// Validates: command-configurator Requirement 5.4, 5.5, 5.6
    fn save_user_aliases(&mut self) {
        let rows = self.alias_editor_panel.rows.clone();
        let toml_rows = match validate_user_alias_rows(&rows) {
            Ok(rows) => rows,
            Err(e) => {
                self.alias_editor_panel.error = Some(e);
                return;
            }
        };
        let text = match serialise_user_aliases(&toml_rows) {
            Ok(text) => text,
            Err(e) => {
                self.alias_editor_panel.error = Some(format!("serialise failed: {e}"));
                return;
            }
        };
        let Some(path) = self.user_aliases_path() else {
            self.alias_editor_panel.error =
                Some("could not resolve the user alias store path".to_string());
            return;
        };
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                self.alias_editor_panel.error = Some(format!("could not create aliases dir: {e}"));
                return;
            }
        }
        if let Err(e) = std::fs::write(&path, text) {
            self.alias_editor_panel.error = Some(format!("could not write user aliases: {e}"));
            return;
        }
        self.alias_editor_panel.error = None;
        // Refresh the live resolver IN PLACE so the new alias resolves at once
        // (Req 5.6): re-seed + re-apply BOTH the per-locale overlay AND the user
        // store in one re-seed, so neither layer is dropped.
        self.refresh_alias_store_in_place();
    }

    /// Rebuild the shell's live `LocaleAliasStore` IN PLACE (Req 5.6): one
    /// re-seed re-applies the Identity_Base, the per-locale `i18n/<locale>/aliases/`
    /// overlay, AND the User_Alias_Store overlay (at the shell's resolved path),
    /// so a save never drops the per-locale aliases and the saved user alias goes
    /// live without a restart. Every holder of the `Arc` (the shell) observes the
    /// swap.
    ///
    /// Validates: command-configurator Requirement 5.6
    pub(super) fn refresh_alias_store_in_place(&self) {
        let locale = self
            .config_handle
            .get_string(ff_config::keys::ui::LOCALE)
            .unwrap_or_else(|_| "en".to_string());
        let Ok(mut guard) = self.i18n.alias_store.lock() else {
            return;
        };
        let roots = guard.ffcmd_roots();
        let root_refs: Vec<&str> = roots.iter().map(String::as_str).collect();
        // Re-seed + per-locale overlay (the existing third-overlay-aware path uses
        // the env-var/default user path; here we re-apply the USER overlay from the
        // shell's resolved path so a test override is honoured).
        let mut store = super::alias_reader::LocaleAliasStore::seed(&root_refs);
        if let Some(root) = super::i18n_bootstrap::resolve_i18n_root() {
            store.apply_overlay_from_dir(&root.join(&locale).join("aliases"));
        }
        if let Some(path) = self.user_aliases_path() {
            store.apply_user_overlay_from_file(&path);
        }
        *guard = store;
    }
}

/// Validate the Alias Editor working rows and convert them to serialisable
/// [`UserAliasRowToml`] rows (Req 5.4). Rejects: (a) an empty surface, (b) an
/// empty canonical, (c) a surface mapped to two different roots WITHIN the SAME
/// target environment, (d) a surface equal to a reserved built-in root
/// (`builtin_ffcmd_roots()` -- the NEW CR-NR-113 pre-pass invariant). On the
/// first failure returns `Err(message)` so the caller leaves the file unchanged.
///
/// Validates: command-configurator Requirement 5.4
fn validate_user_alias_rows(rows: &[AliasRowEdit]) -> Result<Vec<UserAliasRowToml>, String> {
    let reserved = super::alias_reader::builtin_ffcmd_roots();
    // Track (environment, surface) -> canonical for the within-same-environment
    // collision check (c).
    let mut seen: Vec<(AliasEnvironment, String, String)> = Vec::new();
    let mut out: Vec<UserAliasRowToml> = Vec::with_capacity(rows.len());
    for row in rows {
        let surface = row.surface.trim().to_uppercase();
        let canonical = row.canonical.trim().to_uppercase();
        if surface.is_empty() {
            return Err("alias surface must not be empty".to_string());
        }
        if canonical.is_empty() {
            return Err(format!("alias '{surface}': root command must not be empty"));
        }
        // (d) NO-SHADOW: a user surface may not equal a reserved built-in root.
        if reserved.iter().any(|r| r.eq_ignore_ascii_case(&surface)) {
            return Err(format!(
                "alias '{surface}' shadows a reserved built-in root command"
            ));
        }
        // (e) ROOT-VALIDITY (CR-CH-067, Req 20.8): the declared root must be a
        // valid canonical root of the declared environment. This is the SAVE-side
        // surfacing of the same `contains_root` check the LOAD path skips-with-WARN
        // on (Req 20.7), so Gate 2's Alias Editor can reject an invalid
        // (environment, root) pair at save. Gate 1 adds only the hook.
        if !super::environment_registry::EnvironmentRegistry::contains_root(
            row.environment.env_name(),
            &canonical,
        ) {
            return Err(format!(
                "alias '{surface}': root '{canonical}' is not a valid root command of \
                 environment '{}'",
                row.environment.env_name()
            ));
        }
        // (c) within-SAME-environment surface -> two different roots.
        if let Some((_, _, existing)) = seen
            .iter()
            .find(|(env, s, _)| *env == row.environment && s.eq_ignore_ascii_case(&surface))
        {
            if !existing.eq_ignore_ascii_case(&canonical) {
                return Err(format!(
                    "alias '{surface}' maps to two different roots in the same environment \
                     ('{existing}' and '{canonical}')"
                ));
            }
            // Same surface + same root in the same env: a harmless duplicate;
            // skip the second so the file stays clean.
            continue;
        }
        seen.push((row.environment, surface.clone(), canonical.clone()));
        out.push(UserAliasRowToml {
            surface,
            canonical,
            environment: row.environment,
            command_template: row.command_template.trim().to_string(),
            localisation_key: String::new(),
        });
    }
    Ok(out)
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn row(surface: &str, canonical: &str, env: AliasEnvironment, template: &str) -> AliasRowEdit {
        AliasRowEdit {
            surface: surface.to_string(),
            canonical: canonical.to_string(),
            environment: env,
            command_template: template.to_string(),
        }
    }

    // Validates: command-configurator Requirement 5.4 (a) -- an empty surface is
    // rejected.
    #[test]
    fn validate_rejects_empty_surface() {
        let rows = vec![row("", "EDIT", AliasEnvironment::Ffcmd, "")];
        assert!(validate_user_alias_rows(&rows).is_err());
    }

    // Validates: command-configurator Requirement 5.4 (b) -- an empty root is
    // rejected.
    #[test]
    fn validate_rejects_empty_canonical() {
        let rows = vec![row("ECOB", "", AliasEnvironment::Ffcmd, "")];
        assert!(validate_user_alias_rows(&rows).is_err());
    }

    // Validates: command-configurator Requirement 5.4 (c) -- a surface mapped to
    // two different roots WITHIN the same environment is rejected.
    #[test]
    fn validate_rejects_within_same_environment_two_roots() {
        // EDIT and VIEW are both valid FFCMD (file-open) roots, so the rejection
        // here is purely the same-surface-two-roots collision, not root-validity.
        let rows = vec![
            row("DUP", "EDIT", AliasEnvironment::Ffcmd, ""),
            row("DUP", "VIEW", AliasEnvironment::Ffcmd, ""),
        ];
        assert!(validate_user_alias_rows(&rows).is_err());
    }

    // Validates: command-configurator Requirement 5.4 -- the SAME surface in
    // DIFFERENT environments is allowed (no collision across environments).
    #[test]
    fn validate_allows_same_surface_in_different_environments() {
        // FIND is a valid FFEDIT (in-buffer) root; CONFIG is a valid FFCMD root.
        // The same surface in two different environments does not collide.
        let rows = vec![
            row("DUP", "FIND", AliasEnvironment::Ffedit, ""),
            row("DUP", "CONFIG", AliasEnvironment::Ffcmd, ""),
        ];
        let out = validate_user_alias_rows(&rows).expect("different envs do not collide");
        assert_eq!(out.len(), 2);
    }

    // Validates: command-configurator Requirement 5.4 (d) -- a surface equal to a
    // reserved built-in root (here CONFIG, a builtin_ffcmd_roots() member) is
    // rejected (the NEW CR-NR-113 no-shadow invariant).
    #[test]
    fn validate_rejects_builtin_root_shadow() {
        let rows = vec![row("CONFIG", "EDIT", AliasEnvironment::Ffcmd, "")];
        assert!(
            validate_user_alias_rows(&rows).is_err(),
            "a surface equal to a built-in root must be rejected"
        );
    }

    // Validates: command-environments Requirement 20.8 -- the save-side hook
    // rejects a row whose root is NOT a valid canonical root of its declared
    // environment (here FIND is an FFEDIT root, not an FFCMD root), reusing the
    // EnvironmentRegistry::contains_root primitive.
    #[test]
    fn validate_rejects_root_not_valid_in_declared_environment() {
        // FIND is an FFEDIT in-buffer root; it is NOT an FFCMD root.
        let rows = vec![row("FF", "FIND", AliasEnvironment::Ffcmd, "")];
        assert!(
            validate_user_alias_rows(&rows).is_err(),
            "a root that is not valid in the declared environment must be rejected"
        );
        // And the mirror: CONFIG is an FFCMD root, not an FFEDIT root.
        let rows = vec![row("CF", "CONFIG", AliasEnvironment::Ffedit, "")];
        assert!(
            validate_user_alias_rows(&rows).is_err(),
            "an FFCMD root declared under FFEDIT must be rejected"
        );
    }

    // Validates: command-environments Requirement 20.8 -- a row whose root IS a
    // valid root of its declared environment passes the save-side hook: EDIT in
    // FFCMD (file-open family) and FIND in FFEDIT (in-buffer).
    #[test]
    fn validate_accepts_root_valid_in_declared_environment() {
        let rows = vec![
            row("ECOB", "EDIT", AliasEnvironment::Ffcmd, ""),
            row("SOEK", "FIND", AliasEnvironment::Ffedit, ""),
        ];
        let out = validate_user_alias_rows(&rows).expect("valid roots pass");
        assert_eq!(out.len(), 2);
    }

    // Validates: command-configurator Requirement 5.4 -- a valid set serialises
    // to rows with surface/canonical UPPERCASED and the template preserved.
    #[test]
    fn validate_uppercases_and_preserves_template() {
        // EDIT is a valid FFCMD (file-open) root (CR-CH-067), so ECOB -> EDIT is
        // an FFCMD alias.
        let rows = vec![row(
            "ecob",
            "edit",
            AliasEnvironment::Ffcmd,
            "FFWB.COBOL.MASTER(&1)",
        )];
        let out = validate_user_alias_rows(&rows).expect("valid");
        assert_eq!(out[0].surface, "ECOB");
        assert_eq!(out[0].canonical, "EDIT");
        assert_eq!(out[0].command_template, "FFWB.COBOL.MASTER(&1)");
    }
}
