//! Shell-side wiring for the Alias Editor Context (command-configurator Req 5 +
//! Req 6 CR-CH-072).
//!
//! The panel render is pure (`alias_editor_panel::render`); all side effects
//! (load the active-locale EFFECTIVE table, validation, serialisation, override
//! file writes, live-resolver refresh) live here in the command layer, mirroring
//! the Menus Editor (`shell/menus_editor.rs`).
//!
//! CR-CH-072 (Req 6): the editor shows the ACTIVE LOCALE's EFFECTIVE alias table
//! (the shipped read-only seed `i18n/<locale>/aliases/<ENV>.toml` OVERLAID by the
//! User_Locale_Override_Store `<base>/<locale>/<ENV>.toml`) and WRITES edits to
//! the USER override only -- never to the shipped seed.

use super::WorkbenchShell;
use crate::alias_editor_panel::{AliasEditorAction, AliasRowEdit};
use crate::shell::user_aliases::{
    self, serialise_user_aliases, AliasEnvironment, UserAliasRowToml,
};

impl WorkbenchShell {
    /// Resolve the User_Locale_Override_Store BASE directory (localization
    /// Req 13.5): the shell's test override when set, else the user-data-dir
    /// default (`<User_Data_Dir>/FileForgeWorkbench/aliases/`). The per-locale,
    /// per-ENV override file is `<base>/<locale>/<ENV>.toml`.
    pub(super) fn user_alias_override_base(&self) -> Option<std::path::PathBuf> {
        if let Some(path) = &self.dir_overrides.user_aliases {
            return Some(path.clone());
        }
        user_aliases::user_alias_override_base()
    }

    /// The active locale for the alias store (the `ui.locale` config, default
    /// `en`).
    fn active_locale(&self) -> String {
        self.config_handle
            .get_string(ff_config::keys::ui::LOCALE)
            .unwrap_or_else(|_| "en".to_string())
    }

    /// Open the Alias Editor Context: load the ACTIVE-locale EFFECTIVE alias
    /// table into the working rows, then navigate the current tab to a
    /// `CommandConfigurator` `editor = "aliases"` descriptor IN PLACE (push onto
    /// the Navigation_Stack), so END/RETURN returns to whatever the editor was
    /// opened from. Mirrors `open_menus_editor` exactly.
    ///
    /// Validates: command-configurator Requirement 5.2, 5.9, 6.1
    pub(super) fn open_alias_editor(&mut self) {
        let rows = self.load_effective_alias_rows();
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

    /// Read the ACTIVE-locale EFFECTIVE alias table (the shipped seed overlaid by
    /// the user override) from the live `LocaleAliasStore` into editable rows
    /// (Req 6.1). This is why an `af` user SEES the shipped `af` surfaces. A
    /// locked/empty store yields an empty working copy (the editor starts blank).
    ///
    /// Validates: command-configurator Requirement 6.1
    fn load_effective_alias_rows(&self) -> Vec<AliasRowEdit> {
        let Ok(store) = self.i18n.alias_store.lock() else {
            return Vec::new();
        };
        store
            .effective_display_rows()
            .into_iter()
            .map(|(env, surface, canonical, template)| AliasRowEdit {
                act: String::new(),
                surface,
                canonical,
                environment: env,
                command_template: template.unwrap_or_default(),
            })
            .collect()
    }

    /// Apply an [`AliasEditorAction`] produced by the Alias Editor render.
    ///
    /// Per-field edits (surface / root / environment / template / act) are
    /// applied DIRECTLY by the render on the mutable working rows, so they do not
    /// appear as actions here -- only the line-command batch and the primary
    /// command do (Req 5.14, 5.16, 5.22).
    ///
    /// Validates: command-configurator Requirement 5.14, 5.16, 5.22
    pub(super) fn apply_alias_editor_action(&mut self, action: AliasEditorAction) {
        use AliasEditorAction as A;
        match action {
            A::None => {}
            A::ApplyLineCommands => self.apply_pending_line_commands(),
            A::CommandLine(text) => self.apply_alias_command_line(&text),
        }
    }

    /// Validate -> group by environment -> serialise -> write -> refresh the
    /// User_Locale_Override_Store for the active locale (Req 6.3). On ANY
    /// validation or write failure the on-disk override is left UNCHANGED and the
    /// error is surfaced inline (Req 5.4). On success the live `LocaleAliasStore`
    /// is refreshed IN PLACE so the alias resolves immediately without a restart
    /// (Req 6.4). The shipped seed is NEVER written.
    ///
    /// Validates: command-configurator Requirement 5.4, 6.3, 6.4
    ///
    /// CR-CH-071 (Req 20a.1/20a.2): returns `Result<(), String>` so the FFCMD
    /// editable-Context seam's END (save-then-leave) can branch on ok/err WITHOUT
    /// reading panel state. On failure the error is BOTH surfaced inline AND
    /// returned as the `Err`, and the on-disk override is left UNCHANGED (Req 5.4).
    pub(super) fn save_user_aliases(&mut self) -> Result<(), String> {
        let rows = self.alias_editor_panel.rows.clone();
        let toml_rows = match validate_user_alias_rows(&rows) {
            Ok(rows) => rows,
            Err(e) => {
                self.alias_editor_panel.error = Some(e.clone());
                return Err(e);
            }
        };
        let Some(base) = self.user_alias_override_base() else {
            let msg = "could not resolve the user alias override path".to_string();
            self.alias_editor_panel.error = Some(msg.clone());
            return Err(msg);
        };
        let locale = self.active_locale();
        if let Err(e) = write_override_rows(&base, &locale, &toml_rows) {
            self.alias_editor_panel.error = Some(e.clone());
            return Err(e);
        }
        self.alias_editor_panel.error = None;
        // Refresh the live resolver IN PLACE so the new alias resolves at once
        // (Req 6.4): re-seed + re-apply BOTH the shipped seed overlay AND the
        // per-locale user override in one re-seed, so neither layer is dropped.
        self.refresh_alias_store_in_place();
        Ok(())
    }

    /// The Alias Editor's implementation of the CR-CH-071 editable-Context SEAM
    /// PERSIST operation (Req 20a.6): validate + write the working copy + refresh
    /// the live store. Reuses [`WorkbenchShell::save_user_aliases`] verbatim so the
    /// Alias Editor SAVE is one behaviour, whatever verb reaches it.
    ///
    /// Validates: command-environments Requirement 20a.2, 20a.6; command-configurator Req 6.5
    pub(super) fn alias_editor_persist(&mut self) -> Result<(), String> {
        self.save_user_aliases()
    }

    /// The Alias Editor's SEAM DISCARD operation (Req 20a.3): drop the unsaved
    /// working edits (clear the working rows and the inline error) WITHOUT
    /// touching the on-disk override store. Used by CANCEL / RETURN (which then
    /// leave) and by REFRESH (before the reload).
    ///
    /// Validates: command-environments Requirement 20a.3, 20a.6; command-configurator Req 6.5
    pub(super) fn alias_editor_discard(&mut self) {
        self.alias_editor_panel.rows.clear();
        self.alias_editor_panel.error = None;
    }

    /// The Alias Editor's SEAM RELOAD operation (Req 20a.4): re-read the
    /// EFFECTIVE active-locale table (seed + override) into the working rows,
    /// dropping any unsaved edits. `load_rows` also clears the inline error,
    /// command line, and filter, so the Context returns to its freshly-opened
    /// state.
    ///
    /// Validates: command-environments Requirement 20a.4, 20a.6; command-configurator Req 6.5
    pub(super) fn alias_editor_reload(&mut self) {
        let rows = self.load_effective_alias_rows();
        self.alias_editor_panel.load_rows(rows);
    }

    /// Rebuild the shell's live `LocaleAliasStore` IN PLACE (Req 6.4): one re-seed
    /// re-applies the Identity_Base, the shipped per-locale `i18n/<locale>/aliases/`
    /// seed overlay, AND the User_Locale_Override_Store overlay (at the shell's
    /// resolved base), so a save never drops the shipped seed and the saved user
    /// override goes live without a restart. Every holder of the `Arc` (the
    /// shell) observes the swap.
    ///
    /// Validates: command-configurator Requirement 6.4
    pub(super) fn refresh_alias_store_in_place(&self) {
        let locale = self.active_locale();
        let Ok(mut guard) = self.i18n.alias_store.lock() else {
            return;
        };
        let roots = guard.ffcmd_roots();
        let root_refs: Vec<&str> = roots.iter().map(String::as_str).collect();
        // Re-seed + shipped seed overlay, then the per-locale USER override from
        // the shell's resolved base (so a test override is honoured).
        let mut store = super::alias_reader::LocaleAliasStore::seed(&root_refs);
        if let Some(root) = super::i18n_bootstrap::resolve_i18n_root() {
            store.apply_overlay_from_dir(&root.join(&locale).join("aliases"));
        }
        if let Some(base) = self.user_alias_override_base() {
            store.apply_user_overlay_from_base(&base, &locale);
        }
        *guard = store;
    }
}

/// Write the validated override rows to the per-locale override files, grouped by
/// environment: one `<base>/<locale>/<ENV>.toml` per environment, each holding
/// that environment's rows. An environment with NO rows has its override file
/// REMOVED (so deleting every override row clears the override), leaving only the
/// shipped seed. The shipped seed is never touched. On any write failure the
/// error is returned and the caller leaves state unchanged.
///
/// Validates: command-configurator Requirement 6.3
fn write_override_rows(
    base: &std::path::Path,
    locale: &str,
    rows: &[UserAliasRowToml],
) -> Result<(), String> {
    for env in AliasEnvironment::ALL {
        let env_rows: Vec<UserAliasRowToml> = rows
            .iter()
            .filter(|r| r.environment == env)
            .cloned()
            .collect();
        let path = user_aliases::user_alias_override_file(base, locale, env.env_name());
        if env_rows.is_empty() {
            // No override rows for this ENV -> remove any stale override file so
            // the shipped seed is the sole source. An absent file is not an error.
            if path.exists() {
                if let Err(e) = std::fs::remove_file(&path) {
                    return Err(format!(
                        "could not remove stale override {}: {e}",
                        path.display()
                    ));
                }
            }
            continue;
        }
        let text =
            serialise_user_aliases(&env_rows).map_err(|e| format!("serialise failed: {e}"))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("could not create override dir: {e}"))?;
        }
        std::fs::write(&path, text)
            .map_err(|e| format!("could not write user override {}: {e}", path.display()))?;
    }
    Ok(())
}

/// Validate the Alias Editor working rows and convert them to serialisable
/// [`UserAliasRowToml`] rows (Req 5.4, 6.6). Rejects: (a) an empty surface, (b) an
/// empty canonical, (c) a surface mapped to two different roots WITHIN the SAME
/// target environment, (d) a surface equal to a reserved built-in root
/// (`builtin_ffcmd_roots()` -- the no-shadow invariant), (e) a root that is not a
/// valid canonical root of its declared environment (`contains_root`). A user
/// override surface equal to a shipped-seed surface is a CUSTOMISATION (the
/// override wins, Req 6.6), not a collision. On the first failure returns
/// `Err(message)` so the caller leaves the override unchanged.
///
/// Validates: command-configurator Requirement 5.4, 6.6
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
        // valid canonical root of the declared environment.
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
            ..Default::default()
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

    // Validates: command-configurator Requirement 6.6 -- a surface mapped to two
    // different roots WITHIN the same environment is rejected.
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

    // Validates: command-configurator Requirement 6.6 -- the SAME surface in
    // DIFFERENT environments is allowed (no collision across environments).
    #[test]
    fn validate_allows_same_surface_in_different_environments() {
        // FIND is a valid FFEDIT (in-buffer) root; CONFIG is a valid FFCMD root.
        let rows = vec![
            row("DUP", "FIND", AliasEnvironment::Ffedit, ""),
            row("DUP", "CONFIG", AliasEnvironment::Ffcmd, ""),
        ];
        let out = validate_user_alias_rows(&rows).expect("different envs do not collide");
        assert_eq!(out.len(), 2);
    }

    // Validates: command-configurator Requirement 6.6 -- a surface equal to a
    // reserved built-in root (here CONFIG) is rejected (no-shadow invariant).
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
    // environment (FIND is an FFEDIT root, not an FFCMD root), reusing
    // EnvironmentRegistry::contains_root.
    #[test]
    fn validate_rejects_root_not_valid_in_declared_environment() {
        let rows = vec![row("FF", "FIND", AliasEnvironment::Ffcmd, "")];
        assert!(
            validate_user_alias_rows(&rows).is_err(),
            "a root that is not valid in the declared environment must be rejected"
        );
        let rows = vec![row("CF", "CONFIG", AliasEnvironment::Ffedit, "")];
        assert!(
            validate_user_alias_rows(&rows).is_err(),
            "an FFCMD root declared under FFEDIT must be rejected"
        );
    }

    // Validates: command-environments Requirement 20.8 -- a row whose root IS a
    // valid root of its declared environment passes: EDIT in FFCMD (file-open)
    // and FIND in FFEDIT (in-buffer).
    #[test]
    fn validate_accepts_root_valid_in_declared_environment() {
        let rows = vec![
            row("ECOB", "EDIT", AliasEnvironment::Ffcmd, ""),
            row("SOEK", "FIND", AliasEnvironment::Ffedit, ""),
        ];
        let out = validate_user_alias_rows(&rows).expect("valid roots pass");
        assert_eq!(out.len(), 2);
    }

    // Validates: command-environments Requirement 20a.12 -- a user alias row
    // whose SURFACE equals SAVE / CANCEL / REFRESH is REJECTED at save, because
    // those three are FFCMD canonical roots (CR-CH-071 Req 20a.5) in
    // builtin_ffcmd_roots(), which the no-shadow rule reads.
    #[test]
    fn validate_rejects_save_cancel_refresh_shadow() {
        for surface in ["SAVE", "CANCEL", "REFRESH"] {
            let rows = vec![row(surface, "EDIT", AliasEnvironment::Ffcmd, "")];
            assert!(
                validate_user_alias_rows(&rows).is_err(),
                "a user alias named {surface} must be rejected (Req 20a.12)"
            );
        }
    }

    // Validates: command-configurator Requirement 6.3 -- a valid set serialises
    // to rows with surface/canonical UPPERCASED and the template preserved; the
    // L18N (localisation-key) field is gone (CR-CH-072 Req 6.2).
    #[test]
    fn validate_uppercases_and_preserves_template() {
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
